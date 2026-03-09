use bueler::prelude::*;
use bueler::dom::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[wasm_bindgen(start)]
pub fn main() {
    set_title("Bueler Todo App");

    mount("#app", || {
        let todos: Signal<Vec<(String, bool)>> = signal({
            if let Some(saved) = local_storage_get("bueler-todo-app") {
                parse_todos(&saved)
            } else {
                vec![
                    ("Learn Bueler".into(), false),
                    ("Build something awesome".into(), false),
                ]
            }
        });
        let input_val = signal(String::new());
        let filter = signal(0u8); // 0=all, 1=active, 2=done

        // Persist on every change
        create_effect(move || {
            local_storage_set("bueler-todo-app", &serialize_todos(&todos.get()));
        });

        let remaining = memo(move || {
            todos.get().iter().filter(|(_, done)| !done).count()
        });

        let input = create_element("input");
        set_attribute(&input, "type", "text");
        set_attribute(&input, "placeholder", "What needs to be done?");
        set_attribute(&input, "class", "new-todo");
        let iv = input_val;
        add_event_listener(&input, "input", move |e| {
            iv.set(event_target_value(&e));
        });
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

        let root = create_element("div");
        set_attribute(&root, "class", "todoapp");

        // Header
        let header = create_element("header");
        let h1 = create_element("h1");
        append_text(&h1, "todos");
        append_node(&header, &h1);
        append_node(&header, &input);
        append_node(&root, &header);

        // List
        let list = create_element("ul");
        set_attribute(&list, "class", "todo-list");
        let list_ref = list.clone();
        create_effect(move || {
            clear_children(&list_ref);
            let items = todos.get();
            let f = filter.get();
            for (i, (text, done)) in items.iter().enumerate() {
                let show = match f { 1 => !done, 2 => *done, _ => true };
                if !show { continue; }

                let li = create_element("li");
                if *done { set_attribute(&li, "class", "completed"); }

                let cb = create_element("input");
                set_attribute(&cb, "type", "checkbox");
                set_attribute(&cb, "class", "toggle");
                if *done { set_property(&cb, "checked", &JsValue::TRUE); }
                let t = todos;
                let idx = i;
                add_event_listener(&cb, "change", move |_| {
                    t.update(|list| list[idx].1 = !list[idx].1);
                });
                append_node(&li, &cb);

                let label = create_element("label");
                append_text(&label, text);
                append_node(&li, &label);

                let del = create_element("button");
                set_attribute(&del, "class", "destroy");
                append_text(&del, "×");
                let t = todos;
                add_event_listener(&del, "click", move |_| {
                    t.update(|list| { list.remove(idx); });
                });
                append_node(&li, &del);

                append_node(&list_ref, &li);
            }
        });
        append_node(&root, &list);

        // Footer
        let footer = create_element("footer");
        set_attribute(&footer, "class", "footer");

        let count_el = create_element("span");
        set_attribute(&count_el, "class", "todo-count");
        let count_ref = count_el.clone();
        create_effect(move || {
            let n = remaining.get();
            count_ref.set_inner_html(&format!(
                "<strong>{}</strong> item{} left",
                n, if n == 1 { "" } else { "s" }
            ));
        });
        append_node(&footer, &count_el);

        // Filter buttons
        let filters = create_element("ul");
        set_attribute(&filters, "class", "filters");
        let mut filter_btns = Vec::new();
        for (idx, label) in ["All", "Active", "Completed"].iter().enumerate() {
            let li = create_element("li");
            let a = create_element("a");
            append_text(&a, label);
            set_attribute(&a, "href", "#");
            let f = filter;
            let i = idx as u8;
            add_event_listener(&a, "click", move |e| {
                e.prevent_default();
                f.set(i);
            });
            filter_btns.push(a.clone());
            append_node(&li, &a);
            append_node(&filters, &li);
        }
        append_node(&footer, &filters);

        // Update active filter
        create_effect(move || {
            let f = filter.get() as usize;
            for (i, btn) in filter_btns.iter().enumerate() {
                if i == f {
                    set_attribute(btn, "class", "selected");
                } else {
                    btn.remove_attribute("class").ok();
                }
            }
        });

        // Clear completed
        let clear_btn = create_element("button");
        set_attribute(&clear_btn, "class", "clear-completed");
        append_text(&clear_btn, "Clear completed");
        let t = todos;
        add_event_listener(&clear_btn, "click", move |_| {
            t.update(|list| list.retain(|(_, done)| !done));
        });
        append_node(&footer, &clear_btn);

        append_node(&root, &footer);
        root
    });
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
    todos.iter()
        .map(|(t, d)| format!("{} {}", if *d { "[x]" } else { "[ ]" }, t))
        .collect::<Vec<_>>()
        .join("\n")
}
