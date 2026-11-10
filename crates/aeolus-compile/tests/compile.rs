//! The compile-seam contracts, pinned end to end (architecture plan §3.1:
//! "every stage is deterministic and total").
//!
//! Four families:
//! 1. **Value lowering** — the suffix → `Val` pass per registry shape:
//!    the spacing scale (rem by default, px under the compat config),
//!    the px step, negation, fractions, numbers, and the row-table
//!    keyword slots.
//! 2. **Program shape** — the cascade contract: layer 0 is the base,
//!    the rest are specificity-sorted, masks mirror decls, and the
//!    base-conflict projection is exactly the bits a later base group
//!    re-sets (`p-4 px-8`).
//! 3. **Variant folding** — the flat-`Cond` model: typed variant names
//!    fold to the dominant condition; unknown variants are typed
//!    diagnostics, never silent drops.
//! 4. **Honest boundaries** — everything this slice does NOT lower
//!    (importance, modifiers, colour tokens, arbitrary values) comes
//!    back as a spanned diagnostic with a stable code and help text,
//!    per the no-silent-failures rule (CONTRIBUTING §2).
//!
//! Written FIRST (the RED spec): the suite fails against a stub crate
//! and passes only once `compile()` satisfies every pin below.

use aeolus_compile::{compile, CompileError, Config, DiagnosticKind, Spacing};
use aeolus_data::tables;
use aeolus_ir::{Bp, Cond, Decl, Program, Prop, PropMask, Val};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// The index of `name` inside a keyword table — the SAME table the row
/// carries, so the pin is the CONTRACT (index into the row's own table),
/// not a magic number.
fn kw_index(table: &[(&str, &str)], name: &str) -> u8 {
    let idx = table
        .iter()
        .position(|&(entry, _)| entry == name)
        .expect("test table lookup must succeed");
    assert!(idx <= u8::MAX as usize, "table must fit the IR's u8 slot");
    idx as u8
}

/// The composite-table sibling of [`kw_index`] (`(&str, &str, &str)` rows).
fn kw_index3(table: &[(&str, &str, &str)], name: &str) -> u8 {
    let idx = table
        .iter()
        .position(|&(entry, _, _)| entry == name)
        .expect("test table lookup must succeed");
    assert!(idx <= u8::MAX as usize, "table must fit the IR's u8 slot");
    idx as u8
}

/// The base layer's declarations (layer 0 of a compiled program).
fn base(program: &Program) -> &[Decl] {
    program.layers.first().map_or(&[], |l| l.decls)
}

const PADDING_MASK: PropMask = PropMask::of(&[
    Prop::PaddingTop,
    Prop::PaddingRight,
    Prop::PaddingBottom,
    Prop::PaddingLeft,
]);
const PADDING_X_MASK: PropMask = PropMask::of(&[Prop::PaddingRight, Prop::PaddingLeft]);

// ---------------------------------------------------------------------------
// 1. Value lowering
// ---------------------------------------------------------------------------

#[test]
fn p4_lowers_to_the_trbl_rem_fanout() {
    let compiled = compile("p-4", &Config::default()).expect("p-4 compiles");
    assert!(compiled.diagnostics.is_empty(), "no diagnostics: {compiled:?}");
    let program = compiled.program;
    assert!(program.is_sorted(), "single base layer is trivially sorted");
    let layer = program.layers.first().expect("base layer exists");
    assert_eq!(layer.cond, Cond::Base);
    assert_eq!(layer.mask, PADDING_MASK, "mask mirrors the fan-out");
    assert_eq!(
        layer.decls,
        [
            Decl { prop: Prop::PaddingTop, val: Val::Rem(1.0) },
            Decl { prop: Prop::PaddingRight, val: Val::Rem(1.0) },
            Decl { prop: Prop::PaddingBottom, val: Val::Rem(1.0) },
            Decl { prop: Prop::PaddingLeft, val: Val::Rem(1.0) },
        ]
        .as_slice(),
        "4 × 0.25rem, TRBL order"
    );
    assert_eq!(program.touched(), PADDING_MASK);
}

#[test]
fn px_config_bakes_pixels_at_the_configured_rem() {
    let config = Config { spacing: Spacing::Px, ..Config::default() };
    let compiled = compile("p-2", &config).expect("p-2 compiles");
    let decls = base(&compiled.program);
    assert_eq!(decls[0], Decl { prop: Prop::PaddingTop, val: Val::Px(8.0) });
    // The `px` step is one PIXEL in both modes — it is already absolute.
    let compiled = compile("pb-px", &Config::default()).expect("pb-px compiles");
    let decls = base(&compiled.program);
    assert_eq!(decls[0], Decl { prop: Prop::PaddingBottom, val: Val::Px(1.0) });
}

#[test]
fn negated_spacing_negates_the_value() {
    let compiled = compile("-mt-2", &Config::default()).expect("-mt-2 compiles");
    let layer = compiled.program.layers.first().expect("base layer exists");
    assert_eq!(layer.decls, [Decl { prop: Prop::MarginTop, val: Val::Rem(-0.5) }].as_slice());
    assert!(compiled.diagnostics.is_empty());
}

#[test]
fn fraction_lowers_to_a_percentage() {
    let compiled = compile("w-1/2", &Config::default()).expect("w-1/2 compiles");
    let decls = base(&compiled.program);
    assert_eq!(decls, [Decl { prop: Prop::Width, val: Val::Pct(50.0) }].as_slice());
}

#[test]
fn number_scale_is_unitless() {
    let compiled = compile("z-10 opacity-50", &Config::default()).expect("compiles");
    let decls = base(&compiled.program);
    assert!(decls.contains(&Decl { prop: Prop::ZIndex, val: Val::Num(10.0) }));
    assert!(decls.contains(&Decl { prop: Prop::Opacity, val: Val::Num(50.0) }));
}

#[test]
fn keywords_lower_to_the_row_table_slot() {
    let compiled = compile("flex-col", &Config::default()).expect("flex-col compiles");
    let decls = base(&compiled.program);
    let slot = kw_index(tables::FLEX_DIRECTIONS, "col");
    assert_eq!(decls, [Decl { prop: Prop::FlexDirection, val: Val::Kw(slot) }].as_slice());
}

#[test]
fn bare_roots_lower_to_slot_zero() {
    let compiled = compile("hidden", &Config::default()).expect("hidden compiles");
    let decls = base(&compiled.program);
    assert_eq!(decls, [Decl { prop: Prop::Display, val: Val::Kw(0) }].as_slice());
}

#[test]
fn size_keywords_resolve_through_the_anyof_arm() {
    // `w-full` hits the size-keyword arm inside the row's AnyOf, not the
    // spacing arm — the arm choice is the lowering's, first-accept wins.
    let compiled = compile("w-full", &Config::default()).expect("w-full compiles");
    let decls = base(&compiled.program);
    let slot = kw_index(tables::SIZE_KEYWORDS, "full");
    assert_eq!(decls, [Decl { prop: Prop::Width, val: Val::Kw(slot) }].as_slice());
}

#[test]
fn composite_rows_fan_the_pair_across_their_targets() {
    // `text-lg` is the corpus's two-headed utility: font-size AND the
    // paired line-height, both indexed into the same TEXT_SIZES entry.
    let compiled = compile("text-lg", &Config::default()).expect("text-lg compiles");
    let decls = base(&compiled.program);
    let slot = kw_index3(tables::TEXT_SIZES, "lg");
    assert_eq!(
        decls,
        [
            Decl { prop: Prop::FontSize, val: Val::Kw(slot) },
            Decl { prop: Prop::LineHeight, val: Val::Kw(slot) },
        ]
        .as_slice(),
        "one candidate, two longhands, one slot each"
    );
}

// ---------------------------------------------------------------------------
// 2. Program shape
// ---------------------------------------------------------------------------

#[test]
fn base_and_conditional_layers_sort_by_specificity() {
    let compiled = compile("p-4 hover:p-0 md:gap-2", &Config::default()).expect("compiles");
    let layers = compiled.program.layers;
    assert_eq!(layers.len(), 3, "base + hover + md");
    assert_eq!(layers[0].cond, Cond::Base);
    assert_eq!(layers[1].cond, Cond::Hover);
    assert_eq!(layers[2].cond, Cond::Bp(Bp::Md));
    assert!(compiled.program.is_sorted(), "specificity non-decreasing");
    for layer in layers {
        let mut expected = PropMask::EMPTY;
        for decl in layer.decls {
            expected = expected.insert(decl.prop);
        }
        assert_eq!(layer.mask, expected, "mask is data, not cache");
    }
}

#[test]
fn same_condition_duplicates_surface_as_base_conflicts() {
    // `p-4 px-8`: the IR-level cn() merge is a later P2 slice; today the
    // raw cascade is emitted (later declaration wins on a front-to-back
    // read) and the overlap is EXACTLY the base-conflict bits.
    let compiled = compile("p-4 px-8", &Config::default()).expect("compiles");
    let layer = compiled.program.layers.first().expect("base layer exists");
    assert_eq!(layer.decls.len(), 6, "the 4+2 fan-out, later declaration wins per prop");
    assert_eq!(compiled.program.base_conflicts(), PADDING_X_MASK);
}

// ---------------------------------------------------------------------------
// 3. Variant folding
// ---------------------------------------------------------------------------

#[test]
fn stacked_variants_fold_to_the_dominant_condition() {
    // Flat-Cond model (cond.rs): the fold keeps the highest-specificity
    // segment; source order breaks ties.
    let compiled = compile("hover:focus:p-0", &Config::default()).expect("compiles");
    assert_eq!(compiled.program.layers.len(), 1);
    assert_eq!(compiled.program.layers[0].cond, Cond::Focus, "later segment wins the tie");

    let compiled = compile("md:hover:p-0", &Config::default()).expect("compiles");
    assert_eq!(compiled.program.layers[0].cond, Cond::Bp(Bp::Md), "media outranks probe");
}

#[test]
fn named_group_variants_fold_to_the_probe() {
    // `group-hover/menu` folds to the GroupHover probe; the NAME has no
    // IR slot yet (documented boundary, cond.rs).
    let compiled = compile("group-hover/menu:p-0", &Config::default()).expect("compiles");
    assert_eq!(compiled.program.layers[0].cond, Cond::GroupHover);
}

#[test]
fn unsupported_variants_are_typed_and_skip_the_utility() {
    // The lexer accepts the segment; the typed Cond surface does not
    // cover it yet. Honest report, the REST of the string still compiles.
    let compiled = compile("has-data-[x]:p-4 p-2", &Config::default()).expect("compiles");
    assert_eq!(compiled.diagnostics.len(), 1, "one report: {compiled:?}");
    let d = &compiled.diagnostics[0];
    assert_eq!(
        d.kind,
        DiagnosticKind::UnsupportedVariant { variant: "has-data-[x]".to_string() }
    );
    assert_eq!(d.span, (0, 12), "the variant segment's own span");
    assert_eq!(d.code(), "aeolus::compile::unsupported_variant");
    assert!(d.help().contains("Cond"), "help names the typed surface");
    let decls = base(&compiled.program);
    assert!(decls.contains(&Decl { prop: Prop::PaddingTop, val: Val::Rem(1.0) }));
}

// ---------------------------------------------------------------------------
// 4. Honest boundaries
// ---------------------------------------------------------------------------

#[test]
fn importance_is_a_typed_boundary() {
    let compiled = compile("p-4!", &Config::default()).expect("compiles");
    assert_eq!(compiled.diagnostics.len(), 1);
    assert_eq!(compiled.diagnostics[0].kind, DiagnosticKind::UnsupportedImportant);
    assert_eq!(compiled.diagnostics[0].code(), "aeolus::compile::unsupported_important");
    assert!(compiled.program.layers.is_empty(), "nothing lowered");
}

#[test]
fn modifiers_are_a_typed_boundary() {
    let compiled = compile("bg-red-500/50", &Config::default()).expect("compiles");
    assert_eq!(compiled.diagnostics.len(), 1);
    assert_eq!(
        compiled.diagnostics[0].kind,
        DiagnosticKind::UnsupportedModifier { modifier: "50".to_string() }
    );
    assert!(compiled.diagnostics[0].help().contains("theme pass"));
}

#[test]
fn colour_tokens_report_the_theme_pass() {
    // The seam's discovered IR-budget constraint: the colour table is
    // 270 entries and `Val::Kw` is a u8 slot — tokens lower through the
    // theme pass's TokenTable (plan §3.4 T2), not through this seam.
    let compiled = compile("bg-red-500", &Config::default()).expect("compiles");
    assert_eq!(compiled.diagnostics.len(), 1);
    assert!(matches!(
        &compiled.diagnostics[0].kind,
        DiagnosticKind::UnsupportedValue { reason, .. } if reason.contains("theme pass")
    ));
    assert_eq!(compiled.diagnostics[0].code(), "aeolus::compile::unsupported_value");
    assert!(compiled.program.layers.is_empty());
}

#[test]
fn arbitrary_values_report_the_value_pass() {
    let compiled = compile("w-[32px]", &Config::default()).expect("compiles");
    assert_eq!(compiled.diagnostics.len(), 1);
    assert!(matches!(
        &compiled.diagnostics[0].kind,
        DiagnosticKind::UnsupportedValue { reason, .. } if reason.contains("value pass")
    ));
}

#[test]
fn negation_the_row_refuses_is_typed() {
    // `-p-4` is not a legal v4 spelling and the row's `negative` flag
    // says so — a diagnostic, never a silent sign drop.
    let compiled = compile("-p-4", &Config::default()).expect("compiles");
    assert_eq!(compiled.diagnostics.len(), 1);
    assert_eq!(
        compiled.diagnostics[0].kind,
        DiagnosticKind::UnsupportedNegation { body: "p-4".to_string() }
    );
    assert!(compiled.program.layers.is_empty());
}

#[test]
fn unknown_roots_carry_the_lookup_error() {
    let compiled = compile("p-4 not-a-utility w-1/2", &Config::default()).expect("compiles");
    assert_eq!(compiled.diagnostics.len(), 1, "the miss degrades, the rest compiles");
    let d = &compiled.diagnostics[0];
    assert!(matches!(
        &d.kind,
        DiagnosticKind::UnknownRoot { body } if body == "not-a-utility"
    ));
    assert_eq!(d.span, (4, 17), "the token's own span inside the input");
    let decls = base(&compiled.program);
    assert_eq!(decls.len(), 5, "p-4's fan-out plus w-1/2");
}

#[test]
fn shape_rejections_carry_the_tried_labels() {
    // `p-red`: the root matches, the spacing shape rejects the suffix.
    let compiled = compile("p-red", &Config::default()).expect("compiles");
    assert_eq!(compiled.diagnostics.len(), 1);
    assert!(matches!(
        &compiled.diagnostics[0].kind,
        DiagnosticKind::ShapeRejected { root: "p", tried, .. }
            if tried.contains(&"spacing")
    ));
}

#[test]
fn lex_failures_are_spanned_and_degrade() {
    // The lexer's honest degradation, now with the span the pair lost.
    let compiled = compile("p--4 p-2", &Config::default()).expect("compiles");
    assert_eq!(compiled.diagnostics.len(), 1);
    let d = &compiled.diagnostics[0];
    assert!(matches!(
        &d.kind,
        DiagnosticKind::Lex { message } if message.contains("empty utility element")
    ));
    assert_eq!(d.span, (0, 4));
    assert_eq!(d.code(), "aeolus::compile::lex");
    let decls = base(&compiled.program);
    assert_eq!(decls.len(), 4, "p-2's fan-out still lowered");
}

#[test]
fn empty_input_is_a_typed_error() {
    let err = compile("", &Config::default()).unwrap_err();
    assert!(matches!(err, CompileError::Empty));
    assert!(err.to_string().contains("empty"), "{err}");
    assert!(compile("   ", &Config::default()).is_err(), "whitespace-only is empty");
}

// ---------------------------------------------------------------------------
// Gated instrumentation contracts (the CLI's v0.9.1 pattern)
// ---------------------------------------------------------------------------

#[cfg(feature = "miette")]
#[test]
fn compile_error_carries_miette_help() {
    use miette::Diagnostic;
    let err = compile("  ", &Config::default()).unwrap_err();
    let code = err.code().map(|c| c.to_string());
    assert_eq!(code.as_deref(), Some("aeolus::compile::empty_input"));
    let help = err.help().expect("empty input carries help").to_string();
    assert!(help.contains("non-whitespace"), "{help}");
}

#[cfg(feature = "tracing")]
#[test]
fn compile_with_tracing_spans_keeps_the_contract() {
    // No global subscriber is installed: the spans compile and are
    // no-ops. The observable contract must be unchanged.
    let compiled = compile("p-4 hover:p-0", &Config::default()).expect("compiles");
    assert_eq!(compiled.program.layers.len(), 2);
    assert!(compiled.diagnostics.is_empty());
}
