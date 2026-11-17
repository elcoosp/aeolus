//! The of-type structural tier (the structural family's type-relative
//! slice — plan §3.5's structural row's next half). The five v4.1.17
//! spellings `first-of-type:`/`last-of-type:`/`only-of-type:`,
//! `nth-of-type-N:` and `nth-last-of-type-N:` (the upstream
//! `variants.ts` `staticVariant('first-of-type', ['&:first-of-type'])`
//! trio plus the `functional('nth-of-type', …)` /
//! `functional('nth-last-of-type', …)` pair) fold onto the SAME typed
//! [`Cond::Structural`] surface the child-index family rides — the
//! [`Structural`] enum grows the five tests, the [`StructFacts`] seam
//! value grows the TYPE-relative facts (the element's position among
//! its same-type siblings — what "same type" means is the host's own
//! reading, the seam carries the answer, never a tag string), and the
//! interp/plan-bridge/CSS/static arms answer from the same verdict fn.
//!
//! Written FIRST (the RED spec): these pins compile-fail against the
//! un-landed bits (`E0599` on the `Structural` arms, the fold's
//! unknown-variant refusals) and go green with the tier.

use aeolus_compile::{
    Config, Env, InterpNote, Interped, StructFacts, compile, emit_css, emit_plan_theme,
    emit_static, interp, interp_struct,
};
use aeolus_core::{Resolver, TAILWIND_DEFAULT_THEME, TailwindTheme};
use aeolus_ir::{Cond, Prop, Structural, Val};
use aeolus_theme::{ThemeSource, TokenTable};

// ---------------------------------------------------------------------------
// Shared helpers (the structural battery's own shapes)
// ---------------------------------------------------------------------------

/// Compile one class string and expect the walk to refuse nothing.
fn ok(class: &str) -> aeolus_compile::Compiled {
    compile(class, &Config::default()).unwrap_or_else(|e| panic!("{class}: {e:?}"))
}

/// The one longhand the interped walk must have resolved.
fn resolved(interped: &Interped, prop: Prop) -> Val {
    interped
        .decls
        .iter()
        .find(|d| d.prop == prop)
        .map(|d| d.val)
        .unwrap_or_else(|| panic!("no {prop:?} declaration in {:?}", interped.decls))
}

/// The exclusion note for one condition, if the walk reported it.
fn note_for<'a>(interped: &'a Interped, cond: &Cond) -> Option<&'a InterpNote> {
    interped.notes.iter().find(|note| match note {
        InterpNote::StructuralUnqueried { cond: c, .. } => c == cond,
        _ => false,
    })
}

/// The facts for one element that is the FIRST of its type but the
/// MIDDLE child overall: three siblings, two of them the element's
/// type, the element the second child and the first of its type.
fn first_of_type_middle_child() -> StructFacts {
    StructFacts {
        index: Some(1),
        sibling_count: Some(3),
        type_index: Some(0),
        type_sibling_count: Some(2),
        empty: false,
    }
}

/// The facts for one element that is the LAST of its type (the second
/// of two same-type siblings, the final child overall).
fn last_of_type_last_child() -> StructFacts {
    StructFacts {
        index: Some(2),
        sibling_count: Some(3),
        type_index: Some(1),
        type_sibling_count: Some(2),
        empty: false,
    }
}

// ---------------------------------------------------------------------------
// The fold: the spellings → the typed tests
// ---------------------------------------------------------------------------

#[test]
fn the_of_type_spellings_fold_onto_the_structural_tests() {
    let cases = [
        ("first-of-type:w-64", Structural::FirstOfType),
        ("last-of-type:w-64", Structural::LastOfType),
        ("only-of-type:w-64", Structural::OnlyOfType),
        ("nth-of-type-3:w-64", Structural::NthOfType(3)),
        ("nth-last-of-type-2:w-64", Structural::NthLastOfType(2)),
    ];
    for (class, test) in cases {
        let compiled = ok(class);
        assert!(
            compiled
                .program
                .layers
                .iter()
                .any(|layer| layer.cond == Cond::Structural(test)),
            "{class} folds to Cond::Structural({test:?})"
        );
    }
}

#[test]
fn the_fold_order_keeps_nth_last_of_type_out_of_the_nth_arms() {
    // The shared prefixes route by LONGEST first: `nth-last-of-type-2`
    // must reach its own arm, never `nth-last-` (which would parse-fail
    // `-of-type-2`) and never `nth-` (which would parse-fail
    // `-of-type-2`).
    let compiled = ok("nth-last-of-type-2:w-64");
    assert!(
        compiled
            .program
            .layers
            .iter()
            .any(|layer| layer.cond == Cond::Structural(Structural::NthLastOfType(2))),
        "the longest prefix wins"
    );
    // The bracketed selector forms stay outside (the `nth-` arm's own
    // refusal, the family's documented boundary): the typed
    // unsupported-variant diagnostic rides, no layer lowers.
    let refused =
        compile("nth-of-type-[2n+1]:w-64", &Config::default()).expect("total input compiles");
    assert!(
        !refused.diagnostics.is_empty(),
        "the selector form carries the typed unsupported-variant diagnostic"
    );
    assert!(
        refused.program.layers.is_empty(),
        "the refused variant lowers no layers"
    );
}

// ---------------------------------------------------------------------------
// The seam: the type facts answer the of-type tests
// ---------------------------------------------------------------------------

#[test]
fn the_seam_answers_first_of_type_at_the_first_sibling_of_the_type() {
    let compiled = ok("first-of-type:w-64");
    // The first sibling OF THE TYPE: fires.
    let hot = interp_struct(
        &compiled.program,
        &Env::default(),
        &first_of_type_middle_child(),
    )
    .expect("the walk resolves");
    assert_eq!(resolved(&hot, Prop::Width), Val::Rem(16.0));
    // The last sibling of the type: QUIET — no declarations, no note.
    let cold = interp_struct(
        &compiled.program,
        &Env::default(),
        &last_of_type_last_child(),
    )
    .expect("the walk resolves");
    assert!(cold.decls.iter().all(|d| d.prop != Prop::Width));
    assert!(cold.notes.is_empty(), "a source-answered miss is CSS-quiet");
}

#[test]
fn the_type_facts_are_independent_of_the_child_index() {
    // One element, two dimensions: the MIDDLE child overall, the FIRST
    // of its type. The of-type layer fires (type index 0); `first:`
    // quietly misses (index 1) — the type-relative test reads ITS OWN
    // facts, never the child-index ones.
    let compiled = ok("first-of-type:w-64");
    let hot = interp_struct(
        &compiled.program,
        &Env::default(),
        &first_of_type_middle_child(),
    )
    .expect("the walk resolves");
    assert_eq!(resolved(&hot, Prop::Width), Val::Rem(16.0));

    let compiled = ok("first:hidden");
    let cold = interp_struct(
        &compiled.program,
        &Env::default(),
        &first_of_type_middle_child(),
    )
    .expect("the walk resolves");
    assert!(
        cold.decls.iter().all(|d| d.prop != Prop::Display),
        "first: reads the child-index facts — index 1 is not the first child"
    );
}

#[test]
fn nth_of_type_ordinals_are_one_based_and_nth_last_counts_from_the_type_end() {
    // `nth-of-type-2` is the SECOND sibling of the type (type index 1),
    // the CSS 1-based parity the child-index `nth-N` arms encode.
    let compiled = ok("nth-of-type-2:w-64");
    let facts = StructFacts {
        index: Some(3),
        sibling_count: Some(5),
        type_index: Some(1),
        type_sibling_count: Some(4),
        empty: false,
    };
    let hot = interp_struct(&compiled.program, &Env::default(), &facts).expect("the walk resolves");
    assert_eq!(resolved(&hot, Prop::Width), Val::Rem(16.0));

    // `nth-last-of-type-2` counts from the type's end: the second-from-
    // last of four same-type siblings is type index 2.
    let compiled = ok("nth-last-of-type-2:w-64");
    let facts = StructFacts {
        index: Some(1),
        sibling_count: Some(3),
        type_index: Some(2),
        type_sibling_count: Some(4),
        empty: false,
    };
    let hot = interp_struct(&compiled.program, &Env::default(), &facts).expect("the walk resolves");
    assert_eq!(resolved(&hot, Prop::Width), Val::Rem(16.0));
}

#[test]
fn last_of_type_and_only_of_type_answer_their_bounds() {
    // `last-of-type:` fires at the type's final sibling.
    let compiled = ok("last-of-type:w-64");
    let hot = interp_struct(
        &compiled.program,
        &Env::default(),
        &last_of_type_last_child(),
    )
    .expect("the walk resolves");
    assert_eq!(resolved(&hot, Prop::Width), Val::Rem(16.0));

    // `only-of-type:` needs the SINGLE same-type sibling (the `only:`
    // arm's own shape, in the type dimension).
    let compiled = ok("only-of-type:w-64");
    let only = StructFacts {
        index: Some(0),
        sibling_count: Some(3),
        type_index: Some(0),
        type_sibling_count: Some(1),
        empty: false,
    };
    let hot = interp_struct(&compiled.program, &Env::default(), &only).expect("the walk resolves");
    assert_eq!(resolved(&hot, Prop::Width), Val::Rem(16.0));
    let two_of_type = StructFacts {
        index: Some(0),
        sibling_count: Some(3),
        type_index: Some(0),
        type_sibling_count: Some(2),
        empty: false,
    };
    let cold =
        interp_struct(&compiled.program, &Env::default(), &two_of_type).expect("the walk resolves");
    assert!(
        cold.decls.iter().all(|d| d.prop != Prop::Width),
        "a second same-type sibling is not only-of-type"
    );
}

#[test]
fn the_unknown_type_facts_quietly_miss_and_the_child_facts_do_not_answer() {
    // UNKNOWN: every type-relative test quietly misses (the positional
    // selector that cannot be evaluated never matches).
    let compiled = ok("first-of-type:w-64");
    let unknown = interp_struct(&compiled.program, &Env::default(), &StructFacts::UNKNOWN)
        .expect("the walk resolves");
    assert!(unknown.decls.iter().all(|d| d.prop != Prop::Width));
    assert!(unknown.notes.is_empty(), "UNKNOWN facts are a quiet miss");

    // The child-index facts ALONE (the pre-tier hosts: `StructFacts`
    // literals that carry no type facts) quietly miss too — the
    // type-relative selector has no answer to read.
    let position_only = StructFacts::new(0, 3);
    let cold = interp_struct(&compiled.program, &Env::default(), &position_only)
        .expect("the walk resolves");
    assert!(cold.decls.iter().all(|d| d.prop != Prop::Width));
    assert!(cold.notes.is_empty(), "the built-in answered, CSS-quiet");

    // No SOURCE (the plain form): the honest exclusion, reported.
    let sourceless = interp(&compiled.program, &Env::default()).expect("the walk resolves");
    assert!(sourceless.decls.iter().all(|d| d.prop != Prop::Width));
    assert!(
        note_for(&sourceless, &Cond::Structural(Structural::FirstOfType)).is_some(),
        "the sourceless form reports StructuralUnqueried"
    );
}

// ---------------------------------------------------------------------------
// The CSS arm + the static emit
// ---------------------------------------------------------------------------

#[test]
fn the_css_arm_renders_the_of_type_selectors() {
    let sheet = emit_css(
        "first-of-type:w-64 last-of-type:w-10 only-of-type:p-4 nth-of-type-3:w-2 nth-last-of-type-2:p-2",
        &Config::default(),
    )
    .expect("the emit resolves");
    let text = sheet.to_string();
    for selector in [
        ".first-of-type\\:w-64:first-of-type",
        ".last-of-type\\:w-10:last-of-type",
        ".only-of-type\\:p-4:only-of-type",
        ".nth-of-type-3\\:w-2:nth-of-type(3)",
        ".nth-last-of-type-2\\:p-2:nth-last-of-type(2)",
    ] {
        assert!(text.contains(selector), "{selector} renders, got: {text}");
    }
}

#[test]
fn the_static_emit_renders_the_of_type_cond() {
    let compiled = ok("first-of-type:w-64");
    let text = emit_static(&compiled.program);
    assert!(
        text.contains("Cond::Structural(FirstOfType)"),
        "the static item names the structural condition, got:\n{text}"
    );
}

// ---------------------------------------------------------------------------
// The plan bridge: the differential battery
// ---------------------------------------------------------------------------

/// The of-type battery: every spelling the v0.x oracle parses
/// warning-free. `nth-last-of-type-2:` is the deliberate absence (the
/// oracle's `nth-` arm never sets the last flag — the same gap
/// `tests/structural.rs` pins for `nth-last-2:`); the divergence rides
/// its own pin below.
const BATTERY: &[&str] = &[
    "first-of-type:w-10",
    "last-of-type:w-10",
    "only-of-type:w-10",
    "nth-of-type-3:w-10",
];

/// The one plan the walk must agree with: the v0.x oracle's own.
fn oracle_plan(class: &str) -> aeolus_core::StylePlan {
    let theme =
        TailwindTheme::from_theme_css(TAILWIND_DEFAULT_THEME).unwrap_or_else(|e| panic!("{e:?}"));
    let resolver = Resolver::new(&theme);
    let resolution = resolver
        .resolve(class)
        .unwrap_or_else(|e| panic!("{class}: the oracle refused: {e:?}"));
    assert!(
        resolution.warnings.is_empty(),
        "{class}: the oracle warned {:?} — the battery entry is not oracle-clean",
        resolution.warnings
    );
    resolution.plan
}

/// The baked table over the default theme (the battery's token form).
fn baked_battery_tokens() -> TokenTable {
    let source =
        ThemeSource::from_css(TAILWIND_DEFAULT_THEME, None).unwrap_or_else(|e| panic!("{e:?}"));
    TokenTable::baked(&source).unwrap_or_else(|e| panic!("{e:?}"))
}

#[test]
fn differential_battery_agrees_with_the_v0_resolver() {
    let tokens = baked_battery_tokens();
    for class in BATTERY {
        let sheet = emit_plan_theme(class, &tokens, &Config::default())
            .unwrap_or_else(|e| panic!("{class}: the emit refused: {e:?}"));
        assert!(
            sheet.diagnostics.is_empty(),
            "{class}: V1 lowered with diagnostics {:?}",
            sheet.diagnostics
        );
        assert!(
            sheet.notes.is_empty(),
            "{class}: V1 lowered with notes {:?}",
            sheet.notes
        );
        let expected = oracle_plan(class);
        assert_eq!(
            sheet.plan.layers.len(),
            expected.layers.len(),
            "{class}: layer count disagrees"
        );
        for (v1, v0) in sheet.plan.layers.iter().zip(expected.layers.iter()) {
            assert_eq!(v1.condition, v0.condition, "{class}: condition disagrees");
        }
    }
}

/// The deliberate divergence, pinned as a record: `nth-last-of-type-2:`
/// lowers clean through the V1 pipeline (the real v4 grammar) while the
/// v0.x oracle warns (its parser's last-flag gap — the `nth-last-2:`
/// precedent's sibling, `tests/structural.rs`'s own pin).
#[test]
fn nth_last_of_type_diverges_from_the_oracle() {
    let sheet = emit_plan_theme(
        "nth-last-of-type-2:w-10",
        &baked_battery_tokens(),
        &Config::default(),
    )
    .expect("the emit refused");
    assert!(
        sheet.diagnostics.is_empty(),
        "V1 lowers the v4 spelling: {:?}",
        sheet.diagnostics
    );
    assert!(sheet.notes.is_empty(), "notes {:?}", sheet.notes);
    let theme =
        TailwindTheme::from_theme_css(TAILWIND_DEFAULT_THEME).unwrap_or_else(|e| panic!("{e:?}"));
    let resolver = Resolver::new(&theme);
    let resolution = resolver
        .resolve("nth-last-of-type-2:w-10")
        .expect("total input");
    assert!(
        !resolution.warnings.is_empty(),
        "the oracle warns on the spelling it cannot parse"
    );
}
