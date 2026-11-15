//! Consumer demonstration: the GPUI filter tier's polyfill half
//! (architecture plan §3.7's filter row — the landed half of the
//! "polyfill-wrapper backlog"). GPUI has no `filter` slot, but the
//! element's own colour slots ARE the paint a CSS filter recolours, so
//! the seven colour-matrix primitives compose through the CSS
//! shorthand's sRGB matrices, the `drop-shadow-*` presets map onto
//! GPUI's own `Vec<BoxShadow>` (the blur radius the function's third
//! token names), and `filter-none` is the reset — the spatial
//! (`blur-*`) and `backdrop-*` halves keep the typed refusal. The demo
//! resolves into a headless `StyleRefinement` the same way a running
//! app's `div()` would consume the program.
//!
//! ```sh
//! cargo run -p aeolus-gpui --example filters
//! ```

use aeolus_gpui::prelude::*;

fn main() {
    // The image-card surface a gallery would wear: the full matrix
    // chain over a coloured surface — brightness, contrast,
    // grayscale, hue-rotate, invert, saturate, sepia in one program.
    let chain = tw!("bg-red-500 text-white border-red-500 \
                     brightness-110 contrast-125 grayscale \
                     hue-rotate-90 invert saturate-150 sepia-50");

    let style = gpui::StyleRefinement::default().tw(chain);
    println!("── GPUI filter chain from the V1 program ──");
    println!("layers walked:   {}", chain.layers.len());
    // The pinned `Fill` carries one variant (`Color`); the gradient
    // shapes ride INSIDE the `Background` value (the tag), so a solid
    // read is the only arm.
    match style.background {
        Some(gpui::Fill::Color(background)) => match background.as_solid() {
            Some(color) => println!(
                "background:      grayscaled/composed to {:?}",
                gpui::Rgba::from(color)
            ),
            None => println!("background:      a gradient (unexpected for this program)"),
        },
        None => println!("background:      unchanged"),
    }
    let text = style
        .text
        .color
        .map(gpui::Rgba::from)
        .map_or("unchanged".to_string(), |rgba| format!("{rgba:?}"));
    println!("text colour:     {text}");
    let border = style
        .border_color
        .map(gpui::Rgba::from)
        .map_or("unchanged".to_string(), |rgba| format!("{rgba:?}"));
    println!("border colour:   {border}");

    // The drop-shadow presets: the audited band texts parse into GPUI's
    // own band list, composed AFTER the shadow tier's preset bands.
    let shadows = tw!("shadow-lg drop-shadow-md");
    let style = gpui::StyleRefinement::default().tw(shadows);
    println!();
    println!("── drop-shadow bands beside the shadow tier's ──");
    for (i, shadow) in style
        .box_shadow
        .as_deref()
        .unwrap_or_default()
        .iter()
        .enumerate()
    {
        println!(
            "shadow [{i}]:       offset {:?}  blur {:?}  colour {:?}",
            shadow.offset,
            shadow.blur_radius,
            gpui::Rgba::from(shadow.color)
        );
    }

    // The reset arm: `filter-none` clears the chain — the bake paints
    // unfiltered, the way v4's own cascade reads the reset.
    let reset = tw!("bg-red-500 grayscale filter-none");
    let style = gpui::StyleRefinement::default().tw(reset);
    println!();
    println!("── the reset arm ──");
    match style.background {
        Some(gpui::Fill::Color(background)) => println!(
            "after reset:     {:?}",
            background
                .as_solid()
                .map(gpui::Rgba::from)
                .map_or("a gradient".to_string(), |rgba| format!("{rgba:?}"))
        ),
        None => println!("after reset:     unchanged"),
    }

    // The honest boundary: the spatial and backdrop halves refuse at
    // compile time — `tw!("blur-md")` is a compile error, the reason
    // naming the CSS/interp arms that carry the real output.
    println!();
    println!("── the reported boundary ──");
    println!("blur-* / backdrop-*: the static emit refuses them typed (no GPUI slot);");
    println!("the CSS arm renders the v4 var-chain form, the interp answers under any env.");
}
