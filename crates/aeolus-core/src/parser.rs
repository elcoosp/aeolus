//! Tokenizer + parser for Tailwind v4 class strings (v0.2 grammar).
//!
//! Grammar (per whitespace-separated token):
//!
//! ```text
//! token     := important_prefix? variant* utility important_suffix?
//! variant   := (group_prefix? name | has | not | data | aria | structural | arbitrary) ':'
//! utility   := '-'? body ('[' arbitrary ']' | '(' arbitrary ')')? ('/' modifier)?
//! ```
//!
//! v4 rules implemented:
//! * variants stack **left to right** (first variant = outermost condition);
//! * `!` may be a **suffix** (v4) or a **prefix** (v3 compat);
//! * arbitrary values `[...]` and `(...)` may contain `/` and `:` — they are
//!   scanned with bracket+paren awareness so `w-[calc(100%/3)]` parses;
//! * the `/modifier` is kept **raw** (`"2"`, `"[0.37]"`, `"6"`, `"(--o)"`) —
//!   interpretation (fraction denominator vs opacity percent vs
//!   line-height scale) belongs to the utility, not the parser (plan §3.1:
//!   fixes F1/F6/F7 at the source);
//! * unknown variant names parse as [`NamedVariant::Custom`] and are
//!   resolved against theme breakpoints / `@custom-variant` later — the
//!   runtime path never rejects a class for an unknown *name* (the `tw!`
//!   macro enforces the strict compile-time check itself).

use crate::ast::{
    ArbForm, Arbitrary, AttrKind, Breakpoint, ClassToken, INLINE_ELEMENTS, INLINE_VARIANTS, Inline,
    NamedVariant, ParentKind, Segments, Structural, Variant,
};
use crate::color::ParseError;

/// Parse a class string into tokens, tolerating per-class failures.
///
/// Returns `(tokens, errors)` — a class that fails to parse is SKIPPED and
/// reported in `errors` as `(raw_class, message)`, matching the
/// honest-degradation contract: unknown syntax must never take down the
/// whole string. Callers that need the old hard-fail semantics (the `tw!`
/// proc macro) turn non-empty `errors` into a compile error themselves.
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

    // Variants: split on ':' with bracket+paren awareness.
    let mut variant_parts: Vec<&str> = Vec::with_capacity(4);
    let mut scan = rest;
    while let Some(idx) = find_top_level(scan, b':') {
        variant_parts.push(&scan[..idx]);
        scan = &scan[idx + 1..];
    }

    // Parse each variant; surface the first precise parse error.
    let mut parsed: Vec<Variant<'_>> = Vec::with_capacity(variant_parts.len());
    for v in &variant_parts {
        parsed.push(parse_variant(v)?);
    }
    let variants = Inline::<Variant<'_>, INLINE_VARIANTS>::collect(Variant::Arbitrary(""), parsed)
        .ok_or_else(|| ParseError::new(format!("too many variants (max {INLINE_VARIANTS})")))?;

    // Negative.
    let negative = scan.starts_with('-');
    let body_src = if negative { &scan[1..] } else { scan };

    // Arbitrary value + raw modifier with bracket/paren-aware scanning.
    let (body, arbitrary, modifier) = split_body(body_src)?;

    if body.is_empty() {
        // Bare bracket with no utility head: `[mask-type:luminance]` (an
        // arbitrary property) or garbage. Both are honest per-class errors.
        return Err(ParseError::new("utility head is empty"));
    }
    let mut parts = body.split('-');
    let elements =
        Segments::<INLINE_ELEMENTS>::from_iter_fn(|_| parts.next()).ok_or_else(|| {
            ParseError::new(format!("too many utility elements (max {INLINE_ELEMENTS})"))
        })?;
    if elements.as_slice().iter().any(|e| e.is_empty()) {
        return Err(ParseError::new("empty utility element (double '-')"));
    }

    Ok(ClassToken {
        variants,
        elements,
        body,
        arbitrary,
        modifier,
        negative,
        important,
        raw,
    })
}

/// Find the first top-level `needle` byte (not inside `[...]` or `(...)`).
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

/// Split `group-focus/tbd` → (`focus`, Some(`tbd`)); `group-data-[x=a/b]`
/// keeps the bracket intact: the label split only fires when the part after
/// the `/` carries no brackets.
fn split_parent_label(v: &str) -> (&str, Option<&str>) {
    match v.rsplit_once('/') {
        Some((n, l)) if !n.is_empty() && !l.contains('[') && !l.contains(']') => (n, Some(l)),
        _ => (v, None),
    }
}

/// Parse a single variant prefix (`"hover"`, `"group-hover/menu"`,
/// `"not-data-[x]"`, ...). Public grammar surface (fuzz target + `tw!`).
pub fn parse_variant(v: &str) -> Result<Variant<'_>, ParseError> {
    if v.is_empty() {
        return Err(ParseError::new("empty variant before ':'"));
    }
    if v.starts_with('[') {
        return Ok(Variant::Arbitrary(v));
    }
    if v.starts_with("::") {
        return Ok(Variant::PseudoElement(v));
    }
    if let Some(rest) = v.strip_prefix("group-") {
        let (inner, label) = split_parent_label(rest);
        return Ok(Variant::Grouped {
            parent: ParentKind::Group,
            inner: Box::new(parse_variant(inner)?),
            label,
        });
    }
    if let Some(rest) = v.strip_prefix("peer-") {
        let (inner, label) = split_parent_label(rest);
        return Ok(Variant::Grouped {
            parent: ParentKind::Peer,
            inner: Box::new(parse_variant(inner)?),
            label,
        });
    }
    if let Some(rest) = v.strip_prefix("not-") {
        return Ok(Variant::Not(Box::new(parse_variant(rest)?)));
    }
    if let Some(rest) = v.strip_prefix("has-") {
        // `has-checked` (named) or `has-[:focus]` / `has-[>svg]` (selector).
        return Ok(Variant::Has(rest));
    }
    if let Some(rest) = v.strip_prefix("data-") {
        let (name, value) = parse_attr(rest);
        return Ok(Variant::Attr {
            kind: AttrKind::Data,
            name,
            value,
        });
    }
    if let Some(rest) = v.strip_prefix("aria-") {
        let (name, value) = parse_attr(rest);
        return Ok(Variant::Attr {
            kind: AttrKind::Aria,
            name,
            value,
        });
    }
    if v.starts_with("supports-") || v == "*" || v == "**" {
        return Ok(Variant::Arbitrary(v));
    }
    if let Some(rest) = v.strip_prefix("nth-") {
        if let Some(st) = parse_nth(rest, false) {
            return Ok(Variant::Structural(st));
        }
        return Ok(Variant::Arbitrary(v)); // nth-[2n+1] selector forms
    }
    if let Some(st) = structural_of(v) {
        return Ok(Variant::Structural(st));
    }
    Ok(Variant::Named(parse_named_variant(v)))
}

/// The bare structural variants (`first`, `odd`, ...).
fn structural_of(v: &str) -> Option<Structural> {
    Some(match v {
        "first" => Structural::First,
        "last" => Structural::Last,
        "odd" => Structural::Odd,
        "even" => Structural::Even,
        "only" => Structural::Only,
        "empty" => Structural::Empty,
        _ => return None,
    })
}

/// `data-[state=open]` | `data-state` (bracket optional, v4 canonical is
/// bracketed) → (name, value).
fn parse_attr(rest: &str) -> (&str, Option<&str>) {
    if let Some(inner) = rest.strip_prefix('[').and_then(|r| r.strip_suffix(']')) {
        match inner.split_once('=') {
            Some((n, v)) => (n, Some(v)),
            None => (inner, None),
        }
    } else {
        (rest, None)
    }
}

/// `nth-3` / `nth-last-2` → `Some(Structural::…)`; `nth-[2n+1]` → None.
fn parse_nth(rest: &str, last: bool) -> Option<Structural> {
    let n: i32 = rest.parse().ok()?;
    Some(if last {
        Structural::NthLast(n)
    } else {
        Structural::Nth(n)
    })
}

/// Look up a named variant (`"hover"`, `"md"`, `"@sm"`) in the variant
/// table. Unknown identifiers become [`NamedVariant::Custom`] (resolved
/// later against theme breakpoints, container sizes and `@custom-variant`).
pub fn parse_named_variant(name: &str) -> NamedVariant<'_> {
    match name {
        "hover" => NamedVariant::Hover,
        "focus" => NamedVariant::Focus,
        "focus-visible" => NamedVariant::FocusVisible,
        "focus-within" => NamedVariant::FocusWithin,
        "active" => NamedVariant::Active,
        "disabled" => NamedVariant::Disabled,
        "visited" => NamedVariant::Visited,
        "target" => NamedVariant::Target,
        "enabled" => NamedVariant::Enabled,
        "checked" => NamedVariant::Checked,
        "indeterminate" => NamedVariant::Indeterminate,
        "required" => NamedVariant::Required,
        "optional" => NamedVariant::Optional,
        "valid" => NamedVariant::Valid,
        "invalid" => NamedVariant::Invalid,
        "user-valid" => NamedVariant::UserValid,
        "user-invalid" => NamedVariant::UserInvalid,
        "placeholder-shown" => NamedVariant::PlaceholderShown,
        // v0.9 §1: pseudo-element variants v4 names as plain variants
        // (`selection:` → `&::selection`, ...). They classify onto
        // `Test::Pseudo`, not runtime states.
        "selection" => NamedVariant::Pseudo(crate::ast::PseudoElement::Selection),
        "placeholder" => NamedVariant::Pseudo(crate::ast::PseudoElement::Placeholder),
        "file" => NamedVariant::Pseudo(crate::ast::PseudoElement::FileSelectorButton),
        "autofill" => NamedVariant::Autofill,
        "read-only" => NamedVariant::ReadOnly,
        "in-range" => NamedVariant::InRange,
        "out-of-range" => NamedVariant::OutOfRange,
        "dark" => NamedVariant::Dark,
        "print" => NamedVariant::Print,
        "motion-safe" => NamedVariant::MotionSafe,
        "motion-reduce" => NamedVariant::MotionReduce,
        "contrast-more" => NamedVariant::ContrastMore,
        "contrast-less" => NamedVariant::ContrastLess,
        "forced-colors" => NamedVariant::ForcedColors,
        "portrait" => NamedVariant::Portrait,
        "landscape" => NamedVariant::Landscape,
        "rtl" => NamedVariant::Rtl,
        "ltr" => NamedVariant::Ltr,
        "pointer-fine" => NamedVariant::PointerFine,
        "pointer-coarse" => NamedVariant::PointerCoarse,
        "pointer-none" => NamedVariant::PointerNone,
        "any-pointer-fine" => NamedVariant::AnyPointerFine,
        "any-pointer-coarse" => NamedVariant::AnyPointerCoarse,
        "any-pointer-none" => NamedVariant::AnyPointerNone,
        other => {
            // min-* / max-* viewport variants.
            if let Some(bp) = other.strip_prefix("min-").and_then(Breakpoint::from_name) {
                return NamedVariant::Min(bp);
            }
            if let Some(bp) = other.strip_prefix("max-").and_then(Breakpoint::from_name) {
                return NamedVariant::Max(bp);
            }
            // @container forms: @sm, @max-sm, @md/name, @min-[400px].
            if let Some(rest) = other.strip_prefix('@') {
                if let Some(bp) = rest.strip_prefix("max-").and_then(Breakpoint::from_name) {
                    return NamedVariant::MaxContainer(bp);
                }
                let bare = rest.split('/').next().unwrap_or(rest);
                if let Some(bp) = Breakpoint::from_name(bare) {
                    // `@md/name`: the label NAMES the container — it
                    // rides the atom's own `name` field through the
                    // custom path's split, the same route the
                    // unknown-size spellings (`@4xl/name`) already take.
                    // Dropping it here merged two different containers
                    // into one condition.
                    if rest.contains('/') {
                        return NamedVariant::Custom(other);
                    }
                    return NamedVariant::Container(bp);
                }
                // @min-[400px], @4xl, named containers — Custom, resolved by
                // the theme (container scale / @custom-variant).
                return NamedVariant::Custom(other);
            }
            if let Some(b) = Breakpoint::from_name(other) {
                return NamedVariant::Breakpoint(b);
            }
            NamedVariant::Custom(other)
        }
    }
}

/// Split `w-[calc(100%/3)]/50` into body/arbitrary/raw-modifier.
///
/// A top-level `/` ends the body only if no `[ ]`/`( )` group is open, so
/// `w-[calc(100%/3)]` survives and `bg-[#fff]/50` still splits (plan §3.1).
/// The modifier is RAW: `"50"`, `"[0.37]"`, `"(--o)"`.
fn split_body(s: &str) -> Result<(&str, Option<Arbitrary<'_>>, Option<&str>), ParseError> {
    let bytes = s.as_bytes();
    let (mut sq, mut par) = (0usize, 0usize);
    let mut group: Option<(usize, usize, ArbForm)> = None;
    let mut open: Option<(usize, ArbForm)> = None;
    let mut slash: Option<usize> = None;
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'[' if par == 0 => {
                if sq == 0 && slash.is_none() && group.is_none() {
                    open = Some((i, ArbForm::Bracket));
                }
                sq += 1;
            }
            b']' if par == 0 => {
                sq = sq.saturating_sub(1);
                if sq == 0
                    && let Some((st, ArbForm::Bracket)) = open
                {
                    group = Some((st, i, ArbForm::Bracket));
                    open = None;
                }
            }
            b'(' if sq == 0 => {
                if par == 0 && slash.is_none() && group.is_none() {
                    open = Some((i, ArbForm::Paren));
                }
                par += 1;
            }
            b')' if sq == 0 => {
                par = par.saturating_sub(1);
                if par == 0
                    && let Some((st, ArbForm::Paren)) = open
                {
                    group = Some((st, i, ArbForm::Paren));
                    open = None;
                }
            }
            b'/' if sq == 0 && par == 0 && slash.is_none() => slash = Some(i),
            _ => {}
        }
    }
    if sq != 0 || par != 0 {
        return Err(ParseError::new("unbalanced '[' or '(' in utility"));
    }

    let (head, modifier) = match slash {
        Some(i) => (&s[..i], Some(&s[i + 1..])),
        None => (s, None),
    };

    // Extract the arbitrary group out of the head, if one closed there.
    if let Some((bs, be, form)) = group {
        if be >= head.len() {
            // The group extends past the top-level slash — impossible for a
            // well-formed token; treat conservatively as a parse error.
            return Err(ParseError::new("arbitrary group crosses the '/'"));
        }
        let after_group = &head[be + 1..];
        if !after_group.is_empty() {
            return Err(ParseError::new("unexpected text after arbitrary group"));
        }
        let body = head[..bs].strip_suffix('-').unwrap_or(&head[..bs]);
        let raw = &s[bs + 1..be];
        let arb = extract_arbitrary(raw, form)?;
        return Ok((body, Some(arb), modifier));
    }
    Ok((head, None, modifier))
}

/// Build an [`Arbitrary`] from the raw inner text, splitting type hints
/// (`[length:200px]`) and flagging the arbitrary-property form
/// (`[mask-type:luminance]`).
fn extract_arbitrary(raw: &str, form: ArbForm) -> Result<Arbitrary<'_>, ParseError> {
    const HINTS: &[&str] = &[
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
    if raw.is_empty() {
        return Err(ParseError::new("empty arbitrary value"));
    }
    let (hint, value) = match form {
        ArbForm::Bracket => raw
            .split_once(':')
            .map_or((None, raw), |(h, v)| (Some(h), v)),
        ArbForm::Paren => (None, raw),
    };
    // `[mask-type:luminance]` — a bare CSS property declaration. Values
    // carrying a ':' whose prefix is not a known v4 type hint are flagged
    // as properties; the resolver reports them with a precise reason.
    let is_property = form == ArbForm::Bracket && hint.is_some_and(|h| !HINTS.contains(&h));
    Ok(Arbitrary {
        hint,
        value,
        form,
        is_property,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Breakpoint, NamedVariant, PseudoElement};

    /// The named-variant table is the merge-key + classification surface:
    /// every name must land on the typed arm the resolver routes from.
    /// v0.9 §1 pins the three pseudo-element names here (they were
    /// `Custom` — compile errors for `tw!` — before).
    #[test]
    fn named_variant_table_routes_every_family() {
        // States.
        assert_eq!(parse_named_variant("hover"), NamedVariant::Hover);
        assert_eq!(
            parse_named_variant("focus-visible"),
            NamedVariant::FocusVisible
        );
        assert_eq!(
            parse_named_variant("placeholder-shown"),
            NamedVariant::PlaceholderShown
        );
        // Pseudo-elements (v0.9 §1) — their own arm, NOT states.
        assert_eq!(
            parse_named_variant("selection"),
            NamedVariant::Pseudo(PseudoElement::Selection)
        );
        assert_eq!(
            parse_named_variant("placeholder"),
            NamedVariant::Pseudo(PseudoElement::Placeholder)
        );
        assert_eq!(
            parse_named_variant("file"),
            NamedVariant::Pseudo(PseudoElement::FileSelectorButton)
        );
        // Media.
        assert_eq!(parse_named_variant("dark"), NamedVariant::Dark);
        assert_eq!(
            parse_named_variant("motion-reduce"),
            NamedVariant::MotionReduce
        );
        assert_eq!(
            parse_named_variant("pointer-coarse"),
            NamedVariant::PointerCoarse
        );
        // Viewport bounds + breakpoints.
        assert_eq!(
            parse_named_variant("min-md"),
            NamedVariant::Min(Breakpoint::Md)
        );
        assert_eq!(
            parse_named_variant("max-sm"),
            NamedVariant::Max(Breakpoint::Sm)
        );
        assert_eq!(
            parse_named_variant("md"),
            NamedVariant::Breakpoint(Breakpoint::Md)
        );
        // Container forms.
        assert_eq!(
            parse_named_variant("@md"),
            NamedVariant::Container(Breakpoint::Md)
        );
        assert_eq!(
            parse_named_variant("@max-lg"),
            NamedVariant::MaxContainer(Breakpoint::Lg)
        );
        // Unknown names stay `Custom` — the theme resolves custom variants.
        assert!(matches!(
            parse_named_variant("hocus"),
            NamedVariant::Custom(_)
        ));
    }

    /// `parse_token`: negative sign, arbitrary values, raw modifiers and
    /// the per-class honesty errors.
    #[test]
    fn parse_token_shapes() {
        let t = parse_token("bg-red-500").unwrap();
        assert_eq!(&*t.elements, ["bg", "red", "500"]);
        assert_eq!(t.body, "bg-red-500");
        assert!(!t.negative && t.arbitrary.is_none() && t.modifier.is_none());

        let t = parse_token("-mt-4").unwrap();
        assert!(t.negative, "the leading `-` is the negative sign");

        let t = parse_token("w-[calc(100%/3)]").unwrap();
        let arb = t.arbitrary.expect("calc survives the split");
        assert_eq!(arb.value, "calc(100%/3)");

        let t = parse_token("bg-[#fff]/50").unwrap();
        assert_eq!(t.modifier, Some("50"), "top-level `/` still splits");
        assert!(t.arbitrary.is_some());

        // Honesty errors: empty head, double dash.
        assert!(parse_token("[]").is_err());
        assert!(parse_token("p--4").is_err());
        assert!(parse_token("").is_err());
    }

    /// `parse_class_string` splits on whitespace, keeps going on bad
    /// classes, and reports per-class errors alongside the good tokens —
    /// one malformed class never poisons the rest (the cn() contract).
    #[test]
    fn parse_class_string_is_total_per_class() {
        let (tokens, errors) = parse_class_string("bg-red-500   p-4 p--4 hover:bg-blue-500");
        assert_eq!(tokens.len(), 3, "bg-red-500, p-4, hover:bg-blue-500");
        assert_eq!(
            errors.len(),
            1,
            "`p--4` (double dash) is reported, not fatal"
        );
        assert_eq!(errors[0].0, "p--4");
        // Source order is preserved in the token stream.
        assert_eq!(tokens[0].body, "bg-red-500");
        assert_eq!(tokens[2].body, "bg-blue-500");
        // Stacked variants keep source order (outermost first).
        let (tokens, _) = parse_class_string("dark:hover:bg-white");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].variants.len(), 2);
    }
}
