use oxide::prelude::*;
use oxide::dom::*;
use oxide::{Signal, memo};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

// ═══════════════════════════════════════════════════════════════════════════
// Entry point
// ═══════════════════════════════════════════════════════════════════════════

#[wasm_bindgen(start)]
pub fn main() {
    mount("#app", build_app);
}

fn build_app() -> web_sys::Element {
    let shell = el("div", "shell", &[]);
    let sidebar = build_sidebar();
    let main = el("div", "main", &[]);
    main.set_attribute("id", "main-content").ok();

    append_node(&shell, &sidebar);
    append_node(&shell, &main);

    let sections: &[(&str, fn() -> web_sys::Element)] = &[
        ("counter",     demo_counter),
        ("temperature", demo_temperature),
        ("todo",        demo_todo),
        ("stopwatch",   demo_stopwatch),
        ("forms",       demo_forms),
        ("fetch",       demo_fetch),
        ("mouse",       demo_mouse),
        ("keyboard",    demo_keyboard),
        ("canvas",      demo_canvas),
        ("theme",       demo_theme),
        ("notes",       demo_notes),
        ("animation",   demo_animation),
        ("chart",       demo_chart),
        ("modal",       demo_modal),
        ("dnd",         demo_dnd),
        ("clipboard",   demo_clipboard),
    ];

    for &(id, builder) in sections {
        let section = builder();
        section.set_attribute("id", &format!("section-{}", id)).ok();
        append_node(&main, &section);
    }

    shell
}

// ═══════════════════════════════════════════════════════════════════════════
// Sidebar
// ═══════════════════════════════════════════════════════════════════════════

fn build_sidebar() -> web_sys::Element {
    let sidebar = el("nav", "sidebar", &[]);

    let title = el("div", "sidebar-title", &[]);
    title.set_inner_html("🔥 Oxide<span>Rust → WebAssembly</span>");
    append_node(&sidebar, &title);

    let items: &[(&str, &str, &str)] = &[
        ("Reactivity", "", ""),
        ("counter",     "Counter",              "#section-counter"),
        ("temperature", "Temperature",          "#section-temperature"),
        ("todo",        "Todo List",            "#section-todo"),
        ("stopwatch",   "Stopwatch",            "#section-stopwatch"),
        ("Web APIs", "", ""),
        ("forms",       "Form Inputs",          "#section-forms"),
        ("fetch",       "Fetch API",            "#section-fetch"),
        ("mouse",       "Mouse Tracker",        "#section-mouse"),
        ("keyboard",    "Keyboard Events",      "#section-keyboard"),
        ("canvas",      "Canvas Drawing",       "#section-canvas"),
        ("Features", "", ""),
        ("theme",       "Theme Toggle",         "#section-theme"),
        ("notes",       "Persistent Notes",     "#section-notes"),
        ("animation",   "Animation",            "#section-animation"),
        ("chart",       "SVG Chart",            "#section-chart"),
        ("modal",       "Modal Dialog",         "#section-modal"),
        ("dnd",         "Drag & Drop",          "#section-dnd"),
        ("clipboard",   "Clipboard",            "#section-clipboard"),
    ];

    let mut group: Option<web_sys::Element> = None;

    for &(id, label, href) in items {
        if href.is_empty() {
            // Group header
            let g = el("div", "nav-group", &[]);
            let lbl = el("div", "nav-group-label", &[]);
            append_text(&lbl, id);
            append_node(&g, &lbl);
            if let Some(prev) = group.take() {
                append_node(&sidebar, &prev);
            }
            group = Some(g);
        } else {
            let a = create_element("a");
            set_attribute(&a, "class", "nav-link");
            set_attribute(&a, "href", href);
            append_text(&a, label);
            if let Some(ref g) = group {
                append_node(g, &a);
            }
        }
    }
    if let Some(g) = group {
        append_node(&sidebar, &g);
    }

    sidebar
}

// ═══════════════════════════════════════════════════════════════════════════
// Helpers
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

fn section(title: &str, icon: &str, desc: &str, tags: &[(&str, &str)], content: web_sys::Element) -> web_sys::Element {
    let s = el("div", "section", &[]);

    let h2 = create_element("h2");
    let ic = el("span", "icon", &[]);
    ic.set_inner_html(icon);
    append_node(&h2, &ic);
    append_text(&h2, title);
    append_node(&s, &h2);

    let d = el("p", "desc", &[]);
    append_text(&d, desc);
    append_node(&s, &d);

    if !tags.is_empty() {
        let t = el("div", "apis", &[]);
        for &(label, kind) in tags {
            let tag = el("span", &format!("tag {}", kind), &[]);
            append_text(&tag, label);
            append_node(&t, &tag);
        }
        append_node(&s, &t);
    }

    let card = el("div", "card", &[]);
    append_node(&card, &content);
    append_node(&s, &card);
    s
}

fn text_el(tag: &str, text: &str) -> web_sys::Element {
    let e = create_element(tag);
    append_text(&e, text);
    e
}

fn reactive_text(parent: &web_sys::Element, s: Signal<String>) {
    let txt = create_text_node("");
    let tc = txt.clone();
    create_effect(move || {
        tc.set_text_content(Some(&s.get()));
    });
    parent.append_child(&txt).ok();
}

// ═══════════════════════════════════════════════════════════════════════════
// 1. Counter
// ═══════════════════════════════════════════════════════════════════════════

fn demo_counter() -> web_sys::Element {
    let mut count = signal(0i32);
    let display = memo(move || format!("{}", count.get()));

    let content = el("div", "col", &[]);
    let num = el("div", "big-num", &[]);
    reactive_text(&num, display);
    append_node(&content, &num);

    let btns = el("div", "row counter-btns", &[]);
    let dec = text_el("button", "−");
    add_event_listener(&dec, "click", move |_| { count -= 1; });
    let reset = text_el("button", "Reset");
    let c = count;
    add_event_listener(&reset, "click", move |_| { c.set(0); });
    let inc = text_el("button", "+");
    add_event_listener(&inc, "click", move |_| { count += 1; });
    append_node(&btns, &dec);
    append_node(&btns, &reset);
    append_node(&btns, &inc);
    append_node(&content, &btns);

    section("Counter", "🔢", "Fine-grained signals with automatic dependency tracking.",
        &[("Signal", "signal"), ("Effect", "signal"), ("Events", "dom")], content)
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. Temperature Converter
// ═══════════════════════════════════════════════════════════════════════════

fn demo_temperature() -> web_sys::Element {
    let celsius = signal("0".to_string());
    let fahrenheit = signal("32".to_string());

    let content = el("div", "col", &[]);

    // Celsius row
    let r1 = el("div", "temp-row", &[]);
    append_node(&r1, &text_el("label", "Celsius"));
    let c_input = create_element("input");
    set_attribute(&c_input, "type", "number");
    set_attribute(&c_input, "value", "0");
    let f_sig = fahrenheit;
    let c_sig = celsius;
    let c_inp = c_input.clone();
    add_event_listener(&c_input, "input", move |e| {
        let v = event_target_value(&e);
        c_sig.set(v.clone());
        if let Ok(c) = v.parse::<f64>() {
            let f = c * 9.0 / 5.0 + 32.0;
            f_sig.set(format!("{:.1}", f));
        }
    });
    append_node(&r1, &c_input);
    append_node(&content, &r1);

    // Fahrenheit row
    let r2 = el("div", "temp-row", &[]);
    append_node(&r2, &text_el("label", "Fahrenheit"));
    let f_input = create_element("input");
    set_attribute(&f_input, "type", "number");
    set_attribute(&f_input, "value", "32");
    let f_inp = f_input.clone();
    let c_sig2 = celsius;
    let f_sig2 = fahrenheit;
    add_event_listener(&f_input, "input", move |e| {
        let v = event_target_value(&e);
        f_sig2.set(v.clone());
        if let Ok(f) = v.parse::<f64>() {
            let c = (f - 32.0) * 5.0 / 9.0;
            c_sig2.set(format!("{:.1}", c));
        }
    });
    append_node(&r2, &f_input);
    append_node(&content, &r2);

    // Sync inputs from signals
    let c_inp2 = c_inp.clone();
    create_effect(move || {
        set_property(&c_inp2, "value", &JsValue::from_str(&celsius.get()));
    });
    let f_inp2 = f_inp.clone();
    create_effect(move || {
        set_property(&f_inp2, "value", &JsValue::from_str(&fahrenheit.get()));
    });

    section("Temperature Converter", "🌡️", "Bidirectional data flow between two signals.",
        &[("Signal", "signal"), ("memo", "signal"), ("Input", "dom")], content)
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

    // Input row
    let input_row = el("div", "todo-input-row", &[]);
    let input = create_element("input");
    set_attribute(&input, "type", "text");
    set_attribute(&input, "placeholder", "What needs to be done?");
    let iv = input_val;
    add_event_listener(&input, "input", move |e| { iv.set(event_target_value(&e)); });
    let input_ref = input.clone();
    let t = todos;
    let iv2 = input_val;
    add_event_listener(&input, "keydown", move |e| {
        let ke: web_sys::KeyboardEvent = e.dyn_into().unwrap();
        if ke.key() == "Enter" {
            let v = iv2.get();
            if !v.trim().is_empty() {
                t.update(|list| list.push((v, false)));
                iv2.set(String::new());
                set_property(&input_ref, "value", &JsValue::from_str(""));
            }
        }
    });
    let add_btn = text_el("button", "Add");
    set_attribute(&add_btn, "class", "btn-primary");
    let t2 = todos;
    let iv3 = input_val;
    let inp2 = input.clone();
    add_event_listener(&add_btn, "click", move |_| {
        let v = iv3.get();
        if !v.trim().is_empty() {
            t2.update(|list| list.push((v, false)));
            iv3.set(String::new());
            set_property(&inp2, "value", &JsValue::from_str(""));
        }
    });
    append_node(&input_row, &input);
    append_node(&input_row, &add_btn);
    append_node(&content, &input_row);

    // Filters
    let filters = el("div", "todo-filters", &[]);
    let mut filter_btns: Vec<web_sys::Element> = Vec::new();
    for (i, label) in ["All", "Active", "Done"].iter().enumerate() {
        let btn = text_el("button", label);
        set_attribute(&btn, "class", "btn-sm");
        let f = filter;
        let idx = i as u8;
        add_event_listener(&btn, "click", move |_| { f.set(idx); });
        filter_btns.push(btn.clone());
        append_node(&filters, &btn);
    }
    append_node(&content, &filters);

    // Update filter button active states
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

    // List
    let list = el("ul", "todo-list", &[]);
    let list_ref = list.clone();
    create_effect(move || {
        clear_children(&list_ref);
        let items = todos.get();
        let f = filter.get();
        for (i, (text, done)) in items.iter().enumerate() {
            let show = match f {
                1 => !done,
                2 => *done,
                _ => true,
            };
            if !show { continue; }
            let li = el("li", if *done { "todo-item done" } else { "todo-item" }, &[]);

            let cb = create_element("input");
            set_attribute(&cb, "type", "checkbox");
            if *done { set_property(&cb, "checked", &JsValue::TRUE); }
            let t = todos;
            let idx = i;
            add_event_listener(&cb, "change", move |_| {
                t.update(|list| { list[idx].1 = !list[idx].1; });
            });
            append_node(&li, &cb);

            let span = text_el("span", text);
            append_node(&li, &span);

            let del = text_el("button", "✕");
            set_attribute(&del, "class", "btn-sm btn-danger");
            let t = todos;
            add_event_listener(&del, "click", move |_| {
                t.update(|list| { list.remove(idx); });
            });
            append_node(&li, &del);

            append_node(&list_ref, &li);
        }

        // Persist
        let serialized = serialize_todos(&todos.get());
        local_storage_set("oxide-todos", &serialized);
    });
    append_node(&content, &list);

    // Count
    let count_el = el("div", "todo-count", &[]);
    let count_ref = count_el.clone();
    create_effect(move || {
        let items = todos.get();
        let active = items.iter().filter(|(_, d)| !d).count();
        count_ref.set_inner_html(&format!("{} item{} remaining", active, if active == 1 { "" } else { "s" }));
    });
    append_node(&content, &count_el);

    section("Todo List", "✅", "Full CRUD with filtering and localStorage persistence.",
        &[("Signal<Vec>", "signal"), ("localStorage", "api"), ("Events", "dom")], content)
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

    let content = el("div", "col", &[]);
    let time_el = el("div", "stopwatch-time", &[]);
    reactive_text(&time_el, display);
    append_node(&content, &time_el);

    let btns = el("div", "stopwatch-btns", &[]);

    let start_btn = text_el("button", "Start");
    set_attribute(&start_btn, "class", "btn-primary");
    let start_ref = start_btn.clone();
    add_event_listener(&start_btn, "click", move |_| {
        if running.get() {
            clear_interval(interval_id.get());
            running.set(false);
            set_property(&start_ref, "textContent", &"Start".into());
        } else {
            let e = elapsed_ms;
            let id = set_interval(move || { e.update(|v| *v += 10); }, 10);
            interval_id.set(id);
            running.set(true);
            set_property(&start_ref, "textContent", &"Pause".into());
        }
    });

    let reset_btn = text_el("button", "Reset");
    let sb = start_btn.clone();
    add_event_listener(&reset_btn, "click", move |_| {
        clear_interval(interval_id.get());
        running.set(false);
        elapsed_ms.set(0);
        set_property(&sb, "textContent", &"Start".into());
    });

    append_node(&btns, &start_btn);
    append_node(&btns, &reset_btn);
    append_node(&content, &btns);

    section("Stopwatch", "⏱️", "Precise timing with setInterval and formatted display.",
        &[("setInterval", "timer"), ("memo", "signal"), ("Closure", "dom")], content)
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. Form Playground
// ═══════════════════════════════════════════════════════════════════════════

fn demo_forms() -> web_sys::Element {
    let name = signal(String::new());
    let email = signal(String::new());
    let color = signal("#f97316".to_string());
    let range_val = signal(50u32);
    let checked = signal(false);
    let select_val = signal("rust".to_string());

    let content = el("div", "col", &[]);
    let grid = el("div", "form-grid", &[]);

    // Text input
    let f1 = el("div", "form-field", &[]);
    append_node(&f1, &text_el("label", "Name"));
    let inp = create_element("input");
    set_attribute(&inp, "type", "text");
    set_attribute(&inp, "placeholder", "Your name");
    let n = name;
    add_event_listener(&inp, "input", move |e| { n.set(event_target_value(&e)); });
    append_node(&f1, &inp);
    append_node(&grid, &f1);

    // Email input
    let f2 = el("div", "form-field", &[]);
    append_node(&f2, &text_el("label", "Email"));
    let inp2 = create_element("input");
    set_attribute(&inp2, "type", "text");
    set_attribute(&inp2, "placeholder", "you@example.com");
    let em = email;
    add_event_listener(&inp2, "input", move |e| { em.set(event_target_value(&e)); });
    append_node(&f2, &inp2);
    append_node(&grid, &f2);

    // Color picker
    let f3 = el("div", "form-field", &[]);
    append_node(&f3, &text_el("label", "Favorite Color"));
    let cp = create_element("input");
    set_attribute(&cp, "type", "color");
    set_attribute(&cp, "value", "#f97316");
    let c = color;
    add_event_listener(&cp, "input", move |e| { c.set(event_target_value(&e)); });
    append_node(&f3, &cp);
    append_node(&grid, &f3);

    // Range slider
    let f4 = el("div", "form-field", &[]);
    let range_label = el("label", "", &[]);
    let range_label_ref = range_label.clone();
    create_effect(move || {
        range_label_ref.set_inner_html(&format!("Volume: {}", range_val.get()));
    });
    append_node(&f4, &range_label);
    let slider = create_element("input");
    set_attribute(&slider, "type", "range");
    set_attribute(&slider, "min", "0");
    set_attribute(&slider, "max", "100");
    set_attribute(&slider, "value", "50");
    let rv = range_val;
    add_event_listener(&slider, "input", move |e| {
        if let Ok(v) = event_target_value(&e).parse::<u32>() { rv.set(v); }
    });
    append_node(&f4, &slider);
    append_node(&grid, &f4);

    // Checkbox
    let f5 = el("div", "form-field", &[]);
    append_node(&f5, &text_el("label", "Subscribe"));
    let row = el("div", "row", &[]);
    let cb = create_element("input");
    set_attribute(&cb, "type", "checkbox");
    let ch = checked;
    add_event_listener(&cb, "change", move |e| { ch.set(event_target_checked(&e)); });
    append_node(&row, &cb);
    append_node(&row, &text_el("span", "Send me updates"));
    append_node(&f5, &row);
    append_node(&grid, &f5);

    // Select
    let f6 = el("div", "form-field", &[]);
    append_node(&f6, &text_el("label", "Language"));
    let sel = create_element("select");
    for (val, label) in &[("rust", "Rust"), ("ts", "TypeScript"), ("go", "Go"), ("python", "Python")] {
        let opt = create_element("option");
        set_attribute(&opt, "value", val);
        append_text(&opt, label);
        append_node(&sel, &opt);
    }
    let sv = select_val;
    add_event_listener(&sel, "change", move |e| { sv.set(event_target_value(&e)); });
    append_node(&f6, &sel);
    append_node(&grid, &f6);

    append_node(&content, &grid);

    // Output
    let output = el("div", "form-output", &[]);
    let output_ref = output.clone();
    create_effect(move || {
        output_ref.set_inner_html(&format!(
            "<code>{{ name: \"{}\", email: \"{}\", color: \"{}\", volume: {}, subscribed: {}, lang: \"{}\" }}</code>",
            name.get(), email.get(), color.get(), range_val.get(), checked.get(), select_val.get()
        ));
    });
    append_node(&content, &output);

    section("Form Inputs", "📝", "Every HTML input type with live reactive output.",
        &[("Signal", "signal"), ("Input Events", "dom"), ("Forms", "api")], content)
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. Fetch API
// ═══════════════════════════════════════════════════════════════════════════

fn demo_fetch() -> web_sys::Element {
    let result = signal("Click a button to fetch data.".to_string());
    let loading = signal(false);

    let content = el("div", "col", &[]);
    let btns = el("div", "row", &[]);

    // Fetch joke
    let btn1 = text_el("button", "Random Joke");
    let r = result;
    let l = loading;
    add_event_listener(&btn1, "click", move |_| {
        l.set(true);
        r.set("Loading...".into());
        let r = r;
        let l = l;
        wasm_bindgen_futures::spawn_local(async move {
            match fetch_text("https://official-joke-api.appspot.com/random_joke").await {
                Ok(text) => { r.set(text); l.set(false); }
                Err(e) => { r.set(format!("Error: {:?}", e)); l.set(false); }
            }
        });
    });
    append_node(&btns, &btn1);

    // Fetch IP
    let btn2 = text_el("button", "My IP Address");
    let r2 = result;
    let l2 = loading;
    add_event_listener(&btn2, "click", move |_| {
        l2.set(true);
        r2.set("Loading...".into());
        let r = r2;
        let l = l2;
        wasm_bindgen_futures::spawn_local(async move {
            match fetch_text("https://api.ipify.org?format=json").await {
                Ok(text) => { r.set(text); l.set(false); }
                Err(e) => { r.set(format!("Error: {:?}", e)); l.set(false); }
            }
        });
    });
    append_node(&btns, &btn2);

    // Fetch random user
    let btn3 = text_el("button", "Random User");
    let r3 = result;
    let l3 = loading;
    add_event_listener(&btn3, "click", move |_| {
        l3.set(true);
        r3.set("Loading...".into());
        let r = r3;
        let l = l3;
        wasm_bindgen_futures::spawn_local(async move {
            match fetch_text("https://randomuser.me/api/?results=1&noinfo").await {
                Ok(text) => { r.set(text); l.set(false); }
                Err(e) => { r.set(format!("Error: {:?}", e)); l.set(false); }
            }
        });
    });
    append_node(&btns, &btn3);
    append_node(&content, &btns);

    let res_el = el("div", "fetch-result", &[]);
    let res_ref = res_el.clone();
    create_effect(move || {
        res_ref.set_text_content(Some(&result.get()));
    });
    append_node(&content, &res_el);

    section("Fetch API", "🌐", "Asynchronous HTTP requests with loading states.",
        &[("fetch()", "api"), ("async/await", "api"), ("Signal", "signal")], content)
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

    let coords = el("div", "mouse-coords", &[]);
    let coords_ref = coords.clone();
    create_effect(move || {
        coords_ref.set_inner_html(&format!("x: <b>{}</b> · y: <b>{}</b>", mx.get(), my.get()));
    });
    append_node(&content, &coords);

    section("Mouse Tracker", "🖱️", "Real-time mouse position tracking with visual feedback.",
        &[("MouseEvent", "dom"), ("Signal", "signal"), ("getBoundingClientRect", "api")], content)
}

// ═══════════════════════════════════════════════════════════════════════════
// 8. Keyboard Events
// ═══════════════════════════════════════════════════════════════════════════

fn demo_keyboard() -> web_sys::Element {
    let key = signal("?".to_string());
    let code = signal(String::new());
    let modifiers = signal(String::new());

    let content = el("div", "col", &[]);
    append_node(&content, &text_el("p", "Press any key…"));

    let display = el("div", "key-display", &[]);
    let cap = el("div", "key-cap", &[]);
    let cap_ref = cap.clone();
    create_effect(move || { cap_ref.set_text_content(Some(&key.get())); });
    append_node(&display, &cap);
    append_node(&content, &display);

    let info = el("div", "key-info", &[]);
    let info_ref = info.clone();
    create_effect(move || {
        let c = code.get();
        let m = modifiers.get();
        let txt = if c.is_empty() {
            "Waiting for input...".to_string()
        } else {
            format!("Code: {} {}", c, if m.is_empty() { String::new() } else { format!("· Modifiers: {}", m) })
        };
        info_ref.set_text_content(Some(&txt));
    });
    append_node(&content, &info);

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

    section("Keyboard Events", "⌨️", "Capture keystrokes with modifier detection.",
        &[("KeyboardEvent", "dom"), ("keydown", "dom"), ("Signal", "signal")], content)
}

// ═══════════════════════════════════════════════════════════════════════════
// 9. Canvas Drawing
// ═══════════════════════════════════════════════════════════════════════════

fn demo_canvas() -> web_sys::Element {
    let drawing = signal(false);
    let brush_color = signal("#f97316".to_string());

    let content = el("div", "col", &[]);

    // Tools
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

    // Canvas
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

    section("Canvas Drawing", "🎨", "Freehand drawing with HTML5 Canvas API.",
        &[("Canvas2D", "api"), ("MouseEvent", "dom"), ("Signal", "signal")], content)
}

// ═══════════════════════════════════════════════════════════════════════════
// 10. Theme Toggle
// ═══════════════════════════════════════════════════════════════════════════

fn demo_theme() -> web_sys::Element {
    let dark = signal(true);

    let content = el("div", "col", &[]);
    let btn = text_el("button", "Toggle Theme");
    set_attribute(&btn, "class", "btn-primary");
    let d = dark;
    add_event_listener(&btn, "click", move |_| { d.set(!d.get()); });
    append_node(&content, &btn);

    let preview = el("div", "theme-preview dark", &[]);
    let preview_ref = preview.clone();
    create_effect(move || {
        let is_dark = dark.get();
        set_attribute(&preview_ref, "class",
            if is_dark { "theme-preview dark" } else { "theme-preview light" });
        preview_ref.set_inner_html(if is_dark {
            "<h3>🌙 Dark Mode</h3><p>Easy on the eyes for late-night coding.</p>"
        } else {
            "<h3>☀️ Light Mode</h3><p>Bright and clean for daytime work.</p>"
        });
    });
    append_node(&content, &preview);

    section("Theme Toggle", "🎨", "Switch between dark and light themes with CSS.",
        &[("Signal", "signal"), ("classList", "dom"), ("CSS Variables", "dom")], content)
}

// ═══════════════════════════════════════════════════════════════════════════
// 11. Persistent Notes
// ═══════════════════════════════════════════════════════════════════════════

fn demo_notes() -> web_sys::Element {
    let saved = local_storage_get("oxide-notes").unwrap_or_else(|| "Write your notes here...\n\nThey persist across page reloads via localStorage!".into());
    let text = signal(saved);
    let status = signal("Saved ✓".to_string());

    let content = el("div", "col", &[]);
    let ta = create_element("textarea");
    set_attribute(&ta, "class", "notes-area");
    set_attribute(&ta, "rows", "6");
    let ta_ref = ta.clone();
    create_effect(move || {
        set_property(&ta_ref, "value", &JsValue::from_str(&text.get()));
    });
    let t = text;
    let st = status;
    add_event_listener(&ta, "input", move |e| {
        let v = event_target_value(&e);
        t.set(v.clone());
        local_storage_set("oxide-notes", &v);
        st.set("Saved ✓".into());
    });
    append_node(&content, &ta);

    let stat = el("div", "notes-status", &[]);
    let stat_ref = stat.clone();
    create_effect(move || { stat_ref.set_text_content(Some(&status.get())); });
    append_node(&content, &stat);

    section("Persistent Notes", "📒", "Notes that survive page reloads via localStorage.",
        &[("localStorage", "api"), ("textarea", "dom"), ("Signal", "signal")], content)
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

    let btn = text_el("button", "Pause");
    let btn_ref = btn.clone();
    let r = running;
    add_event_listener(&btn, "click", move |_| {
        r.set(!r.get());
        set_property(&btn_ref, "textContent",
            &JsValue::from_str(if r.get() { "Pause" } else { "Resume" }));
    });
    append_node(&content, &btn);

    section("Bouncing Ball", "🏀", "Smooth animation using requestAnimationFrame.",
        &[("rAF", "timer"), ("Signal", "signal"), ("CSS Transform", "dom")], content)
}

// ═══════════════════════════════════════════════════════════════════════════
// 13. SVG Bar Chart
// ═══════════════════════════════════════════════════════════════════════════

fn demo_chart() -> web_sys::Element {
    let data = signal(vec![65u32, 40, 80, 55, 90, 35, 70]);
    let labels = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

    let content = el("div", "col", &[]);
    let chart_wrap = el("div", "chart-wrap", &[]);
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

    let btns = el("div", "chart-btns", &[]);
    let rand_btn = text_el("button", "Randomize");
    add_event_listener(&rand_btn, "click", move |_| {
        d.update(|v| {
            for val in v.iter_mut() {
                *val = pseudo_random(*val);
            }
        });
    });
    append_node(&btns, &rand_btn);
    append_node(&content, &btns);

    section("SVG Bar Chart", "📊", "Dynamic SVG generation with reactive data.",
        &[("SVG", "api"), ("Signal<Vec>", "signal"), ("createElementNS", "dom")], content)
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

    let content = el("div", "col", &[]);
    let btn = text_el("button", "Open Modal");
    set_attribute(&btn, "class", "btn-primary");
    let o = open;
    add_event_listener(&btn, "click", move |_| { o.set(true); });
    append_node(&content, &btn);

    let overlay = el("div", "overlay hidden", &[]);
    let modal = el("div", "modal", &[]);
    modal.set_inner_html("<h3>🔥 Oxide Modal</h3><p>This modal is rendered and controlled entirely by Rust signals compiled to WASM. No JavaScript!</p>");
    let close_btn = text_el("button", "Close");
    let o2 = open;
    add_event_listener(&close_btn, "click", move |_| { o2.set(false); });
    append_node(&modal, &close_btn);
    append_node(&overlay, &modal);

    let overlay_bg = overlay.clone();
    let o3 = open;
    add_event_listener(&overlay_bg, "click", move |e| {
        let target = e.target().unwrap();
        let el: web_sys::Element = target.dyn_into().unwrap();
        if el.class_list().contains("overlay") { o3.set(false); }
    });
    append_node(&content, &overlay);

    let overlay_ref = overlay.clone();
    create_effect(move || {
        if open.get() {
            overlay_ref.class_list().remove_1("hidden").ok();
        } else {
            overlay_ref.class_list().add_1("hidden").ok();
        }
    });

    section("Modal Dialog", "💬", "Overlay dialog controlled by a boolean signal.",
        &[("Signal<bool>", "signal"), ("classList", "dom"), ("Events", "dom")], content)
}

// ═══════════════════════════════════════════════════════════════════════════
// 15. Drag & Drop
// ═══════════════════════════════════════════════════════════════════════════

fn demo_dnd() -> web_sys::Element {
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

    section("Drag & Drop", "🖐️", "Native HTML5 Drag and Drop API.",
        &[("DragEvent", "dom"), ("DataTransfer", "api"), ("preventDefault", "dom")], content)
}

// ═══════════════════════════════════════════════════════════════════════════
// 16. Clipboard
// ═══════════════════════════════════════════════════════════════════════════

fn demo_clipboard() -> web_sys::Element {
    let copied = signal(false);

    let content = el("div", "col", &[]);
    let row = el("div", "row", &[]);

    let text_div = el("div", "clip-text", &[]);
    append_text(&text_div, "🔥 Oxide — Rust frontend framework compiling to WASM");
    append_node(&row, &text_div);

    let btn = text_el("button", "📋 Copy");
    let c = copied;
    add_event_listener(&btn, "click", move |_| {
        let window = web_sys::window().unwrap();
        let nav = window.navigator();
        let clipboard = nav.clipboard();
        let promise = clipboard.write_text("🔥 Oxide — Rust frontend framework compiling to WASM");
        let c = c;
        wasm_bindgen_futures::spawn_local(async move {
            let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
            c.set(true);
            set_timeout(move || { c.set(false); }, 2000);
        });
    });
    append_node(&row, &btn);

    let toast = el("span", "clip-toast", &[]);
    append_text(&toast, "Copied!");
    let toast_ref = toast.clone();
    create_effect(move || {
        if copied.get() {
            toast_ref.class_list().add_1("show").ok();
        } else {
            toast_ref.class_list().remove_1("show").ok();
        }
    });
    append_node(&row, &toast);
    append_node(&content, &row);

    section("Clipboard", "📋", "Copy text to clipboard with the async Clipboard API.",
        &[("Clipboard API", "api"), ("async/await", "api"), ("Signal", "signal")], content)
}
