//! The pseudo-element RENDER TARGET tier (the interp arm's
//! [`PseudoTarget`] seam grown to the whole v4.1.17 pseudo-element
//! variant set, tailwindcss v4.1.17's `packages/tailwindcss/src/variants.ts`
//! pinned verbatim: `staticVariant('first-letter', ['&::first-letter'])`,
//! `staticVariant('first-line', ['&::first-line'])`,
//! `staticVariant('marker', ['& *::marker', '&::marker',
//! '& *::-webkit-details-marker', '&::-webkit-details-marker'])`,
//! `staticVariant('file', ['&::file-selector-button'])`,
//! `staticVariant('backdrop', ['&::backdrop'])`,
//! `staticVariant('details-content', ['&::details-content'])`, and the
//! two functional bodies `before`/`after` — each builds
//! `styleRule('&::before'/'&::after', [decl('content',
//! 'var(--tw-content)'), …nodes])` with the at-root
//! `@property --tw-content` registration beside it). The seam:
//!
//! * the eight typed conditions — [`Cond::Before`]/[`Cond::After`]/[`Cond::Marker`]/
//!   [`Cond::FirstLetter`]/[`Cond::FirstLine`]/[`Cond::FileSelector`]/
//!   [`Cond::Backdrop`]/[`Cond::DetailsContent`] — one bare variant per
//!   spelling, riding the element-own probe tier BESIDE
//!   [`Cond::Selection`]/[`Cond::Placeholder`] (a pseudo-element head is
//!   a selector conjunct on the `&`, never an at-rule);
//! * the render targets — [`PseudoTarget`] grows the eight arms beside
//!   the two it carried, plus the closed-set surface (`ALL`, `from_name`)
//!   the diagnostics and consumers key on;
//! * the walks — the element cascade excludes every pseudo layer with
//!   the [`InterpNote::PseudoElement`] note (the render-target
//!   contract), the pseudo cascades resolve through `interp_pseudo` /
//!   `interp_pseudo_theme` per target;
//! * the CSS arm — the `::` heads render the upstream selector texts;
//!   `marker:` renders the FOUR-selector disjunction upstream registers;
//!   `before:`/`after:` inject the upstream body's `content:
//!   var(--tw-content)` line ahead of the utility's own declarations
//!   (the per-rule `""` fallback standing in for the at-root
//!   `@property --tw-content` registration the per-candidate emitter
//!   cannot carry — the documented divergence);
//! * the plan bridge — `file:` LANDS on the v0.x surface it already
//!   carries (`PseudoElement::FileSelectorButton`); the other seven
//!   report the typed `PlanNoteKind::PseudoElement` boundary (no v0.x
//!   slot — visible, never silent).
//!
//! Written FIRST (the RED spec): the suite fails against the
//! post-form-state tree — every spelling refuses as unsupported-variant
//! and the [`PseudoTarget`] set carries two arms.

use aeolus_compile::{
    Config, Env, InterpNote, PlanNoteKind, PseudoTarget, compile, compile_theme, emit_css,
    emit_plan, interp, interp_pseudo, interp_pseudo_theme,
};
use aeolus_ir::{Cond, Prop, Val};
use aeolus_theme::{ThemeSource, TokenTable};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// The value one longhand resolved to.
fn resolved(interp: &aeolus_compile::Interped, prop: Prop) -> Val {
    interp
        .decls
        .iter()
        .find(|d| d.prop == prop)
        .unwrap_or_else(|| panic!("{prop:?} is in the resolved declarations"))
        .val
}

/// The runtime-tier table over the shadcn source (the themed pins).
fn runtime_table() -> TokenTable {
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    TokenTable::runtime(&source).expect("default-seeded source builds")
}

// ---------------------------------------------------------------------------
// The fold: one arm per family spelling
// ---------------------------------------------------------------------------

#[test]
fn every_pseudo_element_spelling_folds_onto_the_typed_cond() {
    let cases: &[(&str, Cond)] = &[
        ("before", Cond::Before),
        ("after", Cond::After),
        ("marker", Cond::Marker),
        ("first-letter", Cond::FirstLetter),
        ("first-line", Cond::FirstLine),
        ("file", Cond::FileSelector),
        ("backdrop", Cond::Backdrop),
        ("details-content", Cond::DetailsContent),
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
            compiled.program.layers.len(),
            1,
            "{spelling}: exactly the one pseudo layer"
        );
        assert_eq!(
            compiled.program.layers[0].cond, *expected,
            "{spelling}: the typed condition"
        );
    }
}

#[test]
fn pseudo_element_conditions_ride_the_probe_tier() {
    // A pseudo-element head is a selector conjunct on the `&` — the
    // tier `hover:`/`checked:`/`selection:` ride, never an at-rule.
    for cond in [
        Cond::Before,
        Cond::After,
        Cond::Marker,
        Cond::FirstLetter,
        Cond::FirstLine,
        Cond::FileSelector,
        Cond::Backdrop,
        Cond::DetailsContent,
    ] {
        assert_eq!(cond.specificity(), 2, "{cond:?}: the probe tier");
        assert!(
            cond.source().is_some(),
            "{cond:?}: the spelling reconstructs"
        );
    }
}

// ---------------------------------------------------------------------------
// The render-target surface: the closed set
// ---------------------------------------------------------------------------

#[test]
fn pseudo_targets_carry_the_whole_upstream_set() {
    assert_eq!(
        PseudoTarget::ALL,
        &[
            PseudoTarget::Selection,
            PseudoTarget::Placeholder,
            PseudoTarget::Before,
            PseudoTarget::After,
            PseudoTarget::Marker,
            PseudoTarget::FirstLetter,
            PseudoTarget::FirstLine,
            PseudoTarget::FileSelector,
            PseudoTarget::Backdrop,
            PseudoTarget::DetailsContent,
        ],
        "the closed spelling set, upstream registration order"
    );
}

#[test]
fn pseudo_target_from_name_round_trips_the_spellings() {
    for target in PseudoTarget::ALL {
        let name = PseudoTarget::from_name(target.spelling())
            .unwrap_or_else(|| panic!("{}: the spelling parses", target.spelling()));
        assert_eq!(name, *target, "{}: the round trip", target.spelling());
    }
    assert_eq!(
        PseudoTarget::from_name("selection"),
        Some(PseudoTarget::Selection)
    );
    assert_eq!(PseudoTarget::from_name("no-such-target"), None);
    assert_eq!(PseudoTarget::from_name(""), None);
}

// ---------------------------------------------------------------------------
// The walks: element exclusion, pseudo resolution
// ---------------------------------------------------------------------------

#[test]
fn element_walk_excludes_the_pseudo_layers_with_the_note() {
    let compiled = compile("before:p-8 p-4", &Config::default()).expect("the string lowers");
    assert!(compiled.diagnostics.is_empty());
    let interped = interp(&compiled.program, &Env::default()).expect("the walk resolves");
    // The base layer answers; the pseudo layer is the OTHER render
    // target — excluded and reported, never folded in.
    assert_eq!(
        resolved(&interped, Prop::PaddingTop),
        Val::Rem(1.0),
        "the base cascade only"
    );
    let pseudo_note = interped
        .notes
        .iter()
        .find(|note| matches!(note, InterpNote::PseudoElement { .. }))
        .unwrap_or_else(|| panic!("the pseudo layer reports"));
    match pseudo_note {
        InterpNote::PseudoElement { cond, .. } => assert_eq!(*cond, Cond::Before),
        other => panic!("unexpected note: {other:?}"),
    }
}

#[test]
fn every_target_resolves_its_own_cascade() {
    let spellings = [
        "before",
        "after",
        "marker",
        "first-letter",
        "first-line",
        "file",
        "backdrop",
        "details-content",
    ];
    for spelling in spellings {
        let compiled = compile(&format!("{spelling}:p-8"), &Config::default())
            .unwrap_or_else(|e| panic!("{spelling}: lowers ({e})"));
        let target = PseudoTarget::from_name(spelling).expect("the closed set names it");
        let interped = interp_pseudo(&compiled.program, target)
            .unwrap_or_else(|e| panic!("{spelling}: the pseudo walk resolves ({e})"));
        assert_eq!(
            resolved(&interped, Prop::PaddingTop),
            Val::Rem(2.0),
            "{spelling}: the target's own value"
        );
    }
}

#[test]
fn the_pseudo_walk_stays_silent_about_the_element_cascade() {
    let compiled =
        compile("backdrop:p-8 hover:p-4", &Config::default()).expect("the string lowers");
    let interped =
        interp_pseudo(&compiled.program, PseudoTarget::Backdrop).expect("the walk resolves");
    assert_eq!(
        resolved(&interped, Prop::PaddingTop),
        Val::Rem(2.0),
        "only the target's layers participate"
    );
    assert!(
        interped.notes.is_empty(),
        "the element's own layers are a different target, not notes: {:?}",
        interped.notes
    );
}

#[test]
fn themed_pseudo_walk_resolves_the_token_forms() {
    let table = runtime_table();
    let env = Env::default();
    let compiled = compile_theme("first-letter:text-primary", &table, &Config::default())
        .expect("the themed string lowers");
    let interped = interp_pseudo_theme(&compiled.program, &table, &env, PseudoTarget::FirstLetter)
        .expect("the themed walk resolves");
    assert_eq!(interped.decls.len(), 1, "the colour longhand resolved");
    assert!(
        !matches!(interped.decls[0].val, Val::Tok { .. } | Val::ModePair(_)),
        "the token baked to a concrete value: {:?}",
        interped.decls[0].val
    );
}

// ---------------------------------------------------------------------------
// The CSS arm: the upstream selector texts
// ---------------------------------------------------------------------------

#[test]
fn css_arm_renders_the_single_head_spellings() {
    let heads: &[(&str, &str)] = &[
        ("before", "::before"),
        ("after", "::after"),
        ("first-letter", "::first-letter"),
        ("first-line", "::first-line"),
        ("file", "::file-selector-button"),
        ("backdrop", "::backdrop"),
        ("details-content", "::details-content"),
    ];
    for (spelling, head) in heads {
        let sheet = emit_css(&format!("{spelling}:p-8"), &Config::default())
            .unwrap_or_else(|e| panic!("{spelling}: renders ({e})"));
        assert!(sheet.diagnostics.is_empty(), "{spelling}: clean emit");
        assert_eq!(sheet.rules.len(), 1, "{spelling}: one rule");
        assert!(
            sheet.rules[0].selector.ends_with(head),
            "{spelling}: the upstream head, got {}",
            sheet.rules[0].selector
        );
    }
}

#[test]
fn css_arm_renders_the_marker_selector_disjunction() {
    // Upstream registers FOUR selectors on one rule (`& *::marker`,
    // `&::marker`, the -webkit- details-marker pair) — the emitter
    // renders the same comma-joined list, the body carried once.
    let sheet = emit_css("marker:p-8", &Config::default()).expect("the string renders");
    assert!(sheet.diagnostics.is_empty());
    assert_eq!(sheet.rules.len(), 1, "one rule, four selectors");
    assert_eq!(
        sheet.rules[0].selector,
        ".marker\\:p-8 *::marker, .marker\\:p-8::marker, \
         .marker\\:p-8 *::-webkit-details-marker, .marker\\:p-8::-webkit-details-marker",
        "the upstream selector list, verbatim"
    );
    assert!(
        sheet.rules[0].body.contains("padding-top"),
        "{:?}",
        sheet.rules[0].body
    );
}

#[test]
fn before_and_after_inject_the_content_line_first() {
    // Upstream's functional body: `decl('content', 'var(--tw-content)')
    // ahead of `...v.nodes`. The per-rule emitter stands in the at-root
    // `@property --tw-content` registration with the `""` fallback (the
    // documented divergence).
    let sheet = emit_css("before:p-8", &Config::default()).expect("the string renders");
    assert_eq!(sheet.rules.len(), 1);
    assert!(
        sheet.rules[0]
            .body
            .starts_with("content: var(--tw-content, \"\");"),
        "the injected line rides first: {:?}",
        sheet.rules[0].body
    );
    assert!(
        sheet.rules[0].body.contains("padding-top"),
        "the utility's own declarations follow: {:?}",
        sheet.rules[0].body
    );

    let sheet = emit_css("after:p-8", &Config::default()).expect("the string renders");
    assert_eq!(sheet.rules.len(), 1);
    assert!(
        sheet.rules[0]
            .body
            .starts_with("content: var(--tw-content, \"\");"),
        "after: carries the same body contract: {:?}",
        sheet.rules[0].body
    );
}

#[test]
fn content_utility_composes_with_the_injected_line() {
    // `before:content-["hello"]` renders the var() fallback AND the
    // utility's own value — the later declaration wins the cascade, the
    // same reading upstream's `--tw-content` plumbing produces.
    let sheet =
        emit_css("before:content-[\"hello\"]", &Config::default()).expect("the string renders");
    assert_eq!(sheet.rules.len(), 1);
    assert!(
        sheet.rules[0].body.contains("hello"),
        "the utility's value renders: {:?}",
        sheet.rules[0].body
    );
}

// ---------------------------------------------------------------------------
// The plan bridge: the v0.x boundary
// ---------------------------------------------------------------------------

#[test]
fn plan_bridge_lands_the_file_selector_on_the_v0_surface() {
    // The v0.x `PseudoElement` set already carries the file-selector
    // button — the layer LANDS its `Test::Pseudo` slot (the
    // selection:/placeholder: precedent).
    let sheet = emit_plan("file:p-8", &Config::default()).expect("the string lowers");
    assert!(sheet.diagnostics.is_empty());
    let lands = sheet
        .plan
        .rules
        .iter()
        .any(|rule| format!("{:?}", rule.when).contains("FileSelectorButton"));
    assert!(
        lands,
        "the file-selector layer lands, got {:?}",
        sheet.plan.rules
    );
}

#[test]
fn plan_bridge_reports_the_pseudo_element_boundary() {
    // The other seven spellings have no v0.x slot (the v0.x
    // `PseudoElement` set carries selection/placeholder/file only) —
    // the typed note keeps the class visible, never a silent drop.
    for spelling in [
        "before",
        "after",
        "marker",
        "first-letter",
        "first-line",
        "backdrop",
        "details-content",
    ] {
        let sheet = emit_plan(&format!("{spelling}:p-8"), &Config::default())
            .unwrap_or_else(|e| panic!("{spelling}: lowers ({e})"));
        let note = sheet
            .notes
            .iter()
            .find(|note| matches!(note.kind, PlanNoteKind::PseudoElement { .. }))
            .unwrap_or_else(|| panic!("{spelling}: the boundary note rides"));
        assert_eq!(note.raw, format!("{spelling}:p-8"));
        assert_eq!(
            note.kind.code(),
            "aeolus::compile::plan::pseudo_element",
            "{spelling}: the stable code"
        );
    }
}

// ---------------------------------------------------------------------------
// The stacked spellings compose
// ---------------------------------------------------------------------------

#[test]
fn stacked_pseudo_and_state_variants_fold_flat() {
    // `hover:before:` — the flat fold lands the DOMINANT condition (the
    // pseudo-element renders under the hovered element's rule); the
    // CSS head carries both conjuncts.
    let sheet = emit_css("hover:before:p-8", &Config::default()).expect("the string renders");
    assert_eq!(sheet.rules.len(), 1);
    assert!(
        sheet.rules[0].selector.contains(":hover") && sheet.rules[0].selector.ends_with("::before"),
        "the conjunctive head: {}",
        sheet.rules[0].selector
    );
}
