use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

pub use web_sys::Event;

/// Get the global `document`.
fn document() -> web_sys::Document {
    web_sys::window()
        .expect("no global window")
        .document()
        .expect("no document on window")
}

// ---------------------------------------------------------------------------
// Element creation
// ---------------------------------------------------------------------------

/// Create a DOM element by tag name.
pub fn create_element(tag: &str) -> web_sys::Element {
    document().create_element(tag).unwrap_or_else(|_| {
        panic!("failed to create <{}>", tag);
    })
}

/// Create an element in the SVG namespace.
pub fn create_svg_element(tag: &str) -> web_sys::Element {
    document()
        .create_element_ns(Some("http://www.w3.org/2000/svg"), tag)
        .unwrap_or_else(|_| panic!("failed to create SVG <{}>", tag))
}

/// Create an empty text node.
pub fn create_text_node(text: &str) -> web_sys::Text {
    document().create_text_node(text)
}

// ---------------------------------------------------------------------------
// Attributes & properties
// ---------------------------------------------------------------------------

/// Set a static attribute on an element.
pub fn set_attribute(el: &web_sys::Element, name: &str, value: &str) {
    el.set_attribute(name, value)
        .unwrap_or_else(|_| panic!("failed to set attribute {}={}", name, value));
}

/// Set a JavaScript property on an element (e.g. `input.value`, `input.checked`).
pub fn set_property(el: &web_sys::Element, name: &str, value: &JsValue) {
    js_sys::Reflect::set(el, &JsValue::from_str(name), value).ok();
}

/// Set a CSS style property on an element.
pub fn set_style(el: &web_sys::Element, property: &str, value: &str) {
    if let Some(html_el) = el.dyn_ref::<web_sys::HtmlElement>() {
        html_el.style().set_property(property, value).ok();
    }
}

/// Toggle a CSS class on an element.
pub fn toggle_class(el: &web_sys::Element, class: &str, active: bool) {
    let list = el.class_list();
    if active {
        list.add_1(class).ok();
    } else {
        list.remove_1(class).ok();
    }
}

// ---------------------------------------------------------------------------
// DOM tree manipulation
// ---------------------------------------------------------------------------

/// Append a static text node to an element.
pub fn append_text(parent: &web_sys::Element, text: &str) {
    let node = document().create_text_node(text);
    parent
        .append_child(&node)
        .expect("failed to append text node");
}

/// Append any `Node` (element, text, …) to a parent element.
pub fn append_node<N: AsRef<web_sys::Node>>(parent: &web_sys::Element, child: &N) {
    parent
        .append_child(child.as_ref())
        .expect("failed to append child");
}

/// Clear all children from an element.
pub fn clear_children(el: &web_sys::Element) {
    el.set_inner_html("");
}

/// Set innerHTML on an element.
pub fn set_inner_html(el: &web_sys::Element, html: &str) {
    el.set_inner_html(html);
}

// ---------------------------------------------------------------------------
// Query
// ---------------------------------------------------------------------------

/// Query a single element by CSS selector.
pub fn query_selector(selector: &str) -> Option<web_sys::Element> {
    document().query_selector(selector).ok().flatten()
}

/// Get the document body.
pub fn body() -> web_sys::HtmlElement {
    document().body().expect("no body")
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

/// Register an event listener. The closure is leaked intentionally — it lives
/// as long as the DOM element.
pub fn add_event_listener(
    el: &web_sys::Element,
    event: &str,
    handler: impl FnMut(web_sys::Event) + 'static,
) {
    let closure = Closure::wrap(Box::new(handler) as Box<dyn FnMut(web_sys::Event)>);
    el.add_event_listener_with_callback(event, closure.as_ref().unchecked_ref())
        .expect("failed to add event listener");
    closure.forget();
}

/// Add an event listener to the window.
pub fn on_window_event(event: &str, handler: impl FnMut(web_sys::Event) + 'static) {
    let window = web_sys::window().unwrap();
    let closure = Closure::wrap(Box::new(handler) as Box<dyn FnMut(web_sys::Event)>);
    window
        .add_event_listener_with_callback(event, closure.as_ref().unchecked_ref())
        .ok();
    closure.forget();
}

/// Add an event listener to the document.
pub fn on_document_event(event: &str, handler: impl FnMut(web_sys::Event) + 'static) {
    let closure = Closure::wrap(Box::new(handler) as Box<dyn FnMut(web_sys::Event)>);
    document()
        .add_event_listener_with_callback(event, closure.as_ref().unchecked_ref())
        .ok();
    closure.forget();
}

// ---------------------------------------------------------------------------
// Timers
// ---------------------------------------------------------------------------

/// Call `f` once after `ms` milliseconds. Returns a timer ID.
pub fn set_timeout(f: impl FnOnce() + 'static, ms: i32) -> i32 {
    let window = web_sys::window().unwrap();
    let cb = Closure::once(Box::new(f) as Box<dyn FnOnce()>);
    let id = window
        .set_timeout_with_callback_and_timeout_and_arguments_0(
            cb.as_ref().unchecked_ref(),
            ms,
        )
        .unwrap();
    cb.forget();
    id
}

/// Call `f` repeatedly every `ms` milliseconds. Returns an interval ID.
pub fn set_interval(f: impl FnMut() + 'static, ms: i32) -> i32 {
    let window = web_sys::window().unwrap();
    let cb = Closure::wrap(Box::new(f) as Box<dyn FnMut()>);
    let id = window
        .set_interval_with_callback_and_timeout_and_arguments_0(
            cb.as_ref().unchecked_ref(),
            ms,
        )
        .unwrap();
    cb.forget();
    id
}

/// Cancel a timer created by `set_interval`.
pub fn clear_interval(id: i32) {
    web_sys::window().unwrap().clear_interval_with_handle(id);
}

/// Cancel a timer created by `set_timeout`.
pub fn clear_timeout(id: i32) {
    web_sys::window().unwrap().clear_timeout_with_handle(id);
}

/// Schedule a callback on the next animation frame. Returns a request ID.
pub fn request_animation_frame(f: impl FnOnce() + 'static) -> i32 {
    let window = web_sys::window().unwrap();
    let cb = Closure::once(Box::new(f) as Box<dyn FnOnce()>);
    let id = window
        .request_animation_frame(cb.as_ref().unchecked_ref())
        .unwrap();
    cb.forget();
    id
}

// ---------------------------------------------------------------------------
// Local storage
// ---------------------------------------------------------------------------

/// Read a value from localStorage.
pub fn local_storage_get(key: &str) -> Option<String> {
    web_sys::window()?
        .local_storage()
        .ok()
        .flatten()?
        .get_item(key)
        .ok()
        .flatten()
}

/// Write a value to localStorage.
pub fn local_storage_set(key: &str, value: &str) {
    if let Some(storage) = web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
    {
        storage.set_item(key, value).ok();
    }
}

// ---------------------------------------------------------------------------
// Console
// ---------------------------------------------------------------------------

/// Log a message to the browser console.
pub fn log(msg: &str) {
    web_sys::console::log_1(&msg.into());
}

/// Log a warning to the browser console.
pub fn warn(msg: &str) {
    web_sys::console::warn_1(&msg.into());
}

// ---------------------------------------------------------------------------
// Form helpers
// ---------------------------------------------------------------------------

/// Get the `.value` property of an input/textarea/select element from an event.
pub fn event_target_value(event: &web_sys::Event) -> String {
    event
        .target()
        .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
        .map(|el| el.value())
        .or_else(|| {
            event
                .target()
                .and_then(|t| t.dyn_into::<web_sys::HtmlTextAreaElement>().ok())
                .map(|el| el.value())
        })
        .or_else(|| {
            event
                .target()
                .and_then(|t| t.dyn_into::<web_sys::HtmlSelectElement>().ok())
                .map(|el| el.value())
        })
        .unwrap_or_default()
}

/// Check if a checkbox/radio input is checked, from an event.
pub fn event_target_checked(event: &web_sys::Event) -> bool {
    event
        .target()
        .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
        .map(|el| el.checked())
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// Location
// ---------------------------------------------------------------------------

/// Get the current URL hash (e.g. `"#counter"`).
pub fn get_hash() -> String {
    web_sys::window()
        .unwrap()
        .location()
        .hash()
        .unwrap_or_default()
}

/// Set the URL hash without triggering navigation.
pub fn set_hash(hash: &str) {
    web_sys::window()
        .unwrap()
        .location()
        .set_hash(hash)
        .ok();
}

// ---------------------------------------------------------------------------
// Mount
// ---------------------------------------------------------------------------

/// Mount a view into the DOM. `selector` is a CSS selector for the mount
/// point (e.g. `"#app"`). `f` is a closure that builds and returns the root
/// element.
pub fn mount(selector: &str, f: impl FnOnce() -> web_sys::Element) {
    let root = f();
    let mount_point = document()
        .query_selector(selector)
        .expect("query_selector failed")
        .unwrap_or_else(|| panic!("mount point '{}' not found", selector));
    mount_point
        .append_child(&root)
        .expect("failed to mount application");
}
