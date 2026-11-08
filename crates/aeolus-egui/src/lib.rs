//! # Aeolus egui backend — `StylePlan` → egui immediate-mode styles
//!
//! egui has no retained cascade: every frame the app asks *the current
//! situation* for the appearance. Two native surfaces exist, and this
//! backend lowers into both:
//!
//! 1. **[`egui::Frame`] surfaces** — panels, windows, groups and
//!    `Frame::show` scopes. A `ResolvedStyle` lowers into an
//!    [`EguiStyle`] whose [`EguiStyle::frame`] carries the solid fill
//!    (`background`), the border stroke (`border-*`), the per-corner
//!    radius (`rounded-*`), the CSS padding (`inner_margin`) and the CSS
//!    margin (`outer_margin`) — egui's own field docs name those CSS
//!    equivalents — plus the drop shadow.
//! 2. **The per-state widget cascade** — `style.visuals.widgets.
//!    {inactive,hovered,active,disabled}` ([`egui::style::WidgetVisuals`]).
//!    Hover/active/disabled layers of a plan lower onto those state slots
//!    via [`apply_widget_visuals`], which is how `egui::widgets::Button`
//!    picks its appearance: the native immediate-mode cascade, no
//!    per-frame bookkeeping in the app.
//!
//! The rest rides along in typed hints: typography (via [`egui::RichText`] —
//! `.strong()`, `.italics()`, `.underline()`, …), the cursor
//! (`Response::on_hover_cursor`), `item_spacing` (`gap-*`),
//! selectability (`select-none`), wrap mode (`whitespace-*`) and the
//! enable flag (`pointer-events-none`). The parity path for group/peer,
//! compound variants and container queries is the backend-agnostic
//! evaluator: [`EguiProbe`] + [`StyleBook::resolve_at`], re-run per frame
//! — exactly what immediate mode re-resolves anyway
//! ([`aeolus_core::ThemeSwap::Free`]).
//!
//! Whatever egui cannot express lands in a [`LoweringReport`] with a
//! per-slot reason — never a silent no-op.
//!
//! ```no_run
//! use aeolus_core::{Resolver, shadcn_default};
//! use aeolus_egui::StyleBook;
//!
//! # fn build(ctx: &egui::Context, ui: &mut egui::Ui) {
//! let theme = shadcn_default().unwrap();
//! let resolver = Resolver::new(&theme);
//! let book = StyleBook::tw(&resolver,
//!     "p-4 bg-primary text-primary-foreground rounded-md hover:bg-primary/90");
//!
//! // A panel body: base frame + the app's current dark mode.
//! let style = book.resolve_for_ctx(ctx);
//! style.frame.show(ui, |ui| { ui.label("Save changes"); });
//! # }
//! ```

use egui::style::WidgetVisuals;
use egui::{Color32, CornerRadius, CursorIcon, FontFamily, FontId, Frame, Margin, Stroke, Vec2};

use aeolus_core::color::Color;
use aeolus_core::eval::{Probe, Rel};
use aeolus_core::plan::{
    Gradient, ResolvedStyle, SizedValue, Slot, StylePlan, TextProperty, TextValue, VariantCondition,
};

/// Everything the lowering could not express in egui 0.37.
///
/// Aeolus never drops a class silently: conditions/properties without an
/// egui equivalent are reported here (and should be surfaced through
/// `tracing::warn!` by the host, matching `aeolus-core`'s philosophy).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LoweringReport {
    /// `(what, reason)` pairs, e.g. `("bg-linear-*", "Frame fills are
    /// solid Color32; gradients need an app-side mesh painter")`.
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

/// Convert our Oklab [`Color`] to egui's `Color32`.
///
/// `Color32::from_rgba_unmultiplied` takes straight-alpha sRGB bytes;
/// `Color::to_srgba` performs the OKLab → linear-sRGB → sRGB transfer
/// conversion (and clamps out-of-gamut values, which Tailwind's own
/// palette never produces).
pub fn to_egui_color(c: Color) -> Color32 {
    let [r, g, b, a] = c.to_srgba();
    Color32::from_rgba_unmultiplied(
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8,
        (a * 255.0).round() as u8,
    )
}

/// Scale the straight alpha of a `Color32` by `factor` (CSS `opacity`
/// baking). `Color32` stores bytes straight-alpha in egui 0.37's
/// `from_rgba_unmultiplied` convention, so scaling is a byte multiply.
fn scale_alpha32(c: Color32, factor: f32) -> Color32 {
    let [r, g, b, a] = c.to_array();
    Color32::from_rgba_unmultiplied(
        r,
        g,
        b,
        ((a as f32 / 255.0) * factor)
            .clamp(0.0, 1.0)
            .mul_add(255.0, 0.5) as u8,
    )
}

/// Scale the straight alpha of a `Stroke` color.
fn scale_stroke(mut s: Stroke, factor: f32) -> Stroke {
    s.color = scale_alpha32(s.color, factor);
    s
}

// ---------------------------------------------------------------------------
// The lowered per-situation style
// ---------------------------------------------------------------------------

/// Immediate-mode typography hints, applied through [`egui::RichText`] /
/// [`egui::widget_text::WidgetText`] at the call site.
///
/// egui has no cascade for text; [`egui::RichText`] builder methods are the
/// native surface (`RichText::strong` for ≥600 weight, `weak` for ≤300,
/// `italics`, `underline`, `strikethrough`, `color`, `size`, `family`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EguiText {
    /// `text-{size}` / `font-{family}` → the selected [`FontId`]
    /// (`font-sans` → Proportional, `font-mono` → Monospace,
    /// `font-serif` → the app's `serif` family name).
    pub font_id: Option<FontId>,
    /// `text-{color}`.
    pub color: Option<Color32>,
    /// `font-{weight}` 100..=900 (≥600 → [`egui::RichText::strong`], ≤300 →
    /// [`egui::RichText::weak`], otherwise default).
    pub weight: Option<u16>,
    /// `italic`.
    pub italics: Option<bool>,
    /// `underline` / `no-underline`.
    pub underline: Option<bool>,
    /// `line-through` / `no-underline`.
    pub strikethrough: Option<bool>,
}

impl EguiText {
    /// Overlay `other` on top of `self` (later layers win per slot).
    pub fn overlay(&mut self, other: &EguiText) {
        if other.font_id.is_some() {
            self.font_id = other.font_id.clone();
        }
        if other.color.is_some() {
            self.color = other.color;
        }
        if other.weight.is_some() {
            self.weight = other.weight;
        }
        if other.italics.is_some() {
            self.italics = other.italics;
        }
        if other.underline.is_some() {
            self.underline = other.underline;
        }
        if other.strikethrough.is_some() {
            self.strikethrough = other.strikethrough;
        }
    }
}

/// One situation's lowered style: the [`Frame`] payload plus the
/// immediate-mode hints.
#[derive(Clone, Debug, PartialEq)]
pub struct EguiStyle {
    /// The painted box: `inner_margin` = CSS padding, `outer_margin` =
    /// CSS margin, `fill` = `bg-*`, `stroke` = `border-*`, per-corner
    /// `corner_radius` = `rounded-*`, `shadow` = `shadow-*`.
    pub frame: Frame,
    presence: Presence,
    /// The style's own `opacity-*` factor (1.0 unless an opacity class
    /// set it). `lower_resolved` bakes it into this style's colors;
    /// `overlay` re-applies it to the accumulated style so a bare
    /// `disabled:opacity-50` layer still dims the base.
    opacity_factor: f32,
    /// Typography hints (see [`EguiText`]).
    pub text: EguiText,
    /// `cursor-*` → [`egui::Response::on_hover_cursor`].
    pub cursor: Option<CursorIcon>,
    /// `gap-*` → `Ui::spacing_mut().item_spacing` (egui's own suggestion
    /// for "adding space between widgets").
    pub item_spacing: Option<Vec2>,
    /// `select-none` / `select-text` → `Ui::style_mut().interaction.
    /// selectable_labels` (or `Label::selectable`).
    pub selectable: Option<bool>,
    /// `whitespace-nowrap` → [`egui::TextWrapMode::Extend`],
    /// `whitespace-normal` → Wrap, `text-ellipsis` → Truncate.
    pub wrap_mode: Option<egui::TextWrapMode>,
    /// `pointer-events-none` → `Some(false)` (add with
    /// `ui.add_enabled(false, …)` / `ui.disable()`).
    pub enabled: Option<bool>,
    /// `min-w-*`/`min-h-*` → `Ui::set_min_size`.
    pub min_size: Option<Vec2>,
    /// `w-*`/`h-*` in px → `Ui::add_sized` / `Ui::set_width(_height)`.
    pub width: Option<f32>,
    /// `h-*` in px.
    pub height: Option<f32>,
}

impl Default for EguiStyle {
    fn default() -> Self {
        Self {
            frame: Frame::default(),
            presence: Presence::default(),
            // A style with no opacity class paints fully opaque.
            opacity_factor: 1.0,
            text: EguiText::default(),
            cursor: None,
            item_spacing: None,
            selectable: None,
            wrap_mode: None,
            enabled: None,
            min_size: None,
            width: None,
            height: None,
        }
    }
}

impl EguiStyle {
    /// Overlay `other` on top of `self` (later layers win per slot),
    /// mirroring [`ResolvedStyle::overlay`]'s Some-replaces semantics.
    pub fn overlay(&mut self, other: &EguiStyle) {
        // CSS `opacity` composites the whole element: a layer that carries
        // an opacity factor dims everything accumulated so far, not just
        // its own colors (which `lower_resolved` already baked).
        if other.opacity_factor < 1.0 {
            let f = other.opacity_factor;
            let s = &mut self.frame;
            s.fill = scale_alpha32(s.fill, f);
            s.stroke = scale_stroke(s.stroke, f);
            s.shadow.color = scale_alpha32(s.shadow.color, f);
            if let Some(c) = self.text.color {
                self.text.color = Some(scale_alpha32(c, f));
            }
        }
        let f = &other.frame;
        let s = &mut self.frame;
        if other.presence.fill {
            s.fill = f.fill;
        }
        if other.presence.stroke {
            s.stroke = f.stroke;
        }
        if other.presence.corner_radius {
            s.corner_radius = f.corner_radius;
        }
        if other.presence.inner_margin {
            s.inner_margin = f.inner_margin;
        }
        if other.presence.outer_margin {
            s.outer_margin = f.outer_margin;
        }
        if other.presence.shadow {
            s.shadow = f.shadow;
        }
        self.presence.merge(&other.presence);
        self.text.overlay(&other.text);
        if other.cursor.is_some() {
            self.cursor = other.cursor;
        }
        if other.item_spacing.is_some() {
            self.item_spacing = other.item_spacing;
        }
        if other.selectable.is_some() {
            self.selectable = other.selectable;
        }
        if other.wrap_mode.is_some() {
            self.wrap_mode = other.wrap_mode;
        }
        if other.enabled.is_some() {
            self.enabled = other.enabled;
        }
        if other.min_size.is_some() {
            self.min_size = other.min_size;
        }
        if other.width.is_some() {
            self.width = other.width;
        }
        if other.height.is_some() {
            self.height = other.height;
        }
    }
}

/// Which [`Frame`] fields a lowered style explicitly set (the overlay
/// bookkeeping; `Frame` is a plain struct with sentinel defaults).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Presence {
    fill: bool,
    stroke: bool,
    corner_radius: bool,
    inner_margin: bool,
    outer_margin: bool,
    shadow: bool,
}

impl Presence {
    /// Union (a merged style sets whatever either side set).
    fn merge(&mut self, other: &Self) {
        self.fill |= other.fill;
        self.stroke |= other.stroke;
        self.corner_radius |= other.corner_radius;
        self.inner_margin |= other.inner_margin;
        self.outer_margin |= other.outer_margin;
        self.shadow |= other.shadow;
    }
}

// ---------------------------------------------------------------------------
// ResolvedStyle → EguiStyle
// ---------------------------------------------------------------------------

/// Lower one resolved style into an [`EguiStyle`].
///
/// Slots egui cannot express (gradients on `Frame` fills, ring, flex
/// layout, transforms, ...) are recorded as dropped — they are
/// capability-negotiated, never silently ignored.
pub fn lower_resolved(style: &ResolvedStyle, report: &mut LoweringReport) -> EguiStyle {
    let mut out = EguiStyle::default();

    // Opacity is not a per-widget slot in egui; CSS semantics (scale every
    // painted alpha) are baked at lowering time, documented below. The
    // factor is also remembered for `overlay` (a bare `disabled:opacity-50`
    // layer has no colors of its own to bake).
    let opacity = style.opacity.map(|o| o.0.clamp(0.0, 1.0)).unwrap_or(1.0);
    out.opacity_factor = opacity;

    if let Some(cc) = &style.content_color {
        out.text.color = Some(to_egui_color(cc.0));
    }
    if let Some(bg) = &style.background {
        out.frame.fill = to_egui_color(bg.0);
        out.presence.fill = true;
    }
    if let Some(gradient) = &style.background_gradient {
        match gradient {
            Gradient::None => {}
            _ => {
                if style.background.is_some() {
                    report.drop(
                        "background-color under gradient",
                        "CSS layers background-image over background-color; egui Frame \
fills are a single solid Color32 and the fill slot is taken",
                    );
                }
                report.drop(
                    "bg-linear-*/bg-radial/bg-conic*",
                    "egui Frame fills are solid Color32; gradients need an app-side \
mesh painter (Painter::add + RectShape UV trick) — reported, not faked",
                );
            }
        }
    }

    // Border: one (color, width) stroke; per-side utilities honestly
    // flatten (egui Frame carries a single Stroke).
    if let Some(bw) = &style.border_width {
        if !(bw.top == bw.right && bw.right == bw.bottom && bw.bottom == bw.left) {
            report.drop(
                "border-width:per-side",
                "egui Frame carries a single Stroke; using top edge",
            );
        }
        out.frame.stroke.width = bw.top;
        out.presence.stroke = true;
    }
    if let Some(bc) = &style.border_color {
        out.frame.stroke.color = to_egui_color(bc.0);
        out.presence.stroke = true;
    }
    if let Some(o) = &style.outline {
        // No border set → the outline takes the single stroke slot
        // natively (the Floem ring→outline precedent); otherwise report.
        if !out.presence.stroke {
            out.frame.stroke.width = o.width;
            if let Some(c) = o.color {
                out.frame.stroke.color = to_egui_color(c);
            }
            out.presence.stroke = true;
        } else {
            report.drop(
                "outline",
                "egui Frame carries a single Stroke and border-* already uses it",
            );
        }
    }
    if style.ring.is_some() {
        report.drop(
            "ring",
            "no ring primitive in egui; the single Stroke slot would collide with border-*",
        );
    }

    if let Some(cr) = &style.corner_radius {
        // Per-corner native: egui CornerRadius carries all four corners
        // (u8 points, saturating).
        out.frame.corner_radius = CornerRadius {
            nw: cr.top_left.clamp(0.0, 255.0) as u8,
            ne: cr.top_right.clamp(0.0, 255.0) as u8,
            sw: cr.bottom_left.clamp(0.0, 255.0) as u8,
            se: cr.bottom_right.clamp(0.0, 255.0) as u8,
        };
        out.presence.corner_radius = true;
    }

    // CSS padding → inner_margin, CSS margin → outer_margin (egui's own
    // field docs name these equivalents). Margin i8: clamp to ±127.
    if let Some(p) = &style.padding {
        out.frame.inner_margin = Margin {
            left: clamp_i8(p.left),
            right: clamp_i8(p.right),
            top: clamp_i8(p.top),
            bottom: clamp_i8(p.bottom),
        };
        out.presence.inner_margin = true;
    }
    if let Some(m) = &style.margin {
        out.frame.outer_margin = Margin {
            left: clamp_i8(m.left),
            right: clamp_i8(m.right),
            top: clamp_i8(m.top),
            bottom: clamp_i8(m.bottom),
        };
        out.presence.outer_margin = true;
    }
    if style.extras.margin_auto.is_some() {
        report.drop("margin-auto", "no auto margins in egui layout");
    }

    if let Some(s) = &style.shadow {
        if s.inset {
            report.drop("shadow-inner", "egui Shadow has no inset mode");
        }
        out.frame.shadow = egui::Shadow {
            offset: [clamp_i8(s.offset_x), clamp_i8(s.offset_y)],
            blur: s.blur.clamp(0.0, 255.0) as u8,
            spread: s.spread.clamp(0.0, 255.0) as u8,
            color: to_egui_color(s.color),
        };
        out.presence.shadow = true;
    }
    if !style.extras.shadow_rest.is_empty() {
        report.drop(
            "shadow layers 2+",
            "egui paints a single Shadow; extra preset layers reported",
        );
    }
    if !style.extras.inset_shadows.is_empty() {
        report.drop("inset-shadow-*", "egui Shadow has no inset mode");
    }

    if style.layout.is_some() {
        report.drop(
            "layout",
            "immediate-mode layout is code structure (Ui::horizontal/vertical), \
not a per-element style",
        );
    }
    if let Some(d) = &style.min_dimensions {
        out.min_size = Some(Vec2::new(
            d.width.map(sized_px).unwrap_or(0.0),
            d.height.map(sized_px).unwrap_or(0.0),
        ));
    }
    if style.max_dimensions.is_some() {
        report.drop(
            "max-w-/max-h-",
            "no max-size primitive in egui (shrink the Ui allocation instead)",
        );
    }
    if let Some(dims) = &style.dimensions {
        out.width = dims.width.map(sized_px);
        out.height = dims.height.map(sized_px);
    }

    if let Some(g) = &style.gap {
        out.item_spacing = Some(Vec2::splat(g.0));
    }
    if let Some(axes) = &style.extras.gap_axes {
        out.item_spacing = Some(Vec2::new(axes.col.unwrap_or(0.0), axes.row.unwrap_or(0.0)));
    }

    if let Some(cursor) = &style.cursor {
        out.cursor = Some(to_cursor_icon(*cursor));
    }
    if let Some(pe) = &style.pointer_events {
        out.enabled = Some(matches!(pe, aeolus_core::plan::PointerEvents::Auto));
    }

    lower_text(style, &mut out, report);
    report_extras(style, report);

    // Bake CSS opacity into every color the style paints (egui has no
    // per-widget opacity; alpha scaling is the CSS-equivalent composite).
    if opacity < 1.0 {
        out.frame.fill = scale_alpha32(out.frame.fill, opacity);
        out.frame.stroke = scale_stroke(out.frame.stroke, opacity);
        out.frame.shadow.color = scale_alpha32(out.frame.shadow.color, opacity);
        if let Some(c) = out.text.color {
            out.text.color = Some(scale_alpha32(c, opacity));
        }
    }

    out
}

/// Clamp an f32 px length into egui's `i8` margin/offset domain.
fn clamp_i8(v: f32) -> i8 {
    v.round().clamp(-127.0, 127.0) as i8
}

/// Px value of a `SizedValue` when one exists (absolute or scale);
/// symbolic sizes return 0 and stay honest in the coverage ledger.
fn sized_px(v: SizedValue) -> f32 {
    match v {
        SizedValue::Px(px) => px,
        SizedValue::Scale(n) => n * 4.0,
        _ => 0.0,
    }
}

/// The full v4 cursor keyword set → egui `CursorIcon`.
fn to_cursor_icon(c: aeolus_core::plan::Cursor) -> CursorIcon {
    use aeolus_core::plan::Cursor as C;
    match c {
        C::Auto | C::Default => CursorIcon::Default,
        C::None => CursorIcon::None,
        C::Pointer => CursorIcon::PointingHand,
        C::Wait => CursorIcon::Wait,
        C::Progress => CursorIcon::Progress,
        C::Text => CursorIcon::Text,
        C::Move => CursorIcon::Move,
        C::Help => CursorIcon::Help,
        C::NotAllowed => CursorIcon::NotAllowed,
        C::ContextMenu => CursorIcon::ContextMenu,
        C::Cell => CursorIcon::Cell,
        C::Crosshair => CursorIcon::Crosshair,
        C::VerticalText => CursorIcon::VerticalText,
        C::Alias => CursorIcon::Alias,
        C::Copy => CursorIcon::Copy,
        C::NoDrop => CursorIcon::NoDrop,
        C::Grab => CursorIcon::Grab,
        C::Grabbing => CursorIcon::Grabbing,
        C::AllScroll => CursorIcon::AllScroll,
        C::ColResize => CursorIcon::ResizeColumn,
        C::RowResize => CursorIcon::ResizeRow,
        C::NResize | C::SResize => CursorIcon::ResizeVertical,
        C::EResize | C::WResize => CursorIcon::ResizeHorizontal,
        C::NeResize | C::SwResize => CursorIcon::ResizeNeSw,
        C::NwResize | C::SeResize => CursorIcon::ResizeNwSe,
        C::EwResize => CursorIcon::ResizeHorizontal,
        C::NsResize => CursorIcon::ResizeVertical,
        C::NeswResize => CursorIcon::ResizeNeSw,
        C::NwseResize => CursorIcon::ResizeNwSe,
        C::ZoomIn => CursorIcon::ZoomIn,
        C::ZoomOut => CursorIcon::ZoomOut,
    }
}

/// Typography mapping: the slots with a native RichText/Ui surface lower
/// into [`EguiText`] and the wrap/select hints, the rest report.
fn lower_text(style: &ResolvedStyle, out: &mut EguiStyle, report: &mut LoweringReport) {
    for (prop, value) in &style.text {
        match (prop, value) {
            (TextProperty::Color, TextValue::Color(c)) => {
                out.text.color = Some(to_egui_color(*c));
            }
            (TextProperty::FontSize, TextValue::Px(px)) => {
                let id = out
                    .text
                    .font_id
                    .take()
                    .unwrap_or_else(|| FontId::new(*px, FontFamily::Proportional));
                out.text.font_id = Some(FontId::new(*px, id.family));
            }
            (TextProperty::FontFamily, TextValue::Keyword(k)) => {
                let family = match *k {
                    "sans" => FontFamily::Proportional,
                    "mono" => FontFamily::Monospace,
                    "serif" => FontFamily::Name("serif".into()),
                    other => {
                        report.drop(
                            format!("font-{other}"),
                            "custom families need the app's FontDefinitions; \
font-sans/mono/serif map natively",
                        );
                        continue;
                    }
                };
                let id = out
                    .text
                    .font_id
                    .clone()
                    .unwrap_or_else(|| FontId::new(14.0, family.clone()));
                out.text.font_id = Some(FontId::new(id.size, family));
            }
            (TextProperty::FontWeight, TextValue::Weight(w)) => {
                out.text.weight = Some(*w);
            }
            (TextProperty::FontStyle, TextValue::Keyword(k)) => match *k {
                "italic" => out.text.italics = Some(true),
                "normal" => out.text.italics = Some(false),
                other => report.drop(
                    format!("font-style:{other}"),
                    "only italic/normal exist in egui's RichText",
                ),
            },
            (TextProperty::TextDecoration, TextValue::Keyword(k)) => {
                match *k {
                    "underline" => out.text.underline = Some(true),
                    "line-through" => out.text.strikethrough = Some(true),
                    "none" => {
                        // `no-underline` clears both natively.
                        out.text.underline = Some(false);
                        out.text.strikethrough = Some(false);
                    }
                    other => report.drop(
                        format!("text-decoration:{other}"),
                        "overline has no RichText method",
                    ),
                }
            }
            (TextProperty::Whitespace, TextValue::Keyword(k)) => match *k {
                "normal" => out.wrap_mode = Some(egui::TextWrapMode::Wrap),
                "nowrap" => out.wrap_mode = Some(egui::TextWrapMode::Extend),
                other => report.drop(
                    format!("whitespace-{other}"),
                    "egui wrap modes are Wrap/Extend/Truncate; pre/newline reported",
                ),
            },
            (TextProperty::TextOverflow, TextValue::Keyword(k)) if *k == "ellipsis" => {
                out.wrap_mode = Some(egui::TextWrapMode::Truncate);
            }
            (TextProperty::Select, TextValue::Keyword(k)) => match *k {
                "select-none" => out.selectable = Some(false),
                "select-text" => out.selectable = Some(true),
                other => report.drop(
                    other.to_string(),
                    "only select-none/select-text map to egui selectability",
                ),
            },
            (prop, _) => report.drop(
                format!("typography:{prop:?}"),
                "no egui 0.37 RichText/layout equivalent for this text property \
(line-height, letter-spacing, alignment, transform, clamp, variant-numeric)",
            ),
        }
    }
}

/// Report the v0.2 extras egui has no equivalent for, per family.
fn report_extras(style: &ResolvedStyle, report: &mut LoweringReport) {
    let e = &style.extras;
    if e.position.is_some() {
        report.drop("position", "no absolute positioning in egui layout");
    }
    if e.inset.is_some() {
        report.drop("inset-*", "no offset positioning in egui layout");
    }
    if e.z_index.is_some() {
        report.drop("z-index", "z-order is the egui LayerId order, not a style");
    }
    if e.overflow.is_some() {
        report.drop(
            "overflow",
            "clipping is ScrollArea/clip_rect work, not a style",
        );
    }
    if e.visibility.is_some() {
        report.drop(
            "visible/invisible",
            "skip adding the widget instead (immediate mode)",
        );
    }
    if e.aspect_ratio.is_some() {
        report.drop("aspect-*", "no aspect-ratio primitive in egui layout");
    }
    if e.grid.is_some() {
        report.drop("grid-*", "egui::Grid takes column counts as code structure");
    }
    if e.transform.is_some() {
        report.drop("transform", "no per-widget transform in egui painting");
    }
    if e.transition.is_some() {
        report.drop(
            "transition-*",
            "egui animates via Context::animate_* driven by app code, not style rules",
        );
    }
    if e.animation.is_some() {
        report.drop(
            "animate-*",
            "egui animations are per-value (animate_value_with_time)",
        );
    }
    if let Some(bs) = &e.border_style {
        match bs {
            aeolus_core::plan::BorderStyle::Solid => {}
            aeolus_core::plan::BorderStyle::None | aeolus_core::plan::BorderStyle::Hidden => {
                if style.border_width.is_none() {
                    report.drop(
                        "border-none/hidden",
                        "border-style:none lowers to width 0; set it alongside a width class",
                    );
                }
            }
            aeolus_core::plan::BorderStyle::Dashed
            | aeolus_core::plan::BorderStyle::Dotted
            | aeolus_core::plan::BorderStyle::Double => {
                report.drop("border-style", "egui strokes are solid");
            }
        }
    }
    if e.flex_basis.is_some() {
        report.drop("basis-*", "flex sizing is code structure in egui");
    }
    if e.order.is_some() {
        report.drop("order-*", "immediate-mode children paint in call order");
    }
    if e.align_content.is_some() {
        report.drop("content-*", "align-content has no egui equivalent");
    }
}

// ---------------------------------------------------------------------------
// The widget-visuals cascade: base/hover/active/disabled → WidgetVisuals
// ---------------------------------------------------------------------------

/// Map a lowered style onto one state slot of `style.visuals.widgets` —
/// the native egui cascade for interactive widgets (`Button` and friends
/// pick their appearance from these).
///
/// `bg_fill`/`weak_bg_fill` take the background, `bg_stroke` the border,
/// `corner_radius` the radius, and `fg_stroke` carries the text color at
/// egui's standard 1.0 width (the widget's text picks the color up).
pub fn apply_widget_visuals(target: &mut WidgetVisuals, s: &EguiStyle) {
    if s.frame.fill != Color32::TRANSPARENT {
        target.bg_fill = s.frame.fill;
        target.weak_bg_fill = s.frame.fill;
    }
    if s.frame.stroke != Stroke::NONE {
        target.bg_stroke = s.frame.stroke;
    }
    if s.frame.corner_radius != CornerRadius::ZERO {
        target.corner_radius = s.frame.corner_radius;
    }
    if let Some(c) = s.text.color {
        target.fg_stroke = Stroke::new(1.0, c);
    }
}

/// Lower the book's native cascade onto `ui.visuals_mut().widgets` —
/// base → `inactive`, the hover layer → `hovered`, the active layer →
/// `active` (each condition slot starts from the base, CSS-style, then the
/// condition's own slots replace).
///
/// Call inside a `ui.scope(...)` wrapping an [`egui::widgets::Button`] (or
/// any stateful widget): egui then picks the right slot per frame by
/// itself. Disabled/focus conditions are probed per frame via
/// [`StyleBook::resolve_at`] (egui 0.37 has no disabled WidgetVisuals
/// slot).
pub fn apply_widget_visuals_cascade(ui: &mut egui::Ui, book: &StyleBook) {
    let v = ui.visuals_mut();
    apply_widget_visuals(&mut v.widgets.inactive, &book.base);
    for (condition, layer) in &book.layers {
        let mut merged = book.base.clone();
        merged.overlay(layer);
        match condition {
            VariantCondition::Hover => apply_widget_visuals(&mut v.widgets.hovered, &merged),
            VariantCondition::Active => apply_widget_visuals(&mut v.widgets.active, &merged),
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// The style book: plan → per-state resolution
// ---------------------------------------------------------------------------

/// The lowered [`StylePlan`]: base + rank-sorted conditional layers + dark,
/// resolved per state from the element's current situation.
///
/// Construct once (or cache on the app state) and call
/// [`StyleBook::resolve_for`] / [`StyleBook::resolve_for_ctx`] per frame —
/// egui re-resolves everything anyway, which is the
/// [`aeolus_core::ThemeSwap::Free`] contract.
#[derive(Clone, Debug, PartialEq)]
pub struct StyleBook {
    /// Unconditional properties.
    pub base: EguiStyle,
    /// Conditional layers, ascending [`VariantCondition::rank`] order.
    pub layers: Vec<(VariantCondition, EguiStyle)>,
    /// `dark:` override (applied when the egui theme is dark).
    pub dark: Option<EguiStyle>,
    /// The original plan, kept for the [`StyleBook::resolve_at`] parity
    /// path (group/peer/rules/compound variants evaluated by the core
    /// evaluator).
    plan: StylePlan,
}

impl StyleBook {
    /// Lower a full [`StylePlan`].
    ///
    /// Conditions egui's per-state model expresses natively
    /// (hover/active/disabled/focus) stay in `layers`; group/peer, compound
    /// rules, responsive and container conditions are reported as emulated
    /// (they remain available through [`StyleBook::resolve_at`]).
    pub fn new(plan: &StylePlan) -> (Self, LoweringReport) {
        let mut report = LoweringReport::default();

        for (group_id, _) in &plan.group {
            report.drop(
                format!("group:{group_id}"),
                "answered by resolve_at's probe path, not by Frame state",
            );
        }
        for (peer_id, _) in &plan.peer {
            report.drop(
                format!("peer:{peer_id}"),
                "answered by resolve_at's probe path, not by Frame state",
            );
        }
        for rule in &plan.rules {
            report.drop(
                format!("rules:{}", rule.when.rank().0),
                "compound/structural conditions evaluated by resolve_at's probe path",
            );
        }
        for bp in plan.responsive.keys() {
            report.drop(
                format!("responsive:{bp:?}"),
                "viewport width is knowable per frame — resolve_at evaluates it \
via EguiProbe::from_ctx",
            );
        }
        for bp in plan.container.keys() {
            report.drop(
                format!("container:{bp:?}"),
                "container width is knowable per frame — resolve_at evaluates it \
via EguiProbe::container_width",
            );
        }

        let base = lower_resolved(&plan.base, &mut report);
        let mut sorted: Vec<&aeolus_core::plan::ConditionalLayer> = plan.layers.iter().collect();
        sorted.sort_by_key(|l| l.condition.rank());
        let mut layers = Vec::with_capacity(sorted.len());
        for layer in sorted {
            // egui's widget-visual cascade has inactive/hovered/active only
            // (0.37 deliberately styles disabled widgets like enabled ones),
            // so disabled/focus/visited conditions resolve through
            // resolve_at's per-frame probe path instead.
            let native = matches!(
                layer.condition,
                VariantCondition::Hover | VariantCondition::Active
            );
            if !native {
                report.drop(
                    format!("{:?}:", layer.condition),
                    "no disabled/focused WidgetVisuals slot in egui 0.37 — resolved \
per frame by resolve_at (EguiProbe::from_response)",
                );
                continue;
            }
            layers.push((layer.condition, lower_resolved(&layer.style, &mut report)));
        }
        let dark = plan.dark.as_ref().map(|d| lower_resolved(d, &mut report));

        (
            Self {
                base,
                layers,
                dark,
                plan: plan.clone(),
            },
            report,
        )
    }

    /// Parse + lower in one step: the classes go through `resolver` and the
    /// resulting plan lowers into the book. Unparseable class strings panic
    /// in debug builds and degrade to the empty style in release, matching
    /// the honest-failure contract of the other backends.
    pub fn tw(resolver: &aeolus_core::Resolver, classes: &str) -> Self {
        match resolver.resolve(classes) {
            Ok(resolution) => {
                let (book, report) = Self::new(&resolution.plan);
                if !report.is_empty() {
                    tracing::warn!("aeolus-egui: dropped {:?}", report.dropped);
                }
                book
            }
            Err(err) => {
                if cfg!(debug_assertions) {
                    panic!("aeolus: failed to parse class string {classes:?}: {err:?}");
                }
                tracing::warn!("aeolus: dropping unparseable class string {classes:?}: {err:?}");
                let (book, _) = Self::new(&StylePlan::default());
                book
            }
        }
    }

    /// Merge base + dark for one theme without state conditions.
    fn merged_for_theme(&self, dark: bool) -> EguiStyle {
        let mut style = self.base.clone();
        if dark && let Some(d) = &self.dark {
            style.overlay(d);
        }
        style
    }

    /// Resolve for a widget's current state — hover/active/focus/disabled
    /// straight from the [`egui::Response`], dark from the context theme.
    ///
    /// Conditions apply in ascending [`VariantCondition::rank`] order, so a
    /// focused+hovered widget resolves the FOCUS layer over hover (Tailwind
    /// v4's cascade intent, same as the Masonry/Floem backends).
    pub fn resolve_for(&self, response: &egui::Response) -> EguiStyle {
        let probe = EguiProbe::from_response(response);
        let mut style = self.merged_for_theme(probe.dark);
        for (condition, layer) in &self.layers {
            let fires = match condition {
                VariantCondition::Hover => probe.hovered,
                VariantCondition::Active => probe.active,
                VariantCondition::Disabled => probe.disabled,
                VariantCondition::Focus | VariantCondition::FocusVisible => probe.focused,
                VariantCondition::FocusWithin | VariantCondition::Visited => false,
            };
            if fires {
                style.overlay(layer);
            }
        }
        style
    }

    /// Resolve for the context as a whole (base + dark) — panels, window
    /// bodies and everything without widget state.
    pub fn resolve_for_ctx(&self, ctx: &egui::Context) -> EguiStyle {
        self.merged_for_theme(ctx.theme() == egui::Theme::Dark)
    }

    /// The FULL parity path (plan §4.2): flatten the original plan for the
    /// situation the probe describes — group/peer, data/aria, compound
    /// variants, responsive and container conditions included — and lower
    /// the result. Call it per frame; immediate mode re-resolves anyway.
    pub fn resolve_at(&self, probe: &impl Probe) -> EguiStyle {
        let resolved = self.plan.resolve(probe);
        let mut report = LoweringReport::default();
        lower_resolved(&resolved, &mut report)
    }
}

// ---------------------------------------------------------------------------
// The parity probe (plan §4.2)
// ---------------------------------------------------------------------------

/// The element's current situation as egui sees it — construct from a
/// [`egui::Response`] (via [`EguiProbe::from_response`]) or from the app's
/// own state for group/peer/container knowledge.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EguiProbe {
    /// `hover:` / `group-hover:`.
    pub hovered: bool,
    /// `focus:` / `focus-visible:`.
    pub focused: bool,
    /// `active:`.
    pub active: bool,
    /// `disabled:`.
    pub disabled: bool,
    /// `dark:` (from `ctx.theme()`).
    pub dark: bool,
    /// Current viewport width (window), when known.
    pub width: Option<f32>,
    /// Current container width, when this element is inside a container
    /// the app tracks.
    pub container_width: Option<f32>,
}

impl EguiProbe {
    /// The probe for one widget's response: state flags from the
    /// response, dark from the context theme, viewport width from the
    /// screen rect.
    pub fn from_response(response: &egui::Response) -> Self {
        Self {
            hovered: response.hovered(),
            focused: response.has_focus(),
            active: response.is_pointer_button_down_on(),
            disabled: !response.enabled(),
            dark: response.ctx.theme() == egui::Theme::Dark,
            width: Some(response.ctx.viewport_rect().width()),
            container_width: None,
        }
    }

    /// The probe for the context as a whole (panels, window bodies).
    pub fn from_ctx(ctx: &egui::Context) -> Self {
        Self {
            dark: ctx.theme() == egui::Theme::Dark,
            width: Some(ctx.viewport_rect().width()),
            container_width: None,
            ..Self::default()
        }
    }
}

impl Probe for EguiProbe {
    fn test(&self, t: &aeolus_core::Test) -> bool {
        use aeolus_core::{State, Test};
        match t {
            Test::State(s) => match s {
                State::Hover => self.hovered,
                State::Focus | State::FocusVisible => self.focused,
                State::Active => self.active,
                State::Disabled => self.disabled,
                _ => false,
            },
            _ => false,
        }
    }

    fn related(&self, _rel: Rel, _label: Option<&str>, _t: &aeolus_core::Test) -> bool {
        // Group/peer state lives with the app; extend this probe at the
        // call site (the default probe answers "no" — the same semantics
        // CSS has when a selector does not match).
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

// ---------------------------------------------------------------------------
// Call-site sugar
// ---------------------------------------------------------------------------

/// Apply the book to a response after painting: the cursor hint is the
/// only response-level surface (`hover:bg-*` colors already resolve inside
/// the widget visuals / per-frame re-resolution).
pub trait TwResponseExt {
    /// Resolve the book for this response's state and apply the cursor.
    fn tw(self, book: &StyleBook) -> Self;
}

impl TwResponseExt for egui::Response {
    fn tw(self, book: &StyleBook) -> Self {
        let style = book.resolve_for(&self);
        if let Some(cursor) = style.cursor {
            return self.on_hover_cursor(cursor);
        }
        self
    }
}

/// Style a [`egui::RichText`] from the lowered typography hints
/// (size, family, color, strong/weak, italics, underline, strikethrough).
pub fn style_text(mut rich: egui::RichText, text: &EguiText) -> egui::RichText {
    if let Some(id) = &text.font_id {
        rich = rich.size(id.size).family(id.family.clone());
    }
    if let Some(c) = text.color {
        rich = rich.color(c);
    }
    if let Some(w) = text.weight {
        if w >= 600 {
            rich = rich.strong();
        } else if w <= 300 {
            rich = rich.weak();
        }
    }
    if text.italics == Some(true) {
        rich = rich.italics();
    }
    if text.underline == Some(true) {
        rich = rich.underline();
    }
    if text.strikethrough == Some(true) {
        rich = rich.strikethrough();
    }
    rich
}

/// Apply the non-Frame hints to a scoped `Ui` before the content runs
/// (item spacing, selectability, wrap mode, enable flag, min/px sizes,
/// cursor-on-hover of the scope's response).
pub trait TwUiExt {
    /// Run `add_contents` inside a scope whose `Ui` carries the style's
    /// hints; the frame itself is NOT painted (compose with
    /// `style.frame.show(ui, …)` for that).
    fn tw_scope<R>(
        &mut self,
        style: &EguiStyle,
        add_contents: impl FnOnce(&mut egui::Ui) -> R,
    ) -> egui::InnerResponse<R>;
}

impl TwUiExt for egui::Ui {
    fn tw_scope<R>(
        &mut self,
        style: &EguiStyle,
        add_contents: impl FnOnce(&mut egui::Ui) -> R,
    ) -> egui::InnerResponse<R> {
        self.scope(|ui| {
            if let Some(spacing) = style.item_spacing {
                ui.spacing_mut().item_spacing = spacing;
            }
            if let Some(selectable) = style.selectable {
                ui.style_mut().interaction.selectable_labels = selectable;
            }
            if style.enabled == Some(false) {
                ui.disable();
            }
            if let Some(min) = style.min_size {
                ui.set_min_size(min);
            }
            if let Some(w) = style.width {
                ui.set_width(w);
            }
            if let Some(h) = style.height {
                ui.set_height(h);
            }
            if let Some(wrap) = style.wrap_mode {
                ui.style_mut().wrap_mode = Some(wrap);
            }
            add_contents(ui)
        })
    }
}

// ---------------------------------------------------------------------------
// Parity accounting (plan §4.1): every slot is native, emulated, or reported.
// ---------------------------------------------------------------------------

/// How a slot is covered on this backend.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Coverage {
    /// Mapped to a native egui surface (Frame, RichText, spacing, cursor).
    Native,
    /// Emulated inside the lowering (opacity alpha baking) or by the
    /// per-frame probe path.
    Emulated,
    /// Recorded in [`LoweringReport`] with a reason.
    Reported,
}

/// Walk one style and classify every set slot (the parity ledger).
pub fn slot_coverage(s: &ResolvedStyle) -> Vec<(Slot, Coverage)> {
    use Slot as S;
    use aeolus_core::plan::GradientPart;
    let mut v = Vec::new();
    if s.background.is_some() {
        v.push((S::Background, Coverage::Native));
    }
    if s.content_color.is_some() {
        v.push((S::ContentColor, Coverage::Native));
    }
    if s.background_gradient.is_some() {
        v.push((
            S::Gradient(GradientPart::Shape),
            if matches!(s.background_gradient, Some(Gradient::None)) {
                Coverage::Native
            } else {
                Coverage::Reported
            },
        ));
    }
    if s.padding.is_some() {
        v.push((S::Padding, Coverage::Native));
    }
    if s.border_color.is_some() {
        v.push((S::BorderColor, Coverage::Native));
    }
    if s.border_width.is_some() {
        v.push((S::BorderWidth, Coverage::Native));
    }
    if s.corner_radius.is_some() {
        v.push((S::CornerRadius, Coverage::Native));
    }
    if s.dimensions.is_some() {
        v.push((S::Dimensions, Coverage::Native));
    }
    if s.min_dimensions.is_some() {
        v.push((S::MinDimensions, Coverage::Native));
    }
    if s.max_dimensions.is_some() {
        v.push((S::MaxDimensions, Coverage::Reported));
    }
    if s.gap.is_some() {
        v.push((S::Gap, Coverage::Native));
    }
    if s.shadow.is_some() {
        v.push((S::Shadow, Coverage::Native));
    }
    if s.margin.is_some() {
        v.push((S::Margin, Coverage::Native));
    }
    if s.layout.is_some() {
        v.push((S::Layout, Coverage::Reported));
    }
    if s.opacity.is_some() {
        v.push((S::Opacity, Coverage::Emulated));
    }
    if s.ring.is_some() {
        v.push((S::Ring(aeolus_core::RingPart::Width), Coverage::Reported));
    }
    if s.outline.is_some() {
        v.push((S::Outline, Coverage::Native));
    }
    if s.cursor.is_some() {
        v.push((S::Cursor, Coverage::Native));
    }
    if s.pointer_events.is_some() {
        v.push((S::PointerEvents, Coverage::Native));
    }
    for (p, _) in &s.text {
        let cov = match p {
            TextProperty::Color
            | TextProperty::FontSize
            | TextProperty::FontWeight
            | TextProperty::FontFamily
            | TextProperty::FontStyle
            | TextProperty::TextDecoration
            | TextProperty::Whitespace
            | TextProperty::TextOverflow
            | TextProperty::Select => Coverage::Native,
            _ => Coverage::Reported,
        };
        v.push((S::Text(*p), cov));
    }
    let e = &s.extras;
    if e.position.is_some() {
        v.push((S::Position, Coverage::Reported));
    }
    if e.inset.is_some() {
        v.push((S::Inset, Coverage::Reported));
    }
    if e.z_index.is_some() {
        v.push((S::ZIndex, Coverage::Reported));
    }
    if e.overflow.is_some() {
        v.push((S::Overflow, Coverage::Reported));
    }
    if e.visibility.is_some() {
        v.push((S::Visibility, Coverage::Reported));
    }
    if e.aspect_ratio.is_some() {
        v.push((S::AspectRatio, Coverage::Reported));
    }
    if e.grid.is_some() {
        v.push((S::Grid, Coverage::Reported));
    }
    if e.transform.is_some() {
        v.push((S::Transform, Coverage::Reported));
    }
    if e.transition.is_some() {
        v.push((S::Transition, Coverage::Reported));
    }
    if e.animation.is_some() {
        v.push((S::Animation, Coverage::Reported));
    }
    if e.border_style.is_some() {
        v.push((S::BorderStyle, Coverage::Native));
    }
    if e.flex_basis.is_some() {
        v.push((S::FlexBasis, Coverage::Reported));
    }
    if e.order.is_some() {
        v.push((S::Order, Coverage::Reported));
    }
    if e.align_content.is_some() {
        v.push((S::AlignContent, Coverage::Reported));
    }
    if e.margin_auto.is_some() {
        v.push((S::MarginAuto, Coverage::Reported));
    }
    if e.gap_axes.is_some() {
        v.push((S::GapAxes, Coverage::Native));
    }
    if !e.shadow_rest.is_empty() {
        v.push((S::ShadowRest, Coverage::Reported));
    }
    if !e.inset_shadows.is_empty() {
        v.push((S::InsetShadows, Coverage::Reported));
    }
    v
}
