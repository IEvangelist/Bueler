//! Thin DOM wrapper around the host-tested `bueler-showcase-highlight`
//! tokenizer. Produces a `<pre class="bu-hl"><code>` tree with coloured
//! `<span class="hl-…">` runs — and zero JavaScript.

use bueler::dom::*;
use bueler_showcase_highlight::tokenize;

/// Render `code` as a syntax-highlighted Rust snippet in a fresh `<pre>`.
pub fn rust(code: impl AsRef<str>) -> web_sys::Element {
    let pre = pre_element();
    render(&pre, code);
    pre
}

/// Plain-shell rendering — no tokenization, just the same chrome as a
/// highlighted block so shell snippets share the visual style.
pub fn shell(code: impl AsRef<str>) -> web_sys::Element {
    ensure_css();
    let pre = create_element("pre");
    set_attribute(&pre, "class", "bu-hl bu-hl-shell");
    let inner = create_element("code");
    set_attribute(&inner, "class", "bu-hl-code");
    append_text(&inner, code.as_ref());
    append_node(&pre, &inner);
    pre
}

/// Allocate an empty highlight-ready `<pre>` (without content). Use this
/// when the snippet text is reactive and you need a stable element to
/// re-render into via [`render`].
pub fn pre_element() -> web_sys::Element {
    ensure_css();
    let pre = create_element("pre");
    set_attribute(&pre, "class", "bu-hl");
    pre
}

/// (Re-)render `code` into `pre`, replacing any previous contents.
pub fn render(pre: &web_sys::Element, code: impl AsRef<str>) {
    ensure_css();
    set_attribute(pre, "class", "bu-hl");
    pre.set_text_content(None);
    let inner = create_element("code");
    set_attribute(&inner, "class", "bu-hl-code");
    for (text, kind) in tokenize(code.as_ref()) {
        let cls = kind.css_class();
        if cls.is_empty() {
            append_text(&inner, &text);
        } else {
            let span = create_element("span");
            set_attribute(&span, "class", cls);
            append_text(&span, &text);
            append_node(&inner, &span);
        }
    }
    append_node(pre, &inner);
}

fn ensure_css() {
    use std::sync::atomic::{AtomicBool, Ordering};
    static INJECTED: AtomicBool = AtomicBool::new(false);
    if INJECTED.swap(true, Ordering::SeqCst) {
        return;
    }
    let doc = match web_sys::window().and_then(|w| w.document()) {
        Some(d) => d,
        None => return,
    };
    let style = match doc.create_element("style") {
        Ok(s) => s,
        Err(_) => return,
    };
    style.set_text_content(Some(CSS));
    if let Some(head) = doc.head() {
        let _ = head.append_child(&style);
    }
}

const CSS: &str = "\
.bu-hl{background:#0a0a0a;border:1px solid #2a2a2a;border-radius:10px;padding:0.9rem 1.1rem;\
overflow-x:auto;font-family:'Fira Code','JetBrains Mono','SF Mono',Consolas,monospace;\
font-size:0.85rem;color:#d4d4d4;line-height:1.6;margin:0.75rem 0;tab-size:4}\
.bu-hl-shell{color:#cde2c5}\
.bu-hl-code{font:inherit;color:inherit;background:none;padding:0;white-space:pre}\
.bu-hl .hl-kw{color:#f97316;font-weight:600}\
.bu-hl .hl-pty{color:#fbbf24}\
.bu-hl .hl-ty{color:#fbbf24}\
.bu-hl .hl-str{color:#a5e85d}\
.bu-hl .hl-num{color:#f0d040}\
.bu-hl .hl-com{color:#5a6473;font-style:italic}\
.bu-hl .hl-mac{color:#c084fc;font-weight:600}\
.bu-hl .hl-fn{color:#38bdf8}\
.bu-hl .hl-attr{color:#e879f9;font-style:italic}\
.bu-hl .hl-punct{color:#8a8a8a}\
.bu-hl .hl-life{color:#fb923c;font-style:italic}";
