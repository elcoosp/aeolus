//! The "did you mean" seam's microbenchmarks (divan, stable — the
//! workspace bench idiom): the miss-path folds the CLI renders and the
//! editor offers — the registry surface's sweep over a keyword-table
//! miss (the audited-table walk the distance fold bounds), the variant
//! fold over the closed set and the composed answer the linter renders.
//! The fold runs on the cold miss path only; the numbers are the cost
//! a squiggle pays, never a hot-path tax.
//!
//! The bench bodies `expect` on the fixtures: the misses refuse (the
//! diagnostics exist), the answers are deterministic. Numbers land in
//! `reports/08-benchmarks/BENCHMARKS.md` when the gate environment runs
//! them.

use aeolus_compile::{
    Diagnostic, RegistrySpellings, SuggestionSurface, VariantSpellings, compile_theme, suggest_for,
    suggest_over,
};
use aeolus_theme::{ThemeSource, TokenTable};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The built-in shadcn bake (the engine's and the CLI's default table).
fn shadcn_table() -> TokenTable {
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    TokenTable::baked(&source).expect("the baked shadcn table builds")
}

/// The keyword-table miss's FIRST diagnostic (the fold's own input).
fn miss_diagnostic() -> Diagnostic {
    let table = shadcn_table();
    let compiled = compile_theme("justify-betwen flex-clo", &table, &Default::default())
        .expect("only the empty input errors");
    compiled
        .diagnostics
        .into_iter()
        .next()
        .expect("the miss refuses")
}

/// The registry surface's sweep: the audited-table walk over the
/// keyword miss (the bounds' own cost — the early abandon, the full
/// row sweep).
#[divan::bench]
fn registry_surface_sweep(bencher: Bencher) {
    bencher.bench(|| RegistrySpellings.nearest("justify-betwen", 2));
}

/// The variant surface's fold: the closed set over the offending
/// segment.
#[divan::bench]
fn variant_surface_fold(bencher: Bencher) {
    bencher.bench(|| VariantSpellings.nearest("havor", 2));
}

/// The composed answer: the diagnostic-keyed fold the CLI renders (the
/// registry sweep plus the variant-prefix composition).
#[divan::bench]
fn composed_miss_fold(bencher: Bencher) {
    let diagnostic = miss_diagnostic();
    bencher.bench(|| suggest_for(&diagnostic, None));
}

/// The host-composable raw fold: two surfaces over one miss (the
/// trait's own contract — the fold ranks, dedupes and caps). The
/// surfaces construct per iteration (unit structs, zero cost — the
/// fold itself is the measured walk).
#[divan::bench]
fn raw_fold_two_surfaces(bencher: Bencher) {
    bencher.bench(|| {
        let registry: &dyn SuggestionSurface = &RegistrySpellings;
        let variants: &dyn SuggestionSurface = &VariantSpellings;
        suggest_over("flex-clo", &[registry, variants])
    });
}
