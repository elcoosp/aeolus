//! Consumer demonstration: the GPUI ring polyfill (architecture plan
//! §3.7's `ring-*` row — "polyfill as spread-only `BoxShadow`"). GPUI
//! has no ring primitive, so the four `--tw-ring-*` longhands the
//! registry lowers into compose into the v4 box-shadow list — the
//! offset band and the ring band, spread-only — instead of the silent
//! no-op the un-mapped arm used to be. The demo resolves into a
//! headless `StyleRefinement` the same way a running app's `div()`
//! would consume the program.
//!
//! ```sh
//! cargo run -p aeolus-gpui --example ring
//! ```

use aeolus_gpui::prelude::*;

fn main() {
    // The focus-ring surface a settings input would wear: ring width,
    // ring colour, the offset band's width and colour — all four
    // `--tw-ring-*` longhands in one program.
    let program = tw!("bg-white border border-gray-200 \
                       ring-2 ring-blue-500 ring-offset-2 ring-offset-white \
                       focus:ring-4");

    // The base walk: the composed v4 list — the FIRST layer paints on
    // top, so the white offset band sits over the blue ring band's
    // inner edge, exactly the v4 look.
    let style = gpui::StyleRefinement::default().tw(program);
    println!("── GPUI box shadows from the V1 ring program ──");
    println!("  layers walked:   {}", program.layers.len());
    for (i, shadow) in style
        .box_shadow
        .as_deref()
        .unwrap_or_default()
        .iter()
        .enumerate()
    {
        println!(
            "  shadow [{i}]:       spread {:?}  blur {:?}  inset {}",
            shadow.spread_radius, shadow.blur_radius, shadow.inset
        );
    }

    // The focused state: focus:ring-4 widens the ring band to 4px —
    // the walk re-composes the whole list (offset band unchanged, the
    // ring band's spread = offset + width).
    let focused = gpui::StyleRefinement::default().tw_env(
        program,
        Env {
            focus: true,
            ..Env::default()
        },
    );
    println!("── under focus (focus:ring-4 wins the width) ──");
    for (i, shadow) in focused
        .box_shadow
        .as_deref()
        .unwrap_or_default()
        .iter()
        .enumerate()
    {
        println!(
            "  shadow [{i}]:       spread {:?}  blur {:?}  inset {}",
            shadow.spread_radius, shadow.blur_radius, shadow.inset
        );
    }

    // The bare spelling: v4's `ring` is 1px in the CURRENT text colour
    // — the walk reads the element's own `text-*` from the same
    // cascade as the `currentColor` source.
    let bare = tw!("text-red-500 ring");
    let style = gpui::StyleRefinement::default().tw(bare);
    let shadows = style.box_shadow.as_deref().unwrap_or_default();
    println!("── bare ring = 1px in the current text colour ──");
    println!(
        "  bands:           {} (spread {:?})",
        shadows.len(),
        shadows.first().map(|s| s.spread_radius)
    );
}
