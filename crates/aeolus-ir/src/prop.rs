//! The longhand property list and its bitset (architecture plan §3.3).
//!
//! [`Prop`] is `#[repr(u16)]` and the discriminant order IS the
//! cascade/sort order: declarations sorted by `prop as u16` are in valid
//! CSS cascade order, so "sort the output" is an integer compare — the
//! class-sort-order surface the plan §5.5 wants from the registry falls
//! out of the same table.
//!
//! [`PropMask`] is one `u128` bitset over the list. The v0.8 per-edge
//! set-mask model (padding/margin/border/radius four-bool masks)
//! generalizes to the whole longhand surface: dominance (`p-4 ⊕ px-8`),
//! layer overlap (`hover:px-8` vs `p-4`) and conflict eviction are bit
//! operations, not table walks. 128 bits cover the full v4 longhand set
//! this pipeline ports; the type asserts the budget in [`crate::tests`].

/// Every CSS **longhand** the V1 pipeline lowers to. Shorthands (`padding`,
/// `border-width`, `inset`, `gap`, `place-*`) never appear: a utility that
/// spells a shorthand lowers to its longhand fan-out (`p-4` → the four
/// `padding-*` edges), which is what makes the mask algebra work.
///
/// Order = cascade/sort order (display/positioning → box → border →
/// background → typography → visual → interaction). Append-only from here:
/// renumbering breaks stored masks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
pub enum Prop {
    // ---- display & positioning (cascade head) --------------------------
    /// `display`.
    Display,
    /// `position`.
    Position,
    /// `top`.
    Top,
    /// `right`.
    Right,
    /// `bottom`.
    Bottom,
    /// `left`.
    Left,
    /// `inset-inline-start` (`start-*`).
    InsetInlineStart,
    /// `inset-inline-end` (`end-*`).
    InsetInlineEnd,
    /// `z-index`.
    ZIndex,
    /// `order`.
    Order,
    /// `grid-row-start` (`row-start-*`).
    GridRowStart,
    /// `grid-row-end` (`row-end-*`).
    GridRowEnd,
    /// `grid-column-start` (`col-start-*`).
    GridColumnStart,
    /// `grid-column-end` (`col-end-*`).
    GridColumnEnd,

    // ---- flex / grid placement -----------------------------------------
    /// `flex-direction`.
    FlexDirection,
    /// `flex-wrap`.
    FlexWrap,
    /// `flex-grow`.
    FlexGrow,
    /// `flex-shrink`.
    FlexShrink,
    /// `flex-basis`.
    FlexBasis,
    /// `justify-content`.
    JustifyContent,
    /// `justify-items`.
    JustifyItems,
    /// `justify-self`.
    JustifySelf,
    /// `align-items`.
    AlignItems,
    /// `align-content`.
    AlignContent,
    /// `align-self`.
    AlignSelf,
    /// `place-content` (longhand pair folded at emit).
    PlaceContent,
    /// `place-items`.
    PlaceItems,
    /// `place-self`.
    PlaceSelf,
    /// `grid-template-columns` (`grid-cols-*`).
    GridTemplateColumns,
    /// `grid-template-rows` (`grid-rows-*`).
    GridTemplateRows,
    /// `grid-auto-columns` (`auto-cols-*`).
    GridAutoColumns,
    /// `grid-auto-rows` (`auto-rows-*`).
    GridAutoRows,
    /// `gap` row axis (`gap-y-*` / `gap-*` fan-out).
    GapRow,
    /// `gap` column axis.
    GapCol,
    /// `columns` (multicol count).
    Columns,

    // ---- box: size, margin, padding -------------------------------------
    /// `aspect-ratio`.
    AspectRatio,
    /// `width`.
    Width,
    /// `height`.
    Height,
    /// `min-width`.
    MinWidth,
    /// `min-height`.
    MinHeight,
    /// `max-width`.
    MaxWidth,
    /// `max-height`.
    MaxHeight,
    /// `padding-top`.
    PaddingTop,
    /// `padding-right`.
    PaddingRight,
    /// `padding-bottom`.
    PaddingBottom,
    /// `padding-left`.
    PaddingLeft,
    /// `margin-top`.
    MarginTop,
    /// `margin-right`.
    MarginRight,
    /// `margin-bottom`.
    MarginBottom,
    /// `margin-left`.
    MarginLeft,

    // ---- border & radius ------------------------------------------------
    /// `border-top-width`.
    BorderTopWidth,
    /// `border-right-width`.
    BorderRightWidth,
    /// `border-bottom-width`.
    BorderBottomWidth,
    /// `border-left-width`.
    BorderLeftWidth,
    /// `border-color` (v4 keeps one color per side at the corpus surface;
    /// per-side colors fold onto the same longhand with per-edge masks).
    BorderColor,
    /// `border-style`.
    BorderStyle,
    /// `border-top-left-radius`.
    BorderTopLeftRadius,
    /// `border-top-right-radius`.
    BorderTopRightRadius,
    /// `border-bottom-right-radius`.
    BorderBottomRightRadius,
    /// `border-bottom-left-radius`.
    BorderBottomLeftRadius,
    /// `outline-width`.
    OutlineWidth,
    /// `outline-color`.
    OutlineColor,
    /// `outline-style`.
    OutlineStyle,
    /// `outline-offset`.
    OutlineOffset,
    /// `--tw-ring-offset-width` (the v4 ring machinery).
    RingOffsetWidth,
    /// `--tw-ring-offset-color`.
    RingOffsetColor,
    /// `--tw-ring-color`.
    RingColor,
    /// `--tw-ring-width` — ring widths land in the `--tw-*` machinery.
    RingWidth,

    // ---- background ------------------------------------------------------
    /// `background-color` (`bg-{color}`).
    Background,
    /// `background-image` (`bg-linear-*` / `bg-radial` / `bg-conic` / `bg-none`).
    BackgroundImage,
    /// `object-fit`.
    ObjectFit,
    /// `object-position`.
    ObjectPosition,

    // ---- typography -------------------------------------------------------
    /// `color` (`text-{color}`).
    Color,
    /// `font-size` (`text-{size}`).
    FontSize,
    /// `line-height` (`leading-*` / the `text-{size}/{leading}` pair).
    LineHeight,
    /// `font-weight`.
    FontWeight,
    /// `font-family`.
    FontFamily,
    /// `font-style` (`italic` / `not-italic`).
    FontStyle,
    /// `letter-spacing`.
    LetterSpacing,
    /// `text-align`.
    TextAlign,
    /// `text-transform`.
    TextTransform,
    /// `text-decoration-line` (`underline` / `overline` / `line-through` /
    /// `no-underline`).
    TextDecorationLine,
    /// `text-underline-offset`.
    TextUnderlineOffset,
    /// `white-space`.
    WhiteSpace,
    /// `word-break` (`break-*` family).
    WordBreak,
    /// `vertical-align`.
    VerticalAlign,

    // ---- visual -----------------------------------------------------------
    /// `overflow-x` (`overflow-x-*`; bare `overflow-*` fans out to both).
    OverflowX,
    /// `overflow-y`.
    OverflowY,
    /// `overscroll-behavior-x`.
    OverscrollX,
    /// `overscroll-behavior-y`.
    OverscrollY,
    /// `visibility`.
    Visibility,
    /// `opacity`.
    Opacity,
    /// `isolation`.
    Isolation,
    /// `box-shadow` (the composed `--tw-shadow` value).
    BoxShadow,

    // ---- interaction (cascade tail) ---------------------------------------
    /// `container-type` (`@container`).
    ContainerType,
    /// `resize`.
    Resize,
    /// `cursor`.
    Cursor,
    /// `pointer-events`.
    PointerEvents,
    /// `user-select`.
    UserSelect,
    /// `fill` (svg).
    Fill,
    /// `stroke` (svg).
    Stroke,
    /// `--tw-shadow-color` (the shadow COLOR machinery; the width preset
    /// is [`Prop::BoxShadow`]).
    ShadowColor,
    /// `text-overflow` (the `truncate` composite's ellipsis leg).
    TextOverflow,
    /// `--tw-translate-x`.
    TranslateX,
    /// `--tw-translate-y`.
    TranslateY,
    /// `--tw-rotate`.
    Rotate,
    /// `--tw-scale-x`.
    ScaleX,
    /// `--tw-scale-y`.
    ScaleY,
    /// `--tw-skew-x`.
    SkewX,
    /// `--tw-skew-y`.
    SkewY,
}

impl Prop {
    /// Every longhand, discriminant order — the audit surface for the
    /// registry (`generate_docs()`) and the mask round-trip tests.
    pub const ALL: &'static [Prop] = &[
        Self::Display,
        Self::Position,
        Self::Top,
        Self::Right,
        Self::Bottom,
        Self::Left,
        Self::InsetInlineStart,
        Self::InsetInlineEnd,
        Self::ZIndex,
        Self::Order,
        Self::GridRowStart,
        Self::GridRowEnd,
        Self::GridColumnStart,
        Self::GridColumnEnd,
        Self::FlexDirection,
        Self::FlexWrap,
        Self::FlexGrow,
        Self::FlexShrink,
        Self::FlexBasis,
        Self::JustifyContent,
        Self::JustifyItems,
        Self::JustifySelf,
        Self::AlignItems,
        Self::AlignContent,
        Self::AlignSelf,
        Self::PlaceContent,
        Self::PlaceItems,
        Self::PlaceSelf,
        Self::GridTemplateColumns,
        Self::GridTemplateRows,
        Self::GridAutoColumns,
        Self::GridAutoRows,
        Self::GapRow,
        Self::GapCol,
        Self::Columns,
        Self::AspectRatio,
        Self::Width,
        Self::Height,
        Self::MinWidth,
        Self::MinHeight,
        Self::MaxWidth,
        Self::MaxHeight,
        Self::PaddingTop,
        Self::PaddingRight,
        Self::PaddingBottom,
        Self::PaddingLeft,
        Self::MarginTop,
        Self::MarginRight,
        Self::MarginBottom,
        Self::MarginLeft,
        Self::BorderTopWidth,
        Self::BorderRightWidth,
        Self::BorderBottomWidth,
        Self::BorderLeftWidth,
        Self::BorderColor,
        Self::BorderStyle,
        Self::BorderTopLeftRadius,
        Self::BorderTopRightRadius,
        Self::BorderBottomRightRadius,
        Self::BorderBottomLeftRadius,
        Self::OutlineWidth,
        Self::OutlineColor,
        Self::OutlineStyle,
        Self::OutlineOffset,
        Self::RingOffsetWidth,
        Self::RingOffsetColor,
        Self::RingColor,
        Self::RingWidth,
        Self::Background,
        Self::BackgroundImage,
        Self::ObjectFit,
        Self::ObjectPosition,
        Self::Color,
        Self::FontSize,
        Self::LineHeight,
        Self::FontWeight,
        Self::FontFamily,
        Self::FontStyle,
        Self::LetterSpacing,
        Self::TextAlign,
        Self::TextTransform,
        Self::TextDecorationLine,
        Self::TextUnderlineOffset,
        Self::WhiteSpace,
        Self::WordBreak,
        Self::VerticalAlign,
        Self::OverflowX,
        Self::OverflowY,
        Self::OverscrollX,
        Self::OverscrollY,
        Self::Visibility,
        Self::Opacity,
        Self::Isolation,
        Self::BoxShadow,
        Self::ContainerType,
        Self::Resize,
        Self::Cursor,
        Self::PointerEvents,
        Self::UserSelect,
        Self::Fill,
        Self::Stroke,
        Self::ShadowColor,
        Self::TextOverflow,
        Self::TranslateX,
        Self::TranslateY,
        Self::Rotate,
        Self::ScaleX,
        Self::ScaleY,
        Self::SkewX,
        Self::SkewY,
    ];

    /// The CSS longhand name (`padding-top`, `grid-template-columns`, ...).
    /// Table-shaped by design: one arm per longhand, auditable against the
    /// tailwindcss source (the workspace `too_many_lines` rationale).
    pub const fn name(self) -> &'static str {
        match self {
            Prop::Display => "display",
            Prop::Position => "position",
            Prop::Top => "top",
            Prop::Right => "right",
            Prop::Bottom => "bottom",
            Prop::Left => "left",
            Prop::InsetInlineStart => "inset-inline-start",
            Prop::InsetInlineEnd => "inset-inline-end",
            Prop::ZIndex => "z-index",
            Prop::Order => "order",
            Prop::GridRowStart => "grid-row-start",
            Prop::GridRowEnd => "grid-row-end",
            Prop::GridColumnStart => "grid-column-start",
            Prop::GridColumnEnd => "grid-column-end",
            Prop::FlexDirection => "flex-direction",
            Prop::FlexWrap => "flex-wrap",
            Prop::FlexGrow => "flex-grow",
            Prop::FlexShrink => "flex-shrink",
            Prop::FlexBasis => "flex-basis",
            Prop::JustifyContent => "justify-content",
            Prop::JustifyItems => "justify-items",
            Prop::JustifySelf => "justify-self",
            Prop::AlignItems => "align-items",
            Prop::AlignContent => "align-content",
            Prop::AlignSelf => "align-self",
            Prop::PlaceContent => "place-content",
            Prop::PlaceItems => "place-items",
            Prop::PlaceSelf => "place-self",
            Prop::GridTemplateColumns => "grid-template-columns",
            Prop::GridTemplateRows => "grid-template-rows",
            Prop::GridAutoColumns => "grid-auto-columns",
            Prop::GridAutoRows => "grid-auto-rows",
            Prop::GapRow => "row-gap",
            Prop::GapCol => "column-gap",
            Prop::Columns => "columns",
            Prop::AspectRatio => "aspect-ratio",
            Prop::Width => "width",
            Prop::Height => "height",
            Prop::MinWidth => "min-width",
            Prop::MinHeight => "min-height",
            Prop::MaxWidth => "max-width",
            Prop::MaxHeight => "max-height",
            Prop::PaddingTop => "padding-top",
            Prop::PaddingRight => "padding-right",
            Prop::PaddingBottom => "padding-bottom",
            Prop::PaddingLeft => "padding-left",
            Prop::MarginTop => "margin-top",
            Prop::MarginRight => "margin-right",
            Prop::MarginBottom => "margin-bottom",
            Prop::MarginLeft => "margin-left",
            Prop::BorderTopWidth => "border-top-width",
            Prop::BorderRightWidth => "border-right-width",
            Prop::BorderBottomWidth => "border-bottom-width",
            Prop::BorderLeftWidth => "border-left-width",
            Prop::BorderColor => "border-color",
            Prop::BorderStyle => "border-style",
            Prop::BorderTopLeftRadius => "border-top-left-radius",
            Prop::BorderTopRightRadius => "border-top-right-radius",
            Prop::BorderBottomRightRadius => "border-bottom-right-radius",
            Prop::BorderBottomLeftRadius => "border-bottom-left-radius",
            Prop::OutlineWidth => "outline-width",
            Prop::OutlineColor => "outline-color",
            Prop::OutlineStyle => "outline-style",
            Prop::OutlineOffset => "outline-offset",
            Prop::RingOffsetWidth => "--tw-ring-offset-width",
            Prop::RingOffsetColor => "--tw-ring-offset-color",
            Prop::RingColor => "--tw-ring-color",
            Prop::RingWidth => "--tw-ring-width",
            Prop::Background => "background-color",
            Prop::BackgroundImage => "background-image",
            Prop::ObjectFit => "object-fit",
            Prop::ObjectPosition => "object-position",
            Prop::Color => "color",
            Prop::FontSize => "font-size",
            Prop::LineHeight => "line-height",
            Prop::FontWeight => "font-weight",
            Prop::FontFamily => "font-family",
            Prop::FontStyle => "font-style",
            Prop::LetterSpacing => "letter-spacing",
            Prop::TextAlign => "text-align",
            Prop::TextTransform => "text-transform",
            Prop::TextDecorationLine => "text-decoration-line",
            Prop::TextUnderlineOffset => "text-underline-offset",
            Prop::WhiteSpace => "white-space",
            Prop::WordBreak => "word-break",
            Prop::VerticalAlign => "vertical-align",
            Prop::OverflowX => "overflow-x",
            Prop::OverflowY => "overflow-y",
            Prop::OverscrollX => "overscroll-behavior-x",
            Prop::OverscrollY => "overscroll-behavior-y",
            Prop::Visibility => "visibility",
            Prop::Opacity => "opacity",
            Prop::Isolation => "isolation",
            Prop::BoxShadow => "box-shadow",
            Prop::ContainerType => "container-type",
            Prop::Resize => "resize",
            Prop::Cursor => "cursor",
            Prop::PointerEvents => "pointer-events",
            Prop::UserSelect => "user-select",
            Prop::Fill => "fill",
            Prop::Stroke => "stroke",
            Prop::ShadowColor => "--tw-shadow-color",
            Prop::TextOverflow => "text-overflow",
            Prop::TranslateX => "--tw-translate-x",
            Prop::TranslateY => "--tw-translate-y",
            Prop::Rotate => "--tw-rotate",
            Prop::ScaleX => "--tw-scale-x",
            Prop::ScaleY => "--tw-scale-y",
            Prop::SkewX => "--tw-skew-x",
            Prop::SkewY => "--tw-skew-y",
        }
    }
}

/// A bitset over [`Prop`] — one `u128`, `Copy`, `const`-constructible.
///
/// This is the v0.8 per-edge set-mask idea generalized to the whole
/// longhand list: a [`crate::Layer`] carries the mask of the longhands its
/// declarations set, so two layers' interaction (`p-4` vs `hover:px-8`,
/// `size-4` vs `w-4`) is `union`/`contains` — no per-backend tables.
///
/// The v4 longhand surface this pipeline ports is 96 longhands; the
/// remaining 32 bits are headroom (append-only [`Prop`] growth). The
/// budget is asserted, not assumed: see `prop_mask_fits_u128` in tests.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct PropMask(u128);

impl PropMask {
    /// The empty mask (no longhands set).
    pub const EMPTY: Self = Self(0);

    /// Build from a longhand slice (the registry row target lists).
    pub const fn of(props: &[Prop]) -> Self {
        let mut m = 0u128;
        let mut i = 0;
        // `while` is the const-eval loop form (no `for` in const fn).
        while i < props.len() {
            m |= 1 << (props[i] as u16);
            i += 1;
        }
        Self(m)
    }

    /// Set one longhand.
    pub const fn insert(self, p: Prop) -> Self {
        Self(self.0 | 1 << (p as u16))
    }

    /// Is the longhand set?
    pub const fn contains(self, p: Prop) -> bool {
        self.0 & 1 << (p as u16) != 0
    }

    /// Are ALL of `other`'s bits set? (layer dominance: `px-8` covers the
    /// x-edges `p-4` set).
    pub const fn covers(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// No longhands set.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Union (`p-4 ⊕ px-8` — the overlay merge).
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Intersection (the edges two layers BOTH set — where a cascade
    /// decision is needed).
    pub const fn intersect(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    /// Difference (`self` minus `other` — what a later layer evicts).
    pub const fn difference(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }

    /// How many longhands are set (diagnostics + the registry's fan-out
    /// audit).
    pub const fn len(self) -> u32 {
        self.0.count_ones()
    }

    /// Call `f` for every set longhand, in discriminant (cascade) order.
    /// Callback form: the static form never allocates.
    pub fn for_each(self, mut f: impl FnMut(Prop)) {
        let mut bits = self.0;
        while bits != 0 {
            let idx = bits.trailing_zeros() as u16;
            // `from_idx` is total: every bit below 96 is a valid Prop.
            f(Self::from_idx(idx));
            bits &= bits - 1; // clear the lowest set bit
        }
    }

    /// The longhand at bit index `idx` (total for every idx < [`Prop::ALL`]
    /// length — the mask never sets bits beyond it).
    const fn from_idx(idx: u16) -> Prop {
        // Transmute-free reconstruction through the const table: bit index
        // == discriminant by construction (`of`/`insert` shift by
        // `p as u16`), so a scan over ALL finds the arm. Const-friendly.
        let mut i = 0;
        while i < Prop::ALL.len() {
            if Prop::ALL[i] as u16 == idx {
                return Prop::ALL[i];
            }
            i += 1;
        }
        // Unreachable for masks this type can build (bits only come from
        // `Prop` values); a named `unreachable` arm keeps const-eval happy.
        Prop::Display
    }

    /// The set longhands as a Vec (test/report convenience — the static
    /// path uses [`for_each`](Self::for_each)).
    pub fn to_vec(self) -> Vec<Prop> {
        let mut out = Vec::new();
        self.for_each(|p| out.push(p));
        out
    }
}
