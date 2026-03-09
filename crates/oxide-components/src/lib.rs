//! # bueler-components
//!
//! Best-in-class UI component library for Bueler applications.
//! All components use a builder API, include built-in dark-theme CSS
//! (auto-injected on first use), and follow WAI-ARIA accessibility patterns.
//!
//! ```ignore
//! use bueler_components::*;
//!
//! let btn = button("Save").primary().on_click(move |_| save());
//! let inp = text_input("Email").placeholder("you@example.com").bind(email);
//! let card = card("Settings").body(content).build();
//! ```

use bueler_core::{create_effect, signal, Signal};
use bueler_dom::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

// ═══════════════════════════════════════════════════════════════════════════
// CSS injection (lazy, once per component type)
// ═══════════════════════════════════════════════════════════════════════════

fn inject_css(id: &str, css: &str) {
    if query_selector(&format!("style[data-bu=\"{}\"]", id)).is_some() {
        return;
    }
    let doc = web_sys::window().unwrap().document().unwrap();
    let style = doc.create_element("style").unwrap();
    style.set_attribute("data-bu", id).ok();
    style.set_text_content(Some(css));
    if let Some(head) = doc.head() {
        head.append_child(&style).ok();
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 1. Button
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Clone, Copy, PartialEq)]
pub enum Variant { Primary, Secondary, Outline, Danger, Ghost }

#[derive(Clone, Copy)]
pub enum Size { Small, Medium, Large }

pub struct ButtonBuilder {
    label: String,
    variant: Variant,
    size: Size,
    loading: bool,
    disabled: bool,
}

pub fn button(label: &str) -> ButtonBuilder {
    ButtonBuilder {
        label: label.to_string(),
        variant: Variant::Secondary,
        size: Size::Medium,
        loading: false,
        disabled: false,
    }
}

impl ButtonBuilder {
    pub fn primary(mut self) -> Self { self.variant = Variant::Primary; self }
    pub fn outline(mut self) -> Self { self.variant = Variant::Outline; self }
    pub fn danger(mut self) -> Self { self.variant = Variant::Danger; self }
    pub fn ghost(mut self) -> Self { self.variant = Variant::Ghost; self }
    pub fn small(mut self) -> Self { self.size = Size::Small; self }
    pub fn large(mut self) -> Self { self.size = Size::Large; self }
    pub fn loading(mut self, v: bool) -> Self { self.loading = v; self }
    pub fn disabled(mut self, v: bool) -> Self { self.disabled = v; self }

    pub fn on_click(self, handler: impl FnMut(web_sys::Event) + 'static) -> web_sys::Element {
        let el = self.build();
        add_event_listener(&el, "click", handler);
        el
    }

    pub fn build(self) -> web_sys::Element {
        inject_css("btn", BUTTON_CSS);
        let el = create_element("button");
        let mut cls = format!("bu-btn bu-btn-{}", match self.variant {
            Variant::Primary => "primary", Variant::Secondary => "secondary",
            Variant::Outline => "outline", Variant::Danger => "danger", Variant::Ghost => "ghost",
        });
        cls += match self.size { Size::Small => " bu-btn-sm", Size::Large => " bu-btn-lg", _ => "" };
        if self.loading { cls += " bu-btn-loading"; }
        set_attribute(&el, "class", &cls);
        if self.disabled || self.loading { set_attribute(&el, "disabled", ""); }
        if self.loading {
            set_inner_html(&el, &format!("<span class=\"bu-spinner-inline\"></span> {}", self.label));
        } else {
            append_text(&el, &self.label);
        }
        el.set_attribute("role", "button").ok();
        el
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. TextInput
// ═══════════════════════════════════════════════════════════════════════════

pub struct InputBuilder {
    label: String,
    input_type: String,
    placeholder: String,
    required: bool,
    error_msg: Option<String>,
    signal: Option<Signal<String>>,
}

pub fn text_input(label: &str) -> InputBuilder {
    InputBuilder {
        label: label.to_string(),
        input_type: "text".into(),
        placeholder: String::new(),
        required: false,
        error_msg: None,
        signal: None,
    }
}

impl InputBuilder {
    pub fn input_type(mut self, t: &str) -> Self { self.input_type = t.into(); self }
    pub fn placeholder(mut self, p: &str) -> Self { self.placeholder = p.into(); self }
    pub fn required(mut self) -> Self { self.required = true; self }
    pub fn error(mut self, msg: &str) -> Self { self.error_msg = Some(msg.into()); self }
    pub fn bind(mut self, s: Signal<String>) -> Self { self.signal = Some(s); self }

    pub fn build(self) -> web_sys::Element {
        inject_css("input", INPUT_CSS);
        let wrap = create_element("div");
        set_attribute(&wrap, "class", "bu-field");
        if !self.label.is_empty() {
            let lbl = create_element("label");
            set_attribute(&lbl, "class", "bu-label");
            append_text(&lbl, &self.label);
            if self.required { let req = create_element("span"); set_attribute(&req, "class", "bu-required"); append_text(&req, " *"); append_node(&lbl, &req); }
            append_node(&wrap, &lbl);
        }
        let input = create_element("input");
        let cls = if self.error_msg.is_some() { "bu-input bu-input-error" } else { "bu-input" };
        set_attribute(&input, "class", cls);
        set_attribute(&input, "type", &self.input_type);
        if !self.placeholder.is_empty() { set_attribute(&input, "placeholder", &self.placeholder); }
        if self.required { set_attribute(&input, "required", ""); input.set_attribute("aria-required", "true").ok(); }
        if let Some(s) = self.signal {
            let inp = input.clone();
            create_effect(move || { set_property(&inp, "value", &JsValue::from_str(&s.get())); });
            add_event_listener(&input, "input", move |e| { s.set(event_target_value(&e)); });
        }
        append_node(&wrap, &input);
        if let Some(msg) = &self.error_msg {
            let err = create_element("div");
            set_attribute(&err, "class", "bu-error");
            err.set_attribute("role", "alert").ok();
            append_text(&err, msg);
            append_node(&wrap, &err);
        }
        wrap
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. TextArea
// ═══════════════════════════════════════════════════════════════════════════

pub fn textarea(label: &str, value: Signal<String>) -> web_sys::Element {
    inject_css("input", INPUT_CSS);
    let wrap = create_element("div");
    set_attribute(&wrap, "class", "bu-field");
    if !label.is_empty() {
        let lbl = create_element("label");
        set_attribute(&lbl, "class", "bu-label");
        append_text(&lbl, label);
        append_node(&wrap, &lbl);
    }
    let ta = create_element("textarea");
    set_attribute(&ta, "class", "bu-input");
    set_attribute(&ta, "rows", "4");
    let ta_ref = ta.clone();
    create_effect(move || { set_property(&ta_ref, "value", &JsValue::from_str(&value.get())); });
    let v = value;
    add_event_listener(&ta, "input", move |e| { v.set(event_target_value(&e)); });
    append_node(&wrap, &ta);
    wrap
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. Select
// ═══════════════════════════════════════════════════════════════════════════

pub fn select(label: &str, options: &[(&str, &str)], value: Signal<String>) -> web_sys::Element {
    inject_css("input", INPUT_CSS);
    let wrap = create_element("div");
    set_attribute(&wrap, "class", "bu-field");
    if !label.is_empty() {
        let lbl = create_element("label");
        set_attribute(&lbl, "class", "bu-label");
        append_text(&lbl, label);
        append_node(&wrap, &lbl);
    }
    let sel = create_element("select");
    set_attribute(&sel, "class", "bu-input");
    for &(val, text) in options {
        let opt = create_element("option");
        set_attribute(&opt, "value", val);
        append_text(&opt, text);
        append_node(&sel, &opt);
    }
    let sel_ref = sel.clone();
    create_effect(move || { set_property(&sel_ref, "value", &JsValue::from_str(&value.get())); });
    let v = value;
    add_event_listener(&sel, "change", move |e| { v.set(event_target_value(&e)); });
    append_node(&wrap, &sel);
    wrap
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. Checkbox
// ═══════════════════════════════════════════════════════════════════════════

pub fn checkbox(label: &str, checked: Signal<bool>) -> web_sys::Element {
    inject_css("checkbox", CHECKBOX_CSS);
    let wrap = create_element("label");
    set_attribute(&wrap, "class", "bu-checkbox");
    let input = create_element("input");
    set_attribute(&input, "type", "checkbox");
    let inp_ref = input.clone();
    create_effect(move || { set_property(&inp_ref, "checked", &JsValue::from_bool(checked.get())); });
    let c = checked;
    add_event_listener(&input, "change", move |e| { c.set(event_target_checked(&e)); });
    append_node(&wrap, &input);
    let span = create_element("span");
    append_text(&span, label);
    append_node(&wrap, &span);
    wrap
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. Card
// ═══════════════════════════════════════════════════════════════════════════

pub struct CardBuilder {
    title: String,
    body_el: Option<web_sys::Element>,
    footer_el: Option<web_sys::Element>,
}

pub fn card(title: &str) -> CardBuilder {
    CardBuilder { title: title.into(), body_el: None, footer_el: None }
}

impl CardBuilder {
    pub fn body(mut self, el: web_sys::Element) -> Self { self.body_el = Some(el); self }
    pub fn footer(mut self, el: web_sys::Element) -> Self { self.footer_el = Some(el); self }

    pub fn build(self) -> web_sys::Element {
        inject_css("card", CARD_CSS);
        let card = create_element("div");
        set_attribute(&card, "class", "bu-card");
        if !self.title.is_empty() {
            let header = create_element("div");
            set_attribute(&header, "class", "bu-card-header");
            append_text(&header, &self.title);
            append_node(&card, &header);
        }
        if let Some(body) = self.body_el {
            let b = create_element("div");
            set_attribute(&b, "class", "bu-card-body");
            append_node(&b, &body);
            append_node(&card, &b);
        }
        if let Some(footer) = self.footer_el {
            let f = create_element("div");
            set_attribute(&f, "class", "bu-card-footer");
            append_node(&f, &footer);
            append_node(&card, &f);
        }
        card
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. Alert
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Clone, Copy)]
pub enum Severity { Success, Warning, Error, Info }

pub struct AlertBuilder {
    message: String,
    severity: Severity,
    dismissible: Option<Signal<bool>>,
}

pub fn alert(message: &str) -> AlertBuilder {
    AlertBuilder { message: message.into(), severity: Severity::Info, dismissible: None }
}

impl AlertBuilder {
    pub fn success(mut self) -> Self { self.severity = Severity::Success; self }
    pub fn warning(mut self) -> Self { self.severity = Severity::Warning; self }
    pub fn error(mut self) -> Self { self.severity = Severity::Error; self }
    pub fn info(mut self) -> Self { self.severity = Severity::Info; self }
    pub fn dismissible(mut self, visible: Signal<bool>) -> Self { self.dismissible = Some(visible); self }

    pub fn build(self) -> web_sys::Element {
        inject_css("alert", ALERT_CSS);
        let el = create_element("div");
        let sev = match self.severity {
            Severity::Success => "success", Severity::Warning => "warning",
            Severity::Error => "error", Severity::Info => "info",
        };
        set_attribute(&el, "class", &format!("bu-alert bu-alert-{}", sev));
        el.set_attribute("role", "alert").ok();
        let icon = match self.severity {
            Severity::Success => "✓", Severity::Warning => "⚠", Severity::Error => "✕", Severity::Info => "ℹ",
        };
        let ic = create_element("span");
        set_attribute(&ic, "class", "bu-alert-icon");
        append_text(&ic, icon);
        append_node(&el, &ic);
        append_text(&el, &self.message);
        if let Some(vis) = self.dismissible {
            let btn = create_element("button");
            set_attribute(&btn, "class", "bu-alert-close");
            btn.set_attribute("aria-label", "Dismiss").ok();
            append_text(&btn, "×");
            add_event_listener(&btn, "click", move |_| { vis.set(false); });
            append_node(&el, &btn);
            let el_ref = el.clone();
            create_effect(move || { if !vis.get() { set_style(&el_ref, "display", "none"); } else { set_style(&el_ref, "display", ""); } });
        }
        el
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 8. Modal
// ═══════════════════════════════════════════════════════════════════════════

pub struct ModalBuilder {
    open: Signal<bool>,
    title: String,
    body_el: Option<web_sys::Element>,
}

pub fn modal(open: Signal<bool>) -> ModalBuilder {
    ModalBuilder { open, title: String::new(), body_el: None }
}

impl ModalBuilder {
    pub fn title(mut self, t: &str) -> Self { self.title = t.into(); self }
    pub fn body(mut self, el: web_sys::Element) -> Self { self.body_el = Some(el); self }

    pub fn build(self) -> web_sys::Element {
        inject_css("modal", MODAL_CSS);
        let overlay = create_element("div");
        set_attribute(&overlay, "class", "bu-modal-overlay");
        overlay.set_attribute("role", "dialog").ok();
        overlay.set_attribute("aria-modal", "true").ok();

        let dialog = create_element("div");
        set_attribute(&dialog, "class", "bu-modal");
        if !self.title.is_empty() {
            let header = create_element("div");
            set_attribute(&header, "class", "bu-modal-header");
            let h3 = create_element("h3");
            append_text(&h3, &self.title);
            append_node(&header, &h3);
            let close = create_element("button");
            set_attribute(&close, "class", "bu-modal-close");
            close.set_attribute("aria-label", "Close").ok();
            append_text(&close, "×");
            let o = self.open;
            add_event_listener(&close, "click", move |_| { o.set(false); });
            append_node(&header, &close);
            append_node(&dialog, &header);
        }
        if let Some(body) = self.body_el {
            let b = create_element("div");
            set_attribute(&b, "class", "bu-modal-body");
            append_node(&b, &body);
            append_node(&dialog, &b);
        }
        append_node(&overlay, &dialog);

        // Close on backdrop click
        let o2 = self.open;
        let overlay_ref = overlay.clone();
        add_event_listener(&overlay, "click", move |e| {
            if let Some(t) = e.target() {
                if let Some(el) = t.dyn_ref::<web_sys::Element>() {
                    if el.class_list().contains("bu-modal-overlay") { o2.set(false); }
                }
            }
        });

        // Show/hide
        let overlay_ref2 = overlay_ref.clone();
        create_effect(move || {
            if self.open.get() {
                set_style(&overlay_ref2, "display", "flex");
            } else {
                set_style(&overlay_ref2, "display", "none");
            }
        });
        set_style(&overlay, "display", "none");
        overlay
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 9. Spinner
// ═══════════════════════════════════════════════════════════════════════════

pub fn spinner() -> web_sys::Element {
    inject_css("spinner", SPINNER_CSS);
    let el = create_element("div");
    set_attribute(&el, "class", "bu-spinner");
    el.set_attribute("role", "status").ok();
    el.set_attribute("aria-label", "Loading").ok();
    el
}

pub fn spinner_with_text(text: &str) -> web_sys::Element {
    inject_css("spinner", SPINNER_CSS);
    let wrap = create_element("div");
    set_attribute(&wrap, "class", "bu-spinner-wrap");
    let sp = create_element("div");
    set_attribute(&sp, "class", "bu-spinner");
    sp.set_attribute("role", "status").ok();
    append_node(&wrap, &sp);
    let t = create_element("span");
    set_attribute(&t, "class", "bu-spinner-text");
    append_text(&t, text);
    append_node(&wrap, &t);
    wrap
}

// ═══════════════════════════════════════════════════════════════════════════
// 10. Progress
// ═══════════════════════════════════════════════════════════════════════════

pub fn progress(value: Signal<f64>) -> web_sys::Element {
    inject_css("progress", PROGRESS_CSS);
    let wrap = create_element("div");
    set_attribute(&wrap, "class", "bu-progress");
    wrap.set_attribute("role", "progressbar").ok();
    let bar = create_element("div");
    set_attribute(&bar, "class", "bu-progress-bar");
    let bar_ref = bar.clone();
    let wrap_ref = wrap.clone();
    create_effect(move || {
        let v = value.get().clamp(0.0, 100.0);
        set_style(&bar_ref, "width", &format!("{}%", v));
        wrap_ref.set_attribute("aria-valuenow", &format!("{}", v as i32)).ok();
    });
    append_node(&wrap, &bar);
    wrap
}

// ═══════════════════════════════════════════════════════════════════════════
// 11. Tabs
// ═══════════════════════════════════════════════════════════════════════════

pub fn tabs(items: &[(&str, fn() -> web_sys::Element)]) -> web_sys::Element {
    inject_css("tabs", TABS_CSS);
    let active = signal(0usize);
    let wrap = create_element("div");
    set_attribute(&wrap, "class", "bu-tabs");

    let nav = create_element("div");
    set_attribute(&nav, "class", "bu-tabs-nav");
    nav.set_attribute("role", "tablist").ok();
    let mut tab_btns: Vec<web_sys::Element> = Vec::new();
    for (i, &(label, _)) in items.iter().enumerate() {
        let btn = create_element("button");
        set_attribute(&btn, "class", "bu-tab-btn");
        btn.set_attribute("role", "tab").ok();
        append_text(&btn, label);
        let a = active;
        add_event_listener(&btn, "click", move |_| { a.set(i); });
        tab_btns.push(btn.clone());
        append_node(&nav, &btn);
    }
    append_node(&wrap, &nav);

    let panel = create_element("div");
    set_attribute(&panel, "class", "bu-tab-panel");
    panel.set_attribute("role", "tabpanel").ok();

    let views: Vec<fn() -> web_sys::Element> = items.iter().map(|&(_, v)| v).collect();
    let panel_ref = panel.clone();
    create_effect(move || {
        let idx = active.get();
        clear_children(&panel_ref);
        if idx < views.len() {
            let content = views[idx]();
            append_node(&panel_ref, &content);
        }
        for (i, btn) in tab_btns.iter().enumerate() {
            if i == idx {
                set_attribute(btn, "class", "bu-tab-btn bu-tab-active");
                btn.set_attribute("aria-selected", "true").ok();
            } else {
                set_attribute(btn, "class", "bu-tab-btn");
                btn.set_attribute("aria-selected", "false").ok();
            }
        }
    });
    append_node(&wrap, &panel);
    wrap
}

// ═══════════════════════════════════════════════════════════════════════════
// 12. Badge
// ═══════════════════════════════════════════════════════════════════════════

pub fn badge(text: &str, severity: Severity) -> web_sys::Element {
    inject_css("badge", BADGE_CSS);
    let el = create_element("span");
    let sev = match severity {
        Severity::Success => "success", Severity::Warning => "warning",
        Severity::Error => "error", Severity::Info => "info",
    };
    set_attribute(&el, "class", &format!("bu-badge bu-badge-{}", sev));
    append_text(&el, text);
    el
}

// ═══════════════════════════════════════════════════════════════════════════
// 13. Divider
// ═══════════════════════════════════════════════════════════════════════════

pub fn divider() -> web_sys::Element {
    inject_css("divider", ".bu-divider{border:none;border-top:1px solid #333;margin:1.5rem 0}");
    let el = create_element("hr");
    set_attribute(&el, "class", "bu-divider");
    el
}

// ═══════════════════════════════════════════════════════════════════════════
// 14. Skeleton (loading placeholder)
// ═══════════════════════════════════════════════════════════════════════════

pub fn skeleton(width: &str, height: &str) -> web_sys::Element {
    inject_css("skeleton", SKELETON_CSS);
    let el = create_element("div");
    set_attribute(&el, "class", "bu-skeleton");
    set_style(&el, "width", width);
    set_style(&el, "height", height);
    el
}

// ═══════════════════════════════════════════════════════════════════════════
// CSS constants
// ═══════════════════════════════════════════════════════════════════════════

const BUTTON_CSS: &str = "\
.bu-btn{display:inline-flex;align-items:center;gap:0.5rem;padding:0.5rem 1.1rem;font-size:0.85rem;font-family:inherit;font-weight:500;\
border:1px solid #444;border-radius:8px;cursor:pointer;transition:all .15s;outline:none;color:#e0e0e0;background:#1e1e1e}\
.bu-btn:hover{border-color:#f97316;color:#f97316}\
.bu-btn:active{transform:scale(.97)}\
.bu-btn:disabled{opacity:.5;cursor:not-allowed;transform:none}\
.bu-btn:focus-visible{box-shadow:0 0 0 2px #f97316}\
.bu-btn-primary{background:linear-gradient(135deg,#f97316,#ef4444);border-color:transparent;color:#fff;font-weight:600}\
.bu-btn-primary:hover{opacity:.9;color:#fff;border-color:transparent}\
.bu-btn-outline{background:transparent;border-color:#666}\
.bu-btn-outline:hover{background:rgba(249,115,22,.08)}\
.bu-btn-danger{background:transparent;border-color:#ef4444;color:#ef4444}\
.bu-btn-danger:hover{background:#ef4444;color:#fff}\
.bu-btn-ghost{background:transparent;border-color:transparent}\
.bu-btn-ghost:hover{background:rgba(255,255,255,.05)}\
.bu-btn-sm{padding:0.3rem 0.7rem;font-size:0.75rem}\
.bu-btn-lg{padding:0.65rem 1.5rem;font-size:1rem}\
.bu-btn-loading{pointer-events:none}\
.bu-spinner-inline{display:inline-block;width:14px;height:14px;border:2px solid rgba(255,255,255,.3);\
border-top-color:#fff;border-radius:50%;animation:bu-spin .6s linear infinite}\
@keyframes bu-spin{to{transform:rotate(360deg)}}";

const INPUT_CSS: &str = "\
.bu-field{display:flex;flex-direction:column;gap:0.35rem}\
.bu-label{font-size:0.8rem;font-weight:500;color:#aaa}\
.bu-required{color:#ef4444}\
.bu-input{background:#0a0a0a;border:1px solid #444;border-radius:8px;padding:0.5rem 0.75rem;\
color:#e0e0e0;font-size:0.85rem;font-family:inherit;outline:none;transition:border-color .15s;width:100%}\
.bu-input:focus{border-color:#f97316}\
.bu-input-error{border-color:#ef4444}\
.bu-error{font-size:0.75rem;color:#ef4444}\
textarea.bu-input{resize:vertical;min-height:80px}\
select.bu-input{cursor:pointer}";

const CHECKBOX_CSS: &str = "\
.bu-checkbox{display:inline-flex;align-items:center;gap:0.5rem;cursor:pointer;font-size:0.85rem;color:#ccc}\
.bu-checkbox input{accent-color:#f97316;width:18px;height:18px;cursor:pointer}";

const CARD_CSS: &str = "\
.bu-card{background:#141414;border:1px solid #333;border-radius:12px;overflow:hidden}\
.bu-card-header{padding:1rem 1.25rem;font-weight:600;font-size:0.95rem;border-bottom:1px solid #222}\
.bu-card-body{padding:1.25rem}\
.bu-card-footer{padding:0.75rem 1.25rem;border-top:1px solid #222;display:flex;gap:0.5rem;justify-content:flex-end}";

const ALERT_CSS: &str = "\
.bu-alert{display:flex;align-items:center;gap:0.6rem;padding:0.75rem 1rem;border-radius:8px;font-size:0.85rem;position:relative}\
.bu-alert-icon{font-size:1rem;flex-shrink:0}\
.bu-alert-success{background:rgba(34,197,94,.1);border:1px solid rgba(34,197,94,.3);color:#86efac}\
.bu-alert-warning{background:rgba(234,179,8,.1);border:1px solid rgba(234,179,8,.3);color:#fde047}\
.bu-alert-error{background:rgba(239,68,68,.1);border:1px solid rgba(239,68,68,.3);color:#fca5a5}\
.bu-alert-info{background:rgba(59,130,246,.1);border:1px solid rgba(59,130,246,.3);color:#93c5fd}\
.bu-alert-close{position:absolute;right:0.5rem;top:50%;transform:translateY(-50%);background:none;border:none;\
color:inherit;font-size:1.2rem;cursor:pointer;opacity:.6;padding:0.25rem}\
.bu-alert-close:hover{opacity:1}";

const MODAL_CSS: &str = "\
.bu-modal-overlay{position:fixed;inset:0;background:rgba(0,0,0,.7);display:flex;align-items:center;\
justify-content:center;z-index:1000;backdrop-filter:blur(4px)}\
.bu-modal{background:#141414;border:1px solid #333;border-radius:14px;max-width:480px;width:92%;max-height:85vh;overflow-y:auto}\
.bu-modal-header{display:flex;justify-content:space-between;align-items:center;padding:1rem 1.25rem;border-bottom:1px solid #222}\
.bu-modal-header h3{font-size:1.05rem}\
.bu-modal-close{background:none;border:none;color:#888;font-size:1.3rem;cursor:pointer;padding:0.25rem}\
.bu-modal-close:hover{color:#fff}\
.bu-modal-body{padding:1.25rem}";

const SPINNER_CSS: &str = "\
.bu-spinner{width:28px;height:28px;border:3px solid #333;border-top-color:#f97316;\
border-radius:50%;animation:bu-spin .6s linear infinite}\
.bu-spinner-wrap{display:flex;flex-direction:column;align-items:center;gap:0.75rem}\
.bu-spinner-text{font-size:0.85rem;color:#888}";

const PROGRESS_CSS: &str = "\
.bu-progress{width:100%;height:8px;background:#1e1e1e;border-radius:4px;overflow:hidden}\
.bu-progress-bar{height:100%;background:linear-gradient(90deg,#f97316,#ef4444);border-radius:4px;\
transition:width .3s ease}";

const TABS_CSS: &str = "\
.bu-tabs{display:flex;flex-direction:column}\
.bu-tabs-nav{display:flex;border-bottom:1px solid #333;gap:0}\
.bu-tab-btn{padding:0.6rem 1.2rem;background:none;border:none;border-bottom:2px solid transparent;\
color:#888;font-size:0.85rem;cursor:pointer;transition:all .15s;font-family:inherit}\
.bu-tab-btn:hover{color:#e0e0e0}\
.bu-tab-active{color:#f97316!important;border-bottom-color:#f97316}\
.bu-tab-panel{padding:1rem 0}";

const BADGE_CSS: &str = "\
.bu-badge{display:inline-flex;padding:0.15rem 0.55rem;border-radius:999px;font-size:0.7rem;font-weight:600}\
.bu-badge-success{background:rgba(34,197,94,.15);color:#86efac}\
.bu-badge-warning{background:rgba(234,179,8,.15);color:#fde047}\
.bu-badge-error{background:rgba(239,68,68,.15);color:#fca5a5}\
.bu-badge-info{background:rgba(59,130,246,.15);color:#93c5fd}";

const SKELETON_CSS: &str = "\
.bu-skeleton{background:linear-gradient(90deg,#1e1e1e 25%,#2a2a2a 50%,#1e1e1e 75%);\
background-size:200% 100%;animation:bu-shimmer 1.5s infinite;border-radius:6px}\
@keyframes bu-shimmer{0%{background-position:200% 0}100%{background-position:-200% 0}}";

// ═══════════════════════════════════════════════════════════════════════════
// 15. Scroll to Top
// ═══════════════════════════════════════════════════════════════════════════

/// A floating "scroll to top" button that appears after scrolling down.
/// Shows after scrolling past `threshold_px` (default 300).
///
/// ```ignore
/// // Add to your app — it positions itself fixed in the bottom-right corner
/// append_node(&body, &scroll_to_top(300));
/// ```
pub fn scroll_to_top(threshold_px: i32) -> web_sys::Element {
    inject_css("scroll-top", SCROLL_TOP_CSS);
    let btn = create_element("button");
    set_attribute(&btn, "class", "bu-scroll-top");
    btn.set_attribute("aria-label", "Scroll to top").ok();
    set_inner_html(&btn, "&#8679;"); // ⇧ arrow
    set_style(&btn, "display", "none");

    // Show/hide based on scroll position
    let btn_ref = btn.clone();
    on_window_event("scroll", move |_| {
        let y = web_sys::window().unwrap().scroll_y().unwrap_or(0.0);
        if y > threshold_px as f64 {
            set_style(&btn_ref, "display", "flex");
        } else {
            set_style(&btn_ref, "display", "none");
        }
    });

    // Scroll to top on click
    add_event_listener(&btn, "click", move |_| {
        web_sys::window().unwrap().scroll_to_with_x_and_y(0.0, 0.0);
    });

    btn
}

const SCROLL_TOP_CSS: &str = "\
.bu-scroll-top{position:fixed;bottom:2rem;right:2rem;width:44px;height:44px;\
background:linear-gradient(135deg,#f97316,#ef4444);color:#fff;border:none;border-radius:50%;\
font-size:1.3rem;cursor:pointer;display:flex;align-items:center;justify-content:center;\
box-shadow:0 4px 16px rgba(249,115,22,.35);transition:all .2s;z-index:90;opacity:.9}\
.bu-scroll-top:hover{transform:translateY(-3px);opacity:1;box-shadow:0 6px 20px rgba(249,115,22,.5)}\
.bu-scroll-top:active{transform:translateY(0)}";

// ═══════════════════════════════════════════════════════════════════════════
// 16. HStack — horizontal flex container
// ═══════════════════════════════════════════════════════════════════════════

/// Horizontal flex row with configurable gap. Children wrap by default.
///
/// ```ignore
/// let row = hstack("1rem", vec![button("A").build(), button("B").build()]);
/// ```
pub fn hstack(gap: &str, children: Vec<web_sys::Element>) -> web_sys::Element {
    inject_css("hstack", HSTACK_CSS);
    let el = create_element("div");
    set_attribute(&el, "class", "bu-hstack");
    set_style(&el, "gap", gap);
    for child in children {
        append_node(&el, &child);
    }
    el
}

const HSTACK_CSS: &str = ".bu-hstack{display:flex;align-items:center;flex-wrap:wrap}";

// ═══════════════════════════════════════════════════════════════════════════
// 17. VStack — vertical flex container
// ═══════════════════════════════════════════════════════════════════════════

/// Vertical flex column with configurable gap.
///
/// ```ignore
/// let col = vstack("0.5rem", vec![text_input("Name").build(), text_input("Email").build()]);
/// ```
pub fn vstack(gap: &str, children: Vec<web_sys::Element>) -> web_sys::Element {
    inject_css("vstack", VSTACK_CSS);
    let el = create_element("div");
    set_attribute(&el, "class", "bu-vstack");
    set_style(&el, "gap", gap);
    for child in children {
        append_node(&el, &child);
    }
    el
}

const VSTACK_CSS: &str = ".bu-vstack{display:flex;flex-direction:column}";

// ═══════════════════════════════════════════════════════════════════════════
// 18. Center — center content both axes
// ═══════════════════════════════════════════════════════════════════════════

/// Centers its child horizontally and vertically.
pub fn center(child: web_sys::Element) -> web_sys::Element {
    inject_css("center", CENTER_CSS);
    let el = create_element("div");
    set_attribute(&el, "class", "bu-center");
    append_node(&el, &child);
    el
}

const CENTER_CSS: &str = ".bu-center{display:flex;align-items:center;justify-content:center}";

// ═══════════════════════════════════════════════════════════════════════════
// 19. Spacer — flex:1 space
// ═══════════════════════════════════════════════════════════════════════════

/// Flexible spacer that fills available space in a flex container.
pub fn spacer() -> web_sys::Element {
    inject_css("spacer", SPACER_CSS);
    let el = create_element("div");
    set_attribute(&el, "class", "bu-spacer");
    el
}

const SPACER_CSS: &str = ".bu-spacer{flex:1}";

// ═══════════════════════════════════════════════════════════════════════════
// 20. Container — centered max-width wrapper
// ═══════════════════════════════════════════════════════════════════════════

/// Centered max-width (1200 px) container with horizontal padding.
pub fn container(child: web_sys::Element) -> web_sys::Element {
    inject_css("container", CONTAINER_CSS);
    let el = create_element("div");
    set_attribute(&el, "class", "bu-container");
    append_node(&el, &child);
    el
}

const CONTAINER_CSS: &str = ".bu-container{max-width:1200px;margin:0 auto;padding:0 1rem;width:100%}";

// ═══════════════════════════════════════════════════════════════════════════
// 21. Grid — CSS grid layout
// ═══════════════════════════════════════════════════════════════════════════

/// CSS grid with `cols` equal-width columns and a configurable gap.
pub fn grid(cols: u32, gap: &str, children: Vec<web_sys::Element>) -> web_sys::Element {
    inject_css("grid", GRID_CSS);
    let el = create_element("div");
    set_attribute(&el, "class", "bu-grid");
    set_style(&el, "grid-template-columns", &format!("repeat({},1fr)", cols));
    set_style(&el, "gap", gap);
    for child in children {
        append_node(&el, &child);
    }
    el
}

const GRID_CSS: &str = ".bu-grid{display:grid}";

// ═══════════════════════════════════════════════════════════════════════════
// 22. Avatar — circular avatar with initials or image
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Clone, Copy)]
pub enum AvatarSize { Small, Medium, Large }

pub struct AvatarBuilder {
    name: String,
    size: AvatarSize,
    src: Option<String>,
}

/// Create an avatar with initials derived from `name`.
/// Use `.src(url)` to show an image instead, and `.size(AvatarSize)` to pick a size.
///
/// ```ignore
/// let av = avatar("Jane Doe").size(AvatarSize::Large).build();
/// let av_img = avatar("Bot").src("/img/bot.png").build();
/// ```
pub fn avatar(name: &str) -> AvatarBuilder {
    AvatarBuilder { name: name.to_string(), size: AvatarSize::Medium, src: None }
}

impl AvatarBuilder {
    pub fn size(mut self, s: AvatarSize) -> Self { self.size = s; self }
    pub fn src(mut self, url: &str) -> Self { self.src = Some(url.to_string()); self }

    pub fn build(self) -> web_sys::Element {
        inject_css("avatar", AVATAR_CSS);
        let el = create_element("div");
        let size_cls = match self.size {
            AvatarSize::Small => "bu-avatar-sm",
            AvatarSize::Medium => "bu-avatar-md",
            AvatarSize::Large => "bu-avatar-lg",
        };
        set_attribute(&el, "class", &format!("bu-avatar {}", size_cls));
        el.set_attribute("role", "img").ok();
        el.set_attribute("aria-label", &self.name).ok();

        if let Some(url) = &self.src {
            let img = create_element("img");
            set_attribute(&img, "src", url);
            set_attribute(&img, "alt", &self.name);
            set_style(&img, "width", "100%");
            set_style(&img, "height", "100%");
            set_style(&img, "border-radius", "50%");
            set_style(&img, "object-fit", "cover");
            append_node(&el, &img);
        } else {
            let initials: String = self.name
                .split_whitespace()
                .filter_map(|w| w.chars().next())
                .take(2)
                .collect::<String>()
                .to_uppercase();
            append_text(&el, &initials);
        }
        el
    }
}

const AVATAR_CSS: &str = "\
.bu-avatar{display:inline-flex;align-items:center;justify-content:center;border-radius:50%;overflow:hidden;\
font-weight:600;color:#fff;background:#f97316}\
.bu-avatar-sm{width:28px;height:28px;font-size:0.65rem}\
.bu-avatar-md{width:40px;height:40px;font-size:0.8rem}\
.bu-avatar-lg{width:56px;height:56px;font-size:1.1rem}";

// ═══════════════════════════════════════════════════════════════════════════
// 23. Stat — big statistic value with label
// ═══════════════════════════════════════════════════════════════════════════

/// Large gradient-styled statistic number with a muted description below.
pub fn stat(value: &str, label: &str) -> web_sys::Element {
    inject_css("stat", STAT_CSS);
    let el = create_element("div");
    set_attribute(&el, "class", "bu-stat");
    let v = create_element("div");
    set_attribute(&v, "class", "bu-stat-value");
    append_text(&v, value);
    append_node(&el, &v);
    let l = create_element("div");
    set_attribute(&l, "class", "bu-stat-label");
    append_text(&l, label);
    append_node(&el, &l);
    el
}

const STAT_CSS: &str = "\
.bu-stat{display:flex;flex-direction:column;gap:0.25rem}\
.bu-stat-value{font-size:2rem;font-weight:700;background:linear-gradient(135deg,#f97316,#ef4444);\
-webkit-background-clip:text;-webkit-text-fill-color:transparent;background-clip:text}\
.bu-stat-label{font-size:0.85rem;color:#888}";

// ═══════════════════════════════════════════════════════════════════════════
// 24. Tag — small pill with optional remove
// ═══════════════════════════════════════════════════════════════════════════

pub struct TagBuilder {
    text: String,
    severity: Severity,
    removable: Option<Signal<bool>>,
}

/// Small pill tag with an optional × removal button.
///
/// ```ignore
/// let vis = signal(true);
/// let t = tag("Rust").variant(Severity::Success).removable(vis).build();
/// ```
pub fn tag(text: &str) -> TagBuilder {
    TagBuilder { text: text.to_string(), severity: Severity::Info, removable: None }
}

impl TagBuilder {
    pub fn variant(mut self, s: Severity) -> Self { self.severity = s; self }
    pub fn removable(mut self, s: Signal<bool>) -> Self { self.removable = Some(s); self }

    pub fn build(self) -> web_sys::Element {
        inject_css("tag", TAG_CSS);
        let el = create_element("span");
        let sev = match self.severity {
            Severity::Success => "success", Severity::Warning => "warning",
            Severity::Error => "error", Severity::Info => "info",
        };
        set_attribute(&el, "class", &format!("bu-tag bu-tag-{}", sev));
        append_text(&el, &self.text);
        if let Some(vis) = self.removable {
            let btn = create_element("button");
            set_attribute(&btn, "class", "bu-tag-remove");
            btn.set_attribute("aria-label", "Remove tag").ok();
            append_text(&btn, "×");
            add_event_listener(&btn, "click", move |_| { vis.set(false); });
            append_node(&el, &btn);
            let el_ref = el.clone();
            create_effect(move || {
                if !vis.get() { set_style(&el_ref, "display", "none"); }
                else { set_style(&el_ref, "display", ""); }
            });
        }
        el
    }
}

const TAG_CSS: &str = "\
.bu-tag{display:inline-flex;align-items:center;gap:0.3rem;padding:0.15rem 0.6rem;border-radius:999px;\
font-size:0.75rem;font-weight:500}\
.bu-tag-success{background:rgba(34,197,94,.15);color:#86efac}\
.bu-tag-warning{background:rgba(234,179,8,.15);color:#fde047}\
.bu-tag-error{background:rgba(239,68,68,.15);color:#fca5a5}\
.bu-tag-info{background:rgba(59,130,246,.15);color:#93c5fd}\
.bu-tag-remove{background:none;border:none;color:inherit;cursor:pointer;font-size:0.9rem;padding:0;\
line-height:1;opacity:.7}\
.bu-tag-remove:hover{opacity:1}";

// ═══════════════════════════════════════════════════════════════════════════
// 25. CodeBlock — syntax-highlighted code display
// ═══════════════════════════════════════════════════════════════════════════

/// Dark monospace code display block with rounded corners.
pub fn code_block(code: &str) -> web_sys::Element {
    inject_css("codeblock", CODE_BLOCK_CSS);
    let pre = create_element("pre");
    set_attribute(&pre, "class", "bu-code-block");
    let code_el = create_element("code");
    append_text(&code_el, code);
    append_node(&pre, &code_el);
    pre
}

const CODE_BLOCK_CSS: &str = "\
.bu-code-block{background:#0a0a0a;border:1px solid #333;border-radius:8px;padding:1rem;overflow-x:auto;\
font-family:'Fira Code',Consolas,monospace;font-size:0.85rem;color:#e0e0e0;line-height:1.6;margin:0}";

// ═══════════════════════════════════════════════════════════════════════════
// 26. Kbd — keyboard shortcut display
// ═══════════════════════════════════════════════════════════════════════════

/// Renders each key in `keys` (separated by `+`) in a styled keyboard-key box.
///
/// ```ignore
/// let shortcut = kbd("Ctrl+S");
/// ```
pub fn kbd(keys: &str) -> web_sys::Element {
    inject_css("kbd", KBD_CSS);
    let wrap = create_element("span");
    set_attribute(&wrap, "class", "bu-kbd-wrap");
    for (i, key) in keys.split('+').enumerate() {
        if i > 0 {
            let sep = create_element("span");
            set_attribute(&sep, "class", "bu-kbd-sep");
            append_text(&sep, "+");
            append_node(&wrap, &sep);
        }
        let k = create_element("kbd");
        set_attribute(&k, "class", "bu-kbd");
        append_text(&k, key.trim());
        append_node(&wrap, &k);
    }
    wrap
}

const KBD_CSS: &str = "\
.bu-kbd-wrap{display:inline-flex;align-items:center;gap:0.2rem}\
.bu-kbd{display:inline-block;padding:0.15rem 0.45rem;background:#1e1e1e;border:1px solid #444;\
border-radius:4px;font-family:inherit;font-size:0.75rem;color:#ccc;box-shadow:0 2px 0 #333;line-height:1.4}\
.bu-kbd-sep{color:#666;font-size:0.7rem}";

// ═══════════════════════════════════════════════════════════════════════════
// 27. Tooltip — CSS-only hover tooltip
// ═══════════════════════════════════════════════════════════════════════════

/// Wraps `target` in a tooltip container that shows `text` above on hover.
pub fn tooltip(target: web_sys::Element, text: &str) -> web_sys::Element {
    inject_css("tooltip", TOOLTIP_CSS);
    let wrap = create_element("span");
    set_attribute(&wrap, "class", "bu-tooltip-wrap");
    wrap.set_attribute("data-tooltip", text).ok();
    append_node(&wrap, &target);
    wrap
}

const TOOLTIP_CSS: &str = "\
.bu-tooltip-wrap{position:relative;display:inline-block}\
.bu-tooltip-wrap::after{content:attr(data-tooltip);position:absolute;bottom:100%;left:50%;\
transform:translateX(-50%);background:#1e1e1e;color:#e0e0e0;padding:0.3rem 0.6rem;border-radius:6px;\
font-size:0.75rem;white-space:nowrap;pointer-events:none;opacity:0;transition:opacity .15s;\
border:1px solid #444;margin-bottom:6px;z-index:100}\
.bu-tooltip-wrap:hover::after{opacity:1}";

// ═══════════════════════════════════════════════════════════════════════════
// 28. Timeline — vertical timeline
// ═══════════════════════════════════════════════════════════════════════════

/// Vertical timeline with accent-coloured dots and a connecting line.
///
/// ```ignore
/// let tl = timeline(&[("Step 1", "Started"), ("Step 2", "In progress")]);
/// ```
pub fn timeline(items: &[(&str, &str)]) -> web_sys::Element {
    inject_css("timeline", TIMELINE_CSS);
    let el = create_element("div");
    set_attribute(&el, "class", "bu-timeline");
    el.set_attribute("role", "list").ok();
    for &(title, desc) in items {
        let item = create_element("div");
        set_attribute(&item, "class", "bu-timeline-item");
        item.set_attribute("role", "listitem").ok();
        let dot = create_element("div");
        set_attribute(&dot, "class", "bu-timeline-dot");
        append_node(&item, &dot);
        let content = create_element("div");
        set_attribute(&content, "class", "bu-timeline-content");
        let t = create_element("div");
        set_attribute(&t, "class", "bu-timeline-title");
        append_text(&t, title);
        append_node(&content, &t);
        let d = create_element("div");
        set_attribute(&d, "class", "bu-timeline-desc");
        append_text(&d, desc);
        append_node(&content, &d);
        append_node(&item, &content);
        append_node(&el, &item);
    }
    el
}

const TIMELINE_CSS: &str = "\
.bu-timeline{display:flex;flex-direction:column;padding-left:1rem}\
.bu-timeline-item{display:flex;gap:1rem;padding-bottom:1.5rem;position:relative;\
border-left:2px solid #333;padding-left:1.5rem}\
.bu-timeline-dot{position:absolute;left:-6px;top:0;width:10px;height:10px;border-radius:50%;background:#f97316}\
.bu-timeline-content{display:flex;flex-direction:column;gap:0.25rem}\
.bu-timeline-title{font-weight:600;font-size:0.9rem;color:#e0e0e0}\
.bu-timeline-desc{font-size:0.8rem;color:#888}";

// ═══════════════════════════════════════════════════════════════════════════
// 29. DataTable — simple data table
// ═══════════════════════════════════════════════════════════════════════════

/// Striped, horizontally-scrollable data table.
pub fn data_table(headers: &[&str], rows: &[Vec<String>]) -> web_sys::Element {
    inject_css("datatable", DATA_TABLE_CSS);
    let wrap = create_element("div");
    set_attribute(&wrap, "class", "bu-table-wrap");
    wrap.set_attribute("role", "table").ok();
    let table = create_element("table");
    set_attribute(&table, "class", "bu-table");
    let thead = create_element("thead");
    let tr = create_element("tr");
    for &h in headers {
        let th = create_element("th");
        append_text(&th, h);
        append_node(&tr, &th);
    }
    append_node(&thead, &tr);
    append_node(&table, &thead);
    let tbody = create_element("tbody");
    for row in rows {
        let tr = create_element("tr");
        for cell in row {
            let td = create_element("td");
            append_text(&td, cell);
            append_node(&tr, &td);
        }
        append_node(&tbody, &tr);
    }
    append_node(&table, &tbody);
    append_node(&wrap, &table);
    wrap
}

const DATA_TABLE_CSS: &str = "\
.bu-table-wrap{overflow-x:auto;width:100%}\
.bu-table{width:100%;border-collapse:collapse;font-size:0.85rem}\
.bu-table th{text-align:left;padding:0.6rem 0.75rem;border-bottom:2px solid #333;font-weight:600;color:#ccc}\
.bu-table td{padding:0.5rem 0.75rem;border-bottom:1px solid #222;color:#e0e0e0}\
.bu-table tbody tr:nth-child(even){background:#0f0f0f}\
.bu-table tbody tr:hover{background:#1e1e1e}";

// ═══════════════════════════════════════════════════════════════════════════
// 30. Breadcrumb — navigation breadcrumb trail
// ═══════════════════════════════════════════════════════════════════════════

/// Breadcrumb navigation. `items` are `(label, href)` pairs;
/// the last item is rendered as plain text (current page).
pub fn breadcrumb(items: &[(&str, &str)]) -> web_sys::Element {
    inject_css("breadcrumb", BREADCRUMB_CSS);
    let nav = create_element("nav");
    set_attribute(&nav, "class", "bu-breadcrumb");
    nav.set_attribute("aria-label", "Breadcrumb").ok();
    let ol = create_element("ol");
    set_attribute(&ol, "class", "bu-breadcrumb-list");
    for (i, &(label, href)) in items.iter().enumerate() {
        let li = create_element("li");
        set_attribute(&li, "class", "bu-breadcrumb-item");
        if i == items.len() - 1 {
            let span = create_element("span");
            set_attribute(&span, "class", "bu-breadcrumb-current");
            span.set_attribute("aria-current", "page").ok();
            append_text(&span, label);
            append_node(&li, &span);
        } else {
            let a = create_element("a");
            set_attribute(&a, "href", href);
            set_attribute(&a, "class", "bu-breadcrumb-link");
            append_text(&a, label);
            append_node(&li, &a);
            let sep = create_element("span");
            set_attribute(&sep, "class", "bu-breadcrumb-sep");
            append_text(&sep, "/");
            append_node(&li, &sep);
        }
        append_node(&ol, &li);
    }
    append_node(&nav, &ol);
    nav
}

const BREADCRUMB_CSS: &str = "\
.bu-breadcrumb-list{display:flex;align-items:center;gap:0.4rem;list-style:none;padding:0;margin:0;font-size:0.85rem}\
.bu-breadcrumb-item{display:flex;align-items:center;gap:0.4rem}\
.bu-breadcrumb-link{color:#888;text-decoration:none;transition:color .15s}\
.bu-breadcrumb-link:hover{color:#f97316}\
.bu-breadcrumb-current{color:#e0e0e0;font-weight:500}\
.bu-breadcrumb-sep{color:#555}";

// ═══════════════════════════════════════════════════════════════════════════
// 31. Pagination — page navigation
// ═══════════════════════════════════════════════════════════════════════════

/// Reactive page navigation with Previous / Next and numbered page buttons.
pub fn pagination(total_pages: Signal<u32>, current: Signal<u32>) -> web_sys::Element {
    inject_css("pagination", PAGINATION_CSS);
    let wrap = create_element("nav");
    set_attribute(&wrap, "class", "bu-pagination");
    wrap.set_attribute("aria-label", "Pagination").ok();
    let wrap_ref = wrap.clone();
    create_effect(move || {
        clear_children(&wrap_ref);
        let total = total_pages.get();
        let cur = current.get();

        let prev = create_element("button");
        set_attribute(&prev, "class", "bu-page-btn");
        append_text(&prev, "\u{2039}"); // ‹
        if cur <= 1 { set_attribute(&prev, "disabled", ""); }
        let c = current;
        add_event_listener(&prev, "click", move |_| { if c.get() > 1 { c.set(c.get() - 1); } });
        append_node(&wrap_ref, &prev);

        for p in 1..=total {
            let btn = create_element("button");
            let cls = if p == cur { "bu-page-btn bu-page-active" } else { "bu-page-btn" };
            set_attribute(&btn, "class", cls);
            append_text(&btn, &p.to_string());
            let c = current;
            add_event_listener(&btn, "click", move |_| { c.set(p); });
            append_node(&wrap_ref, &btn);
        }

        let next = create_element("button");
        set_attribute(&next, "class", "bu-page-btn");
        append_text(&next, "\u{203a}"); // ›
        if cur >= total { set_attribute(&next, "disabled", ""); }
        let c = current;
        let tp = total_pages;
        add_event_listener(&next, "click", move |_| { if c.get() < tp.get() { c.set(c.get() + 1); } });
        append_node(&wrap_ref, &next);
    });
    wrap
}

const PAGINATION_CSS: &str = "\
.bu-pagination{display:flex;align-items:center;gap:0.25rem}\
.bu-page-btn{background:#1e1e1e;border:1px solid #333;border-radius:6px;color:#ccc;padding:0.3rem 0.6rem;\
font-size:0.8rem;cursor:pointer;transition:all .15s;font-family:inherit}\
.bu-page-btn:hover:not(:disabled){border-color:#f97316;color:#f97316}\
.bu-page-btn:disabled{opacity:.4;cursor:not-allowed}\
.bu-page-active{background:#f97316;border-color:#f97316;color:#fff;font-weight:600}";

// ═══════════════════════════════════════════════════════════════════════════
// 32. Dropdown — click-to-toggle select menu
// ═══════════════════════════════════════════════════════════════════════════

/// A simple dropdown that updates `selected` when an item is picked.
///
/// ```ignore
/// let sel = signal(String::new());
/// let dd = dropdown("Pick one", &["Alpha", "Beta", "Gamma"], sel);
/// ```
pub fn dropdown(trigger_text: &str, items: &[&str], selected: Signal<String>) -> web_sys::Element {
    inject_css("dropdown", DROPDOWN_CSS);
    let open = signal(false);
    let wrap = create_element("div");
    set_attribute(&wrap, "class", "bu-dropdown");

    let trigger = create_element("button");
    set_attribute(&trigger, "class", "bu-dropdown-trigger");
    let trigger_ref = trigger.clone();
    let label_text = trigger_text.to_string();
    create_effect(move || {
        let val = selected.get();
        let display = if val.is_empty() { label_text.clone() } else { val };
        clear_children(&trigger_ref);
        append_text(&trigger_ref, &display);
        let arrow = create_element("span");
        set_attribute(&arrow, "class", "bu-dropdown-arrow");
        append_text(&arrow, "\u{25be}"); // ▾
        append_node(&trigger_ref, &arrow);
    });

    let o = open;
    add_event_listener(&trigger, "click", move |_| { o.set(!o.get()); });
    append_node(&wrap, &trigger);

    let menu = create_element("div");
    set_attribute(&menu, "class", "bu-dropdown-menu");
    for item in items {
        let opt = create_element("div");
        set_attribute(&opt, "class", "bu-dropdown-item");
        append_text(&opt, item);
        let s = selected;
        let o = open;
        let val = item.to_string();
        add_event_listener(&opt, "click", move |_| { s.set(val.clone()); o.set(false); });
        append_node(&menu, &opt);
    }
    append_node(&wrap, &menu);

    let menu_ref = menu.clone();
    create_effect(move || {
        if open.get() { set_style(&menu_ref, "display", "block"); }
        else { set_style(&menu_ref, "display", "none"); }
    });
    set_style(&menu, "display", "none");

    // Close on outside click
    let wrap_ref = wrap.clone();
    let o = open;
    use_click_outside(&wrap_ref, move || { o.set(false); });

    wrap
}

const DROPDOWN_CSS: &str = "\
.bu-dropdown{position:relative;display:inline-block}\
.bu-dropdown-trigger{display:inline-flex;align-items:center;gap:0.5rem;padding:0.5rem 0.9rem;\
background:#1e1e1e;border:1px solid #444;border-radius:8px;color:#e0e0e0;font-size:0.85rem;\
cursor:pointer;font-family:inherit}\
.bu-dropdown-trigger:hover{border-color:#f97316}\
.bu-dropdown-arrow{font-size:0.7rem;color:#888}\
.bu-dropdown-menu{position:absolute;top:100%;left:0;margin-top:4px;background:#141414;border:1px solid #333;\
border-radius:8px;min-width:160px;z-index:50;box-shadow:0 8px 24px rgba(0,0,0,.4);overflow:hidden}\
.bu-dropdown-item{padding:0.5rem 0.9rem;font-size:0.85rem;color:#ccc;cursor:pointer;transition:background .1s}\
.bu-dropdown-item:hover{background:#1e1e1e;color:#f97316}";

// ═══════════════════════════════════════════════════════════════════════════
// 33. Navbar — top navigation bar
// ═══════════════════════════════════════════════════════════════════════════

/// Reusable top navigation bar with a brand label and arbitrary items.
pub fn navbar(brand: &str, items: Vec<web_sys::Element>) -> web_sys::Element {
    inject_css("navbar", NAVBAR_CSS);
    let nav = create_element("nav");
    set_attribute(&nav, "class", "bu-navbar");
    nav.set_attribute("role", "navigation").ok();
    let brand_el = create_element("div");
    set_attribute(&brand_el, "class", "bu-navbar-brand");
    append_text(&brand_el, brand);
    append_node(&nav, &brand_el);
    let links = create_element("div");
    set_attribute(&links, "class", "bu-navbar-items");
    for item in items {
        append_node(&links, &item);
    }
    append_node(&nav, &links);
    nav
}

const NAVBAR_CSS: &str = "\
.bu-navbar{display:flex;align-items:center;justify-content:space-between;padding:0.75rem 1.5rem;\
background:#0a0a0a;border-bottom:1px solid #222}\
.bu-navbar-brand{font-size:1.1rem;font-weight:700;color:#e0e0e0}\
.bu-navbar-items{display:flex;align-items:center;gap:1rem}";

// ═══════════════════════════════════════════════════════════════════════════
// 34. Toast — auto-dismissing notification
// ═══════════════════════════════════════════════════════════════════════════

pub struct ToastBuilder {
    message: String,
    severity: Severity,
    duration_ms: u32,
}

/// Auto-dismissing toast notification appended to `<body>`.
///
/// ```ignore
/// toast("Saved!").severity(Severity::Success).duration_ms(2000).show();
/// ```
pub fn toast(message: &str) -> ToastBuilder {
    ToastBuilder { message: message.to_string(), severity: Severity::Info, duration_ms: 3000 }
}

impl ToastBuilder {
    pub fn severity(mut self, s: Severity) -> Self { self.severity = s; self }
    pub fn duration_ms(mut self, ms: u32) -> Self { self.duration_ms = ms; self }

    pub fn show(self) -> web_sys::Element {
        inject_css("toast", TOAST_CSS);
        let el = create_element("div");
        let sev = match self.severity {
            Severity::Success => "success", Severity::Warning => "warning",
            Severity::Error => "error", Severity::Info => "info",
        };
        set_attribute(&el, "class", &format!("bu-toast bu-toast-{}", sev));
        el.set_attribute("role", "status").ok();
        append_text(&el, &self.message);

        if let Some(b) = web_sys::window().unwrap().document().unwrap().body() {
            b.append_child(&el).ok();
        }

        let el_ref = el.clone();
        let ms = self.duration_ms as i32;
        set_timeout(move || {
            if let Some(parent) = el_ref.parent_node() {
                parent.remove_child(&el_ref).ok();
            }
        }, ms);

        el
    }
}

const TOAST_CSS: &str = "\
.bu-toast{position:fixed;top:1.5rem;right:1.5rem;padding:0.75rem 1.25rem;border-radius:8px;font-size:0.85rem;\
z-index:2000;animation:bu-toast-in .3s ease;box-shadow:0 8px 24px rgba(0,0,0,.4)}\
@keyframes bu-toast-in{from{transform:translateX(100%);opacity:0}to{transform:translateX(0);opacity:1}}\
.bu-toast-success{background:rgba(34,197,94,.15);border:1px solid rgba(34,197,94,.3);color:#86efac}\
.bu-toast-warning{background:rgba(234,179,8,.15);border:1px solid rgba(234,179,8,.3);color:#fde047}\
.bu-toast-error{background:rgba(239,68,68,.15);border:1px solid rgba(239,68,68,.3);color:#fca5a5}\
.bu-toast-info{background:rgba(59,130,246,.15);border:1px solid rgba(59,130,246,.3);color:#93c5fd}";

// ═══════════════════════════════════════════════════════════════════════════
// 35. ConfirmDialog — modal confirmation
// ═══════════════════════════════════════════════════════════════════════════

/// Modal dialog with Cancel / Confirm buttons.
/// `on_confirm` fires once and the dialog closes automatically.
pub fn confirm_dialog(
    title: &str,
    message: &str,
    on_confirm: impl FnOnce() + 'static,
) -> web_sys::Element {
    inject_css("confirmdlg", CONFIRM_DIALOG_CSS);
    let open = signal(true);

    let overlay = create_element("div");
    set_attribute(&overlay, "class", "bu-confirm-overlay");
    overlay.set_attribute("role", "dialog").ok();
    overlay.set_attribute("aria-modal", "true").ok();

    let dialog = create_element("div");
    set_attribute(&dialog, "class", "bu-confirm-dialog");

    let h3 = create_element("h3");
    set_attribute(&h3, "class", "bu-confirm-title");
    append_text(&h3, title);
    append_node(&dialog, &h3);

    let msg = create_element("p");
    set_attribute(&msg, "class", "bu-confirm-msg");
    append_text(&msg, message);
    append_node(&dialog, &msg);

    let actions = create_element("div");
    set_attribute(&actions, "class", "bu-confirm-actions");

    let cancel = create_element("button");
    set_attribute(&cancel, "class", "bu-confirm-cancel");
    append_text(&cancel, "Cancel");
    let o = open;
    add_event_listener(&cancel, "click", move |_| { o.set(false); });
    append_node(&actions, &cancel);

    let confirm = create_element("button");
    set_attribute(&confirm, "class", "bu-confirm-ok");
    append_text(&confirm, "Confirm");
    let cb = std::cell::RefCell::new(Some(on_confirm));
    let o = open;
    add_event_listener(&confirm, "click", move |_| {
        if let Some(f) = cb.borrow_mut().take() { f(); }
        o.set(false);
    });
    append_node(&actions, &confirm);

    append_node(&dialog, &actions);
    append_node(&overlay, &dialog);

    let overlay_ref = overlay.clone();
    create_effect(move || {
        if open.get() { set_style(&overlay_ref, "display", "flex"); }
        else { set_style(&overlay_ref, "display", "none"); }
    });

    overlay
}

const CONFIRM_DIALOG_CSS: &str = "\
.bu-confirm-overlay{position:fixed;inset:0;background:rgba(0,0,0,.7);display:flex;align-items:center;\
justify-content:center;z-index:1000;backdrop-filter:blur(4px)}\
.bu-confirm-dialog{background:#141414;border:1px solid #333;border-radius:14px;padding:1.5rem;max-width:400px;width:92%}\
.bu-confirm-title{font-size:1.05rem;margin:0 0 0.5rem;color:#e0e0e0}\
.bu-confirm-msg{font-size:0.85rem;color:#888;margin:0 0 1.25rem}\
.bu-confirm-actions{display:flex;gap:0.5rem;justify-content:flex-end}\
.bu-confirm-cancel{background:#1e1e1e;border:1px solid #444;border-radius:8px;color:#ccc;padding:0.45rem 1rem;\
font-size:0.85rem;cursor:pointer;font-family:inherit}\
.bu-confirm-cancel:hover{border-color:#888}\
.bu-confirm-ok{background:linear-gradient(135deg,#f97316,#ef4444);border:none;border-radius:8px;color:#fff;\
padding:0.45rem 1rem;font-size:0.85rem;cursor:pointer;font-weight:600;font-family:inherit}\
.bu-confirm-ok:hover{opacity:.9}";

// ═══════════════════════════════════════════════════════════════════════════
// 36. LoadingOverlay — full-screen spinner backdrop
// ═══════════════════════════════════════════════════════════════════════════

/// Full-screen loading overlay with a centred spinner and backdrop blur.
/// Show/hide is controlled by `visible`.
pub fn loading_overlay(visible: Signal<bool>) -> web_sys::Element {
    inject_css("loadoverlay", LOADING_OVERLAY_CSS);
    let el = create_element("div");
    set_attribute(&el, "class", "bu-loading-overlay");
    el.set_attribute("role", "status").ok();
    el.set_attribute("aria-label", "Loading").ok();
    let sp = create_element("div");
    set_attribute(&sp, "class", "bu-loading-spinner");
    append_node(&el, &sp);
    let el_ref = el.clone();
    create_effect(move || {
        if visible.get() { set_style(&el_ref, "display", "flex"); }
        else { set_style(&el_ref, "display", "none"); }
    });
    set_style(&el, "display", "none");
    el
}

const LOADING_OVERLAY_CSS: &str = "\
.bu-loading-overlay{position:fixed;inset:0;background:rgba(0,0,0,.6);display:flex;align-items:center;\
justify-content:center;z-index:1500;backdrop-filter:blur(4px)}\
.bu-loading-spinner{width:40px;height:40px;border:4px solid #333;border-top-color:#f97316;\
border-radius:50%;animation:bu-spin .6s linear infinite}";

// ═══════════════════════════════════════════════════════════════════════════
// 37. EmptyState — no-data placeholder
// ═══════════════════════════════════════════════════════════════════════════

/// Centred placeholder with a large icon, title, and description.
pub fn empty_state(title: &str, description: &str, icon: &str) -> web_sys::Element {
    inject_css("emptystate", EMPTY_STATE_CSS);
    let el = create_element("div");
    set_attribute(&el, "class", "bu-empty-state");
    let ic = create_element("div");
    set_attribute(&ic, "class", "bu-empty-icon");
    append_text(&ic, icon);
    append_node(&el, &ic);
    let t = create_element("div");
    set_attribute(&t, "class", "bu-empty-title");
    append_text(&t, title);
    append_node(&el, &t);
    let d = create_element("div");
    set_attribute(&d, "class", "bu-empty-desc");
    append_text(&d, description);
    append_node(&el, &d);
    el
}

const EMPTY_STATE_CSS: &str = "\
.bu-empty-state{display:flex;flex-direction:column;align-items:center;justify-content:center;\
padding:3rem;gap:0.75rem}\
.bu-empty-icon{font-size:3rem;opacity:.5}\
.bu-empty-title{font-size:1.1rem;font-weight:600;color:#ccc}\
.bu-empty-desc{font-size:0.85rem;color:#666;text-align:center;max-width:320px}";

// ═══════════════════════════════════════════════════════════════════════════
// 38. Toggle — on/off switch
// ═══════════════════════════════════════════════════════════════════════════

/// Sliding toggle switch bound to a `Signal<bool>`.
pub fn toggle(label: &str, checked: Signal<bool>) -> web_sys::Element {
    inject_css("toggle", TOGGLE_CSS);
    let wrap = create_element("label");
    set_attribute(&wrap, "class", "bu-toggle");

    let track = create_element("div");
    set_attribute(&track, "class", "bu-toggle-track");
    track.set_attribute("role", "switch").ok();

    let thumb = create_element("div");
    set_attribute(&thumb, "class", "bu-toggle-thumb");
    append_node(&track, &thumb);

    let track_ref = track.clone();
    create_effect(move || {
        let on = checked.get();
        toggle_class(&track_ref, "bu-toggle-on", on);
        track_ref.set_attribute("aria-checked", if on { "true" } else { "false" }).ok();
    });

    let c = checked;
    add_event_listener(&track, "click", move |_| { c.set(!c.get()); });
    append_node(&wrap, &track);

    let lbl = create_element("span");
    set_attribute(&lbl, "class", "bu-toggle-label");
    append_text(&lbl, label);
    append_node(&wrap, &lbl);
    wrap
}

const TOGGLE_CSS: &str = "\
.bu-toggle{display:inline-flex;align-items:center;gap:0.6rem;cursor:pointer;font-size:0.85rem;color:#ccc}\
.bu-toggle-track{width:40px;height:22px;background:#333;border-radius:11px;position:relative;\
transition:background .2s;cursor:pointer;flex-shrink:0}\
.bu-toggle-thumb{position:absolute;top:2px;left:2px;width:18px;height:18px;background:#fff;border-radius:50%;\
transition:transform .2s}\
.bu-toggle-on{background:#f97316}\
.bu-toggle-on .bu-toggle-thumb{transform:translateX(18px)}\
.bu-toggle-label{user-select:none}";

// ═══════════════════════════════════════════════════════════════════════════
// 39. RadioGroup — radio button group
// ═══════════════════════════════════════════════════════════════════════════

/// Vertical radio button group. `options` are `(value, label)` pairs.
pub fn radio_group(name: &str, options: &[(&str, &str)], selected: Signal<String>) -> web_sys::Element {
    inject_css("radiogroup", RADIO_GROUP_CSS);
    let wrap = create_element("div");
    set_attribute(&wrap, "class", "bu-radio-group");
    wrap.set_attribute("role", "radiogroup").ok();
    for &(value, label) in options {
        let lbl = create_element("label");
        set_attribute(&lbl, "class", "bu-radio-item");
        let input = create_element("input");
        set_attribute(&input, "type", "radio");
        set_attribute(&input, "name", name);
        set_attribute(&input, "value", value);
        let inp_ref = input.clone();
        let val = value.to_string();
        create_effect(move || {
            set_property(&inp_ref, "checked", &JsValue::from_bool(selected.get() == val));
        });
        let s = selected;
        let v = value.to_string();
        add_event_listener(&input, "change", move |_| { s.set(v.clone()); });
        append_node(&lbl, &input);
        let span = create_element("span");
        append_text(&span, label);
        append_node(&lbl, &span);
        append_node(&wrap, &lbl);
    }
    wrap
}

const RADIO_GROUP_CSS: &str = "\
.bu-radio-group{display:flex;flex-direction:column;gap:0.5rem}\
.bu-radio-item{display:inline-flex;align-items:center;gap:0.5rem;cursor:pointer;font-size:0.85rem;color:#ccc}\
.bu-radio-item input{accent-color:#f97316;width:16px;height:16px;cursor:pointer}";

// ═══════════════════════════════════════════════════════════════════════════
// 40. Slider — range input
// ═══════════════════════════════════════════════════════════════════════════

/// Accent-coloured range slider with a numeric value readout.
pub fn slider(min: f64, max: f64, step: f64, value: Signal<f64>) -> web_sys::Element {
    inject_css("slider", SLIDER_CSS);
    let wrap = create_element("div");
    set_attribute(&wrap, "class", "bu-slider-wrap");
    let input = create_element("input");
    set_attribute(&input, "type", "range");
    set_attribute(&input, "class", "bu-slider");
    set_attribute(&input, "min", &min.to_string());
    set_attribute(&input, "max", &max.to_string());
    set_attribute(&input, "step", &step.to_string());
    input.set_attribute("aria-valuemin", &min.to_string()).ok();
    input.set_attribute("aria-valuemax", &max.to_string()).ok();

    let display = create_element("span");
    set_attribute(&display, "class", "bu-slider-value");

    let inp_ref = input.clone();
    let disp_ref = display.clone();
    create_effect(move || {
        let v = value.get();
        set_property(&inp_ref, "value", &JsValue::from_str(&v.to_string()));
        inp_ref.set_attribute("aria-valuenow", &v.to_string()).ok();
        clear_children(&disp_ref);
        append_text(&disp_ref, &v.to_string());
    });

    let v = value;
    add_event_listener(&input, "input", move |e| {
        if let Ok(n) = event_target_value(&e).parse::<f64>() { v.set(n); }
    });

    append_node(&wrap, &input);
    append_node(&wrap, &display);
    wrap
}

const SLIDER_CSS: &str = "\
.bu-slider-wrap{display:flex;align-items:center;gap:0.75rem}\
.bu-slider{flex:1;accent-color:#f97316;height:6px;cursor:pointer}\
.bu-slider-value{font-size:0.8rem;color:#ccc;min-width:2.5rem;text-align:right}";

// ═══════════════════════════════════════════════════════════════════════════
// 41. SearchInput — search field with clear button
// ═══════════════════════════════════════════════════════════════════════════

/// Search input with a magnifying-glass icon and clear (×) button.
pub fn search_input(placeholder: &str, value: Signal<String>) -> web_sys::Element {
    inject_css("searchinput", SEARCH_INPUT_CSS);
    let wrap = create_element("div");
    set_attribute(&wrap, "class", "bu-search-wrap");

    let icon = create_element("span");
    set_attribute(&icon, "class", "bu-search-icon");
    set_inner_html(&icon, "&#128269;"); // 🔍
    append_node(&wrap, &icon);

    let input = create_element("input");
    set_attribute(&input, "type", "search");
    set_attribute(&input, "class", "bu-search-input");
    set_attribute(&input, "placeholder", placeholder);
    input.set_attribute("aria-label", placeholder).ok();

    let inp_ref = input.clone();
    create_effect(move || { set_property(&inp_ref, "value", &JsValue::from_str(&value.get())); });

    let clear = create_element("button");
    set_attribute(&clear, "class", "bu-search-clear");
    set_attribute(&clear, "type", "button");
    clear.set_attribute("aria-label", "Clear search").ok();
    append_text(&clear, "\u{00d7}"); // ×

    let v = value;
    add_event_listener(&clear, "click", move |_| { v.set(String::new()); });

    let v = value;
    add_event_listener(&input, "input", move |e| { v.set(event_target_value(&e)); });

    let clear_ref = clear.clone();
    create_effect(move || {
        if value.get().is_empty() { set_style(&clear_ref, "display", "none"); }
        else { set_style(&clear_ref, "display", "block"); }
    });

    append_node(&wrap, &input);
    append_node(&wrap, &clear);
    wrap
}

const SEARCH_INPUT_CSS: &str = "\
.bu-search-wrap{display:flex;align-items:center;gap:0.5rem;background:#0a0a0a;border:1px solid #444;\
border-radius:8px;padding:0.4rem 0.75rem;transition:border-color .15s}\
.bu-search-wrap:focus-within{border-color:#f97316}\
.bu-search-icon{font-size:0.85rem;flex-shrink:0}\
.bu-search-input{background:transparent;border:none;color:#e0e0e0;font-size:0.85rem;outline:none;\
font-family:inherit;flex:1;width:100%}\
.bu-search-clear{background:none;border:none;color:#888;font-size:1.1rem;cursor:pointer;padding:0;line-height:1}\
.bu-search-clear:hover{color:#e0e0e0}";

// ═══════════════════════════════════════════════════════════════════════════
// 42. PasswordInput — input with show/hide toggle
// ═══════════════════════════════════════════════════════════════════════════

/// Password input that toggles between masked and clear-text display.
pub fn password_input(label: &str, value: Signal<String>) -> web_sys::Element {
    inject_css("passinput", PASSWORD_INPUT_CSS);
    inject_css("input", INPUT_CSS);
    let wrap = create_element("div");
    set_attribute(&wrap, "class", "bu-field");

    if !label.is_empty() {
        let lbl = create_element("label");
        set_attribute(&lbl, "class", "bu-label");
        append_text(&lbl, label);
        append_node(&wrap, &lbl);
    }

    let input_wrap = create_element("div");
    set_attribute(&input_wrap, "class", "bu-pass-wrap");

    let input = create_element("input");
    set_attribute(&input, "type", "password");
    set_attribute(&input, "class", "bu-input bu-pass-input");

    let inp_ref = input.clone();
    create_effect(move || { set_property(&inp_ref, "value", &JsValue::from_str(&value.get())); });
    let v = value;
    add_event_listener(&input, "input", move |e| { v.set(event_target_value(&e)); });

    let toggle_btn = create_element("button");
    set_attribute(&toggle_btn, "class", "bu-pass-toggle");
    set_attribute(&toggle_btn, "type", "button");
    toggle_btn.set_attribute("aria-label", "Toggle password visibility").ok();
    append_text(&toggle_btn, "\u{1f441}"); // 👁

    let show = signal(false);
    let inp_ref = input.clone();
    let btn_ref = toggle_btn.clone();
    create_effect(move || {
        if show.get() {
            set_attribute(&inp_ref, "type", "text");
            clear_children(&btn_ref);
            append_text(&btn_ref, "\u{1f648}"); // 🙈
        } else {
            set_attribute(&inp_ref, "type", "password");
            clear_children(&btn_ref);
            append_text(&btn_ref, "\u{1f441}"); // 👁
        }
    });

    add_event_listener(&toggle_btn, "click", move |_| { show.set(!show.get()); });

    append_node(&input_wrap, &input);
    append_node(&input_wrap, &toggle_btn);
    append_node(&wrap, &input_wrap);
    wrap
}

const PASSWORD_INPUT_CSS: &str = "\
.bu-pass-wrap{display:flex;align-items:center;position:relative}\
.bu-pass-input{flex:1;padding-right:2.5rem}\
.bu-pass-toggle{position:absolute;right:0.5rem;background:none;border:none;cursor:pointer;\
font-size:1rem;padding:0.25rem;color:#888}\
.bu-pass-toggle:hover{color:#e0e0e0}";

// ═══════════════════════════════════════════════════════════════════════════
// 43. NumberInput — input with +/- buttons
// ═══════════════════════════════════════════════════════════════════════════

/// Numeric input with decrement / increment buttons.
pub fn number_input(label: &str, value: Signal<f64>, step: f64) -> web_sys::Element {
    inject_css("numinput", NUMBER_INPUT_CSS);
    inject_css("input", INPUT_CSS);
    let wrap = create_element("div");
    set_attribute(&wrap, "class", "bu-field");

    if !label.is_empty() {
        let lbl = create_element("label");
        set_attribute(&lbl, "class", "bu-label");
        append_text(&lbl, label);
        append_node(&wrap, &lbl);
    }

    let row = create_element("div");
    set_attribute(&row, "class", "bu-num-wrap");

    let dec = create_element("button");
    set_attribute(&dec, "class", "bu-num-btn");
    set_attribute(&dec, "type", "button");
    dec.set_attribute("aria-label", "Decrease").ok();
    append_text(&dec, "\u{2212}"); // −
    let v = value;
    let s = step;
    add_event_listener(&dec, "click", move |_| { v.set(v.get() - s); });

    let input = create_element("input");
    set_attribute(&input, "type", "text");
    set_attribute(&input, "class", "bu-input bu-num-input");
    set_attribute(&input, "inputmode", "decimal");

    let inp_ref = input.clone();
    create_effect(move || {
        set_property(&inp_ref, "value", &JsValue::from_str(&value.get().to_string()));
    });
    let v = value;
    add_event_listener(&input, "change", move |e| {
        if let Ok(n) = event_target_value(&e).parse::<f64>() { v.set(n); }
    });

    let inc = create_element("button");
    set_attribute(&inc, "class", "bu-num-btn");
    set_attribute(&inc, "type", "button");
    inc.set_attribute("aria-label", "Increase").ok();
    append_text(&inc, "+");
    let v = value;
    let s = step;
    add_event_listener(&inc, "click", move |_| { v.set(v.get() + s); });

    append_node(&row, &dec);
    append_node(&row, &input);
    append_node(&row, &inc);
    append_node(&wrap, &row);
    wrap
}

const NUMBER_INPUT_CSS: &str = "\
.bu-num-wrap{display:flex;align-items:center;gap:0}\
.bu-num-btn{width:36px;height:36px;background:#1e1e1e;border:1px solid #444;color:#ccc;font-size:1rem;\
cursor:pointer;display:flex;align-items:center;justify-content:center;font-family:inherit;transition:all .15s}\
.bu-num-btn:first-child{border-radius:8px 0 0 8px}\
.bu-num-btn:last-child{border-radius:0 8px 8px 0}\
.bu-num-btn:hover{border-color:#f97316;color:#f97316}\
.bu-num-input{border-radius:0;text-align:center;width:80px}";

// ═══════════════════════════════════════════════════════════════════════════
// 44. FileUpload — drop zone / file picker
// ═══════════════════════════════════════════════════════════════════════════

/// File drop zone that reads a file as text and calls `on_file(filename, content)`.
pub fn file_upload(on_file: impl FnMut(String, String) + 'static) -> web_sys::Element {
    inject_css("fileupload", FILE_UPLOAD_CSS);
    let wrap = create_element("div");
    set_attribute(&wrap, "class", "bu-file-upload");
    wrap.set_attribute("role", "button").ok();
    wrap.set_attribute("aria-label", "Upload file").ok();

    let label_el = create_element("div");
    set_attribute(&label_el, "class", "bu-file-label");
    set_inner_html(&label_el, "&#128193; Drop a file or <u>browse</u>");
    append_node(&wrap, &label_el);

    let input = create_element("input");
    set_attribute(&input, "type", "file");
    set_style(&input, "display", "none");

    let input_ref = input.clone();
    add_event_listener(&wrap, "click", move |_| {
        if let Some(inp) = input_ref.dyn_ref::<web_sys::HtmlInputElement>() {
            inp.click();
        }
    });

    let cb = std::rc::Rc::new(std::cell::RefCell::new(on_file));
    add_event_listener(&input, "change", move |e| {
        let cb = cb.clone();
        if let Some(target) = e.target() {
            if let Some(inp) = target.dyn_ref::<web_sys::HtmlInputElement>() {
                if let Some(files) = inp.files() {
                    if let Some(file) = files.get(0) {
                        let name = file.name();
                        if let Ok(reader) = web_sys::FileReader::new() {
                            let r2 = reader.clone();
                            let closure = Closure::wrap(Box::new(move || {
                                if let Ok(result) = r2.result() {
                                    if let Some(text) = result.as_string() {
                                        (cb.borrow_mut())(name.clone(), text);
                                    }
                                }
                            }) as Box<dyn FnMut()>);
                            reader.set_onloadend(Some(closure.as_ref().unchecked_ref()));
                            closure.forget();
                            reader.read_as_text(&file).ok();
                        }
                    }
                }
            }
        }
    });

    // Drag-over visual feedback
    let wrap_ref = wrap.clone();
    add_event_listener(&wrap, "dragover", move |e| {
        e.prevent_default();
        toggle_class(&wrap_ref, "bu-file-dragover", true);
    });
    let wrap_ref = wrap.clone();
    add_event_listener(&wrap, "dragleave", move |_| {
        toggle_class(&wrap_ref, "bu-file-dragover", false);
    });
    let wrap_ref = wrap.clone();
    add_event_listener(&wrap, "drop", move |e| {
        e.prevent_default();
        toggle_class(&wrap_ref, "bu-file-dragover", false);
    });

    append_node(&wrap, &input);
    wrap
}

const FILE_UPLOAD_CSS: &str = "\
.bu-file-upload{border:2px dashed #444;border-radius:12px;padding:2rem;text-align:center;cursor:pointer;\
transition:border-color .2s,background .2s;background:#0a0a0a}\
.bu-file-upload:hover,.bu-file-dragover{border-color:#f97316;background:rgba(249,115,22,.05)}\
.bu-file-label{font-size:0.9rem;color:#888}";

// ═══════════════════════════════════════════════════════════════════════════
// 45. FormGroup — labelled form wrapper with optional error
// ═══════════════════════════════════════════════════════════════════════════

pub struct FormGroupBuilder {
    label: String,
    child: web_sys::Element,
    error_msg: Option<String>,
}

/// Wraps a form control with a label and optional error message.
///
/// ```ignore
/// let fg = form_group("Email", text_input("").build()).error("Required").build();
/// ```
pub fn form_group(label: &str, child: web_sys::Element) -> FormGroupBuilder {
    FormGroupBuilder { label: label.to_string(), child, error_msg: None }
}

impl FormGroupBuilder {
    pub fn error(mut self, msg: &str) -> Self { self.error_msg = Some(msg.to_string()); self }

    pub fn build(self) -> web_sys::Element {
        inject_css("input", INPUT_CSS);
        let wrap = create_element("div");
        set_attribute(&wrap, "class", "bu-field");
        if !self.label.is_empty() {
            let lbl = create_element("label");
            set_attribute(&lbl, "class", "bu-label");
            append_text(&lbl, &self.label);
            append_node(&wrap, &lbl);
        }
        append_node(&wrap, &self.child);
        if let Some(msg) = &self.error_msg {
            let err = create_element("div");
            set_attribute(&err, "class", "bu-error");
            err.set_attribute("role", "alert").ok();
            append_text(&err, msg);
            append_node(&wrap, &err);
        }
        wrap
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 46. Drawer — slide-in side panel
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Clone, Copy)]
pub enum DrawerSide { Left, Right }

pub struct DrawerBuilder {
    open: Signal<bool>,
    side: DrawerSide,
    title: String,
    body_el: Option<web_sys::Element>,
}

/// Slide-in panel with a backdrop, controlled by a `Signal<bool>`.
///
/// ```ignore
/// let open = signal(false);
/// let d = drawer(open).side(DrawerSide::Left).title("Menu").body(content).build();
/// ```
pub fn drawer(open: Signal<bool>) -> DrawerBuilder {
    DrawerBuilder { open, side: DrawerSide::Right, title: String::new(), body_el: None }
}

impl DrawerBuilder {
    pub fn side(mut self, s: DrawerSide) -> Self { self.side = s; self }
    pub fn title(mut self, t: &str) -> Self { self.title = t.into(); self }
    pub fn body(mut self, el: web_sys::Element) -> Self { self.body_el = Some(el); self }

    pub fn build(self) -> web_sys::Element {
        inject_css("drawer", DRAWER_CSS);
        let overlay = create_element("div");
        set_attribute(&overlay, "class", "bu-drawer-overlay");

        let panel = create_element("div");
        let side_cls = match self.side {
            DrawerSide::Left  => "bu-drawer bu-drawer-left",
            DrawerSide::Right => "bu-drawer bu-drawer-right",
        };
        set_attribute(&panel, "class", side_cls);
        panel.set_attribute("role", "dialog").ok();
        panel.set_attribute("aria-modal", "true").ok();

        if !self.title.is_empty() {
            let header = create_element("div");
            set_attribute(&header, "class", "bu-drawer-header");
            let h3 = create_element("h3");
            append_text(&h3, &self.title);
            append_node(&header, &h3);
            let close = create_element("button");
            set_attribute(&close, "class", "bu-drawer-close");
            close.set_attribute("aria-label", "Close drawer").ok();
            append_text(&close, "\u{00d7}"); // ×
            let o = self.open;
            add_event_listener(&close, "click", move |_| { o.set(false); });
            append_node(&header, &close);
            append_node(&panel, &header);
        }

        if let Some(body) = self.body_el {
            let b = create_element("div");
            set_attribute(&b, "class", "bu-drawer-body");
            append_node(&b, &body);
            append_node(&panel, &b);
        }

        append_node(&overlay, &panel);

        // Close on backdrop click
        let o = self.open;
        let overlay_ref = overlay.clone();
        add_event_listener(&overlay, "click", move |e| {
            if let Some(t) = e.target() {
                if let Some(el) = t.dyn_ref::<web_sys::Element>() {
                    if el.class_list().contains("bu-drawer-overlay") { o.set(false); }
                }
            }
        });

        // Show / hide
        let overlay_ref2 = overlay_ref.clone();
        let open = self.open;
        create_effect(move || {
            if open.get() { set_style(&overlay_ref2, "display", "flex"); }
            else { set_style(&overlay_ref2, "display", "none"); }
        });
        set_style(&overlay, "display", "none");
        overlay
    }
}

const DRAWER_CSS: &str = "\
.bu-drawer-overlay{position:fixed;inset:0;background:rgba(0,0,0,.6);z-index:1000;display:flex}\
.bu-drawer{background:#141414;border:1px solid #333;width:320px;max-width:85vw;height:100%;overflow-y:auto}\
.bu-drawer-right{margin-left:auto;animation:bu-drawer-right-in .2s ease}\
@keyframes bu-drawer-right-in{from{transform:translateX(100%)}to{transform:translateX(0)}}\
.bu-drawer-left{margin-right:auto;animation:bu-drawer-left-in .2s ease}\
@keyframes bu-drawer-left-in{from{transform:translateX(-100%)}to{transform:translateX(0)}}\
.bu-drawer-header{display:flex;justify-content:space-between;align-items:center;padding:1rem 1.25rem;\
border-bottom:1px solid #222}\
.bu-drawer-header h3{font-size:1rem;margin:0}\
.bu-drawer-close{background:none;border:none;color:#888;font-size:1.3rem;cursor:pointer;padding:0.25rem}\
.bu-drawer-close:hover{color:#fff}\
.bu-drawer-body{padding:1.25rem}";

// ═══════════════════════════════════════════════════════════════════════════
// 47. Accordion — collapsible sections
// ═══════════════════════════════════════════════════════════════════════════

/// Single-open accordion. Click a header to expand its section (collapses others).
pub fn accordion(items: &[(&str, fn() -> web_sys::Element)]) -> web_sys::Element {
    inject_css("accordion", ACCORDION_CSS);
    let active = signal(-1i32);
    let wrap = create_element("div");
    set_attribute(&wrap, "class", "bu-accordion");

    for (i, &(header_text, content_fn)) in items.iter().enumerate() {
        let item = create_element("div");
        set_attribute(&item, "class", "bu-accordion-item");

        let header = create_element("button");
        set_attribute(&header, "class", "bu-accordion-header");
        append_text(&header, header_text);
        let arrow = create_element("span");
        set_attribute(&arrow, "class", "bu-accordion-arrow");
        append_text(&arrow, "\u{25b8}"); // ▸
        append_node(&header, &arrow);

        let a = active;
        let idx = i as i32;
        add_event_listener(&header, "click", move |_| {
            if a.get() == idx { a.set(-1); } else { a.set(idx); }
        });
        append_node(&item, &header);

        let panel = create_element("div");
        set_attribute(&panel, "class", "bu-accordion-panel");
        let content = content_fn();
        let inner = create_element("div");
        set_attribute(&inner, "class", "bu-accordion-inner");
        append_node(&inner, &content);
        append_node(&panel, &inner);

        let panel_ref = panel.clone();
        let header_ref = header.clone();
        let arrow_ref = arrow.clone();
        create_effect(move || {
            if active.get() == idx {
                set_style(&panel_ref, "max-height", "500px");
                toggle_class(&header_ref, "bu-accordion-active", true);
                clear_children(&arrow_ref);
                append_text(&arrow_ref, "\u{25be}"); // ▾
            } else {
                set_style(&panel_ref, "max-height", "0");
                toggle_class(&header_ref, "bu-accordion-active", false);
                clear_children(&arrow_ref);
                append_text(&arrow_ref, "\u{25b8}"); // ▸
            }
        });

        append_node(&item, &panel);
        append_node(&wrap, &item);
    }
    wrap
}

const ACCORDION_CSS: &str = "\
.bu-accordion{display:flex;flex-direction:column;border:1px solid #333;border-radius:8px;overflow:hidden}\
.bu-accordion-item{border-bottom:1px solid #222}\
.bu-accordion-item:last-child{border-bottom:none}\
.bu-accordion-header{display:flex;justify-content:space-between;align-items:center;width:100%;\
padding:0.75rem 1rem;background:#141414;border:none;color:#e0e0e0;font-size:0.9rem;cursor:pointer;\
font-family:inherit;text-align:left;transition:background .15s}\
.bu-accordion-header:hover{background:#1e1e1e}\
.bu-accordion-active{color:#f97316}\
.bu-accordion-arrow{font-size:0.8rem;color:#888;transition:transform .2s}\
.bu-accordion-panel{max-height:0;overflow:hidden;transition:max-height .3s ease;background:#0a0a0a}\
.bu-accordion-inner{padding:1rem}";

// ═══════════════════════════════════════════════════════════════════════════
// 48. Rating — star rating
// ═══════════════════════════════════════════════════════════════════════════

/// Clickable star rating with hover preview.
pub fn rating(value: Signal<u32>, max: u32) -> web_sys::Element {
    inject_css("rating", RATING_CSS);
    let hover = signal(0u32);
    let wrap = create_element("div");
    set_attribute(&wrap, "class", "bu-rating");
    wrap.set_attribute("role", "slider").ok();
    wrap.set_attribute("aria-valuemin", "0").ok();
    wrap.set_attribute("aria-valuemax", &max.to_string()).ok();

    let mut stars: Vec<web_sys::Element> = Vec::new();
    for i in 1..=max {
        let star = create_element("span");
        set_attribute(&star, "class", "bu-star");
        append_text(&star, "\u{2605}"); // ★
        let v = value;
        add_event_listener(&star, "click", move |_| { v.set(i); });
        let h = hover;
        add_event_listener(&star, "mouseenter", move |_| { h.set(i); });
        stars.push(star.clone());
        append_node(&wrap, &star);
    }

    let h = hover;
    add_event_listener(&wrap, "mouseleave", move |_| { h.set(0); });

    create_effect(move || {
        let val = value.get();
        let hov = hover.get();
        let active = if hov > 0 { hov } else { val };
        for (i, star) in stars.iter().enumerate() {
            if (i as u32) < active {
                set_attribute(star, "class", "bu-star bu-star-filled");
            } else {
                set_attribute(star, "class", "bu-star");
            }
        }
    });
    wrap
}

const RATING_CSS: &str = "\
.bu-rating{display:inline-flex;gap:0.15rem}\
.bu-star{font-size:1.5rem;color:#444;cursor:pointer;transition:color .1s;user-select:none}\
.bu-star-filled{color:#f97316}";

// ═══════════════════════════════════════════════════════════════════════════
// 49. CopyButton — click-to-copy with feedback
// ═══════════════════════════════════════════════════════════════════════════

/// Button that copies `text` to the clipboard and shows brief "Copied!" feedback.
pub fn copy_button(text: &str) -> web_sys::Element {
    inject_css("copybtn", COPY_BTN_CSS);
    let btn = create_element("button");
    set_attribute(&btn, "class", "bu-copy-btn");
    btn.set_attribute("aria-label", "Copy to clipboard").ok();
    set_inner_html(&btn, "&#128203; Copy"); // 📋

    let text = text.to_string();
    let btn_ref = btn.clone();
    add_event_listener(&btn, "click", move |_| {
        // Use Clipboard API via js_sys::Reflect
        let window = web_sys::window().unwrap();
        if let Ok(nav) = js_sys::Reflect::get(&window, &JsValue::from_str("navigator")) {
            if let Ok(clip) = js_sys::Reflect::get(&nav, &JsValue::from_str("clipboard")) {
                if let Ok(write_fn) = js_sys::Reflect::get(&clip, &JsValue::from_str("writeText")) {
                    if let Ok(func) = write_fn.dyn_into::<js_sys::Function>() {
                        let _ = func.call1(&clip, &JsValue::from_str(&text));
                    }
                }
            }
        }
        // Visual feedback
        let btn_r = btn_ref.clone();
        set_inner_html(&btn_r, "&#10003; Copied!");
        let btn_r2 = btn_r.clone();
        set_timeout(move || {
            set_inner_html(&btn_r2, "&#128203; Copy");
        }, 2000);
    });
    btn
}

const COPY_BTN_CSS: &str = "\
.bu-copy-btn{display:inline-flex;align-items:center;gap:0.4rem;padding:0.4rem 0.9rem;background:#1e1e1e;\
border:1px solid #444;border-radius:8px;color:#ccc;font-size:0.8rem;cursor:pointer;font-family:inherit;\
transition:all .15s}\
.bu-copy-btn:hover{border-color:#f97316;color:#f97316}";
