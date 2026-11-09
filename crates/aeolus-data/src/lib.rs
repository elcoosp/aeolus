//! The Aeolus V1 utility registry as data (architecture plan §3.2).
//!
//! One auditable row per utility PATTERN: the root (`"p"`, `"mx"`,
//! `"rounded-tl"`), the value kind, the longhand targets as [`Prop`]
//! masks-slices, the `tw_merge` conflict group and the tailwindcss v4
//! source reference. The committed tables in [`rows`] ARE the source of
//! truth this round; the P2 xtask grows the generator story (phf/trie,
//! docs, linter tables, class-sort order) FROM them, and
//! [`registry::generate_docs`] already proves the one-source-of-truth
//! claim by rendering the docs page from the same rows.
//!
//! The row model is deliberately `const`-constructible and
//! dependency-light (only [`aeolus_ir`]): the P2 compile stage reads it
//! at compile time to lower candidates into `Program`s. The conflict
//! group names are the `tw_merge` 0.1.22 table's own strings (the v0.x
//! `aeolus-core::conflict` oracle); the registry tests pin the two
//! tables together, so a drift fails CI, not a downstream backend.
//!
//! What a row does NOT carry: the CSS emission logic. The registry says
//! `p-4` targets the four padding longhands with the spacing scale; how
//! a backend emits `1rem` (or `16px`, or the window's rem) is the
//! backend's job — exactly the moat the plan §1.3 keeps.

pub mod registry;
pub mod rows;
pub mod tables;

use aeolus_ir::Prop;

/// How a utility's value is shaped — the descriptive half of the row.
///
/// The variants describe the SUFFIX grammar after the root (`p-4` →
/// suffix `"4"`; `w-full` → suffix `"full"`; bare `hidden` → no suffix).
/// Lowering suffix → [`aeolus_ir::Val`] is P2's compile stage; the kind
/// is what the registry (and the generated docs/linter) can honestly
/// claim without an emitter.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ValueKind {
    /// No suffix; the utility IS the value (`hidden` → `visibility:
    /// hidden`). The `&'static str` is the CSS value.
    Bare(&'static str),
    /// A closed keyword table: suffix → CSS value (`flex-col` →
    /// `column`, `uppercase` → `uppercase`). Exhaustive and auditable —
    /// the linter's "did you mean" surface grows from here.
    Keywords(&'static [(&'static str, &'static str)]),
    /// A two-value keyword table: suffix → (primary, secondary) CSS —
    /// the `text-{size}` pair (font-size, line-height) is the corpus's
    /// only two-headed utility at the P1 surface.
    Composite(&'static [(&'static str, &'static str, &'static str)]),
    /// The `--spacing` multiple scale: suffix is a (possibly negative,
    /// possibly half-step) number n; the value is n × 0.25rem
    /// ([`tables::SPACING_STEP_REM`]). `p-4`, `mx-0.5`, `-mt-2`.
    Spacing,
    /// A `n/d` fraction rendered as a percentage: `w-1/2` → `50%`.
    Fraction,
    /// A unitless number scale: `z-10`, `order-3`, `opacity-50` (the
    /// per-root scale factor is the codegen's business; the row only
    /// classifies the shape).
    Number,
    /// The arbitrary passthrough: `[...]` / `(...)` values with optional
    /// type hints ride the candidate's arbitrary group verbatim. Shape
    /// acceptance: the bare root (the value rode in the candidate's
    /// arbitrary group — `w-[32px]` lexes body `w`) or a bracket-shaped
    /// suffix (`w-[-32px]` keeps its brackets in some call paths).
    Arbitrary,
    /// A bare percentage suffix (`from-20%`, `via-40%`): the gradient
    /// STOP POSITION utilities — a different conflict group than the
    /// colour stops, which is exactly why it is its own shape.
    Percent,
    /// A root with several legal value shapes (`w-8` spacing, `w-full`
    /// keyword, `w-1/2` fraction, `w-[32px]` arbitrary). The sub-kinds
    /// are tried in order; nesting [`ValueKind::AnyOf`] inside
    /// [`ValueKind::AnyOf`] is forbidden (the tests enforce depth 1).
    AnyOf(&'static [ValueKind]),
}

impl ValueKind {
    /// Does this kind accept a utility whose suffix is `suffix`
    /// (`None` = the bare root)? The shape contract behind
    /// `registry::lookup`'s disambiguation.
    pub fn accepts(self, suffix: Option<&str>) -> bool {
        match self {
            ValueKind::Bare(_) => suffix.is_none(),
            ValueKind::Keywords(t) => suffix.is_some_and(|s| t.iter().any(|&(name, _)| name == s)),
            ValueKind::Composite(t) => {
                suffix.is_some_and(|s| t.iter().any(|&(name, _, _)| name == s))
            }
            // The spacing scale also takes the `px` unit step (`pb-px` —
            // one pixel, the v4 spelling of the 1px border/padding nub).
            ValueKind::Spacing => {
                let s = suffix.unwrap_or("");
                s == "px" || s.strip_prefix('-').unwrap_or(s).parse::<f32>().is_ok()
            }
            ValueKind::Fraction => suffix.is_some_and(|s| {
                s.split_once('/')
                    .is_some_and(|(n, d)| !n.is_empty() && !d.is_empty())
            }),
            ValueKind::Number => suffix.is_some_and(|s| s.parse::<f32>().is_ok()),
            ValueKind::Percent => {
                suffix.is_some_and(|s| s.ends_with('%') && s[..s.len() - 1].parse::<f32>().is_ok())
            }
            ValueKind::Arbitrary => suffix.is_none_or(|s| s.starts_with('[') || s.starts_with('(')),
            ValueKind::AnyOf(kinds) => kinds.iter().any(|&k| k.accepts(suffix)),
        }
    }

    /// The short name the generated docs table renders
    /// (`registry::generate_docs`).
    pub const fn label(self) -> &'static str {
        match self {
            ValueKind::Bare(_) => "bare",
            ValueKind::Keywords(_) => "keywords",
            ValueKind::Composite(_) => "composite",
            ValueKind::Spacing => "spacing",
            ValueKind::Fraction => "fraction",
            ValueKind::Number => "number",
            ValueKind::Percent => "percent",
            ValueKind::Arbitrary => "arbitrary",
            ValueKind::AnyOf(_) => "any-of",
        }
    }
}

/// One registry row: the auditable contract for a utility root.
///
/// Field order mirrors the `row(...)` constructor in [`rows`] (root,
/// kind, targets, group, negative, v4). `Copy` + `const`: the whole
/// registry is a `static` item.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UtilityRow {
    /// The utility root: `"p"`, `"mx"`, `"rounded-tl"`, `"text"`. Root
    /// match is the longest-prefix rule over the body (`rounded-tl-lg`
    /// disambiguates to `rounded-tl`, not `rounded`).
    pub root: &'static str,
    /// The value shape (suffix grammar + tables).
    pub kind: ValueKind,
    /// The longhand targets, in TRBL / TL-TR-BR-BL fan-out order. Never
    /// empty, never with a duplicate (tests pin both).
    pub targets: &'static [Prop],
    /// The `tw_merge` conflict group name — the EXACT string the v0.x
    /// `aeolus-core::conflict::collision_group` oracle returns, so the
    /// two tables can be pinned together in tests.
    pub group: &'static str,
    /// Whether the leading-`-` negated form is legal (`-mt-4` yes;
    /// `-p-4` no).
    pub negative: bool,
    /// The tailwindcss v4 source reference (`"utilities.ts padding"`) —
    /// the audit trail back to upstream.
    pub v4: &'static str,
}

/// The registry version — bumped when rows change shape or meaning (the
/// generated docs and the P2 pipeline assert it).
pub const REGISTRY_VERSION: &str = "1";
