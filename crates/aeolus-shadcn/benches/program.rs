//! The program tables' microbenchmarks (divan, stable — the workspace
//! bench idiom): the F6 story's two arms side by side. The STRING path
//! builds a fresh `String` per call (`button_classes` runs `cn()` over
//! the three verbatim slices) and re-resolves it at the consumer; the
//! PROGRAM path is the table hit — one closed-world cell lookup, the
//! interned `StaticProgram` out, no allocation, no re-resolve.
//!
//! The per-axis misses (the first call per combo) are the compile
//! itself — the arms below pin the STEADY state (the per-frame cost a
//! render loop actually repeats).
//!
//! (`cargo test -p aeolus-shadcn --benches` is this bench's run: the
//! git-pinned iced chain refuses release builds without a renderer
//! feature — the pre-existing reason this crate carried no benches —
//! while the test profile compiles it headless; the arms' numbers are
//! debug-profile, the RATIO is the story.)

use aeolus_rt::{ThemeSource, TokenTable};
use aeolus_shadcn::classes::{ButtonSize, ButtonVariant, button_classes};
use aeolus_shadcn::program::ButtonPrograms;
use divan::Bencher;

fn main() {
    divan::main();
}

/// The session's token table (the built-in shadcn bake).
fn baked_tokens() -> TokenTable {
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    TokenTable::baked(&source).expect("the T0/T1 bake succeeds")
}

#[divan::bench]
fn string_path_per_call(bencher: Bencher) {
    // F6 as measured: the String build (cn() over base + variant +
    // size) a consumer re-resolves per call.
    bencher.bench(|| button_classes(ButtonVariant::Default, ButtonSize::Default));
}

#[divan::bench]
fn program_table_hit(bencher: Bencher) {
    // The const-table seam: one cell lookup, the interned program out.
    let tokens = baked_tokens();
    let tables = ButtonPrograms::new(&tokens);
    bencher.bench_local(|| tables.program(ButtonVariant::Default, ButtonSize::Default));
}
