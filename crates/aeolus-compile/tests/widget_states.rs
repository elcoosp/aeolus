//! The widget-state family's interp surface (`open:`/`inert:`/`default:`
//! — the v0.x-gap closure, tailwindcss v4.1.17's own static variants,
//! pinned verbatim from the source:
//! `packages/tailwindcss/src/variants.ts`'s state/forms blocks:
//! `staticVariant('open', ['&:is([open], :popover-open, :open)'])`,
//! `staticVariant('inert', ['&:is([inert], [inert] *)'])`,
//! `staticVariant('default', ['&:default'])`). The spellings evidence 28
//! left at the v0.x-surface boundary — this family closes the gap in the
//! OTHER direction: the v0.x `State` surface grows the three slots AND
//! the V1 surface grows the three conditions. The seam:
//!
//! * [`Cond::Open`]/[`Cond::Inert`]/[`Cond::Default`] — the three typed
//!   conditions the three spellings fold to, one bare variant per
//!   family spelling (the `target:`/`checked:` shape);
//! * the [`Env`] facts — the widget-own answers are ELEMENT STATE the
//!   host answers (the tier the `hover:`/`target:` bits ride): ONE fact
//!   per question (`opened` — the disclosure/popover answer,
//!   `inert` — the frozen-subtree answer the host computes for every
//!   element inside (or carrying) an `[inert]` root, `form_default` —
//!   the form group's default-control answer), an unset fact the QUIET
//!   miss a selector that does not match is, never an exclusion note
//!   (the `target:`/`autofill:` precedent — every one of the three is
//!   an AFFIRMATIVE one-fact question: a non-open element matches
//!   neither `:open`'s arms the way a non-target element misses
//!   `:target`, so no complement rides);
//! * the CSS head — the upstream `staticVariant` texts verbatim
//!   (`:is([open], :popover-open, :open)`, `:is([inert], [inert] *)`,
//!   `:default`; the family is NOT an at-rule: no `@media` head
//!   renders), the selector-side tier beside `:target`;
//! * the plan bridge — the v0.x `State` surface NOW carries the three
//!   states (the family's own slots), so every layer LANDS its
//!   `Test::State(...)` rule slot — no note rides (the `checked:`
//!   precedent, not the direction boundary);
//! * the `has-` inner grammar answers the three spellings beside
//!   `target:` (one grammar, three consumers — the fold, the CSS arm's
//!   parent-relative heads, the test pin).
//!
//! Written FIRST (the RED spec): the suite fails against the
//! post-element-state-family tree — [`Cond::Open`] and friends do not
//! exist yet (the fold refuses every widget-state spelling as
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
fn every_widget_state_spelling_folds_onto_the_typed_cond() {
    let cases: &[(&str, Cond)] = &[
        ("open", Cond::Open),
        ("inert", Cond::Inert),
        ("default", Cond::Default),
    ];
    for (spelling, want) in cases {
        let compiled = compile(&format!("{spelling}:p-8"), &Config::default()).expect("compiles");
        let cond = compiled
            .program
            .layers
            .iter()
            .find_map(|layer| match layer.cond {
                cond @ (Cond::Open | Cond::Inert | Cond::Default) => Some(cond),
                _ => None,
            })
            .unwrap_or_else(|| panic!("{spelling} folds onto the typed widget state"));
        assert_eq!(cond, *want, "{spelling} folds to the right state");
    }
}

#[test]
fn the_widget_states_ride_the_probe_tier() {
    // The widget-state test IS a selector conjunct (`:is([open], …)`,
    // `:default`), not an at-rule: the specificity contract pins it on
    // the element-own probe tier (beside `hover:` and `target:`), below
    // the media tier the breakpoint and capability forms ride.
    for cond in [Cond::Open, Cond::Inert, Cond::Default] {
        assert_eq!(cond.specificity(), Cond::Hover.specificity());
        assert!(cond.specificity() < Cond::Media(aeolus_ir::Pref::Print).specificity());
    }
}

#[test]
fn the_widget_state_conditions_name_their_own_spelling() {
    // The canonical variant text rides the condition (the diagnostics
    // and the metamorphic labels read it; no payload table needed —
    // the spellings are closed).
    assert_eq!(Cond::Open.source(), Some("open"));
    assert_eq!(Cond::Inert.source(), Some("inert"));
    assert_eq!(Cond::Default.source(), Some("default"));
}

// ---------------------------------------------------------------------------
// The interp: the Env facts answer, the unset bit is a quiet miss
// ---------------------------------------------------------------------------

#[test]
fn the_open_fact_answers_the_layer() {
    let compiled = compile("open:p-8", &Config::default()).expect("compiles");
    let env = Env {
        opened: true,
        ..Env::default()
    };
    let interped = interp(&compiled.program, &env).expect("no theme forms");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));
    assert!(interped.notes.is_empty(), "a fire is not a note");

    // The resting env asserts no open fact — the layer misses QUIETLY
    // (an `:is([open], …)` selector that does not match, never a note).
    let interped = interp(&compiled.program, &Env::default()).expect("no theme forms");
    assert!(interped.decls.is_empty(), "{:?}", interped.decls);
    assert!(interped.notes.is_empty(), "a miss is not a note");
}

#[test]
fn the_inert_fact_answers_the_layer() {
    // The fact is the SUBTREE answer: the host computes "this element
    // is within (or itself carries) an `[inert]` root" per element —
    // one bit answers the compound `:is([inert], [inert] *)` head.
    let compiled = compile("inert:p-8", &Config::default()).expect("compiles");
    let env = Env {
        inert: true,
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
fn the_form_default_fact_answers_the_layer() {
    let compiled = compile("default:p-8", &Config::default()).expect("compiles");
    let env = Env {
        form_default: true,
        ..Env::default()
    };
    let interped = interp(&compiled.program, &env).expect("no theme forms");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));
    assert!(interped.notes.is_empty(), "a fire is not a note");

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
    // selector replaces: `staticVariant('open',
    // ['&:is([open], :popover-open, :open)'])` and siblings.
    let cases: &[(&str, &str)] = &[
        ("open", ":is([open], :popover-open, :open)"),
        ("inert", ":is([inert], [inert] *)"),
        ("default", ":default"),
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
fn the_widget_state_head_is_selector_side() {
    // The family is NOT an at-rule: no `@media`/`@container`/`@supports`
    // head renders (the widget-state test rides the element's own
    // selector, the probe tier the hover/target forms occupy).
    for spelling in ["open", "inert", "default"] {
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
    // The v0.x `State` enum NOW carries the three states (the family's
    // own slots, the gap evidence 28 named): every layer lands its
    // `Test::State(...)` rule — no note rides (the `checked:`
    // precedent, not the direction boundary).
    use aeolus_core::ast::State;
    use aeolus_core::{Atom, Test};
    let cases: &[(&str, State)] = &[
        ("open", State::Open),
        ("inert", State::Inert),
        ("default", State::Default),
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
    let sheet =
        emit_plan("open:p-8 inert:hidden default:ring-2", &Config::default()).expect("emits");
    let json = sheet.plan.to_snapshot_json();
    let reparsed = aeolus_core::StylePlan::from_snapshot_json(&json)
        .expect("the strict reader accepts the widget-state names");
    assert_eq!(sheet.plan.to_snapshot_json(), reparsed.to_snapshot_json());
}

// ---------------------------------------------------------------------------
// The `has-` inner grammar: one grammar, three consumers
// ---------------------------------------------------------------------------

#[test]
fn the_has_inner_grammar_answers_the_widget_states() {
    // The `has-` inner reads the three spellings beside `target:` (the
    // same state_pseudo table the CSS arm's heads render): the rendered
    // `:has(...)` fragment is the star-prefixed selector text (the v4
    // mechanism — the inner variant's `&` becomes `*`).
    for (spelling, fragment) in [
        ("open", "*:is([open], :popover-open, :open)"),
        ("inert", "*:is([inert], [inert] *)"),
        ("default", "*:default"),
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
    // accepts (`every_known_variant_folds` pins the audit) — the three
    // spellings ride the closed table beside the element states.
    for spelling in ["open", "inert", "default"] {
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
fn the_metamorphic_triangle_agrees_on_a_widget_state_program() {
    // The conjunctive checker bakes the widget-state facts into the
    // situation envs it walks (the element-state tier, the state
    // families' own rule): a widget-state program agrees across the
    // three walks — compile == CSS == interp.
    let report =
        aeolus_compile::metamorphic_check("open:p-8 default:p-4 inert:hidden", &Config::default())
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
