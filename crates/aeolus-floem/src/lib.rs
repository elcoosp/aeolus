//! # Aeolus Floem backend — `StylePlan` → Floem programmatic styles
//!
//! Floem has a fully programmatic rule API (no CSS text parsing anywhere):
//! [`floem::style::Style`] nested selector builders — `.hover(...)`,
//! `.focus(...)`, `.active(...)`, `.disabled(...)`, `.focus_within(...)`,
//! `.dark_mode(...)`, `.min_window_width(...)` — attached to views via
//! [`floem::views::Decorators::style`]. A `StylePlan` therefore lowers into a
//! single `Style` value **with zero view-side code** (audit Q3: YES), and
//! hover/active flips restyle+repaint one element without rebuilding widgets
//! (floem's style cache + merge-id diffing).
//!
//! This is the closest native analogue to CSS of all audited frameworks;
//! the only gaps are group/peer selectors (no ancestor/sibling state) and
//! container queries.

use floem::peniko::{Brush, Color as PenikoColor};
use floem::style::unit::{Length, LengthAuto, Pt};
use floem::style::{Style, Transition};
use floem::views::Decorators;

use smallvec::smallvec;

use aeolus_core::plan::{
    ConditionalLayer, FlexDirection, ResolvedStyle, SizedValue, StylePlan, TextProperty,
    VariantCondition,
};
use aeolus_core::{Breakpoint, Color};

/// Everything the lowering could not express in Floem.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LoweringReport {
    /// `(what, reason)` pairs; never dropped silently.
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

/// Convert our Oklab [`Color`] to Floem's `peniko::Color` (straight sRGB).
pub fn to_floem_color(c: Color) -> PenikoColor {
    let [r, g, b, a] = c.to_srgba();
    PenikoColor::new([r, g, b, a])
}

/// Tailwind px → floem [`Length::Pt`], 1:1.
///
/// UNIT AUDIT (Phase 2): floem @ 1351ffb's `Pt` is NOT a CSS point (1/72
/// inch) — `src/style/unit.rs` documents it as "absolute length value in
/// Floem layout points … logical layout points rather than device pixels",
/// and `Length::to_taffy` passes the value through unscaled into taffy
/// (`Length::Pt(v) => LengthPercentage::length(v)`); the window scale factor
/// is applied later at the renderer. So `p-4` = 16 logical px = 16 device px
/// at 96dpi. Pinned by `tests/lowering.rs::pt_is_logical_pixel_not_css_point`
/// (mutation-verified: CSS-point semantics fail it immediately).
fn pt(px: f32) -> Length {
    Length::Pt(px as f64)
}

fn brush(c: Color) -> Brush {
    Brush::Solid(to_floem_color(c))
}

/// Tailwind v4 default breakpoints in px (from `--breakpoint-*`: 40rem etc.).
fn breakpoint_px(bp: Breakpoint) -> f64 {
    f64::from(bp.default_px())
}

/// Apply one resolved style onto a Floem [`Style`] (builder-style).
///
/// Every `Some` slot becomes a Floem prop; unsupported slots are recorded in
/// the report, never silently dropped.
pub fn apply_resolved(mut style: Style, s: &ResolvedStyle, report: &mut LoweringReport) -> Style {
    if let Some(bg) = &s.background {
        style = style.background(brush(bg.0));
    }
    if let Some(cc) = &s.content_color {
        // Floem's inherited text-color prop (`TextColor color`); the setter
        // takes the inner `Color` and wraps it in `Some`.
        style = style.color(to_floem_color(cc.0));
    }
    if let Some(p) = &s.padding {
        style = style
            .padding_left(pt(p.left))
            .padding_top(pt(p.top))
            .padding_right(pt(p.right))
            .padding_bottom(pt(p.bottom));
    }
    if let Some(m) = &s.margin {
        // Floem margin props are LengthAuto; margins accept the same px values.
        let l = LengthAuto::Pt(m.left as f64);
        let t = LengthAuto::Pt(m.top as f64);
        let r = LengthAuto::Pt(m.right as f64);
        let b = LengthAuto::Pt(m.bottom as f64);
        style = style
            .margin_left(l)
            .margin_top(t)
            .margin_right(r)
            .margin_bottom(b);
    }
    if let Some(bc) = &s.border_color {
        let c = brush(bc.0);
        style = style
            .border_left_color(c.clone())
            .border_top_color(c.clone())
            .border_right_color(c.clone())
            .border_bottom_color(c);
    }
    if let Some(bw) = &s.border_width {
        // Floem models border geometry as a per-side `kurbo::Stroke`; the
        // color lives in the separate `border_*_color` props.
        use floem::peniko::kurbo::Stroke;
        let left = Stroke::new(bw.left as f64);
        let top = Stroke::new(bw.top as f64);
        let right = Stroke::new(bw.right as f64);
        let bottom = Stroke::new(bw.bottom as f64);
        style = style
            .border_left(left)
            .border_top(top)
            .border_right(right)
            .border_bottom(bottom);
    }
    if let Some(cr) = &s.corner_radius {
        style = style
            .border_top_left_radius(pt(cr.top_left))
            .border_top_right_radius(pt(cr.top_right))
            .border_bottom_right_radius(pt(cr.bottom_right))
            .border_bottom_left_radius(pt(cr.bottom_left));
    }
    if let Some(d) = &s.dimensions {
        style = apply_dim(style, d.width, |s, v| s.width(v));
        style = apply_dim(style, d.height, |s, v| s.height(v));
    }
    if let Some(d) = &s.min_dimensions {
        style = apply_dim(style, d.width, |s, v| s.min_width(v));
        style = apply_dim(style, d.height, |s, v| s.min_height(v));
    }
    if let Some(d) = &s.max_dimensions {
        style = apply_dim(style, d.width, |s, v| s.max_width(v));
        style = apply_dim(style, d.height, |s, v| s.max_height(v));
    }
    if let Some(g) = &s.gap {
        style = style.gap(pt(g.0));
    }
    if let Some(sh) = &s.shadow {
        // Floem's BoxShadow has spread AND inset-compatible per-side offsets:
        // CSS `offset-x` maps to left/right offsets by sign.
        let zero = Length::Pt(0.0);
        let shadow = floem::style::BoxShadow {
            color: to_floem_color(sh.color),
            blur_radius: Length::Pt(sh.blur as f64),
            spread: Length::Pt(sh.spread as f64),
            left_offset: if sh.offset_x >= 0.0 {
                pt(sh.offset_x)
            } else {
                zero
            },
            right_offset: if sh.offset_x < 0.0 {
                pt(-sh.offset_x)
            } else {
                zero
            },
            top_offset: if sh.offset_y >= 0.0 {
                pt(sh.offset_y)
            } else {
                zero
            },
            bottom_offset: if sh.offset_y < 0.0 {
                pt(-sh.offset_y)
            } else {
                zero
            },
        };
        style = style.apply_box_shadows(smallvec![shadow]);
    }
    if let Some(op) = &s.opacity {
        style = style.opacity(op.0);
    }
    if let Some(layout) = &s.layout {
        if let Some(dir) = &layout.flex_direction {
            style = style.flex_direction(match dir {
                FlexDirection::Row => floem::style::FlexDirection::Row,
                FlexDirection::Column => floem::style::FlexDirection::Column,
                FlexDirection::RowReverse => floem::style::FlexDirection::RowReverse,
                FlexDirection::ColumnReverse => floem::style::FlexDirection::ColumnReverse,
            });
        }
        if let Some(j) = &layout.justify_content {
            use aeolus_core::plan::JustifyContent as J;
            style = style.justify_content(match j {
                J::Start => floem::style::JustifyContent::FlexStart,
                J::Center => floem::style::JustifyContent::Center,
                J::End => floem::style::JustifyContent::FlexEnd,
                J::SpaceBetween => floem::style::JustifyContent::SpaceBetween,
                J::SpaceAround => floem::style::JustifyContent::SpaceAround,
                J::SpaceEvenly => floem::style::JustifyContent::SpaceEvenly,
            });
        }
        if let Some(g) = layout.flex_grow {
            style = style.flex_grow(g);
        }
        if let Some(shr) = layout.flex_shrink {
            style = style.flex_shrink(shr);
        }
    }
    // Text properties: Floem has first-class typography.
    for (prop, value) in &s.text {
        match (prop, value) {
            (TextProperty::FontSize, aeolus_core::plan::TextValue::Px(px)) => {
                style = style.font_size(Pt(*px as f64));
            }
            (TextProperty::Color, aeolus_core::plan::TextValue::Color(c)) => {
                style = style.color(to_floem_color(*c));
            }
            _ => report.drop(
                format!("text:{prop:?}"),
                "no direct Floem prop mapping in v1",
            ),
        }
    }
    for (what, present) in [
        ("ring", s.ring.is_some()),
        ("outline", s.outline.is_some()),
        ("cursor", s.cursor.is_some()),
        ("pointer-events", s.pointer_events.is_some()),
    ] {
        if present {
            report.drop(what, "no direct Floem prop mapping in v1");
        }
    }
    style
}

fn apply_dim(
    style: Style,
    v: Option<SizedValue>,
    f: impl FnOnce(Style, LengthAuto) -> Style,
) -> Style {
    match v {
        Some(SizedValue::Px(px)) => f(style, LengthAuto::Pt(px as f64)),
        Some(SizedValue::Scale(n)) => f(style, LengthAuto::Pt((n * 4.0) as f64)),
        Some(SizedValue::Fraction(frac)) => f(style, LengthAuto::Pct((frac * 100.0) as f64)),
        Some(SizedValue::Full) => f(style, LengthAuto::Pct(100.0)),
        // FitContent/Auto have no Floem equivalent beyond the default Auto.
        Some(SizedValue::FitContent) | Some(SizedValue::Auto) | None => style,
    }
}

fn apply_condition(
    style: Style,
    condition: VariantCondition,
    layer: &ResolvedStyle,
    report: &mut LoweringReport,
) -> Style {
    let f = |s: Style| apply_resolved(s, layer, report);
    match condition {
        VariantCondition::Hover => style.hover(f),
        VariantCondition::Active => style.active(f),
        VariantCondition::Disabled => style.disabled(f),
        VariantCondition::Focus => style.focus(f),
        VariantCondition::FocusVisible => style.focus_visible(f),
        VariantCondition::FocusWithin => style.focus_within(f),
        VariantCondition::Visited => {
            report.drop("visited", "Floem has no :visited selector");
            style
        }
    }
}

/// Lower a full [`StylePlan`] into a Floem [`Style`].
///
/// Layer order follows Floem's selector precedence: nested selector maps are
/// independent (state-gated), so Tailwind's rank order only matters for
/// properties set by MULTIPLE active conditions; Floem applies
/// `disabled > active > focus > hover` state priority internally
/// (`src/style/mod.rs:704-843`), which matches Tailwind's cascade intent.
pub fn lower(plan: &StylePlan) -> (Style, LoweringReport) {
    lower_with_transitions(plan, None)
}

/// Lower with optional per-property transitions on the base layer
/// (background / text color / border colors), e.g. `Some(150.0)` ms —
/// Tailwind's `transition-colors` equivalent.
pub fn lower_with_transitions(
    plan: &StylePlan,
    color_transition_ms: Option<f64>,
) -> (Style, LoweringReport) {
    let mut report = LoweringReport::default();

    for (group_id, _) in &plan.group {
        report.drop(
            format!("group:{group_id}"),
            "no ancestor-state selectors in Floem",
        );
    }
    for (peer_id, _) in &plan.peer {
        report.drop(
            format!("peer:{peer_id}"),
            "no sibling-state selectors in Floem",
        );
    }

    let mut style = apply_resolved(Style::new(), &plan.base, &mut report);

    if let Some(ms) = color_transition_ms {
        let mk = || Transition::linear(std::time::Duration::from_millis(ms as u64));
        style = style
            .transition_background(mk())
            .transition_color(mk())
            .transition_border_left_color(mk())
            .transition_border_top_color(mk())
            .transition_border_right_color(mk())
            .transition_border_bottom_color(mk());
    }

    let mut layers: Vec<&ConditionalLayer> = plan.layers.iter().collect();
    layers.sort_by_key(|l| l.condition.rank());
    for layer in layers {
        style = apply_condition(style, layer.condition, &layer.style, &mut report);
    }

    if let Some(dark) = &plan.dark {
        style = style.dark_mode(|s| apply_resolved(s, dark, &mut report));
    }

    for (bp, responsive) in &plan.responsive {
        let px = breakpoint_px(*bp);
        style = style.min_window_width(Pt(px), |s| apply_resolved(s, responsive, &mut report));
    }
    for bp in plan.container.keys() {
        report.drop(format!("container:{bp:?}"), "no container queries in Floem");
    }

    (style, report)
}

/// Attach a lowered plan to any Floem view: `view.tw(&plan)`.
///
/// User classes appended after component defaults win because repeated
/// `Decorators::style` calls stack and later slots win in the `StyleStack`
/// (`src/views/decorator.rs:80-95`, `src/view/state.rs:265-296`).
pub trait FloemTwExt: Decorators {
    /// Style this view from a pre-resolved plan.
    fn tw(self, plan: &StylePlan) -> Self::Intermediate {
        let (style, report) = lower(plan);
        if !report.is_empty() {
            tracing::warn!("aeolus-floem: dropped {:?} while lowering", report.dropped);
        }
        self.style(move |s| s.apply(style.clone()))
    }

    /// Style this view from classes, parsed once per call with `resolver`.
    fn tw_classes(self, resolver: &aeolus_core::Resolver, classes: &str) -> Self::Intermediate {
        match resolver.resolve(classes) {
            Ok(resolution) => self.tw(&resolution.plan),
            Err(err) => {
                if cfg!(debug_assertions) {
                    panic!("aeolus: failed to parse class string {classes:?}: {err:?}");
                }
                tracing::warn!("aeolus: dropping unparseable class string {classes:?}: {err:?}");
                // Degrade to no-op styling: apply an empty rule so the return
                // type still matches the styled chain.
                self.style(|s| s)
            }
        }
    }
}

impl<V: Decorators> FloemTwExt for V {}
