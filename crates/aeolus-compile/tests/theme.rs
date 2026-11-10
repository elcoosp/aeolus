//! The theme-seam contracts, pinned end to end (architecture plan
//! §3.2's signature gaining its theme half, §3.4's tiers).
//!
//! Four families:
//! 1. **The theme half** — colour tokens lower through the `TokenTable`
//!    (`bg-red-500` resolves, the boundary the P2 slice 1 pinned), and
//!    the `/opacity` modifier rides colour-form values (the audited
//!    premultiplied identity).
//! 2. **Tiers through the seam** — the runtime tier carries
//!    `Tok { id, alpha }`; the baked tier emits literals and dark
//!    `ModePair`s, and an alpha on a pair stays the typed boundary.
//! 3. **The silent path is unchanged** — without a table, every
//!    v0.10.0 pin holds byte-for-byte (the theme-less default is the
//!    honest boundary, not a regression surface).
//! 4. **The finite guard** — the f32 grammar's `nan`/`inf` spellings
//!    are typed refusals, never non-finite literals in the IR (the
//!    micro-debt cleanup this slice ships).
//!
//! Written FIRST (the RED spec): the suite fails against the v0.10.0
//! seam and passes only once `compile_theme` satisfies every pin below.

use aeolus_compile::{CompileError, Config, DiagnosticKind, Spacing, compile, compile_theme};
use aeolus_ir::{Cond, Decl, Prop, Val};
use aeolus_theme::{ThemeSource, TokenTable};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// The base layer's declarations (layer 0 of a compiled program).
fn base(compiled: &aeolus_compile::Compiled) -> &[Decl] {
    compiled
        .program
        .layers
        .first()
        .map_or(&[], |layer| layer.decls)
}

/// The runtime-tier table over the shadcn light+dark source.
fn runtime_table() -> TokenTable {
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    TokenTable::runtime(&source).expect("default-seeded source builds")
}

/// The baked-tier table over the same source.
fn baked_table() -> TokenTable {
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    TokenTable::baked(&source).expect("default-seeded source builds")
}

/// The table over a source with NO shadcn overrides — the v4 default
/// seed only, so shadcn-app token names (`primary`) are shape-legal in
/// the registry but absent from THIS table.
fn bare_table() -> TokenTable {
    let source = ThemeSource::from_css("/* the v4 default seed only */", None)
        .expect("the default seed parses");
    TokenTable::runtime(&source).expect("default-seeded source builds")
}

// ---------------------------------------------------------------------------
// 1. The theme half
// ---------------------------------------------------------------------------

#[test]
fn colour_tokens_lower_through_the_runtime_table() {
    // The boundary the P2 slice 1 pinned (`colour_tokens_report_the_
    // theme_pass`) resolves here: `bg-red-500` is a token now.
    let table = runtime_table();
    let compiled =
        compile_theme("bg-red-500 text-primary p-2", &table, &Config::default()).expect("compiles");
    assert!(
        compiled.diagnostics.is_empty(),
        "no diagnostics: {compiled:?}"
    );
    let decls = base(&compiled);
    let red = table.id("red-500").expect("red-500 is a palette token");
    let primary = table.id("primary").expect("primary is a shadcn token");
    assert!(
        decls.contains(&Decl {
            prop: Prop::Background,
            val: Val::Tok {
                id: red,
                alpha: 1.0
            }
        }),
        "bg-red-500 → Tok {{ id, alpha: 1.0 }}: {decls:?}"
    );
    assert!(
        decls.contains(&Decl {
            prop: Prop::Color,
            val: Val::Tok {
                id: primary,
                alpha: 1.0
            }
        }),
        "text-primary → the primary token: {decls:?}"
    );
    assert!(
        decls.contains(&Decl {
            prop: Prop::PaddingTop,
            val: Val::Rem(0.5)
        }),
        "p-2 lowers unchanged beside the tokens"
    );
}

#[test]
fn alpha_modifier_becomes_the_token_alpha() {
    // The plan §2 identity: opacity on a known token is one multiply —
    // here it rides the Tok slot, the host applies it per frame.
    let table = runtime_table();
    let compiled = compile_theme("bg-primary/90", &table, &Config::default()).expect("compiles");
    assert!(
        compiled.diagnostics.is_empty(),
        "the modifier rode: {compiled:?}"
    );
    let primary = table.id("primary").expect("primary exists");
    let decls = base(&compiled);
    let Some(&Decl {
        prop: Prop::Background,
        val: Val::Tok { id, alpha },
    }) = decls.first()
    else {
        panic!("a single token decl expected, got {decls:?}");
    };
    assert_eq!(id, primary);
    assert!(
        (alpha - 0.9).abs() < 1e-6,
        "/90 normalizes to the 0.9 fraction: {alpha}"
    );
}

#[test]
fn baked_alpha_uses_the_premultiplied_bake() {
    // The differential pin: the seam's baked literal IS the table's
    // own resolve — the audited oracle underneath
    // (Color::mix_with_transparent, corpus-pinned).
    let table = baked_table();
    let compiled = compile_theme("bg-red-500/50", &table, &Config::default()).expect("compiles");
    assert!(compiled.diagnostics.is_empty(), "{compiled:?}");
    let red = table.id("red-500").expect("red-500 is light-only");
    let Val::Rgba(expected) = table.resolve(red, Some(0.5)).expect("bakes") else {
        panic!("the baked tier resolves a literal");
    };
    let decls = base(&compiled);
    assert_eq!(
        decls.first(),
        Some(&Decl {
            prop: Prop::Background,
            val: Val::Rgba(expected)
        }),
        "the /50 bake is the table's premultiplied mix"
    );
}

// ---------------------------------------------------------------------------
// 2. Tiers through the seam
// ---------------------------------------------------------------------------

#[test]
fn baked_tier_literals_and_dark_pairs() {
    let table = baked_table();
    let compiled =
        compile_theme("bg-red-500 dark:bg-primary", &table, &Config::default()).expect("compiles");
    assert!(compiled.diagnostics.is_empty(), "{compiled:?}");
    let red = table.id("red-500").expect("red-500 exists");
    let primary = table.id("primary").expect("primary exists");
    let base_decls = base(&compiled);
    assert!(
        base_decls.contains(&Decl {
            prop: Prop::Background,
            val: Val::Rgba(table.light_rgba(red).expect("bytes"))
        }),
        "the light-only token bakes to a literal: {base_decls:?}"
    );
    let dark_layer = compiled
        .program
        .layers
        .iter()
        .find(|layer| layer.cond == Cond::Dark)
        .expect("the dark variant forms its own layer");
    let Some(&Decl {
        prop: Prop::Background,
        val: Val::ModePair(slot),
    }) = dark_layer.decls.first()
    else {
        panic!("a ModePair expected, got {:?}", dark_layer.decls);
    };
    let (light, dark) = table
        .pair_at(slot)
        .expect("the slot indexes the pair table");
    assert_eq!(light, table.light_rgba(primary).expect("light bytes"));
    assert_eq!(dark, table.dark_rgba(primary).expect("dark bytes"));
}

#[test]
fn alpha_on_a_baked_dark_pair_is_typed() {
    // The pair form has no alpha arm: the typed boundary, with the raw
    // modifier text on the diagnostic. The runtime tier is the answer.
    let table = baked_table();
    let compiled =
        compile_theme("dark:bg-primary/90", &table, &Config::default()).expect("compiles");
    assert_eq!(compiled.diagnostics.len(), 1, "{compiled:?}");
    assert_eq!(
        compiled.diagnostics[0].kind,
        DiagnosticKind::UnsupportedModifier {
            modifier: "90".to_string()
        }
    );
    assert!(compiled.diagnostics[0].kind.help().contains("theme pass"));
    assert!(
        compiled.program.layers.is_empty(),
        "nothing lowered for the candidate"
    );
}

#[test]
fn modifier_on_a_non_colour_lowering_is_still_typed() {
    // `text-sm/6`'s modifier is a LINE-HEIGHT number, not an opacity:
    // it stays the codegen's business. `bg-primary/[nan]` is an
    // unparseable modifier — the same typed boundary, with or without
    // the theme pass.
    let table = runtime_table();
    let compiled = compile_theme("text-sm/6 p-4", &table, &Config::default()).expect("compiles");
    assert_eq!(compiled.diagnostics.len(), 1, "{compiled:?}");
    assert_eq!(
        compiled.diagnostics[0].kind,
        DiagnosticKind::UnsupportedModifier {
            modifier: "6".to_string()
        }
    );
    assert!(
        base(&compiled)
            .iter()
            .any(|decl| decl.prop == Prop::PaddingTop),
        "p-4 still lowers beside the report"
    );

    let compiled = compile_theme("bg-primary/[nan]", &table, &Config::default()).expect("compiles");
    assert_eq!(compiled.diagnostics.len(), 1, "{compiled:?}");
    assert!(matches!(
        &compiled.diagnostics[0].kind,
        DiagnosticKind::UnsupportedModifier { modifier } if modifier == "[nan]"
    ));
}

#[test]
fn unknown_colour_names_report_the_token_table() {
    // Shape-legal but table-absent: `primary` is in the registry's
    // colour keyword table (so the shape contract accepts it) but the
    // bare-seed table has no such token — the not-a-token miss, whose
    // reason names the checklist.
    let table = bare_table();
    let compiled = compile_theme("bg-primary p-2", &table, &Config::default()).expect("compiles");
    assert_eq!(compiled.diagnostics.len(), 1, "{compiled:?}");
    let d = &compiled.diagnostics[0];
    assert!(
        matches!(
            &d.kind,
            DiagnosticKind::UnsupportedValue { reason, .. } if reason.contains("TokenTable")
        ),
        "the reason names the checklist: {:?}",
        d.kind
    );
    assert_eq!(d.kind.code(), "aeolus::compile::unsupported_value");
    assert_eq!(d.span, (0, 10), "the candidate's own span");
    assert!(
        base(&compiled)
            .iter()
            .any(|decl| decl.prop == Prop::PaddingTop),
        "p-2 still lowers beside the report"
    );
}

#[test]
fn suffixes_outside_the_registry_stay_lookup_misses() {
    // A name the REGISTRY does not carry either never reaches the value
    // pass: the lookup's own typed miss (ShapeRejected) is the report —
    // the theme pass does not shadow the registry read side.
    let table = runtime_table();
    let compiled =
        compile_theme("bg-not-a-token p-2", &table, &Config::default()).expect("compiles");
    assert_eq!(compiled.diagnostics.len(), 1, "{compiled:?}");
    assert!(matches!(
        &compiled.diagnostics[0].kind,
        DiagnosticKind::ShapeRejected { root: "bg", .. }
    ));
    assert!(
        base(&compiled)
            .iter()
            .any(|decl| decl.prop == Prop::PaddingTop)
    );
}

// ---------------------------------------------------------------------------
// 3. The silent path is unchanged (the v0.10.0 pins, byte for byte)
// ---------------------------------------------------------------------------

#[test]
fn the_silent_path_is_unchanged_without_a_theme() {
    // Colour tokens: the P2 slice 1 boundary, verbatim.
    let compiled = compile("bg-red-500", &Config::default()).expect("compiles");
    assert_eq!(compiled.diagnostics.len(), 1);
    assert!(matches!(
        &compiled.diagnostics[0].kind,
        DiagnosticKind::UnsupportedValue { reason, .. } if reason.contains("theme pass")
    ));
    // Modifiers: the whole modifier is the boundary without a table,
    // even a parseable one.
    let compiled = compile("bg-red-500/50", &Config::default()).expect("compiles");
    assert_eq!(compiled.diagnostics.len(), 1);
    assert_eq!(
        compiled.diagnostics[0].kind,
        DiagnosticKind::UnsupportedModifier {
            modifier: "50".to_string()
        }
    );
    assert!(compiled.diagnostics[0].kind.help().contains("theme pass"));
    // The px config and the empty-input error are untouched.
    let config = Config {
        spacing: Spacing::Px,
        ..Config::default()
    };
    assert_eq!(
        base(&compile("p-2", &config).expect("compiles"))[0],
        Decl {
            prop: Prop::PaddingTop,
            val: Val::Px(8.0)
        }
    );
    assert!(matches!(
        compile("", &Config::default()),
        Err(CompileError::Empty)
    ));
}

// ---------------------------------------------------------------------------
// 4. The finite guard (the micro-debt cleanup this slice ships)
// ---------------------------------------------------------------------------

#[test]
fn non_finite_suffixes_are_typed_not_lowered() {
    // `f32::from_str` accepts `nan`/`inf`/`1e40` — legal parses, illegal
    // v4 values. Before the guard they lowered SILENTLY into the IR
    // (`Val::Rem(NaN)`, `Val::Num(inf)`); now they are typed refusals
    // and the rest of the string still compiles.
    let compiled = compile("p-nan z-inf w-1e40 p-2", &Config::default()).expect("compiles");
    assert_eq!(compiled.diagnostics.len(), 3, "{compiled:?}");
    for d in &compiled.diagnostics {
        assert!(
            matches!(
                &d.kind,
                DiagnosticKind::UnsupportedValue { reason, .. } if reason.contains("non-finite")
            ),
            "each miss names the finite guard: {:?}",
            d.kind
        );
    }
    let decls = base(&compiled);
    assert!(
        decls.contains(&Decl {
            prop: Prop::PaddingTop,
            val: Val::Rem(0.5)
        }),
        "p-2 still lowers: {decls:?}"
    );
    assert!(
        !decls.iter().any(|decl| matches!(
            decl.val,
            Val::Rem(v) | Val::Num(v) | Val::Pct(v) | Val::Px(v) if !v.is_finite()
        )),
        "no non-finite value reaches the IR"
    );
}

#[test]
fn negated_and_percent_non_finite_forms_are_typed() {
    // The guard covers every numeric arm: spacing (incl. the negated
    // form) and percents.
    let compiled = compile("-mt-nan from-nan% p-2", &Config::default()).expect("compiles");
    let finite_misses = compiled
        .diagnostics
        .iter()
        .filter(|d| matches!(&d.kind, DiagnosticKind::UnsupportedValue { reason, .. } if reason.contains("non-finite")))
        .count();
    assert_eq!(
        finite_misses, 2,
        "both non-finite forms refuse: {compiled:?}"
    );
    // And the px bake: a finite parse whose PRODUCT overflows against
    // a huge `rem_px` refuses too.
    let config = Config {
        spacing: Spacing::Px,
        rem_px: 1.0e30,
    };
    let compiled = compile("p-4000000000000 p-2", &config).expect("compiles");
    assert!(
        compiled.diagnostics.iter().any(|d| matches!(
            &d.kind,
            DiagnosticKind::UnsupportedValue { reason, .. } if reason.contains("non-finite")
        )),
        "the px bake overflow refuses: {compiled:?}"
    );
    assert!(
        !base(&compiled).iter().any(|decl| matches!(decl.val,
            Val::Px(v) | Val::Rem(v) if !v.is_finite())),
        "no overflowed px bake reaches the IR"
    );
}

// ---------------------------------------------------------------------------
// Gated instrumentation contracts (the workspace's opt-in pattern)
// ---------------------------------------------------------------------------

#[cfg(feature = "tracing")]
#[test]
fn compile_theme_spans_keep_the_contract() {
    // No global subscriber is installed: the spans compile and are
    // no-ops. The observable contract must be unchanged.
    let table = runtime_table();
    let compiled =
        compile_theme("bg-primary/90 p-4", &table, &Config::default()).expect("compiles");
    assert_eq!(compiled.program.layers.len(), 1);
    assert!(compiled.diagnostics.is_empty());
}
