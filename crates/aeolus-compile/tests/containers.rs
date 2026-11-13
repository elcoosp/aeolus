//! The named-container fold (plan §3.5's `@container` row, the v0.9
//! oracle's `@md/name` form): the container variant's NAME rides the
//! typed [`Cond`] surface instead of being dropped at the fold.
//!
//! The pins: the named and unnamed containers are DIFFERENT conditions
//! (the fold used to merge `@md/card:` into `@md:` — two different
//! containers collapsing into one cascade slot), the CSS arm renders
//! the named query (`@container card (min-width: …)`, the v4 form),
//! the plan bridge fills the v0.x oracle's `Atom::Container.name`, the
//! interp answers the unnamed question from the Env and reports the
//! named one (the Env carries the nearest container only), the empty
//! name refuses typed, and the static arm carries the payload through
//! the IR's own `Debug` contract.
//!
//! Written FIRST (the RED spec): the suite fails while
//! [`Cond::Container`] carries no name slot.

use aeolus_compile::{Config, Env, InterpNote, compile, emit_css, emit_plan, interp};
use aeolus_core::{Resolver, TAILWIND_DEFAULT_THEME, TailwindTheme};
use aeolus_ir::{Bp, Cond, Prop};

#[test]
fn the_named_and_unnamed_containers_are_different_conditions() {
    let named = compile("@md/card:flex", &Config::default()).expect("named compiles");
    let unnamed = compile("@md:flex", &Config::default()).expect("unnamed compiles");
    assert_eq!(
        named.program.layers[0].cond,
        Cond::Container {
            bp: Bp::Md,
            name: Some("card"),
        },
        "the named fold carries the container's name"
    );
    assert_eq!(
        unnamed.program.layers[0].cond,
        Cond::Container {
            bp: Bp::Md,
            name: None,
        },
        "the unnamed fold stays unnamed"
    );
    // The parity point: two different containers are two different
    // conditions — never one shared cascade slot.
    assert_ne!(named.program.layers[0].cond, unnamed.program.layers[0].cond);
}

#[test]
fn the_max_form_carries_the_name_too() {
    let compiled = compile("@max-lg/sidebar:hidden", &Config::default()).expect("compiles");
    assert_eq!(
        compiled.program.layers[0].cond,
        Cond::MaxContainer {
            bp: Bp::Lg,
            name: Some("sidebar"),
        },
    );
}

#[test]
fn an_empty_container_name_refuses_typed() {
    let compiled = compile("@md/:flex", &Config::default()).expect("compiles");
    assert!(
        !compiled.diagnostics.is_empty(),
        "the empty name is an unknown variant, never a silent unnamed fold"
    );
    assert!(
        compiled.program.layers.is_empty(),
        "the refused class lowers nothing"
    );
}

#[test]
fn the_css_arm_renders_the_named_query() {
    let sheet = emit_css("@md/card:w-64 @md:w-full", &Config::default()).expect("css emits");
    let text = sheet.to_string();
    assert!(
        text.contains("@container card (min-width: 48rem)"),
        "the named query renders the v4 container form: {text}"
    );
    assert!(
        text.contains("@container (min-width: 48rem)"),
        "the unnamed query keeps its form: {text}"
    );
}

#[test]
fn the_plan_bridge_fills_the_oracles_name_field() {
    // The differential oracle: the v0.x resolver fed the same class
    // folds the name into its `Atom::Container` — the V1 plan bridge
    // fills the same field.
    let sheet = emit_plan("@md/card:w-64", &Config::default()).expect("plan emits");
    let theme = TailwindTheme::from_theme_css(TAILWIND_DEFAULT_THEME).expect("theme");
    let oracle = Resolver::new(&theme)
        .resolve("@md/card:w-64")
        .expect("oracle resolves");
    let has_named_atom: fn(&aeolus_core::StylePlan) -> bool = |plan| {
        plan.rules.iter().any(|rule| {
            rule.when.0.iter().any(|atom| {
                matches!(
                    atom,
                    aeolus_core::cond::Atom::Container { name: Some(_), .. }
                )
            })
        })
    };
    assert!(
        has_named_atom(&oracle.plan),
        "the oracle folds the name (the differential source)"
    );
    assert!(
        has_named_atom(&sheet.plan),
        "the plan bridge fills the oracle's own name field"
    );
}

#[test]
fn the_interp_answers_the_unnamed_question_and_reports_the_named_one() {
    let compiled = compile("@md:flex @md/card:w-64", &Config::default()).expect("compiles");
    let env = Env {
        container_rem: Some(64.0),
        ..Env::default()
    };
    let interped = interp(&compiled.program, &env).expect("interps");
    // The unnamed layer fires (the Env carries the nearest container's
    // width); the NAMED layer reports — the Env cannot answer a named
    // container's width, and the note says which one went unqueried.
    assert!(
        interped.notes.iter().any(
            |note| matches!(note, InterpNote::ContainerUnqueried { cond, .. }
                if matches!(cond, Cond::Container { name: Some("card"), .. }))
        ),
        "the named layer reports with its name: {:?}",
        interped.notes
    );
    assert!(
        !interped.notes.iter().any(
            |note| matches!(note, InterpNote::ContainerUnqueried { cond, .. }
                if matches!(cond, Cond::Container { name: None, .. }))
        ),
        "the unnamed layer fired: {:?}",
        interped.notes
    );
    assert!(
        interped.decls.iter().any(|decl| decl.prop == Prop::Display),
        "the unnamed layer's declarations resolved"
    );
}

#[test]
fn the_static_form_carries_the_name_through_the_debug_contract() {
    let compiled = compile("@md/card:flex", &Config::default()).expect("compiles");
    let text = aeolus_compile::emit_static(&compiled.program);
    assert!(
        text.contains("name:"),
        "the emitted static renders the named payload through the IR's own Debug: {text}"
    );
}
