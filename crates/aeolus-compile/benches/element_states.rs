//! The element-state family's benchmarks (the element-state tier beside
//! the validation states): the interp walk over an element-state-mixed
//! program — the fact-bearing env (the fire path: `target:`,
//! `indeterminate:` and `autofill:` answer from their one fact each,
//! `read-only:` from the editability complement) against the default
//! env (the resting path: `read-only:` fires its complement, the other
//! three stay the quiet miss a selector that does not match is).

use aeolus_compile::{Config, DynCache, Env, interp};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The Button string's element-state half — the layers the element
/// states answer in the pipeline's real consumer (`target:` beside its
/// `read-only:` sibling and the one-fact `indeterminate:`/`autofill:`
/// pair; the flat fold keeps one condition per layer, the mix pins the
/// whole tier's walk).
const ELEMENT_STATE_CLASSES: &str =
    "target:p-8 read-only:mt-2 indeterminate:hidden autofill:ring-2";

#[divan::bench]
fn interp_element_state_button(bencher: Bencher) {
    let cache = DynCache::new();
    let compiled = cache
        .compile(ELEMENT_STATE_CLASSES, &Config::default())
        .expect("non-empty input compiles");
    let env = Env {
        url_target: true,
        read_write: false,
        ..Env::default()
    };
    bencher.bench(|| interp(&compiled.program, &env).expect("the program resolves"));
}

/// The same program under the default env: the `target:`,
/// `indeterminate:` and `autofill:` layers are the quiet miss a
/// selector that does not match is — while `read-only:` fires (the
/// complement of the resting `read_write: false` fact, CSS's own
/// non-editable-element reading).
#[divan::bench]
fn interp_element_state_default_env(bencher: Bencher) {
    let cache = DynCache::new();
    let compiled = cache
        .compile(ELEMENT_STATE_CLASSES, &Config::default())
        .expect("non-empty input compiles");
    let env = Env::default();
    bencher.bench(|| interp(&compiled.program, &env).expect("the program resolves"));
}
