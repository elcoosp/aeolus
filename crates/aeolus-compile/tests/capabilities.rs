//! The capability-variant interp surface (the `pointer-*`/`any-pointer-*`
//! input families and the `portrait`/`landscape` orientation forms — the
//! media-feature tier's remaining v4.1.17 spellings beside the preference
//! families). The seam:
//!
//! * [`Cond::Media`] — the same typed condition the preference families
//!   ride, the payload the [`Pref`] bit (`pointer-fine:` →
//!   `Pref::PointerFine`, `portrait:` → `Pref::Portrait`, ...), one arm
//!   per family spelling;
//! * the [`Env`] facts — the capability bits are FRAME STATE (the user
//!   agent's own media-query answers, exactly the tier `dark:` and the
//!   `motion-*` preference bits ride), so the plain walk answers them
//!   from the [`Env`] with no probe seam and no sourceless note: an
//!   unset bit is a QUIET miss (a media query that does not hold),
//!   never an exclusion;
//! * [`CssAt::MediaPref`] — the at-rule head, the real tailwindcss
//!   v4.1.17 texts (`staticVariant` calls, pinned from the source:
//!   `@media (pointer: fine)`, `@media (pointer: coarse)`, `@media
//!   (pointer: none)`, `@media (any-pointer: fine)`, `@media
//!   (any-pointer: coarse)`, `@media (any-pointer: none)`, `@media
//!   (orientation: portrait)`, `@media (orientation: landscape)`);
//! * the plan bridge — the v0.x surface has no media-feature slot, so
//!   the layer reports through the existing [`PlanNoteKind::MediaFeature`]
//!   note, the same boundary the preference families keep.
//!
//! Written FIRST (the RED spec): the suite fails against the
//! post-preference families tree — the `Pref` bits do not exist yet (the
//! fold refuses every capability spelling as unsupported-variant).

use aeolus_compile::{Config, CssRule, Env, PlanNoteKind, compile, emit_css, emit_plan};
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

// ---------------------------------------------------------------------------
// The fold: one arm per family spelling
// ---------------------------------------------------------------------------

#[test]
fn every_capability_spelling_folds_onto_the_typed_cond() {
    let cases: &[(&str, Pref)] = &[
        ("pointer-fine", Pref::PointerFine),
        ("pointer-coarse", Pref::PointerCoarse),
        ("pointer-none", Pref::PointerNone),
        ("any-pointer-fine", Pref::AnyPointerFine),
        ("any-pointer-coarse", Pref::AnyPointerCoarse),
        ("any-pointer-none", Pref::AnyPointerNone),
        ("portrait", Pref::Portrait),
        ("landscape", Pref::Landscape),
    ];
    for (spelling, pref) in cases {
        let compiled = compile(&format!("{spelling}:p-8"), &Config::default()).expect("compiles");
        let cond = compiled
            .program
            .layers
            .iter()
            .find_map(|layer| match layer.cond {
                Cond::Media(inner) => Some(inner),
                _ => None,
            })
            .unwrap_or_else(|| panic!("{spelling} folds onto Cond::Media"));
        assert_eq!(cond, *pref, "{spelling} folds to the right bit");
    }
}

#[test]
fn the_capability_condition_rides_the_media_tier() {
    // The capability forms ARE media queries: the specificity contract
    // pins them beside the breakpoint forms (tier 4), below the
    // container tier (5) — the same tier the preference families ride.
    let pointer = Cond::Media(Pref::PointerFine);
    let orientation = Cond::Media(Pref::Landscape);
    let preference = Cond::Media(Pref::MotionReduce);
    let bp = Cond::Bp(aeolus_ir::Bp::Md);
    let container = Cond::Container {
        bp: aeolus_ir::Bp::Md,
        name: None,
    };
    assert_eq!(pointer.specificity(), bp.specificity());
    assert_eq!(orientation.specificity(), preference.specificity());
    assert!(pointer.specificity() < container.specificity());
}

#[test]
fn the_capability_condition_names_its_own_spelling() {
    // The canonical variant text rides the condition (the diagnostics
    // and the metamorphic labels read it; no payload table needed —
    // the spellings are closed).
    assert_eq!(
        Cond::Media(Pref::PointerFine).source(),
        Some("pointer-fine")
    );
    assert_eq!(
        Cond::Media(Pref::AnyPointerCoarse).source(),
        Some("any-pointer-coarse")
    );
    assert_eq!(Cond::Media(Pref::Portrait).source(), Some("portrait"));
    assert_eq!(Cond::Media(Pref::Landscape).source(), Some("landscape"));
}

// ---------------------------------------------------------------------------
// The interp: the Env answers, the unset bit is a quiet miss
// ---------------------------------------------------------------------------

#[test]
fn the_pointer_fact_answers_the_layer() {
    let compiled = compile("pointer-fine:p-8", &Config::default()).expect("compiles");
    let env = Env {
        pointer_fine: true,
        ..Env::default()
    };
    let interped = aeolus_compile::interp(&compiled.program, &env).expect("no theme forms");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));
    assert!(interped.notes.is_empty(), "a fire is not a note");
}

#[test]
fn the_unset_capability_fact_is_a_quiet_miss() {
    // The default Env asserts no capability fact (the media queries do
    // not hold): the layer misses QUIETLY, the CSS semantics a media
    // query obeys, never an exclusion note.
    let compiled = compile("pointer-coarse:p-8", &Config::default()).expect("compiles");
    let interped =
        aeolus_compile::interp(&compiled.program, &Env::default()).expect("no theme forms");
    assert!(interped.decls.is_empty(), "{:?}", interped.decls);
    assert!(interped.notes.is_empty(), "{:?}", interped.notes);
}

#[test]
fn every_pointer_family_asks_its_own_fact() {
    // One bit per family question: the primary pointer's three values
    // are three questions (a hit on one never implies another), the
    // any-pointer trio the same.
    let cases: &[(&str, fn(&mut Env))] = &[
        ("pointer-fine", |e| e.pointer_fine = true),
        ("pointer-coarse", |e| e.pointer_coarse = true),
        ("pointer-none", |e| e.pointer_none = true),
        ("any-pointer-fine", |e| e.any_pointer_fine = true),
        ("any-pointer-coarse", |e| e.any_pointer_coarse = true),
        ("any-pointer-none", |e| e.any_pointer_none = true),
    ];
    for (spelling, set) in cases {
        let compiled = compile(&format!("{spelling}:p-8"), &Config::default()).expect("compiles");
        let mut env = Env::default();
        set(&mut env);
        let interped = aeolus_compile::interp(&compiled.program, &env).expect("no theme forms");
        assert_eq!(
            resolved(&interped, Prop::PaddingTop),
            Val::Rem(2.0),
            "{spelling} answers from its own fact"
        );
    }
}

#[test]
fn landscape_asks_the_complement() {
    // `landscape:` fires when the viewport is NOT portrait — the same
    // Env bit, the complement question (the feature's two v4 values
    // share one fact, the motion-safe precedent; the orientation
    // feature always HAS an answer, so one of the two always matches).
    let compiled = compile("landscape:p-8", &Config::default()).expect("compiles");
    let interped =
        aeolus_compile::interp(&compiled.program, &Env::default()).expect("no theme forms");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));

    let compiled = compile("portrait:p-8", &Config::default()).expect("compiles");
    let interped =
        aeolus_compile::interp(&compiled.program, &Env::default()).expect("no theme forms");
    assert!(
        interped.decls.is_empty(),
        "the landscape viewport fails the portrait query"
    );

    let interped = aeolus_compile::interp(
        &compile("portrait:p-8", &Config::default())
            .expect("compiles")
            .program,
        &Env {
            portrait: true,
            ..Env::default()
        },
    )
    .expect("no theme forms");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));
}

// ---------------------------------------------------------------------------
// The CSS arm: the corpus's at-rule texts, pinned
// ---------------------------------------------------------------------------

#[test]
fn the_css_arm_renders_the_corpus_texts() {
    let cases: &[(&str, &str)] = &[
        ("pointer-fine", "@media (pointer: fine) {"),
        ("pointer-coarse", "@media (pointer: coarse) {"),
        ("pointer-none", "@media (pointer: none) {"),
        ("any-pointer-fine", "@media (any-pointer: fine) {"),
        ("any-pointer-coarse", "@media (any-pointer: coarse) {"),
        ("any-pointer-none", "@media (any-pointer: none) {"),
        ("portrait", "@media (orientation: portrait) {"),
        ("landscape", "@media (orientation: landscape) {"),
    ];
    for (spelling, head) in cases {
        let sheet = emit_css(&format!("{spelling}:p-8"), &Config::default()).expect("emits");
        assert!(sheet.diagnostics.is_empty(), "{:?}", sheet.diagnostics);
        let rendered = sheet.to_string();
        assert!(
            rendered.contains(head),
            "{spelling} renders the v4.1.17 text: {rendered}"
        );
    }
}

#[test]
fn the_sheet_groups_one_capability_block() {
    // Consecutive rules sharing one at-rule group into a single block
    // (the CssSheet Display contract, the media forms' own rule).
    let sheet = emit_css("pointer-fine:p-2 pointer-fine:p-4", &Config::default()).expect("emits");
    let rendered = sheet.to_string();
    assert_eq!(rendered.matches("@media (pointer: fine)").count(), 1);
    // The later rule wins the cascade inside the block.
    assert!(rendered.contains("padding-top: 1rem"), "{rendered}");
    let rules: Vec<CssRule> = sheet.rules.clone();
    assert_eq!(rules.len(), 2, "one rule per candidate, source order");
}

// ---------------------------------------------------------------------------
// The plan bridge: the v0.x surface's honest boundary
// ---------------------------------------------------------------------------

#[test]
fn the_plan_bridge_reports_the_capability_note() {
    // The v0.x Atom set carries width bounds only — no capability
    // slot: the layer's style does not land, the typed note names the
    // capability (the preference families' own note).
    let sheet = emit_plan("pointer-fine:p-8", &Config::default()).expect("emits");
    let note = sheet
        .notes
        .iter()
        .find(|note| matches!(note.kind, PlanNoteKind::MediaFeature { .. }))
        .expect("the media-feature note rides");
    match &note.kind {
        PlanNoteKind::MediaFeature { pref } => {
            assert_eq!(*pref, Pref::PointerFine);
        }
        other => panic!("the media-feature note, not {other:?}"),
    }
    assert!(
        note.to_string().contains("reports instead of landing"),
        "the note reads for humans: {note}"
    );
}

#[test]
fn the_plan_bridge_reports_the_orientation_note() {
    let sheet = emit_plan("landscape:p-8", &Config::default()).expect("emits");
    let note = sheet
        .notes
        .iter()
        .find(|note| matches!(note.kind, PlanNoteKind::MediaFeature { .. }))
        .expect("the media-feature note rides");
    match &note.kind {
        PlanNoteKind::MediaFeature { pref } => {
            assert_eq!(*pref, Pref::Landscape);
        }
        other => panic!("the media-feature note, not {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// The metamorphic triangle: compile == CSS == interp under the facts
// ---------------------------------------------------------------------------

#[test]
fn the_metamorphic_triangle_agrees_on_a_capability_program() {
    // The conjunctive checker bakes the capability facts into the
    // situation envs it walks (the frame-state tier, the preference
    // families' own rule): a capability program agrees across the
    // three walks — compile == CSS == interp.
    let report = aeolus_compile::metamorphic_check(
        "pointer-fine:p-8 portrait:p-4 any-pointer-coarse:hidden",
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
