//! The CSS filter layer's contracts (architecture plan §3.7's fourth
//! row — the "filters, backdrop-blur … polyfill wrappers backlog"
//! line's pipeline half): the `blur-*`/`brightness-*`/`contrast-*`/
//! `grayscale-*`/`hue-rotate-*`/`invert-*`/`saturate-*`/`sepia-*`/
//! `drop-shadow-*`/`filter` families lower through the SAME typed
//! pipeline as every other registry family — IR longhands, the value
//! pass, the CSS arm's v4 var-chain rendering, the interp arm, the
//! rust-tokens arm's annotations, the static arm, the IR-level `cn()`
//! merge — and the honest boundaries stay typed (bare `brightness` has
//! no v4 default, composite arbitrary values ride the existing
//! composites boundary).
//!
//! The v4.1.17 source of truth (the row tables' texts, the composed
//! var chain) is quoted verbatim from the pinned upstream — the same
//! one-source discipline the motion layer's `TRANSITION_SETS`/
//! `EASE_CURVES` texts ride.
//!
//! Written FIRST (the RED spec): the suite fails until the filter
//! families lower — `Prop` has no filter longhands and the registry
//! carries no filter rows before the milestone.

use aeolus_compile::{
    Config, Env, MetamorphicNote, PlanNoteKind, cn, compile, emit_css, emit_plan, emit_rust,
    interp, keyword_texts, metamorphic_check, sort,
};
use aeolus_ir::{Cond, Prop, Val};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn config() -> Config {
    Config::default()
}

/// The one base declaration `class` lowers to (the filter families are
/// single-longhand rows).
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

/// The composed `filter` chain — v4's own `cssFilterValue`, the exact
/// string every filter class's rule re-declares.
const CHAIN: &str = "var(--tw-blur,) var(--tw-brightness,) var(--tw-contrast,) \
     var(--tw-grayscale,) var(--tw-hue-rotate,) var(--tw-invert,) var(--tw-saturate,) \
     var(--tw-sepia,) var(--tw-drop-shadow,)";

// ---------------------------------------------------------------------------
// 1. The IR shapes: one longhand per family, the row-table slots
// ---------------------------------------------------------------------------

#[test]
fn blur_scale_lowers_onto_the_blur_longhand() {
    // The v4 default blur scale in keyword-slot order (the theme's own
    // values): none=0, xs=1, sm=2, md=3, lg=4, xl=5, 2xl=6, 3xl=7.
    for (class, slot) in [
        ("blur-none", 0u8),
        ("blur-xs", 1),
        ("blur-sm", 2),
        ("blur-md", 3),
        ("blur-lg", 4),
        ("blur-xl", 5),
        ("blur-2xl", 6),
        ("blur-3xl", 7),
    ] {
        let (prop, val) = base_decl_of(class);
        assert_eq!(prop, Prop::FilterBlur, "{class}");
        assert_eq!(val, Val::Kw(slot), "{class}");
    }
}

#[test]
fn bare_blur_lowers_the_v4_default() {
    // Bare `blur` is the v4 default token (`--blur: 8px`): slot 0 is
    // the bare arm's own convention; the annotation carries the text.
    let (prop, val) = base_decl_of("blur");
    assert_eq!(prop, Prop::FilterBlur);
    assert_eq!(val, Val::Kw(0));
    let classes = keyword_texts("blur", &config()).expect("annotates");
    assert_eq!(classes[0].keywords[0].1, "blur(8px)");
}

#[test]
fn percent_families_lower_onto_their_num_slots() {
    // The bare-integer percent form (`brightness-110` → 110, rendered
    // `brightness(110%)`); the IR carries the raw number (the motion
    // layer's millisecond precedent — the unit lives in the render
    // table, the slot is documented on the Prop).
    for (class, prop, n) in [
        ("brightness-110", Prop::FilterBrightness, 110.0f32),
        ("contrast-125", Prop::FilterContrast, 125.0),
        ("saturate-150", Prop::FilterSaturate, 150.0),
    ] {
        let (p, val) = base_decl_of(class);
        assert_eq!(p, prop, "{class}");
        assert_eq!(val, Val::Num(n), "{class}");
    }
}

#[test]
fn default_full_families_lower_the_bare_form() {
    // `grayscale`/`invert`/`sepia` carry the v4 100% default on the
    // bare root (the keyword-less default arm).
    for (class, prop) in [
        ("grayscale", Prop::FilterGrayscale),
        ("invert", Prop::FilterInvert),
        ("sepia", Prop::FilterSepia),
    ] {
        let (p, val) = base_decl_of(class);
        assert_eq!(p, prop, "{class}");
        assert_eq!(val, Val::Kw(0), "{class}: the bare arm's slot");
        let classes = keyword_texts(class, &config()).expect("annotates");
        assert_eq!(
            classes[0].keywords[0].1,
            &format!("{class}(100%)"),
            "{class}: the v4 default text"
        );
    }
}

#[test]
fn hue_rotate_takes_the_negative_flag() {
    // The v4 negative-capable family (`-hue-rotate-30` → -30deg).
    let (prop, val) = base_decl_of("hue-rotate-30");
    assert_eq!(prop, Prop::FilterHueRotate);
    assert_eq!(val, Val::Num(30.0));

    let (prop, val) = base_decl_of("-hue-rotate-30");
    assert_eq!(prop, Prop::FilterHueRotate);
    assert_eq!(val, Val::Num(-30.0));
}

#[test]
fn drop_shadow_presets_lower_onto_the_drop_shadow_longhand() {
    // The v4 preset scale, keyword-slot order: none=0, xs=1, sm=2,
    // md=3, lg=4, xl=5, 2xl=6; the bare root IS the default two-layer
    // preset.
    for (class, slot) in [
        ("drop-shadow-none", 0u8),
        ("drop-shadow-xs", 1),
        ("drop-shadow-sm", 2),
        ("drop-shadow-md", 3),
        ("drop-shadow-lg", 4),
        ("drop-shadow-xl", 5),
        ("drop-shadow-2xl", 6),
    ] {
        let (prop, val) = base_decl_of(class);
        assert_eq!(prop, Prop::FilterDropShadow, "{class}");
        assert_eq!(val, Val::Kw(slot), "{class}");
    }
    let (prop, val) = base_decl_of("drop-shadow");
    assert_eq!(prop, Prop::FilterDropShadow);
    assert_eq!(val, Val::Kw(0), "the bare arm's slot");
    let classes = keyword_texts("drop-shadow", &config()).expect("annotates");
    assert_eq!(
        classes[0].keywords[0].1,
        "drop-shadow(0 1px 2px rgb(0 0 0 / 0.1)) drop-shadow(0 1px 1px rgb(0 0 0 / 0.06))",
        "the v4 --drop-shadow default's two-layer text"
    );
}

#[test]
fn the_composed_filter_root_lowers_onto_the_filter_longhand() {
    // `filter-none` is the composed property's reset; the bare `filter`
    // re-declares the chain (v4's own bare form).
    let (prop, val) = base_decl_of("filter-none");
    assert_eq!(prop, Prop::Filter);
    assert_eq!(val, Val::Kw(0));
    let classes = keyword_texts("filter-none", &config()).expect("annotates");
    assert_eq!(classes[0].keywords[0].1, "none");

    let (prop, val) = base_decl_of("filter");
    assert_eq!(prop, Prop::Filter);
    assert_eq!(val, Val::Kw(0), "the bare arm's slot");
    let classes = keyword_texts("filter", &config()).expect("annotates");
    assert_eq!(classes[0].keywords[0].1, CHAIN, "the bare chain text");
}

// ---------------------------------------------------------------------------
// 2. The honest boundaries stay typed
// ---------------------------------------------------------------------------

#[test]
fn bare_brightness_refuses_the_v4_absence() {
    // v4's brightness utility has NO default (the theme key is
    // absent): the bare root is a typed shape refusal.
    refused("brightness");
    refused("contrast");
    refused("saturate");
    refused("hue-rotate");
}

#[test]
fn arbitrary_filters_lower_the_single_value_forms() {
    // `blur-[2px]` rides the length grammar into the px slot;
    // `brightness-[1.75]` rides the unitless number.
    let (prop, val) = base_decl_of("blur-[2px]");
    assert_eq!(prop, Prop::FilterBlur);
    assert_eq!(val, Val::Px(2.0));

    let (prop, val) = base_decl_of("brightness-[1.75]");
    assert_eq!(prop, Prop::FilterBrightness);
    assert_eq!(val, Val::Num(1.75));

    // A percentage arbitrary keeps the percent form.
    let (prop, val) = base_decl_of("brightness-[50%]");
    assert_eq!(prop, Prop::FilterBrightness);
    assert_eq!(val, Val::Pct(50.0));
}

#[test]
fn composite_arbitrary_filters_refuse_the_documented_boundary() {
    // The arbitrary grammar's composites boundary (`calc()`,
    // underscore joins): `filter-[blur(2px)_brightness(0.5)]` is the
    // codegen host's surface, typed, never silent.
    refused("filter-[blur(2px)_brightness(0.5)]");
}

// ---------------------------------------------------------------------------
// 3. The annotation seam: the row texts ride
// ---------------------------------------------------------------------------

#[test]
fn the_blur_texts_ride_the_annotation_seam() {
    let classes =
        keyword_texts("blur-none blur-xs blur-sm blur-md blur-2xl", &config()).expect("annotates");
    let texts: Vec<&str> = classes.iter().map(|c| c.keywords[0].1).collect();
    assert_eq!(
        texts,
        [" ", "blur(4px)", "blur(8px)", "blur(12px)", "blur(40px)"],
        "the v4 default theme's own blur values (the none arm's off text)"
    );
}

// ---------------------------------------------------------------------------
// 4. The CSS arm: the v4 var-chain form
// ---------------------------------------------------------------------------

#[test]
fn the_css_arm_renders_the_var_plus_chain_form() {
    // v4's own emission: the primitive's var line + the composed
    // chain, one rule per class.
    let sheet = emit_css("blur-md brightness-110", &config()).expect("the sheet");
    assert!(sheet.diagnostics.is_empty(), "{:?}", sheet.diagnostics);
    assert_eq!(sheet.rules.len(), 2, "one rule per class");
    assert_eq!(
        sheet.rules[0].body,
        format!("--tw-blur: blur(12px);\nfilter: {CHAIN};"),
        "blur-md's var line + chain"
    );
    assert_eq!(
        sheet.rules[1].body,
        format!("--tw-brightness: brightness(110%);\nfilter: {CHAIN};"),
        "brightness-110's var line + chain"
    );
}

#[test]
fn the_css_arm_renders_the_percent_and_degree_families() {
    let sheet = emit_css(
        "grayscale invert sepia hue-rotate-30 -hue-rotate-15 brightness-[1.75]",
        &config(),
    )
    .expect("the sheet");
    assert!(sheet.diagnostics.is_empty(), "{:?}", sheet.diagnostics);
    let bodies: Vec<&str> = sheet.rules.iter().map(|r| r.body.as_str()).collect();
    assert_eq!(
        bodies,
        [
            &format!("--tw-grayscale: grayscale(100%);\nfilter: {CHAIN};")[..],
            &format!("--tw-invert: invert(100%);\nfilter: {CHAIN};"),
            &format!("--tw-sepia: sepia(100%);\nfilter: {CHAIN};"),
            &format!("--tw-hue-rotate: hue-rotate(30deg);\nfilter: {CHAIN};"),
            &format!("--tw-hue-rotate: hue-rotate(-15deg);\nfilter: {CHAIN};"),
            &format!("--tw-brightness: brightness(1.75);\nfilter: {CHAIN};"),
        ]
    );
}

#[test]
fn the_css_arm_renders_the_reset_and_the_off_texts() {
    let sheet = emit_css(
        "blur-none drop-shadow-md filter-none drop-shadow",
        &config(),
    )
    .expect("the sheet");
    assert!(sheet.diagnostics.is_empty(), "{:?}", sheet.diagnostics);
    let bodies: Vec<&str> = sheet.rules.iter().map(|r| r.body.as_str()).collect();
    assert_eq!(
        bodies[0],
        format!("--tw-blur:  ;\nfilter: {CHAIN};"),
        "the v4 off text rides the var line verbatim"
    );
    assert_eq!(
        bodies[1],
        format!("--tw-drop-shadow: drop-shadow(0 3px 3px rgb(0 0 0 / 0.12));\nfilter: {CHAIN};"),
        "the md preset's composed text"
    );
    assert_eq!(bodies[2], "filter: none;", "the composed reset");
    assert_eq!(
        bodies[3],
        format!(
            "--tw-drop-shadow: drop-shadow(0 1px 2px rgb(0 0 0 / 0.1)) \
             drop-shadow(0 1px 1px rgb(0 0 0 / 0.06));\nfilter: {CHAIN};"
        ),
        "the bare default's two-layer text"
    );
}

// ---------------------------------------------------------------------------
// 5. The IR-level cn() merge: same-condition filters fold per longhand
// ---------------------------------------------------------------------------

#[test]
fn cn_folds_same_condition_filters_per_longhand() {
    // `blur-md blur-lg` is one FilterBlur slot: last wins (the
    // same-condition fold the IR-level cn() contract pins for every
    // family).
    let a = compile("blur-md", &config()).expect("compiles");
    let b = compile("blur-lg", &config()).expect("compiles");
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
        .filter(|d| d.prop == Prop::FilterBlur)
        .collect();
    assert_eq!(blurs.len(), 1, "one winning blur: {blurs:?}");
}

// ---------------------------------------------------------------------------
// 6. The interp arm: the primitives answer per longhand
// ---------------------------------------------------------------------------

#[test]
fn the_interp_answers_the_filter_primitives() {
    let compiled = compile("blur-md brightness-110", &config()).expect("compiles");
    let interped = interp(&compiled.program, &Env::default()).expect("resolves");
    let touched = interped.touched();
    assert!(touched.contains(Prop::FilterBlur));
    assert!(touched.contains(Prop::FilterBrightness));
}

// ---------------------------------------------------------------------------
// 7. The plan bridge: no v0.x filter slot — the typed note
// ---------------------------------------------------------------------------

#[test]
fn the_plan_bridge_reports_the_filter_slots_as_unsupported() {
    // The v0.x oracle refuses the filter family (its own typed
    // `unsupported` arm); the plan bridge answers the same honesty
    // with its own note kind — the grid-auto-rows precedent.
    let sheet = emit_plan("blur-md saturate-150", &config()).expect("the plan sheet");
    let props: Vec<_> = sheet
        .notes
        .iter()
        .filter_map(|note| match &note.kind {
            PlanNoteKind::SlotUnsupported { prop } => Some(*prop),
            _ => None,
        })
        .collect();
    assert!(
        props.contains(&Prop::FilterBlur) && props.contains(&Prop::FilterSaturate),
        "the filter primitives report: {props:?}"
    );
}

// ---------------------------------------------------------------------------
// 8. The metamorphic triangle: compile == CSS == interp
// ---------------------------------------------------------------------------

#[test]
fn the_metamorphic_walk_agrees_on_the_filter_families() {
    let report =
        metamorphic_check("blur-md brightness-110 grayscale", &config()).expect("the walk");
    assert!(report.agree(), "the three answers agree: {report:?}");
    for situation in &report.situations {
        assert!(
            situation.mismatches.is_empty(),
            "{:?}: {:?}",
            situation.label,
            situation.mismatches
        );
    }
    // The chain line is the composition re-declaration: counted as the
    // typed note, never compared as text.
    assert!(
        report
            .notes
            .iter()
            .any(|note| matches!(note, MetamorphicNote::FilterCompose { .. })),
        "the compose note rides: {:?}",
        report.notes
    );
}

// ---------------------------------------------------------------------------
// 9. The static + rust-tokens arms freeze the layer
// ---------------------------------------------------------------------------

#[test]
fn the_static_arm_freezes_the_filter_layer() {
    let compiled = compile("blur-md", &config()).expect("compiles");
    let text = compiled.emit_static();
    assert!(
        text.contains("Val::Kw(3)"),
        "the blur-md slot freezes: {text}"
    );
    // Byte-stability: two freezes agree.
    let again = compile("blur-md", &config())
        .expect("compiles")
        .emit_static();
    assert_eq!(text, again, "the static text is deterministic");
}

#[test]
fn the_rust_tokens_arm_carries_the_row_text() {
    let sheet = emit_rust("blur-md", &config()).expect("the fragments");
    let rendered = sheet.to_string();
    assert!(
        rendered.contains("blur(12px)"),
        "the annotation resolves the row text: {rendered}"
    );
}

// ---------------------------------------------------------------------------
// 10. The registry's sort order: the filters ride the canonical tail
// ---------------------------------------------------------------------------

#[test]
fn the_sort_seam_orders_the_filters_by_registry_order() {
    let sorted = sort("sepia blur-md brightness-110").expect("sorts");
    let order: Vec<&str> = sorted.classes.iter().map(|c| c.raw.as_str()).collect();
    assert_eq!(
        order,
        ["blur-md", "brightness-110", "sepia"],
        "registry row order (the families appended at the cascade tail)"
    );
}
