//! The theme-pass contracts, pinned end to end (architecture plan §3.4:
//! "pay only for the dynamism you use").
//!
//! Five families:
//! 1. **Sources** — the CSS-first construction through the audited v0.x
//!    reader: the embedded shadcn palettes, the dark variant propagation,
//!    typed parse failures.
//! 2. **The id contract** — token ids are sorted-name order, stable
//!    across rebuilds (they ride compiled `Program`s).
//! 3. **Tiers** — T0 literals, T1 const pairs, T2 runtime tokens, per
//!    the plan's §3.4 table.
//! 4. **The alpha identity** — the `/N` modifier is the premultiplied
//!    mix the golden corpus pins (`Color::mix_with_transparent`), clamped
//!    to the audited bounds, monotone in the modifier.
//! 5. **Typed refusals** — unknown ids, non-finite alphas and the alpha-
//!    on-a-pair-slot boundary are errors, never silent bakes.
//!
//! Written FIRST (the RED spec): the suite fails against a stub crate
//! and passes only once the tier model satisfies every pin below.

use aeolus_core::Color;
use aeolus_ir::Val;
use aeolus_theme::{ResolveError, ThemeError, ThemeSource, Tier, TokenTable};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// A tiny synthetic source: one light-only token (`brand`), one token
/// with a dark value (`surface`) — the tier model's controlled input.
fn mini_source() -> ThemeSource {
    ThemeSource::from_css(
        ":root { --color-brand: #336699; --color-surface: #eeeeee; }",
        Some(".dark { --color-surface: #111111; }"),
    )
    .expect("mini source parses")
}

// ---------------------------------------------------------------------------
// 1. Sources
// ---------------------------------------------------------------------------

#[test]
fn shadcn_source_builds_a_token_table() {
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    let table = TokenTable::runtime(&source).expect("default-seeded source builds");
    assert!(table.len() > 200, "the default-seeded palette alone is 270 tokens");
    for name in ["primary", "background", "red-500"] {
        assert!(table.id(name).is_some(), "{name} is a token");
    }
    assert!(source.dark().is_some(), "shadcn_dark carries the dark variant");
}

#[test]
fn parse_failures_are_typed_and_carry_the_source() {
    let err = ThemeSource::from_css(":root { --color-broken: calc(var(--never) + 1px); }", None)
        .expect_err("a var() that never resolves is a parse failure");
    assert!(matches!(err, ThemeError::Parse { .. }));
    assert!(err.to_string().contains("theme CSS"), "{err}");
}

#[test]
fn source_dark_variant_resolves_the_dark_overrides() {
    let source = mini_source();
    let light = source.light().color("surface").expect("light surface");
    let dark = source.dark().and_then(|theme| theme.color("surface")).expect("dark surface");
    assert_ne!(
        light.to_rgba8(),
        dark.to_rgba8(),
        "the .dark override re-resolved against the dark palette"
    );
}

// ---------------------------------------------------------------------------
// 2. The id contract
// ---------------------------------------------------------------------------

#[test]
fn ids_survive_rebuilds_of_the_same_source() {
    let source = mini_source();
    let first = TokenTable::runtime(&source).expect("builds");
    let second = TokenTable::baked(&source).expect("builds");
    for name in ["brand", "surface", "red-500"] {
        assert_eq!(first.id(name), second.id(name), "{name}'s id is tier-independent");
    }
    let id = first.id("brand").expect("brand exists");
    assert_eq!(first.name(id), Some("brand"), "name(id) inverts id(name)");
}

// ---------------------------------------------------------------------------
// 3. Tiers
// ---------------------------------------------------------------------------

#[test]
fn tokens_without_dark_bake_to_literals() {
    let source = mini_source();
    let table = TokenTable::baked(&source).expect("builds");
    assert_eq!(table.tier(), Tier::Baked);
    let id = table.id("brand").expect("brand exists");
    assert!(!table.has_dark(id), "brand is light-only in the mini source");
    let val = table.resolve(id, None).expect("light-only token resolves");
    assert!(matches!(val, Val::Rgba(_)), "T0 literal: {val:?}");
    assert!(!val.is_token(), "a baked literal is not a runtime token");
}

#[test]
fn tokens_with_dark_form_mode_pairs() {
    let source = mini_source();
    let table = TokenTable::baked(&source).expect("builds");
    let id = table.id("surface").expect("surface exists");
    assert!(table.has_dark(id), "surface carries a dark value");
    let val = table.resolve(id, None).expect("paired token resolves");
    let Val::ModePair(slot) = val else {
        panic!("T1 pair form expected, got {val:?}");
    };
    let (light, dark) = table.pair_at(slot).expect("the slot indexes the pair table");
    assert_eq!(light, table.light_rgba(id).expect("light bytes"));
    assert_eq!(dark, table.dark_rgba(id).expect("dark bytes"));
    assert!(val.is_token(), "the pair form is a runtime-tier value");
    assert!(slot < table.pairs() as u16, "the slot is inside the pair table");
}

#[test]
fn runtime_tier_carries_id_and_unit_alpha() {
    let source = mini_source();
    let table = TokenTable::runtime(&source).expect("builds");
    assert_eq!(table.tier(), Tier::Runtime);
    let id = table.id("surface").expect("surface exists");
    let val = table.resolve(id, None).expect("runtime token resolves");
    assert_eq!(val, Val::Tok { id, alpha: 1.0 }, "no modifier: alpha 1.0");
    assert!(val.is_token());
}

#[test]
fn runtime_alpha_is_carried_not_baked() {
    let source = mini_source();
    let table = TokenTable::runtime(&source).expect("builds");
    let id = table.id("brand").expect("brand exists");
    let val = table.resolve(id, Some(0.37)).expect("runtime alpha rides");
    let Val::Tok { id: tok_id, alpha } = val else {
        panic!("runtime form expected, got {val:?}");
    };
    assert_eq!(tok_id, id);
    assert!((alpha - 0.37).abs() < 1e-6, "the alpha rides the slot: {alpha}");
}

// ---------------------------------------------------------------------------
// 4. The alpha identity (the audited premultiplied mix)
// ---------------------------------------------------------------------------

#[test]
fn alpha_bakes_through_the_premultiplied_identity() {
    // The differential pin: the baked literal IS what the audited
    // oracle produces for the same token and modifier.
    let source = mini_source();
    let table = TokenTable::baked(&source).expect("builds");
    let id = table.id("brand").expect("brand exists");
    let expected = Color::parse("#336699")
        .expect("the mini token is a plain hex colour")
        .mix_with_transparent(50.0)
        .to_rgba8();
    assert_eq!(
        table.resolve(id, Some(0.5)).expect("bakes"),
        Val::Rgba(expected),
        "the /50 bake is the audited premultiplied mix"
    );
}

#[test]
fn alpha_bounds_are_the_audited_clamp() {
    let source = mini_source();
    let table = TokenTable::baked(&source).expect("builds");
    let id = table.id("brand").expect("brand exists");
    let plain = table.resolve(id, None).expect("resolves");
    let Val::Rgba(plain_bytes) = plain else {
        panic!("T0 literal expected, got {plain:?}");
    };
    // /0 is fully transparent: the alpha byte is zero, channels intact.
    let Val::Rgba(zero) = table.resolve(id, Some(0.0)).expect("resolves") else {
        panic!("T0 literal expected");
    };
    assert_eq!(zero[3], 0, "/0 scales the alpha byte to zero");
    assert_eq!(&zero[..3], &plain_bytes[..3], "channels are the token's own");
    // /100 and an over-range /250 clamp to the unmodified token.
    let Val::Rgba(full) = table.resolve(id, Some(1.0)).expect("resolves") else {
        panic!("T0 literal expected");
    };
    assert_eq!(full, plain_bytes, "/100 is the unmodified token");
    let Val::Rgba(clamped) = table.resolve(id, Some(2.5)).expect("resolves") else {
        panic!("T0 literal expected");
    };
    assert_eq!(clamped, plain_bytes, "the audited clamp caps at 100%");
}

#[test]
fn alpha_is_monotone_in_the_modifier() {
    let source = mini_source();
    let table = TokenTable::baked(&source).expect("builds");
    let id = table.id("brand").expect("brand exists");
    let mut last = 0u8;
    for pct in [10.0_f32, 25.0, 50.0, 75.0, 90.0] {
        let Val::Rgba(bytes) = table.resolve(id, Some(pct / 100.0)).expect("resolves") else {
            panic!("T0 literal expected at {pct}");
        };
        assert!(bytes[3] >= last, "alpha byte grows with the modifier: {pct}");
        last = bytes[3];
    }
}

// ---------------------------------------------------------------------------
// 5. Typed refusals
// ---------------------------------------------------------------------------

#[test]
fn unknown_names_and_ids_are_typed() {
    let source = mini_source();
    let table = TokenTable::runtime(&source).expect("builds");
    assert_eq!(table.id("not-a-token"), None, "unknown names are None");
    let err = table
        .resolve(9_999, None)
        .expect_err("an out-of-range id refuses");
    assert_eq!(err, ResolveError::UnknownId(9_999));
    assert!(err.to_string().contains("outside the table"), "{err}");
}

#[test]
fn non_finite_alphas_refuse() {
    let source = mini_source();
    let table = TokenTable::baked(&source).expect("builds");
    let id = table.id("brand").expect("brand exists");
    let err = table.resolve(id, Some(f32::NAN)).expect_err("NaN refuses");
    assert!(matches!(err, ResolveError::AlphaNotFinite(_)), "{err:?}");
    let err = table
        .resolve(id, Some(f32::INFINITY))
        .expect_err("inf refuses");
    assert!(matches!(err, ResolveError::AlphaNotFinite(_)), "{err:?}");
}

#[test]
fn alpha_on_a_baked_dark_pair_is_a_typed_boundary() {
    let source = mini_source();
    let table = TokenTable::baked(&source).expect("builds");
    let id = table.id("surface").expect("surface exists");
    assert!(table.has_dark(id));
    let err = table.resolve(id, Some(0.9)).expect_err("the pair form refuses alpha");
    assert_eq!(err, ResolveError::AlphaOnPair, "the pair slot has no alpha arm");
    // The runtime tier is the documented alternative: the alpha rides.
    let runtime = TokenTable::runtime(&source).expect("builds");
    let val = runtime.resolve(id, Some(0.9)).expect("the runtime tier carries it");
    assert!(matches!(val, Val::Tok { .. }), "{val:?}");
}

// ---------------------------------------------------------------------------
// Gated instrumentation contracts (the workspace's opt-in pattern)
// ---------------------------------------------------------------------------

#[cfg(feature = "miette")]
#[test]
fn theme_errors_carry_miette_code_and_help() {
    use miette::Diagnostic;
    let err = ThemeSource::from_css(":root { --color-broken: calc(var(--never) + 1px); }", None)
        .expect_err("parse failure");
    let code = err.code().map(|c| c.to_string());
    assert_eq!(code.as_deref(), Some("aeolus::theme::parse"));
    let help = err.help().expect("parse failures carry help").to_string();
    assert!(help.contains("@theme"), "{help}");

    let source = mini_source();
    let table = TokenTable::baked(&source).expect("builds");
    let id = table.id("surface").expect("surface exists");
    let err = table.resolve(id, Some(0.9)).expect_err("alpha on pair");
    let code = err.code().map(|c| c.to_string());
    assert_eq!(code.as_deref(), Some("aeolus::theme::alpha_on_pair"));
    let help = err.help().expect("the boundary carries help").to_string();
    assert!(help.contains("Runtime"), "{help}");
}

#[cfg(feature = "tracing")]
#[test]
fn building_and_resolving_under_tracing_keeps_the_contract() {
    // No global subscriber is installed: the spans compile and are
    // no-ops. The observable contract must be unchanged.
    let source = mini_source();
    let table = TokenTable::baked(&source).expect("builds");
    let id = table.id("surface").expect("surface exists");
    let val = table.resolve(id, None).expect("resolves");
    assert!(matches!(val, Val::ModePair(_)), "{val:?}");
}

#[test]
fn pair_overflow_refuses_with_its_own_typed_variant() {
    // The T1 budget refusal is a BUILD boundary, not a parse failure:
    // it carries its own `ThemeError::PairOverflow` variant instead of
    // smuggling a fabricated `ParseError` through the parse arm (the
    // crate's own contract: parse errors are carried verbatim, never
    // re-wrapped). The build path itself is practically unreachable
    // (>65535 paired tokens), so the pin is the typed surface: the
    // always-on Display names the boundary and the count.
    let err = ThemeError::PairOverflow { pairs: 65_536 };
    let text = err.to_string();
    assert!(
        text.contains("65536") && text.contains("u16 ModePair"),
        "the Display names the budget and the count: {text}"
    );
    // And the `std::error::Error` chain stays source-less: there is no
    // parse failure to delegate to — the boundary IS the failure.
    assert!(std::error::Error::source(&err).is_none());
}

#[cfg(feature = "miette")]
#[test]
fn pair_overflow_carries_miette_code_and_help() {
    use miette::Diagnostic;
    let err = ThemeError::PairOverflow { pairs: 65_536 };
    let code = err.code().map(|c| c.to_string());
    assert_eq!(code.as_deref(), Some("aeolus::theme::pair_overflow"));
    let help = err.help().expect("the boundary carries help").to_string();
    assert!(help.contains("Runtime"), "{help}");
}
