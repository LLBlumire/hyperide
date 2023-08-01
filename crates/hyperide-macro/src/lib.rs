use std::fmt::Display;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use proc_macro_crate::{crate_name, FoundCrate};
use proc_macro_error::abort;
use quote::{quote, quote_spanned, ToTokens};
use rstml::{
    atoms::{CloseTag, OpenTag},
    node::{
        KeyedAttribute, KeyedAttributeValue, Node, NodeAttribute, NodeComment, NodeElement,
        NodeFragment, NodeName, NodeText,
    },
    Parser, ParserConfig,
};
use syn::{
    punctuated::{Pair, Punctuated},
    spanned::Spanned,
    ExprPath, LitStr,
};
use uuid::Uuid;

struct HyperideGenerator {
    bindings: TokenStream2,
    idents: Vec<Ident>,
    hyperide: TokenStream2,
    in_disabled_raw: bool,
}
impl HyperideGenerator {
    fn new(hyperide: TokenStream2) -> HyperideGenerator {
        HyperideGenerator {
            bindings: quote_spanned! {Span::call_site()=>},
            idents: Vec::new(),
            hyperide,
            in_disabled_raw: false,
        }
    }

    fn push_raw_hypertext(&mut self, to: TokenStream2) -> Ident {
        let bind = make_ident(Span::call_site());
        let hyperide = &self.hyperide;
        self.bindings.extend(quote_spanned! {Span::call_site()=>
            #[allow(unused_braces)]
            let #bind: #hyperide::HyperText<'_> = #to;
        });
        self.idents.push(bind.clone());
        bind
    }

    fn push_as_hypertext(&mut self, to: TokenStream2) -> Ident {
        let hyperide = &self.hyperide;
        self.push_raw_hypertext(quote_spanned! {to.span()=>
            #hyperide::IntoHyperText::into_hyper_text(#to)
        })
    }

    fn push_lit(&mut self, lit: &LitStr) -> Ident {
        self.push_as_hypertext(lit.to_token_stream())
    }

    fn push_str(&mut self, str: &str, span: Span) -> Ident {
        self.push_lit(&LitStr::new(str, span))
    }

    fn push_ref(&mut self, ident: &Ident) -> Ident {
        self.push_raw_hypertext(quote_spanned! {Span::call_site()=>
            std::ops::Deref::deref(&#ident).into();
        })
    }

    fn push_nodes(&mut self, nodes: &[Node]) {
        for node in nodes {
            match node {
                Node::Comment(NodeComment { value, .. }) => {
                    self.push_lit(value);
                }
                Node::Doctype(doctype) => {
                    self.push_str("<!DOCTYPE html>", doctype.span());
                }
                Node::Fragment(NodeFragment { children, .. }) => {
                    self.push_nodes(&children);
                }
                Node::Block(block) => {
                    self.push_as_hypertext(block.to_token_stream());
                }
                Node::Text(NodeText { value }) => {
                    self.push_lit(value);
                }
                Node::RawText(raw_text) => {
                    if !self.in_disabled_raw {
                        let best_string = raw_text.to_string_best();
                        self.push_str(&best_string, raw_text.span());
                    } else {
                        self.push_as_hypertext(raw_text.to_token_stream());
                    }
                }
                Node::Element(element) => self.push_element(element),
            }
        }
    }

    fn push_element(&mut self, element: &NodeElement) {
        let NodeElement {
            open_tag,
            children,
            close_tag,
        } = element;

        let open_ident = self.push_open_tag(open_tag);
        self.push_nodes(&children);
        self.in_disabled_raw = false;
        self.push_close_tag(close_tag.as_ref(), &open_ident);
    }

    fn push_open_tag(&mut self, open_tag: &OpenTag) -> Ident {
        let OpenTag {
            token_lt,
            name,
            generics,
            attributes,
            end_tag,
        } = open_tag;

        if generics.lt_token.is_some() {
            abort!(generics.lt_token.span(), "Tag must not have generics");
        }

        self.push_str("<", token_lt.span());

        let open_value_ident = match name {
            NodeName::Path(path) => {
                let name = get_path_ident(path);
                self.push_str(&name.to_string(), name.span())
            }
            NodeName::Punctuated(punct) => {
                // custom-elements
                let name = get_punct_hypertext(punct);
                self.push_str(&name, punct.span())
            }
            NodeName::Block(block) => self.push_as_hypertext(block.to_token_stream()),
        };

        for attribute in attributes {
            self.push_str(" ", attribute.span());
            match attribute {
                NodeAttribute::Block(block) => {
                    self.push_as_hypertext(block.to_token_stream());
                }
                NodeAttribute::Attribute(keyed) => {
                    let KeyedAttribute {
                        key,
                        possible_value,
                    } = keyed;

                    match key {
                        NodeName::Path(path) => {
                            let name = get_path_ident(path);
                            if name == "_hr_no_raw" {
                                self.in_disabled_raw = true;
                            }
                            self.push_str(&name.to_string(), key.span());
                        }
                        NodeName::Punctuated(punct) => {
                            // data-attributes
                            let name = get_punct_hypertext(punct);
                            self.push_str(&name, punct.span());
                        }
                        NodeName::Block(block) => {
                            self.push_as_hypertext(block.to_token_stream());
                        }
                    };
                    // SAFETY always pushed to in previous match, pop is done for IntoAttrText
                    let key_ident = unsafe { self.idents.pop().unwrap_unchecked() };

                    match possible_value {
                        KeyedAttributeValue::Binding(binding) => {
                            abort!(
                                binding.span(),
                                "I have no idea what this is open an issue if you see it"
                            )
                        }
                        KeyedAttributeValue::Value(expr) => {
                            let hyperide = &self.hyperide;
                            let value = &expr.value;
                            self.push_as_hypertext(quote_spanned! {expr.span()=>
                                #hyperide::IntoAttrText::into_attr_text(#value, #key_ident)
                            });
                        }
                        KeyedAttributeValue::None => {
                            self.push_ref(&key_ident);
                        }
                    }
                }
            }
        }

        self.push_str(">", end_tag.span());

        open_value_ident
    }

    fn push_close_tag(&mut self, close_tag: Option<&CloseTag>, open_ident: &Ident) {
        if let Some(close_tag) = close_tag {
            let CloseTag {
                start_tag,
                name,
                generics,
                token_gt,
            } = close_tag;

            if generics.lt_token.is_some() {
                abort!(generics.lt_token.span(), "Tag must not have generics");
            }

            self.push_str("</", start_tag.span());

            if name.is_wildcard() {
                self.push_ref(open_ident);
            } else {
                match name {
                    NodeName::Path(path) => {
                        let name = get_path_ident(path);
                        self.push_str(&name.to_string(), name.span());
                    }
                    NodeName::Punctuated(punct) => {
                        let name = get_punct_hypertext(punct);
                        self.push_str(&name.to_string(), punct.span());
                    }
                    NodeName::Block(block) => {
                        self.push_as_hypertext(block.to_token_stream());
                    }
                }
            }

            self.push_str(">", token_gt.span());
        }
    }
}

fn make_ident(span: Span) -> Ident {
    Ident::new(
        &format!("__hyperide_internal_{}", Uuid::new_v4().simple()),
        span,
    )
}

fn get_path_ident(path: &ExprPath) -> &Ident {
    if !path.attrs.is_empty() {
        abort!(path.span(), "Expected ident, found attribute");
    }
    match path.path.get_ident() {
        Some(ident) => ident,
        None => abort!(path.span(), "Expected ident, found path"),
    }
}

fn get_punct_hypertext<T>(punct: &Punctuated<impl Display, T>) -> String {
    let mut name = String::new();
    for term in punct.pairs() {
        match term {
            Pair::Punctuated(term, _punct) => {
                name.push_str(&format!("{term}"));
                name.push('-'); // other puncts are invalid in tags and attrs
            }
            Pair::End(term) => {
                name.push_str(&format!("{term}"));
            }
        }
    }
    name
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

    let mut walker = HyperideGenerator::new(hyperide);
    walker.push_nodes(&nodes);

    let idents = walker.idents;
    let bindings = walker.bindings;

    let errors = errors.into_iter().map(|e| e.emit_as_expr_tokens());
    let alloc_size = make_ident(Span::call_site());
    let string_out = make_ident(Span::call_site());
    let out = quote! {{
        #(#errors;)*
        #bindings
        let #alloc_size = #(
            std::ops::Deref::deref(&#idents).len()
        )+*;
        let mut #string_out = String::with_capacity(#alloc_size);
        #(
            #string_out.push_str(std::ops::Deref::deref(&#idents));
        )*
        #string_out
    }};

    out.into()
}
