//! # Aeolus Iced backend — `StylePlan` → Iced state-closure styles
//!
//! Iced's cascade is the [`container::Catalog`] model: every widget asks a
//! `Box<dyn Fn(&Theme) -> Style>` (a `StyleFn`) for its appearance, once per
//! render, for the widget's current [`button::Status`]. A `StylePlan`
//! therefore lowers into a **style book** — base + per-condition layers +
//! dark — that is *resolved per state at closure time* (the
//! [`aeolus_core::CascadeModel::StateClosures`] contract), with zero widget
//! rebuilds on hover/active/disabled flips.
//!
//! Three consumption paths, mirroring the Xilem/Floem backends:
//!
//! 1. **`container_style` / `button_style` — the native-cascade path.**
//!    [`StyleBook::container_style`] resolves base + `dark:` from the
//!    `&Theme` the catalog hands over; [`StyleBook::button_style`] adds the
//!    status layer (Hovered → `hover:`, Pressed → `active:`, Disabled →
//!    `disabled:`, Focused → `focus-visible:`). Plug either into
//!    `container(...).style(...)` / `button(...).style(...)` directly.
//! 2. **[`IcedProbe`] + [`StyleBook::resolve_at`] — the parity path.**
//!    Everything the native status model cannot express (`group-hover:`,
//!    `data-[state=open]:`, `dark:hover:`, container queries) is answered by
//!    the backend-agnostic evaluator: the app (or a wrapper) keeps the
//!    element's situation in a probe and re-resolves per frame, exactly the
//!    Floem reactive contract.
//! 3. **Widget-argument hints.** Iced carries part of its styling in
//!    *constructor arguments*, not in the cascade: `.padding(...)`,
//!    `.width(...)`, `.spacing(...)`, `.font(...)`, `.size(...)`. Those slots
//!    lower into [`IcedStyle::text`] ([`TextHints`]) and
//!    [`IcedStyle::widget`] ([`WidgetHints`]) so nothing is silently dropped
//!    — the caller applies them at widget construction, the honest
//!    no-silent-failures contract.
//!
//! Whatever Iced 0.14 cannot express at all lands in a
//! [`LoweringReport`] with a per-slot reason — never a silent no-op.
//!
//! ```no_run
//! use std::sync::Arc;
//! use aeolus_core::{Resolver, shadcn_default};
//! use aeolus_iced::StyleBook;
//! use iced_widget::container;
//!
//! # fn build() {
//! let theme = shadcn_default().unwrap();
//! let resolver = Resolver::new(&theme);
//! let book = Arc::new(StyleBook::tw(&resolver,
//!     "p-4 bg-primary text-primary-foreground rounded-md hover:bg-primary/90"));
//!
//! // container(...).style(move |t| book.container_style(t)) — the Catalog
//! // closure; hover flips resolve inside button_status-based widgets.
//! # }
//! ```

use std::sync::Arc;

use iced_core::border::Radius;
use iced_core::font::{Family as IcedFamily, Style as IcedFontStyle, Weight as IcedWeight};
use iced_core::text::{
    Alignment as IcedTextAlign, LineHeight as IcedLineHeight, Wrapping as IcedWrapping,
};
use iced_core::{Background, Color, Gradient, Length, Pixels, Radians, Shadow, Vector};
use iced_widget::button;
use iced_widget::container;

use aeolus_core::color::Color as TwColor;
use aeolus_core::eval::{Probe, Rel};
use aeolus_core::plan::{
    CornerSet, EdgeSet, Gradient as TwGradient, GradientStops, ResolvedStyle, SizedValue, Slot,
    StylePlan, TextProperty, TextValue, VariantCondition,
};

/// Everything the lowering could not express in Iced 0.14.
///
/// Aeolus never drops a class silently: conditions/properties without an
/// Iced equivalent are reported here (and should be surfaced through
/// `tracing::warn!` by the host, matching `aeolus-core`'s philosophy).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LoweringReport {
    /// `(what, reason)` pairs, e.g. `("ring", "no ring primitive — single
    /// border slot")`.
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

/// Convert our Oklab [`Color`] to Iced's sRGB float `Color`.
///
/// Iced colors are straight-alpha sRGB floats; `Color::to_srgba` performs
/// the OKLab → linear-sRGB → sRGB transfer conversion (and clamps
/// out-of-gamut values, which Tailwind's own palette never produces).
pub fn to_iced_color(c: TwColor) -> Color {
    let [r, g, b, a] = c.to_srgba();
    Color { r, g, b, a }
}

/// Scale the alpha of an `Option<Color>` (iced) by `factor`
/// (CSS `opacity` baking).
fn scale_color(c: Option<Color>, factor: f32) -> Option<Color> {
    c.map(|mut c| {
        c.a *= factor;
        c
    })
}

// ---------------------------------------------------------------------------
// The lowered per-situation style
// ---------------------------------------------------------------------------

/// Constructor-level typography hints.
///
/// Iced carries typography in `text(...)`/`container(...)` *constructor
/// arguments*, not in the cascade — so these slots lower here and the caller
/// applies them when building the widget (`text("...").size(h.size())
/// .font(...)`), instead of being silently dropped.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextHints {
    /// `text-{size}` in px → `text(...).size(...)`.
    pub font_size: Option<f32>,
    /// `leading-*` → `text(...).line_height(...)`.
    pub line_height: Option<IcedLineHeight>,
    /// `font-{weight}` → `text(...).font(Font { weight, .. })`.
    pub weight: Option<IcedWeight>,
    /// `font-{sans,serif,mono}` → `text(...).font(Font { family, .. })`.
    pub family: Option<IcedFamily>,
    /// `italic` → `Font { style, .. }`.
    pub font_style: Option<IcedFontStyle>,
    /// `text-left/center/right` → `text(...).align(...)`.
    pub align: Option<IcedTextAlign>,
    /// `whitespace-*` → `text(...).wrapping(...)`.
    pub wrapping: Option<IcedWrapping>,
}

impl TextHints {
    /// Overlay `other` on top of `self` (later layers win per slot).
    pub fn overlay(&mut self, other: &TextHints) {
        if other.font_size.is_some() {
            self.font_size = other.font_size;
        }
        if other.line_height.is_some() {
            self.line_height = other.line_height;
        }
        if other.weight.is_some() {
            self.weight = other.weight;
        }
        if other.family.is_some() {
            self.family = other.family;
        }
        if other.font_style.is_some() {
            self.font_style = other.font_style;
        }
        if other.align.is_some() {
            self.align = other.align;
        }
        if other.wrapping.is_some() {
            self.wrapping = other.wrapping;
        }
    }
}

/// Constructor-level widget-argument hints.
///
/// The slots Iced expresses as widget arguments rather than cascade styles:
/// padding, fixed sizes, Row/Column spacing, cursor and interactivity. All
/// of them are *native* — they just live at the construction site, so the
/// backend hands them over instead of reporting them.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WidgetHints {
    /// `w-*` → `container(...).width(...)`.
    pub width: Option<Length>,
    /// `h-*` → `container(...).height(...)`.
    pub height: Option<Length>,
    /// `p-*` family → `container(...).padding(...)` (per-edge native).
    pub padding: Option<iced_core::Padding>,
    /// v0.8 §1.2: WHICH edges [`WidgetHints::padding`] addresses (the
    /// core IR's set-mask, carried so [`WidgetHints::overlay`] can merge
    /// edge-wise); unset edges are 0.0 placeholders.
    pub padding_set: EdgeSet,
    /// `gap-*` → `Row/Column::spacing` (the single axis Iced supports).
    pub spacing: Option<f32>,
    /// `gap-x-*` (column gap) — pick per widget kind; Iced spacing is
    /// one-axis-per-widget.
    pub spacing_x: Option<f32>,
    /// `gap-y-*` (row gap).
    pub spacing_y: Option<f32>,
    /// `cursor-*` → `MouseArea::on_cursor` / `Interaction` mapping.
    pub cursor: Option<iced_core::mouse::Interaction>,
    /// `pointer-events-none` → `Some(false)` (`container(...).width` stays;
    /// wrap in `MouseArea` or gate `Element::cascade`/sense at the call
    /// site). `pointer-events-auto` → `Some(true)`.
    pub interactive: Option<bool>,
}

impl WidgetHints {
    /// Overlay `other` on top of `self` (later layers win per slot).
    /// v0.8 §1.2: padding merges EDGE-WISE — an edge the other side does
    /// not address (mask off) keeps the accumulated value.
    pub fn overlay(&mut self, other: &WidgetHints) {
        if other.width.is_some() {
            self.width = other.width;
        }
        if other.height.is_some() {
            self.height = other.height;
        }
        if let Some(op) = &other.padding {
            let set = other.padding_set;
            match &mut self.padding {
                Some(sp) => {
                    if set.top {
                        sp.top = op.top;
                    }
                    if set.right {
                        sp.right = op.right;
                    }
                    if set.bottom {
                        sp.bottom = op.bottom;
                    }
                    if set.left {
                        sp.left = op.left;
                    }
                }
                None => self.padding = Some(*op),
            }
            if set.top {
                self.padding_set.top = true;
            }
            if set.right {
                self.padding_set.right = true;
            }
            if set.bottom {
                self.padding_set.bottom = true;
            }
            if set.left {
                self.padding_set.left = true;
            }
        }
        if other.spacing.is_some() {
            self.spacing = other.spacing;
        }
        if other.spacing_x.is_some() {
            self.spacing_x = other.spacing_x;
        }
        if other.spacing_y.is_some() {
            self.spacing_y = other.spacing_y;
        }
        if other.cursor.is_some() {
            self.cursor = other.cursor;
        }
        if other.interactive.is_some() {
            self.interactive = other.interactive;
        }
    }
}

/// Which cascade slots a lowered style explicitly set.
///
/// Iced's `Border`/`Shadow` are plain structs without an `Option` wrapper,
/// so the Some-replaces overlay bookkeeping of [`ResolvedStyle::overlay`]
/// is tracked beside the payload — a layer that sets only `rounded-*`
/// must not wipe the base layer's border color/width.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Presence {
    text_color: bool,
    background: bool,
    border_color: bool,
    border_width: bool,
    radius: bool,
    shadow: bool,
    /// v0.8 §1.2: WHICH corners `radius` addresses, for edge-wise merges.
    radius_set: CornerSet,
}

impl Presence {
    /// Union (a merged style sets whatever either side set).
    fn merge(&mut self, other: &Self) {
        self.text_color |= other.text_color;
        self.background |= other.background;
        self.border_color |= other.border_color;
        self.border_width |= other.border_width;
        self.radius |= other.radius;
        self.shadow |= other.shadow;
        self.radius_set.top_left |= other.radius_set.top_left;
        self.radius_set.top_right |= other.radius_set.top_right;
        self.radius_set.bottom_right |= other.radius_set.bottom_right;
        self.radius_set.bottom_left |= other.radius_set.bottom_left;
    }
}

/// One situation's lowered style: the cascade slots in
/// [`container::Style`] (which `button::Style` reuses field-for-field)
/// plus the two hint groups.
#[derive(Clone, Debug, PartialEq)]
pub struct IcedStyle {
    /// The native cascade payload (background incl. gradient, text color,
    /// border, shadow). `button::Style` carries the same four slots, with
    /// `text_color` non-optional.
    pub container: container::Style,
    presence: Presence,
    /// The style's own `opacity-*` factor (1.0 unless an opacity class
    /// set it). `lower_resolved` bakes it into this style's colors;
    /// `overlay` re-applies it to the accumulated style so a bare
    /// `disabled:opacity-50` layer still dims the base.
    opacity_factor: f32,
    /// Constructor-level typography (see [`TextHints`]).
    pub text: TextHints,
    /// Constructor-level widget arguments (see [`WidgetHints`]).
    pub widget: WidgetHints,
}

impl Default for IcedStyle {
    fn default() -> Self {
        Self {
            container: container::Style::default(),
            presence: Presence::default(),
            // A style with no opacity class paints fully opaque.
            opacity_factor: 1.0,
            text: TextHints::default(),
            widget: WidgetHints::default(),
        }
    }
}

impl IcedStyle {
    /// Overlay `other` on top of `self` (later layers win per slot),
    /// mirroring [`ResolvedStyle::overlay`]'s Some-replaces semantics.
    pub fn overlay(&mut self, other: &IcedStyle) {
        // CSS `opacity` composites the whole element: a layer that carries
        // an opacity factor dims everything accumulated so far, not just
        // its own colors (which `lower_resolved` already baked).
        if other.opacity_factor < 1.0 {
            let f = other.opacity_factor;
            let s = &mut self.container;
            if let Some(c) = s.text_color.as_mut() {
                c.a *= f;
            }
            match s.background.as_mut() {
                Some(Background::Color(c)) => c.a *= f,
                Some(Background::Gradient(g)) => *g = g.scale_alpha(f),
                None => {}
            }
            s.border.color.a *= f;
            s.shadow.color.a *= f;
        }
        let c = &other.container;
        let s = &mut self.container;
        if other.presence.text_color {
            s.text_color = c.text_color;
        }
        if other.presence.background {
            s.background = c.background;
        }
        if other.presence.border_color {
            s.border.color = c.border.color;
        }
        if other.presence.border_width {
            s.border.width = c.border.width;
        }
        if other.presence.radius {
            // v0.8 §1.2: per-corner merge through the corner set-mask.
            let set = other.presence.radius_set;
            if set.top_left {
                s.border.radius.top_left = c.border.radius.top_left;
            }
            if set.top_right {
                s.border.radius.top_right = c.border.radius.top_right;
            }
            if set.bottom_right {
                s.border.radius.bottom_right = c.border.radius.bottom_right;
            }
            if set.bottom_left {
                s.border.radius.bottom_left = c.border.radius.bottom_left;
            }
        }
        if other.presence.shadow {
            s.shadow = c.shadow;
        }
        self.presence.merge(&other.presence);
        self.text.overlay(&other.text);
        self.widget.overlay(&other.widget);
    }
}

// ---------------------------------------------------------------------------
// ResolvedStyle → IcedStyle
// ---------------------------------------------------------------------------

/// Lower one resolved style into an [`IcedStyle`].
///
/// Slots Iced cannot express (ring, outline, margin, flex layout, opacity
/// is baked, position/inset/... extras) are recorded as dropped — they are
/// capability-negotiated, never silently ignored.
pub fn lower_resolved(style: &ResolvedStyle, report: &mut LoweringReport) -> IcedStyle {
    let mut out = IcedStyle::default();

    // Opacity is not a cascade slot in Iced; CSS semantics (scale every
    // painted alpha) are baked at lowering time, documented below. The
    // factor is also remembered for `overlay` (a bare `disabled:opacity-50`
    // layer has no colors of its own to bake).
    let opacity = style.opacity.map_or(1.0, |o| o.0.clamp(0.0, 1.0));
    out.opacity_factor = opacity;

    if let Some(cc) = &style.content_color {
        out.container.text_color = Some(to_iced_color(cc.0));
        out.presence.text_color = true;
    }

    if let Some(bg) = &style.background {
        out.container.background = Some(Background::Color(to_iced_color(bg.0)));
        out.presence.background = true;
    }
    if let Some(gradient) = &style.background_gradient {
        match gradient {
            TwGradient::None => {}
            TwGradient::Linear { angle_deg, stops } => {
                if style.background.is_some() {
                    report.drop(
                        "background-color under gradient",
                        "CSS layers background-image over background-color; the single \
Background slot takes the gradient",
                    );
                }
                out.container.background =
                    Some(Background::Gradient(linear_gradient(*angle_deg, stops)));
                out.presence.background = true;
            }
            TwGradient::Radial { .. } => {
                report.drop(
                    "bg-radial",
                    "Iced 0.14's Gradient is linear-only (upstream marks radial TBD)",
                );
            }
            TwGradient::Conic { .. } => {
                report.drop(
                    "bg-conic-*",
                    "Iced 0.14's Gradient is linear-only (upstream marks conic TBD)",
                );
            }
        }
    }

    // Border: one (color, width) pair; iced's Border width is a uniform
    // f32, so per-side utilities honestly flatten to the top edge.
    let border_color = style.border_color.map(|c| to_iced_color(c.0));
    if let Some(bw) = &style.border_width {
        if !(bw.top == bw.right && bw.right == bw.bottom && bw.bottom == bw.left) {
            report.drop(
                "border-width:per-side",
                "Iced Border width is uniform; using top edge",
            );
        }
        out.container.border.width = bw.top;
        out.presence.border_width = true;
    }
    if let Some(color) = border_color {
        out.container.border.color = color;
        out.presence.border_color = true;
    }

    if let Some(cr) = &style.corner_radius {
        // Per-corner native: iced Radius carries all four corners.
        out.container.border.radius = Radius {
            top_left: cr.top_left,
            top_right: cr.top_right,
            bottom_right: cr.bottom_right,
            bottom_left: cr.bottom_left,
        };
        out.presence.radius = true;
        out.presence.radius_set = cr.set;
    }

    if let Some(s) = &style.shadow {
        if s.spread != 0.0 {
            report.drop(
                "shadow-spread",
                "Iced Shadow has no spread component (offset/blur/color only)",
            );
        }
        if s.inset {
            report.drop("shadow-inner", "Iced Shadow has no inset mode");
        }
        out.container.shadow = Shadow {
            color: to_iced_color(s.color),
            offset: Vector {
                x: s.offset_x,
                y: s.offset_y,
            },
            blur_radius: s.blur,
        };
        out.presence.shadow = true;
    }
    if !style.extras.shadow_rest.is_empty() {
        report.drop(
            "shadow layers 2+",
            "Iced's Style paints a single Shadow; extra preset layers reported",
        );
    }
    if !style.extras.inset_shadows.is_empty() {
        report.drop("inset-shadow-*", "Iced Shadow has no inset mode");
    }

    if style.ring.is_some() {
        report.drop(
            "ring",
            "no ring primitive in Iced; the single Border slot would collide with border-*",
        );
    }
    if style.outline.is_some() {
        report.drop(
            "outline",
            "no outline primitive in Iced; the single Border slot would collide with border-*",
        );
    }

    if style.margin.is_some() {
        report.drop(
            "margin",
            "no margin in Iced layout (wrap in a container's padding instead)",
        );
    }
    if style.extras.margin_auto.is_some() {
        report.drop("margin-auto", "no auto margins in Iced layout");
    }
    if style.layout.is_some() {
        report.drop(
            "layout",
            "flex direction/alignment is the widget CHOICE (Row vs Column + alignment \
constructors), not a cascade style",
        );
    }
    if style.min_dimensions.is_some() {
        report.drop(
            "min-w-/min-h-",
            "no per-widget min constraints in Iced (Length only)",
        );
    }
    if style.max_dimensions.is_some() {
        report.drop(
            "max-w-/max-h-",
            "no per-widget max constraints in Iced (Length only)",
        );
    }

    // Dimensions → Length hints (Px/Scale/Symbolic natively; fractional as
    // a documented FillPortion normalization).
    if let Some(dims) = &style.dimensions {
        out.widget.width = dims.width.map(sized_to_length);
        out.widget.height = dims.height.map(sized_to_length);
    }

    // Padding → widget-argument hint (per-edge native at construction).
    // v0.8 §1.2: the set-mask rides along for edge-wise merges.
    if let Some(p) = &style.padding {
        out.widget.padding = Some(iced_core::Padding {
            top: p.top,
            right: p.right,
            bottom: p.bottom,
            left: p.left,
        });
        out.widget.padding_set = p.set;
    }

    if let Some(g) = &style.gap {
        out.widget.spacing = Some(g.0);
    }
    if let Some(axes) = &style.extras.gap_axes {
        out.widget.spacing_x = axes.col;
        out.widget.spacing_y = axes.row;
    }

    if let Some(cursor) = &style.cursor {
        out.widget.cursor = Some(to_iced_interaction(*cursor));
    }
    if let Some(pe) = &style.pointer_events {
        out.widget.interactive = Some(matches!(pe, aeolus_core::plan::PointerEvents::Auto));
    }

    lower_text(style, &mut out, report);

    // v0.2 extras without any Iced 0.14 cascade equivalent.
    report_extras(style, report);

    // Bake CSS opacity into every color the style paints (Iced has no
    // per-widget opacity primitive; alpha scaling is the CSS-equivalent
    // composite).
    if opacity < 1.0 {
        out.container.text_color = scale_color(out.container.text_color, opacity);
        out.container.background = match out.container.background {
            Some(Background::Color(c)) => Some(Background::Color(Color {
                a: c.a * opacity,
                ..c
            })),
            Some(Background::Gradient(g)) => Some(Background::Gradient(g.scale_alpha(opacity))),
            None => None,
        };
        out.container.border.color.a *= opacity;
        out.container.shadow.color.a *= opacity;
    }

    out
}

/// `SizedValue` → Iced `Length` (the documented normalizations: `full` →
/// Fill, fractions → FillPortion(%) with a minimum of 1, fit/auto → Shrink,
/// viewport → Fill).
fn sized_to_length(v: SizedValue) -> Length {
    match v {
        SizedValue::Px(px) => Length::Fixed(px),
        SizedValue::Scale(n) => Length::Fixed(n * 4.0),
        SizedValue::Fraction(f) => Length::FillPortion(((f * 100.0).round() as u16).max(1)),
        SizedValue::Full | SizedValue::Viewport => Length::Fill,
        SizedValue::FitContent | SizedValue::Auto => Length::Shrink,
    }
}

/// CSS angle (degrees; 0 = to top, clockwise) → Iced linear-gradient angle.
///
/// DERIVED FROM ICED'S PAINT MATH, not its doc comment
/// (`iced_core/src/angle.rs::Radians::to_distance`):
/// `angle = θ − π/2`, `r = (cos, sin)`, `start = center − r·d`,
/// `end = center + r·d` — so the END of the gradient line sits at screen
/// direction `(sin θ, −cos θ)`:
/// - CSS 0° (to top)    → θ = 0     → end (0, −1) up      ✓
/// - CSS 90° (to right) → θ = π/2   → end (1, 0) right    ✓
/// - CSS 180° (to bottom) → θ = π   → end (0, 1) down     ✓
/// - CSS 45° (to top-right) → θ = π/4 → end up-right     ✓
///
/// Iced already uses the CSS compass convention; the conversion is the
/// plain degrees→radians step. Pinned by
/// `tests/lowering.rs::linear_gradient_angle_conversion`.
fn css_to_iced_angle(angle_deg: f32) -> Radians {
    Radians(angle_deg.to_radians())
}

/// One composed gradient → Iced's native `Gradient::Linear` (at most the
/// from/via/to stops, each ≤ 8 as Iced's fixed table requires).
fn linear_gradient(angle_deg: f32, stops: &GradientStops) -> Gradient {
    use aeolus_core::plan::{GRADIENT_FROM_POS_PCT, GRADIENT_TO_POS_PCT, GRADIENT_VIA_POS_PCT};
    let pos = |p: Option<f32>, default: f32| p.unwrap_or(default) / 100.0;
    let mut g = iced_core::gradient::Linear::new(css_to_iced_angle(angle_deg));
    g = g.add_stop(
        pos(stops.from.pos_pct, GRADIENT_FROM_POS_PCT),
        to_iced_color(stops.from.color),
    );
    if let Some(via) = stops.via {
        g = g.add_stop(
            pos(via.pos_pct, GRADIENT_VIA_POS_PCT),
            to_iced_color(via.color),
        );
    }
    g = g.add_stop(
        pos(stops.to.pos_pct, GRADIENT_TO_POS_PCT),
        to_iced_color(stops.to.color),
    );
    Gradient::Linear(g)
}

/// The full v4 cursor keyword set → Iced `mouse::Interaction`.
fn to_iced_interaction(c: aeolus_core::plan::Cursor) -> iced_core::mouse::Interaction {
    use aeolus_core::plan::Cursor as C;
    use iced_core::mouse::Interaction as I;
    match c {
        C::Auto | C::Default => I::Idle,
        C::None => I::Hidden,
        C::Pointer => I::Pointer,
        C::Wait => I::Wait,
        C::Progress => I::Progress,
        C::Text => I::Text,
        C::Move => I::Move,
        C::Help => I::Help,
        C::NotAllowed => I::NotAllowed,
        C::ContextMenu => I::ContextMenu,
        C::Cell => I::Cell,
        C::Crosshair => I::Crosshair,
        C::VerticalText => I::Text,
        C::Alias => I::Alias,
        C::Copy => I::Copy,
        C::NoDrop => I::NoDrop,
        C::Grab => I::Grab,
        C::Grabbing => I::Grabbing,
        C::AllScroll => I::AllScroll,
        C::ColResize => I::ResizingColumn,
        C::RowResize => I::ResizingRow,
        C::NResize | C::SResize => I::ResizingVertically,
        C::EResize | C::WResize => I::ResizingHorizontally,
        C::NeResize | C::SwResize => I::ResizingDiagonallyUp,
        C::NwResize | C::SeResize => I::ResizingDiagonallyDown,
        C::EwResize => I::ResizingHorizontally,
        C::NsResize => I::ResizingVertically,
        C::NeswResize => I::ResizingDiagonallyUp,
        C::NwseResize => I::ResizingDiagonallyDown,
        C::ZoomIn => I::ZoomIn,
        C::ZoomOut => I::ZoomOut,
    }
}

/// Typography mapping: the slots with a native constructor argument lower
/// into [`TextHints`], the rest report.
fn lower_text(style: &ResolvedStyle, out: &mut IcedStyle, report: &mut LoweringReport) {
    for (prop, value) in &style.text {
        match (prop, value) {
            (TextProperty::Color, TextValue::Color(c)) => {
                out.container.text_color = Some(to_iced_color(*c));
                out.presence.text_color = true;
            }
            (TextProperty::FontSize, TextValue::Px(px)) => {
                out.text.font_size = Some(*px);
            }
            (TextProperty::LineHeight, TextValue::Px(px)) => {
                out.text.line_height = Some(IcedLineHeight::Absolute(Pixels(*px)));
            }
            (TextProperty::LineHeight, TextValue::LineHeightRatio(r)) => {
                out.text.line_height = Some(IcedLineHeight::Relative(*r));
            }
            (TextProperty::FontWeight, TextValue::Weight(w)) => {
                out.text.weight = Some(weight_to_iced(*w));
            }
            (TextProperty::FontFamily, TextValue::Keyword(k)) => {
                out.text.family = Some(match *k {
                    "sans" => IcedFamily::SansSerif,
                    "serif" => IcedFamily::Serif,
                    "mono" => IcedFamily::Monospace,
                    other => {
                        report.drop(
                            format!("font-{other}"),
                            "custom families need the app's Font definitions; \
font-sans/serif/mono map natively",
                        );
                        continue;
                    }
                });
            }
            (TextProperty::FontStyle, TextValue::Keyword(k)) if *k == "italic" => {
                out.text.font_style = Some(IcedFontStyle::Italic);
            }
            (TextProperty::TextAlign, TextValue::Keyword(k)) => {
                out.text.align = Some(match *k {
                    "left" => IcedTextAlign::Left,
                    "center" => IcedTextAlign::Center,
                    "right" => IcedTextAlign::Right,
                    other => {
                        report.drop(format!("text-{other}"), "no such text alignment in Iced");
                        continue;
                    }
                });
            }
            (TextProperty::Whitespace, TextValue::Keyword(k)) => {
                out.text.wrapping = Some(match *k {
                    "normal" => IcedWrapping::Word,
                    "nowrap" => IcedWrapping::None,
                    other => {
                        report.drop(
                            format!("whitespace-{other}"),
                            "Iced wrapping is Word/Glyph/None; pre/newline reported",
                        );
                        continue;
                    }
                });
            }
            (prop, _) => report.drop(
                format!("typography:{prop:?}"),
                "no Iced 0.14 cascade/constructor equivalent for this text property",
            ),
        }
    }
}

/// CSS numeric weight 100..=900 → the nearest Iced `font::Weight` step.
fn weight_to_iced(w: u16) -> IcedWeight {
    let w = w.clamp(100, 900);
    match (w + 50) / 100 {
        1 => IcedWeight::Thin,
        2 => IcedWeight::ExtraLight,
        3 => IcedWeight::Light,
        4 => IcedWeight::Normal,
        5 => IcedWeight::Medium,
        6 => IcedWeight::Semibold,
        7 => IcedWeight::Bold,
        8 => IcedWeight::ExtraBold,
        _ => IcedWeight::Black,
    }
}

/// Report the v0.2 extras Iced 0.14 has no equivalent for, per family.
fn report_extras(style: &ResolvedStyle, report: &mut LoweringReport) {
    let e = &style.extras;
    if e.position.is_some() {
        report.drop("position", "no absolute positioning in Iced layout");
    }
    if e.inset.is_some() {
        report.drop("inset-*", "no offset positioning in Iced layout");
    }
    if e.z_index.is_some() {
        report.drop("z-index", "z-order is element-tree order in Iced");
    }
    if e.overflow.is_some() {
        report.drop(
            "overflow",
            "clipping is widget-level (Scrollable/clip), not a style",
        );
    }
    if e.visibility.is_some() {
        report.drop(
            "visible/invisible",
            "no visibility cascade in Iced (drop the element instead)",
        );
    }
    if e.aspect_ratio.is_some() {
        report.drop("aspect-*", "no aspect-ratio cascade in Iced");
    }
    if e.grid.is_some() {
        report.drop(
            "grid-*",
            "Iced's Grid widget takes templates as constructor args",
        );
    }
    if e.transform.is_some() {
        report.drop("transform", "no per-element transform in Iced 0.14");
    }
    if e.transition.is_some() {
        report.drop(
            "transition-*",
            "Iced 0.14 has no style transitions (animation::loom for values instead)",
        );
    }
    if e.animation.is_some() {
        report.drop(
            "animate-*",
            "Iced animations are per-value (animation crate), not classes",
        );
    }
    if let Some(bs) = &e.border_style {
        match bs {
            aeolus_core::plan::BorderStyle::Solid => {}
            aeolus_core::plan::BorderStyle::None | aeolus_core::plan::BorderStyle::Hidden => {
                // A genuine `border-none` clears the width — native.
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
                report.drop("border-style", "Iced borders are solid strokes");
            }
        }
    }
    if e.flex_basis.is_some() {
        report.drop(
            "basis-*",
            "flex sizing is widget-level (width on Row/Column children)",
        );
    }
    if e.order.is_some() {
        report.drop(
            "order-*",
            "Iced has no flex order (list order is source order)",
        );
    }
    if e.align_content.is_some() {
        report.drop("content-*", "align-content has no Iced equivalent");
    }
}

// ---------------------------------------------------------------------------
// The style book: plan → state-closure cascade
// ---------------------------------------------------------------------------

/// The lowered [`StylePlan`]: base + rank-sorted conditional layers + dark,
/// resolved per state by the framework's Catalog closures.
///
/// Construct once per element (or per component class), keep it in the
/// view's state, and call [`StyleBook::container_style`] /
/// [`StyleBook::button_style`] from the `style(...)` closure — or
/// [`StyleBook::resolve_at`] with an [`IcedProbe`] for the full parity path.
#[derive(Clone, Debug, PartialEq)]
pub struct StyleBook {
    /// Unconditional properties.
    pub base: IcedStyle,
    /// Conditional layers, ascending [`VariantCondition::rank`] order.
    pub layers: Vec<(VariantCondition, IcedStyle)>,
    /// `dark:` override (applied when the `&Theme` is dark).
    pub dark: Option<IcedStyle>,
    /// The original plan, kept for the [`StyleBook::resolve_at`] parity path
    /// (group/peer/rules/compound variants evaluated by the core evaluator).
    plan: StylePlan,
}

impl StyleBook {
    /// Lower a full [`StylePlan`].
    ///
    /// Conditions Iced's Status model cannot express natively
    /// (group/peer, compound rules, responsive, container) are reported —
    /// they remain available through [`StyleBook::resolve_at`].
    pub fn new(plan: &StylePlan) -> (Self, LoweringReport) {
        let mut report = LoweringReport::default();

        for (group_id, _) in &plan.group {
            report.drop(
                format!("group:{group_id}"),
                "no ancestor-state signals in a Catalog closure — use resolve_at with \
a group-aware IcedProbe",
            );
        }
        for (peer_id, _) in &plan.peer {
            report.drop(
                format!("peer:{peer_id}"),
                "no sibling-state signals in a Catalog closure — use resolve_at with \
a peer-aware IcedProbe",
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
                "no window-size signal inside a Catalog closure (Iced's responsive() \
widget is view-level)",
            );
        }
        for bp in plan.container.keys() {
            report.drop(format!("container:{bp:?}"), "no container queries in Iced");
        }

        let base = lower_resolved(&plan.base, &mut report);
        let mut sorted: Vec<&aeolus_core::plan::ConditionalLayer> = plan.layers.iter().collect();
        sorted.sort_by_key(|l| l.condition.rank());
        let mut layers = Vec::with_capacity(sorted.len());
        for layer in sorted {
            // Iced 0.14's `button::Status` has no Focused variant and
            // containers are state-less, so focus/visited conditions
            // cannot fire natively — reported, available via resolve_at.
            let native = matches!(
                layer.condition,
                VariantCondition::Hover | VariantCondition::Active | VariantCondition::Disabled
            );
            if !native {
                report.drop(
                    format!("{:?}:", layer.condition),
                    "no focus/visited Status in Iced 0.14's button::Status — \
use resolve_at with IcedProbe.focused",
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
                    tracing::warn!("aeolus-iced: dropped {:?}", report.dropped);
                }
                book
            }
            Err(err) => {
                assert!(
                    !cfg!(debug_assertions),
                    "aeolus: failed to parse class string {classes:?}: {err:?}"
                );
                tracing::warn!("aeolus: dropping unparseable class string {classes:?}: {err:?}");
                let (book, _) = Self::new(&StylePlan::default());
                book
            }
        }
    }

    /// Overlay the plan's dark layer when the Iced theme is dark.
    fn merged_for_theme(&self, theme: &iced_core::Theme) -> IcedStyle {
        let mut style = self.base.clone();
        if theme_is_dark(theme)
            && let Some(dark) = &self.dark
        {
            style.overlay(dark);
        }
        style
    }

    /// The `container::Style` for this theme — the native Catalog closure
    /// payload (base + `dark:`; containers have no Status in Iced).
    pub fn container_style(&self, theme: &iced_core::Theme) -> container::Style {
        self.merged_for_theme(theme).container
    }

    /// The `container::StyleFn` for this book, ready for
    /// `container(...).style(book_fn)` with an owned handle.
    pub fn container_style_fn(self: Arc<Self>) -> impl Fn(&iced_core::Theme) -> container::Style {
        move |theme| self.container_style(theme)
    }

    /// The `button::Style` for `status` — the interactive Catalog path
    /// (Hovered → `hover:`, Pressed → `active:`, Disabled → `disabled:`,
    /// Focused → `focus-visible:`; plus the dark layer from the theme).
    ///
    /// `button::Style.text_color` is not optional, so the theme's palette
    /// text color fills the slot when no class set one.
    pub fn button_style(&self, status: button::Status, theme: &iced_core::Theme) -> button::Style {
        let mut style = self.merged_for_theme(theme);
        for (condition, layer) in &self.layers {
            if condition_matches_status(*condition, status) {
                style.overlay(layer);
            }
        }
        button::Style {
            background: style.container.background,
            text_color: style.container.text_color.unwrap_or(theme.palette().text),
            border: style.container.border,
            shadow: style.container.shadow,
            snap: style.container.snap,
        }
    }

    /// The `button::StyleFn` for this book, ready for
    /// `button(...).style(book_fn)` with an owned handle.
    pub fn button_style_fn(
        self: Arc<Self>,
    ) -> impl Fn(&iced_core::Theme, button::Status) -> button::Style {
        move |theme, status| self.button_style(status, theme)
    }

    /// The FULL parity path (plan §4.2): flatten the original plan for the
    /// situation the probe describes — group/peer, data/aria, compound
    /// variants, structural and container conditions included — and lower
    /// the result. Iced re-asks every frame, so re-resolution here is the
    /// StateClosures contract, not a rebuild.
    pub fn resolve_at(&self, probe: &impl Probe) -> IcedStyle {
        let resolved = self.plan.resolve(probe);
        let mut report = LoweringReport::default();
        lower_resolved(&resolved, &mut report)
    }
}

/// Does the layer condition fire for this Iced button status?
///
/// (Iced 0.14 has no `Status::Focused`; focus conditions are reported at
/// [`StyleBook::new`] and resolve through [`StyleBook::resolve_at`].)
fn condition_matches_status(c: VariantCondition, s: button::Status) -> bool {
    matches!(
        (c, s),
        (VariantCondition::Hover, button::Status::Hovered)
            | (VariantCondition::Active, button::Status::Pressed)
            | (VariantCondition::Disabled, button::Status::Disabled)
    )
}

/// Is this Iced theme a dark one? Decided from the palette's background
/// luminance so the named variants (and `Theme::Custom`) all answer
/// honestly without a per-variant table.
pub fn theme_is_dark(theme: &iced_core::Theme) -> bool {
    let bg = theme.palette().background;
    0.2126 * bg.r + 0.7152 * bg.g + 0.0722 * bg.b < 0.5
}

// ---------------------------------------------------------------------------
// The parity probe (plan §4.2)
// ---------------------------------------------------------------------------

/// The element's current situation as the Iced app sees it.
///
/// The app (or a widget wrapper) keeps this in its state and hands it to
/// [`StyleBook::resolve_at`]; because Iced re-runs the style closure every
/// frame, group/peer and compound variants re-evaluate without any widget
/// rebuild — the Floem reactive contract, in catalog form.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct IcedProbe {
    /// `hover:` / `group-hover:`.
    pub hovered: bool,
    /// `focus:` / `focus-visible:`.
    pub focused: bool,
    /// `active:`.
    pub active: bool,
    /// `disabled:`.
    pub disabled: bool,
    /// `dark:` (derive from `theme_is_dark(theme)`).
    pub dark: bool,
    /// Current viewport width (window), when known.
    pub width: Option<f32>,
    /// Current container width, when this element is inside a container
    /// the app tracks.
    pub container_width: Option<f32>,
}

impl Probe for IcedProbe {
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

impl From<&iced_core::Theme> for IcedProbe {
    /// A probe preset from the Iced theme: only the dark media flag is
    /// knowable from `&Theme` alone.
    fn from(theme: &iced_core::Theme) -> Self {
        Self {
            dark: theme_is_dark(theme),
            ..Self::default()
        }
    }
}

// ---------------------------------------------------------------------------
// Parity accounting (plan §4.1): every slot is native, emulated, or reported.
// ---------------------------------------------------------------------------

/// How a slot is covered on this backend.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Coverage {
    /// Mapped to a native Iced cascade or constructor slot.
    Native,
    /// Emulated inside the lowering (opacity alpha baking).
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
            if matches!(
                s.background_gradient,
                Some(TwGradient::Linear { .. } | TwGradient::None)
            ) {
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
        v.push((S::MinDimensions, Coverage::Reported));
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
        v.push((S::Margin, Coverage::Reported));
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
        v.push((S::Outline, Coverage::Reported));
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
            | TextProperty::LineHeight
            | TextProperty::FontWeight
            | TextProperty::FontFamily
            | TextProperty::FontStyle
            | TextProperty::TextAlign
            | TextProperty::Whitespace => Coverage::Native,
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
