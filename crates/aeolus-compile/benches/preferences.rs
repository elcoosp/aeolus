//! The preference families' microbenchmarks (divan, stable — the
//! workspace bench idiom): the `motion-*`/`contrast-*`/
//! `forced-colors:`/`print:` families' walk cost. The [`Env`] answers
//! every layer (frame state — no probe seam, no sourceless note path),
//! so the per-layer question is the env-field read the whole state
//! tier shares — the bench records the family beside the interp arm's
//! resting-env baseline.
//!
//! The bench bodies `expect` on the interp: a compiled program resolves
//! under any env (errors are caller bugs — unknown ids, non-finite
//! alphas — and none of those can come out of the pipeline).
//!
//! Numbers land in `reports/08-benchmarks/BENCHMARKS.md` when the gate
//! environment runs them (see `reports/10-tdd-evidence/24-preference-families.md`
//! for the red-to-green record).

use aeolus_compile::{Config, DynCache, Env, interp};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The Button string's preference half — the layers the media-feature
/// tier answers in the pipeline's real consumer.
const PREF_CLASSES: &str =
    "motion-reduce:p-8 motion-safe:mt-2 contrast-more:ring-2 forced-colors:hidden print:px-3";

#[divan::bench]
fn interp_preferences_button(bencher: Bencher) {
    let cache = DynCache::new();
    let compiled = cache
        .compile(PREF_CLASSES, &Config::default())
        .expect("non-empty input compiles");
    let env = Env {
        reduced_motion: true,
        print_media: true,
        ..Env::default()
    };
    bencher.bench(|| interp(&compiled.program, &env).expect("the program resolves"));
}

/// The same program under the default env: every preference layer is
/// the quiet miss a media query that does not hold is — the miss path
/// beside the fire path.
#[divan::bench]
fn interp_preferences_default_env(bencher: Bencher) {
    let cache = DynCache::new();
    let compiled = cache
        .compile(PREF_CLASSES, &Config::default())
        .expect("non-empty input compiles");
    let env = Env::default();
    bencher.bench(|| interp(&compiled.program, &env).expect("the program resolves"));
}

/// The media-feature remainder's walk (the v4.1 additions' tier): the
/// two facts answer their layers the same env-field read carries — the
/// bench records the remainder beside the family's own walk.
#[divan::bench]
fn interp_preferences_remainder(bencher: Bencher) {
    let cache = DynCache::new();
    let compiled = cache
        .compile("inverted-colors:p-8 noscript:mt-2", &Config::default())
        .expect("non-empty input compiles");
    let env = Env {
        inverted_colors: true,
        ..Env::default()
    };
    bencher.bench(|| interp(&compiled.program, &env).expect("the program resolves"));
}
