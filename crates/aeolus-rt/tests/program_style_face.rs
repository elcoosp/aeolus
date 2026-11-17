//! The host surface's own style face (the `StaticProgram` method
//! entries this file pins): the frozen program the `tw!` macro emits
//! carries the walk ENTRIES on its own type — `style`/`style_with`
//! (the element walk, the interp's probe seams optional) and
//! `style_pseudo`/`style_pseudo_theme` (the render targets' own
//! cascades) — so a host threads no slices and no imports beyond the
//! `aeolus_rt` surface it already consumes.
//!
//! The contract the pins encode: the methods are the free functions'
//! DELEGATES — same program, same env, same `(ResolvedStyle,
//! StyleWalk)` answer, byte-equal (the differential pin reads both
//! shapes whole) — and the pseudo face rides the same condition-
//! equality verdict the compile-side suite pins. One delegation, two
//! spellings, no third walk.
//!
//! Written FIRST (the RED spec): the suite does not compile against
//! the pre-seam surface — the methods do not exist yet, the compile
//! failure IS the capture.

use aeolus_compile::{Config, Env};
use aeolus_rt::{
    AttrBit, AttrKind, AttrState, ClassCell, PseudoTarget, StaticProgram, StyleWalk, program_style,
    program_style_pseudo, program_style_pseudo_theme, program_style_with,
};
use aeolus_theme::{ThemeSource, TokenTable as ThemeTable};

/// The baked shadcn LIGHT table (T0's shape) — the freeze walk's own
/// construction.
fn baked_table() -> ThemeTable {
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    ThemeTable::baked(&source).expect("the baked shadcn table builds")
}

/// Build a static program through the dynamic lane's own compile —
/// the ClassCell is the runtime path's constructor (the same compiler
/// the macro runs), so the pin's input is a REAL frozen program. The
/// colour utilities lower through a theme table in the V1 pipeline
/// (the themeless compile refuses them — the slow lane's own
/// boundary), so the cell compiles THEMED over the baked table.
fn frozen(classes: &str) -> StaticProgram {
    let cell = ClassCell::new();
    let program = cell
        .get_theme(classes, &baked_table(), &Config::default())
        .expect("the cell compiles what lowers");
    assert!(
        !program.layers.is_empty(),
        "the corpus carries layers: {classes}"
    );
    program
}

#[test]
fn the_style_method_is_the_element_walks_delegate() {
    let program = frozen("p-4 bg-red-500 rounded-md hover:bg-red-500/80");
    let env = Env::default();

    let (method_style, method_walk) = program.style(&env);
    let (free_style, free_walk) = program_style(program.layers, program.keywords, &env);
    assert_eq!(method_style, free_style, "the slots agree");
    assert_eq!(method_walk, free_walk, "the reports agree");
    assert_eq!(method_style.padding.expect("p-4 lands").top, 16.0);
    assert!(method_walk.is_empty(), "a clean walk on the resting env");
}

#[test]
fn the_style_with_method_threads_the_seams() {
    let program = frozen("data-[state=open]:bg-red-500 p-2");
    let env = Env::default();
    let attrs = AttrState::new(&[AttrBit {
        kind: AttrKind::Data,
        name: "state",
        value: "open",
    }]);
    let table = baked_table();

    let (method_style, method_walk) = program.style_with(&env, Some(&attrs), Some(&table));
    let (free_style, free_walk) = program_style_with(
        program.layers,
        program.keywords,
        &env,
        Some(&attrs),
        Some(&table),
    );
    assert_eq!(method_style, free_style, "the slots agree");
    assert_eq!(method_walk, free_walk, "the reports agree");
    assert!(method_style.background.is_some(), "the probe answers");
}

#[test]
fn the_style_pseudo_method_is_the_target_walks_delegate() {
    let program = frozen("selection:bg-red-500 p-4");
    let (method_style, method_walk) = program.style_pseudo(PseudoTarget::Selection);
    let (free_style, free_walk) =
        program_style_pseudo(PseudoTarget::Selection, program.layers, program.keywords);
    assert_eq!(method_style, free_style, "the slots agree");
    assert_eq!(method_walk, free_walk, "the reports agree");
    assert!(
        method_style.background.is_some(),
        "the target's own cascade"
    );
    assert!(method_walk.is_empty(), "a clean target walk");
}

#[test]
fn the_style_pseudo_theme_method_resolves_the_theme_forms() {
    // The ClassCell compiles THEMELESS (no table in the slot) — the
    // colour token refuses and the program carries the boundary, so
    // the themed face's input is a program built through get_theme.
    let cell = ClassCell::new();
    let table = baked_table();
    let program = cell
        .get_theme("selection:bg-primary", &table, &Config::default())
        .expect("the themed cell compiles");
    let env = Env::default();

    let (method_style, method_walk) =
        program.style_pseudo_theme(PseudoTarget::Selection, &table, &env);
    let (free_style, free_walk) = program_style_pseudo_theme(
        PseudoTarget::Selection,
        program.layers,
        program.keywords,
        &table,
        &env,
    );
    assert_eq!(method_style, free_style, "the slots agree");
    assert_eq!(method_walk, free_walk, "the reports agree");
    assert!(method_style.background.is_some(), "the token resolved");
    assert!(
        method_walk.is_empty(),
        "the source answers every theme form"
    );
}

/// The report type rides the re-export whole (the callers' equality
/// pins read it — the `StyleWalk` import stays exercised).
#[test]
fn the_report_type_stays_reachable() {
    fn reads(walk: StyleWalk) -> bool {
        walk.is_empty()
    }
    assert!(reads(StyleWalk::default()));
}

// ---------------------------------------------------------------------------
// Micro-debt: the row-text read side gains its first pins
// ---------------------------------------------------------------------------

#[test]
fn keyword_reads_the_flattened_layer_major_texts() {
    // The annotation slice is flattened in layer-major order ACROSS all
    // layers (the same accumulation the walk reads): the positional
    // form indexes (layer, decl) over that flattening, `""` (the
    // texts a value carries itself) answering `None` like an
    // out-of-bounds read.
    let program = frozen("hidden p-4 bg-red-500");

    // Layer 0's first declaration is the `hidden` display row: the text
    // is the registry's own keyword spelling.
    let display = program.keyword(0, 0).expect("the display row has a text");
    assert_eq!(display, "none", "the row text spells the CSS value");

    // A layer index past the program's own layers is out of bounds —
    // `None`, never a panic (the walker's `get(index + decl)` shape).
    assert_eq!(program.keyword(program.layers.len(), 0), None);
    // A declaration index past the layer's own decls answers `None`
    // through the same bounds read.
    assert_eq!(program.keyword(0, 42), None);
}

#[test]
fn keyword_for_takes_the_cascade_winner_and_skips_empty_texts() {
    // The ergonomic lookup: the LAST matching declaration in the layer
    // wins (the cascade's own front-to-back reading), texts that carry
    // their own value (`""`) never shadow an earlier keyword, and an
    // absent longhand answers `None`.
    let program = frozen("hidden w-full p-4");

    assert_eq!(
        program.keyword_for(0, aeolus_ir::Prop::Display),
        Some("none"),
        "the display row's text"
    );
    // w-full is one of the row-text LENGTH slots (the perf contract's
    // own corpus): the text spells the % value the CSS arm renders.
    assert_eq!(
        program.keyword_for(0, aeolus_ir::Prop::Width),
        Some("100%"),
        "the width row's text"
    );
    // p-4's spacing longhands carry their own value — the empty text
    // never answers (the skip is the contract).
    assert_eq!(
        program.keyword_for(0, aeolus_ir::Prop::PaddingTop),
        None,
        "the empty-text skip"
    );
    // A longhand the layer does not set answers `None`.
    assert_eq!(program.keyword_for(0, aeolus_ir::Prop::ZIndex), None);
}
