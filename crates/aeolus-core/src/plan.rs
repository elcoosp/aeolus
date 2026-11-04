//! The `StylePlan` IR — Aeolus's public intermediate representation.
//!
//! A `StylePlan` is *not* a flat resolved style: it carries a `base` layer plus
//! conditional layers, so each backend can lower conditions into its native
//! cascade (`PropertyStack`+`Selector` on Masonry, nested selector maps on
//! Floem, state-keyed closures on Iced) instead of re-implementing variants.
//!
//! Slot types are named after Masonry's property types where they exist
//! (`Background`, `BorderColor`, `BorderWidth`, `CornerRadius`, `Dimensions`,
//! `Padding`, `BoxShadow`, `ContentColor`, `Gap`) so the Xilem/Masonry
//! lowering is nearly mechanical.

use crate::ast::Breakpoint;
use crate::color::Color;

/// The complete styling plan for one element.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StylePlan {
    /// Unconditional properties.
    pub base: ResolvedStyle,
    /// Conditional layers (`hover:bg-*`, `focus-visible:ring-*`, ...).
    pub layers: Vec<ConditionalLayer>,
    /// `sm:`..`2xl:` responsive overrides.
    pub responsive: BTreeMap<Breakpoint, ResolvedStyle>,
    /// `@sm:`.. container-query overrides.
    pub container: BTreeMap<Breakpoint, ResolvedStyle>,
    /// `dark:` overrides.
    pub dark: Option<ResolvedStyle>,
    /// `group-*:` layers keyed by group label (empty label = default group).
    pub group: Vec<(GroupId, ResolvedStyle)>,
    /// `peer-*:` layers keyed by peer label.
    pub peer: Vec<(PeerId, ResolvedStyle)>,
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
    /// `bg-*` solid fill.
    pub background: Option<Background>,
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
    /// `shadow-*` preset (single layer).
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
}

impl Slot {
    /// Does changing this slot require relayout (geometric lane), or only a
    /// repaint (cosmetic lane)? Mirrors Masonry's `core_property_changed`
    /// lane assignment (masonry_core/src/properties/mod.rs).
    pub fn lane(self) -> Lane {
        match self {
            Slot::Background
            | Slot::ContentColor
            | Slot::BorderColor
            | Slot::Opacity
            | Slot::Cursor
            | Slot::PointerEvents
            | Slot::Text(TextProperty::Color)
            | Slot::Text(TextProperty::UnderlineOffset)
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

/// `bg-*` — solid color in v1 (gradients parse but warn as unsupported).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Background(pub Color);

/// `text-{color}` / icon color.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ContentColor(pub Color);

/// `p-*/px-*/py-*/pt-*/pr-*/pb-*/pl-*` in px.
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
}

/// `m-*` family (same shape as padding; separate type so slots stay distinct).
pub type Margin = Padding;

/// `border-{color}`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BorderColor(pub Color);

/// `border`, `border-2`, `border-x` etc. in px.
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
}

/// `rounded-*` in px.
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
}

/// `gap-*` in px.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gap(pub f32);

/// `shadow-*` — one layer per shadow (v1: preset shadows, single layer).
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
    /// `shadow-inner` — inset shadows are parsed but v1 backends drop them.
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
}

/// The mouse cursor (`cursor-*`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cursor {
    /// `cursor-default`.
    Default,
    /// `cursor-pointer`.
    Pointer,
    /// `cursor-not-allowed`.
    NotAllowed,
    /// `cursor-text`.
    Text,
    /// `cursor-grab`.
    Grab,
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
}

/// The value of one [`TextProperty`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TextValue {
    /// px (resolved from `--text-*` tokens).
    Px(f32),
    /// font-weight 100..=900.
    Weight(u16),
    /// letter-spacing px (negative for `tracking-tighter`).
    Spacing(f32),
    /// `whitespace-nowrap`, `select-none` etc. are keywords.
    Keyword(&'static str),
    /// text color.
    Color(Color),
}

impl ResolvedStyle {
    /// Iterate over set slots (for diffing / capability checks).
    pub fn slots(&self) -> Vec<Slot> {
        let mut v = Vec::with_capacity(20);
        if self.background.is_some() {
            v.push(Slot::Background);
        }
        if self.content_color.is_some() {
            v.push(Slot::ContentColor);
        }
        if self.padding.is_some() {
            v.push(Slot::Padding);
        }
        if self.border_color.is_some() {
            v.push(Slot::BorderColor);
        }
        if self.border_width.is_some() {
            v.push(Slot::BorderWidth);
        }
        if self.corner_radius.is_some() {
            v.push(Slot::CornerRadius);
        }
        if self.dimensions.is_some() {
            v.push(Slot::Dimensions);
        }
        if self.min_dimensions.is_some() {
            v.push(Slot::MinDimensions);
        }
        if self.max_dimensions.is_some() {
            v.push(Slot::MaxDimensions);
        }
        if self.gap.is_some() {
            v.push(Slot::Gap);
        }
        if self.shadow.is_some() {
            v.push(Slot::Shadow);
        }
        if self.margin.is_some() {
            v.push(Slot::Margin);
        }
        if self.layout.is_some() {
            v.push(Slot::Layout);
        }
        if self.opacity.is_some() {
            v.push(Slot::Opacity);
        }
        if self.ring.is_some() {
            v.push(Slot::Ring(RingPart::Width));
        }
        if self.outline.is_some() {
            v.push(Slot::Outline);
        }
        if self.cursor.is_some() {
            v.push(Slot::Cursor);
        }
        if self.pointer_events.is_some() {
            v.push(Slot::PointerEvents);
        }
        for (p, _) in &self.text {
            if !v.contains(&Slot::Text(*p)) {
                v.push(Slot::Text(*p));
            }
        }
        v
    }

    /// Overlay `other` on top of `self` (later layers win per slot).
    pub fn overlay(&mut self, other: &ResolvedStyle) {
        if other.background.is_some() {
            self.background = other.background;
        }
        if other.content_color.is_some() {
            self.content_color = other.content_color;
        }
        if other.padding.is_some() {
            self.padding = other.padding;
        }
        if other.border_color.is_some() {
            self.border_color = other.border_color;
        }
        if other.border_width.is_some() {
            self.border_width = other.border_width;
        }
        if other.corner_radius.is_some() {
            self.corner_radius = other.corner_radius;
        }
        if other.dimensions.is_some() {
            self.dimensions = other.dimensions;
        }
        if other.min_dimensions.is_some() {
            self.min_dimensions = other.min_dimensions;
        }
        if other.max_dimensions.is_some() {
            self.max_dimensions = other.max_dimensions;
        }
        if other.gap.is_some() {
            self.gap = other.gap;
        }
        if other.shadow.is_some() {
            self.shadow = other.shadow.clone();
        }
        if other.margin.is_some() {
            self.margin = other.margin;
        }
        if other.layout.is_some() {
            self.layout = other.layout;
        }
        if other.opacity.is_some() {
            self.opacity = other.opacity;
        }
        if other.ring.is_some() {
            self.ring = other.ring;
        }
        if other.outline.is_some() {
            self.outline = other.outline;
        }
        if other.cursor.is_some() {
            self.cursor = other.cursor;
        }
        if other.pointer_events.is_some() {
            self.pointer_events = other.pointer_events;
        }
        for (p, val) in &other.text {
            if let Some(slot) = self.text.iter_mut().find(|(q, _)| q == p) {
                slot.1 = *val;
            } else {
                self.text.push((*p, *val));
            }
        }
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

/// Diff `next` against `prev`, returning the changed slots.
pub fn diff(prev: &ResolvedStyle, next: &ResolvedStyle) -> StyleDiff {
    let mut changed = Vec::new();
    for slot in next.slots() {
        let changed_slot = match slot {
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
        };
        if changed_slot {
            changed.push(slot);
        }
    }
    // Slots that disappeared.
    for slot in prev.slots() {
        if !next.slots().contains(&slot) && !changed.contains(&slot) {
            changed.push(slot);
        }
    }
    StyleDiff { changed }
}

use std::collections::BTreeMap;
