//! The GPUI grid-placement + flex-basis tier's WALK half (architecture
//! plan §3.7's grid row, the application side of the gate flip): the
//! frozen placement declarations apply onto the pinned GPUI surface's
//! own slots — `col_start`/`col_end`/`row_start`/`row_end` (the
//! refinement's `grid_location`), the v4 span fingerprint
//! (`col-span-3`'s equal start/end pair composes GPUI's `Span`
//! placement), and `flex_basis` for the basis family's rem/percentage/
//! arbitrary spellings.
//!
//! The pins:
//!
//! * LINE placements land on the refinement's `grid_location` axis
//!   ranges (`col-start-2 col-end-5` → `Line(2)..Line(5)`);
//! * the SPAN fingerprint — the registry's `col-span`/`row-span` rows
//!   fan ONE class onto (start, end) with EQUAL values — composes the
//!   `GridPlacement::Span` form GPUI's grid carries (a per-decl Line
//!   mapping would read "start 3, end 3" — a degenerate range, never
//!   Tailwind's span 3);
//! * the CASCADE: a later line placement overwrites the span pair's
//!   longhand it carries (the per-longhand cascade — `col-span-3
//!   col-start-2` reads start Line(2) with the span's end);
//! * NEGATIVE lines pass through (`-col-start-2`, the from-end line);
//! * the basis family lands on `flex_basis` in all three value shapes
//!   (spacing rem, fraction percentage, arbitrary percentage);
//! * an axis the program never mentions stays `Auto` — no accidental
//!   placement from an unrelated program.

use aeolus_gpui::prelude::*;
use gpui::{GridPlacement, relative, rems};

fn column(style: &gpui::StyleRefinement) -> (GridPlacement, GridPlacement) {
    let location = style.grid_location.clone().expect("a grid location");
    (location.column.start, location.column.end)
}

fn row(style: &gpui::StyleRefinement) -> (GridPlacement, GridPlacement) {
    let location = style.grid_location.clone().expect("a grid location");
    (location.row.start, location.row.end)
}

#[test]
fn line_placements_land_on_the_grid_location() {
    let style = gpui::StyleRefinement::default().tw(tw!("col-start-2 col-end-5"));
    assert_eq!(
        column(&style),
        (GridPlacement::Line(2), GridPlacement::Line(5))
    );
    let style = gpui::StyleRefinement::default().tw(tw!("row-start-3"));
    assert_eq!(row(&style), (GridPlacement::Line(3), GridPlacement::Auto));
}

#[test]
fn the_span_fingerprint_composes_the_span_form() {
    // `col-span-3` fans to (start, end) = (3, 3) — the v4 `span 3 / span 3`
    // reading. The walk composes GPUI's Span placement from the pair.
    let style = gpui::StyleRefinement::default().tw(tw!("col-span-3"));
    assert_eq!(
        column(&style),
        (GridPlacement::Span(3), GridPlacement::Span(3))
    );
    let style = gpui::StyleRefinement::default().tw(tw!("row-span-2"));
    assert_eq!(
        row(&style),
        (GridPlacement::Span(2), GridPlacement::Span(2))
    );
}

#[test]
fn a_line_placement_cascades_over_the_span_pair() {
    // The per-longhand cascade: `col-span-3 col-start-2` — the later
    // start overwrites the span pair's; the end keeps the span's value.
    let style = gpui::StyleRefinement::default().tw(tw!("col-span-3 col-start-2"));
    assert_eq!(
        column(&style),
        (GridPlacement::Line(2), GridPlacement::Line(3))
    );
}

#[test]
fn negative_lines_pass_through() {
    let style = gpui::StyleRefinement::default().tw(tw!("-col-start-2"));
    assert_eq!(
        column(&style),
        (GridPlacement::Line(-2), GridPlacement::Auto)
    );
}

#[test]
fn the_basis_family_lands_on_flex_basis() {
    let style = gpui::StyleRefinement::default().tw(tw!("basis-16"));
    assert_eq!(style.flex_basis, Some(rems(4.0).into()), "basis-16 is 4rem");

    let style = gpui::StyleRefinement::default().tw(tw!("basis-1/2"));
    assert_eq!(
        style.flex_basis,
        Some(relative(0.5).into()),
        "basis-1/2 is 50%"
    );

    let style = gpui::StyleRefinement::default().tw(tw!("basis-[30%]"));
    assert_eq!(
        style.flex_basis,
        Some(relative(0.3).into()),
        "basis-[30%] is 30%"
    );
}

#[test]
fn an_unmentioned_axis_stays_auto() {
    // No accidental placement: a padding-only program leaves the grid
    // location untouched.
    let style = gpui::StyleRefinement::default().tw(tw!("p-4"));
    assert!(style.grid_location.is_none());
}
