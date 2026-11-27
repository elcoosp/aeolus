//! The builder-chain emitter's parity battery (TDD: written FIRST —
//! the RED spec is the compile failure before the wire-up exists).
//!
//! The differential target the emit (a) END FORM's contract claims:
//! for a string the chain CAN express, the emitted builder-call
//! closure lands the SAME `StyleRefinement` the frozen-program walk
//! lands — field by field (the perf_contract harness's parity-pin
//! idiom: the comparison is only fair when both sides do the same
//! work, so the stateful faces — hover/focus, the accumulators' lane —
//! are the chain gate's typed refusals' business and stay out of these
//! pins; the four native state methods' shape rides the token-level
//! pins in `crates/aeolus-codegen-gpui/tests/builder.rs`).
//!
//! Plan fidelity: the pinned acceptance criterion is the V1
//! architecture plan's emit (a) landing note
//! (`docs/AEOLUS_V1_ARCHITECTURE.md`, §3.2 status): "The backend
//! METHOD calls the emit (a) end form implies (`FlexDirection::Column`,
//! builder chains) remain the P3 codegen crate's surface." The
//! `tw_builder!` + [`TwExt::tw_chain`] pair is that surface's
//! end-to-end face: compile-time method calls, applied with no walk.

use aeolus_gpui::prelude::*;

/// The colour fields' parity: the composed colours reach the walk
/// through the frozen theme's bake — the chain renders the SAME baked
/// bytes as literals — compared as the rendered bytes (the
/// perf_contract harness's colour-identity contract).
fn colour_close(a: gpui::Hsla, b: gpui::Hsla) -> bool {
    u32::from(gpui::Rgba::from(a)) == u32::from(gpui::Rgba::from(b))
}

/// The `Fill` forms the colour fields carry (the walk's own tag — the
/// pinned `Fill` carries the one `Color` variant): both sides solid
/// colours, compared as the rendered bytes (the perf_contract
/// harness's `colour_close_solid` idiom).
fn colour_close_solid(a: &gpui::Fill, b: &gpui::Fill) -> bool {
    let (gpui::Fill::Color(a), gpui::Fill::Color(b)) = (a, b);
    match (a.as_solid(), b.as_solid()) {
        (Some(x), Some(y)) => colour_close(x, y),
        _ => false,
    }
}

#[test]
fn the_button_chain_lands_the_walks_refinement() {
    set_env(Default::default()); // the frame-Env escape hatch (plan §3.5's debug-assert keys on it)
    let walked = gpui::StyleRefinement::default().tw(tw!(
        "inline-flex items-center justify-center gap-2 p-4 rounded-md bg-primary text-primary-foreground"
    ));
    let chained = gpui::StyleRefinement::default().tw_chain(tw_builder!(
        "inline-flex items-center justify-center gap-2 p-4 rounded-md bg-primary text-primary-foreground"
    ));
    assert_eq!(chained.display, walked.display, "button display");
    assert_eq!(
        chained.align_items, walked.align_items,
        "button align_items"
    );
    assert_eq!(
        chained.justify_content, walked.justify_content,
        "button justify_content"
    );
    assert_eq!(chained.gap, walked.gap, "button gap");
    assert_eq!(
        chained.corner_radii, walked.corner_radii,
        "button corner_radii"
    );
    assert_eq!(chained.padding, walked.padding, "button padding");
    assert!(
        match (&chained.background, &walked.background) {
            (Some(a), Some(b)) => colour_close_solid(a, b),
            _ => false,
        },
        "button background: {:?} vs {:?}",
        chained.background,
        walked.background
    );
    assert!(
        match (&chained.text.color, &walked.text.color) {
            (Some(a), Some(b)) => colour_close(*a, *b),
            _ => false,
        },
        "button text colour: {:?} vs {:?}",
        chained.text.color,
        walked.text.color
    );
}

#[test]
fn the_layout_chain_lands_the_walks_refinement() {
    set_env(Default::default()); // the frame-Env escape hatch (plan §3.5's debug-assert keys on it)
    let walked = gpui::StyleRefinement::default().tw(tw!(
        "w-full h-10 px-3 py-2 border border-red-500 rounded-lg"
    ));
    let chained = gpui::StyleRefinement::default().tw_chain(tw_builder!(
        "w-full h-10 px-3 py-2 border border-red-500 rounded-lg"
    ));
    assert_eq!(chained.size, walked.size, "layout size");
    assert_eq!(chained.padding, walked.padding, "layout padding");
    assert_eq!(
        chained.border_widths, walked.border_widths,
        "layout borders"
    );
    assert!(
        match (&chained.border_color, &walked.border_color) {
            (Some(a), Some(b)) => colour_close(*a, *b),
            _ => false,
        },
        "layout border colour: {:?} vs {:?}",
        chained.border_color,
        walked.border_color
    );
    assert_eq!(
        chained.corner_radii, walked.corner_radii,
        "layout corner_radii"
    );
}

#[test]
fn the_typography_chain_lands_the_walks_refinement() {
    set_env(Default::default()); // the frame-Env escape hatch (plan §3.5's debug-assert keys on it)
    let walked = gpui::StyleRefinement::default().tw(tw!(
        "text-sm font-medium underline text-center whitespace-nowrap"
    ));
    let chained = gpui::StyleRefinement::default().tw_chain(tw_builder!(
        "text-sm font-medium underline text-center whitespace-nowrap"
    ));
    assert_eq!(
        chained.text.font_weight, walked.text.font_weight,
        "typo weight"
    );
    assert_eq!(
        chained.text.underline, walked.text.underline,
        "typo underline"
    );
    assert_eq!(
        chained.text.text_align, walked.text.text_align,
        "typo align"
    );
    assert_eq!(
        chained.text.white_space, walked.text.white_space,
        "typo white-space"
    );
    assert_eq!(chained.text.font_size, walked.text.font_size, "typo size");
}

#[test]
fn the_numeric_chain_lands_the_walks_refinement() {
    set_env(Default::default()); // the frame-Env escape hatch (plan §3.5's debug-assert keys on it)
    let walked = gpui::StyleRefinement::default().tw(tw!(
        "opacity-50 line-clamp-2 grid grid-cols-3 cursor-pointer"
    ));
    let chained = gpui::StyleRefinement::default().tw_chain(tw_builder!(
        "opacity-50 line-clamp-2 grid grid-cols-3 cursor-pointer"
    ));
    assert_eq!(chained.opacity, walked.opacity, "numeric opacity");
    assert_eq!(chained.display, walked.display, "numeric display");
    assert_eq!(chained.grid_cols, walked.grid_cols, "numeric grid_cols");
    assert_eq!(chained.mouse_cursor, walked.mouse_cursor, "numeric cursor");
    // The clamp count rides the text style (the walk's usize reading).
    assert_eq!(
        chained.text.line_clamp, walked.text.line_clamp,
        "numeric clamp"
    );
}

#[test]
fn the_cascade_fold_is_last_wins_on_both_lanes() {
    // Two same-condition layers: the walk applies them front-to-back,
    // the chain renders them in order — the later call wins per
    // longhand on BOTH lanes.
    set_env(Default::default()); // the frame-Env escape hatch (plan §3.5's debug-assert keys on it)
    let walked = gpui::StyleRefinement::default().tw(tw!("p-4 p-8"));
    let chained = gpui::StyleRefinement::default().tw_chain(tw_builder!("p-4 p-8"));
    assert_eq!(chained.padding, walked.padding, "fold padding: p-8 wins");
}
