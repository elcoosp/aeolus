//! Consumer demonstration: the egui gradient mesh-painter seam (the
//! linear tier). egui 0.37's `Frame` paints a solid `Color32` rect — no
//! gradient brush — so the backend ships the painter: the lowering
//! bakes a [`GradientPaint`] payload onto the style and
//! [`gradient_mesh`] builds the banded quad strip over any rect (one
//! band per adjacent stop pair — egui's vertex-color interpolation is
//! exact at every stop and linear between them, the CSS contract). The
//! host adds the mesh to its painter right after the frame:
//!
//! ```ignore
//! let (style, report) = lower_program(&program, &env);
//! style.frame.show(ui, |ui| { … });
//! if let Some(paint) = &style.gradient {
//!     paint_gradient(ui, response.rect, paint);
//! }
//! ```
//!
//! Headless by construction: the walk and the mesh are pure, so the
//! demo prints the baked payload and the strip's vertex table instead
//! of opening a window.
//!
//! ```sh
//! cargo run -p aeolus-egui --example gradient
//! ```

use aeolus_compile::{Config, compile_theme, keyword_alignment, keyword_texts};
use aeolus_egui::{gradient_mesh, lower_program};
use aeolus_rt::{Env, StaticProgram};
use aeolus_theme::{ThemeSource, TokenTable};

/// Compile a class string the way `tw!` does (the baked shadcn table)
/// and freeze it into the static form the macro emits.
fn frozen(classes: &str) -> StaticProgram {
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    let table = TokenTable::baked(&source).expect("the baked shadcn table builds");
    let config = Config::default();
    let compiled = compile_theme(classes, &table, &config).expect("the class string lowers");
    let annotations = keyword_texts(classes, &config).expect("non-empty input annotates");
    let keywords = keyword_alignment(&compiled, &annotations);
    StaticProgram::new(
        compiled.program.layers,
        Box::leak(keywords.into_boxed_slice()),
    )
}

/// Walk one class string, print the baked payload and the strip's
/// vertex table over a fixed rect.
fn demo(classes: &str, rect: egui::Rect) {
    let program = frozen(classes);
    let (style, report) = lower_program(&program, &Env::default());
    println!("── {classes} ──");
    if !report.is_empty() {
        println!("  report: {:?}", report.dropped);
    }
    let Some(paint) = &style.gradient else {
        println!("  (no gradient payload — the classes compose no image)");
        println!();
        return;
    };
    println!(
        "  kind:  {:?}",
        match paint.kind {
            aeolus_egui::GradientKind::Linear { angle_deg } =>
                format!("linear, {angle_deg}° CSS (0 = to top, clockwise)"),
            aeolus_egui::GradientKind::Radial =>
                "radial, circle farthest-corner from the box center".to_string(),
            aeolus_egui::GradientKind::Conic { from_angle_deg } =>
                format!("conic, from {from_angle_deg}° CSS clockwise"),
        }
    );
    for stop in &paint.stops {
        let [r, g, b, a] = stop.color.to_array();
        println!(
            "  stop:  pos {:>5.2}  rgb({r:>3} {g:>3} {b:>3} / a {a:>3})",
            stop.pos
        );
    }
    let mesh = gradient_mesh(rect, paint);
    println!(
        "  mesh over {}×{}: {} vertices, {} triangles, bounds {:?}",
        rect.width(),
        rect.height(),
        mesh.vertices.len(),
        mesh.indices.len() / 3,
        mesh.calc_bounds()
    );
    for (i, v) in mesh.vertices.iter().enumerate() {
        println!(
            "    v{i}: ({:>6.1}, {:>6.1})  {:?}",
            v.pos.x, v.pos.y, v.color
        );
    }
    println!();
}

fn main() {
    let box_rect =
        egui::Rect::from_min_size(egui::Pos2::new(8.0, 8.0), egui::Vec2::new(256.0, 64.0));

    // The three directions (the CSS angle convention pins: 0° = to top,
    // clockwise — screen space runs y down; to-tr rides the corner
    // angle the plan carries).
    demo("bg-linear-to-r from-red-500 to-blue-500", box_rect);
    demo("bg-linear-to-t from-red-500 to-blue-500", box_rect);
    demo("bg-linear-to-tr from-red-500 to-blue-500", box_rect);

    // The via stop bands the strip: one band per adjacent stop pair.
    demo(
        "bg-linear-to-r from-red-500 via-white to-blue-500",
        box_rect,
    );

    // The radial tier: concentric rings from the box center over the
    // farthest-corner radius, clipped to the rect (the disc overflows
    // a wide box; the CSS background clip does not).
    demo("bg-radial from-red-500 via-white to-blue-500", box_rect);

    // The conic tier: sectors sweep clockwise from `from 0deg` (the
    // plan bridge's composition), the fan circumscribed then clipped.
    demo("bg-conic from-red-500 via-white to-blue-500", box_rect);

    // CSS layering: background-image paints OVER background-color, so
    // the solid fill stays and translucent stops show it through.
    demo(
        "bg-red-500 bg-linear-to-r from-red-500/50 to-blue-500/50",
        box_rect,
    );

    // `bg-none` clears the image (the explicit reset).
    demo("bg-linear-to-r from-red-500 to-blue-500 bg-none", box_rect);
}
