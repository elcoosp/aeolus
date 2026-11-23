//! The `.tw()` walk's allocation bench (divan, stable) — the plan §4
//! perf contract's static-application path, beside the alloc-contract
//! battery's pins. divan's own allocation tracking reports the
//! `alloc_count` per arm next to the wall clock: the Copy-slot walks
//! answer zero, the answer's owned `Vec` fills show their growth
//! pattern, the freeze answers zero (a `const fn`).
//!
//! The battery (`tests/alloc_contract.rs`) pins the contract; this file
//! prices it.

use std::sync::Arc;

use aeolus_compile::{Config, Env};
use aeolus_rt::{ClassCell, StaticProgram, StaticProgramBuilder};
use aeolus_theme::{ThemeSource, TokenTable};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The baked shadcn LIGHT table (T0's shape) — the freeze walk's own
/// construction.
fn baked_table() -> Arc<TokenTable> {
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    Arc::new(TokenTable::baked(&source).expect("the baked shadcn table builds"))
}

/// Build a static program through the dynamic lane's own compile (the
/// same compiler the macro runs) — the compile allocates by design and
/// runs outside the benched closures.
fn frozen(classes: &str, table: &TokenTable) -> StaticProgram {
    let cell = ClassCell::new();
    cell.get_theme(classes, table, &Config::default())
        .expect("the cell compiles what lowers")
}

/// The freeze: the two-pointer assembly of the frozen slices — the
/// `const fn` the macro's outlined emit face calls.
#[divan::bench]
fn assemble(bencher: Bencher) {
    let table = baked_table();
    let program = frozen(
        "p-4 bg-red-500 rounded-md hover:bg-red-500/80 text-sm font-medium",
        &table,
    );
    let layers: &[aeolus_ir::Layer] = program.layers;
    let keywords: &[&str] = program.keywords;
    bencher.bench(|| StaticProgramBuilder::assemble(layers, keywords));
}

/// The layout families — every slot Copy, the walk's own machinery at
/// zero heap events.
#[divan::bench]
fn walk_layout(bencher: Bencher) {
    let table = baked_table();
    let program = frozen(
        "p-4 px-8 gap-4 rounded-lg border-2 items-center justify-center",
        &table,
    );
    let env = Env::default();
    bencher.bench(|| program.style(&env));
}

/// The colour family — the direct `from_srgb` conversion, no hex-string
/// round-trip on the per-frame path.
#[divan::bench]
fn walk_colour(bencher: Bencher) {
    let table = baked_table();
    let program = frozen("bg-red-500 text-red-500 border-red-500", &table);
    let env = Env::default();
    bencher.bench(|| program.style(&env));
}

/// The shadcn Button base string — the realistic per-element budget,
/// the answer's owned `Vec` fills included (divan's `alloc_count`
/// column shows the growth pattern the battery upper-bounds).
#[divan::bench]
fn walk_button(bencher: Bencher) {
    let table = baked_table();
    let program = frozen(
        "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium \
         transition-colors bg-primary text-primary-foreground hover:bg-primary/90",
        &table,
    );
    let env = Env::default();
    bencher.bench(|| program.style(&env));
}
