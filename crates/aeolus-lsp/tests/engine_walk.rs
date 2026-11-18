//! The engine's walk-face ride (the compile-walk observability seam's
//! pooled consumer): a host building the engine wires ONE
//! [`WalkObserver`](aeolus_lsp::WalkObserver) handle and the session's
//! lowerings stream their stage events — the same vocabulary the
//! compile crate's free `compile_observed` reports, through the SAME
//! bounded pool the per-keystroke answers ride. The cache face
//! (`engine_cache.rs`) answers "what did the pool decide"; this face
//! answers "what did the walk do when the pool missed".
//!
//! The pins:
//!
//! * `with_walk_observer` chains after `new` (the `with_observer`
//!   shape) — the first analyze is a miss set, one walk event per
//!   lowered string;
//! * the keystroke repeat rides the pool — the second analyze of the
//!   same document streams NOTHING new (the pool answered; the walk
//!   never ran);
//! * a change misses only its new strings — the kept strings hit, the
//!   swapped spelling walks;
//! * the observation-free default answers unchanged, and the face
//!   never changes a decision — the analyzed diagnostics are the
//!   plain engine's own.

use std::sync::Arc;

use aeolus_lsp::{AeolusEngine, EngineConfig, WalkSnapshot, WalkStats};

/// A synthetic .rs document the way `tw!` call sites look: two lowered
/// strings, one unknown root — three ScanHits per analyze.
const DOC: &str = "fn card() {\n    let a = \"p-4 flex-col gap-2 rounded-md\";\n    let b = \"hover:bg-primary/90 dark:shadow-lg\";\n    let bad = \"not-a-root\";\n}\n";

/// The changed document: string `a` rides unchanged, `b` swaps for a
/// new spelling, the refused root stays.
const DOC_CHANGED: &str = "fn card() {\n    let a = \"p-4 flex-col gap-2 rounded-md\";\n    let b = \"hover:bg-primary/50 md:shadow-lg\";\n    let bad = \"not-a-root\";\n}\n";

#[test]
fn the_engine_streams_the_walk_face_on_the_miss_set() {
    let stats = Arc::new(WalkStats::new());
    let mut lsp = AeolusEngine::new(EngineConfig::default()).with_walk_observer(Arc::clone(&stats));
    lsp.open("file:///card.rs", DOC, 1);

    let diagnostics = lsp.analyze("file:///card.rs");
    let snap: WalkSnapshot = stats.snapshot();
    assert_eq!(
        snap.walks, 3,
        "three ScanHits, all first-compiles — one observed walk per string: \
         {snap:?}"
    );
    assert!(
        snap.candidates >= 6,
        "the per-candidate events streamed (three candidates per string, \
         the refused root among them): {snap:?}"
    );
    assert!(
        snap.missed >= 1,
        "`not-a-root` is a walk that misses, not an error — the verdict \
         rides the Missed arm: {snap:?}"
    );
    let _ = diagnostics.len(); // the answer shape is the walk's byproduct
}

#[test]
fn the_keystroke_repeat_streams_nothing_new() {
    let stats = Arc::new(WalkStats::new());
    let mut lsp = AeolusEngine::new(EngineConfig::default()).with_walk_observer(Arc::clone(&stats));
    lsp.open("file:///card.rs", DOC, 1);

    let _ = lsp.analyze("file:///card.rs");
    let after_first = stats.snapshot();
    let _ = lsp.analyze("file:///card.rs");

    assert_eq!(
        stats.snapshot(),
        after_first,
        "the repeat analyze rode the pool — no walk ran, no walk events"
    );
}

#[test]
fn a_change_walks_only_its_new_strings() {
    let stats = Arc::new(WalkStats::new());
    let mut lsp = AeolusEngine::new(EngineConfig::default()).with_walk_observer(Arc::clone(&stats));
    lsp.open("file:///card.rs", DOC, 1);
    let _ = lsp.analyze("file:///card.rs");
    let after_first = stats.snapshot();

    lsp.change("file:///card.rs", DOC_CHANGED, 2);
    let _ = lsp.analyze("file:///card.rs");

    let snap = stats.snapshot();
    assert_eq!(
        snap.walks - after_first.walks,
        1,
        "the kept strings hit; only the swapped spelling lowers — one new \
         walk event set per change"
    );
}

#[test]
fn the_walk_face_never_changes_a_decision() {
    let mut observed =
        AeolusEngine::new(EngineConfig::default()).with_walk_observer(WalkStats::new());
    observed.open("file:///card.rs", DOC, 1);
    let mut plain = AeolusEngine::new(EngineConfig::default());
    plain.open("file:///card.rs", DOC, 1);

    assert_eq!(
        observed.analyze("file:///card.rs"),
        plain.analyze("file:///card.rs"),
        "the face adds a channel, never changes an answer — the analyzed \
         diagnostics are the plain engine's own"
    );
}
