//! The metamorphic seam's contracts (architecture plan §5.3's fuzz
//! line — the `compile ⇒ interpret ⇒ equals emit` target the README
//! promised): the three pipeline walks agree, or the report says
//! exactly where and why they do not.
//!
//! The invariant, in the shape the plan's differential triangle means
//! it: interpreting the compiled [`Program`] under a probe [`Env`]
//! yields the same effective declarations the CSS emit arm renders for
//! the situations that env fires — cascade resolved (later source
//! order wins per longhand on BOTH sides), theme dynamism included,
//! every documented divergence a typed note rather than a lie.
//!
//! The pins, in the order the crate docs promise them:
//!
//! * the two diagnostics channels are byte-equal over the battery (the
//!   compile walk and the CSS walk share the typed lexer channel, the
//!   variant refusal set, the registry read side and the value pass's
//!   boundaries — a drift is a bug in one of the three);
//! * the whole-string sheet is the composition of the per-candidate
//!   sheets (the arm is per-candidate; the parts add up to the whole);
//! * the base situation agrees declaration-for-declaration (`p-4
//!   flex-col` → the same five texts both sides), and the cascade
//!   across same-situation candidates is last-wins on both sides
//!   (`p-4 p-8` → `2rem` everywhere);
//! * probe situations agree under the conjunctive env: hover at
//!   `hover: true`, breakpoints AT their rem, stacked variants under
//!   the env every conjunct holds (the IR's dominant-fold divergence
//!   pinned as the typed note, never a silent mismatch);
//! * pseudo-element layers are separate render targets: out of the
//!   element cascade both sides, the note carried;
//! * theme dynamism agrees in both modes (baked pairs answer the
//!   light/dark halves; runtime tokens resolve to concrete bytes while
//!   the CSS keeps the `var()` reference — counted as the typed
//!   `TokenDynamic` notes, never a text lie);
//! * the `/opacity` modifier identity holds through the checker (the
//!   §2 premultiplied bake: interp bytes, CSS `rgb(r g b / N%)` — one
//!   value, two renderings);
//! * the checker is deterministic and pure, the empty input still the
//!   one whole-string error, and the custom-authoring seam rides the
//!   same walk.
//!
//! Written FIRST (the RED spec): the suite fails against the 0.19.0
//! seam and passes only once the metamorphic checker satisfies every
//! pin below.

use aeolus_compile::{
    CompileError, Config, CustomForm, CustomUtility, CustomVariant, Customs, MetamorphicNote,
    metamorphic_check, metamorphic_check_theme, metamorphic_check_theme_with,
};
use aeolus_ir::{Bp, Cond, Prop};
use aeolus_theme::{ThemeSource, TokenTable};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// The baked-tier table over the shadcn light+dark source (the T0/T1
/// surface: literals and const pairs).
fn baked_table() -> TokenTable {
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    TokenTable::baked(&source).expect("default-seeded source builds")
}

/// The runtime-tier table over the same source (the T2 surface: the
/// `var()` references the CSS keeps dynamic).
fn runtime_table() -> TokenTable {
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    TokenTable::runtime(&source).expect("default-seeded source builds")
}

/// A two-token host source with a baked light/dark pair — the minimal
/// T1 fixture (`--color-brand` overridden under `.dark`).
fn pair_table() -> TokenTable {
    let light = "@theme {\n  --color-brand: #0ea5e9;\n}\n";
    let dark = ".dark {\n  --color-brand: #0284c7;\n}\n";
    let source = ThemeSource::from_css(light, Some(dark)).expect("host CSS parses");
    TokenTable::baked(&source).expect("pair surface builds")
}

/// The strings the diagnostics-equality battery runs over: lowered
/// forms, honest refusals, boundary shapes — the two channels must
/// report IDENTICALLY on all of them.
const BATTERY: [&str; 16] = [
    "p-4",
    "flex-col",
    "p-4 flex-col text-sm",
    "hover:p-8",
    "md:hidden",
    "dark:bg-primary/90",
    "p-4 hover:px-8 md:p-6",
    "w-1/2 -mt-2",
    "z-10 duration-500",
    "not-a-root",
    "hover:not-a-root",
    "p-4 unknown:root-again",
    "text-[13px]",
    "bg-[#0f172a]",
    "rounded-[9999px]",
    "grid-cols-[repeat(2,minmax(0,1fr))]",
];

// ---------------------------------------------------------------------------
// The diagnostics channels agree
// ---------------------------------------------------------------------------

#[test]
fn diagnostics_channels_agree_over_the_battery() {
    for input in BATTERY {
        let report =
            metamorphic_check(input, &Config::default()).expect("battery strings are non-empty");
        assert!(
            report.diagnostics_equal,
            "diagnostics drifted on {input:?}: compile {:#?} vs css {:#?}",
            {
                let compiled =
                    aeolus_compile::compile(input, &Config::default()).expect("non-empty");
                compiled.diagnostics
            },
            {
                let sheet = aeolus_compile::emit_css(input, &Config::default()).expect("non-empty");
                sheet.diagnostics
            },
        );
        assert!(report.agree(), "the battery string {input:?} must agree");
    }
}

// ---------------------------------------------------------------------------
// The base situation agrees, cascade included
// ---------------------------------------------------------------------------

#[test]
fn base_situation_agrees_decl_for_decl() {
    let report = metamorphic_check("p-4 flex-col", &Config::default()).expect("non-empty");
    assert!(report.agree());
    let base = report
        .situations
        .iter()
        .find(|s| s.label == "base")
        .expect("the base situation is walked");
    assert_eq!(base.interp_decls, 5, "four paddings + the flex direction");
    assert_eq!(base.css_decls, 5, "the CSS sheet carries the same five");
    assert!(base.mismatches.is_empty());
}

#[test]
fn cascade_last_wins_across_candidates() {
    let report = metamorphic_check("p-4 p-8", &Config::default()).expect("non-empty");
    assert!(report.agree());
    let base = &report.situations[0];
    // Both sides resolve the cascade: `p-8` (source-last) wins per
    // longhand; the agreement holds because BOTH walks are last-wins.
    assert_eq!(base.interp_decls, 4);
    assert_eq!(base.css_decls, 4);
    assert!(base.mismatches.is_empty());
}

#[test]
fn probe_situations_agree_under_the_conjunctive_env() {
    let report = metamorphic_check("p-4 hover:px-8 md:p-6", &Config::default()).expect("non-empty");
    assert!(report.agree());
    let labels: Vec<&str> = report.situations.iter().map(|s| s.label.as_str()).collect();
    assert!(
        labels.contains(&"base") && labels.contains(&"hover") && labels.contains(&"md"),
        "the battery walks every lowered situation: {labels:?}"
    );
    for situation in &report.situations {
        assert!(
            situation.mismatches.is_empty(),
            "situation {} drifted: {:#?}",
            situation.label,
            situation.mismatches
        );
    }
}

#[test]
fn breakpoint_boundary_fires_at_the_rem() {
    let report = metamorphic_check("md:p-6", &Config::default()).expect("non-empty");
    assert!(report.agree());
    let md = report
        .situations
        .iter()
        .find(|s| s.label == "md")
        .expect("the md situation is walked");
    // The conjunctive env sits AT the boundary: min-width fires AT the
    // rem (the CSS boundary semantics the interp pins).
    assert_eq!(md.env.viewport_rem, Bp::Md.rem());
    assert_eq!(md.interp_decls, 4);
    assert!(md.mismatches.is_empty());
}

#[test]
fn stacked_variants_pin_the_conjunctive_env_and_the_fold_note() {
    let report = metamorphic_check("hover:md:p-8", &Config::default()).expect("non-empty");
    // The agreement holds under the env EVERY conjunct holds.
    assert!(report.agree());
    let stacked = report
        .situations
        .iter()
        .find(|s| s.label == "hover+md")
        .expect("the stacked situation is walked");
    assert!(stacked.env.hover);
    assert_eq!(stacked.env.viewport_rem, Bp::Md.rem());
    assert_eq!(stacked.interp_decls, 4);
    // The documented divergence is the typed note: the CSS arm renders
    // the stack conjunctively, the IR folds to the dominant condition.
    assert!(
        report
            .notes
            .iter()
            .any(|note| matches!(note, MetamorphicNote::StackedVariantFold { .. })),
        "the fold note rides: {:#?}",
        report.notes
    );
}

#[test]
fn pseudo_targets_stay_out_of_the_element_cascade() {
    let report = metamorphic_check("p-4 selection:p-4", &Config::default()).expect("non-empty");
    assert!(report.agree());
    let base = report
        .situations
        .iter()
        .find(|s| s.label == "base")
        .expect("the base situation is walked");
    assert_eq!(base.interp_decls, 4, "only the element's own layers");
    assert_eq!(base.css_decls, 4, "the ::selection rule never fires here");
    assert!(
        report
            .notes
            .iter()
            .any(|note| matches!(note, MetamorphicNote::PseudoTarget { .. })),
        "the render-target note rides: {:#?}",
        report.notes
    );
}

// ---------------------------------------------------------------------------
// The theme half: pairs, tokens, the modifier identity
// ---------------------------------------------------------------------------

#[test]
fn themed_pairs_agree_in_both_modes() {
    let table = pair_table();
    let report =
        metamorphic_check_theme("p-4 bg-brand", &table, &Config::default()).expect("non-empty");
    assert!(report.agree());
    let light = report
        .situations
        .iter()
        .find(|s| s.label == "base")
        .expect("the base situation is walked");
    assert!(light.mismatches.is_empty());
    let dark = report
        .situations
        .iter()
        .find(|s| s.label == "base (dark)")
        .expect("the dark-mode walk rides the battery");
    assert!(dark.mismatches.is_empty());
    assert!(dark.env.dark);
}

#[test]
fn runtime_tokens_are_counted_notes_not_lies() {
    let table = runtime_table();
    let report = metamorphic_check_theme(
        "bg-primary text-primary-foreground",
        &table,
        &Config::default(),
    )
    .expect("non-empty");
    // The CSS keeps the `var()` reference (the theme's own dynamism),
    // the interp answers concrete bytes for the probe — the divergence
    // is the TYPED note, and every other longhand still agrees.
    assert!(report.agree());
    let dynamic = report
        .notes
        .iter()
        .filter(|note| matches!(note, MetamorphicNote::TokenDynamic { .. }))
        .count();
    assert_eq!(
        dynamic, 2,
        "one note per token longhand: {:#?}",
        report.notes
    );
}

#[test]
fn modifier_bake_identity_holds_through_the_checker() {
    let table = baked_table();
    let report =
        metamorphic_check_theme("bg-[#0ea5e9]/50", &table, &Config::default()).expect("non-empty");
    assert!(report.agree());
    let base = &report.situations[0];
    assert_eq!(base.interp_decls, 1);
    assert!(base.mismatches.is_empty(), "{:#?}", base.mismatches);
}

// ---------------------------------------------------------------------------
// The structural contracts
// ---------------------------------------------------------------------------

#[test]
fn compositional_identity_whole_equals_parts() {
    let report =
        metamorphic_check("p-4 hover:px-8 md:hidden", &Config::default()).expect("non-empty");
    assert!(report.compositional, "the sheet is the per-candidate parts");
    assert!(report.agree());
}

#[test]
fn the_report_renders_labels_and_notes() {
    let report = metamorphic_check("hover:md:p-8 selection:bg-red-200", &Config::default())
        .expect("non-empty");
    let rendered = report.to_string();
    assert!(rendered.contains("hover+md"), "{rendered}");
    assert!(rendered.contains("agree"), "{rendered}");
}

#[test]
fn the_checker_is_deterministic() {
    let a = metamorphic_check("p-4 hover:px-8", &Config::default()).expect("non-empty");
    let b = metamorphic_check("p-4 hover:px-8", &Config::default()).expect("non-empty");
    assert_eq!(a.situations.len(), b.situations.len());
    for (x, y) in a.situations.iter().zip(&b.situations) {
        assert_eq!(x.label, y.label);
        assert_eq!(x.agree, y.agree);
        assert_eq!(x.env, y.env);
        assert_eq!(x.interp_decls, y.interp_decls);
        assert_eq!(x.css_decls, y.css_decls);
    }
}

#[test]
fn empty_input_still_errors() {
    let err = metamorphic_check("   ", &Config::default()).expect_err("empty is the one error");
    assert!(matches!(err, CompileError::Empty));
}

// ---------------------------------------------------------------------------
// The custom-authoring seam rides the same walk
// ---------------------------------------------------------------------------

#[test]
fn customs_seam_rides_the_checker() {
    let customs = Customs::new().utility(CustomUtility::functional(
        "brand-p",
        CustomForm::Spacing,
        &[
            Prop::PaddingTop,
            Prop::PaddingRight,
            Prop::PaddingBottom,
            Prop::PaddingLeft,
        ],
    ));
    let report = aeolus_compile::metamorphic_check_with("brand-p-4", &customs, &Config::default())
        .expect("non-empty");
    assert!(report.agree());
    let base = &report.situations[0];
    assert_eq!(base.interp_decls, 4);
    assert!(base.mismatches.is_empty(), "{:#?}", base.mismatches);
}

// ---------------------------------------------------------------------------
// The themed + customized session rides the same walk
// ---------------------------------------------------------------------------

/// The full-session checker entry: `metamorphic_check_theme_with` had no
/// caller, no test, no bench and no example anywhere in the workspace (the
/// re-export block named it; nothing exercised it) — the theme-token
/// custom utility pinned against BOTH theme tiers: the T0 literal with
/// the premultiplied `/opacity` bake, the T1 pair, the T2 `var()` tier.
#[test]
fn themed_with_session_rides_the_checker() {
    // The `--value(color)` functional form: the suffix IS a theme token
    // name — the one custom form lowered through the tiered tables (the
    // `/opacity` modifier rides the theme pass's own arithmetic).
    let customs = Customs::new().utility(CustomUtility::functional(
        "brand-ink",
        CustomForm::Token,
        &[Prop::Color],
    ));
    let baked = baked_table();
    let report =
        metamorphic_check_theme_with("brand-ink-red-500/50", &baked, &customs, &Config::default())
            .expect("non-empty");
    assert!(report.agree(), "{:#?}", report.situations);
    let base = &report.situations[0];
    assert_eq!(base.interp_decls, 1, "the token lowers to one longhand");
    assert!(base.mismatches.is_empty(), "{:#?}", base.mismatches);

    // The T1 pair lowers without a modifier (the pair budget is its own
    // typed boundary — `UnsupportedModifier`, the diagnostics-equality
    // battery's own shape).
    let pair_report =
        metamorphic_check_theme_with("brand-ink-primary", &baked, &customs, &Config::default())
            .expect("non-empty");
    assert!(pair_report.agree(), "{:#?}", pair_report.situations);
    assert!(!pair_report.situations.is_empty());

    // The runtime tier keeps the typed note (the token stays a `var()`
    // reference on the CSS side, concrete bytes on the interp side).
    let runtime = runtime_table();
    let report =
        metamorphic_check_theme_with("brand-ink-primary", &runtime, &customs, &Config::default())
            .expect("non-empty");
    assert!(report.agree(), "{:#?}", report.situations);
    let dynamic = report
        .notes
        .iter()
        .filter(|note| matches!(note, MetamorphicNote::TokenDynamic { .. }))
        .count();
    assert_eq!(dynamic, 1, "one typed note for the token longhand");
}

/// The custom variant alias composes with the theme through the same
/// entry: `night:brand-ink-brand` folds onto the dark condition and the
/// checker walks the stack's OWN situation (a dark stack is its own dark
/// walk — the battery never double-flips it).
#[test]
fn themed_with_variant_alias_agrees_in_both_modes() {
    let customs = Customs::new()
        .utility(CustomUtility::functional(
            "brand-ink",
            CustomForm::Token,
            &[Prop::Color],
        ))
        .variant(CustomVariant::new("night", Cond::Dark));
    let table = pair_table();
    let report = metamorphic_check_theme_with(
        "night:brand-ink-brand",
        &table,
        &customs,
        &Config::default(),
    )
    .expect("non-empty");
    assert!(report.agree(), "{:#?}", report.situations);
    let dark = report
        .situations
        .iter()
        .find(|s| s.label == "dark")
        .expect("the dark stack's own situation is walked");
    assert!(dark.mismatches.is_empty(), "{:#?}", dark.mismatches);
    assert!(dark.env.dark, "the dark walk probes the dark mode");
}
