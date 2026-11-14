//! The backdrop-filter layer's contracts (architecture plan §3.7's
//! fourth row — the "filters, `backdrop-blur`" line's second half): the
//! `backdrop-blur-*`/`backdrop-brightness-*`/`backdrop-contrast-*`/
//! `backdrop-grayscale-*`/`backdrop-hue-rotate-*`/`backdrop-invert-*`/
//! `backdrop-opacity-*`/`backdrop-saturate-*`/`backdrop-sepia-*`/
//! `backdrop-filter` families lower through the SAME typed pipeline as
//! the filter layer — IR longhands, the value pass, the CSS arm's v4
//! var-chain rendering (the `-webkit-` prefix beside the standard
//! property, v4's own emission form), the interp arm, the static arm,
//! the IR-level `cn()` merge — and the honest boundaries stay typed.
//!
//! The one backdrop-only primitive is `backdrop-opacity-*` (the
//! backdrop chain's own ninth var, no filter-half counterpart); the
//! `backdrop-blur` theme keys fall back to `--blur` (v4's own chain:
//! `--backdrop-blur`, `--blur`), which the shared scale expresses.
//!
//! Written FIRST (the RED spec): the suite fails until the backdrop
//! families lower — `Prop` has no backdrop longhands and the registry
//! carries no backdrop rows before the milestone.

use aeolus_compile::{Config, Env, cn, compile, emit_css, interp, keyword_texts};
use aeolus_ir::{Cond, Prop, Val};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn config() -> Config {
    Config::default()
}

/// The one base declaration `class` lowers to (the backdrop families
/// are single-longhand rows).
fn base_decl_of(class: &str) -> (Prop, Val) {
    let compiled = compile(class, &config()).expect("non-empty input compiles");
    let layers = compiled.program.layers;
    assert_eq!(
        layers.len(),
        1,
        "{class}: one base layer, got {:?}",
        layers.iter().map(|l| l.cond).collect::<Vec<_>>()
    );
    let decls = layers[0].decls;
    assert_eq!(decls.len(), 1, "{class}: one declaration, got {decls:?}");
    (decls[0].prop, decls[0].val)
}

fn refused(class: &str) {
    let compiled = compile(class, &config()).expect("the walk is total");
    assert!(
        !compiled.diagnostics.is_empty(),
        "{class}: expected a typed refusal, got none"
    );
}

/// The composed `backdrop-filter` chain — v4's own
/// `cssBackdropFilterValue`, the string every backdrop class's rule
/// re-declares (standard + prefixed property).
const CHAIN: &str = "var(--tw-backdrop-blur,) var(--tw-backdrop-brightness,) \
     var(--tw-backdrop-contrast,) var(--tw-backdrop-grayscale,) var(--tw-backdrop-hue-rotate,) \
     var(--tw-backdrop-invert,) var(--tw-backdrop-opacity,) var(--tw-backdrop-saturate,) \
     var(--tw-backdrop-sepia,)";

// ---------------------------------------------------------------------------
// 1. The IR shapes: one longhand per family, the row-table slots
// ---------------------------------------------------------------------------

#[test]
fn backdrop_blur_scale_lowers_onto_the_backdrop_blur_longhand() {
    // The shared v4 blur scale (the `--backdrop-blur`→`--blur` theme
    // fallback): none=0, xs=1, sm=2, md=3, lg=4, xl=5, 2xl=6, 3xl=7.
    for (class, slot) in [
        ("backdrop-blur-none", 0u8),
        ("backdrop-blur-xs", 1),
        ("backdrop-blur-sm", 2),
        ("backdrop-blur-md", 3),
        ("backdrop-blur-3xl", 7),
    ] {
        let (prop, val) = base_decl_of(class);
        assert_eq!(prop, Prop::BackdropBlur, "{class}");
        assert_eq!(val, Val::Kw(slot), "{class}");
    }
}

#[test]
fn backdrop_percent_families_lower_onto_their_num_slots() {
    for (class, prop, n) in [
        ("backdrop-brightness-90", Prop::BackdropBrightness, 90.0f32),
        ("backdrop-contrast-50", Prop::BackdropContrast, 50.0),
        ("backdrop-saturate-200", Prop::BackdropSaturate, 200.0),
    ] {
        let (p, val) = base_decl_of(class);
        assert_eq!(p, prop, "{class}");
        assert_eq!(val, Val::Num(n), "{class}");
    }
}

#[test]
fn backdrop_default_full_families_lower_the_bare_form() {
    for (class, prop) in [
        ("backdrop-grayscale", Prop::BackdropGrayscale),
        ("backdrop-invert", Prop::BackdropInvert),
        ("backdrop-sepia", Prop::BackdropSepia),
    ] {
        let (p, val) = base_decl_of(class);
        assert_eq!(p, prop, "{class}");
        assert_eq!(val, Val::Kw(0), "{class}: the bare arm's slot");
        let classes = keyword_texts(class, &config()).expect("annotates");
        let bare = format!(
            "{}(100%)",
            class.strip_prefix("backdrop-").expect("the prefix")
        );
        assert_eq!(classes[0].keywords[0].1, bare, "{class}: the v4 default");
    }
}

#[test]
fn backdrop_opacity_is_the_ninth_primitive() {
    // The backdrop chain's own var with no filter-half counterpart:
    // `backdrop-opacity-50` → the percent scale.
    let (prop, val) = base_decl_of("backdrop-opacity-50");
    assert_eq!(prop, Prop::BackdropOpacity);
    assert_eq!(val, Val::Num(50.0));
}

#[test]
fn backdrop_hue_rotate_takes_the_negative_flag() {
    let (prop, val) = base_decl_of("backdrop-hue-rotate-90");
    assert_eq!(prop, Prop::BackdropHueRotate);
    assert_eq!(val, Val::Num(90.0));
    let (prop, val) = base_decl_of("-backdrop-hue-rotate-15");
    assert_eq!(prop, Prop::BackdropHueRotate);
    assert_eq!(val, Val::Num(-15.0));
}

#[test]
fn the_composed_backdrop_filter_root_lowers_onto_its_longhand() {
    let (prop, val) = base_decl_of("backdrop-filter-none");
    assert_eq!(prop, Prop::BackdropFilter);
    assert_eq!(val, Val::Kw(0));
    let classes = keyword_texts("backdrop-filter-none", &config()).expect("annotates");
    assert_eq!(classes[0].keywords[0].1, "none");

    let (prop, val) = base_decl_of("backdrop-filter");
    assert_eq!(prop, Prop::BackdropFilter);
    assert_eq!(val, Val::Kw(0), "the bare arm's slot");
    let classes = keyword_texts("backdrop-filter", &config()).expect("annotates");
    assert_eq!(classes[0].keywords[0].1, CHAIN, "the bare chain text");
}

// ---------------------------------------------------------------------------
// 2. The honest boundaries stay typed
// ---------------------------------------------------------------------------

#[test]
fn bare_backdrop_brightness_refuses_the_v4_absence() {
    refused("backdrop-brightness");
    refused("backdrop-contrast");
    refused("backdrop-saturate");
    refused("backdrop-hue-rotate");
    refused("backdrop-opacity");
}

#[test]
fn arbitrary_backdrop_filters_lower_the_single_value_forms() {
    let (prop, val) = base_decl_of("backdrop-blur-[2px]");
    assert_eq!(prop, Prop::BackdropBlur);
    assert_eq!(val, Val::Px(2.0));

    let (prop, val) = base_decl_of("backdrop-brightness-[1.75]");
    assert_eq!(prop, Prop::BackdropBrightness);
    assert_eq!(val, Val::Num(1.75));
}

// ---------------------------------------------------------------------------
// 3. The CSS arm: the v4 var-chain form, prefixed beside the standard
// ---------------------------------------------------------------------------

#[test]
fn the_css_arm_renders_the_prefixed_var_chain_form() {
    // v4's own emission: the primitive's var line + BOTH composed
    // properties (`-webkit-backdrop-filter` beside `backdrop-filter`),
    // one rule per class.
    let sheet = emit_css("backdrop-blur-md backdrop-brightness-90", &config()).expect("the sheet");
    assert!(sheet.diagnostics.is_empty(), "{:?}", sheet.diagnostics);
    assert_eq!(sheet.rules.len(), 2, "one rule per class");
    assert_eq!(
        sheet.rules[0].body,
        format!(
            "--tw-backdrop-blur: blur(12px);\n-webkit-backdrop-filter: {CHAIN};\nbackdrop-filter: {CHAIN};"
        ),
        "backdrop-blur-md's var line + the prefixed pair"
    );
    assert_eq!(
        sheet.rules[1].body,
        format!(
            "--tw-backdrop-brightness: brightness(90%);\n-webkit-backdrop-filter: {CHAIN};\nbackdrop-filter: {CHAIN};"
        ),
        "backdrop-brightness-90's var line + the prefixed pair"
    );
}

#[test]
fn the_css_arm_renders_the_backdrop_reset_and_off_texts() {
    let sheet = emit_css(
        "backdrop-blur-none backdrop-filter-none backdrop-filter",
        &config(),
    )
    .expect("the sheet");
    assert!(sheet.diagnostics.is_empty(), "{:?}", sheet.diagnostics);
    let bodies: Vec<&str> = sheet.rules.iter().map(|r| r.body.as_str()).collect();
    assert_eq!(
        bodies[0],
        format!(
            "--tw-backdrop-blur:  ;\n-webkit-backdrop-filter: {CHAIN};\nbackdrop-filter: {CHAIN};"
        ),
        "the v4 off text rides the var line verbatim"
    );
    assert_eq!(
        bodies[1], "-webkit-backdrop-filter: none;\nbackdrop-filter: none;",
        "the composed reset, prefixed beside the standard property"
    );
    assert_eq!(
        bodies[2],
        format!("-webkit-backdrop-filter: {CHAIN};\nbackdrop-filter: {CHAIN};"),
        "the bare chain, prefixed beside the standard property"
    );
}

// ---------------------------------------------------------------------------
// 4. The IR-level cn() merge and the interp arm
// ---------------------------------------------------------------------------

#[test]
fn cn_folds_same_condition_backdrop_filters_per_longhand() {
    let a = compile("backdrop-blur-md", &config()).expect("compiles");
    let b = compile("backdrop-blur-lg", &config()).expect("compiles");
    let merged = cn(&[&a, &b]).expect("merges");
    let base = merged
        .program
        .layers
        .iter()
        .find(|l| l.cond == Cond::Base)
        .expect("base layer");
    let blurs: Vec<_> = base
        .decls
        .iter()
        .filter(|d| d.prop == Prop::BackdropBlur)
        .collect();
    assert_eq!(blurs.len(), 1, "one winning backdrop blur: {blurs:?}");
}

#[test]
fn the_interp_answers_the_backdrop_primitives() {
    let compiled = compile("backdrop-blur-md backdrop-brightness-90", &config()).expect("compiles");
    let interped = interp(&compiled.program, &Env::default()).expect("resolves");
    let touched = interped.touched();
    assert!(touched.contains(Prop::BackdropBlur));
    assert!(touched.contains(Prop::BackdropBrightness));
}
