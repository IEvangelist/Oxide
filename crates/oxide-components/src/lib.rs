//! # oxide-components
//!
//! Best-in-class UI component library for Oxide applications.
//! All components use a builder API, include built-in dark-theme CSS
//! (auto-injected on first use), and follow WAI-ARIA accessibility patterns.
//!
//! ```ignore
//! use oxide_components::*;
//!
//! let btn = button("Save").primary().on_click(move |_| save());
//! let inp = text_input("Email").placeholder("you@example.com").bind(email);
//! let card = card("Settings").body(content).build();
//! ```

use oxide_core::{create_effect, signal, Signal};
use oxide_dom::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

// ═══════════════════════════════════════════════════════════════════════════
// CSS injection (lazy, once per component type)
// ═══════════════════════════════════════════════════════════════════════════

fn inject_css(id: &str, css: &str) {
    if query_selector(&format!("style[data-ox=\"{}\"]", id)).is_some() {
        return;
    }
    let doc = web_sys::window().unwrap().document().unwrap();
    let style = doc.create_element("style").unwrap();
    style.set_attribute("data-ox", id).ok();
    style.set_text_content(Some(css));
    if let Some(head) = doc.head() {
        head.append_child(&style).ok();
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 1. Button
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Clone, Copy, PartialEq)]
pub enum Variant { Primary, Secondary, Outline, Danger, Ghost }

#[derive(Clone, Copy)]
pub enum Size { Small, Medium, Large }

pub struct ButtonBuilder {
    label: String,
    variant: Variant,
    size: Size,
    loading: bool,
    disabled: bool,
}

pub fn button(label: &str) -> ButtonBuilder {
    ButtonBuilder {
        label: label.to_string(),
        variant: Variant::Secondary,
        size: Size::Medium,
        loading: false,
        disabled: false,
    }
}

impl ButtonBuilder {
    pub fn primary(mut self) -> Self { self.variant = Variant::Primary; self }
    pub fn outline(mut self) -> Self { self.variant = Variant::Outline; self }
    pub fn danger(mut self) -> Self { self.variant = Variant::Danger; self }
    pub fn ghost(mut self) -> Self { self.variant = Variant::Ghost; self }
    pub fn small(mut self) -> Self { self.size = Size::Small; self }
    pub fn large(mut self) -> Self { self.size = Size::Large; self }
    pub fn loading(mut self, v: bool) -> Self { self.loading = v; self }
    pub fn disabled(mut self, v: bool) -> Self { self.disabled = v; self }

    pub fn on_click(self, handler: impl FnMut(web_sys::Event) + 'static) -> web_sys::Element {
        let el = self.build();
        add_event_listener(&el, "click", handler);
        el
    }

    pub fn build(self) -> web_sys::Element {
        inject_css("btn", BUTTON_CSS);
        let el = create_element("button");
        let mut cls = format!("ox-btn ox-btn-{}", match self.variant {
            Variant::Primary => "primary", Variant::Secondary => "secondary",
            Variant::Outline => "outline", Variant::Danger => "danger", Variant::Ghost => "ghost",
        });
        cls += match self.size { Size::Small => " ox-btn-sm", Size::Large => " ox-btn-lg", _ => "" };
        if self.loading { cls += " ox-btn-loading"; }
        set_attribute(&el, "class", &cls);
        if self.disabled || self.loading { set_attribute(&el, "disabled", ""); }
        if self.loading {
            set_inner_html(&el, &format!("<span class=\"ox-spinner-inline\"></span> {}", self.label));
        } else {
            append_text(&el, &self.label);
        }
        el.set_attribute("role", "button").ok();
        el
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. TextInput
// ═══════════════════════════════════════════════════════════════════════════

pub struct InputBuilder {
    label: String,
    input_type: String,
    placeholder: String,
    required: bool,
    error_msg: Option<String>,
    signal: Option<Signal<String>>,
}

pub fn text_input(label: &str) -> InputBuilder {
    InputBuilder {
        label: label.to_string(),
        input_type: "text".into(),
        placeholder: String::new(),
        required: false,
        error_msg: None,
        signal: None,
    }
}

impl InputBuilder {
    pub fn input_type(mut self, t: &str) -> Self { self.input_type = t.into(); self }
    pub fn placeholder(mut self, p: &str) -> Self { self.placeholder = p.into(); self }
    pub fn required(mut self) -> Self { self.required = true; self }
    pub fn error(mut self, msg: &str) -> Self { self.error_msg = Some(msg.into()); self }
    pub fn bind(mut self, s: Signal<String>) -> Self { self.signal = Some(s); self }

    pub fn build(self) -> web_sys::Element {
        inject_css("input", INPUT_CSS);
        let wrap = create_element("div");
        set_attribute(&wrap, "class", "ox-field");
        if !self.label.is_empty() {
            let lbl = create_element("label");
            set_attribute(&lbl, "class", "ox-label");
            append_text(&lbl, &self.label);
            if self.required { let req = create_element("span"); set_attribute(&req, "class", "ox-required"); append_text(&req, " *"); append_node(&lbl, &req); }
            append_node(&wrap, &lbl);
        }
        let input = create_element("input");
        let cls = if self.error_msg.is_some() { "ox-input ox-input-error" } else { "ox-input" };
        set_attribute(&input, "class", cls);
        set_attribute(&input, "type", &self.input_type);
        if !self.placeholder.is_empty() { set_attribute(&input, "placeholder", &self.placeholder); }
        if self.required { set_attribute(&input, "required", ""); input.set_attribute("aria-required", "true").ok(); }
        if let Some(s) = self.signal {
            let inp = input.clone();
            create_effect(move || { set_property(&inp, "value", &JsValue::from_str(&s.get())); });
            add_event_listener(&input, "input", move |e| { s.set(event_target_value(&e)); });
        }
        append_node(&wrap, &input);
        if let Some(msg) = &self.error_msg {
            let err = create_element("div");
            set_attribute(&err, "class", "ox-error");
            err.set_attribute("role", "alert").ok();
            append_text(&err, msg);
            append_node(&wrap, &err);
        }
        wrap
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. TextArea
// ═══════════════════════════════════════════════════════════════════════════

pub fn textarea(label: &str, value: Signal<String>) -> web_sys::Element {
    inject_css("input", INPUT_CSS);
    let wrap = create_element("div");
    set_attribute(&wrap, "class", "ox-field");
    if !label.is_empty() {
        let lbl = create_element("label");
        set_attribute(&lbl, "class", "ox-label");
        append_text(&lbl, label);
        append_node(&wrap, &lbl);
    }
    let ta = create_element("textarea");
    set_attribute(&ta, "class", "ox-input");
    set_attribute(&ta, "rows", "4");
    let ta_ref = ta.clone();
    create_effect(move || { set_property(&ta_ref, "value", &JsValue::from_str(&value.get())); });
    let v = value;
    add_event_listener(&ta, "input", move |e| { v.set(event_target_value(&e)); });
    append_node(&wrap, &ta);
    wrap
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. Select
// ═══════════════════════════════════════════════════════════════════════════

pub fn select(label: &str, options: &[(&str, &str)], value: Signal<String>) -> web_sys::Element {
    inject_css("input", INPUT_CSS);
    let wrap = create_element("div");
    set_attribute(&wrap, "class", "ox-field");
    if !label.is_empty() {
        let lbl = create_element("label");
        set_attribute(&lbl, "class", "ox-label");
        append_text(&lbl, label);
        append_node(&wrap, &lbl);
    }
    let sel = create_element("select");
    set_attribute(&sel, "class", "ox-input");
    for &(val, text) in options {
        let opt = create_element("option");
        set_attribute(&opt, "value", val);
        append_text(&opt, text);
        append_node(&sel, &opt);
    }
    let sel_ref = sel.clone();
    create_effect(move || { set_property(&sel_ref, "value", &JsValue::from_str(&value.get())); });
    let v = value;
    add_event_listener(&sel, "change", move |e| { v.set(event_target_value(&e)); });
    append_node(&wrap, &sel);
    wrap
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. Checkbox
// ═══════════════════════════════════════════════════════════════════════════

pub fn checkbox(label: &str, checked: Signal<bool>) -> web_sys::Element {
    inject_css("checkbox", CHECKBOX_CSS);
    let wrap = create_element("label");
    set_attribute(&wrap, "class", "ox-checkbox");
    let input = create_element("input");
    set_attribute(&input, "type", "checkbox");
    let inp_ref = input.clone();
    create_effect(move || { set_property(&inp_ref, "checked", &JsValue::from_bool(checked.get())); });
    let c = checked;
    add_event_listener(&input, "change", move |e| { c.set(event_target_checked(&e)); });
    append_node(&wrap, &input);
    let span = create_element("span");
    append_text(&span, label);
    append_node(&wrap, &span);
    wrap
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. Card
// ═══════════════════════════════════════════════════════════════════════════

pub struct CardBuilder {
    title: String,
    body_el: Option<web_sys::Element>,
    footer_el: Option<web_sys::Element>,
}

pub fn card(title: &str) -> CardBuilder {
    CardBuilder { title: title.into(), body_el: None, footer_el: None }
}

impl CardBuilder {
    pub fn body(mut self, el: web_sys::Element) -> Self { self.body_el = Some(el); self }
    pub fn footer(mut self, el: web_sys::Element) -> Self { self.footer_el = Some(el); self }

    pub fn build(self) -> web_sys::Element {
        inject_css("card", CARD_CSS);
        let card = create_element("div");
        set_attribute(&card, "class", "ox-card");
        if !self.title.is_empty() {
            let header = create_element("div");
            set_attribute(&header, "class", "ox-card-header");
            append_text(&header, &self.title);
            append_node(&card, &header);
        }
        if let Some(body) = self.body_el {
            let b = create_element("div");
            set_attribute(&b, "class", "ox-card-body");
            append_node(&b, &body);
            append_node(&card, &b);
        }
        if let Some(footer) = self.footer_el {
            let f = create_element("div");
            set_attribute(&f, "class", "ox-card-footer");
            append_node(&f, &footer);
            append_node(&card, &f);
        }
        card
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. Alert
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Clone, Copy)]
pub enum Severity { Success, Warning, Error, Info }

pub struct AlertBuilder {
    message: String,
    severity: Severity,
    dismissible: Option<Signal<bool>>,
}

pub fn alert(message: &str) -> AlertBuilder {
    AlertBuilder { message: message.into(), severity: Severity::Info, dismissible: None }
}

impl AlertBuilder {
    pub fn success(mut self) -> Self { self.severity = Severity::Success; self }
    pub fn warning(mut self) -> Self { self.severity = Severity::Warning; self }
    pub fn error(mut self) -> Self { self.severity = Severity::Error; self }
    pub fn info(mut self) -> Self { self.severity = Severity::Info; self }
    pub fn dismissible(mut self, visible: Signal<bool>) -> Self { self.dismissible = Some(visible); self }

    pub fn build(self) -> web_sys::Element {
        inject_css("alert", ALERT_CSS);
        let el = create_element("div");
        let sev = match self.severity {
            Severity::Success => "success", Severity::Warning => "warning",
            Severity::Error => "error", Severity::Info => "info",
        };
        set_attribute(&el, "class", &format!("ox-alert ox-alert-{}", sev));
        el.set_attribute("role", "alert").ok();
        let icon = match self.severity {
            Severity::Success => "✓", Severity::Warning => "⚠", Severity::Error => "✕", Severity::Info => "ℹ",
        };
        let ic = create_element("span");
        set_attribute(&ic, "class", "ox-alert-icon");
        append_text(&ic, icon);
        append_node(&el, &ic);
        append_text(&el, &self.message);
        if let Some(vis) = self.dismissible {
            let btn = create_element("button");
            set_attribute(&btn, "class", "ox-alert-close");
            btn.set_attribute("aria-label", "Dismiss").ok();
            append_text(&btn, "×");
            add_event_listener(&btn, "click", move |_| { vis.set(false); });
            append_node(&el, &btn);
            let el_ref = el.clone();
            create_effect(move || { if !vis.get() { set_style(&el_ref, "display", "none"); } else { set_style(&el_ref, "display", ""); } });
        }
        el
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 8. Modal
// ═══════════════════════════════════════════════════════════════════════════

pub struct ModalBuilder {
    open: Signal<bool>,
    title: String,
    body_el: Option<web_sys::Element>,
}

pub fn modal(open: Signal<bool>) -> ModalBuilder {
    ModalBuilder { open, title: String::new(), body_el: None }
}

impl ModalBuilder {
    pub fn title(mut self, t: &str) -> Self { self.title = t.into(); self }
    pub fn body(mut self, el: web_sys::Element) -> Self { self.body_el = Some(el); self }

    pub fn build(self) -> web_sys::Element {
        inject_css("modal", MODAL_CSS);
        let overlay = create_element("div");
        set_attribute(&overlay, "class", "ox-modal-overlay");
        overlay.set_attribute("role", "dialog").ok();
        overlay.set_attribute("aria-modal", "true").ok();

        let dialog = create_element("div");
        set_attribute(&dialog, "class", "ox-modal");
        if !self.title.is_empty() {
            let header = create_element("div");
            set_attribute(&header, "class", "ox-modal-header");
            let h3 = create_element("h3");
            append_text(&h3, &self.title);
            append_node(&header, &h3);
            let close = create_element("button");
            set_attribute(&close, "class", "ox-modal-close");
            close.set_attribute("aria-label", "Close").ok();
            append_text(&close, "×");
            let o = self.open;
            add_event_listener(&close, "click", move |_| { o.set(false); });
            append_node(&header, &close);
            append_node(&dialog, &header);
        }
        if let Some(body) = self.body_el {
            let b = create_element("div");
            set_attribute(&b, "class", "ox-modal-body");
            append_node(&b, &body);
            append_node(&dialog, &b);
        }
        append_node(&overlay, &dialog);

        // Close on backdrop click
        let o2 = self.open;
        let overlay_ref = overlay.clone();
        add_event_listener(&overlay, "click", move |e| {
            if let Some(t) = e.target() {
                if let Some(el) = t.dyn_ref::<web_sys::Element>() {
                    if el.class_list().contains("ox-modal-overlay") { o2.set(false); }
                }
            }
        });

        // Show/hide
        let overlay_ref2 = overlay_ref.clone();
        create_effect(move || {
            if self.open.get() {
                set_style(&overlay_ref2, "display", "flex");
            } else {
                set_style(&overlay_ref2, "display", "none");
            }
        });
        set_style(&overlay, "display", "none");
        overlay
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 9. Spinner
// ═══════════════════════════════════════════════════════════════════════════

pub fn spinner() -> web_sys::Element {
    inject_css("spinner", SPINNER_CSS);
    let el = create_element("div");
    set_attribute(&el, "class", "ox-spinner");
    el.set_attribute("role", "status").ok();
    el.set_attribute("aria-label", "Loading").ok();
    el
}

pub fn spinner_with_text(text: &str) -> web_sys::Element {
    inject_css("spinner", SPINNER_CSS);
    let wrap = create_element("div");
    set_attribute(&wrap, "class", "ox-spinner-wrap");
    let sp = create_element("div");
    set_attribute(&sp, "class", "ox-spinner");
    sp.set_attribute("role", "status").ok();
    append_node(&wrap, &sp);
    let t = create_element("span");
    set_attribute(&t, "class", "ox-spinner-text");
    append_text(&t, text);
    append_node(&wrap, &t);
    wrap
}

// ═══════════════════════════════════════════════════════════════════════════
// 10. Progress
// ═══════════════════════════════════════════════════════════════════════════

pub fn progress(value: Signal<f64>) -> web_sys::Element {
    inject_css("progress", PROGRESS_CSS);
    let wrap = create_element("div");
    set_attribute(&wrap, "class", "ox-progress");
    wrap.set_attribute("role", "progressbar").ok();
    let bar = create_element("div");
    set_attribute(&bar, "class", "ox-progress-bar");
    let bar_ref = bar.clone();
    let wrap_ref = wrap.clone();
    create_effect(move || {
        let v = value.get().clamp(0.0, 100.0);
        set_style(&bar_ref, "width", &format!("{}%", v));
        wrap_ref.set_attribute("aria-valuenow", &format!("{}", v as i32)).ok();
    });
    append_node(&wrap, &bar);
    wrap
}

// ═══════════════════════════════════════════════════════════════════════════
// 11. Tabs
// ═══════════════════════════════════════════════════════════════════════════

pub fn tabs(items: &[(&str, fn() -> web_sys::Element)]) -> web_sys::Element {
    inject_css("tabs", TABS_CSS);
    let active = signal(0usize);
    let wrap = create_element("div");
    set_attribute(&wrap, "class", "ox-tabs");

    let nav = create_element("div");
    set_attribute(&nav, "class", "ox-tabs-nav");
    nav.set_attribute("role", "tablist").ok();
    let mut tab_btns: Vec<web_sys::Element> = Vec::new();
    for (i, &(label, _)) in items.iter().enumerate() {
        let btn = create_element("button");
        set_attribute(&btn, "class", "ox-tab-btn");
        btn.set_attribute("role", "tab").ok();
        append_text(&btn, label);
        let a = active;
        add_event_listener(&btn, "click", move |_| { a.set(i); });
        tab_btns.push(btn.clone());
        append_node(&nav, &btn);
    }
    append_node(&wrap, &nav);

    let panel = create_element("div");
    set_attribute(&panel, "class", "ox-tab-panel");
    panel.set_attribute("role", "tabpanel").ok();

    let views: Vec<fn() -> web_sys::Element> = items.iter().map(|&(_, v)| v).collect();
    let panel_ref = panel.clone();
    create_effect(move || {
        let idx = active.get();
        clear_children(&panel_ref);
        if idx < views.len() {
            let content = views[idx]();
            append_node(&panel_ref, &content);
        }
        for (i, btn) in tab_btns.iter().enumerate() {
            if i == idx {
                set_attribute(btn, "class", "ox-tab-btn ox-tab-active");
                btn.set_attribute("aria-selected", "true").ok();
            } else {
                set_attribute(btn, "class", "ox-tab-btn");
                btn.set_attribute("aria-selected", "false").ok();
            }
        }
    });
    append_node(&wrap, &panel);
    wrap
}

// ═══════════════════════════════════════════════════════════════════════════
// 12. Badge
// ═══════════════════════════════════════════════════════════════════════════

pub fn badge(text: &str, severity: Severity) -> web_sys::Element {
    inject_css("badge", BADGE_CSS);
    let el = create_element("span");
    let sev = match severity {
        Severity::Success => "success", Severity::Warning => "warning",
        Severity::Error => "error", Severity::Info => "info",
    };
    set_attribute(&el, "class", &format!("ox-badge ox-badge-{}", sev));
    append_text(&el, text);
    el
}

// ═══════════════════════════════════════════════════════════════════════════
// 13. Divider
// ═══════════════════════════════════════════════════════════════════════════

pub fn divider() -> web_sys::Element {
    inject_css("divider", ".ox-divider{border:none;border-top:1px solid #333;margin:1.5rem 0}");
    let el = create_element("hr");
    set_attribute(&el, "class", "ox-divider");
    el
}

// ═══════════════════════════════════════════════════════════════════════════
// 14. Skeleton (loading placeholder)
// ═══════════════════════════════════════════════════════════════════════════

pub fn skeleton(width: &str, height: &str) -> web_sys::Element {
    inject_css("skeleton", SKELETON_CSS);
    let el = create_element("div");
    set_attribute(&el, "class", "ox-skeleton");
    set_style(&el, "width", width);
    set_style(&el, "height", height);
    el
}

// ═══════════════════════════════════════════════════════════════════════════
// CSS constants
// ═══════════════════════════════════════════════════════════════════════════

const BUTTON_CSS: &str = "\
.ox-btn{display:inline-flex;align-items:center;gap:0.5rem;padding:0.5rem 1.1rem;font-size:0.85rem;font-family:inherit;font-weight:500;\
border:1px solid #444;border-radius:8px;cursor:pointer;transition:all .15s;outline:none;color:#e0e0e0;background:#1e1e1e}\
.ox-btn:hover{border-color:#f97316;color:#f97316}\
.ox-btn:active{transform:scale(.97)}\
.ox-btn:disabled{opacity:.5;cursor:not-allowed;transform:none}\
.ox-btn:focus-visible{box-shadow:0 0 0 2px #f97316}\
.ox-btn-primary{background:linear-gradient(135deg,#f97316,#ef4444);border-color:transparent;color:#fff;font-weight:600}\
.ox-btn-primary:hover{opacity:.9;color:#fff;border-color:transparent}\
.ox-btn-outline{background:transparent;border-color:#666}\
.ox-btn-outline:hover{background:rgba(249,115,22,.08)}\
.ox-btn-danger{background:transparent;border-color:#ef4444;color:#ef4444}\
.ox-btn-danger:hover{background:#ef4444;color:#fff}\
.ox-btn-ghost{background:transparent;border-color:transparent}\
.ox-btn-ghost:hover{background:rgba(255,255,255,.05)}\
.ox-btn-sm{padding:0.3rem 0.7rem;font-size:0.75rem}\
.ox-btn-lg{padding:0.65rem 1.5rem;font-size:1rem}\
.ox-btn-loading{pointer-events:none}\
.ox-spinner-inline{display:inline-block;width:14px;height:14px;border:2px solid rgba(255,255,255,.3);\
border-top-color:#fff;border-radius:50%;animation:ox-spin .6s linear infinite}\
@keyframes ox-spin{to{transform:rotate(360deg)}}";

const INPUT_CSS: &str = "\
.ox-field{display:flex;flex-direction:column;gap:0.35rem}\
.ox-label{font-size:0.8rem;font-weight:500;color:#aaa}\
.ox-required{color:#ef4444}\
.ox-input{background:#0a0a0a;border:1px solid #444;border-radius:8px;padding:0.5rem 0.75rem;\
color:#e0e0e0;font-size:0.85rem;font-family:inherit;outline:none;transition:border-color .15s;width:100%}\
.ox-input:focus{border-color:#f97316}\
.ox-input-error{border-color:#ef4444}\
.ox-error{font-size:0.75rem;color:#ef4444}\
textarea.ox-input{resize:vertical;min-height:80px}\
select.ox-input{cursor:pointer}";

const CHECKBOX_CSS: &str = "\
.ox-checkbox{display:inline-flex;align-items:center;gap:0.5rem;cursor:pointer;font-size:0.85rem;color:#ccc}\
.ox-checkbox input{accent-color:#f97316;width:18px;height:18px;cursor:pointer}";

const CARD_CSS: &str = "\
.ox-card{background:#141414;border:1px solid #333;border-radius:12px;overflow:hidden}\
.ox-card-header{padding:1rem 1.25rem;font-weight:600;font-size:0.95rem;border-bottom:1px solid #222}\
.ox-card-body{padding:1.25rem}\
.ox-card-footer{padding:0.75rem 1.25rem;border-top:1px solid #222;display:flex;gap:0.5rem;justify-content:flex-end}";

const ALERT_CSS: &str = "\
.ox-alert{display:flex;align-items:center;gap:0.6rem;padding:0.75rem 1rem;border-radius:8px;font-size:0.85rem;position:relative}\
.ox-alert-icon{font-size:1rem;flex-shrink:0}\
.ox-alert-success{background:rgba(34,197,94,.1);border:1px solid rgba(34,197,94,.3);color:#86efac}\
.ox-alert-warning{background:rgba(234,179,8,.1);border:1px solid rgba(234,179,8,.3);color:#fde047}\
.ox-alert-error{background:rgba(239,68,68,.1);border:1px solid rgba(239,68,68,.3);color:#fca5a5}\
.ox-alert-info{background:rgba(59,130,246,.1);border:1px solid rgba(59,130,246,.3);color:#93c5fd}\
.ox-alert-close{position:absolute;right:0.5rem;top:50%;transform:translateY(-50%);background:none;border:none;\
color:inherit;font-size:1.2rem;cursor:pointer;opacity:.6;padding:0.25rem}\
.ox-alert-close:hover{opacity:1}";

const MODAL_CSS: &str = "\
.ox-modal-overlay{position:fixed;inset:0;background:rgba(0,0,0,.7);display:flex;align-items:center;\
justify-content:center;z-index:1000;backdrop-filter:blur(4px)}\
.ox-modal{background:#141414;border:1px solid #333;border-radius:14px;max-width:480px;width:92%;max-height:85vh;overflow-y:auto}\
.ox-modal-header{display:flex;justify-content:space-between;align-items:center;padding:1rem 1.25rem;border-bottom:1px solid #222}\
.ox-modal-header h3{font-size:1.05rem}\
.ox-modal-close{background:none;border:none;color:#888;font-size:1.3rem;cursor:pointer;padding:0.25rem}\
.ox-modal-close:hover{color:#fff}\
.ox-modal-body{padding:1.25rem}";

const SPINNER_CSS: &str = "\
.ox-spinner{width:28px;height:28px;border:3px solid #333;border-top-color:#f97316;\
border-radius:50%;animation:ox-spin .6s linear infinite}\
.ox-spinner-wrap{display:flex;flex-direction:column;align-items:center;gap:0.75rem}\
.ox-spinner-text{font-size:0.85rem;color:#888}";

const PROGRESS_CSS: &str = "\
.ox-progress{width:100%;height:8px;background:#1e1e1e;border-radius:4px;overflow:hidden}\
.ox-progress-bar{height:100%;background:linear-gradient(90deg,#f97316,#ef4444);border-radius:4px;\
transition:width .3s ease}";

const TABS_CSS: &str = "\
.ox-tabs{display:flex;flex-direction:column}\
.ox-tabs-nav{display:flex;border-bottom:1px solid #333;gap:0}\
.ox-tab-btn{padding:0.6rem 1.2rem;background:none;border:none;border-bottom:2px solid transparent;\
color:#888;font-size:0.85rem;cursor:pointer;transition:all .15s;font-family:inherit}\
.ox-tab-btn:hover{color:#e0e0e0}\
.ox-tab-active{color:#f97316!important;border-bottom-color:#f97316}\
.ox-tab-panel{padding:1rem 0}";

const BADGE_CSS: &str = "\
.ox-badge{display:inline-flex;padding:0.15rem 0.55rem;border-radius:999px;font-size:0.7rem;font-weight:600}\
.ox-badge-success{background:rgba(34,197,94,.15);color:#86efac}\
.ox-badge-warning{background:rgba(234,179,8,.15);color:#fde047}\
.ox-badge-error{background:rgba(239,68,68,.15);color:#fca5a5}\
.ox-badge-info{background:rgba(59,130,246,.15);color:#93c5fd}";

const SKELETON_CSS: &str = "\
.ox-skeleton{background:linear-gradient(90deg,#1e1e1e 25%,#2a2a2a 50%,#1e1e1e 75%);\
background-size:200% 100%;animation:ox-shimmer 1.5s infinite;border-radius:6px}\
@keyframes ox-shimmer{0%{background-position:200% 0}100%{background-position:-200% 0}}";
