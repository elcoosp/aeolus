//! The pseudo-element render-target tier's GPUI half: the static-program
//! walk answers the ELEMENT cascade, and a pseudo-element layer is never
//! element state — every pseudo spelling stays excluded under the walk's
//! `Env` (the interp's `decide` semantics shared verbatim; the
//! CSS/interp arms carry the render targets' own cascades).
//!
//! Written FIRST (the RED spec): the spellings refuse at the fold while
//! the tier is un-landed.

use aeolus_gpui::prelude::*;
use gpui::rems;

#[test]
fn pseudo_layers_stay_out_of_the_element_cascade() {
    let program = tw!("before:p-4");
    let style = gpui::StyleRefinement::default().tw(program);
    assert_eq!(
        style.padding.top, None,
        "a pseudo layer is never element state"
    );
}

#[test]
fn the_base_cascade_answers_beside_an_excluded_pseudo_layer() {
    let program = tw!("p-8 before:p-4");
    let style = gpui::StyleRefinement::default().tw(program);
    assert_eq!(
        style.padding.top,
        Some(rems(2.0).into()),
        "the base layer only"
    );
}

#[test]
fn every_pseudo_spelling_stays_excluded() {
    // `tw!` freezes a string literal — one call per spelling (the
    // closed set walked, the walk's exclusion answer pinned each time).
    for style in [
        gpui::StyleRefinement::default().tw(tw!("after:p-4")),
        gpui::StyleRefinement::default().tw(tw!("marker:p-4")),
        gpui::StyleRefinement::default().tw(tw!("first-letter:p-4")),
        gpui::StyleRefinement::default().tw(tw!("first-line:p-4")),
        gpui::StyleRefinement::default().tw(tw!("file:p-4")),
        gpui::StyleRefinement::default().tw(tw!("backdrop:p-4")),
        gpui::StyleRefinement::default().tw(tw!("details-content:p-4")),
    ] {
        assert_eq!(
            style.padding.top, None,
            "a pseudo layer is never element state"
        );
    }
}
