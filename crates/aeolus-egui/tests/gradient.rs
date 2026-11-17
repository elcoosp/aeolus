//! The egui gradient mesh-painter seam — the linear, radial and conic
//! tiers.
//!
//! Written FIRST (the RED spec): the suite does not compile against the
//! pre-seam backend — `EguiStyle` carries no gradient slot and the crate
//! has no mesh builder — and every pin below passes only once the seam
//! lands. egui 0.37's `Frame` paints a solid `Color32` `RectShape` (no
//! custom-paint hook, no gradient brush), so a gradient class used to
//! drop with "gradients need an app-side mesh painter". The seam SHIPS
//! that painter: the lowering bakes a [`GradientPaint`] payload onto the
//! style, [`gradient_mesh`] builds the banded quad strip over any rect,
//! and the host adds the mesh to its painter — real gradient output, the
//! refusal narrowed to what egui truly cannot express.
//!
//! The geometry pins mirror the iced backend's `linear_gradient_angle_
//! conversion` test (the CSS angle convention is workspace-wide): 0° =
//! to top, clockwise; 90° (`to-r`) runs left → right in screen space
//! (y down). Expected colors are derived from the resolved plan through
//! the crate's own [`to_egui_color`] — the pins stay honest to whatever
//! the theme bakes.

use egui::{Color32, Rect};

use aeolus_compile::{Config, compile_theme, keyword_alignment, keyword_texts};
use aeolus_core::plan::{GRADIENT_FROM_POS_PCT, GRADIENT_TO_POS_PCT, GRADIENT_VIA_POS_PCT};
use aeolus_core::{Resolver, shadcn_default};
use aeolus_egui::{
    CONIC_SEGMENTS, EguiProbe, EguiStyle, GradientKind, LoweringReport, RADIAL_SEGMENTS, StyleBook,
    gradient_mesh, lower_program, lower_resolved, to_egui_color,
};
use aeolus_rt::StaticProgram;
use aeolus_theme::{ThemeSource, TokenTable};

fn resolver() -> Resolver<'static> {
    let theme = shadcn_default().unwrap();
    Resolver::new(Box::leak(Box::new(theme)))
}

/// Resolve + lower one class string, handing back the style and its
/// honest report.
fn lowered(classes: &str) -> (EguiStyle, LoweringReport) {
    let resolver = resolver();
    let r = resolver.resolve(classes).unwrap();
    let mut report = LoweringReport::default();
    let style = lower_resolved(&r.plan.base, &mut report);
    (style, report)
}

/// The expected paint payload, derived from the resolved plan itself
/// (the stops' baked oklab colors through the crate's own conversion).
fn expected_paint(classes: &str) -> (GradientKind, Vec<(f32, Color32)>) {
    let resolver = resolver();
    let r = resolver.resolve(classes).unwrap();
    let gradient = r
        .plan
        .base
        .background_gradient
        .expect("the classes compose a gradient");
    let stops_out = |stops: aeolus_core::plan::GradientStops| {
        let mut out = vec![(
            stops.from.pos_pct.unwrap_or(GRADIENT_FROM_POS_PCT) / 100.0,
            to_egui_color(stops.from.color),
        )];
        if let Some(via) = stops.via {
            out.push((
                via.pos_pct.unwrap_or(GRADIENT_VIA_POS_PCT) / 100.0,
                to_egui_color(via.color),
            ));
        }
        out.push((
            stops.to.pos_pct.unwrap_or(GRADIENT_TO_POS_PCT) / 100.0,
            to_egui_color(stops.to.color),
        ));
        out
    };
    match gradient {
        aeolus_core::plan::Gradient::Linear { angle_deg, stops } => {
            (GradientKind::Linear { angle_deg }, stops_out(stops))
        }
        aeolus_core::plan::Gradient::Radial { stops } => (GradientKind::Radial, stops_out(stops)),
        aeolus_core::plan::Gradient::Conic {
            from_angle_deg,
            stops,
        } => (GradientKind::Conic { from_angle_deg }, stops_out(stops)),
        aeolus_core::plan::Gradient::None => {
            panic!("the spec's classes compose gradients")
        }
    }
}

/// Compile a class string the way `tw!` does (the baked shadcn table)
/// and freeze it into the static program form.
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

/// A disabled probe (the crate's own probe type, the plan path's
/// resolve_at input).
fn disabled_probe() -> EguiProbe {
    EguiProbe {
        disabled: true,
        ..EguiProbe::default()
    }
}

/// A hovered probe.
fn hover_probe() -> EguiProbe {
    EguiProbe {
        hovered: true,
        ..EguiProbe::default()
    }
}

#[test]
fn linear_gradient_lowers_to_the_mesh_paint() {
    let (kind, stops) = expected_paint("bg-linear-to-r from-red-500 to-blue-500");
    let (style, report) = lowered("bg-linear-to-r from-red-500 to-blue-500");
    assert!(
        report.is_empty(),
        "the linear tier no longer drops: {report:?}"
    );
    let paint = style.gradient.expect("the payload lands on the style");
    assert_eq!(paint.kind, kind, "to-r is the CSS 90°");
    let got: Vec<(f32, Color32)> = paint.stops.iter().map(|s| (s.pos, s.color)).collect();
    assert_eq!(got, stops, "from@0 → to@1, colors through to_egui_color");
}

/// The banded strip covers the rect exactly: one band per adjacent stop
/// pair, vertices on the rect boundary, indices a valid two-triangle
/// fan per band.
#[test]
fn the_mesh_bands_cover_the_rect_exactly() {
    let (style, _) = lowered("bg-linear-to-r from-red-500 to-blue-500");
    let paint = style.gradient.expect("the payload lands on the style");
    let rect = Rect::from_min_size(egui::Pos2::new(10.0, 20.0), egui::Vec2::splat(100.0));
    let mesh = gradient_mesh(rect, &paint);
    assert!(mesh.is_valid(), "indices bound the vertices");
    assert_eq!(mesh.calc_bounds(), rect, "the strip covers only the rect");
    // 2 stops → 1 band → 4 vertices, 2 triangles.
    assert_eq!(mesh.vertices.len(), 4, "{:?}", mesh.vertices);
    assert_eq!(mesh.indices.len(), 6);
    // to-r (90°): the from color sits on the LEFT edge, to on the RIGHT.
    let left: Vec<_> = mesh
        .vertices
        .iter()
        .filter(|v| (v.pos.x - rect.left()).abs() < f32::EPSILON)
        .collect();
    let right: Vec<_> = mesh
        .vertices
        .iter()
        .filter(|v| (v.pos.x - rect.right()).abs() < f32::EPSILON)
        .collect();
    assert_eq!(left.len(), 2, "the left edge carries two vertices");
    assert_eq!(right.len(), 2, "the right edge carries two vertices");
    assert_eq!(left[0].color, paint.stops[0].color, "from rides the left");
    assert_eq!(right[0].color, paint.stops[1].color, "to rides the right");
    // The untextured mesh paints from the default texture.
    assert_eq!(mesh.texture_id, egui::TextureId::default());
}

/// The via stop splits the strip into two bands whose shared edge
/// carries the via color at the via position.
#[test]
fn the_via_stop_bands_the_strip() {
    let (style, _) = lowered("bg-linear-to-r from-red-500 via-white to-blue-500");
    let paint = style.gradient.expect("the payload lands on the style");
    assert_eq!(paint.stops.len(), 3, "from → via → to");
    assert_eq!(
        paint.stops[1].pos,
        GRADIENT_VIA_POS_PCT / 100.0,
        "the v4 default via position"
    );
    let rect = Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::new(100.0, 40.0));
    let mesh = gradient_mesh(rect, &paint);
    // 3 stops → 2 bands → 6 vertices, 4 triangles.
    assert_eq!(mesh.vertices.len(), 6, "{:?}", mesh.vertices);
    assert_eq!(mesh.indices.len(), 12);
    let mid: Vec<_> = mesh
        .vertices
        .iter()
        .filter(|v| (v.pos.x - 50.0).abs() < f32::EPSILON)
        .collect();
    assert_eq!(mid.len(), 2, "the shared band edge sits at x=50: {mid:?}");
    assert_eq!(
        mid[0].color, paint.stops[1].color,
        "the via color rides the shared edge"
    );
}

/// The CSS angle convention in screen space (y down): 0° runs bottom →
/// top (`to-t`), 180° runs top → bottom (`to-b`).
#[test]
fn the_css_angle_screenspace_convention() {
    let rect = Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::new(40.0, 100.0));
    // to-t = 0°: the from color sits at the BOTTOM edge.
    let (style, _) = lowered("bg-linear-to-t from-red-500 to-blue-500");
    let paint = style.gradient.expect("the payload lands on the style");
    assert_eq!(
        paint.kind,
        GradientKind::Linear { angle_deg: 0.0 },
        "to-t is the CSS 0°"
    );
    let mesh = gradient_mesh(rect, &paint);
    let bottom: Vec<_> = mesh
        .vertices
        .iter()
        .filter(|v| (v.pos.y - rect.bottom()).abs() < f32::EPSILON)
        .collect();
    let top: Vec<_> = mesh
        .vertices
        .iter()
        .filter(|v| (v.pos.y - rect.top()).abs() < f32::EPSILON)
        .collect();
    assert_eq!(bottom.len(), 2, "the bottom edge carries vertices");
    assert_eq!(top.len(), 2, "the top edge carries vertices");
    assert_eq!(
        bottom[0].color, paint.stops[0].color,
        "from rides the bottom at 0°"
    );
    assert_eq!(top[0].color, paint.stops[1].color, "to rides the top at 0°");

    // to-b = 180°: the from color sits at the TOP edge.
    let (style, _) = lowered("bg-linear-to-b from-red-500 to-blue-500");
    let paint = style.gradient.expect("the payload lands on the style");
    let mesh = gradient_mesh(rect, &paint);
    let top: Vec<_> = mesh
        .vertices
        .iter()
        .filter(|v| (v.pos.y - rect.top()).abs() < f32::EPSILON)
        .collect();
    assert_eq!(
        top[0].color, paint.stops[0].color,
        "from rides the top at 180°"
    );
}

/// `opacity-*` bakes into the stop alphas the same way it bakes into the
/// frame fill (CSS opacity composites the whole element).
#[test]
fn opacity_bakes_into_the_stop_alphas() {
    let (base, _) = lowered("bg-linear-to-r from-red-500 to-blue-500");
    let base = base.gradient.expect("the undimmed paint bakes");
    let (style, _) = lowered("bg-linear-to-r from-red-500 to-blue-500 opacity-50");
    let paint = style.gradient.expect("the payload lands on the style");
    for (dimmed, full) in paint.stops.iter().zip(base.stops.iter()) {
        assert!(
            (dimmed.color.a() as f32 - full.color.a() as f32 / 2.0).abs() <= 1.0,
            "every stop alpha halves: {:?} vs {:?}",
            dimmed.color,
            full.color
        );
    }
}

/// Overlay semantics: a later gradient layer REPLACES the payload (the
/// slot-replace rule — a layer addresses the image slot when it
/// composes its own gradient), and a bare `disabled:opacity-50` dims
/// the base stops it accumulates over. A stop patch WITHOUT a shape
/// (`disabled:from-emerald-500` alone) composes nothing in its layer
/// (the v0.6 §1.2 bucket rule) — the base paint survives, matching the
/// plan path's own semantics.
#[test]
fn overlay_replaces_and_dims_the_paint() {
    let resolver = resolver();
    // Replacement: the hovered tier composes its own gradient.
    let r = resolver
        .resolve(
            "bg-linear-to-r from-red-500 to-blue-500 \
             hover:bg-linear-to-l hover:from-emerald-500 hover:to-blue-500",
        )
        .unwrap();
    let (book, report) = StyleBook::new(&r.plan);
    assert!(
        report.dropped.iter().all(|(w, _)| w.starts_with("Hover")),
        "the linear tier no longer drops: {report:?}"
    );
    let hovered = book.resolve_at(&hover_probe());
    let paint = hovered
        .gradient
        .expect("the hovered tier rewrites the paint");
    assert_eq!(
        paint.kind,
        GradientKind::Linear { angle_deg: 270.0 },
        "to-l is the CSS 270°"
    );
    let emerald = resolver.resolve("bg-emerald-500").unwrap();
    assert_eq!(
        paint.stops[0].color,
        to_egui_color(
            emerald
                .plan
                .base
                .background
                .expect("bg-emerald-500 fills")
                .0
        ),
        "emerald-500 replaces red-500 under hover:"
    );

    // The dimming rule: a bare opacity layer over a gradient base.
    let (base, _) = lowered("bg-linear-to-r from-red-500 to-blue-500");
    let base = base.gradient.expect("the undimmed paint bakes");
    let r = resolver
        .resolve("bg-linear-to-r from-red-500 to-blue-500 disabled:opacity-50")
        .unwrap();
    let (book, _) = StyleBook::new(&r.plan);
    let disabled = book.resolve_at(&disabled_probe());
    let paint = disabled.gradient.expect("the base paint survives");
    for (dimmed, full) in paint.stops.iter().zip(base.stops.iter()) {
        assert!(
            (dimmed.color.a() as f32 - full.color.a() as f32 / 2.0).abs() <= 1.0,
            "the later opacity dims the accumulated stops: {:?}",
            dimmed.color
        );
    }
}

/// `bg-none` clears the payload (the explicit background-image reset).
#[test]
fn bg_none_clears_the_paint() {
    let (style, _) = lowered("bg-linear-to-r from-red-500 to-blue-500 bg-none");
    assert!(
        style.gradient.is_none(),
        "bg-none clears the image slot: {:?}",
        style.gradient
    );
}

/// CSS paints `background-image` OVER `background-color`: the solid fill
/// stays (it shows through translucent stops) and nothing reports — the
/// old "background-color under gradient" flattening note retires.
#[test]
fn the_solid_fill_stays_under_the_gradient() {
    let (style, report) = lowered("bg-red-500 bg-linear-to-r from-red-500 to-blue-500");
    assert!(
        !report
            .dropped
            .iter()
            .any(|(w, _)| w == "background-color under gradient"),
        "the layering is now renderable: {report:?}"
    );
    assert!(style.gradient.is_some(), "the paint lands");
    let solid = resolver().resolve("bg-red-500").unwrap();
    assert_eq!(
        style.frame.fill,
        to_egui_color(solid.plan.base.background.expect("bg-red-500 fills").0),
        "the fill slot keeps the solid color"
    );
}

/// The radial tier lowers through the same seam (the second slice): the payload rides
/// the style, positions resolved, nothing reports.
#[test]
fn radial_lowers_to_the_mesh_paint() {
    let (kind, stops) = expected_paint("bg-radial from-red-500 to-blue-500");
    let (style, report) = lowered("bg-radial from-red-500 to-blue-500");
    assert!(
        report.is_empty(),
        "the radial tier no longer drops: {report:?}"
    );
    let paint = style.gradient.expect("the payload lands on the style");
    assert_eq!(paint.kind, kind, "the plan's radial normalization rides");
    let got: Vec<(f32, Color32)> = paint.stops.iter().map(|s| (s.pos, s.color)).collect();
    assert_eq!(got, stops, "from@0 → to@1 through to_egui_color");
}

/// The radial mesh: concentric rings from the box center (the
/// `circle farthest-corner` normalization), clipped to the rect — the
/// disc overflows the box, the CSS background clip does not.
#[test]
fn the_radial_mesh_rings_from_the_center() {
    let (style, _) = lowered("bg-radial from-red-500 to-blue-500");
    let paint = style.gradient.expect("the payload lands on the style");
    let rect = Rect::from_min_size(egui::Pos2::new(10.0, 20.0), egui::Vec2::new(200.0, 60.0));
    let mesh = gradient_mesh(rect, &paint);
    assert!(mesh.is_valid(), "indices bound the vertices");
    assert_eq!(mesh.calc_bounds(), rect, "the clipped rings cover the rect");
    // The fan origin sits at the center carrying the from color (the
    // per-segment fan polygons each contribute their origin copy — the
    // mesh's contract is the paint, not vertex dedup).
    let center = rect.center();
    let at_center: Vec<_> = mesh
        .vertices
        .iter()
        .filter(|v| (v.pos - center).length() < f32::EPSILON)
        .collect();
    assert!(!at_center.is_empty(), "the fan origin sits at the center");
    for v in &at_center {
        assert_eq!(v.color, paint.stops[0].color, "the center carries from");
    }
    // Every vertex stays inside the rect (the clip ran).
    for v in &mesh.vertices {
        assert!(rect.contains(v.pos), "vertex {:?} escaped the rect", v.pos);
    }
    // The rings are segmented (a real disc, not one fat triangle).
    assert!(mesh.indices.len() / 3 >= RADIAL_SEGMENTS as usize);
}

/// The conic tier lowers through the same seam (the second slice): the payload rides
/// the style (the plan bridge composes `from 0deg` — the V1 registry's
/// conic rows carry no angle patch), positions resolved, nothing
/// reports.
#[test]
fn conic_lowers_to_the_mesh_paint() {
    let (kind, stops) = expected_paint("bg-conic from-red-500 to-blue-500");
    let (style, report) = lowered("bg-conic from-red-500 to-blue-500");
    assert!(
        report.is_empty(),
        "the conic tier no longer drops: {report:?}"
    );
    let paint = style.gradient.expect("the payload lands on the style");
    assert_eq!(paint.kind, kind, "from 0deg, CSS clockwise");
    let got: Vec<(f32, Color32)> = paint.stops.iter().map(|s| (s.pos, s.color)).collect();
    assert_eq!(got, stops);
}

/// The conic mesh: sectors sweep around the center clockwise from the
/// CSS start angle, subdivided enough to cover the box, clipped to the
/// rect (the fan's circumscribed radius overshoots the corners).
#[test]
fn the_conic_mesh_sectors_around_the_center() {
    let (style, _) = lowered("bg-conic from-red-500 to-blue-500");
    let paint = style.gradient.expect("the payload lands on the style");
    let rect = Rect::from_min_size(egui::Pos2::new(10.0, 20.0), egui::Vec2::new(200.0, 60.0));
    let mesh = gradient_mesh(rect, &paint);
    assert!(mesh.is_valid(), "indices bound the vertices");
    assert_eq!(mesh.calc_bounds(), rect, "the clipped fan covers the rect");
    // The per-sector center vertices carry the angular midpoints —
    // BETWEEN the endpoint colors channel-wise (the documented
    // approximation: the center is degenerate in CSS conic). The apex
    // appears once per fan TRIANGLE (clipped polygons re-fan from
    // their first vertex), so the count is >= the segmentation, never
    // the mesh's contract — the paint is.
    let center = rect.center();
    let mids: Vec<_> = mesh
        .vertices
        .iter()
        .filter(|v| (v.pos - center).length() < f32::EPSILON)
        .collect();
    assert!(
        mids.len() >= CONIC_SEGMENTS as usize,
        "every sector fans from the center: {}",
        mids.len()
    );
    let (lo, hi) = (
        paint.stops[0].color,
        paint.stops[paint.stops.len() - 1].color,
    );
    for m in &mids {
        for ch in 0..4 {
            let a = lo.to_array()[ch];
            let b = hi.to_array()[ch];
            let v = m.color.to_array()[ch];
            assert!(
                v >= a.min(b) && v <= a.max(b),
                "the sector mid {v} escapes the [{a},{b}] channel range"
            );
        }
    }
    // Every vertex stays inside the rect (the clip ran).
    for v in &mesh.vertices {
        assert!(rect.contains(v.pos), "vertex {:?} escaped the rect", v.pos);
    }
}

/// Solid fills layer under EVERY kind now — the flattening note
/// retired with the seam (the second slice moved the pin from the refusal suite).
#[test]
fn the_solid_fill_stays_under_radial_and_conic() {
    for classes in [
        "bg-red-500 bg-radial from-red-500/50 to-blue-500/50",
        "bg-red-500 bg-conic from-red-500/50 to-blue-500/50",
    ] {
        let (_, report) = lowered(classes);
        assert!(
            !report
                .dropped
                .iter()
                .any(|(w, _)| w == "background-color under gradient"),
            "{classes}: the layering is renderable: {report:?}"
        );
    }
}

/// The mesh paints square corners (egui's rounding shapes the fill slot
/// only); a gradient UNDER a rounded frame reports the divergence —
/// never a silent visual difference.
#[test]
fn rounded_corners_report_the_square_mesh() {
    let (_, report) = lowered("rounded-lg bg-linear-to-r from-red-500 to-blue-500");
    assert!(
        report
            .dropped
            .iter()
            .any(|(w, _)| w == "gradient-corner-radius"),
        "{report:?}"
    );
    // No radius, no note.
    let (_, report) = lowered("bg-linear-to-r from-red-500 to-blue-500");
    assert!(report.is_empty(), "{report:?}");
}

/// The V1 static-program lane agrees with the V1 plan bridge: the same
/// class string walks to the same payload (both lanes share the
/// lowering and the V1 bake). The v0.x ORACLE's stops carry its own
/// hand-rounded oklab literals — the oracle's constants differ from the
/// audited hex rows in the last byte of out-of-gamut colors, the
/// differential suite's own documented tolerance, so the cross-lane
/// pin runs against the bridge (the V1 answer) rather than the oracle.
#[test]
fn the_program_lane_walks_the_same_paint() {
    let classes = "bg-linear-to-r from-red-500 to-blue-500";
    let program = frozen(classes);
    let (style, report) = lower_program(&program, &aeolus_rt::Env::default());
    assert!(report.is_empty(), "{report:?}");

    // The bridge lane: the V1 pipeline's own emitted plan, lowered
    // through the same `lower_resolved`.
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    let table = TokenTable::baked(&source).expect("the baked shadcn table builds");
    let sheet =
        aeolus_compile::emit_plan_theme(classes, &table, &Config::default()).expect("lowers");
    let mut report = LoweringReport::default();
    let bridged = lower_resolved(&sheet.plan.base, &mut report);
    assert!(report.is_empty(), "{report:?}");
    assert_eq!(
        style.gradient, bridged.gradient,
        "the V1 lanes agree on the payload byte-for-byte"
    );
}
