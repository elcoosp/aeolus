//! The validation states' benchmarks (the element-state tier beside
//! `hover:`/`checked:`): the interp walk over a validation-mixed
//! program — the fact-bearing env (the fire path: `invalid:` and
//! `user-invalid:` answer from the constraint fact past the
//! interaction gate) against the default env (the resting path:
//! `valid:` fires its complement, the `user-` pair stays the quiet
//! miss a selector that does not match is).

use aeolus_compile::{Config, DynCache, Env, interp};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The Button string's validation half — the layers the validation
/// states answer in the pipeline's real consumer (`invalid:` beside
/// its `valid:` sibling and the gated `user-invalid:`; the flat fold
/// keeps one condition per layer, the mix pins the whole tier's walk).
const VALIDATION_CLASSES: &str = "invalid:p-8 valid:mt-2 user-invalid:hidden";

#[divan::bench]
fn interp_validation_button(bencher: Bencher) {
    let cache = DynCache::new();
    let compiled = cache
        .compile(VALIDATION_CLASSES, &Config::default())
        .expect("non-empty input compiles");
    let env = Env {
        validation_invalid: true,
        user_validated: true,
        ..Env::default()
    };
    bencher.bench(|| interp(&compiled.program, &env).expect("the program resolves"));
}

/// The same program under the default env: the `invalid:` and
/// `user-invalid:` layers are the quiet miss a selector that does not
/// match is — while `valid:` fires (the complement of the resting
/// `validation_invalid: false` fact, CSS's own pristine-field reading).
#[divan::bench]
fn interp_validation_default_env(bencher: Bencher) {
    let cache = DynCache::new();
    let compiled = cache
        .compile(VALIDATION_CLASSES, &Config::default())
        .expect("non-empty input compiles");
    let env = Env::default();
    bencher.bench(|| interp(&compiled.program, &env).expect("the program resolves"));
}
