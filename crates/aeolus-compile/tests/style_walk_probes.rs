//! The program lane's full probe seam (the `program_style_probes_*`
//! entry this file pins): the widest `*_with` growth — the named
//! container, descendant `has-*` and `@supports` seams the interp arm
//! already answers ride the style walk's `decide` verdicts too, so a
//! frozen program answers those layers the way the effective-
//! declaration walk does instead of reporting them unqueried forever.
//!
//! The seam contract the pins encode:
//!
//! * a NAMED container layer asks the [`ContainerProbe`] seam (the
//!   built-in [`ContainerState`] answering): a hit fires by the same
//!   AT-and-above / strictly-below boundary rules the interp pins,
//!   a container the source does not know is a QUIET miss, and the
//!   SOURCELESS form keeps the reported boundary
//!   ([`InterpNote::ContainerUnqueried`]) — the walk's pre-seam shape,
//!   unchanged for callers that pass no source;
//! * a `has-*` layer asks the [`DescendantProbe`] seam (a YES fires, a
//!   NO is a quiet skip, the sourceless form keeps
//!   [`InterpNote::DescendantUnqueried`]);
//! * a `supports-*` layer asks the [`SupportsProbe`] seam (a YES
//!   fires, a NO is a quiet skip, the sourceless form keeps
//!   [`InterpNote::SupportsUnqueried`]);
//! * the widest form AGREES with the narrow `*_with` entries: the same
//!   program + env + seams, the same `(ResolvedStyle, StyleWalk)` —
//!   one `decide`, two walks, no third verdict path.
//!
//! Written FIRST (the RED spec): the suite does not compile against
//! the pre-seam walk — the widest entry does not exist yet, the
//! compile failure IS the capture.

use aeolus_compile::{
    AttrBit, AttrKind, AttrState, Config, ContainerBound, ContainerState, DescendantFact,
    DescendantKind, DescendantState, Env, InterpNote, StructFacts, SupportsState, TokenSource,
    keyword_alignment, keyword_texts, program_style,
    program_style_probes_containers_struct_descendants_supports, program_style_with,
};
use aeolus_core::plan::ResolvedStyle;
use aeolus_theme::{ThemeSource, TokenTable};

/// The baked shadcn LIGHT table (T0's shape: every colour token bakes
/// to a literal) — the same construction the `tw!` macro's freeze walk
/// runs.
fn baked_table() -> TokenTable {
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    TokenTable::baked(&source).expect("the baked shadcn table builds")
}

/// Compile a class string the way `tw!` does and freeze it into the
/// static form the macro emits: the interned layers plus the row texts
/// the keyword slots index.
fn frozen(classes: &str) -> (&'static [aeolus_ir::Layer], &'static [&'static str]) {
    let config = Config::default();
    let compiled =
        aeolus_compile::compile_theme(classes, &baked_table(), &config).expect("the class lowers");
    assert!(
        compiled.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        compiled.diagnostics
    );
    let annotations = keyword_texts(classes, &config).expect("non-empty input annotates");
    let keywords = keyword_alignment(&compiled, &annotations);
    (
        compiled.program.layers,
        Box::leak(keywords.into_boxed_slice()),
    )
}

/// bg-primary under the shadcn light palette: oklch(0.205 0 0) ≈
/// #171717 — the same constant the plan bridge battery pins.
const PRIMARY: [f32; 4] = [23.0 / 255.0, 23.0 / 255.0, 23.0 / 255.0, 1.0];

/// red-500's gamut-mapped sRGBA (the interp batteries' own constant):
/// oklch(0.637 0.237 25.331) → #fb2c36.
const RED_500: [f32; 4] = [251.0 / 255.0, 44.0 / 255.0, 54.0 / 255.0, 1.0];

/// The sRGBA a baked background carries, for the quantization-tolerant
/// colour reads (both lanes convert through the audited parser, so the
/// agreement is exact up to the 1e-4 Oklab equality epsilon).
fn srgba(c: aeolus_core::Color) -> [f32; 4] {
    c.to_srgba()
}

#[test]
fn named_container_layers_fire_through_the_seam() {
    let (layers, keywords) = frozen("@md/card:w-64");

    // The sourceless form keeps the reported boundary — the walk's
    // pre-seam shape, unchanged for callers that pass no source.
    let (resting, walk) = program_style(layers, keywords, &Env::default());
    assert!(resting.dimensions.is_none(), "the layer stayed quiet");
    assert!(
        walk.notes
            .iter()
            .any(|n| matches!(n, InterpNote::ContainerUnqueried { .. })),
        "the unqueried container is reported: {walk:?}"
    );

    // The seam answers: the card container is 64rem, AT and above the
    // md boundary (48rem) — the width fires (w-64 = 16rem = 256px).
    let card = ContainerState::new(&[ContainerBound {
        name: "card",
        rem: 64.0,
    }]);
    let (fired, walk) = program_style_probes_containers_struct_descendants_supports(
        layers,
        keywords,
        &Env::default(),
        None,
        Some(&card),
        None,
        None,
        None,
        None,
    );
    assert!(walk.notes.is_empty(), "a hit is no note: {walk:?}");
    let dims = fired.dimensions.expect("the container layer fires");
    assert_eq!(
        dims.width,
        Some(aeolus_core::SizedValue::Px(256.0)),
        "w-64 = 16rem bakes through the env's rem: {dims:?}"
    );

    // The answered miss is CSS-quiet — and NOT a note (the container
    // does not hold, a selector that does not match).
    let narrow = ContainerState::new(&[ContainerBound {
        name: "card",
        rem: 32.0,
    }]);
    let (quiet, walk) = program_style_probes_containers_struct_descendants_supports(
        layers,
        keywords,
        &Env::default(),
        None,
        Some(&narrow),
        None,
        None,
        None,
        None,
    );
    assert!(quiet.dimensions.is_none(), "below the boundary stays quiet");
    assert!(walk.notes.is_empty(), "an answered miss is quiet: {walk:?}");

    // The source knows the NAME but the walk still asks: an unknown
    // name is a QUIET miss (the seam's own reading), never a note.
    let other = ContainerState::new(&[ContainerBound {
        name: "sidebar",
        rem: 64.0,
    }]);
    let (_, walk) = program_style_probes_containers_struct_descendants_supports(
        layers,
        keywords,
        &Env::default(),
        None,
        Some(&other),
        None,
        None,
        None,
        None,
    );
    assert!(
        walk.notes.is_empty(),
        "an unknown name is a quiet miss: {walk:?}"
    );
}

#[test]
fn the_named_max_container_answers_strictly_below() {
    let (layers, keywords) = frozen("@max-md/card:w-32");

    // At 48rem exactly: the max form is STRICTLY below — quiet, no note.
    let boundary = ContainerState::new(&[ContainerBound {
        name: "card",
        rem: 48.0,
    }]);
    let (at, walk) = program_style_probes_containers_struct_descendants_supports(
        layers,
        keywords,
        &Env::default(),
        None,
        Some(&boundary),
        None,
        None,
        None,
        None,
    );
    assert!(at.dimensions.is_none(), "the max form is strictly below");
    assert!(walk.notes.is_empty(), "an answered miss is quiet: {walk:?}");

    // Below the boundary the layer fires (w-32 = 8rem = 128px).
    let below = ContainerState::new(&[ContainerBound {
        name: "card",
        rem: 32.0,
    }]);
    let (fired, _) = program_style_probes_containers_struct_descendants_supports(
        layers,
        keywords,
        &Env::default(),
        None,
        Some(&below),
        None,
        None,
        None,
        None,
    );
    let dims = fired.dimensions.expect("the max layer fires below");
    assert_eq!(
        dims.width,
        Some(aeolus_core::SizedValue::Px(128.0)),
        "w-32 = 8rem bakes through the env's rem: {dims:?}"
    );
}

#[test]
fn has_layers_fire_through_the_seam() {
    let (layers, keywords) = frozen("has-focus:bg-red-500");

    // Sourceless: the reported boundary (the walk's pre-seam shape).
    let (resting, walk) = program_style(layers, keywords, &Env::default());
    assert!(resting.background.is_none(), "the layer stayed quiet");
    assert!(
        walk.notes
            .iter()
            .any(|n| matches!(n, InterpNote::DescendantUnqueried { .. })),
        "the unqueried descendant is reported: {walk:?}"
    );

    // The seam answers: the subtree carries a focused descendant — the
    // layer fires (red-500 gamut-mapped to #fb2c36).
    let focused = DescendantState::new(&[DescendantFact {
        kind: DescendantKind::Element,
        fragment: "*:focus",
    }]);
    let (fired, walk) = program_style_probes_containers_struct_descendants_supports(
        layers,
        keywords,
        &Env::default(),
        None,
        None,
        None,
        Some(&focused),
        None,
        None,
    );
    assert!(walk.notes.is_empty(), "a hit is no note: {walk:?}");
    let bg = srgba(fired.background.expect("the has layer fires").0);
    for (got, want) in bg.iter().zip(RED_500.iter()) {
        assert!((got - want).abs() < 1.0 / 255.0, "got {bg:?}");
    }

    // The answered miss is a QUIET skip — a `:has()` that does not
    // match, never an exclusion note.
    let empty = DescendantState::new(&[]);
    let (_, walk) = program_style_probes_containers_struct_descendants_supports(
        layers,
        keywords,
        &Env::default(),
        None,
        None,
        None,
        Some(&empty),
        None,
        None,
    );
    assert!(walk.notes.is_empty(), "an answered miss is quiet: {walk:?}");
}

#[test]
fn supports_layers_fire_through_the_seam() {
    let (layers, keywords) = frozen("supports-[display:grid]:bg-primary");

    // Sourceless: the reported boundary (the walk's pre-seam shape).
    let (resting, walk) = program_style(layers, keywords, &Env::default());
    assert!(resting.background.is_none(), "the layer stayed quiet");
    assert!(
        walk.notes
            .iter()
            .any(|n| matches!(n, InterpNote::SupportsUnqueried { .. })),
        "the unqueried supports layer is reported: {walk:?}"
    );

    // The seam answers yes: the declaration-form condition wraps in
    // parens (the fold's own normalization) — the layer fires.
    let grid = SupportsState::new(&["(display:grid)"]);
    let (fired, walk) = program_style_probes_containers_struct_descendants_supports(
        layers,
        keywords,
        &Env::default(),
        None,
        None,
        None,
        None,
        Some(&grid),
        None,
    );
    assert!(walk.notes.is_empty(), "a hit is no note: {walk:?}");
    let bg = srgba(fired.background.expect("the supports layer fires").0);
    for (got, want) in bg.iter().zip(PRIMARY.iter()) {
        assert!((got - want).abs() < 1.0 / 255.0, "got {bg:?}");
    }

    // The answered miss is a QUIET skip — an `@supports` block the
    // render target fails, never an exclusion note.
    let none = SupportsState::new(&[]);
    let (_, walk) = program_style_probes_containers_struct_descendants_supports(
        layers,
        keywords,
        &Env::default(),
        None,
        None,
        None,
        None,
        Some(&none),
        None,
    );
    assert!(walk.notes.is_empty(), "an answered miss is quiet: {walk:?}");
}

#[test]
fn the_widest_form_agrees_with_the_narrow_entries() {
    // One program, every seam family on it: the narrow `*_with` form
    // (attr + tokens only) and the widest form MUST produce the same
    // style and the same report when the extra seams ride `None` —
    // one `decide`, two walks, no third verdict path.
    let corpus = "p-4 bg-primary data-[state=open]:bg-red-500 first:w-40 \
                  @md/card:w-64 has-focus:opacity-40 supports-[display:grid]:hidden";
    let (layers, keywords) = frozen(corpus);

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

    let (narrow_style, narrow_walk) = program_style_with(
        layers,
        keywords,
        &Env::default(),
        Some(&attrs),
        Some(&table),
    );
    let (wide_style, wide_walk) = program_style_probes_containers_struct_descendants_supports(
        layers,
        keywords,
        &Env::default(),
        Some(&attrs),
        None,
        None,
        None,
        None,
        Some(&table),
    );
    assert_eq!(narrow_style, wide_style, "the None-seams walk agrees");
    assert_eq!(narrow_walk, wide_walk, "the report agrees too");

    // With every seam wired the walk answers each family: the
    // data- layer fires (red), the container fires (w-64), the
    // structural layer fires after it (w-40, the later beat), the
    // has- fires (opacity 0.4) and the supports layer's hidden keeps
    // its typed boundary (the v0.x slots carry no display slot).
    let (style, walk) = program_style_probes_containers_struct_descendants_supports(
        layers,
        keywords,
        &Env::default(),
        Some(&attrs),
        Some(&card),
        Some(&facts),
        Some(&focused),
        Some(&grid),
        Some(&table),
    );
    let bg = srgba(style.background.expect("the data- layer fires").0);
    assert!(
        (bg[0] - RED_500[0]).abs() < 1.0 / 255.0,
        "data-[state=open] fired: {bg:?}"
    );
    let dims = style.dimensions.expect("the width layers fire");
    assert_eq!(
        dims.width,
        Some(aeolus_core::SizedValue::Px(256.0)),
        "@md/card:w-64 is the LATER beat, the cascade's winner: {dims:?}"
    );
    let opacity = style.opacity.expect("the has layer fires");
    assert!((opacity.0 - 0.4).abs() < 1e-6, "got {}", opacity.0);
    assert!(
        !walk.notes.is_empty() || !walk.boundaries.is_empty(),
        "the unsupported slot reports honestly: {walk:?}"
    );
}

#[test]
fn the_seams_stay_object_safe_for_host_sources() {
    // The dyn-compat contract the backends' entry points rely on (the
    // Option<&dyn> shape the widest form threads).
    fn takes_widest(
        _: Option<&dyn aeolus_compile::AttrProbe>,
        _: Option<&dyn aeolus_compile::ContainerProbe>,
        _: Option<&dyn aeolus_compile::StructProbe>,
        _: Option<&dyn aeolus_compile::DescendantProbe>,
        _: Option<&dyn aeolus_compile::SupportsProbe>,
        _: Option<&dyn TokenSource>,
    ) {
    }
    let attrs = AttrState::new(&[]);
    let card = ContainerState::new(&[]);
    let facts = StructFacts::UNKNOWN;
    let focused = DescendantState::new(&[]);
    let grid = SupportsState::new(&[]);
    takes_widest(
        Some(&attrs),
        Some(&card),
        Some(&facts),
        Some(&focused),
        Some(&grid),
        None,
    );
}

#[test]
fn the_resolved_style_type_stays_comparable() {
    // The equality pin needs ResolvedStyle: PartialEq (the narrow/wide
    // agreement above reads it whole).
    fn assert_eq_style(_: ResolvedStyle) {}
    assert_eq_style(ResolvedStyle::default());
}
