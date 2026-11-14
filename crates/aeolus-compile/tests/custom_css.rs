//! The custom-authoring seam's CSS-FILE form — the architecture plan
//! §5.4 "same file as web" half the typed API form left open: the
//! `@custom-variant name (selector);` statements and the
//! `@utility name { @apply …; }` composite bodies a project's theme CSS
//! carries lower onto the SAME typed [`Customs`] surface the API form
//! registers, and the composite (`applied`) utilities carry the
//! multi-decl bodies — different vals per longhand — the single-`Val`
//! fixed/functional forms cannot hold, with the applied body's keyword
//! texts riding the registration (the parallel-slice contract the
//! static program freezes).
//!
//! Families:
//! 1. **The applied utility** — `@utility brand-badge { @apply …; }`
//!    compiles as its apply body's own classes (the differential: the
//!    spliced decls EQUAL the body compiled directly); the candidate's
//!    own variant composes (`hover:brand-badge` splices under
//!    [`Cond::Hover`]); the modifier/negation refusals are the custom
//!    surface's own (no bake of its own).
//! 2. **The annotation seam** — `keyword_texts_with` carries the
//!    customs' applied keyword texts (the row texts the bare
//!    `keyword_texts` walk cannot name for a registered body), so the
//!    alignment stays row-true for keyword-valued classes (`hidden` →
//!    `none`).
//! 3. **The variant statements** — the `@custom-variant` selector forms
//!    fold onto the typed `Cond` surface through the audited selector
//!    table; every spelling outside it is a typed refusal (never a
//!    guessed alias).
//! 4. **The typed refusal set** — the block/bare variant forms, the
//!    raw-CSS (non-`@apply`) utility body, the variant-carrying apply
//!    body (the flat `Cond` model's boundary), the unknown apply class
//!    and the empty apply body refuse with their reasons named.
//! 5. **The additive contract** — a CSS with no custom statements
//!    yields empty registrations (byte-identical behaviour), built-in
//!    roots win their collisions, and registration order resolves
//!    (a later `@apply` may reference an earlier custom root).
//!
//! Written FIRST (the RED spec): the suite does not compile against a
//! tree without the seam (`customs_from_css`, `CustomCssError`,
//! `CustomValue::Applied`, `keyword_texts_with` do not exist before the
//! implementation), which is this slice's RED state by construction.

use aeolus_compile::{
    Config, CustomCssError, CustomValue, Customs, DiagnosticKind, Spacing, compile, compile_with,
    emit_css_with, keyword_alignment, keyword_texts_with,
};
use aeolus_ir::{Cond, Decl, Prop, Val};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// The base layer's declarations (layer 0 of a compiled program).
fn base(program: &aeolus_ir::Program) -> &[Decl] {
    program
        .layers
        .iter()
        .find(|layer| layer.cond == Cond::Base)
        .map(|layer| layer.decls)
        .unwrap_or(&[])
}

/// The applied-body CSS the examples pin: one composite badge utility
/// (four radii + two paddings — different vals per longhand, the shape
/// the single-`Val` forms cannot hold) and one keyword-carrying utility
/// (the `hidden` row's `Val::Kw` slot, the annotation seam's subject).
const BADGE_CSS: &str = "@utility brand-badge { @apply rounded-md px-2 py-0.5; }\n\
                         @utility brand-gone { @apply hidden; }\n";

/// The badge utility's registrations, parsed from the file form.
fn badge_customs() -> Customs {
    customs_from_css_ok(BADGE_CSS)
}

/// `customs_from_css` with the error unwrapped (the happy-path helper).
fn customs_from_css_ok(css: &str) -> Customs {
    aeolus_compile::customs_from_css(css, &Config::default()).expect("parses")
}

// ---------------------------------------------------------------------------
// 1. The applied utility: compile parity with its apply body
// ---------------------------------------------------------------------------

#[test]
fn applied_utility_compiles_as_its_apply_body() {
    let config = Config::default();
    let customs = badge_customs();
    let compiled = compile_with("brand-badge", &customs, &config).expect("compiles");
    let direct = compile("rounded-md px-2 py-0.5", &config).expect("compiles");
    // The differential: the spliced base decls EQUAL the apply body's
    // own base decls (same longhands, same vals, same cascade order —
    // the composite utility is the body, not a re-lowering of it).
    assert_eq!(base(&compiled.program), base(&direct.program));
    // The shape the single-Val forms cannot hold: FOUR radii slots and
    // THREE distinct values in one registration — the radii ride their
    // row-relative `Val::Kw` slots (the parity with the direct compile
    // is the assertion above), the paddings ride the spacing scale's
    // rem forms.
    let props: Vec<Prop> = base(&compiled.program).iter().map(|d| d.prop).collect();
    assert!(props.contains(&Prop::BorderTopLeftRadius));
    assert!(props.contains(&Prop::BorderBottomRightRadius));
    assert!(props.contains(&Prop::PaddingLeft));
    assert!(props.contains(&Prop::PaddingTop));
    let vals: Vec<Val> = base(&compiled.program).iter().map(|d| d.val).collect();
    assert!(vals.contains(&Val::Rem(0.125)));
    assert!(vals.contains(&Val::Rem(0.5)));
}

#[test]
fn applied_utility_cond_composes() {
    let config = Config::default();
    let customs = badge_customs();
    let compiled = compile_with("hover:brand-badge", &customs, &config).expect("compiles");
    // The candidate's own variant is the layer's condition; the applied
    // body splices under it (the flat Cond model's composition rule).
    let hover = compiled
        .program
        .layers
        .iter()
        .find(|layer| layer.cond == Cond::Hover)
        .expect("hover layer");
    let direct = compile("rounded-md px-2 py-0.5", &config).expect("compiles");
    assert_eq!(hover.decls, base(&direct.program));
}

#[test]
fn applied_utility_wins_its_registry_miss_and_loses_the_collision() {
    let config = Config::default();
    // The registry is the oracle: a built-in root `p` stays the padding
    // scale even when a custom `p` is registered.
    let css = "@utility p { @apply hidden; }\n";
    let customs = customs_from_css_ok(css);
    let compiled = compile("p-4", &config).expect("compiles");
    assert!(
        base(&compiled.program)
            .iter()
            .any(|d| d.prop == Prop::PaddingTop),
        "the built-in p-4 row wins the collision"
    );
    let _ = compile_with("p", &customs, &config).expect("the custom root lowers on its own");
}

// ---------------------------------------------------------------------------
// 2. The annotation seam: the applied keyword texts ride the registration
// ---------------------------------------------------------------------------

#[test]
fn applied_keyword_texts_flow_through_the_alignment() {
    let config = Config::default();
    let customs = badge_customs();
    let compiled = compile_with("brand-gone flex-col", &customs, &config).expect("compiles");
    let annotations =
        keyword_texts_with("brand-gone flex-col", &customs, &config).expect("annotations");
    let texts = keyword_alignment(&compiled, &annotations);
    // The `hidden` row's text lands on the applied Display decl (the
    // bare keyword_texts walk cannot name it — the utility carries no
    // row table of its own); the built-in `flex-col` text lands beside.
    let mut texts = texts.into_iter();
    let mut display_none = false;
    let mut flex_column = false;
    for layer in compiled.program.layers {
        for decl in layer.decls {
            let text = texts.next().unwrap_or_default();
            if decl.prop == Prop::Display && text == "none" {
                display_none = true;
            }
            if decl.prop == Prop::FlexDirection && text == "column" {
                flex_column = true;
            }
        }
    }
    assert!(display_none, "the applied `hidden` text aligns");
    assert!(flex_column, "the built-in `flex-col` text aligns");
}

#[test]
fn empty_customs_keep_keyword_texts_byte_identical() {
    let config = Config::default();
    let customs = customs_from_css_ok("@theme { --color-brand: red; }\n");
    let a = keyword_texts_with("hidden flex-col", &customs, &config).expect("annotations");
    let b = aeolus_compile::keyword_texts("hidden flex-col", &config).expect("annotations");
    assert_eq!(a.len(), b.len());
    for (x, y) in a.iter().zip(b.iter()) {
        assert_eq!(x.keywords, y.keywords);
        assert_eq!(x.cond, y.cond);
    }
}

// ---------------------------------------------------------------------------
// 3. The CSS arm renders the applied body
// ---------------------------------------------------------------------------

#[test]
fn emit_css_renders_the_applied_body() {
    let config = Config::default();
    let customs = badge_customs();
    let sheet = emit_css_with("brand-badge", &customs, &config).expect("renders");
    assert!(sheet.diagnostics.is_empty(), "no refusals expected");
    assert_eq!(sheet.rules.len(), 1);
    assert_eq!(sheet.rules[0].selector, ".brand-badge");
    assert!(sheet.rules[0].body.contains("padding-left: 0.5rem"));
    assert!(sheet.rules[0].body.contains("padding-top: 0.125rem"));
    // The radii render the body's own rem form.
    assert!(
        sheet.rules[0]
            .body
            .contains("border-top-left-radius: 0.375rem")
    );
}

#[test]
fn applied_refuses_modifier_and_negation() {
    let config = Config::default();
    let customs = badge_customs();
    for input in ["brand-badge/50", "-brand-badge"] {
        let compiled = compile_with(input, &customs, &config).expect("compiles");
        let reasons = compiled
            .diagnostics
            .iter()
            .filter(|d| {
                matches!(
                    d.kind,
                    DiagnosticKind::UnsupportedModifier { .. }
                        | DiagnosticKind::UnsupportedNegation { .. }
                )
            })
            .count();
        assert_eq!(reasons, 1, "one typed refusal for {input}");
    }
}

// ---------------------------------------------------------------------------
// 4. The variant statements fold onto the typed Cond surface
// ---------------------------------------------------------------------------

#[test]
fn custom_variant_statements_fold_to_their_conds() {
    let config = Config::default();
    let css = "@custom-variant night (&:where(.dark, .dark *));\n\
               @custom-variant hocus (&:hover);\n\
               @custom-variant focusable (&:focus);\n";
    let customs = customs_from_css_ok(css);
    for (input, cond) in [
        ("night:opacity-50", Cond::Dark),
        ("hocus:opacity-50", Cond::Hover),
        ("focusable:opacity-50", Cond::Focus),
    ] {
        let compiled = compile_with(input, &customs, &config).expect("compiles");
        assert!(
            compiled
                .program
                .layers
                .iter()
                .any(|layer| layer.cond == cond
                    && layer.decls.iter().any(|d| d.prop == Prop::Opacity)),
            "{input} folds to {cond:?}"
        );
    }
}

#[test]
fn unknown_selector_is_a_typed_refusal() {
    let err = aeolus_compile::customs_from_css(
        "@custom-variant wide (min-width: 40rem);\n",
        &Config::default(),
    )
    .expect_err("the arbitrary selector has no flat Cond slot");
    assert!(matches!(err, CustomCssError::Selector { .. }));
    let message = err.to_string();
    assert!(message.contains("wide"), "the name is in the message");
    assert!(message.contains("min-width: 40rem"), "the selector is too");
}

#[test]
fn block_and_bare_variant_forms_are_typed_refusals() {
    let config = Config::default();
    // The block form's selector body has no flat Cond slot.
    let err = aeolus_compile::customs_from_css(
        "@custom-variant card {\n  &:where([data-card]);\n}\n",
        &config,
    )
    .expect_err("the block form refuses");
    assert!(matches!(err, CustomCssError::Selector { .. }));
    // The bare form (no selector) is the malformed statement.
    let err = aeolus_compile::customs_from_css("@custom-variant bare;\n", &config)
        .expect_err("the bare form refuses");
    assert!(matches!(err, CustomCssError::Statement { .. }));
}

// ---------------------------------------------------------------------------
// 5. The apply body's typed refusal set
// ---------------------------------------------------------------------------

#[test]
fn variant_carrying_apply_body_is_a_typed_refusal() {
    let err = aeolus_compile::customs_from_css(
        "@utility brand-x { @apply hover:p-4; }\n",
        &Config::default(),
    )
    .expect_err("the flat Cond model carries no slot for the composed condition");
    assert!(matches!(err, CustomCssError::ApplyBody { .. }));
}

#[test]
fn unknown_apply_class_is_a_typed_refusal() {
    let err = aeolus_compile::customs_from_css(
        "@utility brand-x { @apply not-a-thing; }\n",
        &Config::default(),
    )
    .expect_err("the honest-failure contract reaches registration");
    assert!(matches!(err, CustomCssError::ApplyBody { .. }));
    let message = err.to_string();
    assert!(
        message.contains("brand-x"),
        "the utility's name is in the message"
    );
}

#[test]
fn utility_without_apply_is_a_typed_refusal() {
    // The raw-CSS declaration body is the documented boundary: the CSS
    // declaration grammar has no IR lowering for the applied form, so
    // the body refuses typed instead of guessing a fan-out. (The
    // FUNCTIONAL star bodies lower — the M2 slice's subject, pinned in
    // `tests/custom_css_functional.rs`.)
    let err = aeolus_compile::customs_from_css(
        "@utility brand-x { border-radius: 0.375rem; }\n",
        &Config::default(),
    )
    .expect_err("the raw body refuses");
    assert!(matches!(err, CustomCssError::Body { .. }));
}

#[test]
fn empty_apply_body_is_a_typed_refusal() {
    let err =
        aeolus_compile::customs_from_css("@utility brand-x { @apply; }\n", &Config::default())
            .expect_err("a body that lowers nothing is the one silent failure");
    assert!(matches!(err, CustomCssError::ApplyBody { .. }));
}

// ---------------------------------------------------------------------------
// 6. The additive contract
// ---------------------------------------------------------------------------

#[test]
fn css_without_custom_statements_yields_empty_customs() {
    let customs = customs_from_css_ok(
        "@theme { --color-primary: red; }\n\
         :root { --spacing: 0.25rem; }\n\
         @source inline(\"p-4\");\n",
    );
    assert!(
        customs.is_empty(),
        "theme blocks and @source are not customs"
    );
}

#[test]
fn registration_order_resolves_prior_customs() {
    let config = Config::default();
    let css = "@utility brand-a { @apply px-2; }\n\
               @utility brand-b { @apply brand-a rounded-md; }\n";
    let customs = customs_from_css_ok(css);
    let compiled = compile_with("brand-b", &customs, &config).expect("compiles");
    let direct = compile("px-2 rounded-md", &config).expect("compiles");
    assert_eq!(base(&compiled.program), base(&direct.program));
}

#[test]
fn spacing_config_rides_the_applied_body() {
    let config = Config {
        spacing: Spacing::Px,
        ..Config::default()
    };
    let customs =
        aeolus_compile::customs_from_css("@utility brand-badge { @apply px-2; }\n", &config)
            .expect("parses");
    let compiled = compile_with("brand-badge", &customs, &config).expect("compiles");
    // The px bake: the spacing knob's own unit step (the built-in px-2
    // under Spacing::Px), byte-shared with the registry value pass.
    let direct = compile("px-2", &config).expect("compiles");
    assert_eq!(base(&compiled.program), base(&direct.program));
}

// ---------------------------------------------------------------------------
// 7. The applied registration's audit surface
// ---------------------------------------------------------------------------

#[test]
fn applied_value_is_the_registered_body() {
    let customs = badge_customs();
    let utilities = customs.utilities();
    let badge = utilities
        .iter()
        .find(|u| u.root == "brand-badge")
        .expect("the badge registers");
    assert!(matches!(badge.value, CustomValue::Applied(_)));
    let gone = utilities
        .iter()
        .find(|u| u.root == "brand-gone")
        .expect("the hidden utility registers");
    let CustomValue::Applied(body) = gone.value else {
        panic!("the keyword utility registers applied");
    };
    assert_eq!(body.decls.len(), 1);
    assert_eq!(body.decls[0].prop, Prop::Display);
    // The keyword text rides beside the decl (the parallel-slice
    // contract): the `hidden` row's `none`.
    assert_eq!(body.keywords, &[(Prop::Display, "none")]);
}
