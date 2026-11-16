//! The element-state family's interp surface (`target:`/`indeterminate:`/
//! `autofill:`/`read-only:` — the v0.x-gap closure, tailwindcss v4.1.17's
//! own static variants, pinned verbatim from the source:
//! `packages/tailwindcss/src/variants.ts`'s state/forms blocks
//! `staticVariant('target', ['&:target'])`,
//! `staticVariant('indeterminate', ['&:indeterminate'])`,
//! `staticVariant('autofill', ['&:autofill'])`,
//! `staticVariant('read-only', ['&:read-only'])`). The seam:
//!
//! * [`Cond::Target`]/[`Cond::Indeterminate`]/[`Cond::Autofill`]/[`Cond::ReadOnly`]
//!   — the four typed conditions the four spellings fold to, one bare
//!   variant per family spelling (the `valid:`/`checked:` shape);
//! * the [`Env`] facts — the element-own answers are ELEMENT STATE the
//!   host answers (the tier the `hover:`/`checked:` bits ride): ONE fact
//!   per question (`url_target`, `indeterminate`, `autofilled`), and
//!   `read-only:` rides the editability fact's COMPLEMENT (`read_write`
//!   — the pair's two CSS values share one fact, the
//!   `valid:`/`invalid:` precedent: CSS's own reading for a
//!   non-editable element IS `:read-only`, so the resting env answers
//!   the `read-only:` query the way it answers `ltr:`), an unset fact
//!   the QUIET miss a selector that does not match is, never an
//!   exclusion note;
//! * the CSS head — the selector-side pseudo-class (`:target`,
//!   `:read-only`; the family is NOT an at-rule: no `@media` head
//!   renders), the upstream `&:` texts verbatim;
//! * the plan bridge — the v0.x `State` surface ALREADY carries the
//!   four states (`aeolus_core::ast::State`'s target/enabled/forms
//!   blocks), so every layer LANDS its `Test::State(...)` rule slot —
//!   no note rides (the `checked:` precedent, not the direction
//!   boundary);
//! * the `has-` inner grammar answers the four spellings beside
//!   `checked:` (one grammar, three consumers — the fold, the CSS
//!   arm's parent-relative heads, the test pin).
//!
//! Written FIRST (the RED spec): the suite fails against the
//! post-validation-family tree — [`Cond::Target`] and friends do not
//! exist yet (the fold refuses every element-state spelling as
//! unsupported-variant).

use aeolus_compile::{Config, Env, compile, emit_css, emit_plan, interp};
use aeolus_ir::{Cond, Prop, Val};

/// The value one longhand resolved to.
fn resolved(interp: &aeolus_compile::Interped, prop: Prop) -> Val {
    interp
        .decls
        .iter()
        .find(|d| d.prop == prop)
        .unwrap_or_else(|| panic!("{prop:?} is in the resolved declarations"))
        .val
}

// ---------------------------------------------------------------------------
// The fold: one arm per family spelling
// ---------------------------------------------------------------------------

#[test]
fn every_element_state_spelling_folds_onto_the_typed_cond() {
    let cases: &[(&str, Cond)] = &[
        ("target", Cond::Target),
        ("indeterminate", Cond::Indeterminate),
        ("autofill", Cond::Autofill),
        ("read-only", Cond::ReadOnly),
    ];
    for (spelling, want) in cases {
        let compiled = compile(&format!("{spelling}:p-8"), &Config::default()).expect("compiles");
        let cond = compiled
            .program
            .layers
            .iter()
            .find_map(|layer| match layer.cond {
                cond @ (Cond::Target | Cond::Indeterminate | Cond::Autofill | Cond::ReadOnly) => {
                    Some(cond)
                }
                _ => None,
            })
            .unwrap_or_else(|| panic!("{spelling} folds onto the typed element state"));
        assert_eq!(cond, *want, "{spelling} folds to the right state");
    }
}

#[test]
fn the_element_states_ride_the_probe_tier() {
    // The element-state test IS a selector conjunct (`:target`, …), not
    // an at-rule: the specificity contract pins it on the element-own
    // probe tier (beside `hover:` and `checked:`), below the media tier
    // the breakpoint and capability forms ride.
    for cond in [
        Cond::Target,
        Cond::Indeterminate,
        Cond::Autofill,
        Cond::ReadOnly,
    ] {
        assert_eq!(cond.specificity(), Cond::Hover.specificity());
        assert!(cond.specificity() < Cond::Media(aeolus_ir::Pref::Print).specificity());
    }
}

#[test]
fn the_element_state_conditions_name_their_own_spelling() {
    // The canonical variant text rides the condition (the diagnostics
    // and the metamorphic labels read it; no payload table needed —
    // the spellings are closed).
    assert_eq!(Cond::Target.source(), Some("target"));
    assert_eq!(Cond::Indeterminate.source(), Some("indeterminate"));
    assert_eq!(Cond::Autofill.source(), Some("autofill"));
    assert_eq!(Cond::ReadOnly.source(), Some("read-only"));
}

// ---------------------------------------------------------------------------
// The interp: the Env facts answer, the unset bit is a quiet miss
// ---------------------------------------------------------------------------

#[test]
fn the_target_fact_answers_the_layer() {
    let compiled = compile("target:p-8", &Config::default()).expect("compiles");
    let env = Env {
        url_target: true,
        ..Env::default()
    };
    let interped = interp(&compiled.program, &env).expect("no theme forms");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));
    assert!(interped.notes.is_empty(), "a fire is not a note");

    // The resting env asserts no fragment-target fact — the layer misses
    // QUIETLY (a `:target` selector that does not match, never a note).
    let interped = interp(&compiled.program, &Env::default()).expect("no theme forms");
    assert!(interped.decls.is_empty(), "{:?}", interped.decls);
    assert!(interped.notes.is_empty(), "a miss is not a note");
}

#[test]
fn the_indeterminate_fact_answers_the_layer() {
    let compiled = compile("indeterminate:p-8", &Config::default()).expect("compiles");
    let env = Env {
        indeterminate: true,
        ..Env::default()
    };
    let interped = interp(&compiled.program, &env).expect("no theme forms");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));
    assert!(interped.notes.is_empty(), "a fire is not a note");

    let interped = interp(&compiled.program, &Env::default()).expect("no theme forms");
    assert!(interped.decls.is_empty(), "{:?}", interped.decls);
    assert!(interped.notes.is_empty(), "a miss is not a note");
}

#[test]
fn the_autofill_fact_answers_the_layer() {
    let compiled = compile("autofill:p-8", &Config::default()).expect("compiles");
    let env = Env {
        autofilled: true,
        ..Env::default()
    };
    let interped = interp(&compiled.program, &env).expect("no theme forms");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));
    assert!(interped.notes.is_empty(), "a fire is not a note");

    let interped = interp(&compiled.program, &Env::default()).expect("no theme forms");
    assert!(interped.decls.is_empty(), "{:?}", interped.decls);
    assert!(interped.notes.is_empty(), "a miss is not a note");
}

#[test]
fn the_resting_env_answers_the_read_only_query() {
    // `read-only:` rides the editability fact's COMPLEMENT (the pair's
    // two CSS values share one fact, the `valid:`/`invalid:` precedent
    // — and CSS's own reading for a non-editable element IS
    // `:read-only`, so the resting env answers the read-only query the
    // way it answers `ltr:`).
    let compiled = compile("read-only:p-8", &Config::default()).expect("compiles");
    let interped = interp(&compiled.program, &Env::default()).expect("no theme forms");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));
    assert!(interped.notes.is_empty(), "a fire is not a note");

    // With the editability fact asserted (the element is user-editable),
    // the read-only layer misses — one fact, the CSS complement holds.
    let env = Env {
        read_write: true,
        ..Env::default()
    };
    let interped = interp(&compiled.program, &env).expect("no theme forms");
    assert!(
        interped.decls.is_empty(),
        "the user-editable element fails the read-only query"
    );
}

// ---------------------------------------------------------------------------
// The CSS arm: the upstream selector texts, pinned — no at-rule head
// ---------------------------------------------------------------------------

#[test]
fn the_css_arm_renders_the_upstream_pseudo_classes() {
    // The texts are variants.ts's own `staticVariant` selectors
    // (v4.1.17, the state/forms blocks), with the `&` the utility
    // selector replaces: `staticVariant('target', ['&:target'])` and
    // siblings.
    let cases: &[(&str, &str)] = &[
        ("target", ":target"),
        ("indeterminate", ":indeterminate"),
        ("autofill", ":autofill"),
        ("read-only", ":read-only"),
    ];
    for (spelling, head) in cases {
        let sheet = emit_css(&format!("{spelling}:p-8"), &Config::default()).expect("emits");
        assert!(sheet.diagnostics.is_empty(), "{:?}", sheet.diagnostics);
        let rendered = sheet.to_string();
        assert!(
            rendered.contains(head),
            "{spelling} renders the v4.1.17 pseudo-class: {rendered}"
        );
    }
}

#[test]
fn the_element_state_head_is_selector_side() {
    // The family is NOT an at-rule: no `@media`/`@container`/`@supports`
    // head renders (the element-state test rides the element's own
    // selector, the probe tier the hover/checked forms occupy).
    for spelling in ["target", "indeterminate", "autofill", "read-only"] {
        let sheet = emit_css(&format!("{spelling}:p-8"), &Config::default()).expect("emits");
        let rendered = sheet.to_string();
        assert!(
            !rendered.contains("@media"),
            "{spelling} renders no at-rule head: {rendered}"
        );
    }
}

// ---------------------------------------------------------------------------
// The plan bridge: the v0.x State surface's own slots
// ---------------------------------------------------------------------------

#[test]
fn the_plan_bridge_lands_the_state_slots() {
    // The v0.x `State` enum ALREADY carries the four states (the
    // target/enabled/forms blocks): every layer lands its
    // `Test::State(...)` rule — no note rides (the `checked:`
    // precedent, not the direction boundary).
    use aeolus_core::ast::State;
    use aeolus_core::{Atom, Test};
    let cases: &[(&str, State)] = &[
        ("target", State::Target),
        ("indeterminate", State::Indeterminate),
        ("autofill", State::Autofill),
        ("read-only", State::ReadOnly),
    ];
    for (spelling, state) in cases {
        let sheet = emit_plan(&format!("{spelling}:p-8"), &Config::default()).expect("emits");
        assert!(
            sheet.notes.is_empty(),
            "{spelling} lands, never reports: {:?}",
            sheet.notes
        );
        let landed = sheet
            .plan
            .rules
            .iter()
            .any(|rule| rule.when.0 == vec![Atom::On(Test::State(*state))]);
        assert!(landed, "{spelling} lands the {state:?} rule slot");
    }
}

// ---------------------------------------------------------------------------
// The `has-` inner grammar: one grammar, three consumers
// ---------------------------------------------------------------------------

#[test]
fn the_has_inner_grammar_answers_the_element_states() {
    // The `has-` inner reads the four spellings beside `checked:` (the
    // same state_pseudo table the CSS arm's heads render): the rendered
    // `:has(...)` fragment is the star-prefixed pseudo-class.
    for (spelling, fragment) in [
        ("target", "*:target"),
        ("indeterminate", "*:indeterminate"),
        ("autofill", "*:autofill"),
        ("read-only", "*:read-only"),
    ] {
        let compiled = compile(&format!("has-{spelling}:p-8"), &Config::default())
            .unwrap_or_else(|e| panic!("has-{spelling} folds: {e:?}"));
        let cond = compiled
            .program
            .layers
            .iter()
            .find_map(|layer| match layer.cond {
                cond @ Cond::Has(_) => Some(cond),
                _ => None,
            })
            .unwrap_or_else(|| panic!("has-{spelling} folds onto the has shape"));
        // The payload compares by content (the fold interns its own
        // `&'static str`; the literal names the same fragment text).
        assert_eq!(
            cond,
            Cond::Has(fragment),
            "has-{spelling} renders the star-prefixed fragment"
        );
    }
}

// ---------------------------------------------------------------------------
// The metamorphic triangle: compile == CSS == interp under the facts
// ---------------------------------------------------------------------------

#[test]
fn the_metamorphic_triangle_agrees_on_an_element_state_program() {
    // The conjunctive checker bakes the element-state facts into the
    // situation envs it walks (the element-state tier, the state
    // families' own rule): an element-state program agrees across the
    // three walks — compile == CSS == interp.
    let report = aeolus_compile::metamorphic_check(
        "target:p-8 read-only:p-4 indeterminate:p-2 autofill:hidden",
        &Config::default(),
    )
    .expect("compiles");
    let disagreeing: Vec<&aeolus_compile::SituationReport> = report
        .situations
        .iter()
        .filter(|s| !s.agree || s.interp_error.is_some())
        .collect();
    assert!(
        report.agree(),
        "the triangle agrees: {disagreeing:?} (notes {:?})",
        report.notes
    );
}
