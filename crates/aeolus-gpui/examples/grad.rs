//! Consumer demonstration: the GPUI gradient lane (architecture plan
//! §3.7's `bg-linear-*` row — "native for 2 stops"). GPUI's
//! `linear_gradient(angle, from, to)` carries exactly the v4 two-stop
//! linear form, so the direction row's composed text plus the
//! `--tw-gradient-from`/`-to` stops the registry lowers into compose
//! into the applied refinement — instead of the silent no-op the
//! un-mapped arm used to be. The demo resolves into a headless
//! `StyleRefinement` the same way a running app's `div()` would
//! consume the program.
//!
//! ```sh
//! cargo run -p aeolus-gpui --example grad
//! ```

use aeolus_gpui::prelude::*;

/// The applied background's Debug render (GPUI's `Background` fields
/// are crate-private; the Debug form is the surface a refinement dump
/// shows — `LinearGradient(<angle>, <from>, <to>)`).
fn background_of(program: StaticProgram) -> String {
    let style = gpui::StyleRefinement::default().tw(program);
    match style.background {
        Some(gpui::Fill::Color(background)) => format!("{background:?}"),
        other => format!("{other:?}"),
    }
}

fn main() {
    // The hero banner's fade: the direction row (`to right` = the CSS
    // 90°), the from stop with its `20%` position patch, the to stop —
    // the whole gradient family in one program.
    let program = tw!("bg-linear-to-r from-red-500 from-20% to-blue-500");
    println!("── GPUI linear gradient from the V1 program ──");
    println!("  layers walked: {}", program.layers.len());
    println!("  background:    {}", background_of(program));

    // The corner direction: `to top left` = the CSS 315° — the row
    // text's direction map, composed unchanged. (`to-transparent` stays
    // the typed refusal it already is for every colour root — the CSS
    // keyword is not a theme token; see the value pass's boundary.)
    let corner = tw!("bg-linear-to-tl from-primary to-blue-500");
    println!("── the corner direction (to top left = 315°) ──");
    println!("  background:    {}", background_of(corner));

    // The honest boundary: a via stop would need THREE stops and GPUI's
    // gradient carries two — the walk composes nothing (the static emit
    // gate refuses the program typed; the dynamic lane degrades to no
    // gradient, never to a flattened misreading).
    let via = tw!("bg-linear-to-r from-red-500 via-white to-blue-500");
    let style = gpui::StyleRefinement::default().tw(via);
    println!("── the via stop stays the reported boundary ──");
    println!(
        "  background:    {:?} (three stops paint nothing rather than a wrong gradient)",
        style.background
    );

    // The explicit clear: `bg-none` never paints a gradient (a solid
    // `bg-*` the walk applied stays, the CSS layering).
    let cleared = tw!("bg-none");
    let style = gpui::StyleRefinement::default().tw(cleared);
    println!("── bg-none clears the image ──");
    println!("  background:    {:?}", style.background);
}
