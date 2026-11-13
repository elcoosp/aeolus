//! The pseudo-element RENDER TARGETS (the interp arm's own-slice note,
//! architecture plan §3.5's variant-costs row): `::selection` and
//! `::placeholder` style a DIFFERENT render target than the element's
//! own cascade — the plain [`interp`] forms exclude those layers with
//! the [`InterpNote::PseudoElement`] note and stop there. This suite
//! pins the seam that RESOLVES those cascades: the [`PseudoTarget`]
//! selector and the `interp_pseudo`/`interp_pseudo_theme` entry points
//! (the interp arm completing the parity the CSS arm already renders).
//!
//! The pins, in the order the CSS semantics demand:
//!
//! * the target's own layers fire and resolve (the same value the
//!   variantless compile lowers — a variant must not change what the
//!   utility means, the differential pin the value pass rides);
//! * the OTHER pseudo-element's layers stay out of the answer (each
//!   target is its own cascade);
//! * the element's OWN layers (base, hover, …) never participate in a
//!   pseudo cascade — only rules targeting the pseudo-element apply
//!   to it, so the walk is silent about them, not noting them;
//! * the element walk keeps excluding the pseudo layers with its note
//!   (the two walks agree on which target a layer belongs to);
//! * the themed form resolves token slots beside the target, and the
//!   `Compiled` convenience form delegates to the same walk.
//!
//! The colour pins ride the ARBITRARY literal form (`bg-[#eab308]`):
//! the palette spellings (`bg-yellow-500`) are the theme pass's own
//! values and lower only through `compile_theme` — the themeless walk
//! reports them typed, the exact boundary the suite's themed pin
//! exercises with `selection:bg-primary` instead.
//!
//! Written FIRST (the RED spec): the suite fails while the pseudo
//! cascades have no entry point to resolve them.

use aeolus_compile::{
    Config, Env, InterpNote, PseudoTarget, compile, compile_theme, interp, interp_pseudo,
    interp_pseudo_theme,
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

/// The runtime-tier table over the shadcn source.
fn runtime_table() -> TokenTable {
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    TokenTable::runtime(&source).expect("default-seeded source builds")
}

// ---------------------------------------------------------------------------
// The target's own cascade
// ---------------------------------------------------------------------------

#[test]
fn the_selection_target_resolves_the_selection_layers() {
    let compiled = compile("selection:bg-[#eab308]", &Config::default()).expect("compiles");
    let interped =
        interp_pseudo(&compiled.program, PseudoTarget::Selection).expect("the walk resolves");
    let val = resolved(&interped, Prop::Background);
    assert!(
        matches!(val, Val::Rgba([..])),
        "the selection cascade carries the concrete colour: {val:?}"
    );
    assert_eq!(
        interped.conds[0],
        Cond::Selection,
        "the winner's condition is the pseudo-element's own"
    );
    assert!(
        interped.notes.is_empty(),
        "a scoped walk reports nothing: {:?}",
        interped.notes
    );
}

#[test]
fn the_other_target_yields_the_empty_answer() {
    // The placeholder cascade of a selection-only program is EMPTY —
    // no rules target ::placeholder, no notes either (the answer is
    // scoped, not an exclusion report).
    let compiled = compile("selection:bg-[#eab308]", &Config::default()).expect("compiles");
    let interped =
        interp_pseudo(&compiled.program, PseudoTarget::Placeholder).expect("the walk resolves");
    assert!(interped.decls.is_empty(), "the other target never fires");
    assert!(interped.notes.is_empty());
}

#[test]
fn the_placeholder_target_resolves_the_placeholder_layers() {
    let compiled = compile("placeholder:text-[#ef4444]", &Config::default()).expect("compiles");
    let interped =
        interp_pseudo(&compiled.program, PseudoTarget::Placeholder).expect("the walk resolves");
    let val = resolved(&interped, Prop::Color);
    assert!(
        matches!(val, Val::Rgba([..])),
        "the placeholder cascade carries the concrete colour: {val:?}"
    );
}

// ---------------------------------------------------------------------------
// Target isolation: the element's own layers never participate
// ---------------------------------------------------------------------------

#[test]
fn the_element_layers_never_participate_in_a_pseudo_cascade() {
    // The expected value is the variantless compile's own answer (the
    // differential pin — the suite never re-states the palette).
    let plain = compile("bg-[#eab308]", &Config::default()).expect("compiles");
    let expected = resolved(
        &interp(&plain.program, &Env::default()).expect("resolves"),
        Prop::Background,
    );
    let compiled = compile(
        "bg-[#ef4444] hover:bg-[#b91c1c] selection:bg-[#eab308]",
        &Config::default(),
    )
    .expect("compiles");
    let interped =
        interp_pseudo(&compiled.program, PseudoTarget::Selection).expect("the walk resolves");
    assert_eq!(
        resolved(&interped, Prop::Background),
        expected,
        "only the ::selection rule applies to the selection box"
    );
    assert_eq!(
        interped.decls.len(),
        1,
        "the base and hover layers are not part of this target: {:?}",
        interped.decls
    );
}

#[test]
fn the_element_walk_keeps_excluding_the_pseudo_layers() {
    // The parity pin: the plain element walk still reports the pseudo
    // layers it does not fold (the two walks agree on which target a
    // layer belongs to). The base value is the variantless answer.
    let plain = compile("bg-[#ef4444]", &Config::default()).expect("compiles");
    let expected = resolved(
        &interp(&plain.program, &Env::default()).expect("resolves"),
        Prop::Background,
    );
    let compiled =
        compile("bg-[#ef4444] selection:bg-[#eab308]", &Config::default()).expect("compiles");
    let element = interp(&compiled.program, &Env::default()).expect("the walk resolves");
    assert_eq!(
        resolved(&element, Prop::Background),
        expected,
        "the element cascade carries the base colour"
    );
    assert!(
        element
            .notes
            .iter()
            .any(|note| matches!(note, InterpNote::PseudoElement { .. })),
        "the element walk keeps the honest exclusion note"
    );
}

// ---------------------------------------------------------------------------
// The differential pin: a variant must not change the utility's value
// ---------------------------------------------------------------------------

#[test]
fn the_variant_must_not_change_the_utilitys_value() {
    // `selection:bg-[#eab308]`'s pseudo cascade carries the SAME colour
    // the variantless `bg-[#eab308]` lowers to — the interp arm and the
    // CSS arm draw from one value pass.
    let plain = compile("bg-[#eab308]", &Config::default()).expect("compiles");
    let variant = compile("selection:bg-[#eab308]", &Config::default()).expect("compiles");
    let plain_val = resolved(
        &interp(&plain.program, &Env::default()).expect("resolves"),
        Prop::Background,
    );
    let pseudo_val = resolved(
        &interp_pseudo(&variant.program, PseudoTarget::Selection).expect("resolves"),
        Prop::Background,
    );
    assert_eq!(plain_val, pseudo_val, "one value pass behind both arms");
}

// ---------------------------------------------------------------------------
// The themed form and the Compiled convenience form
// ---------------------------------------------------------------------------

#[test]
fn the_themed_form_resolves_tokens_beside_the_target() {
    let table = runtime_table();
    let compiled =
        compile_theme("selection:bg-primary", &table, &Config::default()).expect("theme compiles");
    let interped = interp_pseudo_theme(
        &compiled.program,
        &table,
        &Env::default(),
        PseudoTarget::Selection,
    )
    .expect("the walk resolves");
    let val = resolved(&interped, Prop::Background);
    assert!(
        matches!(val, Val::Rgba([..])),
        "the token resolves to concrete RGBA in the pseudo cascade: {val:?}"
    );
}

#[test]
fn the_compiled_convenience_form_carries_the_target() {
    let compiled = compile("selection:bg-[#eab308]", &Config::default()).expect("compiles");
    let interped = compiled
        .interp_pseudo(PseudoTarget::Selection)
        .expect("the walk resolves");
    assert!(
        interped.decls.iter().any(|d| d.prop == Prop::Background),
        "the Compiled form delegates to the same target walk"
    );
}
