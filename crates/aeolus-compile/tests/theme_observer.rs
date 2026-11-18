//! The theme-resolution observability face's cross-layer pin (TDD:
//! written FIRST beside the face's own battery — the RED capture is
//! the same missing-`ThemeObserver` compile failure, recorded in
//! `reports/10-tdd-evidence/42-theme-observer.md`).
//!
//! The face rides the [`TokenTable`] itself, so the compile walk's
//! lowering consults stream through it with ZERO compile-crate
//! changes: a table wired before `compile_theme` reports the theme
//! stage's per-declaration verdicts while the walk assembles — the
//! stage boundary the `WalkObserver` face's lex/candidate/assemble
//! events sit above. These pins prove the ride and the seam's
//! answer-neutrality at the pipeline level.

use std::sync::Arc;

use aeolus_compile::{Config, compile_theme};
use aeolus_theme::{ThemeSource, ThemeStats, TokenTable};

const THEMED: &str = "bg-primary/90 hover:bg-primary";

/// A runtime table wired with the built-in collector streams the
/// walk's per-declaration consults (the `/90` modifier and the bare
/// token both ride the T2 carry), and the observed table compiles
/// byte-identically to the plain one — the face adds a channel, never
/// changes an answer.
#[test]
fn the_themed_walk_streams_the_theme_face() {
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    let config = Config::default();

    let stats = Arc::new(ThemeStats::new());
    let table = TokenTable::runtime(&source)
        .expect("default-seeded source builds")
        .with_observer(Arc::clone(&stats));
    let observed = compile_theme(THEMED, &table, &config).expect("themed compile");

    let snap = stats.snapshot();
    assert!(
        snap.resolves >= 2,
        "the lowering consulted the table per colour declaration: {snap:?}"
    );
    assert_eq!(
        snap.carried, snap.resolves,
        "the runtime tier carries every consult — no baked arm on a T2 table"
    );
    assert_eq!(
        snap.misses, 0,
        "the themed compile's colour path never refuses"
    );

    let plain = compile_theme(
        THEMED,
        &TokenTable::runtime(&source).expect("default-seeded source builds"),
        &config,
    )
    .expect("themed compile");
    assert_eq!(
        observed, plain,
        "the observed table compiles byte-identically — answer-neutrality at \
         the pipeline level"
    );
}

/// The baked tier's walk totals into the pair/literal arms the T1/T0
/// shapes promise: the dark-carrying `primary` lowers as a mode pair,
/// the light-only tokens as literals — the snapshot splits them.
#[test]
fn the_baked_walk_totals_the_pair_and_literal_arms() {
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    let config = Config::default();

    let stats = Arc::new(ThemeStats::new());
    let table = TokenTable::baked(&source)
        .expect("default-seeded source builds")
        .with_observer(Arc::clone(&stats));
    let observed = compile_theme(THEMED, &table, &config).expect("themed compile");

    let snap = stats.snapshot();
    assert!(
        snap.pairs >= 1,
        "`primary` carries a dark value on the shadcn source — the lowering \
         consults answer as mode pairs: {snap:?}"
    );
    assert_eq!(
        (snap.carried, snap.misses),
        (0, 1),
        "the `/90` modifier on the dark-carrying `primary` refuses typed (the \
         pair form has no alpha arm — the honest T1 boundary); the miss IS \
         the event the host reads"
    );

    let plain = compile_theme(
        THEMED,
        &TokenTable::baked(&source).expect("default-seeded source builds"),
        &config,
    )
    .expect("themed compile");
    assert_eq!(
        observed, plain,
        "the baked walk's answers are the plain walk's, observed or not — \
         the typed refusal included"
    );
}
