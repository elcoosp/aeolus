//! The CSS emit arm's microbenchmarks (divan, stable — the workspace
//! bench idiom): the pipeline's emit stage per class string, themeless
//! and themed, plus the boundary path the linter hits. The IR form's
//! numbers live in `benches/compile.rs`; these are the same inputs
//! through the `Target::Css` renderer, so the two artifact forms'
//! costs stay comparable gate over gate.
//!
//! The bench bodies `expect` on the emit: a non-empty class string
//! always produces a sheet (only the empty input hard-fails).
//!
//! Numbers land in `reports/08-benchmarks/BENCHMARKS.md` when the gate
//! environment runs them.

use aeolus_compile::{emit_css, emit_css_theme, Config};
use aeolus_theme::{ThemeSource, TokenTable};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The shadcn v4 Button table verbatim (base + default variant + default
/// size) — the same 33-class string the compile benches key on, with
/// colour tokens riding the honest degradation path (themeless).
const BUTTON_CLASSES: &str = "inline-flex shrink-0 items-center justify-center gap-2 rounded-md text-sm font-medium whitespace-nowrap transition-all outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50 aria-invalid:border-destructive aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4 bg-primary text-primary-foreground hover:bg-primary/90 h-9 px-4 py-2 has-[>svg]:px-3";

/// The themed variant of the Button surface: the same string with the
/// runtime token table supplied (T2) — the `var()`/`color-mix` render
/// path the web-facing consumers ride.
#[divan::bench]
fn emit_css_button_surface_runtime_cold(bencher: Bencher) {
    let source = ThemeSource::shadcn_dark().expect("the embedded shadcn source parses");
    let tokens = TokenTable::runtime(&source).expect("the pair surface fits the u16 slot");
    bencher.bench(|| {
        emit_css_theme(BUTTON_CLASSES, &tokens, &Config::default())
            .expect("non-empty input renders")
    });
}

/// One utility, no variants — the emit stage's floor (lex, one lookup,
/// one 4-decl render, one rule).
#[divan::bench]
fn emit_css_single_utility_cold(bencher: Bencher) {
    bencher.bench(|| emit_css("p-4", &Config::default()).expect("non-empty input renders"));
}

/// The typed-miss cold path: the whole string refuses, the sheet carries
/// only diagnostics — the linter's cost.
#[divan::bench]
fn emit_css_unknown_root_cold(bencher: Bencher) {
    bencher.bench(|| {
        emit_css("not-a-utility", &Config::default()).expect("non-empty input renders")
    });
}

/// The responsive + variant surface: the selector build, the escaping
/// and the media grouping in one measured pass.
#[divan::bench]
fn emit_css_variant_media_cold(bencher: Bencher) {
    bencher.bench(|| {
        emit_css("hover:focus:p-0 group-hover/menu:p-4 md:hidden", &Config::default())
            .expect("non-empty input renders")
    });
}
