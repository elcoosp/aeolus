//! Consumer demonstration: the GPUI decoration tier (architecture
//! plan §3.7's typography surface — the `decoration-{color}`/
//! `decoration-{thickness}`/`decoration-{style}` family, the landed
//! half of the outline/transform wrapper backlog's typography sibling).
//! The pinned `Styled` surface carries the decoration methods the v4
//! grammar composes — `text_decoration_color` and the
//! `text_decoration_0..8` thickness methods on the underline leg, the
//! `text_decoration_solid`/`text_decoration_wavy` line forms — and the
//! walk maps the three longhands onto them: the colour/thickness
//! patches reach the strikethrough leg only when a line-through already
//! composes (v4's own `text-decoration-color`/`-thickness` reading).
//! The style keywords GPUI renders a single solid line for
//! (`double`/`dotted`/`dashed`) and the thickness keyword/percentage
//! forms stay the emit gates' typed refusal. The demo resolves into a
//! headless `StyleRefinement` the same way a running app's `div()`
//! would consume the program.
//!
//! ```sh
//! cargo run -p aeolus-gpui --example decoration
//! ```

use aeolus_gpui::prelude::*;

fn main() {
    // The annotated-phrase surface a docs renderer would wear: the
    // full family in one program — line, style, thickness and the
    // frozen palette colour.
    let annotated = tw!("underline decoration-wavy decoration-4 decoration-red-500");

    let style = gpui::StyleRefinement::default().tw(annotated);
    println!("── GPUI decoration tier from the V1 program ──");
    println!("layers walked:    {}", annotated.layers.len());
    if let Some(underline) = style.text.underline {
        println!("underline:        present");
        println!("  wavy:           {}", underline.wavy);
        println!("  thickness:      {}", underline.thickness);
        match underline.color {
            Some(color) => println!("  color:          {:?}", gpui::Rgba::from(color)),
            None => println!("  color:          unchanged (the inherited text colour)"),
        }
    } else {
        println!("underline:        absent");
    }
    println!(
        "strikethrough:    {}",
        match style.text.strikethrough {
            Some(_) => "composed (the colour alone never creates one)",
            None => "absent — decoration-{color} never creates a line-through",
        }
    );

    // The two-leg reading: line-through beside the colour and thickness
    // — the recolour/resize reaches BOTH lines (v4's own property
    // semantics), the walk's only-if-present patch.
    let strike = tw!("line-through decoration-red-500 decoration-4");
    let style = gpui::StyleRefinement::default().tw(strike);
    println!();
    println!("── line-through beside the colour and thickness ──");
    if let Some(strikethrough) = style.text.strikethrough {
        println!("strikethrough:    present");
        println!("  thickness:      {}", strikethrough.thickness);
        match strikethrough.color {
            Some(color) => println!("  color:          {:?}", gpui::Rgba::from(color)),
            None => println!("  color:          unchanged"),
        }
    } else {
        println!("strikethrough:    absent");
    }
    let underline_leg = style
        .text
        .underline
        .map(|_| "present (the pinned methods' get-or-insert)")
        .unwrap_or("absent");
    println!("underline leg:    {underline_leg}");

    // The bare-integer thickness scale is not the five named methods:
    // the v4 `isPositiveInteger` grammar's own arm, generalized.
    let custom = tw!("underline decoration-13");
    let style = gpui::StyleRefinement::default().tw(custom);
    if let Some(underline) = style.text.underline {
        println!();
        println!("── decoration-13 (the bare-integer scale) ──");
        println!("  thickness:      {}", underline.thickness);
    }
}
