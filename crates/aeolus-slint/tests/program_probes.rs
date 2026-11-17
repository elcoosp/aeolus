//! The slint backend's full-probe program lane (the widest
//! `lower_program_*` growth — the shared walk's
//! `program_style_probes_containers_struct_descendants_supports` seam
//! riding the backend's report translation): the named-container,
//! descendant `has-*` and `supports-*` layers a plan-lane program
//! carries answer from the host's probe sources instead of the
//! unqueried reports the sourceless form honestly carries.
//!
//! Written FIRST (the RED spec): the suite does not compile against
//! the pre-seam backend — the widest entry does not exist yet, the
//! compile failure IS the capture.

use aeolus_compile::{Config, compile_theme, keyword_alignment, keyword_texts};
use aeolus_rt::{
    ContainerBound, ContainerState, DescendantFact, DescendantKind, DescendantState, Env,
    StaticProgram, SupportsState,
};
use aeolus_slint::{
    LoweringReport, SlintStyle, lower_program,
    lower_program_probes_containers_struct_descendants_supports,
};
use aeolus_theme::{ThemeSource, TokenTable};
use i_slint_core::graphics::Brush;

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

/// The straight-alpha bytes a Slint [`Color`] carries, for the
/// comparison reads (the Oklab equality epsilon is far below one
/// 8-bit step).
fn rgba(c: i_slint_core::graphics::Color) -> [u8; 4] {
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

/// True when the report carries a drop whose reason names this seam's
/// unqueried note (the walk's typed InterpNote text).
fn reports(report: &LoweringReport, needle: &str) -> bool {
    report
        .dropped
        .iter()
        .any(|(_, reason)| reason.contains(needle))
}

#[test]
fn named_container_layers_answer_through_the_backend_lane() {
    let program = frozen("bg-primary @md/card:bg-red-500");

    // Sourceless: the container layer reports unqueried — the walk's
    // pre-seam shape, the report honest.
    let (resting, report) = lower_program(&program, &Env::default());
    assert!(
        reports(&report, "no container context"),
        "the unqueried container is reported: {:?}",
        report.dropped
    );

    // The host's measured container answers above the md boundary: the
    // layer fires (red-500), the report loses the drop.
    let card = ContainerState::new(&[ContainerBound {
        name: "card",
        rem: 64.0,
    }]);
    let (fired, report) = lower_program_probes_containers_struct_descendants_supports(
        &program,
        &Env::default(),
        None,
        Some(&card),
        None,
        None,
        None,
        None,
    );
    assert!(
        !reports(&report, "no container context"),
        "the answered layer is no report: {:?}",
        report.dropped
    );
    assert_ne!(
        fill_of(&fired),
        fill_of(&resting),
        "@md/card:bg-red-500 fired through the seam"
    );

    // The answered miss (the card below the boundary) is CSS-quiet —
    // the resting fill, and NO drop.
    let narrow = ContainerState::new(&[ContainerBound {
        name: "card",
        rem: 32.0,
    }]);
    let (quiet, report) = lower_program_probes_containers_struct_descendants_supports(
        &program,
        &Env::default(),
        None,
        Some(&narrow),
        None,
        None,
        None,
        None,
    );
    assert_eq!(fill_of(&quiet), fill_of(&resting), "the miss stays quiet");
    assert!(
        !reports(&report, "no container context"),
        "an answered miss is quiet, never a drop: {:?}",
        report.dropped
    );
}

#[test]
fn has_layers_answer_through_the_backend_lane() {
    let program = frozen("bg-primary has-focus:bg-red-500");

    let (resting, report) = lower_program(&program, &Env::default());
    assert!(
        reports(&report, "DescendantProbe"),
        "the unqueried has- layer is reported: {:?}",
        report.dropped
    );

    // The subtree fact answers: the layer fires.
    let focused = DescendantState::new(&[DescendantFact {
        kind: DescendantKind::Element,
        fragment: "*:focus",
    }]);
    let (fired, report) = lower_program_probes_containers_struct_descendants_supports(
        &program,
        &Env::default(),
        None,
        None,
        None,
        Some(&focused),
        None,
        None,
    );
    assert!(!reports(&report, "DescendantProbe"), "{:?}", report.dropped);
    assert_ne!(fill_of(&fired), fill_of(&resting), "has-focus fired");

    // The answered miss is a quiet skip — never a drop.
    let empty = DescendantState::new(&[]);
    let (quiet, report) = lower_program_probes_containers_struct_descendants_supports(
        &program,
        &Env::default(),
        None,
        None,
        None,
        Some(&empty),
        None,
        None,
    );
    assert_eq!(fill_of(&quiet), fill_of(&resting), "the miss stays quiet");
    assert!(!reports(&report, "DescendantProbe"), "{:?}", report.dropped);
}

#[test]
fn supports_layers_answer_through_the_backend_lane() {
    let program = frozen("bg-primary supports-[display:grid]:bg-red-500");

    let (resting, report) = lower_program(&program, &Env::default());
    assert!(
        reports(&report, "SupportsProbe"),
        "the unqueried supports layer is reported: {:?}",
        report.dropped
    );

    // The capability answer fires the layer.
    let grid = SupportsState::new(&["(display:grid)"]);
    let (fired, report) = lower_program_probes_containers_struct_descendants_supports(
        &program,
        &Env::default(),
        None,
        None,
        None,
        None,
        Some(&grid),
        None,
    );
    assert!(!reports(&report, "SupportsProbe"), "{:?}", report.dropped);
    assert_ne!(
        fill_of(&fired),
        fill_of(&resting),
        "the supports layer fired"
    );

    // The answered miss is a quiet skip — never a drop.
    let none = SupportsState::new(&[]);
    let (quiet, report) = lower_program_probes_containers_struct_descendants_supports(
        &program,
        &Env::default(),
        None,
        None,
        None,
        None,
        Some(&none),
        None,
    );
    assert_eq!(fill_of(&quiet), fill_of(&resting), "the miss stays quiet");
    assert!(!reports(&report, "SupportsProbe"), "{:?}", report.dropped);
}
