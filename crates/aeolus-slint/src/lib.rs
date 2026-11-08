//! # Aeolus Slint backend — `StylePlan` → Slint brushes, element properties and `.slint` snippets
//!
//! Slint has no cascade and no CSS parser: elements carry typed properties
//! set by bindings, state restyling is authored with `states [ name when
//! condition : { … } ]` blocks, and the whole look is a compile-time widget
//! style. Three native surfaces exist, and this backend lowers into all of
//! them:
//!
//! 1. **Element properties** — a [`ResolvedStyle`] lowers into a
//!    [`SlintStyle`] whose fields are Slint's own value types:
//!    [`slint::Brush`] for `background`/`border-color` (solid fills AND
//!    native gradients — `@linear-gradient`/`@radial-gradient`/
//!    `@conic-gradient` are real brush values), lengths for
//!    `border-width`/per-corner radii/`drop-shadow-*`/`inner-shadow-*`,
//!    `opacity`, `clip`, and the typography surface
//!    ([`SlintText`]) for `Text` items.
//! 2. **Generated `.slint` code** — [`SlintStyle::snippet`] emits the
//!    property bindings a `.slint` element takes verbatim
//!    (`background: #18181b; border-radius: 8px; …`), and
//!    [`StyleBook::to_states_snippet`] emits the `states [ … ]` block that
//!    makes a `TouchArea`-driven element restyle itself — Slint's own state
//!    machinery, exactly the generated-code path the capability audit
//!    prescribes for frameworks without runtime rule injection.
//! 3. **The Rust-side parity path** — [`StyleBook::resolve_at`] + the
//!    [`SlintProbe`] flag struct answer `group-hover:`, `dark:`, responsive
//!    and compound conditions through the backend-agnostic evaluator; the
//!    app pushes the resolved values into properties (or globals) itself.
//!
//! Slint 1.18's `Rectangle` is the strongest paint surface of the four
//! backends: per-corner radii (`border-top-left-radius`…), drop shadows
//! with spread, inner (inset) shadows and native gradient brushes all
//! lower natively. Whatever still cannot be expressed lands in a
//! [`LoweringReport`] with a per-slot reason — never a silent no-op.
//!
//! ```no_run
//! use aeolus_core::{Resolver, shadcn_default};
//! use aeolus_slint::{SlintProbe, StyleBook};
//!
//! let theme = shadcn_default().unwrap();
//! let resolver = Resolver::new(&theme);
//!
//! // Lower once (cache it on the app state) …
//! let book = StyleBook::tw(&resolver,
//!     "p-4 bg-primary text-primary-foreground rounded-md hover:bg-primary/90");
//!
//! // … and use both surfaces:
//! // (a) paste the generated bindings into your `.slint` source
//! println!("{}", book.base_snippet());
//! // (b) resolve in Rust for the element's current situation
//! let probe = SlintProbe { has_hover: true, ..SlintProbe::default() };
//! let _style = book.resolve_for(&probe);
//! ```

use i_slint_core::graphics::{
    Brush, Color, ConicGradientBrush, GradientStop, LinearGradientBrush, RadialGradientBrush,
};
use i_slint_core::items::{BuiltInMouseCursor, TextHorizontalAlignment, TextOverflow, TextWrap};

use aeolus_core::color::Color as TwColor;
use aeolus_core::eval::{Probe, Rel};
use aeolus_core::plan::{
    Gradient, ResolvedStyle, SizedValue, Slot, StylePlan, TextProperty, TextValue, VariantCondition,
};
use std::fmt::Write as _;

// ---------------------------------------------------------------------------
// The honest report
// ---------------------------------------------------------------------------

/// Everything the lowering could not express in Slint 1.18.
///
/// Aeolus never drops a class silently: conditions/properties without a
/// Slint equivalent are reported here (and should be surfaced through
/// `tracing::warn!` by the host, matching `aeolus-core`'s philosophy).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LoweringReport {
    /// `(what, reason)` pairs, e.g. `("ring", "no ring primitive; emulate \
    /// with a nested Rectangle in .slint")`.
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

// ---------------------------------------------------------------------------
// Value conversions
// ---------------------------------------------------------------------------

/// Convert our Oklab [`TwColor`] to Slint's `Color`.
///
/// `Color::from_argb_encoded` takes `0xAARRGGBB`; `Color::to_srgba`
/// performs the OKLab → linear-sRGB → sRGB transfer conversion (and clamps
/// out-of-gamut values, which Tailwind's own palette never produces).
pub fn to_slint_color(c: TwColor) -> Color {
    let [r, g, b, a] = c.to_srgba();
    Color::from_argb_encoded(
        ((a * 255.0).round() as u32) << 24
            | ((r * 255.0).round() as u32) << 16
            | ((g * 255.0).round() as u32) << 8
            | (b * 255.0).round() as u32,
    )
}

/// Format an Oklab [`TwColor`] as a `.slint` color literal: `#RRGGBB` when
/// opaque, `#RRGGBBAA` otherwise (slint parses hex with alpha LAST — the
/// reverse of the Rust API's `0xAARRGGBB`).
pub fn color_snippet(c: TwColor) -> String {
    slint_color_snippet(to_slint_color(c))
}

/// Format Slint's own [`Color`] the same way (the brushes the lowering
/// built carry Slint colors; snippets recover the hex literals).
pub fn slint_color_snippet(c: Color) -> String {
    let argb = c.as_argb_encoded();
    let (r, g, b, a) = (
        ((argb >> 16) & 0xFF) as u8,
        ((argb >> 8) & 0xFF) as u8,
        (argb & 0xFF) as u8,
        ((argb >> 24) & 0xFF) as u8,
    );
    if a == 0xFF {
        format!("#{r:02X}{g:02X}{b:02X}")
    } else {
        format!("#{r:02X}{g:02X}{b:02X}{a:02X}")
    }
}

/// A length as the `.slint`/Rust surface carries it: absolute px or a
/// parent-relative percent. Percent support covers `w-full`/`w-1/2`-style
/// fractions, which Slint lengths express natively (`width: 50%;`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SlintLength {
    /// Absolute logical px.
    Px(f32),
    /// Percent of the containing element (0..=100).
    Percent(f32),
}

impl SlintLength {
    /// The `.slint` literal: `100px` / `50%`.
    pub fn snippet(self) -> String {
        match self {
            SlintLength::Px(px) => format!("{px}px"),
            SlintLength::Percent(p) => format!("{p}%"),
        }
    }
}

/// Map an aeolus [`SizedValue`] to the length Slint can carry, reporting
/// the values the typed slot cannot represent (`w-fit`, `w-screen`, …).
/// `SizedValue::Auto` maps to `None` WITHOUT a report: auto sizing is
/// Slint's default, so the no-op is semantically exact.
fn slint_length(v: SizedValue, what: &str, report: &mut LoweringReport) -> Option<SlintLength> {
    match v {
        SizedValue::Px(px) => Some(SlintLength::Px(px)),
        SizedValue::Scale(n) => Some(SlintLength::Px(n * 4.0)),
        SizedValue::Fraction(f) => Some(SlintLength::Percent(f * 100.0)),
        SizedValue::Full => Some(SlintLength::Percent(100.0)),
        SizedValue::Auto => None,
        SizedValue::FitContent => {
            report.drop(
                format!("{what}:fit"),
                "no fit-content keyword; Slint elements size to content by default — \
                 unset the binding instead",
            );
            None
        }
        SizedValue::Viewport => {
            report.drop(
                format!("{what}:screen"),
                "bind the property to the Window width yourself (100% of the viewport)",
            );
            None
        }
    }
}

/// Map a gradient stop (percent position, `None` = the v4 `@property`
/// default) to Slint's normalized 0..=1 stop position.
fn stop_position(pos_pct: Option<f32>, default_pct: f32) -> f32 {
    pos_pct.unwrap_or(default_pct) / 100.0
}

/// Lower one aeolus [`Gradient`] into a native Slint brush.
///
/// Angle conventions match CSS on both sides (verified against
/// `i-slint-core`'s `line_for_angle`: 0deg = to top, clockwise), so the
/// plan's degrees transfer as-is.
pub fn gradient_to_brush(g: &Gradient) -> Option<Brush> {
    let stops = g.stops();
    let from = GradientStop {
        color: to_slint_color(stops.from.color),
        position: stop_position(stops.from.pos_pct, aeolus_core::plan::GRADIENT_FROM_POS_PCT),
    };
    let via = stops.via.map(|v| GradientStop {
        color: to_slint_color(v.color),
        position: stop_position(v.pos_pct, aeolus_core::plan::GRADIENT_VIA_POS_PCT),
    });
    let to = GradientStop {
        color: to_slint_color(stops.to.color),
        position: stop_position(stops.to.pos_pct, aeolus_core::plan::GRADIENT_TO_POS_PCT),
    };
    let all: Vec<GradientStop> = via.iter().copied().chain([from, to]).collect();
    match g {
        // `bg-none` clears the image; the caller handles slot semantics.
        Gradient::None => None,
        Gradient::Linear { angle_deg, .. } => Some(Brush::LinearGradient(
            LinearGradientBrush::new(*angle_deg, all),
        )),
        // The plan's radial shape is `circle farthest-corner` (the
        // documented normalization); Slint's circle brush matches.
        Gradient::Radial { .. } => {
            Some(Brush::RadialGradient(RadialGradientBrush::new_circle(all)))
        }
        Gradient::Conic { from_angle_deg, .. } => Some(Brush::ConicGradient(
            ConicGradientBrush::new(*from_angle_deg, all),
        )),
    }
}

// ---------------------------------------------------------------------------
// Typography
// ---------------------------------------------------------------------------

/// One situation's lowered typography: the [`slint::Brush`] colors and the
/// `Text`-item property values ([`SlintStyle::text`] carries it; emit the
/// bindings on the `Text` element).
///
/// Slint `Text` has no cascade: `font-size`, `font-weight`,
/// `letter-spacing`, `font-italic`, `color`, `wrap`, `overflow` and the
/// alignments are per-item properties ([`SlintText::snippet`] emits them).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SlintText {
    /// `text-{color}` — the `color` property.
    pub color: Option<Brush>,
    /// `text-{size}` — `font-size` in logical px.
    pub font_size: Option<f32>,
    /// `font-{weight}` 100..=900 — `font-weight`.
    pub font_weight: Option<i32>,
    /// `tracking-*` resolved to px — `letter-spacing`.
    pub letter_spacing: Option<f32>,
    /// `font-{family}` — `font-family` as the family name string
    /// (`font-sans` stays unset: the default font IS the sans face).
    pub font_family: Option<String>,
    /// `italic` / `not-italic` — `font-italic`.
    pub italic: Option<bool>,
    /// `whitespace-nowrap` → [`TextWrap::NoWrap`], `whitespace-normal` →
    /// [`TextWrap::WordWrap`].
    pub wrap: Option<TextWrap>,
    /// `text-ellipsis` → [`TextOverflow::Elide`], `text-clip` →
    /// [`TextOverflow::Clip`].
    pub overflow: Option<TextOverflow>,
    /// `text-left/center/right` — `horizontal-alignment`.
    pub horizontal_alignment: Option<TextHorizontalAlignment>,
}

impl SlintText {
    /// Overlay `other` on top of `self` (later layers win per slot).
    pub fn overlay(&mut self, other: &SlintText) {
        if other.color.is_some() {
            self.color.clone_from(&other.color);
        }
        if other.font_size.is_some() {
            self.font_size = other.font_size;
        }
        if other.font_weight.is_some() {
            self.font_weight = other.font_weight;
        }
        if other.letter_spacing.is_some() {
            self.letter_spacing = other.letter_spacing;
        }
        if other.font_family.is_some() {
            self.font_family.clone_from(&other.font_family);
        }
        if other.italic.is_some() {
            self.italic = other.italic;
        }
        if other.wrap.is_some() {
            self.wrap = other.wrap;
        }
        if other.overflow.is_some() {
            self.overflow = other.overflow;
        }
        if other.horizontal_alignment.is_some() {
            self.horizontal_alignment = other.horizontal_alignment;
        }
    }

    /// The `Text`-item property bindings, one `name: value;` per line.
    ///
    /// Font weights/lengths are plain numbers + `px`; the brush colors use
    /// [`color_snippet`]'s hex literal.
    pub fn snippet(&self) -> String {
        let mut lines = Vec::new();
        if let Some(c) = &self.color {
            lines.push(format!("color: {};", brush_snippet(c)));
        }
        if let Some(px) = self.font_size {
            lines.push(format!("font-size: {}px;", fmt_f32(px)));
        }
        if let Some(w) = self.font_weight {
            lines.push(format!("font-weight: {w};"));
        }
        if let Some(px) = self.letter_spacing {
            lines.push(format!("letter-spacing: {}px;", fmt_f32(px)));
        }
        if let Some(family) = &self.font_family {
            lines.push(format!("font-family: \"{family}\";"));
        }
        if let Some(it) = self.italic {
            lines.push(format!("font-italic: {it};"));
        }
        if let Some(w) = self.wrap {
            lines.push(format!("wrap: {w};"));
        }
        if let Some(o) = self.overflow {
            lines.push(format!("overflow: {o};"));
        }
        if let Some(a) = self.horizontal_alignment {
            lines.push(format!("horizontal-alignment: {a};"));
        }
        lines.join("\n")
    }
}

/// Format an f32 without trailing `.0` noise (`8` not `8.0`) for snippets.
fn fmt_f32(v: f32) -> String {
    let s = format!("{v:.3}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    s.to_string()
}

/// Format a [`Brush`] as a `.slint` brush expression: a hex color literal
/// for solid brushes, `@linear-gradient`/`@radial-gradient`/
/// `@conic-gradient` calls otherwise (stop positions in % for linear and
/// radial, in degrees for conic — slint's own grammar). The linear angle
/// comes back out of [`LinearGradientBrush::angle`]; a conic brush has no
/// angle accessor because [`ConicGradientBrush::new`] bakes the rotation
/// into the normalized stops, so conic snippets are angle-less — the
/// rotation is already in the stops.
pub fn brush_snippet(b: &Brush) -> String {
    match b {
        Brush::SolidColor(c) => slint_color_snippet(*c),
        Brush::LinearGradient(g) => {
            let stops: Vec<String> = g
                .stops()
                .map(|s| {
                    format!(
                        "{} {}%",
                        slint_color_snippet(s.color),
                        fmt_f32(s.position * 100.0)
                    )
                })
                .collect();
            format!(
                "@linear-gradient({}deg, {})",
                fmt_f32(g.angle()),
                stops.join(", ")
            )
        }
        Brush::RadialGradient(g) => {
            let stops: Vec<String> = g
                .stops()
                .map(|s| {
                    format!(
                        "{} {}%",
                        slint_color_snippet(s.color),
                        fmt_f32(s.position * 100.0)
                    )
                })
                .collect();
            format!("@radial-gradient(circle, {})", stops.join(", "))
        }
        Brush::ConicGradient(g) => {
            let stops: Vec<String> = g
                .stops()
                .map(|s| {
                    format!(
                        "{} {}deg",
                        slint_color_snippet(s.color),
                        fmt_f32(s.position * 360.0)
                    )
                })
                .collect();
            format!("@conic-gradient({})", stops.join(", "))
        }
        // `Brush` is #[non_exhaustive] (future variants); `transparent` is
        // slint's brush literal for "paint nothing".
        _ => "transparent".to_string(),
    }
}

// ---------------------------------------------------------------------------
// The lowered per-situation style
// ---------------------------------------------------------------------------

/// One situation's lowered style: the element-property payload in Slint's
/// own value types.
///
/// Apply it by setting the properties on the target element — directly in
/// Rust (the `Rectangle`'s `background`/`border-*`/… properties), by
/// pasting [`SlintStyle::snippet`]'s bindings into `.slint`, or per state
/// through [`StyleBook::to_states_snippet`]. Typography ([`SlintText`])
/// belongs on the `Text` item, padding on the enclosing layout.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SlintStyle {
    /// `bg-{color}` — the solid fill brush. When a gradient is also
    /// present, CSS layers the image OVER the color while Slint's single
    /// `background` brush takes the gradient; emit the solid on an outer
    /// Rectangle if you need the full CSS layering (the coexistence is
    /// reported, never dropped silently).
    pub background: Option<Brush>,
    /// The composed `background-image` gradient (`bg-linear-*`, `bg-radial`,
    /// `bg-conic*`) as a native [`Brush`] gradient. `bg-none` clears this
    /// slot on overlay.
    pub background_gradient: Option<Brush>,
    /// `text-{color}` and the rest of the typography surface — set on the
    /// `Text` item.
    pub text: SlintText,
    /// `border-*` width, uniform (`border-width`). Per-edge utilities
    /// flatten onto the top edge with a report, as in the egui backend.
    pub border_width: Option<f32>,
    /// `border-{color}` — `border-color`.
    pub border_color: Option<Brush>,
    /// `rounded-*` — per-corner native (`border-top-left-radius` …),
    /// exactly the four corners of [`aeolus_core::plan::CornerRadius`].
    pub border_radius: Option<CornerRadiusPx>,
    /// `shadow-*` — the first preset layer as a drop shadow
    /// (`drop-shadow-blur/offset-x/offset-y/spread/color`).
    pub drop_shadow: Option<ShadowPx>,
    /// `inset-shadow-*` / `shadow-inner` — the inset surface
    /// (`inner-shadow-blur/offset-x/offset-y/spread/color`).
    pub inner_shadow: Option<ShadowPx>,
    /// `opacity-*` — the `opacity` property (0..=1).
    pub opacity: Option<f32>,
    /// `overflow-hidden` → `clip: true` on the Rectangle.
    pub clip: Option<bool>,
    /// `invisible` → `visibility: hidden` (v4's `visible`/`invisible`
    /// utilities; `collapse` is reported — Slint's `Visibility` has no
    /// collapse slot on plain elements).
    pub hidden: Option<bool>,
    /// `cursor-*` — `mouse-cursor` on a `TouchArea`.
    pub mouse_cursor: Option<BuiltInMouseCursor>,
    /// `pointer-events-none` → `Some(false)` — wire into `TouchArea::
    /// enabled` (or the widget's `enabled`).
    pub enabled: Option<bool>,
    /// `w-*` / `h-*` — `width`/`height` lengths.
    pub width: Option<SlintLength>,
    pub height: Option<SlintLength>,
    /// `min-w-*` / `min-h-*` — `min-width`/`min-height` lengths.
    pub min_width: Option<SlintLength>,
    pub min_height: Option<SlintLength>,
    /// `max-w-*` / `max-h-*` — `max-width`/`max-height` lengths.
    pub max_width: Option<SlintLength>,
    pub max_height: Option<SlintLength>,
    /// `p-*` family — the CSS padding, carried as a layout hint: Slint
    /// keeps padding on the LAYOUT elements (`HorizontalLayout`/
    /// `VerticalLayout`), so the snippet emits `padding-*` bindings for
    /// the enclosing layout, not the Rectangle.
    pub padding: Option<PaddingPx>,
    /// `bg-none` was seen in this layer: overlay must clear the
    /// accumulated gradient (`background-image: none` semantics).
    gradient_cleared: bool,
}

/// Per-corner radii in px — the four corners of Slint's Rectangle
/// (`border-top-left-radius` …), matching [`aeolus_core::plan::CornerRadius`].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CornerRadiusPx {
    /// Top-left corner.
    pub top_left: f32,
    /// Top-right corner.
    pub top_right: f32,
    /// Bottom-right corner.
    pub bottom_right: f32,
    /// Bottom-left corner.
    pub bottom_left: f32,
}

/// One shadow layer in px — Slint's `drop-shadow-*` / `inner-shadow-*`
/// surface carries blur, both offsets, spread AND color, the full CSS box
/// shadow of one layer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShadowPx {
    /// Horizontal offset in px (positive = right).
    pub offset_x: f32,
    /// Vertical offset in px (positive = down).
    pub offset_y: f32,
    /// Blur radius in px.
    pub blur: f32,
    /// Spread in px (may be negative).
    pub spread: f32,
    /// Shadow color (alpha carried here).
    pub color: Color,
}

/// The CSS padding in px, per edge — a layout hint (see
/// [`SlintStyle::padding`]).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PaddingPx {
    /// Top edge in px.
    pub top: f32,
    /// Right edge in px.
    pub right: f32,
    /// Bottom edge in px.
    pub bottom: f32,
    /// Left edge in px.
    pub left: f32,
}

impl SlintStyle {
    /// Overlay `other` on top of `self` (later layers win per slot),
    /// mirroring [`ResolvedStyle::overlay`]'s Some-replaces semantics.
    pub fn overlay(&mut self, other: &SlintStyle) {
        if other.background.is_some() {
            self.background.clone_from(&other.background);
        }
        // `bg-none` clears the image slot exactly like CSS
        // `background-image: none` clears the composed layers.
        if other.gradient_cleared {
            self.background_gradient = None;
        }
        if other.background_gradient.is_some() {
            self.background_gradient
                .clone_from(&other.background_gradient);
        }
        self.text.overlay(&other.text);
        if other.border_width.is_some() {
            self.border_width = other.border_width;
        }
        if other.border_color.is_some() {
            self.border_color.clone_from(&other.border_color);
        }
        if other.border_radius.is_some() {
            self.border_radius = other.border_radius;
        }
        if other.drop_shadow.is_some() {
            self.drop_shadow = other.drop_shadow;
        }
        if other.inner_shadow.is_some() {
            self.inner_shadow = other.inner_shadow;
        }
        if other.opacity.is_some() {
            self.opacity = other.opacity;
        }
        if other.clip.is_some() {
            self.clip = other.clip;
        }
        if other.hidden.is_some() {
            self.hidden = other.hidden;
        }
        if other.mouse_cursor.is_some() {
            self.mouse_cursor = other.mouse_cursor;
        }
        if other.enabled.is_some() {
            self.enabled = other.enabled;
        }
        if other.width.is_some() {
            self.width = other.width;
        }
        if other.height.is_some() {
            self.height = other.height;
        }
        if other.min_width.is_some() {
            self.min_width = other.min_width;
        }
        if other.min_height.is_some() {
            self.min_height = other.min_height;
        }
        if other.max_width.is_some() {
            self.max_width = other.max_width;
        }
        if other.max_height.is_some() {
            self.max_height = other.max_height;
        }
        if other.padding.is_some() {
            self.padding = other.padding;
        }
    }

    /// The element property bindings, one `name: value;` per line — the
    /// `.slint` payload of this style.
    ///
    /// Typography lines belong on the `Text` item
    /// ([`SlintText::snippet`]) and padding lines on the enclosing
    /// layout; the block is sorted in property-declaration order
    /// (paint → border → shadow → geometry → state).
    pub fn snippet(&self) -> String {
        let mut lines = Vec::new();
        if let Some(b) = self
            .background_gradient
            .as_ref()
            .or(self.background.as_ref())
        {
            lines.push(format!("background: {};", brush_snippet(b)));
        }
        if let Some(r) = &self.border_radius {
            // Uniform case → the single shorthand; per-corner values →
            // the four corner properties verbatim.
            if r.top_left == r.top_right
                && r.top_right == r.bottom_right
                && r.bottom_right == r.bottom_left
            {
                lines.push(format!("border-radius: {}px;", fmt_f32(r.top_left)));
            } else {
                lines.push(format!(
                    "border-top-left-radius: {}px;\nborder-top-right-radius: {}px;\nborder-bottom-right-radius: {}px;\nborder-bottom-left-radius: {}px;",
                    fmt_f32(r.top_left),
                    fmt_f32(r.top_right),
                    fmt_f32(r.bottom_right),
                    fmt_f32(r.bottom_left),
                ));
            }
        }
        if let Some(w) = self.border_width {
            lines.push(format!("border-width: {}px;", fmt_f32(w)));
        }
        if let Some(c) = &self.border_color {
            lines.push(format!("border-color: {};", brush_snippet(c)));
        }
        if let Some(s) = &self.drop_shadow {
            lines.push(format!(
                "drop-shadow-offset-x: {}px;\ndrop-shadow-offset-y: {}px;\ndrop-shadow-blur: {}px;\ndrop-shadow-spread: {}px;\ndrop-shadow-color: {};",
                fmt_f32(s.offset_x),
                fmt_f32(s.offset_y),
                fmt_f32(s.blur),
                fmt_f32(s.spread),
                slint_color_snippet(s.color),
            ));
        }
        if let Some(s) = &self.inner_shadow {
            lines.push(format!(
                "inner-shadow-offset-x: {}px;\ninner-shadow-offset-y: {}px;\ninner-shadow-blur: {}px;\ninner-shadow-spread: {}px;\ninner-shadow-color: {};",
                fmt_f32(s.offset_x),
                fmt_f32(s.offset_y),
                fmt_f32(s.blur),
                fmt_f32(s.spread),
                slint_color_snippet(s.color),
            ));
        }
        if let Some(o) = self.opacity {
            lines.push(format!("opacity: {};", fmt_f32(o)));
        }
        if let Some(clip) = self.clip {
            lines.push(format!("clip: {clip};"));
        }
        if self.hidden == Some(true) {
            // The DSL property is the plain bool  (slint 1.18's
            // reserved common properties: clip, opacity, visible).
            lines.push("visible: false;".to_string());
        }
        if let Some(cursor) = self.mouse_cursor {
            lines.push(format!("mouse-cursor: {cursor};"));
        }
        if let Some(w) = self.width {
            lines.push(format!("width: {};", w.snippet()));
        }
        if let Some(h) = self.height {
            lines.push(format!("height: {};", h.snippet()));
        }
        if let Some(w) = self.min_width {
            lines.push(format!("min-width: {};", w.snippet()));
        }
        if let Some(h) = self.min_height {
            lines.push(format!("min-height: {};", h.snippet()));
        }
        if let Some(w) = self.max_width {
            lines.push(format!("max-width: {};", w.snippet()));
        }
        if let Some(h) = self.max_height {
            lines.push(format!("max-height: {};", h.snippet()));
        }
        if let Some(p) = &self.padding {
            if p.top == p.right && p.right == p.bottom && p.bottom == p.left {
                lines.push(format!("padding: {}px;", fmt_f32(p.top)));
            } else {
                lines.push(format!(
                    "padding-top: {}px;\npadding-right: {}px;\npadding-bottom: {}px;\npadding-left: {}px;",
                    fmt_f32(p.top),
                    fmt_f32(p.right),
                    fmt_f32(p.bottom),
                    fmt_f32(p.left),
                ));
            }
        }
        // Typography is NOT part of the element block: Slint's `Text`
        // item carries those properties ([`SlintText::snippet`]). The
        // interpreter-validated tests pin that separation.
        lines.join("\n")
    }
}

// ---------------------------------------------------------------------------
// ResolvedStyle → SlintStyle
// ---------------------------------------------------------------------------

/// The full v4 cursor keyword set → Slint's `BuiltInMouseCursor`
/// (kebab-case `Display` matches the `.slint` values). The keywords Slint
/// does not carry (`zoom-in/out`, `context-menu`, `cell`,
/// `vertical-text`, `all-scroll`) are reported by the caller.
fn to_builtin_cursor(c: aeolus_core::plan::Cursor) -> Option<BuiltInMouseCursor> {
    use BuiltInMouseCursor as B;
    use aeolus_core::plan::Cursor as C;
    Some(match c {
        // CSS `auto` = the user agent's default cursor.
        C::Auto => B::Default,
        C::Default => B::Default,
        C::None => B::None,
        C::ContextMenu => return None,
        C::Progress => B::Progress,
        C::Wait => B::Wait,
        C::Cell => return None,
        C::Crosshair => B::Crosshair,
        C::VerticalText => return None,
        C::Alias => B::Alias,
        C::Copy => B::Copy,
        C::NoDrop => B::NoDrop,
        C::NotAllowed => B::NotAllowed,
        C::Grab => B::Grab,
        C::Grabbing => B::Grabbing,
        C::AllScroll => return None,
        C::ColResize => B::ColResize,
        C::RowResize => B::RowResize,
        C::NResize => B::NResize,
        C::EResize => B::EResize,
        C::SResize => B::SResize,
        C::WResize => B::WResize,
        C::NeResize => B::NeResize,
        C::NwResize => B::NwResize,
        C::SeResize => B::SeResize,
        C::SwResize => B::SwResize,
        C::EwResize => B::EwResize,
        C::NsResize => B::NsResize,
        C::NeswResize => B::NeswResize,
        C::NwseResize => B::NwseResize,
        C::ZoomIn | C::ZoomOut => return None,
        C::Help => B::Help,
        C::Pointer => B::Pointer,
        C::Text => B::Text,
        C::Move => B::Move,
    })
}

/// Lower one resolved style into a [`SlintStyle`].
///
/// Slots Slint cannot express (ring, flex/grid layout, transforms,
/// transitions, margins…) are recorded as dropped — they are
/// capability-negotiated, never silently ignored. Slint 1.18's Rectangle
/// is otherwise the strongest surface of the backends: gradients, per-corner
/// radii and drop + inner shadows with spread all lower natively.
pub fn lower_resolved(style: &ResolvedStyle, report: &mut LoweringReport) -> SlintStyle {
    let mut out = SlintStyle::default();

    // Background: solid + gradient coexist in CSS semantics; Slint's single
    // `background` brush takes the gradient preferentially (the plan's
    // documented normalization for single-brush backends) while BOTH stay
    // on the struct so the app can emit the solid underneath.
    if let Some(bg) = &style.background {
        out.background = Some(Brush::SolidColor(to_slint_color(bg.0)));
    }
    if let Some(gradient) = &style.background_gradient {
        match gradient {
            Gradient::None => {
                out.gradient_cleared = true;
            }
            _ => {
                out.background_gradient = gradient_to_brush(gradient);
                if style.background.is_some() {
                    report.drop(
                        "background-color under gradient",
                        "CSS layers background-image over background-color; Slint's single \
                         `background` brush takes the gradient — emit the solid on an outer \
                         Rectangle if you need the CSS layering",
                    );
                }
            }
        }
    }

    // Text/icon color + the typography surface.
    lower_text(style, &mut out.text, report);

    // Border: uniform width; per-side utilities flatten to the top edge.
    if let Some(bw) = &style.border_width {
        if !(bw.top == bw.right && bw.right == bw.bottom && bw.bottom == bw.left) {
            report.drop(
                "border-width:per-side",
                "Rectangle carries a single border-width; using the top edge",
            );
        }
        if bw.top > 0.0 {
            out.border_width = Some(bw.top);
        }
    }
    if let Some(bc) = &style.border_color {
        out.border_color = Some(Brush::SolidColor(to_slint_color(bc.0)));
    }
    // Border style: slint paints solid borders only.
    if let Some(bs) = style.extras.border_style {
        report.drop(
            "border-style",
            format!("{bs:?} borders are not paintable; Rectangle borders are solid"),
        );
    }

    // Corner radius: per-corner native (border-top-left-radius …).
    if let Some(cr) = &style.corner_radius {
        out.border_radius = Some(CornerRadiusPx {
            top_left: cr.top_left,
            top_right: cr.top_right,
            bottom_right: cr.bottom_right,
            bottom_left: cr.bottom_left,
        });
    }

    // Outline: take the border slots when they are free (the egui ring→
    // outline precedent); otherwise report the collision.
    if let Some(o) = &style.outline
        && !o.none
    {
        if out.border_width.is_none() && out.border_color.is_none() {
            out.border_width = Some(o.width);
            out.border_color = o.color.map(|c| Brush::SolidColor(to_slint_color(c)));
        } else {
            report.drop(
                "outline",
                "border-* already uses the border slots; Slint has no second border",
            );
        }
    }

    // Ring: no primitive; the .slint emulation is a nested Rectangle.
    if style.ring.is_some() {
        report.drop(
            "ring",
            "no ring primitive; emulate with a nested Rectangle inset by the ring width",
        );
    }

    // Shadows: drop-shadow-* (with spread!) and inner-shadow-* are BOTH
    // native; multi-layer presets report their extra layers.
    if let Some(s) = &style.shadow {
        let shadow = ShadowPx {
            offset_x: s.offset_x,
            offset_y: s.offset_y,
            blur: s.blur,
            spread: s.spread,
            color: to_slint_color(s.color),
        };
        if s.inset {
            out.inner_shadow = Some(shadow);
        } else {
            out.drop_shadow = Some(shadow);
        }
    }
    if !style.extras.shadow_rest.is_empty() {
        report.drop(
            "shadow layers 2+",
            "Rectangle paints a single drop-shadow; extra preset layers reported",
        );
    }
    if !style.extras.inset_shadows.is_empty() {
        let first = &style.extras.inset_shadows[0];
        out.inner_shadow = Some(ShadowPx {
            offset_x: first.offset_x,
            offset_y: first.offset_y,
            blur: first.blur,
            spread: first.spread,
            color: to_slint_color(first.color),
        });
        if style.extras.inset_shadows.len() > 1 {
            report.drop(
                "inset-shadow layers 2+",
                "Rectangle paints a single inner-shadow; extra preset layers reported",
            );
        }
    }

    // Opacity: a real per-element property (0..=1).
    if let Some(o) = &style.opacity {
        out.opacity = Some(o.0.clamp(0.0, 1.0));
    }

    // Overflow: hidden → clip (both axes hidden); mixed axes report.
    if let Some(ov) = &style.extras.overflow {
        match (ov.x, ov.y) {
            (
                Some(aeolus_core::plan::OverflowKind::Hidden),
                Some(aeolus_core::plan::OverflowKind::Hidden),
            ) => {
                out.clip = Some(true);
            }
            (None, Some(aeolus_core::plan::OverflowKind::Hidden))
            | (Some(aeolus_core::plan::OverflowKind::Hidden), None) => {
                report.drop(
                    "overflow:one-axis",
                    "Rectangle clips both axes or none (clip); per-axis overflow reported",
                );
            }
            (None, None) => {}
            _ => report.drop(
                "overflow",
                "only overflow-hidden maps natively (clip: true); scroll/auto need \
                 Flickable in .slint",
            ),
        }
    }

    // Visibility: invisible → visibility: hidden.
    if style.extras.visibility == Some(aeolus_core::plan::Visibility::Hidden) {
        out.hidden = Some(true);
    }

    // Cursor.
    if let Some(c) = &style.cursor {
        match to_builtin_cursor(*c) {
            Some(b) => out.mouse_cursor = Some(b),
            None => report.drop(
                "cursor",
                format!("{c:?} has no BuiltInMouseCursor variant in Slint 1.18"),
            ),
        }
    }

    // Pointer events → the enabled hint.
    if let Some(pe) = &style.pointer_events {
        out.enabled = Some(matches!(pe, aeolus_core::plan::PointerEvents::Auto));
    }

    // Geometry: fixed/min/max sizes. Percentages and full lower to
    // SlintLength::Percent; fit/viewport/auto are handled per value.
    if let Some(d) = &style.dimensions {
        if let Some(w) = d.width {
            out.width = slint_length(w, "w", report);
        }
        if let Some(h) = d.height {
            out.height = slint_length(h, "h", report);
        }
    }
    if let Some(d) = &style.min_dimensions {
        if let Some(w) = d.width {
            out.min_width = slint_length(w, "min-w", report);
        }
        if let Some(h) = d.height {
            out.min_height = slint_length(h, "min-h", report);
        }
    }
    if let Some(d) = &style.max_dimensions {
        if let Some(w) = d.width {
            out.max_width = slint_length(w, "max-w", report);
        }
        if let Some(h) = d.height {
            out.max_height = slint_length(h, "max-h", report);
        }
    }

    // Padding: a layout hint (layout elements carry padding-*).
    if let Some(p) = &style.padding {
        out.padding = Some(PaddingPx {
            top: p.top,
            right: p.right,
            bottom: p.bottom,
            left: p.left,
        });
    }

    // Everything Slint handles through .slint structure rather than
    // per-element properties — reported, never silently dropped.
    report_extras(style, &mut out, report);

    out
}

/// The typography slots: text color, size, weight, spacing, family,
/// italic, wrap, overflow and horizontal alignment lower natively; the
/// rest is reported with the reason the slot is out of reach.
fn lower_text(style: &ResolvedStyle, out: &mut SlintText, report: &mut LoweringReport) {
    for (prop, value) in &style.text {
        match (prop, value) {
            (TextProperty::Color, TextValue::Color(c)) => {
                out.color = Some(Brush::SolidColor(to_slint_color(*c)));
            }
            (TextProperty::FontSize, TextValue::Px(px)) => {
                out.font_size = Some(*px);
            }
            (TextProperty::FontWeight, TextValue::Weight(w)) => {
                out.font_weight = Some(*w as i32);
            }
            (TextProperty::LetterSpacing, TextValue::Spacing(px)) => {
                out.letter_spacing = Some(*px);
            }
            (TextProperty::LetterSpacing, TextValue::TrackingEm(em)) => {
                report.drop(
                    "tracking (em)",
                    format!(
                        "letter-spacing is a length on Text; multiply the em value \
                         ({em}) by the resolved font-size at the call site"
                    ),
                );
            }
            (TextProperty::FontFamily, TextValue::Keyword(family)) => {
                // `sans` IS Slint's default font; the other generic
                // families need an installed font under that name.
                if *family != "sans" {
                    out.font_family = Some((*family).to_string());
                    report.drop(
                        format!("font-{family}"),
                        "set as the font-family string; load a font with that family \
                         name or the property falls back to the default face",
                    );
                }
            }
            (TextProperty::FontStyle, TextValue::Keyword("italic")) => {
                out.italic = Some(true);
            }
            (TextProperty::FontStyle, TextValue::Keyword("not-italic")) => {
                out.italic = Some(false);
            }
            (TextProperty::Whitespace, TextValue::Keyword("nowrap")) => {
                out.wrap = Some(TextWrap::NoWrap);
            }
            (TextProperty::Whitespace, TextValue::Keyword("normal")) => {
                out.wrap = Some(TextWrap::WordWrap);
            }
            (TextProperty::TextOverflow, TextValue::Keyword("ellipsis")) => {
                out.overflow = Some(TextOverflow::Elide);
            }
            (TextProperty::TextOverflow, TextValue::Keyword("clip")) => {
                out.overflow = Some(TextOverflow::Clip);
            }
            (TextProperty::TextAlign, TextValue::Keyword(kind)) => match *kind {
                "left" => out.horizontal_alignment = Some(TextHorizontalAlignment::Left),
                "center" => out.horizontal_alignment = Some(TextHorizontalAlignment::Center),
                "right" => out.horizontal_alignment = Some(TextHorizontalAlignment::Right),
                _ => report.drop(
                    format!("text-{kind}"),
                    "only left/center/right map to TextHorizontalAlignment",
                ),
            },
            (prop, value) => report.drop(
                format!("typography:{prop:?}"),
                format!("no {prop:?} property on Slint Text ({value:?})"),
            ),
        }
    }
}

/// The structure-level slots Slint expresses through `.slint` code layout
/// structure (or not at all): each gets an honest per-family reason.
fn report_extras(style: &ResolvedStyle, _out: &mut SlintStyle, report: &mut LoweringReport) {
    if style.layout.is_some() {
        report.drop(
            "layout",
            "flex/grid layout is .slint structure (HorizontalLayout/VerticalLayout/\
             GridLayout), not a per-element property",
        );
    }
    if style.gap.is_some() || style.extras.gap_axes.is_some() {
        report.drop(
            "gap",
            "layout `spacing` on Horizontal/VerticalLayout, not a per-element property",
        );
    }
    if style.margin.is_some() {
        report.drop(
            "margin",
            "no margin property; space elements with layout spacing",
        );
    }
    if style.extras.margin_auto.is_some() {
        report.drop("margin-auto", "no auto margins in Slint layout");
    }
    if style.extras.position.is_some() {
        report.drop(
            "position",
            "no absolute-positioning mode; place elements with layout structure",
        );
    }
    if style.extras.inset.is_some() {
        report.drop(
            "inset",
            "no top/right/bottom/left offsets on plain elements",
        );
    }
    if style.extras.z_index.is_some() {
        report.drop("z-index", "stacking is tree order in Slint, not a z slot");
    }
    if style.extras.aspect_ratio.is_some() {
        report.drop(
            "aspect-*",
            "no aspect-ratio property; constrain width and height bindings yourself",
        );
    }
    if style.extras.grid.is_some() {
        report.drop(
            "grid-*",
            "grid templates are GridLayout structure in .slint, not per-element properties",
        );
    }
    if style.extras.transform.is_some() {
        report.drop(
            "transform",
            "no transform slot on Rectangle; rotate/translate need dedicated \
             elements (rotation-angle on Image and friends)",
        );
    }
    if style.extras.transition.is_some() {
        report.drop(
            "transition-*",
            "Slint animates via `animate` declarations and states in/out — \
             generate them at the call site from the duration/ease values",
        );
    }
    if style.extras.animation.is_some() {
        report.drop(
            "animate-*",
            "keyframe animations need .slint `animations` blocks; reported, not faked",
        );
    }
    if style.extras.flex_basis.is_some() {
        report.drop(
            "basis-*",
            "flex-basis is layout structure, not a style property",
        );
    }
    if style.extras.order.is_some() {
        report.drop("order-*", "no flex order; reorder the .slint children");
    }
    if style.extras.align_content.is_some() {
        report.drop(
            "content-*",
            "align-content is layout structure, not a style property",
        );
    }
}

// ---------------------------------------------------------------------------
// The book: base + conditional layers, resolved per situation
// ---------------------------------------------------------------------------

/// The lowered [`StylePlan`]: base + rank-sorted conditional layers + dark.
///
/// Slint's state restyling is authored with `states [ name when condition :
/// { … } ]` blocks, so the book keeps EVERY state layer natively — hover,
/// pressed, disabled, focused are all conditions a `TouchArea` exposes as
/// plain properties (`has-hover`, `pressed`, `enabled`, `has-focus`).
/// [`StyleBook::to_states_snippet`] generates that block;
/// [`StyleBook::resolve_for`] resolves the same layers in Rust for apps
/// that push properties programmatically.
#[derive(Clone, Debug, PartialEq)]
pub struct StyleBook {
    /// Unconditional properties.
    pub base: SlintStyle,
    /// Conditional layers, ascending [`VariantCondition::rank`] order.
    pub layers: Vec<(VariantCondition, SlintStyle)>,
    /// `dark:` override — resolve against the app's color-scheme state.
    pub dark: Option<SlintStyle>,
    /// The original plan, kept for the [`StyleBook::resolve_at`] parity
    /// path (group/peer, compound rules, responsive and container
    /// conditions evaluated by the core evaluator).
    plan: StylePlan,
}

impl StyleBook {
    /// Lower a full [`StylePlan`].
    ///
    /// State layers (hover/active/disabled/focus) lower natively — Slint
    /// `states` cover them all. Group/peer, compound rules, responsive and
    /// container conditions are reported as emulated: no selector engine,
    /// they resolve through [`StyleBook::resolve_at`] with a probe the app
    /// feeds (the generated-code path's limit, per the capability audit).
    pub fn new(plan: &StylePlan) -> (Self, LoweringReport) {
        let mut report = LoweringReport::default();

        for (group_id, _) in &plan.group {
            report.drop(
                format!("group:{group_id}"),
                "answered by resolve_at's probe path, not by a `states` condition",
            );
        }
        for (peer_id, _) in &plan.peer {
            report.drop(
                format!("peer:{peer_id}"),
                "answered by resolve_at's probe path, not by a `states` condition",
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
                "width conditions are hand-written `states when width …` in .slint — \
                 resolve_at evaluates them via SlintProbe::width",
            );
        }
        for bp in plan.container.keys() {
            report.drop(
                format!("container:{bp:?}"),
                "container width is knowable per frame — resolve_at evaluates it via \
                 SlintProbe::container_width",
            );
        }

        let base = lower_resolved(&plan.base, &mut report);
        let mut sorted: Vec<&aeolus_core::plan::ConditionalLayer> = plan.layers.iter().collect();
        sorted.sort_by_key(|l| l.condition.rank());
        let mut layers = Vec::with_capacity(sorted.len());
        for layer in sorted {
            // v0.8 §1.2: state blocks re-bind properties wholesale, so a
            // partial-edge layer (`hover:px-8`) is materialized against the
            // base first — its masked-off edges inherit the base's values
            // instead of re-binding the 0.0 placeholders.
            let mut style = layer.style.clone();
            style.materialize_unset_edges_from(&plan.base);
            layers.push((layer.condition, lower_resolved(&style, &mut report)));
        }
        let dark = plan.dark.as_ref().map(|d| {
            let mut style = d.clone();
            style.materialize_unset_edges_from(&plan.base);
            lower_resolved(&style, &mut report)
        });

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
                    tracing::warn!("aeolus-slint: dropped {:?}", report.dropped);
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

    /// Merge base + dark for one color scheme without state conditions.
    fn merged_for_scheme(&self, dark: bool) -> SlintStyle {
        let mut style = self.base.clone();
        if dark && let Some(d) = &self.dark {
            style.overlay(d);
        }
        style
    }

    /// Resolve for the element's current state — the probe's flags stand
    /// in for the `TouchArea`/widget properties (`has-hover`, `pressed`,
    /// `enabled`, `has-focus`) and the app's color scheme.
    ///
    /// Conditions apply in ascending [`VariantCondition::rank`] order, so a
    /// focused+hovered element resolves the FOCUS layer over hover
    /// (Tailwind v4's cascade intent, same as the other backends).
    pub fn resolve_for(&self, probe: &SlintProbe) -> SlintStyle {
        let mut style = self.merged_for_scheme(probe.dark);
        for (condition, layer) in &self.layers {
            let fires = match condition {
                VariantCondition::Hover => probe.has_hover,
                VariantCondition::Active => probe.pressed,
                VariantCondition::Disabled => !probe.enabled,
                VariantCondition::Focus | VariantCondition::FocusVisible => probe.has_focus,
                VariantCondition::FocusWithin | VariantCondition::Visited => false,
            };
            if fires {
                style.overlay(layer);
            }
        }
        style
    }

    /// The FULL parity path (plan §4.2): flatten the original plan for the
    /// situation the probe describes — group/peer, data/aria, compound
    /// variants, responsive and container conditions included — and lower
    /// the result. Call it whenever the app's state changes; the property
    /// graph re-evaluates on write.
    pub fn resolve_at(&self, probe: &impl Probe) -> SlintStyle {
        let resolved = self.plan.resolve(probe);
        let mut report = LoweringReport::default();
        lower_resolved(&resolved, &mut report)
    }

    /// The generated `states [ … ]` block for this book: one named state
    /// per conditional layer, gated on the named `TouchArea`'s properties
    /// (`ta.has-hover`, `ta.pressed`, `!ta.enabled`).
    ///
    /// ```text
    /// states [
    ///     hovered when ta.has-hover : {
    ///         background: #18181B;
    ///     }
    /// ]
    /// ```
    ///
    /// Paste it into the element that also declares the `TouchArea` (named
    /// `ta` here — pass your own name). Keyboard focus is NOT gated here:
    /// `has-focus` lives on a `FocusScope`, not a `TouchArea` — use
    /// [`StyleBook::to_states_snippet_focused`] when the element declares
    /// one. The `dark:` override resolves in Rust via
    /// [`StyleBook::resolve_for`] against the app's color scheme;
    /// generating a `color-scheme` condition would couple the snippet to a
    /// specific widget style's globals.
    pub fn to_states_snippet(&self, touch_area: &str) -> String {
        self.states_block(touch_area, None)
    }

    /// [`StyleBook::to_states_snippet`] plus the `focused` state, gated on
    /// the named `FocusScope`'s `has-focus` property (`has-focus` is a
    /// `FocusScope` property in Slint 1.18, not a `TouchArea` one).
    pub fn to_states_snippet_focused(&self, touch_area: &str, focus_scope: &str) -> String {
        self.states_block(touch_area, Some(focus_scope))
    }

    /// The shared states generator; `focus_scope: Some(name)` adds the
    /// `focused when <name>.has-focus` state.
    fn states_block(&self, touch_area: &str, focus_scope: Option<&str>) -> String {
        let mut blocks = Vec::new();
        for (condition, layer) in &self.layers {
            let (name, gate) = match condition {
                VariantCondition::Hover => ("hovered", format!("{touch_area}.has-hover")),
                VariantCondition::Active => ("pressed", format!("{touch_area}.pressed")),
                VariantCondition::Disabled => ("disabled", format!("!{touch_area}.enabled")),
                VariantCondition::Focus | VariantCondition::FocusVisible => {
                    match focus_scope {
                        Some(fs) => ("focused", format!("{fs}.has-focus")),
                        // No FocusScope named → no honest gate to emit.
                        None => continue,
                    }
                }
                VariantCondition::FocusWithin | VariantCondition::Visited => continue,
            };
            let body = layer.snippet();
            if body.is_empty() {
                continue;
            }
            let mut indented = String::new();
            for l in body.lines() {
                let _ = writeln!(indented, "        {l}");
            }
            blocks.push(format!("    {name} when {gate} : {{\n{indented}    }}"));
        }
        if blocks.is_empty() {
            return String::new();
        }
        format!("states [\n{}\n]", blocks.join("\n"))
    }

    /// The base layer's element bindings ([`SlintStyle::snippet`]) — the
    /// unconditional half of the generated code.
    pub fn base_snippet(&self) -> String {
        self.base.snippet()
    }
}

// ---------------------------------------------------------------------------
// The parity probe
// ---------------------------------------------------------------------------

/// The element's current situation as Slint state flags — construct from
/// the `TouchArea`/widget properties the app reads (`has-hover`,
/// `pressed`, `enabled`, `has-focus`) and the color scheme.
///
/// Every flag defaults to its "no" answer except [`SlintProbe::enabled`]
/// (elements are enabled unless told otherwise), so
/// `SlintProbe::default()` is a plain idle element.
#[derive(Clone, Debug, PartialEq)]
pub struct SlintProbe {
    /// `hover:` — `TouchArea.has-hover`.
    pub has_hover: bool,
    /// `active:` — `TouchArea.pressed`.
    pub pressed: bool,
    /// The element is enabled (`disabled:` fires when this is `false`).
    pub enabled: bool,
    /// `focus:` / `focus-visible:` — `has-focus`.
    pub has_focus: bool,
    /// `checked:` — the view's checked state.
    pub checked: bool,
    /// `dark:` — the app's color scheme (`Palette.color-scheme`).
    pub dark: bool,
    /// Current window width in px, when known (`sm:`/`md:`/… conditions).
    pub width: Option<f32>,
    /// Current container width, when this element is inside a container
    /// the app tracks.
    pub container_width: Option<f32>,
}

impl Default for SlintProbe {
    fn default() -> Self {
        Self {
            has_hover: false,
            pressed: false,
            enabled: true,
            has_focus: false,
            checked: false,
            dark: false,
            width: None,
            container_width: None,
        }
    }
}

impl Probe for SlintProbe {
    fn test(&self, t: &aeolus_core::Test) -> bool {
        use aeolus_core::{State, Test};
        match t {
            Test::State(s) => match s {
                State::Hover => self.has_hover,
                State::Focus | State::FocusVisible => self.has_focus,
                State::Active => self.pressed,
                State::Disabled => !self.enabled,
                State::Checked => self.checked,
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
// The slot coverage ledger (verification tooling, like the other backends)
// ---------------------------------------------------------------------------

/// How completely one [`ResolvedStyle`] mapped onto Slint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Coverage {
    /// Mapped to a native Slint element property (Rectangle paint slots,
    /// lengths, `opacity`, typography on `Text`).
    Native,
    /// Carried as a hint (padding → the enclosing layout) or emulated
    /// inside the lowering (outline → the free border slots).
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
    if s.background_gradient.is_some() {
        // Gradients are native Slint brushes (`@linear-gradient` …); the
        // solid-under-gradient coexistence is reported but the image slot
        // itself is expressed.
        v.push((S::Gradient(GradientPart::Shape), Coverage::Native));
    }
    if s.content_color.is_some() {
        v.push((S::ContentColor, Coverage::Native));
    }
    if s.padding.is_some() {
        v.push((S::Padding, Coverage::Emulated));
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
        v.push((S::MaxDimensions, Coverage::Native));
    }
    if s.gap.is_some() || s.extras.gap_axes.is_some() {
        v.push((S::Gap, Coverage::Reported));
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
        v.push((S::Opacity, Coverage::Native));
    }
    if s.ring.is_some() {
        v.push((S::Ring(aeolus_core::RingPart::Width), Coverage::Reported));
    }
    if s.outline.is_some() {
        v.push((S::Outline, Coverage::Emulated));
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
            | TextProperty::LetterSpacing
            | TextProperty::FontFamily
            | TextProperty::FontStyle
            | TextProperty::Whitespace
            | TextProperty::TextOverflow
            | TextProperty::TextAlign
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
        v.push((S::Overflow, Coverage::Emulated));
    }
    if e.visibility.is_some() {
        v.push((S::Visibility, Coverage::Native));
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
        v.push((S::BorderStyle, Coverage::Reported));
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
    if !e.shadow_rest.is_empty() {
        v.push((S::ShadowRest, Coverage::Reported));
    }
    if !e.inset_shadows.is_empty() {
        v.push((S::InsetShadows, Coverage::Native));
    }
    v
}
