//! The compile-walk observability seam's microbenchmarks (divan,
//! stable — the workspace bench idiom): the dyn-cache observer seam's
//! own twin, pointed at the walk. Three arms pin the cost split: the
//! observation-free baseline is the plain `compile` (the seam's
//! byte-neutral contract — no observer, no hooks), the built-in
//! counter's walk rides `WalkStats`' atomic adds at the stage hooks,
//! and the worst-case recording observer (a `Mutex<Vec<_>>` push per
//! event) bounds the channel a host could pay for.

use std::sync::{Arc, Mutex};

use aeolus_compile::{CandidateOutcome, Config, WalkObserver, compile, compile_observed};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The walked input: the same shadcn v4 Button slice the CLI pipeline
/// benchmarks key on — every stage busy (the variant folds, the
/// registry fan-out, the cascade sort).
const BUTTON: &str = "inline-flex shrink-0 items-center justify-center gap-2 rounded-md text-sm font-medium whitespace-nowrap outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50 bg-primary text-primary-foreground hover:bg-primary/90 h-9 px-4 py-2";

#[divan::bench]
fn walk_unobserved(bencher: Bencher) {
    // The baseline: the plain entry point, no observer — the seam's
    // `None` arm IS this path (the differential pins the bytes).
    bencher.bench(|| compile(BUTTON, &Config::default()).expect("non-empty input compiles"));
}

#[divan::bench]
fn walk_counted(bencher: Bencher) {
    let stats = Arc::new(aeolus_compile::WalkStats::new());
    bencher.bench(|| {
        compile_observed(
            BUTTON,
            None,
            &aeolus_compile::Customs::new(),
            &Config::default(),
            stats.clone() as Arc<dyn WalkObserver>,
        )
        .expect("non-empty input compiles")
    });
}

/// The worst-case host channel: one mutex-guarded push per event (the
/// per-candidate verdicts dominate — the stage totals ride twice per
/// walk).
struct Recorder {
    events: Mutex<Vec<(String, usize, usize)>>,
}

impl WalkObserver for Recorder {
    fn on_candidate(&self, raw: &str, outcome: CandidateOutcome) {
        let (decls, diagnostics) = match outcome {
            CandidateOutcome::Lowered { decls, diagnostics } => (decls, diagnostics),
            CandidateOutcome::Missed { diagnostics } => (0, diagnostics),
        };
        self.events.lock().expect("the bench never poisons").push((
            raw.to_string(),
            decls,
            diagnostics,
        ));
    }
}

#[divan::bench]
fn walk_recorded(bencher: Bencher) {
    let recorder = Arc::new(Recorder {
        events: Mutex::new(Vec::new()),
    });
    bencher.bench(|| {
        compile_observed(
            BUTTON,
            None,
            &aeolus_compile::Customs::new(),
            &Config::default(),
            recorder.clone() as Arc<dyn WalkObserver>,
        )
        .expect("non-empty input compiles")
    });
}
