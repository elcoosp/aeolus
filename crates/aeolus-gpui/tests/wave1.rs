//! The parity wave 1's GPUI walk contracts (headless, the
//! `StyleRefinement` walk the suites pin): the native clamp/variant
//! arms land in the element's own slots, the between-child wrapper
//! composes the child margins/borders from the same program, and the
//! static front doors freeze the new families identically.

use aeolus_gpui::prelude::*;
use aeolus_ir::Prop;
use gpui::{BorderStyle, FontFeatures, Length, px, rems};

/// `line-clamp-2` lands natively: the text-style clamp slot AND the
/// overflow-hidden the pinned `Styled::line_clamp` method composes.
#[test]
fn the_clamp_count_lands_natively() {
    aeolus_gpui::set_env(Default::default()); // the frame-Env escape hatch (plan §3.5's debug-assert keys on it)
    let program = tw!("line-clamp-2");
    let style = gpui::StyleRefinement::default().tw(program);
    let text = &style.text;
    assert_eq!(text.line_clamp, Some(2), "the clamp count slots");
    assert_eq!(
        style.overflow.y,
        Some(gpui::Overflow::Hidden),
        "the overflow-hidden the pinned method composes"
    );
}

/// `tabular-nums` + `ordinal` compose ADDITIVELY (the v4 var
/// machinery's reading: one tag per family, both enabled), and
/// `normal-nums` resets the set.
#[test]
fn the_variant_tags_compose_and_reset() {
    aeolus_gpui::set_env(Default::default()); // the frame-Env escape hatch (plan §3.5's debug-assert keys on it)
    let program = tw!("tabular-nums ordinal");
    let style = gpui::StyleRefinement::default().tw(program);
    let features = style.text.font_features.as_ref().expect("the tags compose");
    let tags: Vec<&str> = features
        .tag_value_list()
        .iter()
        .map(|(t, _)| t.as_str())
        .collect();
    assert!(tags.contains(&"tnum"), "tabular: {tags:?}");
    assert!(tags.contains(&"ordn"), "ordinal: {tags:?}");

    let program = tw!("normal-nums");
    let style = gpui::StyleRefinement::default().tw(program);
    let features = style.text.font_features.as_ref().expect("the reset slots");
    assert!(
        features.tag_value_list().is_empty(),
        "normal clears the set: {:?}",
        features.tag_value_list()
    );
}

/// The between-child wrapper composes the child margins (the state the
/// plain lane drops, the wrapper lane consumes — the POLYFILL tier's
/// runtime half).
#[test]
fn the_wrapper_composes_the_between_child_parts() {
    aeolus_gpui::set_env(Default::default()); // the frame-Env escape hatch (plan §3.5's debug-assert keys on it)
    let wrapper = gpui::div().tw_space(tw!(
        "space-x-4 space-y-2 divide-y divide-red-500 divide-dashed"
    ));
    let state = wrapper.state();
    assert_eq!(state.space_x, Some((aeolus_ir::Val::Rem(1.0), "")));
    assert_eq!(state.space_y, Some((aeolus_ir::Val::Rem(0.5), "")));
    assert_eq!(state.divide_y, Some((aeolus_ir::Val::Kw(0), "1px")));
    assert_eq!(state.divide_style, Some("dashed"));
    assert!(
        state.divide_color.is_some(),
        "the shadcn bake bakes the colour"
    );
    assert_eq!(
        state.margin(aeolus_gpui::Axis::X),
        Some(Length::Definite(rems(1.0).into()))
    );
    assert_eq!(
        state.border(aeolus_gpui::Axis::Y),
        Some(gpui::AbsoluteLength::Pixels(px(1.0)))
    );
}

/// The reverse flip composes (the margin moves to the start side — the
/// wrapper's patch reads the flag).
#[test]
fn the_reverse_flip_composes() {
    aeolus_gpui::set_env(Default::default()); // the frame-Env escape hatch (plan §3.5's debug-assert keys on it)
    let wrapper = gpui::div().tw_space(tw!("space-x-2 space-x-reverse"));
    let state = wrapper.state();
    assert!(state.space_x_reverse);
    assert_eq!(state.space_x, Some((aeolus_ir::Val::Rem(0.5), "")));
}

/// The divide widths land through the absolute slot (the px rows).
#[test]
fn the_divide_widths_land_absolute() {
    aeolus_gpui::set_env(Default::default()); // the frame-Env escape hatch (plan §3.5's debug-assert keys on it)
    let wrapper = gpui::div().tw_space(tw!("divide-x-2 divide-y-reverse"));
    let state = wrapper.state();
    assert_eq!(
        state.border(aeolus_gpui::Axis::X),
        Some(gpui::AbsoluteLength::Pixels(px(2.0))),
        "divide-x-2: the 2px scale"
    );
    assert!(state.divide_y_reverse, "the flip flag");
}

/// The between-child longhands do NOT land in the per-element walk
/// (the plain lane's documented drop — the wrapper is the consumer).
#[test]
fn the_plain_lane_drops_the_between_child_parts() {
    aeolus_gpui::set_env(Default::default()); // the frame-Env escape hatch (plan §3.5's debug-assert keys on it)
    let program = tw!("space-x-4");
    let style = gpui::StyleRefinement::default().tw(program);
    assert_eq!(style.margin.right, None);
    assert_eq!(style.margin.left, None);
}

/// The static front doors freeze the wave-1 families identically: the
/// same longhand slots the dynamic walk applies, through
/// `tw_static!`/`variants_static!` (the neutral entries' freeze).
#[test]
fn the_static_front_doors_freeze_the_wave1_families() {
    let program = aeolus_macro::tw_static!("line-clamp-2 tabular-nums space-x-4");
    let base = &program.layers[0];
    let props: Vec<Prop> = base.decls.iter().map(|d| d.prop).collect();
    assert!(props.contains(&Prop::LineClamp), "{props:?}");
    assert!(props.contains(&Prop::FontVariantNumeric), "{props:?}");
    assert!(props.contains(&Prop::SpaceX), "{props:?}");
    // The keyword text rides beside the layer (the clamp reset's
    // per-decl verdict at the gate needs it).
    let keywords = program.keywords;
    assert!(keywords.contains(&"tabular-nums"), "{keywords:?}");
}

/// The wrapper composes the divide STYLE through the pinned surface's
/// closed set (dashed/solid).
#[test]
fn the_divide_style_maps_the_pinned_set() {
    aeolus_gpui::set_env(Default::default()); // the frame-Env escape hatch (plan §3.5's debug-assert keys on it)
    let wrapper = gpui::div().tw_space(tw!("divide-dashed"));
    assert_eq!(wrapper.state().divide_style, Some("dashed"));
    let _ = BorderStyle::Dashed; // the pinned surface's own enum
    let _ = FontFeatures::default(); // the tags' slot type
}
