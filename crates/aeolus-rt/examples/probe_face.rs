//! Consumer demonstration: the host surface's probe face — the
//! [`StaticProgram`] type carries the structural and widest walk
//! entries (`style_struct`, `style_probes_containers_struct_
//! descendants_supports`) beside the style-face methods, so a host
//! threads no slices and no imports beyond the `aeolus_rt` surface it
//! already consumes. The demo wires the five probe seams over a frozen
//! program — the named-container answers ride the
//! [`MeasuredContainers`] hand-off, the very seam the measuring
//! wrapper publishes into — and reads the element, structural and
//! widest faces headless, the walk's reports printed beside the
//! resolved slots.
//!
//! ```sh
//! cargo run -p aeolus-rt --example probe_face
//! ```

use aeolus_compile::{
    Config, DescendantFact, DescendantKind, DescendantState, Env, InterpNote, StructFacts,
    SupportsState,
};
use aeolus_rt::{
    ClassCell, MeasuredContainers, ResolvedStyle, StaticProgram, StyleWalk, program_style_struct,
};

/// The width slot's absolute-px read (`SizedValue::Px` carries the px;
/// the symbolic scales do not answer in px).
fn width_px(style: &ResolvedStyle) -> Option<f32> {
    match style.dimensions.and_then(|d| d.width) {
        Some(aeolus_core::SizedValue::Px(px)) => Some(px),
        _ => None,
    }
}

fn main() {
    // The dynamic lane is the runtime path's constructor: the cell
    // compiles one program carrying a layer per seam family (attribute,
    // named container, structural, descendant, supports) and hands back
    // the frozen form.
    let cell = ClassCell::new();
    let program: StaticProgram = cell
        .get(
            "inline-flex items-center rounded-md bg-red-500 px-4 \
             data-[state=open]:bg-red-500/90 @md/card:w-64 first:w-40 \
             has-focus:opacity-40 supports-[display:grid]:hidden",
            &Config::default(),
        )
        .expect("the cell compiles what lowers");

    // The measured hand-off IS the container probe: the measuring
    // wrapper publishes its layout widths into one handle, the walk
    // reads the same answers through the seam. One publish, the widest
    // face reads it.
    let measured = MeasuredContainers::new();
    let changed = measured.publish("card", 64.0);
    println!("── publish card = 64rem (changed: {changed}) ──");

    let attrs = aeolus_compile::AttrState::new(&[aeolus_compile::AttrBit {
        kind: aeolus_compile::AttrKind::Data,
        name: "state",
        value: "open",
    }]);
    let focused = DescendantState::new(&[DescendantFact {
        kind: DescendantKind::Element,
        fragment: "*:focus",
    }]);
    let grid = SupportsState::new(&["(display:grid)"]);
    let facts = StructFacts::new(0, 2);

    println!("── the widest face: every seam wired ──");
    let (style, walk) = program.style_probes_containers_struct_descendants_supports(
        &Env::default(),
        Some(&attrs),
        Some(&measured),
        Some(&facts),
        Some(&focused),
        Some(&grid),
        None,
    );
    let _: ResolvedStyle = style;
    println!("  width: {:?} px", width_px(&style));
    println!("  opacity: {:?}", style.opacity.map(|o| o.0));
    println!(
        "  walk: notes={} boundaries={} refusals={}",
        walk.notes.len(),
        walk.boundaries.len(),
        walk.refusals.len()
    );
    assert_eq!(
        width_px(&style),
        Some(256.0),
        "@md/card:w-64 fires at the published 64rem (16rem = 256px)"
    );
    assert_eq!(style.opacity.map(|o| o.0), Some(0.4), "has-focus fires");
    assert!(
        !walk.boundaries.is_empty(),
        "the supports layer's hidden keeps its typed boundary"
    );

    // The resize story: the wrapper publishes the new width, the same
    // handle answers, the max-boundary layer stops firing — one
    // publish, no re-compile.
    let changed = measured.publish("card", 32.0);
    println!("── re-publish card = 32rem (changed: {changed}) ──");
    let (narrower, _) = program.style_probes_containers_struct_descendants_supports(
        &Env::default(),
        Some(&attrs),
        Some(&measured),
        Some(&facts),
        Some(&focused),
        Some(&grid),
        None,
    );
    println!("  width: {:?} px", width_px(&narrower));
    assert_eq!(
        width_px(&narrower),
        Some(160.0),
        "below the md boundary the container layer quiets and the \
         structural first:w-40 (10rem) is the standing beat"
    );

    // The structural face on its own: the DOM-position questions the
    // host answers, the free entry's delegate riding the same walk.
    println!("── the structural face: first:w-40 beside the free entry ──");
    let (method_style, method_walk) = program.style_struct(&Env::default(), Some(&facts));
    let (free_style, free_walk) = program_style_struct(
        program.layers,
        program.keywords,
        &Env::default(),
        Some(&facts),
    );
    assert_eq!(
        method_style, free_style,
        "one walk, two spellings: the slots agree"
    );
    assert_eq!(method_walk, free_walk, "the reports agree");
    println!("  width: {:?} px", width_px(&method_style));

    // The narrow faces: a host naming ONE seam threads one Option —
    // the container face here, answering through the same
    // MeasuredContainers hand-off.
    println!("── the narrow container face: one seam, one Option ──");
    measured.publish("card", 64.0);
    let (narrow_style, narrow_walk) =
        program.style_containers(&Env::default(), Some(&measured), None);
    println!("  width: {:?} px", width_px(&narrow_style));
    println!(
        "  walk: notes={} boundaries={} refusals={}",
        narrow_walk.notes.len(),
        narrow_walk.boundaries.len(),
        narrow_walk.refusals.len()
    );
    assert_eq!(
        width_px(&narrow_style),
        Some(256.0),
        "the narrow face answers the same boundary"
    );

    // The sourceless widest keeps the reported boundary — the walk's
    // pre-seam shape, the observability contract the suites pin.
    println!("── the sourceless widest: the reported boundaries ──");
    let (_, walk) = program.style_probes_containers_struct_descendants_supports(
        &Env::default(),
        None,
        None,
        None,
        None,
        None,
        None,
    );
    let reported: Vec<&str> = walk
        .notes
        .iter()
        .map(|note| match note {
            InterpNote::ContainerUnqueried { .. } => "container",
            InterpNote::StructuralUnqueried { .. } => "structural",
            InterpNote::DescendantUnqueried { .. } => "descendant",
            InterpNote::SupportsUnqueried { .. } => "supports",
            _ => "other",
        })
        .collect();
    println!("  notes: {reported:?}");
    for family in ["container", "structural", "descendant", "supports"] {
        assert!(
            reported.contains(&family),
            "the {family} boundary is reported: {reported:?}"
        );
    }

    // The StyleWalk report type rides the re-export whole.
    let _: StyleWalk = walk;
    println!("✓ the probe face answered every family");
}
