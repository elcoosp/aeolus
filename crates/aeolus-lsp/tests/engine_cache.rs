//! The engine's dyn-cache ride (the P5 row's latency half): the
//! editor's per-keystroke answers dedupe through the bounded pool —
//! the SAME `DynCache` seam the compile crate documents — so a
//! re-analyzed document's unchanged strings ride the pool instead of a
//! fresh compile, and the observability face rides the engine's own
//! constructor.
//!
//! The pins, in the order the module docs promise them:
//!
//! * a keystroke that leaves the strings alone is a HIT set — the
//!   second analyze of the same document pools, never re-lowers;
//! * the cached answers stay byte-identical to the uncached walk (the
//!   differential contract — the cache never changes a decision);
//! * a change misses only its NEW strings (the kept ones hit);
//! * the cursor answers (hover) ride the pool too — every compile site
//!   shares the one cache;
//! * the observer face is the engine's surface: `with_observer` chains
//!   after `new`, the built-in `CacheStats` totals the session, and the
//!   observation-free default answers unchanged.

use std::sync::Arc;

use aeolus_lsp::{AeolusEngine, CacheSnapshot, CacheStats, EngineConfig};

/// A synthetic .rs document the way `tw!` call sites look: two lowered
/// strings, one unknown root — three ScanHits per analyze.
const DOC: &str = "fn card() {\n    let a = \"p-4 flex-col gap-2 rounded-md\";\n    let b = \"hover:bg-primary/90 dark:shadow-lg\";\n    let bad = \"not-a-root\";\n}\n";

/// The changed document: string `a` rides unchanged, `b` swaps for a
/// new spelling, the refused root stays — one new string per change.
const DOC_CHANGED: &str = "fn card() {\n    let a = \"p-4 flex-col gap-2 rounded-md\";\n    let b = \"hover:bg-primary/50 md:shadow-lg\";\n    let bad = \"not-a-root\";\n}\n";

#[test]
fn the_keystroke_repeat_rides_the_pool() {
    let stats = Arc::new(CacheStats::new());
    let mut lsp = AeolusEngine::new(EngineConfig::default()).with_observer(Arc::clone(&stats));
    lsp.open("file:///card.rs", DOC, 1);

    let first = lsp.analyze("file:///card.rs");
    let snapshot: CacheSnapshot = stats.snapshot();
    assert_eq!(
        snapshot,
        CacheSnapshot {
            hits: 0,
            misses: 3,
            stores: 3,
            evictions: 0,
        },
        "the first analyze lowers every ScanHit once (two lowered strings \
         + the refused root)"
    );

    let second = lsp.analyze("file:///card.rs");
    let snapshot = stats.snapshot();
    assert_eq!(
        snapshot,
        CacheSnapshot {
            hits: 3,
            misses: 3,
            stores: 3,
            evictions: 0,
        },
        "the re-analyze of an unchanged document rides the pool — three \
         hits, zero fresh lowers"
    );
    assert_eq!(first, second, "the cached answers are the same answers");
}

#[test]
fn the_cached_answers_match_the_uncached_walk() {
    let mut cached = AeolusEngine::new(EngineConfig::default());
    cached.open("file:///card.rs", DOC, 1);
    // The uncached shape: a fresh engine per answer — no pool to ride.
    let uncached_first = {
        let mut fresh = AeolusEngine::new(EngineConfig::default());
        fresh.open("file:///card.rs", DOC, 1);
        fresh.analyze("file:///card.rs")
    };
    let uncached_second = {
        let mut fresh = AeolusEngine::new(EngineConfig::default());
        fresh.open("file:///card.rs", DOC, 1);
        fresh.analyze("file:///card.rs")
    };
    assert_eq!(uncached_first, uncached_second);
    assert_eq!(
        cached.analyze("file:///card.rs"),
        uncached_first,
        "the cache never changes a decision — the diagnostics are the \
         pipeline's own, byte-identical beside the pool"
    );
}

#[test]
fn a_change_misses_only_its_new_strings() {
    let stats = Arc::new(CacheStats::new());
    let mut lsp = AeolusEngine::new(EngineConfig::default()).with_observer(Arc::clone(&stats));
    lsp.open("file:///card.rs", DOC, 1);
    lsp.analyze("file:///card.rs");

    lsp.change("file:///card.rs", DOC_CHANGED, 2);
    lsp.analyze("file:///card.rs");

    let snapshot = stats.snapshot();
    assert_eq!(
        snapshot,
        CacheSnapshot {
            hits: 2,
            misses: 4,
            stores: 4,
            evictions: 0,
        },
        "the kept strings (`a`, the refused root) hit; the swapped `b` \
         spelling is the change's one fresh lower"
    );
}

#[test]
fn the_cursor_answers_ride_the_pool_too() {
    let stats = Arc::new(CacheStats::new());
    let mut lsp = AeolusEngine::new(EngineConfig::default()).with_observer(Arc::clone(&stats));
    lsp.open("file:///card.rs", DOC, 1);
    let offset = DOC.find("bg-primary").expect("the fixture holds") + 4;
    let line = DOC[..offset].matches('\n').count() as u32;
    let line_start = DOC[..offset].rfind('\n').map_or(0, |i| i + 1);
    let character = DOC[line_start..offset]
        .chars()
        .map(|ch| ch.len_utf16() as u32)
        .sum::<u32>();

    let first = lsp.hover("file:///card.rs", line, character);
    let second = lsp.hover("file:///card.rs", line, character);
    assert!(
        first.is_some(),
        "the fixture's cursor sits in a lowered class"
    );
    assert_eq!(first, second, "the hover's own answer is deterministic");
    let snapshot = stats.snapshot();
    assert_eq!(
        snapshot.hits, 1,
        "the second hover's single-candidate compile rides the pool"
    );
    assert_eq!(
        snapshot.misses, 1,
        "the first hover lowered its candidate once"
    );
}

#[test]
fn the_observation_free_default_answers_unchanged() {
    let mut quiet = AeolusEngine::new(EngineConfig::default());
    quiet.open("file:///card.rs", DOC, 1);
    let quiet_answer = quiet.analyze("file:///card.rs");

    let stats = Arc::new(CacheStats::new());
    let mut observed = AeolusEngine::new(EngineConfig::default()).with_observer(Arc::clone(&stats));
    observed.open("file:///card.rs", DOC, 1);
    let observed_answer = observed.analyze("file:///card.rs");

    assert_eq!(
        quiet_answer, observed_answer,
        "the observer adds a channel; it never changes a decision"
    );
    assert_eq!(
        stats.snapshot(),
        CacheSnapshot {
            hits: 0,
            misses: 3,
            stores: 3,
            evictions: 0,
        },
        "the wired handle totals the engine's own session"
    );
}
