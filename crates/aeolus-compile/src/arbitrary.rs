//! The arbitrary-value unit grammar (architecture plan §3.1's value
//! pass — the last named P2 value-half slice): one bracket/paren group
//! plus one registry row lowers into the IR's typed [`Val`] slots.
//!
//! ```text
//! w-[32px]            → Val::Px(32.0)        on the row's own targets
//! p-[1.5rem]          → Val::Rem(1.5)        (rem is first-class, §3.3)
//! w-[50%]             → Val::Pct(50.0)
//! opacity-[0.37]      → Val::Num(0.37)       (the v4 verbatim reading)
//! rotate-[45deg]      → Val::Num(45.0)       (the row's number form IS deg)
//! bg-[#0ea5e9]        → Val::Rgba([14, 165, 233, 255])
//! bg-[var(--color-x)] → the theme pass's token path (the tiered forms)
//! ```
//!
//! The grammar is deliberately NARROW — it lowers exactly what the IR's
//! value slots can carry, and refuses everything else TYPED (the
//! no-silent-failures rule, CONTRIBUTING §2):
//!
//! * **Units** — `px`, `rem`, `%`; bare numbers only where the row's own
//!   shape contract carries a `Number` arm (`z-[13]`, `opacity-[0.37]`);
//!   `deg` only on the rows whose number form IS degrees (the emitter's
//!   `NUMBER_FORMS` table: `rotate`/`skew-x`/`skew-y`). The `number:`
//!   hint is the caller's explicit type assertion and lowers the
//!   unitless value even where the row has no `Number` arm.
//! * **Colours** — through the audited [`aeolus_core::Color`] parser
//!   (hex spellings and the CSS functional forms it accepts), the same
//!   parser the v0.x oracle and the `color_parse` fuzzer exercise; an
//!   `/opacity` modifier bakes through the audited premultiplied
//!   identity, exactly like the token path's literals.
//! * **Type hints** — `length:`, `percentage:`, `number:`, `color:`
//!   constrain the parse; a MISMATCHED hint is a typed refusal, never a
//!   silent re-interpretation. The wider v4 hint table (`url`, `image`,
//!   `angle`, `time`, `ratio`, `position`, …) has no IR slot and refuses
//!   with the reason naming the covered set. The hint list IS the
//!   lexer's ([`aeolus_syntax::HINTS`]) — one table, two consumers.
//! * **`var()`/`(--var)` references** — the theme pass's business: the
//!   name resolves through the supplied `TokenTable` (the same colour
//!   token path `bg-primary` rides), and the theme-less default refuses
//!   typed, exactly like the colour tokens' own boundary.
//!
//! The sibling-row disambiguation is DATA-DRIVEN, not guessed: a row
//! accepts a value class only when its tw_merge conflict group says so
//! ([`COLOUR_GROUPS`], or the row's own numeric arms for the scalar
//! class). `text-[13px]` matches TWO rows (font-size + colour) — the
//! colour row never matched (a length is not a colour) and skips
//! SILENTLY, the same way `bg-red-500`'s image sibling never matched;
//! what remains is exactly v4's reading, with no diagnostic noise. When
//! NO row of the root takes the class, the pipeline renders one typed
//! boundary (the `run` walks' deferred-reason dedupe + catch-all —
//! never a silent drop, never a per-sibling diagnostic pile).
//!
//! What this slice deliberately is NOT (each documented where it
//! belongs): the multi-token composite grammar (`calc()`, underscore
//! joins, url/image values, shadow/filter lists — the codegen host's
//! surface), the arbitrary-PROPERTY form (`[mask-type:luminance]` — the
//! row already names the property), keyword values on the row-relative
//! `Val::Kw` slots (the codegen crate owns the names, plan §3.3), and
//! the bare-number forms on rows without a `Number` arm (the v0.x
//! oracle refuses those too).

use aeolus_core::Color;
use aeolus_data::{UtilityRow, ValueKind};
use aeolus_ir::Prop;
use aeolus_syntax::Arb;

use crate::lower::Reject;

/// Why one arbitrary group did not lower onto ONE row. The split is the
/// sibling-row model's: a class mismatch is not a refusal (another row
/// of the root takes the class — the silent skip), a grammar boundary is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ArbRejection {
    /// The value's class does not fit THIS row's conflict group — the
    /// sibling-row skip. Silent by design: `text-[13px]`'s colour
    /// sibling never matched, exactly like `bg-red-500`'s image row.
    NotThisRow,
    /// The value is outside the unit grammar entirely — the typed
    /// boundary the diagnostic layer renders (`Reject::Value`).
    Boundary {
        /// The stable reason text (one of this module's `REASON_*`).
        reason: &'static str,
    },
}

/// One classified arbitrary value — the grammar's output. The colour
/// variant carries the FINAL rgba (the audited bake already applied
/// when an alpha modifier rode).
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ArbVal {
    Px(f32),
    Rem(f32),
    Pct(f32),
    Num(f32),
    /// The baked RGBA (premultiplied when an alpha modifier rode).
    Colour {
        rgba: [u8; 4],
    },
    /// A `var()`/`(--var)` reference: the theme-pass token name (the
    /// `--color-`/`--` prefix stripped), resolved through the supplied
    /// `TokenTable` by the caller.
    Token {
        name: String,
    },
}

/// The normalized arbitrary text the grammar classifies — the lexed
/// group's fields, or the bracket-kept suffix's.
pub(crate) struct ArbText<'a> {
    /// The value text without brackets (hint stripped).
    pub(crate) value: &'a str,
    /// The type hint, when declared.
    pub(crate) hint: Option<&'a str>,
    /// The arbitrary-PROPERTY spelling (`[mask-type:luminance]`).
    pub(crate) is_property: bool,
    /// The `(...)` paren shorthand (a var reference by v4.1 grammar).
    pub(crate) paren: bool,
}

/// The arbitrary group's value text: the candidate's lexed group when
/// the lexer split it (`w-[32px]` → body `w` + `Arb`), else the
/// bracket-kept suffix some call paths carry (`w-[-32px]` through the
/// joined read). `None` when neither rode — the drift arm. The
/// bracket-kept fallback reuses the lexer's own hint table; the
/// property-form flag cannot arise there (the lexer flags it only when
/// it split the group).
pub(crate) fn text_of<'a>(arb: Option<Arb<'a>>, suffix: Option<&'a str>) -> Option<ArbText<'a>> {
    match arb {
        Some(arb) => Some(ArbText {
            value: arb.value,
            hint: arb.hint,
            is_property: arb.is_property,
            paren: arb.form == aeolus_syntax::Form::Paren,
        }),
        None => suffix.and_then(|s| {
            let inner = s.strip_prefix('[')?.strip_suffix(']')?;
            let (hint, value) = match inner.split_once(':') {
                Some((h, v)) if aeolus_syntax::HINTS.contains(&h) => (Some(h), v),
                _ => (None, inner),
            };
            Some(ArbText {
                value,
                hint,
                is_property: false,
                paren: false,
            })
        }),
    }
}

/// Why one value did not fit the unit grammar — the typed boundary set
/// (the stable strings the diagnostics render and the tests pin).
pub(crate) const REASON_ARBITRARY_UNIT: &str = "the arbitrary value does not fit the unit grammar: the row's \
     slots take px/rem/% lengths, deg on the rotate/skew rows, colours, or a bare number where the row \
     carries a Number arm; keyword values have no slot to lower into (the row-relative keyword table is \
     the codegen crate's own surface)";

pub(crate) const REASON_ARBITRARY_HINT: &str = "the type hint does not match the value (length: takes px/rem, \
     percentage: a %, number: a unitless decimal, color: a colour), or names a type the IR has no slot \
     for (url/image/angle/time/ratio/position and friends lower in the codegen pass)";

pub(crate) const REASON_ARBITRARY_PROPERTY: &str = "the bracket names a PROPERTY ([mask-type:luminance] form), not a \
     value: the registry row already names the property, and the arbitrary-property form is the codegen \
     pass's surface";

pub(crate) const REASON_ARBITRARY_COMPOSITE: &str = "multi-token arbitrary values (underscore joins, calc(), \
     url()/image() and other non-colour functions, shadow/filter lists) are the codegen host's composite \
     grammar: the unit grammar lowers one typed value per row";

pub(crate) const REASON_ARBITRARY_TABLE: &str = "var() references resolve through the theme pass: \
     compile_theme() carries the TokenTable the variable name resolves from (the colour tokens' own \
     boundary — the theme-less default refuses, never a silent passthrough)";

/// Is this reason one of the arbitrary grammar's typed boundaries? The
/// run walks use it to DEFER the per-row reports and render them
/// deduped (one class, one boundary — the sibling rows' shared miss).
pub(crate) fn is_boundary_reason(reason: &str) -> bool {
    matches!(
        reason,
        REASON_ARBITRARY_UNIT
            | REASON_ARBITRARY_HINT
            | REASON_ARBITRARY_PROPERTY
            | REASON_ARBITRARY_COMPOSITE
            | REASON_ARBITRARY_TABLE
    )
}

/// The conflict groups whose longhands take a COLOUR value. The
/// registry's own tw_merge group names are the oracle — the drift test
/// below fails when a row grows a colour arm without joining this table
/// (the `NUMBER_FORMS` audit pattern).
pub(crate) const COLOUR_GROUPS: &[&str] = &[
    "bg-color",
    "text-color",
    "border-color",
    "outline-color",
    "ring-color",
    "ring-offset-color",
    "shadow-color",
    "from-color",
    "via-color",
    "to-color",
    "fill",
    "stroke",
];

/// The one manually-listed scalar group: the `text` font-size row's
/// arbitrary length lowers onto `FontSize` ONLY (the audited v0.x
/// oracle's reading — the composite (size, line-height) pair rides the
/// NAMED sizes; `text-[13px]` never touches line-height). The row has
/// no numeric arm (its shape is the composite table), so the mechanical
/// scalar rule misses it — the entry here is the audit surface.
pub(crate) const EXTRA_SCALAR_GROUPS: &[&str] = &["font-size"];

/// The font-size row's arbitrary fan: `FontSize` alone (see
/// [`EXTRA_SCALAR_GROUPS`] for the why).
pub(crate) const FONT_SIZE_ONLY: &[Prop] = &[Prop::FontSize];

/// The arbitrary fan-out for one row: the override for the font-size
/// row, the row's own targets otherwise.
pub(crate) fn targets_of(row: &UtilityRow) -> &'static [Prop] {
    if EXTRA_SCALAR_GROUPS.contains(&row.group) && row.group == "font-size" {
        FONT_SIZE_ONLY
    } else {
        row.targets
    }
}

/// Does this row take a SCALAR arbitrary value (px/rem/%/number)? The
/// row's own shape contract decides: any numeric arm (spacing scale,
/// fraction, number, percent) means the longhands are scalar slots.
/// `group` disambiguates the sibling rows that share a root.
pub(crate) fn scalar_eligible(row: &UtilityRow) -> bool {
    let mut arms = Vec::new();
    flatten(row.kind, &mut arms);
    arms.iter().any(|arm| {
        matches!(
            arm,
            ValueKind::Spacing | ValueKind::Fraction | ValueKind::Number | ValueKind::Percent
        )
    }) || EXTRA_SCALAR_GROUPS.contains(&row.group)
}

/// Does this row take a COLOUR arbitrary value? The conflict group's
/// own name (see [`COLOUR_GROUPS`]).
pub(crate) fn colour_eligible(row: &UtilityRow) -> bool {
    COLOUR_GROUPS.contains(&row.group)
}

/// The AnyOf arms in declaration order — the value pass's own walk,
/// mirrored for the shape contract's numeric-arm test (lower.rs's
/// `flatten` is the original; this module's audit needs it too).
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

/// Does the row's number form render degrees? The emitter's
/// `NUMBER_FORMS` table is the oracle (`rotate`, `skew-x`, `skew-y`).
pub(crate) fn deg_row(root: &str) -> bool {
    crate::lower::NUMBER_FORMS
        .iter()
        .any(|&(r, form, _)| r == root && form == crate::lower::NumForm::Deg)
}

/// Classify ONE arbitrary group against ONE row: the value grammar plus
/// the row's class gate, the alpha modifier baked where it rides. Pure
/// and total — every input answers.
pub(crate) fn classify(
    text: &ArbText<'_>,
    alpha: Option<f32>,
    row: &UtilityRow,
) -> Result<ArbVal, ArbRejection> {
    // The arbitrary-PROPERTY spelling: the row already names the
    // property; the form is the codegen host's. Typed, never guessed.
    if text.is_property {
        return Err(ArbRejection::Boundary {
            reason: REASON_ARBITRARY_PROPERTY,
        });
    }

    // The hint half: the explicit type contract constrains the parse.
    if let Some(hint) = text.hint {
        return classify_hinted(hint, text.value, alpha, row);
    }

    // The free grammar: var references, colours, then the units.
    classify_free(text.value, text.paren, alpha, row)
}

/// The hinted grammar: one class only, mismatches refuse typed.
fn classify_hinted(
    hint: &str,
    value: &str,
    alpha: Option<f32>,
    row: &UtilityRow,
) -> Result<ArbVal, ArbRejection> {
    let value = value.trim();
    let val = match hint {
        "length" => {
            if value == "0" {
                // CSS sanctions the unitless zero for lengths — under
                // the explicit hint it lowers as a pixel length.
                ArbVal::Px(0.0)
            } else if let Some(n) = value.strip_suffix("px") {
                ArbVal::Px(parse_num(n)?)
            } else if let Some(n) = value.strip_suffix("rem") {
                ArbVal::Rem(parse_num(n)?)
            } else {
                return Err(hint_mismatch());
            }
        }
        "percentage" => match value.strip_suffix('%') {
            Some(n) => ArbVal::Pct(parse_num(n)?),
            None => return Err(hint_mismatch()),
        },
        // The caller's explicit type assertion: the unitless value
        // lowers even where the row has no `Number` arm (the hint IS
        // the contract; v4 emits the value verbatim).
        "number" => ArbVal::Num(parse_num(value)?),
        "color" => match colour_of(value, alpha) {
            Some(rgba) => ArbVal::Colour { rgba },
            None => return Err(hint_mismatch()),
        },
        // The wider v4 hint table: no IR slot (url/image/angle/time/
        // ratio/position/integer/string/family names, …) — typed, named.
        _ => {
            return Err(ArbRejection::Boundary {
                reason: REASON_ARBITRARY_HINT,
            });
        }
    };
    gate(val, row)
}

/// The free grammar (no hint): var references, colours, units, the
/// row-gated bare numbers.
fn classify_free(
    value: &str,
    paren: bool,
    alpha: Option<f32>,
    row: &UtilityRow,
) -> Result<ArbVal, ArbRejection> {
    let value = value.trim();

    // Multi-token composites: the underscore IS a space in v4's grammar,
    // so any underscore marks a value the unit grammar does not lower.
    if value.contains('_') {
        return Err(ArbRejection::Boundary {
            reason: REASON_ARBITRARY_COMPOSITE,
        });
    }

    // var() references and the v4.1 `(...)` shorthand: the theme pass's
    // token path, gated to the colour rows (the table is the colour
    // half — a length var has no resolver and refuses typed downstream).
    if paren || value.starts_with("var(") {
        let name = value
            .trim_start_matches("var(")
            .trim_end_matches(')')
            .trim()
            .trim_start_matches("--")
            .strip_prefix("color-")
            .unwrap_or(value);
        let name = if name.is_empty() { value } else { name };
        return gate(
            ArbVal::Token {
                name: name.to_string(),
            },
            row,
        );
    }

    // Colours: the audited parser answers (hex spellings and the CSS
    // functional forms it accepts); a parse miss is a boundary — the
    // caller meant a colour (or a function), and guessing a unit would
    // lie. Non-function, non-hex text never enters this branch, so
    // `w-[3px]` cannot misread as a colour attempt.
    if value.starts_with('#') || value.contains('(') {
        return match colour_of(value, alpha) {
            Some(rgba) => gate(ArbVal::Colour { rgba }, row),
            None => Err(ArbRejection::Boundary {
                reason: REASON_ARBITRARY_UNIT,
            }),
        };
    }

    // Units: px / rem / %, deg on the Deg rows, bare numbers where the
    // row's shape contract carries a Number arm.
    let val = if let Some(n) = value.strip_suffix("px") {
        ArbVal::Px(parse_num(n)?)
    } else if let Some(n) = value.strip_suffix("rem") {
        ArbVal::Rem(parse_num(n)?)
    } else if let Some(n) = value.strip_suffix('%') {
        ArbVal::Pct(parse_num(n)?)
    } else if let Some(n) = value.strip_suffix("deg") {
        if deg_row(row.root) {
            ArbVal::Num(parse_num(n)?)
        } else {
            return Err(ArbRejection::Boundary {
                reason: REASON_ARBITRARY_UNIT,
            });
        }
    } else if let Ok(n) = value.parse::<f32>() {
        // Bare number: the row's own contract decides (`z-[13]` yes,
        // `w-[13]` no — the `Number` arm is the legality oracle), and
        // the finite guard keeps the f32 grammar's spellings out.
        if !n.is_finite() {
            return Err(ArbRejection::Boundary {
                reason: REASON_ARBITRARY_UNIT,
            });
        }
        let mut arms = Vec::new();
        flatten(row.kind, &mut arms);
        if arms.iter().any(|arm| matches!(arm, ValueKind::Number)) {
            ArbVal::Num(n)
        } else {
            return Err(ArbRejection::Boundary {
                reason: REASON_ARBITRARY_UNIT,
            });
        }
    } else {
        return Err(ArbRejection::Boundary {
            reason: REASON_ARBITRARY_UNIT,
        });
    };
    gate(val, row)
}

/// The row's class gate: a value the row's longhands cannot carry is the
/// SILENT sibling-row skip, never a refusal (the pipeline's catch-all
/// renders the boundary when NO row of the root took the class).
fn gate(val: ArbVal, row: &UtilityRow) -> Result<ArbVal, ArbRejection> {
    let fits = match &val {
        ArbVal::Colour { .. } | ArbVal::Token { .. } => colour_eligible(row),
        ArbVal::Px(_) | ArbVal::Rem(_) | ArbVal::Pct(_) | ArbVal::Num(_) => scalar_eligible(row),
    };
    if fits {
        Ok(val)
    } else {
        Err(ArbRejection::NotThisRow)
    }
}

fn hint_mismatch() -> ArbRejection {
    ArbRejection::Boundary {
        reason: REASON_ARBITRARY_HINT,
    }
}

/// The finite f32 parse — the pipeline's own guard (`nan`/`inf`/
/// overflow are legal parses and illegal values).
fn parse_num(s: &str) -> Result<f32, ArbRejection> {
    let n = s
        .trim()
        .parse::<f32>()
        .map_err(|_| ArbRejection::Boundary {
            reason: REASON_ARBITRARY_UNIT,
        })?;
    if n.is_finite() {
        Ok(n)
    } else {
        Err(ArbRejection::Boundary {
            reason: REASON_ARBITRARY_UNIT,
        })
    }
}

/// The colour parse through the audited [`Color`], with the alpha
/// modifier baked on the parsed object (ONE srgb round-trip, byte-
/// identical to the token path's literal bake): `None` = "not a
/// colour at all" (the caller falls through or refuses). The sRGB bake
/// runs the CSS Color 4 gamut mapping (`Color::gamut_map`, plan §9's
/// mitigation) after the alpha mix — mix first, then map, the CSS
/// reading — so a literal and its token spelling agree and an
/// out-of-gamut `oklch()` lands ON the gamut boundary, hue-true.
fn colour_of(value: &str, alpha: Option<f32>) -> Option<[u8; 4]> {
    match Color::parse(value) {
        Ok(color) => Some(match alpha {
            None => color.gamut_map().to_rgba8(),
            Some(alpha) => color
                .mix_with_transparent(alpha * 100.0)
                .gamut_map()
                .to_rgba8(),
        }),
        // The audited parser's own typed miss: junk text is not a
        // colour — the unit grammar's boundary names it.
        Err(_) => None,
    }
}

/// The value pass's typed rejection, mapped from this module's split:
/// the silent sibling-row skip is `Ok` (an EMPTY fan — no declarations
/// for THIS row), the boundaries are the caller's `Reject::Value`.
pub(crate) fn map_rejection(rejection: ArbRejection) -> Result<Option<&'static str>, Reject> {
    match rejection {
        ArbRejection::NotThisRow => Ok(None),
        ArbRejection::Boundary { reason } => Err(Reject::Value { reason }),
    }
}

// ---------------------------------------------------------------------------
// The rendering-table audit (the drift catch, test-pinned in place)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use aeolus_data::rows::ROWS;
    use aeolus_data::tables::COLOR_KEYWORD_TABLE;

    fn has_arm(kind: ValueKind, pred: &impl Fn(&ValueKind) -> bool) -> bool {
        let mut arms = Vec::new();
        flatten(kind, &mut arms);
        arms.iter().any(pred)
    }

    /// Every registry row that lowers COLOUR keyword values must have
    /// its group in [`COLOUR_GROUPS`] — a new colour row without the
    /// gate entry is a test failure here, never a silently-skipped
    /// arbitrary colour (the `NUMBER_FORMS` audit pattern).
    #[test]
    fn every_colour_row_is_in_the_colour_gate() {
        // The colour table's own slice, extracted from the row-table
        // const (the rows reference the same static data).
        let ValueKind::Keywords(colour_table) = COLOR_KEYWORD_TABLE else {
            unreachable!("the colour table const is a Keywords kind")
        };
        for row in ROWS {
            let colour_row = has_arm(row.kind, &|arm| match arm {
                ValueKind::Keywords(table) => std::ptr::eq(*table, colour_table),
                _ => false,
            });
            if colour_row {
                assert!(
                    COLOUR_GROUPS.contains(&row.group),
                    "root {:?} lowers colour values but group {:?} is not in COLOUR_GROUPS",
                    row.root,
                    row.group
                );
            }
        }
    }

    /// Every row with a numeric arm takes scalar arbitrary values by
    /// construction — pinned so a refactor of [`scalar_eligible`] cannot
    /// silently drop the spacing/fraction/number families.
    #[test]
    fn every_numeric_row_is_scalar_eligible() {
        for row in ROWS {
            let numeric = has_arm(row.kind, &|arm| {
                matches!(
                    arm,
                    ValueKind::Spacing
                        | ValueKind::Fraction
                        | ValueKind::Number
                        | ValueKind::Percent
                )
            });
            if numeric {
                assert!(
                    scalar_eligible(row),
                    "root {:?} lowers numeric values but is not scalar-eligible",
                    row.root
                );
            }
        }
    }

    /// The keyword-only rows (no numeric arm, no colour table) take NO
    /// arbitrary value class — their `Val::Kw` slots are row-relative
    /// (the codegen crate owns the names). Pinned per family.
    #[test]
    fn keyword_only_rows_take_no_arbitrary_class() {
        for root in [
            "display",
            "cursor",
            "whitespace",
            "overflow",
            "items",
            "self",
        ] {
            let row = ROWS
                .iter()
                .find(|row| row.root == root)
                .unwrap_or_else(|| panic!("the {root} row exists"));
            assert!(
                !scalar_eligible(row) && !colour_eligible(row),
                "the {root} row must take no arbitrary class"
            );
        }
    }

    /// The class gate: `text`'s two rows split the classes cleanly, and
    /// the font-size row's fan is the FontSize-only override.
    #[test]
    fn the_text_rows_split_the_classes() {
        let font_row = ROWS
            .iter()
            .find(|row| row.root == "text" && row.group == "font-size")
            .expect("the font-size row");
        let colour_row = ROWS
            .iter()
            .find(|row| row.root == "text" && row.group == "text-color")
            .expect("the colour row");
        assert!(scalar_eligible(font_row), "font-size takes lengths");
        assert!(!colour_eligible(font_row), "font-size takes no colour");
        assert!(colour_eligible(colour_row), "text-color takes colours");
        assert!(!scalar_eligible(colour_row), "text-color takes no length");
        assert_eq!(targets_of(font_row), FONT_SIZE_ONLY, "the override fan");
        assert_eq!(targets_of(colour_row), colour_row.targets, "the row fan");
    }
}
