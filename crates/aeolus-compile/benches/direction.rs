//! The direction family's microbenchmarks (divan, stable — the
//! workspace bench idiom): the `ltr:`/`rtl:` selector-side tier's walk
//! cost. The [`Env`] answers every layer (frame state — no probe seam,
//! no sourceless note path), so the per-layer question is the env-field
//! read the whole frame-state tier shares — the bench records the family
//! beside the interp arm's resting-env baseline.
//!
//! The bench bodies `expect` on the interp: a compiled program resolves
//! under any env (errors are caller bugs — unknown ids, non-finite
//! alphas — and none of those can come out of the pipeline).
//!
//! Numbers land in `reports/08-benchmarks/BENCHMARKS.md` when the gate
//! environment runs them (see `reports/10-tdd-evidence/26-direction-family.md`
//! for the red-to-green record).

use aeolus_compile::{Config, DynCache, Env, interp};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The Button string's direction half — the layers the direction tier
/// answers in the pipeline's real consumer (`rtl:` beside its `ltr:`
/// sibling; the flat fold keeps one condition per layer, the pair
/// pins both spellings' walk).
const DIR_CLASSES: &str = "rtl:p-8 ltr:mt-2 rtl:hidden";

#[divan::bench]
fn interp_direction_button(bencher: Bencher) {
    let cache = DynCache::new();
    let compiled = cache
        .compile(DIR_CLASSES, &Config::default())
        .expect("non-empty input compiles");
    let env = Env {
        direction_rtl: true,
        ..Env::default()
    };
    bencher.bench(|| interp(&compiled.program, &env).expect("the program resolves"));
}

/// The same program under the default env: the `rtl:` layers are the
/// quiet miss a selector that does not match is — while `ltr:` fires
/// (the complement of the resting `direction_rtl: false` fact — CSS's
/// own initial `direction` value is ltr, the orientation precedent).
#[divan::bench]
fn interp_direction_default_env(bencher: Bencher) {
    let cache = DynCache::new();
    let compiled = cache
        .compile(DIR_CLASSES, &Config::default())
        .expect("non-empty input compiles");
    let env = Env::default();
    bencher.bench(|| interp(&compiled.program, &env).expect("the program resolves"));
}
