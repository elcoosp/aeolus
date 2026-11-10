//! The static-data emit arm's microbenchmarks (divan, stable — the
//! workspace bench idiom): the plan §3.1 emit (b) artifact cost —
//! freezing a compiled program into the byte-stable `static` item —
//! over the same shadcn Button program the other arm benches key on.
//! The text IS the deliverable here: one String build over the interned
//! layers, one `format!` per declaration.
//!
//! Numbers land in `reports/08-benchmarks/BENCHMARKS.md` when the gate
//! environment runs them (see `reports/10-tdd-evidence/17-emit-arms.md`
//! for the red-to-green record).

use aeolus_compile::{Config, DynCache, compile_theme, emit_static};
use aeolus_theme::{ThemeSource, TokenTable};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The shadcn v4 Button table verbatim (base + default variant + default
/// size) — the same 33-class string every arm's benches key on.
const BUTTON_CLASSES: &str = "inline-flex shrink-0 items-center justify-center gap-2 rounded-md text-sm font-medium whitespace-nowrap transition-all outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50 aria-invalid:border-destructive aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4 bg-primary text-primary-foreground hover:bg-primary/90 h-9 px-4 py-2 has-[>svg]:px-3";

/// Compile once (outside the measured loop) — the emitter's input is
/// the interned program.
fn button_program() -> aeolus_compile::Compiled {
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    let table = TokenTable::runtime(&source).expect("default-seeded source builds");
    let cache = DynCache::new();
    cache
        .compile_theme(BUTTON_CLASSES, &table, &Config::default())
        .expect("non-empty input compiles")
}

/// The static-data emit of the Button program: the artifact cost.
#[divan::bench]
fn emit_static_button(bencher: Bencher) {
    let compiled = button_program();
    bencher.bench(|| emit_static(&compiled.program));
}

/// The uncached floor beside it: the compile the static arm freezes
/// (the same floor the compile-seam benches measure — here for the
/// emit-vs-compile ratio in one place).
#[divan::bench]
fn compile_uncached_button(bencher: Bencher) {
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    let table = TokenTable::runtime(&source).expect("default-seeded source builds");
    bencher.bench(|| {
        compile_theme(BUTTON_CLASSES, &table, &Config::default()).expect("non-empty input compiles")
    });
}
