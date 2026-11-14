//! The capability gate's microbenchmarks (divan, stable — the
//! workspace bench idiom): the walk the macro surface runs per `tw!`
//! expansion, `audit` over a frozen program. The per-expansion cost
//! the gate adds is the number this bench exists for: one
//! `unsupported_reason` ask per declaration (a small match wall), the
//! same interned-program read the emit arm keys on.
//!
//! Numbers land in `reports/08-benchmarks/BENCHMARKS.md` when the gate
//! environment runs them.

use aeolus_codegen_gpui::audit;
use aeolus_compile::{Config, compile_theme, keyword_alignment, keyword_texts};
use aeolus_theme::{ThemeSource, TokenTable};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The shadcn Button's own class string — the realistic per-expansion
/// shape (layout + colour + typography + the ring polyfill, one state
/// layer), everything expressible, every declaration still gated.
const BUTTON_CLASSES: &str = "inline-flex items-center justify-center gap-2 rounded-md bg-primary \
     text-primary-foreground hover:bg-primary/90";

fn frozen_button() -> (aeolus_ir::Program, Vec<&'static str>) {
    let source = ThemeSource::shadcn().expect("embedded palette parses");
    let tokens = TokenTable::baked(&source).expect("the baked table builds");
    let compiled = compile_theme(BUTTON_CLASSES, &tokens, &Config::default())
        .expect("non-empty input compiles");
    let annotations = keyword_texts(BUTTON_CLASSES, &Config::default()).expect("annotates");
    let keywords = keyword_alignment(&compiled, &annotations);
    (compiled.program, keywords)
}

/// The clean audit: every declaration walks the gates and passes (the
/// hot path — the cost every `tw!` expansion pays).
#[divan::bench]
fn audit_clean(bencher: Bencher) {
    let (program, keywords) = frozen_button();
    bencher.bench(|| audit(&program, &keywords));
}
