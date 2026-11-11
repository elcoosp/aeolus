//! Registry-lookup microbenchmarks (divan, stable) — the read paths the
//! P2 compile stage hits per utility body: the silent longest-prefix
//! lookup, the typed detailed seam on a hit, and the two typed misses
//! (unknown root; shape rejection — the linter's "did you mean" cold
//! path with its tried-labels allocation).

use aeolus_data::registry::{lookup, lookup_detailed, lookup_linear};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The silent hot path: a cache-shaped hit (`p-4`) — since the
/// generator story landed, this walks the GENERATED roots index (the
/// phf probe) instead of the row scan; the linear oracle below is the
/// before number.
#[divan::bench]
fn lookup_hit(bencher: Bencher) {
    bencher.bench(|| lookup("p-4").for_each(drop));
}

/// The pre-index linear scan on the same hit — the documented
/// differential oracle, and the delta the index buys (the scan reads
/// all 178 rows; the probe reads one hash slot).
#[divan::bench]
fn lookup_linear_hit(bencher: Bencher) {
    bencher.bench(|| lookup_linear("p-4").for_each(drop));
}

/// The indexed path on a multi-dash body (`rounded-tl-lg`): three
/// prefix probes longest-first, the winning root's rows shape-filtered
/// — the disambiguation case the prefix rule exists for.
#[divan::bench]
fn lookup_multi_dash_hit(bencher: Bencher) {
    bencher.bench(|| lookup("rounded-tl-lg").for_each(drop));
}

/// The linear oracle on the same multi-dash body (the before number).
#[divan::bench]
fn lookup_linear_multi_dash_hit(bencher: Bencher) {
    bencher.bench(|| lookup_linear("rounded-tl-lg").for_each(drop));
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

/// The linear oracle on the same shape-rejection miss — the before
/// number for the typed miss path (the root scan is the dominant cost;
/// the tried-labels allocation is unchanged).
#[divan::bench]
fn lookup_linear_shape_rejected(bencher: Bencher) {
    bencher.bench(|| lookup_linear("p-red").for_each(drop));
}
