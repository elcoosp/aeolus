//! The program lane's pseudo-target face's microbenchmarks (divan,
//! stable — the workspace bench idiom): the [`PseudoTarget`]'s own
//! cascade folded from the frozen slices (the `interp_pseudo` contract
//! on the program lane). Compile ONCE (the macro's freeze), walk per
//! render target — the interesting numbers are the target's own fire
//! path (the selection cascade over the shadcn Button string's pseudo
//! layers) beside the quiet path (a target the program does not name)
//! and the themeless refusal report (the runtime-tier slots walking
//! with no source).
//!
//! Numbers land in `reports/08-benchmarks/BENCHMARKS.md` when the gate
//! environment runs them.

use aeolus_compile::{
    Config, PseudoTarget, keyword_alignment, keyword_texts, program_style_pseudo,
    program_style_pseudo_theme,
};
use aeolus_theme::{ThemeSource, TokenTable};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The shadcn v4 Button table verbatim (the other program-lane
/// benches' own string) grown by the pseudo-element render targets the
/// fold answers: the selection recolour, the placeholder text, the
/// marker and the backdrop — one corpus, several targets.
const CORPUS: &str = "inline-flex items-center justify-center gap-2 rounded-md bg-primary text-primary-foreground hover:bg-primary/90 h-9 px-4 py-2 selection:bg-primary/90 placeholder:text-red-500 marker:text-primary-foreground backdrop:opacity-50";

/// The baked-tier table (pairs + literals): the program's colour
/// tokens bake to literals the themeless fold reads directly.
fn baked_table() -> TokenTable {
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    TokenTable::baked(&source).expect("default-seeded source builds")
}

/// The runtime-tier table (T2): the colour tokens stay `Tok` slots the
/// themed fold resolves through the source — and the themeless fold
/// refuses into its report.
fn runtime_table() -> TokenTable {
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    TokenTable::runtime(&source).expect("default-seeded source builds")
}

/// Compile once and freeze into the two slices the macro emits (the
/// tests' own `frozen` shape).
fn frozen(table: &TokenTable) -> (&'static [aeolus_ir::Layer], &'static [&'static str]) {
    let config = Config::default();
    let compiled =
        aeolus_compile::compile_theme(CORPUS, table, &config).expect("non-empty input compiles");
    let annotations = keyword_texts(CORPUS, &config).expect("non-empty input annotates");
    let keywords = keyword_alignment(&compiled, &annotations);
    (
        compiled.program.layers,
        Box::leak(keywords.into_boxed_slice()),
    )
}

/// The selection target's own cascade over the baked program: the
/// fold fires the target's layers and composes the slot.
#[divan::bench]
fn pseudo_fold_selection_fires(bencher: Bencher) {
    let (layers, keywords) = frozen(&baked_table());
    bencher.bench(|| program_style_pseudo(PseudoTarget::Selection, layers, keywords));
}

/// The themed selection fold: the runtime-tier slots resolve through
/// the source (T2's one load + one multiply on the target's own
/// cascade).
#[divan::bench]
fn pseudo_fold_selection_themed(bencher: Bencher) {
    let (layers, keywords) = frozen(&runtime_table());
    let table = runtime_table();
    let env = aeolus_compile::Env::default();
    bencher.bench(|| {
        program_style_pseudo_theme(PseudoTarget::Selection, layers, keywords, &table, &env)
    });
}

/// The placeholder target over the same program: the sibling target's
/// own layers fire, the selection layers stay silent (the
/// render-target contract's condition-equality verdict).
#[divan::bench]
fn pseudo_fold_placeholder_fires(bencher: Bencher) {
    let (layers, keywords) = frozen(&baked_table());
    bencher.bench(|| program_style_pseudo(PseudoTarget::Placeholder, layers, keywords));
}

/// A target the corpus does not name: the quiet path — the walk
/// answers the empty cascade at the same front-to-back pass cost.
#[divan::bench]
fn pseudo_fold_quiet_target(bencher: Bencher) {
    let (layers, keywords) = frozen(&baked_table());
    bencher.bench(|| program_style_pseudo(PseudoTarget::FirstLetter, layers, keywords));
}

/// The runtime-tier program walked with NO source: every theme form
/// refuses per longhand into the walk's report (the honest slow lane's
/// boundary, the report path the backends render).
#[divan::bench]
fn pseudo_fold_themeless_refusal(bencher: Bencher) {
    let (layers, keywords) = frozen(&runtime_table());
    bencher.bench(|| program_style_pseudo(PseudoTarget::Selection, layers, keywords));
}
