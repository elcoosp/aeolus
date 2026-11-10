//! The CSS emit arm's contracts (`Target::Css` — plan §3.1 emit (d)),
//! pinned end to end: the exact rendered stylesheet for every value
//! form (spacing, fraction, keywords, composites, the number forms,
//! colour tokens per tier, dark pairs, breakpoints, group/peer), the
//! honest boundaries (the same diagnostics the IR form reports), and
//! the drift-catch pins that keep the two walks reporting identically.
//!
//! Written FIRST (the RED spec): the suite fails until `emit_css`
//! renders every pin below.

use aeolus_compile::{
    compile, compile_theme, emit_css, emit_css_theme, CompileError, Config, Spacing,
};
use aeolus_theme::{ThemeSource, TokenTable};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// The test-local rgb renderer — the SAME form the emitter renders
/// (`rgb(r g b)` full opacity, `rgb(r g b / a%)` otherwise); the colour
/// VALUES come from the audited `TokenTable`, never a hardcode.
fn rgb_text(rgba: [u8; 4]) -> String {
    let [r, g, b, a] = rgba;
    if a == 255 {
        format!("rgb({r} {g} {b})")
    } else {
        let pct = (a as f32) / 255.0 * 100.0;
        let rounded = (pct * 1000.0).round() / 1000.0;
        let mut s = format!("{rounded}");
        if s.contains('.') {
            s = s.trim_end_matches('0').trim_end_matches('.').to_string();
        }
        format!("rgb({r} {g} {b} / {s})")
    }
}

/// The runtime table over the shadcn light+dark palette (T2).
fn runtime_tokens() -> TokenTable {
    let source = ThemeSource::shadcn_dark().expect("the embedded shadcn source parses");
    TokenTable::runtime(&source).expect("the pair surface fits the u16 slot")
}

/// The baked table over the same source (T0/T1).
fn baked_tokens() -> TokenTable {
    let source = ThemeSource::shadcn_dark().expect("the embedded shadcn source parses");
    TokenTable::baked(&source).expect("the pair surface fits the u16 slot")
}

// ---------------------------------------------------------------------------
// Value forms
// ---------------------------------------------------------------------------

#[test]
fn spacing_bakes_rem_and_sorts_by_cascade() {
    let sheet = emit_css("p-4", &Config::default()).expect("non-empty input renders");
    assert!(sheet.diagnostics.is_empty());
    assert_eq!(sheet.to_string(), ".p-4 {\n  padding-top: 1rem;\n  padding-right: 1rem;\n  padding-bottom: 1rem;\n  padding-left: 1rem;\n}\n");
}

#[test]
fn px_compat_config_bakes_pixels() {
    let config = Config { spacing: Spacing::Px, ..Config::default() };
    let sheet = emit_css("p-4", &config).expect("non-empty input renders");
    assert!(sheet.to_string().contains("padding-top: 16px;"));
}

#[test]
fn half_steps_and_the_px_step_render() {
    let sheet = emit_css("p-0.5 pb-px", &Config::default()).expect("non-empty input renders");
    let text = sheet.to_string();
    assert!(text.contains("padding-top: 0.125rem;"), "{text}");
    assert!(text.contains("padding-bottom: 1px;"), "{text}");
}

#[test]
fn negation_flips_the_sign() {
    let sheet = emit_css("-mt-2", &Config::default()).expect("non-empty input renders");
    assert!(sheet.to_string().contains("margin-top: -0.5rem;"));
}

#[test]
fn fraction_renders_percent() {
    let sheet = emit_css("w-1/2", &Config::default()).expect("non-empty input renders");
    assert_eq!(
        sheet.to_string(),
        ".w-1\\/2 {\n  width: 50%;\n}\n",
        "the class text escapes, the value renders the row's percent math"
    );
}

#[test]
fn keywords_render_the_row_tables() {
    let sheet = emit_css("flex flex-col font-sans", &Config::default())
        .expect("non-empty input renders");
    assert_eq!(
        sheet.to_string(),
        ".flex {\n  display: flex;\n}\n.flex-col {\n  flex-direction: column;\n}\n.font-sans {\n  font-family: var(--font-sans);\n}\n",
    );
}

#[test]
fn composite_renders_both_halves() {
    let sheet = emit_css("text-sm", &Config::default()).expect("non-empty input renders");
    assert_eq!(
        sheet.to_string(),
        ".text-sm {\n  font-size: 0.875rem;\n  line-height: calc(1.25 / 0.875);\n}\n",
    );
}

#[test]
fn number_forms_render_their_units() {
    let sheet = emit_css("rotate-45 opacity-50 stroke-2 grid-cols-2 z-10", &Config::default())
        .expect("non-empty input renders");
    let text = sheet.to_string();
    assert!(text.contains("--tw-rotate: 45deg;"), "{text}");
    assert!(text.contains("opacity: 50%;"), "{text}");
    assert!(text.contains("stroke-width: 2px;"), "{text}");
    assert!(text.contains("grid-template-columns: repeat(2, minmax(0, 1fr));"), "{text}");
    assert!(text.contains("z-index: 10;"), "{text}");
}

#[test]
fn truncate_renders_all_three_registry_arms() {
    // The composite used to fan ONE bare value over four longhands
    // (`ellipsis` rode overflow-x and white-space too, where it is not
    // a legal value); three root-sharing rows render it faithfully now.
    let sheet = emit_css("truncate", &Config::default()).expect("non-empty input renders");
    assert!(sheet.diagnostics.is_empty(), "{:?}", sheet.diagnostics);
    assert_eq!(
        sheet.to_string(),
        ".truncate {\n  white-space: nowrap;\n  overflow-x: hidden;\n  overflow-y: hidden;\n  text-overflow: ellipsis;\n}\n",
    );
}

// ---------------------------------------------------------------------------
// Conditions
// ---------------------------------------------------------------------------

#[test]
fn probe_variants_stack_conjunctively() {
    let sheet = emit_css("hover:focus:p-4", &Config::default()).expect("non-empty input renders");
    let text = sheet.to_string();
    assert!(text.starts_with(".hover\\:focus\\:p-4:hover:focus {\n"), "{text}");
}

#[test]
fn media_variants_group_into_a_block() {
    let sheet = emit_css("p-4 md:p-6", &Config::default()).expect("non-empty input renders");
    assert_eq!(
        sheet.to_string(),
        ".p-4 {\n  padding-top: 1rem;\n  padding-right: 1rem;\n  padding-bottom: 1rem;\n  padding-left: 1rem;\n}\n@media (min-width: 48rem) {\n  .md\\:p-6 {\n    padding-top: 1.5rem;\n    padding-right: 1.5rem;\n    padding-bottom: 1.5rem;\n    padding-left: 1.5rem;\n  }\n}\n",
    );
}

#[test]
fn named_group_and_peer_render_combinators() {
    let sheet = emit_css("group-hover/menu:p-4 peer-focus:p-2", &Config::default())
        .expect("non-empty input renders");
    let text = sheet.to_string();
    assert!(text.contains(".group\\/menu:hover .group-hover\\/menu\\:p-4 {"), "{text}");
    assert!(text.contains(".peer\\:focus ~ .peer-focus\\:p-2 {"), "{text}");
}

#[test]
fn max_media_uses_the_range_form() {
    let sheet = emit_css("max-md:p-4", &Config::default()).expect("non-empty input renders");
    assert!(sheet.to_string().starts_with("@media (width < 48rem) {\n"));
    let sheet = emit_css("@lg:p-4", &Config::default()).expect("non-empty input renders");
    assert!(sheet.to_string().starts_with("@container (min-width: 64rem) {\n"));
}

// ---------------------------------------------------------------------------
// The theme tiers
// ---------------------------------------------------------------------------

#[test]
fn runtime_tokens_render_the_var_and_the_mix() {
    let tokens = runtime_tokens();
    let sheet = emit_css_theme("bg-primary/90", &tokens, &Config::default())
        .expect("non-empty input renders");
    assert!(sheet.diagnostics.is_empty(), "{:?}", sheet.diagnostics);
    assert_eq!(
        sheet.to_string(),
        ".bg-primary\\/90 {\n  background-color: color-mix(in oklab, var(--color-primary) 90%, transparent);\n}\n",
    );
}

#[test]
fn runtime_tokens_without_a_modifier_render_the_bare_var() {
    let tokens = runtime_tokens();
    let sheet = emit_css_theme("hover:bg-primary", &tokens, &Config::default())
        .expect("non-empty input renders");
    assert_eq!(
        sheet.to_string(),
        ".hover\\:bg-primary:hover {\n  background-color: var(--color-primary);\n}\n",
    );
}

#[test]
fn baked_pairs_render_light_and_dark_companions() {
    let tokens = baked_tokens();
    let sheet = emit_css_theme("bg-primary", &tokens, &Config::default())
        .expect("non-empty input renders");
    assert!(sheet.diagnostics.is_empty(), "{:?}", sheet.diagnostics);
    let light = tokens.light_rgba(tokens.id("primary").expect("the token exists")).expect("light");
    let dark = tokens.dark_rgba(tokens.id("primary").expect("the token exists")).expect("dark");
    assert_eq!(
        sheet.to_string(),
        format!(
            ".bg-primary {{\n  background-color: {};\n}}\n:where(.dark, .dark *) .bg-primary {{\n  background-color: {};\n}}\n",
            rgb_text(light),
            rgb_text(dark)
        ),
    );
}

#[test]
fn dark_candidates_render_the_dark_half_only() {
    // A `dark:`-conditioned candidate must NOT render the pair's light
    // half unconditionally — the light value never appears.
    let tokens = baked_tokens();
    let sheet = emit_css_theme("dark:bg-primary", &tokens, &Config::default())
        .expect("non-empty input renders");
    let text = sheet.to_string();
    assert!(text.contains(".dark\\:bg-primary:where(.dark, .dark *) {"), "{text}");
    assert!(!text.contains(".bg-primary {"), "{text}");
}

#[test]
fn baked_literals_render_rgb() {
    let source = ThemeSource::shadcn().expect("the embedded light source parses");
    let tokens = TokenTable::baked(&source).expect("the table builds");
    let sheet = emit_css_theme("bg-primary", &tokens, &Config::default())
        .expect("non-empty input renders");
    let light = tokens.light_rgba(tokens.id("primary").expect("the token exists")).expect("light");
    assert_eq!(
        sheet.to_string(),
        format!(".bg-primary {{\n  background-color: {};\n}}\n", rgb_text(light)),
    );
}

#[test]
fn baked_literal_alpha_renders_the_premultiplied_bake() {
    let source = ThemeSource::shadcn().expect("the embedded light source parses");
    let tokens = TokenTable::baked(&source).expect("the table builds");
    let sheet = emit_css_theme("bg-primary/90", &tokens, &Config::default())
        .expect("non-empty input renders");
    let baked = tokens
        .resolve(tokens.id("primary").expect("the token exists"), Some(0.9))
        .expect("a T0 token bakes the alpha");
    let aeolus_ir::Val::Rgba(rgba) = baked else {
        panic!("the T0 resolve bakes a literal, got {baked:?}");
    };
    assert_eq!(
        sheet.to_string(),
        format!(".bg-primary\\/90 {{\n  background-color: {};\n}}\n", rgb_text(rgba)),
    );
}

// ---------------------------------------------------------------------------
// Honest boundaries
// ---------------------------------------------------------------------------

#[test]
fn boundaries_ride_the_sheet_diagnostics() {
    let sheet = emit_css("p-red not-a-utility has-[x]:p-4 !p-4", &Config::default())
        .expect("non-empty input renders");
    assert!(sheet.rules.is_empty(), "nothing lowered: {:?}", sheet.rules);
    assert_eq!(sheet.diagnostics.len(), 4, "{:?}", sheet.diagnostics);
    let codes: Vec<&str> = sheet.diagnostics.iter().map(|d| d.kind.code()).collect();
    assert_eq!(
        codes,
        vec![
            "aeolus::compile::shape_rejected",
            "aeolus::compile::unknown_root",
            "aeolus::compile::unsupported_variant",
            "aeolus::compile::unsupported_important",
        ]
    );
    // The help texts stay actionable (the linter renders them).
    for d in &sheet.diagnostics {
        assert!(!d.kind.help().is_empty());
    }
}

#[test]
fn modifiers_without_the_theme_pass_stay_boundaries() {
    let sheet = emit_css("bg-primary/90", &Config::default()).expect("non-empty input renders");
    assert!(sheet.rules.is_empty());
    assert_eq!(
        sheet.diagnostics[0].kind.code(),
        "aeolus::compile::unsupported_modifier"
    );
}

#[test]
fn lex_failures_are_spanned_in_the_sheet() {
    let sheet = emit_css("p--4 p-2", &Config::default()).expect("non-empty input renders");
    assert_eq!(sheet.diagnostics.len(), 1);
    assert_eq!(sheet.diagnostics[0].span, (0, 4));
    assert_eq!(sheet.diagnostics[0].kind.code(), "aeolus::compile::lex");
    assert!(sheet.to_string().contains(".p-2 {"), "the rest still renders");
}

#[test]
fn empty_input_fails_the_sheet() {
    let err = emit_css("   ", &Config::default()).unwrap_err();
    assert!(matches!(err, CompileError::Empty));
}

// ---------------------------------------------------------------------------
// The drift catch: the two walks report identically
// ---------------------------------------------------------------------------

#[test]
fn the_two_walks_report_identically_themeless() {
    let battery = [
        "p-4",
        "p-red",
        "not-a-utility",
        "w-[]",
        "has-[x]:p-4",
        "!p-4",
        "bg-primary/90",
        "-p-4",
        "hover:focus:md:p-0",
        "p--4 p-2",
        "w-1/2 h-full",
        "dark:hidden",
        "truncate",
    ];
    for input in battery {
        let compiled = compile(input, &Config::default()).expect("non-empty input compiles");
        let sheet = emit_css(input, &Config::default()).expect("non-empty input renders");
        assert_eq!(
            compiled.diagnostics, sheet.diagnostics,
            "diagnostic drift between the IR form and the CSS form on {input:?}"
        );
    }
}

#[test]
fn the_two_walks_report_identically_themed() {
    let tokens = runtime_tokens();
    let battery = [
        "bg-primary/90",
        "bg-red-500",
        "hover:bg-primary/40",
        "dark:bg-primary",
        "bg-primary/[0.37]",
        "border-2 border-primary",
    ];
    for input in battery {
        let compiled = compile_theme(input, &tokens, &Config::default())
            .expect("non-empty input compiles");
        let sheet = emit_css_theme(input, &tokens, &Config::default())
            .expect("non-empty input renders");
        assert_eq!(
            compiled.diagnostics, sheet.diagnostics,
            "diagnostic drift between the IR form and the CSS form on {input:?}"
        );
    }
}
