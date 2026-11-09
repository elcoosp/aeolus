//! Registry-lookup microbenchmarks (divan, stable) — the read paths the
//! P2 compile stage hits per utility body: the silent longest-prefix
//! lookup, the typed detailed seam on a hit, and the two typed misses
//! (unknown root; shape rejection — the linter's "did you mean" cold
//! path with its tried-labels allocation).

use aeolus_data::registry::{lookup, lookup_detailed};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The silent hot path: a cache-shaped hit (`p-4`).
#[divan::bench]
fn lookup_hit(bencher: Bencher) {
    bencher.bench(|| lookup("p-4").for_each(drop));
}

/// The typed seam on the same hit: identical rows, plus the Result
/// shell — the seam's WHY must cost nothing on success.
#[divan::bench]
fn lookup_detailed_hit(bencher: Bencher) {
    bencher.bench(|| lookup_detailed("p-4").unwrap_or_default());
}

/// The unknown-root miss: the typed error allocation (linter cold path).
#[divan::bench]
fn lookup_detailed_unknown_root(bencher: Bencher) {
    bencher.bench(|| {
        let _ = lookup_detailed("not-a-utility");
    });
}

/// The shape-rejection miss: the root scan plus the tried-labels
/// allocation (`p-red` under root `p`, shapes spacing + arbitrary).
#[divan::bench]
fn lookup_detailed_shape_rejected(bencher: Bencher) {
    bencher.bench(|| {
        let _ = lookup_detailed("p-red");
    });
}
