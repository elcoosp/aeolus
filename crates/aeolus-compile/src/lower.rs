//! The value pass: one registry row + one suffix → longhand `Decl`s
//! (architecture plan §3.1's "MIR resolved vs theme; longhand decls").
//!
//! The row IS the contract: its [`ValueKind`] says which suffix shapes
//! are legal, its `targets` say which longhands fan out, its `negative`
//! flag says whether the `-` form is legal at all. This module turns the
//! accepted shape into [`Val`]s:
//!
//! * `Spacing` → `Rem(n × SPACING_STEP_REM)` (or `Px` under the compat
//!   config; the `px` step is one absolute pixel in both modes);
//! * `Fraction`/`Percent` → `Pct`;
//! * `Number` → `Num`;
//! * `Keywords`/`Composite`/`Bare` → `Val::Kw` — the slot indexes the
//!   row's OWN table (the codegen crate owns the names), with one
//!   resolved exception the theme pass owns: the colour table is 270
//!   entries and the IR's `Kw` slot is a u8, so colour tokens lower
//!   through the [`TokenTable`] when the caller supplies one (plan
//!   §3.4) — tiered `Rgba`/`ModePair`/`Tok` values, never truncation.
//!   Without a table the refusal stays typed (`REASON_TOKEN_TABLE`);
//! * `Arbitrary` → the unit grammar (the `arbitrary` module: units,
//!   hints, colours, var-token references, and the typed boundaries).
//!
//! An opacity modifier (`/90`) rides only COLOUR-form values: token
//! decls carry it (the `Tok` slot's alpha, or the premultiplied bake
//! for a literal), everything else refuses with [`Reject::Modifier`]
//! — leading arithmetic and gradient modifiers are the codegen's, and
//! a modifier never half-applies to a mixed fan-out.
//!
//! Every refusal is a typed [`Reject`] — the reachable public path
//! carries no panic (CONTRIBUTING §2), and no value that is not finite
//! reaches the IR (the f32 grammar's `nan`/`inf` spellings are typed
//! refusals, not silent literals).

use aeolus_data::tables::SPACING_STEP_REM;
use aeolus_data::{UtilityRow, ValueKind};
use aeolus_ir::{Decl, Prop, Val};
use aeolus_syntax::Arb;
use aeolus_theme::{ResolveError, TokenTable};

use crate::arbitrary::{self, ArbVal};
use crate::{Config, Spacing};

/// Why one row's lowering refused — the typed boundary the diagnostic
/// layer renders.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Reject {
    /// The `-` form rode along on a root whose row does not take it
    /// (the registry's `negative` flag is the legality oracle).
    Negation,
    /// The value half does not lower in this slice, with the reason.
    Value { reason: &'static str },
    /// The keyword table exceeds the u8 `Val::Kw` slot — internal only:
    /// the caller resolves it (the theme's TokenTable when supplied,
    /// `REASON_TOKEN_TABLE` when not) and it never escapes this module.
    TokenOverflow,
    /// An opacity modifier rode a non-colour lowering (or a baked dark
    /// pair, whose slot has no alpha arm) — the diagnostic layer renders
    /// the raw modifier text.
    Modifier,
}

// Arbitrary `[...]`/`(...)` values lower through the `arbitrary`
// module's unit grammar — the typed boundary set lives there (the
// unit/hint/property/composite/table reasons and the row-class gate).

/// The seam's discovered IR-budget constraint: a keyword table larger
/// than the u8 `Val::Kw` slot (the colour table's 270 entries) lowers
/// through the theme pass's TokenTable when the caller supplies one
/// (plan §3.4), and refuses typed when not — never truncation.
pub(crate) const REASON_TOKEN_TABLE: &str = "the keyword table exceeds the IR's u8 keyword slot: colour tokens \
     lower through the theme pass's TokenTable (plan §3.4 T2)";

/// The suffix is neither a theme token nor lowerable through the u8
/// keyword slot — the table-driven miss the linter's "did you mean"
/// surface renders (`TokenTable::names()` is the checklist).
pub(crate) const REASON_NOT_A_TOKEN: &str = "the suffix is not a colour token in the supplied TokenTable (see \
     TokenTable::names()) and the palette table exceeds the u8 keyword slot";

/// The f32 grammar's non-finite spellings (`nan`, `inf`, `1e40`) are
/// legal PARSES and illegal v4 values: the value pass refuses them — a
/// non-finite literal never reaches the IR (the snapshot writer's
/// string-encoded infinities exist because the reader is strict; the
/// IR never expected to carry them in the first place).
pub(crate) const REASON_NON_FINITE: &str = "the suffix parses as a non-finite f32 (nan/inf): the v4 value grammar \
     is finite decimals, so the value pass refuses instead of lowering";

/// The shape contract accepted the suffix but no arm lowered it —
/// registry/lowering drift, reportable, never a panic.
pub(crate) const REASON_SHAPE_MISMATCH: &str = "the row's shape contract accepted the suffix but no arm lowered it \
     (registry/compile drift — please report)";

/// Lower ONE matched row for one candidate body. The registry guarantees
/// the body addresses the row's root and at least one of its value
/// shapes accepts the suffix; everything past that guarantee is typed.
/// `alpha` is the normalized opacity modifier (`/90` → `Some(0.9)`) and
/// rides COLOUR-form decls only; `tokens` is the theme pass's table
/// (plan §3.4), which owns the oversized colour-keyword surface; `arb`
/// is the candidate's lexed arbitrary group ([`ValueKind::Arbitrary`]
/// rows lower it through the `arbitrary` module's unit grammar).
pub(crate) fn row_to_decls(
    row: UtilityRow,
    body: &str,
    negative: bool,
    alpha: Option<f32>,
    tokens: Option<&TokenTable>,
    config: Config,
    arb: Option<Arb<'_>>,
) -> Result<Vec<Decl>, Reject> {
    if negative && !row.negative {
        return Err(Reject::Negation);
    }
    let suffix = suffix_of(body, row.root)?;
    let mut arms = Vec::new();
    flatten(row.kind, &mut arms);
    for arm in arms {
        // A candidate carrying an arbitrary group lowers through the
        // Arbitrary arm ONLY — the value rode in the group, not the
        // suffix, so the bare/keyword arms (which accept the bare
        // root) must not swallow it (`rounded-[3px]` is the radius
        // value, never the `0.25rem` bare form).
        if arb.is_some() && !matches!(arm, ValueKind::Arbitrary) {
            continue;
        }
        if !arm.accepts(suffix) {
            continue;
        }
        let decls = arm_to_decls(arm, suffix, negative, alpha, row, tokens, config, arb)?;
        // A modifier never half-applies: it rides only when EVERY decl
        // the row produced is colour-form (the token path already
        // applied it; anything else refuses the whole row).
        if alpha.is_some() && !decls.iter().all(|decl| is_colour_val(decl.val)) {
            return Err(Reject::Modifier);
        }
        return Ok(decls);
    }
    Err(Reject::Value {
        reason: REASON_SHAPE_MISMATCH,
    })
}

/// The colour-form values — the only ones an opacity modifier rides
/// (the theme pass's outputs; `Val::Rgba` is produced solely by the
/// token path, so the check cannot misread a baked literal from
/// elsewhere).
fn is_colour_val(val: Val) -> bool {
    matches!(val, Val::Rgba(_) | Val::ModePair(_) | Val::Tok { .. })
}

/// The suffix after the root — the registry's own longest-prefix rule,
/// mirrored (`"4"` for `p-4`, `None` for `hidden`). The lookup already
/// matched the body against the root, so a miss here is the drift arm.
pub(crate) fn suffix_of<'a>(body: &'a str, root: &str) -> Result<Option<&'a str>, Reject> {
    if body == root {
        return Ok(None);
    }
    body.strip_prefix(root)
        .and_then(|rest| rest.strip_prefix('-'))
        .map(Some)
        .ok_or(Reject::Value {
            reason: REASON_SHAPE_MISMATCH,
        })
}

/// The AnyOf arms in declaration order (the first ACCEPTING arm lowers —
/// the shape contract's own try order). Nesting flattens recursively so
/// the walk is total even if the registry ever grows a second level.
fn flatten(kind: ValueKind, out: &mut Vec<ValueKind>) {
    match kind {
        ValueKind::AnyOf(inner) => {
            for k in inner {
                flatten(*k, out);
            }
        }
        k => out.push(k),
    }
}

fn arm_to_decls(
    arm: ValueKind,
    suffix: Option<&str>,
    negative: bool,
    alpha: Option<f32>,
    row: UtilityRow,
    tokens: Option<&TokenTable>,
    config: Config,
    arb: Option<Arb<'_>>,
) -> Result<Vec<Decl>, Reject> {
    let targets = row.targets;
    match arm {
        ValueKind::Bare(_) => Ok(fan(targets, Val::Kw(0))),
        ValueKind::Keywords(table) => match keyword_slot(table, suffix) {
            Ok(slot) => Ok(fan(targets, Val::Kw(slot))),
            // The oversized-table boundary: the theme pass owns it.
            Err(Reject::TokenOverflow) => match tokens {
                Some(table) => token_decls(suffix, alpha, targets, table),
                None => Err(Reject::Value {
                    reason: REASON_TOKEN_TABLE,
                }),
            },
            Err(other) => Err(other),
        },
        ValueKind::Composite(table) => {
            let slot = composite_slot(table, suffix)?;
            Ok(fan(targets, Val::Kw(slot)))
        }
        ValueKind::Spacing => spacing_decls(suffix, negative, targets, config),
        ValueKind::Fraction => {
            let (n, d) = suffix
                .and_then(|s| s.split_once('/'))
                .ok_or(Reject::Value {
                    reason: REASON_SHAPE_MISMATCH,
                })?;
            let pct = finite(parse_num(n)? / parse_num(d)? * 100.0)?;
            let pct = if negative { -pct } else { pct };
            Ok(fan(targets, Val::Pct(pct)))
        }
        ValueKind::Number => {
            let mut n = parse_num(suffix.unwrap_or_default())?;
            if negative {
                n = -n;
            }
            Ok(fan(targets, Val::Num(n)))
        }
        ValueKind::Percent => {
            let s = suffix.unwrap_or_default();
            let mut pct = parse_num(s.strip_suffix('%').unwrap_or(s))?;
            if negative {
                pct = -pct;
            }
            Ok(fan(targets, Val::Pct(pct)))
        }
        ValueKind::Arbitrary => {
            let Some(text) = arbitrary::text_of(arb, suffix) else {
                // The shape contract accepted but no group rode — the
                // drift arm, typed like every other.
                return Err(Reject::Value {
                    reason: REASON_SHAPE_MISMATCH,
                });
            };
            match arbitrary::classify(&text, alpha, &row) {
                Ok(val) => arbitrary_val_decls(val, negative, alpha, row, tokens),
                Err(rejection) => match arbitrary::map_rejection(rejection) {
                    // The silent sibling-row skip: this row never
                    // matched the value's class (another row of the
                    // root takes it — the run walks' deferred-reason
                    // pass renders the boundary when none did).
                    Ok(None) => Ok(Vec::new()),
                    Ok(Some(reason)) => Err(Reject::Value { reason }),
                    Err(reject) => Err(reject),
                },
            }
        }
        // Flattened away before the walk; the arm exists so the match
        // stays total without an unreachable panic.
        ValueKind::AnyOf(_) => Err(Reject::Value {
            reason: REASON_SHAPE_MISMATCH,
        }),
    }
}

/// The classified arbitrary value → the row's declarations (the IR
/// form's lowering half). Scalars flip under the row-legal negation;
/// colours arrive baked (the audited parser + premultiplied identity);
/// token references ride the theme pass's own path.
fn arbitrary_val_decls(
    val: ArbVal,
    negative: bool,
    alpha: Option<f32>,
    row: UtilityRow,
    tokens: Option<&TokenTable>,
) -> Result<Vec<Decl>, Reject> {
    let targets = arbitrary::targets_of(&row);
    match val {
        ArbVal::Token { name } => match tokens {
            Some(table) => token_decls(Some(&name), alpha, targets, table),
            None => Err(Reject::Value {
                reason: arbitrary::REASON_ARBITRARY_TABLE,
            }),
        },
        ArbVal::Colour { rgba } => Ok(fan(targets, Val::Rgba(rgba))),
        ArbVal::Px(v) => Ok(fan(targets, Val::Px(flip(v, negative)))),
        ArbVal::Rem(v) => Ok(fan(targets, Val::Rem(flip(v, negative)))),
        ArbVal::Pct(v) => Ok(fan(targets, Val::Pct(flip(v, negative)))),
        ArbVal::Num(v) => Ok(fan(targets, Val::Num(flip(v, negative)))),
    }
}

/// The row-legal negation flip (the value's own sign parses naturally;
/// the utility's `-` prefix flips where the row allows it).
fn flip(v: f32, negative: bool) -> f32 {
    if negative { -v } else { v }
}

/// The colour-token path: the suffix IS the token name (`"red-500"`,
/// `"primary"`), the table resolves it per its tier, and the caller's
/// alpha — already normalized — rides the colour-form value. `Tok`
/// carries it as the slot's alpha; a baked literal takes the
/// premultiplied bake; a baked dark pair REFUSES (the pair form has no
/// alpha arm — the runtime tier is the alternative).
fn token_decls(
    suffix: Option<&str>,
    alpha: Option<f32>,
    targets: &'static [Prop],
    tokens: &TokenTable,
) -> Result<Vec<Decl>, Reject> {
    let Some(name) = suffix else {
        // Unreachable through `lookup` (the colour arm accepts a suffix
        // only); the refusal stays typed regardless.
        return Err(Reject::Value {
            reason: REASON_SHAPE_MISMATCH,
        });
    };
    let Some(id) = tokens.id(name) else {
        return Err(Reject::Value {
            reason: REASON_NOT_A_TOKEN,
        });
    };
    let val = tokens.resolve(id, alpha).map_err(reject_resolve)?;
    Ok(fan(targets, val))
}

/// The table's typed refusals, mapped onto the value pass's own boundary
/// set: an unknown id is the not-a-token miss (the id came from this
/// table, so the arm is read-side drift), an alpha on a pair slot or a
/// non-finite alpha is the modifier boundary (the raw modifier text
/// rides the diagnostic).
fn reject_resolve(err: ResolveError) -> Reject {
    match err {
        ResolveError::UnknownId(_) => Reject::Value {
            reason: REASON_NOT_A_TOKEN,
        },
        ResolveError::AlphaOnPair | ResolveError::AlphaNotFinite(_) => Reject::Modifier,
    }
}

/// The spacing scale: `n × 0.25rem` (the `tailwind` config emits rem —
/// GPUI has `rems()`, so rem honours the window's rem size), the `px`
/// compat config bakes `n × 0.25 × rem_px`. The `px` step is one
/// absolute pixel in both modes. Negation flips the sign.
fn spacing_decls(
    suffix: Option<&str>,
    negative: bool,
    targets: &'static [Prop],
    config: Config,
) -> Result<Vec<Decl>, Reject> {
    let s = suffix.unwrap_or_default();
    let val = if s == "px" {
        Val::Px(if negative { -1.0 } else { 1.0 })
    } else {
        let rem = finite(parse_num(s)? * SPACING_STEP_REM)?;
        match config.spacing {
            Spacing::Tailwind => Val::Rem(if negative { -rem } else { rem }),
            Spacing::Px => {
                // The bake can overflow to ±inf for a huge n against a
                // large `rem_px` — a typed refusal, never a non-finite
                // literal in the IR.
                let px = finite(rem * config.rem_px)?;
                Val::Px(if negative { -px } else { px })
            }
        }
    };
    Ok(fan(targets, val))
}

/// The `Val::Kw` slot of `suffix` inside the row's own keyword table —
/// AFTER the table-size guard (the colour table does not fit the u8
/// slot and must not truncate): an oversized table is the internal
/// `TokenOverflow` marker the caller resolves (the theme pass's
/// TokenTable, or the typed `REASON_TOKEN_TABLE` refusal).
fn keyword_slot(
    table: &'static [(&'static str, &'static str)],
    suffix: Option<&str>,
) -> Result<u8, Reject> {
    if table.len() > u8::MAX as usize {
        return Err(Reject::TokenOverflow);
    }
    let s = suffix.unwrap_or_default();
    table
        .iter()
        .position(|&(name, _)| name == s)
        .map(|idx| idx as u8)
        .ok_or(Reject::Value {
            reason: REASON_SHAPE_MISMATCH,
        })
}

/// The composite table's sibling (`text-{size}`'s (name, size,
/// line-height) rows): one slot shared by every target longhand.
fn composite_slot(
    table: &'static [(&'static str, &'static str, &'static str)],
    suffix: Option<&str>,
) -> Result<u8, Reject> {
    if table.len() > u8::MAX as usize {
        // A composite table is NOT theme-eligible (the token path lowers
        // colour keywords only), so the oversized-composite refusal is
        // typed here, never routed through the token seam.
        return Err(Reject::Value {
            reason: REASON_TOKEN_TABLE,
        });
    }
    let s = suffix.unwrap_or_default();
    table
        .iter()
        .position(|&(name, _, _)| name == s)
        .map(|idx| idx as u8)
        .ok_or(Reject::Value {
            reason: REASON_SHAPE_MISMATCH,
        })
}

/// The finite guard: a value that overflowed to ±inf (or arrived NaN)
/// is a typed refusal, never a non-finite literal in the IR. `parse_num`
/// guards the PARSES; this guards the PRODUCTS (`1e38 / 1e-38`, the
/// px bake against a large `rem_px`).
pub(crate) fn finite(val: f32) -> Result<f32, Reject> {
    if val.is_finite() {
        Ok(val)
    } else {
        Err(Reject::Value {
            reason: REASON_NON_FINITE,
        })
    }
}

pub(crate) fn parse_num(s: &str) -> Result<f32, Reject> {
    let n = s.parse::<f32>().map_err(|_| Reject::Value {
        reason: REASON_SHAPE_MISMATCH,
    })?;
    // `f32::from_str` accepts `nan`/`inf`/`infinity` (any case) — legal
    // parses, illegal v4 values.
    finite(n)
}

/// Fan the value out over the row's longhand targets, in the row's own
/// TRBL / TL-TR-BR-BL order (the layer re-sorts into cascade order).
pub(crate) fn fan(targets: &'static [Prop], val: Val) -> Vec<Decl> {
    targets.iter().map(|&prop| Decl { prop, val }).collect()
}

// ---------------------------------------------------------------------------
// The keyword annotations (the rust-tokens emit arm's read side —
// plan §3.1 emit (a))
// ---------------------------------------------------------------------------

/// The keyword-slot annotations for one candidate's row: `(Prop, text)`
/// pairs for every declaration the row fans, when the lowering rides the
/// row's OWN keyword/bare/composite tables — the texts the [`Val::Kw`]
/// slot indexes but a bare `Program` cannot recover (the slot is
/// row-relative; the codegen crate owns the names, plan §3.3). The
/// rust-tokens emit arm renders them as trailing comments; the
/// program-only emitters (the static-data arm, the interpreter) carry
/// the slot alone, honestly.
///
/// The arm selection mirrors [`row_to_decls`]'s (the same suffix rule,
/// the same AnyOf flattening and acceptance order) but only READS the
/// row tables — no lowering, no refusals: a row that would reject
/// yields no annotations, and the caller renders the fragment only from
/// rows that already lowered. Arbitrary candidates never consult this
/// (the arbitrary arm lowers concrete values, never row-table slots).
pub(crate) fn kw_annotations(row: UtilityRow, body: &str) -> Vec<(Prop, &'static str)> {
    let Ok(suffix) = suffix_of(body, row.root) else {
        return Vec::new();
    };
    let mut arms = Vec::new();
    flatten(row.kind, &mut arms);
    for arm in arms {
        match arm {
            // The whole utility IS the value (`hidden` → `none`): one
            // text fans over every target, slot 0.
            ValueKind::Bare(css) if suffix.is_none() => {
                return row.targets.iter().map(|&prop| (prop, css)).collect();
            }
            // The closed keyword table: the suffix's own text fans over
            // every target (the per-target variety is the composite's).
            ValueKind::Keywords(table) => {
                let s = suffix.unwrap_or_default();
                if u8::try_from(table.len()).is_ok()
                    && let Some(text) = table
                        .iter()
                        .find(|&&(name, _)| name == s)
                        .map(|&(_, text)| text)
                {
                    return row.targets.iter().map(|&prop| (prop, text)).collect();
                }
            }
            // The composite pair (`text-sm` → font-size + line-height):
            // targets[0] takes the primary text, targets[1] the
            // secondary — the same per-target split the CSS renderer
            // uses (the IR carries one shared slot; the texts are the
            // annotation).
            ValueKind::Composite(table) => {
                let s = suffix.unwrap_or_default();
                if u8::try_from(table.len()).is_ok()
                    && let Some(idx) = table.iter().position(|&(name, _, _)| name == s)
                {
                    let (_, primary, secondary) = table[idx];
                    return row
                        .targets
                        .iter()
                        .enumerate()
                        .map(|(i, &prop)| (prop, if i == 0 { primary } else { secondary }))
                        .collect();
                }
            }
            _ => {}
        }
    }
    Vec::new()
}

// ---------------------------------------------------------------------------
// The CSS form (the Target::Css arm — plan §3.1's emit stage, option (d))
// ---------------------------------------------------------------------------

/// One row's CSS-rendered value: the base-condition text and, when the
/// value carries a baked `(light, dark)` pair, the dark text the CSS
/// target renders under its own `:where(.dark, .dark *)` selector.
/// Token references render as `var(--color-…)` (tier-swapped by the
/// theme's own `.dark` block), so they are dark-less here by design.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CssValue {
    /// The value text under the rule's own conditions.
    pub(crate) base: String,
    /// The dark-mode text, when the value IS a baked pair (the runtime
    /// token form carries none — the var is theme-swapped upstream).
    pub(crate) dark: Option<String>,
}

/// How a `ValueKind::Number` value renders — the per-root form table
/// (`NUMBER_FORMS`): the unit suffix, the `repeat()`/`span()` composite
/// forms the grid rows carry, and the property override where the
/// longhand's `Prop::name` is the paint slot, not the width slot
/// (`stroke-2` is `stroke-width`, the `Stroke` prop stays the colour arm).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NumForm {
    /// `{n}px` — the border/outline/ring/offset widths.
    Px,
    /// `{n}deg` — rotate/skew.
    Deg,
    /// `{n}%` — opacity and the scale family (v4 renders `opacity: 50%`).
    Pct,
    /// `{n}` — the unitless scales (flex-grow, order, z-index, columns).
    Unit,
    /// `repeat({n}, minmax(0, 1fr))` — the `grid-cols-N`/`grid-rows-N`
    /// template composite the row's `Num` models.
    Repeat,
    /// `span {n}` — the `col-span-N`/`row-span-N` placement form.
    Span,
    /// `{n}` — the grid line numbers (`col-start-2` is line 2; the same
    /// text as [`NumForm::Unit`], a named arm for the audit table).
    Line,
}

/// The `Number`-row rendering table, keyed by root. The registry tests
/// in `tests/css.rs` walk `ROWS` and fail when a `Number`-bearing root
/// is missing here (registry/emitter drift is a test failure, never a
/// wrong stylesheet).
pub(crate) const NUMBER_FORMS: &[(&str, NumForm, Option<&str>)] = &[
    ("grow", NumForm::Unit, None),
    ("shrink", NumForm::Unit, None),
    ("order", NumForm::Unit, None),
    ("z", NumForm::Unit, None),
    ("opacity", NumForm::Pct, None),
    ("columns", NumForm::Unit, None),
    ("border", NumForm::Px, None),
    ("border-t", NumForm::Px, None),
    ("border-r", NumForm::Px, None),
    ("border-b", NumForm::Px, None),
    ("border-l", NumForm::Px, None),
    ("border-x", NumForm::Px, None),
    ("border-y", NumForm::Px, None),
    ("border-s", NumForm::Px, None),
    ("border-e", NumForm::Px, None),
    ("outline", NumForm::Px, None),
    ("outline-offset", NumForm::Px, None),
    ("ring", NumForm::Px, None),
    ("ring-offset", NumForm::Px, None),
    ("underline-offset", NumForm::Px, None),
    ("rotate", NumForm::Deg, None),
    ("scale", NumForm::Pct, None),
    ("scale-x", NumForm::Pct, None),
    ("scale-y", NumForm::Pct, None),
    ("skew-x", NumForm::Deg, None),
    ("skew-y", NumForm::Deg, None),
    ("stroke", NumForm::Px, Some("stroke-width")),
    ("grid-cols", NumForm::Repeat, None),
    ("grid-rows", NumForm::Repeat, None),
    ("col-span", NumForm::Span, None),
    ("row-span", NumForm::Span, None),
    ("col-start", NumForm::Line, None),
    ("col-end", NumForm::Line, None),
    ("row-start", NumForm::Line, None),
    ("row-end", NumForm::Line, None),
];

/// Lower ONE matched row for one candidate body into CSS-form values —
/// the [`Target`](crate::Target)`::Css` arm's value pass. The arm walk
/// is the SAME shape resolution [`row_to_decls`] runs (same negation
/// guard, same suffix rule, same AnyOf order, same modifier-rides-
/// colour-only rule, same token overflow routing); what differs is the
/// RENDERER: the row's own keyword tables carry the CSS text, so the
/// rendering never needs the IR's row-relative `Kw` slot — the one
/// ambiguity the in-pipeline emit exists to avoid.
///
/// # Errors
/// The same [`Reject`] set as [`row_to_decls`] (the diagnostic layer
/// renders them identically for both forms).
pub(crate) fn row_to_css(
    row: UtilityRow,
    body: &str,
    negative: bool,
    alpha: Option<f32>,
    tokens: Option<&TokenTable>,
    config: Config,
    arb: Option<Arb<'_>>,
) -> Result<Vec<CssDecl>, Reject> {
    if negative && !row.negative {
        return Err(Reject::Negation);
    }
    let suffix = suffix_of(body, row.root)?;
    let mut arms = Vec::new();
    flatten(row.kind, &mut arms);
    for arm in arms {
        // A candidate carrying an arbitrary group lowers through the
        // Arbitrary arm ONLY (the IR form's own rule — the two walks
        // share the arm-selection semantics).
        if arb.is_some() && !matches!(arm, ValueKind::Arbitrary) {
            continue;
        }
        if !arm.accepts(suffix) {
            continue;
        }
        let rendered = arm_to_css(arm, suffix, negative, alpha, row, tokens, config, arb)?;
        // A modifier never half-applies (the same contract as the IR
        // form): it rides only when EVERY rendered value is colour-form.
        if alpha.is_some() && !rendered.iter().all(|r| r.colour_form) {
            return Err(Reject::Modifier);
        }
        return Ok(rendered
            .into_iter()
            .map(|r| CssDecl {
                prop: r.prop,
                prop_name: r.prop_name,
                value: r.value,
            })
            .collect());
    }
    Err(Reject::Value {
        reason: REASON_SHAPE_MISMATCH,
    })
}

/// One rendered declaration: the IR longhand, the CSS property name it
/// renders as (usually [`Prop::name`]; the `stroke` width arm renders
/// `stroke-width`), and the value text. The CSS arm's emit unit.
pub(crate) struct CssDecl {
    /// The longhand slot (cascade/sort key).
    pub(crate) prop: Prop,
    /// The property name in the rendered CSS.
    pub(crate) prop_name: &'static str,
    /// The rendered value(s).
    pub(crate) value: CssValue,
}

/// One rendered declaration plus its colour-form flag (the modifier
/// rule's input). Internal to this module's css rendering.
struct RenderedCss {
    prop: Prop,
    prop_name: &'static str,
    value: CssValue,
    colour_form: bool,
}

fn arm_to_css(
    arm: ValueKind,
    suffix: Option<&str>,
    negative: bool,
    alpha: Option<f32>,
    row: UtilityRow,
    tokens: Option<&TokenTable>,
    config: Config,
    arb: Option<Arb<'_>>,
) -> Result<Vec<RenderedCss>, Reject> {
    let root = row.root;
    let targets = row.targets;
    match arm {
        ValueKind::Bare(css) => {
            // `bare` is never colour-form: the modifier refuses (the
            // caller's post-check), matching the IR form's boundary.
            Ok(targets
                .iter()
                .map(|&prop| RenderedCss {
                    prop,
                    prop_name: prop.name(),
                    value: CssValue {
                        base: css.to_string(),
                        dark: None,
                    },
                    colour_form: false,
                })
                .collect())
        }
        ValueKind::Keywords(table) => match keyword_text(table, suffix) {
            Ok(css) => Ok(fan_css(targets, &css, false)),
            // The oversized-table boundary: the theme pass owns it.
            Err(Reject::TokenOverflow) => match tokens {
                Some(table) => token_css(suffix, alpha, targets, table),
                None => Err(Reject::Value {
                    reason: REASON_TOKEN_TABLE,
                }),
            },
            Err(other) => Err(other),
        },
        ValueKind::Composite(table) => {
            let slot = composite_slot(table, suffix)?;
            // One slot, two texts: targets[0] takes the primary value,
            // targets[1] the secondary (the `text-{size}` pair's
            // (font-size, line-height)).
            let (_, primary, secondary) = table[slot as usize];
            let mut out = Vec::with_capacity(targets.len());
            for (i, &prop) in targets.iter().enumerate() {
                out.push(RenderedCss {
                    prop,
                    prop_name: prop.name(),
                    value: CssValue {
                        base: if i == 0 {
                            primary.to_string()
                        } else {
                            secondary.to_string()
                        },
                        dark: None,
                    },
                    colour_form: false,
                });
            }
            Ok(out)
        }
        ValueKind::Spacing => spacing_css(suffix, negative, targets, config),
        ValueKind::Fraction => {
            let (n, d) = suffix
                .and_then(|s| s.split_once('/'))
                .ok_or(Reject::Value {
                    reason: REASON_SHAPE_MISMATCH,
                })?;
            let pct = finite(parse_num(n)? / parse_num(d)? * 100.0)?;
            let pct = if negative { -pct } else { pct };
            Ok(fan_css(targets, &format!("{pct}%"), false))
        }
        ValueKind::Number => {
            let mut n = parse_num(suffix.unwrap_or_default())?;
            if negative {
                n = -n;
            }
            number_css(root, n, targets)
        }
        ValueKind::Percent => {
            let s = suffix.unwrap_or_default();
            let mut pct = parse_num(s.strip_suffix('%').unwrap_or(s))?;
            if negative {
                pct = -pct;
            }
            Ok(fan_css(targets, &format!("{pct}%"), false))
        }
        ValueKind::Arbitrary => {
            let Some(text) = arbitrary::text_of(arb, suffix) else {
                // The shape contract accepted but no group rode — the
                // drift arm, typed like every other.
                return Err(Reject::Value {
                    reason: REASON_SHAPE_MISMATCH,
                });
            };
            match arbitrary::classify(&text, alpha, &row) {
                Ok(val) => arbitrary_val_css(&text, val, negative, alpha, row, tokens),
                Err(rejection) => match arbitrary::map_rejection(rejection) {
                    // The silent sibling-row skip (the IR form's own arm
                    // documents it): this row never matched the class.
                    Ok(None) => Ok(Vec::new()),
                    Ok(Some(reason)) => Err(Reject::Value { reason }),
                    Err(reject) => Err(reject),
                },
            }
        }
        // Flattened away before the walk (the IR form's own arm).
        ValueKind::AnyOf(_) => Err(Reject::Value {
            reason: REASON_SHAPE_MISMATCH,
        }),
    }
}

/// The classified arbitrary value → the row's rendered declarations
/// (the CSS form's lowering half — the same resolution the IR form
/// lowers, rendered from the value's own source of truth: the verbatim
/// text for colours without a modifier (the corpus's hex form), the
/// unit text for the scalars, the token path's forms for the var
/// references, the baked `rgb(r g b / a%)` when the modifier baked).
fn arbitrary_val_css(
    text: &arbitrary::ArbText<'_>,
    val: ArbVal,
    negative: bool,
    alpha: Option<f32>,
    row: UtilityRow,
    tokens: Option<&TokenTable>,
) -> Result<Vec<RenderedCss>, Reject> {
    let targets = arbitrary::targets_of(&row);
    match val {
        ArbVal::Token { name } => match tokens {
            Some(table) => token_css(Some(&name), alpha, targets, table),
            None => Err(Reject::Value {
                reason: arbitrary::REASON_ARBITRARY_TABLE,
            }),
        },
        ArbVal::Colour { rgba } => {
            // The verbatim value text when the value renders as-is (the
            // corpus's `#0ea5e9` form); the baked space-separated form
            // when the modifier consumed the literal.
            let value = match alpha {
                None => text.value.trim().to_string(),
                Some(_) => rgba_text(rgba),
            };
            Ok(fan_css(targets, &value, true))
        }
        ArbVal::Px(v) => Ok(fan_css(
            targets,
            &format!("{}px", num_text(flip(v, negative))),
            false,
        )),
        ArbVal::Rem(v) => Ok(fan_css(
            targets,
            &format!("{}rem", num_text(flip(v, negative))),
            false,
        )),
        ArbVal::Pct(v) => Ok(fan_css(
            targets,
            &format!("{}%", num_text(flip(v, negative))),
            false,
        )),
        ArbVal::Num(v) => Ok(fan_css(targets, &num_text(flip(v, negative)), false)),
    }
}

/// The scalar number text: whole values render bare (`32px`, not
/// `32.0px` — the corpus's form), the shortest round-trip otherwise.
fn num_text(v: f32) -> String {
    if v.fract() == 0.0 && v.abs() < 1e9 {
        format!("{v:.0}")
    } else {
        format!("{v}")
    }
}

/// Fan one text over the targets.
fn fan_css(targets: &'static [Prop], text: &str, colour_form: bool) -> Vec<RenderedCss> {
    targets
        .iter()
        .map(|&prop| RenderedCss {
            prop,
            prop_name: prop.name(),
            value: CssValue {
                base: text.to_string(),
                dark: None,
            },
            colour_form,
        })
        .collect()
}

/// The suffix's CSS text inside the row's OWN keyword table — the
/// rendering-side sibling of [`keyword_slot`] (same overflow guard, the
/// text instead of the slot index).
fn keyword_text(
    table: &'static [(&'static str, &'static str)],
    suffix: Option<&str>,
) -> Result<String, Reject> {
    if table.len() > u8::MAX as usize {
        return Err(Reject::TokenOverflow);
    }
    let s = suffix.unwrap_or_default();
    table
        .iter()
        .find(|&&(name, _)| name == s)
        .map(|&(_, css)| css.to_string())
        .ok_or(Reject::Value {
            reason: REASON_SHAPE_MISMATCH,
        })
}

/// The spacing scale rendered: `n × 0.25rem` (the `tailwind` config), or
/// the px bake under the compat config. Same math and refusal set as the
/// IR form's [`spacing_decls`].
fn spacing_css(
    suffix: Option<&str>,
    negative: bool,
    targets: &'static [Prop],
    config: Config,
) -> Result<Vec<RenderedCss>, Reject> {
    let s = suffix.unwrap_or_default();
    let text = if s == "px" {
        if negative {
            "-1px".to_string()
        } else {
            "1px".to_string()
        }
    } else {
        let rem = finite(parse_num(s)? * SPACING_STEP_REM)?;
        match config.spacing {
            Spacing::Tailwind => {
                let rem = if negative { -rem } else { rem };
                format!("{rem}rem")
            }
            Spacing::Px => {
                let px = finite(rem * config.rem_px)?;
                let px = if negative { -px } else { px };
                format!("{px}px")
            }
        }
    };
    Ok(fan_css(targets, &text, false))
}

/// The `Number`-row rendering: the per-root [`NUMBER_FORMS`] table is the
/// audit surface (the registry tests pin every `Number`-bearing root into
/// it). A root missing from the table is the shape-mismatch drift arm —
/// never a guessed unit.
fn number_css(
    root: &'static str,
    n: f32,
    targets: &'static [Prop],
) -> Result<Vec<RenderedCss>, Reject> {
    let Some(&(_, form, prop_override)) = NUMBER_FORMS.iter().find(|(r, _, _)| *r == root) else {
        return Err(Reject::Value {
            reason: REASON_SHAPE_MISMATCH,
        });
    };
    let text = match form {
        NumForm::Px => format!("{n}px"),
        NumForm::Deg => format!("{n}deg"),
        NumForm::Pct => format!("{n}%"),
        NumForm::Unit | NumForm::Line => format!("{n}"),
        NumForm::Repeat => format!("repeat({n}, minmax(0, 1fr))"),
        NumForm::Span => format!("span {n}"),
    };
    let Some(override_name) = prop_override else {
        return Ok(fan_css(targets, &text, false));
    };
    // The override renames the rendered property for every target (the
    // `stroke` width arm): the Prop slot stays the IR's, the CSS text
    // takes the real property name.
    Ok(targets
        .iter()
        .map(|&prop| RenderedCss {
            prop,
            prop_name: override_name,
            value: CssValue {
                base: text.clone(),
                dark: None,
            },
            colour_form: false,
        })
        .collect())
}

/// The colour-token path rendered — the CSS-form sibling of
/// [`token_decls`]: the tier resolves the token, the alpha rides as the
/// premultiplied bake (literal), the `color-mix` form (runtime token) or
/// a typed refusal (alpha on a baked pair, unknown token, non-finite).
fn token_css(
    suffix: Option<&str>,
    alpha: Option<f32>,
    targets: &'static [Prop],
    tokens: &TokenTable,
) -> Result<Vec<RenderedCss>, Reject> {
    let Some(name) = suffix else {
        // Unreachable through `lookup` (the colour arm accepts a suffix
        // only); the refusal stays typed regardless.
        return Err(Reject::Value {
            reason: REASON_SHAPE_MISMATCH,
        });
    };
    let Some(id) = tokens.id(name) else {
        return Err(Reject::Value {
            reason: REASON_NOT_A_TOKEN,
        });
    };
    // The resolve's tier decides the rendered form; its refusals map
    // onto the same boundary set as the IR form's [`reject_resolve`].
    let rendered = match tokens.resolve(id, alpha) {
        Ok(Val::Rgba(rgba)) => CssValue {
            base: rgba_text(rgba),
            dark: None,
        },
        Ok(Val::Tok {
            alpha: slot_alpha, ..
        }) => {
            // The id came from this name, so the rendered var is the
            // same token; the RESOLVED alpha rides (the audited clamp,
            // exactly what the IR form's `Tok` slot carries). The slot's
            // alpha is f32 (1.0 when no modifier rode); it re-enters the
            // option form only when it is a real transparency — a plain
            // `var()` renders otherwise.
            CssValue {
                base: token_text(name, (slot_alpha < 1.0).then_some(slot_alpha)),
                dark: None,
            }
        }
        Ok(Val::ModePair(slot)) => {
            let Some((light, dark)) = tokens.pair_at(slot) else {
                // The slot came from this table; a miss is read-side
                // drift, typed not panned.
                return Err(Reject::Value {
                    reason: REASON_NOT_A_TOKEN,
                });
            };
            CssValue {
                base: rgba_text(light),
                dark: Some(rgba_text(dark)),
            }
        }
        // The colour path only resolves colour-form values; a numeric
        // or keyword escape is lowering drift, typed not rendered.
        Ok(Val::Px(_) | Val::Rem(_) | Val::Pct(_) | Val::Num(_) | Val::Kw(_)) => {
            return Err(Reject::Value {
                reason: REASON_SHAPE_MISMATCH,
            });
        }
        Err(ResolveError::UnknownId(_)) => {
            return Err(Reject::Value {
                reason: REASON_NOT_A_TOKEN,
            });
        }
        Err(ResolveError::AlphaOnPair) => return Err(Reject::Modifier),
        Err(ResolveError::AlphaNotFinite(_)) => return Err(Reject::Modifier),
    };
    Ok(targets
        .iter()
        .map(|&prop| RenderedCss {
            prop,
            prop_name: prop.name(),
            value: CssValue {
                base: rendered.base.clone(),
                dark: rendered.dark.clone(),
            },
            colour_form: true,
        })
        .collect())
}

/// An `[u8; 4]` rgba rendered in the modern space-separated form; the
/// alpha rides as a percentage when the bake left any transparency.
pub(crate) fn rgba_text(rgba: [u8; 4]) -> String {
    let [r, g, b, a] = rgba;
    if a == 255 {
        format!("rgb({r} {g} {b})")
    } else {
        let pct = (a as f32) / 255.0 * 100.0;
        format!("rgb({r} {g} {b} / {})", pct_text(pct))
    }
}

/// A runtime token rendered the way v4 emits it: the `--color-` var, and
/// the opacity modifier as the `color-mix` identity (plan §2 — mixing
/// with transparent in the premultiplied space scales alpha only).
pub(crate) fn token_text(name: &str, alpha: Option<f32>) -> String {
    let var = format!("var(--color-{name})");
    match alpha {
        None => var,
        Some(a) => format!(
            "color-mix(in oklab, {var} {}%, transparent)",
            pct_text(a * 100.0)
        ),
    }
}

/// The alpha percentage text: the f32 product rounded to three decimals,
/// trailing zeros trimmed (`0.9 → "90"`, `0.37 → "37"`, the u8 bake's
/// `229.5/255 → "90.196"`). Deterministic, shortest-to-round-trip is the
/// IR's own `Display` discipline; three decimals keep the emitted
/// stylesheet byte-stable across platforms.
fn pct_text(v: f32) -> String {
    let rounded = (v * 1000.0).round() / 1000.0;
    let s = format!("{rounded}");
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s
    }
}

// ---------------------------------------------------------------------------
// The rendering-table audit (the drift catch, test-pinned in place — the
// table is crate-internal, so the audit lives beside it)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use aeolus_data::ValueKind;
    use aeolus_data::rows::ROWS;

    fn has_number_arm(kind: ValueKind) -> bool {
        let mut arms = Vec::new();
        flatten(kind, &mut arms);
        arms.iter().any(|k| matches!(k, ValueKind::Number))
    }

    /// Every registry row that lowers `Number` values must have its root
    /// in [`NUMBER_FORMS`] — a new Number row without a render form is a
    /// test failure here, never a guessed unit at render time.
    #[test]
    fn every_number_row_has_a_render_form() {
        for row in ROWS {
            if has_number_arm(row.kind) {
                assert!(
                    NUMBER_FORMS.iter().any(|(root, _, _)| *root == row.root),
                    "root {:?} lowers Number values but has no NUMBER_FORMS entry",
                    row.root
                );
            }
        }
    }

    /// The reverse: no phantom render forms for roots the registry
    /// numbers nowhere.
    #[test]
    fn every_render_form_has_a_number_row() {
        for (root, _, _) in NUMBER_FORMS {
            assert!(
                ROWS.iter()
                    .any(|row| row.root == *root && has_number_arm(row.kind)),
                "NUMBER_FORMS root {root:?} has no Number-bearing registry row"
            );
        }
    }

    /// The alpha percentage text: three decimals, trailing zeros
    /// trimmed — the byte-stable rendering the theme tests pin.
    #[test]
    fn alpha_percent_text_is_deterministic() {
        assert_eq!(pct_text(90.0), "90");
        assert_eq!(pct_text(37.0), "37");
        assert_eq!(pct_text(100.0), "100");
        let a_pct = 230.0_f32 / 255.0 * 100.0;
        assert_eq!(pct_text(a_pct), "90.196");
    }
}
