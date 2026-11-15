//! The `plan` command's microbenchmarks (divan, stable — the workspace
//! bench idiom): the §7 migration seam's consumer path, class strings
//! through the Target::Plan arm. Three arms pin the cost split: the
//! themeless bridge (every lowered class, token boundaries honest), the
//! themed bridge (the baked table's colours convert into the v0.x Oklab
//! slots) and the note path (the slot-boundary reports ride stderr).

use aeolus_compile::{Config, emit_plan, emit_plan_theme};
use aeolus_theme::{ThemeSource, TokenTable};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The bridged arm's input: the same shadcn v4 Button slice the CLI
/// pipeline benchmarks key on (all classes lower; the boundary notes
/// carry the slot-less longhands honestly).
const BUTTON: &str = "inline-flex shrink-0 items-center justify-center gap-2 rounded-md text-sm font-medium whitespace-nowrap outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50 bg-primary text-primary-foreground hover:bg-primary/90 h-9 px-4 py-2";

#[divan::bench]
fn plan_themeless(bencher: Bencher) {
    bencher.bench(|| emit_plan(BUTTON, &Config::default()).expect("non-empty input compiles"));
}

#[divan::bench]
fn plan_themed(bencher: Bencher) {
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    let tokens = TokenTable::baked(&source).expect("default-seeded source builds");
    bencher.bench(|| {
        emit_plan_theme(BUTTON, &tokens, &Config::default()).expect("non-empty input compiles")
    });
}

#[divan::bench]
fn plan_reported(bencher: Bencher) {
    // The diagnostics path: one real class, one miss — the honest
    // per-class report the command's stderr carries.
    bencher.bench(|| {
        emit_plan("p-4 definitely-not-a-class-xyz", &Config::default())
            .expect("non-empty input compiles")
    });
}
