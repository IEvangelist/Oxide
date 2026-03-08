use std::any::Any;
use std::cell::RefCell;

use crate::hooks::{fire, HookEvent};

struct ReactiveNode {
    value: Box<dyn Any>,
    subscribers: Vec<usize>,
}

struct EffectNode {
    f: Option<Box<dyn FnMut()>>,
    dependencies: Vec<usize>,
}

struct RuntimeInner {
    signals: Vec<ReactiveNode>,
    effects: Vec<EffectNode>,
    tracking: Option<usize>,
    batching: bool,
    pending_effects: Vec<usize>,
}

thread_local! {
    static RUNTIME: RefCell<RuntimeInner> = RefCell::new(RuntimeInner {
        signals: Vec::new(),
        effects: Vec::new(),
        tracking: None,
        batching: false,
        pending_effects: Vec::new(),
    });
}

// ---------------------------------------------------------------------------
// Signal operations (called from Signal<T> methods)
// ---------------------------------------------------------------------------

pub(crate) fn create_signal_rt<T: 'static>(value: T) -> usize {
    let id = RUNTIME.with(|rt| {
        let mut inner = rt.borrow_mut();
        let id = inner.signals.len();
        inner.signals.push(ReactiveNode {
            value: Box::new(value),
            subscribers: Vec::new(),
        });
        id
    });
    fire(HookEvent::SignalCreate { id });
    id
}

pub(crate) fn read_signal_rt<T: 'static + Clone>(id: usize) -> T {
    let val = RUNTIME.with(|rt| {
        {
            let mut inner = rt.borrow_mut();
            if let Some(effect_id) = inner.tracking {
                if !inner.signals[id].subscribers.contains(&effect_id) {
                    inner.signals[id].subscribers.push(effect_id);
                }
                if !inner.effects[effect_id].dependencies.contains(&id) {
                    inner.effects[effect_id].dependencies.push(id);
                }
            }
        }
        let inner = rt.borrow();
        inner.signals[id].value.downcast_ref::<T>().unwrap().clone()
    });
    fire(HookEvent::SignalRead { id });
    val
}

pub(crate) fn write_signal_rt<T: 'static>(id: usize, value: T) {
    fire(HookEvent::SignalWrite { id });
    RUNTIME.with(|rt| {
        let (subscribers, batching) = {
            let mut inner = rt.borrow_mut();
            inner.signals[id].value = Box::new(value);
            let subs = inner.signals[id].subscribers.clone();
            (subs, inner.batching)
        };

        if batching {
            let mut inner = rt.borrow_mut();
            for sub in subscribers {
                if !inner.pending_effects.contains(&sub) {
                    inner.pending_effects.push(sub);
                }
            }
        } else {
            for effect_id in subscribers {
                run_effect(rt, effect_id);
            }
        }
    });
}

pub(crate) fn update_signal_rt<T: 'static>(id: usize, f: impl FnOnce(&mut T)) {
    RUNTIME.with(|rt| {
        let (subscribers, batching) = {
            let mut inner = rt.borrow_mut();
            let val = inner.signals[id].value.downcast_mut::<T>().unwrap();
            f(val);
            let subs = inner.signals[id].subscribers.clone();
            (subs, inner.batching)
        };

        if batching {
            let mut inner = rt.borrow_mut();
            for sub in subscribers {
                if !inner.pending_effects.contains(&sub) {
                    inner.pending_effects.push(sub);
                }
            }
        } else {
            for effect_id in subscribers {
                run_effect(rt, effect_id);
            }
        }
    });
}

// ---------------------------------------------------------------------------
// Effects
// ---------------------------------------------------------------------------

/// Create a reactive effect that automatically re-runs when its signal
/// dependencies change. The effect runs immediately once upon creation.
pub fn create_effect(f: impl FnMut() + 'static) {
    RUNTIME.with(|rt| {
        let id = {
            let mut inner = rt.borrow_mut();
            let id = inner.effects.len();
            inner.effects.push(EffectNode {
                f: Some(Box::new(f)),
                dependencies: Vec::new(),
            });
            id
        };
        run_effect(rt, id);
    });
}

/// Execute the given effect: clear old dependency edges, set tracking context,
/// run the closure (which re-establishes deps), then restore previous context.
///
/// **Key invariant**: the `RefCell` borrow is *dropped* before calling user
/// code so that signal reads/writes inside the closure don't panic.
fn run_effect(rt: &RefCell<RuntimeInner>, effect_id: usize) {
    fire(HookEvent::EffectRun { id: effect_id });
    let start = now();

    let prev_tracking = {
        let mut inner = rt.borrow_mut();
        let prev = inner.tracking;

        // Remove this effect from all of its old signal subscriber lists.
        let old_deps = std::mem::take(&mut inner.effects[effect_id].dependencies);
        for sig_id in old_deps {
            if sig_id < inner.signals.len() {
                inner.signals[sig_id]
                    .subscribers
                    .retain(|&s| s != effect_id);
            }
        }

        inner.tracking = Some(effect_id);
        prev
    }; // borrow dropped

    // Take the closure out so we can call it without holding a borrow.
    let mut f = {
        let mut inner = rt.borrow_mut();
        inner.effects[effect_id]
            .f
            .take()
            .expect("effect closure missing — possible infinite cycle")
    }; // borrow dropped

    // --- user code runs here (may read/write signals) ---
    f();

    // Put the closure back and restore tracking.
    {
        let mut inner = rt.borrow_mut();
        inner.effects[effect_id].f = Some(f);
        inner.tracking = prev_tracking;
    }

    fire(HookEvent::EffectComplete { id: effect_id, duration_ms: now() - start });
}

/// High-resolution timestamp (ms). Falls back to 0.0 outside the browser.
fn now() -> f64 {
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Date::now()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        0.0
    }
}

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

/// Run a closure *without* tracking signal reads (useful for one-off reads
/// that should not create subscriptions).
pub fn untrack<T>(f: impl FnOnce() -> T) -> T {
    RUNTIME.with(|rt| {
        let prev = {
            let mut inner = rt.borrow_mut();
            let prev = inner.tracking;
            inner.tracking = None;
            prev
        };
        let result = f();
        {
            let mut inner = rt.borrow_mut();
            inner.tracking = prev;
        }
        result
    })
}

/// Batch multiple signal updates — effects are deferred until the batch ends.
pub fn batch(f: impl FnOnce()) {
    fire(HookEvent::BatchStart);
    RUNTIME.with(|rt| {
        {
            rt.borrow_mut().batching = true;
        }
        f();
        let pending = {
            let mut inner = rt.borrow_mut();
            inner.batching = false;
            std::mem::take(&mut inner.pending_effects)
        };
        let count = pending.len();
        for effect_id in pending {
            run_effect(rt, effect_id);
        }
        fire(HookEvent::BatchEnd { effect_count: count });
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signal::Signal;
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn signal_read_write() {
        let s = Signal::new(10);
        assert_eq!(s.get(), 10);
        s.set(20);
        assert_eq!(s.get(), 20);
    }

    #[test]
    fn effect_tracks_signal() {
        let s = Signal::new(0);
        let observed = Rc::new(Cell::new(-1));
        let obs = observed.clone();

        create_effect(move || {
            obs.set(s.get());
        });

        // Effect runs immediately
        assert_eq!(observed.get(), 0);

        // Effect re-runs on set
        s.set(42);
        assert_eq!(observed.get(), 42);
    }

    #[test]
    fn batch_defers_effects() {
        let a = Signal::new(0);
        let b = Signal::new(0);
        let run_count = Rc::new(Cell::new(0u32));
        let rc = run_count.clone();

        create_effect(move || {
            let _ = a.get() + b.get();
            rc.set(rc.get() + 1);
        });

        assert_eq!(run_count.get(), 1); // initial run

        batch(move || {
            a.set(1);
            b.set(2);
        });

        // Only one extra run (not two)
        assert_eq!(run_count.get(), 2);
    }

    #[test]
    fn untrack_prevents_subscription() {
        let s = Signal::new(0);
        let observed = Rc::new(Cell::new(-1));
        let obs = observed.clone();

        create_effect(move || {
            let val = untrack(|| s.get());
            obs.set(val);
        });

        assert_eq!(observed.get(), 0);

        s.set(99);
        // Effect should NOT have re-run
        assert_eq!(observed.get(), 0);
    }

    #[test]
    fn signal_add_assign() {
        let mut s = Signal::new(5);
        s += 3;
        assert_eq!(s.get(), 8);
    }

    #[test]
    fn signal_sub_assign() {
        let mut s = Signal::new(10);
        s -= 3;
        assert_eq!(s.get(), 7);
    }

    #[test]
    fn signal_update_in_place() {
        let s = Signal::new(vec![1, 2, 3]);
        s.update(|v| v.push(4));
        assert_eq!(s.get(), vec![1, 2, 3, 4]);
    }

    #[test]
    fn signal_display() {
        let s = Signal::new(42);
        assert_eq!(format!("{}", s), "42");
    }

    #[test]
    fn memo_computes_derived_value() {
        let count = Signal::new(3);
        let doubled = crate::memo::memo(move || count.get() * 2);
        assert_eq!(doubled.get(), 6);
        count.set(10);
        assert_eq!(doubled.get(), 20);
    }

    #[test]
    fn memo_chains() {
        let a = Signal::new(2);
        let b = crate::memo::memo(move || a.get() + 1);
        let c = crate::memo::memo(move || b.get() * 10);
        assert_eq!(c.get(), 30);
        a.set(5);
        assert_eq!(b.get(), 6);
        assert_eq!(c.get(), 60);
    }

    #[test]
    fn context_provide_and_use() {
        crate::context::provide_context(42i32);
        assert_eq!(crate::context::use_context::<i32>(), Some(42));
        assert_eq!(crate::context::use_context::<String>(), None);
    }

    #[test]
    fn context_overwrite() {
        crate::context::provide_context("first".to_string());
        assert_eq!(crate::context::use_context::<String>(), Some("first".to_string()));
        crate::context::provide_context("second".to_string());
        assert_eq!(crate::context::use_context::<String>(), Some("second".to_string()));
    }

    #[test]
    fn multiple_effects_on_one_signal() {
        let s = Signal::new(0);
        let sum = Rc::new(Cell::new(0i32));
        let s1 = sum.clone();
        let s2 = sum.clone();

        create_effect(move || { s1.set(s1.get() + s.get()); });
        create_effect(move || { s2.set(s2.get() + s.get() * 10); });

        // Initial: both effects ran once with s=0
        assert_eq!(sum.get(), 0);

        s.set(1);
        // Effect 1: sum += 1, Effect 2: sum += 10 → 11
        assert_eq!(sum.get(), 11);
    }

    #[test]
    fn effect_resubscribes_on_condition_change() {
        let flag = Signal::new(true);
        let a = Signal::new(1);
        let b = Signal::new(2);
        let observed = Rc::new(Cell::new(0));
        let obs = observed.clone();

        create_effect(move || {
            let val = if flag.get() { a.get() } else { b.get() };
            obs.set(val);
        });

        assert_eq!(observed.get(), 1); // reads a

        flag.set(false);
        assert_eq!(observed.get(), 2); // now reads b

        a.set(99);
        // Effect should NOT re-run since it no longer depends on a
        assert_eq!(observed.get(), 2);

        b.set(42);
        assert_eq!(observed.get(), 42);
    }

    #[test]
    fn nested_batch() {
        let s = Signal::new(0);
        let count = Rc::new(Cell::new(0u32));
        let c = count.clone();

        create_effect(move || {
            let _ = s.get();
            c.set(c.get() + 1);
        });

        assert_eq!(count.get(), 1);

        batch(|| {
            s.set(1);
            s.set(2);
            s.set(3);
        });

        // Only 1 additional run despite 3 sets
        assert_eq!(count.get(), 2);
        assert_eq!(s.get(), 3);
    }

    #[test]
    fn watch_skips_initial() {
        let s = Signal::new(0);
        let seen = Rc::new(Cell::new(false));
        let se = seen.clone();

        crate::reactive::watch(move || s.get(), move |_val| {
            se.set(true);
        });

        // watch should NOT have fired on initial value
        assert!(!seen.get());

        s.set(1);
        assert!(seen.get());
    }

    #[test]
    fn watch_receives_new_value() {
        let s = Signal::new(0);
        let observed = Rc::new(Cell::new(-1));
        let obs = observed.clone();

        crate::reactive::watch(move || s.get(), move |val| {
            obs.set(val);
        });

        s.set(42);
        assert_eq!(observed.get(), 42);

        s.set(100);
        assert_eq!(observed.get(), 100);
    }

    #[test]
    fn signal_copy_semantics() {
        let s = Signal::new(7);
        let s2 = s; // Copy
        assert_eq!(s.get(), 7);
        assert_eq!(s2.get(), 7);
        s.set(99);
        assert_eq!(s2.get(), 99); // Same underlying signal
    }

    #[test]
    fn hooks_fire_on_signal_operations() {
        use crate::hooks::{set_hook, clear_hook, HookEvent};
        use std::sync::atomic::{AtomicU32, Ordering};

        static HOOK_COUNT: AtomicU32 = AtomicU32::new(0);

        fn test_hook(_event: HookEvent) {
            HOOK_COUNT.fetch_add(1, Ordering::SeqCst);
        }

        HOOK_COUNT.store(0, Ordering::SeqCst);
        set_hook(test_hook);

        let s = Signal::new(0);  // SignalCreate
        let _ = s.get();          // SignalRead
        s.set(1);                 // SignalWrite + EffectRun (if any)

        clear_hook();

        // At minimum: create + read + write = 3 events
        assert!(HOOK_COUNT.load(Ordering::SeqCst) >= 3);
    }
}
