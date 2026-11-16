//! The direction variants' interp surface (`ltr:`/`rtl:` — the
//! text-direction family, tailwindcss v4.1.17's own static variants,
//! pinned verbatim from the source: `packages/tailwindcss/src/variants.ts`
//! `staticVariant('ltr', ['&:where(:dir(ltr), [dir="ltr"], [dir="ltr"]
//! *)'])` and the `rtl` sibling beside it). The seam:
//!
//! * [`Cond::Dir`] — the one typed condition the two spellings fold to,
//!   the payload the [`Direction`] bit (`ltr:` → `Direction::Ltr`,
//!   `rtl:` → `Direction::Rtl`), one arm per family spelling;
//! * the [`Env`] fact — the direction is FRAME STATE (the element's
//!   resolved `direction`, the tier the capability and preference bits
//!   ride): the [`Env`] carries ONE fact (`direction_rtl`) and the two
//!   spellings answer from it — `rtl:` on the bit, `ltr:` its
//!   COMPLEMENT (the feature's two v4 values share one fact, the
//!   `portrait:`/`landscape:` precedent — and CSS's own initial
//!   `direction` value is `ltr`, so the resting env honestly answers
//!   `ltr:`), an unset fact the QUIET miss a selector that does not
//!   match is, never an exclusion note;
//! * the CSS head — the selector-side probe (the family is NOT an
//!   at-rule: no `@media` head renders), the upstream `:where(...)`
//!   texts verbatim;
//! * the plan bridge — the v0.x surface has no direction slot, so the
//!   layer reports through the new [`PlanNoteKind::Direction`] note, the
//!   media-feature boundary's sibling.
//!
//! Written FIRST (the RED spec): the suite fails against the
//! post-capability-tier tree — `Direction`, [`Cond::Dir`], the
//! [`Env::direction_rtl`] fact and the note do not exist yet (the fold
//! refuses every direction spelling as unsupported-variant).

use aeolus_compile::{Config, Env, PlanNoteKind, compile, emit_css, emit_plan};
use aeolus_ir::{Cond, Direction, Prop, Val};

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
fn every_direction_spelling_folds_onto_the_typed_cond() {
    let cases: &[(&str, Direction)] = &[("ltr", Direction::Ltr), ("rtl", Direction::Rtl)];
    for (spelling, dir) in cases {
        let compiled = compile(&format!("{spelling}:p-8"), &Config::default()).expect("compiles");
        let cond = compiled
            .program
            .layers
            .iter()
            .find_map(|layer| match layer.cond {
                Cond::Dir(inner) => Some(inner),
                _ => None,
            })
            .unwrap_or_else(|| panic!("{spelling} folds onto Cond::Dir"));
        assert_eq!(cond, *dir, "{spelling} folds to the right bit");
    }
}

#[test]
fn the_direction_condition_rides_the_probe_tier() {
    // The direction test IS a selector conjunct (`:where(:dir(rtl), …)`),
    // not an at-rule: the specificity contract pins it on the element
    // -own probe tier (beside `hover:` and the attribute probes), below
    // the media tier the breakpoint and capability forms ride.
    let rtl = Cond::Dir(Direction::Rtl);
    assert_eq!(rtl.specificity(), Cond::Hover.specificity());
    assert!(rtl.specificity() < Cond::Media(aeolus_ir::Pref::Print).specificity());
}

#[test]
fn the_direction_condition_names_its_own_spelling() {
    // The canonical variant text rides the condition (the diagnostics
    // and the metamorphic labels read it; no payload table needed —
    // the spellings are closed).
    assert_eq!(Cond::Dir(Direction::Ltr).source(), Some("ltr"));
    assert_eq!(Cond::Dir(Direction::Rtl).source(), Some("rtl"));
}

// ---------------------------------------------------------------------------
// The interp: the Env fact answers, the unset bit is a quiet miss
// ---------------------------------------------------------------------------

#[test]
fn the_rtl_fact_answers_the_layer() {
    let compiled = compile("rtl:p-8", &Config::default()).expect("compiles");
    let env = Env {
        direction_rtl: true,
        ..Env::default()
    };
    let interped = aeolus_compile::interp(&compiled.program, &env).expect("no theme forms");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));
    assert!(interped.notes.is_empty(), "a fire is not a note");
}

#[test]
fn the_unset_direction_fact_is_a_quiet_miss() {
    // The default Env asserts no rtl fact (the resolved direction is
    // not rtl): the layer misses QUIETLY, the CSS semantics a selector
    // that does not match obeys, never an exclusion note.
    let compiled = compile("rtl:p-8", &Config::default()).expect("compiles");
    let interped =
        aeolus_compile::interp(&compiled.program, &Env::default()).expect("no theme forms");
    assert!(interped.decls.is_empty(), "{:?}", interped.decls);
    assert!(interped.notes.is_empty(), "{:?}", interped.notes);
}

#[test]
fn ltr_answers_the_complement() {
    // `ltr:` fires when the resolved direction is NOT rtl — the same
    // Env fact, the complement question (the feature's two v4 values
    // share one fact, the `portrait:`/`landscape:` precedent — and
    // CSS's own initial `direction` value IS ltr, so the resting env
    // answers the ltr query the way the orientation query answers
    // `landscape:`).
    let compiled = compile("ltr:p-8", &Config::default()).expect("compiles");
    let interped =
        aeolus_compile::interp(&compiled.program, &Env::default()).expect("no theme forms");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));
    assert!(interped.notes.is_empty(), "a fire is not a note");

    // With the rtl fact asserted, the ltr layer misses and the rtl
    // layer fires — one fact, both spellings answer.
    let interped = aeolus_compile::interp(
        &compile("ltr:p-8", &Config::default())
            .expect("compiles")
            .program,
        &Env {
            direction_rtl: true,
            ..Env::default()
        },
    )
    .expect("no theme forms");
    assert!(
        interped.decls.is_empty(),
        "the rtl direction fails the ltr query"
    );
}

// ---------------------------------------------------------------------------
// The CSS arm: the upstream selector texts, pinned — no at-rule head
// ---------------------------------------------------------------------------

#[test]
fn the_css_arm_renders_the_upstream_selectors() {
    // The texts are variants.ts's own `staticVariant` selectors
    // (v4.1.17), with the `&` the utility selector replaces.
    let cases: &[(&str, &str)] = &[
        ("rtl", ":where(:dir(rtl), [dir=\"rtl\"], [dir=\"rtl\"] *)"),
        ("ltr", ":where(:dir(ltr), [dir=\"ltr\"], [dir=\"ltr\"] *)"),
    ];
    for (spelling, head) in cases {
        let sheet = emit_css(&format!("{spelling}:p-8"), &Config::default()).expect("emits");
        assert!(sheet.diagnostics.is_empty(), "{:?}", sheet.diagnostics);
        let rendered = sheet.to_string();
        assert!(
            rendered.contains(head),
            "{spelling} renders the v4.1.17 selector: {rendered}"
        );
    }
}

#[test]
fn the_direction_head_is_selector_side() {
    // The family is NOT an at-rule: no `@media`/`@container`/`@supports`
    // head renders (the direction test rides the element's own selector,
    // the probe tier the hover/attribute forms occupy) — the
    // selector-side boundary the probe tier keeps.
    for spelling in ["rtl", "ltr"] {
        let sheet = emit_css(&format!("{spelling}:p-8"), &Config::default()).expect("emits");
        let rendered = sheet.to_string();
        assert!(
            !rendered.contains("@media"),
            "{spelling} renders no at-rule head: {rendered}"
        );
        assert!(rendered.contains(":dir("), "{rendered}");
    }
}

// ---------------------------------------------------------------------------
// The plan bridge: the v0.x surface's honest boundary
// ---------------------------------------------------------------------------

#[test]
fn the_plan_bridge_reports_the_direction_note() {
    // The v0.x Atom set carries width bounds and state probes only — no
    // direction slot: the layer's style does not land, the typed note
    // names the direction (the media-feature note's sibling).
    for (spelling, dir) in [("rtl", Direction::Rtl), ("ltr", Direction::Ltr)] {
        let sheet = emit_plan(&format!("{spelling}:p-8"), &Config::default()).expect("emits");
        let note = sheet
            .notes
            .iter()
            .find(|note| matches!(note.kind, PlanNoteKind::Direction { .. }))
            .unwrap_or_else(|| panic!("the direction note rides for {spelling}"));
        match &note.kind {
            PlanNoteKind::Direction { cond } => {
                assert_eq!(*cond, dir);
            }
            other => panic!("the direction note, not {other:?}"),
        }
        assert!(
            note.to_string().contains("reports instead of landing"),
            "the note reads for humans: {note}"
        );
    }
}

// ---------------------------------------------------------------------------
// The metamorphic triangle: compile == CSS == interp under the fact
// ---------------------------------------------------------------------------

#[test]
fn the_metamorphic_triangle_agrees_on_a_direction_program() {
    // The conjunctive checker bakes the direction fact into the
    // situation envs it walks (the frame-state tier, the capability
    // families' own rule): a direction program agrees across the three
    // walks — compile == CSS == interp.
    let report =
        aeolus_compile::metamorphic_check("rtl:p-8 ltr:p-4", &Config::default()).expect("compiles");
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
