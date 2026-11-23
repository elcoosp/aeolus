//! Lexer microbenchmarks (divan, stable) — the cold paths the compile
//! pipeline hits per class token: variant segmentation, the full
//! candidate parse, the whole-string sweep over a realistic utility,
//! and the crate's ONE alloc-bearing read (`variant_prefix` — the merge
//! key the `cn()` engine keys on, the `String` the `no_std + alloc`
//! face carries; plan §3.2's crate-table note).

use aeolus_syntax::{parse_candidate, parse_class_string};
use divan::Bencher;

fn main() {
    divan::main();
}

/// One bare utility — the fastest path.
#[divan::bench]
fn lex_bare(bencher: Bencher) {
    bencher.bench(|| parse_candidate("p-4"));
}

/// The stacked-variant, arbitrary-value, modifier corner the plan §1
/// quotes (`dark:group-hover:bg-primary/90`).
#[divan::bench]
fn lex_stacked(bencher: Bencher) {
    bencher.bench(|| parse_candidate("dark:group-hover:bg-primary/90"));
}

/// The calc-arbitrary scanner corner (`w-[calc(100%/3)]` — brackets and
/// slashes interleave).
#[divan::bench]
fn lex_calc_arbitrary(bencher: Bencher) {
    bencher.bench(|| parse_candidate("w-[calc(100%/3)]"));
}

/// A whole realistic class string (the shadcn card surface), whitespace
/// split + per-token lexing + error bookkeeping.
#[divan::bench]
fn lex_class_string(bencher: Bencher) {
    let class = "flex flex-col gap-4 rounded-xl border p-6 shadow-sm \
                 hover:shadow-md focus-visible:ring-2 md:p-8 dark:bg-gray-900";
    bencher.bench(|| parse_class_string(class));
}

/// The merge-key join (`Candidate::variant_prefix`) over the stacked
/// corner — the crate's only `String` allocation, priced here (the
/// `no_std + alloc` face's heap touch, the plan §3.2 note's own cost).
#[divan::bench]
fn variant_prefix_join(bencher: Bencher) {
    let candidate =
        parse_candidate("dark:group-hover:bg-primary/90").expect("stacked corner lexes");
    bencher.bench(|| candidate.variant_prefix());
}
