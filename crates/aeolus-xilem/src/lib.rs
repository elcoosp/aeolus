//! # Aeolus Xilem backend — `StylePlan` → Masonry properties
//!
//! Lowers [`aeolus_core::StylePlan`] into Masonry's native styling primitives.
//! Two paths are provided, mirroring the mission brief's capability model:
//!
//! 1. **`lower` — the native-cascade path.** Produces a real
//!    [`masonry::core::PropertyStack`] whose layers are gated by
//!    [`masonry::core::Selector`] pseudo-flags. Once upstream gap #4
//!    (view-side stack registration, see `reports/sections/xilem-masonry.md`
//!    Q10.4) lands, this is the zero-widget-work path.
//! 2. **[`widget::StyledWidget`] — the documented fallback.** A passthrough
//!    box widget that resolves the plan *per state at pre-paint time*
//!    (repaint-only on hover/active/focus/disabled flips, no view rebuild,
//!    no relayout for cosmetic changes). Works on today's Masonry because
//!    views cannot attach stack layers yet (audit Q1).
//!
//! Both paths produce identical visual results for the demo classes; test
//! `lower_matches_handwritten` pins the lowering and test
//! `hover_restyle_is_paint_only` pins the invalidation behavior.

pub mod button;
pub mod program;
pub mod view;
pub mod widget;

pub use program::{REM_PX, env_for_state, lower_program, lower_program_with};

use masonry::core::PropertySet;
use masonry::kurbo::Point;
use masonry::layout::{Dim, Length, UnitPoint};
use masonry::peniko::color::{ColorSpaceTag, HueDirection};
use masonry::peniko::{ColorStops, Extend};
use masonry::properties::types::{
    Gradient as MasonryGradient, GradientShape, RadialGradientExtent, RadialGradientShape,
};
use masonry::properties::{
    Background, BorderColor, BorderWidth, BoxShadow, ContentColor, CornerRadius, Dimensions, Gap,
    Padding,
};

use aeolus_core::color::Color;
use aeolus_core::plan::{
    Gradient, GradientStops, ResolvedStyle, SizedValue, StylePlan, VariantCondition,
};

/// Everything the lowering could not express in Masonry properties.
///
/// Aeolus never drops a class silently: conditions/properties without a
/// Masonry equivalent are reported here (and should be surfaced through
/// `tracing::warn!` by the host, matching `aeolus-core`'s philosophy).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LoweringReport {
    /// `(what, reason)` pairs, e.g. `("dark", "no per-widget dark signal")`.
    pub dropped: Vec<(String, String)>,
}

impl LoweringReport {
    fn drop(&mut self, what: impl Into<String>, reason: impl Into<String>) {
        self.dropped.push((what.into(), reason.into()));
    }

    /// True when nothing was dropped (the plan fully mapped).
    pub fn is_empty(&self) -> bool {
        self.dropped.is_empty()
    }
}

/// Convert our Oklab `Color` to Masonry's `AlphaColor<Srgb>`.
///
/// Masonry colors are straight-alpha sRGB floats; `Color::to_srgba` performs
/// the OKLab → linear-sRGB → sRGB transfer conversion (and clamps out-of-gamut
/// values, which Tailwind's own palette never produces).
pub fn to_masonry_color(
    c: Color,
) -> masonry::peniko::color::AlphaColor<masonry::peniko::color::Srgb> {
    let [r, g, b, a] = c.to_srgba();
    masonry::peniko::color::AlphaColor::new([r, g, b, a])
}

/// Lower one resolved style into a Masonry [`PropertySet`].
///
/// Slots Masonry cannot express (margin, layout, opacity, ring, outline,
/// cursor, pointer-events, typography) are recorded as dropped — they are
/// capability-negotiated, never silently ignored.
pub fn lower_resolved(style: &ResolvedStyle, report: &mut LoweringReport) -> PropertySet {
    let mut set = PropertySet::new();
    if let Some(bg) = &style.background {
        set.insert(Background::Color(to_masonry_color(bg.0)));
    }
    // v0.6 §1.5: gradients lower NATIVELY — masonry's layout-invariant
    // `Background::Gradient` takes angle-based shapes and oklab
    // interpolation, exactly the CSS model the plan composes. The
    // single-brush reality: masonry's `Background` is Color|Gradient, so
    // a solid + gradient coexistence keeps the GRADIENT (CSS layers
    // background-image over background-color) and records the flattening.
    if let Some(gradient) = &style.background_gradient {
        match gradient {
            Gradient::None => {}
            Gradient::Conic { .. } => {
                report.drop(
                    "bg-conic-*",
                    "masonry's sweep paint carries an upstream vello angle-offset TODO \
(angles outside vello's 0..2π cull the gradient)",
                );
            }
            Gradient::Linear { angle_deg, stops } => {
                if style.background.is_some() {
                    report.drop(
                        "background-color under gradient",
                        "CSS layers background-image over background-color; the single \
Background property takes the gradient",
                    );
                }
                set.insert(Background::Gradient(linear_gradient(*angle_deg, stops)));
            }
            Gradient::Radial { stops } => {
                if style.background.is_some() {
                    report.drop(
                        "background-color under gradient",
                        "CSS layers background-image over background-color; the single \
Background property takes the gradient",
                    );
                }
                // Masonry's RadialGradientShape has no ellipse radii
                // (upstream `#[non_exhaustive]` TODO) — the CSS default
                // ellipse-farthest-corner lowers as the documented
                // circle-farthest-corner normalization.
                report.drop(
                    "bg-radial shape",
                    "CSS radial default is an ellipse; masonry has no ellipse radii — \
lowering as circle farthest-corner",
                );
                set.insert(Background::Gradient(MasonryGradient {
                    shape: GradientShape::Radial {
                        center: UnitPoint::CENTER,
                        shape: RadialGradientShape::CircleTo(RadialGradientExtent::FarthestCorner),
                    },
                    extend: Extend::Pad,
                    interpolation_cs: ColorSpaceTag::Oklab,
                    hue_direction: HueDirection::Shorter,
                    stops: masonry_stops(stops),
                }));
            }
        }
    }
    if let Some(cc) = &style.content_color {
        set.insert(ContentColor::new(to_masonry_color(cc.0)));
    }
    if let Some(p) = &style.padding {
        set.insert(Padding {
            left: Length::px(p.left as f64),
            right: Length::px(p.right as f64),
            top: Length::px(p.top as f64),
            bottom: Length::px(p.bottom as f64),
        });
    }
    if let Some(bc) = &style.border_color {
        set.insert(BorderColor::new(to_masonry_color(bc.0)));
    }
    if let Some(bw) = &style.border_width {
        // Masonry's BorderWidth is uniform (`border_width.rs:14-17`); Tailwind's
        // default border utilities are uniform too. Per-side `border-t` etc.
        // would need the per-side upstream gap — HONESTY RULE: report the
        // flattening, never silently drop it (only the top edge survives).
        if !(bw.top == bw.right && bw.right == bw.bottom && bw.bottom == bw.left) {
            report.drop(
                "border-width:per-side",
                "Masonry BorderWidth is uniform; using top edge",
            );
        }
        set.insert(BorderWidth::all(Length::px(bw.top as f64)));
    }
    if let Some(cr) = &style.corner_radius {
        // Masonry's CornerRadius is a single radius (`corner_radius.rs:13-15`);
        // use the max of the four corners so clipping stays conservative.
        let r = cr
            .top_left
            .max(cr.top_right)
            .max(cr.bottom_right)
            .max(cr.bottom_left);
        set.insert(CornerRadius::all(Length::px(r as f64)));
    }
    if let Some(dims) = &style.dimensions {
        let w = sized_to_dim(dims.width);
        let h = sized_to_dim(dims.height);
        match (w, h) {
            (Some(w), Some(h)) => {
                set.insert(Dimensions::new(w, h));
            }
            _ => report.drop(
                "w-/h-",
                "fractional/symbolic sizes have no Masonry Dim equivalent",
            ),
        }
    }
    if let Some(g) = &style.gap {
        set.insert(Gap::new(Length::px(g.0 as f64)));
    }
    if let Some(s) = &style.shadow {
        // Masonry's BoxShadow has no spread or inset yet
        // (`box_shadow.rs:10-16` upstream TODO) — those parts are dropped.
        set.insert(BoxShadow {
            color: to_masonry_color(s.color),
            offset: Point::new(s.offset_x as f64, s.offset_y as f64),
            blur_radius: Length::px(s.blur as f64),
        });
    }
    if style.margin.is_some() {
        report.drop("margin", "no Masonry property for this slot");
    }
    if style.layout.is_some() {
        report.drop("layout", "no Masonry property for this slot");
    }
    if style.opacity.is_some() {
        report.drop("opacity", "no Masonry property for this slot");
    }
    if style.ring.is_some() {
        report.drop("ring", "no Masonry property for this slot");
    }
    if style.outline.is_some() {
        report.drop("outline", "no Masonry property for this slot");
    }
    if style.cursor.is_some() {
        report.drop("cursor", "no Masonry property for this slot");
    }
    if style.pointer_events.is_some() {
        report.drop("pointer-events", "no Masonry property for this slot");
    }
    if !style.text.is_empty() {
        report.drop(
            "typography",
            "Masonry text styling is parley StyleSet per-widget (audit Q5a)",
        );
    }
    set
}

fn sized_to_dim(v: Option<SizedValue>) -> Option<Dim> {
    match v? {
        SizedValue::Px(px) => Some(Dim::Fixed(Length::px(px as f64))),
        SizedValue::Scale(n) => Some(Dim::Fixed(Length::px((n * 4.0) as f64))),
        _ => None,
    }
}

/// CSS angle (degrees; 0 = to top, clockwise) → masonry's linear-gradient
/// angle (radians).
///
/// DERIVED FROM MASONRY'S PAINT MATH, not its doc comment
/// (`get_peniko_linear_for_rect`): `x = sin(a)·len/2`, `y = cos(a)·len/2`,
/// start = center − (x, y), end = center + (x, y) — so angle 0 puts the
/// END of the gradient line at the BOTTOM (to bottom = CSS 180°), and the
/// exact CSS-parity conversion is `a = π − θ`:
/// - CSS 0° (to top)    → a = π   → end at top      ✓
/// - CSS 90° (to right) → a = π/2 → end at right    ✓
/// - CSS 180° (to bottom) → a = 0 → end at bottom   ✓
/// - CSS 45° (to top right) → a = 135° → end up-right ✓
///
/// Pinned by `tests/lowering.rs::linear_gradient_angle_conversion`.
fn css_angle_to_masonry(angle_deg: f32) -> f64 {
    std::f64::consts::PI - (angle_deg as f64).to_radians()
}

/// One composed gradient → masonry's layout-invariant `Gradient`.
fn linear_gradient(angle_deg: f32, stops: &GradientStops) -> MasonryGradient {
    MasonryGradient {
        shape: GradientShape::Linear {
            angle: css_angle_to_masonry(angle_deg),
        },
        extend: Extend::Pad,
        // v4.1.17 interpolates gradients `in oklab` by default (captured:
        // `--tw-gradient-position: to right in oklab`).
        interpolation_cs: ColorSpaceTag::Oklab,
        hue_direction: HueDirection::Shorter,
        stops: masonry_stops(stops),
    }
}

/// The composed from/via/to stops → peniko `ColorStops` with fraction
/// offsets. Missing positions are the v4 `@property` initial values
/// (from 0%, via 50%, to 100%).
fn masonry_stops(stops: &GradientStops) -> ColorStops {
    use aeolus_core::plan::{GRADIENT_FROM_POS_PCT, GRADIENT_TO_POS_PCT, GRADIENT_VIA_POS_PCT};
    let pos = |p: Option<f32>, default: f32| p.unwrap_or(default) / 100.0;
    let mut out: Vec<masonry::peniko::ColorStop> = Vec::with_capacity(3);
    out.push(masonry::peniko::ColorStop {
        offset: pos(stops.from.pos_pct, GRADIENT_FROM_POS_PCT),
        color: masonry::peniko::color::DynamicColor::from_alpha_color(to_masonry_color(
            stops.from.color,
        )),
    });
    if let Some(via) = stops.via {
        out.push(masonry::peniko::ColorStop {
            offset: pos(via.pos_pct, GRADIENT_VIA_POS_PCT),
            color: masonry::peniko::color::DynamicColor::from_alpha_color(to_masonry_color(
                via.color,
            )),
        });
    }
    out.push(masonry::peniko::ColorStop {
        offset: pos(stops.to.pos_pct, GRADIENT_TO_POS_PCT),
        color: masonry::peniko::color::DynamicColor::from_alpha_color(to_masonry_color(
            stops.to.color,
        )),
    });
    ColorStops::from(out.as_slice())
}

/// Read property `P` from layer `index` of a lowered stack.
/// (Diagnostic/test helper; the resolution itself is Masonry's job.)
pub fn layer_get<P: masonry::core::Property>(
    stack: &masonry::core::PropertyStack,
    index: usize,
) -> Option<&P> {
    stack.get_layers().get(index)?.1.get::<P>()
}

/// Lower a full [`StylePlan`] into a Masonry [`PropertyStack`](masonry::core::PropertyStack).
///
/// Layer order follows Masonry's top-of-stack-wins resolution
/// (`property_stack.rs:107-115`): the base layer is pushed first, then
/// conditional layers in ascending [`VariantCondition::rank`] order, so a
/// hover+active widget resolves the *active* layer — matching Tailwind v4's
/// cascade intent.
pub fn lower(plan: &StylePlan) -> (masonry::core::PropertyStack, LoweringReport) {
    use masonry::core::{PropertyStack, Selector};

    let mut report = LoweringReport::default();

    // Conditions Masonry's `Selector` cannot express.
    for (group_id, _) in &plan.group {
        report.drop(
            format!("group:{group_id}"),
            "no ancestor-state selectors (audit Q2 gap)",
        );
    }
    for (peer_id, _) in &plan.peer {
        report.drop(
            format!("peer:{peer_id}"),
            "no sibling-state selectors (audit Q2 gap)",
        );
    }
    if plan.dark.is_some() {
        report.drop(
            "dark",
            "no per-widget dark-mode signal; use RenderRoot::set_default_properties",
        );
    }
    for bp in plan.responsive.keys() {
        report.drop(
            format!("responsive:{bp:?}"),
            "no window-size class flag (audit Q10 gap #2)",
        );
    }
    for bp in plan.container.keys() {
        report.drop(
            format!("container:{bp:?}"),
            "no container queries in Masonry",
        );
    }

    let mut stack = PropertyStack::new();
    stack.push_layer(Selector::new(), lower_resolved(&plan.base, &mut report));

    let mut layers: Vec<&aeolus_core::plan::ConditionalLayer> = plan.layers.iter().collect();
    layers.sort_by_key(|l| l.condition.rank());
    for layer in layers {
        let selector = match layer.condition {
            VariantCondition::Hover => Selector::new().with_hovered(true),
            VariantCondition::Active => Selector::new().with_active(true),
            VariantCondition::Disabled => Selector::new().with_disabled(true),
            // Tailwind `focus:` targets keyboard focus; Masonry's Selector has a
            // single focus flag (`has_focus_target`), so both map to it.
            VariantCondition::Focus | VariantCondition::FocusVisible => {
                Selector::new().with_focused(true)
            }
            VariantCondition::FocusWithin => {
                report.drop("focus-within", "Selector has no focus-within flag");
                continue;
            }
            VariantCondition::Visited => {
                report.drop("visited", "Selector has no visited flag");
                continue;
            }
        };
        stack.push_layer(selector, {
            // v0.8 §1.2: stack entries replace WHOLE properties, so a
            // partial-edge layer (`hover:px-8`) is materialized against the
            // base first — its masked-off edges inherit the base's values
            // instead of wiping them with the 0.0 placeholders.
            let mut style = layer.style.clone();
            style.materialize_unset_edges_from(&plan.base);
            lower_resolved(&style, &mut report)
        });
    }
    (stack, report)
}
