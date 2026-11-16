//! The form-state family's interp surface (`enabled:`/`required:`/
//! `optional:`/`in-range:`/`out-of-range:` — the matches-neither closure,
//! tailwindcss v4.1.17's own static variants, pinned verbatim from the
//! source: `packages/tailwindcss/src/variants.ts`'s state/forms blocks:
//! `staticVariant('enabled', ['&:enabled'])`,
//! `staticVariant('required', ['&:required'])`,
//! `staticVariant('optional', ['&:optional'])`,
//! `staticVariant('in-range', ['&:in-range'])`,
//! `staticVariant('out-of-range', ['&:out-of-range'])`). The spellings
//! evidence 27 left at the matches-neither boundary — this family closes
//! it: a non-form element matches NEITHER half of each pair, so the
//! honest [`Env`] shape is a MEMBERSHIP fact per universe beside the
//! state bit (two facts per pair, not one). The seam:
//!
//! * [`Cond::Enabled`]/[`Cond::Required`]/[`Cond::Optional`]/
//!   [`Cond::InRange`]/[`Cond::OutOfRange`] — the five typed conditions
//!   the five spellings fold to, one bare variant per family spelling
//!   (the `target:`/`checked:` shape);
//! * the [`Env`] facts — the form-own answers are ELEMENT STATE the host
//!   answers (the tier the `hover:`/`target:` bits ride): `form_associated`
//!   (the membership fact — the element is a form-associated control,
//!   the HTML list the `:enabled`/`:required`/`:optional` pseudo-classes
//!   read), `required_field` (the `required` attribute the required/
//!   optional pair splits on), `range_bounded` (the membership fact for
//!   the range pair — the element tests its value against declared
//!   min/max bounds and holds a bounds answer for the current value) and
//!   `range_in` (the value sits within the bounds, the pair's shared
//!   position fact — `out-of-range:` its complement within the bounded
//!   universe, the `validation_invalid` precedent). Within each universe
//!   the pair shares one state fact; WITHOUT the membership fact the
//!   resting env matches neither half — the matches-neither reading the
//!   one-bit shape could not answer honestly;
//! * the CSS head — the upstream `staticVariant` texts verbatim
//!   (`:enabled`, `:required`, `:optional`, `:in-range`,
//!   `:out-of-range`; the family is NOT an at-rule: no `@media` head
//!   renders), the selector-side tier beside `:target`;
//! * the plan bridge — the v0.x `State` surface ALREADY carries the five
//!   states (the landed slots evidence 27 documented), so every layer
//!   LANDS its `Test::State(...)` rule slot — no note rides (the
//!   `checked:` precedent, not the direction boundary);
//! * the `has-` inner grammar answers the five spellings beside
//!   `target:` (one grammar, three consumers — the fold, the CSS arm's
//!   parent-relative heads, the test pin).
//!
//! Written FIRST (the RED spec): the suite fails against the
//! post-widget-state-family tree — [`Cond::Enabled`] and friends do not
//! exist yet (the fold refuses every form-state spelling as
//! unsupported-variant).

use aeolus_compile::{Config, Env, KNOWN_VARIANTS, compile, emit_css, emit_plan, interp};
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
fn every_form_state_spelling_folds_onto_the_typed_cond() {
    let cases: &[(&str, Cond)] = &[
        ("enabled", Cond::Enabled),
        ("required", Cond::Required),
        ("optional", Cond::Optional),
        ("in-range", Cond::InRange),
        ("out-of-range", Cond::OutOfRange),
    ];
    for (spelling, want) in cases {
        let compiled = compile(&format!("{spelling}:p-8"), &Config::default()).expect("compiles");
        let cond = compiled
            .program
            .layers
            .iter()
            .find_map(|layer| match layer.cond {
                cond @ (Cond::Enabled | Cond::Required | Cond::Optional) => Some(cond),
                cond @ (Cond::InRange | Cond::OutOfRange) => Some(cond),
                _ => None,
            })
            .unwrap_or_else(|| panic!("{spelling} folds onto the typed form state"));
        assert_eq!(cond, *want, "{spelling} folds to the right state");
    }
}

#[test]
fn the_form_states_ride_the_probe_tier() {
    // The form-state test IS a selector conjunct (`:enabled`,
    // `:in-range`), not an at-rule: the specificity contract pins it on
    // the element-own probe tier (beside `hover:` and `target:`), below
    // the media tier the breakpoint and capability forms ride.
    for cond in [
        Cond::Enabled,
        Cond::Required,
        Cond::Optional,
        Cond::InRange,
        Cond::OutOfRange,
    ] {
        assert_eq!(cond.specificity(), Cond::Hover.specificity());
        assert!(cond.specificity() < Cond::Media(aeolus_ir::Pref::Print).specificity());
    }
}

#[test]
fn the_form_state_conditions_name_their_own_spelling() {
    // The canonical variant text rides the condition (the diagnostics
    // and the metamorphic labels read it; no payload table needed —
    // the spellings are closed).
    assert_eq!(Cond::Enabled.source(), Some("enabled"));
    assert_eq!(Cond::Required.source(), Some("required"));
    assert_eq!(Cond::Optional.source(), Some("optional"));
    assert_eq!(Cond::InRange.source(), Some("in-range"));
    assert_eq!(Cond::OutOfRange.source(), Some("out-of-range"));
}

// ---------------------------------------------------------------------------
// The interp: the membership fact answers, the unset universe is a quiet miss
// ---------------------------------------------------------------------------

#[test]
fn the_enabled_fact_answers_the_form_associated_element() {
    // The enabled/disabled pair is the form-membership pair: a form
    // control that is not disabled is `:enabled` (CSS's own reading).
    // A non-form element matches NEITHER half — the resting env's
    // unset-membership miss answers even though the disabled bit is
    // false, the matches-neither shape one Env bit could not encode
    // (evidence 27's own boundary).
    let compiled = compile("enabled:p-8", &Config::default()).expect("compiles");
    let env = Env {
        form_associated: true,
        ..Env::default()
    };
    let interped = interp(&compiled.program, &env).expect("no theme forms");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));
    assert!(interped.notes.is_empty(), "a fire is not a note");

    // The disabled form control: the membership rides, the disabled bit
    // answers — `:enabled` does not match a disabled control.
    let env = Env {
        form_associated: true,
        disabled: true,
        ..Env::default()
    };
    let interped = interp(&compiled.program, &env).expect("no theme forms");
    assert!(interped.decls.is_empty(), "{:?}", interped.decls);

    // The resting env: no membership — the non-form element matches
    // neither half (a quiet miss, never a note).
    let interped = interp(&compiled.program, &Env::default()).expect("no theme forms");
    assert!(interped.decls.is_empty(), "{:?}", interped.decls);
    assert!(interped.notes.is_empty(), "a miss is not a note");
}

#[test]
fn the_required_fact_answers_the_marked_field() {
    // The required half rides the membership fact AND the required
    // attribute: only a form control that CARRIES the attribute is
    // `:required`.
    let compiled = compile("required:p-8", &Config::default()).expect("compiles");
    let env = Env {
        form_associated: true,
        required_field: true,
        ..Env::default()
    };
    let interped = interp(&compiled.program, &env).expect("no theme forms");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));
    assert!(interped.notes.is_empty(), "a fire is not a note");

    // The unmarked form control: the membership rides without the
    // attribute — the `:optional` half's answer, not `:required`'s.
    let env = Env {
        form_associated: true,
        ..Env::default()
    };
    let interped = interp(&compiled.program, &env).expect("no theme forms");
    assert!(interped.decls.is_empty(), "{:?}", interped.decls);

    // The resting env: a non-form element matches neither half — the
    // attribute bit alone never fires (the membership is the answer's
    // gate, the matches-neither boundary the family closes).
    let env = Env {
        required_field: true,
        ..Env::default()
    };
    let interped = interp(&compiled.program, &env).expect("no theme forms");
    assert!(interped.decls.is_empty(), "{:?}", interped.decls);
    assert!(interped.notes.is_empty(), "a miss is not a note");
}

#[test]
fn the_optional_fact_answers_the_unmarked_field() {
    // The optional half is the required attribute's COMPLEMENT within
    // the form universe (the `validation_invalid` precedent — within
    // the membership the pair shares one state fact): a form control
    // that carries no `required` attribute is `:optional`. The resting
    // env (no membership) matches neither half — the complement is not
    // a resting answer, the boundary evidence 27 documented.
    let compiled = compile("optional:p-8", &Config::default()).expect("compiles");
    let env = Env {
        form_associated: true,
        ..Env::default()
    };
    let interped = interp(&compiled.program, &env).expect("no theme forms");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));
    assert!(interped.notes.is_empty(), "a fire is not a note");

    // The marked field: the attribute answers — `:optional` does not
    // match a required control.
    let env = Env {
        form_associated: true,
        required_field: true,
        ..Env::default()
    };
    let interped = interp(&compiled.program, &env).expect("no theme forms");
    assert!(interped.decls.is_empty(), "{:?}", interped.decls);

    // The resting env: no membership — neither half answers (a quiet
    // miss, never a note).
    let interped = interp(&compiled.program, &Env::default()).expect("no theme forms");
    assert!(interped.decls.is_empty(), "{:?}", interped.decls);
    assert!(interped.notes.is_empty(), "a miss is not a note");
}

#[test]
fn the_in_range_fact_answers_the_bounded_value() {
    // The range pair rides the bounded universe's membership fact
    // beside the value's position: a bounded field whose value sits
    // within the declared min/max bounds is `:in-range`.
    let compiled = compile("in-range:p-8", &Config::default()).expect("compiles");
    let env = Env {
        range_bounded: true,
        range_in: true,
        ..Env::default()
    };
    let interped = interp(&compiled.program, &env).expect("no theme forms");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));
    assert!(interped.notes.is_empty(), "a fire is not a note");

    // The bounded field whose value sits outside: the position fact's
    // complement answers — `:in-range` does not match it (the
    // `out-of-range` half's answer).
    let env = Env {
        range_bounded: true,
        ..Env::default()
    };
    let interped = interp(&compiled.program, &env).expect("no theme forms");
    assert!(interped.decls.is_empty(), "{:?}", interped.decls);

    // The resting env: no bounds answer — the element matches neither
    // half (a non-range element, or a field with no position answer
    // yet — the host keeps the membership fact off until the value
    // exists, the HTML reading for a pristine empty field).
    let interped = interp(&compiled.program, &Env::default()).expect("no theme forms");
    assert!(interped.decls.is_empty(), "{:?}", interped.decls);
    assert!(interped.notes.is_empty(), "a miss is not a note");
}

#[test]
fn the_out_of_range_fact_answers_the_bounded_complement() {
    // The out-of-range half is the position fact's COMPLEMENT within
    // the bounded universe: a bounded field whose value sits outside
    // the declared bounds is `:out-of-range`. The resting env matches
    // neither half — the complement is not a resting answer (the
    // `optional:` arm's own reading).
    let compiled = compile("out-of-range:p-8", &Config::default()).expect("compiles");
    let env = Env {
        range_bounded: true,
        ..Env::default()
    };
    let interped = interp(&compiled.program, &env).expect("no theme forms");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));
    assert!(interped.notes.is_empty(), "a fire is not a note");

    // The value within bounds: the position fact answers —
    // `:out-of-range` does not match it.
    let env = Env {
        range_bounded: true,
        range_in: true,
        ..Env::default()
    };
    let interped = interp(&compiled.program, &env).expect("no theme forms");
    assert!(interped.decls.is_empty(), "{:?}", interped.decls);

    // The resting env: no membership — neither half answers (a quiet
    // miss, never a note).
    let interped = interp(&compiled.program, &Env::default()).expect("no theme forms");
    assert!(interped.decls.is_empty(), "{:?}", interped.decls);
    assert!(interped.notes.is_empty(), "a miss is not a note");
}

// ---------------------------------------------------------------------------
// The CSS arm: the upstream selector texts, pinned — no at-rule head
// ---------------------------------------------------------------------------

#[test]
fn the_css_arm_renders_the_upstream_selector_texts() {
    // The texts are variants.ts's own `staticVariant` selectors
    // (v4.1.17, the state/forms blocks), with the `&` the utility
    // selector replaces: `staticVariant('enabled', ['&:enabled'])` and
    // siblings.
    let cases: &[(&str, &str)] = &[
        ("enabled", ":enabled"),
        ("required", ":required"),
        ("optional", ":optional"),
        ("in-range", ":in-range"),
        ("out-of-range", ":out-of-range"),
    ];
    for (spelling, head) in cases {
        let sheet = emit_css(&format!("{spelling}:p-8"), &Config::default()).expect("emits");
        assert!(sheet.diagnostics.is_empty(), "{:?}", sheet.diagnostics);
        let rendered = sheet.to_string();
        assert!(
            rendered.contains(head),
            "{spelling} renders the v4.1.17 selector text: {rendered}"
        );
    }
}

#[test]
fn the_form_state_head_is_selector_side() {
    // The family is NOT an at-rule: no `@media`/`@container`/`@supports`
    // head renders (the form-state test rides the element's own
    // selector, the probe tier the hover/target forms occupy).
    for spelling in [
        "enabled",
        "required",
        "optional",
        "in-range",
        "out-of-range",
    ] {
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
    // The v0.x `State` enum ALREADY carries the five states (the landed
    // slots evidence 27 documented at the boundary): every layer lands
    // its `Test::State(...)` rule — no note rides (the `checked:`
    // precedent, not the direction boundary).
    use aeolus_core::ast::State;
    use aeolus_core::{Atom, Test};
    let cases: &[(&str, State)] = &[
        ("enabled", State::Enabled),
        ("required", State::Required),
        ("optional", State::Optional),
        ("in-range", State::InRange),
        ("out-of-range", State::OutOfRange),
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

#[test]
fn the_state_slots_round_trip_the_snapshot() {
    // The snapshot's strict reader accepts the family's own state
    // names (the byte-stable JSON contract): the plan bridge's landed
    // rules survive a round trip unchanged.
    let sheet = emit_plan(
        "enabled:p-8 required:hidden optional:ring-2 in-range:mt-2 out-of-range:p-4",
        &Config::default(),
    )
    .expect("emits");
    let json = sheet.plan.to_snapshot_json();
    let reparsed = aeolus_core::StylePlan::from_snapshot_json(&json)
        .expect("the strict reader accepts the form-state names");
    assert_eq!(sheet.plan.to_snapshot_json(), reparsed.to_snapshot_json());
}

// ---------------------------------------------------------------------------
// The `has-` inner grammar: one grammar, three consumers
// ---------------------------------------------------------------------------

#[test]
fn the_has_inner_grammar_answers_the_form_states() {
    // The `has-` inner reads the five spellings beside `target:` (the
    // same state_pseudo table the CSS arm's heads render): the rendered
    // `:has(...)` fragment is the star-prefixed selector text (the v4
    // mechanism — the inner variant's `&` becomes `*`).
    for (spelling, fragment) in [
        ("enabled", "*:enabled"),
        ("required", "*:required"),
        ("optional", "*:optional"),
        ("in-range", "*:in-range"),
        ("out-of-range", "*:out-of-range"),
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
// The completion surface: the closed spelling set carries the family
// ---------------------------------------------------------------------------

#[test]
fn the_known_variant_surface_carries_the_family() {
    // The completion surface only ever suggests spellings the fold
    // accepts (`every_known_variant_folds` pins the audit) — the five
    // spellings ride the closed table beside the widget states.
    for spelling in [
        "enabled",
        "required",
        "optional",
        "in-range",
        "out-of-range",
    ] {
        assert!(
            KNOWN_VARIANTS.contains(&spelling),
            "the completion surface names {spelling:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// The metamorphic triangle: compile == CSS == interp under the facts
// ---------------------------------------------------------------------------

#[test]
fn the_metamorphic_triangle_agrees_on_a_form_state_program() {
    // The conjunctive checker bakes the form-state facts into the
    // situation envs it walks (the element-state tier, the membership
    // facts beside the state bits): a form-state program agrees across
    // the three walks — compile == CSS == interp.
    let report = aeolus_compile::metamorphic_check(
        "enabled:p-8 required:p-4 optional:hidden in-range:ring-2 out-of-range:mt-2",
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
