//! The GPUI gradient lane (architecture plan §3.7's `bg-linear-*` row —
//!"native for 2 stops; ≥3 stops/radial/conic → diagnostic"). GPUI's
//! `linear_gradient(angle, from, to)` carries exactly the v4 two-stop
//! form, so the walk accumulates the gradient longhands the registry
//! lowers into (`--tw-gradient-from`/`-to` + their positions, the
//! direction row's `BackgroundImage` text) into a [`aeolus_gpui::GradState`]
//! and composes the linear gradient — instead of the silent no-op the
//! un-mapped `_ => style` arm used to be.
//!
//! The pins walk real `tw!`-compiled programs (the same path a running
//! app's `div()` consumes) and inspect the applied
//! [`gpui::StyleRefinement`] headless. GPUI's `Background` fields are
//! `pub(crate)`, so the pins read the crate's own `Debug` render —
//! `LinearGradient(<angle>, <from>, <to>)` — the same surface a
//! developer sees in a refinement dump.

use aeolus_gpui::prelude::*;

/// The linear-gradient Debug prefix a composed gradient renders — the
/// angle first (GPUI's convention: 0° = to top, clockwise, the CSS
/// convention).
fn gradient_debug(program: StaticProgram) -> String {
    let style = gpui::StyleRefinement::default().tw(program);
    let Some(gpui::Fill::Color(background)) = style.background else {
        panic!("the gradient paints a background fill");
    };
    assert!(
        background.as_solid().is_none(),
        "the composed slot is the gradient, not a solid: {background:?}"
    );
    format!("{background:?}")
}

#[test]
fn the_two_stop_gradient_composes_at_the_css_angle() {
    // `bg-linear-to-r` = `to right` = the CSS 90°; the stops carry the
    // baked table's red-500/blue-500 (the ring suite's differential
    // style: no absolute hex pinned here, the stop Debug rides).
    let program = tw!("bg-linear-to-r from-red-500 to-blue-500");
    let debug = gradient_debug(program);
    assert!(
        debug.starts_with("LinearGradient(90"),
        "`to right` composes at 90°: {debug}"
    );
    assert!(debug.contains("LinearColorStop"), "two stops: {debug}");
}

#[test]
fn the_direction_map_is_the_css_convention() {
    // 0° = to top, clockwise: `to top` 0, `to bottom` 180, the corners
    // at 45° steps (`to top left` 315).
    let debug = gradient_debug(tw!("bg-linear-to-t from-red-500 to-blue-500"));
    assert!(
        debug.starts_with("LinearGradient(0"),
        "`to top` composes at 0°: {debug}"
    );
    let debug = gradient_debug(tw!("bg-linear-to-b from-red-500 to-blue-500"));
    assert!(
        debug.starts_with("LinearGradient(180"),
        "`to bottom` composes at 180°: {debug}"
    );
    let debug = gradient_debug(tw!("bg-linear-to-l from-red-500 to-blue-500"));
    assert!(
        debug.starts_with("LinearGradient(270"),
        "`to left` composes at 270°: {debug}"
    );
    let debug = gradient_debug(tw!("bg-linear-to-tl from-red-500 to-blue-500"));
    assert!(
        debug.starts_with("LinearGradient(315"),
        "`to top left` composes at 315°: {debug}"
    );
    let debug = gradient_debug(tw!("bg-linear-to-br from-red-500 to-blue-500"));
    assert!(
        debug.starts_with("LinearGradient(135"),
        "`to bottom right` composes at 135°: {debug}"
    );
}

#[test]
fn a_from_only_program_composes_the_transparent_to_stop() {
    // v4's `--tw-gradient-to: transparent` initial: the walk composes
    // the missing edge stop so a lone `from-*` still paints.
    let program = tw!("bg-linear-to-b from-red-500");
    let debug = gradient_debug(program);
    assert!(debug.starts_with("LinearGradient(180"), "{debug}");
    assert!(debug.contains("a: 0.0"), "the transparent to-stop: {debug}");
}

#[test]
fn the_stop_positions_ride_the_stops() {
    // `from-20%` patches the from stop's position (GPUI's stop
    // percentage is the 0..1 fraction).
    let program = tw!("bg-linear-to-r from-red-500 from-20% to-blue-500");
    let debug = gradient_debug(program);
    assert!(
        debug.contains("percentage: 0.2"),
        "the from-20% position rides: {debug}"
    );
}

#[test]
fn bg_none_never_paints_a_gradient() {
    let program = tw!("bg-none");
    let style = gpui::StyleRefinement::default().tw(program);
    assert!(
        style.background.is_none(),
        "the explicit clear: {:?}",
        style.background
    );
}

#[test]
fn the_state_unit_composes_from_the_longhands() {
    // The accumulator's own surface (the RingState suite's shape): the
    // walk feeds the longhands, the state composes the v4 form.
    use aeolus_gpui::GradState;
    use aeolus_ir::{Prop, Val};

    let mut state = GradState::default();
    state.apply(
        Prop::BackgroundImage,
        Val::Kw(0),
        "linear-gradient(to right, var(--tw-gradient-stops))",
    );
    state.apply(Prop::GradientFrom, Val::Rgba([251, 44, 54, 255]), "");
    state.apply(Prop::GradientFromPosition, Val::Pct(20.0), "");
    state.apply(Prop::GradientTo, Val::Rgba([59, 130, 246, 255]), "");

    let debug = format!("{:?}", state.background().expect("the gradient composes"));
    assert!(debug.starts_with("LinearGradient(90"), "{debug}");
    assert!(debug.contains("percentage: 0.2"), "{debug}");
    assert!(!state.is_empty(), "a composed state is not empty");
}

#[test]
fn a_direction_without_stops_composes_nothing() {
    // v4's model: `--tw-gradient-stops` needs the gradient function to
    // paint; a direction row alone (or stops without a direction)
    // compose nothing.
    use aeolus_gpui::GradState;
    use aeolus_ir::{Prop, Val};

    let mut state = GradState::default();
    state.apply(
        Prop::BackgroundImage,
        Val::Kw(0),
        "linear-gradient(to right, var(--tw-gradient-stops))",
    );
    assert!(state.background().is_none(), "no stops, no gradient");

    let mut state = GradState::default();
    state.apply(Prop::GradientFrom, Val::Rgba([251, 44, 54, 255]), "");
    assert!(state.background().is_none(), "no direction, no gradient");
}
