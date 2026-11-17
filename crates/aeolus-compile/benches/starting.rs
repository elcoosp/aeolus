//! The `starting:` variant's benchmark (the media-feature families'
//! sibling — the at-rule tier's frame-fact walk): the interp walk over
//! a starting-style-mixed program — the fact-bearing env (the fire
//! path: the transition's first frame the host gates) against the
//! default env (the resting path: the layer stays the quiet miss a
//! frame the element is not in is).

use aeolus_compile::{Config, DynCache, Env, interp};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The Button string's starting-style half — the layer the frame fact
/// answers in the pipeline's real consumer, beside one base
/// declaration (the fire path carries the at-rule tier's env read).
const STARTING_CLASSES: &str = "starting:p-8 p-4";

#[divan::bench]
fn interp_starting_fire(bencher: Bencher) {
    let cache = DynCache::new();
    let compiled = cache
        .compile(STARTING_CLASSES, &Config::default())
        .expect("non-empty input compiles");
    let env = Env {
        starting_style: true,
        ..Env::default()
    };
    bencher.bench(|| interp(&compiled.program, &env).expect("the program resolves"));
}

/// The same program under the default env: the `starting:` layer stays
/// the quiet miss a frame the element is not in is (the resting env
/// asserts no starting-style bit — the frame fact's own reading).
#[divan::bench]
fn interp_starting_resting(bencher: Bencher) {
    let cache = DynCache::new();
    let compiled = cache
        .compile(STARTING_CLASSES, &Config::default())
        .expect("non-empty input compiles");
    let env = Env::default();
    bencher.bench(|| interp(&compiled.program, &env).expect("the program resolves"));
}
