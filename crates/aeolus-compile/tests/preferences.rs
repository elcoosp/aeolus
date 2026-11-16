//! The preference-variant interp surface (the `motion-*`/`contrast-*`/
//! `forced-colors:`/`print:` families — the media-feature tier beside
//! the breakpoint forms). The seam:
//!
//! * [`Cond::Media`] — the typed condition whose payload is the
//!   [`Pref`] bit (`motion-safe:` → `Pref::MotionSafe`, `print:` →
//!   `Pref::Print`, ...), one arm per family spelling;
//! * the [`Env`] facts — the preference bits are FRAME STATE (the user
//!   agent's media-query answers, exactly the tier `dark:` rides), so
//!   the plain walk answers them from the [`Env`] with no probe seam
//!   and no sourceless note: an unset bit is a QUIET miss (a media
//!   query that does not hold), never an exclusion;
//! * [`CssAt::MediaPref`] — the at-rule head, the real tailwindcss
//!   v4.1.17 texts (`staticVariant` calls, pinned from the source:
//!   `@media (prefers-reduced-motion: reduce)`,
//!   `@media (prefers-reduced-motion: no-preference)`,
//!   `@media (prefers-contrast: more)`, `@media (prefers-contrast:
//!   less)`, `@media (forced-colors: active)`, `@media print`);
//! * the plan bridge — the v0.x surface has no media-feature slot (the
//!   `Atom::Viewport` form carries width bounds only), so the layer
//!   reports through the dedicated [`PlanNoteKind::MediaFeature`] note,
//!   the same boundary the supports conditions keep.
//!
//! Written FIRST (the RED spec): the suite fails against the
//! post-supports seam — the condition, the at-rule form and the
//! [`Env`] facts do not exist yet (the fold refuses every family
//! spelling as unsupported-variant).

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
fn every_family_spelling_folds_onto_the_typed_cond() {
    let cases: &[(&str, Pref)] = &[
        ("motion-safe", Pref::MotionSafe),
        ("motion-reduce", Pref::MotionReduce),
        ("contrast-more", Pref::ContrastMore),
        ("contrast-less", Pref::ContrastLess),
        ("forced-colors", Pref::ForcedColors),
        ("print", Pref::Print),
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
fn the_condition_rides_the_media_tier() {
    // The preference forms ARE media queries: the specificity contract
    // pins them beside the breakpoint forms (tier 4), below the
    // container tier (5).
    let media = Cond::Media(Pref::MotionReduce);
    let bp = Cond::Bp(aeolus_ir::Bp::Md);
    let container = Cond::Container {
        bp: aeolus_ir::Bp::Md,
        name: None,
    };
    assert_eq!(media.specificity(), bp.specificity());
    assert!(media.specificity() < container.specificity());
}

#[test]
fn the_condition_names_its_own_spelling() {
    // The canonical variant text rides the condition (the diagnostics
    // and the metamorphic labels read it; no payload table needed —
    // the spellings are closed).
    assert_eq!(Cond::Media(Pref::MotionSafe).source(), Some("motion-safe"));
    assert_eq!(Cond::Media(Pref::Print).source(), Some("print"));
    assert_eq!(
        Cond::Media(Pref::ForcedColors).source(),
        Some("forced-colors")
    );
}

// ---------------------------------------------------------------------------
// The interp: the Env answers, the unset bit is a quiet miss
// ---------------------------------------------------------------------------

#[test]
fn the_env_fact_answers_the_layer() {
    let compiled = compile("motion-reduce:p-8", &Config::default()).expect("compiles");
    let env = Env {
        reduced_motion: true,
        ..Env::default()
    };
    let interped = aeolus_compile::interp(&compiled.program, &env).expect("no theme forms");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));
    assert!(interped.notes.is_empty(), "a fire is not a note");
}

#[test]
fn the_unset_fact_is_a_quiet_miss() {
    // The default Env answers every preference question (all false —
    // the media queries do not hold): the layer misses QUIETLY, the
    // CSS semantics a media query obeys, never an exclusion note.
    let compiled = compile("motion-reduce:p-8", &Config::default()).expect("compiles");
    let interped =
        aeolus_compile::interp(&compiled.program, &Env::default()).expect("no theme forms");
    assert!(interped.decls.is_empty(), "{:?}", interped.decls);
    assert!(interped.notes.is_empty(), "{:?}", interped.notes);
}

#[test]
fn motion_safe_asks_the_complement() {
    // `motion-safe:` fires when the user has NO reduced-motion
    // preference — the same Env bit, the complement question.
    let compiled = compile("motion-safe:p-8", &Config::default()).expect("compiles");
    let resting = Env {
        reduced_motion: false,
        ..Env::default()
    };
    let interped = aeolus_compile::interp(&compiled.program, &resting).expect("no theme forms");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));

    let reducing = Env {
        reduced_motion: true,
        ..Env::default()
    };
    let interped = aeolus_compile::interp(&compiled.program, &reducing).expect("no theme forms");
    assert!(
        interped.decls.is_empty(),
        "the reduce user fails no-preference"
    );
}

#[test]
fn every_family_asks_its_own_fact() {
    // One bit per family question: contrast-more/less are two answers
    // of one feature, forced-colors and print carry their own.
    let cases: &[(&str, Box<dyn Fn(&mut Env) -> bool>)] = &[
        (
            "contrast-more",
            Box::new(|e: &mut Env| {
                e.more_contrast = true;
                true
            }),
        ),
        (
            "contrast-less",
            Box::new(|e: &mut Env| {
                e.less_contrast = true;
                true
            }),
        ),
        (
            "forced-colors",
            Box::new(|e: &mut Env| {
                e.forced_colors = true;
                true
            }),
        ),
        (
            "print",
            Box::new(|e: &mut Env| {
                e.print_media = true;
                true
            }),
        ),
    ];
    for (spelling, set) in cases {
        let compiled = compile(&format!("{spelling}:p-8"), &Config::default()).expect("compiles");
        let mut env = Env::default();
        assert!(set(&mut env));
        let interped = aeolus_compile::interp(&compiled.program, &env).expect("no theme forms");
        assert_eq!(
            resolved(&interped, Prop::PaddingTop),
            Val::Rem(2.0),
            "{spelling} answers from its own fact"
        );
    }
}

// ---------------------------------------------------------------------------
// The CSS arm: the corpus's at-rule texts, pinned
// ---------------------------------------------------------------------------

#[test]
fn the_css_arm_renders_the_corpus_texts() {
    let cases: &[(&str, &str)] = &[
        (
            "motion-safe",
            "@media (prefers-reduced-motion: no-preference) {",
        ),
        ("motion-reduce", "@media (prefers-reduced-motion: reduce) {"),
        ("contrast-more", "@media (prefers-contrast: more) {"),
        ("contrast-less", "@media (prefers-contrast: less) {"),
        ("forced-colors", "@media (forced-colors: active) {"),
        ("print", "@media print {"),
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
fn the_sheet_groups_one_at_rule_block() {
    // Consecutive rules sharing one at-rule group into a single block
    // (the CssSheet Display contract, the media forms' own rule).
    let sheet = emit_css("motion-reduce:p-2 motion-reduce:p-4", &Config::default()).expect("emits");
    let rendered = sheet.to_string();
    assert_eq!(
        rendered
            .matches("@media (prefers-reduced-motion: reduce)")
            .count(),
        1
    );
    // The later rule wins the cascade inside the block.
    assert!(rendered.contains("padding-top: 1rem"), "{rendered}");
    let rules: Vec<CssRule> = sheet.rules.clone();
    assert_eq!(rules.len(), 2, "one rule per candidate, source order");
}

// ---------------------------------------------------------------------------
// The plan bridge: the v0.x surface's honest boundary
// ---------------------------------------------------------------------------

#[test]
fn the_plan_bridge_reports_the_media_feature_note() {
    // The v0.x Atom set carries width bounds only — no preference
    // slot: the layer's style does not land, the typed note names the
    // preference (the supports-condition boundary's sibling).
    let sheet = emit_plan("motion-reduce:p-8", &Config::default()).expect("emits");
    let note = sheet
        .notes
        .iter()
        .find(|note| matches!(note.kind, PlanNoteKind::MediaFeature { .. }))
        .expect("the media-feature note rides");
    match &note.kind {
        PlanNoteKind::MediaFeature { pref } => {
            assert_eq!(*pref, Pref::MotionReduce);
        }
        other => panic!("the media-feature note, not {other:?}"),
    }
    assert!(
        note.to_string().contains("reports instead of landing"),
        "the note reads for humans: {note}"
    );
    let plain = emit_plan("p-8", &Config::default()).expect("emits");
    assert!(plain.notes.is_empty(), "{:?}", plain.notes);
}
