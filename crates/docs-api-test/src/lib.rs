//! # bueler-docs-api-test
//!
//! **This crate exists to keep `README.md` and `examples/showcase/docs.html` honest.**
//!
//! Every public symbol that appears in the Bueler documentation is imported
//! and used somewhere in this file. If anyone renames, removes, or changes
//! the signature of a documented API, this crate stops compiling and CI fails.
//!
//! It is a `cdylib` targeting `wasm32-unknown-unknown` because most of the
//! documented APIs are wasm-only. It is **never published**.
//!
//! When adding a new public API to a Bueler crate that you intend to
//! document on the site, also add a reference here.

#![allow(unused, clippy::all, dead_code)]

use bueler::prelude::*;
use bueler::{
    // §3 Core Reactivity
    batch, clear_hook, create_effect, memo, on_cleanup, on_mount, provide_context, set_hook,
    signal, untrack, use_context, watch, HookEvent, Signal,
    // §16 Component System
    Component,
};
use bueler::components::{
    alert, badge, button, card, checkbox, divider, modal, progress, scroll_to_top, select,
    skeleton, spinner, spinner_with_text, tabs, textarea, text_input, Severity, Size, Variant,
};
use bueler::dom::{
    // §6 DOM Utilities
    add_event_listener, append_node, append_text, body, clear_children, create_element,
    create_svg_element, create_text_node, event_target_checked, event_target_value, mount,
    on_document_event, on_window_event, query_selector, set_attribute, set_inner_html,
    set_property, set_style, toggle_class,
    // §10 Timers
    clear_interval, clear_timeout, request_animation_frame, set_interval, set_timeout,
    // §11 Storage
    local_storage_get, local_storage_set,
    // §12 Console
    log, warn,
    // §13 Location
    get_hash, set_hash,
    // §7 Head management
    set_meta, set_title,
    // §5 Reactive hooks
    use_click_outside, use_debounce, use_focus, use_interval, use_local_storage, use_media_query,
    use_mouse, use_online, use_preferred_dark, use_scroll, use_throttle, use_window_size,
    Event,
};
use bueler::router::{
    link, nav_link, navigate, route, use_param, use_params, use_route, Router, RouterMode,
};
use bueler::resiliency::{
    default_error_boundary, error_boundary, retry, sleep, with_timeout, CircuitBreaker,
    CircuitBreakerConfig, CircuitError, CircuitState, RetryConfig, RetryError, TimeoutError,
};
use bueler::telemetry::{
    self, clear_spans, get_spans, get_stats, init as telemetry_init, span, traced_fetch,
    Config as TelemetryConfig, SpanGuard, SpanKind, SpanRecord, Stats,
};

use wasm_bindgen::prelude::*;

// ─── §3 Core Reactivity ─────────────────────────────────────────────────────

#[wasm_bindgen]
pub fn assert_reactivity_api() {
    let count: Signal<i32> = signal(0);
    let _val = count.get();
    count.set(5);
    count.update(|n| *n += 1);

    // Signal is Copy
    let _copied = count;
    let _again = count;

    let _doubled = memo(move || count.get() * 2);

    create_effect(move || {
        log(&format!("count is {}", count.get()));
    });

    batch(|| {
        count.set(10);
        count.set(20);
    });

    create_effect(move || {
        let _a = count.get();
        let _b = untrack(|| count.get());
    });

    watch(move || count.get(), |new_val: i32| {
        log(&format!("changed to {new_val}"));
    });

    on_mount(|| log("mounted"));
    on_cleanup(|| log("teardown"));

    provide_context(count);
    let _retrieved: Option<Signal<i32>> = use_context::<Signal<i32>>();

    set_hook(reactivity_hook);
    clear_hook();
}

fn reactivity_hook(event: HookEvent) {
    match event {
        HookEvent::SignalCreate { .. } => {}
        HookEvent::SignalRead { .. } => {}
        HookEvent::SignalWrite { .. } => {}
        HookEvent::EffectRun { .. } => {}
        HookEvent::EffectComplete { .. } => {}
        HookEvent::BatchStart => {}
        HookEvent::BatchEnd { .. } => {}
    }
}

// ─── §4 View Macro ──────────────────────────────────────────────────────────

#[wasm_bindgen]
pub fn assert_view_macro() {
    let count = signal(0i32);
    let name = signal(String::from("Bueler"));
    let checked = signal(false);
    let show = signal(true);
    let items = signal::<Vec<i32>>(vec![1, 2, 3]);

    let _root = view! {
        <div class="card" class:active={show.get()}>
            <h1>"Hello"</h1>
            <p>"Count: " {count}</p>
            <div class={name}>"styled"</div>
            <input bind:value={name} />
            <input type="checkbox" bind:checked={checked} />
            {if show.get() {
                <p>"Visible!"</p>
            } else {
                <p>"Hidden"</p>
            }}
            {for item in items.get() {
                <li>{item}</li>
            }}
            <button on:click={move |_: Event| count.update(|n| *n += 1)}>
                "Click"
            </button>
            <DocsCounter initial={5} />
            <svg width="100" height="100">
                <circle cx="50" cy="50" r="20" />
            </svg>
        </div>
    };
}

// ─── §16 Component System ───────────────────────────────────────────────────

struct DocsCounter {
    initial: i32,
}

impl Component for DocsCounter {
    fn render(self) -> web_sys::Element {
        let count = signal(self.initial);
        view! {
            <button on:click={move |_: Event| count.update(|n| *n += 1)}>
                {count}
            </button>
        }
    }
}

// ─── §5 Reactive Hooks ──────────────────────────────────────────────────────

#[wasm_bindgen]
pub fn assert_hooks() {
    let _theme: Signal<String> = use_local_storage("theme", "dark".to_string());
    let _ticks: Signal<u64> = use_interval(1000);

    let query = signal("".to_string());
    let _debounced: Signal<String> = use_debounce(query, 300);
    let _throttled: Signal<String> = use_throttle(query, 200);

    let (_w, _h): (Signal<i32>, Signal<i32>) = use_window_size();
    let (_sx, _sy): (Signal<f64>, Signal<f64>) = use_scroll();
    let (_mx, _my): (Signal<i32>, Signal<i32>) = use_mouse();

    let _is_mobile: Signal<bool> = use_media_query("(max-width: 768px)");
    let _online: Signal<bool> = use_online();
    let _dark: Signal<bool> = use_preferred_dark();

    let el = create_element("div");
    use_click_outside(&el, || log("outside"));
    let _focused: Signal<bool> = use_focus(&el);
}

// ─── §6 DOM Utilities ───────────────────────────────────────────────────────

#[wasm_bindgen]
pub fn assert_dom_utilities() {
    let el = create_element("div");
    let _svg = create_svg_element("circle");
    let _txt = create_text_node("hello");

    set_attribute(&el, "id", "main");
    set_property(&el, "value", &JsValue::from_str("text"));
    set_style(&el, "color", "red");
    toggle_class(&el, "active", true);

    append_text(&el, "some text");
    let child = create_element("span");
    append_node(&el, &child);
    clear_children(&el);
    set_inner_html(&el, "<b>bold</b>");

    let _el: Option<web_sys::Element> = query_selector("#app");
    let _b: web_sys::HtmlElement = body();

    add_event_listener(&el, "click", |_e: Event| {});
    on_window_event("resize", |_| {});
    on_document_event("keydown", |_| {});
}

// ─── §7 Head Management ─────────────────────────────────────────────────────

#[wasm_bindgen]
pub fn assert_head_management() {
    set_title("Page");
    set_meta("description", "desc");
}

// ─── §8 Routing ─────────────────────────────────────────────────────────────

fn route_home() -> web_sys::Element {
    create_element("div")
}
fn route_about() -> web_sys::Element {
    create_element("div")
}
fn route_user() -> web_sys::Element {
    let _id = use_param("id");
    let _all = use_params();
    create_element("div")
}

#[wasm_bindgen]
pub fn assert_routing() {
    let routes = [
        route("/", route_home),
        route("/about", route_about),
        route("/user/:id", route_user),
    ];
    let _hash = Router::new(RouterMode::Hash, &routes);
    let _hist = Router::new(RouterMode::History, &routes);
    let r = Router::new(RouterMode::Hash, &routes);
    let _outlet: web_sys::Element = r.view();
    let _path: Signal<String> = use_route();

    navigate("/user/42");

    let _a: web_sys::Element = link("/", "Home");
    let _b: web_sys::Element = nav_link("/about", "About", "nav-item");
}

// ─── §9 Pre-built Components ────────────────────────────────────────────────

fn tab_one() -> web_sys::Element {
    create_element("div")
}
fn tab_two() -> web_sys::Element {
    create_element("div")
}

#[wasm_bindgen]
pub fn assert_prebuilt_components() {
    // Documented form: button("Save").primary().on_click(|_| save()) — returns
    // an Element directly (no .build() needed when using on_click).
    let _: web_sys::Element = button("Save").primary().on_click(|_| ());
    // ButtonBuilder also has .build() if you don't want a click handler.
    let _: web_sys::Element = button("Cancel").outline().build();
    let email = signal(String::new());
    let _: web_sys::Element = text_input("Email")
        .placeholder("you@example.com")
        .bind(email)
        .build();
    let notes = signal(String::new());
    let _: web_sys::Element = textarea("Notes", notes);
    let lang = signal("rs".to_string());
    let _: web_sys::Element = select("Language", &[("rs", "Rust"), ("ts", "TypeScript")], lang);
    let subscribed = signal(false);
    let _: web_sys::Element = checkbox("Subscribe to updates", subscribed);
    let _: web_sys::Element = card("Settings")
        .body(create_element("div"))
        .footer(create_element("div"))
        .build();
    let _: web_sys::Element = alert("Saved!").success().build();
    let is_open = signal(false);
    let _: web_sys::Element = modal(is_open)
        .title("Confirm")
        .body(create_element("div"))
        .build();
    let _: web_sys::Element = spinner();
    let _: web_sys::Element = spinner_with_text("Loading...");
    let progress_signal = signal(0.5f64);
    let _: web_sys::Element = progress(progress_signal);
    let _: web_sys::Element = tabs(&[("Tab 1", tab_one as fn() -> web_sys::Element), ("Tab 2", tab_two)]);
    let _: web_sys::Element = badge("New", Severity::Success);
    let _: web_sys::Element = divider();
    let _: web_sys::Element = skeleton("100%", "20px");
    let _: web_sys::Element = scroll_to_top(300);

    let _ = Variant::Primary;
    let _ = Size::Medium;
}

// ─── §10 Timers ─────────────────────────────────────────────────────────────

#[wasm_bindgen]
pub fn assert_timers() {
    let id: i32 = set_timeout(|| log("done"), 1000);
    clear_timeout(id);

    let id: i32 = set_interval(|| log("tick"), 500);
    clear_interval(id);

    let _: i32 = request_animation_frame(|| {});
}

// ─── §11 Storage ────────────────────────────────────────────────────────────

#[wasm_bindgen]
pub fn assert_storage() {
    local_storage_set("key", "value");
    let _val: Option<String> = local_storage_get("key");
}

// ─── §12 Console ────────────────────────────────────────────────────────────

#[wasm_bindgen]
pub fn assert_console() {
    log("info");
    warn("warning");
}

// ─── §13 Location ───────────────────────────────────────────────────────────

#[wasm_bindgen]
pub fn assert_location() {
    let _h: String = get_hash();
    set_hash("section-2");
}

// ─── §14 Telemetry ──────────────────────────────────────────────────────────

#[wasm_bindgen]
pub fn assert_telemetry() {
    telemetry_init(TelemetryConfig {
        service_name: "my-app",
        endpoint: None,
        trace_reads: false,
    });
    {
        let _guard: SpanGuard = span("compute_layout");
    }
    let _spans: Vec<SpanRecord> = get_spans();
    clear_spans();
    let _stats: Stats = get_stats();

    telemetry::set_hook(reactivity_hook);
    telemetry::clear_hook();

    let _cfg = TelemetryConfig::default();
    let _ = SpanKind::Internal;
    let _ = SpanKind::Client;
}

#[wasm_bindgen]
pub async fn assert_traced_fetch() {
    let _ = traced_fetch("https://example.com").await;
}

// ─── §15 Resiliency ─────────────────────────────────────────────────────────

#[wasm_bindgen]
pub fn assert_resiliency_sync() {
    let _: web_sys::Element = error_boundary(
        || create_element("div"),
        |err| {
            view! { <p class="err">{err}</p> }
        },
    );
    let _: web_sys::Element = default_error_boundary(|| create_element("div"));

    let _cfg_exp = RetryConfig::exponential(3, 100);
    let _cfg_fixed = RetryConfig::fixed(5, 200);

    let breaker = CircuitBreaker::new(CircuitBreakerConfig {
        failure_threshold: 5,
        reset_timeout_ms: 30_000,
    });
    breaker.reset();

    let _ = CircuitState::Closed;
    let _ = CircuitState::Open;
    let _ = CircuitState::HalfOpen;

    let err: CircuitError<String> = CircuitError::Open;
    let _ = format!("{}", err);

    let err: RetryError<&str> = RetryError {
        attempts: 3,
        last_error: "timeout",
    };
    let _ = format!("{}", err);

    let to = TimeoutError { ms: 5000 };
    let _ = format!("{}", to);
}

#[wasm_bindgen]
pub async fn assert_resiliency_async() {
    sleep(100).await;

    let _: Result<i32, _> = with_timeout(5000, async { 42 }).await;

    let _: Result<i32, RetryError<&str>> =
        retry(RetryConfig::exponential(3, 1), || async { Ok::<i32, &str>(1) }).await;

    let breaker = CircuitBreaker::new(CircuitBreakerConfig {
        failure_threshold: 3,
        reset_timeout_ms: 5000,
    });
    let _: Result<i32, CircuitError<&str>> = breaker.call(|| async { Ok::<i32, &str>(1) }).await;
}

// ─── README front-page snippet ──────────────────────────────────────────────
//
// This is the EXACT block of code from the top of README.md (modulo the
// #[wasm_bindgen(start)] which would collide with other entry points here).
// If the README front page changes, this must compile.

fn _readme_counter_snippet() {
    mount("#app", || {
        let mut count = signal(0);
        view! {
            <div>
                <p>"Count: " {count}</p>
                <button on:click={move |_: bueler::dom::Event| count += 1}>
                    "Increment"
                </button>
            </div>
        }
    });
}
