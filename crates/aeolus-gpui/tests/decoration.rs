//! The decoration family's walk contracts (architecture plan §3.7's
//! typography surface — the GPUI half): the three decoration longhands
//! apply through the pinned `Styled` surface's own methods and the
//! walk's direct refinement patches:
//!
//! * `decoration-{color}` → `Styled::text_decoration_color` on the
//!   underline leg (the pinned method's own get-or-insert semantics),
//!   plus the strikethrough leg's recolour when a line-through already
//!   composes (no pinned method; v4's own `text-decoration-color`
//!   reading);
//! * `decoration-{thickness}` → the underline leg's thickness (the
//!   pinned `text_decoration_0..8` methods' semantics), the
//!   strikethrough leg patched only when a line-through already
//!   composes;
//! * `decoration-solid`/`decoration-wavy` → the pinned
//!   `text_decoration_solid`/`text_decoration_wavy` methods.
//!
//! The double/dotted/dashed style keywords and the auto/from-font
//! thickness keywords never reach the walk — the emit gates refuse them
//! (the `tests/gate.rs` battery in aeolus-codegen-gpui pins that half).
//!
//! Written FIRST (the RED spec): the suite fails until the walk maps
//! the decoration longhands.

use aeolus_gpui::prelude::*;
use gpui::px;

#[test]
fn underline_and_wavy_compose() {
    aeolus_gpui::set_env(Default::default()); // the frame-Env escape hatch (plan §3.5's debug-assert keys on it)
    let program = tw!("underline decoration-wavy");
    let style = gpui::StyleRefinement::default().tw(program);
    let underline = style.text.underline.expect("underline composes");
    assert!(underline.wavy, "the wavy keyword composes");
}

#[test]
fn solid_resets_the_wavy_form() {
    aeolus_gpui::set_env(Default::default()); // the frame-Env escape hatch (plan §3.5's debug-assert keys on it)
    let program = tw!("underline decoration-wavy decoration-solid");
    let style = gpui::StyleRefinement::default().tw(program);
    let underline = style.text.underline.expect("underline composes");
    assert!(!underline.wavy, "decoration-solid resets the wavy form");
}

#[test]
fn thickness_composes_on_the_underline_leg() {
    aeolus_gpui::set_env(Default::default()); // the frame-Env escape hatch (plan §3.5's debug-assert keys on it)
    // The pinned `text_decoration_0..8` methods' own reading: the
    // thickness arm lands on the underline style's absolute-px slot.
    let program = tw!("underline decoration-4");
    let style = gpui::StyleRefinement::default().tw(program);
    let underline = style.text.underline.expect("underline composes");
    assert_eq!(
        underline.thickness,
        px(4.0),
        "decoration-4 patches thickness"
    );

    // The integer scale is not limited to the five named methods: the
    // bare integers are v4's own grammar (isPositiveInteger).
    let program = tw!("underline decoration-13");
    let style = gpui::StyleRefinement::default().tw(program);
    let underline = style.text.underline.expect("underline composes");
    assert_eq!(
        underline.thickness,
        px(13.0),
        "decoration-13 patches thickness"
    );
}

#[test]
fn thickness_patches_the_strikethrough_leg_when_it_composes() {
    aeolus_gpui::set_env(Default::default()); // the frame-Env escape hatch (plan §3.5's debug-assert keys on it)
    // The v4 reading: `text-decoration-thickness` recolours/resizes BOTH
    // lines when they compose; alone it creates no line (the pinned
    // methods get-or-insert the underline leg, the walk mirrors them).
    let program = tw!("line-through decoration-4");
    let style = gpui::StyleRefinement::default().tw(program);
    let strikethrough = style.text.strikethrough.expect("line-through composes");
    assert_eq!(
        strikethrough.thickness,
        px(4.0),
        "the thickness reaches the strikethrough leg"
    );
}

#[test]
fn colour_composes_on_both_legs() {
    aeolus_gpui::set_env(Default::default()); // the frame-Env escape hatch (plan §3.5's debug-assert keys on it)
    let program = tw!("underline decoration-red-500");
    let style = gpui::StyleRefinement::default().tw(program);
    let underline = style.text.underline.expect("underline composes");
    assert!(
        underline.color.is_some(),
        "the colour lands on the underline leg"
    );

    // The strikethrough leg recolours only when the line-through
    // already composes (v4's own `text-decoration-color` reading).
    let program = tw!("line-through decoration-red-500");
    let style = gpui::StyleRefinement::default().tw(program);
    let strikethrough = style.text.strikethrough.expect("line-through composes");
    assert!(
        strikethrough.color.is_some(),
        "the colour reaches the strikethrough leg"
    );
    // The frozen palette is one source of truth: both legs carry the
    // SAME frozen red-500 (the baked literal's own colour).
    let program = tw!("underline decoration-red-500");
    let style = gpui::StyleRefinement::default().tw(program);
    let underline = style.text.underline.expect("underline composes");
    assert_eq!(
        strikethrough.color, underline.color,
        "both legs carry the frozen palette colour"
    );
}

#[test]
fn colour_alone_does_not_create_a_line_through() {
    aeolus_gpui::set_env(Default::default()); // the frame-Env escape hatch (plan §3.5's debug-assert keys on it)
    // `decoration-red-500` alone: the underline leg follows the pinned
    // method's get-or-insert (an underline composes — GPUI's own
    // reading of the class), but NO line-through appears (the
    // strikethrough patch is only-if-present, v4-true).
    let program = tw!("decoration-red-500");
    let style = gpui::StyleRefinement::default().tw(program);
    assert!(
        style.text.strikethrough.is_none(),
        "the colour alone never creates a line-through"
    );
    assert!(
        style.text.underline.is_some(),
        "the underline leg get-or-inserts"
    );
}

#[test]
fn the_decoration_family_composes_through_the_macro() {
    aeolus_gpui::set_env(Default::default()); // the frame-Env escape hatch (plan §3.5's debug-assert keys on it)
    // The full family in one string — the frozen program walks all four
    // longhands.
    let program = tw!("underline decoration-wavy decoration-4 decoration-red-500");
    let style = gpui::StyleRefinement::default().tw(program);
    let underline = style.text.underline.expect("underline composes");
    assert!(underline.wavy);
    assert_eq!(underline.thickness, px(4.0));
    assert!(underline.color.is_some());
}
