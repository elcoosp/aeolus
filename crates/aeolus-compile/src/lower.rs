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
//! * `Arbitrary` → the P2 value pass (unit grammar + type hints).
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
use aeolus_theme::{ResolveError, TokenTable};

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

/// Arbitrary `[...]`/`(...)` values: the unit grammar is the P2 value pass.
pub(crate) const REASON_ARBITRARY: &str =
    "arbitrary values lower in the P2 value pass: the bracket/paren \
     grammar needs the unit table (length/percentage/colour type hints)";

/// The seam's discovered IR-budget constraint: a keyword table larger
/// than the u8 `Val::Kw` slot (the colour table's 270 entries) lowers
/// through the theme pass's TokenTable when the caller supplies one
/// (plan §3.4), and refuses typed when not — never truncation.
pub(crate) const REASON_TOKEN_TABLE: &str =
    "the keyword table exceeds the IR's u8 keyword slot: colour tokens \
     lower through the theme pass's TokenTable (plan §3.4 T2)";

/// The suffix is neither a theme token nor lowerable through the u8
/// keyword slot — the table-driven miss the linter's "did you mean"
/// surface renders (`TokenTable::names()` is the checklist).
pub(crate) const REASON_NOT_A_TOKEN: &str =
    "the suffix is not a colour token in the supplied TokenTable (see \
     TokenTable::names()) and the palette table exceeds the u8 keyword slot";

/// The f32 grammar's non-finite spellings (`nan`, `inf`, `1e40`) are
/// legal PARSES and illegal v4 values: the value pass refuses them — a
/// non-finite literal never reaches the IR (the snapshot writer's
/// string-encoded infinities exist because the reader is strict; the
/// IR never expected to carry them in the first place).
pub(crate) const REASON_NON_FINITE: &str =
    "the suffix parses as a non-finite f32 (nan/inf): the v4 value grammar \
     is finite decimals, so the value pass refuses instead of lowering";

/// The shape contract accepted the suffix but no arm lowered it —
/// registry/lowering drift, reportable, never a panic.
pub(crate) const REASON_SHAPE_MISMATCH: &str =
    "the row's shape contract accepted the suffix but no arm lowered it \
     (registry/compile drift — please report)";

/// Lower ONE matched row for one candidate body. The registry guarantees
/// the body addresses the row's root and at least one of its value
/// shapes accepts the suffix; everything past that guarantee is typed.
/// `alpha` is the normalized opacity modifier (`/90` → `Some(0.9)`) and
/// rides COLOUR-form decls only; `tokens` is the theme pass's table
/// (plan §3.4), which owns the oversized colour-keyword surface.
pub(crate) fn row_to_decls(
    row: UtilityRow,
    body: &str,
    negative: bool,
    alpha: Option<f32>,
    tokens: Option<&TokenTable>,
    config: &Config,
) -> Result<Vec<Decl>, Reject> {
    if negative && !row.negative {
        return Err(Reject::Negation);
    }
    let suffix = suffix_of(body, row.root)?;
    let mut arms = Vec::new();
    flatten(row.kind, &mut arms);
    for arm in arms {
        if !arm.accepts(suffix) {
            continue;
        }
        let decls = arm_to_decls(arm, suffix, negative, alpha, row.targets, tokens, config)?;
        // A modifier never half-applies: it rides only when EVERY decl
        // the row produced is colour-form (the token path already
        // applied it; anything else refuses the whole row).
        if alpha.is_some()
            && !decls.iter().all(|decl| is_colour_val(decl.val))
        {
            return Err(Reject::Modifier);
        }
        return Ok(decls);
    }
    Err(Reject::Value { reason: REASON_SHAPE_MISMATCH })
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
fn suffix_of<'a>(body: &'a str, root: &str) -> Result<Option<&'a str>, Reject> {
    if body == root {
        return Ok(None);
    }
    body.strip_prefix(root)
        .and_then(|rest| rest.strip_prefix('-'))
        .map(Some)
        .ok_or(Reject::Value { reason: REASON_SHAPE_MISMATCH })
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
    targets: &'static [Prop],
    tokens: Option<&TokenTable>,
    config: &Config,
) -> Result<Vec<Decl>, Reject> {
    match arm {
        ValueKind::Bare(_) => Ok(fan(targets, Val::Kw(0))),
        ValueKind::Keywords(table) => match keyword_slot(table, suffix) {
            Ok(slot) => Ok(fan(targets, Val::Kw(slot))),
            // The oversized-table boundary: the theme pass owns it.
            Err(Reject::TokenOverflow) => match tokens {
                Some(table) => token_decls(suffix, alpha, targets, table),
                None => Err(Reject::Value { reason: REASON_TOKEN_TABLE }),
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
                .ok_or(Reject::Value { reason: REASON_SHAPE_MISMATCH })?;
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
        ValueKind::Arbitrary => Err(Reject::Value { reason: REASON_ARBITRARY }),
        // Flattened away before the walk; the arm exists so the match
        // stays total without an unreachable panic.
        ValueKind::AnyOf(_) => Err(Reject::Value { reason: REASON_SHAPE_MISMATCH }),
    }
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
        return Err(Reject::Value { reason: REASON_SHAPE_MISMATCH });
    };
    let Some(id) = tokens.id(name) else {
        return Err(Reject::Value { reason: REASON_NOT_A_TOKEN });
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
        ResolveError::UnknownId(_) => Reject::Value { reason: REASON_NOT_A_TOKEN },
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
    config: &Config,
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
        .ok_or(Reject::Value { reason: REASON_SHAPE_MISMATCH })
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
        return Err(Reject::Value { reason: REASON_TOKEN_TABLE });
    }
    let s = suffix.unwrap_or_default();
    table
        .iter()
        .position(|&(name, _, _)| name == s)
        .map(|idx| idx as u8)
        .ok_or(Reject::Value { reason: REASON_SHAPE_MISMATCH })
}

/// The finite guard: a value that overflowed to ±inf (or arrived NaN)
/// is a typed refusal, never a non-finite literal in the IR. `parse_num`
/// guards the PARSES; this guards the PRODUCTS (`1e38 / 1e-38`, the
/// px bake against a large `rem_px`).
fn finite(val: f32) -> Result<f32, Reject> {
    if val.is_finite() {
        Ok(val)
    } else {
        Err(Reject::Value { reason: REASON_NON_FINITE })
    }
}

fn parse_num(s: &str) -> Result<f32, Reject> {
    let n = s
        .parse::<f32>()
        .map_err(|_| Reject::Value { reason: REASON_SHAPE_MISMATCH })?;
    // `f32::from_str` accepts `nan`/`inf`/`infinity` (any case) — legal
    // parses, illegal v4 values.
    finite(n)
}

/// Fan the value out over the row's longhand targets, in the row's own
/// TRBL / TL-TR-BR-BL order (the layer re-sorts into cascade order).
fn fan(targets: &'static [Prop], val: Val) -> Vec<Decl> {
    targets.iter().map(|&prop| Decl { prop, val }).collect()
}
