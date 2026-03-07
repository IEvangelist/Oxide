use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static CONTEXT: RefCell<HashMap<TypeId, Box<dyn Any>>> = RefCell::new(HashMap::new());
}

/// Inject a value into the global context. Retrieve it later with
/// [`use_context`].
pub fn provide_context<T: 'static>(value: T) {
    CONTEXT.with(|ctx| {
        ctx.borrow_mut().insert(TypeId::of::<T>(), Box::new(value));
    });
}

/// Retrieve a value previously injected with [`provide_context`].
/// Returns `None` if no value of this type has been provided.
pub fn use_context<T: 'static + Clone>() -> Option<T> {
    CONTEXT.with(|ctx| {
        ctx.borrow()
            .get(&TypeId::of::<T>())
            .and_then(|v| v.downcast_ref::<T>())
            .cloned()
    })
}
