//! Lexer microbenchmarks (divan, stable) — the cold paths the compile
//! pipeline hits per class token: variant segmentation, the full
//! candidate parse, and the whole-string sweep over a realistic utility.

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
