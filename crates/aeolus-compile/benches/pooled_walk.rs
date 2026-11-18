//! The pooled-path walk channel's microbenchmarks (divan, stable — the
//! workspace bench idiom): the walk face threaded through the dyn
//! cache, the pooled path the LSP engine rides. The arms answer the
//! channel's own question at the pool level — the observed MISS (the
//! walk lowers with the observer attached, the events stream) beside
//! the observed HIT (the pool answers, no walk runs, the channel stays
//! silent — the cache's reason for existing kept) — over the same
//! shadcn Button string the walk face's bench uses, so the two stories
//! read side by side.
//!
//! Numbers land in `reports/08-benchmarks/BENCHMARKS.md` when the gate
//! environment runs them (see `reports/10-tdd-evidence/43-pooled-walk.md`
//! for the method note).

use std::sync::Arc;

use aeolus_compile::{Config, DynCache, WalkObserver, WalkStats};
use aeolus_theme::{ThemeSource, TokenTable};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The shadcn v4 Button string — the walk face's bench's own fixture.
const BUTTON: &str = "flex items-center justify-center rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground shadow hover:bg-primary/90";

/// The observed pooled MISS: the first compile of a string lowers
/// through the observed walk (the stage events stream beside the
/// pool's store).
#[divan::bench]
fn pooled_observed_miss(bencher: Bencher) {
    let config = Config::default();
    bencher.bench(|| {
        let cache = DynCache::new();
        let stats = Arc::new(WalkStats::new());
        cache
            .compile_observed(BUTTON, &config, Arc::clone(&stats) as Arc<dyn WalkObserver>)
            .expect("the button string lowers")
    });
}

/// The observed pooled HIT: the repeat compile answers from the pool —
/// no walk runs, the observer never fires (the numbers the cache
/// face's bench prints, one observer beside).
#[divan::bench]
fn pooled_observed_hit(bencher: Bencher) {
    let config = Config::default();
    let cache = DynCache::new();
    let stats = Arc::new(WalkStats::new());
    let _ = cache
        .compile_observed(BUTTON, &config, Arc::clone(&stats) as Arc<dyn WalkObserver>)
        .expect("the button string lowers");
    bencher.bench(|| {
        cache
            .compile_observed(BUTTON, &config, Arc::clone(&stats) as Arc<dyn WalkObserver>)
            .expect("the pooled answer")
    });
}

/// The observed pooled THEMED miss: the themed walk lowers with the
/// observer attached, the table's tier consults riding the theme face
/// beside (a wired `TokenTable` streams them).
#[divan::bench]
fn pooled_theme_observed_miss(bencher: Bencher) {
    let config = Config::default();
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    let table = TokenTable::runtime(&source).expect("default-seeded source builds");
    bencher.bench(|| {
        let cache = DynCache::new();
        let stats = Arc::new(WalkStats::new());
        cache
            .compile_theme_observed(
                BUTTON,
                &table,
                &config,
                Arc::clone(&stats) as Arc<dyn WalkObserver>,
            )
            .expect("the themed button lowers")
    });
}
