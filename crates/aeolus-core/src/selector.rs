//! Arbitrary selector variants — classification into typed probe atoms
//! (v0.3 plan §1, `AEOLUS_V0.3.md`).
//!
//! The parser stores arbitrary variants RAW (`Variant::Arbitrary`) —
//! semantics belong to the resolver (v0.2 plan §1.2). This module is that
//! semantics: it classifies the raw variant text into either a conjunction
//! of [`Atom`]s the `Probe` model can evaluate, or a precise
//! `Unsupported` reason (the honesty contract, v0.2 plan §7).
//!
//! Semantics verified against tailwindcss v4.1.17 source, not the docs:
//! * `packages/tailwindcss/src/candidate.ts` — an arbitrary variant whose
//!   selector has no `&` (and does not start with `>`/`+`/`~`/`@`) is
//!   wrapped as `&:is(sel)` — an implicit SELF test (`[p]:` ≡ the element
//!   is a `p`); relative selectors (`[>img]:`) are invalid standalone;
//!   at-rule variants must not contain `&` or `{`;
//! * `compile.ts` — `[&…]` nests the selector under the element, so
//!   `[&_svg]`/`[&>*]` style OTHER elements;
//! * `variants.ts` — `*` = `:is(& > *)`, `**` = `:is(& *)` (also other
//!   elements), `hover` = `&:hover` (so `[&:hover]` ≡ `hover:`).
//!
//! The typed subset is exactly what constrains THE ELEMENT — the only
//! thing [`crate::eval::Probe`] answers about. Everything else is reported
//! with a per-form reason, never a blanket "arbitrary variants
//! unsupported" and never a silent drop.

use crate::ast::{Media, Structural};
use crate::cond::{Atom, Test};

// ---------------------------------------------------------------------------
// Precise reasons (&'static str, one per rejected FORM, not per input)
// ---------------------------------------------------------------------------

const OTHER_ELEMENTS: &str = "arbitrary selector styles other elements (descendant/child/sibling of the element) — the Probe model styles the element itself — reported, never silent (v0.3 §1.3)";
const ANCESTOR: &str = "ancestor tag/class selectors have no typed equivalent in the Probe model — reported, never silent (v0.3 §1.3)";
const PSEUDO_ELEMENT: &str = "pseudo-elements have no native equivalent in Rust UI toolkits — reported, never silent (plan §7)";
const CLASS_SEL: &str = "class/id selector tests have no typed equivalent in the Probe model — reported, never silent (v0.3 §1.3)";
const SUPPORTS: &str = "@supports queries have no typed equivalent in the Probe model — reported, never silent (v0.3 §1.3)";
const DISJUNCTION: &str = "selector/media variant lists (,) are a disjunction — the Cond model is a conjunction — reported, never silent (v0.3 §1.3)";
const ANB: &str = "arbitrary An+B structural selectors have no typed equivalent (use nth-<n>) — reported, never silent (v0.3 §1.3)";
const MEDIA_FEATURE: &str =
    "media feature has no typed equivalent in the Probe model — reported, never silent (v0.3 §1.3)";
const MEDIA_LENGTH: &str =
    "media query width must be a px/rem length — reported, never silent (v0.3 §1.3)";
const UNKNOWN_PSEUDO: &str = "unknown pseudo-class in arbitrary selector (only the v4 state/structural/not/has/is forms map) — reported, never silent (v0.3 §1.3)";
const UNKNOWN_ATTR: &str = "bare attribute selectors other than data-*/aria-* have no typed equivalent — reported, never silent (v0.3 §1.3)";
const RELATIVE: &str = "relative selectors (>…) are only valid inside has-*/group-*/peer- (v4.1.17 candidate.ts) — reported, never silent (v0.3 §1.1)";
const AT_RULE: &str = "unsupported at-rule variant — reported, never silent (v0.3 §1.3)";
const AT_RULE_SELECTOR: &str = "at-rule variants containing selectors are deprecated in v4 (use stacked variants) — reported, never silent (v0.3 §1.1)";
const EMPTY_SEL: &str = "empty arbitrary selector — reported, never silent";
const UNRECOGNIZED: &str =
    "unrecognized arbitrary variant form — reported, never silent (v0.3 §1.3)";
const BARE_AMP: &str = "bare '&' variant adds no condition — reported, never silent";
const NESTED_NOT: &str = "negated arbitrary selectors cannot nest in not-*/group-* inners — reported, never silent (v0.3 §1.3)";
const COMPOUND_INNER: &str = "not-*/group-*/peer- inners need a single self test (a compound is a conjunction they cannot express) — reported, never silent (v0.3 §1.3)";

// ---------------------------------------------------------------------------
// Entry points
// ---------------------------------------------------------------------------

/// Classify one raw arbitrary variant (WITH its brackets: `[&_svg]`,
/// `supports-grid`, `*`, `nth-[2n+1]`) into the typed atoms its selector
/// constrains — or a precise reason it has no typed equivalent.
///
/// The result is a CONJUNCTION: `[&:hover:focus]` yields two atoms, both
/// of which must hold (`&:is(...)`-wrapping makes even the implicit forms
/// self tests, so conjunction is always the correct reading).
pub(crate) fn classify(raw: &str) -> Result<Vec<Atom>, &'static str> {
    // The v4 lateral variants style other elements (`*` = `:is(& > *)`,
    // `**` = `:is(& *)` — variants.ts).
    if raw == "*" || raw == "**" {
        return Err(OTHER_ELEMENTS);
    }
    // `supports-[…]` / `supports-grid` — no capability query in the model.
    if raw.starts_with("supports-") {
        return Err(SUPPORTS);
    }
    // `nth-[2n+1]` — only the An+B forms reach the classifier (plain
    // `nth-3` parses as `Structural` in the parser).
    if raw.starts_with("nth-") {
        return Err(ANB);
    }
    let inner = strip_brackets(raw)?;
    let sel = decode_underscores(inner);
    if sel.starts_with('@') {
        return classify_at_rule(&sel);
    }
    if sel.starts_with(['>', '+', '~']) {
        return Err(RELATIVE);
    }
    classify_selector(&sel)
}

/// Classify an arbitrary variant for a `not-*`/`group-*`/`peer-*` INNER:
/// those wrap ONE probe `Test` (negation / related-element), so the
/// selector must classify to exactly one self test.
pub(crate) fn single_test(raw: &str) -> Result<Test, &'static str> {
    let atoms = classify(raw)?;
    if atoms.len() != 1 {
        return Err(COMPOUND_INNER);
    }
    match &atoms[0] {
        Atom::On(t) => Ok(t.clone()),
        // `not-[&:not(:hover)]` would double-negate; the model has no way
        // to express it honestly — report instead.
        Atom::Not(_) => Err(NESTED_NOT),
        _ => Err(COMPOUND_INNER),
    }
}

// ---------------------------------------------------------------------------
// At-rule variants: [@media(…)], [@container(…)]
// ---------------------------------------------------------------------------

fn classify_at_rule(sel: &str) -> Result<Vec<Atom>, &'static str> {
    // v4 (candidate.ts): at-rule variants must not carry selectors —
    // `[@media(print){&:hover}]:` is the deprecated form.
    if sel.contains('{') || sel.contains('&') {
        return Err(AT_RULE_SELECTOR);
    }
    let open = sel.find(['(', ' ']).ok_or(AT_RULE)?;
    let (name, rest) = sel.split_at(open);
    if name.eq_ignore_ascii_case("@supports") {
        return Err(SUPPORTS);
    }
    // The query stays parenthesized; parts strip their own parens so an
    // `and` list keeps every part's parens intact.
    let query = rest.trim();
    if !query.starts_with('(') || !query.ends_with(')') {
        return Err(AT_RULE);
    }
    if name.eq_ignore_ascii_case("@media") {
        media_atoms(query)
    } else if name.eq_ignore_ascii_case("@container") {
        container_atom(query).map(|a| vec![a])
    } else {
        Err(AT_RULE)
    }
}

/// `print`, `prefers-color-scheme:dark`, `min-width:900px`, … — one media
/// condition; `and`-conjunctions conjoin, `,` lists are a disjunction.
fn media_atoms(query: &str) -> Result<Vec<Atom>, &'static str> {
    if query.contains(',') {
        return Err(DISJUNCTION);
    }
    split_and(query)
        .iter()
        .map(|part| media_condition(part))
        .collect::<Result<Vec<Vec<Atom>>, &'static str>>()
        .map(|nested| nested.concat())
}

fn media_condition(part: &str) -> Result<Vec<Atom>, &'static str> {
    let part = part.trim();
    if part.to_ascii_lowercase().contains(" or ") {
        return Err(DISJUNCTION);
    }
    let body = strip_parens(part).trim();
    // Width windows first: `(min-width:900px)`, `(max-width:600px)`, and
    // the v4 canonical range syntax `(width>=640px)` / `(width<=640px)`.
    if let Some(atom) = window_atom(body)? {
        return Ok(vec![atom]);
    }
    let (feature, value) = match body.split_once(':') {
        Some((f, v)) => (f.trim(), Some(v.trim())),
        None => (body, None),
    };
    let feature = feature.to_ascii_lowercase();
    let eq = |v: Option<&str>, expected: &str| v.is_some_and(|v| v.eq_ignore_ascii_case(expected));
    let m = match (feature.as_str(), value) {
        ("print", None) => Media::Print,
        ("prefers-color-scheme", v) if eq(v, "dark") => Media::Dark,
        ("orientation", v) if eq(v, "landscape") => Media::Landscape,
        ("orientation", v) if eq(v, "portrait") => Media::Portrait,
        ("prefers-reduced-motion", v) if eq(v, "reduce") => Media::MotionReduce,
        ("prefers-reduced-motion", v) if eq(v, "no-preference") => Media::MotionSafe,
        ("prefers-contrast", v) if eq(v, "more") => Media::ContrastMore,
        ("prefers-contrast", v) if eq(v, "less") => Media::ContrastLess,
        ("forced-colors", v) if eq(v, "active") => Media::ForcedColors,
        ("pointer", v) if eq(v, "fine") => Media::PointerFine,
        ("pointer", v) if eq(v, "coarse") => Media::PointerCoarse,
        ("pointer", v) if eq(v, "none") => Media::PointerNone,
        ("any-pointer", v) if eq(v, "fine") => Media::AnyPointerFine,
        ("any-pointer", v) if eq(v, "coarse") => Media::AnyPointerCoarse,
        ("any-pointer", v) if eq(v, "none") => Media::AnyPointerNone,
        _ => return Err(MEDIA_FEATURE),
    };
    Ok(vec![Atom::Media(m)])
}

/// `[@container(min-width:400px)]` — container windows (unnamed). An
/// `and` pair of width windows merges into one `[min, max)` window.
fn container_atom(query: &str) -> Result<Atom, &'static str> {
    if query.contains(',') {
        return Err(DISJUNCTION);
    }
    let parts = split_and(query);
    let mut min = None;
    let mut max = None;
    for part in &parts {
        match window_atom(strip_parens(part.trim()).trim())? {
            Some(Atom::Viewport {
                min: Some(px),
                max: None,
            }) => min = Some(px),
            Some(Atom::Viewport {
                min: None,
                max: Some(px),
            }) => max = Some(px),
            _ => return Err(MEDIA_FEATURE),
        }
    }
    Ok(Atom::Container {
        min,
        max,
        name: None,
    })
}

/// `(min-width:900px)` / `(width>=900px)` / `(max-width:…)` / `(width<=…)`
/// → a viewport-window atom; `None` when the condition is not a width
/// window; `Err` for unparseable widths.
fn window_atom(body: &str) -> Result<Option<Atom>, &'static str> {
    let body = body.trim();
    // Range syntax first: `width>=900px`, `width <= 900px`.
    for (needle, is_min) in [(">=", true), ("<=", false)] {
        if let Some(idx) = body.find(needle) {
            let (feat, val) = (&body[..idx], &body[idx + needle.len()..]);
            if !feat.trim().eq_ignore_ascii_case("width") {
                return Err(MEDIA_FEATURE);
            }
            let px = crate::theme::parse_length(val.trim())
                .ok()
                .ok_or(MEDIA_LENGTH)?;
            return Ok(Some(window(is_min, Some(px))));
        }
    }
    let Some((feat, val)) = body.split_once(':') else {
        return Ok(None);
    };
    if !feat.trim().eq_ignore_ascii_case("min-width")
        && !feat.trim().eq_ignore_ascii_case("max-width")
    {
        return Ok(None);
    }
    let px = crate::theme::parse_length(val.trim())
        .ok()
        .ok_or(MEDIA_LENGTH)?;
    Ok(Some(window(
        feat.eq_ignore_ascii_case("min-width"),
        Some(px),
    )))
}

fn window(min: bool, px: Option<f32>) -> Atom {
    if min {
        Atom::Viewport { min: px, max: None }
    } else {
        Atom::Viewport { min: None, max: px }
    }
}

/// Split on a case-insensitive ` and `.
fn split_and(query: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut rest = query;
    loop {
        match rest.to_ascii_lowercase().find(" and ") {
            Some(i) => {
                parts.push(&rest[..i]);
                rest = &rest[i + 5..];
            }
            None => {
                parts.push(rest);
                break;
            }
        }
    }
    parts
}

// ---------------------------------------------------------------------------
// Selector forms: [&…], [a&], [:hover], [p], […]
// ---------------------------------------------------------------------------

fn classify_selector(sel: &str) -> Result<Vec<Atom>, &'static str> {
    let sel = sel.trim();
    if sel.is_empty() {
        return Err(EMPTY_SEL);
    }
    match sel.find('&') {
        None => {
            // v4 (candidate.ts): no `&` → `&:is(sel)` — an implicit SELF
            // test, so the compound (tag, pseudo-classes, attributes)
            // constrains the element itself.
            classify_compound(sel)
        }
        Some(i) => {
            let before_raw = &sel[..i];
            let after = &sel[i + 1..];
            let mut atoms = Vec::new();
            if !before_raw.is_empty() {
                // `[a&]`, `[a&:hover]` — the element is (also) this tag.
                // Whitespace before the `&` (`a &`) is an ANCESTOR
                // selector — a question the Probe cannot ask.
                if before_raw.len() != before_raw.trim().len() {
                    return Err(ANCESTOR);
                }
                atoms.push(Atom::On(tag_test(before_raw)?));
            }
            atoms.extend(classify_compound(after)?);
            if atoms.is_empty() {
                return Err(BARE_AMP);
            }
            Ok(atoms)
        }
    }
}

/// A compound of pseudo-classes and attribute tests (plus an optional
/// leading tag for the implicit-self form) — all conjunctive. A
/// WHITESPACE at the compound's start (`& svg`) is a descendant
/// combinator — other elements, reported.
fn classify_compound(s: &str) -> Result<Vec<Atom>, &'static str> {
    if s.starts_with(char::is_whitespace) {
        return Err(OTHER_ELEMENTS);
    }
    let mut atoms = Vec::new();
    let mut rest = s;
    // Optional leading tag: `p:hover` (implicit form) / `&svg` compound.
    let tag = tag_len(rest);
    if tag > 0 {
        atoms.push(Atom::On(Test::Tag(rest[..tag].into())));
        rest = &rest[tag..];
    }
    while !rest.is_empty() {
        if rest.starts_with("::") {
            return Err(PSEUDO_ELEMENT);
        }
        if let Some(after) = rest.strip_prefix(':') {
            let (name, args, next) = split_pseudo(after)?;
            atoms.extend(pseudo_atoms(name, args)?);
            rest = next;
            continue;
        }
        if let Some(after) = rest.strip_prefix('[') {
            let close = after.find(']').ok_or(UNKNOWN_ATTR)?;
            atoms.push(Atom::On(attr_test(&after[..close])?));
            rest = &after[close + 1..];
            continue;
        }
        // A tag / combinator / class after the positioned part targets
        // other elements (`&>svg`, `&+&`, `&.x`).
        if rest.starts_with('.') || rest.starts_with('#') {
            return Err(CLASS_SEL);
        }
        return Err(OTHER_ELEMENTS);
    }
    Ok(atoms)
}

fn tag_test(s: &str) -> Result<Test, &'static str> {
    let s = s.trim();
    if !s.is_empty() && tag_len(s) == s.len() {
        return Ok(Test::Tag(s.into()));
    }
    Err(CLASS_SEL)
}

/// Length of an ASCII tag name at the start of `s` (0 when absent).
fn tag_len(s: &str) -> usize {
    s.find(|c: char| !(c.is_ascii_alphanumeric() || c == '-'))
        .unwrap_or(s.len())
}

/// `name`, optional `(args)`, and the remainder.
fn split_pseudo(s: &str) -> Result<(&str, Option<&str>, &str), &'static str> {
    let name_len = s
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '-'))
        .unwrap_or(s.len());
    let (name, rest) = s.split_at(name_len);
    if let Some(after_open) = rest.strip_prefix('(') {
        let close = after_open.rfind(')').ok_or(UNKNOWN_PSEUDO)?;
        return Ok((name, Some(&after_open[..close]), &after_open[close + 1..]));
    }
    Ok((name, None, rest))
}

fn pseudo_atoms(name: &str, args: Option<&str>) -> Result<Vec<Atom>, &'static str> {
    // Self states (`:hover`, `:focus-visible`, …) — the same table the
    // named variants use.
    if args.is_none()
        && let Some(s) = state_name(name)
    {
        return Ok(vec![Atom::On(Test::State(s))]);
    }
    match (name, args) {
        ("first-child", None) => Ok(vec![Atom::On(Test::Structural(Structural::First))]),
        ("last-child", None) => Ok(vec![Atom::On(Test::Structural(Structural::Last))]),
        ("only-child", None) => Ok(vec![Atom::On(Test::Structural(Structural::Only))]),
        ("empty", None) => Ok(vec![Atom::On(Test::Structural(Structural::Empty))]),
        ("nth-child", Some(arg)) => nth_arg(arg).map(|s| vec![Atom::On(Test::Structural(s))]),
        ("nth-last-child", Some(arg)) => {
            let arg = arg.trim();
            if arg.contains('n') {
                return Err(ANB);
            }
            arg.parse::<i32>()
                .map(|n| vec![Atom::On(Test::Structural(Structural::NthLast(n)))])
                .map_err(|_| ANB)
        }
        // `:not(:hover)` — one simple inner test.
        ("not", Some(inner)) => simple_test(inner).map(|t| vec![Atom::Not(t)]),
        // `:has(:focus)` — descendant tests (v0.9 §2 generalizes the typed
        // inner from states to any single self test: states, data/aria
        // attributes, structural position, element kind).
        ("has", Some(inner)) => simple_test(inner).map(|t| vec![Atom::Has(t)]),
        // `:is(...)` / `:where(...)` — pure wrappers; a LIST is a
        // disjunction, a single inner classifies recursively.
        ("is" | "where", Some(inner)) => {
            if inner.contains(',') {
                return Err(DISJUNCTION);
            }
            classify_compound(inner)
        }
        _ => Err(UNKNOWN_PSEUDO),
    }
}

/// `nth-child(3)` / `nth-child(odd)` / `nth-child(even)`; An+B forms are
/// reported (`3n`, `2n+1` — anything containing `n`).
fn nth_arg(arg: &str) -> Result<Structural, &'static str> {
    let arg = arg.trim();
    match arg {
        "odd" => return Ok(Structural::Odd),
        "even" => return Ok(Structural::Even),
        _ => {}
    }
    if arg.contains('n') {
        return Err(ANB);
    }
    arg.parse::<i32>().map(Structural::Nth).map_err(|_| ANB)
}

/// The inner of `:not(...)` / `:has(...)`: `:hover` / `hover` /
/// `[data-x=y]` / `:nth-child(3)`.
fn simple_test(inner: &str) -> Result<Test, &'static str> {
    let inner = inner.trim();
    let atoms = classify_compound(inner)?;
    if atoms.len() != 1 {
        return Err(COMPOUND_INNER);
    }
    match &atoms[0] {
        Atom::On(t) => Ok(t.clone()),
        _ => Err(COMPOUND_INNER),
    }
}

/// `[data-state=open]` / `[aria-busy=true]` / `[data-open]`; bare
/// non-data/aria attributes are reported, not guessed.
fn attr_test(inner: &str) -> Result<Test, &'static str> {
    let inner = inner.trim();
    let (name, value) = match inner.split_once('=') {
        Some((n, v)) => (n.trim(), Some(v.trim().trim_matches('"'))),
        None => (inner, None),
    };
    if let Some(d) = name.strip_prefix("data-") {
        return Ok(Test::Data(d.into(), value.map(Into::into)));
    }
    if let Some(a) = name.strip_prefix("aria-") {
        return Ok(Test::Aria(a.into(), value.map(Into::into)));
    }
    Err(UNKNOWN_ATTR)
}

/// `[…]` → the inner text.
fn strip_brackets(raw: &str) -> Result<&str, &'static str> {
    let inner = raw.strip_prefix('[').ok_or(UNRECOGNIZED)?;
    inner.strip_suffix(']').ok_or(UNRECOGNIZED)
}

/// `(...)` → the inner text, or the input unchanged when unparenthesized.
fn strip_parens(s: &str) -> &str {
    s.strip_prefix('(')
        .and_then(|inner| inner.strip_suffix(')'))
        .unwrap_or(s)
}

/// Tailwind's `decodeArbitraryValue`: `_` becomes a space, `\_` escapes a
/// literal underscore.
fn decode_underscores(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                if let Some(next) = chars.next() {
                    out.push(next);
                }
            }
            '_' => out.push(' '),
            other => out.push(other),
        }
    }
    out
}

use crate::resolve::state_name;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::State;
    use crate::cond::Test;

    /// The named boundary (v0.3 §1.3): v4 lateral variants and capability
    /// queries style OTHER elements or ask questions the Probe model cannot
    /// ask — the classifier reports them with the typed reason strings,
    /// never silently.
    #[test]
    fn the_documented_boundary_reports_instead_of_parsing() {
        // Lateral variants (`*` = `:is(& > *)`): other elements.
        assert_eq!(classify("*"), Err(OTHER_ELEMENTS));
        assert_eq!(classify("**"), Err(OTHER_ELEMENTS));
        // Capability queries.
        assert_eq!(classify("supports-[display:grid]"), Err(SUPPORTS));
        // An+B structural selectors (plain `nth-3` is the typed form).
        assert_eq!(classify("nth-[2n+1]"), Err(ANB));
        // Relative selectors outside has-*/group-*/peer-*.
        assert_eq!(classify("[>svg]"), Err(RELATIVE));
        // Descendant combinator (`& svg`): whitespace after `&` = other
        // elements.
        assert_eq!(classify("[& svg]"), Err(OTHER_ELEMENTS));
        // Class/id selectors: no typed equivalent.
        assert_eq!(classify("[.border-b]"), Err(CLASS_SEL));
        // Ancestor selectors.
        assert_eq!(classify("[a &]"), Err(ANCESTOR));
        // Bare `&`: adds no condition.
        assert_eq!(classify("[&]"), Err(BARE_AMP));
    }

    /// The self-test forms the classifier DOES express — pinned at the
    /// type level (these feed `has-*`/`not-*`/`group-*` inners and the
    /// snapshot IR).
    #[test]
    fn self_tests_classify_onto_the_typed_arms() {
        // Byte-identical arbitrary form ≡ the named variant (v0.3 §1.2):
        // same band, same test.
        assert_eq!(
            classify("[&:hover]"),
            Ok(vec![Atom::On(Test::State(State::Hover))])
        );
        // Implicit-self compound: `p:hover` (no `&` → `&:is(p:hover)`).
        assert_eq!(
            classify("[p:hover]"),
            Ok(vec![
                Atom::On(Test::Tag("p".into())),
                Atom::On(Test::State(State::Hover))
            ])
        );
        // data/aria attribute tests (the `&[attr]` compound form — the
        // bare `data-state=open` form reads `data-state` as a tag).
        assert_eq!(
            classify("[&[data-state=open]]"),
            Ok(vec![Atom::On(Test::Data(
                "state".into(),
                Some("open".into())
            ))])
        );
        assert_eq!(
            classify("[&[aria-expanded]]"),
            Ok(vec![Atom::On(Test::Aria("expanded".into(), None))])
        );
        // Structural pseudo-classes.
        assert_eq!(
            classify("[&:first-child]"),
            Ok(vec![Atom::On(Test::Structural(
                crate::ast::Structural::First
            ))])
        );
    }

    /// `single_test` unwraps exactly one self test; compound inners and
    /// nested negations are rejected (they cannot be expressed honestly).
    #[test]
    fn single_test_needs_exactly_one_self_test() {
        assert_eq!(single_test("[&:hover]"), Ok(Test::State(State::Hover)));
        // A compound is a conjunction — the inner model is a single test.
        assert_eq!(single_test("[p:hover]"), Err(COMPOUND_INNER));
        // Nested negation: `not-[&:not(:hover)]` would double-negate.
        assert_eq!(single_test("[&:not(:hover)]"), Err(NESTED_NOT));
        // The boundary errors propagate unchanged.
        assert_eq!(single_test("[& svg]"), Err(OTHER_ELEMENTS));
    }

    /// Structural Nth forms: `nth-3` is the typed `Structural::Nth(3)`; the
    /// arbitrary `[&:nth-child(3)]` form classifies onto the same arm.
    #[test]
    fn nth_forms_classify_to_the_structural_arm() {
        assert_eq!(
            classify("[&:nth-child(3)]"),
            Ok(vec![Atom::On(Test::Structural(
                crate::ast::Structural::Nth(3)
            ))])
        );
        assert_eq!(
            classify("[&:nth-last-child(2)]"),
            Ok(vec![Atom::On(Test::Structural(
                crate::ast::Structural::NthLast(2)
            ))])
        );
    }
}
