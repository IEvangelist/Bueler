//! Reactive hooks — composable utilities built on signals + DOM APIs.
//!
//! These follow the `use_*` naming convention and return reactive signals
//! that auto-update when the underlying browser state changes.

use bueler_core::{create_effect, signal, Signal};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

// ═══════════════════════════════════════════════════════════════════════════
// Storage
// ═══════════════════════════════════════════════════════════════════════════

/// A signal backed by localStorage. Reads the initial value from storage
/// and writes back on every change.
///
/// ```ignore
/// let name = use_local_storage("user-name", "World".to_string());
/// // name.get() reads from signal (fast)
/// // name.set("Bueler") writes to signal AND localStorage
/// ```
pub fn use_local_storage(key: &str, default: String) -> Signal<String> {
    let initial = crate::local_storage_get(key).unwrap_or(default);
    let s = signal(initial);
    let k = key.to_string();
    create_effect(move || {
        crate::local_storage_set(&k, &s.get());
    });
    s
}

// ═══════════════════════════════════════════════════════════════════════════
// Timing
// ═══════════════════════════════════════════════════════════════════════════

/// Returns a signal that ticks up every `ms` milliseconds.
///
/// ```ignore
/// let tick = use_interval(1000); // increments every second
/// ```
pub fn use_interval(ms: i32) -> Signal<u64> {
    let count = signal(0u64);
    let c = count;
    crate::set_interval(move || { c.update(|v| *v += 1); }, ms);
    count
}

/// Returns a debounced version of `source`. The output signal only
/// updates after `source` has been stable for `ms` milliseconds.
pub fn use_debounce<T: Clone + 'static + std::fmt::Display>(source: Signal<T>, ms: i32) -> Signal<T> {
    let debounced = signal(source.get());
    let timeout_id = signal(0i32);
    create_effect(move || {
        let val = source.get();
        let old_id = timeout_id.get();
        if old_id != 0 { crate::clear_timeout(old_id); }
        let d = debounced;
        let id = crate::set_timeout(move || { d.set(val); }, ms);
        timeout_id.set(id);
    });
    debounced
}

/// Returns a throttled version of `source`. Updates at most once
/// every `ms` milliseconds.
pub fn use_throttle<T: Clone + 'static>(source: Signal<T>, ms: i32) -> Signal<T> {
    let throttled = signal(source.get());
    let last_fire = signal(0.0f64);
    create_effect(move || {
        let val = source.get();
        let now = js_sys::Date::now();
        if now - last_fire.get() >= ms as f64 {
            throttled.set(val);
            last_fire.set(now);
        }
    });
    throttled
}

// ═══════════════════════════════════════════════════════════════════════════
// Window / viewport
// ═══════════════════════════════════════════════════════════════════════════

/// Reactive window dimensions. Updates on resize.
///
/// ```ignore
/// let (width, height) = use_window_size();
/// ```
pub fn use_window_size() -> (Signal<i32>, Signal<i32>) {
    let window = web_sys::window().unwrap();
    let w = signal(window.inner_width().unwrap().as_f64().unwrap_or(0.0) as i32);
    let h = signal(window.inner_height().unwrap().as_f64().unwrap_or(0.0) as i32);
    crate::on_window_event("resize", move |_| {
        if let Some(win) = web_sys::window() {
            w.set(win.inner_width().unwrap().as_f64().unwrap_or(0.0) as i32);
            h.set(win.inner_height().unwrap().as_f64().unwrap_or(0.0) as i32);
        }
    });
    (w, h)
}

/// Reactive scroll position of the document.
pub fn use_scroll() -> (Signal<f64>, Signal<f64>) {
    let x = signal(0.0f64);
    let y = signal(0.0f64);
    crate::on_window_event("scroll", move |_| {
        if let Some(win) = web_sys::window() {
            x.set(win.scroll_x().unwrap_or(0.0));
            y.set(win.scroll_y().unwrap_or(0.0));
        }
    });
    (x, y)
}

/// Reactive global mouse position.
pub fn use_mouse() -> (Signal<i32>, Signal<i32>) {
    let x = signal(0i32);
    let y = signal(0i32);
    crate::on_document_event("mousemove", move |e| {
        if let Some(me) = e.dyn_ref::<web_sys::MouseEvent>() {
            x.set(me.client_x());
            y.set(me.client_y());
        }
    });
    (x, y)
}

// ═══════════════════════════════════════════════════════════════════════════
// Media / network
// ═══════════════════════════════════════════════════════════════════════════

/// Reactive CSS media query. Returns `true` when the query matches.
///
/// ```ignore
/// let is_mobile = use_media_query("(max-width: 768px)");
/// ```
pub fn use_media_query(query: &str) -> Signal<bool> {
    let window = web_sys::window().unwrap();
    let mql = window.match_media(query).unwrap().unwrap();
    let matches = signal(mql.matches());
    let m = matches;
    let closure = Closure::wrap(Box::new(move |e: web_sys::Event| {
        if let Some(mql) = e.target() {
            if let Ok(mql) = mql.dyn_into::<web_sys::MediaQueryList>() {
                m.set(mql.matches());
            }
        }
    }) as Box<dyn FnMut(web_sys::Event)>);
    mql.add_event_listener_with_callback("change", closure.as_ref().unchecked_ref()).ok();
    closure.forget();
    matches
}

/// Reactive network online/offline status.
pub fn use_online() -> Signal<bool> {
    let online = signal(web_sys::window().unwrap().navigator().on_line());
    let o1 = online;
    let o2 = online;
    crate::on_window_event("online", move |_| { o1.set(true); });
    crate::on_window_event("offline", move |_| { o2.set(false); });
    online
}

/// Whether the user prefers dark color scheme.
pub fn use_preferred_dark() -> Signal<bool> {
    use_media_query("(prefers-color-scheme: dark)")
}

// ═══════════════════════════════════════════════════════════════════════════
// Element-level hooks
// ═══════════════════════════════════════════════════════════════════════════

/// Detect clicks outside of `el`. Calls `handler` when a click
/// occurs anywhere else in the document.
pub fn use_click_outside(el: &web_sys::Element, mut handler: impl FnMut() + 'static) {
    let el_clone = el.clone();
    crate::on_document_event("click", move |e| {
        if let Some(target) = e.target() {
            if let Some(node) = target.dyn_ref::<web_sys::Node>() {
                if !el_clone.contains(Some(node)) {
                    handler();
                }
            }
        }
    });
}

/// Track whether an element has focus.
pub fn use_focus(el: &web_sys::Element) -> Signal<bool> {
    let focused = signal(false);
    let f1 = focused;
    let f2 = focused;
    crate::add_event_listener(el, "focus", move |_| { f1.set(true); });
    crate::add_event_listener(el, "blur", move |_| { f2.set(false); });
    focused
}

// ═══════════════════════════════════════════════════════════════════════════
// Head management
// ═══════════════════════════════════════════════════════════════════════════

/// Set the document title.
pub fn set_title(title: &str) {
    if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
        doc.set_title(title);
    }
}

/// Set or create a `<meta>` tag by name.
pub fn set_meta(name: &str, content: &str) {
    if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
        let selector = format!("meta[name=\"{}\"]", name);
        let meta = doc.query_selector(&selector).ok().flatten().unwrap_or_else(|| {
            let m = doc.create_element("meta").unwrap();
            m.set_attribute("name", name).ok();
            if let Some(head) = doc.head() {
                head.append_child(&m).ok();
            }
            m
        });
        meta.set_attribute("content", content).ok();
    }
}
