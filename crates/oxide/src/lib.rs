/// Re-exports from `bueler-core` — reactive primitives.
pub use bueler_core::{batch, create_effect, memo, provide_context, signal, untrack, use_context, Signal};
pub use bueler_core::{set_hook, clear_hook, HookEvent};
pub use bueler_core::{watch, on_mount, on_cleanup};

/// Re-exports from `bueler-macros` — the `view!` macro.
pub use bueler_macros::view;

/// DOM renderer utilities.
pub mod dom {
    pub use bueler_dom::*;
}

/// Client-side router.
pub mod router {
    pub use bueler_router::*;
}

/// OpenTelemetry-compatible tracing.
pub mod telemetry {
    pub use bueler_telemetry::*;
}

/// Resiliency patterns — error boundaries, retry, circuit breaker, timeout.
pub mod resiliency {
    pub use bueler_resiliency::*;
}

/// Pre-built UI components — buttons, inputs, cards, modals, and more.
pub mod components {
    pub use bueler_components::*;
}

/// The Component trait — implement this for struct-based components.
///
/// ```ignore
/// struct Counter { initial: i32 }
///
/// impl bueler::Component for Counter {
///     fn render(self) -> web_sys::Element {
///         let count = signal(self.initial);
///         view! { <div>{count}</div> }
///     }
/// }
///
/// // In view!: <Counter initial={5} />
/// ```
pub trait Component {
    fn render(self) -> web_sys::Element;
}

/// Convenient glob import: `use bueler::prelude::*;`
pub mod prelude {
    pub use bueler_core::{batch, create_effect, memo, provide_context, signal, untrack, use_context, Signal};
    pub use bueler_dom::mount;
    pub use bueler_macros::view;
    pub use super::Component;
}
