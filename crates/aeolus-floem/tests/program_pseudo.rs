//! The floem backend's pseudo-target face (the render-target contract's
//! backend half — the `lower_program_pseudo` / `lower_program_pseudo_theme`
//! entries this file pins): one [`PseudoTarget`]'s own cascade lowers
//! from the frozen static program into a Floem [`Style`] through the
//! crate's own walk — the answer a host that PAINTS the target (a
//! selection layer, a custom placeholder) consumes.
//!
//! The walk keeps the floem lane's own shape: the verdict is CONDITION
//! EQUALITY (the interp arm's pseudo walk — only the layers carrying
//! the target's condition participate, the element's own layers and the
//! sibling pseudo targets SILENT, not notes), the declarations ride the
//! SAME `apply_decl` mapping the element walk uses, and the themeless
//! form's theme refusals land in the crate's [`LoweringReport`] with
//! the prop named — never a fabricated colour, never a silent drop.
//!
//! Written FIRST (the RED spec): the suite does not compile against the
//! pre-seam backend — the entries do not exist yet, the compile failure
//! IS the capture.

use floem::style::{Background, PaddingLeft, Style, unit::Length};

use aeolus_compile::{Config, PseudoTarget, compile_theme, keyword_alignment, keyword_texts};
use aeolus_floem::{lower_program_pseudo, lower_program_pseudo_theme};
use aeolus_rt::{Env, StaticProgram};
use aeolus_theme::{ThemeSource, TokenTable};

/// The baked shadcn LIGHT table (T0's shape: every colour token bakes
/// to a literal) — the same construction the `tw!` macro's freeze walk
/// runs.
fn baked_table() -> TokenTable {
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    TokenTable::baked(&source).expect("the baked shadcn table builds")
}

/// Compile a class string the way `tw!` does and freeze it into the
/// static form the macro emits.
fn frozen(classes: &str) -> StaticProgram {
    let config = Config::default();
    let compiled = compile_theme(classes, &baked_table(), &config).expect("the class lowers");
    assert!(
        compiled.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        compiled.diagnostics
    );
    let annotations = keyword_texts(classes, &config).expect("non-empty input annotates");
    let keywords = keyword_alignment(&compiled, &annotations);
    StaticProgram::new(
        compiled.program.layers,
        Box::leak(keywords.into_boxed_slice()),
    )
}

fn brush_rgba(b: &floem::peniko::Brush) -> [u8; 4] {
    match b {
        // peniko 0.6 has no `[u8; 4]` compare for Rgba8 — raw components.
        floem::peniko::Brush::Solid(c) => c.components.map(|v| (v * 255.0).round() as u8),
        _ => panic!("expected a solid brush"),
    }
}

/// bg-primary under the shadcn light palette: oklch(0.205 0 0) ≈ #171717.
const PRIMARY: [u8; 4] = [23, 23, 23, 255];

/// red-500's gamut-mapped sRGBA: oklch(0.637 0.237 25.331) → #fb2c36.
const RED_500: [u8; 4] = [251, 44, 54, 255];

// ---------------------------------------------------------------------------
// The fold: the target's own cascade, from the frozen program
// ---------------------------------------------------------------------------

#[test]
fn the_selection_cascade_lowers_beside_the_silent_element_layers() {
    let program = frozen("selection:bg-red-500 selection:bg-primary p-4");

    // The target's own cascade: only the selection layers participate,
    // later fired wins per longhand (bg-primary is the LATER beat);
    // p-4 rides the ELEMENT walk, never the target's own style.
    let (style, report) = lower_program_pseudo(&program, PseudoTarget::Selection);
    assert_eq!(
        brush_rgba(style.get(Background).as_ref().expect("the cascade fires")),
        PRIMARY
    );
    assert_eq!(
        style.get(PaddingLeft),
        Style::default().get(PaddingLeft),
        "the element's own p-4 never styles the target"
    );
    assert!(
        report.is_empty(),
        "a clean target walk reports nothing: {report:?}"
    );
}

#[test]
fn the_element_layers_and_sibling_targets_stay_silent() {
    let program = frozen("backdrop:p-8 hover:p-4");

    // The SELECTION fold: neither the element layers nor the sibling
    // pseudo target participate — the answer is the untouched default,
    // silence not notes (the element walk reports them on ITS side).
    let (style, report) = lower_program_pseudo(&program, PseudoTarget::Selection);
    assert!(
        style.get(Background).is_none(),
        "the target answers nothing"
    );
    assert_eq!(
        style.get(PaddingLeft),
        Style::default().get(PaddingLeft),
        "the element's own layers never style the target"
    );
    assert!(
        report.is_empty(),
        "the other target's layers are silence, not notes: {report:?}"
    );
}

#[test]
fn every_target_walks_headless() {
    // The closed set: every render target's own cascade lowers from the
    // frozen program, the value landing in the slot the walk's mapping
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
        let program = frozen(&format!("{spelling}:p-8"));
        let target = PseudoTarget::from_name(spelling).expect("the closed set names it");
        let (style, report) = lower_program_pseudo(&program, target);
        // p-8 = 2rem = 32px, the target's own padding.
        assert_eq!(
            style.get(PaddingLeft),
            Length::Pt(32.0),
            "{spelling}: p-8 = 2rem = 32px"
        );
        assert!(
            report.is_empty(),
            "{spelling}: a clean walk reports nothing"
        );
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
    // explicit slow lane's boundary, named in the backend's own report.
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    let runtime = TokenTable::runtime(&source).expect("the runtime table builds");
    let config = Config::default();
    let compiled =
        compile_theme("selection:bg-primary", &runtime, &config).expect("the string lowers");
    assert!(
        compiled.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        compiled.diagnostics
    );
    let annotations =
        keyword_texts("selection:bg-primary", &config).expect("non-empty input annotates");
    let keywords = keyword_alignment(&compiled, &annotations);
    let program = StaticProgram::new(
        compiled.program.layers,
        Box::leak(keywords.into_boxed_slice()),
    );

    let (style, report) = lower_program_pseudo(&program, PseudoTarget::Selection);
    assert!(
        style.get(Background).is_none(),
        "the themeless face never fabricates a colour"
    );
    assert!(
        !report.is_empty(),
        "the refusal lands in the backend's report: {report:?}"
    );
    assert!(
        report
            .dropped
            .iter()
            .any(|(what, _)| what.contains("Background")),
        "the refusal names the longhand: {report:?}"
    );
}

#[test]
fn the_themed_form_resolves_through_the_source() {
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    let runtime = TokenTable::runtime(&source).expect("the runtime table builds");
    let config = Config::default();
    let compiled =
        compile_theme("selection:bg-primary", &runtime, &config).expect("the string lowers");
    assert!(
        compiled.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        compiled.diagnostics
    );
    let annotations =
        keyword_texts("selection:bg-primary", &config).expect("non-empty input annotates");
    let keywords = keyword_alignment(&compiled, &annotations);
    let program = StaticProgram::new(
        compiled.program.layers,
        Box::leak(keywords.into_boxed_slice()),
    );

    // The themed face: the table rides as the TokenSource, the env rides
    // the RESOLUTION (the mode selects the pair halves), never the firing.
    let (style, report) =
        lower_program_pseudo_theme(&program, PseudoTarget::Selection, &runtime, &Env::default());
    assert_eq!(
        brush_rgba(
            style
                .get(Background)
                .as_ref()
                .expect("the themed face resolves")
        ),
        PRIMARY
    );
    assert!(
        report.is_empty(),
        "a clean themed walk reports nothing: {report:?}"
    );
}

#[test]
fn the_red_beat_lowers_through_the_element_mapping() {
    // The face rides the SAME `apply_decl` mapping the element walk
    // uses: the selection cascade's red-500 lands as the same solid
    // brush the element walk's `bg-red-500` produces.
    let program = frozen("selection:bg-red-500");
    let (style, report) = lower_program_pseudo(&program, PseudoTarget::Selection);
    assert_eq!(
        brush_rgba(style.get(Background).as_ref().expect("the cascade fires")),
        RED_500
    );
    assert!(report.is_empty(), "unexpected drops: {report:?}");
}
