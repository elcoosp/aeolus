//! The `explain` command's microbenchmarks (divan, stable — the
//! workspace bench idiom): the §5.5 tooling-parity path, class strings
//! through the Target::Rust arm. Three arms pin the cost split: the
//! themeless emit (every lowered class, tokens annotated with their
//! row/table names), the themed emit (the tiered table lowering colour
//! tokens) and the diagnostics path (every miss reported, none lower).

use aeolus_compile::{Config, emit_rust, emit_rust_theme};
use aeolus_theme::{ThemeSource, TokenTable};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The lowered arm's input: the same shadcn v4 Button slice the CLI
/// pipeline benchmarks key on (all classes lower, the themeless token
/// boundary untouched).
const BUTTON: &str = "inline-flex shrink-0 items-center justify-center gap-2 rounded-md text-sm font-medium whitespace-nowrap outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50 bg-primary text-primary-foreground hover:bg-primary/90 h-9 px-4 py-2";

#[divan::bench]
fn explain_themeless(bencher: Bencher) {
    bencher.bench(|| emit_rust(BUTTON, &Config::default()).expect("non-empty input compiles"));
}

#[divan::bench]
fn explain_themed(bencher: Bencher) {
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    let tokens = TokenTable::runtime(&source).expect("default-seeded source builds");
    bencher.bench(|| {
        emit_rust_theme(BUTTON, &tokens, &Config::default()).expect("non-empty input compiles")
    });
}

#[divan::bench]
fn explain_reported(bencher: Bencher) {
    // The diagnostics path: one real class, one miss — the honest
    // per-class report the command's exit 1 carries.
    bencher.bench(|| {
        emit_rust("p-4 definitely-not-a-class-xyz", &Config::default())
            .expect("non-empty input compiles")
    });
}
