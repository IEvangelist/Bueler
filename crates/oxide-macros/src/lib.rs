use proc_macro::TokenStream;
use proc_macro2::{Delimiter, TokenStream as TokenStream2, TokenTree};
use quote::{format_ident, quote};

/// Modern JSX-like view macro with reactive rendering.
///
/// Supports: elements, dynamic attrs, events, `bind:value`, `bind:checked`,
/// `class:name`, `{if ... { } else { }}`, `{for x in y { }}`, and
/// `<Component prop={val} />`.
#[proc_macro]
pub fn view(input: TokenStream) -> TokenStream {
    let input2: TokenStream2 = input.into();
    let mut parser = ViewParser::new(input2);
    let node = parser
        .parse_node()
        .expect("bueler view!: expected at least one element");
    let mut counter = 0usize;
    gen_root(&node, &mut counter).into()
}

// ═══════════════════════════════════════════════════════════════════════════
// AST
// ═══════════════════════════════════════════════════════════════════════════

enum ViewNode {
    Element {
        tag: String,
        attrs: Vec<Attr>,
        children: Vec<ViewNode>,
    },
    Text(String),
    DynExpr(TokenStream2),
    Conditional {
        condition: TokenStream2,
        if_true: Vec<ViewNode>,
        if_false: Vec<ViewNode>,
    },
    EachLoop {
        binding: TokenStream2,
        iterable: TokenStream2,
        body: Vec<ViewNode>,
    },
    Component {
        name: String,
        props: Vec<(String, TokenStream2)>,
    },
}

enum Attr {
    Static { name: String, value: String },
    Event { event: String, handler: TokenStream2 },
    Dynamic { name: String, value: TokenStream2 },
    Bind { prop: String, signal: TokenStream2 },
    ClassToggle { class: String, condition: TokenStream2 },
}

// ═══════════════════════════════════════════════════════════════════════════
// Parser
// ═══════════════════════════════════════════════════════════════════════════

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
            panic!("bueler view!: expected '{}', found {:?}", ch, self.peek());
        }
        self.advance();
    }

    fn expect_ident(&mut self) -> String {
        match self.advance() {
            Some(TokenTree::Ident(i)) => i.to_string(),
            other => panic!("bueler view!: expected identifier, found {:?}", other),
        }
    }

    // ── Node ─────────────────────────────────────────────────────────────

    fn parse_node(&mut self) -> Option<ViewNode> {
        match self.peek()? {
            TokenTree::Punct(p) if p.as_char() == '<' => {
                if matches!(self.peek_at(1), Some(TokenTree::Punct(p)) if p.as_char() == '/') {
                    return None;
                }
                Some(self.parse_element())
            }
            TokenTree::Literal(_) => {
                let lit = self.advance().unwrap();
                if let TokenTree::Literal(l) = &lit {
                    let raw = l.to_string();
                    if raw.starts_with('"') && raw.ends_with('"') {
                        Some(ViewNode::Text(raw[1..raw.len() - 1].to_string()))
                    } else {
                        panic!("bueler view!: only string literals as text, got: {}", raw);
                    }
                } else {
                    unreachable!()
                }
            }
            TokenTree::Group(g) if g.delimiter() == Delimiter::Brace => {
                let g = self.advance().unwrap();
                if let TokenTree::Group(g) = g {
                    let inner: Vec<TokenTree> = g.stream().into_iter().collect();
                    if matches!(inner.first(), Some(TokenTree::Ident(id)) if id.to_string() == "if")
                    {
                        return Some(parse_conditional(inner));
                    }
                    if matches!(inner.first(), Some(TokenTree::Ident(id)) if id.to_string() == "for")
                    {
                        return Some(parse_for_loop(inner));
                    }
                    Some(ViewNode::DynExpr(TokenStream2::from_iter(inner)))
                } else {
                    unreachable!()
                }
            }
            other => panic!("bueler view!: unexpected token {:?}", other),
        }
    }

    fn parse_children_until_end(&mut self) -> Vec<ViewNode> {
        let mut out = Vec::new();
        while self.peek().is_some() {
            match self.parse_node() {
                Some(n) => out.push(n),
                None => break,
            }
        }
        out
    }

    // ── Element ──────────────────────────────────────────────────────────

    fn parse_element(&mut self) -> ViewNode {
        self.expect_punct('<');
        let tag = self.expect_ident();

        if tag.chars().next().unwrap().is_uppercase() {
            return self.parse_component(tag);
        }

        let mut attrs = Vec::new();
        loop {
            if self.is_punct('>') {
                self.advance();
                break;
            }
            if self.is_punct('/') {
                self.advance();
                self.expect_punct('>');
                return ViewNode::Element { tag, attrs, children: vec![] };
            }
            attrs.push(self.parse_attr());
        }

        let mut children = Vec::new();
        loop {
            if self.is_punct('<')
                && matches!(self.peek_at(1), Some(TokenTree::Punct(p)) if p.as_char() == '/')
            {
                self.advance();
                self.advance();
                let close = self.expect_ident();
                assert_eq!(close, tag, "bueler view!: mismatched </{}> for <{}>", close, tag);
                self.expect_punct('>');
                break;
            }
            match self.parse_node() {
                Some(n) => children.push(n),
                None => break,
            }
        }

        ViewNode::Element { tag, attrs, children }
    }

    // ── Attribute ────────────────────────────────────────────────────────

    fn parse_attr(&mut self) -> Attr {
        let name = self.expect_ident();

        // Namespaced: on:click, bind:value, class:active
        if self.is_punct(':') {
            self.advance();
            let suffix = self.expect_ident();
            self.expect_punct('=');
            let value = self.parse_braced_expr();
            return match name.as_str() {
                "on" => Attr::Event { event: suffix, handler: value },
                "bind" => Attr::Bind { prop: suffix, signal: value },
                "class" => Attr::ClassToggle { class: suffix, condition: value },
                _ => panic!("bueler view!: unknown namespace '{}:'. Use on:, bind:, class:", name),
            };
        }

        self.expect_punct('=');

        match self.peek() {
            Some(TokenTree::Literal(_)) => {
                let lit = self.advance().unwrap();
                if let TokenTree::Literal(l) = &lit {
                    let raw = l.to_string();
                    if raw.starts_with('"') && raw.ends_with('"') {
                        Attr::Static { name, value: raw[1..raw.len()-1].to_string() }
                    } else {
                        panic!("bueler view!: attr value must be \"string\" or {{expr}}");
                    }
                } else { unreachable!() }
            }
            Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Brace => {
                let stream = self.parse_braced_expr();
                if let Some(event) = name.strip_prefix("on") {
                    Attr::Event { event: event.to_string(), handler: stream }
                } else {
                    Attr::Dynamic { name, value: stream }
                }
            }
            _ => panic!("bueler view!: expected value after '='"),
        }
    }

    fn parse_braced_expr(&mut self) -> TokenStream2 {
        match self.advance() {
            Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Brace => g.stream(),
            other => panic!("bueler view!: expected {{expr}}, found {:?}", other),
        }
    }

    // ── Component ────────────────────────────────────────────────────────

    fn parse_component(&mut self, name: String) -> ViewNode {
        let mut props = Vec::new();
        loop {
            if self.is_punct('/') {
                self.advance();
                self.expect_punct('>');
                break;
            }
            if self.is_punct('>') {
                self.advance();
                // Skip children until </Name>
                loop {
                    if self.is_punct('<')
                        && matches!(self.peek_at(1), Some(TokenTree::Punct(p)) if p.as_char() == '/')
                    {
                        self.advance();
                        self.advance();
                        let close = self.expect_ident();
                        assert_eq!(close, name);
                        self.expect_punct('>');
                        break;
                    }
                    self.advance();
                }
                break;
            }
            let key = self.expect_ident();
            self.expect_punct('=');
            let val = self.parse_braced_expr();
            props.push((key, val));
        }
        ViewNode::Component { name, props }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Conditional / Loop helpers (parse inner brace tokens)
// ═══════════════════════════════════════════════════════════════════════════

fn parse_conditional(tokens: Vec<TokenTree>) -> ViewNode {
    let mut cur = 1; // skip "if"
    let mut condition = Vec::new();
    while cur < tokens.len() {
        if matches!(&tokens[cur], TokenTree::Group(g) if g.delimiter() == Delimiter::Brace) {
            break;
        }
        condition.push(tokens[cur].clone());
        cur += 1;
    }
    let if_true = match tokens.get(cur) {
        Some(TokenTree::Group(g)) => {
            cur += 1;
            ViewParser::new(g.stream()).parse_children_until_end()
        }
        _ => panic!("bueler view!: expected {{ view }} after if condition"),
    };
    let if_false = if let Some(TokenTree::Ident(id)) = tokens.get(cur) {
        if id.to_string() == "else" {
            cur += 1;
            match tokens.get(cur) {
                Some(TokenTree::Group(g)) => ViewParser::new(g.stream()).parse_children_until_end(),
                _ => panic!("bueler view!: expected {{ view }} after else"),
            }
        } else { vec![] }
    } else { vec![] };
    ViewNode::Conditional { condition: TokenStream2::from_iter(condition), if_true, if_false }
}

fn parse_for_loop(tokens: Vec<TokenTree>) -> ViewNode {
    let mut cur = 1; // skip "for"
    let mut binding = Vec::new();
    while cur < tokens.len() {
        if let TokenTree::Ident(id) = &tokens[cur] {
            if id.to_string() == "in" { cur += 1; break; }
        }
        binding.push(tokens[cur].clone());
        cur += 1;
    }
    let mut iterable = Vec::new();
    while cur < tokens.len() {
        if matches!(&tokens[cur], TokenTree::Group(g) if g.delimiter() == Delimiter::Brace) {
            break;
        }
        iterable.push(tokens[cur].clone());
        cur += 1;
    }
    let body = match tokens.get(cur) {
        Some(TokenTree::Group(g)) => ViewParser::new(g.stream()).parse_children_until_end(),
        _ => panic!("bueler view!: expected {{ view }} after for ... in expression"),
    };
    ViewNode::EachLoop {
        binding: TokenStream2::from_iter(binding),
        iterable: TokenStream2::from_iter(iterable),
        body,
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Code generation
// ═══════════════════════════════════════════════════════════════════════════

fn gen_root(node: &ViewNode, c: &mut usize) -> TokenStream2 {
    match node {
        ViewNode::Element { tag, attrs, children } => {
            let el = format_ident!("__el_{}", *c);
            *c += 1;
            let a = gen_attrs(attrs, &el, c);
            let ch = gen_children(children, &el, c, false);
            quote! {{ let #el = ::bueler::dom::create_element(#tag); #a #ch #el }}
        }
        _ => panic!("bueler view!: root must be an element"),
    }
}

fn gen_attrs(attrs: &[Attr], el: &proc_macro2::Ident, c: &mut usize) -> TokenStream2 {
    let ss: Vec<_> = attrs.iter().map(|a| gen_attr(a, el, c)).collect();
    quote! { #(#ss)* }
}

fn gen_attr(attr: &Attr, el: &proc_macro2::Ident, c: &mut usize) -> TokenStream2 {
    match attr {
        Attr::Static { name, value } => quote! {
            ::bueler::dom::set_attribute(&#el, #name, #value);
        },
        Attr::Event { event, handler } => quote! {
            ::bueler::dom::add_event_listener(&#el, #event, #handler);
        },
        Attr::Dynamic { name, value } => {
            let d = format_ident!("__dyn_{}", *c); *c += 1;
            quote! {{
                let #d = #el.clone();
                ::bueler::create_effect(move || {
                    ::bueler::dom::set_attribute(&#d, #name, &::std::format!("{}", #value));
                });
            }}
        }
        Attr::Bind { prop, signal } => {
            let b = format_ident!("__bind_{}", *c); *c += 1;
            if prop == "checked" {
                quote! {{
                    let #b = #el.clone();
                    ::bueler::create_effect(move || {
                        ::bueler::dom::set_property(&#b, "checked",
                            &::wasm_bindgen::JsValue::from_bool(#signal.get()));
                    });
                    ::bueler::dom::add_event_listener(&#el, "change",
                        move |__e: ::bueler::dom::Event| {
                            #signal.set(::bueler::dom::event_target_checked(&__e));
                        });
                }}
            } else {
                quote! {{
                    let #b = #el.clone();
                    ::bueler::create_effect(move || {
                        ::bueler::dom::set_property(&#b, #prop,
                            &::wasm_bindgen::JsValue::from_str(&::std::format!("{}", #signal)));
                    });
                    ::bueler::dom::add_event_listener(&#el, "input",
                        move |__e: ::bueler::dom::Event| {
                            let __v = ::bueler::dom::event_target_value(&__e);
                            if let ::std::result::Result::Ok(__p) = __v.parse() {
                                #signal.set(__p);
                            }
                        });
                }}
            }
        }
        Attr::ClassToggle { class, condition } => {
            let t = format_ident!("__cls_{}", *c); *c += 1;
            quote! {{
                let #t = #el.clone();
                ::bueler::create_effect(move || {
                    ::bueler::dom::toggle_class(&#t, #class, #condition);
                });
            }}
        }
    }
}

fn gen_children(children: &[ViewNode], p: &proc_macro2::Ident, c: &mut usize, reactive: bool) -> TokenStream2 {
    let ss: Vec<_> = children.iter().map(|ch| gen_child(ch, p, c, reactive)).collect();
    quote! { #(#ss)* }
}

fn gen_child(node: &ViewNode, p: &proc_macro2::Ident, c: &mut usize, reactive: bool) -> TokenStream2 {
    match node {
        ViewNode::Element { tag, attrs, children } => {
            let el = format_ident!("__el_{}", *c); *c += 1;
            let a = gen_attrs(attrs, &el, c);
            let ch = gen_children(children, &el, c, reactive);
            quote! {
                let #el = ::bueler::dom::create_element(#tag);
                #a #ch
                ::bueler::dom::append_node(&#p, &#el);
            }
        }
        ViewNode::Text(text) => quote! {
            ::bueler::dom::append_text(&#p, #text);
        },
        ViewNode::DynExpr(expr) => {
            if reactive {
                let t = format_ident!("__txt_{}", *c); *c += 1;
                quote! {
                    let #t = ::bueler::dom::create_text_node(&::std::format!("{}", #expr));
                    ::bueler::dom::append_node(&#p, &#t);
                }
            } else {
                let t = format_ident!("__txt_{}", *c);
                let tc = format_ident!("__tc_{}", *c);
                *c += 1;
                quote! {
                    let #t = ::bueler::dom::create_text_node("");
                    let #tc = #t.clone();
                    ::bueler::create_effect(move || {
                        #tc.set_text_content(::std::option::Option::Some(
                            &::std::format!("{}", #expr)));
                    });
                    ::bueler::dom::append_node(&#p, &#t);
                }
            }
        }
        ViewNode::Conditional { condition, if_true, if_false } => {
            let w = format_ident!("__cond_{}", *c);
            let wr = format_ident!("__cr_{}", *c);
            *c += 1;
            let ts = gen_children(if_true, &wr, c, true);
            let fs = if if_false.is_empty() {
                quote! {}
            } else {
                let s = gen_children(if_false, &wr, c, true);
                quote! { else { #s } }
            };
            quote! {{
                let #w = ::bueler::dom::create_element("span");
                ::bueler::dom::set_style(&#w, "display", "contents");
                let #wr = #w.clone();
                ::bueler::create_effect(move || {
                    ::bueler::dom::clear_children(&#wr);
                    if #condition { #ts } #fs
                });
                ::bueler::dom::append_node(&#p, &#w);
            }}
        }
        ViewNode::EachLoop { binding, iterable, body } => {
            let w = format_ident!("__each_{}", *c);
            let wr = format_ident!("__er_{}", *c);
            *c += 1;
            let bs = gen_children(body, &wr, c, true);
            quote! {{
                let #w = ::bueler::dom::create_element("span");
                ::bueler::dom::set_style(&#w, "display", "contents");
                let #wr = #w.clone();
                ::bueler::create_effect(move || {
                    ::bueler::dom::clear_children(&#wr);
                    for #binding in #iterable { #bs }
                });
                ::bueler::dom::append_node(&#p, &#w);
            }}
        }
        ViewNode::Component { name, props } => {
            let comp: TokenStream2 = name.parse().unwrap();
            let fields: Vec<TokenStream2> = props.iter().map(|(k, v)| {
                let key: TokenStream2 = k.parse().unwrap();
                quote! { #key: #v }
            }).collect();
            if fields.is_empty() {
                quote! {
                    ::bueler::dom::append_node(&#p,
                        &::bueler::Component::render(#comp {}));
                }
            } else {
                quote! {
                    ::bueler::dom::append_node(&#p,
                        &::bueler::Component::render(#comp { #(#fields),* }));
                }
            }
        }
    }
}
