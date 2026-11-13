//! The GPUI capability gate's gradient contracts (architecture plan
//! §3.7's `bg-linear-*` row, the emit half): the two-stop linear
//! gradient family joins the NATIVE tier — GPUI's
//! `linear_gradient(angle, from, to)` carries exactly the v4 2-stop
//! form — and everything the slot cannot take refuses TYPED: the via
//! stop (three stops), the radial/conic/placeholder row texts (no
//! `Fill` slot for the shape), with the row text the bare IR cannot
//! recover as the gate's annotation.
//!
//! Written FIRST (the RED spec): the suite fails while the gradient
//! longhands sit outside every tier and `lower_class` refuses any
//! program carrying one.

use aeolus_codegen::Emit;
use aeolus_codegen_gpui::{Diagnostics, GpuiEmit, lower_class, polyfilled, supports};
use aeolus_compile::{Config, compile_theme, keyword_texts};
use aeolus_ir::Prop;
use aeolus_theme::{ThemeSource, TokenTable};

fn table() -> TokenTable {
    let source = ThemeSource::shadcn().unwrap();
    TokenTable::baked(&source).unwrap()
}

/// The keyword annotations parallel to a compiled program's flattened
/// declarations (the gate's row-text channel).
fn keywords_of(literal: &str) -> Vec<&'static str> {
    let compiled = compile_theme(literal, &table(), &Config::default()).unwrap();
    let texts = keyword_texts(literal, &Config::default()).unwrap();
    aeolus_compile::keyword_alignment(&compiled, &texts)
}

#[test]
fn the_two_stop_linear_family_joins_the_native_tier() {
    // The stops and their positions are native (GPUI's stops carry
    // colours and percentages); the direction longhand is gated by the
    // ROW TEXT (below), and the via stop stays out (three stops).
    assert!(supports(Prop::GradientFrom));
    assert!(supports(Prop::GradientTo));
    assert!(supports(Prop::GradientFromPosition));
    assert!(supports(Prop::GradientToPosition));
    assert!(!supports(Prop::GradientVia));
    assert!(!supports(Prop::BackgroundImage), "text-gated, not per-prop");
    // The polyfill tier stays the ring family's — gradients are native.
    assert!(!polyfilled(Prop::GradientFrom));
}

#[test]
fn a_two_stop_linear_program_emits() {
    let literal = "bg-linear-to-r from-red-500 to-blue-500";
    let keywords = keywords_of(literal);
    let compiled = compile_theme(literal, &table(), &Config::default()).unwrap();
    let tokens = GpuiEmit::new(&keywords)
        .emit(&compiled.program)
        .expect("the 2-stop linear program expresses");
    let compact = tokens.to_string().replace(' ', "");
    assert!(compact.contains("StaticProgram::new"));
    assert!(
        compact.contains("GradientFrom"),
        "the stop longhands ride the frozen program: {compact}"
    );
}

#[test]
fn the_via_stop_refuses_with_the_two_stop_reason() {
    let diagnostics: Diagnostics = lower_class(
        "bg-linear-to-r from-red-500 via-white to-blue-500",
        &Config::default(),
    )
    .unwrap_err();
    let via = diagnostics
        .unsupported
        .iter()
        .find(|entry| entry.prop == Prop::GradientVia)
        .expect("the via stop refuses");
    assert!(
        via.reason.contains("two stops"),
        "the reason names the slot's shape: {via:?}"
    );
}

#[test]
fn the_non_linear_direction_texts_refuse() {
    // The row text is the annotation the bare IR cannot recover: the
    // radial/conic/placeholder forms carry no `linear-gradient(to …)`
    // direction, and GPUI's `Fill` has no slot for the shape.
    for literal in [
        "bg-radial from-red-500 to-blue-500",
        "bg-conic from-red-500 to-blue-500",
        "bg-linear",
    ] {
        let diagnostics: Diagnostics = lower_class(literal, &Config::default()).unwrap_err();
        let entry = diagnostics
            .unsupported
            .iter()
            .find(|entry| entry.prop == Prop::BackgroundImage)
            .unwrap_or_else(|| panic!("{literal}: the direction text refuses"));
        assert!(
            entry.reason.contains("linear"),
            "{literal}: the reason names the linear-only slot: {entry:?}"
        );
    }
}

#[test]
fn bg_none_and_direction_only_programs_emit() {
    // `bg-none` is the explicit clear (the walk composes nothing); a
    // direction with only a from stop composes the v4 transparent
    // to-stop default.
    lower_class("bg-none", &Config::default()).expect("bg-none expresses");
    lower_class("bg-linear-to-b from-red-500", &Config::default())
        .expect("direction + from expresses");
}

#[test]
fn the_missing_annotation_refuses() {
    // A BackgroundImage decl with NO row text (the bare `GpuiEmit`
    // default) cannot be gate-checked — the annotation IS the verdict's
    // input, so the gate refuses rather than guessing.
    let compiled = compile_theme("bg-linear-to-r", &table(), &Config::default()).unwrap();
    let err = GpuiEmit::default().emit(&compiled.program).unwrap_err();
    assert!(
        err.iter().any(|entry| entry.prop == Prop::BackgroundImage),
        "the text-less gate refuses: {err:?}"
    );
}
