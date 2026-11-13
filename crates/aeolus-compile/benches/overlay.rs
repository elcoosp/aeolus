//! The IR-level overlay seam's microbenchmarks (divan, stable — the
//! workspace bench idiom): the composition of two compiled programs
//! over the shadcn Button surface, the frozen⊕runtime shape (§3.6's
//! non-compile-time path), and the `cn` fold that delegate-walks the
//! same seam — the three shapes a host composes, gate over gate.
//!
//! The bench bodies `expect` on the composition: the demo inputs lower
//! clean (the same contract the suites pin).
//!
//! Numbers land in `reports/08-benchmarks/BENCHMARKS.md` when the gate
//! environment runs them.

use aeolus_compile::{Config, cn, compile, compile_theme, overlay};
use aeolus_theme::{ThemeSource, TokenTable};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The shadcn v4 Button table verbatim (base + default variant + default
/// size) — the same 33-class string the compile benches key on.
const BUTTON_CLASSES: &str = "inline-flex shrink-0 items-center justify-center gap-2 rounded-md text-sm font-medium whitespace-nowrap transition-all outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50 aria-invalid:border-destructive aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4 bg-primary text-primary-foreground hover:bg-primary/90 h-9 px-4 py-2 has-[>svg]:px-3";

/// The user override string: the sizing half a host composes at runtime.
const OVERRIDE_CLASSES: &str = "px-8 py-3 rounded-full text-base";

fn themeless(classes: &str) -> aeolus_compile::Compiled {
    compile(classes, &Config::default()).expect("non-empty input compiles")
}

fn themed(classes: &str) -> aeolus_compile::Compiled {
    let source = ThemeSource::shadcn_dark().expect("the embedded shadcn source parses");
    let tokens = TokenTable::runtime(&source).expect("the pair surface fits the u16 slot");
    compile_theme(classes, &tokens, &Config::default()).expect("non-empty input compiles")
}

/// The composition itself: two compiled Button-scale programs, the
/// base ⊕ override fold (the assembly: group, sort, last-wins, intern).
#[divan::bench]
fn overlay_button_surface(bencher: Bencher) {
    let base = themeless(BUTTON_CLASSES);
    let over = themeless(OVERRIDE_CLASSES);
    bencher.bench(|| overlay(&base.program, &over.program));
}

/// The frozen⊕runtime shape (§3.6): the themed Button program (the
/// macro's form) under a themeless override compile.
#[divan::bench]
fn overlay_frozen_under_runtime_override(bencher: Bencher) {
    let base = themed(BUTTON_CLASSES);
    let over = themeless(OVERRIDE_CLASSES);
    bencher.bench(|| overlay(&base.program, &over.program));
}

/// The `cn` fold that delegate-walks the same seam (the diagnostic
/// channel + the N-ary pairwise composition) — the cost delta against
/// the bare [`overlay_button_surface`] is the wrapper's own.
#[divan::bench]
fn cn_button_surface(bencher: Bencher) {
    let base = themeless(BUTTON_CLASSES);
    let over = themeless(OVERRIDE_CLASSES);
    bencher.bench(|| {
        let refs = [&base, &over];
        cn(&refs).expect("non-empty inputs merge")
    });
}
