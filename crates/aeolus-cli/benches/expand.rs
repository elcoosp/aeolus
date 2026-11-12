//! The `expand` command's microbenchmarks (divan, stable — the
//! workspace bench idiom): the interp arm's human-readable form, §3.2's
//! CLI row. Three arms pin the cost split: the themeless effective walk
//! (the keyword texts riding the winner attribution), the themed walk
//! (the tiered table resolving the token forms) and the keyword-heavy
//! render (every slot reading its row text through the `(cond, prop)`
//! annotation table).
//!
//! The benches measure the WALK the command drives (compile → interp →
//! annotation table), not the render's `writeln!` — the sinks are the
//! host's, the same split the `explain` bench keeps.

use aeolus_compile::{Config, Env, compile, compile_theme, interp_theme};
use aeolus_theme::{ThemeSource, TokenTable};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The lowered arm's input: the same shadcn v4 Button slice the CLI
/// pipeline benchmarks key on (all classes lower on the built-in
/// source, the themeless token boundary untouched).
const BUTTON: &str = "inline-flex shrink-0 items-center justify-center gap-2 rounded-md text-sm font-medium whitespace-nowrap outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50 bg-primary text-primary-foreground hover:bg-primary/90 h-9 px-4 py-2";

/// The keyword-heavy arm's input: every slot is a row-relative `Val::Kw`
/// the render reads its row text for.
const KEYWORDS: &str = "hidden block flex-col items-center justify-center uppercase font-bold rounded-md text-sm whitespace-nowrap";

#[divan::bench]
fn expand_themeless_effective(bencher: Bencher) {
    let compiled = compile(BUTTON, &Config::default()).expect("non-empty input compiles");
    bencher.bench(|| {
        compiled
            .interp(&Env::default())
            .expect("no theme forms in the themeless walk")
    });
}

#[divan::bench]
fn expand_themed_effective(bencher: Bencher) {
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    let tokens = TokenTable::runtime(&source).expect("default-seeded source builds");
    let compiled =
        compile_theme(BUTTON, &tokens, &Config::default()).expect("non-empty input compiles");
    bencher.bench(|| {
        interp_theme(&compiled.program, &tokens, &Env::default())
            .expect("the themed walk resolves the tokens")
    });
}

#[divan::bench]
fn expand_keyword_slots(bencher: Bencher) {
    let compiled = compile(KEYWORDS, &Config::default()).expect("non-empty input compiles");
    bencher.bench(|| compiled.interp(&Env::default()).expect("no theme forms"));
}
