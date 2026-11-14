//! The STRUCTURAL-variant probe fold (plan §3.5's probe-row successor):
//! the element's own DOM-position variants — `first:`, `last:`, `odd:`,
//! `even:`, `only:`, `empty:`, `nth-N:`, `nth-last-N:` — fold onto the
//! typed [`Cond`] surface (the attribute-probe fold's precedent: the
//! differential oracle is the v0.x resolver's own `Structural` table,
//! pinned against the real tailwindcss v4.1.17 output), and the
//! interpreter resolves them through a host-answerable seam.
//!
//! This suite pins the seam that closes the boundary `variant.rs`
//! documents ("the compile stage has no `first:` surface yet"): the
//! [`StructProbe`] trait (one question — what DOM-position facts does
//! the element carry?), the built-in [`StructFacts`] value, and the
//! `*_struct` entry points the `*_probes`/`*_containers` convention
//! carries. The CSS semantics the pins encode:
//!
//! * a structural layer fires from the FACTS alone: `first:` is child
//!   index 0, `last:` the final child, `odd:`/`even:` the 1-based
//!   parity (`nth-child(odd)` — index 0 is odd), `only:` the single
//!   child, `empty:` the childless element, `nth-3:` the third child
//!   (1-based, CSS's own), `nth-last-2:` the second-from-the-end;
//! * a source-ANSWERED miss is a QUIET skip (a positional selector that
//!   does not match), never an exclusion note — the sourceless forms
//!   keep the reported exclusion ([`InterpNote::StructuralUnqueried`],
//!   the honest boundary where the host gave no answer at all);
//! * the CSS arm renders the v4 positional selectors (`:first-child`,
//!   `:nth-child(odd)`, ...) — the same forms the v0.x oracle renders;
//! * the plan bridge lowers the structural layer onto the oracle's own
//!   `Test::Structural` atoms — the differential battery pins the two
//!   generations layer for layer;
//! * the static emit renders the condition (the IR's own `Debug` wall),
//!   and the program lane's style fold fires and reports through the
//!   same `decide` verdicts.
//!
//! Written FIRST (the RED spec): the suite fails while the structural
//! variants have no fold, no seam and no render.

use aeolus_compile::{
    AttrBit, AttrKind, AttrState, Config, Env, InterpNote, Interped, StructFacts, compile,
    compile_theme, emit_css, emit_plan_theme, emit_static, interp, interp_probes_struct,
    interp_struct, interp_theme_struct, program_style_struct,
};
use aeolus_core::{Resolver, StylePlan, TAILWIND_DEFAULT_THEME, TailwindTheme};
use aeolus_ir::{Cond, Prop, Structural, Val};
use aeolus_theme::{ThemeSource, TokenTable};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// The value one longhand resolved to.
fn resolved(interp: &Interped, prop: Prop) -> Val {
    interp
        .decls
        .iter()
        .find(|d| d.prop == prop)
        .unwrap_or_else(|| panic!("{prop:?} is in the resolved declarations"))
        .val
}

/// The note one cond produced, if any.
fn note_for<'a>(interp: &'a Interped, cond: &Cond) -> Option<&'a InterpNote> {
    interp.notes.iter().find(|note| match note {
        InterpNote::PseudoElement { cond: c, .. }
        | InterpNote::ContainerUnqueried { cond: c, .. }
        | InterpNote::AttributeUnqueried { cond: c, .. }
        | InterpNote::StructuralUnqueried { cond: c, .. } => c == cond,
    })
}

/// The first-child facts: the parent's first element child (of three).
fn first_child() -> StructFacts {
    StructFacts::new(0, 3)
}

/// The last-child facts: the parent's final element child (of three).
fn last_child() -> StructFacts {
    StructFacts::new(2, 3)
}

/// The middle-child facts (index 1 of 3): neither first nor last.
fn middle_child() -> StructFacts {
    StructFacts::new(1, 3)
}

/// The childless-element facts.
fn empty_element() -> StructFacts {
    StructFacts {
        index: None,
        sibling_count: None,
        empty: true,
    }
}

/// Compile one class string and expect the walk to refuse nothing.
fn ok(class: &str) -> aeolus_compile::Compiled {
    compile(class, &Config::default()).unwrap_or_else(|e| panic!("{class}: {e:?}"))
}

// ---------------------------------------------------------------------------
// The fold: every spelling lands on the typed surface
// ---------------------------------------------------------------------------

/// The six bare spellings fold to their [`Structural`] test and lower —
/// each resolves through the seam, so a fold without a resolution path
/// cannot pass this pin.
#[test]
fn every_bare_spelling_folds_and_resolves() {
    let cases: [(&str, Structural, StructFacts); 6] = [
        ("first:w-64", Structural::First, first_child()),
        ("last:w-64", Structural::Last, last_child()),
        ("odd:w-64", Structural::Odd, first_child()),
        ("even:w-64", Structural::Even, middle_child()),
        ("only:w-64", Structural::Only, StructFacts::new(0, 1)),
        ("empty:w-64", Structural::Empty, empty_element()),
    ];
    for (class, expected, facts) in cases {
        let compiled = ok(class);
        let layer = compiled
            .program
            .layers
            .iter()
            .find(|layer| layer.cond == Cond::Structural(expected))
            .unwrap_or_else(|| panic!("{class}: folds to Cond::Structural({expected:?})"));
        assert_eq!(layer.cond.specificity(), 2, "{class}: the probe band");
        let hot = interp_struct(&compiled.program, &Env::default(), &facts)
            .unwrap_or_else(|e| panic!("{class}: {e:?}"));
        assert_eq!(
            resolved(&hot, Prop::Width),
            Val::Rem(16.0),
            "{class}: w-64 lowers to 16rem and the layer fires on the facts"
        );
    }
}

/// The `nth-N` forms fold with their 1-based ordinal: `nth-3:` is the
/// third child, `nth-last-2:` the second from the end. The same i32
/// grammar the v0.x oracle's `parse_nth` accepts (`nth--1` folds and
/// renders a selector CSS never matches — the honest spelling pass).
#[test]
fn nth_forms_fold_with_their_ordinals() {
    let compiled = ok("nth-3:w-64");
    assert!(
        compiled
            .program
            .layers
            .iter()
            .any(|layer| layer.cond == Cond::Structural(Structural::Nth(3))),
        "nth-3 folds to Structural::Nth(3)"
    );
    let compiled = ok("nth-last-2:w-64");
    assert!(
        compiled
            .program
            .layers
            .iter()
            .any(|layer| layer.cond == Cond::Structural(Structural::NthLast(2))),
        "nth-last-2 folds to Structural::NthLast(2)"
    );
    // The oracle's own grammar: the negative ordinal parses (a selector
    // that never matches — CSS-honest).
    let compiled = ok("nth--1:w-64");
    assert!(
        compiled
            .program
            .layers
            .iter()
            .any(|layer| layer.cond == Cond::Structural(Structural::Nth(-1))),
        "nth--1 folds to Structural::Nth(-1) (the oracle's parse_nth grammar)"
    );
}

/// The arbitrary `nth-` selector forms (`nth-[2n+1]:`) STAY refused —
/// the typed unsupported-variant diagnostic, never a silent drop (the
/// oracle routes them to `Variant::Arbitrary` too).
#[test]
fn selector_nth_forms_stay_refused() {
    let compiled = compile("nth-[2n+1]:w-64", &Config::default()).expect("total input compiles");
    assert!(
        !compiled.diagnostics.is_empty(),
        "nth-[2n+1] carries the typed unsupported-variant diagnostic"
    );
    assert!(
        compiled.program.layers.is_empty(),
        "the refused variant lowers no layers"
    );
}

// ---------------------------------------------------------------------------
// The seam: facts decide, misses stay quiet, sourceless reports
// ---------------------------------------------------------------------------

#[test]
fn the_seam_answers_first_at_the_first_child_only() {
    let compiled = ok("first:w-64");
    // The first child: fires.
    let hot = interp_struct(&compiled.program, &Env::default(), &first_child())
        .expect("the walk resolves");
    assert_eq!(resolved(&hot, Prop::Width), Val::Rem(16.0));
    // The middle child: QUIET — no declarations, no note.
    let cold = interp_struct(&compiled.program, &Env::default(), &middle_child())
        .expect("the walk resolves");
    assert!(cold.decls.iter().all(|d| d.prop != Prop::Width));
    assert!(cold.notes.is_empty(), "a source-answered miss is CSS-quiet");
    // No facts at all (UNKNOWN): quiet too — the built-in answered
    // "unknown", which no positional selector can match.
    let unknown = interp_struct(&compiled.program, &Env::default(), &StructFacts::UNKNOWN)
        .expect("the walk resolves");
    assert!(unknown.decls.iter().all(|d| d.prop != Prop::Width));
    assert!(unknown.notes.is_empty(), "UNKNOWN facts are a quiet miss");
    // No SOURCE (the plain form): the honest exclusion, reported.
    let sourceless = interp(&compiled.program, &Env::default()).expect("the walk resolves");
    assert!(sourceless.decls.iter().all(|d| d.prop != Prop::Width));
    assert!(
        note_for(&sourceless, &Cond::Structural(Structural::First)).is_some(),
        "the sourceless form reports StructuralUnqueried"
    );
}

#[test]
fn the_seam_answers_last_at_the_final_child_only() {
    let compiled = ok("last:w-64");
    let hot = interp_struct(&compiled.program, &Env::default(), &last_child())
        .expect("the walk resolves");
    assert_eq!(resolved(&hot, Prop::Width), Val::Rem(16.0));
    let cold = interp_struct(&compiled.program, &Env::default(), &middle_child())
        .expect("the walk resolves");
    assert!(cold.decls.iter().all(|d| d.prop != Prop::Width));
    assert!(cold.notes.is_empty(), "a source-answered miss is CSS-quiet");
}

#[test]
fn odd_and_even_split_the_one_based_parity() {
    // CSS parity: the FIRST child (index 0) is `:nth-child(odd)`, the
    // second (index 1) even.
    let odd = ok("odd:w-64");
    let even = ok("even:w-64");
    let hot =
        interp_struct(&odd.program, &Env::default(), &first_child()).expect("the walk resolves");
    assert_eq!(
        resolved(&hot, Prop::Width),
        Val::Rem(16.0),
        "child 1 is odd"
    );
    let hot =
        interp_struct(&even.program, &Env::default(), &middle_child()).expect("the walk resolves");
    assert_eq!(
        resolved(&hot, Prop::Width),
        Val::Rem(16.0),
        "child 2 is even"
    );
    let cold =
        interp_struct(&odd.program, &Env::default(), &middle_child()).expect("the walk resolves");
    assert!(cold.decls.iter().all(|d| d.prop != Prop::Width));
}

#[test]
fn only_needs_the_single_child() {
    let compiled = ok("only:w-64");
    let hot = interp_struct(&compiled.program, &Env::default(), &StructFacts::new(0, 1))
        .expect("the walk resolves");
    assert_eq!(resolved(&hot, Prop::Width), Val::Rem(16.0));
    // Three children with index 0: first, not only.
    let cold = interp_struct(&compiled.program, &Env::default(), &first_child())
        .expect("the walk resolves");
    assert!(cold.decls.iter().all(|d| d.prop != Prop::Width));
}

#[test]
fn nth_ordinal_is_one_based_and_nth_last_counts_from_the_end() {
    let nth = ok("nth-3:w-64");
    // Index 2 of 3 children: the third child.
    let hot = interp_struct(&nth.program, &Env::default(), &StructFacts::new(2, 3))
        .expect("the walk resolves");
    assert_eq!(resolved(&hot, Prop::Width), Val::Rem(16.0));
    let cold =
        interp_struct(&nth.program, &Env::default(), &first_child()).expect("the walk resolves");
    assert!(cold.decls.iter().all(|d| d.prop != Prop::Width));

    let nth_last = ok("nth-last-2:w-64");
    // Index 1 of 3: second from the end.
    let hot = interp_struct(&nth_last.program, &Env::default(), &middle_child())
        .expect("the walk resolves");
    assert_eq!(resolved(&hot, Prop::Width), Val::Rem(16.0));
    let cold = interp_struct(&nth_last.program, &Env::default(), &last_child())
        .expect("the walk resolves");
    assert!(cold.decls.iter().all(|d| d.prop != Prop::Width));
}

#[test]
fn empty_needs_the_childless_element() {
    let compiled = ok("empty:w-64");
    let hot = interp_struct(&compiled.program, &Env::default(), &empty_element())
        .expect("the walk resolves");
    assert_eq!(resolved(&hot, Prop::Width), Val::Rem(16.0));
    let cold = interp_struct(&compiled.program, &Env::default(), &first_child())
        .expect("the walk resolves");
    assert!(cold.decls.iter().all(|d| d.prop != Prop::Width));
}

/// The themed form resolves token values through the same seam family:
/// `first:bg-primary` fires on the facts and bakes the token colour.
#[test]
fn the_themed_struct_form_resolves_tokens() {
    let source = ThemeSource::shadcn().expect("embedded palette parses");
    let tokens = TokenTable::baked(&source).expect("the baked table builds");
    let compiled = compile_theme("first:bg-primary", &tokens, &Config::default())
        .expect("the themed compile resolves");
    let hot = interp_theme_struct(&compiled.program, &tokens, &Env::default(), &first_child())
        .expect("the walk resolves");
    assert!(
        matches!(resolved(&hot, Prop::Background), Val::Rgba(_)),
        "the token resolved to a concrete colour"
    );
}

/// The probe-composed form: attribute bits and structural facts ride
/// ONE walk (the full-family shape) — two classes, two layers, each
/// answered through its own seam in one call.
#[test]
fn the_seams_compose_in_one_walk() {
    let compiled = ok("data-open:w-64 first:w-64");
    let facts = first_child();
    let bits = [AttrBit {
        kind: AttrKind::Data,
        name: "open",
        value: "",
    }];
    let attrs = AttrState::new(&bits);
    let hot = interp_probes_struct(&compiled.program, &Env::default(), &attrs, &facts)
        .expect("the walk resolves");
    assert_eq!(resolved(&hot, Prop::Width), Val::Rem(16.0));
    assert!(
        hot.conds
            .iter()
            .any(|cond| matches!(cond, Cond::Structural(_))),
        "the structural layer's winner condition is reported"
    );
}

// ---------------------------------------------------------------------------
// The CSS arm: the v4 positional selectors
// ---------------------------------------------------------------------------

#[test]
fn the_css_arm_renders_the_positional_selectors() {
    let cases = [
        ("first:w-64", ".first\\:w-64:first-child"),
        ("last:w-64", ".last\\:w-64:last-child"),
        ("odd:w-64", ".odd\\:w-64:nth-child(odd)"),
        ("even:w-64", ".even\\:w-64:nth-child(even)"),
        ("only:w-64", ".only\\:w-64:only-child"),
        ("empty:w-64", ".empty\\:w-64:empty"),
        ("nth-3:w-64", ".nth-3\\:w-64:nth-child(3)"),
        ("nth-last-2:w-64", ".nth-last-2\\:w-64:nth-last-child(2)"),
    ];
    for (class, selector) in cases {
        let sheet =
            emit_css(class, &Config::default()).unwrap_or_else(|e| panic!("{class}: {e:?}"));
        assert!(
            sheet.diagnostics.is_empty(),
            "{class}: diagnostics {:?}",
            sheet.diagnostics
        );
        let text = sheet.to_string();
        assert!(
            text.contains(&format!("{selector} {{")),
            "{class}: renders {selector}, got:\n{text}"
        );
    }
}

/// The fold and the render agree: a class the FOLD accepts renders
/// (the diagnostics-equality contract the css suite pins), and the
/// refuse-set stays refused in BOTH channels.
#[test]
fn the_css_arm_refuses_what_the_fold_refuses() {
    let sheet = emit_css("nth-[2n+1]:w-64", &Config::default()).expect("total input");
    assert!(
        !sheet.diagnostics.is_empty(),
        "the refused variant carries the same typed diagnostic in the CSS arm"
    );
}

// ---------------------------------------------------------------------------
// The static emit: the condition renders through the IR's own wall
// ---------------------------------------------------------------------------

#[test]
fn the_static_emit_renders_the_structural_cond() {
    let compiled = ok("first:w-64");
    let text = emit_static(&compiled.program);
    assert!(
        text.contains("Cond::Structural(First)"),
        "the static item names the structural condition, got:\n{text}"
    );
}

// ---------------------------------------------------------------------------
// The plan bridge: the differential battery
// ---------------------------------------------------------------------------

/// Every structural class the differential pins — all of them lower
/// through the registry AND resolve through the v0.x oracle
/// warning-free (the battery's own contract, plan_bridge.rs's shape).
/// One variant per class: the flat fold collapses stacks to the
/// dominant condition (the documented boundary), so stacked forms are
/// the oracle's own multi-atom shape and stay out of the battery.
/// `nth-last-2:` is the battery's one DELIBERATE absence — the v0.x
/// parser's `parse_nth(rest, false)` last-flag is never set, so the
/// oracle warns on the spelling the real v4 compiler ships; the V1
/// fold follows the real grammar, and `nth_last_diverges_from_the_oracle`
/// pins the divergence instead.
const BATTERY: &[&str] = &[
    "first:w-10",
    "last:w-10",
    "odd:bg-red-500",
    "even:bg-red-500",
    "only:w-10",
    "nth-3:w-10",
    "first:p-4",
];

/// The one plan the walk must agree with: the v0.x oracle's own.
fn oracle_plan(class: &str) -> StylePlan {
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
        // Layer-for-layer: the structural layers carry the SAME
        // conditions on both sides (the bridge maps the typed
        // `Cond::Structural` onto the oracle's `Test::Structural`).
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

/// The deliberate divergence, pinned as a record: `nth-last-2:` lowers
/// clean through the V1 pipeline (the real v4 grammar — the CSS arm
/// renders `:nth-last-child(2)`) while the v0.x oracle warns (its
/// parser's last-flag gap). A disagreement is a bug in one of the
/// three — here it is the oracle's documented gap, kept honest.
#[test]
fn nth_last_diverges_from_the_oracle() {
    let sheet = emit_plan_theme(
        "nth-last-2:w-10",
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
    let resolution = resolver.resolve("nth-last-2:w-10").expect("total input");
    assert!(
        !resolution.warnings.is_empty(),
        "the oracle warns on the spelling it cannot parse"
    );
}

// ---------------------------------------------------------------------------
// The style walk: the program lane shares the verdicts
// ---------------------------------------------------------------------------

#[test]
fn the_style_walk_fires_and_reports_through_the_same_verdicts() {
    let compiled = ok("first:w-64");
    let layers: &'static [aeolus_ir::Layer] = compiled.program.layers;
    // Fired: the first-child facts write the slot (`w-64` carries no
    // keyword slots — the empty annotation table is the honest form).
    let (_, walk) = program_style_struct(layers, &[], &Env::default(), Some(&first_child()));
    assert!(walk.is_empty(), "the fired walk reports nothing: {walk:?}");
    // Sourceless: the note rides the walk's report.
    let (_, walk) = program_style_struct(layers, &[], &Env::default(), None);
    assert!(
        walk.notes
            .iter()
            .any(|note| matches!(note, InterpNote::StructuralUnqueried { .. })),
        "the sourceless style walk reports the structural exclusion: {:?}",
        walk.notes
    );
}
