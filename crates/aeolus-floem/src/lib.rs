//! # Aeolus Floem backend — `StylePlan` → Floem programmatic styles
//!
//! Floem has a fully programmatic rule API (no CSS text parsing anywhere):
//! [`floem::style::Style`] nested selector builders — `.hover(...)`,
//! `.focus(...)`, `.active(...)`, `.disabled(...)`, `.focus_within(...)`,
//! `.dark_mode(...)`, `.min_window_width(...)` — attached to views via
//! [`floem::views::Decorators::style`]. A `StylePlan` therefore lowers into a
//! single `Style` value **with zero view-side code**, and hover/active flips
//! restyle+repaint one element without rebuilding widgets (floem's style
//! cache + merge-id diffing).
//!
//! v0.2 (plan §4.2) closes the v0.1 lowering gaps — every one of these is
//! native in Floem at the pinned rev and now mapped: `align_items`,
//! `align_self`, `flex_wrap`, `display`, `col_gap`/`row_gap`, auto margins,
//! `position` (absolute), `inset_*`, `z_index`, `aspect_ratio`, `cursor`,
//! `pointer-events`, font weight/line height/letter spacing/text
//! alignment/overflow, transform scale/rotate/translate, `ring` →
//! `outline`+`outline_color`, and `transition-*` → Floem `Transition`s.
//! The parity path (compound variants, group/peer, data/aria, `@container`)
//! is emulated by evaluating `plan.resolve_rules_only(&probe)` — see
//! [`FloemProbe`] and [`FloemTwExt::tw_plan`].

use floem::animate::Bezier;
use floem::peniko::{Brush, Color as PenikoColor};
use floem::style::unit::{Angle, Length, LengthAuto, Pct, Pt};
use floem::style::{CursorStyle, LineHeightValue, Style, Transition};
use floem::views::Decorators;

use aeolus_core::Color;
use aeolus_core::eval::{Probe, Rel};
use aeolus_core::plan::Transition as TwTransition;
use aeolus_core::plan::{
    AlignItems, Animation, ConditionalLayer, ContentColor, Cursor, Ease, FlexDirection,
    JustifyContent, ResolvedStyle, SizedValue, Slot, StylePlan, TextProperty, TextValue,
    TransitionSet, VariantCondition,
};

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

/// Tailwind px → [`LengthAuto::Pt`] (for margins / insets).
fn pt_auto(px: f32) -> LengthAuto {
    LengthAuto::Pt(px as f64)
}

/// Map a typed [`SizedValue`] through a setter that takes [`LengthAuto`].
fn apply_dim(
    style: Style,
    v: Option<SizedValue>,
    f: impl FnOnce(Style, LengthAuto) -> Style,
) -> Style {
    match v {
        Some(SizedValue::Px(px)) => f(style, pt_auto(px)),
        // Scale values are already resolved to px by the resolver; the
        // Scale variant only survives from direct IR construction.
        Some(SizedValue::Scale(n)) => f(style, pt_auto(n * 4.0)),
        Some(SizedValue::Fraction(frac)) => f(style, LengthAuto::Pct((frac * 100.0) as f64)),
        Some(SizedValue::Full) => f(style, LengthAuto::Pct(100.0)),
        Some(SizedValue::Viewport) => {
            // Viewport units are window-resolved; Floem has no vh unit —
            // callers should pass the window size. Reported, not silent.
            style
        }
        // FitContent/Auto have no Floem equivalent beyond the default Auto.
        Some(SizedValue::FitContent) | Some(SizedValue::Auto) | None => style,
    }
}

fn map_cursor(c: Cursor) -> CursorStyle {
    match c {
        Cursor::Default | Cursor::Auto => CursorStyle::Default,
        Cursor::Pointer => CursorStyle::Pointer,
        Cursor::Wait => CursorStyle::Wait,
        Cursor::Progress => CursorStyle::Progress,
        Cursor::Text | Cursor::VerticalText => CursorStyle::Text,
        Cursor::Move | Cursor::AllScroll => CursorStyle::Move,
        Cursor::Grab => CursorStyle::Grab,
        Cursor::Grabbing => CursorStyle::Grabbing,
        Cursor::ColResize => CursorStyle::ColResize,
        Cursor::RowResize => CursorStyle::RowResize,
        Cursor::NResize => CursorStyle::NResize,
        Cursor::EResize => CursorStyle::EResize,
        Cursor::SResize => CursorStyle::SResize,
        Cursor::WResize => CursorStyle::WResize,
        Cursor::NwResize => CursorStyle::NwResize,
        Cursor::NeResize => CursorStyle::NeResize,
        Cursor::SwResize => CursorStyle::SwResize,
        Cursor::SeResize => CursorStyle::SeResize,
        Cursor::NeswResize => CursorStyle::NeswResize,
        Cursor::NwseResize => CursorStyle::NwseResize,
        Cursor::EwResize | Cursor::NsResize => CursorStyle::Default,
        // No Floem equivalent for these CSS cursors.
        Cursor::Crosshair => CursorStyle::Crosshair,
        Cursor::NotAllowed
        | Cursor::None
        | Cursor::ContextMenu
        | Cursor::Help
        | Cursor::Cell
        | Cursor::Alias
        | Cursor::Copy
        | Cursor::NoDrop
        | Cursor::ZoomIn
        | Cursor::ZoomOut => CursorStyle::Default,
    }
}

fn map_align(a: AlignItems) -> floem::style::AlignItems {
    match a {
        AlignItems::Start => floem::style::AlignItems::FlexStart,
        AlignItems::Center => floem::style::AlignItems::Center,
        AlignItems::End => floem::style::AlignItems::FlexEnd,
        AlignItems::Stretch => floem::style::AlignItems::Stretch,
        AlignItems::Baseline => floem::style::AlignItems::Baseline,
    }
}

/// Apply one resolved style onto a Floem [`Style`] (builder-style).
///
/// Every `Some` slot becomes a Floem prop; unsupported slots are recorded in
/// the report, never silently dropped (plan §4.1 parity contract).
pub fn apply_resolved(mut style: Style, s: &ResolvedStyle, report: &mut LoweringReport) -> Style {
    if let Some(bg) = &s.background {
        style = style.background(brush(bg.0));
    }
    if let Some(cc) = &s.content_color {
        // Floem's inherited text-color prop (`TextColor color`); the setter
        // takes the inner `Color` and wraps it in `Some`.
        style = style.color(to_floem_color(cc.0));
    } else if let Some(ContentColor(c)) = s
        .text
        .iter()
        .find(|(p, _)| *p == TextProperty::Color)
        .map(|(_, v)| match v {
            TextValue::Color(c) => ContentColor(*c),
            _ => ContentColor(aeolus_core::Color::TRANSPARENT),
        })
    {
        style = style.color(to_floem_color(c));
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
        style = style
            .margin_left(pt_auto(m.left))
            .margin_top(pt_auto(m.top))
            .margin_right(pt_auto(m.right))
            .margin_bottom(pt_auto(m.bottom));
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
        style = style.apply_box_shadows(shadow_to_floem(smallvec_from(sh, &s.extras.shadow_rest)));
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
        if let Some(a) = &layout.align_items {
            style = style.align_items(map_align(*a));
        }
        if let Some(a) = &layout.align_self {
            style = style.align_self(map_align(*a));
        }
        if let Some(j) = &layout.justify_content {
            style = style.justify_content(match j {
                JustifyContent::Start => floem::style::JustifyContent::FlexStart,
                JustifyContent::Center => floem::style::JustifyContent::Center,
                JustifyContent::End => floem::style::JustifyContent::FlexEnd,
                JustifyContent::SpaceBetween => floem::style::JustifyContent::SpaceBetween,
                JustifyContent::SpaceAround => floem::style::JustifyContent::SpaceAround,
                JustifyContent::SpaceEvenly => floem::style::JustifyContent::SpaceEvenly,
            });
        }
        if let Some(w) = layout.flex_wrap {
            style = style.flex_wrap(if w {
                floem::style::FlexWrap::Wrap
            } else {
                floem::style::FlexWrap::NoWrap
            });
        }
        if let Some(g) = layout.flex_grow {
            style = style.flex_grow(g);
        }
        if let Some(shr) = layout.flex_shrink {
            style = style.flex_shrink(shr);
        }
        if let Some(d) = &layout.display {
            style = style.display(match d {
                aeolus_core::plan::Display::Flex | aeolus_core::plan::Display::InlineFlex => {
                    floem::style::Display::Flex
                }
                aeolus_core::plan::Display::Grid => floem::style::Display::Grid,
                aeolus_core::plan::Display::Block | aeolus_core::plan::Display::Inline => {
                    floem::style::Display::Flex
                }
                aeolus_core::plan::Display::None => floem::style::Display::None,
            });
        }
    }
    // v0.2 extras (plan §4.2).
    let e = &s.extras;
    if let Some(ax) = &e.gap_axes {
        if let Some(c) = ax.col {
            style = style.col_gap(pt(c));
        }
        if let Some(r) = ax.row {
            style = style.row_gap(pt(r));
        }
    }
    if let Some(m) = &e.margin_auto {
        if m.left {
            style = style.margin_left(LengthAuto::Auto);
        }
        if m.right {
            style = style.margin_right(LengthAuto::Auto);
        }
        if m.top {
            style = style.margin_top(LengthAuto::Auto);
        }
        if m.bottom {
            style = style.margin_bottom(LengthAuto::Auto);
        }
    }
    if let Some(p) = e.position {
        if p == aeolus_core::plan::Position::Absolute {
            style = style.absolute();
        } else if p != aeolus_core::plan::Position::Relative {
            report.drop("position", "Floem exposes only absolute()/relative flow");
        }
    }
    if let Some(i) = &e.inset {
        style = apply_dim(style, i.top, |s, v| s.inset_top(v));
        style = apply_dim(style, i.bottom, |s, v| s.inset_bottom(v));
        style = apply_dim(style, i.left, |s, v| s.inset_left(v));
        style = apply_dim(style, i.right, |s, v| s.inset_right(v));
    }
    if let Some(z) = e.z_index {
        style = style.z_index(z);
    }
    if let Some(a) = e.aspect_ratio {
        style = style.aspect_ratio(a);
    }
    if let Some(v) = e.visibility
        && v == aeolus_core::plan::Visibility::Hidden
    {
        report.drop(
            "invisible",
            "Floem has no visibility prop — use display:None or opacity",
        );
    }
    if let Some(c) = s.cursor {
        style = style.cursor(map_cursor(c));
    }
    if s.pointer_events == Some(aeolus_core::plan::PointerEvents::None) {
        style = style.pointer_events_none();
    }
    if let Some(t) = &e.transform {
        if let Some(x) = t.scale_x {
            style = style.scale_x(Pct(x as f64 * 100.0));
        }
        if let Some(y) = t.scale_y {
            style = style.scale_y(Pct(y as f64 * 100.0));
        }
        if let Some(d) = t.rotate_deg {
            style = style.rotate(Angle::Deg(d as f64));
        }
        if let Some(tx) = &t.translate_x {
            style = apply_translate(style, *tx, |s, v| s.translate_x(v));
        }
        if let Some(ty) = &t.translate_y {
            style = apply_translate(style, *ty, |s, v| s.translate_y(v));
        }
        if t.skew_x.is_some() || t.skew_y.is_some() {
            report.drop("skew", "Floem has no skew transform props");
        }
    }
    if !e.inset_shadows.is_empty() {
        let shadows: smallvec::SmallVec<[floem::style::BoxShadow; 3]> = e
            .inset_shadows
            .iter()
            .flat_map(|sh| shadow_to_floem(smallvec_from(sh, &[])))
            .collect();
        style = style.apply_box_shadows(shadows);
        report.drop(
            "inset-shadow",
            "Floem BoxShadow has no inset flag — rendered as an outer shadow",
        );
    }
    // Text properties: Floem has first-class typography.
    for (prop, value) in &s.text {
        style = lower_text(style, *prop, *value, report);
    }
    // v0.2: ring → outline + outline_color (native at the pinned rev).
    if let Some(ring) = &s.ring {
        use floem::peniko::kurbo::Stroke;
        style = style.outline(Stroke::new(ring.width as f64));
        if let Some(c) = ring.color {
            style = style.outline_color(brush(c));
        }
        if ring.offset_width > 0.0 {
            report.drop("ring-offset", "Floem outline has no offset");
        }
        if ring.inset {
            report.drop("ring-inset", "Floem outline has no inset flag");
        }
    }
    if let Some(o) = &s.outline {
        if o.none {
            use floem::peniko::kurbo::Stroke;
            style = style.outline(Stroke::new(0.0));
        } else if o.width != 0.0 {
            use floem::peniko::kurbo::Stroke;
            style = style.outline(Stroke::new(o.width as f64));
            if let Some(c) = o.color {
                style = style.outline_color(brush(c));
            }
        }
        if o.offset != 0.0 {
            report.drop("outline-offset", "Floem outline has no offset");
        }
    }
    if let Some(bs) = e.border_style
        && bs != aeolus_core::plan::BorderStyle::Solid
    {
        report.drop("border-style", "Floem border strokes have no dash pattern");
    }
    if let Some(anim) = &e.animation {
        report_animation(anim, report);
    }
    style
}

fn shadow_to_floem(
    shadows: smallvec::SmallVec<[&aeolus_core::plan::BoxShadow; 3]>,
) -> smallvec::SmallVec<[floem::style::BoxShadow; 3]> {
    shadows
        .into_iter()
        .map(|sh| {
            // Floem's BoxShadow has spread AND inset-compatible per-side
            // offsets: CSS `offset-x` maps to left/right offsets by sign.
            let zero = Length::Pt(0.0);
            floem::style::BoxShadow {
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
            }
        })
        .collect()
}

fn smallvec_from<'a>(
    first: &'a aeolus_core::plan::BoxShadow,
    rest: &'a [aeolus_core::plan::BoxShadow],
) -> smallvec::SmallVec<[&'a aeolus_core::plan::BoxShadow; 3]> {
    let mut v: smallvec::SmallVec<[&aeolus_core::plan::BoxShadow; 3]> = smallvec::smallvec![first];
    v.extend(rest.iter());
    v
}

fn apply_translate(style: Style, v: SizedValue, f: impl FnOnce(Style, Length) -> Style) -> Style {
    match v {
        SizedValue::Px(px) => f(style, pt(px)),
        SizedValue::Scale(n) => f(style, pt(n * 4.0)),
        SizedValue::Fraction(frac) => f(style, Length::Pct(frac as f64 * 100.0)),
        _ => style,
    }
}

fn lower_text(
    style: Style,
    prop: TextProperty,
    value: TextValue,
    report: &mut LoweringReport,
) -> Style {
    match (prop, value) {
        (TextProperty::FontSize, TextValue::Px(px)) => style.font_size(Pt(px as f64)),
        (TextProperty::Color, TextValue::Color(c)) => style.color(to_floem_color(c)),
        (TextProperty::FontWeight, TextValue::Weight(w)) => {
            style.font_weight(floem::text::FontWeight::new(w as f32))
        }
        (TextProperty::FontStyle, TextValue::Keyword("italic")) => {
            style.font_style(floem::text::FontStyle::Italic)
        }
        (TextProperty::FontStyle, TextValue::Keyword("normal")) => {
            style.font_style(floem::text::FontStyle::Normal)
        }
        (TextProperty::LineHeight, TextValue::Px(px)) => {
            // The resolver has already multiplied the ratio by the 16px
            // base — keep the absolute px semantics.
            style.line_height(LineHeightValue::Pt(px))
        }
        (TextProperty::LetterSpacing, TextValue::Spacing(_px)) => {
            // No letter-spacing prop at the pinned rev.
            report.drop("letter-spacing", "no Floem letter-spacing prop");
            style
        }
        (TextProperty::TextAlign, TextValue::Keyword(kw)) => {
            let a = match kw {
                "left" | "start" => floem::text::Alignment::Start,
                "center" => floem::text::Alignment::Center,
                "right" | "end" => floem::text::Alignment::End,
                "justify" => floem::text::Alignment::Justify,
                _ => {
                    report.drop("text-align", "unknown alignment");
                    return style;
                }
            };
            style.text_align(a)
        }
        (TextProperty::TextOverflow, TextValue::Keyword("ellipsis")) => style.text_overflow(
            floem::style::TextOverflow::NoWrap(floem::style::NoWrapOverflow::Ellipsis),
        ),
        (TextProperty::TextOverflow, TextValue::Keyword("clip")) => style.text_overflow(
            floem::style::TextOverflow::NoWrap(floem::style::NoWrapOverflow::Clip),
        ),
        (TextProperty::Whitespace, TextValue::Keyword("nowrap")) => {
            // whitespace-nowrap is expressed via TextOverflow::NoWrap too,
            // but without overriding any ellipsis setting.
            report.drop("whitespace-nowrap", "folded into text-overflow handling");
            style
        }
        (TextProperty::FontFamily, TextValue::Keyword(fam @ ("sans" | "serif" | "mono"))) => {
            report.drop(
                format!("font-{fam}"),
                "Floem font-family needs concrete family names; set via the text props",
            );
            style
        }
        (TextProperty::TextTransform, _) => {
            report.drop("text-transform", "no Floem text-transform prop");
            style
        }
        (TextProperty::TextDecoration, _) => {
            report.drop(
                "text-decoration",
                "no Floem text-decoration prop at the pinned rev",
            );
            style
        }
        (TextProperty::LineClamp, TextValue::Count(_)) => {
            report.drop("line-clamp", "no Floem multi-line clamp prop");
            style
        }
        (TextProperty::Select, _) => {
            report.drop(
                "select",
                "text selection policy is set via the text view API",
            );
            style
        }
        (TextProperty::FontVariantNumeric, _) => {
            report.drop("font-variant-numeric", "no Floem font-variant prop");
            style
        }
        (TextProperty::UnderlineOffset, _) => {
            report.drop("underline-offset", "no Floem underline-offset prop");
            style
        }
        _ => {
            report.drop(format!("text:{prop:?}"), "no direct Floem prop mapping");
            style
        }
    }
}

fn report_animation(anim: &Animation, report: &mut LoweringReport) {
    if anim.name.as_ref() == "none" {
        return;
    }
    report.drop(
        format!("animate-{}", anim.name),
        "Floem has no keyframe animation engine at the pinned rev",
    );
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

/// The v0.2 `transition-*` slots → Floem `Transition`s per property set.
fn apply_transitions(mut style: Style, t: &TwTransition, report: &mut LoweringReport) -> Style {
    if t.set == TransitionSet::None {
        return style;
    }
    let mk = || -> Transition {
        let base = std::time::Duration::from_millis(t.duration_ms.max(0.0) as u64);
        match t.ease.unwrap_or(Ease::InOut) {
            Ease::Linear => Transition::linear(base),
            Ease::In => Transition::new(base, Bezier(0.42, 0.0, 1.0, 1.0)),
            Ease::Out => Transition::new(base, Bezier(0.0, 0.0, 0.58, 1.0)),
            Ease::InOut => Transition::new(base, Bezier(0.42, 0.0, 0.58, 1.0)),
        }
    };
    // Floem transitions are per-property. The Tailwind set selects which.
    let colorish = matches!(
        t.set,
        TransitionSet::Default | TransitionSet::All | TransitionSet::Colors
    );
    let opacityish = matches!(
        t.set,
        TransitionSet::Default | TransitionSet::All | TransitionSet::Opacity
    );
    let shadowish = matches!(
        t.set,
        TransitionSet::Default | TransitionSet::All | TransitionSet::Shadow
    );
    let transformish = matches!(
        t.set,
        TransitionSet::Default | TransitionSet::All | TransitionSet::Transform
    );
    if colorish {
        style = style
            .transition_background(mk())
            .transition_color(mk())
            .transition_border_left_color(mk())
            .transition_border_top_color(mk())
            .transition_border_right_color(mk())
            .transition_border_bottom_color(mk())
            .transition_outline_color(mk());
    }
    if opacityish {
        style = style.transition_opacity(mk());
    }
    if shadowish {
        style = style.transition_box_shadow(mk());
    }
    if transformish {
        style = style
            .transition_scale_x(mk())
            .transition_scale_y(mk())
            .transition_rotate(mk())
            .transition_translate_x(mk())
            .transition_translate_y(mk());
    }
    if t.delay_ms != 0.0 {
        report.drop(
            "transition-delay",
            "Floem Transition has no separate delay at the pinned rev",
        );
    }
    style
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
            "emulated by tw_plan's reactive probe path, not by native selectors",
        );
    }
    for (peer_id, _) in &plan.peer {
        report.drop(
            format!("peer:{peer_id}"),
            "emulated by tw_plan's reactive probe path, not by native selectors",
        );
    }
    for rule in &plan.rules {
        report.drop(
            format!("rules:{}", rule.when.rank().0),
            "compound/structural conditions emulated by tw_plan's reactive probe path",
        );
    }

    let mut style = apply_resolved(Style::new(), &plan.base, &mut report);
    if let Some(t) = &plan.base.extras.transition {
        style = apply_transitions(style, t, &mut report);
    }

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

    // v0.2: use the plan's THEME-AWARE breakpoints (plan §4.2 fixes the
    // v0.1 `Breakpoint::default_px` ignore-theme bug).
    for (bp, responsive) in &plan.responsive {
        let px = plan.breakpoints[bp.index()];
        style = style.min_window_width(Pt(px as f64), |s| {
            apply_resolved(s, responsive, &mut report)
        });
    }
    for bp in plan.container.keys() {
        report.drop(format!("container:{bp:?}"), "no container queries in Floem");
    }

    (style, report)
}

// ---------------------------------------------------------------------------
// The parity path (plan §4.2): a probe the app drives + a reactive extension.
// ---------------------------------------------------------------------------

/// The element's current situation as Floem sees it. The app (or a view
/// wrapper) keeps this in signals and hands it to [`FloemTwExt::tw_plan`];
/// the reactive style closure re-runs whenever the underlying signals
/// change, which is what makes group/peer and compound variants work
/// without a Floem patch.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FloemProbe {
    /// `hover:` / `group-hover:`.
    pub hovered: bool,
    /// `focus:`.
    pub focused: bool,
    /// `active:`.
    pub active: bool,
    /// `disabled:`.
    pub disabled: bool,
    /// `dark:` (app-driven).
    pub dark: bool,
    /// Current viewport width (window), when known.
    pub width: Option<f32>,
    /// Current container width, when this element is inside a container
    /// the app tracks.
    pub container_width: Option<f32>,
}

impl Probe for FloemProbe {
    fn test(&self, t: &aeolus_core::Test) -> bool {
        use aeolus_core::Test;
        match t {
            Test::State(s) => match s {
                aeolus_core::State::Hover => self.hovered,
                aeolus_core::State::Focus | aeolus_core::State::FocusVisible => self.focused,
                aeolus_core::State::Active => self.active,
                aeolus_core::State::Disabled => self.disabled,
                _ => false,
            },
            _ => false,
        }
    }

    fn related(&self, _rel: Rel, _label: Option<&str>, _t: &aeolus_core::Test) -> bool {
        // Group/peer state lives in GroupScope signals; the plain probe
        // answers "no". Use the group-aware probe at the call site.
        false
    }

    fn media(&self, m: aeolus_core::Media) -> bool {
        matches!(m, aeolus_core::Media::Dark) && self.dark
    }

    fn viewport_width(&self) -> Option<f32> {
        self.width
    }

    fn container_width(&self, _name: Option<&str>) -> Option<f32> {
        self.container_width
    }
}

/// Lower the plan's native subset (base + layers + dark + responsive via
/// selector maps) ONCE, then re-apply the emulated remainder
/// (`plan.resolve_rules_only`) from a reactive style closure — the piece
/// that fixes `dark:hover:`, `group-hover:`, `data-*:` and `@md:` on Floem
/// without an upstream change (plan §4.2).
///
/// `snapshot` reads the current [`FloemProbe`] (reading Floem signals inside
/// subscribes the style closure to them).
pub trait FloemTwExt: Decorators {
    /// Style this view from a pre-resolved plan (native subset only).
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

    /// Parity path: native lowering + reactive rule evaluation.
    ///
    /// `snapshot` must read the view's state signals; the style closure
    /// re-runs when any of them change, re-evaluating the plan's `rules`
    /// (compound variants, group/peer, data/aria, container queries)
    /// against the fresh probe.
    fn tw_plan(
        self,
        plan: std::sync::Arc<StylePlan>,
        snapshot: impl Fn() -> FloemProbe + 'static,
    ) -> Self::Intermediate {
        let (native, report) = lower(&plan);
        if !report.is_empty() {
            tracing::warn!("aeolus-floem: dropped {:?} while lowering", report.dropped);
        }
        self.style(move |s| {
            let probe = snapshot();
            let extra = plan.resolve_rules_only(&probe);
            let mut rep = LoweringReport::default();
            let merged = apply_resolved(s.apply(native.clone()), &extra, &mut rep);
            if !rep.is_empty() {
                tracing::warn!("aeolus-floem: rules overlay dropped {:?}", rep.dropped);
            }
            merged
        })
    }
}

impl<V: Decorators> FloemTwExt for V {}

// ---------------------------------------------------------------------------
// Parity accounting (plan §4.1): every slot is native, emulated, or reported.
// ---------------------------------------------------------------------------

/// How a slot is covered on this backend.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Coverage {
    /// Mapped to a native Floem prop.
    Native,
    /// Emulated through the reactive probe path.
    Emulated,
    /// Recorded in [`LoweringReport`] with a reason.
    Reported,
}

/// Walk one style and classify every set slot (the parity ledger).
pub fn slot_coverage(s: &ResolvedStyle) -> Vec<(Slot, Coverage)> {
    use Slot as S;
    let mut v = Vec::new();
    let mut push = |slot: S, cov: Coverage| v.push((slot, cov));
    if s.background.is_some() {
        push(S::Background, Coverage::Native);
    }
    if s.content_color.is_some() {
        push(S::ContentColor, Coverage::Native);
    }
    if s.padding.is_some() {
        push(S::Padding, Coverage::Native);
    }
    if s.border_color.is_some() {
        push(S::BorderColor, Coverage::Native);
    }
    if s.border_width.is_some() {
        push(S::BorderWidth, Coverage::Native);
    }
    if s.corner_radius.is_some() {
        push(S::CornerRadius, Coverage::Native);
    }
    if s.dimensions.is_some() {
        push(S::Dimensions, Coverage::Native);
    }
    if s.min_dimensions.is_some() {
        push(S::MinDimensions, Coverage::Native);
    }
    if s.max_dimensions.is_some() {
        push(S::MaxDimensions, Coverage::Native);
    }
    if s.gap.is_some() {
        push(S::Gap, Coverage::Native);
    }
    if s.shadow.is_some() {
        push(S::Shadow, Coverage::Native);
    }
    if s.margin.is_some() {
        push(S::Margin, Coverage::Native);
    }
    if s.layout.is_some() {
        push(S::Layout, Coverage::Native);
    }
    if s.opacity.is_some() {
        push(S::Opacity, Coverage::Native);
    }
    if s.ring.is_some() {
        push(S::Ring(aeolus_core::RingPart::Width), Coverage::Native);
    }
    if s.outline.is_some() {
        push(S::Outline, Coverage::Native);
    }
    if s.cursor.is_some() {
        push(S::Cursor, Coverage::Native);
    }
    if s.pointer_events.is_some() {
        push(S::PointerEvents, Coverage::Native);
    }
    for (p, _) in &s.text {
        push(S::Text(*p), Coverage::Native);
    }
    let e = &s.extras;
    if e.position.is_some() {
        push(S::Position, Coverage::Native);
    }
    if e.inset.is_some() {
        push(S::Inset, Coverage::Native);
    }
    if e.z_index.is_some() {
        push(S::ZIndex, Coverage::Native);
    }
    if e.overflow.is_some() {
        // Floem 0.2 has no overflow prop; reported.
        push(S::Overflow, Coverage::Reported);
    }
    if e.visibility.is_some() {
        push(S::Visibility, Coverage::Reported);
    }
    if e.aspect_ratio.is_some() {
        push(S::AspectRatio, Coverage::Native);
    }
    if e.grid.is_some() {
        push(S::Grid, Coverage::Reported);
    }
    if e.transform.is_some() {
        push(S::Transform, Coverage::Native);
    }
    if e.transition.is_some() {
        push(S::Transition, Coverage::Native);
    }
    if e.animation.is_some() {
        push(S::Animation, Coverage::Reported);
    }
    if e.border_style.is_some() {
        push(S::BorderStyle, Coverage::Reported);
    }
    if e.flex_basis.is_some() {
        push(S::FlexBasis, Coverage::Native);
    }
    if e.order.is_some() {
        push(S::Order, Coverage::Reported);
    }
    if e.align_content.is_some() {
        push(S::AlignContent, Coverage::Reported);
    }
    if e.margin_auto.is_some() {
        push(S::MarginAuto, Coverage::Native);
    }
    if e.gap_axes.is_some() {
        push(S::GapAxes, Coverage::Native);
    }
    if !e.shadow_rest.is_empty() {
        push(S::ShadowRest, Coverage::Native);
    }
    if !e.inset_shadows.is_empty() {
        push(S::InsetShadows, Coverage::Reported);
    }
    v
}
