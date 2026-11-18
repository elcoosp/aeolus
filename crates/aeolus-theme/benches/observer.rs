//! The theme-resolution observability microbenchmarks (divan, stable —
//! the workspace bench idiom): the face's own cost, three arms per
//! channel — the unobserved baseline (the plain `resolve` the table
//! always served), the built-in `ThemeStats` counter (the atomic
//! totals) and a worst-case recording observer (the event payload
//! built and pushed per consult). The seam is answer-neutral by
//! contract; these numbers are the price of the channel when it is
//! wired and the proof it is free when it is not.
//!
//! Numbers land in `reports/08-benchmarks/BENCHMARKS.md` when the gate
//! environment runs them (see `reports/10-tdd-evidence/42-theme-observer.md`
//! for the method note).

use std::sync::{Arc, Mutex};

use aeolus_theme::{ResolveOutcome, ThemeObserver, ThemeSource, ThemeStats, TokenTable};
use divan::Bencher;

fn main() {
    divan::main();
}

/// A worst-case host observer: every consult builds and stores its
/// event line — the upper bound a telemetry host can pay.
struct Recorder {
    events: Mutex<Vec<String>>,
}

impl ThemeObserver for Recorder {
    fn on_resolve(&self, id: u16, alpha: Option<f32>, outcome: ResolveOutcome) {
        self.events
            .lock()
            .expect("the bench never poisons")
            .push(format!("resolve {id} {alpha:?} {outcome:?}"));
    }
}

/// The runtime tier's hot resolve, unobserved — the T2 baseline.
#[divan::bench]
fn resolve_unobserved(bencher: Bencher) {
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    let table = TokenTable::runtime(&source).expect("builds");
    let id = table.id("primary").expect("primary is a shadcn token");
    bencher.bench(|| table.resolve(id, Some(0.9)).expect("resolves"));
}

/// The runtime tier's hot resolve through the built-in counter — the
/// wired telemetry a host ships with.
#[divan::bench]
fn resolve_counted(bencher: Bencher) {
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    let table = TokenTable::runtime(&source)
        .expect("builds")
        .with_observer(ThemeStats::new());
    let id = table.id("primary").expect("primary is a shadcn token");
    bencher.bench(|| table.resolve(id, Some(0.9)).expect("resolves"));
}

/// The runtime tier's hot resolve through a worst-case recorder — the
/// channel's upper bound.
#[divan::bench]
fn resolve_recorded(bencher: Bencher) {
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    let table = TokenTable::runtime(&source)
        .expect("builds")
        .with_observer(Recorder {
            events: Mutex::new(Vec::new()),
        });
    let id = table.id("primary").expect("primary is a shadcn token");
    bencher.bench(|| table.resolve(id, Some(0.9)).expect("resolves"));
}

/// The baked tier's hot resolve (the pair-index form), unobserved.
#[divan::bench]
fn resolve_baked_unobserved(bencher: Bencher) {
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    let table = TokenTable::baked(&source).expect("builds");
    let id = table.id("primary").expect("primary is a shadcn token");
    bencher.bench(|| table.resolve(id, None).expect("resolves"));
}

/// The baked tier's hot resolve through the built-in counter.
#[divan::bench]
fn resolve_baked_counted(bencher: Bencher) {
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    let table = TokenTable::baked(&source)
        .expect("builds")
        .with_observer(ThemeStats::new());
    let id = table.id("primary").expect("primary is a shadcn token");
    bencher.bench(|| table.resolve(id, None).expect("resolves"));
}

/// The rgba channel's runtime resolve (the mode half), counted — the
/// face's second channel rides the same slot.
#[divan::bench]
fn resolve_rgba_counted(bencher: Bencher) {
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    let table = TokenTable::runtime(&source)
        .expect("builds")
        .with_observer(ThemeStats::new());
    let id = table.id("primary").expect("primary is a shadcn token");
    bencher.bench(|| table.resolve_rgba(id, 0.9, true).expect("resolves"));
}

/// The session snapshot itself: the `Copy` value a host reads between
/// walks (nine relaxed loads, no locks).
#[divan::bench]
fn theme_stats_snapshot(bencher: Bencher) {
    let stats = Arc::new(ThemeStats::new());
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    let table = TokenTable::runtime(&source)
        .expect("builds")
        .with_observer(Arc::clone(&stats));
    let id = table.id("primary").expect("primary is a shadcn token");
    let _ = table.resolve(id, None);
    bencher.bench(|| stats.snapshot());
}
