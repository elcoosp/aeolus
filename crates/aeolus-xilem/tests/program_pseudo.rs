//! The xilem backend's pseudo-target face (the render-target contract's
//! backend half — the `lower_program_pseudo` / `lower_program_pseudo_theme`
//! entries this file pins): one [`PseudoTarget`]'s own cascade lowers
//! from the frozen static program into the plan path's own
//! [`ResolvedStyle`], the answer a host that PAINTS the target (a
//! selection layer, a custom placeholder) consumes — the same effective
//! style the shared walk's `program_style_pseudo` resolves, through the
//! backend's own report translation.
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
//!   [`TokenSource`];
//! * the face AGREES with the shared walk: same program, same target,
//!   the same effective style — one verdict walk, two consumers.
//!
//! Written FIRST (the RED spec): the suite does not compile against the
//! pre-seam backend — the entries do not exist yet, the compile failure
//! IS the capture.

use aeolus_compile::{Config, PseudoTarget, compile_theme, keyword_alignment, keyword_texts};
use aeolus_rt::{Env, StaticProgram, program_style_pseudo, program_style_pseudo_theme};
use aeolus_theme::{ThemeSource, TokenTable};
use aeolus_xilem::{lower_program_pseudo, lower_program_pseudo_theme};

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

/// The background slot this backend resolves (the plan path's own
/// v0.x style), as sRGBA for the comparison reads.
fn bg_of(style: &aeolus_core::plan::ResolvedStyle) -> Option<[f32; 4]> {
    style.background.as_ref().map(|b| b.0.to_srgba())
}

/// bg-primary under the shadcn light palette: oklch(0.205 0 0) ≈
/// #171717 — the same constant the element-walk battery pins.
const PRIMARY: [f32; 4] = [23.0 / 255.0, 23.0 / 255.0, 23.0 / 255.0, 1.0];

fn assert_colour(got: [f32; 4], want: [f32; 4]) {
    for (g, w) in got.iter().zip(want.iter()) {
        assert!((g - w).abs() < 1.0 / 255.0, "got {got:?}, want {want:?}");
    }
}

// ---------------------------------------------------------------------------
// The fold: the target's own cascade, from the frozen program
// ---------------------------------------------------------------------------

#[test]
fn the_selection_cascade_lowers_beside_the_silent_element_layers() {
    let program = frozen("selection:bg-red-500 selection:bg-primary p-4");

    // The target's own cascade: only the selection layers participate,
    // later fired wins per longhand (bg-primary is the LATER beat).
    let (style, report) = lower_program_pseudo(&program, PseudoTarget::Selection);
    assert_colour(bg_of(&style).expect("the selection cascade fires"), PRIMARY);
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
    // pseudo target participate — silent, not noted (the element walk
    // reports them on ITS side, the render-target contract's other
    // half).
    let (style, report) = lower_program_pseudo(&program, PseudoTarget::Selection);
    assert_eq!(
        style,
        aeolus_core::plan::ResolvedStyle::default(),
        "the target answers nothing"
    );
    assert!(
        report.is_empty(),
        "the other target's layers are silence, not notes: {report:?}"
    );
}

#[test]
fn every_target_walks_headless() {
    // The closed set: every render target's own cascade lowers from the
    // frozen program, the value landing in the slot the shared walk
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
        let padding = style.padding.expect("{spelling}: the target's own value");
        assert_eq!(padding.top, 32.0, "{spelling}: p-8 = 2rem = 32px");
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
        style.background.is_none(),
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
    assert_colour(
        bg_of(&style).expect("the themed face resolves the token"),
        PRIMARY,
    );
    assert!(
        report.is_empty(),
        "a clean themed walk reports nothing: {report:?}"
    );
}

// ---------------------------------------------------------------------------
// The differential: the face agrees with the shared walk
// ---------------------------------------------------------------------------

#[test]
fn the_face_agrees_with_the_shared_walk() {
    // One verdict walk, two consumers: the backend's face and the
    // pipeline's own `program_style_pseudo` resolve the SAME program to
    // the SAME effective style (the backend's report translation rides
    // beside, never changing the answer).
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    let runtime = TokenTable::runtime(&source).expect("the runtime table builds");
    for classes in [
        "selection:bg-red-500 selection:bg-primary p-4",
        "after:rounded-md after:shadow-lg backdrop:p-8",
        "placeholder:text-muted-foreground",
    ] {
        let program = frozen(classes);
        for target in PseudoTarget::ALL {
            let (faced, _) = lower_program_pseudo(&program, *target);
            let (walked, _) = program_style_pseudo(*target, program.layers, program.keywords);
            assert_eq!(
                faced, walked,
                "{classes} / {target:?}: the two spellings disagree"
            );

            // The themed form rides the same equality (the runtime
            // tier's table as the source, the resting env).
            let (faced_theme, _) =
                lower_program_pseudo_theme(&program, *target, &runtime, &Env::default());
            let (walked_theme, _) = program_style_pseudo_theme(
                *target,
                program.layers,
                program.keywords,
                &runtime,
                &Env::default(),
            );
            assert_eq!(
                faced_theme, walked_theme,
                "{classes} / {target:?}: the themed spellings disagree"
            );
        }
    }
}
