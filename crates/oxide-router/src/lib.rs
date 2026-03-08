//! # oxide-router
//!
//! Client-side router for Oxide applications. Supports both hash-based
//! and history (pushState) routing modes.
//!
//! ```ignore
//! use oxide_router::*;
//!
//! let router = Router::new(RouterMode::Hash, &[
//!     route("/",        || view! { <h1>"Home"</h1> }),
//!     route("/about",   || view! { <h1>"About"</h1> }),
//!     route("/user/:id", || {
//!         let id = use_param("id");
//!         view! { <h1>"User " {id}</h1> }
//!     }),
//! ]);
//!
//! // In your app:
//! mount("#app", || router.view());
//! ```

use oxide_core::{create_effect, signal, Signal};
use oxide_dom::{create_element, set_attribute, append_text, append_node, add_event_listener, clear_children};
use std::cell::RefCell;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

// ═══════════════════════════════════════════════════════════════════════════
// Public API
// ═══════════════════════════════════════════════════════════════════════════

/// Routing mode.
#[derive(Clone, Copy, PartialEq)]
pub enum RouterMode {
    /// Uses URL hash (`#/path`). Works everywhere without server config.
    Hash,
    /// Uses History API (`pushState`). Requires server to serve index.html for all routes.
    History,
}

/// A route definition.
pub struct Route {
    pub path: &'static str,
    pub view: fn() -> web_sys::Element,
}

/// Convenience constructor.
pub fn route(path: &'static str, view: fn() -> web_sys::Element) -> Route {
    Route { path, view }
}

/// The router instance.
pub struct Router {
    mode: RouterMode,
    routes: Vec<Route>,
    current_path: Signal<String>,
}

impl Router {
    /// Create a new router.
    pub fn new(mode: RouterMode, routes: &[Route]) -> Self {
        let initial = current_path(mode);
        let current_path = signal(initial);

        // Listen for navigation events
        let cp = current_path;
        let m = mode;
        if mode == RouterMode::Hash {
            oxide_dom::on_window_event("hashchange", move |_| {
                cp.set(current_path_fn(m));
            });
        } else {
            oxide_dom::on_window_event("popstate", move |_| {
                cp.set(current_path_fn(m));
            });
        }

        // Move routes into owned vec
        let owned_routes: Vec<Route> = routes.iter().map(|r| Route { path: r.path, view: r.view }).collect();

        Router { mode, routes: owned_routes, current_path }
    }

    /// Build the router outlet element. Place this in your view tree where
    /// matched routes should render.
    pub fn view(&self) -> web_sys::Element {
        let outlet = create_element("div");
        set_attribute(&outlet, "data-oxide-router", "outlet");

        let outlet_ref = outlet.clone();
        let routes: Vec<(&'static str, fn() -> web_sys::Element)> =
            self.routes.iter().map(|r| (r.path, r.view)).collect();
        let cp = self.current_path;

        create_effect(move || {
            clear_children(&outlet_ref);
            let path = cp.get();

            // Store matched params globally
            PARAMS.with(|p| p.borrow_mut().clear());

            let matched = routes.iter().find(|(pattern, _)| {
                matches_route(pattern, &path)
            });

            if let Some((pattern, view_fn)) = matched {
                // Extract params
                let params = extract_params(pattern, &path);
                PARAMS.with(|p| *p.borrow_mut() = params);
                let el = view_fn();
                append_node(&outlet_ref, &el);
            } else {
                let el = create_element("div");
                append_text(&el, &format!("404 — No route matches: {}", path));
                append_node(&outlet_ref, &el);
            }
        });

        outlet
    }

    /// Get the current path signal.
    pub fn current(&self) -> Signal<String> {
        self.current_path
    }
}

/// Navigate to a path programmatically.
pub fn navigate(path: &str) {
    let window = web_sys::window().unwrap();
    // Detect mode from presence of hash
    let loc = window.location();
    let current_hash = loc.hash().unwrap_or_default();

    if current_hash.starts_with('#') || path.starts_with('#') {
        // Hash mode
        let hash_path = if path.starts_with('#') { path.to_string() } else { format!("#{}", path) };
        loc.set_hash(&hash_path).ok();
    } else {
        // History mode
        let history = window.history().unwrap();
        history.push_state_with_url(&JsValue::NULL, "", Some(path)).ok();
        // Dispatch popstate to notify router
        let event = web_sys::PopStateEvent::new("popstate").unwrap();
        window.dispatch_event(&event).ok();
    }
}

/// Create a `<a>` link element that uses client-side navigation.
pub fn link(href: &str, text: &str) -> web_sys::Element {
    let a = create_element("a");
    set_attribute(&a, "href", href);
    append_text(&a, text);
    let h = href.to_string();
    add_event_listener(&a, "click", move |e| {
        e.prevent_default();
        navigate(&h);
    });
    a
}

/// Create a `<a>` link with an "active" class when the current route matches.
pub fn nav_link(href: &str, text: &str, _active_class: &str) -> web_sys::Element {
    let a = link(href, text);
    // TODO: add reactive class based on current route matching href
    a
}

/// Get the current route path as a signal.
pub fn use_route() -> Signal<String> {
    let mode = if web_sys::window()
        .unwrap()
        .location()
        .hash()
        .unwrap_or_default()
        .starts_with('#')
    {
        RouterMode::Hash
    } else {
        RouterMode::History
    };
    let path = signal(current_path(mode));
    let p = path;
    let m = mode;
    oxide_dom::on_window_event("hashchange", move |_| { p.set(current_path_fn(m)); });
    oxide_dom::on_window_event("popstate", move |_| { p.set(current_path_fn(m)); });
    path
}

/// Get route parameters (e.g., `:id` → `"42"`).
pub fn use_param(name: &str) -> String {
    PARAMS.with(|p| {
        p.borrow().get(name).cloned().unwrap_or_default()
    })
}

/// Get all route parameters.
pub fn use_params() -> HashMap<String, String> {
    PARAMS.with(|p| p.borrow().clone())
}

// ═══════════════════════════════════════════════════════════════════════════
// Internal
// ═══════════════════════════════════════════════════════════════════════════

thread_local! {
    static PARAMS: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
}

fn current_path(mode: RouterMode) -> String {
    current_path_fn(mode)
}

fn current_path_fn(mode: RouterMode) -> String {
    let window = web_sys::window().unwrap();
    let loc = window.location();
    match mode {
        RouterMode::Hash => {
            let hash = loc.hash().unwrap_or_default();
            let path = hash.strip_prefix('#').unwrap_or(&hash);
            if path.is_empty() { "/".to_string() } else { path.to_string() }
        }
        RouterMode::History => {
            loc.pathname().unwrap_or_else(|_| "/".to_string())
        }
    }
}

/// Check if a route pattern matches a path. Supports `:param` segments.
fn matches_route(pattern: &str, path: &str) -> bool {
    let pat_parts: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();
    let path_parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    if pat_parts.len() != path_parts.len() {
        return false;
    }

    pat_parts.iter().zip(path_parts.iter()).all(|(pat, pth)| {
        pat.starts_with(':') || pat == pth
    })
}

/// Extract named params from a matched route.
fn extract_params(pattern: &str, path: &str) -> HashMap<String, String> {
    let pat_parts: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();
    let path_parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    let mut params = HashMap::new();

    for (pat, pth) in pat_parts.iter().zip(path_parts.iter()) {
        if let Some(name) = pat.strip_prefix(':') {
            params.insert(name.to_string(), pth.to_string());
        }
    }

    params
}
