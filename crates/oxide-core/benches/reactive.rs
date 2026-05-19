//! Criterion micro-benchmarks for the bueler-core reactive primitives.
//!
//! Run with `cargo bench -p bueler-core` (native target only — criterion
//! does not support `wasm32`). Results are also collected by the weekly
//! `benchmarks.yml` workflow which appends a summary entry to
//! `examples/showcase/benchmarks.json` for display on the docs site.
//!
//! Bench groups intentionally cover the primitives we claim are fast on
//! the marketing pages:
//!
//! * `signal/create`, `signal/read`, `signal/write` — raw signal hot path
//! * `effect/run`                                   — effect invocation cost
//! * `memo/recompute`                               — memoized derivation
//! * `batch/coalesce`                               — batched writes
//! * `watch/trigger`                                — change subscription

use std::time::Duration;

use bueler_core::{batch, create_effect, memo, signal, watch, Signal};
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};

fn bench_signal_create(c: &mut Criterion) {
    let mut group = c.benchmark_group("signal");
    group.bench_function("create", |b| {
        b.iter(|| {
            let s = signal(black_box(0_u64));
            black_box(s);
        });
    });
    group.finish();
}

fn bench_signal_read(c: &mut Criterion) {
    let mut group = c.benchmark_group("signal");
    let s = signal(42_u64);
    group.throughput(Throughput::Elements(1));
    group.bench_function("read", |b| {
        b.iter(|| {
            black_box(s.get());
        });
    });
    group.finish();
}

fn bench_signal_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("signal");
    let s = signal(0_u64);
    let mut i = 0_u64;
    group.throughput(Throughput::Elements(1));
    group.bench_function("write", |b| {
        b.iter(|| {
            i = i.wrapping_add(1);
            s.set(black_box(i));
        });
    });
    group.finish();
}

fn bench_effect_run(c: &mut Criterion) {
    let mut group = c.benchmark_group("effect");
    let src = signal(0_u64);
    let sink = signal(0_u64);
    create_effect(move || {
        let v = src.get();
        sink.set(v.wrapping_add(1));
    });
    let mut i = 0_u64;
    group.bench_function("run", |b| {
        b.iter(|| {
            i = i.wrapping_add(1);
            src.set(black_box(i));
        });
    });
    group.finish();
}

fn bench_memo_recompute(c: &mut Criterion) {
    let mut group = c.benchmark_group("memo");
    let a = signal(1_u64);
    let b = signal(2_u64);
    let m: Signal<u64> = memo(move || a.get().wrapping_mul(b.get()));
    let mut i = 0_u64;
    group.bench_function("recompute", |bench| {
        bench.iter(|| {
            i = i.wrapping_add(1);
            a.set(black_box(i));
            black_box(m.get());
        });
    });
    group.finish();
}

fn bench_batch_coalesce(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch");
    let s = signal(0_u64);
    let mut runs = 0_u64;
    create_effect(move || {
        let _ = s.get();
        runs = runs.wrapping_add(1);
    });
    let mut i = 0_u64;
    group.bench_function("coalesce", |bench| {
        bench.iter(|| {
            batch(|| {
                for _ in 0..10 {
                    i = i.wrapping_add(1);
                    s.set(black_box(i));
                }
            });
        });
    });
    group.finish();
}

fn bench_watch_trigger(c: &mut Criterion) {
    let mut group = c.benchmark_group("watch");
    let src = signal(0_u64);
    let sink = signal(0_u64);
    watch(move || src.get(), move |v| sink.set(v));
    let mut i = 0_u64;
    group.bench_function("trigger", |bench| {
        bench.iter(|| {
            i = i.wrapping_add(1);
            src.set(black_box(i));
        });
    });
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_millis(300))
        .measurement_time(Duration::from_secs(2))
        .sample_size(50);
    targets =
        bench_signal_create,
        bench_signal_read,
        bench_signal_write,
        bench_effect_run,
        bench_memo_recompute,
        bench_batch_coalesce,
        bench_watch_trigger,
}
criterion_main!(benches);
