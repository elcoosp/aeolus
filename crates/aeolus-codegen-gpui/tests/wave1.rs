//! The parity wave 1's audit-gate contracts (the `tw!` freeze's honest
//! acceptance): the five new families' verdicts at the GPUI emit gate —
//! the clamp count and the font-variant-numeric tags are NATIVE (the
//! pinned `Styled::line_clamp`/`font_features` surface, verified at the
//! `gpui-pre` pin), the between-child families are POLYFILL (the
//! `tw_space` wrapper seam composes them, the gate accepts so
//! `tw!("space-x-4")` compiles), and the sr-only composite refuses on
//! its clip leg (the honest full refusal — the composite's other legs
//! ride their own longhands' verdicts).

use aeolus_codegen::conformance::Tier;
use aeolus_codegen_gpui::audit;
use aeolus_compile::{Config, compile_theme, keyword_alignment, keyword_texts};
use aeolus_ir::Prop;
use aeolus_theme::{ThemeSource, TokenTable};

/// The frozen pair the macros compute: the program + its keyword
/// annotations (the `tw!` path, verbatim).
fn frozen(literal: &str) -> (aeolus_ir::Program, Vec<&'static str>) {
    let source = ThemeSource::shadcn().expect("embedded palette parses");
    let tokens = TokenTable::baked(&source).expect("the baked table builds");
    let compiled = compile_theme(literal, &tokens, &Config::default()).expect("compiles");
    let annotations = keyword_texts(literal, &Config::default()).expect("annotates");
    let keywords = keyword_alignment(&compiled, &annotations);
    (compiled.program, keywords)
}

/// The clamp count is NATIVE: the pinned surface carries
/// `Styled::line_clamp` — `tw!("line-clamp-2")` freezes clean.
#[test]
fn the_clamp_count_is_native() {
    let (program, keywords) = frozen("line-clamp-2 overflow-hidden");
    let refusals = audit(&program, &keywords);
    assert!(
        refusals.iter().all(|u| u.prop != Prop::LineClamp),
        "line-clamp-2: the count freezes (got {refusals:?})"
    );
}

/// The `line-clamp-none` reset refuses per-decl: the pinned method
/// SETS a count, the unset has no slot (the emit gate's own reason).
#[test]
fn the_clamp_reset_refuses_per_decl() {
    let (program, keywords) = frozen("line-clamp-none");
    let refusals = audit(&program, &keywords);
    assert!(
        refusals
            .iter()
            .any(|u| u.prop == Prop::LineClamp && u.reason.contains("none reset")),
        "line-clamp-none: the unset refuses with its reason, got {refusals:?}"
    );
}

/// The font-variant-numeric family is NATIVE: the tags compose into
/// the `FontFeatures` slot — `tw!("tabular-nums")` freezes clean.
#[test]
fn the_variant_numeric_family_is_native() {
    let (program, keywords) = frozen(
        "tabular-nums ordinal slashed-zero lining-nums oldstyle-nums proportional-nums diagonal-fractions stacked-fractions normal-nums",
    );
    let refusals = audit(&program, &keywords);
    assert!(
        refusals.iter().all(|u| u.prop != Prop::FontVariantNumeric),
        "the variant family freezes: got {refusals:?}"
    );
}

/// The between-child families are POLYFILL: the gate accepts (the
/// `tw_space` wrapper is the runtime consumer) — `tw!("space-x-4")`
/// compiles, and so do the divide colours/styles and the reverse flips.
#[test]
fn the_between_child_families_are_polyfill() {
    for literal in [
        "space-x-4",
        "space-y-2",
        "-space-x-2",
        "space-x-reverse",
        "space-y-reverse",
        "divide-x",
        "divide-y-2",
        "divide-x-reverse",
        "divide-dashed",
        "divide-primary",
    ] {
        let (program, keywords) = frozen(literal);
        let refusals = audit(&program, &keywords);
        assert!(
            refusals.is_empty(),
            "{literal}: the polyfill tier compiles, got {refusals:?}"
        );
    }
}

/// The sr-only composite REFUSES on its clip leg (the honest verdict:
/// no framework slot carries clip-path) — and not-sr-only beside it.
#[test]
fn the_sr_only_composite_refuses_on_the_clip_leg() {
    for literal in ["sr-only", "not-sr-only"] {
        let (program, keywords) = frozen(literal);
        let refusals = audit(&program, &keywords);
        assert!(
            refusals
                .iter()
                .any(|u| u.prop == Prop::ClipPath && u.reason.contains("clip-path")),
            "{literal}: the clip leg refuses with its reason, got {refusals:?}"
        );
    }
}

/// The capability table's wave-1 rows: the tiers the generator froze
/// match the gate's verdicts (the one-partition-three-renderings pin).
#[test]
fn the_caps_table_rows_match_the_gate() {
    use aeolus_codegen_gpui::{PROP_CAPABILITIES, polyfilled, supports};
    let row_of = |prop: Prop| {
        PROP_CAPABILITIES
            .iter()
            .find(|row| row.prop == prop)
            .expect("the wave-1 rows exist in the committed table")
    };
    assert_eq!(row_of(Prop::LineClamp).tier, Tier::Native);
    assert!(supports(Prop::LineClamp));
    assert_eq!(row_of(Prop::FontVariantNumeric).tier, Tier::Native);
    assert!(supports(Prop::FontVariantNumeric));
    for prop in [
        Prop::SpaceX,
        Prop::SpaceXReverse,
        Prop::SpaceY,
        Prop::SpaceYReverse,
        Prop::DivideX,
        Prop::DivideXReverse,
        Prop::DivideY,
        Prop::DivideYReverse,
        Prop::DivideColor,
        Prop::DivideStyle,
    ] {
        assert_eq!(
            row_of(prop).tier,
            Tier::Polyfill,
            "{prop:?}: the wrapper tier"
        );
        assert!(polyfilled(prop));
    }
    assert_eq!(row_of(Prop::ClipPath).tier, Tier::Refused);
    assert!(!supports(Prop::ClipPath) && !polyfilled(Prop::ClipPath));
}
