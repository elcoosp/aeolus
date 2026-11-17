//! The program-lane walk's microbenchmarks (divan, stable — the
//! workspace bench idiom): the §7 re-target's style fold over the same
//! shadcn Button string the interp-arm benches key on, grown by the
//! full-probe seam. The per-frame loop's cost is the number this lane
//! exists for: compile ONCE (the macro's freeze), walk per env flip —
//! the interesting numbers are the sourceless walk's report path (the
//! pre-seam shape every plan-lane backend still rides) beside the
//! probe-wired fire path (the seams answering, the same `decide`
//! verdicts the interp arm pins).
//!
//! Numbers land in `reports/08-benchmarks/BENCHMARKS.md` when the gate
//! environment runs them.

use aeolus_compile::{
    AttrBit, AttrKind, AttrState, Config, ContainerBound, ContainerState, DescendantFact,
    DescendantKind, DescendantState, Env, StructFacts, SupportsState, keyword_alignment,
    keyword_texts, program_style, program_style_probes_containers_struct_descendants_supports,
    program_style_with,
};
use aeolus_theme::{ThemeSource, TokenTable};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The shadcn v4 Button table verbatim (the interp-arm benches' own
/// string) grown by the four probe families the widest form answers:
/// a named container, a descendant `has-*`, an `@supports` layer, the
/// structural head and an element-own attribute probe — one corpus,
/// every seam the walk can consult.
const CORPUS: &str = "inline-flex shrink-0 items-center justify-center gap-2 rounded-md text-sm font-medium bg-primary text-primary-foreground hover:bg-primary/90 h-9 px-4 py-2 disabled:opacity-50 @md/card:w-64 has-focus:opacity-40 supports-[display:grid]:bg-red-500 first:w-8 data-[state=open]:ring-2";

/// The baked-tier table (pairs + literals): the program carries
/// `ModePair` slots the walk resolves through the token seam.
fn baked_table() -> TokenTable {
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    TokenTable::baked(&source).expect("default-seeded source builds")
}

/// Compile once and freeze into the two slices the macro emits (the
/// tests' own `frozen` shape) — the walk's input is the interned
/// program plus the row texts, never the string.
fn frozen() -> (&'static [aeolus_ir::Layer], &'static [&'static str]) {
    let table = baked_table();
    let config = Config::default();
    let compiled =
        aeolus_compile::compile_theme(CORPUS, &table, &config).expect("non-empty input compiles");
    let annotations = keyword_texts(CORPUS, &config).expect("non-empty input annotates");
    let keywords = keyword_alignment(&compiled, &annotations);
    (
        compiled.program.layers,
        Box::leak(keywords.into_boxed_slice()),
    )
}

/// The resting env: only the base rule fires, every probe layer
/// reports unqueried on the sourceless walks.
fn resting_env() -> Env {
    Env::default()
}

/// The probe sources, every seam answering: the attribute bit the
/// corpus's data- layer folds on, the named card container above its
/// md boundary, the first-child facts, the focused-subtree fact and
/// the display:grid capability — the fire path's one-stop shop.
fn wired_probes() -> (
    AttrState<'static>,
    ContainerState<'static>,
    StructFacts,
    DescendantState<'static>,
    SupportsState<'static>,
) {
    (
        AttrState::new(&[AttrBit {
            kind: AttrKind::Data,
            name: "state",
            value: "open",
        }]),
        ContainerState::new(&[ContainerBound {
            name: "card",
            rem: 64.0,
        }]),
        StructFacts::new(0, 2),
        DescendantState::new(&[DescendantFact {
            kind: DescendantKind::Element,
            fragment: "*:focus",
        }]),
        SupportsState::new(&["(display:grid)"]),
    )
}

/// The sourceless walk, resting env: the pre-seam shape every
/// plan-lane backend rides today — the probe layers report
/// unqueried, the base rule still folds.
#[divan::bench]
fn walk_sourceless_resting(bencher: Bencher) {
    let (layers, keywords) = frozen();
    let env = resting_env();
    bencher.bench(|| program_style(layers, keywords, &env));
}

/// The attr+token walk (`program_style_with`): the plan-lane backends'
/// current wired ceiling — attribute probes and theme forms answer,
/// the container/descendant/supports layers keep reporting.
#[divan::bench]
fn walk_attr_wired(bencher: Bencher) {
    let (layers, keywords) = frozen();
    let (attrs, _, _, _, _) = wired_probes();
    let table = baked_table();
    let env = resting_env();
    bencher.bench(|| program_style_with(layers, keywords, &env, Some(&attrs), Some(&table)));
}

/// The full-probe walk, every seam firing: the named container, the
/// structural facts, the descendant fact and the supports condition
/// answer beside the attribute bit — the M1 fire path.
#[divan::bench]
fn walk_full_probe_wired(bencher: Bencher) {
    let (layers, keywords) = frozen();
    let (attrs, card, facts, focused, grid) = wired_probes();
    let table = baked_table();
    let env = resting_env();
    bencher.bench(|| {
        program_style_probes_containers_struct_descendants_supports(
            layers,
            keywords,
            &env,
            Some(&attrs),
            Some(&card),
            Some(&facts),
            Some(&focused),
            Some(&grid),
            Some(&table),
        )
    });
}

/// The full-probe walk with NO sources riding: the widest form's
/// report path — every probe layer lands its typed unqueried note
/// (the same report the sourceless entries carry, the notes the
/// backends' LoweringReport translation renders).
#[divan::bench]
fn walk_full_probe_sourceless(bencher: Bencher) {
    let (layers, keywords) = frozen();
    let env = resting_env();
    bencher.bench(|| {
        program_style_probes_containers_struct_descendants_supports(
            layers, keywords, &env, None, None, None, None, None, None,
        )
    });
}
