//! The candidate model + the zero-copy lexing passes.
//!
//! Two passes, both allocation-light (one `Vec` per candidate for the
//! variant slices, borrowed strings everywhere else):
//!
//! 1. **Variant segmentation** — a bracket/paren-aware split on
//!    top-level `:`. Mirrors the oracle exactly: every top-level colon
//!    ends a variant segment, so `w-[calc(100%:2)]`'s colons stay
//!    inside the group and `group-hover/menu:` splits as one segment.
//! 2. **Utility split** — negation, the body, the `[...]`/`(...)`
//!    arbitrary group (with type hint + property detection) and the RAW
//!    `/modifier` (`"2"`, `"[0.37]"`, `"6"`, `"(--o)"`) — interpretation
//!    belongs to the utility, not the lexer (plan §3.1: fixes F1/F6/F7
//!    at the source).

use crate::error::{LexError, LexErrorKind, LexFailure};

/// Which arbitrary syntax a value used (`[...]` vs the v4.1 `(...)`
/// shorthand). Mirrors the oracle's `ArbForm`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Form {
    /// `[value]`.
    Bracket,
    /// `(value)` — the v4.1 `var()` shorthand.
    Paren,
}

/// The arbitrary group of a candidate: `[value]`, `int:value]` or
/// `(--var)`. Mirrors the oracle's `Arbitrary` (hint split, property
/// detection), with borrowed slices.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Arb<'a> {
    /// Type hint (`length`, `color`, `url`, ...), when declared.
    pub hint: Option<&'a str>,
    /// The value text without brackets (hint stripped).
    pub value: &'a str,
    /// Bracket `[...]` or shorthand `(...)`.
    pub form: Form,
    /// True for the arbitrary-property form (`[mask-type:luminance]`) —
    /// a bracket whose `:`-prefix is NOT a known v4 type hint.
    pub is_property: bool,
}

/// One lexed class token with source spans. Every `&str` borrows from
/// the input; `span` is the token's byte range inside the class STRING
/// passed to [`parse_class_string`] (0..len for [`parse_candidate`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Candidate<'a> {
    /// The raw token (`"dark:bg-primary/90"`).
    pub raw: &'a str,
    /// Byte range inside the class string.
    pub span: (usize, usize),
    /// The variant segments in SOURCE order (left-to-right stacking,
    /// v4 semantics) — raw slices like `"dark"`, `"group-hover/menu"`,
    /// `"[@media(print)]"`. Classification (typed variant kinds) is the
    /// registry pipeline's job; the lexer segments and spans.
    pub variants: Vec<&'a str>,
    /// The utility slice (variants, `-` sign, modifier and arbitrary
    /// brackets stripped): `"bg-primary"` for `dark:bg-primary/90`,
    /// `"w"` for `w-[32px]`.
    pub body: &'a str,
    /// The `[...]`/`(...)` arbitrary group, when present.
    pub arbitrary: Option<Arb<'a>>,
    /// The RAW `/modifier` suffix: `"2"` for `w-1/2`, `"[0.37]"` for
    /// `bg-primary/[0.37]`, `"6"` for `text-sm/6`, `"(--o)"` for
    /// `bg-red-500/(--o)`.
    pub modifier: Option<&'a str>,
    /// Leading `-` negation (`-mt-4`).
    pub negative: bool,
    /// v4 trailing `!` or legacy leading `!`.
    pub important: bool,
}

impl Candidate<'_> {
    /// The variant segments joined by `:` — the merge key prefix the
    /// `cn()` engine keys on (empty for bare utilities).
    pub fn variant_prefix(&self) -> String {
        let mut s = String::new();
        let mut first = true;
        for v in &self.variants {
            if !first {
                s.push(':');
            }
            first = false;
            s.push_str(v);
        }
        s
    }
}

/// Parse a single whitespace-free class token into a [`Candidate`].
///
/// Failure is a typed [`LexError`] with a byte span — the same
/// failure/Success split as the v0.x oracle, minus the oracle's inline
/// caps (documented divergence: `too many variants/elements` are
/// core-only rejections).
pub fn parse_candidate(raw: &str) -> Result<Candidate<'_>, LexError> {
    let rest_all = raw;

    // Importance: v4 suffix `!` or legacy prefix `!` (either wins).
    let mut important = false;
    let mut rest = rest_all;
    if let Some(r) = rest.strip_prefix('!') {
        important = true;
        rest = r;
    }
    if let Some(r) = rest.strip_suffix('!') {
        important = true;
        rest = r;
    }

    // Pass 1: variant segmentation — bracket-aware top-level ':' split.
    let (variant_slices, utility) = split_variants(rest)?;

    // Pass 2: the utility — negation, body, arbitrary group, modifier.
    let negative = utility.starts_with('-');
    let body_src = if negative { &utility[1..] } else { utility };
    let (body, arbitrary, modifier) = split_body(body_src)?;

    if body.is_empty() {
        return Err(LexError::whole(rest_all.len(), LexErrorKind::EmptyHead));
    }
    // Elements: the dash-split must have no empty segments (the oracle's
    // "double '-'" rejection; NO cap — the documented divergence).
    let mut start = 0usize;
    for (i, &b) in body.as_bytes().iter().enumerate() {
        if b == b'-' {
            if i == start {
                return Err(LexError::whole(rest_all.len(), LexErrorKind::EmptyElement));
            }
            start = i + 1;
        }
    }
    if start == body.len() {
        return Err(LexError::whole(rest_all.len(), LexErrorKind::EmptyElement));
    }

    Ok(Candidate {
        raw,
        span: (0, rest_all.len()),
        variants: variant_slices,
        body,
        arbitrary,
        modifier,
        negative,
        important,
    })
}

/// Parse a class STRING into per-token candidates, tolerating per-class
/// failures — the honest-degradation contract: unknown syntax never takes
/// down the whole string, it reports typed [`LexFailure`]s with the
/// token's byte span inside `input`. Spans index into `input`.
pub fn parse_class_string(input: &str) -> (Vec<Candidate<'_>>, Vec<LexFailure<'_>>) {
    let mut tokens = Vec::new();
    let mut errors = Vec::new();
    let mut offset = 0usize;
    for raw in input.split_ascii_whitespace() {
        // The token's span inside `input` (whitespace-skipped).
        let start = input[offset..].find(raw).map_or(offset, |i| offset + i);
        let end = start + raw.len();
        offset = end;
        match parse_candidate(raw) {
            Ok(mut c) => {
                c.span = (start, end);
                tokens.push(c);
            }
            Err(e) => errors.push(LexFailure {
                span: (start, end),
                raw,
                kind: e.kind,
            }),
        }
    }
    (tokens, errors)
}

/// Split the variant stack off the utility: every top-level `:` (not
/// inside `[...]`/`(...)`) ends a segment. An EMPTY segment is the one
/// variant-level error (the oracle's "empty variant before ':'").
fn split_variants(rest: &str) -> Result<(Vec<&str>, &str), LexError> {
    let mut segments: Vec<&str> = Vec::new();
    let mut scan = rest;
    while let Some(idx) = find_top_level(scan, b':') {
        let seg = &scan[..idx];
        if seg.is_empty() {
            return Err(LexError::whole(rest.len(), LexErrorKind::EmptyVariant));
        }
        segments.push(seg);
        scan = &scan[idx + 1..];
    }
    Ok((segments, scan))
}

/// Find the first top-level `needle` byte (not inside `[...]`/`(...)`) —
/// the oracle's scanner, shared by both passes.
fn find_top_level(s: &str, needle: u8) -> Option<usize> {
    let bytes = s.as_bytes();
    let (mut sq, mut par) = (0usize, 0usize);
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'[' if par == 0 => sq += 1,
            b']' if par == 0 => sq = sq.saturating_sub(1),
            b'(' if sq == 0 => par += 1,
            b')' if sq == 0 => par = par.saturating_sub(1),
            _ if b == needle && sq == 0 && par == 0 => return Some(i),
            _ => {}
        }
    }
    None
}

/// Split `w-[calc(100%/3)]/50` into body/arbitrary/raw-modifier — the
/// oracle's bracket-aware body splitter, byte for byte: a top-level `/`
/// ends the body only when no group is open; the group must close before
/// the slash; nothing may ride after it; the `-` joining body and group
/// is stripped from the body.
fn split_body(s: &str) -> Result<(&str, Option<Arb<'_>>, Option<&str>), LexError> {
    let bytes = s.as_bytes();
    let (mut sq, mut par) = (0usize, 0usize);
    let mut group: Option<(usize, usize, Form)> = None;
    let mut open: Option<(usize, Form)> = None;
    let mut slash: Option<usize> = None;
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'[' if par == 0 => {
                if sq == 0 && slash.is_none() && group.is_none() {
                    open = Some((i, Form::Bracket));
                }
                sq += 1;
            }
            b']' if par == 0 => {
                sq = sq.saturating_sub(1);
                if sq == 0
                    && let Some((st, Form::Bracket)) = open
                {
                    group = Some((st, i, Form::Bracket));
                    open = None;
                }
            }
            b'(' if sq == 0 => {
                if par == 0 && slash.is_none() && group.is_none() {
                    open = Some((i, Form::Paren));
                }
                par += 1;
            }
            b')' if sq == 0 => {
                par = par.saturating_sub(1);
                if par == 0
                    && let Some((st, Form::Paren)) = open
                {
                    group = Some((st, i, Form::Paren));
                    open = None;
                }
            }
            b'/' if sq == 0 && par == 0 && slash.is_none() => slash = Some(i),
            _ => {}
        }
    }
    if sq != 0 || par != 0 {
        return Err(LexError::whole(s.len(), LexErrorKind::UnbalancedGroup));
    }

    let (head, modifier) = match slash {
        Some(i) => (&s[..i], Some(&s[i + 1..])),
        None => (s, None),
    };

    // Extract the arbitrary group out of the head, if one closed there.
    if let Some((bs, be, form)) = group {
        if be >= head.len() {
            return Err(LexError::whole(s.len(), LexErrorKind::GroupCrossesSlash));
        }
        let after_group = &head[be + 1..];
        if !after_group.is_empty() {
            return Err(LexError::whole(s.len(), LexErrorKind::TextAfterGroup));
        }
        let body = head[..bs].strip_suffix('-').unwrap_or(&head[..bs]);
        let raw = &s[bs + 1..be];
        let arb = extract_arbitrary(raw, form)?;
        return Ok((body, Some(arb), modifier));
    }
    Ok((head, None, modifier))
}

/// The v4 type hints (`[length:200px]`); anything else with a `:` in
/// bracket form is the arbitrary-PROPERTY syntax. Public because the
/// pipeline's arbitrary-value grammar reads the SAME table (one source
/// of truth: a hint the lexer classifies, the value pass constrains
/// with — a drift between the two would be a compile error away).
pub const HINTS: &[&str] = &[
    "length",
    "color",
    "url",
    "image",
    "number",
    "percentage",
    "integer",
    "angle",
    "time",
    "ratio",
    "position",
    "family-name",
    "generic-name",
    "absolute-size",
    "relative-size",
    "string",
];

/// Build an [`Arb`] from the raw inner text, splitting type hints and
/// flagging the property form — the oracle's `extract_arbitrary`.
fn extract_arbitrary(raw: &str, form: Form) -> Result<Arb<'_>, LexError> {
    if raw.is_empty() {
        return Err(LexError::whole(raw.len(), LexErrorKind::EmptyArbitrary));
    }
    let (hint, value) = match form {
        Form::Bracket => raw
            .split_once(':')
            .map_or((None, raw), |(h, v)| (Some(h), v)),
        Form::Paren => (None, raw),
    };
    let is_property = form == Form::Bracket && hint.is_some_and(|h| !HINTS.contains(&h));
    Ok(Arb {
        hint,
        value,
        form,
        is_property,
    })
}
