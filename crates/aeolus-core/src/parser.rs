//! Tokenizer + parser for Tailwind v4 class strings.
//!
//! Grammar (per whitespace-separated token):
//!
//! ```text
//! token     := important_prefix? variant* utility important_suffix?
//! variant   := (group_prefix? name | arbitrary) ':'
//! utility   := '-'? element ('-' element)* ('[' arbitrary ']')? ('/' opacity)?
//! ```
//!
//! v4 rules implemented:
//! * variants stack **left to right** (first variant = outermost condition);
//! * `!` may be a **suffix** (v4) or a **prefix** (v3 compat);
//! * arbitrary values `[...]` may contain `/` and `:` — they are scanned with
//!   bracket awareness so `w-[calc(100%/3)]` parses;
//! * opacity modifier `/50` applies to the utility, not the variants.

use crate::ast::{Breakpoint, ClassToken, NamedVariant, ParentKind, Variant};
use crate::color::ParseError;

/// Parse a class string into tokens, tolerating per-class failures.
///
/// Returns `(tokens, errors)` — a class that fails to parse (e.g. an unknown
/// variant like `has-[>svg]:px-3`) is SKIPPED and reported in `errors` as
/// `(raw_class, message)`, matching the honest-degradation contract: unknown
/// syntax must never take down the whole string. Callers that need the old
/// hard-fail semantics (the `tw!` proc macro) turn non-empty `errors` into a
/// compile error themselves.
pub fn parse_class_string(input: &str) -> (Vec<ClassToken<'_>>, Vec<(String, String)>) {
    let mut tokens = Vec::new();
    let mut errors = Vec::new();
    for raw in input.split_ascii_whitespace() {
        match parse_token(raw) {
            Ok(t) => tokens.push(t),
            Err(e) => errors.push((raw.to_string(), format!("{raw}: {}", e.message))),
        }
    }
    (tokens, errors)
}

/// Parse a single whitespace-free class token.
pub fn parse_token(raw: &str) -> Result<ClassToken<'_>, ParseError> {
    let mut rest = raw;

    let mut important = false;
    if let Some(r) = rest.strip_prefix('!') {
        important = true;
        rest = r;
    }
    if let Some(r) = rest.strip_suffix('!') {
        important = true;
        rest = r;
    }

    // Variants: split on ':' with bracket awareness.
    let mut variants = Vec::new();
    while let Some(idx) = find_top_level(rest, b':') {
        let v = &rest[..idx];
        variants.push(parse_variant(v)?);
        rest = &rest[idx + 1..];
    }

    // Negative.
    let negative = rest.starts_with('-');
    if negative {
        rest = &rest[1..];
    }

    // Arbitrary value + opacity with bracket-aware scanning.
    let (body, arbitrary, opacity) = split_body(rest)?;

    if body.is_empty() {
        return Err(ParseError::new("empty utility"));
    }
    let elements: Vec<&str> = body.split('-').collect();
    if elements.iter().any(|e| e.is_empty()) {
        return Err(ParseError::new("empty utility element (double '-')"));
    }

    Ok(ClassToken {
        variants,
        elements,
        arbitrary,
        opacity,
        negative,
        important,
        raw,
    })
}

/// Find the first top-level `needle` byte (not inside `[...]`).
fn find_top_level(s: &str, needle: u8) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut depth = 0usize;
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'[' => depth += 1,
            b']' => depth = depth.saturating_sub(1),
            _ if b == needle && depth == 0 => return Some(i),
            _ => {}
        }
    }
    None
}

fn parse_variant(v: &str) -> Result<Variant<'_>, ParseError> {
    if v.is_empty() {
        return Err(ParseError::new("empty variant before ':'"));
    }
    let (parent, name) = if let Some(n) = v.strip_prefix("group-") {
        (Some(ParentKind::Group), n)
    } else if let Some(n) = v.strip_prefix("peer-") {
        (Some(ParentKind::Peer), n)
    } else {
        (None, v)
    };
    if v.starts_with('[') || name.starts_with("data-") || name.starts_with("aria-") {
        return Ok(Variant::Arbitrary(v));
    }
    // group-hover/fancy label form: `group-focus/tbd`
    let (name, label) = match name.split_once('/') {
        Some((n, l)) if parent.is_some() => (n, Some(l)),
        _ => (name, None),
    };
    let named = parse_named_variant(name)
        .ok_or_else(|| ParseError::new(format!("unknown variant {v:?} (v1 supports the Shadcn set: hover, focus, focus-visible, focus-within, active, disabled, dark, breakpoints, @container, group-*, peer-*)")))?;
    Ok(match parent {
        Some(parent_kind) => Variant::Grouped {
            parent: parent_kind,
            name: named,
            label,
        },
        None => Variant::Named(named),
    })
}

/// Look up a named variant (`"hover"`, `"md"`, `"@sm"`) in the variant table.
pub fn parse_named_variant(name: &str) -> Option<NamedVariant> {
    Some(match name {
        "hover" => NamedVariant::Hover,
        "focus" => NamedVariant::Focus,
        "focus-visible" => NamedVariant::FocusVisible,
        "focus-within" => NamedVariant::FocusWithin,
        "active" => NamedVariant::Active,
        "disabled" => NamedVariant::Disabled,
        "visited" => NamedVariant::Visited,
        "dark" => NamedVariant::Dark,
        other => {
            if let Some(b) = other.strip_prefix('@') {
                NamedVariant::Container(Breakpoint::from_name(b)?)
            } else {
                NamedVariant::Breakpoint(Breakpoint::from_name(other)?)
            }
        }
    })
}

/// Split `w-[calc(100%/3)]/50` into body/`Some("calc(100%/3)")`/`Some(50.0)`.
fn split_body(s: &str) -> Result<(&str, Option<&str>, Option<f32>), ParseError> {
    let bytes = s.as_bytes();
    let mut depth = 0usize;
    let mut bracket: Option<(usize, usize)> = None;
    let mut slash = None;
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'[' => {
                if depth == 0 {
                    bracket = Some((i, 0));
                }
                depth += 1;
            }
            b']' => {
                depth = depth.saturating_sub(1);
                if depth == 0
                    && let Some((start, _)) = bracket
                {
                    bracket = Some((start, i));
                }
            }
            b'/' if depth == 0 && slash.is_none() && bracket.is_none() => slash = Some(i),
            _ => {}
        }
    }
    if depth != 0 {
        return Err(ParseError::new("unbalanced '[' in utility"));
    }
    // Trailing opacity after the arbitrary bracket, e.g. `bg-[#fff]/50`.
    if let Some((bs, be)) = bracket {
        let after = &s[be + 1..];
        let opacity = parse_opacity(after)?;
        let body = s[..bs].trim_end_matches('-');
        let arb = &s[bs + 1..be];
        return Ok((body, Some(arb), opacity));
    }
    match slash {
        Some(i) => {
            let opacity = parse_opacity(&s[i + 1..])?;
            Ok((s[..i].trim_end_matches('-'), None, opacity))
        }
        None => Ok((s, None, None)),
    }
}

fn parse_opacity(s: &str) -> Result<Option<f32>, ParseError> {
    if s.is_empty() {
        return Ok(None);
    }
    let inner = s
        .strip_prefix('[')
        .and_then(|r| r.strip_suffix(']'))
        .unwrap_or(s);
    // Arbitrary opacity `/[0.37]` is a raw alpha fraction in v4.
    if inner.contains('.') && !inner.contains('%') {
        let v: f32 = inner
            .parse()
            .map_err(|_| ParseError::new(format!("bad opacity {s:?}")))?;
        return Ok(Some(v * 100.0));
    }
    let v: f32 = inner
        .trim_end_matches('%')
        .parse()
        .map_err(|_| ParseError::new(format!("bad opacity {s:?}")))?;
    Ok(Some(v))
}
