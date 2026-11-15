//! The capability GATE at the macro surface (the honest-failure
//! contract's `tw!` half): `audit` walks a frozen program's
//! declarations through the SAME `unsupported_reason` gates the
//! `Emit` impl renders — the publish-the-emitter's-verdicts rule — so
//! the macro's freeze step can refuse a class the GPUI target cannot
//! express (the motion families, the gradient via stop) as a COMPILE
//! error naming the class, per plan §9's per-class diagnostic story.
//!
//! Written FIRST (the RED spec): the suite fails while `audit` does
//! not exist and the macros freeze silently.

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

/// The plain surface audits clean: layout, colour, typography and the
/// RING polyfill (the polyfilled tier the emit gates carry) pass.
#[test]
fn audit_passes_the_expressible_surface() {
    let (program, keywords) = frozen("p-4 bg-red-500 rounded-md hover:ring-4 ring-red-500");
    assert!(
        audit(&program, &keywords).is_empty(),
        "the expressible surface (native + polyfilled) audits clean"
    );
}

/// The motion families refuse: GPUI's style surface carries no
/// transition slot, and `with_animation` needs an element id the
/// frozen static program does not name (the emit gates' own reasons).
#[test]
fn audit_refuses_the_motion_families() {
    for literal in ["animate-spin", "transition-colors duration-300 ease-in-out"] {
        let (program, keywords) = frozen(literal);
        let refusals = audit(&program, &keywords);
        assert!(
            !refusals.is_empty(),
            "{literal}: the motion longhands refuse"
        );
        assert!(
            refusals.iter().any(|u| matches!(
                u.prop,
                Prop::Animation | Prop::TransitionProperty | Prop::TransitionDuration
            )),
            "{literal}: the refusal names the motion longhand, got {refusals:?}"
        );
    }
}

/// The gradient via stop refuses (the two-stop limit the emit gate
/// carries), and the refusal is the EMITTER'S verdict — the same
/// `unsupported_reason` the `Emit` impl renders, never a parallel
/// claim.
#[test]
fn audit_refuses_the_gradient_via_stop() {
    let source = ThemeSource::shadcn().expect("embedded palette parses");
    let tokens = TokenTable::baked(&source).expect("the baked table builds");
    let compiled = compile_theme(
        "from-red-500 via-blue-500 to-green-500",
        &tokens,
        &Config::default(),
    )
    .expect("compiles");
    let annotations = keyword_texts("from-red-500 via-blue-500 to-green-500", &Config::default())
        .expect("annotates");
    let keywords = keyword_alignment(&compiled, &annotations);
    let refusals = audit(&compiled.program, &keywords);
    assert!(
        refusals.iter().any(|u| u.prop == Prop::GradientVia),
        "the via stop refuses (the two-stop limit), got {refusals:?}"
    );
}

// ---------------------------------------------------------------------------
// The decoration family (the §3.7 typography surface's landed half): the
// colour and thickness longhands are NATIVE (GPUI's pinned surface
// carries `text_decoration_color` and the `text_decoration_0..8`
// thickness methods), the style keywords split by the pinned line shape
// (solid/wavy expressible; double/dotted/dashed refuse — GPUI renders a
// single solid line), and the thickness keywords/auto forms refuse (the
// absolute-pixel slot has no auto/from-font answer).
// ---------------------------------------------------------------------------

/// The decoration family's expressible surface audits clean: the line
/// row, the colour row, the numeric thickness, the solid/wavy styles.
#[test]
fn audit_passes_the_expressible_decoration_surface() {
    for literal in [
        "underline decoration-red-500 decoration-2",
        "underline decoration-wavy decoration-solid decoration-[3px]",
        "line-through decoration-[#0f0f0f]",
    ] {
        let (program, keywords) = frozen(literal);
        let refusals = audit(&program, &keywords);
        assert!(
            refusals.is_empty(),
            "{literal}: the expressible decoration surface audits clean, got {refusals:?}"
        );
    }
}

/// The style keywords GPUI renders no slot for refuse per-decl (the row
/// TEXT's verdict, the box-shadow preset gate's pattern), and the
/// thickness keywords/percentage forms refuse on the absolute-px slot.
#[test]
fn audit_refuses_the_unrenderable_style_and_thickness_forms() {
    for literal in [
        "decoration-double",
        "decoration-dotted",
        "decoration-dashed",
        "decoration-auto",
        "decoration-from-font",
        "decoration-[50%]",
        "decoration-[0.25rem]",
    ] {
        let (program, keywords) = frozen(literal);
        let refusals = audit(&program, &keywords);
        assert!(
            !refusals.is_empty(),
            "{literal}: the unrenderable form refuses at the gate"
        );
    }
}
