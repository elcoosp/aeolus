//! The shadow tier's plan benchmarks (divan, stable — the workspace
//! bench idiom): the preset resolution through the oracle's own theme
//! reader and the slot write, the recolour interaction's compose, and
//! the differential walk beside the oracle's own resolve (the pair the
//! battery pins).

use divan::Bencher;

use aeolus_compile::{Config, emit_plan_theme};
use aeolus_core::{Resolver, TailwindTheme};
use aeolus_theme::{ThemeSource, TokenTable};

fn tokens() -> TokenTable {
    let source = ThemeSource::from_css(aeolus_core::TAILWIND_DEFAULT_THEME, None)
        .expect("the audited theme parses");
    TokenTable::baked(&source).expect("the baked table")
}

/// The preset walk: `shadow-lg` through lex → fold → lookup → value
/// pass → the theme-reader resolution → the parsed band slot write.
#[divan::bench]
fn plan_shadow_preset(bencher: Bencher) {
    let table = tokens();
    bencher.bench(|| {
        emit_plan_theme("shadow-lg", &table, &Config::default()).expect("non-empty input emits")
    });
}

/// The recolour interaction: the preset and the `shadow-{color}`
/// recolour composing in the oracle's own pass order (the two-candidate
/// bucket walk the battery pins).
#[divan::bench]
fn plan_shadow_recolour(bencher: Bencher) {
    let table = tokens();
    bencher.bench(|| {
        emit_plan_theme("shadow-lg shadow-red-500", &table, &Config::default())
            .expect("non-empty input emits")
    });
}

/// The differential pair's V1 half: the same string through the bridge
/// walk (the oracle's own `resolve` runs beside it in the battery —
/// this is the arm the plan rides).
#[divan::bench]
fn plan_shadow_string_cascade(bencher: Bencher) {
    let table = tokens();
    bencher.bench(|| {
        emit_plan_theme(
            "shadow-sm shadow-lg shadow-none shadow-red-500",
            &table,
            &Config::default(),
        )
        .expect("non-empty input emits")
    });
}

/// The oracle's own resolve of the same cascade — the comparison the
/// differential battery runs, for the record.
#[divan::bench]
fn oracle_shadow_resolve(bencher: Bencher) {
    let theme = Box::leak(Box::new(TailwindTheme::default()));
    let resolver = Resolver::new(theme);
    bencher.bench(|| {
        resolver
            .resolve("shadow-sm shadow-lg shadow-none shadow-red-500")
            .expect("the oracle resolves the shadow cascade")
    });
}

fn main() {
    divan::main();
}
