use proc_macro::TokenStream;
use proc_macro2::{Ident, Literal, Span, TokenTree};
use proc_macro_crate::{crate_name, FoundCrate};
use proc_macro_error::abort;
use quote::{quote, ToTokens};
use rstml::{
    atoms::OpenTag,
    node::{
        KeyedAttribute, Node, NodeAttribute, NodeBlock, NodeComment, NodeDoctype, NodeElement,
        NodeFragment, NodeName,
    },
    Parser, ParserConfig,
};
use syn::{punctuated::Pair, Block};

#[derive(Default)]
struct NodeWalker {
    index: usize,
    static_format: String,
    values: Vec<proc_macro2::TokenStream>,
    in_disabled_raw: bool,
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
                r_walk_nodes(children, walker);
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
                                    let path = path.to_token_stream().to_string();
                                    if path == "_hr_no_raw" {
                                        walker.in_disabled_raw = true;
                                    }
                                    walker.push(&path);
                                }
                                NodeName::Punctuated(punct) => {
                                    // TODO: this is where to hook int on: / hx:
                                    for term in punct.pairs() {
                                        match term {
                                            Pair::Punctuated(item, _) => {
                                                walker.push(&item.to_token_stream().to_string());
                                                walker.push("-");
                                            }
                                            Pair::End(item) => {
                                                walker.push(&item.to_token_stream().to_string());
                                            }
                                        }
                                    }
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

                r_walk_nodes(children, walker);

                walker.in_disabled_raw = false;

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
                if !walker.in_disabled_raw {
                    walker
                        .eval(TokenTree::from(Literal::string(&raw_text.to_string_best())).into());
                } else {
                    let x = syn::parse2::<Block>(raw_text.to_token_stream()).unwrap();
                    r_walk_nodes(&[Node::Block(NodeBlock::ValidBlock(x))], walker)
                }
            }
        }
    }
}

/// Converts a HTML like syntax into a string.
///
/// ```rust
/// use hyperide::hyperide;
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
///         <!-- "wildcard close tag ⬆️" -->
///         {my_component("Foo", "bar")}
///     </body>
///     </html>
/// };
/// ```
///
/// Will generate:
///
/// ```html
/// <!DOCTYPE html>
/// <html lang="en">
///   <head>
///     <meta charset="utf-8" />
///   </head>
///   <body>
///     <h1>Hello, world!</h1>
///     <p>This is in a closed paragraph.</p>
///     <!-- "wildcard close tag ⬆️" -->
///     <p><strong>Foo: </strong>bar</p>
///   </body>
/// </html>
/// ```
#[proc_macro_error::proc_macro_error]
#[proc_macro]
pub fn hyperide(tokens: TokenStream) -> TokenStream {
    let Ok(hyperide) = crate_name("hyperide") else {
        abort!(proc_macro2::TokenStream::from(tokens), "hyperide crate must be available")
    };
    let hyperide = match hyperide {
        FoundCrate::Itself => quote! { ::hyperide },
        FoundCrate::Name(name) => {
            let ident = Ident::new(&name, Span::call_site());
            quote! { ::#ident }
        }
    };

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
            format!(
                #html_string,
                #(
                    #hyperide ::IntoView::into_view( #[allow(unused_braces)] { #values } )
                ),*
            )
        }
    };

    out.into()
}
