//! The metamorphic seam's microbenchmarks (divan, stable — the
//! workspace bench idiom): the full three-walk check over the shadcn
//! Button surface (the same 33-class string the compile/CSS/interp
//! benches key on, so the checker's cost stays comparable gate over
//! gate), the themed walk over the same surface, and the
//! single-utility floor against the compile seam's own bench (the
//! checker runs compile + emit + interp + the per-candidate parts —
//! the ratio IS the verification overhead).
//!
//! The bench bodies `expect` on the check: a non-empty class string
//! always produces a report (only the empty input hard-fails), and the
//! corpus battery's agreement is the tests' contract, not the bench's.
//!
//! Numbers land in `reports/08-benchmarks/BENCHMARKS.md` when the gate
//! environment runs them.

use aeolus_compile::{Config, metamorphic_check, metamorphic_check_theme};
use aeolus_theme::{ThemeSource, TokenTable};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The shadcn v4 Button table verbatim (base + default variant + default
/// size) — the same 33-class string the compile/css/interp benches key
/// on: colour tokens ride the honest degradation path themeless, the
/// full lowering themed.
const BUTTON_CLASSES: &str = "inline-flex shrink-0 items-center justify-center gap-2 rounded-md text-sm font-medium whitespace-nowrap transition-all outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50 aria-invalid:border-destructive aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4 bg-primary text-primary-foreground hover:bg-primary/90 h-9 px-4 py-2 has-[>svg]:px-3";

/// The themeless three-walk check over the Button surface: compile +
/// CSS emit + interp + the per-candidate parts, the token boundaries
/// riding both diagnostics channels identically.
#[divan::bench]
fn check_button_themeless(bencher: Bencher) {
    let config = Config::default();
    bencher.bench(|| metamorphic_check(BUTTON_CLASSES, &config).expect("non-empty input"));
}

/// The themed three-walk check over the Button surface: the tiered
/// tables lower the colour tokens, the interp resolves the probe's
/// bytes, the runtime `var()` references ride the typed notes.
#[divan::bench]
fn check_button_theme(bencher: Bencher) {
    let config = Config::default();
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    let table = TokenTable::baked(&source).expect("default-seeded source builds");
    bencher.bench(|| {
        metamorphic_check_theme(BUTTON_CLASSES, &table, &config).expect("non-empty input")
    });
}

/// The single-utility floor: the checker's walk over one spacing class
/// (base situation only) — the verification overhead's own floor, the
/// compile bench's single-utility arm's sibling.
#[divan::bench]
fn check_single_utility(bencher: Bencher) {
    let config = Config::default();
    bencher.bench(|| metamorphic_check("p-4", &config).expect("non-empty input"));
}
