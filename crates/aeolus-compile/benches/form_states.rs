//! The form-state family's benchmarks (the tier beside the
//! widget-state family): the interp walk over a form-state-mixed
//! program — the fact-bearing env (the fire path: `enabled:`,
//! `required:`, `optional:`, `in-range:` and `out-of-range:` answer
//! from their membership + state fact pairs) against the default env
//! (the resting path: all five stay the quiet miss a selector that
//! does not match is — the matches-neither boundary the family
//! encodes).

use aeolus_compile::{Config, DynCache, Env, interp};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The Button string's form-state half — the layers the form states
/// answer in the pipeline's real consumer (`enabled:` beside its
/// `required:`/`optional:`/`in-range:`/`out-of-range:` siblings; the
/// flat fold keeps one condition per layer, the mix pins the whole
/// tier's walk).
const FORM_STATE_CLASSES: &str =
    "enabled:p-8 required:hidden optional:ring-2 in-range:mt-2 out-of-range:p-4";

#[divan::bench]
fn interp_form_state_button(bencher: Bencher) {
    let cache = DynCache::new();
    let compiled = cache
        .compile(FORM_STATE_CLASSES, &Config::default())
        .expect("non-empty input compiles");
    let env = Env {
        form_associated: true,
        required_field: true,
        range_bounded: true,
        range_in: true,
        ..Env::default()
    };
    bencher.bench(|| interp(&compiled.program, &env).expect("the program resolves"));
}

/// The same program under the default env: the `enabled:`,
/// `required:`, `optional:`, `in-range:` and `out-of-range:` layers are
/// all the quiet miss a selector that does not match is (the resting
/// env asserts none of the membership facts — the matches-neither
/// boundary the two-fact shape encodes).
#[divan::bench]
fn interp_form_state_default_env(bencher: Bencher) {
    let cache = DynCache::new();
    let compiled = cache
        .compile(FORM_STATE_CLASSES, &Config::default())
        .expect("non-empty input compiles");
    let env = Env::default();
    bencher.bench(|| interp(&compiled.program, &env).expect("the program resolves"));
}
