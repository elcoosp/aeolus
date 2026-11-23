//! The walk's row-text slot resolution (the P0 harness's own finding):
//! the frozen program carries the row texts BESIDE the `Val::Kw` slots
//! (the emit_static contract), and the length/number slots whose row
//! text is a plain px/rem spelling apply through the walk — the bare
//! `border` 1px default, the `border-2` integer scale, the radius
//! scale's `0.375rem`, the composite text sizes' `0.875rem` primary.
//! Before this seam the walk's absolute/definite readers matched
//! `Val::Rem`/`Val::Px` only, so every keyword-slot length silently
//! no-oped (`border-2`, `rounded-md`, `text-sm` — the P0 harness's
//! parity pins caught it).
//!
//! The documented limit stays: the CALC texts (the composite
//! line-height pair's `calc(1.25 / 0.875)`) are the CSS arm's own
//! output — the walk does not evaluate expressions, the field stays
//! unset (never a guessed value).
//!
//! Written FIRST (the RED spec): the suite fails while the walk's
//! length readers ignore the keyword slot.

use aeolus_gpui::prelude::*;
use gpui::{px, rems};

#[test]
fn border_widths_apply_from_the_number_and_keyword_forms() {
    aeolus_gpui::set_env(Default::default()); // the frame-Env escape hatch (plan §3.5's debug-assert keys on it)
    // The integer scale: `border-2` is 2px (v4's own reading — the bare
    // integers are the px scales). The bare root: the 1px default the
    // row table carries.
    let program = tw!("border-2");
    let style = gpui::StyleRefinement::default().tw(program);
    assert_eq!(
        style.border_widths.top,
        Some(px(2.0).into()),
        "border-2 top"
    );
    assert_eq!(style.border_widths.bottom, Some(px(2.0).into()));

    let program = tw!("border");
    let style = gpui::StyleRefinement::default().tw(program);
    assert_eq!(
        style.border_widths.top,
        Some(px(1.0).into()),
        "border bare 1px"
    );
    assert_eq!(style.border_widths.left, Some(px(1.0).into()));
}

#[test]
fn radius_and_text_size_apply_from_the_row_texts() {
    aeolus_gpui::set_env(Default::default()); // the frame-Env escape hatch (plan §3.5's debug-assert keys on it)
    // The radius scale's rem text (`rounded-md` → 0.375rem) and the
    // composite text size's primary (`text-sm` → 0.875rem) apply from
    // the row texts the frozen program carries.
    let program = tw!("rounded-md");
    let style = gpui::StyleRefinement::default().tw(program);
    assert_eq!(
        style.corner_radii.top_left,
        Some(rems(0.375).into()),
        "rounded-md top-left"
    );
    assert_eq!(style.corner_radii.bottom_right, Some(rems(0.375).into()));

    let program = tw!("text-sm");
    let style = gpui::StyleRefinement::default().tw(program);
    assert_eq!(
        style.text.font_size,
        Some(rems(0.875).into()),
        "text-sm size"
    );

    // The documented limit: the composite pair's line-height text is a
    // calc() — the walk does not evaluate expressions, the field stays
    // unset (never a guessed value).
    assert_eq!(
        style.text.line_height, None,
        "the calc text is the CSS arm's"
    );
}

#[test]
fn the_spacing_scale_is_unchanged() {
    aeolus_gpui::set_env(Default::default()); // the frame-Env escape hatch (plan §3.5's debug-assert keys on it)
    // The regression guard: the spacing scale rode `Val::Rem` before the
    // seam and reads identically after it.
    let program = tw!("p-4 gap-2");
    let style = gpui::StyleRefinement::default().tw(program);
    assert_eq!(style.padding.top, Some(rems(1.0).into()));
    assert_eq!(style.gap.height, Some(rems(0.5).into()));
}

/// The TEXT_ALIGN keyword slot applies through the walk's own mapping
/// ([`aeolus_gpui::text_align_value`]) — the helper's first pins (the
/// doc claimed test exposure; nothing pinned the closed set). GPUI's
/// `TextAlign` carries LEFT/CENTER/RIGHT: `start`/`end` fold to the LTR
/// readings and `justify` rides the documented LEFT default (the
/// platform's own closed set).
#[test]
fn text_align_applies_through_the_walks_own_mapping() {
    use gpui::TextAlign;

    aeolus_gpui::set_env(Default::default()); // the frame-Env escape hatch (plan §3.5's debug-assert keys on it)
    let program = tw!("text-center");
    let style = gpui::StyleRefinement::default().tw(program);
    let ts = &style.text;
    assert_eq!(ts.text_align, Some(TextAlign::Center));

    let program = tw!("text-end");
    let style = gpui::StyleRefinement::default().tw(program);
    assert_eq!(
        style.text.text_align,
        Some(TextAlign::Right),
        "`end` is the LTR right"
    );

    let program = tw!("text-start");
    let style = gpui::StyleRefinement::default().tw(program);
    assert_eq!(
        style.text.text_align,
        Some(TextAlign::Left),
        "`start` is the LTR left"
    );

    let program = tw!("text-justify");
    let style = gpui::StyleRefinement::default().tw(program);
    assert_eq!(
        style.text.text_align,
        Some(TextAlign::Left),
        "no GPUI slot for justify - the documented LEFT default, pinned"
    );
}
