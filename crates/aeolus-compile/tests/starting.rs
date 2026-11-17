//! The `starting:` variant (`@starting-style`, the transition's first
//! frame — the tailwindcss v4.1.17 `variants.ts`
//! `staticVariant('starting', ['@starting-style'])` registration, the
//! media-feature tier's at-rule sibling whose `Env` reading the
//! pseudo-element landing deliberately deferred). The family:
//!
//! * the typed at-rule condition — [`Cond::Starting`], the at-rule
//!   tier's specificity 4 (an `@starting-style` block WRAPS the rule,
//!   the `@media` shape, never a selector conjunct);
//! * the `Env` fact — ONE frame fact (`starting_style`: the element is
//!   in its starting-style frame, the host's first-render gate beside
//!   the transition), an unset bit the QUIET miss a frame the element
//!   is not in is (the preference facts' own reading, no probe seam
//!   and no sourceless note);
//! * the CSS arm — the at-rule head IS the upstream text
//!   (`@starting-style`, no selector suffix, the media tier's render);
//! * the plan bridge — the typed `PlanNoteKind::StartingStyle` note
//!   rides (the v0.x plan surface has no starting-style slot, the
//!   media-feature boundary's sibling);
//! * the CLI's `--starting` probe flag (one flag per FACT, the
//!   `--portrait` precedent).
//!
//! Written FIRST (the RED spec): the suite fails against the post-
//! of-type tree — the spelling refuses as unsupported-variant and the
//! `Cond` set carries no starting-style arm.

use aeolus_compile::{Config, Env, PlanNoteKind, compile, emit_css, emit_plan, interp};
use aeolus_ir::{Cond, Prop, Val};

/// The value one longhand resolved to.
fn resolved(interped: &aeolus_compile::Interped, prop: Prop) -> Val {
    interped
        .decls
        .iter()
        .find(|d| d.prop == prop)
        .unwrap_or_else(|| panic!("{prop:?} is in the resolved declarations"))
        .val
}

#[test]
fn the_spelling_folds_onto_the_at_rule_cond() {
    let compiled = compile("starting:p-8", &Config::default()).expect("the string lowers");
    assert!(
        compiled
            .program
            .layers
            .iter()
            .any(|layer| layer.cond == Cond::Starting),
        "starting: folds to Cond::Starting"
    );
    assert!(
        compiled.diagnostics.is_empty(),
        "the fold is diagnostic-clean: {:?}",
        compiled.diagnostics
    );
}

#[test]
fn the_at_rule_tier_sorts_at_four() {
    // `@starting-style` wraps the rule — the `@media` tier, never the
    // selector-conjunct probe tier.
    assert_eq!(Cond::Starting.specificity(), 4);
}

#[test]
fn the_source_canon_names_the_spelling() {
    assert_eq!(Cond::Starting.source(), Some("starting"));
}

#[test]
fn the_unset_fact_is_the_quiet_miss() {
    let compiled = compile("starting:p-8", &Config::default()).expect("the string lowers");
    let interped = interp(&compiled.program, &Env::default()).expect("the walk resolves");
    assert!(
        interped.decls.iter().all(|d| d.prop != Prop::PaddingTop),
        "an unset frame fact is the quiet miss a frame the element is not in is"
    );
    assert!(
        interped.notes.is_empty(),
        "the frame facts carry no sourceless note (the media tier's reading): {:?}",
        interped.notes
    );
}

#[test]
fn the_fact_answers_the_layer() {
    let compiled = compile("starting:p-8", &Config::default()).expect("the string lowers");
    let env = Env {
        starting_style: true,
        ..Env::default()
    };
    let interped = interp(&compiled.program, &env).expect("the walk resolves");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));
}

#[test]
fn the_css_arm_renders_the_upstream_at_rule_head() {
    let sheet = emit_css("starting:p-8", &Config::default()).expect("the string renders");
    assert!(sheet.diagnostics.is_empty());
    assert_eq!(
        sheet.rules[0].media.as_ref().map(|at| at.text()),
        Some("@starting-style".to_string()),
        "the upstream registration text — the bare at-rule head"
    );
}

#[test]
fn the_plan_bridge_reports_the_boundary() {
    let sheet = emit_plan("starting:p-8", &Config::default())
        .unwrap_or_else(|e| panic!("starting: lowers ({e})"));
    let note = sheet
        .notes
        .iter()
        .find(|note| matches!(note.kind, PlanNoteKind::StartingStyle))
        .unwrap_or_else(|| panic!("starting: the starting-style boundary rides"));
    assert!(
        matches!(note.kind, PlanNoteKind::StartingStyle),
        "the typed note names the boundary: {:?}",
        note.kind
    );
}

#[test]
fn the_metamorphic_checker_agrees_under_the_fired_fact() {
    // The checker's conjunctive env bakes the fact the layer asks
    // about, so the compile == CSS == interp triangle agrees (the
    // media-feature families' own contract).
    let report = aeolus_compile::metamorphic_check("starting:p-8", &Config::default())
        .expect("the checker runs");
    assert!(
        report.agree(),
        "the triangle agrees under the conjunctive env: {:?}",
        report.notes
    );
}
