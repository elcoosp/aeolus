//! The custom-authoring seam's contracts — `@utility`/`@custom-variant`
//! (architecture plan §5.4 "CSS-first compatibility: `@theme`,
//! `@utility` (incl. functional `--value()`), `@custom-variant`", the
//! remaining P2 slice; plan §3.1's "utilities" entering the pipeline as
//! REGISTERED data, not match arms).
//!
//! Six families:
//! 1. **Registration + precedence** — a registered custom utility
//!    lowers exactly like a registry row; built-in roots win name
//!    collisions (the registry is the oracle, customs are additive);
//!    the typed read side (unknown root vs rejected shape) carries the
//!    CUSTOM root's name, same `LookupError` shape.
//! 2. **Value forms** — the functional forms lower the suffix exactly
//!    like their built-in shapes (spacing honours the `Spacing` config
//!    and the `px` step; integer/number/percentage are the numeric
//!    grammar); the `-` negated form rides functional forms and refuses
//!    on fixed ones; a fixed [`Val::Kw`] is a typed refusal (the
//!    keyword slot is row-relative — a custom utility carries no row
//!    table).
//! 3. **Custom variants** — a registered alias folds to its typed
//!    [`Cond`] in the IR walk; the CSS walk renders the same alias into
//!    the same condition shape (the two walks share the refusal set —
//!    the differential battery pins diagnostics EQUAL with customs in
//!    play); an unregistered alias stays the typed
//!    unsupported-variant boundary.
//! 4. **The CSS arm** — a custom utility renders its rule (escaped
//!    selector, cascade-ordered declarations) through the same value
//!    text forms; the custom alias renders its probe suffix.
//! 5. **The empty-Customs differential** — every `*_with` entry point
//!    with empty registrations is byte-identical to its theme-less
//!    sibling over a battery (the seam is additive, never a behaviour
//!    switch).
//! 6. **The merge** — `cn()` merges custom-lowered programs under the
//!    same last-wins contract (the fan-outs ARE the groups, whatever
//!    registered them).
//!
//! Written FIRST (the RED spec): the suite does not compile against a
//! tree without the seam (the API surface it imports — `Customs`,
//! `CustomUtility`, `CustomForm`, `CustomVariant`, `ClassifyVariant`,
//! `compile_with`, `compile_theme_with`, `emit_css_with`,
//! `emit_css_theme_with` — does not exist before the implementation),
//! which is this slice's RED state by construction.

use aeolus_compile::{
    CustomForm, CustomUtility, CustomVariant, Customs, DiagnosticKind, Spacing, compile,
    compile_theme_with, compile_with, emit_css_with,
};
use aeolus_ir::{Bp, Cond, Decl, Prop, Val};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// The base layer's declarations (layer 0 of a compiled program).
fn base(program: &aeolus_ir::Program) -> &[Decl] {
    program.layers.first().map_or(&[], |layer| layer.decls)
}

/// The runtime-tier table over the shadcn light+dark source (the theme
/// half of `compile_theme_with`).
fn runtime_table() -> aeolus_theme::TokenTable {
    let source = aeolus_theme::ThemeSource::shadcn_dark().expect("embedded palettes parse");
    aeolus_theme::TokenTable::runtime(&source).expect("default-seeded source builds")
}

/// The demo registrations the families share: one functional spacing
/// utility over the padding longhands, one fixed literal, one alias.
fn demo_customs() -> Customs {
    Customs::new()
        .utility(CustomUtility::functional(
            "brand-p",
            CustomForm::Spacing,
            &[
                Prop::PaddingTop,
                Prop::PaddingRight,
                Prop::PaddingBottom,
                Prop::PaddingLeft,
            ],
        ))
        .utility(CustomUtility::fixed(
            "brand-r",
            Val::Rem(0.375),
            &[
                Prop::BorderTopLeftRadius,
                Prop::BorderTopRightRadius,
                Prop::BorderBottomRightRadius,
                Prop::BorderBottomLeftRadius,
            ],
        ))
        .variant(CustomVariant::new("hocus", Cond::Hover))
}

// ---------------------------------------------------------------------------
// 1. Registration + precedence
// ---------------------------------------------------------------------------

#[test]
fn a_registered_functional_utility_lowers_like_a_row() {
    let customs = demo_customs();
    let compiled = compile_with("brand-p-4", &customs, &aeolus_compile::Config::default())
        .expect("non-empty input compiles");
    assert!(
        compiled.diagnostics.is_empty(),
        "no diagnostics: {compiled:?}"
    );
    assert_eq!(
        base(&compiled.program),
        [
            Decl {
                prop: Prop::PaddingTop,
                val: Val::Rem(1.0)
            },
            Decl {
                prop: Prop::PaddingRight,
                val: Val::Rem(1.0)
            },
            Decl {
                prop: Prop::PaddingBottom,
                val: Val::Rem(1.0)
            },
            Decl {
                prop: Prop::PaddingLeft,
                val: Val::Rem(1.0)
            },
        ]
        .as_slice(),
        "4 × 0.25rem through the custom seam, TRBL order"
    );
    let layer = compiled.program.layers.first().expect("base layer exists");
    assert_eq!(layer.cond, Cond::Base, "no variants: the base layer");
}

#[test]
fn built_in_roots_win_name_collisions() {
    // `p` is a built-in root; a custom `p` registration is shadowed by
    // the registry (the registry is the semantics oracle, customs are
    // additive surface). `p-4` keeps lowering through the built-in row.
    let shadowed = Customs::new().utility(CustomUtility::functional(
        "p",
        CustomForm::Number,
        &[Prop::Width],
    ));
    let config = aeolus_compile::Config::default();
    let compiled = compile_with("p-4", &shadowed, &config).expect("non-empty input compiles");
    assert!(
        compiled.diagnostics.is_empty(),
        "the built-in row lowered: {compiled:?}"
    );
    let decls = base(&compiled.program);
    assert!(
        decls.contains(&Decl {
            prop: Prop::PaddingTop,
            val: Val::Rem(1.0)
        }),
        "the built-in padding fan-out, not the custom Width arm"
    );
    assert!(!decls.contains(&Decl {
        prop: Prop::Width,
        val: Val::Num(4.0)
    }));
}

#[test]
fn custom_misses_carry_the_typed_read_side() {
    let customs = demo_customs();
    let config = aeolus_compile::Config::default();

    // Unknown root: neither the registry nor the customs know the body.
    let compiled = compile_with("not-a-utility", &customs, &config).expect("compiles");
    assert!(matches!(
        &compiled.diagnostics[0].kind,
        DiagnosticKind::UnknownRoot { body } if body == "not-a-utility"
    ));

    // Shape rejection: the custom root matched but the suffix fits none
    // of its forms — the SAME LookupError shape the registry reports,
    // naming the CUSTOM root.
    let compiled = compile_with("brand-p-wide", &customs, &config).expect("compiles");
    assert_eq!(compiled.diagnostics.len(), 1, "one report: {compiled:?}");
    match &compiled.diagnostics[0].kind {
        DiagnosticKind::ShapeRejected { body, root, tried } => {
            assert_eq!(body, "brand-p-wide");
            assert_eq!(*root, "brand-p", "the custom root's own name");
            assert!(
                tried.contains(&"spacing"),
                "the form's label is the tried list"
            );
        }
        other => panic!("expected a shape rejection, got {other:?}"),
    }
    assert_eq!(
        compiled.diagnostics[0].kind.code(),
        "aeolus::compile::shape_rejected",
        "the same stable code the built-in path reports"
    );
}

// ---------------------------------------------------------------------------
// 2. Value forms
// ---------------------------------------------------------------------------

#[test]
fn functional_spacing_honours_the_config_and_the_px_step() {
    let customs = demo_customs();
    let config = aeolus_compile::Config {
        spacing: Spacing::Px,
        ..Default::default()
    };
    let compiled = compile_with("brand-p-2", &customs, &config).expect("compiles");
    assert_eq!(
        base(&compiled.program)[0],
        Decl {
            prop: Prop::PaddingTop,
            val: Val::Px(8.0)
        },
        "the px compat config bakes pixels exactly like the built-in scale"
    );
    let compiled = compile_with("brand-p-px", &customs, &config).expect("compiles");
    assert_eq!(
        base(&compiled.program)[0],
        Decl {
            prop: Prop::PaddingTop,
            val: Val::Px(1.0)
        },
        "the px step is one absolute pixel in both modes"
    );
}

#[test]
fn numeric_forms_lower_their_grammar() {
    let customs = Customs::new()
        .utility(CustomUtility::functional(
            "brand-z",
            CustomForm::Number,
            &[Prop::ZIndex],
        ))
        .utility(CustomUtility::functional(
            "brand-cols",
            CustomForm::Integer,
            &[Prop::GridColumnStart],
        ))
        .utility(CustomUtility::functional(
            "brand-w",
            CustomForm::Percentage,
            &[Prop::Width],
        ));
    let config = aeolus_compile::Config::default();
    let compiled =
        compile_with("brand-z-2.5 brand-cols-3 brand-w-40%", &customs, &config).expect("compiles");
    assert!(
        compiled.diagnostics.is_empty(),
        "no diagnostics: {compiled:?}"
    );
    let decls = base(&compiled.program);
    assert!(decls.contains(&Decl {
        prop: Prop::ZIndex,
        val: Val::Num(2.5)
    }));
    assert!(decls.contains(&Decl {
        prop: Prop::GridColumnStart,
        val: Val::Num(3.0)
    }));
    assert!(decls.contains(&Decl {
        prop: Prop::Width,
        val: Val::Pct(40.0)
    }));
}

#[test]
fn integer_refuses_a_fractional_suffix() {
    let customs = Customs::new().utility(CustomUtility::functional(
        "brand-cols",
        CustomForm::Integer,
        &[Prop::GridColumnStart],
    ));
    let compiled = compile_with(
        "brand-cols-2.5",
        &customs,
        &aeolus_compile::Config::default(),
    )
    .expect("compiles");
    assert_eq!(
        compiled.diagnostics.len(),
        1,
        "the integral grammar refuses: {compiled:?}"
    );
    assert!(matches!(
        &compiled.diagnostics[0].kind,
        DiagnosticKind::ShapeRejected {
            root: "brand-cols",
            ..
        }
    ));
}

#[test]
fn negation_rides_functional_forms_and_refuses_on_fixed() {
    let customs = demo_customs();
    let config = aeolus_compile::Config::default();
    let compiled = compile_with("-brand-p-2", &customs, &config).expect("compiles");
    assert!(
        compiled.diagnostics.is_empty(),
        "functional negation lowers: {compiled:?}"
    );
    assert_eq!(
        base(&compiled.program)[0],
        Decl {
            prop: Prop::PaddingTop,
            val: Val::Rem(-0.5)
        },
    );
    let compiled = compile_with("-brand-r", &customs, &config).expect("compiles");
    assert_eq!(
        compiled.diagnostics[0].kind,
        DiagnosticKind::UnsupportedNegation {
            body: "brand-r".to_string()
        },
        "a fixed value has no sign to flip"
    );
    assert!(compiled.program.layers.is_empty(), "nothing lowered");
}

#[test]
fn a_fixed_keyword_slot_is_a_typed_refusal() {
    // `Val::Kw` indexes the ROW'S OWN keyword table (the codegen crate
    // owns the names, plan §3.3) — a custom utility carries no row
    // table, so the keyword form is the built-in registry's surface.
    // The refusal is typed at lower time, never a slot with no name.
    let customs = Customs::new().utility(CustomUtility::fixed(
        "brand-flex",
        Val::Kw(0),
        &[Prop::FlexDirection],
    ));
    let compiled =
        compile_with("brand-flex", &customs, &aeolus_compile::Config::default()).expect("compiles");
    assert_eq!(compiled.diagnostics.len(), 1, "one report: {compiled:?}");
    assert!(matches!(
        &compiled.diagnostics[0].kind,
        DiagnosticKind::UnsupportedValue { reason, .. } if reason.contains("row")
    ));
    assert!(compiled.program.layers.is_empty());
}

#[test]
fn non_finite_suffixes_refuse_like_the_built_in_grammar() {
    // The f32 grammar's `nan`/`inf` spellings are legal PARSES and
    // illegal v4 values — the built-in two-stage shape: the numeric
    // shape ACCEPTS the parse and the finite guard refuses it at lower
    // time (`REASON_NON_FINITE`, an `UnsupportedValue` report). The
    // custom seam mirrors the same two stages, never a silent literal.
    let customs = Customs::new().utility(CustomUtility::functional(
        "brand-z",
        CustomForm::Number,
        &[Prop::ZIndex],
    ));
    for raw in ["brand-z-nan", "brand-z-inf"] {
        let compiled =
            compile_with(raw, &customs, &aeolus_compile::Config::default()).expect("compiles");
        assert_eq!(compiled.diagnostics.len(), 1, "{raw}: {compiled:?}");
        assert!(matches!(
            &compiled.diagnostics[0].kind,
            DiagnosticKind::UnsupportedValue { reason, .. } if reason.contains("non-finite")
        ));
        assert!(compiled.program.layers.is_empty(), "nothing lowered");
    }
}

// ---------------------------------------------------------------------------
// 3. Custom variants
// ---------------------------------------------------------------------------

#[test]
fn a_registered_alias_folds_to_its_condition() {
    let customs = demo_customs();
    let compiled = compile_with(
        "hocus:brand-p-4",
        &customs,
        &aeolus_compile::Config::default(),
    )
    .expect("compiles");
    assert!(
        compiled.diagnostics.is_empty(),
        "the alias folded: {compiled:?}"
    );
    let layer = compiled.program.layers.first().expect("the layer exists");
    assert_eq!(
        layer.cond,
        Cond::Hover,
        "the alias maps to its typed condition"
    );
}

#[test]
fn stacked_aliases_fold_by_the_dominant_condition() {
    let customs = demo_customs();
    let compiled = compile_with(
        "hocus:md:brand-p-4",
        &customs,
        &aeolus_compile::Config::default(),
    )
    .expect("compiles");
    let layer = compiled.program.layers.first().expect("the layer exists");
    assert_eq!(
        layer.cond,
        Cond::Bp(Bp::Md),
        "media outranks the aliased probe"
    );
}

#[test]
fn an_unregistered_alias_stays_the_typed_boundary() {
    let customs = demo_customs();
    let compiled = compile_with(
        "hoverish:brand-p-4",
        &customs,
        &aeolus_compile::Config::default(),
    )
    .expect("compiles");
    assert_eq!(compiled.diagnostics.len(), 1, "one report: {compiled:?}");
    assert_eq!(
        compiled.diagnostics[0].kind,
        DiagnosticKind::UnsupportedVariant {
            variant: "hoverish".to_string()
        },
        "the alias surface is opt-in; unknown spellings report honestly"
    );
}

#[test]
fn an_alias_of_the_dark_tier_carries_the_mode_condition() {
    // The real-world v4 spell: re-skinning `dark` under a project's own
    // name. The alias folds to `Cond::Dark` — the mode tier — in both
    // walks (the runtime mode SELECTOR is the theme's business; the
    // condition is the IR's).
    let customs = Customs::new().variant(CustomVariant::new("night", Cond::Dark));
    let compiled =
        compile_with("night:p-4", &customs, &aeolus_compile::Config::default()).expect("compiles");
    assert!(compiled.diagnostics.is_empty(), "{compiled:?}");
    let layer = compiled.program.layers.first().expect("the layer exists");
    assert_eq!(layer.cond, Cond::Dark);
    let specificity = layer.cond.specificity();
    assert_eq!(specificity, 1, "the mode tier keeps its rank");
}

// ---------------------------------------------------------------------------
// 4. The CSS arm
// ---------------------------------------------------------------------------

#[test]
fn the_css_arm_renders_a_custom_utility_rule() {
    let customs = demo_customs();
    let sheet = emit_css_with("brand-p-4", &customs, &aeolus_compile::Config::default())
        .expect("non-empty input renders");
    assert!(sheet.diagnostics.is_empty(), "no diagnostics: {sheet:?}");
    assert_eq!(
        sheet.to_string(),
        ".brand-p-4 {\n  padding-top: 1rem;\n  padding-right: 1rem;\n  \
         padding-bottom: 1rem;\n  padding-left: 1rem;\n}\n",
        "the same cascade-ordered declaration block the built-in path renders"
    );
}

#[test]
fn the_css_arm_renders_the_alias_probe() {
    let customs = demo_customs();
    let sheet = emit_css_with(
        "hocus:brand-r",
        &customs,
        &aeolus_compile::Config::default(),
    )
    .expect("non-empty input renders");
    assert!(sheet.diagnostics.is_empty(), "{sheet:?}");
    assert_eq!(
        sheet.to_string(),
        ".hocus\\:brand-r:hover {\n  border-top-left-radius: 0.375rem;\n  \
         border-top-right-radius: 0.375rem;\n  border-bottom-right-radius: 0.375rem;\n  \
         border-bottom-left-radius: 0.375rem;\n}\n",
        "the alias renders the SAME probe suffix its condition spells"
    );
}

#[test]
fn a_fixed_rgba_renders_the_rgb_form() {
    let customs = Customs::new().utility(CustomUtility::fixed(
        "brand-ink",
        Val::Rgba([18, 52, 86, 255]),
        &[Prop::Color],
    ));
    let sheet = emit_css_with("brand-ink", &customs, &aeolus_compile::Config::default())
        .expect("non-empty input renders");
    assert!(sheet.diagnostics.is_empty(), "{sheet:?}");
    assert!(
        sheet.to_string().contains("color: rgb(18 52 86);"),
        "{sheet}"
    );
}

// ---------------------------------------------------------------------------
// 5. The empty-Customs differential
// ---------------------------------------------------------------------------

#[test]
fn empty_customs_are_byte_identical_to_the_theme_less_siblings() {
    let empty = Customs::new();
    let battery = [
        "p-4",
        "p-red",
        "not-a-utility",
        "w-[]",
        "has-[x]:p-4",
        "!p-4",
        "bg-primary/90",
        "-p-4",
        "hover:focus:md:p-0",
        "p--4 p-2",
        "w-1/2 h-full",
        "dark:hidden",
        "truncate",
        "text-lg",
    ];
    for input in battery {
        let plain =
            compile(input, &aeolus_compile::Config::default()).expect("non-empty input compiles");
        let seamed = compile_with(input, &empty, &aeolus_compile::Config::default())
            .expect("non-empty input compiles");
        assert_eq!(
            plain, seamed,
            "the seam is additive: empty registrations change nothing on {input:?}"
        );
    }
}

#[test]
fn the_two_walks_report_identically_with_customs() {
    let customs = demo_customs();
    let battery = [
        "brand-p-4",
        "brand-p-wide",
        "-brand-r",
        "hocus:brand-p-4",
        "hoverish:brand-p-4",
        "brand-flex",
        "brand-p-4 not-a-utility",
        "hocus:md:brand-p-4",
    ];
    for input in battery {
        let compiled = compile_with(input, &customs, &aeolus_compile::Config::default())
            .expect("non-empty input compiles");
        let sheet = emit_css_with(input, &customs, &aeolus_compile::Config::default())
            .expect("non-empty input renders");
        assert_eq!(
            compiled.diagnostics, sheet.diagnostics,
            "diagnostic drift between the IR form and the CSS form on {input:?}"
        );
    }
}

#[test]
fn the_two_walks_report_identically_with_token_customs() {
    let customs = ink_customs();
    let tokens = runtime_table();
    let battery = [
        "brand-ink-primary",
        "brand-ink-primary/90",
        "brand-ink-not-a-token",
        "brand-ink",
        "brand-ink-primary hover:brand-ink-primary",
    ];
    for input in battery {
        let compiled =
            compile_theme_with(input, &tokens, &customs, &aeolus_compile::Config::default())
                .expect("non-empty input compiles");
        let sheet = aeolus_compile::emit_css_theme_with(
            input,
            &tokens,
            &customs,
            &aeolus_compile::Config::default(),
        )
        .expect("non-empty input renders");
        assert_eq!(
            compiled.diagnostics, sheet.diagnostics,
            "diagnostic drift between the IR form and the CSS form on {input:?}"
        );
    }
}

#[test]
fn themed_customs_lower_tokens_through_the_theme_pass() {
    // The theme half rides the seam: a custom utility's own fixed
    // values are independent of the table, and a colour-token CLASS
    // (`bg-primary`) still lowers through the theme pass exactly like
    // `compile_theme` — the registrations never fence the theme off.
    let customs = demo_customs();
    let tokens = runtime_table();
    let compiled = compile_theme_with(
        "bg-primary brand-p-2",
        &tokens,
        &customs,
        &aeolus_compile::Config::default(),
    )
    .expect("compiles");
    assert!(
        compiled.diagnostics.is_empty(),
        "both halves lowered: {compiled:?}"
    );
    let decls = base(&compiled.program);
    assert!(
        decls.iter().any(|decl| decl.prop == Prop::Background),
        "the token lowered through the theme pass"
    );
    assert!(
        decls.contains(&Decl {
            prop: Prop::PaddingTop,
            val: Val::Rem(0.5)
        }),
        "the custom spacing utility lowered alongside"
    );
}

// ---------------------------------------------------------------------------
// 6. The theme-token functional form (`--value(color)`)
// ---------------------------------------------------------------------------

/// A colour custom utility over `Color`: `@utility brand-ink-* {
/// color: --value(color); }` — the suffix IS the token name.
fn ink_customs() -> Customs {
    Customs::new().utility(CustomUtility::functional(
        "brand-ink",
        CustomForm::Token,
        &[Prop::Color],
    ))
}

#[test]
fn the_token_form_lowers_through_the_runtime_tier() {
    let customs = ink_customs();
    let tokens = runtime_table();
    let primary = tokens.id("primary").expect("the token exists");
    let compiled = compile_theme_with(
        "brand-ink-primary",
        &tokens,
        &customs,
        &aeolus_compile::Config::default(),
    )
    .expect("compiles");
    assert!(compiled.diagnostics.is_empty(), "{compiled:?}");
    assert_eq!(
        base(&compiled.program),
        [Decl {
            prop: Prop::Color,
            val: Val::Tok {
                id: primary,
                alpha: 1.0
            },
        }]
        .as_slice(),
        "the T2 slot carries id + unit alpha, the built-in colour path's own shape"
    );
}

#[test]
fn the_token_form_bakes_through_the_premultiplied_identity() {
    // The differential pin, the theme suite's own shape: the seam's
    // baked literal IS the table's resolve (the audited oracle
    // underneath).
    let customs = ink_customs();
    let source = aeolus_theme::ThemeSource::shadcn_dark().expect("embedded palettes parse");
    let tokens = aeolus_theme::TokenTable::baked(&source).expect("default-seeded source builds");
    let red = tokens.id("red-500").expect("red-500 exists");
    let compiled = compile_theme_with(
        "brand-ink-red-500/50",
        &tokens,
        &customs,
        &aeolus_compile::Config::default(),
    )
    .expect("compiles");
    assert!(compiled.diagnostics.is_empty(), "{compiled:?}");
    let Val::Rgba(expected) = tokens.resolve(red, Some(0.5)).expect("bakes") else {
        panic!("the baked tier resolves a literal");
    };
    assert_eq!(
        base(&compiled.program),
        [Decl {
            prop: Prop::Color,
            val: Val::Rgba(expected)
        }]
        .as_slice(),
        "the /50 modifier rides the custom token form (the plan §2 bake)"
    );
}

#[test]
fn the_token_form_stays_a_typed_boundary_without_the_theme_pass() {
    let customs = ink_customs();
    let compiled = compile_with(
        "brand-ink-primary",
        &customs,
        &aeolus_compile::Config::default(),
    )
    .expect("compiles");
    assert_eq!(compiled.diagnostics.len(), 1, "{compiled:?}");
    assert!(matches!(
        &compiled.diagnostics[0].kind,
        DiagnosticKind::UnsupportedValue { reason, .. } if reason.contains("theme pass")
    ));
    assert!(compiled.program.layers.is_empty());
}

#[test]
fn an_unknown_token_name_is_the_typed_not_a_token_miss() {
    let customs = ink_customs();
    let tokens = runtime_table();
    let compiled = compile_theme_with(
        "brand-ink-not-a-token",
        &tokens,
        &customs,
        &aeolus_compile::Config::default(),
    )
    .expect("compiles");
    assert_eq!(compiled.diagnostics.len(), 1, "{compiled:?}");
    assert!(matches!(
        &compiled.diagnostics[0].kind,
        DiagnosticKind::UnsupportedValue { reason, .. } if reason.contains("TokenTable")
    ));
}

#[test]
fn an_alpha_on_a_baked_pair_stays_the_modifier_boundary() {
    let customs = ink_customs();
    let source = aeolus_theme::ThemeSource::shadcn_dark().expect("embedded palettes parse");
    let tokens = aeolus_theme::TokenTable::baked(&source).expect("default-seeded source builds");
    let compiled = compile_theme_with(
        "brand-ink-primary/50",
        &tokens,
        &customs,
        &aeolus_compile::Config::default(),
    )
    .expect("compiles");
    assert_eq!(compiled.diagnostics.len(), 1, "{compiled:?}");
    assert_eq!(
        compiled.diagnostics[0].kind,
        DiagnosticKind::UnsupportedModifier {
            modifier: "50".to_string()
        },
        "the pair form has no alpha arm — the same typed boundary the built-ins pin"
    );
}

#[test]
fn the_token_form_renders_the_var_and_the_mix() {
    let customs = ink_customs();
    let tokens = runtime_table();
    let sheet = aeolus_compile::emit_css_theme_with(
        "brand-ink-primary brand-ink-primary/90",
        &tokens,
        &customs,
        &aeolus_compile::Config::default(),
    )
    .expect("non-empty input renders");
    assert!(sheet.diagnostics.is_empty(), "{sheet:?}");
    let text = sheet.to_string();
    assert!(text.contains("color: var(--color-primary);"), "{text}");
    assert!(
        text.contains("color: color-mix(in oklab, var(--color-primary) 90%, transparent);"),
        "{text}"
    );
}

// ---------------------------------------------------------------------------
// 7. The merge
// ---------------------------------------------------------------------------

#[test]
fn cn_merges_custom_lowered_programs_last_wins() {
    let customs = demo_customs();
    let config = aeolus_compile::Config::default();
    let first = compile_with("brand-p-4 brand-r", &customs, &config).expect("compiles");
    let second = compile_with("brand-p-2", &customs, &config).expect("compiles");
    let merged = aeolus_compile::cn(&[&first, &second]).expect("non-empty inputs merge");
    assert!(merged.diagnostics.is_empty(), "{merged:?}");
    assert!(
        merged.program.base_conflicts().is_empty(),
        "the merge dedupes the overlap"
    );
    let decls = base(&merged.program);
    let top = decls
        .iter()
        .find(|decl| decl.prop == Prop::PaddingTop)
        .expect("padding set");
    assert_eq!(
        *top,
        Decl {
            prop: Prop::PaddingTop,
            val: Val::Rem(0.5)
        },
        "the rightmost custom utility wins per longhand"
    );
    assert!(
        decls
            .iter()
            .any(|decl| decl.prop == Prop::BorderTopLeftRadius),
        "the untouched longhand survives the merge"
    );
}
