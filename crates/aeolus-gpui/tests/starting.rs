//! The `starting:` variant's GPUI half: the frame fact the walk answers
//! from the host's `Env` — `starting:p-8` fires only while the frame
//! env carries the element's starting-style bit (the transition's
//! first frame, the host's first-render gate), the unset bit the QUIET
//! miss a frame the element is not in is (the media-feature facts'
//! own reading, the interp verdicts shared verbatim).
//!
//! Written FIRST (the RED spec): the suite fails while the `Cond` set
//! carries no starting-style arm.

use aeolus_gpui::prelude::*;
use gpui::rems;

#[test]
fn the_starting_layer_fires_on_the_frame_fact() {
    let program = tw!("starting:p-4");
    let style = gpui::StyleRefinement::default().tw_env(
        program,
        Env {
            starting_style: true,
            ..Env::default()
        },
    );
    assert_eq!(
        style.padding.top,
        Some(rems(1.0).into()),
        "the starting-style layer answers the frame fact"
    );
}

#[test]
fn the_unset_fact_stays_the_quiet_miss() {
    let program = tw!("starting:p-4");
    let style = gpui::StyleRefinement::default().tw(program);
    assert_eq!(
        style.padding.top, None,
        "an unset frame fact is the quiet miss — never a panic, never a note"
    );
}

#[test]
fn the_base_layer_answers_beside_a_quiet_starting_layer() {
    let program = tw!("p-8 starting:p-4");
    let style = gpui::StyleRefinement::default().tw(program);
    assert_eq!(
        style.padding.top,
        Some(rems(2.0).into()),
        "the base cascade answers beside the quiet miss"
    );
}
