//! The shadcn PROGRAM TABLES (architecture plan §3.6's surface (2), the
//! "closed-world components — the shadcn win" row: "`class-variance-
//! authority`/`tailwind-variants` as a _compile-time_ table ...
//! `aeolus-shadcn` collapses into a set of `variants!` declarations —
//! F6 disappears", and §10's "shadcn components are `const` tables"):
//! the verbatim v4 class strings compile through the V1 pipeline ONCE
//! per closed-world combo and intern — `Button::program(variant, size)`
//! is a table hit, never a `String` build plus a re-resolve per call
//! (the F6 finding).
//!
//! The pins:
//! * every closed-world combo lowers — the same verbatim strings the
//!   string surface ships, through the V1 pipeline with the built-in
//!   shadcn theme baked;
//! * a repeat call hands back the SAME interned program — the table
//!   hit costs no compile (the pointer proves it);
//! * the table's program IS the canonical string's program — the
//!   differential pin walks the table's layers beside a fresh compile
//!   of [`crate::classes::button_classes`];
//! * the effective declarations resolve the theme tokens through the
//!   table (the `bg-primary` slot reads the baked colour back);
//! * the first compile's diagnostics ride the table — every refusal is
//!   one of the DOCUMENTED arbitrary-variant/attr-boundary families the
//!   capability notes list, never a dropped utility in silence;
//! * the base-only components (input, label, card family, separator,
//!   skeleton) intern the same way.

use aeolus_compile::{Config, Env, compile_theme, interp_theme};
use aeolus_rt::{ThemeSource, TokenTable};
use aeolus_shadcn::classes::{AlertVariant, BadgeVariant, ButtonSize, ButtonVariant};
use aeolus_shadcn::program::{AlertPrograms, BadgePrograms, BasePrograms, ButtonPrograms};

fn baked_tokens() -> TokenTable {
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    TokenTable::baked(&source).expect("the T0/T1 bake succeeds")
}

#[test]
fn every_button_combo_compiles_through_the_v1_pipeline() {
    let tokens = baked_tokens();
    let tables = ButtonPrograms::new(&tokens);
    for variant in [
        ButtonVariant::Default,
        ButtonVariant::Destructive,
        ButtonVariant::Outline,
        ButtonVariant::Secondary,
        ButtonVariant::Ghost,
        ButtonVariant::Link,
    ] {
        for size in [
            ButtonSize::Default,
            ButtonSize::Xs,
            ButtonSize::Sm,
            ButtonSize::Lg,
            ButtonSize::Icon,
            ButtonSize::IconXs,
            ButtonSize::IconSm,
            ButtonSize::IconLg,
        ] {
            let program = tables.program(variant, size);
            assert!(
                !program.layers.is_empty(),
                "{variant:?} {size:?} lowers to layers"
            );
        }
    }
}

#[test]
fn a_repeat_call_is_the_same_interned_program() {
    let tokens = baked_tokens();
    let tables = ButtonPrograms::new(&tokens);
    let first = tables.program(ButtonVariant::Default, ButtonSize::Default);
    let second = tables.program(ButtonVariant::Default, ButtonSize::Default);
    assert_eq!(
        first.program().layers.as_ptr(),
        second.program().layers.as_ptr(),
        "the table hit is the interned program, not a second compile"
    );
}

#[test]
fn the_tables_program_is_the_canonical_strings_program() {
    // The differential pin: the table compiles exactly the string the
    // string surface ships (`button_classes`), so a fresh compile of
    // that string gives the SAME layers.
    let tokens = baked_tokens();
    let tables = ButtonPrograms::new(&tokens);
    for variant in [
        ButtonVariant::Default,
        ButtonVariant::Destructive,
        ButtonVariant::Outline,
        ButtonVariant::Secondary,
        ButtonVariant::Ghost,
        ButtonVariant::Link,
    ] {
        let program = tables.program(variant, ButtonSize::Default);
        let canonical = aeolus_shadcn::classes::button_classes(variant, ButtonSize::Default);
        let fresh = compile_theme(&canonical, &tokens, &Config::default())
            .expect("the canonical string compiles");
        assert_eq!(
            program.program().layers,
            fresh.program.layers,
            "{variant:?}: the table's layers ARE the canonical string's layers"
        );
    }
}

#[test]
fn the_effective_declarations_resolve_the_theme_tokens() {
    // The `bg-primary` slot rides the table's `Val::Tok` id into the
    // baked colour — the T0 tier's one lookup, the interp's own answer.
    let tokens = baked_tokens();
    let tables = ButtonPrograms::new(&tokens);
    let program = tables.program(ButtonVariant::Default, ButtonSize::Default);
    let effective =
        interp_theme(&program.program(), &tokens, &Env::default()).expect("the interp resolves");
    let background = effective
        .decls
        .iter()
        .find(|decl| decl.prop == aeolus_ir::Prop::Background)
        .expect("the default button paints its background");
    assert!(
        matches!(background.val, aeolus_ir::Val::Rgba(_)),
        "the token slot resolved to the baked colour: {:?}",
        background.val
    );
}

#[test]
fn the_single_axis_tables_intern_the_same_way() {
    let tokens = baked_tokens();
    let badges = BadgePrograms::new(&tokens);
    let alerts = AlertPrograms::new(&tokens);

    for variant in [
        BadgeVariant::Default,
        BadgeVariant::Secondary,
        BadgeVariant::Destructive,
        BadgeVariant::Outline,
        BadgeVariant::Ghost,
        BadgeVariant::Link,
    ] {
        let first = badges.program(variant);
        let second = badges.program(variant);
        assert_eq!(
            first.program().layers.as_ptr(),
            second.program().layers.as_ptr()
        );
        assert!(!first.program().layers.is_empty(), "{variant:?} lowers");
    }
    for variant in [AlertVariant::Default, AlertVariant::Destructive] {
        let first = alerts.program(variant);
        let second = alerts.program(variant);
        assert_eq!(
            first.program().layers.as_ptr(),
            second.program().layers.as_ptr()
        );
        assert!(!first.program().layers.is_empty(), "{variant:?} lowers");
    }
}

#[test]
fn the_base_components_intern_the_same_way() {
    let tokens = baked_tokens();
    let base = BasePrograms::new(&tokens);

    let first = base.input();
    let second = base.input();
    assert_eq!(
        first.program().layers.as_ptr(),
        second.program().layers.as_ptr()
    );
    assert!(!first.program().layers.is_empty(), "input lowers");

    let first = base.label();
    let second = base.label();
    assert_eq!(
        first.program().layers.as_ptr(),
        second.program().layers.as_ptr()
    );
    assert!(!first.program().layers.is_empty(), "label lowers");

    for (name, program) in [
        ("card", base.card()),
        ("card_header", base.card_header()),
        ("card_title", base.card_title()),
        ("card_description", base.card_description()),
        ("card_content", base.card_content()),
        ("card_footer", base.card_footer()),
        ("separator", base.separator()),
        ("separator_horizontal", base.separator_horizontal()),
        ("separator_vertical", base.separator_vertical()),
        ("skeleton", base.skeleton()),
        ("alert_title", base.alert_title()),
        ("alert_description", base.alert_description()),
    ] {
        assert!(!program.program().layers.is_empty(), "{name} lowers");
    }
}

#[test]
fn every_refusal_is_the_documented_arbitrary_variant_family() {
    // The capability notes' own table, observed: the ONLY refusals any
    // combo produces are the arbitrary/has variant spellings the v4
    // registry strings carry (`[&_svg]`, `[&_svg:not([class*='size-'])]`,
    // `has-[>svg]`) — the typed UnsupportedVariant boundary, never an
    // unknown ROOT (the registry knows every class name the tables
    // ship), never a silent drop.
    let tokens = baked_tokens();
    let tables = ButtonPrograms::new(&tokens);
    let variants = [
        ButtonVariant::Default,
        ButtonVariant::Destructive,
        ButtonVariant::Outline,
        ButtonVariant::Secondary,
        ButtonVariant::Ghost,
        ButtonVariant::Link,
    ];
    let sizes = [
        ButtonSize::Default,
        ButtonSize::Xs,
        ButtonSize::Sm,
        ButtonSize::Lg,
        ButtonSize::Icon,
        ButtonSize::IconXs,
        ButtonSize::IconSm,
        ButtonSize::IconLg,
    ];
    for v in variants {
        for s in sizes {
            for diagnostic in tables.diagnostics(v, s) {
                let aeolus_compile::DiagnosticKind::UnsupportedVariant { variant } =
                    &diagnostic.kind
                else {
                    panic!(
                        "{v:?} {s:?}: unexpected refusal kind: {:?}",
                        diagnostic.kind
                    )
                };
                assert!(
                    variant.starts_with('[') || variant.starts_with("has-["),
                    "{v:?} {s:?}: the refusal is the documented family: {variant}"
                );
            }
        }
    }
}
