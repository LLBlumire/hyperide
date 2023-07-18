use proc_macro::TokenStream;
use proc_macro2::{Literal, TokenTree};
use quote::{quote, ToTokens};
use rstml::{
    atoms::OpenTag,
    node::{
        KeyedAttribute, Node, NodeAttribute, NodeComment, NodeDoctype, NodeElement, NodeFragment,
        NodeName,
    },
    Parser, ParserConfig,
};

#[derive(Default)]
struct NodeWalker {
    index: usize,
    static_format: String,
    values: Vec<proc_macro2::TokenStream>,
}
impl NodeWalker {
    fn push(&mut self, s: &str) {
        self.static_format.push_str(s)
    }
    fn eval(&mut self, s: proc_macro2::TokenStream) -> usize {
        let value_index = self.index;
        self.static_format.push_str(&format!("{{{}}}", value_index));
        self.index += 1;
        self.values.push(s);
        value_index
    }
    fn cached(&mut self, value_index: usize) {
        self.static_format.push_str(&format!("{{{}}}", value_index));
    }
}

fn walk_nodes(nodes: &[Node]) -> (String, Vec<proc_macro2::TokenStream>) {
    let mut walker = NodeWalker::default();
    r_walk_nodes(nodes, &mut walker);
    (walker.static_format, walker.values)
}

fn r_walk_nodes(nodes: &[Node], walker: &mut NodeWalker) {
    for node in nodes {
        match node {
            Node::Comment(NodeComment { value, .. }) => {
                walker.push("<!-- ");
                walker.push(&value.to_token_stream().to_string());
                walker.push(" -->")
            }
            Node::Doctype(NodeDoctype { value, .. }) => {
                walker.push("<!DOCTYPE ");
                walker.push(&value.to_token_stream_string());
                walker.push(">");
            }
            Node::Fragment(NodeFragment { children, .. }) => {
                r_walk_nodes(&children, walker);
            }
            Node::Element(NodeElement {
                open_tag:
                    open_tag @ OpenTag {
                        name, attributes, ..
                    },
                children,
                close_tag,
            }) => {
                walker.push("<");
                let mut auto_name = None;
                match name {
                    NodeName::Path(path) => {
                        walker.push(&path.to_token_stream().to_string());
                    }
                    NodeName::Punctuated(punct) => {
                        walker.push(&punct.to_token_stream().to_string());
                    }
                    NodeName::Block(block) => {
                        auto_name = Some(walker.eval(block.to_token_stream()));
                    }
                }
                for attribute in attributes {
                    walker.push(" ");
                    match attribute {
                        NodeAttribute::Block(block) => {
                            walker.eval(block.to_token_stream());
                        }
                        NodeAttribute::Attribute(attribute @ KeyedAttribute { key, .. }) => {
                            match key {
                                NodeName::Path(path) => {
                                    walker.push(&path.to_token_stream().to_string());
                                }
                                NodeName::Punctuated(punct) => {
                                    // TODO: this is where to hook int on: / hx:
                                    walker.push(&punct.to_token_stream().to_string());
                                }
                                NodeName::Block(block) => {
                                    walker.eval(block.to_token_stream());
                                }
                            }
                            if let Some(value) = attribute.value() {
                                walker.push("=\"");
                                walker.eval(value.to_token_stream());
                                walker.push("\"");
                            }
                        }
                    }
                }
                if open_tag.is_self_closed() {
                    walker.push(" /");
                }
                walker.push(">");

                let Some(close_tag) = close_tag else {
                    continue
                };

                r_walk_nodes(&children, walker);

                walker.push("</");

                if close_tag.name.is_wildcard() {
                    if let Some(auto_name) = auto_name {
                        walker.cached(auto_name);
                        walker.push(">");
                        continue;
                    }
                }

                match &close_tag.name {
                    NodeName::Path(path) => {
                        walker.push(&path.to_token_stream().to_string());
                    }
                    NodeName::Punctuated(punct) => {
                        walker.push(&punct.to_token_stream().to_string());
                    }
                    NodeName::Block(block) => {
                        walker.eval(block.to_token_stream());
                    }
                }
                walker.push(">");
            }
            Node::Block(block) => {
                walker.eval(block.to_token_stream());
            }
            Node::Text(text) => {
                walker.push(&text.value_string());
            }
            Node::RawText(raw_text) => {
                walker.eval(TokenTree::from(Literal::string(&raw_text.to_string_best())).into());
            }
        }
    }
}

/// Converts a HTML like syntax into a string.
///
/// ```rust
/// use hyperide_macro::hyperide;
/// fn returns_tag() -> char {
///     'p'
/// }
/// fn my_component(a: &str, b: &str) -> String {
///     hyperide! {
///         <p><strong>{a}{": "}</strong>{b}</p>
///     }
/// }
/// let my_str = hyperide! {
///     <!DOCTYPE html>
///     <html lang="en">
///     <head>
///         <meta charset="utf-8" />
///     </head>
///     <body>
///         <h1>{"Hello, world!"}</h1>
///         <{returns_tag()}>This is in a closed paragraph.</_>
///         <!-- "                      wildcard close tag ^^^^" -->
///         {my_component("Foo", "bar")}
///     </body>
///     </html>
/// };
/// ```
#[proc_macro_error::proc_macro_error]
#[proc_macro]
pub fn hyperide(tokens: TokenStream) -> TokenStream {
    let config = ParserConfig::new()
        .recover_block(true)
        // https://developer.mozilla.org/en-US/docs/Glossary/Empty_element
        .always_self_closed_elements(
            [
                "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta",
                "param", "source", "track", "wbr",
            ]
            .into_iter()
            .collect(),
        )
        .raw_text_elements(["script", "style"].into_iter().collect())
        .element_close_wildcard(|_, close_tag| close_tag.name.is_wildcard());

    let parser = Parser::new(config);
    let (nodes, errors) = parser.parse_recoverable(tokens).split_vec();

    let (html_string, values) = walk_nodes(&nodes);
    let errors = errors.into_iter().map(|e| e.emit_as_expr_tokens());
    let out = quote! {
        {
            #(#errors;)*
            format!(#html_string, #(#values),*)
        }
    };

    out.into()
}
