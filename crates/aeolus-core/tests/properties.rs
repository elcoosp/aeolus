//! Phase-4 property tests (proptest, dev-dependency only).
//!
//! Properties pinned:
//! 1. `cn()` is idempotent: `cn(cn(x)) == cn(x)`.
//! 2. Parser roundtrip: `parse(serialize(parse(x))) == parse(x)` — the
//!    grammar is re-entrant (serialize is the test-local debug writer
//!    below; aeolus-core deliberately has no serializer).
//! 3. OKLab ↔ sRGB roundtrip within tolerance (1e-3 per component, one
//!    8-bit sRGB step ≈ 4e-3 in Oklab, so 1e-3 is safely visual-nothing).
//! 4. Opacity modifiers preserve hue (v4 color-mix semantics).

use proptest::prelude::*;

use aeolus_core::Color;
use aeolus_core::ast::{ArbForm, ClassToken, ParentKind, Variant};
use aeolus_core::parser::parse_class_string;

/// Debug serializer for a parsed token (roundtrip oracle input).
fn serialize_token(t: &ClassToken<'_>) -> String {
    let mut s = String::new();
    for v in t.variants.iter() {
        write_variant(&mut s, v);
        s.push(':');
    }
    if t.negative {
        s.push('-');
    }
    s.push_str(t.body);
    if let Some(arb) = t.arbitrary {
        match arb.form {
            ArbForm::Bracket => {
                s.push('[');
                if let Some(h) = arb.hint {
                    s.push_str(h);
                    s.push(':');
                }
                s.push_str(arb.value);
                s.push(']');
            }
            ArbForm::Paren => {
                s.push('(');
                s.push_str(arb.value);
                s.push(')');
            }
        }
    }
    if let Some(m) = t.modifier {
        s.push('/');
        s.push_str(m);
    }
    if t.important {
        s.push('!');
    }
    s
}

fn write_variant(s: &mut String, v: &Variant<'_>) {
    match v {
        Variant::Named(n) => s.push_str(&n.canonical()),
        Variant::Grouped {
            parent,
            inner,
            label,
        } => {
            s.push_str(match parent {
                ParentKind::Group => "group-",
                ParentKind::Peer => "peer-",
            });
            write_variant(s, inner);
            if let Some(l) = label {
                s.push('/');
                s.push_str(l);
            }
        }
        Variant::Not(inner) => {
            s.push_str("not-");
            write_variant(s, inner);
        }
        Variant::Has(inner) => {
            s.push_str("has-");
            s.push_str(inner);
        }
        Variant::Attr { kind, name, value } => {
            s.push_str(match kind {
                aeolus_core::AttrKind::Data => "data-",
                aeolus_core::AttrKind::Aria => "aria-",
            });
            s.push_str(name);
            if let Some(v) = value {
                s.push_str(&format!("={v}"));
            }
        }
        Variant::Structural(st) => s.push_str(st.canonical()),
        Variant::PseudoElement(p) => s.push_str(p),
        Variant::Arbitrary(a) => s.push_str(a),
    }
}

fn word() -> impl Strategy<Value = String> {
    // Utility heads the parser supports, with plausible value shapes.
    r"(hover:|focus:|dark:|md:)?(bg|p|px|py|pt|pb|pl|pr|m|mx|w|h|gap|rounded|border|ring|text|font|leading|tracking|opacity|outline|underline-offset|pointer-events|select|whitespace|items|justify|self|flex|grow|shrink|cursor|shadow)(-[a-z0-9]+){0,3}(/[0-9]{1,3})?!?"
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    /// cn() idempotence: merging an already-merged list changes nothing.
    #[test]
    fn cn_is_idempotent(words in prop::collection::vec(word(), 0..24)) {
        let refs: Vec<String> = words;
        let once_refs: Vec<&str> = refs.iter().map(String::as_str).collect();
        let once = aeolus_core::cn(&once_refs);
        let twice_refs: Vec<&str> = once.split_ascii_whitespace().collect();
        let twice = aeolus_core::cn(&twice_refs);
        prop_assert_eq!(twice, once);
    }

    /// Parser roundtrip: serialize(parse(x)) re-parses to the same token.
    #[test]
    fn parser_roundtrip(token in word()) {
        let (tokens, errs) = parse_class_string(&token);
        prop_assert!(errs.is_empty(), "{token:?} must parse: {errs:?}");
        let t = &tokens[0];
        let s = serialize_token(t);
        let (tokens2, errs2) = parse_class_string(&s);
        prop_assert!(errs2.is_empty(), "reserialized {s:?} must parse: {errs2:?}");
        let t2 = &tokens2[0];
        prop_assert_eq!(t2.variants.as_slice(), t.variants.as_slice());
        prop_assert_eq!(t2.body, t.body);
        prop_assert_eq!(t2.arbitrary, t.arbitrary);
        prop_assert_eq!(t2.modifier, t.modifier);
        prop_assert_eq!(t2.negative, t.negative);
        prop_assert_eq!(t2.important, t.important);
    }

    /// OKLab ↔ sRGB roundtrip within 1e-3 (visual nothing).
    #[test]
    fn oklab_srgb_roundtrip(
        l in 0.0f32..=1.0,
        a in -0.5f32..=0.5,
        b in -0.5f32..=0.5,
        alpha in 0.0f32..=1.0,
    ) {
        let c = Color::oklab(l, a, b, alpha);
        let [r, g, bb, al] = c.to_srgba();
        prop_assert!(
            r.is_finite() && g.is_finite() && bb.is_finite() && al.is_finite(),
            "to_srgba must never produce NaN/inf"
        );
        let c2 = Color::from_srgb(r, g, bb, al);
        // In-gamut check: recompute the LINEAR sRGB components (the same
        // matrix linear_srgb_from_oklab applies); if none of them was
        // clamped, the conversion is lossless and the roundtrip is tight.
        // Out-of-gamut colors are clamped by design (lossy), so only
        // finiteness is asserted for them. This property caught the Phase-4
        // bug where oklab(0, 0, 0.274) produced NaN via powf on a negative
        // linear component.
        let linear_rgb = |c: Color| {
            let l_ = c.l + 0.396_337_78 * c.a + 0.215_803_76 * c.b;
            let m_ = c.l - 0.105_561_35 * c.a - 0.063_854_17 * c.b;
            let s_ = c.l - 0.089_484_18 * c.a - 1.291_485_6 * c.b;
            let (l3, m3, s3) = (l_ * l_ * l_, m_ * m_ * m_, s_ * s_ * s_);
            [
                4.076_741_7 * l3 - 3.307_711_6 * m3 + 0.230_969_94 * s3,
                -1.268_438 * l3 + 2.609_757_4 * m3 - 0.341_319_4 * s3,
                -0.004_196_08 * l3 - 0.703_418_6 * m3 + 1.707_614_7 * s3,
            ]
        };
        let [lr, lg, lb] = linear_rgb(c);
        let in_gamut = (0.0..=1.0).contains(&lr)
            && (0.0..=1.0).contains(&lg)
            && (0.0..=1.0).contains(&lb);
        if in_gamut {
            prop_assert!((c.l - c2.l).abs() < 1e-3);
            prop_assert!((c.a - c2.a).abs() < 1e-3);
            prop_assert!((c.b - c2.b).abs() < 1e-3);
            prop_assert!((c.alpha - c2.alpha).abs() < 1e-6, "alpha drift");
        }
    }

    /// Opacity modifiers preserve hue and lightness, scale chroma+alpha.
    #[test]
    fn opacity_preserves_hue(
        l in 0.05f32..=0.95,
        a in -0.3f32..=0.3,
        b in -0.3f32..=0.3,
        pct in 1.0f32..=99.0,
    ) {
        let c = Color::oklab(l, a, b, 1.0);
        let m = c.mix_with_transparent(pct);
        let hue = |c: Color| c.b.atan2(c.a);
        prop_assert!((hue(m) - hue(c)).abs() < 1e-4);
        prop_assert!((m.alpha - pct / 100.0).abs() < 1e-4);
        prop_assert!((m.l - c.l).abs() < 1e-6);
    }
}
