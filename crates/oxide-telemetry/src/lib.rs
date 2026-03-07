//! # oxide-telemetry
//!
//! OpenTelemetry-compatible tracing for Oxide WASM applications.
//!
//! ```ignore
//! oxide_telemetry::init(Config {
//!     service_name: "my-app",
//!     endpoint: None, // console-only
//! });
//! // All signal/effect activity is now traced automatically.
//! ```

use oxide_core::{set_hook, HookEvent};
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

// ═══════════════════════════════════════════════════════════════════════════
// Public API
// ═══════════════════════════════════════════════════════════════════════════

/// Telemetry configuration.
pub struct Config {
    /// Service name reported in traces.
    pub service_name: &'static str,
    /// OTLP/HTTP endpoint. `None` = console-only mode.
    pub endpoint: Option<&'static str>,
    /// Whether to auto-trace signal reads (high volume — off by default).
    pub trace_reads: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            service_name: "oxide-app",
            endpoint: None,
            trace_reads: false,
        }
    }
}

/// Initialize telemetry. Installs runtime hooks for automatic tracing.
pub fn init(config: Config) {
    TELEMETRY.with(|t| {
        let mut tel = t.borrow_mut();
        tel.service_name = config.service_name;
        tel.endpoint = config.endpoint;
        tel.trace_reads = config.trace_reads;
    });
    set_hook(telemetry_hook);
}

/// Get the current collected spans (for display/debugging).
pub fn get_spans() -> Vec<SpanRecord> {
    TELEMETRY.with(|t| t.borrow().spans.clone())
}

/// Clear all collected spans.
pub fn clear_spans() {
    TELEMETRY.with(|t| t.borrow_mut().spans.clear());
}

/// Get the total count of events observed.
pub fn get_stats() -> Stats {
    TELEMETRY.with(|t| t.borrow().stats.clone())
}

/// Create a manual span for custom instrumentation.
pub fn span(name: &str) -> SpanGuard {
    let trace_id = TELEMETRY.with(|t| t.borrow().current_trace_id());
    let span_id = random_id_8();
    SpanGuard {
        name: name.to_string(),
        trace_id,
        span_id,
        start: now(),
        attributes: Vec::new(),
    }
}

/// Perform a fetch with W3C trace context propagation.
pub async fn traced_fetch(url: &str) -> Result<String, JsValue> {
    let _span = span("http.fetch");
    let trace_id = TELEMETRY.with(|t| t.borrow().current_trace_id());
    let span_id = random_id_8();
    let traceparent = format!("00-{}-{}-01", trace_id, span_id);

    let window = web_sys::window().unwrap();
    let opts = web_sys::RequestInit::new();
    opts.set_method("GET");
    let request = web_sys::Request::new_with_str_and_init(url, &opts)?;
    request.headers().set("traceparent", &traceparent).ok();

    let resp = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: web_sys::Response = resp.dyn_into()?;
    let text = wasm_bindgen_futures::JsFuture::from(resp.text()?).await?;

    TELEMETRY.with(|t| {
        t.borrow_mut().spans.push(SpanRecord {
            name: format!("HTTP GET {}", url),
            trace_id: trace_id.clone(),
            span_id,
            duration_ms: now() - _span.start,
            kind: SpanKind::Client,
            attributes: vec![
                ("http.method".into(), "GET".into()),
                ("http.url".into(), url.into()),
                ("http.status".into(), format!("{}", resp.status())),
            ],
        });
    });

    Ok(text.as_string().unwrap_or_default())
}

// ═══════════════════════════════════════════════════════════════════════════
// Types
// ═══════════════════════════════════════════════════════════════════════════

/// A recorded span.
#[derive(Debug, Clone)]
pub struct SpanRecord {
    pub name: String,
    pub trace_id: String,
    pub span_id: String,
    pub duration_ms: f64,
    pub kind: SpanKind,
    pub attributes: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
pub enum SpanKind {
    Internal,
    Client,
}

/// Cumulative statistics.
#[derive(Debug, Clone, Default)]
pub struct Stats {
    pub signals_created: u64,
    pub signal_reads: u64,
    pub signal_writes: u64,
    pub effects_run: u64,
    pub total_effect_time_ms: f64,
}

/// RAII span guard — records the span when dropped.
pub struct SpanGuard {
    name: String,
    trace_id: String,
    span_id: String,
    start: f64,
    attributes: Vec<(String, String)>,
}

impl SpanGuard {
    pub fn set_attribute(&mut self, key: &str, value: &str) {
        self.attributes.push((key.into(), value.into()));
    }
}

impl Drop for SpanGuard {
    fn drop(&mut self) {
        let record = SpanRecord {
            name: std::mem::take(&mut self.name),
            trace_id: std::mem::take(&mut self.trace_id),
            span_id: std::mem::take(&mut self.span_id),
            duration_ms: now() - self.start,
            kind: SpanKind::Internal,
            attributes: std::mem::take(&mut self.attributes),
        };
        TELEMETRY.with(|t| t.borrow_mut().spans.push(record));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Internal state
// ═══════════════════════════════════════════════════════════════════════════

struct Telemetry {
    service_name: &'static str,
    endpoint: Option<&'static str>,
    trace_reads: bool,
    spans: Vec<SpanRecord>,
    stats: Stats,
}

impl Telemetry {
    fn current_trace_id(&self) -> String {
        // Single trace per session for simplicity
        random_id_16()
    }
}

thread_local! {
    static TELEMETRY: RefCell<Telemetry> = RefCell::new(Telemetry {
        service_name: "oxide-app",
        endpoint: None,
        trace_reads: false,
        spans: Vec::new(),
        stats: Stats::default(),
    });
}

fn telemetry_hook(event: HookEvent) {
    TELEMETRY.with(|t| {
        let mut tel = t.borrow_mut();
        match event {
            HookEvent::SignalCreate { id } => {
                tel.stats.signals_created += 1;
                tel.spans.push(SpanRecord {
                    name: format!("signal.create[{}]", id),
                    trace_id: String::new(),
                    span_id: random_id_8(),
                    duration_ms: 0.0,
                    kind: SpanKind::Internal,
                    attributes: vec![("signal.id".into(), id.to_string())],
                });
            }
            HookEvent::SignalRead { id } => {
                tel.stats.signal_reads += 1;
                if tel.trace_reads {
                    tel.spans.push(SpanRecord {
                        name: format!("signal.read[{}]", id),
                        trace_id: String::new(),
                        span_id: random_id_8(),
                        duration_ms: 0.0,
                        kind: SpanKind::Internal,
                        attributes: vec![],
                    });
                }
            }
            HookEvent::SignalWrite { id } => {
                tel.stats.signal_writes += 1;
                tel.spans.push(SpanRecord {
                    name: format!("signal.write[{}]", id),
                    trace_id: String::new(),
                    span_id: random_id_8(),
                    duration_ms: 0.0,
                    kind: SpanKind::Internal,
                    attributes: vec![("signal.id".into(), id.to_string())],
                });
            }
            HookEvent::EffectRun { .. } => {
                tel.stats.effects_run += 1;
            }
            HookEvent::EffectComplete { id, duration_ms } => {
                tel.stats.total_effect_time_ms += duration_ms;
                if duration_ms > 0.01 {
                    tel.spans.push(SpanRecord {
                        name: format!("effect.run[{}]", id),
                        trace_id: String::new(),
                        span_id: random_id_8(),
                        duration_ms,
                        kind: SpanKind::Internal,
                        attributes: vec![
                            ("effect.id".into(), id.to_string()),
                            ("duration_ms".into(), format!("{:.3}", duration_ms)),
                        ],
                    });
                }
            }
            HookEvent::BatchStart => {
                tel.spans.push(SpanRecord {
                    name: "batch.start".into(),
                    trace_id: String::new(),
                    span_id: random_id_8(),
                    duration_ms: 0.0,
                    kind: SpanKind::Internal,
                    attributes: vec![],
                });
            }
            HookEvent::BatchEnd { effect_count } => {
                tel.spans.push(SpanRecord {
                    name: "batch.end".into(),
                    trace_id: String::new(),
                    span_id: random_id_8(),
                    duration_ms: 0.0,
                    kind: SpanKind::Internal,
                    attributes: vec![("effects".into(), effect_count.to_string())],
                });
            }
        }

        // Keep span buffer bounded
        if tel.spans.len() > 500 {
            tel.spans.drain(0..250);
        }
    });
}

// ═══════════════════════════════════════════════════════════════════════════
// Utilities
// ═══════════════════════════════════════════════════════════════════════════

fn now() -> f64 {
    js_sys::Date::now()
}

fn random_id_8() -> String {
    format!("{:016x}", (js_sys::Math::random() * u64::MAX as f64) as u64)
}

fn random_id_16() -> String {
    format!(
        "{:016x}{:016x}",
        (js_sys::Math::random() * u64::MAX as f64) as u64,
        (js_sys::Math::random() * u64::MAX as f64) as u64,
    )
}
