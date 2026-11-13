//! The named-container seam's microbenchmarks (divan, stable — the
//! workspace bench idiom): the plan §3.5 container row's RESOLUTION
//! half, the walk the `ContainerProbe` seam arms. The per-frame cost
//! the seam adds is the number this bench exists for: one trait-object
//! ask per named layer (a small table scan), the rest of the walk
//! unchanged — the same interned-program read the interp bench keys on.
//!
//! The bench bodies `expect` on the walk: a compiled program resolves
//! under any env + source pair (errors are caller bugs — unknown token
//! ids, non-finite alphas — and none of those can come out of the
//! pipeline).
//!
//! Numbers land in `reports/08-benchmarks/BENCHMARKS.md` when the gate
//! environment runs them.

use aeolus_compile::{
    Config, ContainerBound, ContainerState, DynCache, Env, interp_containers,
    interp_theme_containers,
};
use aeolus_theme::{ThemeSource, TokenTable};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The container-mixed string: two NAMED container layers (a min and a
/// max form, the fold's two spellings), one UNNAMED container layer and
/// one base declaration — the four-arm walk the seam rides.
const CONTAINER_CLASSES: &str = "@md/card:w-64 @max-lg/sidebar:hidden @md:flex p-4";

/// The runtime-tier table over the shadcn source (the themed bench's
/// token forms).
fn runtime_table() -> TokenTable {
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    TokenTable::runtime(&source).expect("default-seeded source builds")
}

/// Compile once (outside the measured loop) — the interp's input is the
/// interned program, exactly the cached-loop shape.
fn container_program(table: &TokenTable) -> aeolus_compile::Compiled {
    let cache = DynCache::new();
    cache
        .compile_theme(CONTAINER_CLASSES, table, &Config::default())
        .expect("non-empty input compiles")
}

/// The bounds table the names HIT: card and sidebar wide enough — both
/// named layers fire (the hot path: two table scans, two comparisons).
fn hit_bounds() -> ContainerState<'static> {
    const BOUNDS: [ContainerBound<'static>; 2] = [
        ContainerBound {
            name: "card",
            rem: 48.0,
        },
        ContainerBound {
            name: "sidebar",
            rem: 30.0,
        },
    ];
    ContainerState::new(&BOUNDS)
}

/// The bounds table the names MISS: the source knows other containers —
/// the quiet-skip path (a CSS selector that does not match).
fn miss_bounds() -> ContainerState<'static> {
    const BOUNDS: [ContainerBound<'static>; 1] = [ContainerBound {
        name: "menu",
        rem: 20.0,
    }];
    ContainerState::new(&BOUNDS)
}

/// The themeless hit: both named layers fire, the unnamed layer answers
/// from the Env (one walk, three container comparisons).
#[divan::bench]
fn interp_containers_hit(bencher: Bencher) {
    let table = runtime_table();
    let compiled = container_program(&table);
    let env = Env {
        container_rem: Some(48.0),
        ..Env::default()
    };
    let bounds = hit_bounds();
    bencher.bench(|| {
        interp_containers(&compiled.program, &env, &bounds).expect("the program resolves")
    });
}

/// The themeless quiet miss: the names go unanswered — the skip path's
/// cost, for the record (the layers never fire, no notes either).
#[divan::bench]
fn interp_containers_quiet_miss(bencher: Bencher) {
    let table = runtime_table();
    let compiled = container_program(&table);
    let env = Env {
        container_rem: Some(48.0),
        ..Env::default()
    };
    let bounds = miss_bounds();
    bencher.bench(|| {
        interp_containers(&compiled.program, &env, &bounds).expect("the program resolves")
    });
}

/// The themed form over the same program: the token slots resolve
/// beside the container seam (both seams ride one walk — the DynCache
/// loop's shape).
#[divan::bench]
fn interp_theme_containers_hit(bencher: Bencher) {
    let table = runtime_table();
    let compiled = container_program(&table);
    let env = Env {
        container_rem: Some(48.0),
        ..Env::default()
    };
    let bounds = hit_bounds();
    bencher.bench(|| {
        interp_theme_containers(&compiled.program, &table, &env, &bounds)
            .expect("the program resolves")
    });
}
