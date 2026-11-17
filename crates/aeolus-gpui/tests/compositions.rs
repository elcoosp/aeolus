//! The composition tier's GPUI half: the frame facts the walk answers
//! from the host's `Env` — `group-active:p-4` fires only while the
//! frame env carries a `.group` ancestor's active bit, the unset bit
//! the QUIET miss a frame the composition does not hold in is (the
//! starting tier's own reading, the interp verdicts shared verbatim).
//!
//! Written FIRST (the RED spec): the suite fails while the `Cond` set
//! carries no composition arms.

use aeolus_gpui::prelude::*;
use gpui::rems;

#[test]
fn the_group_composition_fires_on_the_frame_fact() {
    let program = tw!("group-active:p-4");
    let style = gpui::StyleRefinement::default().tw_env(
        program,
        Env {
            group_active: true,
            ..Env::default()
        },
    );
    assert_eq!(
        style.padding.top,
        Some(rems(1.0).into()),
        "the group-active layer answers the frame fact"
    );
}

#[test]
fn the_peer_composition_fires_on_the_frame_fact() {
    let program = tw!("peer-disabled:p-2");
    let style = gpui::StyleRefinement::default().tw_env(
        program,
        Env {
            peer_disabled: true,
            ..Env::default()
        },
    );
    assert_eq!(
        style.padding.top,
        Some(rems(0.5).into()),
        "the peer-disabled layer answers the frame fact"
    );
}

#[test]
fn the_unset_facts_stay_the_quiet_miss() {
    let program = tw!("group-active:p-4 peer-disabled:p-2");
    let style = gpui::StyleRefinement::default().tw(program);
    assert_eq!(
        style.padding.top, None,
        "an unset composition fact is the quiet miss — never a panic, never a note"
    );
}

#[test]
fn the_base_layer_answers_beside_a_quiet_composition() {
    let program = tw!("p-8 group-checked:p-4");
    let style = gpui::StyleRefinement::default().tw(program);
    assert_eq!(
        style.padding.top,
        Some(rems(2.0).into()),
        "the base cascade answers beside the quiet miss"
    );
}
