//! The transform lane's microbenchmark (the wave's M1c half): the
//! frozen read over a program (the base+env walk recording the seven
//! 2D longhands) and the matrix compose itself — the host-side cost of
//! the lane the element walk cannot carry (no transform slot at the
//! pin).
use divan::Bencher;

use aeolus_compile::Config;
use aeolus_gpui::prelude::*;
use aeolus_rt::ClassCell;

fn program_of(classes: &str) -> StaticProgram {
    let cell = ClassCell::new();
    cell.get(classes, &Config::default())
        .expect("the transform program compiles through the dynamic lane")
}

#[divan::bench]
fn transform_frozen_read(bencher: Bencher) {
    // The lane's frozen read: the base+env layers' seven longhands
    // into one `TransformState` (the banner corpus: translate, rotate,
    // scale, skew in one program).
    let program = program_of("translate-x-4 rotate-45 scale-75 skew-x-12");
    bencher.bench(|| transform_state(&program, &aeolus_gpui::env()));
}

#[divan::bench]
fn transform_compose(bencher: Bencher) {
    // The matrix compose: scale (the skew folded into the 2×2), then
    // rotate, then translate — GPUI's own TransformationMatrix.
    let program = program_of("translate-x-4 rotate-45 scale-75 skew-x-12");
    let state = transform_state(&program, &aeolus_gpui::env());
    bencher.bench(|| state.transformation(16.0));
}

fn main() {
    divan::main();
}
