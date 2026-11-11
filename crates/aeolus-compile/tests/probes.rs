//! The attribute-probe interp surface (architecture plan §3.5 — "the
//! element's own attributes answer through TYPED STATE BITS passed
//! with the element, not string probes"): the follow-up the
//! interpreter arm's `AttributeUnqueried` note has named since the
//! attribute-probe fold landed. The seam:
//!
//! * [`AttrKind`] — the element-own attribute families the typed
//!   `Cond` surface folds (`data-*`, `aria-*`);
//! * [`AttrProbe`] — the one question a host answers (the same shape
//!   as `TokenSource`/`ClassifyVariant`/`DynStore`: the walk is
//!   generic, the host wires its own);
//! * [`AttrState`] — the built-in source: the element's attribute bits
//!   as a static table (`("state", "open")` under `AttrKind::Data`);
//! * [`interp_probes`]/[`interp_theme_probes`] — the `*_with` entry
//!   points carrying the seam (the `compile_with` convention).
//!
//! The CSS semantics the pins encode: a probe layer FIRES when the
//! element carries the attribute (presence) or carries it with the
//! folded value (equality); a miss is a QUIET skip (a CSS selector
//! that does not match, not an exclusion) — the note stays for the
//! UNQUERIED cases only (no source rides, or the probe is a
//! descendant shape: `has-`/`group-has-`/`peer-has-` are the plan
//! §3.5's diagnosed boundary, honest failure kept).
//!
//! Written FIRST (the RED spec): the suite fails against the 0.17.0
//! seam — the entry points and the seam types do not exist yet.

use aeolus_compile::{
    AttrBit, AttrKind, AttrState, Config, CustomVariant, Customs, Env, InterpNote, Interped,
    compile, compile_theme, compile_with, interp, interp_probes, interp_theme_probes,
};
use aeolus_ir::{Cond, Prop, Val};
use aeolus_theme::{ThemeSource, TokenTable};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn runtime_table() -> TokenTable {
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    TokenTable::runtime(&source).expect("default-seeded source builds")
}

/// The element state the probes answer for: a data-open card, aria busy.
fn card_state() -> AttrState<'static> {
    AttrState::new(&[
        AttrBit {
            kind: AttrKind::Data,
            name: "state",
            value: "open",
        },
        AttrBit {
            kind: AttrKind::Data,
            name: "slot",
            value: "card-action",
        },
        AttrBit {
            kind: AttrKind::Aria,
            name: "busy",
            value: "false",
        },
    ])
}

/// The value one longhand resolved to.
fn resolved(interp: &Interped, prop: Prop) -> Val {
    interp
        .decls
        .iter()
        .find(|d| d.prop == prop)
        .unwrap_or_else(|| panic!("{prop:?} is in the resolved declarations"))
        .val
}

fn note_for<'i>(interp: &'i Interped, cond: &Cond) -> Option<&'i InterpNote> {
    interp.notes.iter().find(|note| match note {
        InterpNote::AttributeUnqueried { cond: c, .. } => c == cond,
        _ => false,
    })
}

// ---------------------------------------------------------------------------
// 1. The probe layers fire under the state bits
// ---------------------------------------------------------------------------

#[test]
fn the_value_probe_fires_when_the_element_carries_the_value() {
    let compiled = compile("data-[state=open]:p-8", &Config::default()).unwrap();
    let interp = interp_probes(&compiled.program, &Env::default(), &card_state()).unwrap();
    assert!(interp.notes.is_empty(), "no exclusions: {:?}", interp.notes);
    assert_eq!(resolved(&interp, Prop::PaddingTop), Val::Rem(2.0));
}

#[test]
fn the_presence_probe_fires_when_the_element_carries_the_attribute() {
    // The bare spelling folds to the presence test (`data-open:`), so
    // ANY value answers it — the slot bit answers even though the
    // state bit's value is what the other layer wanted.
    let compiled = compile("data-slot:p-8", &Config::default()).unwrap();
    let interp = interp_probes(&compiled.program, &Env::default(), &card_state()).unwrap();
    assert!(interp.notes.is_empty(), "no exclusions: {:?}", interp.notes);
    assert_eq!(resolved(&interp, Prop::PaddingTop), Val::Rem(2.0));
}

#[test]
fn the_aria_probe_fires_on_the_exact_value() {
    // The bare aria spelling normalizes to "true" — the element's
    // aria-busy bit says "false", so the layer does NOT fire.
    let busy = compile("aria-busy:p-8", &Config::default()).unwrap();
    let interp = interp_probes(&busy.program, &Env::default(), &card_state()).unwrap();
    assert!(interp.decls.is_empty(), "false busy does not fire");
    assert!(interp.notes.is_empty(), "answered-no is CSS-quiet");
}

// ---------------------------------------------------------------------------
// 2. Misses are CSS-quiet, never exclusions
// ---------------------------------------------------------------------------

#[test]
fn a_value_mismatch_is_a_quiet_skip_not_a_note() {
    let compiled = compile("data-[state=closed]:p-8", &Config::default()).unwrap();
    let interp = interp_probes(&compiled.program, &Env::default(), &card_state()).unwrap();
    assert!(interp.decls.is_empty(), "closed does not match open");
    assert!(
        interp.notes.is_empty(),
        "the probe was ANSWERED; a miss is a selector that does not match: {:?}",
        interp.notes
    );
}

#[test]
fn an_absent_attribute_is_a_quiet_skip_not_a_note() {
    let compiled = compile("data-[state=open]:p-8", &Config::default()).unwrap();
    let empty = AttrState::new(&[]);
    let interp = interp_probes(&compiled.program, &Env::default(), &empty).unwrap();
    assert!(interp.decls.is_empty());
    assert!(interp.notes.is_empty(), "answered (absent) is CSS-quiet");
}

// ---------------------------------------------------------------------------
// 3. The unqueried boundary stays the note (back-compat + the has- family)
// ---------------------------------------------------------------------------

#[test]
fn the_sourceless_entry_points_still_report_the_note() {
    let compiled = compile("data-[state=open]:p-8", &Config::default()).unwrap();
    let interp = interp(&compiled.program, &Env::default()).unwrap();
    assert!(interp.decls.is_empty());
    assert!(
        note_for(
            &interp,
            &Cond::Data {
                name: "state",
                value: Some("open")
            }
        )
        .is_some(),
        "the plain interp has no probe source: the note reports, never fires"
    );
}

#[test]
fn the_descendant_probes_stay_the_diagnosed_boundary() {
    // The plan §3.5 row: `has-*` is a descendant question — diagnosed
    // with a reason, honest failure kept — even WITH a probe source.
    let compiled = compile(
        "has-focus:p-8 group-has-[:focus]:p-8 peer-has-data-[slot=card-action]:p-8",
        &Config::default(),
    )
    .unwrap();
    let interp = interp_probes(&compiled.program, &Env::default(), &card_state()).unwrap();
    assert!(interp.decls.is_empty());
    assert_eq!(
        interp.notes.len(),
        3,
        "all three reported: {:?}",
        interp.notes
    );
}

// ---------------------------------------------------------------------------
// 4. The cascade: a fired probe layer wins per longhand, in order
// ---------------------------------------------------------------------------

#[test]
fn the_fired_probe_layer_wins_the_longhand_cascade() {
    // Base padding, then the probe layer: the fired probe overrides
    // per longhand, exactly like a fired state layer does.
    let compiled = compile("p-4 data-[state=open]:p-8", &Config::default()).unwrap();
    let interp = interp_probes(&compiled.program, &Env::default(), &card_state()).unwrap();
    assert_eq!(resolved(&interp, Prop::PaddingTop), Val::Rem(2.0));
    assert_eq!(resolved(&interp, Prop::PaddingLeft), Val::Rem(2.0));
}

#[test]
fn the_unfired_probe_layer_leaves_the_base() {
    let compiled = compile("p-4 data-[state=closed]:p-8", &Config::default()).unwrap();
    let interp = interp_probes(&compiled.program, &Env::default(), &card_state()).unwrap();
    assert_eq!(resolved(&interp, Prop::PaddingTop), Val::Rem(1.0));
}

// ---------------------------------------------------------------------------
// 5. Theme dynamism composes with the probes
// ---------------------------------------------------------------------------

#[test]
fn the_themed_entry_point_resolves_tokens_and_probes_together() {
    // `bg-primary` is a theme token; the probe layer rides beside it.
    let classes = "bg-primary data-[state=open]:p-8";
    let compiled = compile_theme(classes, &runtime_table(), &Config::default()).unwrap();
    let interp = interp_theme_probes(
        &compiled.program,
        &runtime_table(),
        &Env::default(),
        &card_state(),
    )
    .unwrap();
    assert!(interp.notes.is_empty(), "{:?}", interp.notes);
    assert!(matches!(resolved(&interp, Prop::Background), Val::Rgba(_)));
    assert_eq!(resolved(&interp, Prop::PaddingTop), Val::Rem(2.0));
}

// ---------------------------------------------------------------------------
// 6. The custom seam composes: an alias onto a probe cond fires
// ---------------------------------------------------------------------------

#[test]
fn a_custom_variant_alias_onto_a_probe_cond_fires() {
    let customs = Customs::new().variant(CustomVariant::new(
        "open",
        Cond::Data {
            name: "state",
            value: Some("open"),
        },
    ));
    let compiled = compile_with("open:p-8", &customs, &Config::default()).unwrap();
    let interp = interp_probes(&compiled.program, &Env::default(), &card_state()).unwrap();
    assert!(interp.notes.is_empty(), "{:?}", interp.notes);
    assert_eq!(resolved(&interp, Prop::PaddingTop), Val::Rem(2.0));
}

// ---------------------------------------------------------------------------
// 7. The ergonomic half and the determinism pin
// ---------------------------------------------------------------------------

#[test]
fn the_compiled_carries_the_probe_surface() {
    let compiled = compile("data-[state=open]:p-8", &Config::default()).unwrap();
    let interp = compiled
        .interp_probes(&Env::default(), &card_state())
        .unwrap();
    assert_eq!(resolved(&interp, Prop::PaddingTop), Val::Rem(2.0));
}

#[test]
fn the_probe_walk_is_pure() {
    let compiled = compile("data-[state=open]:p-8", &Config::default()).unwrap();
    let a = interp_probes(&compiled.program, &Env::default(), &card_state()).unwrap();
    let b = interp_probes(&compiled.program, &Env::default(), &card_state()).unwrap();
    assert_eq!(a, b, "same program + env + state, same answer");
}
