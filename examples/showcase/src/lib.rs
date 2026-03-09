use oxide::prelude::*;
use oxide::dom::*;
use oxide::{Signal, memo};
use oxide::telemetry;
use oxide::resiliency;
use oxide::router::{Router, RouterMode, route, navigate};
use oxide::components::{self, Severity, AvatarSize, DrawerSide};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

// ═══════════════════════════════════════════════════════════════════════════
// Entry point — SPA with hash-based routing
// ═══════════════════════════════════════════════════════════════════════════

#[wasm_bindgen(start)]
pub fn main() {
    mount("#app", || {
        let router = Router::new(RouterMode::Hash, &[
            route("/",                    page_landing),
            route("/playground",          page_playground_home),
            route("/demos",               page_demos),
            route("/components",          page_catalog),
            route("/components/button",   pg_button),
            route("/components/input",    pg_input),
            route("/components/textarea", pg_textarea),
            route("/components/select",   pg_select),
            route("/components/checkbox", pg_checkbox),
            route("/components/card",     pg_card),
            route("/components/alert",    pg_alert),
            route("/components/modal",    pg_modal),
            route("/components/spinner",  pg_spinner),
            route("/components/progress", pg_progress),
            route("/components/tabs",     pg_tabs),
            route("/components/badge",    pg_badge),
            route("/components/divider",  pg_divider),
            route("/components/skeleton", pg_skeleton),
            route("/components/avatar",          pg_avatar),
            route("/components/stat",            pg_stat),
            route("/components/tag",             pg_tag),
            route("/components/toggle",          pg_toggle),
            route("/components/radio",           pg_radio),
            route("/components/slider",          pg_slider),
            route("/components/search",          pg_search),
            route("/components/password",        pg_password),
            route("/components/number",          pg_number),
            route("/components/accordion",       pg_accordion),
            route("/components/breadcrumb",      pg_breadcrumb),
            route("/components/pagination",      pg_pagination),
            route("/components/dropdown",        pg_dropdown),
            route("/components/toast",           pg_toast),
            route("/components/drawer",          pg_drawer),
            route("/components/timeline",        pg_timeline),
            route("/components/table",           pg_table),
            route("/components/tooltip",         pg_tooltip),
            route("/components/rating",          pg_rating),
            route("/components/copy-button",     pg_copy_button),
            route("/components/empty-state",     pg_empty_state),
            route("/components/loading-overlay", pg_loading_overlay),
            route("/components/layout",          pg_layout),
            route("/components/kbd",             pg_kbd),
            route("/components/code-block",      pg_code_block),
            route("/components/file-upload",     pg_file_upload),
            route("/components/form-group",      pg_form_group),
            route("/forms",               page_forms),
            route("/composition",         page_composition),
            route("/tutorials/login",     tutorial_login),
            route("/tutorials/dashboard", tutorial_dashboard),
            route("/docs",                page_docs),
        ]);

        build_app_shell(router)
    });

    body().append_child(&components::scroll_to_top(300)).ok();
}

// ═══════════════════════════════════════════════════════════════════════════
// App Shell — header nav + router outlet
// ═══════════════════════════════════════════════════════════════════════════

fn build_app_shell(router: Router) -> web_sys::Element {
    let current_route = router.current();
    let shell = create_element("div");
    set_attribute(&shell, "class", "app-shell");

    // Header
    let header = create_element("header");
    set_attribute(&header, "class", "app-header");
    let header_container = create_element("div");
    set_attribute(&header_container, "class", "container");

    // Logo
    let logo = create_element("div");
    set_attribute(&logo, "class", "logo");
    let logo_a = create_element("a");
    let logo_a_ref = logo_a.clone();
    add_event_listener(&logo_a, "click", move |e: web_sys::Event| {
        e.prevent_default();
        navigate("/");
    });
    set_attribute(&logo_a_ref, "href", "#/");
    let logo_emoji = create_element("span");
    set_attribute(&logo_emoji, "class", "logo-emoji");
    append_text(&logo_emoji, "\u{1f525}");
    append_node(&logo_a, &logo_emoji);
    let logo_text = create_element("span");
    set_attribute(&logo_text, "class", "gradient-text");
    append_text(&logo_text, "Oxide");
    append_node(&logo_a, &logo_text);
    append_node(&logo, &logo_a);
    append_node(&header_container, &logo);

    // Hamburger toggle
    let nav_open = signal(false);
    let toggle_btn = create_element("button");
    set_attribute(&toggle_btn, "class", "nav-toggle");
    set_attribute(&toggle_btn, "aria-label", "Menu");
    for _ in 0..3 {
        let s = create_element("span");
        append_node(&toggle_btn, &s);
    }
    add_event_listener(&toggle_btn, "click", move |_| {
        nav_open.set(!nav_open.get());
    });
    append_node(&header_container, &toggle_btn);

    // Nav
    let nav = create_element("nav");
    set_attribute(&nav, "class", "app-nav");

    let nav_ref = nav.clone();
    create_effect(move || {
        if nav_open.get() {
            nav_ref.class_list().add_1("open").ok();
        } else {
            nav_ref.class_list().remove_1("open").ok();
        }
    });

    let nav_items: &[(&str, &str)] = &[
        ("Home", "/"),
        ("Playground", "/playground"),
        ("Components", "/components"),
        ("Docs", "/docs"),
        ("Tutorials", "/tutorials/login"),
    ];
    let mut nav_links: Vec<(web_sys::Element, String)> = Vec::new();
    for &(label, path) in nav_items {
        let a = create_element("a");
        set_attribute(&a, "href", &format!("#{}", path));
        append_text(&a, label);
        let p = path.to_string();
        let no = nav_open;
        add_event_listener(&a, "click", move |e: Event| {
            e.prevent_default();
            navigate(&p);
            no.set(false);
        });
        nav_links.push((a.clone(), path.to_string()));
        append_node(&nav, &a);
    }

    // Reactively highlight active nav link
    create_effect(move || {
        let route = current_route.get();
        for (link, path) in &nav_links {
            let is_active = if *path == "/" {
                route == "/"
            } else {
                route.starts_with(path)
            };
            if is_active {
                link.class_list().add_1("active").ok();
            } else {
                link.class_list().remove_1("active").ok();
            }
        }
    });

    // GitHub link
    let gh_link = create_element("a");
    set_attribute(&gh_link, "href", "https://github.com/IEvangelist/Oxide");
    set_attribute(&gh_link, "target", "_blank");
    set_attribute(&gh_link, "rel", "noopener");
    set_attribute(&gh_link, "class", "gh-btn");
    append_text(&gh_link, "GitHub");
    append_node(&nav, &gh_link);

    append_node(&header_container, &nav);
    append_node(&header, &header_container);
    append_node(&shell, &header);

    // Content area with router outlet
    let content = create_element("div");
    set_attribute(&content, "class", "app-content");
    append_node(&content, &router.view());
    append_node(&shell, &content);

    shell
}

// ═══════════════════════════════════════════════════════════════════════════
// Helpers (kept from original)
// ═══════════════════════════════════════════════════════════════════════════

fn el(tag: &str, class: &str, children: &[&web_sys::Element]) -> web_sys::Element {
    let e = create_element(tag);
    if !class.is_empty() {
        set_attribute(&e, "class", class);
    }
    for c in children {
        append_node(&e, c);
    }
    e
}

fn text_el(tag: &str, text: &str) -> web_sys::Element {
    let e = create_element(tag);
    append_text(&e, text);
    e
}

// ═══════════════════════════════════════════════════════════════════════════
// Page: Landing — marketing home page
// ═══════════════════════════════════════════════════════════════════════════

fn page_landing() -> web_sys::Element {
    let page = create_element("div");
    set_attribute(&page, "class", "landing");

    // ── Hero ──
    let hero = create_element("section");
    set_attribute(&hero, "class", "hero");

    // Floating orbs
    for (i, cls) in ["orb orb-1", "orb orb-2", "orb orb-3"].iter().enumerate() {
        let orb = create_element("div");
        set_attribute(&orb, "class", cls);
        let _ = i;
        append_node(&hero, &orb);
    }

    let hero_container = el("div", "container", &[]);

    // Hero text
    let hero_text = el("div", "hero-text", &[]);
    let h1 = create_element("h1");
    set_attribute(&h1, "class", "fade-in");
    append_text(&h1, "Build web apps in ");
    let br = create_element("br");
    append_node(&h1, &br);
    let rust_word = create_element("span");
    set_attribute(&rust_word, "class", "rust-word");
    append_text(&rust_word, "Rust.");
    append_node(&h1, &rust_word);
    append_node(&hero_text, &h1);

    let hero_p = create_element("p");
    set_attribute(&hero_p, "class", "fade-in d2");
    append_text(&hero_p, "Fine-grained reactivity. Zero JavaScript. Direct DOM manipulation. Compiles to WebAssembly.");
    append_node(&hero_text, &hero_p);

    let hero_buttons = el("div", "hero-buttons fade-in d3", &[]);
    let btn_get_started = create_element("a");
    set_attribute(&btn_get_started, "class", "hero-btn-primary");
    set_attribute(&btn_get_started, "href", "#/playground");
    append_text(&btn_get_started, "Get Started \u{2192}");
    add_event_listener(&btn_get_started, "click", |e: Event| { e.prevent_default(); navigate("/playground"); });
    append_node(&hero_buttons, &btn_get_started);

    let btn_components = create_element("a");
    set_attribute(&btn_components, "class", "hero-btn-outline");
    set_attribute(&btn_components, "href", "#/components");
    append_text(&btn_components, "See Components \u{2193}");
    add_event_listener(&btn_components, "click", |e: Event| { e.prevent_default(); navigate("/components"); });
    append_node(&hero_buttons, &btn_components);
    append_node(&hero_text, &hero_buttons);
    append_node(&hero_container, &hero_text);

    // Hero code block
    let hero_code = el("div", "hero-code fade-in d4", &[]);
    let code_header = el("div", "hero-code-header", &[]);
    for cls in &["dot dot-r", "dot dot-y", "dot dot-g"] {
        let d = create_element("div");
        set_attribute(&d, "class", cls);
        append_node(&code_header, &d);
    }
    let fname = create_element("span");
    append_text(&fname, "main.rs");
    append_node(&code_header, &fname);
    append_node(&hero_code, &code_header);

    let pre = create_element("pre");
    set_inner_html(&pre, r#"<span class="kw">use</span> oxide::prelude::*;

<span class="kw">let</span> count = <span class="fn">signal</span>(<span class="num">0</span>);

<span class="fn">view!</span> {
    &lt;<span class="tag-name">div</span>&gt;
        &lt;<span class="tag-name">p</span>&gt;<span class="str">"Count: "</span> {count}&lt;/<span class="tag-name">p</span>&gt;
        &lt;<span class="tag-name">button</span> <span class="attr">on:click</span>={<span class="kw">move</span> |_| count += <span class="num">1</span>}&gt;
            <span class="str">"Increment"</span>
        &lt;/<span class="tag-name">button</span>&gt;
    &lt;/<span class="tag-name">div</span>&gt;
}"#);
    append_node(&hero_code, &pre);
    append_node(&hero_container, &hero_code);
    append_node(&hero, &hero_container);
    append_node(&page, &hero);

    // ── Features ──
    let features_section = create_element("section");
    set_attribute(&features_section, "class", "features-section");
    let fc = el("div", "container", &[]);

    let fl = el("div", "section-label fade-in", &[]);
    append_text(&fl, "Why Oxide");
    append_node(&fc, &fl);

    let ft = create_element("h2");
    set_attribute(&ft, "class", "section-title fade-in d1");
    set_inner_html(&ft, "Everything you need.<br>Nothing you don't.");
    append_node(&fc, &ft);

    let fs = el("p", "section-sub fade-in d2", &[]);
    append_text(&fs, "A Rust-native web framework that compiles to WebAssembly with fine-grained reactivity, zero JavaScript overhead, and full type safety.");
    append_node(&fc, &fs);

    let fg = el("div", "features-grid", &[]);
    let features: &[(&str, &str, &str, &str)] = &[
        ("d2", "\u{1f3af}", "Fine-Grained Reactivity", "Signals track exactly what changes. No virtual DOM diffing. O(1) updates directly to the DOM nodes that need them."),
        ("d3", "\u{1f680}", "Zero JavaScript", "Write your entire frontend in Rust. Type-safe, memory-safe, compile-checked. The only JS is the WASM loader shim."),
        ("d4", "\u{1f4e6}", "Tiny Bundles", "30 KB hello world. 178 KB for a full 18-demo showcase. Tree-shaking at the Rust compiler level."),
        ("d5", "\u{26a1}", "Modern DX", "JSX-like view! macro with if/else, for loops, bind:value, class toggling. Feels like React, runs like C."),
        ("d6", "\u{1f6e1}\u{fe0f}", "Type Safe", "Catch bugs at compile time, not in production. Strongly-typed signals, props, and events. No runtime surprises."),
        ("d7", "\u{1f527}", "Full Web API Access", "Canvas, Fetch, LocalStorage, Drag & Drop, Clipboard, SVG, Audio \u{2014} every browser API, from Rust."),
    ];
    for &(delay, icon, title, desc) in features {
        let card = create_element("div");
        set_attribute(&card, "class", &format!("feature-card fade-in {}", delay));
        let ic = el("div", "feature-icon", &[]);
        append_text(&ic, icon);
        append_node(&card, &ic);
        let h3 = text_el("h3", title);
        append_node(&card, &h3);
        let p = create_element("p");
        append_text(&p, desc);
        append_node(&card, &p);
        append_node(&fg, &card);
    }
    append_node(&fc, &fg);
    append_node(&features_section, &fc);
    append_node(&page, &features_section);

    // ── CLI Section ──
    let cli_section = create_element("section");
    set_attribute(&cli_section, "class", "cli-section");
    let cc = el("div", "container", &[]);

    let cl = el("div", "section-label fade-in", &[]);
    append_text(&cl, "Developer Experience");
    append_node(&cc, &cl);
    let ct = create_element("h2");
    set_attribute(&ct, "class", "section-title fade-in d1");
    append_text(&ct, "One CLI to rule them all.");
    append_node(&cc, &ct);
    let cs = el("p", "section-sub fade-in d2", &[]);
    append_text(&cs, "From scaffold to production in three commands. The Oxide CLI handles the complexity so you can focus on building.");
    append_node(&cc, &cs);

    // Install block
    let install_block = create_element("div");
    set_attribute(&install_block, "class", "code-block fade-in d2");
    set_attribute(&install_block, "style", "margin: 1.5rem 0;");
    let install_label = create_element("div");
    set_attribute(&install_label, "style", "font-size: 0.75rem; color: var(--fg3); margin-bottom: 0.5rem;");
    append_text(&install_label, "Install the CLI");
    append_node(&install_block, &install_label);
    let install_pre = create_element("pre");
    set_attribute(&install_pre, "style", "background: var(--bg); padding: 0.8rem 1rem; border-radius: 8px; border: 1px solid var(--fg3); overflow-x: auto; font-size: 0.85rem;");
    set_inner_html(&install_pre, r#"<span class="prompt">$</span> cargo install --git https://github.com/IEvangelist/Oxide oxide-cli"#);
    append_node(&install_block, &install_pre);
    append_node(&cc, &install_block);

    let cli_grid = el("div", "cli-grid", &[]);
    let cli_cards: &[(&str, &str, &str)] = &[
        ("oxide new my-app", "\u{2713} Created project my-app\n\u{2713} Added oxide dependencies\n\u{2713} Configured wasm32 target\n\u{2713} Generated index.html\n\n\u{2192} cd my-app && oxide dev", "d3"),
        ("oxide dev", "\u{1f525} Oxide dev server v0.1.0\n  Compiling my-app...\n  Finished in 1.2s\n  Serving on http://localhost:3000\n  Live reload enabled\n  DWARF debug info \u{2713}", "d4"),
        ("oxide build", "  Compiling my-app (release)...\n  Optimizing WASM...\n  Running wasm-opt -Oz\n  Finished in 3.8s\n\n  dist/app.wasm \u{2014} 30 KB gzipped", "d5"),
    ];
    for &(cmd, output, delay) in cli_cards {
        let card = create_element("div");
        set_attribute(&card, "class", &format!("cli-card fade-in {}", delay));
        let header = el("div", "cli-card-header", &[]);
        for c in &["dot dot-r", "dot dot-y", "dot dot-g"] {
            let d = create_element("div");
            set_attribute(&d, "class", c);
            append_node(&header, &d);
        }
        append_node(&card, &header);
        let body = el("div", "cli-card-body", &[]);
        let cmd_div = el("div", "cli-cmd", &[]);
        let prompt = create_element("span");
        set_attribute(&prompt, "class", "prompt");
        append_text(&prompt, "$ ");
        append_node(&cmd_div, &prompt);
        append_text(&cmd_div, cmd);
        append_node(&body, &cmd_div);
        let out_div = el("div", "cli-output", &[]);
        // Convert newlines to <br>
        set_inner_html(&out_div, &output.replace('\n', "<br>"));
        append_node(&body, &out_div);
        append_node(&card, &body);
        append_node(&cli_grid, &card);
    }
    append_node(&cc, &cli_grid);

    let cli_note = el("div", "cli-note fade-in d6", &[]);
    set_inner_html(&cli_note, r#"<strong>💡 Debug like a pro:</strong> <code>oxide dev</code> embeds DWARF debug info. Install Chrome's "C/C++ DevTools Support (DWARF)" extension to set breakpoints and step through your Rust source directly in the browser."#);
    append_node(&cc, &cli_note);
    append_node(&cli_section, &cc);
    append_node(&page, &cli_section);

    // ── Stats ──
    let stats_section = create_element("section");
    set_attribute(&stats_section, "class", "stats-section");
    let sc = el("div", "container", &[]);
    let sr = el("div", "stats-row", &[]);
    let stats: &[(&str, &str, &str)] = &[
        ("30 KB", "Hello World Bundle", "d1"),
        ("18", "Interactive Demos", "d2"),
        ("0", "Lines of JavaScript", "d3"),
        ("9", "Framework Crates", "d4"),
    ];
    for &(num, label, delay) in stats {
        let card = create_element("div");
        set_attribute(&card, "class", &format!("stat-card fade-in {}", delay));
        let n = el("div", "stat-num", &[]);
        append_text(&n, num);
        append_node(&card, &n);
        let l = el("div", "stat-label", &[]);
        append_text(&l, label);
        append_node(&card, &l);
        append_node(&sr, &card);
    }
    append_node(&sc, &sr);
    append_node(&stats_section, &sc);
    append_node(&page, &stats_section);

    // ── Production Ready ──
    let prod_section = create_element("section");
    set_attribute(&prod_section, "class", "production-section");
    let pc = el("div", "container", &[]);
    let pl = el("div", "section-label fade-in", &[]);
    append_text(&pl, "Production Grade");
    append_node(&pc, &pl);
    let pt = create_element("h2");
    set_attribute(&pt, "class", "section-title fade-in d1");
    set_inner_html(&pt, "Battle-tested.<br>Observable. Resilient.");
    append_node(&pc, &pt);
    let ps = el("p", "section-sub fade-in d2", &[]);
    append_text(&ps, "Built-in observability and fault tolerance. No extra dependencies required.");
    append_node(&pc, &ps);

    let pg = el("div", "production-grid", &[]);
    let prod_cards: &[(&str, &str, &str, &str)] = &[
        ("d3", "\u{1f4e1}", "OpenTelemetry Built-in", "Automatic tracing of signals, effects, and fetches. W3C trace context propagation. Zero overhead when disabled."),
        ("d4", "\u{1f6e1}\u{fe0f}", "Resiliency Framework", "Error boundaries, retry with backoff, circuit breakers, async timeouts. Production-grade fault tolerance."),
    ];
    for &(delay, icon, title, desc) in prod_cards {
        let card = create_element("div");
        set_attribute(&card, "class", &format!("feature-card fade-in {}", delay));
        let ic = el("div", "feature-icon", &[]);
        append_text(&ic, icon);
        append_node(&card, &ic);
        let h3 = text_el("h3", title);
        append_node(&card, &h3);
        let p = create_element("p");
        append_text(&p, desc);
        append_node(&card, &p);
        append_node(&pg, &card);
    }
    append_node(&pc, &pg);
    append_node(&prod_section, &pc);
    append_node(&page, &prod_section);

    // ── Benchmarks ──
    let bench_section = create_element("section");
    set_attribute(&bench_section, "class", "benchmarks-section");
    let bc = el("div", "container", &[]);
    let bl = el("div", "section-label fade-in", &[]);
    append_text(&bl, "Performance");
    append_node(&bc, &bl);
    let bt_h2 = create_element("h2");
    set_attribute(&bt_h2, "class", "section-title fade-in d1");
    set_inner_html(&bt_h2, "Built for speed.<br>Measured to prove it.");
    append_node(&bc, &bt_h2);
    let bs = el("p", "section-sub fade-in d2", &[]);
    append_text(&bs, "See how Oxide stacks up against the most popular web frameworks.");
    append_node(&bc, &bs);

    let bg = el("div", "bench-grid", &[]);

    // Bar chart
    let bar_div = create_element("div");
    set_attribute(&bar_div, "class", "fade-in d3");
    let bar_title = create_element("h3");
    set_attribute(&bar_title, "style", "font-size:1rem;font-weight:700;margin-bottom:1.25rem;");
    append_text(&bar_title, "Bundle Size (Hello World, gzipped)");
    append_node(&bar_div, &bar_title);

    let bar_chart = el("div", "bar-chart", &[]);
    let bars: &[(&str, &str, bool)] = &[
        ("Svelte", "3 KB", false), ("Solid", "7 KB", false),
        ("Oxide", "15 KB", true), ("Leptos", "25 KB", false),
        ("Vue", "33 KB", false), ("React", "42 KB", false),
        ("Yew", "50 KB", false),
    ];
    let widths = ["6%", "14%", "30%", "50%", "66%", "84%", "100%"];
    for (i, &(name, size, is_oxide)) in bars.iter().enumerate() {
        let row = el("div", "bar-row", &[]);
        let fw = el("div", "bar-framework", &[]);
        if is_oxide {
            set_attribute(&fw, "style", "color:var(--accent);font-weight:700;");
        }
        append_text(&fw, name);
        append_node(&row, &fw);
        let track = el("div", "bar-track", &[]);
        let fill = create_element("div");
        let fill_class = if is_oxide { "bar-fill oxide-bar" } else { "bar-fill" };
        set_attribute(&fill, "class", fill_class);
        set_attribute(&fill, "style", &format!("width:{}", widths[i]));
        let sz = create_element("span");
        set_attribute(&sz, "class", "bar-size");
        append_text(&sz, size);
        append_node(&fill, &sz);
        append_node(&track, &fill);
        append_node(&row, &track);
        append_node(&bar_chart, &row);
    }
    append_node(&bar_div, &bar_chart);
    append_node(&bg, &bar_div);

    // Comparison table
    let table_div = create_element("div");
    set_attribute(&table_div, "class", "fade-in d4");
    let table_title = create_element("h3");
    set_attribute(&table_title, "style", "font-size:1rem;font-weight:700;margin-bottom:1.25rem;");
    append_text(&table_title, "Feature Comparison");
    append_node(&table_div, &table_title);
    let tw = el("div", "table-wrap", &[]);
    let table = create_element("table");
    set_attribute(&table, "class", "comparison-table");
    set_inner_html(&table, r#"<thead><tr><th>Feature</th><th>React</th><th>Vue</th><th>Svelte</th><th>Solid</th><th>Leptos</th><th class="col-oxide">Oxide</th></tr></thead>
<tbody>
<tr><td>Language</td><td>JS/TS</td><td>JS/TS</td><td>JS/TS</td><td>JS/TS</td><td>Rust</td><td class="col-oxide">Rust</td></tr>
<tr><td>Reactivity</td><td>VDOM</td><td>Proxy</td><td>Compiler</td><td>Signals</td><td>Signals</td><td class="col-oxide">Signals</td></tr>
<tr><td>Bundle Size</td><td>42 KB</td><td>33 KB</td><td>3 KB</td><td>7 KB</td><td>25 KB</td><td class="col-oxide">15 KB</td></tr>
<tr><td>Type Safety</td><td>Optional</td><td>Optional</td><td>Optional</td><td>Optional</td><td>Native</td><td class="col-oxide">Native</td></tr>
<tr><td>Memory Safety</td><td class="check-no">No</td><td class="check-no">No</td><td class="check-no">No</td><td class="check-no">No</td><td class="check-yes">Yes</td><td class="col-oxide check-yes">Yes</td></tr>
<tr><td>WASM Native</td><td class="check-no">No</td><td class="check-no">No</td><td class="check-no">No</td><td class="check-no">No</td><td class="check-yes">Yes</td><td class="col-oxide check-yes">Yes</td></tr>
<tr><td>Two-way Bind</td><td class="check-no">❌</td><td class="check-yes">✅</td><td class="check-yes">✅</td><td class="check-no">❌</td><td class="check-yes">✅</td><td class="col-oxide check-yes">✅</td></tr>
<tr><td>Cond. Render</td><td class="check-yes">✅</td><td class="check-yes">✅</td><td class="check-yes">✅</td><td class="check-yes">✅</td><td class="check-yes">✅</td><td class="col-oxide check-yes">✅</td></tr>
</tbody>"#);
    append_node(&tw, &table);
    append_node(&table_div, &tw);
    append_node(&bg, &table_div);
    append_node(&bc, &bg);
    append_node(&bench_section, &bc);
    append_node(&page, &bench_section);

    // ── Featured Demos (static code previews) ──
    let demos_section = create_element("section");
    set_attribute(&demos_section, "class", "demos-section");
    let dc = el("div", "container", &[]);
    let dl = el("div", "section-label fade-in", &[]);
    append_text(&dl, "Interactive");
    append_node(&dc, &dl);
    let dt = create_element("h2");
    set_attribute(&dt, "class", "section-title fade-in d1");
    append_text(&dt, "Try It Live");
    append_node(&dc, &dt);
    let ds = el("p", "section-sub fade-in d2", &[]);
    append_text(&ds, "Every demo runs entirely in Rust compiled to WebAssembly. Zero JavaScript.");
    append_node(&dc, &ds);

    let df = el("div", "demo-featured", &[]);

    // Counter preview card
    let counter_card = create_element("div");
    set_attribute(&counter_card, "class", "demo-card fade-in d3");
    let counter_code = el("div", "demo-code-panel", &[]);
    let counter_title = el("div", "demo-code-title", &[]);
    append_text(&counter_title, "Counter \u{2014} Source");
    append_node(&counter_code, &counter_title);
    let counter_pre = create_element("pre");
    set_inner_html(&counter_pre, r#"<span class="kw">let mut</span> count = <span class="fn">signal</span>(<span class="num">0</span>);

<span class="fn">view!</span> {
    &lt;<span class="tag-name">div</span>&gt;
        &lt;<span class="tag-name">p</span> <span class="attr">class</span>=<span class="str">"big-num"</span>&gt;{count}&lt;/<span class="tag-name">p</span>&gt;
        &lt;<span class="tag-name">div</span> <span class="attr">class</span>=<span class="str">"row"</span>&gt;
            &lt;<span class="tag-name">button</span> <span class="attr">on:click</span>={<span class="kw">move</span> |_| count -= <span class="num">1</span>}&gt;<span class="str">"−"</span>&lt;/<span class="tag-name">button</span>&gt;
            &lt;<span class="tag-name">button</span> <span class="attr">on:click</span>={<span class="kw">move</span> |_| count.<span class="fn">set</span>(<span class="num">0</span>)}&gt;<span class="str">"Reset"</span>&lt;/<span class="tag-name">button</span>&gt;
            &lt;<span class="tag-name">button</span> <span class="attr">on:click</span>={<span class="kw">move</span> |_| count += <span class="num">1</span>}&gt;<span class="str">"+"</span>&lt;/<span class="tag-name">button</span>&gt;
        &lt;/<span class="tag-name">div</span>&gt;
    &lt;/<span class="tag-name">div</span>&gt;
}"#);
    append_node(&counter_code, &counter_pre);
    append_node(&counter_card, &counter_code);
    let counter_live = el("div", "demo-live-panel", &[]);
    let counter_live_title = el("div", "demo-live-title", &[]);
    let live_dot1 = el("span", "live-dot", &[]);
    append_node(&counter_live_title, &live_dot1);
    append_text(&counter_live_title, " Live Demo");
    append_node(&counter_live, &counter_live_title);
    let counter_demo = create_element("div");
    append_node(&counter_demo, &demo_counter());
    append_node(&counter_live, &counter_demo);
    append_node(&counter_card, &counter_live);
    append_node(&df, &counter_card);

    // Todo preview card
    let todo_card = create_element("div");
    set_attribute(&todo_card, "class", "demo-card fade-in d4");
    let todo_code = el("div", "demo-code-panel", &[]);
    let todo_title = el("div", "demo-code-title", &[]);
    append_text(&todo_title, "Todo List \u{2014} Source");
    append_node(&todo_code, &todo_title);
    let todo_pre = create_element("pre");
    set_inner_html(&todo_pre, r#"<span class="kw">let</span> todos = <span class="fn">signal</span>(<span class="fn">vec!</span>[...]);
<span class="kw">let</span> input = <span class="fn">signal</span>(<span class="type">String</span>::<span class="fn">new</span>());

<span class="fn">view!</span> {
    &lt;<span class="tag-name">input</span> <span class="attr">bind:value</span>={input} /&gt;
    &lt;<span class="tag-name">ul</span>&gt;
        {<span class="kw">for</span> (text, done) <span class="kw">in</span> todos.<span class="fn">get</span>() {
            &lt;<span class="tag-name">li</span> <span class="attr">class:done</span>={done}&gt;
                {text}
            &lt;/<span class="tag-name">li</span>&gt;
        }}
    &lt;/<span class="tag-name">ul</span>&gt;
}"#);
    append_node(&todo_code, &todo_pre);
    append_node(&todo_card, &todo_code);
    let todo_live = el("div", "demo-live-panel", &[]);
    let todo_live_title = el("div", "demo-live-title", &[]);
    let live_dot2 = el("span", "live-dot", &[]);
    append_node(&todo_live_title, &live_dot2);
    append_text(&todo_live_title, " Live Demo");
    append_node(&todo_live, &todo_live_title);
    let todo_demo = create_element("div");
    append_node(&todo_demo, &demo_todo());
    append_node(&todo_live, &todo_demo);
    append_node(&todo_card, &todo_live);
    append_node(&df, &todo_card);

    append_node(&dc, &df);

    // CTA buttons
    let cta_row = create_element("div");
    set_attribute(&cta_row, "style", "display:flex;gap:1rem;justify-content:center;flex-wrap:wrap;margin-top:2.5rem;");
    let cta_links: &[(&str, &str)] = &[
        ("Component Library \u{2192}", "/components"),
        ("All 18 Demos \u{2192}", "/demos"),
        ("Form Patterns \u{2192}", "/forms"),
    ];
    for &(label, path) in cta_links {
        let a = create_element("a");
        set_attribute(&a, "class", "see-all");
        set_attribute(&a, "style", "margin-top:0;");
        append_text(&a, label);
        let p = path.to_string();
        add_event_listener(&a, "click", move |_| { navigate(&p); });
        append_node(&cta_row, &a);
    }
    append_node(&dc, &cta_row);
    append_node(&demos_section, &dc);
    append_node(&page, &demos_section);

    // ── Footer ──
    let footer = create_element("footer");
    set_attribute(&footer, "class", "site-footer");
    let footer_c = el("div", "container", &[]);
    let fb = el("div", "footer-brand", &[]);
    append_text(&fb, "Built with \u{1f525} by the Oxide community");
    append_node(&footer_c, &fb);

    let fl_div = el("div", "footer-links", &[]);
    let pg_link = create_element("a");
    set_attribute(&pg_link, "href", "#/playground");
    append_text(&pg_link, "Playground");
    add_event_listener(&pg_link, "click", |e: Event| { e.prevent_default(); navigate("/playground"); });
    append_node(&fl_div, &pg_link);
    let docs_a = create_element("a");
    set_attribute(&docs_a, "href", "#/docs");
    append_text(&docs_a, "Docs");
    add_event_listener(&docs_a, "click", |e: Event| { e.prevent_default(); navigate("/docs"); });
    append_node(&fl_div, &docs_a);
    let gh_a = create_element("a");
    set_attribute(&gh_a, "href", "https://github.com/IEvangelist/Oxide");
    set_attribute(&gh_a, "target", "_blank");
    set_attribute(&gh_a, "rel", "noopener");
    append_text(&gh_a, "GitHub");
    append_node(&fl_div, &gh_a);
    append_node(&footer_c, &fl_div);

    let ft_div = el("div", "footer-tagline", &[]);
    append_text(&ft_div, "Rust + WebAssembly = The Future of the Web");
    append_node(&footer_c, &ft_div);
    append_node(&footer, &footer_c);
    append_node(&page, &footer);

    page
}

// ═══════════════════════════════════════════════════════════════════════════
// Page: Playground Home
// ═══════════════════════════════════════════════════════════════════════════

fn page_playground_home() -> web_sys::Element {
    let page = el("div", "pg-page", &[]);

    let hero = el("div", "home-hero", &[]);
    let h1 = text_el("h1", "\u{1f525} Oxide Playground");
    append_node(&hero, &h1);
    let sub = text_el("p", "Explore interactive demos and component playgrounds — all running in Rust compiled to WebAssembly.");
    append_node(&hero, &sub);

    let cards = el("div", "home-cards", &[]);

    // Demos card
    let demo_card = el("div", "home-card", &[]);
    let demo_h3 = text_el("h3", "18 Interactive Demos \u{2192}");
    let demo_p = text_el("p", "Counter, Todo, Canvas, Charts, Drag & Drop and more — all Rust/WASM.");
    append_node(&demo_card, &demo_h3);
    append_node(&demo_card, &demo_p);
    add_event_listener(&demo_card, "click", |_| { navigate("/demos"); });
    append_node(&cards, &demo_card);

    // Components card
    let comp_card = el("div", "home-card", &[]);
    let comp_h3 = text_el("h3", "Component Library \u{2192}");
    let comp_p = text_el("p", "Buttons, Inputs, Modals, Tabs, Progress and more with live playgrounds.");
    append_node(&comp_card, &comp_h3);
    append_node(&comp_card, &comp_p);
    add_event_listener(&comp_card, "click", |_| { navigate("/components"); });
    append_node(&cards, &comp_card);

    append_node(&hero, &cards);
    append_node(&page, &hero);
    page
}

// ═══════════════════════════════════════════════════════════════════════════
// Page: Demos — all 18 demos on one scrollable page
// ═══════════════════════════════════════════════════════════════════════════

fn page_demos() -> web_sys::Element {
    let page = el("div", "pg-page", &[]);

    let h2 = text_el("h2", "Interactive Demos");
    append_node(&page, &h2);
    let desc = el("p", "pg-desc", &[]);
    append_text(&desc, "All 18 demos running in Rust \u{2192} WebAssembly. Zero JavaScript.");
    append_node(&page, &desc);

    let demos: &[(&str, fn() -> web_sys::Element)] = &[
        ("\u{1f522} Counter",           demo_counter),
        ("\u{1f321}\u{fe0f} Temperature Converter", demo_temperature),
        ("\u{2705} Todo List",          demo_todo),
        ("\u{23f1}\u{fe0f} Stopwatch",  demo_stopwatch),
        ("\u{1f4dd} Form Playground",   demo_forms),
        ("\u{1f310} Fetch API",         demo_fetch),
        ("\u{1f5b1}\u{fe0f} Mouse Tracker", demo_mouse),
        ("\u{2328}\u{fe0f} Keyboard Events", demo_keyboard),
        ("\u{1f3a8} Canvas Drawing",    demo_canvas),
        ("\u{1f3a8} Theme Toggle",      demo_theme),
        ("\u{1f4dd} Persistent Notes",  demo_notes),
        ("\u{1f3ac} Bouncing Ball",     demo_animation),
        ("\u{1f4ca} SVG Bar Chart",     demo_chart),
        ("\u{1fa9f} Modal Dialog",      demo_modal),
        ("\u{1fac3} Drag & Drop",       demo_dnd),
        ("\u{1f4cb} Clipboard",         demo_clipboard),
        ("\u{1f4e1} Telemetry",         demo_telemetry),
        ("\u{1f6e1}\u{fe0f} Resiliency", demo_resiliency),
    ];

    for &(name, builder) in demos {
        let section = el("div", "demo-section", &[]);
        let h3 = text_el("h3", name);
        append_node(&section, &h3);
        append_node(&section, &builder());
        append_node(&page, &section);
    }

    page
}

// ═══════════════════════════════════════════════════════════════════════════
// Page: Component Catalog
// ═══════════════════════════════════════════════════════════════════════════

fn page_catalog() -> web_sys::Element {
    let page = el("div", "pg-page", &[]);
    append_node(&page, &text_el("h2", "Component Library"));
    let desc = el("p", "pg-desc", &[]);
    append_text(&desc, "48 pre-built components with interactive playgrounds. Click any card to explore.");
    append_node(&page, &desc);

    let categories: &[(&str, &[(&str, &str, &str, &str)])] = &[
        ("Forms", &[
            ("\u{1f518}", "Button", "Versatile button with variants and states", "/components/button"),
            ("\u{270f}\u{fe0f}", "TextInput", "Text input with label and validation", "/components/input"),
            ("\u{1f4c4}", "TextArea", "Multi-line text input", "/components/textarea"),
            ("\u{1f53d}", "Select", "Dropdown select", "/components/select"),
            ("\u{2611}\u{fe0f}", "Checkbox", "Checkbox with label", "/components/checkbox"),
            ("\u{1f504}", "Toggle", "On/off switch", "/components/toggle"),
            ("\u{1f518}", "Radio", "Radio button group", "/components/radio"),
            ("\u{1f39a}\u{fe0f}", "Slider", "Range slider input", "/components/slider"),
            ("\u{1f50d}", "Search", "Search input field", "/components/search"),
            ("\u{1f512}", "Password", "Password with show/hide", "/components/password"),
            ("\u{1f522}", "Number", "Number input with +/-", "/components/number"),
            ("\u{1f4c2}", "FileUpload", "File upload drop zone", "/components/file-upload"),
            ("\u{1f4cb}", "FormGroup", "Form group wrapper", "/components/form-group"),
        ]),
        ("Layout", &[
            ("\u{1f4d0}", "Layout", "hstack, vstack, grid, center, spacer", "/components/layout"),
            ("\u{1f4c7}", "Card", "Container with header/body/footer", "/components/card"),
            ("\u{2796}", "Divider", "Horizontal separator", "/components/divider"),
            ("\u{1f4c1}", "Tabs", "Tabbed interface", "/components/tabs"),
            ("\u{1f4d6}", "Accordion", "Collapsible sections", "/components/accordion"),
        ]),
        ("Data Display", &[
            ("\u{1f464}", "Avatar", "User avatar with initials/image", "/components/avatar"),
            ("\u{1f4c8}", "Stat", "Large stat with label", "/components/stat"),
            ("\u{1f3f7}\u{fe0f}", "Tag", "Removable tag", "/components/tag"),
            ("\u{1f3f7}\u{fe0f}", "Badge", "Colored label badge", "/components/badge"),
            ("\u{1f4cb}", "Table", "Data table with headers", "/components/table"),
            ("\u{1f4c5}", "Timeline", "Vertical timeline", "/components/timeline"),
            ("\u{1f4dd}", "CodeBlock", "Styled code display", "/components/code-block"),
            ("\u{2328}\u{fe0f}", "Kbd", "Keyboard shortcut display", "/components/kbd"),
            ("\u{1f4ac}", "Tooltip", "Hover tooltip", "/components/tooltip"),
            ("\u{1f4a0}", "Skeleton", "Loading placeholder", "/components/skeleton"),
        ]),
        ("Feedback", &[
            ("\u{1f514}", "Alert", "Notification with severity", "/components/alert"),
            ("\u{1f514}", "Toast", "Auto-dismiss notification", "/components/toast"),
            ("\u{1fa9f}", "Modal", "Overlay dialog", "/components/modal"),
            ("\u{1f5c2}\u{fe0f}", "Drawer", "Side panel overlay", "/components/drawer"),
            ("\u{1f504}", "Spinner", "Loading indicator", "/components/spinner"),
            ("\u{1f4ca}", "Progress", "Progress bar", "/components/progress"),
            ("\u{1f4ed}", "EmptyState", "No data placeholder", "/components/empty-state"),
            ("\u{23f3}", "LoadingOverlay", "Full-screen loading", "/components/loading-overlay"),
            ("\u{1f514}", "ConfirmDialog", "Confirmation dialog", ""),
        ]),
        ("Navigation", &[
            ("\u{1f4cd}", "Breadcrumb", "Navigation trail", "/components/breadcrumb"),
            ("\u{1f4c4}", "Pagination", "Page navigation", "/components/pagination"),
            ("\u{2b07}\u{fe0f}", "Dropdown", "Click dropdown menu", "/components/dropdown"),
            ("\u{2b50}", "Rating", "Interactive star rating", "/components/rating"),
            ("\u{1f4cb}", "CopyButton", "Copy to clipboard", "/components/copy-button"),
            ("\u{2b06}\u{fe0f}", "ScrollToTop", "Scroll to top button", ""),
        ]),
    ];

    for &(category, items) in categories {
        let cat_heading = create_element("h3");
        set_attribute(&cat_heading, "class", "catalog-category");
        append_text(&cat_heading, category);
        append_node(&page, &cat_heading);

        let grid = el("div", "catalog-grid", &[]);
        for &(icon, name, desc_text, path) in items {
            let card = el("div", "catalog-card", &[]);
            let ic = el("div", "cc-icon", &[]);
            append_text(&ic, icon);
            append_node(&card, &ic);
            let nm = el("div", "cc-name", &[]);
            append_text(&nm, name);
            append_node(&card, &nm);
            let ds = el("div", "cc-desc", &[]);
            append_text(&ds, desc_text);
            append_node(&card, &ds);
            if !path.is_empty() {
                let p = path.to_string();
                add_event_listener(&card, "click", move |_| { navigate(&p); });
                set_attribute(&card, "style", "cursor:pointer");
            }
            append_node(&grid, &card);
        }
        append_node(&page, &grid);
    }

    page
}

// ═══════════════════════════════════════════════════════════════════════════
// Playground helper: builds a standard playground page layout
// ═══════════════════════════════════════════════════════════════════════════

fn pg_shell(title: &str, description: &str) -> (web_sys::Element, web_sys::Element, web_sys::Element, web_sys::Element) {
    let page = el("div", "pg-page", &[]);

    // Back link
    let back = el("a", "pg-back", &[]);
    set_attribute(&back, "href", "#/components");
    append_text(&back, "\u{2190} Components");
    add_event_listener(&back, "click", |e: Event| { e.prevent_default(); navigate("/components"); });
    append_node(&page, &back);

    let h2 = text_el("h2", title);
    append_node(&page, &h2);
    let desc = el("p", "pg-desc", &[]);
    append_text(&desc, description);
    append_node(&page, &desc);

    let layout = el("div", "pg-layout", &[]);
    let preview = el("div", "pg-preview", &[]);
    let controls = el("div", "pg-controls", &[]);
    append_node(&layout, &preview);
    append_node(&layout, &controls);
    append_node(&page, &layout);

    (page, preview, controls, layout)
}

fn api_table(props: &[(&str, &str, &str, &str)]) -> web_sys::Element {
    let wrap = create_element("div");
    set_attribute(&wrap, "class", "pg-api");
    let title = create_element("h3");
    append_text(&title, "API Reference");
    append_node(&wrap, &title);

    let table = create_element("table");
    set_attribute(&table, "class", "api-table");
    let thead = create_element("thead");
    let hr = create_element("tr");
    for h in &["Property", "Type", "Default", "Description"] {
        let th = create_element("th");
        append_text(&th, h);
        append_node(&hr, &th);
    }
    append_node(&thead, &hr);
    append_node(&table, &thead);

    let tbody = create_element("tbody");
    for &(name, typ, default, desc) in props {
        let tr = create_element("tr");
        let td_name = create_element("td");
        let code = create_element("code");
        append_text(&code, name);
        append_node(&td_name, &code);
        append_node(&tr, &td_name);

        let td_type = create_element("td");
        append_text(&td_type, typ);
        append_node(&tr, &td_type);

        let td_def = create_element("td");
        let code2 = create_element("code");
        append_text(&code2, default);
        append_node(&td_def, &code2);
        append_node(&tr, &td_def);

        let td_desc = create_element("td");
        append_text(&td_desc, desc);
        append_node(&tr, &td_desc);

        append_node(&tbody, &tr);
    }
    append_node(&table, &tbody);
    append_node(&wrap, &table);
    wrap
}

// ═══════════════════════════════════════════════════════════════════════════
// Component Playground: Button
// ═══════════════════════════════════════════════════════════════════════════

fn pg_button() -> web_sys::Element {
    let (page, preview, controls, _layout) = pg_shell(
        "Button",
        "Versatile button with multiple variants, sizes, and states.",
    );

    let label = signal("Click me".to_string());
    let variant = signal("primary".to_string());
    let size = signal("medium".to_string());
    let loading = signal(false);
    let disabled = signal(false);

    // Controls
    append_node(&controls, &components::text_input("Label").placeholder("Button text").bind(label).build());
    append_node(&controls, &components::select("Variant", &[
        ("primary", "Primary"), ("outline", "Outline"), ("danger", "Danger"), ("ghost", "Ghost"), ("default", "Default"),
    ], variant));
    append_node(&controls, &components::select("Size", &[
        ("small", "Small"), ("medium", "Medium"), ("large", "Large"),
    ], size));
    append_node(&controls, &components::checkbox("Loading", loading));
    append_node(&controls, &components::checkbox("Disabled", disabled));

    // Preview
    let preview_ref = preview.clone();
    create_effect(move || {
        clear_children(&preview_ref);
        let l = label.get();
        let mut b = components::button(&l);
        match variant.get().as_str() {
            "primary"   => { b = b.primary(); }
            "outline"   => { b = b.outline(); }
            "danger"    => { b = b.danger(); }
            "ghost"     => { b = b.ghost(); }
            _           => {}
        }
        match size.get().as_str() {
            "small" => { b = b.small(); }
            "large" => { b = b.large(); }
            _       => {}
        }
        b = b.loading(loading.get());
        b = b.disabled(disabled.get());
        append_node(&preview_ref, &b.build());
    });

    // Code block
    let code = create_element("pre");
    set_attribute(&code, "class", "pg-code");
    let code_ref = code.clone();
    create_effect(move || {
        let l = label.get();
        let v = variant.get();
        let s = size.get();
        let ld = loading.get();
        let dis = disabled.get();

        let mut chain = format!("button(\"{}\")", l);
        chain.push_str(&format!(".{}()", v));
        if s != "medium" { chain.push_str(&format!(".{}()", s)); }
        if ld { chain.push_str(".loading(true)"); }
        if dis { chain.push_str(".disabled(true)"); }
        chain.push_str(".build()");
        code_ref.set_text_content(Some(&chain));
    });
    append_node(&page, &code);

    append_node(&page, &api_table(&[
        ("label", "&str", "\u{2014}", "Button text content"),
        (".primary()", "self", "\u{2014}", "Primary gradient style"),
        (".outline()", "self", "\u{2014}", "Outline border style"),
        (".danger()", "self", "\u{2014}", "Red danger style"),
        (".ghost()", "self", "\u{2014}", "Transparent ghost style"),
        (".small()", "self", "\u{2014}", "Small size (0.75rem)"),
        (".large()", "self", "\u{2014}", "Large size (1rem)"),
        (".loading(bool)", "self", "false", "Show spinner, disable clicks"),
        (".disabled(bool)", "self", "false", "Disable the button"),
        (".on_click(handler)", "Element", "\u{2014}", "Attach click handler, returns element"),
        (".build()", "Element", "\u{2014}", "Build without click handler"),
    ]));

    page
}

// ═══════════════════════════════════════════════════════════════════════════
// Component Playground: TextInput
// ═══════════════════════════════════════════════════════════════════════════

fn pg_input() -> web_sys::Element {
    let (page, preview, controls, _layout) = pg_shell(
        "TextInput",
        "Text input with label, placeholder, validation, and signal binding.",
    );

    let label = signal("Email".to_string());
    let placeholder = signal("you@example.com".to_string());
    let input_type = signal("text".to_string());
    let required = signal(false);
    let error_msg = signal(String::new());
    let bound_value = signal(String::new());

    // Controls
    append_node(&controls, &components::text_input("Label").placeholder("Input label").bind(label).build());
    append_node(&controls, &components::text_input("Placeholder").placeholder("Placeholder text").bind(placeholder).build());
    append_node(&controls, &components::select("Type", &[
        ("text", "Text"), ("email", "Email"), ("password", "Password"), ("number", "Number"),
    ], input_type));
    append_node(&controls, &components::checkbox("Required", required));
    append_node(&controls, &components::text_input("Error message").placeholder("Leave blank for none").bind(error_msg).build());

    // Preview
    let preview_ref = preview.clone();
    create_effect(move || {
        clear_children(&preview_ref);
        let mut b = components::text_input(&label.get())
            .placeholder(&placeholder.get())
            .input_type(&input_type.get())
            .bind(bound_value);
        if required.get() { b = b.required(); }
        let err = error_msg.get();
        if !err.is_empty() { b = b.error(&err); }
        set_style(&preview_ref, "flex-direction", "column");
        set_style(&preview_ref, "align-items", "stretch");
        append_node(&preview_ref, &b.build());
    });

    // Code block
    let code = create_element("pre");
    set_attribute(&code, "class", "pg-code");
    let code_ref = code.clone();
    create_effect(move || {
        let l = label.get();
        let p = placeholder.get();
        let t = input_type.get();
        let r = required.get();
        let e = error_msg.get();
        let mut chain = format!("text_input(\"{}\")", l);
        chain.push_str(&format!("\n    .placeholder(\"{}\")", p));
        if t != "text" { chain.push_str(&format!("\n    .input_type(\"{}\")", t)); }
        chain.push_str("\n    .bind(value_signal)");
        if r { chain.push_str("\n    .required()"); }
        if !e.is_empty() { chain.push_str(&format!("\n    .error(\"{}\")", e)); }
        chain.push_str("\n    .build()");
        code_ref.set_text_content(Some(&chain));
    });
    append_node(&page, &code);

    append_node(&page, &api_table(&[
        ("label", "&str", "\u{2014}", "Input label text"),
        (".placeholder()", "self", "\u{2014}", "Placeholder text"),
        (".input_type()", "self", "\"text\"", "HTML input type"),
        (".required()", "self", "false", "Mark as required"),
        (".error(msg)", "self", "\u{2014}", "Show validation error"),
        (".bind(Signal)", "self", "\u{2014}", "Two-way signal binding"),
        (".build()", "Element", "\u{2014}", "Build the input element"),
    ]));

    page
}

// ═══════════════════════════════════════════════════════════════════════════
// Component Playground: TextArea
// ═══════════════════════════════════════════════════════════════════════════

fn pg_textarea() -> web_sys::Element {
    let (page, preview, controls, _layout) = pg_shell(
        "TextArea",
        "Multi-line text input with label and reactive signal binding.",
    );

    let label = signal("Comment".to_string());
    let value = signal("Type something here...".to_string());

    append_node(&controls, &components::text_input("Label").placeholder("TextArea label").bind(label).build());

    let preview_ref = preview.clone();
    create_effect(move || {
        clear_children(&preview_ref);
        set_style(&preview_ref, "flex-direction", "column");
        set_style(&preview_ref, "align-items", "stretch");
        append_node(&preview_ref, &components::textarea(&label.get(), value));
    });

    let code = create_element("pre");
    set_attribute(&code, "class", "pg-code");
    let code_ref = code.clone();
    create_effect(move || {
        let l = label.get();
        code_ref.set_text_content(Some(&format!(
            "let value = signal(String::new());\ntextarea(\"{}\", value)", l
        )));
    });
    append_node(&page, &code);

    append_node(&page, &api_table(&[
        ("label", "&str", "\u{2014}", "TextArea label text"),
        ("value", "Signal<String>", "\u{2014}", "Two-way bound value signal"),
    ]));

    page
}

// ═══════════════════════════════════════════════════════════════════════════
// Component Playground: Select
// ═══════════════════════════════════════════════════════════════════════════

fn pg_select() -> web_sys::Element {
    let (page, preview, controls, _layout) = pg_shell(
        "Select",
        "Dropdown select with label, options, and signal binding.",
    );

    let label = signal("Country".to_string());
    let selected = signal("us".to_string());

    append_node(&controls, &components::text_input("Label").placeholder("Select label").bind(label).build());

    // Show selected value
    let sel_display = el("div", "mono", &[]);
    let sel_ref = sel_display.clone();
    create_effect(move || {
        sel_ref.set_text_content(Some(&format!("Selected: {}", selected.get())));
    });
    append_node(&controls, &sel_display);

    let preview_ref = preview.clone();
    create_effect(move || {
        clear_children(&preview_ref);
        set_style(&preview_ref, "flex-direction", "column");
        set_style(&preview_ref, "align-items", "stretch");
        append_node(&preview_ref, &components::select(&label.get(), &[
            ("us", "United States"),
            ("uk", "United Kingdom"),
            ("ca", "Canada"),
            ("de", "Germany"),
            ("jp", "Japan"),
        ], selected));
    });

    let code = create_element("pre");
    set_attribute(&code, "class", "pg-code");
    let code_ref = code.clone();
    create_effect(move || {
        let l = label.get();
        code_ref.set_text_content(Some(&format!(
            "let value = signal(\"us\".to_string());\nselect(\"{}\", &[\n    (\"us\", \"United States\"),\n    (\"uk\", \"United Kingdom\"),\n    (\"ca\", \"Canada\"),\n], value)", l
        )));
    });
    append_node(&page, &code);

    append_node(&page, &api_table(&[
        ("label", "&str", "\u{2014}", "Select label text"),
        ("options", "&[(&str,&str)]", "\u{2014}", "Value-label option pairs"),
        ("value", "Signal<String>", "\u{2014}", "Two-way bound selected value"),
    ]));

    page
}

// ═══════════════════════════════════════════════════════════════════════════
// Component Playground: Checkbox
// ═══════════════════════════════════════════════════════════════════════════

fn pg_checkbox() -> web_sys::Element {
    let (page, preview, controls, _layout) = pg_shell(
        "Checkbox",
        "Checkbox with label and boolean signal binding.",
    );

    let label = signal("I agree to the terms".to_string());
    let checked = signal(false);

    append_node(&controls, &components::text_input("Label text").placeholder("Checkbox label").bind(label).build());

    // Show checked state
    let state_display = el("div", "mono", &[]);
    let state_ref = state_display.clone();
    create_effect(move || {
        state_ref.set_text_content(Some(&format!("Checked: {}", checked.get())));
    });
    append_node(&controls, &state_display);

    let preview_ref = preview.clone();
    create_effect(move || {
        clear_children(&preview_ref);
        append_node(&preview_ref, &components::checkbox(&label.get(), checked));
    });

    let code = create_element("pre");
    set_attribute(&code, "class", "pg-code");
    let code_ref = code.clone();
    create_effect(move || {
        let l = label.get();
        code_ref.set_text_content(Some(&format!(
            "let checked = signal(false);\ncheckbox(\"{}\", checked)", l
        )));
    });
    append_node(&page, &code);

    append_node(&page, &api_table(&[
        ("label", "&str", "\u{2014}", "Checkbox label text"),
        ("checked", "Signal<bool>", "false", "Two-way bound checked state"),
    ]));

    page
}

// ═══════════════════════════════════════════════════════════════════════════
// Component Playground: Card
// ═══════════════════════════════════════════════════════════════════════════

fn pg_card() -> web_sys::Element {
    let (page, preview, controls, _layout) = pg_shell(
        "Card",
        "Container card with title, body content, and optional footer.",
    );

    let title = signal("Settings".to_string());
    let show_footer = signal(true);

    append_node(&controls, &components::text_input("Title").placeholder("Card title").bind(title).build());
    append_node(&controls, &components::checkbox("Show footer", show_footer));

    let preview_ref = preview.clone();
    create_effect(move || {
        clear_children(&preview_ref);
        set_style(&preview_ref, "flex-direction", "column");
        set_style(&preview_ref, "align-items", "stretch");
        let body = text_el("p", "Configure your preferences and settings here.");
        let mut builder = components::card(&title.get()).body(body);
        if show_footer.get() {
            builder = builder.footer(components::button("Save").primary().build());
        }
        append_node(&preview_ref, &builder.build());
    });

    let code = create_element("pre");
    set_attribute(&code, "class", "pg-code");
    let code_ref = code.clone();
    create_effect(move || {
        let t = title.get();
        let sf = show_footer.get();
        let mut s = format!("card(\"{}\")\n    .body(content)", t);
        if sf { s.push_str("\n    .footer(button(\"Save\").primary().build())"); }
        s.push_str("\n    .build()");
        code_ref.set_text_content(Some(&s));
    });
    append_node(&page, &code);

    append_node(&page, &api_table(&[
        ("title", "&str", "\u{2014}", "Card title text"),
        (".body(Element)", "self", "\u{2014}", "Card body content"),
        (".footer(Element)", "self", "\u{2014}", "Optional footer element"),
        (".build()", "Element", "\u{2014}", "Build the card element"),
    ]));

    page
}

// ═══════════════════════════════════════════════════════════════════════════
// Component Playground: Alert
// ═══════════════════════════════════════════════════════════════════════════

fn pg_alert() -> web_sys::Element {
    let (page, preview, controls, _layout) = pg_shell(
        "Alert",
        "Notification alert with multiple severity levels and dismissible option.",
    );

    let message = signal("Operation completed successfully!".to_string());
    let severity = signal("success".to_string());
    let dismissible = signal(false);
    let alert_visible = signal(true);

    append_node(&controls, &components::text_input("Message").placeholder("Alert message").bind(message).build());
    append_node(&controls, &components::select("Severity", &[
        ("success", "Success"), ("warning", "Warning"), ("error", "Error"), ("info", "Info"),
    ], severity));
    append_node(&controls, &components::checkbox("Dismissible", dismissible));

    // Reset button to show alert again after dismiss
    let reset_btn = components::button("Reset Alert").small().build();
    add_event_listener(&reset_btn, "click", move |_| { alert_visible.set(true); });
    append_node(&controls, &reset_btn);

    let preview_ref = preview.clone();
    create_effect(move || {
        clear_children(&preview_ref);
        set_style(&preview_ref, "flex-direction", "column");
        set_style(&preview_ref, "align-items", "stretch");
        let mut b = components::alert(&message.get());
        match severity.get().as_str() {
            "success" => { b = b.success(); }
            "warning" => { b = b.warning(); }
            "error"   => { b = b.error(); }
            _         => { b = b.info(); }
        }
        if dismissible.get() {
            b = b.dismissible(alert_visible);
        }
        append_node(&preview_ref, &b.build());
    });

    let code = create_element("pre");
    set_attribute(&code, "class", "pg-code");
    let code_ref = code.clone();
    create_effect(move || {
        let m = message.get();
        let s = severity.get();
        let d = dismissible.get();
        let mut chain = format!("alert(\"{}\")\n    .{}()", m, s);
        if d { chain.push_str("\n    .dismissible(visible_signal)"); }
        chain.push_str("\n    .build()");
        code_ref.set_text_content(Some(&chain));
    });
    append_node(&page, &code);

    append_node(&page, &api_table(&[
        ("message", "&str", "\u{2014}", "Alert message text"),
        (".success()", "self", "\u{2014}", "Green success style"),
        (".warning()", "self", "\u{2014}", "Yellow warning style"),
        (".error()", "self", "\u{2014}", "Red error style"),
        (".info()", "self", "\u{2014}", "Blue info style"),
        (".dismissible(Signal)", "self", "\u{2014}", "Allow dismissal via signal"),
        (".build()", "Element", "\u{2014}", "Build the alert element"),
    ]));

    page
}

// ═══════════════════════════════════════════════════════════════════════════
// Component Playground: Modal
// ═══════════════════════════════════════════════════════════════════════════

fn pg_modal() -> web_sys::Element {
    let (page, preview, _controls, _layout) = pg_shell(
        "Modal",
        "Overlay modal dialog controlled by a boolean signal. Click the button to open it.",
    );

    let is_open = signal(false);

    // Preview: button to open + modal
    let open_btn = components::button("Open Modal").primary().build();
    add_event_listener(&open_btn, "click", move |_| { is_open.set(true); });
    append_node(&preview, &open_btn);

    let modal_body = text_el("p", "This modal is rendered and controlled entirely by Rust signals compiled to WASM.");
    let modal_el = components::modal(is_open)
        .title("Oxide Modal")
        .body(modal_body)
        .build();
    append_node(&preview, &modal_el);

    let code = create_element("pre");
    set_attribute(&code, "class", "pg-code");
    code.set_text_content(Some(
        "let is_open = signal(false);\n\nmodal(is_open)\n    .title(\"Oxide Modal\")\n    .body(content)\n    .build()"
    ));
    append_node(&page, &code);

    append_node(&page, &api_table(&[
        ("open", "Signal<bool>", "\u{2014}", "Controls modal visibility"),
        (".title()", "self", "\u{2014}", "Modal title text"),
        (".body(Element)", "self", "\u{2014}", "Modal body content"),
        (".build()", "Element", "\u{2014}", "Build the modal element"),
    ]));

    page
}

// ═══════════════════════════════════════════════════════════════════════════
// Component Playground: Spinner
// ═══════════════════════════════════════════════════════════════════════════

fn pg_spinner() -> web_sys::Element {
    let (page, preview, controls, _layout) = pg_shell(
        "Spinner",
        "Rotating loading indicator, with or without text.",
    );

    let show_text = signal(true);

    append_node(&controls, &components::checkbox("Show text", show_text));

    let preview_ref = preview.clone();
    create_effect(move || {
        clear_children(&preview_ref);
        if show_text.get() {
            append_node(&preview_ref, &components::spinner_with_text("Loading..."));
        } else {
            append_node(&preview_ref, &components::spinner());
        }
    });

    let code = create_element("pre");
    set_attribute(&code, "class", "pg-code");
    let code_ref = code.clone();
    create_effect(move || {
        if show_text.get() {
            code_ref.set_text_content(Some("spinner_with_text(\"Loading...\")"));
        } else {
            code_ref.set_text_content(Some("spinner()"));
        }
    });
    append_node(&page, &code);

    append_node(&page, &api_table(&[
        ("(no args)", "\u{2014}", "\u{2014}", "Plain spinner with no text"),
        ("text", "&str", "\u{2014}", "Spinner with text label (spinner_with_text)"),
    ]));

    page
}

// ═══════════════════════════════════════════════════════════════════════════
// Component Playground: Progress
// ═══════════════════════════════════════════════════════════════════════════

fn pg_progress() -> web_sys::Element {
    let (page, preview, controls, _layout) = pg_shell(
        "Progress",
        "Animated progress bar driven by a signal value (0–100).",
    );

    let value = signal(65.0f64);
    let range_str = signal("65".to_string());

    // Range slider
    let slider_wrap = el("div", "col", &[]);
    let slider_label = el("div", "", &[]);
    let slider_label_ref = slider_label.clone();
    create_effect(move || {
        slider_label_ref.set_text_content(Some(&format!("Value: {:.0}%", value.get())));
    });
    append_node(&slider_wrap, &slider_label);
    let slider = create_element("input");
    set_attribute(&slider, "type", "range");
    set_attribute(&slider, "min", "0");
    set_attribute(&slider, "max", "100");
    set_attribute(&slider, "value", "65");
    let rs = range_str;
    add_event_listener(&slider, "input", move |e| {
        let v = event_target_value(&e);
        rs.set(v.clone());
        if let Ok(n) = v.parse::<f64>() {
            value.set(n);
        }
    });
    append_node(&slider_wrap, &slider);
    append_node(&controls, &slider_wrap);

    // Preview
    set_style(&preview, "flex-direction", "column");
    set_style(&preview, "align-items", "stretch");
    append_node(&preview, &components::progress(value));

    let code = create_element("pre");
    set_attribute(&code, "class", "pg-code");
    let code_ref = code.clone();
    create_effect(move || {
        code_ref.set_text_content(Some(&format!(
            "let value = signal({:.0}.0);\nprogress(value)", value.get()
        )));
    });
    append_node(&page, &code);

    append_node(&page, &api_table(&[
        ("value", "Signal<f64>", "\u{2014}", "Progress value 0\u{2013}100"),
    ]));

    page
}

// ═══════════════════════════════════════════════════════════════════════════
// Component Playground: Tabs
// ═══════════════════════════════════════════════════════════════════════════

fn pg_tabs() -> web_sys::Element {
    let (page, preview, _controls, _layout) = pg_shell(
        "Tabs",
        "Tabbed interface with ARIA-compliant panels. Each tab renders its own content.",
    );

    set_style(&preview, "flex-direction", "column");
    set_style(&preview, "align-items", "stretch");

    append_node(&preview, &components::tabs(&[
        ("Profile", || text_el("div", "User profile settings and personal information.")),
        ("Settings", || text_el("div", "Application settings, notifications, and preferences.")),
        ("Billing", || text_el("div", "Subscription plans, payment methods, and invoices.")),
    ]));

    let code = create_element("pre");
    set_attribute(&code, "class", "pg-code");
    code.set_text_content(Some(
        "tabs(&[\n    (\"Profile\",  || view! { <div>\"Profile content\"</div> }),\n    (\"Settings\", || view! { <div>\"Settings content\"</div> }),\n    (\"Billing\",  || view! { <div>\"Billing content\"</div> }),\n])"
    ));
    append_node(&page, &code);

    append_node(&page, &api_table(&[
        ("items", "&[(&str, fn()->Element)]", "\u{2014}", "Tab label and content builder pairs"),
    ]));

    page
}

// ═══════════════════════════════════════════════════════════════════════════
// Component Playground: Badge
// ═══════════════════════════════════════════════════════════════════════════

fn pg_badge() -> web_sys::Element {
    let (page, preview, controls, _layout) = pg_shell(
        "Badge",
        "Small colored label badge with severity-based variants.",
    );

    let text = signal("Active".to_string());
    let severity_str = signal("success".to_string());

    append_node(&controls, &components::text_input("Text").placeholder("Badge text").bind(text).build());
    append_node(&controls, &components::select("Severity", &[
        ("success", "Success"), ("warning", "Warning"), ("error", "Error"), ("info", "Info"),
    ], severity_str));

    let preview_ref = preview.clone();
    create_effect(move || {
        clear_children(&preview_ref);
        let sev = match severity_str.get().as_str() {
            "success" => Severity::Success,
            "warning" => Severity::Warning,
            "error"   => Severity::Error,
            _         => Severity::Info,
        };
        append_node(&preview_ref, &components::badge(&text.get(), sev));
    });

    let code = create_element("pre");
    set_attribute(&code, "class", "pg-code");
    let code_ref = code.clone();
    create_effect(move || {
        let t = text.get();
        let s = severity_str.get();
        let sev_name = match s.as_str() {
            "success" => "Success",
            "warning" => "Warning",
            "error"   => "Error",
            _         => "Info",
        };
        code_ref.set_text_content(Some(&format!(
            "badge(\"{}\", Severity::{})", t, sev_name
        )));
    });
    append_node(&page, &code);

    append_node(&page, &api_table(&[
        ("text", "&str", "\u{2014}", "Badge label text"),
        ("severity", "Severity", "\u{2014}", "Color variant (Success, Warning, Error, Info)"),
    ]));

    page
}

// ═══════════════════════════════════════════════════════════════════════════
// Component Playground: Divider
// ═══════════════════════════════════════════════════════════════════════════

fn pg_divider() -> web_sys::Element {
    let (page, preview, _controls, _layout) = pg_shell(
        "Divider",
        "A simple horizontal divider for separating content sections.",
    );

    set_style(&preview, "flex-direction", "column");
    set_style(&preview, "align-items", "stretch");
    set_style(&preview, "gap", "1rem");

    append_node(&preview, &text_el("p", "Content above the divider."));
    append_node(&preview, &components::divider());
    append_node(&preview, &text_el("p", "Content below the divider."));

    let code = create_element("pre");
    set_attribute(&code, "class", "pg-code");
    code.set_text_content(Some("divider()"));
    append_node(&page, &code);

    append_node(&page, &api_table(&[
        ("(no args)", "\u{2014}", "\u{2014}", "Renders a horizontal divider line"),
    ]));

    page
}

// ═══════════════════════════════════════════════════════════════════════════
// Component Playground: Skeleton
// ═══════════════════════════════════════════════════════════════════════════

fn pg_skeleton() -> web_sys::Element {
    let (page, preview, _controls, _layout) = pg_shell(
        "Skeleton",
        "Shimmer-animated loading placeholder for content that is still loading.",
    );

    set_style(&preview, "flex-direction", "column");
    set_style(&preview, "align-items", "stretch");
    set_style(&preview, "gap", "0.75rem");

    append_node(&preview, &components::skeleton("100%", "20px"));
    append_node(&preview, &components::skeleton("80%", "20px"));
    append_node(&preview, &components::skeleton("60%", "20px"));
    append_node(&preview, &components::skeleton("200px", "100px"));

    let code = create_element("pre");
    set_attribute(&code, "class", "pg-code");
    code.set_text_content(Some(
        "skeleton(\"100%\", \"20px\")\nskeleton(\"80%\", \"20px\")\nskeleton(\"200px\", \"100px\")"
    ));
    append_node(&page, &code);

    append_node(&page, &api_table(&[
        ("width", "&str", "\u{2014}", "CSS width (e.g. \"100%\", \"200px\")"),
        ("height", "&str", "\u{2014}", "CSS height (e.g. \"20px\", \"100px\")"),
    ]));

    page
}

// ═══════════════════════════════════════════════════════════════════════════
// New Component Playgrounds
// ═══════════════════════════════════════════════════════════════════════════

fn pg_avatar() -> web_sys::Element {
    let (page, preview, controls, _) = pg_shell("Avatar", "User avatar with initials or image source.");

    let name = signal("Jane Doe".to_string());
    let size_str = signal("medium".to_string());

    append_node(&controls, &components::text_input("Name").placeholder("User name").bind(name).build());
    append_node(&controls, &components::select("Size", &[
        ("small", "Small"), ("medium", "Medium"), ("large", "Large"),
    ], size_str));

    let preview_ref = preview.clone();
    create_effect(move || {
        clear_children(&preview_ref);
        let sz = match size_str.get().as_str() {
            "small" => AvatarSize::Small,
            "large" => AvatarSize::Large,
            _ => AvatarSize::Medium,
        };
        append_node(&preview_ref, &components::avatar(&name.get()).size(sz).build());
    });

    let code = create_element("pre");
    set_attribute(&code, "class", "pg-code");
    let code_ref = code.clone();
    create_effect(move || {
        let n = name.get();
        let s = size_str.get();
        let mut c = format!("avatar(\"{}\")", n);
        if s != "medium" { c.push_str(&format!(".size(AvatarSize::{:?})", if s == "small" { "Small" } else { "Large" })); }
        c.push_str(".build()");
        code_ref.set_text_content(Some(&c));
    });
    append_node(&page, &code);

    append_node(&page, &api_table(&[
        ("name", "&str", "\u{2014}", "User name (used for initials)"),
        (".size(AvatarSize)", "self", "Medium", "Avatar size (Small, Medium, Large)"),
        (".src(url)", "self", "\u{2014}", "Image URL instead of initials"),
        (".build()", "Element", "\u{2014}", "Build the avatar element"),
    ]));

    page
}

fn pg_stat() -> web_sys::Element {
    let (page, preview, controls, _) = pg_shell("Stat", "Large stat display with value and label.");

    let value = signal("1,234".to_string());
    let label = signal("Total Users".to_string());

    append_node(&controls, &components::text_input("Value").placeholder("Stat value").bind(value).build());
    append_node(&controls, &components::text_input("Label").placeholder("Stat label").bind(label).build());

    let preview_ref = preview.clone();
    create_effect(move || {
        clear_children(&preview_ref);
        append_node(&preview_ref, &components::stat(&value.get(), &label.get()));
    });

    let code = create_element("pre");
    set_attribute(&code, "class", "pg-code");
    let code_ref = code.clone();
    create_effect(move || {
        code_ref.set_text_content(Some(&format!("stat(\"{}\", \"{}\")", value.get(), label.get())));
    });
    append_node(&page, &code);

    append_node(&page, &api_table(&[
        ("value", "&str", "\u{2014}", "Large display value"),
        ("label", "&str", "\u{2014}", "Descriptive label below value"),
    ]));

    page
}

fn pg_tag() -> web_sys::Element {
    let (page, preview, controls, _) = pg_shell("Tag", "Removable tag with severity-based color variants.");

    let text = signal("Rust".to_string());
    let sev_str = signal("info".to_string());

    append_node(&controls, &components::text_input("Text").placeholder("Tag text").bind(text).build());
    append_node(&controls, &components::select("Severity", &[
        ("success", "Success"), ("warning", "Warning"), ("error", "Error"), ("info", "Info"),
    ], sev_str));

    let preview_ref = preview.clone();
    create_effect(move || {
        clear_children(&preview_ref);
        let sev = match sev_str.get().as_str() {
            "success" => Severity::Success,
            "warning" => Severity::Warning,
            "error" => Severity::Error,
            _ => Severity::Info,
        };
        append_node(&preview_ref, &components::tag(&text.get()).variant(sev).build());
    });

    let code = create_element("pre");
    set_attribute(&code, "class", "pg-code");
    let code_ref = code.clone();
    create_effect(move || {
        let sev_name = match sev_str.get().as_str() {
            "success" => "Success", "warning" => "Warning", "error" => "Error", _ => "Info",
        };
        code_ref.set_text_content(Some(&format!("tag(\"{}\").variant(Severity::{}).build()", text.get(), sev_name)));
    });
    append_node(&page, &code);

    append_node(&page, &api_table(&[
        ("text", "&str", "\u{2014}", "Tag label text"),
        (".variant(Severity)", "self", "\u{2014}", "Color variant"),
        (".removable(Signal)", "self", "\u{2014}", "Removable with signal control"),
        (".build()", "Element", "\u{2014}", "Build the tag element"),
    ]));

    page
}

fn pg_toggle() -> web_sys::Element {
    let (page, preview, controls, _) = pg_shell("Toggle", "On/off switch with label and boolean signal binding.");

    let checked = signal(false);

    let state_display = el("div", "mono", &[]);
    let state_ref = state_display.clone();
    create_effect(move || {
        state_ref.set_text_content(Some(&format!("Value: {}", checked.get())));
    });
    append_node(&controls, &state_display);

    append_node(&preview, &components::toggle("Dark Mode", checked));

    let code = create_element("pre");
    set_attribute(&code, "class", "pg-code");
    code.set_text_content(Some("let on = signal(false);\ntoggle(\"Dark Mode\", on)"));
    append_node(&page, &code);

    append_node(&page, &api_table(&[
        ("label", "&str", "\u{2014}", "Toggle label text"),
        ("checked", "Signal<bool>", "false", "Two-way bound toggle state"),
    ]));

    page
}

fn pg_radio() -> web_sys::Element {
    let (page, preview, controls, _) = pg_shell("Radio Group", "Radio button group with named options and signal binding.");

    let selected = signal("opt1".to_string());

    let state_display = el("div", "mono", &[]);
    let state_ref = state_display.clone();
    create_effect(move || {
        state_ref.set_text_content(Some(&format!("Selected: {}", selected.get())));
    });
    append_node(&controls, &state_display);

    set_style(&preview, "flex-direction", "column");
    set_style(&preview, "align-items", "stretch");
    append_node(&preview, &components::radio_group("plan", &[
        ("opt1", "Free Plan"),
        ("opt2", "Pro Plan"),
        ("opt3", "Enterprise"),
    ], selected));

    let code = create_element("pre");
    set_attribute(&code, "class", "pg-code");
    code.set_text_content(Some("let selected = signal(\"opt1\".to_string());\nradio_group(\"plan\", &[\n    (\"opt1\", \"Free Plan\"),\n    (\"opt2\", \"Pro Plan\"),\n    (\"opt3\", \"Enterprise\"),\n], selected)"));
    append_node(&page, &code);

    append_node(&page, &api_table(&[
        ("name", "&str", "\u{2014}", "Group name attribute"),
        ("options", "&[(&str,&str)]", "\u{2014}", "Value-label option pairs"),
        ("selected", "Signal<String>", "\u{2014}", "Two-way bound selected value"),
    ]));

    page
}

fn pg_slider() -> web_sys::Element {
    let (page, preview, controls, _) = pg_shell("Slider", "Range slider input with min, max, step, and signal binding.");

    let value = signal(50.0f64);

    let display = el("div", "mono", &[]);
    let display_ref = display.clone();
    create_effect(move || {
        display_ref.set_text_content(Some(&format!("Value: {:.0}", value.get())));
    });
    append_node(&controls, &display);

    set_style(&preview, "flex-direction", "column");
    set_style(&preview, "align-items", "stretch");
    append_node(&preview, &components::slider(0.0, 100.0, 1.0, value));

    let code = create_element("pre");
    set_attribute(&code, "class", "pg-code");
    code.set_text_content(Some("let value = signal(50.0);\nslider(0.0, 100.0, 1.0, value)"));
    append_node(&page, &code);

    append_node(&page, &api_table(&[
        ("min", "f64", "\u{2014}", "Minimum value"),
        ("max", "f64", "\u{2014}", "Maximum value"),
        ("step", "f64", "\u{2014}", "Step increment"),
        ("value", "Signal<f64>", "\u{2014}", "Two-way bound value"),
    ]));

    page
}

fn pg_search() -> web_sys::Element {
    let (page, preview, controls, _) = pg_shell("Search Input", "Search input field with placeholder and signal binding.");

    let value = signal(String::new());

    let display = el("div", "mono", &[]);
    let display_ref = display.clone();
    create_effect(move || {
        display_ref.set_text_content(Some(&format!("Query: \"{}\"", value.get())));
    });
    append_node(&controls, &display);

    set_style(&preview, "flex-direction", "column");
    set_style(&preview, "align-items", "stretch");
    append_node(&preview, &components::search_input("Search components...", value));

    let code = create_element("pre");
    set_attribute(&code, "class", "pg-code");
    code.set_text_content(Some("let query = signal(String::new());\nsearch_input(\"Search components...\", query)"));
    append_node(&page, &code);

    append_node(&page, &api_table(&[
        ("placeholder", "&str", "\u{2014}", "Placeholder text"),
        ("value", "Signal<String>", "\u{2014}", "Two-way bound search value"),
    ]));

    page
}

fn pg_password() -> web_sys::Element {
    let (page, preview, controls, _) = pg_shell("Password Input", "Password input with label and signal binding.");

    let value = signal(String::new());

    let display = el("div", "mono", &[]);
    let display_ref = display.clone();
    create_effect(move || {
        let v = value.get();
        display_ref.set_text_content(Some(&format!("Length: {}", v.len())));
    });
    append_node(&controls, &display);

    set_style(&preview, "flex-direction", "column");
    set_style(&preview, "align-items", "stretch");
    append_node(&preview, &components::password_input("Password", value));

    let code = create_element("pre");
    set_attribute(&code, "class", "pg-code");
    code.set_text_content(Some("let pw = signal(String::new());\npassword_input(\"Password\", pw)"));
    append_node(&page, &code);

    append_node(&page, &api_table(&[
        ("label", "&str", "\u{2014}", "Input label text"),
        ("value", "Signal<String>", "\u{2014}", "Two-way bound password value"),
    ]));

    page
}

fn pg_number() -> web_sys::Element {
    let (page, preview, controls, _) = pg_shell("Number Input", "Number input with label, step control, and signal binding.");

    let value = signal(0.0f64);

    let display = el("div", "mono", &[]);
    let display_ref = display.clone();
    create_effect(move || {
        display_ref.set_text_content(Some(&format!("Value: {}", value.get())));
    });
    append_node(&controls, &display);

    set_style(&preview, "flex-direction", "column");
    set_style(&preview, "align-items", "stretch");
    append_node(&preview, &components::number_input("Quantity", value, 1.0));

    let code = create_element("pre");
    set_attribute(&code, "class", "pg-code");
    code.set_text_content(Some("let qty = signal(0.0);\nnumber_input(\"Quantity\", qty, 1.0)"));
    append_node(&page, &code);

    append_node(&page, &api_table(&[
        ("label", "&str", "\u{2014}", "Input label text"),
        ("value", "Signal<f64>", "\u{2014}", "Two-way bound numeric value"),
        ("step", "f64", "\u{2014}", "Step increment for +/- buttons"),
    ]));

    page
}

fn pg_accordion() -> web_sys::Element {
    let (page, preview, _, _) = pg_shell("Accordion", "Collapsible accordion sections with title and content.");

    set_style(&preview, "flex-direction", "column");
    set_style(&preview, "align-items", "stretch");
    append_node(&preview, &components::accordion(&[
        ("Getting Started", || text_el("div", "Install Oxide and create your first WASM app in minutes.")),
        ("Components", || text_el("div", "Explore 40+ pre-built components with full API docs.")),
        ("Deployment", || text_el("div", "Deploy to any static host — just HTML, CSS, and WASM.")),
    ]));

    let code = create_element("pre");
    set_attribute(&code, "class", "pg-code");
    code.set_text_content(Some("accordion(&[\n    (\"Getting Started\", || view()),\n    (\"Components\", || view()),\n    (\"Deployment\", || view()),\n])"));
    append_node(&page, &code);

    append_node(&page, &api_table(&[
        ("items", "&[(&str, fn()->Element)]", "\u{2014}", "Title and content builder pairs"),
    ]));

    page
}

fn pg_breadcrumb() -> web_sys::Element {
    let (page, preview, _, _) = pg_shell("Breadcrumb", "Breadcrumb navigation trail with links.");

    set_style(&preview, "flex-direction", "column");
    set_style(&preview, "align-items", "stretch");
    append_node(&preview, &components::breadcrumb(&[
        ("Home", "#/"),
        ("Components", "#/components"),
        ("Breadcrumb", ""),
    ]));

    let code = create_element("pre");
    set_attribute(&code, "class", "pg-code");
    code.set_text_content(Some("breadcrumb(&[\n    (\"Home\", \"#/\"),\n    (\"Components\", \"#/components\"),\n    (\"Breadcrumb\", \"\"),\n])"));
    append_node(&page, &code);

    append_node(&page, &api_table(&[
        ("items", "&[(&str, &str)]", "\u{2014}", "Label-href pairs (empty href = current page)"),
    ]));

    page
}

fn pg_pagination() -> web_sys::Element {
    let (page, preview, controls, _) = pg_shell("Pagination", "Pagination control with clickable page numbers.");

    let total = signal(10u32);
    let current = signal(1u32);

    let display = el("div", "mono", &[]);
    let display_ref = display.clone();
    create_effect(move || {
        display_ref.set_text_content(Some(&format!("Page {} of {}", current.get(), total.get())));
    });
    append_node(&controls, &display);

    set_style(&preview, "flex-direction", "column");
    set_style(&preview, "align-items", "stretch");
    append_node(&preview, &components::pagination(total, current));

    let code = create_element("pre");
    set_attribute(&code, "class", "pg-code");
    code.set_text_content(Some("let total = signal(10u32);\nlet current = signal(1u32);\npagination(total, current)"));
    append_node(&page, &code);

    append_node(&page, &api_table(&[
        ("total_pages", "Signal<u32>", "\u{2014}", "Total number of pages"),
        ("current", "Signal<u32>", "\u{2014}", "Current active page (1-based)"),
    ]));

    page
}

fn pg_dropdown() -> web_sys::Element {
    let (page, preview, controls, _) = pg_shell("Dropdown", "Click-activated dropdown menu with selectable items.");

    let selected = signal(String::new());

    let display = el("div", "mono", &[]);
    let display_ref = display.clone();
    create_effect(move || {
        let v = selected.get();
        display_ref.set_text_content(Some(&format!("Selected: {}", if v.is_empty() { "(none)" } else { &v })));
    });
    append_node(&controls, &display);

    append_node(&preview, &components::dropdown("Choose framework", &["Oxide", "React", "Vue", "Svelte"], selected));

    let code = create_element("pre");
    set_attribute(&code, "class", "pg-code");
    code.set_text_content(Some("let sel = signal(String::new());\ndropdown(\"Choose framework\", &[\"Oxide\", \"React\", \"Vue\"], sel)"));
    append_node(&page, &code);

    append_node(&page, &api_table(&[
        ("trigger_text", "&str", "\u{2014}", "Button text for the dropdown trigger"),
        ("items", "&[&str]", "\u{2014}", "Selectable item labels"),
        ("selected", "Signal<String>", "\u{2014}", "Two-way bound selected value"),
    ]));

    page
}

fn pg_toast() -> web_sys::Element {
    let (page, preview, controls, _) = pg_shell("Toast", "Auto-dismiss notification toast with severity and duration.");

    let sev_str = signal("success".to_string());
    append_node(&controls, &components::select("Severity", &[
        ("success", "Success"), ("warning", "Warning"), ("error", "Error"), ("info", "Info"),
    ], sev_str));

    let btn = components::button("Show Toast").primary().build();
    let sev_sig = sev_str;
    add_event_listener(&btn, "click", move |_| {
        let sev = match sev_sig.get().as_str() {
            "success" => Severity::Success,
            "warning" => Severity::Warning,
            "error" => Severity::Error,
            _ => Severity::Info,
        };
        components::toast("Operation completed!").severity(sev).duration_ms(3000).show();
    });
    append_node(&preview, &btn);

    let code = create_element("pre");
    set_attribute(&code, "class", "pg-code");
    code.set_text_content(Some("toast(\"Operation completed!\")\n    .severity(Severity::Success)\n    .duration_ms(3000)\n    .show()"));
    append_node(&page, &code);

    append_node(&page, &api_table(&[
        ("message", "&str", "\u{2014}", "Toast message text"),
        (".severity(Severity)", "self", "Info", "Color variant"),
        (".duration_ms(u32)", "self", "3000", "Auto-dismiss duration in ms"),
        (".show()", "Element", "\u{2014}", "Show the toast (appends to body)"),
    ]));

    page
}

fn pg_drawer() -> web_sys::Element {
    let (page, preview, controls, _) = pg_shell("Drawer", "Side panel overlay drawer controlled by a signal.");

    let is_open = signal(false);
    let side_str = signal("right".to_string());

    append_node(&controls, &components::select("Side", &[
        ("left", "Left"), ("right", "Right"),
    ], side_str));

    let preview_ref = preview.clone();
    create_effect(move || {
        clear_children(&preview_ref);
        let side = match side_str.get().as_str() {
            "left" => DrawerSide::Left,
            _ => DrawerSide::Right,
        };
        let open_btn2 = components::button("Open Drawer").primary().build();
        add_event_listener(&open_btn2, "click", move |_| { is_open.set(true); });
        append_node(&preview_ref, &open_btn2);
        let body = text_el("p", "Drawer content goes here. Close by clicking the overlay.");
        let d = components::drawer(is_open).side(side).title("Settings").body(body).build();
        append_node(&preview_ref, &d);
    });

    let code = create_element("pre");
    set_attribute(&code, "class", "pg-code");
    code.set_text_content(Some("let open = signal(false);\ndrawer(open)\n    .side(DrawerSide::Right)\n    .title(\"Settings\")\n    .body(content)\n    .build()"));
    append_node(&page, &code);

    append_node(&page, &api_table(&[
        ("open", "Signal<bool>", "\u{2014}", "Controls drawer visibility"),
        (".side(DrawerSide)", "self", "Right", "Drawer side (Left, Right)"),
        (".title()", "self", "\u{2014}", "Drawer title text"),
        (".body(Element)", "self", "\u{2014}", "Drawer body content"),
        (".build()", "Element", "\u{2014}", "Build the drawer element"),
    ]));

    page
}

fn pg_timeline() -> web_sys::Element {
    let (page, preview, _, _) = pg_shell("Timeline", "Vertical timeline of events with title and description.");

    set_style(&preview, "flex-direction", "column");
    set_style(&preview, "align-items", "stretch");
    append_node(&preview, &components::timeline(&[
        ("Project Created", "Initial commit and project scaffolding."),
        ("Alpha Release", "Core components and routing implemented."),
        ("Beta Release", "40+ components with full documentation."),
        ("v1.0 Stable", "Production-ready release with SSR support."),
    ]));

    let code = create_element("pre");
    set_attribute(&code, "class", "pg-code");
    code.set_text_content(Some("timeline(&[\n    (\"Project Created\", \"Initial commit.\"),\n    (\"Alpha Release\", \"Core components.\"),\n    (\"v1.0 Stable\", \"Production ready.\"),\n])"));
    append_node(&page, &code);

    append_node(&page, &api_table(&[
        ("items", "&[(&str, &str)]", "\u{2014}", "Title-description pairs for timeline entries"),
    ]));

    page
}

fn pg_table() -> web_sys::Element {
    let (page, preview, _, _) = pg_shell("Data Table", "Data table with headers and rows.");

    set_style(&preview, "flex-direction", "column");
    set_style(&preview, "align-items", "stretch");
    append_node(&preview, &components::data_table(
        &["Name", "Role", "Status"],
        &[
            vec!["Alice".into(), "Engineer".into(), "Active".into()],
            vec!["Bob".into(), "Designer".into(), "Away".into()],
            vec!["Carol".into(), "PM".into(), "Active".into()],
        ],
    ));

    let code = create_element("pre");
    set_attribute(&code, "class", "pg-code");
    code.set_text_content(Some("data_table(\n    &[\"Name\", \"Role\", \"Status\"],\n    &[\n        vec![\"Alice\".into(), \"Engineer\".into(), \"Active\".into()],\n        vec![\"Bob\".into(), \"Designer\".into(), \"Away\".into()],\n    ],\n)"));
    append_node(&page, &code);

    append_node(&page, &api_table(&[
        ("headers", "&[&str]", "\u{2014}", "Column header labels"),
        ("rows", "&[Vec<String>]", "\u{2014}", "Row data as string vectors"),
    ]));

    page
}

fn pg_tooltip() -> web_sys::Element {
    let (page, preview, _, _) = pg_shell("Tooltip", "Hover tooltip on any element.");

    let target = components::button("Hover me").primary().build();
    append_node(&preview, &components::tooltip(target, "This is a tooltip!"));

    let code = create_element("pre");
    set_attribute(&code, "class", "pg-code");
    code.set_text_content(Some("let btn = button(\"Hover me\").primary().build();\ntooltip(btn, \"This is a tooltip!\")"));
    append_node(&page, &code);

    append_node(&page, &api_table(&[
        ("target", "Element", "\u{2014}", "Element to attach tooltip to"),
        ("text", "&str", "\u{2014}", "Tooltip text content"),
    ]));

    page
}

fn pg_rating() -> web_sys::Element {
    let (page, preview, controls, _) = pg_shell("Rating", "Interactive star rating with signal binding.");

    let value = signal(3u32);

    let display = el("div", "mono", &[]);
    let display_ref = display.clone();
    create_effect(move || {
        display_ref.set_text_content(Some(&format!("Rating: {}/5", value.get())));
    });
    append_node(&controls, &display);

    append_node(&preview, &components::rating(value, 5));

    let code = create_element("pre");
    set_attribute(&code, "class", "pg-code");
    code.set_text_content(Some("let stars = signal(3u32);\nrating(stars, 5)"));
    append_node(&page, &code);

    append_node(&page, &api_table(&[
        ("value", "Signal<u32>", "\u{2014}", "Two-way bound rating value"),
        ("max", "u32", "\u{2014}", "Maximum number of stars"),
    ]));

    page
}

fn pg_copy_button() -> web_sys::Element {
    let (page, preview, _, _) = pg_shell("Copy Button", "Button that copies text to the clipboard.");

    append_node(&preview, &components::copy_button("Hello from Oxide!"));

    let code = create_element("pre");
    set_attribute(&code, "class", "pg-code");
    code.set_text_content(Some("copy_button(\"Hello from Oxide!\")"));
    append_node(&page, &code);

    append_node(&page, &api_table(&[
        ("text", "&str", "\u{2014}", "Text to copy to clipboard on click"),
    ]));

    page
}

fn pg_empty_state() -> web_sys::Element {
    let (page, preview, _, _) = pg_shell("Empty State", "No-data empty state placeholder with icon and description.");

    set_style(&preview, "flex-direction", "column");
    set_style(&preview, "align-items", "stretch");
    append_node(&preview, &components::empty_state(
        "No results found",
        "Try adjusting your search or filters to find what you\u{2019}re looking for.",
        "\u{1f50d}",
    ));

    let code = create_element("pre");
    set_attribute(&code, "class", "pg-code");
    code.set_text_content(Some("empty_state(\n    \"No results found\",\n    \"Try adjusting your search.\",\n    \"\u{1f50d}\",\n)"));
    append_node(&page, &code);

    append_node(&page, &api_table(&[
        ("title", "&str", "\u{2014}", "Empty state heading"),
        ("description", "&str", "\u{2014}", "Descriptive help text"),
        ("icon", "&str", "\u{2014}", "Emoji or icon character"),
    ]));

    page
}

fn pg_loading_overlay() -> web_sys::Element {
    let (page, preview, _, _) = pg_shell("Loading Overlay", "Full-screen loading overlay controlled by a signal.");

    let visible = signal(false);

    let show_btn = components::button("Show Overlay (2s)").primary().build();
    add_event_listener(&show_btn, "click", move |_| {
        visible.set(true);
        set_timeout(move || { visible.set(false); }, 2000);
    });
    append_node(&preview, &show_btn);
    append_node(&preview, &components::loading_overlay(visible));

    let code = create_element("pre");
    set_attribute(&code, "class", "pg-code");
    code.set_text_content(Some("let visible = signal(false);\nloading_overlay(visible)"));
    append_node(&page, &code);

    append_node(&page, &api_table(&[
        ("visible", "Signal<bool>", "false", "Controls overlay visibility"),
    ]));

    page
}

fn pg_layout() -> web_sys::Element {
    let (page, _, _, _) = pg_shell("Layout", "Layout utilities: hstack, vstack, center, grid, spacer, container.");

    // hstack example
    let h_section = el("div", "pg-preview", &[]);
    set_style(&h_section, "flex-direction", "column");
    set_style(&h_section, "align-items", "stretch");
    let h_label = text_el("strong", "hstack");
    append_node(&h_section, &h_label);
    let h_demo = components::hstack("0.5rem", vec![
        components::button("A").build(),
        components::button("B").primary().build(),
        components::button("C").outline().build(),
    ]);
    append_node(&h_section, &h_demo);
    append_node(&page, &h_section);

    // vstack example
    let v_section = el("div", "pg-preview", &[]);
    set_style(&v_section, "flex-direction", "column");
    set_style(&v_section, "align-items", "stretch");
    let v_label = text_el("strong", "vstack");
    append_node(&v_section, &v_label);
    let v_demo = components::vstack("0.5rem", vec![
        components::badge("Item 1", Severity::Info),
        components::badge("Item 2", Severity::Success),
        components::badge("Item 3", Severity::Warning),
    ]);
    append_node(&v_section, &v_demo);
    append_node(&page, &v_section);

    // grid example
    let g_section = el("div", "pg-preview", &[]);
    set_style(&g_section, "flex-direction", "column");
    set_style(&g_section, "align-items", "stretch");
    let g_label = text_el("strong", "grid (3 columns)");
    append_node(&g_section, &g_label);
    let g_demo = components::grid(3, "0.5rem", vec![
        components::stat("42", "Users"),
        components::stat("128", "Posts"),
        components::stat("99%", "Uptime"),
    ]);
    append_node(&g_section, &g_demo);
    append_node(&page, &g_section);

    // center example
    let c_section = el("div", "pg-preview", &[]);
    set_style(&c_section, "flex-direction", "column");
    set_style(&c_section, "align-items", "stretch");
    set_style(&c_section, "min-height", "100px");
    let c_label = text_el("strong", "center");
    append_node(&c_section, &c_label);
    append_node(&c_section, &components::center(components::spinner()));
    append_node(&page, &c_section);

    let code = create_element("pre");
    set_attribute(&code, "class", "pg-code");
    code.set_text_content(Some("hstack(\"1rem\", vec![a, b, c])\nvstack(\"1rem\", vec![x, y, z])\ngrid(3, \"1rem\", items)\ncenter(spinner())\ncontainer(content)\nspacer()"));
    append_node(&page, &code);

    append_node(&page, &api_table(&[
        ("hstack(gap, children)", "Element", "\u{2014}", "Horizontal flex row"),
        ("vstack(gap, children)", "Element", "\u{2014}", "Vertical flex column"),
        ("grid(cols, gap, children)", "Element", "\u{2014}", "CSS grid with N equal columns"),
        ("center(child)", "Element", "\u{2014}", "Center element horizontally and vertically"),
        ("container(child)", "Element", "\u{2014}", "Max-width 1200px centered wrapper"),
        ("spacer()", "Element", "\u{2014}", "Flexible spacer (flex: 1)"),
    ]));

    page
}

fn pg_kbd() -> web_sys::Element {
    let (page, preview, _, _) = pg_shell("Kbd", "Keyboard shortcut display element.");

    let shortcuts = el("div", "", &[]);
    set_style(&shortcuts, "display", "flex");
    set_style(&shortcuts, "gap", "1rem");
    set_style(&shortcuts, "align-items", "center");
    set_style(&shortcuts, "flex-wrap", "wrap");
    append_node(&shortcuts, &components::kbd("Ctrl+C"));
    append_node(&shortcuts, &components::kbd("Ctrl+V"));
    append_node(&shortcuts, &components::kbd("Ctrl+Z"));
    append_node(&shortcuts, &components::kbd("Shift+Enter"));
    append_node(&preview, &shortcuts);

    let code = create_element("pre");
    set_attribute(&code, "class", "pg-code");
    code.set_text_content(Some("kbd(\"Ctrl+C\")\nkbd(\"Ctrl+V\")\nkbd(\"Shift+Enter\")"));
    append_node(&page, &code);

    append_node(&page, &api_table(&[
        ("keys", "&str", "\u{2014}", "Keyboard shortcut text (e.g. \"Ctrl+C\")"),
    ]));

    page
}

fn pg_code_block() -> web_sys::Element {
    let (page, preview, _, _) = pg_shell("Code Block", "Syntax-highlighted code block display.");

    set_style(&preview, "flex-direction", "column");
    set_style(&preview, "align-items", "stretch");
    append_node(&preview, &components::code_block(
        "fn main() {\n    let count = signal(0);\n    mount(\"#app\", || {\n        button(\"Click me\")\n            .on_click(move |_| count += 1)\n    });\n}"
    ));

    let code = create_element("pre");
    set_attribute(&code, "class", "pg-code");
    code.set_text_content(Some("code_block(\"fn main() { ... }\")"));
    append_node(&page, &code);

    append_node(&page, &api_table(&[
        ("code", "&str", "\u{2014}", "Source code text to display"),
    ]));

    page
}

fn pg_file_upload() -> web_sys::Element {
    let (page, preview, controls, _) = pg_shell("File Upload", "File upload drop zone with callback.");

    let file_name = signal("(no file selected)".to_string());

    let display = el("div", "mono", &[]);
    let display_ref = display.clone();
    create_effect(move || {
        display_ref.set_text_content(Some(&file_name.get()));
    });
    append_node(&controls, &display);

    set_style(&preview, "flex-direction", "column");
    set_style(&preview, "align-items", "stretch");
    append_node(&preview, &components::file_upload(move |name, _content| {
        file_name.set(format!("Selected: {}", name));
    }));

    let code = create_element("pre");
    set_attribute(&code, "class", "pg-code");
    code.set_text_content(Some("file_upload(|name, content| {\n    log(&format!(\"File: {}\", name));\n})"));
    append_node(&page, &code);

    append_node(&page, &api_table(&[
        ("on_file", "FnMut(String, String)", "\u{2014}", "Callback with (filename, content)"),
    ]));

    page
}

fn pg_form_group() -> web_sys::Element {
    let (page, preview, controls, _) = pg_shell("Form Group", "Form group wrapper with label and optional error.");

    let show_error = signal(false);
    append_node(&controls, &components::checkbox("Show error", show_error));

    let preview_ref = preview.clone();
    create_effect(move || {
        clear_children(&preview_ref);
        set_style(&preview_ref, "flex-direction", "column");
        set_style(&preview_ref, "align-items", "stretch");
        let input = components::text_input("").placeholder("Enter email").build();
        let mut fg = components::form_group("Email Address", input);
        if show_error.get() {
            fg = fg.error("Please enter a valid email address");
        }
        append_node(&preview_ref, &fg.build());
    });

    let code = create_element("pre");
    set_attribute(&code, "class", "pg-code");
    code.set_text_content(Some("form_group(\"Email\", text_input(\"\").build())\n    .error(\"Invalid email\")\n    .build()"));
    append_node(&page, &code);

    append_node(&page, &api_table(&[
        ("label", "&str", "\u{2014}", "Group label text"),
        ("child", "Element", "\u{2014}", "Form input element"),
        (".error(msg)", "self", "\u{2014}", "Validation error message"),
        (".build()", "Element", "\u{2014}", "Build the form group"),
    ]));

    page
}

// ═══════════════════════════════════════════════════════════════════════════
// Page: Forms — form composition patterns
// ═══════════════════════════════════════════════════════════════════════════

fn page_forms() -> web_sys::Element {
    let page = el("div", "pg-page", &[]);
    let h2 = text_el("h2", "Form Patterns");
    append_node(&page, &h2);
    let desc = el("p", "pg-desc", &[]);
    append_text(&desc, "Composing Oxide components into real-world forms.");
    append_node(&page, &desc);

    // ── Login Form ──
    let section1 = el("div", "demo-section", &[]);
    append_node(&section1, &text_el("h3", "\u{1f512} Login Form"));
    let login_preview = el("div", "pg-preview", &[]);
    set_style(&login_preview, "flex-direction", "column");
    set_style(&login_preview, "align-items", "stretch");
    let email = signal(String::new());
    let password = signal(String::new());
    let login_card = components::card("Sign In")
        .body(components::vstack("1rem", vec![
            components::text_input("Email").placeholder("you@example.com").input_type("email").bind(email).build(),
            components::password_input("Password", password),
            components::button("Sign In").primary().build(),
        ]))
        .build();
    append_node(&login_preview, &login_card);
    append_node(&section1, &login_preview);
    let code1 = create_element("pre");
    set_attribute(&code1, "class", "pg-code");
    code1.set_text_content(Some("card(\"Sign In\").body(vstack(\"1rem\", vec![\n    text_input(\"Email\").input_type(\"email\").bind(email).build(),\n    password_input(\"Password\", pw),\n    button(\"Sign In\").primary().build(),\n])).build()"));
    append_node(&section1, &code1);
    append_node(&page, &section1);

    // ── Signup Form ──
    let section2 = el("div", "demo-section", &[]);
    append_node(&section2, &text_el("h3", "\u{1f4dd} Signup Form"));
    let signup_preview = el("div", "pg-preview", &[]);
    set_style(&signup_preview, "flex-direction", "column");
    set_style(&signup_preview, "align-items", "stretch");
    let s_name = signal(String::new());
    let s_email = signal(String::new());
    let s_pw = signal(String::new());
    let s_terms = signal(false);
    let signup_card = components::card("Create Account")
        .body(components::vstack("1rem", vec![
            components::text_input("Name").placeholder("Jane Doe").bind(s_name).build(),
            components::text_input("Email").placeholder("jane@example.com").input_type("email").bind(s_email).build(),
            components::password_input("Password", s_pw),
            components::checkbox("I agree to the Terms of Service", s_terms),
            components::button("Create Account").primary().build(),
        ]))
        .build();
    append_node(&signup_preview, &signup_card);
    append_node(&section2, &signup_preview);
    let code2 = create_element("pre");
    set_attribute(&code2, "class", "pg-code");
    code2.set_text_content(Some("card(\"Create Account\").body(vstack(\"1rem\", vec![\n    text_input(\"Name\").bind(name).build(),\n    text_input(\"Email\").input_type(\"email\").bind(email).build(),\n    password_input(\"Password\", pw),\n    checkbox(\"I agree to the Terms\", terms),\n    button(\"Create Account\").primary().build(),\n])).build()"));
    append_node(&section2, &code2);
    append_node(&page, &section2);

    // ── Settings Form ──
    let section3 = el("div", "demo-section", &[]);
    append_node(&section3, &text_el("h3", "\u{2699}\u{fe0f} Settings Form"));
    let settings_preview = el("div", "pg-preview", &[]);
    set_style(&settings_preview, "flex-direction", "column");
    set_style(&settings_preview, "align-items", "stretch");
    let dark_mode = signal(false);
    let font_size = signal(16.0f64);
    let language = signal("en".to_string());
    let settings_card = components::card("Preferences")
        .body(components::vstack("1rem", vec![
            components::toggle("Dark Mode", dark_mode),
            components::slider(12.0, 24.0, 1.0, font_size),
            components::select("Language", &[("en", "English"), ("es", "Espa\u{f1}ol"), ("fr", "Fran\u{e7}ais")], language),
            components::button("Save Settings").primary().build(),
        ]))
        .build();
    append_node(&settings_preview, &settings_card);
    append_node(&section3, &settings_preview);
    let code3 = create_element("pre");
    set_attribute(&code3, "class", "pg-code");
    code3.set_text_content(Some("card(\"Preferences\").body(vstack(\"1rem\", vec![\n    toggle(\"Dark Mode\", dark_mode),\n    slider(12.0, 24.0, 1.0, font_size),\n    select(\"Language\", &[(\"en\",\"English\")], lang),\n    button(\"Save\").primary().build(),\n])).build()"));
    append_node(&section3, &code3);
    append_node(&page, &section3);

    page
}

// ═══════════════════════════════════════════════════════════════════════════
// Page: Composition — layout composition patterns
// ═══════════════════════════════════════════════════════════════════════════

fn page_composition() -> web_sys::Element {
    let page = el("div", "pg-page", &[]);
    let h2 = text_el("h2", "Composition Patterns");
    append_node(&page, &h2);
    let desc = el("p", "pg-desc", &[]);
    append_text(&desc, "Combining layout utilities and components into complex UIs.");
    append_node(&page, &desc);

    // ── Dashboard Layout ──
    let section1 = el("div", "demo-section", &[]);
    append_node(&section1, &text_el("h3", "\u{1f4ca} Dashboard Layout"));
    let dash_preview = el("div", "pg-preview", &[]);
    set_style(&dash_preview, "flex-direction", "column");
    set_style(&dash_preview, "align-items", "stretch");
    let stat_grid = components::grid(3, "1rem", vec![
        components::stat("1,234", "Total Users"),
        components::stat("$56.7K", "Revenue"),
        components::stat("99.9%", "Uptime"),
    ]);
    append_node(&dash_preview, &stat_grid);
    append_node(&section1, &dash_preview);
    let code1 = create_element("pre");
    set_attribute(&code1, "class", "pg-code");
    code1.set_text_content(Some("grid(3, \"1rem\", vec![\n    stat(\"1,234\", \"Total Users\"),\n    stat(\"$56.7K\", \"Revenue\"),\n    stat(\"99.9%\", \"Uptime\"),\n])"));
    append_node(&section1, &code1);
    append_node(&page, &section1);

    // ── Card Grid ──
    let section2 = el("div", "demo-section", &[]);
    append_node(&section2, &text_el("h3", "\u{1f4c7} Card Grid"));
    let card_preview = el("div", "pg-preview", &[]);
    set_style(&card_preview, "flex-direction", "column");
    set_style(&card_preview, "align-items", "stretch");
    let card_grid = components::grid(2, "1rem", vec![
        components::card("Users").body(text_el("p", "Manage user accounts and permissions.")).build(),
        components::card("Analytics").body(text_el("p", "View traffic and engagement metrics.")).build(),
        components::card("Settings").body(text_el("p", "Configure application preferences.")).build(),
        components::card("Billing").body(text_el("p", "Manage subscriptions and invoices.")).build(),
    ]);
    append_node(&card_preview, &card_grid);
    append_node(&section2, &card_preview);
    let code2 = create_element("pre");
    set_attribute(&code2, "class", "pg-code");
    code2.set_text_content(Some("grid(2, \"1rem\", vec![\n    card(\"Users\").body(content).build(),\n    card(\"Analytics\").body(content).build(),\n])"));
    append_node(&section2, &code2);
    append_node(&page, &section2);

    // ── Data Table with Pagination ──
    let section3 = el("div", "demo-section", &[]);
    append_node(&section3, &text_el("h3", "\u{1f4cb} Data Table with Pagination"));
    let table_preview = el("div", "pg-preview", &[]);
    set_style(&table_preview, "flex-direction", "column");
    set_style(&table_preview, "align-items", "stretch");
    let total_pg = signal(5u32);
    let curr_pg = signal(1u32);
    append_node(&table_preview, &components::data_table(
        &["ID", "Name", "Email", "Role"],
        &[
            vec!["1".into(), "Alice".into(), "alice@oxide.dev".into(), "Admin".into()],
            vec!["2".into(), "Bob".into(), "bob@oxide.dev".into(), "Editor".into()],
            vec!["3".into(), "Carol".into(), "carol@oxide.dev".into(), "Viewer".into()],
        ],
    ));
    append_node(&table_preview, &components::pagination(total_pg, curr_pg));
    append_node(&section3, &table_preview);
    let code3 = create_element("pre");
    set_attribute(&code3, "class", "pg-code");
    code3.set_text_content(Some("vstack(\"1rem\", vec![\n    data_table(&[\"ID\",\"Name\",\"Email\"], &rows),\n    pagination(total, current),\n])"));
    append_node(&section3, &code3);
    append_node(&page, &section3);

    // ── Alert + Toast Flow ──
    let section4 = el("div", "demo-section", &[]);
    append_node(&section4, &text_el("h3", "\u{1f514} Notification Flow"));
    let notif_preview = el("div", "pg-preview", &[]);
    set_style(&notif_preview, "flex-direction", "column");
    set_style(&notif_preview, "align-items", "stretch");
    set_style(&notif_preview, "gap", "1rem");
    append_node(&notif_preview, &components::alert("Deployment successful!").success().build());
    append_node(&notif_preview, &components::alert("Disk usage at 85%.").warning().build());
    let toast_btn = components::button("Trigger Toast").primary().build();
    add_event_listener(&toast_btn, "click", |_| {
        components::toast("Saved successfully!").severity(Severity::Success).duration_ms(2000).show();
    });
    append_node(&notif_preview, &toast_btn);
    append_node(&section4, &notif_preview);
    let code4 = create_element("pre");
    set_attribute(&code4, "class", "pg-code");
    code4.set_text_content(Some("alert(\"Deployment successful!\").success().build()\nalert(\"Disk usage at 85%.\").warning().build()\ntoast(\"Saved!\").severity(Severity::Success).show()"));
    append_node(&section4, &code4);
    append_node(&page, &section4);

    page
}

// ═══════════════════════════════════════════════════════════════════════════
// Tutorial: Build a Login Form
// ═══════════════════════════════════════════════════════════════════════════

fn tutorial_login() -> web_sys::Element {
    let page = el("div", "pg-page", &[]);
    let h2 = text_el("h2", "Tutorial: Build a Login Form");
    append_node(&page, &h2);
    let desc = el("p", "pg-desc", &[]);
    append_text(&desc, "Step-by-step guide to building a login form with Oxide components.");
    append_node(&page, &desc);

    // Step 1
    let s1 = el("div", "demo-section", &[]);
    append_node(&s1, &text_el("h3", "Step 1 \u{2014} Create Signals"));
    append_node(&s1, &text_el("p", "Start by creating reactive signals for your form state:"));
    let c1 = create_element("pre");
    set_attribute(&c1, "class", "pg-code");
    c1.set_text_content(Some("let email = signal(String::new());\nlet password = signal(String::new());\nlet error = signal(String::new());\nlet loading = signal(false);"));
    append_node(&s1, &c1);
    append_node(&page, &s1);

    // Step 2
    let s2 = el("div", "demo-section", &[]);
    append_node(&s2, &text_el("h3", "Step 2 \u{2014} Build Inputs"));
    append_node(&s2, &text_el("p", "Create form inputs bound to signals:"));
    let c2 = create_element("pre");
    set_attribute(&c2, "class", "pg-code");
    c2.set_text_content(Some("let email_input = text_input(\"Email\")\n    .placeholder(\"you@example.com\")\n    .input_type(\"email\")\n    .required()\n    .bind(email)\n    .build();\n\nlet pw_input = password_input(\"Password\", password);"));
    append_node(&s2, &c2);
    append_node(&page, &s2);

    // Step 3
    let s3 = el("div", "demo-section", &[]);
    append_node(&s3, &text_el("h3", "Step 3 \u{2014} Add Validation"));
    append_node(&s3, &text_el("p", "Validate inputs before submission:"));
    let c3 = create_element("pre");
    set_attribute(&c3, "class", "pg-code");
    c3.set_text_content(Some("let validate = move || -> bool {\n    let e = email.get();\n    let p = password.get();\n    if e.is_empty() || !e.contains('@') {\n        error.set(\"Please enter a valid email\".into());\n        return false;\n    }\n    if p.len() < 8 {\n        error.set(\"Password must be 8+ characters\".into());\n        return false;\n    }\n    error.set(String::new());\n    true\n};"));
    append_node(&s3, &c3);
    append_node(&page, &s3);

    // Step 4
    let s4 = el("div", "demo-section", &[]);
    append_node(&s4, &text_el("h3", "Step 4 \u{2014} Submit Handler"));
    append_node(&s4, &text_el("p", "Create the submit button with loading state:"));
    let c4 = create_element("pre");
    set_attribute(&c4, "class", "pg-code");
    c4.set_text_content(Some("let submit = button(\"Sign In\")\n    .primary()\n    .loading(loading.get())\n    .on_click(move |_| {\n        if validate() {\n            loading.set(true);\n            // perform async login...\n        }\n    });"));
    append_node(&s4, &c4);
    append_node(&page, &s4);

    // Step 5 — Final composition with live preview
    let s5 = el("div", "demo-section", &[]);
    append_node(&s5, &text_el("h3", "Step 5 \u{2014} Compose the Form"));
    append_node(&s5, &text_el("p", "Wrap everything in a card:"));
    let c5 = create_element("pre");
    set_attribute(&c5, "class", "pg-code");
    c5.set_text_content(Some("card(\"Sign In\")\n    .body(vstack(\"1rem\", vec![\n        email_input,\n        pw_input,\n        submit,\n    ]))\n    .build()"));
    append_node(&s5, &c5);
    // Live preview
    let live = el("div", "pg-preview", &[]);
    set_style(&live, "flex-direction", "column");
    set_style(&live, "align-items", "stretch");
    set_style(&live, "margin-top", "1rem");
    let t_email = signal(String::new());
    let t_pw = signal(String::new());
    let final_form = components::card("Sign In")
        .body(components::vstack("1rem", vec![
            components::text_input("Email").placeholder("you@example.com").input_type("email").required().bind(t_email).build(),
            components::password_input("Password", t_pw),
            components::button("Sign In").primary().build(),
        ]))
        .build();
    append_node(&live, &final_form);
    append_node(&s5, &live);
    append_node(&page, &s5);

    // Navigation to dashboard tutorial
    let nav_link = el("div", "", &[]);
    set_style(&nav_link, "margin-top", "2rem");
    let next = components::button("Next: Dashboard Tutorial \u{2192}").outline().build();
    add_event_listener(&next, "click", |_| { navigate("/tutorials/dashboard"); });
    append_node(&nav_link, &next);
    append_node(&page, &nav_link);

    page
}

// ═══════════════════════════════════════════════════════════════════════════
// Tutorial: Build a Dashboard
// ═══════════════════════════════════════════════════════════════════════════

fn tutorial_dashboard() -> web_sys::Element {
    let page = el("div", "pg-page", &[]);
    let h2 = text_el("h2", "Tutorial: Build a Dashboard");
    append_node(&page, &h2);
    let desc = el("p", "pg-desc", &[]);
    append_text(&desc, "Step-by-step guide to building a dashboard with stats, tables, and charts.");
    append_node(&page, &desc);

    // Step 1 — Layout
    let s1 = el("div", "demo-section", &[]);
    append_node(&s1, &text_el("h3", "Step 1 \u{2014} Create the Layout"));
    append_node(&s1, &text_el("p", "Start with a vertical stack for the main layout:"));
    let c1 = create_element("pre");
    set_attribute(&c1, "class", "pg-code");
    c1.set_text_content(Some("let dashboard = vstack(\"2rem\", vec![\n    text_el(\"h1\", \"Dashboard\"),\n    stat_row,\n    data_section,\n]);"));
    append_node(&s1, &c1);
    append_node(&page, &s1);

    // Step 2 — Stat Cards
    let s2 = el("div", "demo-section", &[]);
    append_node(&s2, &text_el("h3", "Step 2 \u{2014} Add Stat Cards"));
    append_node(&s2, &text_el("p", "Use a grid of stat components for key metrics:"));
    let c2 = create_element("pre");
    set_attribute(&c2, "class", "pg-code");
    c2.set_text_content(Some("let stat_row = grid(4, \"1rem\", vec![\n    stat(\"2,451\", \"Users\"),\n    stat(\"$12.4K\", \"Revenue\"),\n    stat(\"342\", \"Orders\"),\n    stat(\"99.9%\", \"Uptime\"),\n]);"));
    append_node(&s2, &c2);
    // Live preview
    let live2 = el("div", "pg-preview", &[]);
    set_style(&live2, "flex-direction", "column");
    set_style(&live2, "align-items", "stretch");
    append_node(&live2, &components::grid(4, "1rem", vec![
        components::stat("2,451", "Users"),
        components::stat("$12.4K", "Revenue"),
        components::stat("342", "Orders"),
        components::stat("99.9%", "Uptime"),
    ]));
    append_node(&s2, &live2);
    append_node(&page, &s2);

    // Step 3 — Data Table
    let s3 = el("div", "demo-section", &[]);
    append_node(&s3, &text_el("h3", "Step 3 \u{2014} Add a Data Table"));
    append_node(&s3, &text_el("p", "Display tabular data with the data_table component:"));
    let c3 = create_element("pre");
    set_attribute(&c3, "class", "pg-code");
    c3.set_text_content(Some("let table = data_table(\n    &[\"Order\", \"Customer\", \"Amount\", \"Status\"],\n    &[\n        vec![\"#1001\".into(), \"Alice\".into(), \"$99\".into(), \"Shipped\".into()],\n        vec![\"#1002\".into(), \"Bob\".into(), \"$149\".into(), \"Processing\".into()],\n    ],\n);"));
    append_node(&s3, &c3);
    let live3 = el("div", "pg-preview", &[]);
    set_style(&live3, "flex-direction", "column");
    set_style(&live3, "align-items", "stretch");
    append_node(&live3, &components::data_table(
        &["Order", "Customer", "Amount", "Status"],
        &[
            vec!["#1001".into(), "Alice".into(), "$99".into(), "Shipped".into()],
            vec!["#1002".into(), "Bob".into(), "$149".into(), "Processing".into()],
            vec!["#1003".into(), "Carol".into(), "$249".into(), "Delivered".into()],
        ],
    ));
    append_node(&s3, &live3);
    append_node(&page, &s3);

    // Step 4 — Putting it all together
    let s4 = el("div", "demo-section", &[]);
    append_node(&s4, &text_el("h3", "Step 4 \u{2014} Compose Everything"));
    append_node(&s4, &text_el("p", "Combine the layout, stats, table, and timeline into a full dashboard:"));
    let c4 = create_element("pre");
    set_attribute(&c4, "class", "pg-code");
    c4.set_text_content(Some("container(vstack(\"2rem\", vec![\n    hstack(\"1rem\", vec![\n        text_el(\"h1\", \"Dashboard\"),\n        spacer(),\n        button(\"Export\").outline().build(),\n    ]),\n    stat_row,\n    card(\"Recent Orders\").body(table).build(),\n    card(\"Activity\").body(timeline).build(),\n]))"));
    append_node(&s4, &c4);
    let live4 = el("div", "pg-preview", &[]);
    set_style(&live4, "flex-direction", "column");
    set_style(&live4, "align-items", "stretch");
    append_node(&live4, &components::vstack("1.5rem", vec![
        components::hstack("1rem", vec![
            text_el("h3", "\u{1f4ca} Dashboard Preview"),
            components::spacer(),
            components::button("Export").outline().build(),
        ]),
        components::grid(3, "1rem", vec![
            components::stat("2,451", "Users"),
            components::stat("$12.4K", "Revenue"),
            components::stat("99.9%", "Uptime"),
        ]),
        components::card("Activity").body(
            components::timeline(&[
                ("Deploy v2.1", "Production deployment completed."),
                ("New signup", "Alice joined the platform."),
                ("Alert resolved", "CPU usage back to normal."),
            ])
        ).build(),
    ]));
    append_node(&s4, &live4);
    append_node(&page, &s4);

    // Navigation
    let nav_link = el("div", "", &[]);
    set_style(&nav_link, "margin-top", "2rem");
    let prev = components::button("\u{2190} Login Tutorial").outline().build();
    add_event_listener(&prev, "click", |_| { navigate("/tutorials/login"); });
    append_node(&nav_link, &prev);
    append_node(&page, &nav_link);

    page
}
// ═══════════════════════════════════════════════════════════════════════════

fn demo_counter() -> web_sys::Element {
    let mut count = signal(0i32);

    view! {
        <div class="col">
            <div class="big-num">{count}</div>
            <div class="row counter-btns">
                <button on:click={move |_: Event| { count -= 1; }}>"−"</button>
                <button on:click={move |_: Event| { count.set(0); }}>"Reset"</button>
                <button on:click={move |_: Event| { count += 1; }}>"+"</button>
            </div>
        </div>
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. Temperature Converter
// ═══════════════════════════════════════════════════════════════════════════

fn demo_temperature() -> web_sys::Element {
    let celsius = signal("0".to_string());
    let fahrenheit = signal("32".to_string());

    view! {
        <div class="col">
            <div class="temp-row">
                <label>"Celsius"</label>
                <input type="number" bind:value={celsius}
                    on:input={move |e: Event| {
                        if let Ok(c) = event_target_value(&e).parse::<f64>() {
                            fahrenheit.set(format!("{:.1}", c * 9.0 / 5.0 + 32.0));
                        }
                    }} />
            </div>
            <div class="temp-row">
                <label>"Fahrenheit"</label>
                <input type="number" bind:value={fahrenheit}
                    on:input={move |e: Event| {
                        if let Ok(f) = event_target_value(&e).parse::<f64>() {
                            celsius.set(format!("{:.1}", (f - 32.0) * 5.0 / 9.0));
                        }
                    }} />
            </div>
        </div>
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. Todo List
// ═══════════════════════════════════════════════════════════════════════════

fn demo_todo() -> web_sys::Element {
    let todos: Signal<Vec<(String, bool)>> = signal({
        if let Some(saved) = local_storage_get("oxide-todos") {
            parse_todos(&saved)
        } else {
            vec![
                ("Learn Rust".into(), true),
                ("Build with Oxide".into(), false),
                ("Deploy to WASM".into(), false),
            ]
        }
    });
    let input_val = signal(String::new());
    let filter = signal(0u8); // 0=all, 1=active, 2=done

    let content = el("div", "col", &[]);

    // Input row with bind:value
    let input_row = view! {
        <div class="todo-input-row">
            <input type="text" placeholder="What needs to be done?"
                bind:value={input_val}
                on:keydown={move |e: Event| {
                    let ke: web_sys::KeyboardEvent = e.dyn_into().unwrap();
                    if ke.key() == "Enter" {
                        let v = input_val.get();
                        if !v.trim().is_empty() {
                            todos.update(|list| list.push((v, false)));
                            input_val.set(String::new());
                        }
                    }
                }} />
            <button class="btn-primary" on:click={move |_: Event| {
                let v = input_val.get();
                if !v.trim().is_empty() {
                    todos.update(|list| list.push((v, false)));
                    input_val.set(String::new());
                }
            }}>"Add"</button>
        </div>
    };
    append_node(&content, &input_row);

    // Filters
    let filters = el("div", "todo-filters", &[]);
    let mut filter_btns: Vec<web_sys::Element> = Vec::new();
    for (i, label) in ["All", "Active", "Done"].iter().enumerate() {
        let btn = text_el("button", label);
        set_attribute(&btn, "class", "btn-sm");
        let idx = i as u8;
        add_event_listener(&btn, "click", move |_| { filter.set(idx); });
        filter_btns.push(btn.clone());
        append_node(&filters, &btn);
    }
    append_node(&content, &filters);

    create_effect(move || {
        let f = filter.get() as usize;
        for (i, btn) in filter_btns.iter().enumerate() {
            if i == f {
                set_attribute(btn, "class", "btn-sm active");
            } else {
                set_attribute(btn, "class", "btn-sm");
            }
        }
    });

    // Todo list using {for ...} with pre-filtered data and {if ...} for checkbox state
    let list = view! {
        <ul class="todo-list">
            {for (idx, text, done) in visible_todos(&todos.get(), filter.get()).into_iter() {
                <li class="todo-item" class:done={done}>
                    {if done {
                        <input type="checkbox" checked="checked"
                            on:change={move |_: Event| { todos.update(|list| { list[idx].1 = !list[idx].1; }); }} />
                    } else {
                        <input type="checkbox"
                            on:change={move |_: Event| { todos.update(|list| { list[idx].1 = !list[idx].1; }); }} />
                    }}
                    <span>{text}</span>
                    <button class="btn-sm btn-danger"
                        on:click={move |_: Event| { todos.update(|list| { list.remove(idx); }); }}>"✕"</button>
                </li>
            }}
        </ul>
    };
    append_node(&content, &list);

    // Persist on change
    create_effect(move || {
        let items = todos.get();
        local_storage_set("oxide-todos", &serialize_todos(&items));
    });

    // Count display
    let count_el = el("div", "todo-count", &[]);
    let count_ref = count_el.clone();
    create_effect(move || {
        let items = todos.get();
        let active = items.iter().filter(|(_, d)| !d).count();
        count_ref.set_inner_html(&format!("{} item{} remaining", active, if active == 1 { "" } else { "s" }));
    });
    append_node(&content, &count_el);

    content
}

fn visible_todos(items: &[(String, bool)], f: u8) -> Vec<(usize, String, bool)> {
    items.iter().enumerate()
        .filter(|(_, (_, d))| match f { 1 => !d, 2 => *d, _ => true })
        .map(|(i, (t, d))| (i, t.clone(), *d))
        .collect()
}

fn parse_todos(s: &str) -> Vec<(String, bool)> {
    s.lines().filter_map(|line| {
        let (done, text) = if let Some(t) = line.strip_prefix("[x] ") { (true, t) }
        else if let Some(t) = line.strip_prefix("[ ] ") { (false, t) }
        else { return None; };
        Some((text.to_string(), done))
    }).collect()
}

fn serialize_todos(todos: &[(String, bool)]) -> String {
    todos.iter().map(|(t, d)| format!("{} {}", if *d { "[x]" } else { "[ ]" }, t)).collect::<Vec<_>>().join("\n")
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. Stopwatch
// ═══════════════════════════════════════════════════════════════════════════

fn demo_stopwatch() -> web_sys::Element {
    let elapsed_ms = signal(0u64);
    let running = signal(false);
    let interval_id = signal(0i32);
    let display = memo(move || {
        let ms = elapsed_ms.get();
        let mins = ms / 60000;
        let secs = (ms % 60000) / 1000;
        let centis = (ms % 1000) / 10;
        format!("{:02}:{:02}.{:02}", mins, secs, centis)
    });
    let btn_label = memo(move || {
        if running.get() { "Pause".to_string() } else { "Start".to_string() }
    });

    view! {
        <div class="col">
            <div class="stopwatch-time">{display}</div>
            <div class="stopwatch-btns">
                <button class="btn-primary"
                    on:click={move |_: Event| {
                        if running.get() {
                            clear_interval(interval_id.get());
                            running.set(false);
                        } else {
                            let e = elapsed_ms;
                            let id = set_interval(move || { e.update(|v| *v += 10); }, 10);
                            interval_id.set(id);
                            running.set(true);
                        }
                    }}>{btn_label}</button>
                <button on:click={move |_: Event| {
                    clear_interval(interval_id.get());
                    running.set(false);
                    elapsed_ms.set(0);
                }}>"Reset"</button>
            </div>
        </div>
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. Form Playground
// ═══════════════════════════════════════════════════════════════════════════

fn demo_forms() -> web_sys::Element {
    let name = signal(String::new());
    let email = signal(String::new());
    let color = signal("#f97316".to_string());
    let range_val = signal("50".to_string());
    let checked = signal(false);
    let select_val = signal("rust".to_string());

    let output_text = memo(move || {
        format!("{{ name: \"{}\", email: \"{}\", color: \"{}\", volume: {}, subscribed: {}, lang: \"{}\" }}",
            name.get(), email.get(), color.get(), range_val.get(), checked.get(), select_val.get())
    });

    view! {
        <div class="col">
            <div class="form-grid">
                <div class="form-field">
                    <label>"Name"</label>
                    <input type="text" placeholder="Your name" bind:value={name} />
                </div>
                <div class="form-field">
                    <label>"Email"</label>
                    <input type="text" placeholder="you@example.com" bind:value={email} />
                </div>
                <div class="form-field">
                    <label>"Favorite Color"</label>
                    <input type="color" bind:value={color} />
                </div>
                <div class="form-field">
                    <label>"Volume: " {range_val}</label>
                    <input type="range" min="0" max="100" bind:value={range_val} />
                </div>
                <div class="form-field">
                    <label>"Subscribe"</label>
                    <div class="row">
                        <input type="checkbox" bind:checked={checked} />
                        <span>"Send me updates"</span>
                    </div>
                </div>
                <div class="form-field">
                    <label>"Language"</label>
                    <select on:change={move |e: Event| { select_val.set(event_target_value(&e)); }}>
                        <option value="rust">"Rust"</option>
                        <option value="ts">"TypeScript"</option>
                        <option value="go">"Go"</option>
                        <option value="python">"Python"</option>
                    </select>
                </div>
            </div>
            <div class="form-output">
                <code>{output_text}</code>
            </div>
        </div>
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. Fetch API
// ═══════════════════════════════════════════════════════════════════════════

fn demo_fetch() -> web_sys::Element {
    let result = signal("Click a button to fetch data.".to_string());
    let loading = signal(false);

    fn do_fetch(url: &'static str, result: Signal<String>, loading: Signal<bool>) {
        loading.set(true);
        result.set("Loading...".into());
        wasm_bindgen_futures::spawn_local(async move {
            match fetch_text(url).await {
                Ok(text) => { result.set(text); loading.set(false); }
                Err(e) => { result.set(format!("Error: {:?}", e)); loading.set(false); }
            }
        });
    }

    view! {
        <div class="col">
            <div class="row">
                <button on:click={move |_: Event| {
                    do_fetch("https://official-joke-api.appspot.com/random_joke", result, loading);
                }}>"Random Joke"</button>
                <button on:click={move |_: Event| {
                    do_fetch("https://api.ipify.org?format=json", result, loading);
                }}>"My IP Address"</button>
                <button on:click={move |_: Event| {
                    do_fetch("https://randomuser.me/api/?results=1&noinfo", result, loading);
                }}>"Random User"</button>
            </div>
            <div class="fetch-result">{result}</div>
        </div>
    }
}

async fn fetch_text(url: &str) -> Result<String, JsValue> {
    let window = web_sys::window().unwrap();
    let resp_val = wasm_bindgen_futures::JsFuture::from(window.fetch_with_str(url)).await?;
    let resp: web_sys::Response = resp_val.dyn_into()?;
    let text = wasm_bindgen_futures::JsFuture::from(resp.text()?).await?;
    Ok(text.as_string().unwrap_or_default())
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. Mouse Tracker
// ═══════════════════════════════════════════════════════════════════════════

fn demo_mouse() -> web_sys::Element {
    let mx = signal(0i32);
    let my = signal(0i32);
    let coords_text = memo(move || format!("x: {} · y: {}", mx.get(), my.get()));

    let content = el("div", "col", &[]);
    let area = el("div", "mouse-area", &[]);
    let dot = el("div", "mouse-dot", &[]);
    append_node(&area, &dot);

    let area_ref = area.clone();
    let dot_ref = dot.clone();
    add_event_listener(&area, "mousemove", move |e| {
        let me: web_sys::MouseEvent = e.dyn_into().unwrap();
        let rect = area_ref.get_bounding_client_rect();
        let x = me.client_x() - rect.left() as i32;
        let y = me.client_y() - rect.top() as i32;
        mx.set(x);
        my.set(y);
        set_style(&dot_ref, "left", &format!("{}px", x));
        set_style(&dot_ref, "top", &format!("{}px", y));
    });
    append_node(&content, &area);

    let coords = view! { <div class="mouse-coords">{coords_text}</div> };
    append_node(&content, &coords);

    content
}

// ═══════════════════════════════════════════════════════════════════════════
// 8. Keyboard Events
// ═══════════════════════════════════════════════════════════════════════════

fn demo_keyboard() -> web_sys::Element {
    let key = signal("?".to_string());
    let code = signal(String::new());
    let modifiers = signal(String::new());

    let info_text = memo(move || {
        let c = code.get();
        let m = modifiers.get();
        if c.is_empty() {
            "Waiting for input...".to_string()
        } else {
            format!("Code: {} {}", c, if m.is_empty() { String::new() } else { format!("· Modifiers: {}", m) })
        }
    });

    let content = view! {
        <div class="col">
            <p>"Press any key…"</p>
            <div class="key-display">
                <div class="key-cap">{key}</div>
            </div>
            <div class="key-info">{info_text}</div>
        </div>
    };

    on_document_event("keydown", move |e| {
        let ke: web_sys::KeyboardEvent = e.dyn_into().unwrap();
        key.set(ke.key());
        code.set(ke.code());
        let mut mods = Vec::new();
        if ke.ctrl_key() { mods.push("Ctrl"); }
        if ke.shift_key() { mods.push("Shift"); }
        if ke.alt_key() { mods.push("Alt"); }
        if ke.meta_key() { mods.push("Meta"); }
        modifiers.set(mods.join(" + "));
    });

    content
}

// ═══════════════════════════════════════════════════════════════════════════
// 9. Canvas Drawing
// ═══════════════════════════════════════════════════════════════════════════

fn demo_canvas() -> web_sys::Element {
    let drawing = signal(false);
    let brush_color = signal("#f97316".to_string());

    let content = el("div", "col", &[]);

    let tools = el("div", "canvas-tools", &[]);
    append_node(&tools, &text_el("label", "Color:"));
    let cp = create_element("input");
    set_attribute(&cp, "type", "color");
    set_attribute(&cp, "value", "#f97316");
    let bc = brush_color;
    add_event_listener(&cp, "input", move |e| { bc.set(event_target_value(&e)); });
    append_node(&tools, &cp);
    let clear_btn = text_el("button", "Clear");
    append_node(&tools, &clear_btn);
    append_node(&content, &tools);

    // Canvas requires imperative API
    let wrap = el("div", "canvas-wrap", &[]);
    let canvas: web_sys::HtmlCanvasElement = create_element("canvas").dyn_into().unwrap();
    canvas.set_width(800);
    canvas.set_height(300);
    let ctx: web_sys::CanvasRenderingContext2d = canvas
        .get_context("2d").unwrap().unwrap()
        .dyn_into().unwrap();

    ctx.set_line_width(3.0);
    ctx.set_line_cap("round");
    ctx.set_line_join("round");

    let canvas_el: web_sys::Element = canvas.clone().into();
    let ctx2 = ctx.clone();
    let canvas3 = canvas.clone();
    add_event_listener(&clear_btn, "click", move |_| {
        ctx2.clear_rect(0.0, 0.0, canvas3.width() as f64, canvas3.height() as f64);
    });

    let ctx3 = ctx.clone();
    let bc2 = brush_color;
    let d = drawing;
    let canvas4 = canvas.clone();
    add_event_listener(&canvas_el, "mousedown", move |e| {
        d.set(true);
        let me: web_sys::MouseEvent = e.dyn_into().unwrap();
        let rect = canvas4.get_bounding_client_rect();
        let sx = (me.client_x() as f64 - rect.left()) * (canvas4.width() as f64 / rect.width());
        let sy = (me.client_y() as f64 - rect.top()) * (canvas4.height() as f64 / rect.height());
        ctx3.set_stroke_style_str(&bc2.get());
        ctx3.begin_path();
        ctx3.move_to(sx, sy);
    });

    let ctx4 = ctx.clone();
    let d2 = drawing;
    let canvas5 = canvas.clone();
    let canvas_el2: web_sys::Element = canvas.clone().into();
    add_event_listener(&canvas_el2, "mousemove", move |e| {
        if !d2.get() { return; }
        let me: web_sys::MouseEvent = e.dyn_into().unwrap();
        let rect = canvas5.get_bounding_client_rect();
        let x = (me.client_x() as f64 - rect.left()) * (canvas5.width() as f64 / rect.width());
        let y = (me.client_y() as f64 - rect.top()) * (canvas5.height() as f64 / rect.height());
        ctx4.line_to(x, y);
        ctx4.stroke();
    });

    let d3 = drawing;
    let canvas_el3: web_sys::Element = canvas.clone().into();
    add_event_listener(&canvas_el3, "mouseup", move |_| { d3.set(false); });
    let d4 = drawing;
    let canvas_el4: web_sys::Element = canvas.clone().into();
    add_event_listener(&canvas_el4, "mouseleave", move |_| { d4.set(false); });

    append_node(&wrap, &canvas_el4);
    append_node(&content, &wrap);

    content
}

// ═══════════════════════════════════════════════════════════════════════════
// Docs — comprehensive API reference rendered as a SPA route
// ═══════════════════════════════════════════════════════════════════════════

fn page_docs() -> web_sys::Element {
    let page = el("div", "pg-page", &[]);

    // Title
    append_node(&page, &text_el("h2", "API Reference"));
    let desc = el("p", "pg-desc", &[]);
    append_text(&desc, "Comprehensive reference for every API in the Oxide framework.");
    append_node(&page, &desc);

    // ── Table of contents ────────────────────────────────────────────────
    let toc = el("div", "doc-section", &[]);
    append_node(&toc, &text_el("h3", "Contents"));
    let toc_list = create_element("ul");
    let sections = [
        ("sec-quickstart",   "Quick Start"),
        ("sec-cli",          "CLI"),
        ("sec-reactivity",   "Core Reactivity"),
        ("sec-view",         "View Macro"),
        ("sec-hooks",        "Reactive Hooks"),
        ("sec-dom",          "DOM Utilities"),
        ("sec-head",         "Head Management"),
        ("sec-routing",      "Routing"),
        ("sec-otel",         "OpenTelemetry"),
        ("sec-resiliency",   "Resiliency"),
        ("sec-components",   "Components"),
        ("sec-debugging",    "Debugging"),
    ];
    for (id, label) in &sections {
        let li = create_element("li");
        let a = create_element("a");
        set_attribute(&a, "href", &format!("#{}", id));
        append_text(&a, label);
        append_node(&li, &a);
        append_node(&toc_list, &li);
    }
    append_node(&toc, &toc_list);
    append_node(&page, &toc);
    append_node(&page, &components::divider());

    // ── Quick Start ──────────────────────────────────────────────────────
    let sec = el("div", "doc-section", &[]);
    let h = text_el("h3", "Quick Start");
    set_attribute(&h, "id", "sec-quickstart");
    append_node(&sec, &h);
    append_node(&sec, &text_el("p", "Install the Oxide CLI globally with cargo:"));
    append_node(&sec, &components::code_block("cargo install oxide-cli"));
    append_node(&sec, &text_el("p", "Create a new project:"));
    append_node(&sec, &components::code_block("oxide new my-app\ncd my-app"));
    append_node(&sec, &text_el("p", "Start the dev server with hot-reload:"));
    append_node(&sec, &components::code_block("oxide dev"));
    append_node(&sec, &text_el("p", "Build for production:"));
    append_node(&sec, &components::code_block("oxide build --release"));
    append_node(&sec, &text_el("p", "Serve the optimised output locally:"));
    append_node(&sec, &components::code_block("oxide serve"));
    append_node(&page, &sec);
    append_node(&page, &components::divider());

    // ── CLI ──────────────────────────────────────────────────────────────
    let sec = el("div", "doc-section", &[]);
    let h = text_el("h3", "CLI");
    set_attribute(&h, "id", "sec-cli");
    append_node(&sec, &h);
    append_node(&sec, &text_el("p", "The oxide CLI provides project scaffolding, dev server, build, and preview commands."));
    append_node(&sec, &components::code_block(
        "oxide new <name>   # Scaffold a new Oxide project\n\
         oxide dev          # Start dev server with hot-reload\n\
         oxide build        # Compile WASM + bundle assets\n\
         oxide serve        # Preview the production build locally"
    ));
    append_node(&page, &sec);
    append_node(&page, &components::divider());

    // ── Core Reactivity ──────────────────────────────────────────────────
    let sec = el("div", "doc-section", &[]);
    let h = text_el("h3", "Core Reactivity");
    set_attribute(&h, "id", "sec-reactivity");
    append_node(&sec, &h);
    append_node(&sec, &text_el("p", "Signals are the foundation of Oxide\u{2019}s reactivity system."));
    append_node(&sec, &components::code_block(
        "// Create a signal with an initial value\n\
         let count = signal(0);\n\
         \n\
         // Read / write / update\n\
         let value = count.get();\n\
         count.set(5);\n\
         count.update(|n| *n += 1);"
    ));
    append_node(&sec, &text_el("p", "Effects re-run whenever their tracked signals change:"));
    append_node(&sec, &components::code_block(
        "create_effect(move || {\n\
         \x20   log!(\"{}\", count.get());\n\
         });"
    ));
    append_node(&sec, &text_el("p", "Memos are derived computations that cache their result:"));
    append_node(&sec, &components::code_block(
        "let doubled = memo(move || count.get() * 2);"
    ));
    append_node(&sec, &text_el("p", "Batch multiple signal writes into a single update, or untrack reads:"));
    append_node(&sec, &components::code_block(
        "batch(|| {\n\
         \x20   count.set(1);\n\
         \x20   name.set(\"Oxide\".into());\n\
         });\n\
         \n\
         let snapshot = untrack(|| count.get());"
    ));
    append_node(&sec, &text_el("p", "Lifecycle hooks and context:"));
    append_node(&sec, &components::code_block(
        "on_mount(|| log!(\"mounted\"));\n\
         on_cleanup(|| log!(\"disposed\"));\n\
         \n\
         // Dependency injection\n\
         provide_context(my_service);\n\
         let svc = use_context::<MyService>();"
    ));
    append_node(&page, &sec);
    append_node(&page, &components::divider());

    // ── View Macro ───────────────────────────────────────────────────────
    let sec = el("div", "doc-section", &[]);
    let h = text_el("h3", "View Macro");
    set_attribute(&h, "id", "sec-view");
    append_node(&sec, &h);
    append_node(&sec, &text_el("p", "The view! macro provides declarative HTML-like syntax:"));
    append_node(&sec, &components::code_block(
        "view! {\n\
         \x20   <div class=\"card\">\n\
         \x20       <h1>{title}</h1>\n\
         \x20       <p>{move || description.get()}</p>\n\
         \x20   </div>\n\
         }"
    ));
    append_node(&sec, &text_el("p", "Dynamic attributes, events, and two-way binding:"));
    append_node(&sec, &components::code_block(
        "view! {\n\
         \x20   <input\n\
         \x20       class:active={move || is_active.get()}\n\
         \x20       bind:value={name}\n\
         \x20       on:input={move |e| handle(e)}\n\
         \x20   />\n\
         \x20   <input type=\"checkbox\" bind:checked={agreed} />\n\
         }"
    ));
    append_node(&sec, &text_el("p", "Control flow — conditionals and loops:"));
    append_node(&sec, &components::code_block(
        "view! {\n\
         \x20   {move || if show.get() {\n\
         \x20       view! { <p>\"Visible\"</p> }\n\
         \x20   } else {\n\
         \x20       view! { <p>\"Hidden\"</p> }\n\
         \x20   }}\n\
         \x20   <ul>\n\
         \x20       {move || items.get().iter().map(|i|\n\
         \x20           view! { <li>{i}</li> }\n\
         \x20       ).collect::<Vec<_>>()}\n\
         \x20   </ul>\n\
         }"
    ));
    append_node(&page, &sec);
    append_node(&page, &components::divider());

    // ── Reactive Hooks ───────────────────────────────────────────────────
    let sec = el("div", "doc-section", &[]);
    let h = text_el("h3", "Reactive Hooks");
    set_attribute(&h, "id", "sec-hooks");
    append_node(&sec, &h);
    append_node(&sec, &text_el("p", "Ready-made hooks for common browser APIs:"));
    append_node(&sec, &components::code_block(
        "let stored = use_local_storage(\"key\", \"default\");\n\
         use_interval(move || tick(), 1000);\n\
         let debounced = use_debounce(move || search(), 300);\n\
         let throttled = use_throttle(move || scroll(), 100);"
    ));
    append_node(&sec, &text_el("p", "Window, pointer, and media hooks:"));
    append_node(&sec, &components::code_block(
        "let (w, h) = use_window_size();\n\
         let (sx, sy) = use_scroll();\n\
         let (mx, my) = use_mouse();\n\
         let is_dark  = use_preferred_dark();\n\
         let online   = use_online();\n\
         let mobile   = use_media_query(\"(max-width: 768px)\");"
    ));
    append_node(&sec, &text_el("p", "Focus and click-outside:"));
    append_node(&sec, &components::code_block(
        "use_click_outside(&menu_ref, move || close_menu());\n\
         use_focus(&input_ref);"
    ));
    append_node(&page, &sec);
    append_node(&page, &components::divider());

    // ── DOM Utilities ────────────────────────────────────────────────────
    let sec = el("div", "doc-section", &[]);
    let h = text_el("h3", "DOM Utilities");
    set_attribute(&h, "id", "sec-dom");
    append_node(&sec, &h);
    append_node(&sec, &text_el("p", "Low-level helpers for imperative DOM manipulation:"));
    append_node(&sec, &components::code_block(
        "let el = create_element(\"div\");\n\
         set_attribute(&el, \"class\", \"card\");\n\
         set_property(&el, \"textContent\", \"hello\");\n\
         set_style(&el, \"color\", \"red\");\n\
         toggle_class(&el, \"active\", true);"
    ));
    append_node(&sec, &text_el("p", "Tree operations and event listeners:"));
    append_node(&sec, &components::code_block(
        "append_node(&parent, &child);\n\
         clear_children(&container);\n\
         let found = query_selector(\".my-class\");\n\
         add_event_listener(&btn, \"click\", move |e| { ... });\n\
         on_window_event(\"resize\", move |_| { ... });"
    ));
    append_node(&page, &sec);
    append_node(&page, &components::divider());

    // ── Head Management ──────────────────────────────────────────────────
    let sec = el("div", "doc-section", &[]);
    let h = text_el("h3", "Head Management");
    set_attribute(&h, "id", "sec-head");
    append_node(&sec, &h);
    append_node(&sec, &text_el("p", "Dynamically update the document title and meta tags:"));
    append_node(&sec, &components::code_block(
        "set_title(\"My Page | Oxide\");\n\
         set_meta(\"description\", \"Built with Oxide.\");"
    ));
    append_node(&page, &sec);
    append_node(&page, &components::divider());

    // ── Routing ──────────────────────────────────────────────────────────
    let sec = el("div", "doc-section", &[]);
    let h = text_el("h3", "Routing");
    set_attribute(&h, "id", "sec-routing");
    append_node(&sec, &h);
    append_node(&sec, &text_el("p", "Hash-based or history-based SPA routing:"));
    append_node(&sec, &components::code_block(
        "let router = Router::new(RouterMode::Hash, &[\n\
         \x20   route(\"/\",         page_home),\n\
         \x20   route(\"/about\",    page_about),\n\
         \x20   route(\"/user/:id\", page_user),\n\
         ]);"
    ));
    append_node(&sec, &text_el("p", "Programmatic navigation and reading params:"));
    append_node(&sec, &components::code_block(
        "navigate(\"/about\");\n\
         link(\"/about\", \"Go to About\");\n\
         \n\
         let current = use_route();\n\
         let id = use_param(\"id\");"
    ));
    append_node(&page, &sec);
    append_node(&page, &components::divider());

    // ── OpenTelemetry ────────────────────────────────────────────────────
    let sec = el("div", "doc-section", &[]);
    let h = text_el("h3", "OpenTelemetry");
    set_attribute(&h, "id", "sec-otel");
    append_node(&sec, &h);
    append_node(&sec, &text_el("p", "Built-in tracing and telemetry for WASM apps:"));
    append_node(&sec, &components::code_block(
        "telemetry::init(\"my-app\");\n\
         \n\
         let _span = telemetry::span(\"fetch-users\");\n\
         let resp = telemetry::traced_fetch(\"/api/users\").await;"
    ));
    append_node(&sec, &text_el("p", "Inspect collected data:"));
    append_node(&sec, &components::code_block(
        "let spans = telemetry::get_spans();\n\
         let stats = telemetry::get_stats();"
    ));
    append_node(&page, &sec);
    append_node(&page, &components::divider());

    // ── Resiliency ───────────────────────────────────────────────────────
    let sec = el("div", "doc-section", &[]);
    let h = text_el("h3", "Resiliency");
    set_attribute(&h, "id", "sec-resiliency");
    append_node(&sec, &h);
    append_node(&sec, &text_el("p", "Error boundaries, retries, circuit breakers, and timeouts:"));
    append_node(&sec, &components::code_block(
        "let view = resiliency::error_boundary(|| {\n\
         \x20   risky_component()\n\
         });"
    ));
    append_node(&sec, &components::code_block(
        "let result = resiliency::retry(3, || async {\n\
         \x20   fetch(\"/api/data\").await\n\
         }).await;"
    ));
    append_node(&sec, &components::code_block(
        "let cb = resiliency::CircuitBreaker::new(5, 30_000);\n\
         let val = cb.call(|| fetch_data()).await;\n\
         \n\
         let resp = resiliency::with_timeout(5000, fetch_data()).await;\n\
         resiliency::sleep(1000).await;"
    ));
    append_node(&page, &sec);
    append_node(&page, &components::divider());

    // ── Components ───────────────────────────────────────────────────────
    let sec = el("div", "doc-section", &[]);
    let h = text_el("h3", "Components");
    set_attribute(&h, "id", "sec-components");
    append_node(&sec, &h);
    append_node(&sec, &text_el("p", "Oxide ships 48 ready-made components. Explore each in the Components section."));
    let comp_list = [
        "button \u{2014} Primary, secondary, outline, ghost, danger, and icon buttons",
        "input \u{2014} Text input with label, placeholder, and validation",
        "textarea \u{2014} Multi-line text input with auto-resize",
        "select \u{2014} Dropdown select with option groups",
        "checkbox \u{2014} Checkbox with indeterminate state",
        "radio \u{2014} Radio button group",
        "toggle \u{2014} On/off toggle switch",
        "slider \u{2014} Range slider with min/max/step",
        "search \u{2014} Search input with clear button",
        "password \u{2014} Password input with visibility toggle",
        "number \u{2014} Numeric input with increment/decrement",
        "file-upload \u{2014} Drag-and-drop file upload zone",
        "form-group \u{2014} Form field wrapper with label and error",
        "card \u{2014} Content container with header, body, and footer",
        "alert \u{2014} Dismissible alert banners (info, success, warning, error)",
        "modal \u{2014} Accessible dialog overlay",
        "drawer \u{2014} Slide-in panel from any edge",
        "toast \u{2014} Temporary notification popups",
        "dropdown \u{2014} Trigger + floating menu",
        "tooltip \u{2014} Hover/focus tooltip",
        "tabs \u{2014} Tabbed content panels",
        "accordion \u{2014} Collapsible content sections",
        "badge \u{2014} Status badge with severity colors",
        "tag \u{2014} Removable tag / chip",
        "avatar \u{2014} User avatar with fallback initials",
        "spinner \u{2014} Loading spinner animation",
        "progress \u{2014} Determinate progress bar",
        "skeleton \u{2014} Placeholder loading skeleton",
        "loading-overlay \u{2014} Full-area loading mask",
        "empty-state \u{2014} Placeholder for empty data",
        "divider \u{2014} Horizontal rule separator",
        "breadcrumb \u{2014} Navigation breadcrumb trail",
        "pagination \u{2014} Page navigation controls",
        "table \u{2014} Data table with sorting and selection",
        "timeline \u{2014} Vertical event timeline",
        "stat \u{2014} Statistic display with label and trend",
        "rating \u{2014} Star rating input",
        "copy-button \u{2014} One-click copy to clipboard",
        "kbd \u{2014} Keyboard shortcut display",
        "code-block \u{2014} Syntax-highlighted code display",
        "layout \u{2014} Responsive grid and flex layout helpers",
        "scroll-to-top \u{2014} Floating scroll-to-top button",
        "color-picker \u{2014} Interactive color chooser",
        "date-picker \u{2014} Calendar date selection",
        "stepper \u{2014} Multi-step wizard indicator",
        "tree-view \u{2014} Hierarchical tree display",
        "chart \u{2014} Simple SVG chart components",
        "command-palette \u{2014} Keyboard-driven command launcher",
    ];
    let list = create_element("ul");
    for desc in &comp_list {
        let li = create_element("li");
        append_text(&li, desc);
        append_node(&list, &li);
    }
    append_node(&sec, &list);
    append_node(&page, &sec);
    append_node(&page, &components::divider());

    // ── Debugging ────────────────────────────────────────────────────────
    let sec = el("div", "doc-section", &[]);
    let h = text_el("h3", "Debugging");
    set_attribute(&h, "id", "sec-debugging");
    append_node(&sec, &h);
    append_node(&sec, &text_el("p", "Use oxide dev for DWARF debug info and source maps:"));
    append_node(&sec, &components::code_block(
        "oxide dev   # builds with debug symbols"
    ));
    append_node(&sec, &text_el("p", "Install the C/C++ DWARF Chrome extension to set breakpoints directly in Rust source from DevTools."));
    append_node(&sec, &components::code_block(
        "// In Chrome DevTools → Sources → your .rs files\n\
         // Set breakpoints, inspect locals, step through Rust code"
    ));
    append_node(&page, &sec);

    page
}

// ═══════════════════════════════════════════════════════════════════════════
// 10. Theme Toggle
// ═══════════════════════════════════════════════════════════════════════════

fn demo_theme() -> web_sys::Element {
    let dark = signal(true);

    view! {
        <div class="col">
            <button class="btn-primary"
                on:click={move |_: Event| { dark.set(!dark.get()); }}>
                "Toggle Theme"
            </button>
            <div class="theme-preview" class:dark={dark.get()} class:light={!dark.get()}>
                {if dark.get() {
                    <h3>"🌙 Dark Mode"</h3>
                    <p>"Easy on the eyes for late-night coding."</p>
                } else {
                    <h3>"☀️ Light Mode"</h3>
                    <p>"Bright and clean for daytime work."</p>
                }}
            </div>
        </div>
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 11. Persistent Notes
// ═══════════════════════════════════════════════════════════════════════════

fn demo_notes() -> web_sys::Element {
    let saved = local_storage_get("oxide-notes")
        .unwrap_or_else(|| "Write your notes here...\n\nThey persist across page reloads via localStorage!".into());
    let text = signal(saved);
    let status = signal("Saved ✓".to_string());

    // Persist on every change
    let t = text;
    let st = status;

    let content = el("div", "col", &[]);
    let ta = create_element("textarea");
    set_attribute(&ta, "class", "notes-area");
    set_attribute(&ta, "rows", "6");
    let ta_ref = ta.clone();
    create_effect(move || {
        set_property(&ta_ref, "value", &JsValue::from_str(&text.get()));
    });
    add_event_listener(&ta, "input", move |e| {
        let v = event_target_value(&e);
        t.set(v.clone());
        local_storage_set("oxide-notes", &v);
        st.set("Saved ✓".into());
    });
    append_node(&content, &ta);

    let stat = view! { <div class="notes-status">{status}</div> };
    append_node(&content, &stat);

    content
}

// ═══════════════════════════════════════════════════════════════════════════
// 12. Bouncing Ball Animation
// ═══════════════════════════════════════════════════════════════════════════

fn demo_animation() -> web_sys::Element {
    let running = signal(true);
    let x = signal(50.0f64);
    let y = signal(50.0f64);
    let dx = signal(2.5f64);
    let dy = signal(2.0f64);
    let btn_label = memo(move || {
        if running.get() { "Pause".to_string() } else { "Resume".to_string() }
    });

    let content = el("div", "col", &[]);
    let stage = el("div", "anim-stage", &[]);
    let ball = el("div", "anim-ball", &[]);
    append_node(&stage, &ball);
    append_node(&content, &stage);

    let ball_ref = ball.clone();
    let stage_ref = stage.clone();

    fn tick(
        x: Signal<f64>, y: Signal<f64>,
        dx: Signal<f64>, dy: Signal<f64>,
        running: Signal<bool>,
        ball: web_sys::Element, stage: web_sys::Element,
    ) {
        if !running.get() {
            request_animation_frame(move || tick(x, y, dx, dy, running, ball, stage));
            return;
        }
        let w = stage.client_width() as f64 - 30.0;
        let h = stage.client_height() as f64 - 30.0;
        let mut nx = x.get() + dx.get();
        let mut ny = y.get() + dy.get();
        if nx <= 0.0 || nx >= w { dx.set(-dx.get()); nx = nx.clamp(0.0, w); }
        if ny <= 0.0 || ny >= h { dy.set(-dy.get()); ny = ny.clamp(0.0, h); }
        x.set(nx);
        y.set(ny);
        set_style(&ball, "left", &format!("{}px", nx));
        set_style(&ball, "top", &format!("{}px", ny));
        request_animation_frame(move || tick(x, y, dx, dy, running, ball, stage));
    }

    request_animation_frame(move || tick(x, y, dx, dy, running, ball_ref, stage_ref));

    let toggle_btn = view! {
        <button on:click={move |_: Event| { running.set(!running.get()); }}>
            {btn_label}
        </button>
    };
    append_node(&content, &toggle_btn);

    content
}

// ═══════════════════════════════════════════════════════════════════════════
// 13. SVG Bar Chart
// ═══════════════════════════════════════════════════════════════════════════

fn demo_chart() -> web_sys::Element {
    let data = signal(vec![65u32, 40, 80, 55, 90, 35, 70]);
    let labels = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

    let content = el("div", "col", &[]);
    let chart_wrap = el("div", "chart-wrap", &[]);
    // SVG requires createElementNS — imperative is the right approach here
    let svg = create_svg_element("svg");
    set_attribute(&svg, "viewBox", "0 0 700 220");
    set_attribute(&svg, "preserveAspectRatio", "xMidYMid meet");

    let svg_ref = svg.clone();
    let d = data;
    create_effect(move || {
        clear_children(&svg_ref);
        let vals = d.get();
        let bar_w = 60.0f64;
        let gap = 40.0f64;
        let max_h = 180.0f64;
        let max_val = *vals.iter().max().unwrap_or(&100) as f64;

        for (i, &val) in vals.iter().enumerate() {
            let x = 20.0 + i as f64 * (bar_w + gap);
            let h = (val as f64 / max_val) * max_h;
            let y = 190.0 - h;

            let rect = create_svg_element("rect");
            set_attribute(&rect, "x", &format!("{}", x));
            set_attribute(&rect, "y", &format!("{}", y));
            set_attribute(&rect, "width", &format!("{}", bar_w));
            set_attribute(&rect, "height", &format!("{}", h));
            set_attribute(&rect, "rx", "4");
            set_attribute(&rect, "fill", &format!("hsl({}, 80%, 55%)", 20 + i * 40));
            set_attribute(&rect, "class", "chart-bar");
            append_node(&svg_ref, &rect);

            let label = create_svg_element("text");
            set_attribute(&label, "x", &format!("{}", x + bar_w / 2.0));
            set_attribute(&label, "y", "210");
            set_attribute(&label, "text-anchor", "middle");
            set_attribute(&label, "class", "chart-label");
            append_text(&label, labels[i]);
            append_node(&svg_ref, &label);

            let value = create_svg_element("text");
            set_attribute(&value, "x", &format!("{}", x + bar_w / 2.0));
            set_attribute(&value, "y", &format!("{}", y - 5.0));
            set_attribute(&value, "text-anchor", "middle");
            set_attribute(&value, "class", "chart-value");
            append_text(&value, &format!("{}", val));
            append_node(&svg_ref, &value);
        }
    });

    append_node(&chart_wrap, &svg);
    append_node(&content, &chart_wrap);

    let rand_btn = view! {
        <div class="chart-btns">
            <button on:click={move |_: Event| {
                d.update(|v| {
                    for val in v.iter_mut() { *val = pseudo_random(*val); }
                });
            }}>"Randomize"</button>
        </div>
    };
    append_node(&content, &rand_btn);

    content
}

fn pseudo_random(seed: u32) -> u32 {
    let x = seed.wrapping_mul(1103515245).wrapping_add(12345);
    (x >> 16) % 100 + 10
}

// ═══════════════════════════════════════════════════════════════════════════
// 14. Modal Dialog
// ═══════════════════════════════════════════════════════════════════════════

fn demo_modal() -> web_sys::Element {
    let open = signal(false);

    view! {
        <div class="col">
            <button class="btn-primary"
                on:click={move |_: Event| { open.set(true); }}>
                "Open Modal"
            </button>
            <div class="overlay" class:hidden={!open.get()}
                on:click={move |e: Event| {
                    let target = e.target().unwrap();
                    let el: web_sys::Element = target.dyn_into().unwrap();
                    if el.class_list().contains("overlay") { open.set(false); }
                }}>
                <div class="modal">
                    <h3>"🔥 Oxide Modal"</h3>
                    <p>"This modal is rendered and controlled entirely by Rust signals compiled to WASM. No JavaScript!"</p>
                    <button on:click={move |_: Event| { open.set(false); }}>"Close"</button>
                </div>
            </div>
        </div>
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 15. Drag & Drop
// ═══════════════════════════════════════════════════════════════════════════

fn demo_dnd() -> web_sys::Element {
    // Drag & Drop uses DataTransfer API — needs imperative event handling
    let content = el("div", "col", &[]);
    let container = el("div", "dnd-container", &[]);

    let pool = el("div", "dnd-pool", &[]);
    let pool_label = el("div", "dnd-label", &[]);
    append_text(&pool_label, "DRAG FROM HERE");
    append_node(&pool, &pool_label);
    for item in &["Rust 🦀", "WASM ⚡", "Oxide 🔥", "Signals 📡", "Macros 🏗️"] {
        let chip = el("div", "dnd-chip", &[]);
        set_attribute(&chip, "draggable", "true");
        append_text(&chip, item);
        let text = item.to_string();
        add_event_listener(&chip, "dragstart", move |e| {
            let de: web_sys::DragEvent = e.dyn_into().unwrap();
            if let Some(dt) = de.data_transfer() {
                dt.set_data("text/plain", &text).ok();
            }
        });
        append_node(&pool, &chip);
    }

    let drop_zone = el("div", "dnd-drop", &[]);
    let drop_label = el("div", "dnd-label", &[]);
    append_text(&drop_label, "DROP HERE");
    append_node(&drop_zone, &drop_label);

    let dz = drop_zone.clone();
    add_event_listener(&drop_zone, "dragover", move |e| {
        e.prevent_default();
        dz.class_list().add_1("over").ok();
    });
    let dz2 = drop_zone.clone();
    add_event_listener(&drop_zone, "dragleave", move |_| {
        dz2.class_list().remove_1("over").ok();
    });
    let dz3 = drop_zone.clone();
    add_event_listener(&drop_zone, "drop", move |e| {
        e.prevent_default();
        dz3.class_list().remove_1("over").ok();
        let de: web_sys::DragEvent = e.dyn_into().unwrap();
        if let Some(dt) = de.data_transfer() {
            if let Ok(text) = dt.get_data("text/plain") {
                let chip = el("div", "dnd-chip", &[]);
                append_text(&chip, &text);
                append_node(&dz3, &chip);
            }
        }
    });

    append_node(&container, &pool);
    append_node(&container, &drop_zone);
    append_node(&content, &container);

    content
}

// ═══════════════════════════════════════════════════════════════════════════
// 16. Clipboard
// ═══════════════════════════════════════════════════════════════════════════

fn demo_clipboard() -> web_sys::Element {
    let copied = signal(false);

    view! {
        <div class="col">
            <div class="row">
                <div class="clip-text">"🔥 Oxide — Rust frontend framework compiling to WASM"</div>
                <button on:click={move |_: Event| {
                    let window = web_sys::window().unwrap();
                    let nav = window.navigator();
                    let clipboard = nav.clipboard();
                    let promise = clipboard.write_text("🔥 Oxide — Rust frontend framework compiling to WASM");
                    wasm_bindgen_futures::spawn_local(async move {
                        let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
                        copied.set(true);
                        set_timeout(move || { copied.set(false); }, 2000);
                    });
                }}>"📋 Copy"</button>
                <span class="clip-toast" class:show={copied.get()}>"Copied!"</span>
            </div>
        </div>
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 17. Telemetry
// ═══════════════════════════════════════════════════════════════════════════

fn demo_telemetry() -> web_sys::Element {
    telemetry::init(telemetry::Config::default());

    let count = signal(0i32);
    let span_text = signal(String::from("(waiting for spans…)"));
    let stats_text = signal(String::from("—"));

    // Refresh display every 500ms
    let st = span_text;
    let stx = stats_text;
    set_interval(move || {
        let spans = telemetry::get_spans();
        let last10: Vec<String> = spans.iter().rev().take(10).map(|s| {
            let attrs: String = s.attributes.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join(" ");
            format!("[{}] {:.2}ms {}", s.name, s.duration_ms, attrs)
        }).collect();
        st.set(if last10.is_empty() {
            "(no spans yet)".into()
        } else {
            last10.join("\n")
        });

        let stats = telemetry::get_stats();
        stx.set(format!(
            "signals: {} · reads: {} · writes: {} · effects: {} · effect_time: {:.1}ms",
            stats.signals_created, stats.signal_reads, stats.signal_writes,
            stats.effects_run, stats.total_effect_time_ms
        ));
    }, 500);

    let content = el("div", "col", &[]);

    // Increment button
    let btn = view! {
        <button class="btn-primary" on:click={move |_: Event| { count.update(|c| *c += 1); }}>
            "Increment"
        </button>
    };
    append_node(&content, &btn);

    let counter_display = view! { <div class="mono">"Count: " {count}</div> };
    append_node(&content, &counter_display);

    // Stats line
    let stats_el = el("div", "mono", &[]);
    let stats_ref = stats_el.clone();
    create_effect(move || {
        stats_ref.set_inner_html(&stats_text.get());
    });
    append_node(&content, &stats_el);

    // Span log panel
    let panel = create_element("pre");
    set_style(&panel, "background", "rgba(0,0,0,0.4)");
    set_style(&panel, "padding", "0.5rem");
    set_style(&panel, "border-radius", "6px");
    set_style(&panel, "font-family", "monospace");
    set_style(&panel, "font-size", "0.7rem");
    set_style(&panel, "max-height", "150px");
    set_style(&panel, "overflow-y", "auto");
    set_style(&panel, "white-space", "pre-wrap");
    set_style(&panel, "color", "#a5f3fc");
    let panel_ref = panel.clone();
    create_effect(move || {
        panel_ref.set_inner_html(&span_text.get());
    });
    append_node(&content, &panel);

    // Clear button
    let clear_btn = view! {
        <button on:click={move |_: Event| { telemetry::clear_spans(); }}>
            "Clear Spans"
        </button>
    };
    append_node(&content, &clear_btn);

    content
}

// ═══════════════════════════════════════════════════════════════════════════
// 18. Resiliency
// ═══════════════════════════════════════════════════════════════════════════

fn demo_resiliency() -> web_sys::Element {
    use std::rc::Rc;
    let content = el("div", "col", &[]);

    // ── Error Boundary section ──
    let eb_label = text_el("strong", "Error Boundary");
    append_node(&content, &eb_label);

    let error_area = el("div", "", &[]);
    let error_area_ref = error_area.clone();
    let trigger_btn = view! {
        <button on:click={move |_: Event| {
            clear_children(&error_area_ref);
            let card = resiliency::default_error_boundary(|| {
                panic!("Simulated component crash!")
            });
            error_area_ref.append_child(&card).ok();
        }}>"Trigger Error"</button>
    };
    append_node(&content, &trigger_btn);
    append_node(&content, &error_area);

    // ── Circuit Breaker section ──
    let cb_label = text_el("strong", "Circuit Breaker");
    set_style(&cb_label, "margin-top", "0.75rem");
    append_node(&content, &cb_label);

    let breaker = Rc::new(resiliency::CircuitBreaker::new(resiliency::CircuitBreakerConfig {
        failure_threshold: 3,
        reset_timeout_ms: 5000,
    }));

    let cb_state = breaker.state;
    let cb_fail = breaker.failure_count;
    let cb_succ = breaker.success_count;

    let state_text = memo(move || {
        format!(
            "State: {} · Failures: {} · Successes: {}",
            cb_state.get(), cb_fail.get(), cb_succ.get()
        )
    });

    let info_el = view! { <div class="mono">{state_text}</div> };
    append_node(&content, &info_el);

    let btn_row = el("div", "row", &[]);

    // "Call (will fail)" button
    let b1 = breaker.clone();
    let fail_btn = view! {
        <button on:click={move |_: Event| {
            let b = b1.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let _ = b.call(|| async {
                    Err::<(), &str>("simulated failure")
                }).await;
            });
        }}>"Call (fail)"</button>
    };
    append_node(&btn_row, &fail_btn);

    // "Call (will succeed)" button
    let b2 = breaker.clone();
    let succ_btn = view! {
        <button on:click={move |_: Event| {
            let b = b2.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let _ = b.call(|| async { Ok::<&str, &str>("ok") }).await;
            });
        }}>"Call (ok)"</button>
    };
    append_node(&btn_row, &succ_btn);

    // "Reset" button
    let b3 = breaker.clone();
    let reset_btn = view! {
        <button on:click={move |_: Event| {
            b3.reset();
        }}>"Reset"</button>
    };
    append_node(&btn_row, &reset_btn);
    append_node(&content, &btn_row);

    // ── Retry section ──
    let retry_label = text_el("strong", "Retry (Exponential Backoff)");
    set_style(&retry_label, "margin-top", "0.75rem");
    append_node(&content, &retry_label);

    let retry_status = signal("Press button to start".to_string());
    let retry_el = el("div", "mono", &[]);
    let retry_el_ref = retry_el.clone();
    create_effect(move || {
        retry_el_ref.set_inner_html(&retry_status.get());
    });
    append_node(&content, &retry_el);

    let attempt_count = signal(0u32);
    let max_attempts: u32 = 4;
    let retry_btn = view! {
        <button on:click={move |_: Event| {
            retry_status.set("Retrying…".into());
            attempt_count.set(0);
            let rs = retry_status;
            let ac = attempt_count;
            let ma = max_attempts;
            wasm_bindgen_futures::spawn_local(async move {
                let config = resiliency::RetryConfig::exponential(ma, 300);
                let result = resiliency::retry(config, || {
                    ac.update(|c| *c += 1);
                    rs.set(format!("Attempt {}…", ac.get()));
                    async { Err::<(), &str>("simulated network error") }
                }).await;
                match result {
                    Ok(_) => rs.set("Success!".into()),
                    Err(e) => rs.set(format!("Failed after {} attempts: {}", ac.get(), e)),
                }
            });
        }}>"Retry Fetch"</button>
    };
    append_node(&content, &retry_btn);

    content
}
