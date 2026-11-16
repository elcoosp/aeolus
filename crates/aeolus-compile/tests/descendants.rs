//! The descendant-probe interp surface (architecture plan §3.5 — the
//! `has-*` family's row, the follow-up the `AttributeUnqueried` note's
//! descendant case named since the attribute-probe fold landed). The
//! seam:
//!
//! * [`DescendantKind`] — the three `:has()` scopes the typed `Cond`
//!   surface folds (the element's own subtree, the `.group` ancestor's,
//!   the `.peer` sibling's);
//! * [`DescendantProbe`] — the one question a host answers (the same
//!   shape as `AttrProbe`/`ContainerProbe`/`StructProbe`: the walk is
//!   generic, the host wires its own);
//! * [`DescendantState`] — the built-in source: the subtree facts as a
//!   static table (`(Element, "*:focus")`);
//! * the `*_descendants` entry points — the seam-family convention's
//!   fourth member (the `*_probes`/`*_containers`/`*_struct` precedent,
//!   the full composed matrix included).
//!
//! The CSS semantics the pins encode: a descendant layer FIRES when the
//! source answers yes for ITS scope's fragment; a source-answered no is
//! a QUIET skip (a `:has()` selector that does not match, not an
//! exclusion) — the note stays for the UNQUERIED cases only (no source
//! rides), where it becomes the dedicated [`InterpNote::DescendantUnqueried`]
//! (the per-seam note convention the container and structural seams
//! follow; the blanket `AttributeUnqueried` spelling retires).
//!
//! Written FIRST (the RED spec): the suite fails against the 0.19.0
//! seam — the entry points and the seam types do not exist yet.

use aeolus_compile::{
    AttrBit, AttrKind, AttrState, Compiled, Config, ContainerBound, ContainerState, DescendantFact,
    DescendantKind, DescendantState, Env, InterpNote, Interped, StructFacts, compile,
    compile_theme, interp, interp_containers_descendants, interp_containers_struct_descendants,
    interp_descendants, interp_probes_containers_descendants,
    interp_probes_containers_struct_descendants, interp_probes_descendants,
    interp_probes_struct_descendants, interp_theme_containers_descendants,
    interp_theme_containers_struct_descendants, interp_theme_descendants,
    interp_theme_probes_containers_descendants, interp_theme_probes_containers_struct_descendants,
    interp_theme_probes_descendants, interp_theme_probes_struct_descendants,
    interp_theme_struct_descendants,
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

/// The subtree facts the probes answer for: the element's own subtree
/// matches `*:focus` and the card-action data attribute shape; the
/// group ancestor's subtree matches `*:focus`; the peer sibling's does
/// not.
fn tree_state() -> DescendantState<'static> {
    DescendantState::new(&[
        DescendantFact {
            kind: DescendantKind::Element,
            fragment: "*:focus",
        },
        DescendantFact {
            kind: DescendantKind::Element,
            fragment: "*[data-slot=\"card-action\"]",
        },
        DescendantFact {
            kind: DescendantKind::Element,
            fragment: "*[aria-invalid=\"true\"]",
        },
        DescendantFact {
            kind: DescendantKind::Group,
            fragment: "*:focus",
        },
    ])
}

/// The element state the attribute probes answer for (the composed
/// walks' attribute half).
fn card_attrs() -> AttrState<'static> {
    AttrState::new(&[AttrBit {
        kind: AttrKind::Data,
        name: "state",
        value: "open",
    }])
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
        InterpNote::DescendantUnqueried { cond: c, .. } => c == cond,
        _ => false,
    })
}

// ---------------------------------------------------------------------------
// The element-own half: Cond::Has
// ---------------------------------------------------------------------------

#[test]
fn descendant_layer_fires_when_the_subtree_matches() {
    let compiled = compile("has-focus:p-8", &Config::default()).expect("compiles");
    let interped =
        interp_descendants(&compiled.program, &Env::default(), &tree_state()).expect("resolves");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));
    assert!(interped.notes.is_empty(), "a fire is not a note");
}

#[test]
fn descendant_layer_misses_quietly_when_the_subtree_does_not_match() {
    // The subtree matches `*:focus` and the card-action shape, NOT
    // `*:hover` — the `has-hover` layer misses, and an ANSWERED miss is
    // CSS-quiet (a selector that does not match), never a note.
    let compiled = compile("has-hover:p-8", &Config::default()).expect("compiles");
    let interped =
        interp_descendants(&compiled.program, &Env::default(), &tree_state()).expect("resolves");
    assert!(interped.decls.is_empty(), "{:?}", interped.decls);
    assert!(interped.notes.is_empty(), "{:?}", interped.notes);
}

#[test]
fn data_and_aria_fragments_ask_the_same_seam() {
    // The compound inners fold to the same rendered fragments the
    // attribute-probe family pins (`has-data-[slot=card-action]` →
    // `*[data-slot="card-action"]`).
    let compiled = compile(
        "has-data-[slot=card-action]:pt-8 has-focus:pb-4",
        &Config::default(),
    )
    .expect("compiles");
    let interped =
        interp_descendants(&compiled.program, &Env::default(), &tree_state()).expect("resolves");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));
    assert_eq!(resolved(&interped, Prop::PaddingBottom), Val::Rem(1.0));
}

#[test]
fn sourceless_descendant_layers_stay_reported() {
    // The plain `interp` carries no source: the layer reports through
    // the DEDICATED note (the per-seam convention), never silently.
    let compiled = compile("has-focus:p-8", &Config::default()).expect("compiles");
    let interped = interp(&compiled.program, &Env::default()).expect("no theme forms");
    assert!(interped.decls.is_empty());
    let note = note_for(&interped, &Cond::Has("*:focus")).expect("the note rides");
    match note {
        InterpNote::DescendantUnqueried { cond, props } => {
            assert_eq!(*cond, Cond::Has("*:focus"));
            assert_eq!(props.len(), 4, "the four padding longhands ride the note");
        }
        other => panic!("the descendant note, not {other:?}"),
    }
    assert!(!note.to_string().is_empty(), "the note reads for humans");
}

// ---------------------------------------------------------------------------
// The parent-relative half: Cond::GroupHas / Cond::PeerHas
// ---------------------------------------------------------------------------

#[test]
fn group_scopes_ask_the_group_half() {
    let compiled = compile("group-has-focus:p-8", &Config::default()).expect("compiles");
    let interped =
        interp_descendants(&compiled.program, &Env::default(), &tree_state()).expect("resolves");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));
}

#[test]
fn peer_scopes_ask_the_peer_half_and_discriminate() {
    // The table answers the Group scope for `*:focus` but carries no
    // Peer row: the peer layer must NOT ride the group answer — the
    // kind discriminates the scope.
    let compiled = compile("peer-has-focus:p-8", &Config::default()).expect("compiles");
    let interped =
        interp_descendants(&compiled.program, &Env::default(), &tree_state()).expect("resolves");
    assert!(interped.decls.is_empty(), "{:?}", interped.decls);
    assert!(interped.notes.is_empty(), "an answered miss is quiet");
}

// ---------------------------------------------------------------------------
// The composed matrix: the seam joins the walk family
// ---------------------------------------------------------------------------

#[test]
fn attribute_and_descendant_probes_compose() {
    // Both probes ride one walk: the data- layer answers from the
    // attribute seam, the has- layer from the descendant seam.
    let compiled =
        compile("data-[state=open]:pt-2 has-focus:pb-8", &Config::default()).expect("compiles");
    let interped = interp_probes_descendants(
        &compiled.program,
        &Env::default(),
        &card_attrs(),
        &tree_state(),
    )
    .expect("resolves");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(0.5));
    assert_eq!(resolved(&interped, Prop::PaddingBottom), Val::Rem(2.0));
    assert!(interped.notes.is_empty(), "{:?}", interped.notes);
}

#[test]
fn container_and_descendant_probes_compose() {
    // The named container answers from the container seam, the has-
    // layer from the descendant seam.
    let compiled = compile("@md/card:w-64 has-focus:pb-8", &Config::default()).expect("compiles");
    let bounds = [ContainerBound {
        name: "card",
        rem: 48.0,
    }];
    let interped = interp_containers_descendants(
        &compiled.program,
        &Env::default(),
        &ContainerState::new(&bounds),
        &tree_state(),
    )
    .expect("resolves");
    assert_eq!(resolved(&interped, Prop::Width), Val::Rem(16.0));
    assert_eq!(resolved(&interped, Prop::PaddingBottom), Val::Rem(2.0));
}

#[test]
fn structural_and_descendant_probes_compose() {
    let compiled = compile("first:pt-2 has-focus:pb-8", &Config::default()).expect("compiles");
    let interped = interp_probes_struct_descendants(
        &compiled.program,
        &Env::default(),
        &AttrState::new(&[]),
        &StructFacts {
            index: Some(0),
            sibling_count: Some(3),
            empty: false,
        },
        &tree_state(),
    )
    .expect("resolves");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(0.5));
    assert_eq!(resolved(&interped, Prop::PaddingBottom), Val::Rem(2.0));
}

#[test]
fn the_themeless_full_form_rides_every_seam() {
    // Attribute + container + structural + descendant probes, one
    // walk: the FULL composed form the matrix completion lands.
    let compiled = compile(
        "data-[state=open]:pt-2 @md/card:mt-4 first:ml-1 has-focus:pb-8",
        &Config::default(),
    )
    .expect("compiles");
    let bounds = [ContainerBound {
        name: "card",
        rem: 48.0,
    }];
    let interped = interp_probes_containers_struct_descendants(
        &compiled.program,
        &Env::default(),
        &card_attrs(),
        &ContainerState::new(&bounds),
        &StructFacts {
            index: Some(0),
            sibling_count: Some(3),
            empty: false,
        },
        &tree_state(),
    )
    .expect("resolves");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(0.5));
    assert_eq!(resolved(&interped, Prop::MarginTop), Val::Rem(1.0));
    assert_eq!(resolved(&interped, Prop::PaddingBottom), Val::Rem(2.0));
    assert!(interped.notes.is_empty(), "{:?}", interped.notes);
}

#[test]
fn the_themed_full_form_resolves_tokens_too() {
    // The theme forms ride the same walk: one token value resolves
    // through the source while every seam answers.
    let compiled = compile_theme(
        "bg-background has-focus:p-8 data-[state=open]:pt-2 @md/card:mt-4 first:ml-1",
        &runtime_table(),
        &Config::default(),
    )
    .expect("compiles");
    let bounds = [ContainerBound {
        name: "card",
        rem: 48.0,
    }];
    let interped = interp_theme_probes_containers_struct_descendants(
        &compiled.program,
        &runtime_table(),
        &Env::default(),
        &card_attrs(),
        &ContainerState::new(&bounds),
        &StructFacts {
            index: Some(0),
            sibling_count: Some(3),
            empty: false,
        },
        &tree_state(),
    )
    .expect("resolves");
    assert!(matches!(
        resolved(&interped, Prop::Background),
        Val::Rgba(_)
    ));
    // `data-[state=open]:pt-2` rides the same probe tier as the
    // `has-focus:p-8` layer and comes LATER in the source string, so
    // the last-per-longhand rule hands it the top padding; the p-8
    // value keeps the sides nothing overrides.
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(0.5));
    assert_eq!(resolved(&interped, Prop::PaddingBottom), Val::Rem(2.0));
    assert_eq!(resolved(&interped, Prop::MarginTop), Val::Rem(1.0));
    assert!(interped.notes.is_empty(), "{:?}", interped.notes);
}

#[test]
fn the_matrix_themed_pairs_answer_their_seams() {
    // The remaining themed pairs: theme+containers+descendants,
    // theme+struct+descendants, theme+probes+descendants.
    let bounds = [ContainerBound {
        name: "card",
        rem: 48.0,
    }];
    let facts = StructFacts {
        index: Some(0),
        sibling_count: Some(3),
        empty: false,
    };

    let compiled = compile_theme(
        "@md/card:w-64 has-focus:p-8",
        &runtime_table(),
        &Config::default(),
    )
    .expect("c");
    let interped = interp_theme_containers_descendants(
        &compiled.program,
        &runtime_table(),
        &Env::default(),
        &ContainerState::new(&bounds),
        &tree_state(),
    )
    .expect("resolves");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));

    let compiled = compile_theme(
        "empty:pt-2 has-focus:p-8",
        &runtime_table(),
        &Config::default(),
    )
    .expect("c");
    let interped = interp_theme_struct_descendants(
        &compiled.program,
        &runtime_table(),
        &Env::default(),
        &facts,
        &tree_state(),
    )
    .expect("resolves");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));

    let compiled = compile_theme(
        "data-[state=open]:pt-2 has-focus:p-8",
        &runtime_table(),
        &Config::default(),
    )
    .expect("c");
    let interped = interp_theme_probes_descendants(
        &compiled.program,
        &runtime_table(),
        &Env::default(),
        &card_attrs(),
        &tree_state(),
    )
    .expect("resolves");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));

    // The themed D-alone form: the descendant seam rides the themed
    // walk with nothing else composed.
    let compiled = compile_theme("has-focus:p-8", &runtime_table(), &Config::default()).expect("c");
    let interped = interp_theme_descendants(
        &compiled.program,
        &runtime_table(),
        &Env::default(),
        &tree_state(),
    )
    .expect("resolves");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));

    // The themed triples: theme+probes+containers+descendants and
    // theme+containers+struct+descendants.
    let compiled = compile_theme(
        "data-[state=open]:pt-2 @md/card:mt-4 has-focus:p-8",
        &runtime_table(),
        &Config::default(),
    )
    .expect("c");
    let interped = interp_theme_probes_containers_descendants(
        &compiled.program,
        &runtime_table(),
        &Env::default(),
        &card_attrs(),
        &ContainerState::new(&bounds),
        &tree_state(),
    )
    .expect("resolves");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));

    let compiled = compile_theme(
        "@md/card:w-64 empty:pt-2 has-focus:p-8",
        &runtime_table(),
        &Config::default(),
    )
    .expect("c");
    let interped = interp_theme_containers_struct_descendants(
        &compiled.program,
        &runtime_table(),
        &Env::default(),
        &ContainerState::new(&bounds),
        &facts,
        &tree_state(),
    )
    .expect("resolves");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));

    // The themed triple: theme+probes+struct+descendants.
    let compiled = compile_theme(
        "data-[state=open]:pt-2 first:mt-1 has-focus:p-8",
        &runtime_table(),
        &Config::default(),
    )
    .expect("c");
    let interped = interp_theme_probes_struct_descendants(
        &compiled.program,
        &runtime_table(),
        &Env::default(),
        &card_attrs(),
        &facts,
        &tree_state(),
    )
    .expect("resolves");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));
}

#[test]
fn the_matrix_remaining_themeless_triples_answer_their_seams() {
    let bounds = [ContainerBound {
        name: "card",
        rem: 48.0,
    }];
    let facts = StructFacts {
        index: Some(0),
        sibling_count: Some(3),
        empty: false,
    };

    let compiled = compile("first:pt-2 has-focus:p-8", &Config::default()).expect("c");
    let interped = interp_containers_struct_descendants(
        &compiled.program,
        &Env::default(),
        &ContainerState::new(&bounds),
        &facts,
        &tree_state(),
    )
    .expect("resolves");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));

    let compiled = compile("data-[state=open]:pt-2 has-focus:p-8", &Config::default()).expect("c");
    let interped = interp_probes_containers_descendants(
        &compiled.program,
        &Env::default(),
        &card_attrs(),
        &ContainerState::new(&bounds),
        &tree_state(),
    )
    .expect("resolves");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));
}

// ---------------------------------------------------------------------------
// The ergonomic half: the Compiled method face
// ---------------------------------------------------------------------------

#[test]
fn the_compiled_method_face_carries_the_seam() {
    let compiled: Compiled =
        compile("has-focus:p-8", &Config::default()).expect("the class string compiles");
    let interped = compiled
        .interp_descendants(&Env::default(), &tree_state())
        .expect("the program resolves");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));

    let themed = compile_theme(
        "bg-background has-focus:p-8",
        &runtime_table(),
        &Config::default(),
    )
    .expect("c");
    let interped = themed
        .interp_theme_descendants(&runtime_table(), &Env::default(), &tree_state())
        .expect("resolves");
    assert!(matches!(
        resolved(&interped, Prop::Background),
        Val::Rgba(_)
    ));
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));
}

// ---------------------------------------------------------------------------
// The built-in table: first-match semantics
// ---------------------------------------------------------------------------

#[test]
fn the_built_in_table_answers_presence() {
    // Presence in the table is the answer: a row fires the layer, its
    // absence quietly misses — the zero-allocation per-frame form.
    let table = DescendantState::new(&[DescendantFact {
        kind: DescendantKind::Element,
        fragment: "*:hover",
    }]);
    let hot = compile("has-hover:p-8", &Config::default()).expect("compiles");
    let interped = interp_descendants(&hot.program, &Env::default(), &table).expect("resolves");
    assert_eq!(resolved(&interped, Prop::PaddingTop), Val::Rem(2.0));
}
