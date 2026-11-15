//! The filter tier's polyfill-half microbenchmark (divan, stable — the
//! workspace bench idiom): the per-frame cost of the §3.7 filter row's
//! landed half — the walk applying a cell-compiled matrix-chain
//! program (the spec matrices recolouring the element's own colour
//! slots), and the accumulator's own shade as the unit surface.

use divan::Bencher;

use aeolus_compile::Config;
use aeolus_gpui::prelude::*;

/// The cell-compiled programs (the compile cost rides the setup, the
/// walk rides the bench — the same split the ring/grad benches run).
fn program(classes: &str) -> StaticProgram {
    let source = aeolus_rt::ThemeSource::shadcn().expect("the built-in shadcn source parses");
    let table = aeolus_rt::TokenTable::baked(&source).expect("the T0/T1 bake succeeds");
    aeolus_rt::ClassCell::new()
        .get_theme(classes, &table, &Config::default())
        .expect("the bench string lowers clean")
}

#[divan::bench]
fn matrix_chain_walk(bencher: Bencher) {
    // The full seven-matrix chain over a coloured surface: the walk
    // records the amounts, recolours the background/text/border slots
    // and re-applies them filtered.
    let program = program(
        "bg-red-500 text-white border-red-500 brightness-110 contrast-125 \
         grayscale hue-rotate-90 invert saturate-150 sepia-50",
    );
    bencher.bench(|| gpui::StyleRefinement::default().tw_env(program, Env::default()));
}

#[divan::bench]
fn drop_shadow_walk(bencher: Bencher) {
    // The drop-shadow band parse + composition beside the shadow
    // tier's preset bands.
    let program = program("shadow-lg drop-shadow-md bg-primary");
    bencher.bench(|| gpui::StyleRefinement::default().tw_env(program, Env::default()));
}

#[divan::bench]
fn empty_chain_walk(bencher: Bencher) {
    // No filter class in the program: the walk's pre-filter cost — the
    // accumulator stays an identity, nothing composes.
    let program = program("bg-red-500 text-white p-4 shadow-lg");
    bencher.bench(|| gpui::StyleRefinement::default().tw_env(program, Env::default()));
}

#[divan::bench]
fn shade_unit(bencher: Bencher) {
    // The unit surface: the composed chain recolouring one colour —
    // the seven spec matrices in v4's var-canonical order.
    use aeolus_ir::{Prop, Val};
    bencher.bench(|| {
        let mut state = FilterState::default();
        state.apply(Prop::FilterBrightness, Val::Num(110.0), "");
        state.apply(Prop::FilterGrayscale, Val::Num(100.0), "");
        state.apply(Prop::FilterSaturate, Val::Num(150.0), "");
        state.shade(gpui::hsla(0.02, 0.83, 0.58, 1.0))
    });
}

fn main() {
    divan::main();
}
