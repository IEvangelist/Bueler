use std::fmt;
use std::marker::PhantomData;
use std::ops::{Add, AddAssign, Sub, SubAssign};

use crate::runtime::{create_signal_rt, read_signal_rt, update_signal_rt, write_signal_rt};

/// A lightweight, `Copy`-able handle to a reactive value stored in the
/// thread-local runtime.  Reading a signal inside an effect automatically
/// subscribes the effect to future changes.
pub struct Signal<T: 'static> {
    id: usize,
    _marker: PhantomData<T>,
}

impl<T: 'static> Copy for Signal<T> {}
impl<T: 'static> Clone for Signal<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: 'static + Clone> Signal<T> {
    /// Create a new signal with the given initial value.
    pub fn new(value: T) -> Self {
        Signal {
            id: create_signal_rt(value),
            _marker: PhantomData,
        }
    }

    /// Read the current value.  If called inside an effect, the effect is
    /// automatically subscribed to this signal.
    pub fn get(&self) -> T {
        read_signal_rt(self.id)
    }

    /// Replace the value and notify all subscribers.
    pub fn set(&self, value: T) {
        write_signal_rt(self.id, value);
    }

    /// Mutate the value in-place and notify all subscribers.
    pub fn update(&self, f: impl FnOnce(&mut T)) {
        update_signal_rt(self.id, f);
    }

    /// Runtime-internal ID (used by generated macro code).
    #[doc(hidden)]
    pub fn id(&self) -> usize {
        self.id
    }
}

impl<T: 'static + Clone + fmt::Display> fmt::Display for Signal<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get())
    }
}

// Arithmetic convenience impls for numeric signals.

impl<T> Add<T> for Signal<T>
where
    T: 'static + Clone + Add<Output = T>,
{
    type Output = T;
    fn add(self, rhs: T) -> T {
        self.get() + rhs
    }
}

impl<T> Sub<T> for Signal<T>
where
    T: 'static + Clone + Sub<Output = T>,
{
    type Output = T;
    fn sub(self, rhs: T) -> T {
        self.get() - rhs
    }
}

impl<T> AddAssign<T> for Signal<T>
where
    T: 'static + Clone + Add<Output = T>,
{
    fn add_assign(&mut self, rhs: T) {
        let new_val = self.get() + rhs;
        self.set(new_val);
    }
}

impl<T> SubAssign<T> for Signal<T>
where
    T: 'static + Clone + Sub<Output = T>,
{
    fn sub_assign(&mut self, rhs: T) {
        let new_val = self.get() - rhs;
        self.set(new_val);
    }
}

/// Convenience constructor — `signal(0)` is equivalent to `Signal::new(0)`.
pub fn signal<T: 'static + Clone>(value: T) -> Signal<T> {
    Signal::new(value)
}
