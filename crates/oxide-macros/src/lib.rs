use proc_macro::TokenStream;
use proc_macro2::{Delimiter, TokenStream as TokenStream2, TokenTree};
use quote::{format_ident, quote};

/// JSX-like view macro for declaring reactive UI.
///
/// ```ignore
/// view! {
///     <div>
///         <p>"Count: " {count}</p>
///         <button on:click={move |_| count += 1}>
///             "Increment"
///         </button>
///     </div>
/// }
/// ```
#[proc_macro]
pub fn view(input: TokenStream) -> TokenStream {
    let input2: TokenStream2 = input.into();
    let mut parser = ViewParser::new(input2);
    let node = parser.parse_node().expect("view! macro requires at least one element");
    let mut counter = 0;
    let output = generate(&node, &mut counter);
    output.into()
}

// ---------------------------------------------------------------------------
// AST
// ---------------------------------------------------------------------------

enum ViewNode {
    Element {
        tag: String,
        attrs: Vec<Attr>,
        children: Vec<ViewNode>,
    },
    Text(String),
    DynExpr(TokenStream2),
}

enum Attr {
    Static {
        name: String,
        value: String,
    },
    Event {
        event: String,
        handler: TokenStream2,
    },
    Dynamic {
        name: String,
        value: TokenStream2,
    },
}

// ---------------------------------------------------------------------------
// Parser — recursive descent over proc_macro2 token trees
// ---------------------------------------------------------------------------

struct ViewParser {
    tokens: Vec<TokenTree>,
    cursor: usize,
}

impl ViewParser {
    fn new(input: TokenStream2) -> Self {
        Self {
            tokens: input.into_iter().collect(),
            cursor: 0,
        }
    }

    fn peek(&self) -> Option<&TokenTree> {
        self.tokens.get(self.cursor)
    }

    fn peek_at(&self, offset: usize) -> Option<&TokenTree> {
        self.tokens.get(self.cursor + offset)
    }

    fn advance(&mut self) -> Option<TokenTree> {
        if self.cursor < self.tokens.len() {
            let tt = self.tokens[self.cursor].clone();
            self.cursor += 1;
            Some(tt)
        } else {
            None
        }
    }

    fn is_punct(&self, ch: char) -> bool {
        matches!(self.peek(), Some(TokenTree::Punct(p)) if p.as_char() == ch)
    }

    fn expect_punct(&mut self, ch: char) {
        if !self.is_punct(ch) {
            panic!(
                "oxide view!: expected '{}', found {:?}",
                ch,
                self.peek()
            );
        }
        self.advance();
    }

    fn expect_ident(&mut self) -> String {
        match self.advance() {
            Some(TokenTree::Ident(i)) => i.to_string(),
            other => panic!("oxide view!: expected identifier, found {:?}", other),
        }
    }

    // --- Parsing -----------------------------------------------------------

    fn parse_node(&mut self) -> Option<ViewNode> {
        match self.peek()? {
            // `<` — either opening element or closing tag
            TokenTree::Punct(p) if p.as_char() == '<' => {
                // Closing tag? `< /`
                if matches!(self.peek_at(1), Some(TokenTree::Punct(p)) if p.as_char() == '/') {
                    return None;
                }
                Some(self.parse_element())
            }
            // String literal — static text
            TokenTree::Literal(_) => {
                let lit = self.advance().unwrap();
                if let TokenTree::Literal(l) = &lit {
                    let raw = l.to_string();
                    if raw.starts_with('"') && raw.ends_with('"') {
                        let text = raw[1..raw.len() - 1].to_string();
                        Some(ViewNode::Text(text))
                    } else {
                        panic!("oxide view!: only string literals are supported as text nodes, got: {}", raw);
                    }
                } else {
                    unreachable!()
                }
            }
            // `{ expr }` — reactive expression
            TokenTree::Group(g) if g.delimiter() == Delimiter::Brace => {
                let g = self.advance().unwrap();
                if let TokenTree::Group(g) = g {
                    Some(ViewNode::DynExpr(g.stream()))
                } else {
                    unreachable!()
                }
            }
            other => panic!("oxide view!: unexpected token {:?}", other),
        }
    }

    fn parse_element(&mut self) -> ViewNode {
        self.expect_punct('<');
        let tag = self.expect_ident();
        let mut attrs = Vec::new();

        // Attributes until `>` or `/>`
        loop {
            if self.is_punct('>') {
                self.advance();
                break;
            }
            if self.is_punct('/') {
                self.advance();
                self.expect_punct('>');
                return ViewNode::Element {
                    tag,
                    attrs,
                    children: vec![],
                };
            }

            let attr_name = self.expect_ident();

            // `on:event` pattern
            if self.is_punct(':') {
                self.advance(); // ':'
                let event_name = self.expect_ident();
                self.expect_punct('=');
                let handler = self.parse_braced_expr();
                attrs.push(Attr::Event {
                    event: event_name,
                    handler,
                });
            } else {
                self.expect_punct('=');
                match self.peek() {
                    Some(TokenTree::Literal(_)) => {
                        let lit = self.advance().unwrap();
                        if let TokenTree::Literal(l) = &lit {
                            let raw = l.to_string();
                            if raw.starts_with('"') && raw.ends_with('"') {
                                attrs.push(Attr::Static {
                                    name: attr_name,
                                    value: raw[1..raw.len() - 1].to_string(),
                                });
                            } else {
                                panic!("oxide view!: attribute value must be a \"string\" or {{expression}}");
                            }
                        }
                    }
                    Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Brace => {
                        let stream = self.parse_braced_expr();
                        if let Some(event) = attr_name.strip_prefix("on") {
                            attrs.push(Attr::Event {
                                event: event.to_string(),
                                handler: stream,
                            });
                        } else {
                            attrs.push(Attr::Dynamic {
                                name: attr_name,
                                value: stream,
                            });
                        }
                    }
                    _ => panic!("oxide view!: expected attribute value after '='"),
                }
            }
        }

        // Children until `</tag>`
        let mut children = Vec::new();
        loop {
            // Closing tag?
            if self.is_punct('<') {
                if matches!(self.peek_at(1), Some(TokenTree::Punct(p)) if p.as_char() == '/') {
                    self.advance(); // <
                    self.advance(); // /
                    let close_tag = self.expect_ident();
                    assert_eq!(
                        close_tag, tag,
                        "oxide view!: mismatched tags — expected </{}>, found </{}>",
                        tag, close_tag
                    );
                    self.expect_punct('>');
                    break;
                }
            }

            match self.parse_node() {
                Some(node) => children.push(node),
                None => break,
            }
        }

        ViewNode::Element {
            tag,
            attrs,
            children,
        }
    }

    fn parse_braced_expr(&mut self) -> TokenStream2 {
        match self.advance() {
            Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Brace => g.stream(),
            other => panic!(
                "oxide view!: expected {{expression}}, found {:?}",
                other
            ),
        }
    }
}

// ---------------------------------------------------------------------------
// Code generation
// ---------------------------------------------------------------------------

fn generate(node: &ViewNode, counter: &mut usize) -> TokenStream2 {
    match node {
        ViewNode::Element {
            tag,
            attrs,
            children,
        } => {
            let el = format_ident!("__el_{}", *counter);
            *counter += 1;

            let mut stmts: Vec<TokenStream2> = Vec::new();

            // Create element
            stmts.push(quote! {
                let #el = ::oxide::dom::create_element(#tag);
            });

            // Attributes
            for attr in attrs {
                match attr {
                    Attr::Static { name, value } => {
                        stmts.push(quote! {
                            ::oxide::dom::set_attribute(&#el, #name, #value);
                        });
                    }
                    Attr::Event { event, handler } => {
                        stmts.push(quote! {
                            ::oxide::dom::add_event_listener(&#el, #event, #handler);
                        });
                    }
                    Attr::Dynamic { name, value } => {
                        let el_dyn = format_ident!("__dyn_{}", *counter);
                        *counter += 1;
                        stmts.push(quote! {
                            {
                                let #el_dyn = #el.clone();
                                ::oxide::create_effect(move || {
                                    ::oxide::dom::set_attribute(
                                        &#el_dyn, #name,
                                        &::std::format!("{}", #value)
                                    );
                                });
                            }
                        });
                    }
                }
            }

            // Children
            for child in children {
                match child {
                    ViewNode::Text(text) => {
                        stmts.push(quote! {
                            ::oxide::dom::append_text(&#el, #text);
                        });
                    }
                    ViewNode::DynExpr(expr) => {
                        let txt = format_ident!("__txt_{}", *counter);
                        let tc = format_ident!("__tc_{}", *counter);
                        *counter += 1;
                        stmts.push(quote! {
                            let #txt = ::oxide::dom::create_text_node("");
                            let #tc = #txt.clone();
                            ::oxide::create_effect(move || {
                                #tc.set_text_content(
                                    ::std::option::Option::Some(
                                        &::std::format!("{}", #expr)
                                    )
                                );
                            });
                            ::oxide::dom::append_node(&#el, &#txt);
                        });
                    }
                    ViewNode::Element { .. } => {
                        let child_code = generate(child, counter);
                        stmts.push(quote! {
                            ::oxide::dom::append_node(&#el, &#child_code);
                        });
                    }
                }
            }

            quote! {
                {
                    #(#stmts)*
                    #el
                }
            }
        }
        ViewNode::Text(_) | ViewNode::DynExpr(_) => {
            panic!("oxide view!: top-level node must be an element")
        }
    }
}
