//! The host surface's probe face (the `StaticProgram` method entries
//! this file pins): the widest walk entry —
//! `style_probes_containers_struct_descendants_supports` — and the
//! structural face — `style_struct` — join the method entries the
//! style-face handoff landed (`style`/`style_with`/
//! `style_pseudo`/`style_pseudo_theme`). The free walk entries
//! (`program_style_struct`, `program_style_probes_containers_struct_
//! descendants_supports`) are the five backend walks' own load-bearing
//! surface, and a host holding the macro's frozen form threaded the
//! frozen slices into them BY HAND — the exact field plumbing the
//! style face's handoff retired for the element walk. The face catches
//! up: every walk entry the program lane carries rides the type the
//! host already consumes, the slices threaded once by the methods.
//!
//! The contract the pins encode: the methods are the free functions'
//! DELEGATES — same program, same env, same seams, the same
//! `(ResolvedStyle, StyleWalk)` answer, byte-equal (the differential
//! pin reads both shapes whole) — the probe seams' verdicts stay the
//! interp arm's own `decide` (one implementation, two spellings), the
//! theme forms resolve through the widest form's token tail, and the
//! sourceless forms keep the reported boundaries (the walk's pre-seam
//! shape, unchanged for callers that pass no source).
//!
//! Written FIRST (the RED spec): the suite does not compile against
//! the pre-seam surface — the methods do not exist yet, the compile
//! failure IS the capture.

use aeolus_compile::{
    AttrBit, AttrKind, AttrState, Config, ContainerBound, ContainerState, DescendantFact,
    DescendantKind, DescendantState, Env, InterpNote, StructFacts, SupportsState,
};
use aeolus_ir::Prop;
use aeolus_rt::{
    ClassCell, StaticProgram, program_style_containers, program_style_descendants,
    program_style_probes_containers_struct_descendants_supports, program_style_struct,
    program_style_supports,
};
use aeolus_theme::{ThemeSource, TokenTable};

/// The baked shadcn LIGHT table (T0's shape: every colour token bakes
/// to a literal) — the same construction the `tw!` macro's freeze walk
/// runs. Its handle is also the widest form's token tail (`TokenTable`
/// is the built-in [`TokenSource`]).
fn baked_table() -> TokenTable {
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    TokenTable::baked(&source).expect("the baked shadcn table builds")
}

/// Build a static program through the dynamic lane's own compile —
/// the ClassCell is the runtime path's constructor (the same compiler
/// the macro runs), so the pin's input is a REAL frozen program.
fn frozen(classes: &str) -> StaticProgram {
    let cell = ClassCell::new();
    let program = cell
        .get_theme(classes, &baked_table(), &Config::default())
        .expect("the cell compiles what lowers");
    assert!(
        !program.layers.is_empty(),
        "the corpus carries layers: {classes}"
    );
    program
}

#[test]
fn the_style_struct_method_is_the_structural_walks_delegate() {
    let program = frozen("first:p-4 p-2");
    let env = Env::default();
    let facts = StructFacts::new(0, 2);

    // The method and the free entry answer byte-equal on the same
    // seams: one walk, two spellings, no third verdict path.
    let (method_style, method_walk) = program.style_struct(&env, Some(&facts));
    let (free_style, free_walk) =
        program_style_struct(program.layers, program.keywords, &env, Some(&facts));
    assert_eq!(method_style, free_style, "the slots agree");
    assert_eq!(method_walk, free_walk, "the reports agree");
    assert!(
        method_style.padding.is_some_and(|p| p.top == 16.0),
        "the structural layer fired: first:p-4 beats the resting p-2: {method_style:?}"
    );
    assert!(method_walk.is_empty(), "a clean walk on the answered seam");

    // The UNKNOWN facts are the quiet miss (no positional selector
    // matches "unknown") — the resting p-2 stands, no note.
    let (quiet, walk) = program.style_struct(&env, Some(&StructFacts::UNKNOWN));
    assert!(quiet.padding.is_some_and(|p| p.top == 8.0), "p-2 stands");
    assert!(
        walk.notes.is_empty(),
        "the UNKNOWN facts are a quiet miss: {walk:?}"
    );

    // The sourceless form keeps the reported boundary — the walk's
    // pre-seam shape: the honest exclusion, reported.
    let (resting, walk) = program.style_struct(&env, None);
    assert!(
        walk.notes
            .iter()
            .any(|n| matches!(n, InterpNote::StructuralUnqueried { .. })),
        "the unqueried structural layer is reported: {walk:?}"
    );
    assert!(resting.padding.is_some_and(|p| p.top == 8.0));
}

#[test]
fn the_widest_method_threads_the_full_probe_surface() {
    // One program, every seam family on it — the widest entry's own
    // corpus grown by the attribute layer.
    let program = frozen(
        "p-4 bg-primary data-[state=open]:bg-red-500 \
         @md/card:w-64 has-focus:opacity-40 supports-[display:grid]:hidden",
    );
    let env = Env::default();
    let attrs = AttrState::new(&[AttrBit {
        kind: AttrKind::Data,
        name: "state",
        value: "open",
    }]);
    let card = ContainerState::new(&[ContainerBound {
        name: "card",
        rem: 64.0,
    }]);
    let facts = StructFacts::new(0, 2);
    let focused = DescendantState::new(&[DescendantFact {
        kind: DescendantKind::Element,
        fragment: "*:focus",
    }]);
    let grid = SupportsState::new(&["(display:grid)"]);
    let table = baked_table();

    let (method_style, method_walk) = program.style_probes_containers_struct_descendants_supports(
        &env,
        Some(&attrs),
        Some(&card),
        Some(&facts),
        Some(&focused),
        Some(&grid),
        Some(&table),
    );
    let (free_style, free_walk) = program_style_probes_containers_struct_descendants_supports(
        program.layers,
        program.keywords,
        &env,
        Some(&attrs),
        Some(&card),
        Some(&facts),
        Some(&focused),
        Some(&grid),
        Some(&table),
    );
    assert_eq!(method_style, free_style, "the slots agree");
    assert_eq!(method_walk, free_walk, "the reports agree");

    // The wired verdicts fire: the container's width, the supports
    // layer's display, the attribute layer's background, the
    // descendant layer's opacity.
    let dims = method_style
        .dimensions
        .expect("the container layer fires through the method");
    assert_eq!(
        dims.width,
        Some(aeolus_core::SizedValue::Px(256.0)),
        "w-64 = 16rem bakes through the env's rem: {dims:?}"
    );
    assert!(
        method_style.background.is_some(),
        "the attribute layer fires: {method_style:?}"
    );
    assert_eq!(
        method_style.opacity,
        Some(aeolus_core::plan::Opacity(0.4)),
        "the descendant layer fires: {method_style:?}"
    );
    // The supports layer FIRES, and its `hidden` keeps the typed
    // boundary the compile-side suite pins (the v0.x slots carry no
    // display slot) — the honest report, never a silent drop.
    assert!(
        !method_walk.boundaries.is_empty(),
        "the display slot reports its boundary: {method_walk:?}"
    );
    assert!(
        method_walk.notes.is_empty(),
        "every seam answered: {method_walk:?}"
    );
}

#[test]
fn the_sourceless_widest_keeps_the_reported_boundaries() {
    // The method with every seam riding None is the walk's pre-seam
    // shape: the probe layers never fire and each one REPORTS — the
    // observability contract the interp arm pins, carried to the host
    // face unchanged.
    let program = frozen("p-4 @md/card:w-64 has-focus:opacity-40 supports-[display:grid]:hidden");
    let (style, walk) = program.style_probes_containers_struct_descendants_supports(
        &Env::default(),
        None,
        None,
        None,
        None,
        None,
        None,
    );
    assert!(
        walk.notes
            .iter()
            .any(|n| matches!(n, InterpNote::ContainerUnqueried { .. })),
        "the unqueried container is reported: {walk:?}"
    );
    assert!(
        walk.notes
            .iter()
            .any(|n| matches!(n, InterpNote::DescendantUnqueried { .. })),
        "the unqueried descendant is reported: {walk:?}"
    );
    assert!(
        walk.notes
            .iter()
            .any(|n| matches!(n, InterpNote::SupportsUnqueried { .. })),
        "the unqueried supports layer is reported: {walk:?}"
    );
    assert!(style.dimensions.is_none(), "the quiet layers stay quiet");
    assert!(style.padding.is_some(), "the element's own layer lands");
}

#[test]
fn the_widest_methods_token_tail_resolves_the_theme_forms() {
    // The widest form's token tail is the `_with` convention kept: the
    // theme forms the compiled program carries (the T1
    // [`Val::ModePair`] slots the DARK-pair source's bake leaves to the
    // walk) resolve through the source — the themed face of the probe
    // walk, no separate entry. The light-only source's bake lands T0
    // literals (no walk-time forms); the dark pair is the tier that
    // needs the source.
    let cell = ClassCell::new();
    let dark_source = aeolus_theme::ThemeSource::shadcn_dark().expect("embedded palettes parse");
    let dark_table = TokenTable::baked(&dark_source).expect("the baked dark table builds");
    let program = cell
        .get_theme(
            "bg-primary p-4 @md/card:w-64",
            &dark_table,
            &Config::default(),
        )
        .expect("the themed cell compiles");
    let card = ContainerState::new(&[ContainerBound {
        name: "card",
        rem: 64.0,
    }]);

    let (fired, walk) = program.style_probes_containers_struct_descendants_supports(
        &Env::default(),
        None,
        Some(&card),
        None,
        None,
        None,
        Some(&dark_table),
    );
    assert!(fired.background.is_some(), "the token resolved: {walk:?}");
    assert!(
        walk.refusals.is_empty(),
        "the tail answers every theme form: {walk:?}"
    );
    assert!(walk.notes.is_empty(), "the container answered: {walk:?}");

    // The themeless form refuses per longhand instead — the report the
    // themeless entries document, riding the same method.
    let (refused, walk) = program.style_probes_containers_struct_descendants_supports(
        &Env::default(),
        None,
        Some(&card),
        None,
        None,
        None,
        None,
    );
    assert!(refused.background.is_none(), "no source, no resolution");
    assert!(
        walk.refusals
            .iter()
            .any(|(prop, _)| *prop == Prop::Background),
        "the theme refusal names the longhand: {walk:?}"
    );
}

/// Micro-debt: the frozen program's IR view (`StaticProgram::program`,
/// the const [`aeolus_ir::Program`] constructor the codegen seams read
/// through) had no coverage anywhere in the workspace — the view is
/// the layer slices' own alias, and the pin holds it to that.
#[test]
fn the_program_view_is_the_layer_slices_own_alias() {
    let program = frozen("p-4 hidden");
    let view = program.program();
    assert_eq!(
        view.layers.len(),
        program.layers.len(),
        "the view carries the same layers"
    );
    assert_eq!(
        view.layers, program.layers,
        "the view is the frozen slices' alias, never a copy"
    );
}

// ---------------------------------------------------------------------------
// The narrow probe faces (M2): the interp arm's per-seam entries, the
// program lane's parity
// ---------------------------------------------------------------------------

#[test]
fn the_narrow_container_face_answers_the_named_boundary() {
    let program = frozen("@md/card:w-64");
    let env = Env::default();
    let card = ContainerState::new(&[ContainerBound {
        name: "card",
        rem: 64.0,
    }]);
    let table = baked_table();

    // The method and the free entry answer byte-equal: one walk, two
    // spellings.
    let (method_style, method_walk) = program.style_containers(&env, Some(&card), Some(&table));
    let (free_style, free_walk) = program_style_containers(
        program.layers,
        program.keywords,
        &env,
        Some(&card),
        Some(&table),
    );
    assert_eq!(method_style, free_style, "the slots agree");
    assert_eq!(method_walk, free_walk, "the reports agree");

    // The narrow face is the widest one with the seams it does not
    // name riding None — one `decide`, no third verdict path.
    let (wide_style, wide_walk) = program.style_probes_containers_struct_descendants_supports(
        &env,
        None,
        Some(&card),
        None,
        None,
        None,
        Some(&table),
    );
    assert_eq!(method_style, wide_style, "the narrow face agrees");
    assert_eq!(method_walk, wide_walk, "the narrow report agrees");
    assert_eq!(
        method_style.dimensions.and_then(|d| d.width),
        Some(aeolus_core::SizedValue::Px(256.0)),
        "the container layer fires through the narrow face"
    );
    assert!(method_walk.is_empty(), "a clean walk: {method_walk:?}");

    // The sourceless form keeps the reported boundary.
    let (_, walk) = program.style_containers(&env, None, None);
    assert!(
        walk.notes
            .iter()
            .any(|n| matches!(n, InterpNote::ContainerUnqueried { .. })),
        "the unqueried container is reported: {walk:?}"
    );
}

#[test]
fn the_narrow_descendant_face_answers_the_has_layers() {
    let program = frozen("has-focus:opacity-40");
    let env = Env::default();
    let focused = DescendantState::new(&[DescendantFact {
        kind: DescendantKind::Element,
        fragment: "*:focus",
    }]);

    let (method_style, method_walk) = program.style_descendants(&env, Some(&focused), None);
    let (free_style, free_walk) =
        program_style_descendants(program.layers, program.keywords, &env, Some(&focused), None);
    assert_eq!(method_style, free_style, "the slots agree");
    assert_eq!(method_walk, free_walk, "the reports agree");
    let (wide_style, wide_walk) = program.style_probes_containers_struct_descendants_supports(
        &env,
        None,
        None,
        None,
        Some(&focused),
        None,
        None,
    );
    assert_eq!(method_style, wide_style, "the narrow face agrees");
    assert_eq!(method_walk, wide_walk, "the narrow report agrees");
    assert_eq!(
        method_style.opacity,
        Some(aeolus_core::plan::Opacity(0.4)),
        "the has layer fires through the narrow face"
    );
    assert!(method_walk.is_empty(), "a clean walk: {method_walk:?}");

    // The sourceless form keeps the reported boundary.
    let (_, walk) = program.style_descendants(&env, None, None);
    assert!(
        walk.notes
            .iter()
            .any(|n| matches!(n, InterpNote::DescendantUnqueried { .. })),
        "the unqueried descendant is reported: {walk:?}"
    );
}

#[test]
fn the_narrow_supports_face_answers_the_capability_layers() {
    let program = frozen("supports-[display:grid]:hidden");
    let env = Env::default();
    let grid = SupportsState::new(&["(display:grid)"]);

    let (method_style, method_walk) = program.style_supports(&env, Some(&grid), None);
    let (free_style, free_walk) =
        program_style_supports(program.layers, program.keywords, &env, Some(&grid), None);
    assert_eq!(method_style, free_style, "the slots agree");
    assert_eq!(method_walk, free_walk, "the reports agree");
    let (wide_style, wide_walk) = program.style_probes_containers_struct_descendants_supports(
        &env,
        None,
        None,
        None,
        None,
        Some(&grid),
        None,
    );
    assert_eq!(method_style, wide_style, "the narrow face agrees");
    assert_eq!(method_walk, wide_walk, "the narrow report agrees");

    // The supports layer fires, and its `hidden` keeps the typed
    // boundary (the v0.x slots carry no display slot — the
    // compile-side suite's own pin).
    assert!(
        !method_walk.boundaries.is_empty(),
        "the display slot reports its boundary: {method_walk:?}"
    );

    // The sourceless form keeps the reported boundary.
    let (_, walk) = program.style_supports(&env, None, None);
    assert!(
        walk.notes
            .iter()
            .any(|n| matches!(n, InterpNote::SupportsUnqueried { .. })),
        "the unqueried supports layer is reported: {walk:?}"
    );
}
