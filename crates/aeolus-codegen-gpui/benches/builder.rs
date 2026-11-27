//! The builder-chain emitter's microbenchmarks (divan, stable — the
//! workspace bench idiom): the per-expansion cost the `tw_builder!`
//! face adds beside the static emit's own gate bench (`gate.rs`) — one
//! `emit_builder` walk over a frozen program (the flat gate pass, the
//! render match per declaration, the closure assembly). The emitted
//! CHAIN's application cost (the §4 target face — the chain against
//! the walk against the hand-written builder calls) is the runtime
//! bench's (`crates/aeolus-gpui/benches/builder.rs`).
//!
//! Numbers land in `reports/08-benchmarks/BENCHMARKS.md` when the gate
//! environment runs them.

use aeolus_codegen_gpui::{audit_builder, emit_builder};
use aeolus_compile::{Config, compile_theme, keyword_alignment, keyword_texts};
use aeolus_theme::{ThemeSource, TokenTable};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The shadcn Button's own base-cascade shape — the chain-expressible
/// slice of the realistic string (the state layer is the chain gate's
/// typed refusal, so the frozen program here carries the base layers
/// only).
const BUTTON_CLASSES: &str =
    "inline-flex items-center justify-center gap-2 rounded-md bg-primary text-primary-foreground";

fn frozen_button() -> (aeolus_ir::Program, Vec<&'static str>) {
    let source = ThemeSource::shadcn().expect("embedded palette parses");
    let tokens = TokenTable::baked(&source).expect("the baked table builds");
    let compiled = compile_theme(BUTTON_CLASSES, &tokens, &Config::default())
        .expect("non-empty input compiles");
    assert!(
        compiled.diagnostics.is_empty(),
        "{:?}",
        compiled.diagnostics
    );
    let classes = keyword_texts(BUTTON_CLASSES, &Config::default()).expect("annotations resolve");
    let keywords = keyword_alignment(&compiled, &classes);
    (compiled.program, keywords)
}

#[divan::bench]
fn builder_gate(bencher: Bencher) {
    // The macro face's gate: `audit_builder` over the frozen program —
    // the same flat walk the static gate's bench prices, the chain's
    // own verdicts beside the static gate's.
    let (program, keywords) = frozen_button();
    bencher.bench(|| audit_builder(&program, &keywords));
}

#[divan::bench]
fn builder_emit(bencher: Bencher) {
    // The full emit: gate + render + closure assembly — the
    // per-expansion cost `tw_builder!` adds to a build.
    let (program, keywords) = frozen_button();
    bencher.bench(|| emit_builder(&program, &keywords));
}
