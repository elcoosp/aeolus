//! The rust-tokens emit arm's microbenchmarks (divan, stable — the
//! workspace bench idiom): the plan §3.1 emit (a) artifact cost —
//! walking the SAME shadcn Button program the other arm benches key on
//! and rendering one annotated fragment per lowered class. The walk is
//! the compile seam's own (lex → lookup → lower → render), so the
//! compile floor rides beside it for the emit-vs-compile ratio in one
//! place.
//!
//! Numbers land in `reports/08-benchmarks/BENCHMARKS.md` when the gate
//! environment runs them (see `reports/10-tdd-evidence/19-rust-tokens-arm.md`
//! for the red-to-green record).

use aeolus_compile::{Config, DynCache, compile_theme, emit_rust_theme};
use aeolus_theme::{ThemeSource, TokenTable};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The shadcn v4 Button table verbatim (base + default variant + default
/// size) — the same 33-class string every arm's benches key on.
const BUTTON_CLASSES: &str = "inline-flex shrink-0 items-center justify-center gap-2 rounded-md text-sm font-medium whitespace-nowrap transition-all outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50 aria-invalid:border-destructive aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4 bg-primary text-primary-foreground hover:bg-primary/90 h-9 px-4 py-2 has-[>svg]:px-3";

/// The runtime-tier table over the shadcn light+dark source.
fn runtime_table() -> TokenTable {
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    TokenTable::runtime(&source).expect("default-seeded source builds")
}

/// The rust-tokens emit of the Button string: the full walk + the
/// annotated fragment render — the emit (a) artifact cost.
#[divan::bench]
fn emit_rust_button(bencher: Bencher) {
    let table = runtime_table();
    bencher.bench(|| {
        emit_rust_theme(BUTTON_CLASSES, &table, &Config::default())
            .expect("non-empty input renders")
    });
}

/// The uncached floor beside it: the compile the fragments render from
/// (the same floor the compile-seam benches measure).
#[divan::bench]
fn compile_uncached_button(bencher: Bencher) {
    let table = runtime_table();
    bencher.bench(|| {
        compile_theme(BUTTON_CLASSES, &table, &Config::default()).expect("non-empty input compiles")
    });
}

/// The cached compile under the emit (the DynCache path the other arm
/// benches measure — here for the ratio against the fragment render).
#[divan::bench]
fn compile_cached_button(bencher: Bencher) {
    let table = runtime_table();
    let cache = DynCache::new();
    bencher.bench(|| {
        cache
            .compile_theme(BUTTON_CLASSES, &table, &Config::default())
            .expect("non-empty input compiles")
    });
}
