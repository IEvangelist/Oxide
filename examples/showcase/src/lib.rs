use oxide::prelude::*;
use oxide::dom::*;
use oxide::{Signal, memo};
use oxide::telemetry;
use oxide::resiliency;
use oxide::router::{Router, RouterMode, route, navigate};
use oxide::components::{self, Severity};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

// ═══════════════════════════════════════════════════════════════════════════
// Entry point — SPA with hash-based routing
// ═══════════════════════════════════════════════════════════════════════════

#[wasm_bindgen(start)]
pub fn main() {
    mount("#app", || {
        let router = Router::new(RouterMode::Hash, &[
            route("/",                    page_home),
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
        ]);

        build_app_shell(router)
    });
}

// ═══════════════════════════════════════════════════════════════════════════
// App Shell — header nav + router outlet
// ═══════════════════════════════════════════════════════════════════════════

fn build_app_shell(router: Router) -> web_sys::Element {
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
    set_attribute(&logo_a, "href", "index.html");
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
        ("Demos", "/demos"),
        ("Components", "/components"),
    ];
    for &(label, path) in nav_items {
        let a = create_element("a");
        append_text(&a, label);
        let p = path.to_string();
        let no = nav_open;
        add_event_listener(&a, "click", move |_| {
            navigate(&p);
            no.set(false);
        });
        append_node(&nav, &a);
    }

    // Docs link
    let docs_link = create_element("a");
    set_attribute(&docs_link, "href", "docs.html");
    append_text(&docs_link, "Docs");
    append_node(&nav, &docs_link);

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

    // Scroll-to-top
    let b = body();
    b.append_child(&components::scroll_to_top(300)).ok();

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
// Page: Home
// ═══════════════════════════════════════════════════════════════════════════

fn page_home() -> web_sys::Element {
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
    let h2 = text_el("h2", "Component Library");
    append_node(&page, &h2);
    let desc = el("p", "pg-desc", &[]);
    append_text(&desc, "Pre-built Oxide components with interactive playgrounds. Click any card to explore.");
    append_node(&page, &desc);

    let grid = el("div", "catalog-grid", &[]);

    let items: &[(&str, &str, &str, &str)] = &[
        ("\u{1f518}", "Button",    "Versatile button with variants, sizes, and loading states.", "/components/button"),
        ("\u{270f}\u{fe0f}",  "TextInput", "Text input with label, validation, and signal binding.",   "/components/input"),
        ("\u{1f4c4}", "TextArea",  "Multi-line text input with label and signal binding.",       "/components/textarea"),
        ("\u{1f53d}", "Select",    "Dropdown select with label and signal binding.",             "/components/select"),
        ("\u{2611}\u{fe0f}",  "Checkbox",  "Checkbox with label and boolean signal binding.",          "/components/checkbox"),
        ("\u{1f4c7}", "Card",      "Container card with title, body, and optional footer.",      "/components/card"),
        ("\u{1f514}", "Alert",     "Notification alert with severity levels.",                   "/components/alert"),
        ("\u{1fa9f}", "Modal",     "Overlay modal dialog controlled by signals.",                "/components/modal"),
        ("\u{1f504}", "Spinner",   "Rotating loading indicator with optional text.",             "/components/spinner"),
        ("\u{1f4ca}", "Progress",  "Animated progress bar driven by a signal.",                  "/components/progress"),
        ("\u{1f4c1}", "Tabs",      "Tabbed interface with ARIA-compliant panels.",               "/components/tabs"),
        ("\u{1f3f7}\u{fe0f}",  "Badge",     "Colored label badge with severity variants.",              "/components/badge"),
        ("\u{2796}", "Divider",   "Simple horizontal divider element.",                         "/components/divider"),
        ("\u{1f4a0}", "Skeleton",  "Shimmer-animated loading placeholder.",                      "/components/skeleton"),
    ];

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
        let p = path.to_string();
        add_event_listener(&card, "click", move |_| { navigate(&p); });
        append_node(&grid, &card);
    }

    append_node(&page, &grid);
    page
}

// ═══════════════════════════════════════════════════════════════════════════
// Playground helper: builds a standard playground page layout
// ═══════════════════════════════════════════════════════════════════════════

fn pg_shell(title: &str, description: &str) -> (web_sys::Element, web_sys::Element, web_sys::Element, web_sys::Element) {
    let page = el("div", "pg-page", &[]);

    // Back link
    let back = el("a", "pg-back", &[]);
    append_text(&back, "\u{2190} Components");
    add_event_listener(&back, "click", |_| { navigate("/components"); });
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

    page
}

// ═══════════════════════════════════════════════════════════════════════════
// 1. Counter (original demo — unchanged)
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
