//! The supports-variant seam's microbenchmarks (divan, stable — the
//! workspace bench idiom): the `supports-[…]` family's walk cost, the
//! seam's own slice of the interp arm's per-frame story. The built-in
//! table answers by a linear scan over a handful of condition texts
//! (one row per capability the target declares), so the per-layer
//! question is the walk's dominant term — the same shape the
//! descendant seam's bench records for its own family.
//!
//! The bench bodies `expect` on the interp: a compiled program resolves
//! under any env (errors are caller bugs — unknown ids, non-finite
//! alphas — and none of those can come out of the pipeline).
//!
//! Numbers land in `reports/08-benchmarks/BENCHMARKS.md` when the gate
//! environment runs them (see `reports/10-tdd-evidence/23-supports-seam.md`
//! for the red-to-green record).

use aeolus_compile::{Config, DynCache, Env, SupportsState, interp, interp_supports};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The capability facts the table answers for: the render target
/// supports the grid declaration probe, the selector function form and
/// a media-style range condition — the realistic per-target set (the
/// capabilities a card component's stylesheet actually declares).
const TARGET_FACTS: &[&str] = &["(display:grid)", "selector(:hover)", "(width_>_400px)"];

/// The Button string's supports half — the layers the seam answers in
/// the pipeline's real consumer, beside the fold's declaration and
/// function forms.
const SUPPORTS_CLASSES: &str =
    "supports-[display:grid]:p-8 supports-[selector(:hover)]:px-3 supports-[(width_>_400px)]:mt-4";

#[divan::bench]
fn interp_supports_button(bencher: Bencher) {
    let cache = DynCache::new();
    let compiled = cache
        .compile(SUPPORTS_CLASSES, &Config::default())
        .expect("non-empty input compiles");
    let table = SupportsState::new(TARGET_FACTS);
    let env = Env::default();
    bencher
        .bench(|| interp_supports(&compiled.program, &env, &table).expect("the program resolves"));
}

/// The same program through the SOURCELESS walk: every supports layer
/// is the reported exclusion — the note path's cost, for the record
/// (the plain `interp` the pre-seam walks ran).
#[divan::bench]
fn interp_sourceless_button(bencher: Bencher) {
    let cache = DynCache::new();
    let compiled = cache
        .compile(SUPPORTS_CLASSES, &Config::default())
        .expect("non-empty input compiles");
    let env = Env::default();
    bencher.bench(|| interp(&compiled.program, &env).expect("the program resolves"));
}
