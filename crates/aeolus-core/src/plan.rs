//! The `StylePlan` IR — Aeolus's public intermediate representation.
//!
//! A `StylePlan` is *not* a flat resolved style: it carries a `base` layer plus
//! conditional layers, so each backend can lower conditions into its native
//! cascade (`PropertyStack`+`Selector` on Masonry, nested selector maps on
//! Floem, state-keyed closures on Iced) instead of re-implementing variants.
//!
//! v0.2 (plan §2.1) adds — **additively**, so v0.1 backends keep compiling:
//! * [`StylePlan::rules`] — conditional [`Rule`]s for everything the buckets
//!   cannot express (compound variants, group/peer, data/aria, has/not,
//!   structural, container queries), evaluated by the [`crate::eval::Probe`]
//!   parity engine;
//! * [`StylePlan::breakpoints`] — the theme-aware breakpoint px table;
//! * [`ResolvedStyle::extras`] — the v0.2 typed slots (position, inset,
//!   z-index, overflow, grid, transform, transition, animation, ...).

use crate::ast::Breakpoint;
use crate::color::Color;
use crate::cond::Rule;
use std::collections::BTreeMap;

/// The complete styling plan for one element.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StylePlan {
    /// Unconditional properties.
    pub base: ResolvedStyle,
    /// Conditional layers (`hover:bg-*`, `focus-visible:ring-*`, ...).
    pub layers: Vec<ConditionalLayer>,
    /// `sm:`..`2xl:` responsive overrides.
    pub responsive: BTreeMap<Breakpoint, ResolvedStyle>,
    /// `@sm:`.. container-query overrides (v0.1 legacy bucket; v0.2 routes
    /// container queries into `rules` — this stays for backend compat and is
    /// only populated when a backend opts into bucket-mode routing).
    pub container: BTreeMap<Breakpoint, ResolvedStyle>,
    /// `dark:` overrides.
    pub dark: Option<ResolvedStyle>,
    /// `group-*:` layers keyed by group label (empty label = default group).
    pub group: Vec<(GroupId, ResolvedStyle)>,
    /// `peer-*:` layers keyed by peer label.
    pub peer: Vec<(PeerId, ResolvedStyle)>,
    /// v0.2: conditional rules for everything the buckets cannot express.
    /// A token whose variant stack is exactly one classic variant keeps
    /// going to the v0.1 bucket; **everything else** lands here (plan §2.1).
    pub rules: Vec<Rule>,
    /// v0.2: theme-aware breakpoint px (index = [`Breakpoint::index`]).
    /// Backends MUST use this instead of hardcoded defaults (plan §4.2).
    pub breakpoints: [f32; 5],
}

/// A `(Condition, ResolvedStyle)` pair.
#[derive(Clone, Debug, PartialEq)]
pub struct ConditionalLayer {
    /// The pseudo-state that guards this layer.
    pub condition: VariantCondition,
    /// Properties applied while the condition holds.
    pub style: ResolvedStyle,
}

/// The condition guarding a layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VariantCondition {
    /// `hover:` — pointer over the element (or a descendant).
    Hover,
    /// `active:` — the element is being pressed.
    Active,
    /// `disabled:` — the element is disabled.
    Disabled,
    /// `focus:` — the element holds keyboard focus.
    Focus,
    /// `focus-visible:` — keyboard-revealed focus (ring idioms).
    FocusVisible,
    /// `focus-within:` — the element or a descendant holds focus.
    FocusWithin,
    /// `visited:` — the element was visited (links; rarely native).
    Visited,
}

impl VariantCondition {
    /// Canonical layer order (v4 `hover` < `focus` < `active` < `disabled`
    /// cascade intent: later conditions win when several are active).
    pub fn rank(self) -> u8 {
        match self {
            VariantCondition::Hover => 0,
            VariantCondition::Focus => 1,
            VariantCondition::FocusVisible => 2,
            VariantCondition::FocusWithin => 3,
            VariantCondition::Active => 4,
            VariantCondition::Disabled => 5,
            VariantCondition::Visited => 6,
        }
    }
}

/// Label of a `group-hover/name` container. Empty string = unnamed group.
pub type GroupId = Box<str>;
/// Label of a `peer-focus/name` sibling. Empty string = unnamed peer.
pub type PeerId = Box<str>;

// ---------------------------------------------------------------------------
// Resolved style: typed slots
// ---------------------------------------------------------------------------

/// The typed style slots. `None` means "not set by any class".
///
/// This is intentionally close to Masonry's property set; slots Masonry
/// lacks (layout, margin, ring, opacity...) are marked in [`Slot`]
/// so backends can negotiate support via [`crate::caps::BackendCapabilities`].
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ResolvedStyle {
    /// `bg-{color}` solid fill (the `background-color` declaration).
    pub background: Option<Background>,
    /// The composed `background-image` gradient (`bg-linear-*`,
    /// `bg-radial`, `bg-conic*`; v0.6 §1.1). Coexists with `background`
    /// exactly as CSS's `background-color` + `background-image` do.
    pub background_gradient: Option<Gradient>,
    /// `text-{color}` / icon color.
    pub content_color: Option<ContentColor>,
    /// `p-*` family.
    pub padding: Option<Padding>,
    /// `border-{color}`.
    pub border_color: Option<BorderColor>,
    /// `border-*` width family.
    pub border_width: Option<BorderWidth>,
    /// `rounded-*` family.
    pub corner_radius: Option<CornerRadius>,
    /// `w-*` / `h-*` / `size-*`.
    pub dimensions: Option<Dimensions>,
    /// `min-w-*` / `min-h-*`.
    pub min_dimensions: Option<Dimensions>,
    /// `max-w-*` / `max-h-*`.
    pub max_dimensions: Option<Dimensions>,
    /// `gap-*`.
    pub gap: Option<Gap>,
    /// `shadow-*` preset (first layer of the multi-layer v4 presets).
    pub shadow: Option<BoxShadow>,
    /// `m-*` family (no Masonry property; capability-negotiated).
    pub margin: Option<Margin>,
    /// `flex-*`, `items-*`, `justify-*`, `grid`... display + axis settings.
    pub layout: Option<Layout>,
    /// `opacity-*`.
    pub opacity: Option<Opacity>,
    /// `ring-*` family (width + color + offset).
    pub ring: Option<Ring>,
    /// `outline-*` family.
    pub outline: Option<Outline>,
    /// `cursor-*`.
    pub cursor: Option<Cursor>,
    /// `pointer-events-*`.
    pub pointer_events: Option<PointerEvents>,
    /// Typography and text-flow properties (`text-sm`, `font-medium`, ...).
    pub text: Vec<(TextProperty, TextValue)>,
    /// v0.2 slots (position, inset, z-index, overflow, grid, transform,
    /// transition, animation, ...). See the [`Extras`] fields.
    pub extras: Extras,
}

/// The v0.2 typed slots (plan §2.1 `Extras`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Extras {
    /// `absolute`/`relative`/... positioning mode.
    pub position: Option<Position>,
    /// `top-*`/`inset-*`/... offsets.
    pub inset: Option<Inset>,
    /// `z-*`.
    pub z_index: Option<i32>,
    /// `overflow-*`.
    pub overflow: Option<Overflow>,
    /// `visible`/`invisible`.
    pub visibility: Option<Visibility>,
    /// `aspect-*` as width/height ratio.
    pub aspect_ratio: Option<f32>,
    /// `grid-cols-*`/`col-span-*`/...
    pub grid: Option<Grid>,
    /// `scale-*`/`rotate-*`/`translate-*`/`skew-*`.
    pub transform: Option<Transform>,
    /// `transition-*`/`duration-*`/`ease-*`/`delay-*`.
    pub transition: Option<Transition>,
    /// `animate-*`.
    pub animation: Option<Animation>,
    /// `border-{solid,dashed,...}`.
    pub border_style: Option<BorderStyle>,
    /// `basis-*`.
    pub flex_basis: Option<SizedValue>,
    /// `order-*`.
    pub order: Option<i16>,
    /// `content-*` (align-content).
    pub align_content: Option<ContentAlign>,
    /// `mx-auto`/`m-auto`/... auto-margin edges.
    pub margin_auto: Option<AutoEdges>,
    /// `gap-x-*`/`gap-y-*` (per-axis gap).
    pub gap_axes: Option<GapAxes>,
    /// Remaining shadow layers of a multi-layer preset (`shadow-sm`'s
    /// second layer). `shadow` carries the first (dominant) layer.
    pub shadow_rest: Vec<BoxShadow>,
    /// `inset-shadow-*` presets.
    pub inset_shadows: Vec<BoxShadow>,
}

/// One typed slot of [`ResolvedStyle`], used for diffs and capability checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Slot {
    /// Unconditional properties (the base layer).
    Background,
    /// Text/icon color.
    ContentColor,
    /// Padding (all edges).
    Padding,
    /// Border color.
    BorderColor,
    /// Border width.
    BorderWidth,
    /// Corner radius.
    CornerRadius,
    /// Fixed dimensions.
    Dimensions,
    /// Minimum dimensions.
    MinDimensions,
    /// Maximum dimensions.
    MaxDimensions,
    /// Flex/grid gap.
    Gap,
    /// Box shadow.
    Shadow,
    /// Margin.
    Margin,
    /// Display / flex layout slot.
    Layout,
    /// Opacity factor.
    Opacity,
    /// Ring (with the sub-part that changed).
    Ring(RingPart),
    /// Outline.
    Outline,
    /// Cursor icon.
    Cursor,
    /// Pointer events.
    PointerEvents,
    /// Text slot (individual text properties are sub-keyed).
    Text(TextProperty),
    // ---- v0.2 slots -------------------------------------------------------
    /// Positioning mode.
    Position,
    /// Inset offsets.
    Inset,
    /// Z-index.
    ZIndex,
    /// Overflow.
    Overflow,
    /// Visibility.
    Visibility,
    /// Aspect ratio.
    AspectRatio,
    /// Grid template/span.
    Grid,
    /// Transform (scale/rotate/translate/skew).
    Transform,
    /// Transition.
    Transition,
    /// Animation.
    Animation,
    /// Border style.
    BorderStyle,
    /// Flex basis.
    FlexBasis,
    /// Order.
    Order,
    /// Align content.
    AlignContent,
    /// Auto margins.
    MarginAuto,
    /// Per-axis gap.
    GapAxes,
    /// Extra shadow layers.
    ShadowRest,
    /// Inset shadows.
    InsetShadows,
    /// `background-image` gradient (with the part that changed).
    Gradient(GradientPart),
}

/// Which part of the composed [`Gradient`] changed (v0.6 §1.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GradientPart {
    /// The shape (`bg-linear-*`/`bg-radial`/`bg-conic*`/`bg-none`).
    Shape,
    /// The `from-*` stop (color or position patch).
    From,
    /// The `via-*` stop (color or position patch).
    Via,
    /// The `to-*` stop (color or position patch).
    To,
}

/// Dense slot index for the O(1) diff bitmask (plan §3.4.5).
impl Slot {
    /// Stable dense index (< 128) used by the diff bitmask.
    pub fn index(self) -> u16 {
        match self {
            Slot::Background => 0,
            Slot::ContentColor => 1,
            Slot::Padding => 2,
            Slot::BorderColor => 3,
            Slot::BorderWidth => 4,
            Slot::CornerRadius => 5,
            Slot::Dimensions => 6,
            Slot::MinDimensions => 7,
            Slot::MaxDimensions => 8,
            Slot::Gap => 9,
            Slot::Shadow => 10,
            Slot::Margin => 11,
            Slot::Layout => 12,
            Slot::Opacity => 13,
            Slot::Ring(_) => 14,
            Slot::Outline => 15,
            Slot::Cursor => 16,
            Slot::PointerEvents => 17,
            Slot::Text(_) => 18,
            Slot::Position => 19,
            Slot::Inset => 20,
            Slot::ZIndex => 21,
            Slot::Overflow => 22,
            Slot::Visibility => 23,
            Slot::AspectRatio => 24,
            Slot::Grid => 25,
            Slot::Transform => 26,
            Slot::Transition => 27,
            Slot::Animation => 28,
            Slot::BorderStyle => 29,
            Slot::FlexBasis => 30,
            Slot::Order => 31,
            Slot::AlignContent => 32,
            Slot::MarginAuto => 33,
            Slot::GapAxes => 34,
            Slot::ShadowRest => 35,
            Slot::InsetShadows => 36,
            Slot::Gradient(_) => 37,
        }
    }
}

impl Slot {
    /// Does changing this slot require relayout (geometric lane), or only a
    /// repaint (cosmetic lane)? Mirrors Masonry's `core_property_changed`
    /// lane assignment (masonry_core/src/properties/mod.rs).
    pub fn lane(self) -> Lane {
        match self {
            Slot::Background
            | Slot::Gradient(_)
            | Slot::ContentColor
            | Slot::BorderColor
            | Slot::Opacity
            | Slot::Cursor
            | Slot::PointerEvents
            | Slot::Visibility
            | Slot::Overflow
            | Slot::ZIndex
            | Slot::Transform
            | Slot::Transition
            | Slot::Animation
            | Slot::BorderStyle
            | Slot::Text(TextProperty::Color | TextProperty::UnderlineOffset)
            | Slot::Ring(_) => Lane::Cosmetic,
            // Masonry requests layout for Padding/BorderWidth/CornerRadius/
            // Dimensions/BoxShadow; text size changes parley layout.
            Slot::Padding
            | Slot::BorderWidth
            | Slot::CornerRadius
            | Slot::Dimensions
            | Slot::MinDimensions
            | Slot::MaxDimensions
            | Slot::Gap
            | Slot::Shadow
            | Slot::Margin
            | Slot::Layout
            | Slot::Outline
            | Slot::Position
            | Slot::Inset
            | Slot::AspectRatio
            | Slot::Grid
            | Slot::FlexBasis
            | Slot::Order
            | Slot::AlignContent
            | Slot::MarginAuto
            | Slot::GapAxes
            | Slot::ShadowRest
            | Slot::InsetShadows
            | Slot::Text(_) => Lane::Geometric,
        }
    }
}

/// Invalidation lane for the two-lane (cosmetic vs geometric) model.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lane {
    /// Repaint-only invalidation (colors, opacity, cursor...).
    Cosmetic,
    /// Full relayout required (padding, dimensions, text metrics...).
    Geometric,
}

/// Which half of `Ring` changed (`Ring::width` / `Ring::color` sub-keys).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RingPart {
    /// `ring-*` width.
    Width,
    /// `ring-{color}`.
    Color,
    /// `ring-offset-*` width.
    OffsetWidth,
    /// `ring-offset-{color}`.
    OffsetColor,
}

// --- slot payloads ---------------------------------------------------------

/// `bg-{color}` — the solid `background-color` fill.
///
/// v0.6 keeps the v4 model: a solid background and a `background-image`
/// gradient are SEPARATE declarations that coexist (CSS layers the image
/// over the color). The composed image lives in
/// [`ResolvedStyle::background_gradient`]; backends with a single brush
/// slot lower the gradient preferentially and record the coexistence.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Background(pub Color);

/// The v4.1.17-pinned default stop positions (the `@property` initial
/// values: `--tw-gradient-from-position: 0%`, `--tw-gradient-via-position:
/// 50%`, `--tw-gradient-to-position: 100%` — captured verbatim by
/// `tools/capture_gradients.mjs`).
pub const GRADIENT_FROM_POS_PCT: f32 = 0.0;
/// v4 default via-stop position.
pub const GRADIENT_VIA_POS_PCT: f32 = 50.0;
/// v4 default to-stop position.
pub const GRADIENT_TO_POS_PCT: f32 = 100.0;

/// One gradient color stop. `pos_pct: None` = the v4 `@property` default
/// for the edge (see [`GRADIENT_FROM_POS_PCT`]/[`GRADIENT_VIA_POS_PCT`]/
/// [`GRADIENT_TO_POS_PCT`]); `Some(p)` = the `from-{P}%` position patch.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GradientStop {
    /// The stop color (oklab IR; backends convert per their pipeline).
    pub color: Color,
    /// Stop position in percent (0..=100), when patched by a position class.
    pub pos_pct: Option<f32>,
}

/// The three composed stops of a v4 gradient
/// (`from-*`, optional `via-*`, `to-*`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GradientStops {
    /// `from-{color}` (transparent when absent — v4 `@property` initial
    /// value `#0000`).
    pub from: GradientStop,
    /// `via-{color}` when a via class is in the string.
    pub via: Option<GradientStop>,
    /// `to-{color}` (transparent when absent).
    pub to: GradientStop,
}

/// A `background-image` gradient (`bg-linear-*`, `bg-radial`,
/// `bg-conic*`), composed at resolve time (v0.6 §1.1).
///
/// Angle conventions are CSS's: `angle_deg` 0 = to top, positive
/// clockwise (`bg-linear-to-r` = 90). Conic `from_angle_deg` follows the
/// same convention (`bg-conic-45` = `from 45deg`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Gradient {
    /// `bg-none` — `background-image: none` (explicitly clears the image;
    /// overlays like a value, matching the `shadow-none` precedent).
    None,
    /// `bg-linear-*` — a linear gradient along the CSS angle.
    Linear {
        /// CSS angle in degrees (0 = to top, clockwise; `to-r` = 90).
        angle_deg: f32,
        /// The composed from/via/to stops.
        stops: GradientStops,
    },
    /// `bg-radial` — a radial gradient from the box center.
    ///
    /// The shape is `circle farthest-corner` (the CSS default is an
    /// ELLIPSE farthest-corner; Masonry's `RadialGradientShape` has no
    /// ellipse radii at the pinned rev, so the circle form is the
    /// documented normalization — v0.6 §1.5).
    Radial {
        /// The composed from/via/to stops.
        stops: GradientStops,
    },
    /// `bg-conic-*` — a conic gradient. Carried in the IR for v4
    /// fidelity; BOTH backends report it (Masonry's sweep paint carries
    /// an upstream vello angle-offset TODO, Floem has no gradient brush
    /// support) — v0.6 §1.5.
    Conic {
        /// CSS `from <angle>` in degrees (0 = to top, clockwise).
        from_angle_deg: f32,
        /// The composed from/via/to stops.
        stops: GradientStops,
    },
}

impl Gradient {
    /// The composed stops, whichever shape this is (`Gradient::None` has
    /// none — transparent from/to).
    pub fn stops(&self) -> &GradientStops {
        match self {
            Gradient::None => &NO_STOPS,
            Gradient::Linear { stops, .. }
            | Gradient::Radial { stops }
            | Gradient::Conic { stops, .. } => stops,
        }
    }
}

/// The shared transparent stops behind [`Gradient::None`].
const NO_STOPS: GradientStops = GradientStops {
    from: GradientStop {
        color: Color::TRANSPARENT,
        pos_pct: None,
    },
    via: None,
    to: GradientStop {
        color: Color::TRANSPARENT,
        pos_pct: None,
    },
};

/// `text-{color}` / icon color.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ContentColor(pub Color);

/// Which edges a per-edge family (`p-*`/`m-*`/`border-*`) actually
/// addressed — the v0.8 set-mask (AEOLUS_V0.8.md §1).
///
/// `false` means "this bucket/layer/plan never named that edge": the edge
/// must NOT replace a lower cascade entry when the style is applied, and
/// its `f32` value is the documented `0.0` placeholder. `true` means the
/// edge carries a real value (which may legitimately be `0.0` — `pt-0`).
///
/// The same shape as [`AutoEdges`] (which says which edges are `auto`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EdgeSet {
    /// The top edge is addressed.
    pub top: bool,
    /// The right edge is addressed.
    pub right: bool,
    /// The bottom edge is addressed.
    pub bottom: bool,
    /// The left edge is addressed.
    pub left: bool,
}

impl EdgeSet {
    /// Every edge addressed (a whole-edge family like `p-4`, or a merged
    /// result that covers all four).
    pub const ALL: EdgeSet = EdgeSet {
        top: true,
        right: true,
        bottom: true,
        left: true,
    };

    /// True when every edge is addressed.
    pub fn is_all(self) -> bool {
        self == EdgeSet::ALL
    }
}

/// `p-*/px-*/py-*/pt-*/pr-*/pb-*/pl-*` in px.
///
/// v0.8: the [`EdgeSet`] mask records which edges the class string named;
/// edges outside the mask are `0.0` placeholders, never cascade inputs.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Padding {
    /// Top edge in px.
    pub top: f32,
    /// Right edge in px.
    pub right: f32,
    /// Bottom edge in px.
    pub bottom: f32,
    /// Left edge in px.
    pub left: f32,
    /// Which edges this value addresses ([`EdgeSet::ALL`] for whole-edge
    /// families and merged results).
    pub set: EdgeSet,
}

/// `m-*` family (same shape as padding; separate type so slots stay distinct).
pub type Margin = Padding;

/// `border-{color}`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BorderColor(pub Color);

/// `border`, `border-2`, `border-x` etc. in px.
///
/// v0.8: carries the same [`EdgeSet`] mask as [`Padding`] — `border-r-4`
/// addresses only the right side.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BorderWidth {
    /// Top edge in px.
    pub top: f32,
    /// Right edge in px.
    pub right: f32,
    /// Bottom edge in px.
    pub bottom: f32,
    /// Left edge in px.
    pub left: f32,
    /// Which edges this value addresses.
    pub set: EdgeSet,
}

/// Which corners a `rounded-*` family actually addressed — the corner
/// counterpart of [`EdgeSet`] (v0.8 §1).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CornerSet {
    /// The top-left corner is addressed.
    pub top_left: bool,
    /// The top-right corner is addressed.
    pub top_right: bool,
    /// The bottom-right corner is addressed.
    pub bottom_right: bool,
    /// The bottom-left corner is addressed.
    pub bottom_left: bool,
}

impl CornerSet {
    /// Every corner addressed (whole-corner families like `rounded-md`).
    pub const ALL: CornerSet = CornerSet {
        top_left: true,
        top_right: true,
        bottom_right: true,
        bottom_left: true,
    };

    /// True when every corner is addressed.
    pub fn is_all(self) -> bool {
        self == CornerSet::ALL
    }
}

/// `rounded-*` in px.
///
/// v0.8: the [`CornerSet`] mask records which corners the class string
/// named; corners outside the mask are `0.0` placeholders.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CornerRadius {
    /// Top-left corner in px.
    pub top_left: f32,
    /// Top-right corner in px.
    pub top_right: f32,
    /// Bottom-right corner in px.
    pub bottom_right: f32,
    /// Bottom-left corner in px.
    pub bottom_left: f32,
    /// Which corners this value addresses.
    pub set: CornerSet,
}

/// `w-*/h-*/size-*` fixed sizes in px; `f32::INFINITY` is not used —
/// `None`-fields mean "not set"; percentage/full/fit sizes are represented
/// with [`SizedValue`].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Dimensions {
    /// Width, `None` when unset.
    pub width: Option<SizedValue>,
    /// Height, `None` when unset.
    pub height: Option<SizedValue>,
}

/// A dimension value: absolute px, fraction (Tailwind scale n/4 rem), or
/// symbolic (`full`, `fit`, `auto`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SizedValue {
    /// Absolute px.
    Px(f32),
    /// Tailwind numeric scale: `n` -> `n * 0.25rem` (4px base).
    Scale(f32),
    /// `w-full` (100%), `w-1/2` etc. as a 0..1 fraction.
    Fraction(f32),
    /// `w-full` symbolic (100%).
    Full,
    /// `w-fit` (fit-content).
    FitContent,
    /// `w-auto`.
    Auto,
    /// `w-screen` / `h-dvh` — 100% of the viewport; backends resolve the
    /// actual size from the window (v0.2, plan §3.2).
    Viewport,
}

/// `gap-*` in px.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gap(pub f32);

/// `shadow-*` — one layer per shadow (v4 presets are multi-layer: the first
/// layer lives here, the rest in `Extras::shadow_rest`).
#[derive(Clone, Debug, PartialEq)]
pub struct BoxShadow {
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
    /// `shadow-inner` / `inset-shadow-*` — inset shadows.
    pub inset: bool,
}

/// `flex`, `grid`, `items-*`, `justify-*`, `flex-col`, `grow/shrink`...
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Layout {
    /// `flex` / `grid` / `block` / `hidden`...
    pub display: Option<Display>,
    /// `flex-col` etc.
    pub flex_direction: Option<FlexDirection>,
    /// `items-*`.
    pub align_items: Option<AlignItems>,
    /// `justify-*` (content axis).
    pub justify_content: Option<JustifyContent>,
    /// `flex-wrap`.
    pub flex_wrap: Option<bool>,
    /// `grow-*`.
    pub flex_grow: Option<f32>,
    /// `shrink-*`.
    pub flex_shrink: Option<f32>,
    /// `self-*`.
    pub align_self: Option<AlignItems>,
}

/// The CSS `display` value (subset).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Display {
    /// `flex`.
    Flex,
    /// `inline-flex`.
    InlineFlex,
    /// `grid`.
    Grid,
    /// `block`.
    Block,
    /// `inline`.
    Inline,
    /// `hidden`.
    None,
}

/// The flex main-axis direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlexDirection {
    /// `flex-row`.
    Row,
    /// `flex-col`.
    Column,
    /// `flex-row-reverse`.
    RowReverse,
    /// `flex-col-reverse`.
    ColumnReverse,
}

/// The cross-axis alignment (`items-*` / `self-*`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlignItems {
    /// `items-start`.
    Start,
    /// `items-center`.
    Center,
    /// `items-end`.
    End,
    /// `items-stretch`.
    Stretch,
    /// `items-baseline`.
    Baseline,
}

/// The main-axis distribution (`justify-*`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JustifyContent {
    /// `justify-start`.
    Start,
    /// `justify-center`.
    Center,
    /// `justify-end`.
    End,
    /// `justify-between`.
    SpaceBetween,
    /// `justify-around`.
    SpaceAround,
    /// `justify-evenly`.
    SpaceEvenly,
}

/// `opacity-*` as a 0..1 factor.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Opacity(pub f32);

/// `ring-*` (box ring): width + color + optional offset (v4: `ring-[3px]`,
/// `ring-ring/50`, `ring-offset-2`, `ring-offset-background`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Ring {
    /// `ring-*` width in px.
    pub width: f32,
    /// `ring-{color}`.
    pub color: Option<Color>,
    /// `ring-offset-*` width in px.
    pub offset_width: f32,
    /// `ring-offset-{color}`.
    pub offset_color: Option<Color>,
    /// `ring-inset` — draw the ring inside the border box (v0.2).
    pub inset: bool,
}

/// `outline-none`, `outline-*`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Outline {
    /// `outline-*` width in px.
    pub width: f32,
    /// `outline-{color}`.
    pub color: Option<Color>,
    /// `outline-none` in v4 sets width 0 + `outline-style: none` semantically.
    pub none: bool,
    /// `outline-offset-*` in px (v0.2).
    pub offset: f32,
}

/// The mouse cursor (`cursor-*`) — the full v4 keyword set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cursor {
    /// `cursor-auto`.
    Auto,
    /// `cursor-default`.
    Default,
    /// `cursor-pointer`.
    Pointer,
    /// `cursor-wait`.
    Wait,
    /// `cursor-text`.
    Text,
    /// `cursor-move`.
    Move,
    /// `cursor-help`.
    Help,
    /// `cursor-not-allowed`.
    NotAllowed,
    /// `cursor-none`.
    None,
    /// `cursor-context-menu`.
    ContextMenu,
    /// `cursor-progress`.
    Progress,
    /// `cursor-cell`.
    Cell,
    /// `cursor-crosshair`.
    Crosshair,
    /// `cursor-vertical-text`.
    VerticalText,
    /// `cursor-alias`.
    Alias,
    /// `cursor-copy`.
    Copy,
    /// `cursor-no-drop`.
    NoDrop,
    /// `cursor-grab`.
    Grab,
    /// `cursor-grabbing`.
    Grabbing,
    /// `cursor-all-scroll`.
    AllScroll,
    /// `cursor-col-resize`.
    ColResize,
    /// `cursor-row-resize`.
    RowResize,
    /// `cursor-n-resize`.
    NResize,
    /// `cursor-e-resize`.
    EResize,
    /// `cursor-s-resize`.
    SResize,
    /// `cursor-w-resize`.
    WResize,
    /// `cursor-ne-resize`.
    NeResize,
    /// `cursor-nw-resize`.
    NwResize,
    /// `cursor-se-resize`.
    SeResize,
    /// `cursor-sw-resize`.
    SwResize,
    /// `cursor-ew-resize`.
    EwResize,
    /// `cursor-ns-resize`.
    NsResize,
    /// `cursor-nesw-resize`.
    NeswResize,
    /// `cursor-nwse-resize`.
    NwseResize,
    /// `cursor-zoom-in`.
    ZoomIn,
    /// `cursor-zoom-out`.
    ZoomOut,
}

/// The `pointer-events` value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerEvents {
    /// `pointer-events-none`.
    None,
    /// `pointer-events-auto`.
    Auto,
}

/// Text properties carried in `ResolvedStyle::text`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TextProperty {
    /// `text-{size}` in px.
    FontSize,
    /// `leading-*` in px.
    LineHeight,
    /// `font-{weight}` 100..=900.
    FontWeight,
    /// `tracking-*` in px.
    LetterSpacing,
    /// `text-{color}`.
    Color,
    /// `underline-offset-*` in px.
    UnderlineOffset,
    /// `whitespace-*` keyword.
    Whitespace,
    /// `text-left/center/right` alignment.
    TextAlign,
    /// `select-none` / `select-text`.
    Select,
    // ---- v0.2 --------------------------------------------------------------
    /// `italic` / `not-italic`.
    FontStyle,
    /// `uppercase` / `lowercase` / `capitalize` / `normal-case`.
    TextTransform,
    /// `underline` / `overline` / `line-through` / `no-underline`.
    TextDecoration,
    /// `font-sans` / `font-serif` / `font-mono`.
    FontFamily,
    /// `text-ellipsis` / `text-clip` (with `truncate`).
    TextOverflow,
    /// `line-clamp-N`.
    LineClamp,
    /// `tabular-nums` / `ordinal` / ... font-variant-numeric.
    FontVariantNumeric,
}

/// The value of one [`TextProperty`].
///
/// Element-relative units (v0.5 §1, `AEOLUS_V0.5.md`): the token forms of
/// `tracking-*` and `leading-*` carry the RELATIVE unit the class names
/// (em, unitless ratio) — exactly what v4's `--tracking-*`/`--leading-*`
/// tokens hold. Multiplying by an element's font-size is the renderer's
/// cascade duty (CSS `em` semantics; Floem resolves the multiplier at
/// paint), not the compiler's: the compiler cannot know which font-size
/// the element ends up with.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TextValue {
    /// px (resolved from `--text-*` tokens).
    Px(f32),
    /// font-weight 100..=900.
    Weight(u16),
    /// letter-spacing px (negative for `tracking-tighter`).
    Spacing(f32),
    /// letter-spacing in em (the `--tracking-*` token form:
    /// `tracking-tight` = −0.025em). Element-relative — v0.5 §1.1.
    TrackingEm(f32),
    /// unitless line-height ratio (the `--leading-*` token form:
    /// `leading-tight` = 1.25). Element-relative — v0.5 §1.1.
    LineHeightRatio(f32),
    /// `whitespace-nowrap`, `select-none`, `italic`, `uppercase`,
    /// `underline`, `sans`, `ellipsis`, `tabular-nums`... keywords.
    Keyword(&'static str),
    /// A count (`line-clamp-3`).
    Count(u16),
    /// text color.
    Color(Color),
}

// ---------------------------------------------------------------------------
// v0.2 Extras payloads
// ---------------------------------------------------------------------------

/// The CSS `position` value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Position {
    /// `static`.
    Static,
    /// `fixed`.
    Fixed,
    /// `absolute`.
    Absolute,
    /// `relative`.
    Relative,
    /// `sticky`.
    Sticky,
}

/// `top-*`/`right-*`/`bottom-*`/`left-*`/`inset-*` offsets.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Inset {
    /// `top-*` offset.
    pub top: Option<SizedValue>,
    /// `right-*` offset.
    pub right: Option<SizedValue>,
    /// `bottom-*` offset.
    pub bottom: Option<SizedValue>,
    /// `left-*` offset.
    pub left: Option<SizedValue>,
}

/// The overflow kind per axis.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverflowKind {
    /// `overflow-visible`.
    Visible,
    /// `overflow-hidden`.
    Hidden,
    /// `overflow-clip`.
    Clip,
    /// `overflow-scroll`.
    Scroll,
    /// `overflow-auto`.
    Auto,
}

/// `overflow-*` per axis.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Overflow {
    /// `overflow-x-*`.
    pub x: Option<OverflowKind>,
    /// `overflow-y-*`.
    pub y: Option<OverflowKind>,
}

/// `visible` / `invisible`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Visibility {
    /// `visible`.
    Visible,
    /// `invisible`.
    Hidden,
}

/// `grid-cols-*` / `grid-rows-*` / spans / starts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Grid {
    /// `grid-cols-N`.
    pub cols: Option<u16>,
    /// `grid-rows-N`.
    pub rows: Option<u16>,
    /// `col-span-N`.
    pub col_span: Option<u16>,
    /// `row-span-N`.
    pub row_span: Option<u16>,
    /// `col-start-N`.
    pub col_start: Option<i16>,
    /// `row-start-N`.
    pub row_start: Option<i16>,
}

/// `scale-*`/`rotate-*`/`translate-*`/`skew-*`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Transform {
    /// `scale-x-*` (1.0 = 100%).
    pub scale_x: Option<f32>,
    /// `scale-y-*` (1.0 = 100%).
    pub scale_y: Option<f32>,
    /// `rotate-*` degrees (positive = clockwise).
    pub rotate_deg: Option<f32>,
    /// `translate-x-*`.
    pub translate_x: Option<SizedValue>,
    /// `translate-y-*`.
    pub translate_y: Option<SizedValue>,
    /// `skew-x-*` degrees.
    pub skew_x: Option<f32>,
    /// `skew-y-*` degrees.
    pub skew_y: Option<f32>,
}

/// The timing function (`ease-*`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ease {
    /// `ease-linear`.
    Linear,
    /// `ease-in` (`--ease-in`).
    In,
    /// `ease-out` (`--ease-out`).
    Out,
    /// `ease-in-out` (`--ease-in-out`).
    InOut,
}

/// Which property set a `transition-*` applies to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransitionSet {
    /// `transition` — the v4 default set (colors, opacity, shadow, transform,
    /// filter, backdrop-filter).
    Default,
    /// `transition-all`.
    All,
    /// `transition-colors`.
    Colors,
    /// `transition-opacity`.
    Opacity,
    /// `transition-shadow`.
    Shadow,
    /// `transition-transform`.
    Transform,
    /// `transition-none`.
    None,
}

/// `transition-*` + `duration-*` + `ease-*` + `delay-*`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transition {
    /// Which property set transitions.
    pub set: TransitionSet,
    /// `duration-*` ms (`--default-transition-duration` = 150 when unset).
    pub duration_ms: f32,
    /// `ease-*`.
    pub ease: Option<Ease>,
    /// `delay-*` ms.
    pub delay_ms: f32,
}

/// `animate-*`.
#[derive(Clone, Debug, PartialEq)]
pub struct Animation {
    /// Animation name (`spin`, `ping`, `pulse`, `bounce`, or the token
    /// suffix of a custom `--animate-*`).
    pub name: Box<str>,
    /// `duration-*` override in ms, when set.
    pub duration_ms: Option<f32>,
    /// `ease-*` override, when set.
    pub ease: Option<Ease>,
    /// Infinite by default (v4 presets), unless the token says otherwise.
    pub infinite: bool,
}

/// `border-{solid,dashed,dotted,double,...}`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BorderStyle {
    /// `border-solid`.
    Solid,
    /// `border-dashed`.
    Dashed,
    /// `border-dotted`.
    Dotted,
    /// `border-double`.
    Double,
    /// `border-none`.
    None,
    /// `border-hidden`.
    Hidden,
}

/// `content-*` (align-content).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContentAlign {
    /// `content-start`.
    Start,
    /// `content-center`.
    Center,
    /// `content-end`.
    End,
    /// `content-between`.
    SpaceBetween,
    /// `content-around`.
    SpaceAround,
    /// `content-evenly`.
    SpaceEvenly,
    /// `content-baseline`.
    Baseline,
    /// `content-stretch`.
    Stretch,
    /// `content-normal`.
    Normal,
}

/// `mx-auto`/`m-auto`/... — which edges are auto.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AutoEdges {
    /// `ml-auto` / `m-auto`.
    pub left: bool,
    /// `mr-auto` / `m-auto`.
    pub right: bool,
    /// `mt-auto` / `m-auto`.
    pub top: bool,
    /// `mb-auto` / `m-auto`.
    pub bottom: bool,
}

/// `gap-x-*` / `gap-y-*` (per-axis gap in px).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GapAxes {
    /// `gap-x-*` (column gap) in px.
    pub col: Option<f32>,
    /// `gap-y-*` (row gap) in px.
    pub row: Option<f32>,
}

// ---------------------------------------------------------------------------
// slots(), overlay(), diff
// ---------------------------------------------------------------------------

// Walk every present slot. ONE source of truth for slot presence,
// shared by `slots()` (the public Vec API) and `bits_of` (the diff
// fast path — which must NOT allocate, per the rebuild-path contract
// pinned by tests/alloc_contract.rs, v0.5 §3).
macro_rules! for_each_present_slot {
    ($s:expr, $push:expr) => {{
        let s = $s;
        if s.background.is_some() {
            $push(Slot::Background);
        }
        if s.content_color.is_some() {
            $push(Slot::ContentColor);
        }
        if s.padding.is_some() {
            $push(Slot::Padding);
        }
        if s.border_color.is_some() {
            $push(Slot::BorderColor);
        }
        if s.border_width.is_some() {
            $push(Slot::BorderWidth);
        }
        if s.corner_radius.is_some() {
            $push(Slot::CornerRadius);
        }
        if s.dimensions.is_some() {
            $push(Slot::Dimensions);
        }
        if s.min_dimensions.is_some() {
            $push(Slot::MinDimensions);
        }
        if s.max_dimensions.is_some() {
            $push(Slot::MaxDimensions);
        }
        if s.gap.is_some() {
            $push(Slot::Gap);
        }
        if s.shadow.is_some() {
            $push(Slot::Shadow);
        }
        if s.margin.is_some() {
            $push(Slot::Margin);
        }
        if s.layout.is_some() {
            $push(Slot::Layout);
        }
        if s.opacity.is_some() {
            $push(Slot::Opacity);
        }
        if s.ring.is_some() {
            $push(Slot::Ring(RingPart::Width));
        }
        if s.outline.is_some() {
            $push(Slot::Outline);
        }
        if s.cursor.is_some() {
            $push(Slot::Cursor);
        }
        if s.pointer_events.is_some() {
            $push(Slot::PointerEvents);
        }
        for (p, _) in &s.text {
            $push(Slot::Text(*p));
        }
        let e = &s.extras;
        if e.position.is_some() {
            $push(Slot::Position);
        }
        if e.inset.is_some() {
            $push(Slot::Inset);
        }
        if e.z_index.is_some() {
            $push(Slot::ZIndex);
        }
        if e.overflow.is_some() {
            $push(Slot::Overflow);
        }
        if e.visibility.is_some() {
            $push(Slot::Visibility);
        }
        if e.aspect_ratio.is_some() {
            $push(Slot::AspectRatio);
        }
        if e.grid.is_some() {
            $push(Slot::Grid);
        }
        if e.transform.is_some() {
            $push(Slot::Transform);
        }
        if e.transition.is_some() {
            $push(Slot::Transition);
        }
        if e.animation.is_some() {
            $push(Slot::Animation);
        }
        if e.border_style.is_some() {
            $push(Slot::BorderStyle);
        }
        if e.flex_basis.is_some() {
            $push(Slot::FlexBasis);
        }
        if e.order.is_some() {
            $push(Slot::Order);
        }
        if e.align_content.is_some() {
            $push(Slot::AlignContent);
        }
        if e.margin_auto.is_some() {
            $push(Slot::MarginAuto);
        }
        if e.gap_axes.is_some() {
            $push(Slot::GapAxes);
        }
        if !e.shadow_rest.is_empty() {
            $push(Slot::ShadowRest);
        }
        if !e.inset_shadows.is_empty() {
            $push(Slot::InsetShadows);
        }
        if s.background_gradient.is_some() {
            $push(Slot::Gradient(GradientPart::Shape));
        }
    }};
}

/// Edge-wise `Some`-replaces for one 4-edge slot (v0.8 §1): every edge the
/// override addresses wins; every edge it does not address keeps the
/// destination's value. `None` destinations seed a default so partial
/// overrides compose against nothing without inventing cascade inputs.
/// (`Padding`/`Margin` and `BorderWidth` share the exact field shape.)
macro_rules! overlay_edge_slot {
    ($dst:expr, $src:expr) => {
        if let Some(o) = $src {
            let d = $dst.get_or_insert_with(Default::default);
            if o.set.top {
                d.top = o.top;
                d.set.top = true;
            }
            if o.set.right {
                d.right = o.right;
                d.set.right = true;
            }
            if o.set.bottom {
                d.bottom = o.bottom;
                d.set.bottom = true;
            }
            if o.set.left {
                d.left = o.left;
                d.set.left = true;
            }
        }
    };
}

/// Corner-wise counterpart of [`overlay_edge_slot!`].
macro_rules! overlay_corner_slot {
    ($dst:expr, $src:expr) => {
        if let Some(o) = $src {
            let d = $dst.get_or_insert_with(Default::default);
            if o.set.top_left {
                d.top_left = o.top_left;
                d.set.top_left = true;
            }
            if o.set.top_right {
                d.top_right = o.top_right;
                d.set.top_right = true;
            }
            if o.set.bottom_right {
                d.bottom_right = o.bottom_right;
                d.set.bottom_right = true;
            }
            if o.set.bottom_left {
                d.bottom_left = o.bottom_left;
                d.set.bottom_left = true;
            }
        }
    };
}

/// Fill the destination slot's masked-off edges from `base` (values AND
/// mask), leaving `None` slots and addressed edges alone.
macro_rules! materialize_edge_slot {
    ($dst:expr, $base:expr) => {
        if let Some(d) = $dst
            && let Some(b) = $base
        {
            if !d.set.top {
                d.top = b.top;
                d.set.top = true;
            }
            if !d.set.right {
                d.right = b.right;
                d.set.right = true;
            }
            if !d.set.bottom {
                d.bottom = b.bottom;
                d.set.bottom = true;
            }
            if !d.set.left {
                d.left = b.left;
                d.set.left = true;
            }
        }
    };
}

/// Corner-wise counterpart of [`materialize_edge_slot!`].
macro_rules! materialize_corner_slot {
    ($dst:expr, $base:expr) => {
        if let Some(d) = $dst
            && let Some(b) = $base
        {
            if !d.set.top_left {
                d.top_left = b.top_left;
                d.set.top_left = true;
            }
            if !d.set.top_right {
                d.top_right = b.top_right;
                d.set.top_right = true;
            }
            if !d.set.bottom_right {
                d.bottom_right = b.bottom_right;
                d.set.bottom_right = true;
            }
            if !d.set.bottom_left {
                d.bottom_left = b.bottom_left;
                d.set.bottom_left = true;
            }
        }
    };
}

impl ResolvedStyle {
    /// Iterate over set slots (for diffing / capability checks).
    pub fn slots(&self) -> Vec<Slot> {
        let mut v = Vec::with_capacity(40);
        for_each_present_slot!(self, |sl: Slot| {
            if !v.contains(&sl) {
                v.push(sl);
            }
        });
        v
    }

    /// Overlay `other` on top of `self` (later layers win per slot).
    ///
    /// v0.8: the four per-edge families ([`Padding`]/[`Margin`],
    /// [`BorderWidth`], [`CornerRadius`]) merge EDGE-WISE through the
    /// set-mask — an edge the override does not address keeps the base
    /// value, so `p-4` ⊕ `px-8` is `Padding { top: 16, right: 32, bottom:
    /// 16, left: 32 }` (the CSS cascade), not the v0.7 zeroed replacement.
    /// Every other slot is `Some`-replaces as before.
    pub fn overlay(&mut self, other: &ResolvedStyle) {
        overlay_edge_slot!(&mut self.padding, &other.padding);
        overlay_edge_slot!(&mut self.margin, &other.margin);
        overlay_edge_slot!(&mut self.border_width, &other.border_width);
        overlay_corner_slot!(&mut self.corner_radius, &other.corner_radius);
        macro_rules! rep {
            ($($f:ident),* $(,)?) => {
                $(if other.$f.is_some() { self.$f = other.$f; })*
            };
        }
        rep!(
            background,
            background_gradient,
            content_color,
            border_color,
            dimensions,
            min_dimensions,
            max_dimensions,
            gap,
            layout,
            opacity,
            ring,
            outline,
            cursor,
            pointer_events
        );
        if other.shadow.is_some() {
            self.shadow.clone_from(&other.shadow);
        }
        for (p, val) in &other.text {
            if let Some(slot) = self.text.iter_mut().find(|(q, _)| q == p) {
                slot.1 = *val;
            } else {
                self.text.push((*p, *val));
            }
        }
        // Extras: per-field `Some`-replacing; Vec slots replace when non-empty.
        macro_rules! ex {
            ($($f:ident),* $(,)?) => {
                $(if other.extras.$f.is_some() { self.extras.$f = other.extras.$f.clone(); })*
            };
        }
        ex!(
            position,
            inset,
            z_index,
            overflow,
            visibility,
            aspect_ratio,
            grid,
            transform,
            transition,
            animation,
            border_style,
            flex_basis,
            order,
            align_content,
            margin_auto,
            gap_axes
        );
        if !other.extras.shadow_rest.is_empty() {
            self.extras
                .shadow_rest
                .clone_from(&other.extras.shadow_rest);
        }
        if !other.extras.inset_shadows.is_empty() {
            self.extras
                .inset_shadows
                .clone_from(&other.extras.inset_shadows);
        }
    }

    /// Fill THIS style's unset edges (mask off) from `base` for the four
    /// per-edge families — the layer-application primitive for cascades
    /// that replace WHOLE properties (Masonry's `PropertyStack` entries,
    /// Slint's state re-bindings).
    ///
    /// A `hover:px-8` layer materializes its top/bottom from the base
    /// plan's padding instead of carrying the `0.0` placeholders, so the
    /// applied entry computes what CSS computes (`p-4 hover:px-8` keeps
    /// `padding-top: 16px`). Slots that are `None` here stay `None` (the
    /// layer never addressed them; the lower cascade entry keeps serving
    /// them), and a materialized value's mask ends full.
    pub fn materialize_unset_edges_from(&mut self, base: &ResolvedStyle) {
        materialize_edge_slot!(&mut self.padding, &base.padding);
        materialize_edge_slot!(&mut self.margin, &base.margin);
        materialize_edge_slot!(&mut self.border_width, &base.border_width);
        materialize_corner_slot!(&mut self.corner_radius, &base.corner_radius);
    }
}

impl StylePlan {
    /// Compose `over` on top of `self` — the plan-level mirror of `cn()`.
    ///
    /// Where [`crate::cn`] merges class STRINGS before resolution (theme
    /// aware, parse-based), `overlay` merges RESOLVED plans after it:
    /// a component's cached plan can gain per-instance overrides at runtime
    /// without re-resolving, re-parsing, or a theme reference.
    ///
    /// Semantics (v0.7 §1, every rule later-wins):
    /// - `base`: [`ResolvedStyle::overlay`] — `Some` slots of `over.base`
    ///   replace, unset slots keep the base value.
    /// - `layers`: layers with the SAME [`VariantCondition`] merge per slot;
    ///   conditions only `over` has are appended. The result stays in
    ///   ascending [`VariantCondition::rank`] order (the cascade contract).
    /// - `responsive`/`container`: per-bucket overlay, union of buckets.
    /// - `dark`: per-slot overlay when both carry a dark bucket.
    /// - `group`/`peer`: merged by id (label equal), new ids appended.
    /// - `rules`: merged by [`crate::cond::Cond`] equality (the same conjunction is one
    ///   rule), new conditions appended in `over` order.
    /// - `breakpoints`: the override's table wins (it is theme-derived and
    ///   the override is the later consumer; mixing plans from different
    ///   theme versions is documented best-effort).
    ///
    /// The merged plan is idempotent (`p.overlay(o).overlay(o) == p.overlay(o)`)
    /// and diff-integrated: `diff(&base.base, &merged.base)` reports exactly
    /// the slots the override changed, so backends keep their repaint vs
    /// relayout lanes when composing.
    pub fn overlay(&self, over: &StylePlan) -> StylePlan {
        let mut merged = self.clone();
        merged.base.overlay(&over.base);

        // Layers: merge by condition, keep ascending rank order (stable).
        for layer in &over.layers {
            if let Some(slot) = merged
                .layers
                .iter_mut()
                .find(|l| l.condition == layer.condition)
            {
                slot.style.overlay(&layer.style);
            } else {
                merged.layers.push(layer.clone());
            }
        }
        merged.layers.sort_by_key(|l| l.condition.rank());

        // Responsive / container buckets: per-bucket overlay, union of keys.
        for (bp, style) in &over.responsive {
            match merged.responsive.get_mut(bp) {
                Some(dst) => dst.overlay(style),
                None => {
                    merged.responsive.insert(*bp, style.clone());
                }
            }
        }
        for (bp, style) in &over.container {
            match merged.container.get_mut(bp) {
                Some(dst) => dst.overlay(style),
                None => {
                    merged.container.insert(*bp, style.clone());
                }
            }
        }
        // Dark: overlay when both plans carry a dark bucket.
        match (&mut merged.dark, &over.dark) {
            (Some(dst), Some(src)) => dst.overlay(src),
            (None, Some(src)) => merged.dark = Some(src.clone()),
            _ => {}
        }
        // group/peer: merge by id, preserve base-first order.
        for (id, style) in &over.group {
            match merged.group.iter_mut().find(|(gid, _)| gid == id) {
                Some((_, dst)) => dst.overlay(style),
                None => merged.group.push((id.clone(), style.clone())),
            }
        }
        for (id, style) in &over.peer {
            match merged.peer.iter_mut().find(|(pid, _)| pid == id) {
                Some((_, dst)) => dst.overlay(style),
                None => merged.peer.push((id.clone(), style.clone())),
            }
        }
        // Rules: merge by condition equality (one rule per conjunction).
        for rule in &over.rules {
            match merged.rules.iter_mut().find(|r| r.when == rule.when) {
                Some(dst) => dst.style.overlay(&rule.style),
                None => merged.rules.push(rule.clone()),
            }
        }
        merged.breakpoints = over.breakpoints;
        merged
    }
}

/// Per-slot diff between two resolved styles.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StyleDiff {
    /// Slots that differ between the two compared styles.
    pub changed: Vec<Slot>,
}

impl StyleDiff {
    /// True when nothing changed.
    pub fn is_empty(&self) -> bool {
        self.changed.is_empty()
    }

    /// True if any changed slot requires relayout.
    pub fn needs_layout(&self) -> bool {
        self.changed.iter().any(|s| s.lane() == Lane::Geometric)
    }
}

/// Bitset of slot indices (all slot indices are < 128).
type SlotBits = u128;

fn bits_of(style: &ResolvedStyle) -> SlotBits {
    let mut bits: SlotBits = 0;
    // Zero-allocation walk (v0.5 §3): the diff path runs per rebuild —
    // routing it through `slots()`' scratch Vec broke the
    // allocates-nothing-on-unchanged contract.
    for_each_present_slot!(style, |sl: Slot| bits |= 1u128 << sl.index());
    bits
}

/// Diff `next` against `prev`, returning the changed slots.
///
/// v0.2: computed via a per-field compare into a slot bitmask — O(n) with
/// no allocation and no `contains` scans (plan §3.4.5).
pub fn diff(prev: &ResolvedStyle, next: &ResolvedStyle) -> StyleDiff {
    let pb = bits_of(prev);
    let nb = bits_of(next);
    let mut changed = Vec::new();
    let mut xor = pb ^ nb;
    while xor != 0 {
        let idx = xor.trailing_zeros() as u16;
        xor &= xor - 1;
        if let Some(slot) = slot_from_index(idx) {
            changed.push(slot);
        }
    }
    // Same-slot-set but different values (e.g. width changed, slot present
    // on both sides): fall back to the per-field compare for the shared bits.
    // The shared bits are walked directly (set-bit order = index order) —
    // NOT through `next.slots()`, whose scratch `Vec` allocated on every
    // diff, breaking the rebuild path's zero-alloc-on-unchanged contract
    // (pinned exactly by tests/alloc_contract.rs, v0.5 §3).
    let shared = pb & nb;
    let mut s = shared;
    while s != 0 {
        let idx = s.trailing_zeros() as u16;
        s &= s - 1;
        if let Some(slot) = slot_from_index(idx)
            && slot_differs(slot, prev, next)
        {
            changed.push(slot);
        }
    }
    changed.sort_by_key(|s| s.index());
    changed.dedup();
    StyleDiff { changed }
}

fn slot_from_index(idx: u16) -> Option<Slot> {
    // Reconstruct from a canonical instance list.
    const ALL: &[Slot] = &[
        Slot::Background,
        Slot::ContentColor,
        Slot::Padding,
        Slot::BorderColor,
        Slot::BorderWidth,
        Slot::CornerRadius,
        Slot::Dimensions,
        Slot::MinDimensions,
        Slot::MaxDimensions,
        Slot::Gap,
        Slot::Shadow,
        Slot::Margin,
        Slot::Layout,
        Slot::Opacity,
        Slot::Ring(RingPart::Width),
        Slot::Ring(RingPart::Color),
        Slot::Ring(RingPart::OffsetWidth),
        Slot::Ring(RingPart::OffsetColor),
        Slot::Outline,
        Slot::Cursor,
        Slot::PointerEvents,
        Slot::Position,
        Slot::Inset,
        Slot::ZIndex,
        Slot::Overflow,
        Slot::Visibility,
        Slot::AspectRatio,
        Slot::Grid,
        Slot::Transform,
        Slot::Transition,
        Slot::Animation,
        Slot::BorderStyle,
        Slot::FlexBasis,
        Slot::Order,
        Slot::AlignContent,
        Slot::MarginAuto,
        Slot::GapAxes,
        Slot::ShadowRest,
        Slot::InsetShadows,
        Slot::Gradient(GradientPart::Shape),
        Slot::Gradient(GradientPart::From),
        Slot::Gradient(GradientPart::Via),
        Slot::Gradient(GradientPart::To),
    ];
    ALL.iter().copied().find(|s| s.index() == idx).or_else(|| {
        // Text slots: the enum's discriminants start after the variants above
        // in declaration order; reconstruct by scanning all text properties.
        TEXT_PROPS
            .iter()
            .map(|p| Slot::Text(*p))
            .find(|s| s.index() == idx)
    })
}

const TEXT_PROPS: &[TextProperty] = &[
    TextProperty::FontSize,
    TextProperty::LineHeight,
    TextProperty::FontWeight,
    TextProperty::LetterSpacing,
    TextProperty::Color,
    TextProperty::UnderlineOffset,
    TextProperty::Whitespace,
    TextProperty::TextAlign,
    TextProperty::Select,
    TextProperty::FontStyle,
    TextProperty::TextTransform,
    TextProperty::TextDecoration,
    TextProperty::FontFamily,
    TextProperty::TextOverflow,
    TextProperty::LineClamp,
    TextProperty::FontVariantNumeric,
];

fn slot_differs(slot: Slot, prev: &ResolvedStyle, next: &ResolvedStyle) -> bool {
    match slot {
        Slot::Background => prev.background != next.background,
        Slot::ContentColor => prev.content_color != next.content_color,
        Slot::Padding => prev.padding != next.padding,
        Slot::BorderColor => prev.border_color != next.border_color,
        Slot::BorderWidth => prev.border_width != next.border_width,
        Slot::CornerRadius => prev.corner_radius != next.corner_radius,
        Slot::Dimensions => prev.dimensions != next.dimensions,
        Slot::MinDimensions => prev.min_dimensions != next.min_dimensions,
        Slot::MaxDimensions => prev.max_dimensions != next.max_dimensions,
        Slot::Gap => prev.gap != next.gap,
        Slot::Shadow => prev.shadow != next.shadow,
        Slot::Margin => prev.margin != next.margin,
        Slot::Layout => prev.layout != next.layout,
        Slot::Opacity => prev.opacity != next.opacity,
        Slot::Ring(_) => prev.ring != next.ring,
        Slot::Outline => prev.outline != next.outline,
        Slot::Cursor => prev.cursor != next.cursor,
        Slot::PointerEvents => prev.pointer_events != next.pointer_events,
        Slot::Text(p) => {
            let pv = prev.text.iter().find(|(q, _)| *q == p).map(|(_, v)| *v);
            let nv = next.text.iter().find(|(q, _)| *q == p).map(|(_, v)| *v);
            pv != nv
        }
        Slot::Position => prev.extras.position != next.extras.position,
        Slot::Inset => prev.extras.inset != next.extras.inset,
        Slot::ZIndex => prev.extras.z_index != next.extras.z_index,
        Slot::Overflow => prev.extras.overflow != next.extras.overflow,
        Slot::Visibility => prev.extras.visibility != next.extras.visibility,
        Slot::AspectRatio => prev.extras.aspect_ratio != next.extras.aspect_ratio,
        Slot::Grid => prev.extras.grid != next.extras.grid,
        Slot::Transform => prev.extras.transform != next.extras.transform,
        Slot::Transition => prev.extras.transition != next.extras.transition,
        Slot::Animation => prev.extras.animation != next.extras.animation,
        Slot::BorderStyle => prev.extras.border_style != next.extras.border_style,
        Slot::FlexBasis => prev.extras.flex_basis != next.extras.flex_basis,
        Slot::Order => prev.extras.order != next.extras.order,
        Slot::AlignContent => prev.extras.align_content != next.extras.align_content,
        Slot::MarginAuto => prev.extras.margin_auto != next.extras.margin_auto,
        Slot::GapAxes => prev.extras.gap_axes != next.extras.gap_axes,
        Slot::ShadowRest => prev.extras.shadow_rest != next.extras.shadow_rest,
        Slot::InsetShadows => prev.extras.inset_shadows != next.extras.inset_shadows,
        Slot::Gradient(part) => match part {
            GradientPart::Shape => prev.background_gradient != next.background_gradient,
            GradientPart::From => gradient_edge_changed(prev, next, GradientEdgeSel::From),
            GradientPart::Via => gradient_edge_changed(prev, next, GradientEdgeSel::Via),
            GradientPart::To => gradient_edge_changed(prev, next, GradientEdgeSel::To),
        },
    }
}

/// Which composed stop edge to compare.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GradientEdgeSel {
    From,
    Via,
    To,
}

/// The stop at `sel` (via is `None` when no via class composed).
fn gradient_edge_stop(g: &Gradient, sel: GradientEdgeSel) -> Option<GradientStop> {
    let s = g.stops();
    match sel {
        GradientEdgeSel::From => Some(s.from),
        GradientEdgeSel::Via => s.via,
        GradientEdgeSel::To => Some(s.to),
    }
}

/// Compare ONE gradient edge across two styles (a side without a gradient
/// means "changed" when the other side carries that edge).
fn gradient_edge_changed(prev: &ResolvedStyle, next: &ResolvedStyle, sel: GradientEdgeSel) -> bool {
    match (&prev.background_gradient, &next.background_gradient) {
        (Some(p), Some(n)) => gradient_edge_stop(p, sel) != gradient_edge_stop(n, sel),
        (None, None) => false,
        _ => true,
    }
}
