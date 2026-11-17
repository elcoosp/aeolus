//! The floem backend's full-probe program lane (the widest
//! `lower_program_*` growth — the interp arm's decide verdicts growing
//! the container, structural, descendant and supports seams onto THIS
//! backend's own walk): the named-container, `has-*` and `@supports`
//! layers a plan-lane program carries answer from the host's probe
//! sources instead of the unqueried reports the sourceless form
//! honestly carries.
//!
//! Floem is the last backend on the lane (the probe-seam handoff rode
//! xilem/egui/iced/slint; this crate's own walk kept the pre-seam
//! shape). The verdicts stay the interp arm's OWN semantics — the same
//! at-and-above / strictly-below container boundaries, the descendant
//! YES-fires/NO-quiet-skips, the capability answers — applied in this
//! walk's `decide`, so the widest form cannot disagree with the four
//! delegating backends on when a layer fires.
//!
//! Written FIRST (the RED spec): the suite does not compile against the
//! pre-seam backend — the widest entry does not exist yet, the compile
//! failure IS the capture.

use floem::style::{Background, Style};

use aeolus_compile::{Config, compile_theme, keyword_alignment, keyword_texts};
use aeolus_floem::{
    LoweringReport, lower_program, lower_program_probes_containers_struct_descendants_supports,
};
use aeolus_rt::{
    ContainerBound, ContainerState, DescendantFact, DescendantKind, DescendantState, Env,
    StaticProgram, SupportsState,
};
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

/// The solid fill of the walked style, when one is set — the same read
/// the differential pins apply.
fn fill_of(style: &Style) -> Option<[u8; 4]> {
    style.get(Background).map(|b| match b {
        floem::peniko::Brush::Solid(c) => c.components.map(|v| (v * 255.0).round() as u8),
        _ => panic!("expected a solid brush"),
    })
}

/// True when the report carries a drop whose reason names this seam's
/// unqueried boundary (the walk's typed note text).
fn reports(report: &LoweringReport, needle: &str) -> bool {
    report
        .dropped
        .iter()
        .any(|(_, reason)| reason.contains(needle))
}

#[test]
fn named_container_layers_answer_through_the_backend_lane() {
    let program = frozen("bg-primary @md/card:bg-red-500");

    // Sourceless: the named container reports unqueried — the walk's
    // pre-seam shape, the report honest.
    let (resting, report) = lower_program(&program, &Env::default());
    assert!(
        reports(&report, "ContainerProbe"),
        "the unqueried named container is reported: {:?}",
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
        !reports(&report, "ContainerProbe"),
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
        !reports(&report, "ContainerProbe"),
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

#[test]
fn structural_layers_answer_through_the_backend_lane() {
    let program = frozen("bg-primary first:bg-red-500 last:bg-accent");

    let (resting, report) = lower_program(&program, &Env::default());
    assert!(
        reports(&report, "StructProbe"),
        "the unqueried structural layers are reported: {:?}",
        report.dropped
    );

    // The DOM-position facts answer: first fires (index 0 of three).
    let facts = aeolus_rt::StructFacts::new(0, 3);
    let (fired, report) = lower_program_probes_containers_struct_descendants_supports(
        &program,
        &Env::default(),
        None,
        None,
        Some(&facts),
        None,
        None,
        None,
    );
    assert!(!reports(&report, "StructProbe"), "{:?}", report.dropped);
    assert_ne!(
        fill_of(&fired),
        fill_of(&resting),
        "first:bg-red-500 fired through the seam"
    );

    // The answered miss (the middle sibling of three — neither first
    // nor last) is a quiet skip — never a drop.
    let middle = aeolus_rt::StructFacts::new(1, 3);
    let (quiet, report) = lower_program_probes_containers_struct_descendants_supports(
        &program,
        &Env::default(),
        None,
        None,
        Some(&middle),
        None,
        None,
        None,
    );
    assert_eq!(fill_of(&quiet), fill_of(&resting), "the miss stays quiet");
    assert!(!reports(&report, "StructProbe"), "{:?}", report.dropped);
}
