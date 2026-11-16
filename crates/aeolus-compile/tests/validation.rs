//! The validation states' interp surface (`valid:`/`invalid:`/
//! `user-valid:`/`user-invalid:` — the form-state family, tailwindcss
//! v4.1.17's own static variants, pinned verbatim from the source:
//! `packages/tailwindcss/src/variants.ts`'s forms block
//! `staticVariant('valid', ['&:valid'])`, the `invalid`/`user-valid`/
//! `user-invalid` siblings beside it). The seam:
//!
//! * [`Cond::Valid`]/[`Cond::Invalid`]/[`Cond::UserValid`]/[`Cond::UserInvalid`]
//!   — the four typed conditions the four spellings fold to, one bare
//!   variant per family spelling (the `hover:`/`checked:` shape);
//! * the [`Env`] facts — the constraint state is ELEMENT STATE the host
//!   answers (the tier the `hover:`/`checked:` bits ride): ONE fact per
//!   complement pair (`validation_invalid` answers `invalid:` on the
//!   bit, `valid:` its COMPLEMENT — CSS's own reading for a pristine
//!   field is `:valid`, so the resting env answers the `valid:` query
//!   the way it answers `ltr:`) plus the interaction gate the HTML
//!   spec's `:user-*` pseudo-classes require (`user_validated`; BOTH
//!   `user-` spellings stay quiet until the gate is set), an unset fact
//!   the QUIET miss a selector that does not match is, never an
//!   exclusion note;
//! * the CSS head — the selector-side pseudo-class (`:valid`,
//!   `:user-invalid`; the family is NOT an at-rule: no `@media` head
//!   renders), the upstream `&:` texts verbatim;
//! * the plan bridge — the v0.x `State` surface ALREADY carries the
//!   four states (`aeolus_core::ast::State`'s forms block), so every
//!   layer LANDS its `Test::State(...)` rule slot — no note rides (the
//!   `checked:` precedent, not the direction boundary).
//!
//! Written FIRST (the RED spec): the suite fails against the
//! post-direction-family tree — [`Cond::Valid`] and friends do not
//! exist yet (the fold refuses every validation spelling as
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
fn every_validation_spelling_folds_onto_the_typed_cond() {
    let cases: &[(&str, Cond)] = &[
        ("valid", Cond::Valid),
        ("invalid", Cond::Invalid),
        ("user-valid", Cond::UserValid),
        ("user-invalid", Cond::UserInvalid),
    ];
    for (spelling, want) in cases {
        let compiled = compile(&format!("{spelling}:p-8"), &Config::default()).expect("compiles");
        let cond = compiled
            .program
            .layers
            .iter()
            .find_map(|layer| match layer.cond {
                cond @ (Cond::Valid | Cond::Invalid | Cond::UserValid | Cond::UserInvalid) => {
                    Some(cond)
                }
                _ => None,
            })
            .unwrap_or_else(|| panic!("{spelling} folds onto the typed validation state"));
        assert_eq!(cond, *want, "{spelling} folds to the right state");
    }
}

#[test]
fn the_validation_states_ride_the_probe_tier() {
    // The validation test IS a selector conjunct (`:invalid`, …), not an
    // at-rule: the specificity contract pins it on the element-own
    // probe tier (beside `hover:` and `checked:`), below the media tier
    // the breakpoint and capability forms ride.
    for cond in [
        Cond::Valid,
        Cond::Invalid,
        Cond::UserValid,
        Cond::UserInvalid,
    ] {
        assert_eq!(cond.specificity(), Cond::Hover.specificity());
        assert!(cond.specificity() < Cond::Media(aeolus_ir::Pref::Print).specificity());
    }
}

#[test]
fn the_validation_conditions_name_their_own_spelling() {
    // The canonical variant text rides the condition (the diagnostics
    // and the metamorphic labels read it; no payload table needed —
    // the spellings are closed).
    assert_eq!(Cond::Valid.source(), Some("valid"));
    assert_eq!(Cond::Invalid.source(), Some("invalid"));
    assert_eq!(Cond::UserValid.source(), Some("user-valid"));
    assert_eq!(Cond::UserInvalid.source(), Some("user-invalid"));
}

// ---------------------------------------------------------------------------
// The interp: the Env facts answer, the unset bit is a quiet miss
// ---------------------------------------------------------------------------

#[test]
fn the_invalid_fact_answers_the_layer() {
    let compiled = compile("invalid:p-8", &Config::default()).expect("compiles");
    let env = Env {
        validation_invalid: true,
        ..Env::default()
    };
    let interped = interp(&compiled.program, &env).expect("no theme forms");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));
    assert!(interped.notes.is_empty(), "a fire is not a note");
}

#[test]
fn the_resting_env_answers_the_valid_query() {
    // `valid:` fires when the constraints do NOT fail — the same Env
    // fact, the complement question (the pair's two v4 values share one
    // fact, the `rtl:`/`ltr:` precedent — and CSS's own reading for a
    // pristine field IS :valid, so the resting env answers the valid
    // query the way it answers `ltr:`).
    let compiled = compile("valid:p-8", &Config::default()).expect("compiles");
    let interped = interp(&compiled.program, &Env::default()).expect("no theme forms");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));
    assert!(interped.notes.is_empty(), "a fire is not a note");

    // With the failing-constraints fact asserted, the valid layer
    // misses and the invalid layer fires — one fact, both spellings
    // answer.
    let env = Env {
        validation_invalid: true,
        ..Env::default()
    };
    let interped = interp(
        &compile("valid:p-8", &Config::default())
            .expect("compiles")
            .program,
        &env,
    )
    .expect("no theme forms");
    assert!(
        interped.decls.is_empty(),
        "the failing constraints fail the valid query"
    );
}

#[test]
fn the_user_pair_stays_quiet_until_interaction() {
    // The HTML spec's `:user-*` pseudo-classes match only AFTER user
    // interaction: at the resting env (no interaction fact) BOTH
    // spellings stay the quiet miss a selector that does not match is —
    // never a fire, never an exclusion note.
    for spelling in ["user-valid", "user-invalid"] {
        let compiled = compile(&format!("{spelling}:p-8"), &Config::default()).expect("compiles");
        let interped = interp(&compiled.program, &Env::default()).expect("no theme forms");
        assert!(
            interped.decls.is_empty(),
            "{spelling}: {:?}",
            interped.decls
        );
        assert!(
            interped.notes.is_empty(),
            "{spelling}: {:?}",
            interped.notes
        );
    }
}

#[test]
fn the_user_gate_answers_with_the_constraint_fact() {
    // Past the interaction gate, the `user-` pair answers from the SAME
    // constraint fact the `valid:`/`invalid:` pair shares: `user-valid`
    // on (gate AND not-failing), `user-invalid` on (gate AND failing) —
    // both quiet on the complement side, never a note.
    let cases: &[(&str, bool)] = &[("user-valid", false), ("user-invalid", true)];
    for (spelling, failing) in cases {
        let compiled = compile(&format!("{spelling}:p-8"), &Config::default()).expect("compiles");
        let env = Env {
            validation_invalid: *failing,
            user_validated: true,
            ..Env::default()
        };
        let interped = interp(&compiled.program, &env).expect("no theme forms");
        assert_eq!(
            resolved(&interped, Prop::PaddingTop),
            Val::Rem(2.0),
            "{spelling} fires past the gate with the matching fact"
        );
        assert!(interped.notes.is_empty(), "a fire is not a note");
    }
}

// ---------------------------------------------------------------------------
// The CSS arm: the upstream selector texts, pinned — no at-rule head
// ---------------------------------------------------------------------------

#[test]
fn the_css_arm_renders_the_upstream_pseudo_classes() {
    // The texts are variants.ts's own `staticVariant` selectors
    // (v4.1.17, the forms block), with the `&` the utility selector
    // replaces: `staticVariant('valid', ['&:valid'])` and siblings.
    let cases: &[(&str, &str)] = &[
        ("valid", ":valid"),
        ("invalid", ":invalid"),
        ("user-valid", ":user-valid"),
        ("user-invalid", ":user-invalid"),
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
fn the_validation_head_is_selector_side() {
    // The family is NOT an at-rule: no `@media`/`@container`/`@supports`
    // head renders (the validation test rides the element's own
    // selector, the probe tier the hover/checked forms occupy).
    for spelling in ["valid", "invalid", "user-valid", "user-invalid"] {
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
    // The v0.x `State` enum ALREADY carries the four states (the forms
    // block v0.9 landed): every layer lands its `Test::State(...)` rule
    // — no note rides (the `checked:` precedent, not the direction
    // boundary).
    use aeolus_core::ast::State;
    use aeolus_core::{Atom, Test};
    let cases: &[(&str, State)] = &[
        ("valid", State::Valid),
        ("invalid", State::Invalid),
        ("user-valid", State::UserValid),
        ("user-invalid", State::UserInvalid),
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
// The metamorphic triangle: compile == CSS == interp under the facts
// ---------------------------------------------------------------------------

#[test]
fn the_metamorphic_triangle_agrees_on_a_validation_program() {
    // The conjunctive checker bakes the constraint facts into the
    // situation envs it walks (the element-state tier, the state
    // families' own rule): a validation program agrees across the three
    // walks — compile == CSS == interp.
    let report = aeolus_compile::metamorphic_check(
        "invalid:p-8 valid:p-4 user-valid:p-2 user-invalid:hidden",
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
