//! The pooled-path walk channel's contract tests (TDD: written FIRST —
//! the RED capture is the "no `DynCache::compile_observed` /
//! `compile_theme_observed` entry" compile failure, recorded in
//! `reports/10-tdd-evidence/43-pooled-walk.md`) — the walk face
//! ([`WalkObserver`]) threaded through the dyn cache, the pooled path
//! the LSP engine rides, so a host sees the walk's stage events
//! WITHOUT bypassing the pool.
//!
//! The pins, in the order the module docs promise them:
//!
//! * a MISS lowers through the observed walk — the stage events stream
//!   exactly as the free [`aeolus_compile::compile_observed`] reports
//!   them;
//! * a HIT answers from the pool — no walk ran, no walk events (the
//!   cache's reason for existing, kept: the channel reports
//!   lowerings, not answers);
//! * the observed pooled compile is byte-identical to the plain pooled
//!   compile (the face adds a channel, never changes a decision);
//! * one `Arc<WalkStats>` handle spans the free walk and the pooled
//!   walk (the `Arc<T>` blanket's shared-handle shape) — and the
//!   themed pooled variant streams the same stage vocabulary with the
//!   table's tier consults riding beside (the theme face's channel).

use std::sync::Arc;

use aeolus_compile::{Config, Customs, DynCache, WalkObserver, WalkStats, compile_observed};
use aeolus_theme::{ThemeSource, TokenTable};

/// A themed string the pipeline lowers through the token table (the
/// `/90` modifier rides the runtime carry).
const THEMED: &str = "bg-primary/90 hover:bg-primary";

#[test]
fn the_pooled_miss_lowers_through_the_observed_walk() {
    let config = Config::default();
    let cache = DynCache::new();
    let stats = Arc::new(WalkStats::new());

    let pooled = cache
        .compile_observed(
            "p-4 flex z-10",
            &config,
            Arc::clone(&stats) as Arc<dyn WalkObserver>,
        )
        .expect("the first compile is a miss that lowers");

    let snap = stats.snapshot();
    assert_eq!(
        (snap.walks, snap.layers),
        (1, pooled.program.layers.len() as u64),
        "the miss ran ONE observed walk; the layer event answers the \
         program's own count"
    );
    assert!(
        snap.candidates >= 3,
        "the per-candidate events streamed the lexed tokens: {snap:?}"
    );
    assert!(
        snap.lowered >= 2 && snap.missed <= 1,
        "the fold/read verdicts split lowered from missed: {snap:?}"
    );
}

#[test]
fn the_pooled_hit_streams_nothing() {
    let config = Config::default();
    let cache = DynCache::new();
    let stats = Arc::new(WalkStats::new());

    let _ = cache
        .compile_observed(
            "p-4 flex z-10",
            &config,
            Arc::clone(&stats) as Arc<dyn WalkObserver>,
        )
        .expect("the miss lowers");
    let after_miss = stats.snapshot();
    let _ = cache
        .compile_observed(
            "p-4 flex z-10",
            &config,
            Arc::clone(&stats) as Arc<dyn WalkObserver>,
        )
        .expect("the hit rides the pool");

    assert_eq!(
        stats.snapshot(),
        after_miss,
        "the hit answered from the pool — no walk ran, no walk events (the \
         channel reports lowerings, not answers)"
    );
}

#[test]
fn the_observed_pooled_compile_stays_byte_identical() {
    let config = Config::default();
    let cache = DynCache::new();
    let stats = Arc::new(WalkStats::new());

    let observed = cache
        .compile_observed(
            "p-4 hover:bg-primary/90",
            &config,
            Arc::clone(&stats) as Arc<dyn WalkObserver>,
        )
        .expect("lowers");
    let plain = cache
        .compile("p-4 hover:bg-primary/90", &config)
        .expect("lowers");

    assert_eq!(
        observed, plain,
        "the face adds a channel, never changes a decision — the pooled \
         artifact is the plain pool's own"
    );
}

#[test]
fn the_free_walk_and_the_pooled_walk_share_one_handle() {
    let config = Config::default();
    let cache = DynCache::new();
    let stats = Arc::new(WalkStats::new());

    let _ = compile_observed(
        "p-4 flex",
        None,
        &Customs::new(),
        &config,
        Arc::clone(&stats) as Arc<dyn WalkObserver>,
    )
    .expect("the free observed walk lowers");
    let _ = cache
        .compile_observed(
            "gap-2 rounded-md",
            &config,
            Arc::clone(&stats) as Arc<dyn WalkObserver>,
        )
        .expect("the pooled observed walk lowers");

    let snap = stats.snapshot();
    assert_eq!(
        snap.walks, 2,
        "one handle totals the free walk AND the pooled walk (the Arc \
         blanket's shared-handle shape)"
    );
}

#[test]
fn the_pooled_theme_walk_streams_the_stage_events() {
    let config = Config::default();
    let cache = DynCache::new();
    let stats = Arc::new(WalkStats::new());
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    let table = TokenTable::runtime(&source).expect("default-seeded source builds");

    let pooled = cache
        .compile_theme_observed(
            THEMED,
            &table,
            &config,
            Arc::clone(&stats) as Arc<dyn WalkObserver>,
        )
        .expect("the themed miss lowers");
    let after_miss = stats.snapshot();
    let _ = cache
        .compile_theme_observed(
            THEMED,
            &table,
            &config,
            Arc::clone(&stats) as Arc<dyn WalkObserver>,
        )
        .expect("the themed hit rides the pool");

    assert_eq!(after_miss.walks, 1, "the themed miss ran ONE observed walk");
    assert_eq!(
        stats.snapshot(),
        after_miss,
        "the themed hit streams nothing — the pool answered"
    );
    assert_eq!(
        pooled.program.layers.len() as u64,
        after_miss.layers,
        "the assembly event answers the themed program's own layer count"
    );
}
