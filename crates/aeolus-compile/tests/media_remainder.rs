//! The media-feature REMAINDER (the preference/capability tier's last
//! two v4.1.17 static variants, `packages/tailwindcss/src/variants.ts`
//! pinned verbatim: `staticVariant('inverted-colors',
//! ['@media (inverted-colors: inverted)'])` and
//! `staticVariant('noscript', ['@media (scripting: none)'])`). The
//! family:
//!
//! * the two typed bits — [`Pref::InvertedColors`]/[`Pref::NoScript`]
//!   riding the closed [`Pref`] set the [`Cond::Media`] payload
//!   carries (the pointer/orientation tier's own shape);
//! * the `Env` facts — the user agent's media-query answers are FRAME
//!   STATE (the tier every preference bit rides): ONE fact per
//!   question (`inverted_colors`, `scripting_none`), an unset bit the
//!   QUIET miss a media query that does not hold is;
//! * the CSS arm — the at-rule heads ARE the upstream texts (no
//!   selector suffix renders, the at-rule tier);
//! * the plan bridge — the typed `PlanNoteKind::MediaFeature` note
//!   rides (the v0.x Atom set carries width bounds only, the
//!   media-feature boundary — no new note kind);
//! * the CLI's `--inverted-colors`/`--noscript` probe flags (the
//!   capability flags' sibling — one flag per FACT, the `--portrait`
//!   precedent).
//!
//! Written FIRST (the RED spec): the suite fails against the
//! post-pseudo-element tree — both spellings refuse as
//! unsupported-variant and the [`Pref`] set carries neither bit.

use aeolus_compile::{Config, Env, PlanNoteKind, compile, emit_css, emit_plan, interp};
use aeolus_ir::{Cond, Pref, Prop, Val};

/// The value one longhand resolved to.
fn resolved(interp: &aeolus_compile::Interped, prop: Prop) -> Val {
    interp
        .decls
        .iter()
        .find(|d| d.prop == prop)
        .unwrap_or_else(|| panic!("{prop:?} is in the resolved declarations"))
        .val
}

#[test]
fn the_remainder_spellings_fold_onto_the_typed_bits() {
    let cases: &[(&str, Pref)] = &[
        ("inverted-colors", Pref::InvertedColors),
        ("noscript", Pref::NoScript),
    ];
    for (spelling, expected) in cases {
        let compiled = compile(&format!("{spelling}:p-8"), &Config::default())
            .unwrap_or_else(|e| panic!("{spelling}: lowers ({e})"));
        assert!(
            compiled.diagnostics.is_empty(),
            "{spelling}: unexpected diagnostics {:?}",
            compiled.diagnostics
        );
        assert_eq!(
            compiled.program.layers[0].cond,
            Cond::Media(*expected),
            "{spelling}: the typed media bit"
        );
        assert_eq!(
            expected.source(),
            *spelling,
            "{spelling}: the canonical text reconstructs"
        );
    }
}

#[test]
fn the_media_tier_sorts_at_the_at_rule_level() {
    assert_eq!(Cond::Media(Pref::InvertedColors).specificity(), 4);
    assert_eq!(Cond::Media(Pref::NoScript).specificity(), 4);
}

#[test]
fn the_unset_facts_are_the_quiet_miss() {
    let compiled = compile("inverted-colors:p-8 noscript:mt-2", &Config::default())
        .expect("the string lowers");
    let interped = interp(&compiled.program, &Env::default()).expect("the walk resolves");
    assert!(
        interped.decls.is_empty(),
        "no media query holds in the resting env: {:?}",
        interped.decls
    );
    assert!(
        interped.notes.is_empty(),
        "an unset frame fact is the quiet miss, never a note: {:?}",
        interped.notes
    );
}

#[test]
fn the_facts_answer_their_questions() {
    let compiled = compile("inverted-colors:p-8", &Config::default()).expect("the string lowers");
    let env = Env {
        inverted_colors: true,
        ..Env::default()
    };
    let interped = interp(&compiled.program, &env).expect("the walk resolves");
    assert_eq!(
        resolved(&interped, Prop::PaddingTop),
        Val::Rem(2.0),
        "the inverted-colors layer fires on the fact"
    );

    let compiled = compile("noscript:mt-2", &Config::default()).expect("the string lowers");
    let env = Env {
        scripting_none: true,
        ..Env::default()
    };
    let interped = interp(&compiled.program, &env).expect("the walk resolves");
    assert_eq!(
        resolved(&interped, Prop::MarginTop),
        Val::Rem(0.5),
        "the noscript layer fires on the fact"
    );
}

#[test]
fn the_css_arm_renders_the_upstream_at_rule_texts() {
    let sheet = emit_css("inverted-colors:p-8", &Config::default()).expect("the string renders");
    assert!(sheet.diagnostics.is_empty());
    assert_eq!(
        sheet.rules[0].media.as_ref().map(|at| at.text()),
        Some("@media (inverted-colors: inverted)".to_string()),
        "the upstream registration text"
    );

    let sheet = emit_css("noscript:mt-2", &Config::default()).expect("the string renders");
    assert_eq!(
        sheet.rules[0].media.as_ref().map(|at| at.text()),
        Some("@media (scripting: none)".to_string()),
        "the upstream registration text"
    );
}

#[test]
fn the_plan_bridge_keeps_the_media_feature_boundary() {
    for (spelling, expected) in [
        ("inverted-colors", Pref::InvertedColors),
        ("noscript", Pref::NoScript),
    ] {
        let sheet = emit_plan(&format!("{spelling}:p-8"), &Config::default())
            .unwrap_or_else(|e| panic!("{spelling}: lowers ({e})"));
        let note = sheet
            .notes
            .iter()
            .find(|note| matches!(note.kind, PlanNoteKind::MediaFeature { .. }))
            .unwrap_or_else(|| panic!("{spelling}: the media-feature boundary rides"));
        match &note.kind {
            PlanNoteKind::MediaFeature { pref } => assert_eq!(*pref, expected),
            other => panic!("unexpected note: {other:?}"),
        }
    }
}
