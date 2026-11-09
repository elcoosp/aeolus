//! The differential harness: the V1 lexer against the v0.x parser, per
//! candidate, over a tricky battery AND every golden-corpus class.
//!
//! The v0.x `aeolus-core::parser` stays the grammar oracle until the
//! registry-as-data pipeline replaces it (plan §5.2 "differential
//! testing triangle"): for every candidate both parsers must agree on
//! the failure/Success split, the variant segmentation, negation,
//! importance, body, arbitrary form/hint/value and raw modifier.
//!
//! One documented divergence: the v0.x parser caps variants (8) and
//! elements (10) in stack-inline arrays; the V1 lexer has no inline
//! caps. Core-only failures starting with `too many` are skipped.

use aeolus_core::ast::{ArbForm, ClassToken, ParentKind, Variant};
use aeolus_core::parser::{
    parse_class_string as core_parse_string, parse_token as core_parse_token, parse_variant,
};
use aeolus_syntax::parse_candidate as v1_parse_candidate;
use aeolus_syntax::parse_class_string as v1_parse_string;

/// The tricky battery: every scanner corner the grammar has.
const BATTERY: &[&str] = &[
    "p-4",
    "bg-primary",
    "hover:bg-primary/90",
    "dark:hover:bg-white",
    "md:p-6",
    "@md:w-full",
    "-mt-4",
    "-mx-0.5",
    "!p-4",
    "p-4!",
    "w-[calc(100%/3)]",
    "w-[calc(100%_-_2rem)]",
    "bg-[#fff]/50",
    "bg-[#ff880066]",
    "w-[length:200px]",
    "text-[color:var(--brand)]",
    "bg-(--my-var)",
    "ask-type:luminance]",
    "[&:hover]:underline",
    "group-hover/menu:opacity-70",
    "peer-focus/name:ring-2",
    "not-hover:underline",
    "has-[:focus]:ring-2",
    "has-data-[state=open]:flex",
    "first:pt-0",
    "nth-3:visible",
    "sm:max-w-lg",
    "text-sm/6",
    "text-sm/tight",
    "bg-red-500/(--o)",
    "selection:bg-yellow-200",
    "hover:focus:active:outline-none",
    "rounded-t-lg",
    "-tracking-widest",
    "supports-[display:grid]:flex",
    "*:p-4",
    "**:p-4",
    "p--4",
    "w-[",
    "hover:",
    "::p-4",
    "hover::p-4",
    "w-[]",
    "w-[4px]x",
    "w-1/2/3",
    "underline-offset-4",
    "bg-linear-to-r",
    "truncate",
    "tabular-nums",
    "w-auto",
    "grid-cols-[repeat(3,minmax(0,1fr))]",
    "@container/card-header",
    "[@media(print)]:hidden",
    "nth-last-2:visible",
    "text-lg/loose",
    "bg-red-500!",
    "hover:!p-4",
    "-translate-y-1",
    "ring-ring/50",
];

/// The golden corpus' classes (the differential's second leg).
fn corpus_classes() -> Vec<String> {
    let raw = include_str!("../../aeolus-core/tests/fixtures/golden.corpus");
    raw.lines()
        .filter_map(|line| line.strip_prefix("@entry "))
        .filter_map(|rest| rest.split_once(' '))
        .map(|(_, class)| class.to_string())
        .collect()
}

/// Render a core variant in its CANONICAL text — the comparison basis.
/// Both sides of the differential are compared in this space: the v0.x
/// canonicalizer normalizes bracket spellings (`data-[state=open]` →
/// `data-state=open`) and folds structural families (`nth-3` → `nth`),
/// so the V1 lexer's raw segments are pushed through the SAME
/// canonicalizer (see `v1_variants_canonical`) before the compare.
fn variant_canonical(v: &Variant<'_>) -> String {
    match v {
        Variant::Named(n) => n.canonical(),
        Variant::Grouped {
            parent,
            inner,
            label,
        } => {
            let mut s = String::new();
            s.push_str(match parent {
                ParentKind::Group => "group-",
                ParentKind::Peer => "peer-",
            });
            s.push_str(&variant_canonical(inner));
            if let Some(l) = label {
                s.push('/');
                s.push_str(l);
            }
            s
        }
        Variant::Not(inner) => format!("not-{}", variant_canonical(inner)),
        Variant::Has(target) => format!("has-{target}"),
        Variant::Attr { kind, name, value } => {
            let mut s = String::new();
            s.push_str(match kind {
                aeolus_core::ast::AttrKind::Data => "data-",
                aeolus_core::ast::AttrKind::Aria => "aria-",
            });
            s.push_str(name);
            if let Some(v) = value {
                s.push('=');
                s.push_str(v);
            }
            s
        }
        Variant::Structural(st) => st.canonical().to_string(),
        Variant::PseudoElement(p) => (*p).to_string(),
        Variant::Arbitrary(a) => (*a).to_string(),
    }
}

/// The V1 raw segments, canonicalized through the oracle's own
/// `parse_variant` — one canonicalizer for both sides.
fn v1_variants_canonical(segments: &[&str]) -> Vec<String> {
    segments
        .iter()
        .map(|seg| {
            let v = parse_variant(seg)
                .unwrap_or_else(|e| panic!("v1 segment {seg:?} must re-parse: {}", e.message));
            variant_canonical(&v)
        })
        .collect()
}

/// The comparison shape both parsers must produce for one token.
#[derive(Debug, PartialEq)]
struct Shape {
    variants: Vec<String>,
    body: String,
    arbitrary: Option<(Option<String>, String, ArbForm, bool)>,
    modifier: Option<String>,
    negative: bool,
    important: bool,
}

fn core_shape(t: &ClassToken<'_>) -> Shape {
    Shape {
        variants: t.variants.iter().map(variant_canonical).collect(),
        body: t.body.to_string(),
        arbitrary: t.arbitrary.map(|a| {
            (
                a.hint.map(str::to_string),
                a.value.to_string(),
                a.form,
                a.is_property,
            )
        }),
        modifier: t.modifier.map(str::to_string),
        negative: t.negative,
        important: t.important,
    }
}

fn v1_shape(c: &aeolus_syntax::Candidate<'_>) -> Shape {
    Shape {
        variants: v1_variants_canonical(&c.variants),
        body: c.body.to_string(),
        arbitrary: c.arbitrary.map(|a| {
            (
                a.hint.map(str::to_string),
                a.value.to_string(),
                match a.form {
                    aeolus_syntax::Form::Bracket => ArbForm::Bracket,
                    aeolus_syntax::Form::Paren => ArbForm::Paren,
                },
                a.is_property,
            )
        }),
        modifier: c.modifier.map(str::to_string),
        negative: c.negative,
        important: c.important,
    }
}

#[test]
fn battery_agrees_with_the_oracle() {
    for raw in BATTERY {
        let core = core_parse_token(raw);
        let v1 = v1_parse_candidate(raw);
        match (core, v1) {
            (Ok(t), Ok(c)) => {
                assert_eq!(core_shape(&t), v1_shape(&c), "shape divergence on {raw:?}");
            }
            (Err(core_err), Err(v1_err)) => {
                // Both reject — same honest per-class failure.
                let core_msg = core_err.message;
                assert!(
                    !core_msg.starts_with("too many"),
                    "core-only cap rejection on {raw:?}: {core_msg}"
                );
                assert!(
                    !v1_err.message().is_empty(),
                    "v1 error text empty on {raw:?}"
                );
            }
            (Ok(_), Err(e)) => panic!("v1 rejects what the oracle accepts: {raw:?} ({e})"),
            (Err(e), Ok(_)) => panic!("v1 accepts what the oracle rejects: {raw:?} ({e})"),
        }
    }
}

#[test]
fn corpus_agrees_with_the_oracle() {
    for class in corpus_classes() {
        // Per-class splits — the string parser's honest degradation.
        let (core_tokens, core_errors) = core_parse_string(class.trim());
        let (v1_tokens, v1_errors) = v1_parse_string(class.trim());
        // The ok/err SPLIT must agree, skipping the core's inline caps.
        for (raw, msg) in &core_errors {
            if msg.starts_with(raw) && core_is_cap(msg, raw) {
                continue;
            }
            assert!(
                v1_errors.iter().any(|(r, _)| r == raw),
                "core rejects {raw:?} ({msg}) but v1 accepts it (class {class:?})"
            );
        }
        for (raw, _) in &v1_errors {
            assert!(
                core_errors.iter().any(|(r, _)| r == raw)
                    || core_tokens.iter().any(|t| t.raw == raw),
                "v1 rejects {raw:?} but the oracle never saw it (class {class:?})"
            );
        }
        // Every token both accept must carry the same shape.
        assert_eq!(
            core_tokens.len(),
            v1_tokens.len(),
            "token count on {class:?}"
        );
        for (t, c) in core_tokens.iter().zip(v1_tokens.iter()) {
            assert_eq!(core_shape(t), v1_shape(c), "shape divergence in {class:?}");
        }
    }
}

/// Is this core error one of the documented inline-cap rejections?
fn core_is_cap(msg: &str, raw: &str) -> bool {
    (msg.contains("too many variants") || msg.contains("too many utility elements"))
        && !msg.starts_with(raw)
        || msg.starts_with(raw) && msg.contains("too many")
}

#[test]
fn class_string_spans_index_the_input() {
    let input = "  p-4   hover:bg-red-500\n\ttruncate ";
    let (tokens, errors) = v1_parse_string(input);
    assert!(errors.is_empty());
    for t in &tokens {
        let (start, end) = t.span;
        assert_eq!(&input[start..end], t.raw, "span must round-trip {t:?}");
        assert!(!t.raw.is_empty());
    }
    // Three tokens, in source order.
    assert_eq!(
        tokens.iter().map(|t| t.raw).collect::<Vec<_>>(),
        vec!["p-4", "hover:bg-red-500", "truncate"]
    );
}

#[test]
fn the_flagship_examples_agree() {
    // The plan §1 examples that motivated the V1 revamp.
    for raw in ["dark:group-hover:bg-primary/90!", "hover:px-8", "w-[32px]"] {
        let t = core_parse_token(raw).expect("oracle parses");
        let c = v1_parse_candidate(raw).expect("v1 parses");
        assert_eq!(core_shape(&t), v1_shape(&c), "on {raw}");
    }
}
