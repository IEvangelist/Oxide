use crate::runtime::create_effect;
use crate::signal::{signal, Signal};

/// Watch a signal source and run a callback whenever it changes.
/// Unlike `create_effect`, the callback receives the new value and
/// does NOT run on initial creation.
///
/// ```ignore
/// let count = signal(0);
/// watch(move || count.get(), |val| {
///     log(&format!("Count changed to {}", val));
/// });
/// ```
pub fn watch<T: Clone + PartialEq + 'static>(
    source: impl Fn() -> T + 'static,
    mut callback: impl FnMut(T) + 'static,
) {
    let prev: Signal<Option<T>> = signal(None);
    create_effect(move || {
        let val = source();
        let old = prev.get();
        if old.as_ref() != Some(&val) {
            prev.set(Some(val.clone()));
            if old.is_some() {
                callback(val);
            }
        }
    });
}

/// Run a callback once on the next microtask (after current reactive
/// context completes). Useful for post-mount DOM access.
///
/// ```ignore
/// on_mount(|| {
///     log("Component mounted!");
/// });
/// ```
#[cfg(target_arch = "wasm32")]
pub fn on_mount(f: impl FnOnce() + 'static) {
    let promise = js_sys::Promise::resolve(&wasm_bindgen::JsValue::NULL);
    let closure = wasm_bindgen::closure::Closure::once(move |_: wasm_bindgen::JsValue| { f(); });
    let _ = promise.then(&closure);
    closure.forget();
}

#[cfg(not(target_arch = "wasm32"))]
pub fn on_mount(f: impl FnOnce() + 'static) {
    f();
}

/// Register a cleanup callback. In a full scope-based system these
/// run when the scope is disposed. Currently they run on page unload.
///
/// ```ignore
/// on_cleanup(|| {
///     log("Cleaning up resources");
/// });
/// ```
#[cfg(target_arch = "wasm32")]
pub fn on_cleanup(f: impl FnOnce() + 'static) {
    use wasm_bindgen::JsCast;

    let closure = wasm_bindgen::closure::Closure::once(move |_: web_sys::Event| { f(); });
    web_sys::window()
        .unwrap()
        .add_event_listener_with_callback("beforeunload", closure.as_ref().unchecked_ref())
        .ok();
    closure.forget();
}

#[cfg(not(target_arch = "wasm32"))]
pub fn on_cleanup(_f: impl FnOnce() + 'static) {}
