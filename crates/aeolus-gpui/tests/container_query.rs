//! The element-bounds CONTAINER WRAPPER at the GPUI host (architecture
//! plan §3.5's `@container` row, the half the docs assign to the host
//! as "the element-bounds WRAPPER — who MEASURES the widths"):
//! [`MeasuredContainers`] is the layout-phase hand-off the
//! [`ContainerQuery`] wrapper publishes its measured width into, and
//! the walk ([`TwExt::tw_containers`]) reads through the ONE
//! [`ContainerProbe`] question — the wrapper never styles, the walk
//! never measures.
//!
//! The pins:
//! * the px→rem conversion the wrapper publishes is the window rem
//!   size's own division, and a degenerate (non-positive) rem size is
//!   NO measurement — the quiet miss, never a half verdict;
//! * a published width drives the walk exactly like a host-supplied
//!   `ContainerState` table — the AT/strictly-below boundary rules are
//!   the viewport breakpoints' own, shared verbatim;
//! * a re-publish follows the resize: the same name carries the NEW
//!   width and the walk's verdicts follow;
//! * a name never published is the CSS-quiet miss — the layers stay
//!   quiet, never an exclusion;
//! * the UNNAMED container forms keep answering from the [`Env`]'s
//!   nearest-container slot — the wrapper is for the NAMED question;
//! * the wrapper element carries its name as its element id (the
//!   stateless-element shape: no per-element state, the shared handle
//!   is the only carrier);
//! * the publish/refresh handshake's change test: re-publishing the
//!   SAME width is not a change (the re-render loop converges), a
//!   different width is.

use aeolus_compile::Env;
use aeolus_gpui::prelude::*;
use gpui::{Display, Element, ElementId, SharedString, px, rems};

#[test]
fn container_rem_is_the_window_rem_sizes_own_division() {
    // 768px at 16px rem: 48rem — the md boundary's own number.
    assert_eq!(container_rem(px(768.0), px(16.0)), Some(48.0));
    // The same width at a doubled rem halves the answer (the rem size
    // is the window's, not a constant).
    assert_eq!(container_rem(px(768.0), px(32.0)), Some(24.0));
    // A degenerate rem size is NO measurement: the quiet miss, never a
    // half verdict from a division the window never defined.
    assert_eq!(container_rem(px(768.0), px(0.0)), None);
}

#[test]
fn published_width_drives_the_walk_like_the_built_in_table() {
    let program = tw!("@md/card:p-4");
    let measured = MeasuredContainers::new();
    measured.publish("card", 48.0);

    let applied = gpui::StyleRefinement::default().tw_containers(program, &measured);
    assert_eq!(
        applied.padding.top,
        Some(rems(1.0).into()),
        "the measured AT-boundary answer fires the layer"
    );

    // The differential pin: the same answer through the borrowed
    // static table gives the same verdict — the handle IS the probe,
    // not a second semantics.
    let rows = [ContainerBound {
        name: "card",
        rem: 48.0,
    }];
    let built_in = ContainerState::new(&rows);
    let via_table = gpui::StyleRefinement::default().tw_containers(program, &built_in);
    assert_eq!(applied.padding, via_table.padding);
}

#[test]
fn republish_follows_the_resize() {
    let program = tw!("@md/card:p-4");
    let measured = MeasuredContainers::new();
    measured.publish("card", 47.9);
    let narrow = gpui::StyleRefinement::default().tw_containers(program, &measured);
    assert_eq!(narrow.padding.top, None, "below the boundary misses");

    measured.publish("card", 48.0);
    let resized = gpui::StyleRefinement::default().tw_containers(program, &measured);
    assert_eq!(
        resized.padding.top,
        Some(rems(1.0).into()),
        "the re-publish carries the NEW width and the verdicts follow"
    );
}

#[test]
fn unpublished_name_is_the_quiet_miss() {
    let program = tw!("@md/card:p-4");
    let measured = MeasuredContainers::new();
    let applied = gpui::StyleRefinement::default().tw_containers(program, &measured);
    assert_eq!(
        applied.padding.top, None,
        "a name never published is the CSS selector that does not match"
    );
}

#[test]
fn unnamed_forms_keep_answering_from_the_env() {
    // The wrapper is for the NAMED question only: the unnamed `@md:`
    // layer answers from the Env's nearest-container slot, untouched
    // by the handle.
    let program = tw!("@md:p-4");
    let env = Env {
        container_rem: Some(48.0),
        ..Env::default()
    };
    let measured = MeasuredContainers::new();
    let applied = gpui::StyleRefinement::default().tw_env_containers(program, env, &measured);
    assert_eq!(
        applied.padding.top,
        Some(rems(1.0).into()),
        "the unnamed form answers from the Env slot"
    );
}

#[test]
fn max_form_fires_strictly_below_the_measured_width() {
    let program = tw!("@max-lg/sidebar:hidden");
    let measured = MeasuredContainers::new();
    measured.publish("sidebar", 63.9);
    let applied = gpui::StyleRefinement::default().tw_containers(program, &measured);
    assert_eq!(applied.display, Some(Display::None), "strictly below fires");

    measured.publish("sidebar", 64.0);
    let applied = gpui::StyleRefinement::default().tw_containers(program, &measured);
    assert_ne!(
        applied.display,
        Some(Display::None),
        "AT the boundary the max form misses"
    );
}

#[test]
fn the_wrapper_carries_its_name_as_its_element_id() {
    let measured = MeasuredContainers::new();
    let wrapper = gpui::div().tw_container("card", &measured);
    assert_eq!(
        wrapper.id(),
        Some(ElementId::Name(SharedString::from("card"))),
        "the name doubles as the element id — the stateless-element shape"
    );
}

#[test]
fn republishing_the_same_width_is_not_a_change() {
    // The publish/refresh handshake's change test: the SAME width is
    // not a change (the re-render loop converges), a different width
    // is — the wrapper refreshes only on a real measurement delta.
    let measured = MeasuredContainers::new();
    assert!(
        measured.publish("card", 48.0),
        "the first publish is a change — the wrapper asks for its first styled frame"
    );
    assert!(
        !measured.publish("card", 48.0),
        "the same width is not a change — the re-render loop converges"
    );
    assert!(
        measured.publish("card", 47.5),
        "a different width IS a change — the resize asks for a re-render"
    );
}
