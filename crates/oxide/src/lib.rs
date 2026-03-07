/// Re-exports from `oxide-core` — reactive primitives.
pub use oxide_core::{batch, create_effect, signal, untrack, Signal};

/// Re-exports from `oxide-macros` — the `view!` macro.
pub use oxide_macros::view;

/// DOM renderer utilities.
pub mod dom {
    pub use oxide_dom::*;
}

/// Convenient glob import: `use oxide::prelude::*;`
pub mod prelude {
    pub use oxide_core::{batch, create_effect, signal, untrack, Signal};
    pub use oxide_dom::mount;
    pub use oxide_macros::view;
}
