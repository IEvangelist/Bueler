use crate::runtime::create_effect;
use crate::signal::Signal;

/// Create a computed (derived) signal that automatically updates when its
/// dependencies change. Similar to `useMemo` in React or `computed` in Vue.
///
/// ```ignore
/// let count = signal(1);
/// let doubled = memo(move || count.get() * 2);
/// assert_eq!(doubled.get(), 2);
/// count.set(5);
/// assert_eq!(doubled.get(), 10);
/// ```
pub fn memo<T: 'static + Clone>(f: impl Fn() -> T + 'static) -> Signal<T> {
    let initial = f();
    let s = Signal::new(initial);
    create_effect(move || {
        let val = f();
        s.set(val);
    });
    s
}
