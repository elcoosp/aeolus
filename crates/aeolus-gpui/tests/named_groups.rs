//! The named-group IR slot's GPUI half: the label form
//! (`group-hover/menu:`) is a DIFFERENT condition — the named-container
//! fold's own rule — so the env walk keeps it the quiet miss (the
//! unnamed boolean cannot answer a named question), the native
//! `.group_hover(label, …)`/`.group_active(label, …)` closures the
//! [`TwStateExt::tw_state`] state walk wires are the label dispatch's
//! seam (exercised by the `group_states` example — the closure shape
//! needs a real element tree), and the `tw!` audit gate passes the
//! named programs (the audit is per-decl; the named condition changes
//! nothing about the declarations' expressibility).
//!
//! Written FIRST (the RED spec): the suite fails while the `Cond`
//! variants carry no name payload.

use aeolus_gpui::prelude::*;
use gpui::rems;

#[test]
fn the_named_label_is_a_different_condition() {
    // The named label folds to its own condition — the payload rides
    // the IR slot, two different groups are two different conditions.
    let bare = tw!("group-hover:bg-red-500");
    let named = tw!("group-hover/menu:bg-red-500");
    match (bare.layers[0].cond, named.layers[0].cond) {
        (
            aeolus_ir::Cond::GroupHover { name: None },
            aeolus_ir::Cond::GroupHover { name: Some("menu") },
        ) => {}
        other => panic!("the named fold rides the IR slot: {other:?}"),
    }
}

#[test]
fn the_named_label_stays_quiet_in_the_env_walk() {
    // The env's UNNAMED boolean cannot answer a named question: the
    // named layer is the quiet miss in the plain walk (the interp's own
    // reading), never the unnamed fact's answer.
    let program = tw!("group-hover/menu:p-4");
    let style = gpui::StyleRefinement::default().tw_env(
        program,
        Env {
            group_hover: true,
            ..Env::default()
        },
    );
    assert_eq!(
        style.padding.top, None,
        "the unnamed fact does not answer the named condition"
    );
}

#[test]
fn the_unnamed_form_keeps_the_boolean_fold() {
    // Byte-identical to the wave-1 behavior: the BARE form answers the
    // env's unnamed fact.
    let program = tw!("group-hover:p-4");
    let style = gpui::StyleRefinement::default().tw_env(
        program,
        Env {
            group_hover: true,
            ..Env::default()
        },
    );
    assert_eq!(
        style.padding.top,
        Some(rems(1.0).into()),
        "the bare form is the flat .group fact"
    );
}

#[test]
fn the_named_programs_pass_the_audit_gate() {
    // The audit is per-decl: a named condition changes nothing about
    // the declarations' expressibility — the named group-hover and
    // group-active programs compile through `tw!` (the macro's audit
    // gate ran at expansion; a refusal would have panicked here).
    let hover = tw!("group-hover/menu:bg-red-500 from-blue-500");
    assert!(hover.layers.iter().any(|layer| !layer.decls.is_empty()));
    let active = tw!("group-active/nav:underline");
    assert!(active.layers.iter().any(|layer| !layer.decls.is_empty()));
}

#[test]
fn the_freeze_is_identical_through_the_static_face() {
    // The named condition freezes through the SAME emit face the
    // unnamed one does — the payload is the frozen `&'static str` (the
    // named-container form's own shape). The static program's layers
    // carry the label; a `tw_static!`-lane freeze (the interpreter
    // backends' neutral face) rides the same slices.
    let program = tw!("group-focus-within/nav:underline");
    match program.layers[0].cond {
        aeolus_ir::Cond::GroupFocusWithin { name: Some("nav") } => {}
        other => panic!("the frozen program carries the label: {other:?}"),
    }
}
