//! Consumer demonstration: the egui backend's two native surfaces —
//! the immediate-mode frame (panels, windows, groups) and the
//! per-state widget cascade (`visuals.widgets`). egui re-asks every
//! frame; this example resolves into a headless `Context::default()`
//! the same way a running app's frame would.
//!
//! ```sh
//! cargo run -p aeolus-egui --example panel_style
//! ```

use aeolus_core::{Resolver, shadcn_default};
use aeolus_egui::{EguiProbe, StyleBook, apply_widget_visuals};

fn main() {
    let theme = shadcn_default().expect("the embedded shadcn default theme parses");
    let resolver = Resolver::new(Box::leak(Box::new(theme)));

    // The card surface a panel or window body would wear: base frame,
    // hover layer.
    let book = StyleBook::tw(
        &resolver,
        "p-4 bg-card text-card-foreground rounded-md border hover:bg-accent",
    );

    // 1. The FRAME surface: one resolved style per situation. A real app
    //    calls this inside `Frame::show` / a panel body every frame.
    let ctx = egui::Context::default();
    let style = book.resolve_for_ctx(&ctx);
    println!("── egui frame surface (base, light ctx) ──");
    println!(
        "  inner_margin (css padding): {:?}",
        style.frame.inner_margin
    );
    println!(
        "  outer_margin (css margin):  {:?}",
        style.frame.outer_margin
    );
    println!(
        "  corner_radius (rounded-*):  {:?}",
        style.frame.corner_radius
    );
    println!("  stroke (border-*):          {:?}", style.frame.stroke);
    println!("  shadow:                     {:?}", style.frame.shadow);

    // 2. The WIDGET cascade: the hover layer lowers onto
    //    `visuals.widgets.hovered` — the native immediate-mode state
    //    slots a Button reads. The probe is what a running app builds
    //    from `Response` each frame (here: the hover flag, directly),
    //    and the target slot is the app's widget style (what a real
    //    app's `Button` paints from — the context's own `visuals`).
    let hovered = book.resolve_at(&EguiProbe {
        hovered: true,
        ..EguiProbe::default()
    });
    let mut widget_visuals = egui::Visuals::default().widgets.hovered;
    apply_widget_visuals(&mut widget_visuals, &hovered);
    println!("── egui widget cascade (hovered) ──");
    println!("  bg_fill:   {:?}", widget_visuals.bg_fill);
    println!("  fg_stroke: {:?}", widget_visuals.fg_stroke);
}
