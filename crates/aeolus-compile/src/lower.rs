//! The value pass: one registry row + one suffix → longhand `Decl`s
//! (architecture plan §3.1's "MIR resolved vs theme; longhand decls",
//! minus the theme half — tokens lower in the theme pass).
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
//!   honest exception the seam surfaced: the colour table is 270
//!   entries and the IR's `Kw` slot is a u8, so colour tokens lower
//!   through the theme pass's `TokenTable` (plan §3.4 T2), typed and
//!   reported, never truncated;
//! * `Arbitrary` → the P2 value pass (unit grammar + type hints).
//!
//! Every refusal is a typed [`Reject`] — the reachable public path
//! carries no panic (CONTRIBUTING §2).

use aeolus_data::tables::SPACING_STEP_REM;
use aeolus_data::{UtilityRow, ValueKind};
use aeolus_ir::{Decl, Prop, Val};

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
}

/// Arbitrary `[...]`/`(...)` values: the unit grammar is the P2 value pass.
pub(crate) const REASON_ARBITRARY: &str =
    "arbitrary values lower in the P2 value pass: the bracket/paren \
     grammar needs the unit table (length/percentage/colour type hints)";

/// The seam's discovered IR-budget constraint: a keyword table larger
/// than the u8 `Val::Kw` slot (the colour table's 270 entries) must go
/// through the theme pass's TokenTable, not through truncation.
pub(crate) const REASON_TOKEN_TABLE: &str =
    "the keyword table exceeds the IR's u8 keyword slot: colour tokens \
     lower through the theme pass's TokenTable (plan §3.4 T2)";

/// The shape contract accepted the suffix but no arm lowered it —
/// registry/lowering drift, reportable, never a panic.
pub(crate) const REASON_SHAPE_MISMATCH: &str =
    "the row's shape contract accepted the suffix but no arm lowered it \
     (registry/compile drift — please report)";

/// Lower ONE matched row for one candidate body. The registry guarantees
/// the body addresses the row's root and at least one of its value
/// shapes accepts the suffix; everything past that guarantee is typed.
pub(crate) fn row_to_decls(
    row: UtilityRow,
    body: &str,
    negative: bool,
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
        return arm_to_decls(arm, suffix, negative, row.targets, config);
    }
    Err(Reject::Value { reason: REASON_SHAPE_MISMATCH })
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
    targets: &'static [Prop],
    config: &Config,
) -> Result<Vec<Decl>, Reject> {
    match arm {
        ValueKind::Bare(_) => Ok(fan(targets, Val::Kw(0))),
        ValueKind::Keywords(table) => {
            let slot = keyword_slot(table, suffix)?;
            Ok(fan(targets, Val::Kw(slot)))
        }
        ValueKind::Composite(table) => {
            let slot = composite_slot(table, suffix)?;
            Ok(fan(targets, Val::Kw(slot)))
        }
        ValueKind::Spacing => spacing_decls(suffix, negative, targets, config),
        ValueKind::Fraction => {
            let (n, d) = suffix
                .and_then(|s| s.split_once('/'))
                .ok_or(Reject::Value { reason: REASON_SHAPE_MISMATCH })?;
            let mut pct = parse_num(n)? / parse_num(d)? * 100.0;
            if negative {
                pct = -pct;
            }
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
        let rem = parse_num(s)? * SPACING_STEP_REM;
        match config.spacing {
            Spacing::Tailwind => Val::Rem(if negative { -rem } else { rem }),
            Spacing::Px => {
                let px = rem * config.rem_px;
                Val::Px(if negative { -px } else { px })
            }
        }
    };
    Ok(fan(targets, val))
}

/// The `Val::Kw` slot of `suffix` inside the row's own keyword table —
/// AFTER the table-size guard (the colour table does not fit the u8
/// slot and must not truncate).
fn keyword_slot(
    table: &'static [(&'static str, &'static str)],
    suffix: Option<&str>,
) -> Result<u8, Reject> {
    if table.len() > u8::MAX as usize {
        return Err(Reject::Value { reason: REASON_TOKEN_TABLE });
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
        return Err(Reject::Value { reason: REASON_TOKEN_TABLE });
    }
    let s = suffix.unwrap_or_default();
    table
        .iter()
        .position(|&(name, _, _)| name == s)
        .map(|idx| idx as u8)
        .ok_or(Reject::Value { reason: REASON_SHAPE_MISMATCH })
}

fn parse_num(s: &str) -> Result<f32, Reject> {
    s.parse::<f32>().map_err(|_| Reject::Value { reason: REASON_SHAPE_MISMATCH })
}

/// Fan the value out over the row's longhand targets, in the row's own
/// TRBL / TL-TR-BR-BL order (the layer re-sorts into cascade order).
fn fan(targets: &'static [Prop], val: Val) -> Vec<Decl> {
    targets.iter().map(|&prop| Decl { prop, val }).collect()
}
