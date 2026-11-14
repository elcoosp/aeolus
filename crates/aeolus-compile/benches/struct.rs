//! The structural seam's microbenchmarks (divan, stable — the workspace
//! bench idiom): the plan §3.5 structural row's RESOLUTION half, the
//! walk the [`StructProbe`] seam arms. The per-frame cost the seam adds
//! is the number this bench exists for: one trait-object ask per
//! structural layer (one Copy facts value, the verdict a few integer
//! compares), the rest of the walk unchanged — the same interned-
//! program read the interp bench keys on.
//!
//! The bench bodies `expect` on the walk: a compiled program resolves
//! under any env + source pair (errors are caller bugs — unknown token
//! ids, non-finite alphas — and none of those can come out of the
//! pipeline).
//!
//! Numbers land in `reports/08-benchmarks/BENCHMARKS.md` when the gate
//! environment runs them.

use aeolus_compile::{
    Config, DynCache, Env, StructFacts, compile, interp_struct, interp_theme_struct,
};
use aeolus_theme::{ThemeSource, TokenTable};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The structural-mixed string: one first layer, one odd layer, one
/// nth layer and one base declaration — the four-arm walk the seam
/// rides (each structural layer is ONE trait-object ask).
const STRUCT_CLASSES: &str = "first:w-64 odd:hidden nth-3:hidden p-4";

/// The baked table over the shadcn source (the themed bench's token
/// forms).
fn baked_table() -> TokenTable {
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    TokenTable::baked(&source).expect("default-seeded source builds")
}

/// Compile once (outside the measured loop) — the interp's input is the
/// interned program, exactly the cached-loop shape.
fn struct_program(table: &TokenTable) -> aeolus_compile::Compiled {
    let cache = DynCache::new();
    cache
        .compile_theme(STRUCT_CLASSES, table, &Config::default())
        .expect("non-empty input compiles")
}

/// The facts the layers HIT: a first child of three (first fires,
/// odd/nth quietly skip — the partial-fire path's cost).
fn hit_facts() -> StructFacts {
    StructFacts::new(0, 3)
}

/// The themeless hit: the first layer fires, the others quietly skip
/// (three fact asks, three integer-compare verdicts).
#[divan::bench]
fn interp_struct_hit(bencher: Bencher) {
    let compiled = compile(STRUCT_CLASSES, &Config::default()).expect("non-empty input compiles");
    let facts = hit_facts();
    bencher.bench(|| {
        interp_struct(&compiled.program, &Env::default(), &facts).expect("the program resolves")
    });
}

/// The unknown-facts miss: the built-in answers "unknown" — every
/// index-based test quietly skips (the miss path's cost, for the
/// record).
#[divan::bench]
fn interp_struct_unknown_miss(bencher: Bencher) {
    let compiled = compile(STRUCT_CLASSES, &Config::default()).expect("non-empty input compiles");
    bencher.bench(|| {
        interp_struct(&compiled.program, &Env::default(), &StructFacts::UNKNOWN)
            .expect("the program resolves")
    });
}

/// The themed form over the same program: the token slots resolve
/// beside the structural seam (both seams ride one walk — the DynCache
/// loop's shape).
#[divan::bench]
fn interp_theme_struct_hit(bencher: Bencher) {
    let table = baked_table();
    let compiled = struct_program(&table);
    let facts = hit_facts();
    bencher.bench(|| {
        interp_theme_struct(&compiled.program, &table, &Env::default(), &facts)
            .expect("the program resolves")
    });
}
