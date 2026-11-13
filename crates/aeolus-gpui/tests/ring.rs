//! The GPUI ring polyfill (architecture plan §3.7's `ring-*` row:
//! "polyfill as spread-only `BoxShadow` — v4 semantics: ring = box-shadow
//! spread"). The `--tw-ring-*` machinery the registry lowers into the
//! four `Ring*` longhands composes into GPUI's `Vec<BoxShadow>` — the
//! offset band and the ring band, in v4's own order — instead of the
//! silent no-op the un-mapped `_ => style` arm used to be.
//!
//! The pins walk real `tw!`-compiled programs (the same path a running
//! app's `div()` consumes) and inspect the applied
//! [`gpui::StyleRefinement`] headless.

use aeolus_gpui::prelude::*;
use gpui::{hsla, px, rgba};

/// The shadcn red-500 bake (#fb2c36) as the Hsla the polyfill emits —
/// the audited parser's gamut-mapped value.
fn red_500() -> gpui::Hsla {
    rgba(0xfb2c_36ff).into()
}

#[test]
fn ring_width_and_color_compose_one_spread_layer() {
    // ring-2 = 2px spread in the ring colour — v4's
    // `0 0 0 calc(var(--tw-ring-offset-width) + var(--tw-ring-width))
    // var(--tw-ring-color)` with the offset width at its 0 default.
    let program = tw!("ring-2 ring-red-500");
    let style = gpui::StyleRefinement::default().tw(program);
    let shadows = style.box_shadow.expect("the ring lowers into box shadows");
    assert_eq!(shadows.len(), 1, "one layer without an offset: {shadows:?}");
    assert_eq!(shadows[0].spread_radius, px(2.0));
    assert_eq!(shadows[0].blur_radius, px(0.0), "spread-only, no blur");
    assert_eq!(shadows[0].offset.x, px(0.0));
    assert_eq!(shadows[0].color, red_500());
}

#[test]
fn the_offset_band_rides_under_the_ring_band() {
    // ring-offset-2 + ring-2: v4's TWO layers — the offset band first
    // (its own colour), the ring band spread over
    // offset-width + ring-width on top.
    let program = tw!("ring-2 ring-red-500 ring-offset-2 ring-offset-white");
    let style = gpui::StyleRefinement::default().tw(program);
    let shadows = style.box_shadow.expect("the ring lowers into box shadows");
    assert_eq!(shadows.len(), 2, "offset band + ring band: {shadows:?}");
    // v4's list order — `var(--tw-ring-offset-shadow), var(--tw-ring-
    // shadow)` — the FIRST layer paints on top, so the offset band sits
    // over the ring band's inner edge (the v4 look). Both are
    // zero-offset/zero-blur spreads.
    assert_eq!(shadows[0].spread_radius, px(2.0), "the offset band");
    assert_eq!(shadows[0].color, rgba(0xffff_ffff).into(), "offset white");
    assert_eq!(shadows[1].spread_radius, px(4.0), "offset + ring width");
    assert_eq!(shadows[1].color, red_500());
}

#[test]
fn bare_ring_falls_back_to_the_current_text_colour() {
    // v4's `--tw-ring-color: currentColor` default: the ring band paints
    // in the element's own text colour, which the walk reads from the
    // SAME cascade (text-red-500 sets Prop::Color).
    let program = tw!("text-red-500 ring");
    let style = gpui::StyleRefinement::default().tw(program);
    let shadows = style.box_shadow.expect("bare ring lowers");
    assert_eq!(shadows.len(), 1);
    assert_eq!(shadows[0].spread_radius, px(1.0), "v4's bare ring = 1px");
    assert_eq!(
        shadows[0].color,
        red_500(),
        "currentColor = the text colour"
    );
}

#[test]
fn ring_0_clears_the_polyfill() {
    // ring-0 sets the width to 0: no band left, no box shadows — the
    // element keeps its plain refinement (the `ring-2 ring-0` override
    // the tw_merge table's own `ring-w` group documents).
    let program = tw!("ring-2 ring-red-500 ring-0");
    let style = gpui::StyleRefinement::default().tw(program);
    assert!(
        style
            .box_shadow
            .as_ref()
            .is_none_or(std::vec::Vec::is_empty),
        "ring-0 leaves no shadow layers: {:?}",
        style.box_shadow
    );
}

#[test]
fn the_ring_state_composes_the_v4_list() {
    // The unit surface of the polyfill: the accumulator the walk feeds,
    // exposed for hosts that pre-build the state.
    let mut state = aeolus_gpui::RingState::default();
    assert!(
        state.shadows().is_empty(),
        "the default state paints nothing"
    );
    state.apply(
        aeolus_ir::Prop::RingOffsetWidth,
        aeolus_ir::Val::Num(2.0),
        "",
    );
    state.apply(
        aeolus_ir::Prop::RingOffsetColor,
        aeolus_ir::Val::Rgba([255, 255, 255, 255]),
        "",
    );
    state.apply(aeolus_ir::Prop::RingWidth, aeolus_ir::Val::Num(3.0), "");
    state.apply(
        aeolus_ir::Prop::RingColor,
        aeolus_ir::Val::Rgba([251, 44, 54, 255]),
        "",
    );
    let shadows = state.shadows();
    assert_eq!(shadows.len(), 2);
    assert_eq!(shadows[0].spread_radius, px(2.0));
    assert_eq!(shadows[0].color, hsla(0.0, 0.0, 1.0, 1.0), "white band");
    assert_eq!(shadows[1].spread_radius, px(5.0));
    assert_eq!(shadows[1].color, red_500());
}

#[test]
fn later_widths_win_the_cascade() {
    // The walk applies layers in cascade order: `ring-2 hover:ring-4`
    // under a hovering env resolves the 4px band (the longhand the LAST
    // fired layer wrote wins, like every other family).
    let program = tw!("ring-2 ring-red-500 hover:ring-4");
    let idle = gpui::StyleRefinement::default().tw(program);
    assert_eq!(
        idle.box_shadow.as_ref().expect("idle ring")[0].spread_radius,
        px(2.0)
    );
    let hovered = gpui::StyleRefinement::default().tw_env(
        program,
        Env {
            hover: true,
            ..Env::default()
        },
    );
    assert_eq!(
        hovered.box_shadow.as_ref().expect("hovered ring")[0].spread_radius,
        px(4.0),
        "hover:ring-4 wins under the hovering env"
    );
}
