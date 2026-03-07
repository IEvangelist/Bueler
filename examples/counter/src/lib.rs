use oxide::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
    mount("#app", || {
        let mut count = signal(0i32);

        view! {
            <div class="container">
                <h1>"Oxide Counter"</h1>
                <p class="count">"Count: " {count}</p>
                <button on:click={move |_: oxide::dom::Event| count += 1}>
                    "Increment"
                </button>
            </div>
        }
    });
}
