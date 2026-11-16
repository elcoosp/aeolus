//! The capability families' microbenchmarks (divan, stable — the
//! workspace bench idiom): the `pointer-*`/`any-pointer-*` input tier
//! and the `portrait`/`landscape` orientation forms' walk cost. The
//! [`Env`] answers every layer (frame state — no probe seam, no
//! sourceless note path), so the per-layer question is the env-field
//! read the whole media tier shares — the bench records the family
//! beside the interp arm's resting-env baseline.
//!
//! The bench bodies `expect` on the interp: a compiled program resolves
//! under any env (errors are caller bugs — unknown ids, non-finite
//! alphas — and none of those can come out of the pipeline).
//!
//! Numbers land in `reports/08-benchmarks/BENCHMARKS.md` when the gate
//! environment runs them (see `reports/10-tdd-evidence/25-capability-tier.md`
//! for the red-to-green record).

use aeolus_compile::{Config, DynCache, Env, interp};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The Button string's capability half — the layers the input-capability
/// and orientation tier answers in the pipeline's real consumer.
const CAP_CLASSES: &str =
    "pointer-fine:p-8 pointer-coarse:mt-2 any-pointer-none:hidden portrait:px-3 landscape:flex";

#[divan::bench]
fn interp_capabilities_button(bencher: Bencher) {
    let cache = DynCache::new();
    let compiled = cache
        .compile(CAP_CLASSES, &Config::default())
        .expect("non-empty input compiles");
    let env = Env {
        pointer_fine: true,
        portrait: true,
        ..Env::default()
    };
    bencher.bench(|| interp(&compiled.program, &env).expect("the program resolves"));
}

/// The same program under the default env: the pointer layers are the
/// quiet miss a media query that does not hold is — while
/// `landscape:` fires (the complement of the resting `portrait: false`
/// fact, the orientation query always holding an answer).
#[divan::bench]
fn interp_capabilities_default_env(bencher: Bencher) {
    let cache = DynCache::new();
    let compiled = cache
        .compile(CAP_CLASSES, &Config::default())
        .expect("non-empty input compiles");
    let env = Env::default();
    bencher.bench(|| interp(&compiled.program, &env).expect("the program resolves"));
}
