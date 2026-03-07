//! # oxide-resiliency
//!
//! Production-grade resiliency patterns for Oxide applications.
//!
//! - [`error_boundary`] — Catch panics and render fallback UI
//! - [`retry`] — Retry failed operations with exponential backoff
//! - [`CircuitBreaker`] — Stop calling failing services
//! - [`with_timeout`] — Abort operations that take too long
//! - [`sleep`] — Async-friendly sleep for WASM

use oxide_core::signal;
use oxide_core::Signal;
use oxide_dom::{create_element, append_text, set_style};
use wasm_bindgen::JsCast;
use std::future::Future;
use std::pin::Pin;

// ═══════════════════════════════════════════════════════════════════════════
// Error Boundary
// ═══════════════════════════════════════════════════════════════════════════

/// Catch panics from `builder` and render `fallback` UI instead of crashing.
///
/// ```ignore
/// error_boundary(
///     || risky_component(),
///     |err| view! { <p>"Error: " {err}</p> },
/// )
/// ```
pub fn error_boundary(
    builder: impl FnOnce() -> web_sys::Element,
    fallback: impl FnOnce(String) -> web_sys::Element,
) -> web_sys::Element {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(builder)) {
        Ok(el) => el,
        Err(e) => {
            let msg = if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = e.downcast_ref::<&str>() {
                s.to_string()
            } else {
                "An unexpected error occurred".to_string()
            };
            fallback(msg)
        }
    }
}

/// Render a default styled error boundary (red error card).
pub fn default_error_boundary(builder: impl FnOnce() -> web_sys::Element) -> web_sys::Element {
    error_boundary(builder, |msg| {
        let card = create_element("div");
        set_style(&card, "padding", "1rem");
        set_style(&card, "border-radius", "8px");
        set_style(&card, "background", "rgba(239,68,68,0.1)");
        set_style(&card, "border", "1px solid #ef4444");
        set_style(&card, "color", "#fca5a5");
        set_style(&card, "font-size", "0.85rem");
        let title = create_element("strong");
        append_text(&title, "⚠️ Component Error");
        card.append_child(&title).ok();
        let br = create_element("br");
        card.append_child(&br).ok();
        let detail = create_element("code");
        set_style(&detail, "font-size", "0.75rem");
        set_style(&detail, "opacity", "0.8");
        append_text(&detail, &msg);
        card.append_child(&detail).ok();
        card
    })
}

// ═══════════════════════════════════════════════════════════════════════════
// Retry
// ═══════════════════════════════════════════════════════════════════════════

/// Configuration for retry with backoff.
#[derive(Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay_ms: u32,
    pub backoff_factor: f64,
    pub max_delay_ms: u32,
}

impl RetryConfig {
    /// Exponential backoff: delay doubles each attempt.
    pub fn exponential(max_attempts: u32, initial_delay_ms: u32) -> Self {
        Self {
            max_attempts,
            initial_delay_ms,
            backoff_factor: 2.0,
            max_delay_ms: 30_000,
        }
    }

    /// Fixed delay between attempts.
    pub fn fixed(max_attempts: u32, delay_ms: u32) -> Self {
        Self {
            max_attempts,
            initial_delay_ms: delay_ms,
            backoff_factor: 1.0,
            max_delay_ms: delay_ms,
        }
    }
}

/// Retry an async operation with the given backoff strategy.
///
/// Returns the first successful result, or the last error after exhausting attempts.
pub async fn retry<T, E, F, Fut>(config: RetryConfig, f: F) -> Result<T, RetryError<E>>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    let mut delay = config.initial_delay_ms;
    let mut last_err = None;

    for attempt in 0..config.max_attempts {
        match f().await {
            Ok(v) => return Ok(v),
            Err(e) => {
                last_err = Some(e);
                if attempt < config.max_attempts - 1 {
                    sleep(delay).await;
                    delay = ((delay as f64) * config.backoff_factor)
                        .min(config.max_delay_ms as f64) as u32;
                }
            }
        }
    }

    Err(RetryError {
        attempts: config.max_attempts,
        last_error: last_err.unwrap(),
    })
}

/// Error returned when all retry attempts are exhausted.
pub struct RetryError<E> {
    pub attempts: u32,
    pub last_error: E,
}

impl<E: std::fmt::Debug> std::fmt::Display for RetryError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "all {} retry attempts failed: {:?}", self.attempts, self.last_error)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Circuit Breaker
// ═══════════════════════════════════════════════════════════════════════════

/// Circuit breaker states.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitState {
    /// Normal operation — requests pass through.
    Closed,
    /// Too many failures — requests are rejected immediately.
    Open,
    /// Testing recovery — one request allowed through.
    HalfOpen,
}

impl std::fmt::Display for CircuitState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Closed => write!(f, "Closed"),
            Self::Open => write!(f, "Open"),
            Self::HalfOpen => write!(f, "Half-Open"),
        }
    }
}

/// Circuit breaker configuration.
pub struct CircuitBreakerConfig {
    /// Number of consecutive failures before opening the circuit.
    pub failure_threshold: u32,
    /// Time in ms before transitioning from Open → HalfOpen.
    pub reset_timeout_ms: u32,
}

/// A circuit breaker that prevents cascading failures.
///
/// ```ignore
/// let breaker = CircuitBreaker::new(CircuitBreakerConfig {
///     failure_threshold: 3,
///     reset_timeout_ms: 5000,
/// });
///
/// match breaker.call(|| fetch_data()).await {
///     Ok(data) => { /* use data */ }
///     Err(CircuitError::Open) => { /* circuit is open, show cached data */ }
///     Err(CircuitError::Failed(e)) => { /* request failed */ }
/// }
/// ```
pub struct CircuitBreaker {
    pub state: Signal<CircuitState>,
    pub failure_count: Signal<u32>,
    pub success_count: Signal<u32>,
    config: CircuitBreakerConfig,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            state: signal(CircuitState::Closed),
            failure_count: signal(0),
            success_count: signal(0),
            config,
        }
    }

    /// Execute an async operation through the circuit breaker.
    pub async fn call<T, E, F, Fut>(&self, f: F) -> Result<T, CircuitError<E>>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, E>>,
    {
        match self.state.get() {
            CircuitState::Open => return Err(CircuitError::Open),
            CircuitState::HalfOpen | CircuitState::Closed => {}
        }

        match f().await {
            Ok(val) => {
                self.failure_count.set(0);
                self.success_count.update(|c| *c += 1);
                self.state.set(CircuitState::Closed);
                Ok(val)
            }
            Err(e) => {
                let count = self.failure_count.get() + 1;
                self.failure_count.set(count);

                if count >= self.config.failure_threshold {
                    self.state.set(CircuitState::Open);
                    let state = self.state;
                    let timeout = self.config.reset_timeout_ms;
                    oxide_dom::set_timeout(
                        move || state.set(CircuitState::HalfOpen),
                        timeout as i32,
                    );
                }

                Err(CircuitError::Failed(e))
            }
        }
    }

    /// Reset the circuit breaker to its initial state.
    pub fn reset(&self) {
        self.state.set(CircuitState::Closed);
        self.failure_count.set(0);
    }
}

/// Error type for circuit breaker calls.
pub enum CircuitError<E> {
    /// The circuit is open — the request was not attempted.
    Open,
    /// The request was attempted but failed.
    Failed(E),
}

impl<E: std::fmt::Debug> std::fmt::Display for CircuitError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open => write!(f, "circuit breaker is open"),
            Self::Failed(e) => write!(f, "request failed: {:?}", e),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Timeout
// ═══════════════════════════════════════════════════════════════════════════

/// Wrap a future with a timeout. Returns `Err(TimeoutError)` if the
/// future doesn't complete within `ms` milliseconds.
pub async fn with_timeout<T: 'static>(
    ms: u32,
    future: impl Future<Output = T> + 'static,
) -> Result<T, TimeoutError> {
    let (sender, receiver) = futures_channel::<Option<T>>();

    let sender_ok = sender.clone();
    wasm_bindgen_futures::spawn_local(async move {
        let result = future.await;
        sender_ok(Some(result));
    });

    let timeout_id = {
        let sender_timeout = sender.clone();
        oxide_dom::set_timeout(move || { sender_timeout(None); }, ms as i32)
    };

    match receiver().await {
        Some(val) => {
            oxide_dom::clear_timeout(timeout_id);
            Ok(val)
        }
        None => Err(TimeoutError { ms }),
    }
}

/// A timeout occurred.
#[derive(Debug)]
pub struct TimeoutError {
    pub ms: u32,
}

impl std::fmt::Display for TimeoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "operation timed out after {}ms", self.ms)
    }
}

// Simple one-shot channel using closures (avoids adding a channels dependency)
fn futures_channel<T: 'static>() -> (impl Fn(T) + Clone + 'static, impl FnOnce() -> Pin<Box<dyn Future<Output = T>>>) {
    use std::cell::RefCell;
    use std::rc::Rc;

    let slot: Rc<RefCell<Option<T>>> = Rc::new(RefCell::new(None));
    let waker: Rc<RefCell<Option<std::task::Waker>>> = Rc::new(RefCell::new(None));

    let slot_s = slot.clone();
    let waker_s = waker.clone();
    let sender = move |val: T| {
        *slot_s.borrow_mut() = Some(val);
        if let Some(w) = waker_s.borrow_mut().take() {
            w.wake();
        }
    };

    let receiver = move || -> Pin<Box<dyn Future<Output = T>>> {
        Box::pin(std::future::poll_fn(move |cx| {
            if let Some(val) = slot.borrow_mut().take() {
                std::task::Poll::Ready(val)
            } else {
                *waker.borrow_mut() = Some(cx.waker().clone());
                std::task::Poll::Pending
            }
        }))
    };

    (sender, receiver)
}

// ═══════════════════════════════════════════════════════════════════════════
// Sleep
// ═══════════════════════════════════════════════════════════════════════════

/// Async-friendly sleep for WASM. Resolves after `ms` milliseconds.
pub async fn sleep(ms: u32) {
    let promise = js_sys::Promise::new(&mut |resolve, _| {
        web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                &resolve,
                ms as i32,
            )
            .ok();
    });
    wasm_bindgen_futures::JsFuture::from(promise).await.ok();
}
