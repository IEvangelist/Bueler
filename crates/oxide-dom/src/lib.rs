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

/// Create a DOM element by tag name.
pub fn create_element(tag: &str) -> web_sys::Element {
    document().create_element(tag).unwrap_or_else(|_| {
        panic!("failed to create <{}>", tag);
    })
}

/// Create an empty text node.
pub fn create_text_node(text: &str) -> web_sys::Text {
    document().create_text_node(text)
}

/// Set a static attribute on an element.
pub fn set_attribute(el: &web_sys::Element, name: &str, value: &str) {
    el.set_attribute(name, value)
        .unwrap_or_else(|_| panic!("failed to set attribute {}={}", name, value));
}

/// Append a static text node to an element.
pub fn append_text(parent: &web_sys::Element, text: &str) {
    let node = document().create_text_node(text);
    parent
        .append_child(&node)
        .expect("failed to append text node");
}

/// Append any node (element, text, …) to a parent element.
pub fn append_node<N: AsRef<web_sys::Node>>(parent: &web_sys::Element, child: &N) {
    parent
        .append_child(child.as_ref())
        .expect("failed to append child");
}

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
