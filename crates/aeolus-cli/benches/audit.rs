//! The `audit` command's microbenchmarks (divan, stable — the workspace
//! bench idiom): the §5.1 capability-audit face's cost split. Two arms
//! pin the shape: the themed per-string compile+audit walk over the
//! Button slice's real shape (the report face's steady state — every
//! row a verdict, the ring family's polyfill half included) and the
//! refusal-heavy battery (every observed longhand answers through the
//! generated table's boundary reason).

use aeolus_cli::audit_string;
use aeolus_theme::{ThemeSource, TokenTable};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The lowered arm's input: the same shadcn v4 Button slice the CLI
/// pipeline benchmarks key on (all classes lower, the ring family rides
/// its polyfill tier).
const BUTTON: &str = "inline-flex shrink-0 items-center justify-center gap-2 rounded-md text-sm font-medium whitespace-nowrap outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50 bg-primary text-primary-foreground hover:bg-primary/90 h-9 px-4 py-2";

/// The refused arm's input: one class per documented boundary (the
/// z-index and blur refusals the generated table names, beside a native
/// shape) — the worst-case read: every row a verdict plus a reason.
const REFUSALS: &str = "z-10 order-4 blur-md drop-shadow-xl p-4 flex";

fn table() -> TokenTable {
    let source = ThemeSource::shadcn().expect("embedded palettes parse");
    TokenTable::runtime(&source).expect("default-seeded source builds")
}

#[divan::bench]
fn audit_button_themed(bencher: Bencher) {
    let tokens = table();
    bencher.bench(|| audit_string(BUTTON, &tokens).expect("non-empty input compiles"));
}

#[divan::bench]
fn audit_refusals_themed(bencher: Bencher) {
    let tokens = table();
    bencher.bench(|| audit_string(REFUSALS, &tokens).expect("non-empty input compiles"));
}
