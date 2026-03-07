use std::cell::Cell;

/// Lifecycle events emitted by the reactive runtime.
/// Install a hook with [`set_hook`] to observe these events.
#[derive(Debug, Clone)]
pub enum HookEvent {
    /// A new signal was created.
    SignalCreate { id: usize },
    /// A signal value was read (may establish a subscription).
    SignalRead { id: usize },
    /// A signal value was written (will notify subscribers).
    SignalWrite { id: usize },
    /// An effect began executing.
    EffectRun { id: usize },
    /// An effect finished executing (duration in milliseconds).
    EffectComplete { id: usize, duration_ms: f64 },
    /// A batch of updates began.
    BatchStart,
    /// A batch of updates completed.
    BatchEnd { effect_count: usize },
}

thread_local! {
    static HOOK_FN: Cell<Option<fn(HookEvent)>> = Cell::new(None);
}

/// Install a global hook to observe reactive runtime events.
/// Only one hook can be active at a time — calling this replaces any
/// previous hook.
pub fn set_hook(hook: fn(HookEvent)) {
    HOOK_FN.with(|h| h.set(Some(hook)));
}

/// Remove the global hook.
pub fn clear_hook() {
    HOOK_FN.with(|h| h.set(None));
}

/// Fire a hook event. No-op if no hook is installed.
#[inline]
pub(crate) fn fire(event: HookEvent) {
    HOOK_FN.with(|h| {
        if let Some(f) = h.get() {
            f(event);
        }
    });
}
