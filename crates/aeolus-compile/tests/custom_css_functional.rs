//! The custom-authoring CSS-file form's FUNCTIONAL half — the
//! architecture plan §5.4 "same file as web" line's `@utility` star
//! spelling (`@utility brand-w-* { width: --value(percentage); }`): the
//! CSS-file functional bodies register through the SAME
//! [`CustomForm`](aeolus_compile::CustomForm) grammar the typed API
//! form uses, and the suffix lowers through the SAME numeric, spacing
//! and theme-token paths the built-in value pass rides.
//!
//! Families:
//! 1. **The star form registers** — `name-*` headers carry functional
//!    bodies; every `--value(type)`/`--spacing(n)` declaration in the
//!    body maps onto the typed [`CustomForm`] and the CSS property
//!    names onto the IR longhand targets (the audited shorthands fan
//!    like the registry rows do); the differential pins the file form
//!    compiling EXACTLY as the equivalent API registration.
//! 2. **The value forms** — integer/number/percentage/spacing/token
//!    lower through their built-in shapes (the `px` bake rides the
//!    spacing config, the token form resolves through the tiered
//!    tables, the `/opacity` modifier rides the token path).
//! 3. **The typed refusal set** — mixed forms in one body, literal
//!    CSS values, unknown property names and unknown value forms
//!    refuse with the reason named; an empty functional body is the
//!    one silent failure the pipeline does not have.
//!
//! Written FIRST (the RED spec): the suite does not compile against a
//! tree without the functional file form (the star spelling refused in
//! the M1 slice), which is this slice's RED state by construction.

use aeolus_compile::{
    Config, CustomCssError, CustomForm, CustomUtility, CustomValue, Customs, Spacing, compile,
    compile_theme_with, compile_with,
};
use aeolus_ir::{Cond, Decl, Prop, Val};
use aeolus_theme::{ThemeSource, TokenTable};

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

/// `customs_from_css` with the error unwrapped (the happy-path helper).
fn customs_from_css_ok(css: &str, config: Config) -> Customs {
    aeolus_compile::customs_from_css(css, &config).expect("parses")
}

// ---------------------------------------------------------------------------
// 1. The star form registers through the typed grammar
// ---------------------------------------------------------------------------

#[test]
fn star_form_registers_the_typed_form() {
    let config = Config::default();
    let customs = customs_from_css_ok(
        "@utility brand-w-* { width: --value(percentage); }\n",
        config,
    );
    let utilities = customs.utilities();
    assert_eq!(utilities.len(), 1);
    assert_eq!(utilities[0].root, "brand-w");
    assert_eq!(
        utilities[0].value,
        CustomValue::Functional(CustomForm::Percentage)
    );
    assert_eq!(utilities[0].targets, &[Prop::Width]);
}

#[test]
fn file_form_compiles_as_the_api_registration() {
    let config = Config::default();
    // The differential: the file form IS the API registration — the
    // compiled suffix grammar, the fan-out, the diagnostics.
    let from_file = customs_from_css_ok(
        "@utility brand-w-* { width: --value(percentage); }\n",
        config,
    );
    let from_api = Customs::new().utility(CustomUtility::functional(
        "brand-w",
        CustomForm::Percentage,
        &[Prop::Width],
    ));
    let compiled_file = compile_with("brand-w-40%", &from_file, &config).expect("compiles");
    let compiled_api = compile_with("brand-w-40%", &from_api, &config).expect("compiles");
    assert!(compiled_file.diagnostics.is_empty());
    assert!(compiled_api.diagnostics.is_empty());
    assert_eq!(base(&compiled_file.program), base(&compiled_api.program));
    assert!(base(&compiled_file.program).contains(&Decl {
        prop: Prop::Width,
        val: Val::Pct(40.0),
    }));
}

#[test]
fn integer_and_number_forms_lower_their_shapes() {
    let config = Config::default();
    let customs = customs_from_css_ok(
        "@utility brand-cols-* { grid-template-columns: --value(integer); }\n\
         @utility brand-z-* { order: --value(number); }\n",
        config,
    );
    let compiled = compile_with("brand-cols-3 brand-z-2.5", &customs, &config).expect("compiles");
    let decls = base(&compiled.program);
    assert!(decls.contains(&Decl {
        prop: Prop::GridTemplateColumns,
        val: Val::Num(3.0),
    }));
    assert!(decls.contains(&Decl {
        prop: Prop::Order,
        val: Val::Num(2.5),
    }));
    // The fractional suffix refuses the integer shape (the built-in
    // grammar's own two-stage shape): the miss is the typed shape
    // rejection naming the form's label.
    let refused = compile_with("brand-cols-2.5", &customs, &config).expect("compiles");
    assert!(
        refused
            .diagnostics
            .iter()
            .any(|d| { matches!(d.kind, aeolus_compile::DiagnosticKind::ShapeRejected { .. }) })
    );
}

// ---------------------------------------------------------------------------
// 2. The audited shorthands fan like the registry rows
// ---------------------------------------------------------------------------

#[test]
fn padding_shorthand_fans_the_four_longhands() {
    let config = Config::default();
    let customs = customs_from_css_ok("@utility brand-pad-* { padding: --spacing(n); }\n", config);
    let utilities = customs.utilities();
    assert_eq!(utilities[0].targets.len(), 4);
    let compiled = compile_with("brand-pad-4", &customs, &config).expect("compiles");
    let direct = compile("p-4", &config).expect("compiles");
    // The differential: the spacing scale's own values (1 unit =
    // 0.25rem), byte-shared with the registry value pass.
    assert_eq!(base(&compiled.program), base(&direct.program));
}

#[test]
fn radius_shorthand_fans_the_corners() {
    let config = Config::default();
    let customs = customs_from_css_ok(
        "@utility brand-rad-* { border-radius: --spacing(n); }\n",
        config,
    );
    let compiled = compile_with("brand-rad-2", &customs, &config).expect("compiles");
    let props: Vec<Prop> = base(&compiled.program).iter().map(|d| d.prop).collect();
    assert_eq!(props.len(), 4);
    for prop in [
        Prop::BorderTopLeftRadius,
        Prop::BorderTopRightRadius,
        Prop::BorderBottomRightRadius,
        Prop::BorderBottomLeftRadius,
    ] {
        assert!(props.contains(&prop), "{prop:?} fans");
    }
}

#[test]
fn the_px_bake_rides_the_spacing_config() {
    let config = Config {
        spacing: Spacing::Px,
        ..Config::default()
    };
    let customs = customs_from_css_ok("@utility brand-pad-* { padding: --spacing(n); }\n", config);
    let compiled = compile_with("brand-pad-4", &customs, &config).expect("compiles");
    let direct = compile("p-4", &config).expect("compiles");
    assert_eq!(base(&compiled.program), base(&direct.program));
    assert!(base(&compiled.program).contains(&Decl {
        prop: Prop::PaddingTop,
        val: Val::Px(16.0),
    }));
}

// ---------------------------------------------------------------------------
// 3. The token form resolves through the tiered tables
// ---------------------------------------------------------------------------

#[test]
fn token_form_resolves_the_tiered_table() {
    let config = Config::default();
    let customs = customs_from_css_ok("@utility brand-ink-* { color: --value(color); }\n", config);
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    let tokens = TokenTable::baked(&source).expect("the T0/T1 bake succeeds");
    let empty = Customs::new();
    // The differential: the token form resolves through the SAME tier
    // path the built-in colour row rides — the custom's baked value
    // EQUALS the token row's own (T0 literal here; the longhand is the
    // body's own `color:`, the row's is `background`).
    let compiled =
        compile_theme_with("brand-ink-primary", &tokens, &customs, &config).expect("compiles");
    let direct = compile_theme_with("bg-primary", &tokens, &empty, &config).expect("compiles");
    assert!(compiled.diagnostics.is_empty());
    let Val::Rgba(baked) = base(&compiled.program)[0].val else {
        panic!("the T0 tier bakes a literal");
    };
    assert_eq!(base(&direct.program)[0].val, Val::Rgba(baked));
    // The `/opacity` modifier RIDES the token path (the plan §2
    // premultiplied identity — the theme pass's own arithmetic): the
    // custom's faded value EQUALS the row's faded value.
    let faded =
        compile_theme_with("brand-ink-primary/50", &tokens, &customs, &config).expect("compiles");
    let direct_faded =
        compile_theme_with("bg-primary/50", &tokens, &empty, &config).expect("compiles");
    assert_eq!(
        base(&faded.program)[0].val,
        base(&direct_faded.program)[0].val
    );
}

// ---------------------------------------------------------------------------
// 4. The typed refusal set
// ---------------------------------------------------------------------------

#[test]
fn mixed_forms_refuse() {
    let err = aeolus_compile::customs_from_css(
        "@utility brand-x-* { width: --value(percentage); color: --value(color); }\n",
        &Config::default(),
    )
    .expect_err("one suffix fills ONE form — the multi-form body has no slot");
    assert!(matches!(err, CustomCssError::Functional { .. }));
    assert!(err.to_string().contains("brand-x"));
}

#[test]
fn literal_values_refuse() {
    let err = aeolus_compile::customs_from_css(
        "@utility brand-x-* { width: 2rem; }\n",
        &Config::default(),
    )
    .expect_err("a literal CSS value has no functional slot to fill");
    assert!(matches!(err, CustomCssError::Functional { .. }));
}

#[test]
fn unknown_property_and_form_refuse() {
    let config = Config::default();
    // The CSS property name outside the IR's longhand set.
    let err = aeolus_compile::customs_from_css(
        "@utility brand-x-* { text-shadow: --value(number); }\n",
        &config,
    )
    .expect_err("text-shadow has no IR longhand");
    assert!(matches!(err, CustomCssError::Functional { .. }));
    // The value form outside the typed grammar.
    let err = aeolus_compile::customs_from_css(
        "@utility brand-x-* { width: --value(length); }\n",
        &config,
    )
    .expect_err("the length hint is the arbitrary-value pass's surface, not a form");
    assert!(matches!(err, CustomCssError::Functional { .. }));
    // An empty functional body lowers nothing.
    let err = aeolus_compile::customs_from_css(
        "@utility brand-x-* { width: --value(integer); height:; }\n",
        &config,
    )
    .expect_err("the empty value is the one silent failure");
    assert!(matches!(err, CustomCssError::Functional { .. }));
    // A star-less header carrying a functional body: to the applied
    // reader the body is a raw declaration list (no `@apply`), the
    // M1 form's own refusal.
    let err = aeolus_compile::customs_from_css(
        "@utility brand-x { width: --value(percentage); }\n",
        &config,
    )
    .expect_err("a fixed header cannot carry a functional body");
    assert!(matches!(err, CustomCssError::Body { .. }));
}

#[test]
fn the_applied_and_functional_forms_coexist() {
    let config = Config::default();
    let customs = customs_from_css_ok(
        "@utility brand-badge { @apply rounded-md px-2; }\n\
         @utility brand-pad-* { padding: --spacing(n); }\n",
        config,
    );
    let applied = compile_with("brand-badge", &customs, &config).expect("compiles");
    let functional = compile_with("brand-pad-2", &customs, &config).expect("compiles");
    assert!(
        base(&applied.program)
            .iter()
            .any(|d| d.prop == Prop::PaddingLeft)
    );
    assert!(base(&functional.program).contains(&Decl {
        prop: Prop::PaddingLeft,
        val: Val::Rem(0.5),
    }));
}
