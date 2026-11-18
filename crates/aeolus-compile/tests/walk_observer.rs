//! The compile-walk observability seam's contract tests (TDD: written
//! FIRST — the RED capture is the "no `WalkObserver` trait / no
//! `compile_observed` entry" compile failure, recorded in
//! `reports/10-tdd-evidence/41-walk-observer.md`).
//!
//! The seam mirrors the dyn-cache observability face exactly: a
//! default-no-op observer trait over the walk's own stage boundaries
//! (lex → per-candidate fold/read → assemble), a `NoWalkObserver`
//! constructor default, the `Arc` blanket forwarder, and a
//! `WalkStats` collector whose snapshot the tests assert on. The pins
//! are the DynObserver suite's own: the event sequence against a known
//! string, the collector's totals, the no-observer byte-neutrality
//! (the differential contract — the seam adds a channel, never
//! changes a decision), and the shared-handle shape.

use std::sync::Arc;

use aeolus_compile::{
    CandidateOutcome, Config, NoWalkObserver, WalkObserver, WalkStats, compile, compile_observed,
};
use aeolus_theme::{ThemeSource, TokenTable};

/// The battery's input: every event at least once — three lowered
/// candidates (four + one + one declarations), one unknown root (the
/// typed miss), one degraded token (the lexer's typed per-token
/// failure), the base condition only (one layer).
const MIXED: &str = "p-4 flex z-10 not-a-root-xyz :broken";

#[test]
fn the_collector_totals_the_walk_stages() {
    let stats = Arc::new(WalkStats::new());
    compile_observed(
        MIXED,
        None,
        &aeolus_compile::Customs::new(),
        &Config::default(),
        stats.clone() as Arc<dyn WalkObserver>,
    )
    .expect("non-empty input compiles");
    let snap = stats.snapshot();
    assert_eq!(snap.walks, 1, "one string, one walk");
    assert_eq!(
        snap.candidates, 4,
        "four candidates; the broken token never lexes"
    );
    assert_eq!(snap.degraded, 1, "the `:broken` head is the typed lex miss");
    assert_eq!(snap.lowered, 3, "p-4, flex, z-10 lower");
    assert_eq!(snap.missed, 1, "the unknown root reports typed");
    assert_eq!(snap.decls, 6, "p-4 fans four, flex one, z-10 one");
    assert_eq!(snap.diagnostics, 2, "the unknown root + the degraded token");
    assert_eq!(snap.layers, 1, "every candidate rides the base condition");
}

#[test]
fn the_per_candidate_events_carry_the_outcome_split() {
    // The events the collector totals, spelled per candidate: a custom
    // observer records the (raw, outcome) pairs in walk order — the
    // trait's own contract the snapshot's counters summarize.
    #[derive(Default)]
    struct Recorder {
        events: std::sync::Mutex<Vec<(String, &'static str)>>,
    }
    impl WalkObserver for Recorder {
        fn on_candidate(&self, raw: &str, outcome: CandidateOutcome) {
            let word = match outcome {
                CandidateOutcome::Lowered { .. } => "lowered",
                CandidateOutcome::Missed { .. } => "missed",
            };
            self.events
                .lock()
                .expect("the lock holds")
                .push((raw.to_string(), word));
        }
    }
    let recorder = Arc::new(Recorder::default());
    compile_observed(
        "p-4 hover:p-8 not-a-root-xyz",
        None,
        &aeolus_compile::Customs::new(),
        &Config::default(),
        recorder.clone() as Arc<dyn WalkObserver>,
    )
    .expect("non-empty input compiles");
    let events = recorder.events.lock().expect("the lock holds");
    assert_eq!(
        events
            .iter()
            .map(|(raw, word)| (raw.as_str(), *word))
            .collect::<Vec<_>>(),
        vec![
            ("p-4", "lowered"),
            ("hover:p-8", "lowered"),
            ("not-a-root-xyz", "missed"),
        ],
        "walk order, the fold/read verdict per candidate"
    );
}

#[test]
fn the_multi_layer_walk_answers_the_assembly_count() {
    let stats = Arc::new(WalkStats::new());
    compile_observed(
        "md:p-8 hover:p-4 p-2",
        None,
        &aeolus_compile::Customs::new(),
        &Config::default(),
        stats.clone() as Arc<dyn WalkObserver>,
    )
    .expect("non-empty input compiles");
    let snap = stats.snapshot();
    assert_eq!(
        snap.layers, 3,
        "base, hover, md — the specificity sort's own slots"
    );
    assert_eq!(snap.candidates, 3);
    assert_eq!(snap.lowered, 3);
}

#[test]
fn the_themed_walk_observese_the_same_stages() {
    let source = ThemeSource::shadcn().expect("embedded palettes parse");
    let tokens = TokenTable::runtime(&source).expect("default-seeded source builds");
    let stats = Arc::new(WalkStats::new());
    compile_observed(
        "bg-primary p-4",
        Some(&tokens),
        &aeolus_compile::Customs::new(),
        &Config::default(),
        stats.clone() as Arc<dyn WalkObserver>,
    )
    .expect("non-empty input compiles");
    let snap = stats.snapshot();
    assert_eq!(snap.candidates, 2);
    assert_eq!(
        snap.lowered, 2,
        "the token slot lowers through the themed walk"
    );
    assert_eq!(snap.diagnostics, 0);
}

#[test]
fn the_no_observer_compiles_byte_identically() {
    // The differential contract: the seam adds a channel, never
    // changes a decision — the observed walk's artifact is the plain
    // compile's artifact, byte for byte (the program's Debug and the
    // diagnostics' set).
    let plain = compile(MIXED, &Config::default()).expect("non-empty input compiles");
    let silent = compile_observed(
        MIXED,
        None,
        &aeolus_compile::Customs::new(),
        &Config::default(),
        Arc::new(NoWalkObserver) as Arc<dyn WalkObserver>,
    )
    .expect("non-empty input compiles");
    assert_eq!(plain.program, silent.program, "the layers are identical");
    assert_eq!(
        plain.diagnostics.len(),
        silent.diagnostics.len(),
        "the typed reports are identical"
    );
    for (a, b) in plain.diagnostics.iter().zip(&silent.diagnostics) {
        assert_eq!(a.raw, b.raw);
        assert_eq!(a.kind, b.kind);
    }
}

#[test]
fn the_stats_handle_shares_across_walks() {
    // The CacheStats shape: one handle totals several walks (the
    // session's own budget story), the snapshot Copy.
    let stats = Arc::new(WalkStats::new());
    for input in ["p-4", "flex", "z-10"] {
        compile_observed(
            input,
            None,
            &aeolus_compile::Customs::new(),
            &Config::default(),
            stats.clone() as Arc<dyn WalkObserver>,
        )
        .expect("non-empty input compiles");
    }
    let snap = stats.snapshot();
    assert_eq!(snap.walks, 3);
    assert_eq!(snap.candidates, 3);
    assert_eq!(snap.lowered, 3);
    assert_eq!(snap.decls, 6, "p-4 four + flex one + z-10 one");
}
