//! The committed registry rows — the source of truth. The P2 xtask
//! generates the phf/trie, the docs and the linter tables FROM these;
//! `generate_docs()` proves the claim, and the row order doubles as the
//! canonical utility progression the class-sort seam reads
//! (`registry::sort_index` — plan §5.5's "official order from the
//! registry").
//!
//! Row order within a root is the disambiguation order. Group names are
//! the `tw_merge` 0.1.22 table's own strings (the
//! `aeolus_core::conflict::collision_group` oracle) — the registry tests
//! pin the two tables together over the golden corpus, so a drift is a
//! CI failure, not a backend surprise. Roots the oracle leaves
//! `None` (transforms, `truncate`, `fill`/`stroke`) carry a registry-local
//! group name and are marked as such in their `v4` reference comment.

use crate::tables;
use crate::tables::FLEX_TRIPLETS;
use crate::{UtilityRow, ValueKind};
use aeolus_ir::Prop;

// -- Shared longhand target slices (TRBL / TL-TR-BR-BL order) ---------------

/// The four padding edges (TRBL) — the padding-utility fan-out order.
pub const PADDING: &[Prop] = &[
    Prop::PaddingTop,
    Prop::PaddingRight,
    Prop::PaddingBottom,
    Prop::PaddingLeft,
];
const PADDING_X: &[Prop] = &[Prop::PaddingRight, Prop::PaddingLeft];
const PADDING_Y: &[Prop] = &[Prop::PaddingTop, Prop::PaddingBottom];
const PADDING_T: &[Prop] = &[Prop::PaddingTop];
const PADDING_R: &[Prop] = &[Prop::PaddingRight];
const PADDING_B: &[Prop] = &[Prop::PaddingBottom];
const PADDING_L: &[Prop] = &[Prop::PaddingLeft];

/// The four margin edges (TRBL) — the margin-utility fan-out order.
pub const MARGIN: &[Prop] = &[
    Prop::MarginTop,
    Prop::MarginRight,
    Prop::MarginBottom,
    Prop::MarginLeft,
];
const MARGIN_X: &[Prop] = &[Prop::MarginRight, Prop::MarginLeft];
const MARGIN_Y: &[Prop] = &[Prop::MarginTop, Prop::MarginBottom];
const MARGIN_T: &[Prop] = &[Prop::MarginTop];
const MARGIN_R: &[Prop] = &[Prop::MarginRight];
const MARGIN_B: &[Prop] = &[Prop::MarginBottom];
const MARGIN_L: &[Prop] = &[Prop::MarginLeft];

const BORDER_W: &[Prop] = &[
    Prop::BorderTopWidth,
    Prop::BorderRightWidth,
    Prop::BorderBottomWidth,
    Prop::BorderLeftWidth,
];
const BORDER_W_X: &[Prop] = &[Prop::BorderRightWidth, Prop::BorderLeftWidth];
const BORDER_W_Y: &[Prop] = &[Prop::BorderTopWidth, Prop::BorderBottomWidth];
const BORDER_W_T: &[Prop] = &[Prop::BorderTopWidth];
const BORDER_W_R: &[Prop] = &[Prop::BorderRightWidth];
const BORDER_W_B: &[Prop] = &[Prop::BorderBottomWidth];
const BORDER_W_L: &[Prop] = &[Prop::BorderLeftWidth];

const INSET_ALL: &[Prop] = &[Prop::Top, Prop::Right, Prop::Bottom, Prop::Left];
const INSET_TOP: &[Prop] = &[Prop::Top];
const INSET_RIGHT: &[Prop] = &[Prop::Right];
const INSET_BOTTOM: &[Prop] = &[Prop::Bottom];
const INSET_LEFT: &[Prop] = &[Prop::Left];
const INSET_X: &[Prop] = &[Prop::Right, Prop::Left];
const INSET_Y: &[Prop] = &[Prop::Top, Prop::Bottom];

/// The four corners (TL-TR-BR-BL) — the radius-utility fan-out order.
pub const CORNERS: &[Prop] = &[
    Prop::BorderTopLeftRadius,
    Prop::BorderTopRightRadius,
    Prop::BorderBottomRightRadius,
    Prop::BorderBottomLeftRadius,
];
const CORNER_TOP: &[Prop] = &[Prop::BorderTopLeftRadius, Prop::BorderTopRightRadius];
const CORNER_RIGHT: &[Prop] = &[Prop::BorderTopRightRadius, Prop::BorderBottomRightRadius];
const CORNER_BOTTOM: &[Prop] = &[Prop::BorderBottomRightRadius, Prop::BorderBottomLeftRadius];
const CORNER_LEFT: &[Prop] = &[Prop::BorderTopLeftRadius, Prop::BorderBottomLeftRadius];
const CORNER_TL: &[Prop] = &[Prop::BorderTopLeftRadius];
const CORNER_TR: &[Prop] = &[Prop::BorderTopRightRadius];
const CORNER_BR: &[Prop] = &[Prop::BorderBottomRightRadius];
const CORNER_BL: &[Prop] = &[Prop::BorderBottomLeftRadius];

const W_H: &[Prop] = &[Prop::Width, Prop::Height];
const WIDTH: &[Prop] = &[Prop::Width];
const HEIGHT: &[Prop] = &[Prop::Height];
const MIN_W: &[Prop] = &[Prop::MinWidth];
const MIN_H: &[Prop] = &[Prop::MinHeight];
const MAX_W: &[Prop] = &[Prop::MaxWidth];
const MAX_H: &[Prop] = &[Prop::MaxHeight];

// -- Kind helpers (the `row(...)` table reads as data, not macros) ----------

const fn bare(v: &'static str) -> ValueKind {
    ValueKind::Bare(v)
}
const fn kw(t: &'static [(&'static str, &'static str)]) -> ValueKind {
    ValueKind::Keywords(t)
}
const fn any(kinds: &'static [ValueKind]) -> ValueKind {
    ValueKind::AnyOf(kinds)
}
const fn spacing() -> ValueKind {
    ValueKind::Spacing
}
const fn num() -> ValueKind {
    ValueKind::Number
}
const fn frac() -> ValueKind {
    ValueKind::Fraction
}
const fn arb() -> ValueKind {
    ValueKind::Arbitrary
}
const fn color() -> ValueKind {
    tables::COLOR_KEYWORD_TABLE
}

/// The shared sizing shape: spacing multiples, fractions, the size
/// keywords and the arbitrary passthrough (`w-8`, `w-1/2`, `w-full`,
/// `w-[32px]`). A macro rather than a fn: the `AnyOf` array must live
/// directly in the `static ROWS` initializer to be `'static`.
macro_rules! sized {
    ($extra:expr) => {
        ValueKind::AnyOf(&[
            ValueKind::Spacing,
            ValueKind::Fraction,
            ValueKind::Keywords($extra),
            ValueKind::Arbitrary,
        ])
    };
}

/// The outline per-step fan-out the docs generator renders beside the
/// `outline` root rows (the width scale itself).
pub const OUTLINE_WIDTH_ROWS: &[(&str, &str)] = tables::OUTLINE_WIDTHS;

const fn row(
    root: &'static str,
    kind: ValueKind,
    targets: &'static [Prop],
    group: &'static str,
    negative: bool,
    v4: &'static str,
) -> UtilityRow {
    UtilityRow {
        root,
        kind,
        targets,
        group,
        negative,
        v4,
    }
}

// -- Shared prebuilt rows (referenced across the table) ----------------------

/// `truncate` — the composite utility, split across THREE root-sharing
/// rows (the registry lowers every accepting row of the winning root, so
/// one class emits all three declarations). The one-bare-value fan-out
/// the composite used to model could not carry per-prop values — the
/// `ellipsis` string rode `overflow-x`/`white-space` too, where it is
/// not a legal value. The oracle leaves `truncate` ungrouped; the
/// registry-local group keeps all three rows auditable as one merge
/// unit.
pub const TRUNCATE_OVERFLOW_ROW: UtilityRow = row(
    "truncate",
    bare("hidden"),
    &[Prop::OverflowX, Prop::OverflowY],
    "truncate",
    false,
    "utilities.ts truncate (registry-local group; oracle None)",
);

/// `truncate` — the ellipsis arm (`text-overflow: ellipsis`).
pub const TRUNCATE_ELLIPSIS_ROW: UtilityRow = row(
    "truncate",
    bare("ellipsis"),
    &[Prop::TextOverflow],
    "truncate",
    false,
    "utilities.ts truncate (registry-local group; oracle None)",
);

/// `truncate` — the nowrap arm (`white-space: nowrap`).
pub const TRUNCATE_NOWRAP_ROW: UtilityRow = row(
    "truncate",
    bare("nowrap"),
    &[Prop::WhiteSpace],
    "truncate",
    false,
    "utilities.ts truncate (registry-local group; oracle None)",
);

/// `grow` — the bare default (`flex-grow: 1`).
pub const GROW_BARE_ROW: UtilityRow = row(
    "grow",
    bare("1"),
    &[Prop::FlexGrow],
    "flex-grow",
    false,
    "utilities.ts flex-grow",
);

/// `shrink` — the bare default (`flex-shrink: 1`).
pub const SHRINK_BARE_ROW: UtilityRow = row(
    "shrink",
    bare("1"),
    &[Prop::FlexShrink],
    "flex-shrink",
    false,
    "utilities.ts flex-shrink",
);

/// `outline-none`/`outline-hidden` — the width-reset arm (tw_merge
/// models both under `outline-w`).
pub const OUTLINE_NONE_ROW: UtilityRow = row(
    "outline",
    kw(&[("none", "none"), ("hidden", "none")]),
    &[Prop::OutlineWidth],
    "outline-w",
    false,
    "utilities.ts outline (none arm)",
);

// -- The motion family's shared texts (the plan P6 row) ---------------------
//
// The keyword texts are the v4 default theme's own values — the same
// strings the v0.x oracle renders (`transition_property_list`'s property
// lists, the `--ease-*`/`--animate-*` tokens) — one source of truth the
// CSS arm renders verbatim and the plan bridge maps back onto the v0.x
// `Transition`/`Animation` extras. `pub`: the compile seam's plan bridge
// matches the texts (the gradient direction's `linear_gradient_angle`
// precedent — one source, several readers).

/// `transition` (bare) — the v4 default set's property list, the exact
/// string the v0.x oracle renders for `TransitionSet::Default`.
pub const TRANSITION_DEFAULT_SET: &str = "color, background-color, border-color, outline-color, \
     text-decoration-color, fill, stroke, --tw-gradient-from, --tw-gradient-via, \
     --tw-gradient-to, opacity, box-shadow, transform, translate, scale, rotate, filter, \
     -webkit-backdrop-filter, backdrop-filter, display, content-visibility, overlay, \
     pointer-events";

/// The `transition-{suffix}` property lists — the v4 set families in
/// keyword-slot order (`transition-colors` → slot 1), the exact strings
/// the v0.x oracle renders per `TransitionSet`.
pub const TRANSITION_SETS: &[(&str, &str)] = &[
    ("all", "all"),
    (
        "colors",
        "color, background-color, border-color, outline-color, text-decoration-color, fill, \
         stroke, --tw-gradient-from, --tw-gradient-via, --tw-gradient-to",
    ),
    ("opacity", "opacity"),
    ("shadow", "box-shadow"),
    ("transform", "transform, translate, scale, rotate"),
    ("none", "none"),
];

/// The `ease-{suffix}` timing curves — the v4 default theme's `--ease-*`
/// values in keyword-slot order (`ease-in` → slot 1).
pub const EASE_CURVES: &[(&str, &str)] = &[
    ("linear", "linear"),
    ("in", "cubic-bezier(0.4, 0, 1, 1)"),
    ("out", "cubic-bezier(0, 0, 0.2, 1)"),
    ("in-out", "cubic-bezier(0.4, 0, 0.2, 1)"),
];

/// The `animate-{suffix}` presets — the v4 default theme's `--animate-*`
/// shorthands in keyword-slot order (`animate-spin` → slot 0), the text
/// the plan bridge parses into the v0.x `Animation` extras and the CSS
/// arm renders verbatim.
pub const ANIMATE_PRESETS: &[(&str, &str)] = &[
    ("spin", "spin 1s linear infinite"),
    ("ping", "ping 1s cubic-bezier(0, 0, 0.2, 1) infinite"),
    ("pulse", "pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite"),
    ("bounce", "bounce 1s infinite"),
    ("none", "none"),
];

// -- The registry ---------------------------------------------------------

/// Every registry row, in root order. The P1 port grew to **184 rows**
/// in-tree (189 since the P6 motion layer: the transition quartet and
/// the animation root appended at the cascade tail) — the audit count
/// `registry::generate_docs` renders from these rows. The plan's "top
/// ~150" was the floor, not the ceiling.
pub static ROWS: &[UtilityRow] = &[
    // ---- padding (tw_merge: padding family) -----------------------------
    row(
        "p",
        any(&[spacing(), arb()]),
        PADDING,
        "padding",
        false,
        "utilities.ts padding",
    ),
    row(
        "px",
        any(&[spacing(), arb()]),
        PADDING_X,
        "padding-x",
        false,
        "utilities.ts padding",
    ),
    row(
        "py",
        any(&[spacing(), arb()]),
        PADDING_Y,
        "padding-y",
        false,
        "utilities.ts padding",
    ),
    row(
        "pt",
        any(&[spacing(), arb()]),
        PADDING_T,
        "padding-top",
        false,
        "utilities.ts padding",
    ),
    row(
        "pr",
        any(&[spacing(), arb()]),
        PADDING_R,
        "padding-right",
        false,
        "utilities.ts padding",
    ),
    row(
        "pb",
        any(&[spacing(), arb()]),
        PADDING_B,
        "padding-bottom",
        false,
        "utilities.ts padding",
    ),
    row(
        "pl",
        any(&[spacing(), arb()]),
        PADDING_L,
        "padding-left",
        false,
        "utilities.ts padding",
    ),
    // Logical aliases map onto the same groups as their physical edges
    // (the oracle: ["pr" | "pe"] → padding-right).
    row(
        "ps",
        any(&[spacing(), arb()]),
        PADDING_L,
        "padding-left",
        false,
        "utilities.ts padding (logical alias)",
    ),
    row(
        "pe",
        any(&[spacing(), arb()]),
        PADDING_R,
        "padding-right",
        false,
        "utilities.ts padding (logical alias)",
    ),
    // ---- margin (tw_merge: margin family; negation legal) ----------------
    row(
        "m",
        any(&[spacing(), arb()]),
        MARGIN,
        "margin",
        true,
        "utilities.ts margin",
    ),
    row(
        "mx",
        any(&[spacing(), kw(tables::AUTO_KEYWORD), arb()]),
        MARGIN_X,
        "margin-x",
        true,
        "utilities.ts margin",
    ),
    row(
        "my",
        any(&[spacing(), arb()]),
        MARGIN_Y,
        "margin-y",
        true,
        "utilities.ts margin",
    ),
    row(
        "mt",
        any(&[spacing(), arb()]),
        MARGIN_T,
        "margin-top",
        true,
        "utilities.ts margin",
    ),
    row(
        "mr",
        any(&[spacing(), arb()]),
        MARGIN_R,
        "margin-right",
        true,
        "utilities.ts margin",
    ),
    row(
        "mb",
        any(&[spacing(), arb()]),
        MARGIN_B,
        "margin-bottom",
        true,
        "utilities.ts margin",
    ),
    row(
        "ml",
        any(&[spacing(), arb()]),
        MARGIN_L,
        "margin-left",
        true,
        "utilities.ts margin",
    ),
    row(
        "ms",
        any(&[spacing(), arb()]),
        MARGIN_L,
        "margin-left",
        true,
        "utilities.ts margin (logical alias)",
    ),
    row(
        "me",
        any(&[spacing(), arb()]),
        MARGIN_R,
        "margin-right",
        true,
        "utilities.ts margin (logical alias)",
    ),
    // ---- inset / position edges ------------------------------------------
    row(
        "inset",
        any(&[spacing(), arb()]),
        INSET_ALL,
        "inset",
        true,
        "utilities.ts inset",
    ),
    row(
        "inset-x",
        any(&[spacing(), arb()]),
        INSET_X,
        "inset-x",
        true,
        "utilities.ts inset",
    ),
    row(
        "inset-y",
        any(&[spacing(), arb()]),
        INSET_Y,
        "inset-y",
        true,
        "utilities.ts inset",
    ),
    row(
        "start",
        any(&[spacing(), arb()]),
        &[Prop::InsetInlineStart],
        "start",
        true,
        "utilities.ts inset (logical)",
    ),
    row(
        "end",
        any(&[spacing(), arb()]),
        &[Prop::InsetInlineEnd],
        "end",
        true,
        "utilities.ts inset (logical)",
    ),
    row(
        "top",
        any(&[spacing(), kw(tables::AUTO_KEYWORD), arb()]),
        INSET_TOP,
        "top",
        true,
        "utilities.ts top",
    ),
    row(
        "right",
        any(&[spacing(), kw(tables::AUTO_KEYWORD), arb()]),
        INSET_RIGHT,
        "right",
        true,
        "utilities.ts right",
    ),
    row(
        "bottom",
        any(&[spacing(), kw(tables::AUTO_KEYWORD), arb()]),
        INSET_BOTTOM,
        "bottom",
        true,
        "utilities.ts bottom",
    ),
    row(
        "left",
        any(&[spacing(), kw(tables::AUTO_KEYWORD), arb()]),
        INSET_LEFT,
        "left",
        true,
        "utilities.ts left",
    ),
    // ---- sizing -----------------------------------------------------------
    row(
        "size",
        sized!(tables::SIZE_KEYWORDS),
        W_H,
        "size",
        false,
        "utilities.ts size",
    ),
    row(
        "w",
        sized!(tables::SIZE_KEYWORDS),
        WIDTH,
        "width",
        false,
        "utilities.ts width",
    ),
    row(
        "min-w",
        sized!(tables::SIZE_KEYWORDS),
        MIN_W,
        "min-width",
        false,
        "utilities.ts min-width",
    ),
    row(
        "max-w",
        any(&[
            spacing(),
            kw(tables::SIZE_KEYWORDS),
            kw(tables::CONTAINER_MAX_WIDTHS),
            arb(),
        ]),
        MAX_W,
        "max-width",
        false,
        "utilities.ts max-width",
    ),
    row(
        "h",
        sized!(tables::SIZE_KEYWORDS),
        HEIGHT,
        "height",
        false,
        "utilities.ts height",
    ),
    row(
        "min-h",
        sized!(tables::SIZE_KEYWORDS),
        MIN_H,
        "min-height",
        false,
        "utilities.ts min-height",
    ),
    row(
        "max-h",
        any(&[
            spacing(),
            kw(tables::SIZE_KEYWORDS),
            kw(tables::CONTAINER_MAX_WIDTHS),
            arb(),
        ]),
        MAX_H,
        "max-height",
        false,
        "utilities.ts max-height",
    ),
    row(
        "basis",
        any(&[spacing(), frac(), arb()]),
        &[Prop::FlexBasis],
        "flex-basis",
        false,
        "utilities.ts flex-basis",
    ),
    // ---- gap ---------------------------------------------------------------
    row(
        "gap",
        any(&[spacing(), arb()]),
        &[Prop::GapRow, Prop::GapCol],
        "gap",
        false,
        "utilities.ts gap",
    ),
    row(
        "gap-x",
        any(&[spacing(), arb()]),
        &[Prop::GapCol],
        "gap",
        false,
        "utilities.ts gap",
    ),
    row(
        "gap-y",
        any(&[spacing(), arb()]),
        &[Prop::GapRow],
        "gap",
        false,
        "utilities.ts gap",
    ),
    // ---- flex ---------------------------------------------------------------
    // One root, four arms — the oracle's element split (bare → display;
    // col/row → direction; 1/auto/initial/none → the shorthand triplets
    // sharing the "flex-direction" group; wrap → flex-wrap).
    row(
        "flex",
        bare("flex"),
        &[Prop::Display],
        "display",
        false,
        "utilities.ts display",
    ),
    row(
        "flex",
        kw(tables::FLEX_DIRECTIONS),
        &[Prop::FlexDirection],
        "flex-direction",
        false,
        "utilities.ts flex-direction",
    ),
    row(
        "flex",
        kw(tables::FLEX_WRAPS),
        &[Prop::FlexWrap],
        "flex-wrap",
        false,
        "utilities.ts flex-wrap",
    ),
    row(
        "flex",
        kw(FLEX_TRIPLETS),
        &[Prop::FlexGrow, Prop::FlexShrink, Prop::FlexBasis],
        "flex-direction",
        false,
        "utilities.ts flex shorthand",
    ),
    GROW_BARE_ROW,
    row(
        "grow",
        num(),
        &[Prop::FlexGrow],
        "flex-grow",
        false,
        "utilities.ts flex-grow",
    ),
    SHRINK_BARE_ROW,
    row(
        "shrink",
        num(),
        &[Prop::FlexShrink],
        "flex-shrink",
        false,
        "utilities.ts flex-shrink",
    ),
    row(
        "order",
        any(&[num(), arb()]),
        &[Prop::Order],
        "order",
        false,
        "utilities.ts order",
    ),
    // ---- grid ---------------------------------------------------------------
    row(
        "grid-cols",
        any(&[num(), kw(tables::GRID_TEMPLATES), arb()]),
        &[Prop::GridTemplateColumns],
        "grid-template-columns",
        false,
        "utilities.ts grid-template-columns",
    ),
    row(
        "grid-rows",
        any(&[num(), kw(tables::GRID_TEMPLATES), arb()]),
        &[Prop::GridTemplateRows],
        "grid-template-rows",
        false,
        "utilities.ts grid-template-rows",
    ),
    row(
        "auto-cols",
        any(&[kw(tables::AUTO_TRACKS), arb()]),
        &[Prop::GridAutoColumns],
        "grid-auto-columns",
        false,
        "utilities.ts grid-auto-columns",
    ),
    row(
        "auto-rows",
        any(&[kw(tables::AUTO_TRACKS), arb()]),
        &[Prop::GridAutoRows],
        "grid-auto-rows",
        false,
        "utilities.ts grid-auto-rows",
    ),
    row(
        "col-span",
        any(&[num(), arb()]),
        &[Prop::GridColumnStart, Prop::GridColumnEnd],
        "col-span",
        false,
        "utilities.ts grid-column",
    ),
    row(
        "col-start",
        any(&[num(), arb()]),
        &[Prop::GridColumnStart],
        "col-start",
        true,
        "utilities.ts grid-column",
    ),
    row(
        "col-end",
        any(&[num(), arb()]),
        &[Prop::GridColumnEnd],
        "col-end",
        true,
        "utilities.ts grid-column",
    ),
    row(
        "row-span",
        any(&[num(), arb()]),
        &[Prop::GridRowStart, Prop::GridRowEnd],
        "row-span",
        false,
        "utilities.ts grid-row",
    ),
    row(
        "row-start",
        any(&[num(), arb()]),
        &[Prop::GridRowStart],
        "row-start",
        true,
        "utilities.ts grid-row",
    ),
    row(
        "row-end",
        any(&[num(), arb()]),
        &[Prop::GridRowEnd],
        "row-end",
        true,
        "utilities.ts grid-row",
    ),
    // ---- alignment -----------------------------------------------------------
    row(
        "items",
        any(&[kw(tables::ITEM_ALIGNMENTS), arb()]),
        &[Prop::AlignItems],
        "align-items",
        false,
        "utilities.ts align-items",
    ),
    row(
        "content",
        any(&[kw(tables::CONTENT_ALIGNMENTS), arb()]),
        &[Prop::AlignContent],
        "align-content",
        false,
        "utilities.ts align-content",
    ),
    row(
        "justify",
        any(&[kw(tables::JUSTIFY_ALIGNMENTS), arb()]),
        &[Prop::JustifyContent],
        "justify-content",
        false,
        "utilities.ts justify-content",
    ),
    row(
        "justify-items",
        any(&[kw(tables::ITEM_ALIGNMENTS), arb()]),
        &[Prop::JustifyItems],
        "justify-items",
        false,
        "utilities.ts justify-items",
    ),
    row(
        "justify-self",
        any(&[kw(tables::SELF_ALIGNMENTS), arb()]),
        &[Prop::JustifySelf],
        "justify-self",
        false,
        "utilities.ts justify-self",
    ),
    row(
        "self",
        any(&[kw(tables::SELF_ALIGNMENTS), arb()]),
        &[Prop::AlignSelf],
        "align-self",
        false,
        "utilities.ts align-self",
    ),
    row(
        "place-content",
        any(&[kw(tables::PLACE_ALIGNMENTS), arb()]),
        &[Prop::PlaceContent],
        "place-content",
        false,
        "utilities.ts place-content",
    ),
    row(
        "place-items",
        any(&[kw(tables::PLACE_ALIGNMENTS), arb()]),
        &[Prop::PlaceItems],
        "place-items",
        false,
        "utilities.ts place-items",
    ),
    row(
        "place-self",
        any(&[kw(tables::PLACE_ALIGNMENTS), arb()]),
        &[Prop::PlaceSelf],
        "place-self",
        false,
        "utilities.ts place-self",
    ),
    // ---- display (bare roots; the oracle names the display group) -------------
    row(
        "display",
        any(&[kw(tables::DISPLAYS), arb()]),
        &[Prop::Display],
        "display",
        false,
        "utilities.ts display",
    ),
    row(
        "block",
        bare("block"),
        &[Prop::Display],
        "display",
        false,
        "utilities.ts display",
    ),
    row(
        "inline",
        bare("inline"),
        &[Prop::Display],
        "display",
        false,
        "utilities.ts display",
    ),
    row(
        "inline-block",
        bare("inline-block"),
        &[Prop::Display],
        "display",
        false,
        "utilities.ts display",
    ),
    row(
        "inline-flex",
        bare("inline-flex"),
        &[Prop::Display],
        "display",
        false,
        "utilities.ts display",
    ),
    row(
        "inline-grid",
        bare("inline-grid"),
        &[Prop::Display],
        "display",
        false,
        "utilities.ts display",
    ),
    row(
        "grid",
        bare("grid"),
        &[Prop::Display],
        "display",
        false,
        "utilities.ts display",
    ),
    row(
        "contents",
        bare("contents"),
        &[Prop::Display],
        "display",
        false,
        "utilities.ts display",
    ),
    row(
        "hidden",
        bare("none"),
        &[Prop::Display],
        "display",
        false,
        "utilities.ts display",
    ),
    // ---- position --------------------------------------------------------------
    row(
        "static",
        bare("static"),
        &[Prop::Position],
        "position",
        false,
        "utilities.ts position",
    ),
    row(
        "fixed",
        bare("fixed"),
        &[Prop::Position],
        "position",
        false,
        "utilities.ts position",
    ),
    row(
        "absolute",
        bare("absolute"),
        &[Prop::Position],
        "position",
        false,
        "utilities.ts position",
    ),
    row(
        "relative",
        bare("relative"),
        &[Prop::Position],
        "position",
        false,
        "utilities.ts position",
    ),
    row(
        "sticky",
        bare("sticky"),
        &[Prop::Position],
        "position",
        false,
        "utilities.ts position",
    ),
    // ---- visibility / isolation / z / opacity -----------------------------------
    row(
        "visible",
        bare("visible"),
        &[Prop::Visibility],
        "visibility",
        false,
        "utilities.ts visibility",
    ),
    row(
        "invisible",
        bare("hidden"),
        &[Prop::Visibility],
        "visibility",
        false,
        "utilities.ts visibility",
    ),
    row(
        "collapse",
        bare("collapse"),
        &[Prop::Visibility],
        "visibility",
        false,
        "utilities.ts visibility",
    ),
    row(
        "isolate",
        bare("isolate"),
        &[Prop::Isolation],
        "isolation",
        false,
        "utilities.ts isolation",
    ),
    row(
        "isolation-auto",
        bare("auto"),
        &[Prop::Isolation],
        "isolation",
        false,
        "utilities.ts isolation",
    ),
    row(
        "z",
        any(&[num(), kw(tables::Z_LEVELS), arb()]),
        &[Prop::ZIndex],
        "z-index",
        true,
        "utilities.ts z-index",
    ),
    row(
        "opacity",
        any(&[num(), arb()]),
        &[Prop::Opacity],
        "opacity",
        false,
        "utilities.ts opacity",
    ),
    // ---- overflow / overscroll ---------------------------------------------------
    row(
        "overflow",
        any(&[kw(tables::OVERFLOWS), arb()]),
        &[Prop::OverflowX, Prop::OverflowY],
        "overflow",
        false,
        "utilities.ts overflow",
    ),
    row(
        "overflow-x",
        any(&[kw(tables::OVERFLOWS), arb()]),
        &[Prop::OverflowX],
        "overflow-x",
        false,
        "utilities.ts overflow",
    ),
    row(
        "overflow-y",
        any(&[kw(tables::OVERFLOWS), arb()]),
        &[Prop::OverflowY],
        "overflow-y",
        false,
        "utilities.ts overflow",
    ),
    row(
        "overscroll",
        any(&[kw(tables::OVERSCROLLS), arb()]),
        &[Prop::OverscrollX, Prop::OverscrollY],
        "overscroll-behavior",
        false,
        "utilities.ts overscroll",
    ),
    row(
        "overscroll-x",
        any(&[kw(tables::OVERSCROLLS), arb()]),
        &[Prop::OverscrollX],
        "overscroll-x",
        false,
        "utilities.ts overscroll",
    ),
    row(
        "overscroll-y",
        any(&[kw(tables::OVERSCROLLS), arb()]),
        &[Prop::OverscrollY],
        "overscroll-y",
        false,
        "utilities.ts overscroll",
    ),
    // ---- object / aspect / columns ------------------------------------------------
    row(
        "object",
        any(&[kw(tables::OBJECT_FITS), arb()]),
        &[Prop::ObjectFit],
        "object-fit",
        false,
        "utilities.ts object-fit",
    ),
    row(
        "object",
        any(&[kw(tables::OBJECT_POSITIONS), arb()]),
        &[Prop::ObjectPosition],
        "object-position",
        false,
        "utilities.ts object-position",
    ),
    row(
        "aspect",
        any(&[kw(tables::ASPECTS), frac(), arb()]),
        &[Prop::AspectRatio],
        "aspect",
        false,
        "utilities.ts aspect-ratio",
    ),
    row(
        "columns",
        any(&[num(), kw(tables::COLUMN_KEYWORDS), arb()]),
        &[Prop::Columns],
        "columns",
        false,
        "utilities.ts columns",
    ),
    // ---- colours ---------------------------------------------------------------------
    // `bg` splits: the colour group vs the gradient-image group COEXIST
    // (v0.6 note in the oracle).
    row(
        "bg",
        any(&[color(), arb()]),
        &[Prop::Background],
        "bg-color",
        false,
        "utilities.ts background-color",
    ),
    row(
        "bg",
        any(&[kw(tables::BG_IMAGES), arb()]),
        &[Prop::BackgroundImage],
        "bg-image",
        false,
        "utilities.ts background-image",
    ),
    // The gradient stop edges: colour and position are SEPARATE
    // longhands that COMPOSE with the direction row (v4's own
    // `--tw-gradient-*` custom-property set — the shorthand-shaped
    // BackgroundImage fold let the last stop replace the direction in
    // the per-longhand cascade). The stop POSITION shape is the bare
    // percentage (`from-20%`).
    row(
        "from",
        any(&[color(), arb()]),
        &[Prop::GradientFrom],
        "from-color",
        false,
        "utilities.ts gradient-color-stops",
    ),
    row(
        "from",
        ValueKind::Percent,
        &[Prop::GradientFromPosition],
        "from-pos",
        false,
        "utilities.ts gradient-color-stops",
    ),
    row(
        "via",
        any(&[color(), arb()]),
        &[Prop::GradientVia],
        "via-color",
        false,
        "utilities.ts gradient-color-stops",
    ),
    row(
        "via",
        ValueKind::Percent,
        &[Prop::GradientViaPosition],
        "via-pos",
        false,
        "utilities.ts gradient-color-stops",
    ),
    row(
        "to",
        any(&[color(), arb()]),
        &[Prop::GradientTo],
        "to-color",
        false,
        "utilities.ts gradient-color-stops",
    ),
    row(
        "to",
        ValueKind::Percent,
        &[Prop::GradientToPosition],
        "to-pos",
        false,
        "utilities.ts gradient-color-stops",
    ),
    // ---- border widths + colours --------------------------------------------------------
    // The width/color split follows the oracle's is_border_width rule:
    // numeric-or-edge suffixes are widths, palette names are colours,
    // and the bare root is the 1px default.
    row(
        "border",
        any(&[bare("1px"), num(), kw(tables::BORDER_WIDTH_STEPS), arb()]),
        BORDER_W,
        "border-w",
        false,
        "utilities.ts border-width",
    ),
    row(
        "border-t",
        any(&[bare("1px"), num(), kw(tables::BORDER_WIDTH_STEPS), arb()]),
        BORDER_W_T,
        "border-w-t",
        false,
        "utilities.ts border-width",
    ),
    row(
        "border-r",
        any(&[bare("1px"), num(), kw(tables::BORDER_WIDTH_STEPS), arb()]),
        BORDER_W_R,
        "border-w-r",
        false,
        "utilities.ts border-width",
    ),
    row(
        "border-b",
        any(&[bare("1px"), num(), kw(tables::BORDER_WIDTH_STEPS), arb()]),
        BORDER_W_B,
        "border-w-b",
        false,
        "utilities.ts border-width",
    ),
    row(
        "border-l",
        any(&[bare("1px"), num(), kw(tables::BORDER_WIDTH_STEPS), arb()]),
        BORDER_W_L,
        "border-w-l",
        false,
        "utilities.ts border-width",
    ),
    row(
        "border-x",
        any(&[bare("1px"), num(), kw(tables::BORDER_WIDTH_STEPS), arb()]),
        BORDER_W_X,
        "border-w-x",
        false,
        "utilities.ts border-width",
    ),
    row(
        "border-y",
        any(&[bare("1px"), num(), kw(tables::BORDER_WIDTH_STEPS), arb()]),
        BORDER_W_Y,
        "border-w-y",
        false,
        "utilities.ts border-width",
    ),
    row(
        "border-s",
        any(&[bare("1px"), num(), kw(tables::BORDER_WIDTH_STEPS), arb()]),
        BORDER_W_L,
        "border-w-l",
        false,
        "utilities.ts border-width (logical alias)",
    ),
    row(
        "border-e",
        any(&[bare("1px"), num(), kw(tables::BORDER_WIDTH_STEPS), arb()]),
        BORDER_W_R,
        "border-w-r",
        false,
        "utilities.ts border-width (logical alias)",
    ),
    // Colour rows — per-side colour suffixes all group "border-color"
    // (the oracle has no border-color-t arm).
    row(
        "border",
        any(&[color(), arb()]),
        &[Prop::BorderColor],
        "border-color",
        false,
        "utilities.ts border-color",
    ),
    row(
        "border-t",
        any(&[color(), arb()]),
        &[Prop::BorderColor],
        "border-color",
        false,
        "utilities.ts border-color",
    ),
    row(
        "border-r",
        any(&[color(), arb()]),
        &[Prop::BorderColor],
        "border-color",
        false,
        "utilities.ts border-color",
    ),
    row(
        "border-b",
        any(&[color(), arb()]),
        &[Prop::BorderColor],
        "border-color",
        false,
        "utilities.ts border-color",
    ),
    row(
        "border-l",
        any(&[color(), arb()]),
        &[Prop::BorderColor],
        "border-color",
        false,
        "utilities.ts border-color",
    ),
    row(
        "border-x",
        any(&[color(), arb()]),
        &[Prop::BorderColor],
        "border-color",
        false,
        "utilities.ts border-color",
    ),
    row(
        "border-y",
        any(&[color(), arb()]),
        &[Prop::BorderColor],
        "border-color",
        false,
        "utilities.ts border-color",
    ),
    // Style keywords: the tw_merge quirk — border-dashed groups under
    // border-color (documented in tables::BORDER_STYLES).
    row(
        "border",
        kw(tables::BORDER_STYLES),
        &[Prop::BorderStyle],
        "border-color",
        false,
        "utilities.ts border-style (grouped under border-color per tw_merge)",
    ),
    // ---- radius --------------------------------------------------------------------------
    row(
        "rounded",
        any(&[bare("0.25rem"), kw(tables::RADII), frac(), arb()]),
        CORNERS,
        "rounded",
        false,
        "utilities.ts border-radius",
    ),
    row(
        "rounded-t",
        any(&[kw(tables::RADII), frac(), arb()]),
        CORNER_TOP,
        "rounded-t",
        false,
        "utilities.ts border-radius",
    ),
    row(
        "rounded-r",
        any(&[kw(tables::RADII), frac(), arb()]),
        CORNER_RIGHT,
        "rounded-r",
        false,
        "utilities.ts border-radius",
    ),
    row(
        "rounded-b",
        any(&[kw(tables::RADII), frac(), arb()]),
        CORNER_BOTTOM,
        "rounded-b",
        false,
        "utilities.ts border-radius",
    ),
    row(
        "rounded-l",
        any(&[kw(tables::RADII), frac(), arb()]),
        CORNER_LEFT,
        "rounded-l",
        false,
        "utilities.ts border-radius",
    ),
    row(
        "rounded-s",
        any(&[kw(tables::RADII), frac(), arb()]),
        CORNER_LEFT,
        "rounded-l",
        false,
        "utilities.ts border-radius (logical alias)",
    ),
    row(
        "rounded-e",
        any(&[kw(tables::RADII), frac(), arb()]),
        CORNER_RIGHT,
        "rounded-r",
        false,
        "utilities.ts border-radius (logical alias)",
    ),
    row(
        "rounded-tl",
        any(&[kw(tables::RADII), frac(), arb()]),
        CORNER_TL,
        "rounded-tl",
        false,
        "utilities.ts border-radius",
    ),
    row(
        "rounded-tr",
        any(&[kw(tables::RADII), frac(), arb()]),
        CORNER_TR,
        "rounded-tr",
        false,
        "utilities.ts border-radius",
    ),
    row(
        "rounded-br",
        any(&[kw(tables::RADII), frac(), arb()]),
        CORNER_BR,
        "rounded-br",
        false,
        "utilities.ts border-radius",
    ),
    row(
        "rounded-bl",
        any(&[kw(tables::RADII), frac(), arb()]),
        CORNER_BL,
        "rounded-bl",
        false,
        "utilities.ts border-radius",
    ),
    // ---- outline (four independent groups) ----------------------------------------------
    OUTLINE_NONE_ROW,
    row(
        "outline",
        any(&[num(), kw(tables::OUTLINE_WIDTHS), arb()]),
        &[Prop::OutlineWidth],
        "outline-w",
        false,
        "utilities.ts outline-width",
    ),
    row(
        "outline",
        kw(tables::OUTLINE_STYLES),
        &[Prop::OutlineStyle],
        "outline-style",
        false,
        "utilities.ts outline-style",
    ),
    row(
        "outline",
        any(&[color(), arb()]),
        &[Prop::OutlineColor],
        "outline-color",
        false,
        "utilities.ts outline-color",
    ),
    row(
        "outline-offset",
        any(&[num(), arb()]),
        &[Prop::OutlineOffset],
        "outline-offset",
        false,
        "utilities.ts outline-offset",
    ),
    // ---- ring (width / color / offset-width / offset-color) ------------------------------
    row(
        "ring",
        any(&[bare("1px"), num(), arb()]),
        &[Prop::RingWidth],
        "ring-w",
        false,
        "utilities.ts ring-width",
    ),
    row(
        "ring",
        any(&[color(), arb()]),
        &[Prop::RingColor],
        "ring-color",
        false,
        "utilities.ts ring-color",
    ),
    row(
        "ring-inset",
        bare("inset"),
        &[Prop::BoxShadow],
        "ring-color",
        false,
        "utilities.ts ring (inset arm; tw_merge groups it ring-color)",
    ),
    row(
        "ring-offset",
        any(&[num(), kw(tables::RING_OFFSET_WIDTHS), arb()]),
        &[Prop::RingOffsetWidth],
        "ring-offset-w",
        false,
        "utilities.ts ring-offset-width",
    ),
    row(
        "ring-offset",
        any(&[color(), arb()]),
        &[Prop::RingOffsetColor],
        "ring-offset-color",
        false,
        "utilities.ts ring-offset-color",
    ),
    // ---- shadow (preset width vs shadow colour) -------------------------------------------
    row(
        "shadow",
        any(&[bare("var(--tw-shadow)"), kw(tables::SHADOW_PRESETS), arb()]),
        &[Prop::BoxShadow],
        "shadow",
        false,
        "utilities.ts box-shadow",
    ),
    row(
        "shadow",
        any(&[color(), arb()]),
        &[Prop::ShadowColor],
        "shadow-color",
        false,
        "utilities.ts box-shadow-color",
    ),
    // ---- typography ------------------------------------------------------------------------
    row(
        "text",
        any(&[ValueKind::Composite(tables::TEXT_SIZES), arb()]),
        &[Prop::FontSize, Prop::LineHeight],
        "font-size",
        false,
        "utilities.ts font-size",
    ),
    row(
        "text",
        any(&[color(), arb()]),
        &[Prop::Color],
        "text-color",
        false,
        "utilities.ts color",
    ),
    row(
        "text",
        kw(tables::TEXT_ALIGNS),
        &[Prop::TextAlign],
        "text-align",
        false,
        "utilities.ts text-align",
    ),
    row(
        "font",
        kw(tables::FONT_FAMILIES),
        &[Prop::FontFamily],
        "font-family",
        false,
        "utilities.ts font-family",
    ),
    row(
        "font",
        any(&[kw(tables::FONT_WEIGHTS), arb()]),
        &[Prop::FontWeight],
        "font-weight",
        false,
        "utilities.ts font-weight",
    ),
    row(
        "italic",
        bare("italic"),
        &[Prop::FontStyle],
        "font-style",
        false,
        "utilities.ts font-style",
    ),
    row(
        "not-italic",
        bare("normal"),
        &[Prop::FontStyle],
        "font-style",
        false,
        "utilities.ts font-style",
    ),
    row(
        "leading",
        any(&[kw(tables::LEADING_RATIOS), spacing(), arb()]),
        &[Prop::LineHeight],
        "line-height",
        false,
        "utilities.ts line-height",
    ),
    row(
        "tracking",
        any(&[kw(tables::TRACKING), arb()]),
        &[Prop::LetterSpacing],
        "letter-spacing",
        false,
        "utilities.ts letter-spacing",
    ),
    row(
        "underline",
        bare("underline"),
        &[Prop::TextDecorationLine],
        "text-decoration",
        false,
        "utilities.ts text-decoration",
    ),
    row(
        "overline",
        bare("overline"),
        &[Prop::TextDecorationLine],
        "text-decoration",
        false,
        "utilities.ts text-decoration",
    ),
    row(
        "line-through",
        bare("line-through"),
        &[Prop::TextDecorationLine],
        "text-decoration",
        false,
        "utilities.ts text-decoration",
    ),
    row(
        "no-underline",
        bare("none"),
        &[Prop::TextDecorationLine],
        "text-decoration",
        false,
        "utilities.ts text-decoration",
    ),
    row(
        "underline-offset",
        any(&[num(), spacing(), arb()]),
        &[Prop::TextUnderlineOffset],
        "underline-offset",
        false,
        "utilities.ts text-underline-offset",
    ),
    row(
        "uppercase",
        bare("uppercase"),
        &[Prop::TextTransform],
        "text-transform",
        false,
        "utilities.ts text-transform",
    ),
    row(
        "lowercase",
        bare("lowercase"),
        &[Prop::TextTransform],
        "text-transform",
        false,
        "utilities.ts text-transform",
    ),
    row(
        "capitalize",
        bare("capitalize"),
        &[Prop::TextTransform],
        "text-transform",
        false,
        "utilities.ts text-transform",
    ),
    row(
        "normal-case",
        bare("none"),
        &[Prop::TextTransform],
        "text-transform",
        false,
        "utilities.ts text-transform",
    ),
    row(
        "whitespace",
        any(&[kw(tables::WHITESPACE), arb()]),
        &[Prop::WhiteSpace],
        "whitespace",
        false,
        "utilities.ts white-space",
    ),
    row(
        "break",
        any(&[kw(tables::WORD_BREAKS), arb()]),
        &[Prop::WordBreak],
        "word-break",
        false,
        "utilities.ts word-break",
    ),
    row(
        "align",
        any(&[kw(tables::VERTICAL_ALIGNS), arb()]),
        &[Prop::VerticalAlign],
        "vertical-align",
        false,
        "utilities.ts vertical-align",
    ),
    TRUNCATE_OVERFLOW_ROW,
    TRUNCATE_ELLIPSIS_ROW,
    TRUNCATE_NOWRAP_ROW,
    // ---- transforms (oracle None — registry-local groups) -------------------------------
    row(
        "translate-x",
        any(&[spacing(), frac(), arb()]),
        &[Prop::TranslateX],
        "translate-x",
        true,
        "utilities.ts translate (registry-local group; oracle None)",
    ),
    row(
        "translate-y",
        any(&[spacing(), frac(), arb()]),
        &[Prop::TranslateY],
        "translate-y",
        true,
        "utilities.ts translate (registry-local group; oracle None)",
    ),
    row(
        "rotate",
        any(&[num(), arb()]),
        &[Prop::Rotate],
        "rotate",
        true,
        "utilities.ts rotate (registry-local group; oracle None)",
    ),
    row(
        "scale",
        any(&[num(), frac(), arb()]),
        &[Prop::ScaleX, Prop::ScaleY],
        "scale",
        false,
        "utilities.ts scale (registry-local group; oracle None)",
    ),
    row(
        "scale-x",
        any(&[num(), frac(), arb()]),
        &[Prop::ScaleX],
        "scale-x",
        false,
        "utilities.ts scale (registry-local group; oracle None)",
    ),
    row(
        "scale-y",
        any(&[num(), frac(), arb()]),
        &[Prop::ScaleY],
        "scale-y",
        false,
        "utilities.ts scale (registry-local group; oracle None)",
    ),
    row(
        "skew-x",
        any(&[num(), arb()]),
        &[Prop::SkewX],
        "skew-x",
        true,
        "utilities.ts skew (registry-local group; oracle None)",
    ),
    row(
        "skew-y",
        any(&[num(), arb()]),
        &[Prop::SkewY],
        "skew-y",
        true,
        "utilities.ts skew (registry-local group; oracle None)",
    ),
    // ---- interaction ---------------------------------------------------------------------
    row(
        "cursor",
        any(&[kw(tables::CURSORS), arb()]),
        &[Prop::Cursor],
        "cursor",
        false,
        "utilities.ts cursor",
    ),
    row(
        "pointer-events",
        any(&[bare("auto"), kw(&[("none", "none"), ("auto", "auto")])]),
        &[Prop::PointerEvents],
        "pointer-events",
        false,
        "utilities.ts pointer-events",
    ),
    row(
        "select",
        any(&[
            kw(&[
                ("none", "none"),
                ("text", "text"),
                ("all", "all"),
                ("auto", "auto"),
            ]),
            arb(),
        ]),
        &[Prop::UserSelect],
        "user-select",
        false,
        "utilities.ts user-select",
    ),
    row(
        "resize",
        any(&[bare("both"), kw(tables::RESIZES)]),
        &[Prop::Resize],
        "resize",
        false,
        "utilities.ts resize",
    ),
    // ---- container queries -----------------------------------------------------------------
    row(
        "@container",
        any(&[bare("inline-size"), kw(tables::CONTAINER_TYPES)]),
        &[Prop::ContainerType],
        "container-type",
        false,
        "utilities.ts container-type",
    ),
    // ---- svg ---------------------------------------------------------------------------------
    row(
        "fill",
        any(&[color(), arb()]),
        &[Prop::Fill],
        "fill",
        false,
        "utilities.ts fill (registry-local group; oracle None)",
    ),
    row(
        "stroke",
        any(&[color(), num(), arb()]),
        &[Prop::Stroke],
        "stroke",
        false,
        "utilities.ts stroke (registry-local group; oracle None)",
    ),
    // ---- motion (the plan P6 row; the interaction cascade tail) ------------
    //
    // The `transition-*`/`duration-*`/`ease-*`/`delay-*`/`animate-*`
    // families. The tw_merge oracle leaves ALL five roots ungrouped
    // (`collision_group` returns `None`), so each carries a
    // registry-local group name, marked in its v4 reference — the
    // transforms/truncate/fill/stroke precedent. The keyword texts are
    // the v4 default theme's own values (the same strings the v0.x
    // oracle renders: `transition_property_list`'s property lists, the
    // `--ease-*`/`--animate-*` tokens) — one source of truth the CSS
    // arm renders and the plan bridge maps back onto the v0.x
    // `Transition`/`Animation` extras. The texts live in the pub consts
    // above [`ROWS`].
    row(
        "transition",
        any(&[bare(TRANSITION_DEFAULT_SET), kw(TRANSITION_SETS)]),
        &[Prop::TransitionProperty],
        "transition",
        false,
        "utilities.ts transition (registry-local group; oracle None)",
    ),
    row(
        "duration",
        num(),
        &[Prop::TransitionDuration],
        "duration",
        false,
        "utilities.ts transition-duration (registry-local group; oracle None)",
    ),
    row(
        "delay",
        num(),
        &[Prop::TransitionDelay],
        "delay",
        false,
        "utilities.ts transition-delay (registry-local group; oracle None)",
    ),
    row(
        "ease",
        kw(EASE_CURVES),
        &[Prop::TransitionTimingFunction],
        "ease",
        false,
        "utilities.ts transition-timing-function (registry-local group; oracle None)",
    ),
    row(
        "animate",
        kw(ANIMATE_PRESETS),
        &[Prop::Animation],
        "animate",
        false,
        "utilities.ts animation (registry-local group; oracle None)",
    ),
];
