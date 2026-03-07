/// Re-exports from `oxide-core` — reactive primitives.
pub use oxide_core::{batch, create_effect, memo, provide_context, signal, untrack, use_context, Signal};

/// Re-exports from `oxide-macros` — the `view!` macro.
pub use oxide_macros::view;

/// DOM renderer utilities.
pub mod dom {
    pub use oxide_dom::*;
}

/// The Component trait — implement this for struct-based components.
///
/// ```ignore
/// struct Counter { initial: i32 }
///
/// impl oxide::Component for Counter {
///     fn render(self) -> web_sys::Element {
///         let count = signal(self.initial);
///         view! { <div>{count}</div> }
///     }
/// }
///
/// // In view!: <Counter initial={5} />
/// ```
pub trait Component {
    fn render(self) -> web_sys::Element;
}

/// Convenient glob import: `use oxide::prelude::*;`
pub mod prelude {
    pub use oxide_core::{batch, create_effect, memo, provide_context, signal, untrack, use_context, Signal};
    pub use oxide_dom::mount;
    pub use oxide_macros::view;
    pub use super::Component;
}
