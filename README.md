# 🔥 Oxide

**A Rust frontend framework that compiles to WebAssembly.**

Oxide lets you write reactive browser applications entirely in Rust — no JavaScript required. It uses fine-grained reactivity (like SolidJS) with direct DOM updates, zero virtual DOM overhead, and a JSX-like `view!` macro.

```rust
use oxide::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
    mount("#app", || {
        let mut count = signal(0);

        view! {
            <div>
                <p>"Count: " {count}</p>
                <button on:click={move |_: oxide::dom::Event| count += 1}>
                    "Increment"
                </button>
            </div>
        }
    });
}
```

## Architecture

```
oxide/
├── crates/
│   ├── oxide-core     # Reactive runtime — signals, effects, batching
│   ├── oxide-macros   # Proc macros — view! DSL
│   ├── oxide-dom      # DOM renderer — web-sys bindings
│   └── oxide          # Facade crate — re-exports everything
└── examples/
    └── counter        # Working counter demo
```

### Design Principles

| Principle | How |
|---|---|
| **Fine-grained reactivity** | Signals are `Copy` handles into a thread-local runtime. Effects auto-track dependencies. No virtual DOM — updates go straight to the real DOM. |
| **Minimal runtime** | No GC, no diffing, no scheduler. Just signals → effects → DOM mutations. |
| **Strong typing** | Props are Rust structs. Components are Rust functions. The compiler catches errors at build time. |
| **Small bundles** | Only `wasm-bindgen` + `web-sys` as runtime deps. No JS framework overhead. |

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (1.70+)
- `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/): `cargo install wasm-pack`

### Run the Counter Example

```sh
cd examples/counter
wasm-pack build --target web
# Serve index.html with any HTTP server:
python -m http.server 8080
# Open http://localhost:8080
```

### Create a New Project

Add `oxide` as a dependency:

```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
oxide = { git = "https://github.com/nicepine/Oxide" }
wasm-bindgen = "0.2"
```

## Core Concepts

### Signals

Signals are reactive values. Reading a signal inside an effect auto-subscribes it.

```rust
let count = signal(0);       // Create
let val = count.get();        // Read (tracks in effects)
count.set(5);                 // Write (notifies subscribers)
count.update(|n| *n += 1);   // Mutate in-place
```

`Signal<T>` is `Copy` — no `Rc`, no `clone()`, just pass it around.

### Effects

Effects re-run whenever their signal dependencies change:

```rust
create_effect(move || {
    log(&format!("Count is now: {}", count.get()));
});
```

### Batching

Batch multiple updates to coalesce effect runs:

```rust
batch(|| {
    a.set(1);
    b.set(2);
    // Effects run once after both updates, not twice.
});
```

### The `view!` Macro

JSX-like syntax that compiles to direct DOM API calls:

```rust
view! {
    <div class="card">
        <h1>"Hello"</h1>
        <p>{name}</p>
        <button on:click={move |_: oxide::dom::Event| count += 1}>
            "Click me"
        </button>
    </div>
}
```

- `"text"` — static text nodes
- `{expr}` — reactive expressions (auto-updates when signals change)
- `on:event={handler}` — event listeners
- `attr="value"` — static attributes

## License

MIT
