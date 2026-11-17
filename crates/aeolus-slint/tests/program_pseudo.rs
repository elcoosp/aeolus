//! The slint backend's pseudo-target face (the render-target contract's
//! backend half — the `lower_program_pseudo` / `lower_program_pseudo_theme`
//! entries this file pins): one [`PseudoTarget`]'s own cascade lowers
//! from the frozen static program into the crate's own [`SlintStyle`]
//! through the SAME `lower_resolved` mapping the element walk rides —
//! the answer a host that PAINTS the target (a selection layer, a
//! custom placeholder) consumes.
//!
//! The seam contract the pins encode (the compile lane's
//! `tests/style_walk_pseudo.rs` suite, on the backend's face):
//!
//! * only the layers carrying the target's condition participate —
//!   condition equality is the verdict, later fired declarations win
//!   per longhand;
//! * the element's own layers and the sibling pseudo targets are
//!   SILENT on this side (the element walk reports them on ITS side);
//! * the themeless form refuses the theme forms per longhand into the
//!   crate's [`LoweringReport`]; the themed form resolves through one
//!   [`TokenSource`].
//!
//! Written FIRST (the RED spec): the suite does not compile against the
//! pre-seam backend — the entries do not exist yet, the compile failure
//! IS the capture.

use aeolus_compile::{Config, PseudoTarget, compile_theme, keyword_alignment, keyword_texts};
use aeolus_rt::{Env, StaticProgram};
use aeolus_slint::{REM_PX, SlintStyle, lower_program_pseudo, lower_program_pseudo_theme};
use aeolus_theme::{ThemeSource, TokenTable};
use i_slint_core::graphics::{Brush, Color};

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

fn rgba(c: Color) -> [u8; 4] {
    let argb = c.as_argb_encoded();
    [
        ((argb >> 16) & 0xFF) as u8,
        ((argb >> 8) & 0xFF) as u8,
        (argb & 0xFF) as u8,
        ((argb >> 24) & 0xFF) as u8,
    ]
}

/// The solid fill of the style's background slot, when it is one.
fn fill_of(style: &SlintStyle) -> Option<[u8; 4]> {
    match &style.background {
        Some(Brush::SolidColor(c)) => Some(rgba(*c)),
        _ => None,
    }
}

/// bg-primary under the shadcn light palette: oklch(0.205 0 0) ≈ #171717.
const PRIMARY: [u8; 4] = [23, 23, 23, 255];

// ---------------------------------------------------------------------------
// The fold: the target's own cascade, from the frozen program
// ---------------------------------------------------------------------------

#[test]
fn the_selection_cascade_lowers_beside_the_silent_element_layers() {
    let program = frozen("selection:bg-red-500 selection:bg-primary p-4");

    // The target's own cascade: only the selection layers participate,
    // later fired wins per longhand (bg-primary is the LATER beat);
    // p-4 = 1rem = 16px under the backend's REM_PX constant rides the
    // ELEMENT walk, never the target's answer.
    let (style, report) = lower_program_pseudo(&program, PseudoTarget::Selection);
    let fill = fill_of(&style).expect("the selection cascade fires");
    for (got, want) in fill.iter().zip(PRIMARY.iter()) {
        assert!(i16::from(*got) - i16::from(*want) <= 1, "got {fill:?}");
    }
    assert!(
        style.padding.is_none(),
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
    assert_eq!(style, SlintStyle::default(), "the target answers nothing");
    assert!(
        report.is_empty(),
        "the other target's layers are silence, not notes: {report:?}"
    );
}

#[test]
fn every_target_walks_headless() {
    // The closed set: every render target's own cascade lowers from the
    // frozen program, the value landing in the slot the element walk's
    // mapping resolves it to.
    assert_eq!(REM_PX, 16.0);
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
        // p-8 = 2rem = 32px, the target's own padding hint.
        let padding = style.padding.expect("{spelling}: the target's own value");
        assert_eq!(padding.top, 32.0, "{spelling}: p-8 = 2rem = 32px");
        assert_eq!(padding.left, 32.0, "{spelling}: p-8 = 2rem = 32px");
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
        fill_of(&style).is_none(),
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
            .any(|(what, _)| what.contains("background")),
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
    let fill = fill_of(&style).expect("the themed face resolves the token");
    for (got, want) in fill.iter().zip(PRIMARY.iter()) {
        assert!(i16::from(*got) - i16::from(*want) <= 1, "got {fill:?}");
    }
    assert!(
        report.is_empty(),
        "a clean themed walk reports nothing: {report:?}"
    );
}
