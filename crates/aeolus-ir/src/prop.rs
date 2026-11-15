//! The longhand property list and its bitset (architecture plan §3.3).
//!
//! [`Prop`] is `#[repr(u16)]` and the discriminant order IS the
//! cascade/sort order: declarations sorted by `prop as u16` are in valid
//! CSS cascade order, so "sort the output" is an integer compare — the
//! class-sort-order surface the plan §5.5 wants from the registry falls
//! out of the same table.
//!
//! [`PropMask`] is a two-word `u128` bitset over the list (the mask
//! outgrew one word when the filter/backdrop layers pushed the longhand
//! count past 128). The v0.8 per-edge
//! set-mask model (padding/margin/border/radius four-bool masks)
//! generalizes to the whole longhand surface: dominance (`p-4 ⊕ px-8`),
//! layer overlap (`hover:px-8` vs `p-4`) and conflict eviction are bit
//! operations, not table walks. The two words cover the full v4 longhand
//! set this pipeline ports; the type asserts the budget in the crate's
//! test module.

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
    /// `--tw-gradient-from` — the gradient's from-stop colour
    /// (`from-{color}`), v4's own custom-property longhand set: the
    /// stops are SEPARATE declarations that compose with the
    /// `background-image` direction, the same longhand split the
    /// per-edge families took (the shorthand-shaped fold let the last
    /// stop replace the direction in the cascade).
    GradientFrom,
    /// `--tw-gradient-via` — the via-stop colour (`via-{color}`).
    GradientVia,
    /// `--tw-gradient-to` — the to-stop colour (`to-{color}`).
    GradientTo,
    /// `--tw-gradient-from-position` — the from stop's percentage
    /// (`from-{p}%`), `Val::Pct`; absent = the v4 `@property` 0% default.
    GradientFromPosition,
    /// `--tw-gradient-via-position` — the via stop's percentage
    /// (`via-{p}%`), the 50% default.
    GradientViaPosition,
    /// `--tw-gradient-to-position` — the to stop's percentage
    /// (`to-{p}%`), the 100% default.
    GradientToPosition,
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
    // ---- motion (cascade tail; the plan P6 row) --------------------------
    /// `transition-property` (the `transition-*` set families; the
    /// keyword text is the property list the row's table carries).
    TransitionProperty,
    /// `transition-duration` (`duration-*`), `Val::Num` milliseconds.
    TransitionDuration,
    /// `transition-timing-function` (`ease-*`; the keyword text is the
    /// curve the row's table carries).
    TransitionTimingFunction,
    /// `transition-delay` (`delay-*`), `Val::Num` milliseconds.
    TransitionDelay,
    /// `animation` (`animate-*`; the keyword text is the animation
    /// shorthand the row's table carries — the v4 default theme's
    /// `--animate-*` values).
    Animation,
    // ---- filters (cascade tail; the plan §3.7 backlog row's pipeline
    // half) ---------------------------------------------------------------
    /// `--tw-blur` (`blur-*`; the keyword text is the blur function the
    /// row's table carries — the v4 default theme's `--blur-*` values;
    /// the `none` arm's off text is the v4 one-space form). Arbitrary
    /// lengths ride the `Px` slot.
    FilterBlur,
    /// `--tw-brightness` (`brightness-*`), the percent-scale `Num`
    /// slots (v4's bare-integer percent form; the unit lives in the
    /// render tables, the motion layer's precedent).
    FilterBrightness,
    /// `--tw-contrast` (`contrast-*`), the percent-scale `Num` slots.
    FilterContrast,
    /// `--tw-grayscale` (`grayscale-*`; the bare root carries the v4
    /// 100% default), the percent-scale `Num` slots.
    FilterGrayscale,
    /// `--tw-hue-rotate` (`hue-rotate-*`), the degree `Num` slots; the
    /// row's negative flag rides (`-hue-rotate-30`).
    FilterHueRotate,
    /// `--tw-invert` (`invert-*`; the bare root carries the v4 100%
    /// default), the percent-scale `Num` slots.
    FilterInvert,
    /// `--tw-saturate` (`saturate-*`), the percent-scale `Num` slots.
    FilterSaturate,
    /// `--tw-sepia` (`sepia-*`; the bare root carries the v4 100%
    /// default), the percent-scale `Num` slots.
    FilterSepia,
    /// `--tw-drop-shadow` (`drop-shadow-*`; the keyword text is the
    /// composed drop-shadow list the row's table carries — the v4
    /// default theme's `--drop-shadow-*` presets, the bare root the
    /// two-layer default). Arbitrary single shadows ride the text
    /// boundaries; the `{color}` recolour arm is the documented
    /// boundary (the shadow tier's own colour machinery reaches it).
    FilterDropShadow,
    /// `filter` — the COMPOSED property (`filter`, `filter-none`,
    /// `filter-[…]`): the keyword text is the composed chain the
    /// primitives' var references resolve (the CSS arm re-declares it
    /// beside every primitive's var line, v4's own emission form).
    Filter,
    // ---- backdrop filters (cascade tail; the plan §3.7 backlog row's
    // second half) --------------------------------------------------------
    /// `--tw-backdrop-blur` (`backdrop-blur-*`; the row shares the
    /// filter half's scale — v4's own `--backdrop-blur`→`--blur` theme
    /// fallback).
    BackdropBlur,
    /// `--tw-backdrop-brightness` (`backdrop-brightness-*`), the
    /// percent-scale `Num` slots.
    BackdropBrightness,
    /// `--tw-backdrop-contrast` (`backdrop-contrast-*`), the
    /// percent-scale `Num` slots.
    BackdropContrast,
    /// `--tw-backdrop-grayscale` (`backdrop-grayscale-*`; the bare root
    /// carries the v4 100% default), the percent-scale `Num` slots.
    BackdropGrayscale,
    /// `--tw-backdrop-hue-rotate` (`backdrop-hue-rotate-*`), the degree
    /// `Num` slots; the row's negative flag rides
    /// (`-backdrop-hue-rotate-15`).
    BackdropHueRotate,
    /// `--tw-backdrop-invert` (`backdrop-invert-*`; the bare root
    /// carries the v4 100% default), the percent-scale `Num` slots.
    BackdropInvert,
    /// `--tw-backdrop-opacity` (`backdrop-opacity-*`) — the backdrop
    /// chain's OWN ninth var, no filter-half counterpart.
    BackdropOpacity,
    /// `--tw-backdrop-saturate` (`backdrop-saturate-*`), the
    /// percent-scale `Num` slots.
    BackdropSaturate,
    /// `--tw-backdrop-sepia` (`backdrop-sepia-*`; the bare root carries
    /// the v4 100% default), the percent-scale `Num` slots.
    BackdropSepia,
    /// `backdrop-filter` — the COMPOSED property (`backdrop-filter`,
    /// `backdrop-filter-none`, `backdrop-filter-[…]`); the CSS arm
    /// renders the `-webkit-` prefix beside the standard property
    /// (v4's own emission form).
    BackdropFilter,
    // ---- grid auto-flow (cascade tail; the plan §3.7 grid row's
    // auto-flow half) --------------------------------------------------
    /// `grid-auto-flow` (`grid-flow-*`): the keyword slots carry the
    /// v4.1 scale's own values (`row`, `column`, `dense`, the dense
    /// pairs) — the track-folding keywords the auto-placement engine
    /// obeys. The v0.x plan carries no auto-flow slot: the bridge
    /// reports the longhand (the grid-auto-rows precedent).
    GridAutoFlow,
    // ---- pseudo-element content (cascade tail; the wider v4 hint
    // table's string half) -----------------------------------------------
    /// `content` — the pseudo-element paint property the
    /// `content-["…"]` arbitrary spelling writes (the `string:` hint's
    /// own target). The root's keyword half stays `align-content` (the
    /// tw_merge oracle's group name) — the string form routes here
    /// through the value pass's hint targets. The v0.x plan carries no
    /// content slot: the bridge reports the longhand (the
    /// grid-auto-flow precedent).
    Content,
    // ---- text decoration (cascade tail; the §3.7 typography surface's
    // decoration half) -----------------------------------------------------
    /// `text-decoration-color` (`decoration-{color}`): the colour the
    /// composed decoration lines render (the underline AND line-through
    /// legs, v4's own property). Theme tokens lower through the theme
    /// pass's tiered table (the `bg-{color}` grammar); GPUI's pinned
    /// surface carries the `text_decoration_color` method.
    TextDecorationColor,
    /// `text-decoration-thickness` (`decoration-{thickness}`): the
    /// static keywords (`auto`, `from-font`) ride the row's keyword
    /// table, the bare NON-NEGATIVE integers (the v4
    /// `isPositiveInteger` round-trip rule) lower as pixel `Num`s.
    /// GPUI's pinned surface carries the `text_decoration_0..8`
    /// thickness methods.
    TextDecorationThickness,
    /// `text-decoration-style` (`decoration-solid`/`double`/`dotted`/
    /// `dashed`/`wavy`): the v4 static utilities' keyword slots. GPUI
    /// renders a single line: solid and wavy express (the pinned
    /// `text_decoration_solid`/`text_decoration_wavy` methods), the
    /// other three are the per-decl typed refusal (the emit gate's
    /// row-text verdict, the box-shadow preset gate's pattern).
    TextDecorationStyle,
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
        Self::GradientFrom,
        Self::GradientVia,
        Self::GradientTo,
        Self::GradientFromPosition,
        Self::GradientViaPosition,
        Self::GradientToPosition,
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
        Self::TransitionProperty,
        Self::TransitionDuration,
        Self::TransitionTimingFunction,
        Self::TransitionDelay,
        Self::Animation,
        Self::FilterBlur,
        Self::FilterBrightness,
        Self::FilterContrast,
        Self::FilterGrayscale,
        Self::FilterHueRotate,
        Self::FilterInvert,
        Self::FilterSaturate,
        Self::FilterSepia,
        Self::FilterDropShadow,
        Self::Filter,
        Self::BackdropBlur,
        Self::BackdropBrightness,
        Self::BackdropContrast,
        Self::BackdropGrayscale,
        Self::BackdropHueRotate,
        Self::BackdropInvert,
        Self::BackdropOpacity,
        Self::BackdropSaturate,
        Self::BackdropSepia,
        Self::BackdropFilter,
        Self::GridAutoFlow,
        Self::Content,
        Self::TextDecorationColor,
        Self::TextDecorationThickness,
        Self::TextDecorationStyle,
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
            Prop::GridAutoFlow => "grid-auto-flow",
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
            Prop::GradientFrom => "--tw-gradient-from",
            Prop::GradientVia => "--tw-gradient-via",
            Prop::GradientTo => "--tw-gradient-to",
            Prop::GradientFromPosition => "--tw-gradient-from-position",
            Prop::GradientViaPosition => "--tw-gradient-via-position",
            Prop::GradientToPosition => "--tw-gradient-to-position",
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
            Prop::TransitionProperty => "transition-property",
            Prop::TransitionDuration => "transition-duration",
            Prop::TransitionTimingFunction => "transition-timing-function",
            Prop::TransitionDelay => "transition-delay",
            Prop::Animation => "animation",
            // The filter family renders its var-reference names (the
            // shadow tier's `--tw-shadow-color` precedent): the CSS
            // arm's var lines and the metamorphic comparison key are
            // the same string.
            Prop::FilterBlur => "--tw-blur",
            Prop::FilterBrightness => "--tw-brightness",
            Prop::FilterContrast => "--tw-contrast",
            Prop::FilterGrayscale => "--tw-grayscale",
            Prop::FilterHueRotate => "--tw-hue-rotate",
            Prop::FilterInvert => "--tw-invert",
            Prop::FilterSaturate => "--tw-saturate",
            Prop::FilterSepia => "--tw-sepia",
            Prop::FilterDropShadow => "--tw-drop-shadow",
            Prop::Filter => "filter",
            // The backdrop family renders its var-reference names too —
            // the composed property's name is the standard spelling (the
            // `-webkit-` prefix is the CSS arm's own render form).
            Prop::BackdropBlur => "--tw-backdrop-blur",
            Prop::BackdropBrightness => "--tw-backdrop-brightness",
            Prop::BackdropContrast => "--tw-backdrop-contrast",
            Prop::BackdropGrayscale => "--tw-backdrop-grayscale",
            Prop::BackdropHueRotate => "--tw-backdrop-hue-rotate",
            Prop::BackdropInvert => "--tw-backdrop-invert",
            Prop::BackdropOpacity => "--tw-backdrop-opacity",
            Prop::BackdropSaturate => "--tw-backdrop-saturate",
            Prop::BackdropSepia => "--tw-backdrop-sepia",
            Prop::BackdropFilter => "backdrop-filter",
            Prop::Content => "content",
            Prop::TextDecorationColor => "text-decoration-color",
            Prop::TextDecorationThickness => "text-decoration-thickness",
            Prop::TextDecorationStyle => "text-decoration-style",
        }
    }
}

/// A bitset over [`Prop`] — one `u128`, `Copy`, `const`-constructible.
/// The plan §3.3 sketch's `u128×k` mask, at k=2: the longhand list
/// outgrew one word when the CSS filter family joined the cascade tail
/// (131 longhands), so the mask carries TWO `u128` words — the low
/// word holds discriminants 0..128, the high word the rest. The
/// algebra is word-blind: every operation walks both words, and the
/// tests pin a one-word implementation cannot pass
/// (`mask_two_word.rs`).
///
/// This is the v0.8 per-edge set-mask idea generalized to the whole
/// longhand list: a [`crate::Layer`] carries the mask of the longhands its
/// declarations set, so two layers' interaction (`p-4` vs `hover:px-8`,
/// `size-4` vs `w-4`) is `union`/`contains` — no per-backend tables.
///
/// The budget is asserted, not assumed: the two words hold 256 bits
/// against the current 146-longhand surface (the filter + backdrop
/// layers' twenty, the content longhand, and the decoration half's
/// three are the latest append-only blocks; append-only growth is the
/// plan's own rule) — see
/// `prop_mask_fits_two_words` in tests.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct PropMask([u128; 2]);

/// The bits per mask word (the layout the discriminant→(word, bit)
/// split computes from).
const WORD_BITS: u32 = 128;

impl PropMask {
    /// The empty mask (no longhands set).
    pub const EMPTY: Self = Self([0; 2]);

    /// Build from a longhand slice (the registry row target lists).
    pub const fn of(props: &[Prop]) -> Self {
        let mut m = Self::EMPTY;
        let mut i = 0;
        // `while` is the const-eval loop form (no `for` in const fn).
        while i < props.len() {
            m = m.insert(props[i]);
            i += 1;
        }
        m
    }

    /// Set one longhand.
    pub const fn insert(self, p: Prop) -> Self {
        let bit = p as u16 as u32;
        let word = (bit / WORD_BITS) as usize;
        let mut words = self.0;
        words[word] |= 1 << (bit % WORD_BITS);
        Self(words)
    }

    /// Is the longhand set?
    pub const fn contains(self, p: Prop) -> bool {
        let bit = p as u16 as u32;
        self.0[(bit / WORD_BITS) as usize] & 1 << (bit % WORD_BITS) != 0
    }

    /// Are ALL of `other`'s bits set? (layer dominance: `px-8` covers the
    /// x-edges `p-4` set).
    pub const fn covers(self, other: Self) -> bool {
        self.0[0] & other.0[0] == other.0[0] && self.0[1] & other.0[1] == other.0[1]
    }

    /// No longhands set.
    pub const fn is_empty(self) -> bool {
        self.0[0] == 0 && self.0[1] == 0
    }

    /// Union (`p-4 ⊕ px-8` — the overlay merge).
    pub const fn union(self, other: Self) -> Self {
        Self([self.0[0] | other.0[0], self.0[1] | other.0[1]])
    }

    /// Intersection (the edges two layers BOTH set — where a cascade
    /// decision is needed).
    pub const fn intersect(self, other: Self) -> Self {
        Self([self.0[0] & other.0[0], self.0[1] & other.0[1]])
    }

    /// Difference (`self` minus `other` — what a later layer evicts).
    pub const fn difference(self, other: Self) -> Self {
        Self([self.0[0] & !other.0[0], self.0[1] & !other.0[1]])
    }

    /// How many longhands are set (diagnostics + the registry's fan-out
    /// audit).
    pub const fn len(self) -> u32 {
        self.0[0].count_ones() + self.0[1].count_ones()
    }

    /// Call `f` for every set longhand, in discriminant (cascade) order.
    /// Callback form: the static form never allocates.
    pub fn for_each(self, mut f: impl FnMut(Prop)) {
        // Low word first, then high: discriminant order IS bit order
        // across the two words.
        let mut word = 0usize;
        while word < self.0.len() {
            let mut bits = self.0[word];
            while bits != 0 {
                let idx = word as u32 * WORD_BITS + bits.trailing_zeros();
                // `from_idx` is total: every bit the mask can set is a
                // valid Prop (bits only come from `Prop` values).
                f(Self::from_idx(idx as u16));
                bits &= bits - 1; // clear the lowest set bit
            }
            word += 1;
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
