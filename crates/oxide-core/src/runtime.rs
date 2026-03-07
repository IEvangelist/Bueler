use std::any::Any;
use std::cell::RefCell;

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
    RUNTIME.with(|rt| {
        let mut inner = rt.borrow_mut();
        let id = inner.signals.len();
        inner.signals.push(ReactiveNode {
            value: Box::new(value),
            subscribers: Vec::new(),
        });
        id
    })
}

pub(crate) fn read_signal_rt<T: 'static + Clone>(id: usize) -> T {
    RUNTIME.with(|rt| {
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
    })
}

pub(crate) fn write_signal_rt<T: 'static>(id: usize, value: T) {
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
        for effect_id in pending {
            run_effect(rt, effect_id);
        }
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
}
