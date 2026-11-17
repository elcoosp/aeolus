//! The composition tier's benchmark (the parent-relative probe walk —
//! the group/peer relations' frame-fact walk, the starting tier's
//! sibling): the interp walk over a composition-mixed program — the
//! fact-bearing env (the fire path: a `.group` ancestor in the
//! composed state) against the default env (the resting path: the
//! layers stay the quiet miss a frame the compositions do not hold
//! in).

use aeolus_compile::{Config, DynCache, Env, interp};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The Button string's composition half — the layers the frame facts
/// answer in the pipeline's real consumer, beside one base declaration
/// (the fire path carries the parent-relative tier's env read).
const COMPOSITION_CLASSES: &str = "p-4 group-active:p-8 peer-disabled:p-2 group-focus-visible:p-1";

#[divan::bench]
fn interp_compositions_fire(bencher: Bencher) {
    let cache = DynCache::new();
    let compiled = cache
        .compile(COMPOSITION_CLASSES, &Config::default())
        .expect("non-empty input compiles");
    let env = Env {
        group_active: true,
        peer_disabled: true,
        group_focus_visible: true,
        ..Env::default()
    };
    bencher.bench(|| interp(&compiled.program, &env).expect("the program resolves"));
}

/// The same program under the default env: the composition layers stay
/// the quiet miss a frame the compositions do not hold in (the resting
/// env asserts no composition bits — the frame facts' own reading).
#[divan::bench]
fn interp_compositions_resting(bencher: Bencher) {
    let cache = DynCache::new();
    let compiled = cache
        .compile(COMPOSITION_CLASSES, &Config::default())
        .expect("non-empty input compiles");
    bencher.bench(|| interp(&compiled.program, &Env::default()).expect("the program resolves"));
}

/// The fold's cost: compiling the composition-mixed string through the
/// full pipeline (the lex → fold → lower → sort walk the cache keys),
/// the composition spellings riding the same classifier wall the
/// landed group/peer pair shares.
#[divan::bench]
fn compile_compositions(bencher: Bencher) {
    let cache = DynCache::new();
    bencher.bench(|| {
        cache
            .compile(COMPOSITION_CLASSES, &Config::default())
            .expect("non-empty input compiles")
    });
}
