//! Consumer demonstration: the GPUI shadow tier (architecture plan §3.7's
//! `shadow-*` row — "direct; note: 1 `Vec` alloc/element"). GPUI's own
//! `box_shadow: Vec<BoxShadow>` carries the v4 band shape — offset,
//! blur, spread, colour, even the `inset` flag — so the `shadow-*`
//! presets, the `shadow-{color}` recolour and the `shadow-inner`
//! placeholder compose natively through the [`ShadowState`]
//! accumulator beside the ring's bands. The demo resolves into a
//! headless `StyleRefinement` the same way a running app's `div()`
//! would consume the program.
//!
//! ```sh
//! cargo run -p aeolus-gpui --example shadow
//! ```

use aeolus_gpui::prelude::*;

fn main() {
    // The card surface a shadcn Card would wear: the two-band lg
    // preset, its `shadow-{color}` recolour and the text colour the
    // `currentColor` band would read — the whole `shadow-*` machinery
    // in one program.
    let program = tw!("bg-white text-red-500 \
                       shadow-lg shadow-red-500/50 \
                       hover:shadow-2xl");

    // The base walk: the composed v4 band list — the preset's geometry
    // (0 10px 15px -3px, 0 4px 6px -4px) with the recolour over every
    // band's colour (v4's `--tw-shadow-colored`; the /50 alpha riding
    // the baked colour).
    let style = gpui::StyleRefinement::default().tw(program);
    println!("── GPUI box shadows from the V1 shadow program ──");
    println!("layers walked:   {}", program.layers.len());
    for (i, shadow) in style
        .box_shadow
        .as_deref()
        .unwrap_or_default()
        .iter()
        .enumerate()
    {
        println!(
            "shadow [{i}]:       offset ({:?}, {:?})  blur {:?}  spread {:?}  inset {}",
            shadow.offset.x,
            shadow.offset.y,
            shadow.blur_radius,
            shadow.spread_radius,
            shadow.inset
        );
    }

    // The hovered state: hover:shadow-2xl replaces the preset wholesale
    // (the per-longhand cascade, last-write-wins) — one band, the
    // audited 0 25px 50px -12px geometry, the recolour still applied
    // (the shadow colour persists across state layers the way CSS's
    // vars do).
    let hovered = gpui::StyleRefinement::default().tw_env(
        program,
        Env {
            hover: true,
            ..Env::default()
        },
    );
    println!("── under hover (hover:shadow-2xl wins the preset) ──");
    for (i, shadow) in hovered
        .box_shadow
        .as_deref()
        .unwrap_or_default()
        .iter()
        .enumerate()
    {
        println!(
            "shadow [{i}]:       offset ({:?}, {:?})  blur {:?}  spread {:?}",
            shadow.offset.x, shadow.offset.y, shadow.blur_radius, shadow.spread_radius
        );
    }

    // The inset placeholder: v4's `inset 0 0 0 0 currentColor` — the
    // pinned `BoxShadow` carries the `inset` flag, the `currentColor`
    // band reading the element's own text colour.
    let inner = tw!("text-red-500 shadow-inner");
    let style = gpui::StyleRefinement::default().tw(inner);
    let shadows = style.box_shadow.as_deref().unwrap_or_default();
    println!("── shadow-inner = the inset placeholder in the text colour ──");
    println!(
        "bands:           {} (inset {}, colour = the text red)",
        shadows.len(),
        shadows.first().map(|s| s.inset).unwrap_or(false)
    );

    // The clear: `shadow-none` lands v4's own `0 0 #0000` transparent
    // band — the composed surface the cascade reads.
    let none = tw!("shadow-lg shadow-none");
    let style = gpui::StyleRefinement::default().tw(none);
    let shadows = style.box_shadow.as_deref().unwrap_or_default();
    println!("── shadow-lg shadow-none = the v4 clear ──");
    println!("bands:           {} (the transparent band)", shadows.len());
}
