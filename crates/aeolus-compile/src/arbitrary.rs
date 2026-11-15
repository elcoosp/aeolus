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
//! * **Composites** — the [`CompositeKind`] grammar: the sized functions
//!   (`calc()`/`min()`/`max()`/`clamp()`) on the single-target scalar
//!   rows, the underscore-joined track lists and track functions on the
//!   grid-template/auto rows, the image functions (`url()`/`image()`/the
//!   gradients) on the bg-image row and the bare `n/d` ratios on the
//!   aspect row lower through the IR's [`Val::Verbatim`] slot — the
//!   decoded text (the underscores ARE spaces, `\_` a literal
//!   underscore), emitted as typed (v4's own arbitrary contract: the
//!   author owns the CSS validity). The kind gate is the design's
//!   centre: a composite refuses TYPED on every row whose slots cannot
//!   carry its shape — the shorthand fans (one value per longhand), the
//!   filter/backdrop handles, the shadow lists — and the negative flag
//!   and the opacity modifier compose no composite. A truncated
//!   function is the malformed boundary, never a broken-CSS passthrough.
//! * **Type hints** — `length:`, `percentage:`, `number:`, `color:`
//!   constrain the parse; a MISMATCHED hint is a typed refusal, never a
//!   silent re-interpretation. The composite-aware hints lower through
//!   the same slot (`length:` takes the sized functions, `url:` wraps
//!   the bare path, `image:` and `ratio:` ride decoded); the hint list
//!   IS the lexer's ([`aeolus_syntax::HINTS`]) — one table, two
//!   consumers.
//! * **`var()`/`(--var)` references** — the theme pass's business: the
//!   name resolves through the supplied `TokenTable` (the same colour
//!   token path `bg-primary` rides), and the theme-less default refuses
//!   typed, exactly like the colour tokens' own boundary. The var branch
//!   leads the free walk — an underscore inside a custom-property name
//!   is the name's own byte, never a space to decode.
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
//! belongs): the arbitrary-PROPERTY form (`[mask-type:luminance]` — the
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
    /// The composite grammar's verbatim value: the DECODED text (the
    /// underscores ARE spaces, the `\_` escapes a literal underscore —
    /// v4's own grammar), interned into the leak pool by the caller.
    /// The kind gate has already answered eligibility when this variant
    /// is built.
    Verbatim {
        text: String,
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

pub(crate) const REASON_ARBITRARY_HINT: &str = "the type hint does not match the value (length: takes px/rem \
     and the sized functions, percentage: a %, number: a unitless decimal, color: a colour, url:/image: \
     an image value on the image rows, ratio: an n/d form on the aspect row, angle: a deg spelling on \
     the degree rows, time: an ms/s spelling on the millisecond rows, position: a position form on the \
     object-position row, family-name:/generic-name: a family on the font-family row, \
     absolute-size:/relative-size: a size keyword on the font-size row, string: a quoted string on the \
     content root)";

pub(crate) const REASON_ARBITRARY_PROPERTY: &str = "the bracket names a PROPERTY ([mask-type:luminance] form), not a \
     value: the registry row already names the property, and the arbitrary-property form is the codegen \
     pass's surface";

pub(crate) const REASON_ARBITRARY_COMPOSITE: &str = "the row's slots take no composite value: the sized \
     functions (calc()/min()/max()/clamp()) lower on the single-target scalar rows, the track lists \
     (underscore joins, repeat()/minmax()/fit-content()) on the grid-template/auto rows, the image \
     functions (url()/image()/the gradients) on the bg-image row and the bare n/d ratios on the aspect \
     row; the shorthand fans, the filter/backdrop handles and the shadow lists take one value per slot";

pub(crate) const REASON_ARBITRARY_MALFORMED: &str = "the composite function's parens do not balance \
     (calc()/min()/max()/clamp(), url() and the track functions must close) — a truncated value is a \
     boundary, never a broken-CSS passthrough";

pub(crate) const REASON_ARBITRARY_COMPOSITE_NEGATIVE: &str = "the negative flag has no composite form: the \
     scalar slots flip their number, the verbatim text carries its own signs (write the minus inside \
     the value)";

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
            | REASON_ARBITRARY_MALFORMED
            | REASON_ARBITRARY_COMPOSITE_NEGATIVE
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

/// The manually-listed scalar groups: rows whose shape contract carries
/// no numeric arm yet take a scalar arbitrary value. The `text`
/// font-size row's arbitrary length lowers onto `FontSize` ONLY (the
/// audited v0.x oracle's reading — the composite (size, line-height)
/// pair rides the NAMED sizes; `text-[13px]` never touches
/// line-height). The filter family's keyword-scale rows (`blur`,
/// `drop-shadow`, and the backdrop mirror `backdrop-blur`) take the
/// single-value lengths (`blur-[2px]`), the v4 `${fn}(${value})`
/// handle's own passthrough — no numeric arm (the scales are keyword
/// tables), so the mechanical scalar rule misses them — the entries
/// here are the audit surface.
pub(crate) const EXTRA_SCALAR_GROUPS: &[&str] =
    &["font-size", "blur", "drop-shadow", "backdrop-blur"];

/// The font-size row's arbitrary fan: `FontSize` alone (see
/// [`EXTRA_SCALAR_GROUPS`] for the why).
pub(crate) const FONT_SIZE_ONLY: &[Prop] = &[Prop::FontSize];

/// The content root's arbitrary fan: the `string:` hint's value is the
/// CSS `content` property (the pseudo-element paint), NOT the root's
/// keyword half — `content-center` stays `AlignContent` (the group the
/// tw_merge oracle names), the string form routes to the new longhand.
pub(crate) const CONTENT_ONLY: &[Prop] = &[Prop::Content];

/// The hint-eligible conflict groups, per hint — the registry's own
/// tw_merge group names are the oracle (the [`COLOUR_GROUPS`] audit
/// pattern). A hint lowers onto the rows whose group carries the
/// shape; everywhere else it is the silent sibling-row skip.
///
/// `position:` — the object-position row (the one position-shape
/// longhand; background-position has no registry row, its classes
/// refuse through the catch-all).
pub(crate) const POSITION_GROUPS: &[&str] = &["object-position"];

/// `family-name:`/`generic-name:` — the font-family row (the weight
/// sibling shares the root and refuses silently: a family is not a
/// weight).
pub(crate) const FAMILY_GROUPS: &[&str] = &["font-family"];

/// `absolute-size:`/`relative-size:` — the font-size row's arbitrary
/// fan (the v4.1 size keywords are font-size values).
pub(crate) const SIZE_KEYWORD_GROUPS: &[&str] = &["font-size"];

/// `string:` — the content root (the row's own group name stays the
/// oracle's `align-content`; the STRING value routes to the CSS
/// `content` property through [`CONTENT_ONLY`], v4's own reading).
pub(crate) const STRING_ROOTS: &[&str] = &["content"];

/// Does the row's number form render milliseconds? The emitter's
/// `NUMBER_FORMS` table is the oracle (`duration`, `delay`) — the
/// `time:` hint's own gate (the [`deg_row`] pattern).
pub(crate) fn time_row(root: &str) -> bool {
    crate::lower::NUMBER_FORMS
        .iter()
        .any(|&(r, form, _)| r == root && form == crate::lower::NumForm::Ms)
}

/// The arbitrary fan-out for one row: the override for the font-size
/// row, the row's own targets otherwise.
pub(crate) fn targets_of(row: &UtilityRow) -> &'static [Prop] {
    if EXTRA_SCALAR_GROUPS.contains(&row.group) && row.group == "font-size" {
        FONT_SIZE_ONLY
    } else if STRING_ROOTS.contains(&row.root) {
        CONTENT_ONLY
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
            } else if leading_composite_fn(value) == Some(CompositeKind::Sized) {
                // The hint IS the type contract: the sized functions
                // lower through the verbatim slot where the unit
                // grammar's spellings stop — the composite gate decides
                // the row's eligibility, never the scalar gate.
                return composite_gate(
                    CompositeKind::Sized,
                    ArbVal::Verbatim {
                        text: decode_underscores(value),
                    },
                    row,
                );
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
        // The integer hint's stricter shape: whole-number syntax only.
        "integer" => match value.parse::<i64>() {
            Ok(n) => ArbVal::Num(n as f32),
            Err(_) => return Err(hint_mismatch()),
        },
        "color" => match colour_of(value, alpha) {
            Some(rgba) => ArbVal::Colour { rgba },
            None => return Err(hint_mismatch()),
        },
        // The hinted image forms: `url:` wraps the bare path in the
        // function (v4's own reading), `image:`'s value is already a
        // function and rides decoded. Both gate to the image rows.
        "url" => {
            return composite_gate(
                CompositeKind::Image,
                ArbVal::Verbatim {
                    text: format!("url({})", decode_underscores(value)),
                },
                row,
            );
        }
        "image" => {
            return composite_gate(
                CompositeKind::Image,
                ArbVal::Verbatim {
                    text: decode_underscores(value),
                },
                row,
            );
        }
        // The ratio hint: the aspect rows' own shape (`16/9`, or the
        // decimal form `1.5`).
        "ratio" => {
            return composite_gate(
                CompositeKind::Ratio,
                ArbVal::Verbatim {
                    text: decode_underscores(value),
                },
                row,
            );
        }
        // The angle hint: a `deg` spelling on the degree rows — the
        // free grammar's own gate (the `deg_row` oracle; the Num slot
        // IS the degree form). rad/grad/turn have no IR slot: the
        // typed hint mismatch, never a conversion the bytes cannot
        // name.
        "angle" => {
            let Some(n) = value.strip_suffix("deg") else {
                return Err(hint_mismatch());
            };
            if !deg_row(row.root) {
                return Err(ArbRejection::NotThisRow);
            }
            ArbVal::Num(parse_num(n)?)
        }
        // The time hint: an ms/s spelling on the millisecond rows —
        // the slots ARE milliseconds (the motion layer's number form),
        // so `s` bakes ×1000 (one exact multiply).
        "time" => {
            let ms = if let Some(n) = value.strip_suffix("ms") {
                parse_num(n)?
            } else if let Some(n) = value.strip_suffix('s') {
                parse_num(n)? * 1000.0
            } else {
                return Err(hint_mismatch());
            };
            if !time_row(row.root) {
                return Err(ArbRejection::NotThisRow);
            }
            ArbVal::Num(ms)
        }
        // The verbatim hints: the decoded text IS the value (the
        // composite slot's own contract — the underscores are spaces,
        // the author owns the CSS validity), gated to the rows whose
        // slots carry the shape (the group tables above). A miss is
        // the silent sibling-row skip — another row of the root may
        // take the class; the run's catch-all renders one boundary
        // when none did.
        "position" | "family-name" | "generic-name" | "absolute-size" | "relative-size"
        | "string" => {
            let eligible = match hint {
                "position" => POSITION_GROUPS.contains(&row.group),
                "family-name" | "generic-name" => FAMILY_GROUPS.contains(&row.group),
                "absolute-size" | "relative-size" => SIZE_KEYWORD_GROUPS.contains(&row.group),
                _ => STRING_ROOTS.contains(&row.root),
            };
            if !eligible {
                return Err(ArbRejection::NotThisRow);
            }
            return Ok(ArbVal::Verbatim {
                text: decode_underscores(value),
            });
        }
        // The drift arm: the lexer's HINTS table is the one source of
        // truth — a hint added there but not here lands on this typed
        // refusal until the value pass learns it.
        _ => {
            return Err(ArbRejection::Boundary {
                reason: REASON_ARBITRARY_HINT,
            });
        }
    };
    gate(val, row)
}

/// The free grammar (no hint): the composite candidates, var references,
/// colours, units, the row-gated bare numbers.
fn classify_free(
    value: &str,
    paren: bool,
    alpha: Option<f32>,
    row: &UtilityRow,
) -> Result<ArbVal, ArbRejection> {
    let value = value.trim();

    // var() references and the v4.1 `(...)` shorthand: the theme pass's
    // token path, gated to the colour rows (the table is the colour
    // half — a length var has no resolver and refuses typed downstream).
    // This branch leads: an underscore inside a custom-property name is
    // the name's own byte, never a space to decode.
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

    // The quoted-string form: v4's own content spelling
    // (`content-["hello_world"]`) — the CSS `content` property takes a
    // string, the quotes riding the value as typed (the underscores
    // decode inside the quotes). The branch leads the composite walk:
    // an underscore join inside a string is the string's own space,
    // never a track list. Other roots keep the unit grammar's boundary
    // (the family names ride the `family-name:` hint, the explicit
    // contract).
    if value.len() >= 2
        && ((value.starts_with('"') && value.ends_with('"'))
            || (value.starts_with('\'') && value.ends_with('\'')))
    {
        if STRING_ROOTS.contains(&row.root) {
            return Ok(ArbVal::Verbatim {
                text: decode_underscores(value),
            });
        }
        // Not a string row: the walk continues — the quoted text hits
        // the same typed boundaries it always did (never a guessed
        // slot).
    }

    // The composite grammar: the sized functions, the track lists, the
    // image functions and the bare ratios — the verbatim slot's
    // kind-gated forms. `None` = not a composite candidate; the unit
    // grammar's own paths continue below.
    if let Some(outcome) = classify_composite(value, row) {
        return outcome;
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
    } else if let Some(n) = value.strip_suffix("ms") {
        // ms on the millisecond rows — the `deg_row` pattern: the
        // row's number form IS the unit's oracle (the motion layer's
        // slots are milliseconds).
        if time_row(row.root) {
            ArbVal::Num(parse_num(n)?)
        } else {
            return Err(ArbRejection::Boundary {
                reason: REASON_ARBITRARY_UNIT,
            });
        }
    } else if let Some(n) = value.strip_suffix('s') {
        // s on the millisecond rows: ×1000 (one exact multiply — the
        // slots are ms, the `time:` hint's own bake).
        if time_row(row.root) {
            ArbVal::Num(parse_num(n)? * 1000.0)
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

// ---------------------------------------------------------------------------
// The composite grammar: the verbatim slot's kind-gated forms
// ---------------------------------------------------------------------------

/// The composite value kinds — which row families carry which shape. The
/// kind IS the type gate: a composite lowers onto the rows whose slots
/// can carry it and refuses TYPED everywhere else (never an
/// invalid-CSS passthrough on a slot that cannot hold the value).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CompositeKind {
    /// The sized functions (`calc()`/`min()`/`max()`/`clamp()`) — the
    /// scalar slots' own computation forms.
    Sized,
    /// The track lists: the underscore-joined multi-token values and the
    /// track functions (`repeat()`/`minmax()`/`fit-content()`) — the
    /// grid-template/auto rows' shape.
    Tracks,
    /// The image functions (`url()`/`image()`/`image-set()`/the
    /// gradients) — the bg-image row's shape.
    Image,
    /// The bare `n/d` slash ratios — the aspect row's shape.
    Ratio,
}

/// The sized functions, in their CSS spellings.
const SIZED_FUNCTIONS: &[&str] = &["calc", "min", "max", "clamp"];

/// The track functions, in their CSS spellings.
const TRACK_FUNCTIONS: &[&str] = &["repeat", "minmax", "fit-content"];

/// The image functions, in their CSS spellings (the gradients included —
/// the bg-image row carries them once the unit/colour paths step aside).
const IMAGE_FUNCTIONS: &[&str] = &[
    "url",
    "image",
    "image-set",
    "linear-gradient",
    "repeating-linear-gradient",
    "radial-gradient",
    "repeating-radial-gradient",
    "conic-gradient",
    "repeating-conic-gradient",
];

/// The tw_merge groups whose rows carry TRACK-shaped values (the grid
/// template/auto families). The audit-surface pattern (`COLOUR_GROUPS`,
/// `EXTRA_SCALAR_GROUPS`): the drift test fails when a track row grows
/// without joining this table.
const TRACK_GROUPS: &[&str] = &[
    "grid-template-columns",
    "grid-template-rows",
    "grid-auto-columns",
    "grid-auto-rows",
];

/// The tw_merge groups whose rows carry IMAGE-shaped values.
const IMAGE_GROUPS: &[&str] = &["bg-image"];

/// The tw_merge groups whose rows carry the bare ratio shape.
const RATIO_GROUPS: &[&str] = &["aspect"];

/// The roots whose arbitrary handles take ONE value — the filter/
/// backdrop families (the §3.7 row's own boundary: the composite stays
/// the codegen host's surface for these).
const COMPOSITE_REFUSING_ROOTS: &[&str] = &[
    "blur",
    "brightness",
    "contrast",
    "grayscale",
    "hue-rotate",
    "invert",
    "saturate",
    "sepia",
    "drop-shadow",
    "filter",
    "backdrop-blur",
    "backdrop-brightness",
    "backdrop-contrast",
    "backdrop-grayscale",
    "backdrop-hue-rotate",
    "backdrop-invert",
    "backdrop-opacity",
    "backdrop-saturate",
    "backdrop-sepia",
    "backdrop-filter",
];

/// The composite candidate's kind: the function forms lead (the
/// underscores decode INSIDE them — `calc(100%_-_2rem)` is the sized
/// function's own authoring form), the underscore join is a track list,
/// the bare `n/d` slash is a ratio.
fn composite_kind(value: &str) -> Option<CompositeKind> {
    if let Some(kind) = leading_composite_fn(value) {
        return Some(kind);
    }
    if value.contains('_') {
        return Some(CompositeKind::Tracks);
    }
    if !value.contains('(') && composite_ratio(value) {
        return Some(CompositeKind::Ratio);
    }
    None
}

/// The leading function's composite kind, when the value STARTS with one
/// of the grammar's own functions (`fn(`-spelled).
fn leading_composite_fn(value: &str) -> Option<CompositeKind> {
    let (name, _) = value.split_once('(')?;
    if SIZED_FUNCTIONS.contains(&name) {
        return Some(CompositeKind::Sized);
    }
    if TRACK_FUNCTIONS.contains(&name) {
        return Some(CompositeKind::Tracks);
    }
    if IMAGE_FUNCTIONS.contains(&name) {
        return Some(CompositeKind::Image);
    }
    None
}

/// The bare `n/d` slash shape: both sides parse as finite numbers (the
/// aspect row's `16/9`; a decimal half `1.5` is a legal CSS ratio too).
fn composite_ratio(value: &str) -> bool {
    let Some((n, d)) = value.split_once('/') else {
        return false;
    };
    let finite = |s: &str| s.trim().parse::<f32>().map(f32::is_finite).unwrap_or(false);
    finite(n) && finite(d)
}

/// The paren balance: a composite function's value must close — a
/// truncated value is a boundary, never a broken-CSS passthrough. Lists
/// without parens balance trivially.
fn balanced(value: &str) -> bool {
    let mut depth = 0i32;
    for c in value.chars() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth < 0 {
                    return false;
                }
            }
            _ => {}
        }
    }
    depth == 0
}

/// The v4 underscore grammar: `_` is a space, `\_` a literal underscore.
/// The custom-property names never reach here (the var branch leads the
/// free walk), so the decode cannot corrupt a `--name`.
pub(crate) fn decode_underscores(text: &str) -> String {
    if !text.contains('_') {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c == '\\' && chars.next() == Some('_') {
            out.push('_');
        } else if c == '_' {
            out.push(' ');
        } else {
            out.push(c);
        }
    }
    out
}

/// Does the row's kind carry this composite shape? The gate is the
/// design's centre: single-target rows only (the shorthand fans take ONE
/// value per longhand — `p-[calc(...)]` cannot fan a list over four
/// longhands honestly), the filter/backdrop handles out (one value per
/// handle), then the kind's own group table.
fn composite_eligible(kind: CompositeKind, row: &UtilityRow) -> bool {
    if row.targets.len() != 1 || COMPOSITE_REFUSING_ROOTS.contains(&row.root) {
        return false;
    }
    match kind {
        CompositeKind::Sized => scalar_eligible(row),
        CompositeKind::Tracks => TRACK_GROUPS.contains(&row.group),
        CompositeKind::Image => IMAGE_GROUPS.contains(&row.group),
        CompositeKind::Ratio => RATIO_GROUPS.contains(&row.group),
    }
}

/// The composite gate's typed miss — the boundary the diagnostics render
/// when the row's slots take no composite value.
fn composite_boundary() -> ArbRejection {
    ArbRejection::Boundary {
        reason: REASON_ARBITRARY_COMPOSITE,
    }
}

/// Gate a classified composite onto the row: a hit hands the value back,
/// a miss is the TYPED composite boundary (never a silent skip — the
/// composite grammar is new surface, its misses name the grammar).
fn composite_gate(
    kind: CompositeKind,
    val: ArbVal,
    row: &UtilityRow,
) -> Result<ArbVal, ArbRejection> {
    if composite_eligible(kind, row) {
        Ok(val)
    } else {
        Err(composite_boundary())
    }
}

/// The free walk's composite path: `None` = not a composite candidate
/// (the colour/unit paths continue), the classified answer otherwise.
fn classify_composite(value: &str, row: &UtilityRow) -> Option<Result<ArbVal, ArbRejection>> {
    let kind = composite_kind(value)?;
    // The function forms must balance — a truncated value is a boundary
    // of its own, named (the lists without parens balance trivially).
    if value.contains('(') && !balanced(value) {
        return Some(Err(ArbRejection::Boundary {
            reason: REASON_ARBITRARY_MALFORMED,
        }));
    }
    let text = decode_underscores(value);
    Some(composite_gate(kind, ArbVal::Verbatim { text }, row))
}

/// The row's class gate: a value the row's longhands cannot carry is the
/// SILENT sibling-row skip, never a refusal (the pipeline's catch-all
/// renders the boundary when NO row of the root took the class). The
/// composite values never ride this gate — they carry their own
/// ([`composite_gate`], whose misses name the grammar instead of staying
/// quiet); a [`ArbVal::Verbatim`] here is the drift arm.
fn gate(val: ArbVal, row: &UtilityRow) -> Result<ArbVal, ArbRejection> {
    let fits = match &val {
        ArbVal::Colour { .. } | ArbVal::Token { .. } => colour_eligible(row),
        ArbVal::Px(_) | ArbVal::Rem(_) | ArbVal::Pct(_) | ArbVal::Num(_) => scalar_eligible(row),
        ArbVal::Verbatim { .. } => false,
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
