//! The program lane's pseudo-target face (the `program_style_pseudo` /
//! `program_style_pseudo_theme` entries this file pins): the interp
//! arm's own-cascade walk (`interp_pseudo`/`interp_pseudo_theme`)
//! grows onto the FROZEN static program the `tw!` macro emits — the
//! two slices the macro freezes resolve a [`PseudoTarget`]'s own
//! cascade into the v0.x [`ResolvedStyle`], the same `(style, walk)`
//! shape the element walk answers.
//!
//! The seam contract the pins encode:
//!
//! * only the layers carrying the target's condition participate (the
//!   flat fold collapses stacked variants onto the pseudo condition) —
//!   later fired declarations win per longhand, the cascade's own
//!   front-to-back reading;
//! * the element's own layers and the sibling pseudo targets are
//!   SILENT on this side — the element walk's
//!   [`InterpNote::PseudoElement`] note is the other half of the same
//!   render-target contract (reported on ITS side);
//! * the themeless form refuses [`Val::Tok`]/[`Val::ModePair`]
//!   per-longhand into [`StyleWalk::refusals`] (the style fold's
//!   report shape — never a hard error, never a silent transparent
//!   colour); the themed form resolves through one [`TokenSource`]
//!   (the env rides the pair selection, not the firing);
//! * the fold AGREES with the interp arm: same program, same target,
//!   the same effective declarations — one verdict walk (condition
//!   equality), two consumers.
//!
//! Written FIRST (the RED spec): the suite does not compile against
//! the pre-seam walk — the entries do not exist yet, the compile
//! failure IS the capture.

use aeolus_compile::{
    Config, Env, InterpNote, PseudoTarget, StyleWalk, compile_theme, interp_pseudo,
    keyword_alignment, keyword_texts, program_style, program_style_pseudo,
    program_style_pseudo_theme,
};
use aeolus_core::plan::ResolvedStyle;
use aeolus_theme::{ThemeSource, TokenTable};

/// The baked shadcn LIGHT table (T0's shape: every colour token bakes
/// to a literal) — the same construction the `tw!` macro's freeze walk
/// runs.
fn baked_table() -> TokenTable {
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    TokenTable::baked(&source).expect("the baked shadcn table builds")
}

/// Compile a class string the way `tw!` does and freeze it into the
/// static form the macro emits: the interned layers plus the row texts
/// the keyword slots index.
fn frozen(classes: &str) -> (&'static [aeolus_ir::Layer], &'static [&'static str]) {
    let config = Config::default();
    let compiled =
        aeolus_compile::compile_theme(classes, &baked_table(), &config).expect("the class lowers");
    assert!(
        compiled.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        compiled.diagnostics
    );
    let annotations = keyword_texts(classes, &config).expect("non-empty input annotates");
    let keywords = keyword_alignment(&compiled, &annotations);
    (
        compiled.program.layers,
        Box::leak(keywords.into_boxed_slice()),
    )
}

/// bg-primary under the shadcn light palette: oklch(0.205 0 0) ≈
/// #171717 — the same constant the probe battery pins.
const PRIMARY: [f32; 4] = [23.0 / 255.0, 23.0 / 255.0, 23.0 / 255.0, 1.0];

/// red-500's gamut-mapped sRGBA (the interp batteries' own constant):
/// oklch(0.637 0.237 25.331) → #fb2c36.
const RED_500: [f32; 4] = [251.0 / 255.0, 44.0 / 255.0, 54.0 / 255.0, 1.0];

/// The sRGBA a baked background carries, for the quantization-tolerant
/// colour reads (both lanes convert through the audited parser, so the
/// agreement is exact up to the 1e-4 Oklab equality epsilon).
fn srgba(c: aeolus_core::Color) -> [f32; 4] {
    c.to_srgba()
}

fn assert_colour(got: [f32; 4], want: [f32; 4]) {
    for (g, w) in got.iter().zip(want.iter()) {
        assert!((g - w).abs() < 1.0 / 255.0, "got {got:?}, want {want:?}");
    }
}

// ---------------------------------------------------------------------------
// The fold: the target's own cascade, from the frozen slices
// ---------------------------------------------------------------------------

#[test]
fn the_selection_cascade_resolves_from_the_frozen_slices() {
    let (layers, keywords) = frozen("selection:bg-red-500 selection:bg-primary p-4");

    // The element walk: the pseudo layers are the EXCLUDED render
    // targets (reported), the element's p-4 is the whole answer.
    let (element, walk) = program_style(layers, keywords, &Env::default());
    assert!(
        element.background.is_none(),
        "selection never styles the element"
    );
    assert_eq!(element.padding.expect("p-4 is the element's own").top, 16.0);
    assert!(
        walk.notes
            .iter()
            .any(|n| matches!(n, InterpNote::PseudoElement { .. })),
        "the element walk reports the pseudo layers: {walk:?}"
    );

    // The target's own cascade: only the selection layers participate,
    // later fired wins per longhand (bg-primary is the LATER beat).
    let (style, walk) = program_style_pseudo(PseudoTarget::Selection, layers, keywords);
    assert_colour(
        srgba(style.background.expect("the selection cascade fires").0),
        PRIMARY,
    );
    assert!(
        walk.is_empty(),
        "a clean target walk reports nothing: {walk:?}"
    );
}

#[test]
fn the_element_layers_stay_silent_on_the_target_side() {
    let (layers, keywords) = frozen("backdrop:p-8 hover:p-4");

    // The SELECTION fold: neither the element layers nor the sibling
    // pseudo target participate — silent, not noted (the element walk
    // reports them on ITS side, the render-target contract's other
    // half).
    let (style, walk) = program_style_pseudo(PseudoTarget::Selection, layers, keywords);
    assert_eq!(
        style,
        ResolvedStyle::default(),
        "the target answers nothing"
    );
    assert!(
        walk.is_empty(),
        "the other target's layers are silence, not notes: {walk:?}"
    );
}

#[test]
fn every_target_walks_headless() {
    // The closed set: every render target's own cascade resolves from
    // the frozen slices, the value landing in the slot the interp arm
    // resolves it to.
    for spelling in [
        "selection",
        "placeholder",
        "before",
        "after",
        "marker",
        "first-letter",
        "first-line",
        "file",
        "backdrop",
        "details-content",
    ] {
        let (layers, keywords) = frozen(&format!("{spelling}:p-8"));
        let target = PseudoTarget::from_name(spelling).expect("the closed set names it");
        let (style, walk) = program_style_pseudo(target, layers, keywords);
        let padding = style.padding.expect("{spelling}: the target's own value");
        assert_eq!(padding.top, 32.0, "{spelling}: p-8 = 2rem = 32px");
        assert!(walk.is_empty(), "{spelling}: a clean walk reports nothing");
    }
}

// ---------------------------------------------------------------------------
// The theme forms: the report refusal and the themed resolution
// ---------------------------------------------------------------------------

#[test]
fn the_themeless_token_refuses_into_the_report() {
    // The T2 runtime tier is where `Val::Tok` slots exist (the walk's
    // theme forms are the theme pass's outputs): the program compiles
    // under the runtime table, then resolves WITHOUT a source — the
    // explicit slow lane's boundary.
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    let runtime = TokenTable::runtime(&source).expect("the runtime table builds");
    let config = Config::default();
    let compiled =
        compile_theme("selection:bg-primary", &runtime, &config).expect("the string lowers");
    let annotations =
        keyword_texts("selection:bg-primary", &config).expect("non-empty input annotates");
    let keywords = keyword_alignment(&compiled, &annotations);
    let (layers, keywords) = (
        compiled.program.layers,
        Box::leak(keywords.into_boxed_slice()),
    );

    // No source rides: the token refuses PER LONGHAND into the walk's
    // report — the style fold's own shape (never a hard error, never a
    // silent transparent colour).
    let (style, walk) = program_style_pseudo(PseudoTarget::Selection, layers, keywords);
    assert!(
        style.background.is_none(),
        "the refused longhand wrote no slot"
    );
    let bg = walk
        .refusals
        .iter()
        .find(|(prop, _)| *prop == aeolus_ir::Prop::Background)
        .expect("the token refusal reports");
    assert!(
        bg.1.contains("no TokenSource"),
        "the reason names the missing source: {bg:?}"
    );

    // The themed form over the SAME runtime table resolves the slot
    // (T2's one load + one multiply) — the slow lane's other face.
    let (style, walk) = program_style_pseudo_theme(
        PseudoTarget::Selection,
        layers,
        keywords,
        &runtime,
        &Env::default(),
    );
    assert!(style.background.is_some(), "the runtime table answers");
    assert!(
        walk.is_empty(),
        "the source answers every theme form: {walk:?}"
    );
}

#[test]
fn the_themed_form_resolves_through_the_source() {
    let (layers, keywords) = frozen("selection:bg-primary");
    let table = baked_table();

    let (style, walk) = program_style_pseudo_theme(
        PseudoTarget::Selection,
        layers,
        keywords,
        &table,
        &Env::default(),
    );
    assert_colour(
        srgba(style.background.expect("the themed form resolves").0),
        PRIMARY,
    );
    assert!(
        walk.is_empty(),
        "the source answers every theme form: {walk:?}"
    );
}

// ---------------------------------------------------------------------------
// The differential pin: one verdict walk, two consumers
// ---------------------------------------------------------------------------

#[test]
fn the_pseudo_fold_agrees_with_the_interp_arm() {
    // Same program, both targets, both lanes: the effective
    // declarations the interp arm resolves are the slots the style
    // fold writes — one verdict walk (condition equality), two
    // consumers.
    let config = Config::default();
    let classes = "placeholder:text-red-500 selection:bg-primary";
    let compiled = compile_theme(classes, &baked_table(), &config).expect("the string lowers");
    let annotations = keyword_texts(classes, &config).expect("non-empty input annotates");
    let keywords = keyword_alignment(&compiled, &annotations);
    let keywords: &'static [&'static str] = Box::leak(keywords.into_boxed_slice());
    let table = baked_table();
    let env = aeolus_compile::Env::default();

    for (target, colour) in [
        (PseudoTarget::Selection, PRIMARY),
        (PseudoTarget::Placeholder, RED_500),
    ] {
        let interped = interp_pseudo(&compiled.program, target).expect("the interp arm resolves");
        let (style, walk) = program_style_pseudo(target, compiled.program.layers, keywords);
        match target {
            PseudoTarget::Selection => {
                assert_colour(srgba(style.background.expect("the slot writes").0), colour);
                assert!(
                    interped
                        .decls
                        .iter()
                        .any(|d| d.prop == aeolus_ir::Prop::Background),
                    "the interp arm resolved the same longhand: {:?}",
                    interped.decls
                );
            }
            PseudoTarget::Placeholder => {
                assert_colour(
                    srgba(style.content_color.expect("the slot writes").0),
                    colour,
                );
                assert!(
                    interped
                        .decls
                        .iter()
                        .any(|d| d.prop == aeolus_ir::Prop::Color),
                    "the interp arm resolved the same longhand: {:?}",
                    interped.decls
                );
            }
            _ => unreachable!("the two targets the corpus names"),
        }
        assert!(
            walk.is_empty(),
            "{target:?}: the baked fold reports nothing"
        );

        // The themed face agrees with the themed interp arm too.
        let themed =
            program_style_pseudo_theme(target, compiled.program.layers, keywords, &table, &env);
        assert_eq!(
            style, themed.0,
            "the themed fold is the themeless answer here"
        );
        assert!(themed.1.is_empty());
    }
}

#[test]
fn the_keyword_texts_ride_the_flattened_index() {
    // The annotations are flattened in layer-major order across ALL
    // layers — the pseudo walk reads the RIGHT text for the RIGHT layer
    // whether the kw-slot layer rides before or after the target's
    // own. `hidden` (a Display kw slot the v0.x slots cannot carry)
    // never participates in the SELECTION fold: its text stays unread,
    // the target's background lands either way.
    let (before, kw_before) = frozen("hidden selection:bg-red-500");
    let (style, walk) = program_style_pseudo(PseudoTarget::Selection, before, kw_before);
    assert_colour(
        srgba(style.background.expect("the target layer fires").0),
        RED_500,
    );
    assert!(walk.is_empty(), "the element kw never wrote: {walk:?}");

    let (after, kw_after) = frozen("selection:bg-red-500 hidden");
    let (style, walk) = program_style_pseudo(PseudoTarget::Selection, after, kw_after);
    assert_colour(
        srgba(style.background.expect("the target layer fires").0),
        RED_500,
    );
    assert!(walk.is_empty(), "the element kw never wrote: {walk:?}");
}

#[test]
fn the_target_kw_reports_its_slot_boundary() {
    // A kw-slot utility UNDER the target's own variant: the fold
    // writes the longhand the v0.x slots cannot carry — the honest
    // boundary note (never silent), the same channel the element walk
    // reports through.
    let (layers, keywords) = frozen("selection:hidden");
    let (style, walk) = program_style_pseudo(PseudoTarget::Selection, layers, keywords);
    assert!(
        !walk.boundaries.is_empty(),
        "the display longhand reports its slot boundary: {walk:?}"
    );
    assert!(
        style == ResolvedStyle::default() || style.layout.is_some(),
        "the display slot answers where the v0.x surface carries it: {style:?}"
    );
}

/// The report type stays comparable and defaultable (the callers'
/// equality pins read it whole).
#[test]
fn the_report_type_stays_comparable() {
    let walk = StyleWalk::default();
    assert!(walk.is_empty());
}
