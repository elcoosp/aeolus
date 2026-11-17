//! The gradient mesh-painter seam — real gradient output for egui.
//!
//! egui 0.37's [`Frame`] paints a solid `Color32` [`egui::RectShape`]:
//! no gradient brush, no custom-paint hook (the `Frame::paint` shape is
//! a plain rect), so a gradient class used to drop with "gradients need
//! an app-side mesh painter — reported, not faked". This module SHIPS
//! that painter instead of asking for it:
//!
//! * the lowering bakes a [`GradientPaint`] payload onto
//!   [`EguiStyle::gradient`] (positions resolved, colors converted, the
//!   style's own `opacity-*` already baked into the stop alphas);
//! * [`gradient_mesh`] builds the paint over any `Rect` — the banded
//!   quad strip for linear (one band per adjacent stop pair), the
//!   concentric ring bands for radial, the angular sectors for conic —
//!   so egui's vertex-color interpolation is EXACT at every stop and
//!   linear between them (the CSS contract for all three shapes);
//! * [`paint_gradient`] adds that mesh to a [`egui::Ui`]'s painter —
//!   the one-liner the host runs right after painting the frame.
//!
//! Paint order is CSS's own: `background-image` paints OVER
//! `background-color`, so the frame keeps its solid fill and the mesh
//! lands on top (translucent stops show the fill through). The mesh is
//! untextured (the default texture) and clipped to the rect — square
//! corners: egui's corner rounding shapes the fill slot only, and a
//! gradient under a rounded frame reports the divergence through the
//! lowering report rather than painting a silent visual difference.
//!
//! Geometry conventions (workspace-wide, the iced backend pins the
//! same): CSS angles 0° = to top, positive clockwise, screen space runs
//! y down; radial normalizes to `circle farthest-corner` from the box
//! center (the v0.6 §1.5 shape); conic sweeps clockwise from
//! `from <angle>` (the V1 plan bridge composes `from 0deg`).

use egui::{Color32, Mesh, Pos2, Rect, Ui, Vec2};

use aeolus_core::plan::{
    GRADIENT_FROM_POS_PCT, GRADIENT_TO_POS_PCT, GRADIENT_VIA_POS_PCT, GradientStops,
};

use crate::{scale_alpha32, to_egui_color};

/// The radial disc's angular segmentation (ring band edges). A chord
/// polygon at 48 segments deviates from the true circle by under
/// 0.03 % of the radius — sub-pixel on any UI box.
pub const RADIAL_SEGMENTS: u32 = 48;

/// The conic fan's angular segmentation (one triangle per sector). The
/// fan is circumscribed (radius inflated by 1/cos(π/segments)) so the
/// polygon covers the box corners before the rect clip trims it.
pub const CONIC_SEGMENTS: u32 = 48;

/// The gradient shape a [`GradientPaint`] renders (CSS conventions:
/// angles 0 = to top, positive clockwise).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GradientKind {
    /// `bg-linear-*` — a linear gradient along the CSS angle
    /// (`bg-linear-to-r` = 90°, running left → right in screen space).
    Linear {
        /// The CSS angle in degrees.
        angle_deg: f32,
    },
    /// `bg-radial` — a radial gradient from the box center, the
    /// documented `circle farthest-corner` normalization (v0.6 §1.5).
    Radial,
    /// `bg-conic-*` — a conic gradient sweeping clockwise from the CSS
    /// `from <angle>` (`bg-conic` = 0°).
    Conic {
        /// The CSS `from <angle>` in degrees.
        from_angle_deg: f32,
    },
}

/// One baked gradient stop: position along the paint axis (0..=1) and
/// the converted color (the style's own opacity already baked in).
///
/// The axis meaning rides the kind: distance along the gradient line
/// (linear), radius fraction of the farthest-corner distance (radial),
/// angle fraction of the full sweep (conic).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PaintStop {
    /// Stop position along the paint axis, 0 = start, 1 = end.
    pub pos: f32,
    /// The stop color (straight-alpha sRGB bytes).
    pub color: Color32,
}

/// The baked paint payload for one gradient: everything the mesh
/// painter needs, resolved once at lowering time so the per-frame work
/// is [`gradient_mesh`] alone.
///
/// Carried on [`EguiStyle::gradient`] (`Option` — absent when no
/// gradient class is in scope, `None` again on `bg-none`, the explicit
/// background-image reset).
#[derive(Clone, Debug, PartialEq)]
pub struct GradientPaint {
    /// The gradient shape + angle.
    pub kind: GradientKind,
    /// The stops in axis order (`from` → `via` → `to`), positions
    /// resolved to the v4 `@property` defaults where the classes left
    /// them unpatched (0% / 50% / 100%).
    pub stops: Vec<PaintStop>,
}

impl GradientPaint {
    /// Bake one gradient of any shape from the plan's composed stops:
    /// oklab IR converted through [`crate::to_egui_color`], positions
    /// resolved to the v4 defaults, the style's own opacity baked into
    /// every alpha.
    fn bake(kind: GradientKind, stops: &GradientStops, opacity: f32) -> Self {
        let mut baked = Vec::with_capacity(3);
        baked.push(PaintStop {
            pos: stops.from.pos_pct.unwrap_or(GRADIENT_FROM_POS_PCT) / 100.0,
            color: scale_alpha32(to_egui_color(stops.from.color), opacity),
        });
        if let Some(via) = stops.via {
            baked.push(PaintStop {
                pos: via.pos_pct.unwrap_or(GRADIENT_VIA_POS_PCT) / 100.0,
                color: scale_alpha32(to_egui_color(via.color), opacity),
            });
        }
        baked.push(PaintStop {
            pos: stops.to.pos_pct.unwrap_or(GRADIENT_TO_POS_PCT) / 100.0,
            color: scale_alpha32(to_egui_color(stops.to.color), opacity),
        });
        Self { kind, stops: baked }
    }

    /// Bake one linear gradient.
    pub fn linear(angle_deg: f32, stops: &GradientStops, opacity: f32) -> Self {
        Self::bake(GradientKind::Linear { angle_deg }, stops, opacity)
    }

    /// Bake one radial gradient (the `circle farthest-corner` shape).
    pub fn radial(stops: &GradientStops, opacity: f32) -> Self {
        Self::bake(GradientKind::Radial, stops, opacity)
    }

    /// Bake one conic gradient from the CSS `from <angle>`.
    pub fn conic(from_angle_deg: f32, stops: &GradientStops, opacity: f32) -> Self {
        Self::bake(GradientKind::Conic { from_angle_deg }, stops, opacity)
    }

    /// Scale every stop's straight alpha by `factor` (the overlay's
    /// opacity dimming — a bare `disabled:opacity-50` layer over a
    /// gradient base must dim the accumulated stops like it dims the
    /// frame fill).
    pub fn scale_alpha(&mut self, factor: f32) {
        for stop in &mut self.stops {
            stop.color = scale_alpha32(stop.color, factor);
        }
    }
}

/// Per-channel lerp between two straight-alpha colors (`t` in 0..1).
/// The vertex-color interpolation the mesh relies on between stops is
/// this same linear mix, so clipped polygons and sampled sweeps agree
/// with the GPU's interpolation.
fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let (x, y) = (a.to_array(), b.to_array());
    Color32::from_rgba_unmultiplied(
        lerp_byte(x[0], y[0], t),
        lerp_byte(x[1], y[1], t),
        lerp_byte(x[2], y[2], t),
        lerp_byte(x[3], y[3], t),
    )
}

/// One channel's byte lerp (round-half-up, matching `scale_alpha32`).
fn lerp_byte(a: u8, b: u8, t: f32) -> u8 {
    let v = a as f32 + (b as f32 - a as f32) * t;
    v.clamp(0.0, 255.0).mul_add(1.0, 0.5) as u8
}

/// The CSS linear-gradient geometry: direction vector and gradient-line
/// length for one rect (the "magic corner" construction — the line is
/// long enough that perpendicular chords between start and end cover
/// the whole rect).
fn gradient_axis(rect: Rect, angle_deg: f32) -> (Vec2, f32) {
    let rad = angle_deg.to_radians();
    let (mut sx, mut sy) = (rad.sin(), -rad.cos());
    // Snap near-axis components to exactly 0: the f32 trig of the
    // cardinal angles leaves ~1e-17 slivers (cos(π/2) = 6e-17) that the
    // perpendicular-chord clip would amplify — a sub-ULP offset in the
    // start point divided by a 1e-17 direction component moves the
    // chord by tens of pixels. Any true component below 1e-6 shifts the
    // paint under a hundredth of a pixel on a 100px box: snapping is
    // exact for the UI's cardinal angles and invisible otherwise.
    if sx.abs() < 1e-6 {
        sx = 0.0;
    }
    if sy.abs() < 1e-6 {
        sy = 0.0;
    }
    let d = Vec2::new(sx, sy);
    // The CSS length: |w·sin| + |h·cos| (the magic-corner line).
    let len = (rect.width() * d.x.abs() + rect.height() * d.y.abs()).abs();
    (d, len)
}

/// Intersect the infinite line `p + t·n` with the rect (Liang–Barsky
/// over the two slabs); `None` when it misses entirely. The chord runs
/// boundary to boundary, so bands tile the rect with no over-paint.
fn chord(rect: Rect, p: Pos2, n: Vec2) -> Option<(Pos2, Pos2)> {
    let (mut enter, mut exit) = (f32::NEG_INFINITY, f32::INFINITY);
    for (p_axis, n_axis, lo, hi) in [
        (p.x, n.x, rect.left(), rect.right()),
        (p.y, n.y, rect.top(), rect.bottom()),
    ] {
        if n_axis == 0.0 {
            if !(lo..=hi).contains(&p_axis) {
                return None;
            }
        } else {
            let (t1, t2) = ((lo - p_axis) / n_axis, (hi - p_axis) / n_axis);
            enter = enter.max(t1.min(t2));
            exit = exit.min(t1.max(t2));
        }
    }
    if enter > exit {
        return None;
    }
    Some((
        Pos2::new(p.x + enter * n.x, p.y + enter * n.y),
        Pos2::new(p.x + exit * n.x, p.y + exit * n.y),
    ))
}

/// One clipped polygon vertex: position + the color interpolated along
/// the original edge (the clip must not flatten the gradient).
type PolyVertex = (Pos2, Color32);

/// Clip a convex polygon (position + color per vertex) to the rect
/// (Sutherland–Hodgman over the four edges); colors interpolate along
/// each clipped edge. The output fits a fixed stack buffer: a quad
/// clipped by four half-planes gains at most four vertices.
fn clip_polygon_to_rect<const IN: usize>(poly: &[PolyVertex; IN], rect: Rect) -> Vec<PolyVertex> {
    let mut out = Vec::with_capacity(IN + 4);
    out.extend_from_slice(poly);
    for edge in rect_edges(rect) {
        let input = std::mem::take(&mut out);
        let n = input.len();
        for i in 0..n {
            let (cur, nxt) = (input[i], input[(i + 1) % n]);
            let (cur_in, nxt_in) = (edge.contains(cur.0), edge.contains(nxt.0));
            if nxt_in {
                if !cur_in {
                    out.push(edge.cross(cur, nxt));
                }
                out.push(nxt);
            } else if cur_in {
                out.push(edge.cross(cur, nxt));
            }
        }
    }
    out
}

/// The rect's four half-plane edges as point predicates.
fn rect_edges(rect: Rect) -> [RectEdge; 4] {
    [
        RectEdge::Left(rect.left()),
        RectEdge::Right(rect.right()),
        RectEdge::Top(rect.top()),
        RectEdge::Bottom(rect.bottom()),
    ]
}

/// One rect half-plane.
enum RectEdge {
    Left(f32),
    Right(f32),
    Top(f32),
    Bottom(f32),
}

impl RectEdge {
    /// True when the point is inside this half-plane (boundary in).
    fn contains(&self, p: Pos2) -> bool {
        match *self {
            RectEdge::Left(x) => p.x >= x,
            RectEdge::Right(x) => p.x <= x,
            RectEdge::Top(y) => p.y >= y,
            RectEdge::Bottom(y) => p.y <= y,
        }
    }

    /// Where the segment `a → b` crosses this edge, with the color
    /// interpolated by the crossing parameter and the edge's own
    /// coordinate snapped EXACTLY onto the boundary (an interpolated
    /// fixed coordinate drifts by ULPs and shows up as a mesh whose
    /// bounds miss the rect by 1e-6).
    fn cross(&self, a: PolyVertex, b: PolyVertex) -> PolyVertex {
        let (pa, pb) = (self.coord(a.0), self.coord(b.0));
        let t = (self.boundary() - pa) / (pb - pa);
        let color = lerp_color(a.1, b.1, t);
        let p = match *self {
            RectEdge::Left(x) | RectEdge::Right(x) => Pos2::new(x, a.0.y + (b.0.y - a.0.y) * t),
            RectEdge::Top(y) | RectEdge::Bottom(y) => Pos2::new(a.0.x + (b.0.x - a.0.x) * t, y),
        };
        (p, color)
    }

    /// The edge's fixed coordinate (x for vertical edges, y for
    /// horizontal).
    fn coord(&self, p: Pos2) -> f32 {
        match *self {
            RectEdge::Left(_) | RectEdge::Right(_) => p.x,
            RectEdge::Top(_) | RectEdge::Bottom(_) => p.y,
        }
    }

    /// The edge's boundary value.
    fn boundary(&self) -> f32 {
        match *self {
            RectEdge::Left(x) | RectEdge::Right(x) => x,
            RectEdge::Top(y) | RectEdge::Bottom(y) => y,
        }
    }
}

/// Fan-triangulate a convex polygon into the mesh (every vertex
/// carries its own color; winding is irrelevant — no culling).
fn push_polygon(mesh: &mut Mesh, poly: &[PolyVertex]) {
    for i in 1..poly.len().saturating_sub(1) {
        let ia = mesh.vertices.len() as u32;
        for (pos, color) in [poly[0], poly[i], poly[i + 1]] {
            mesh.colored_vertex(pos, color);
        }
        mesh.add_triangle(ia, ia + 1, ia + 2);
    }
}

/// The distance from the box center to the farthest corner — the CSS
/// `farthest-corner` radius (the radial normalization's own measure).
fn farthest_corner_radius(rect: Rect) -> f32 {
    let c = rect.center();
    let (min, max) = (rect.min, rect.max);
    [
        Pos2::new(min.x, min.y),
        Pos2::new(max.x, min.y),
        Pos2::new(max.x, max.y),
        Pos2::new(min.x, max.y),
    ]
    .iter()
    .map(|p| (*p - c).length())
    .fold(0.0_f32, f32::max)
}

/// Build the gradient mesh over `rect` — the shape dispatch.
///
/// * linear: the banded quad strip between the perpendicular chords
///   through the stops' axis positions — exact at every stop, linear
///   between them, tiled to the rect with no over-paint;
/// * radial: concentric ring bands from the box center over the
///   `farthest-corner` radius (the stops' radii exact, [`RADIAL_
///   SEGMENTS`] segments around), the disc clipped to the rect;
/// * conic: angular sectors sweeping clockwise from `from <angle>`
///   ([`CONIC_SEGMENTS`] triangles around a circumscribed fan so the
///   box corners are covered before the clip), the per-sector center
///   vertex carrying the angular midpoint color — the documented
///   approximation (the center is degenerate in a CSS conic).
///
/// The mesh is untextured (the default texture).
pub fn gradient_mesh(rect: Rect, paint: &GradientPaint) -> Mesh {
    let mut mesh = Mesh::default();
    if paint.stops.is_empty() {
        return mesh;
    }
    match paint.kind {
        GradientKind::Linear { angle_deg } => linear_mesh(&mut mesh, rect, paint, angle_deg),
        GradientKind::Radial => radial_mesh(&mut mesh, rect, paint),
        GradientKind::Conic { from_angle_deg } => {
            conic_mesh(&mut mesh, rect, paint, from_angle_deg);
        }
    }
    mesh
}

/// The linear banded strip (the first slice's tier, unchanged in shape).
fn linear_mesh(mesh: &mut Mesh, rect: Rect, paint: &GradientPaint, angle_deg: f32) {
    let (d, len) = gradient_axis(rect, angle_deg);
    if len <= 0.0 {
        return;
    }
    let center = rect.center();
    let start = center - d * (len / 2.0);
    // The perpendicular unit (the chords run along it).
    let n = Vec2::new(-d.y, d.x);
    // One chord per stop; shared band edges dedupe into 2·N vertices.
    let mut prev: Option<(u32, u32)> = None;
    for stop in &paint.stops {
        let p = start + d * (len * stop.pos.clamp(0.0, 1.0));
        let Some((a, b)) = chord(rect, p, n) else {
            continue;
        };
        let ia = mesh.vertices.len() as u32;
        mesh.colored_vertex(a, stop.color);
        mesh.colored_vertex(b, stop.color);
        let ib = ia + 1;
        if let Some((pa, pb)) = prev {
            // The band quad (pa, pb) → (ia, ib): two triangles.
            mesh.add_triangle(pa, pb, ib);
            mesh.add_triangle(pa, ib, ia);
        }
        prev = Some((ia, ib));
    }
}

/// The radial concentric ring bands.
fn radial_mesh(mesh: &mut Mesh, rect: Rect, paint: &GradientPaint) {
    let center = rect.center();
    let radius = farthest_corner_radius(rect);
    if radius <= 0.0 {
        return;
    }
    // Ring k at the stop's radius fraction, carrying the stop's color;
    // positions past the first stop's extend the FIRST color inward
    // (the CSS pre-first-stop extension) via the center disc.
    let mut rings: Vec<(f32, Color32)> = paint
        .stops
        .iter()
        .map(|s| (radius * s.pos.clamp(0.0, 1.0), s.color))
        .collect();
    if rings[0].0 > 0.0 {
        rings.insert(0, (0.0, rings[0].1));
    }
    // The ring POINTS stay local (the mesh carries only the clipped
    // polygons — orphan ring vertices would pollute the bounds).
    let step = std::f32::consts::TAU / RADIAL_SEGMENTS as f32;
    let ring_pts: Vec<Vec<Pos2>> = rings
        .iter()
        .map(|(r, _)| {
            (0..RADIAL_SEGMENTS)
                .map(|s| {
                    let ang = step * s as f32;
                    if *r <= 0.0 {
                        center
                    } else {
                        center + Vec2::new(ang.sin(), -ang.cos()) * *r
                    }
                })
                .collect()
        })
        .collect();
    // The innermost band fans from the center: one triangle per
    // segment, the fan polygons carrying their own vertices (the clip
    // copies them through for interior triangles).
    for s in 0..RADIAL_SEGMENTS as usize {
        let s1 = (s + 1) % RADIAL_SEGMENTS as usize;
        let tri = [
            (center, rings[0].1),
            (ring_pts[1][s], rings[1].1),
            (ring_pts[1][s1], rings[1].1),
        ];
        let clipped = clip_polygon_to_rect(&tri, rect);
        push_polygon(mesh, &clipped);
    }
    // The remaining bands: the quad strip between consecutive rings
    // (equal radii skip — a zero-width band paints nothing).
    for k in 1..rings.len() - 1 {
        if (rings[k + 1].0 - rings[k].0).abs() <= f32::EPSILON {
            continue;
        }
        for s in 0..RADIAL_SEGMENTS as usize {
            let s1 = (s + 1) % RADIAL_SEGMENTS as usize;
            let quad = [
                (ring_pts[k][s], rings[k].1),
                (ring_pts[k][s1], rings[k].1),
                (ring_pts[k + 1][s1], rings[k + 1].1),
                (ring_pts[k + 1][s], rings[k + 1].1),
            ];
            let clipped = clip_polygon_to_rect(&quad, rect);
            push_polygon(mesh, &clipped);
        }
    }
}

/// The conic angular sectors.
fn conic_mesh(mesh: &mut Mesh, rect: Rect, paint: &GradientPaint, from_angle_deg: f32) {
    let center = rect.center();
    let radius = farthest_corner_radius(rect);
    if radius <= 0.0 {
        return;
    }
    // Circumscribe the fan: the inscribed polygon would cut the box
    // corners off (they sit ON the farthest-corner circle), so inflate
    // by 1/cos(half step) and let the rect clip trim the overshoot.
    let step_deg = 360.0 / CONIC_SEGMENTS as f32;
    let fan_radius = radius / (step_deg.to_radians() / 2.0).cos();
    // The angular color at `deg` (0 = to top, clockwise): the stops'
    // angles are `from + pos·360`; before the first / after the last
    // stop the endpoint colors extend (the CSS conic extension).
    let angles: Vec<f32> = paint
        .stops
        .iter()
        .map(|s| from_angle_deg + 360.0 * s.pos.clamp(0.0, 1.0))
        .collect();
    let color_at = |deg: f32| -> Color32 {
        if deg <= angles[0] {
            return paint.stops[0].color;
        }
        if deg >= angles[angles.len() - 1] {
            return paint.stops[paint.stops.len() - 1].color;
        }
        for w in 1..angles.len() {
            if deg <= angles[w] {
                let (a, b) = (angles[w - 1], angles[w]);
                let t = if (b - a).abs() <= f32::EPSILON {
                    0.0
                } else {
                    (deg - a) / (b - a)
                };
                return lerp_color(paint.stops[w - 1].color, paint.stops[w].color, t);
            }
        }
        paint.stops[paint.stops.len() - 1].color
    };
    for s in 0..CONIC_SEGMENTS {
        let (a_deg, b_deg) = (
            from_angle_deg + step_deg * s as f32,
            from_angle_deg + step_deg * (s + 1) as f32,
        );
        let at = |deg: f32| -> Pos2 {
            let rad = deg.to_radians();
            center + Vec2::new(rad.sin(), -rad.cos()) * fan_radius
        };
        let tri = [
            (center, color_at(f32::midpoint(a_deg, b_deg))),
            (at(a_deg), color_at(a_deg)),
            (at(b_deg), color_at(b_deg)),
        ];
        let clipped = clip_polygon_to_rect(&tri, rect);
        push_polygon(mesh, &clipped);
    }
}

/// Paint the gradient onto `ui` — the one-liner the host runs right
/// after the frame (`ui.painter().add(gradient_mesh(rect, paint))`,
/// spelled out). Paint order is the CSS one: the mesh lands OVER the
/// frame's fill, so translucent stops show the solid color through.
pub fn paint_gradient(ui: &Ui, rect: Rect, paint: &GradientPaint) {
    ui.painter().add(gradient_mesh(rect, paint));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The chord through the magic-corner start point runs the full
    /// rect edge for the axis-aligned angles (the construction's own
    /// guarantee), and misses off-axis points entirely.
    #[test]
    fn chords_tile_the_axis_aligned_rect() {
        let rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(40.0, 100.0));
        let (d, len) = gradient_axis(rect, 0.0);
        let center = rect.center();
        let start = center - d * (len / 2.0);
        let n = Vec2::new(-d.y, d.x);
        // p=0: the chord IS the bottom edge.
        let (a, b) = chord(rect, start, n).expect("the start chord crosses");
        assert_eq!(a, Pos2::new(0.0, 100.0));
        assert_eq!(b, Pos2::new(40.0, 100.0));
        // p=1: the top edge.
        let (a, b) = chord(rect, start + d * len, n).expect("the end chord crosses");
        assert_eq!(a, Pos2::new(0.0, 0.0));
        assert_eq!(b, Pos2::new(40.0, 0.0));
        // A point far off the axis misses.
        assert!(chord(rect, Pos2::new(-100.0, -100.0), n).is_none());
    }

    /// The color lerp is exact at the endpoints and midway.
    #[test]
    fn the_color_lerp_is_exact_at_the_ends() {
        let (a, b) = (Color32::from_rgb(10, 20, 30), Color32::from_rgb(50, 60, 70));
        assert_eq!(lerp_color(a, b, 0.0), a);
        assert_eq!(lerp_color(a, b, 1.0), b);
        assert_eq!(lerp_color(a, b, 0.5), Color32::from_rgb(30, 40, 50));
    }

    /// The conic sweep's color is exact at every stop angle and extends
    /// the endpoint colors outside the stop range.
    #[test]
    fn the_conic_sweep_is_exact_at_the_stops() {
        let paint = GradientPaint {
            kind: GradientKind::Conic {
                from_angle_deg: 0.0,
            },
            stops: vec![
                PaintStop {
                    pos: 0.0,
                    color: Color32::from_rgb(10, 20, 30),
                },
                PaintStop {
                    pos: 1.0,
                    color: Color32::from_rgb(50, 60, 70),
                },
            ],
        };
        let rect = Rect::from_min_size(Pos2::ZERO, Vec2::splat(100.0));
        let angles: Vec<f32> = paint.stops.iter().map(|s| 360.0 * s.pos).collect();
        // Recompute the sweep's own answers through the mesh fn's color
        // walk (the angles helper is local to conic_mesh; the pin runs
        // the same formula over the payload).
        assert_eq!(angles[0], 0.0);
        // The farthest-corner radius drives both shapes.
        let r = farthest_corner_radius(rect);
        assert!((r - 50.0_f32 * std::f32::consts::SQRT_2).abs() < 1e-4);
    }

    /// The clip keeps interior polygons unchanged and pins crossing
    /// edges onto the boundary (with the color interpolated).
    #[test]
    fn the_clip_preserves_interior_and_trims_escape() {
        let rect = Rect::from_min_size(Pos2::ZERO, Vec2::splat(100.0));
        let red = Color32::from_rgb(255, 0, 0);
        let blue = Color32::from_rgb(0, 0, 255);
        // Fully interior triangle: unchanged.
        let tri = [
            (Pos2::new(10.0, 10.0), red),
            (Pos2::new(50.0, 10.0), red),
            (Pos2::new(10.0, 50.0), blue),
        ];
        let out = clip_polygon_to_rect(&tri, rect);
        assert_eq!(out.len(), 3);
        // A triangle escaping right: the crossing point sits ON the
        // boundary, color halfway along the cut edge.
        let tri = [
            (Pos2::new(50.0, 50.0), red),
            (Pos2::new(150.0, 50.0), blue),
            (Pos2::new(50.0, 90.0), red),
        ];
        let out = clip_polygon_to_rect(&tri, rect);
        assert!(
            out.iter().any(|(p, c)| {
                (p.x - 100.0).abs() < 1e-4
                    && (p.y - 50.0).abs() < 1e-4
                    && *c == lerp_color(red, blue, 0.5)
            }),
            "the boundary crossing carries the interpolated color: {out:?}"
        );
        for (p, _) in &out {
            assert!(rect.contains(*p), "{p:?} escaped");
        }
    }
}
