//! The gradient-stop longhand contracts (architecture plan §3.7's
//! `bg-linear-*` row, the pipeline half): the `from-*`/`via-*`/`to-*`
//! stop classes lower onto their OWN longhands — the v4 custom-property
//! set (`--tw-gradient-from`, `--tw-gradient-via`, `--tw-gradient-to`
//! and the per-edge `-position` vars) — instead of folding onto
//! `Prop::BackgroundImage`, where the last stop clobbered the gradient
//! direction in the per-longhand cascade.
//!
//! The pins, in the order the plan promises them:
//!
//! * stop colours lower through the SAME theme-token path the colour
//!   roots run (a stop agrees with `bg-{token}`'s value, alpha
//!   modifiers ride), never through the u8 keyword slot;
//! * the direction class and the stop classes COEXIST in one program
//!   (the clobber regression is the pin);
//! * stop positions (`from-20%`) lower onto the position longhands as
//!   `Val::Pct`, v4's `--tw-gradient-from-position` var;
//! * the CSS emit arm renders the v4 custom-property forms;
//! * the plan bridge COMPOSES the stops with the row text's direction
//!   into the v0.x `Gradient` slot (`Linear`/`Radial`/`Conic`/`None`)
//!   — the differential oracle's own gradient comparison, the five
//!   plan-path backends getting real gradients through their slot;
//! * the themeless refusal stays typed (the colour table still exceeds
//!   the u8 slot when no table rides).
//!
//! Written FIRST (the RED spec): the suite fails against the
//! `BackgroundImage`-folded lowering and passes only once the stop
//! longhands land.

use aeolus_compile::{
    Config, PlanNoteKind, compile, compile_theme, emit_css_theme, emit_plan_theme, interp_theme,
};
use aeolus_core::{TAILWIND_DEFAULT_THEME, TailwindTheme};
use aeolus_ir::{Prop, Val};
use aeolus_theme::{ThemeSource, TokenTable};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// The baked-tier table over the shadcn light+dark source.
fn baked_table() -> TokenTable {
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    TokenTable::baked(&source).expect("default-seeded source builds")
}

/// The declaration `(prop, val)` pairs of one compiled string, flattened
/// in layer order.
fn decls_of(compiled: &aeolus_compile::Compiled) -> Vec<(Prop, Val)> {
    compiled
        .program
        .layers
        .iter()
        .flat_map(|layer| layer.decls.iter().map(|decl| (decl.prop, decl.val)))
        .collect()
}

/// The value one longhand carries.
fn value_of(pairs: &[(Prop, Val)], prop: Prop) -> Val {
    pairs
        .iter()
        .find(|(p, _)| *p == prop)
        .unwrap_or_else(|| panic!("no {prop:?} in {pairs:?}"))
        .1
}

/// The sRGB quantization tolerance (the plan_bridge harness's own
/// constant): the baked table's ±0.5/255 rounding per channel, well
/// inside 0.02 per Oklab axis.
const OKLAB_EPSILON: f32 = 0.02;

/// The two colours agree within the baked-table quantization.
fn colors_agree(a: &aeolus_core::Color, b: &aeolus_core::Color) -> bool {
    (a.l - b.l).abs() <= OKLAB_EPSILON
        && (a.a - b.a).abs() <= OKLAB_EPSILON
        && (a.b - b.b).abs() <= OKLAB_EPSILON
}

// ---------------------------------------------------------------------------
// The value path: stops are colour longhands
// ---------------------------------------------------------------------------

#[test]
fn stop_colors_lower_onto_their_own_longhands() {
    // The stop classes ride the SAME token path the colour roots run:
    // `from-red-500` agrees with `bg-red-500`'s value (the differential
    // pin — no absolute hex in the suite), and the decl lands on
    // `GradientFrom`, never on `BackgroundImage`.
    let table = baked_table();
    let bg = compile_theme("bg-red-500", &table, &Config::default()).expect("bg lowers");
    let from = compile_theme("from-red-500", &table, &Config::default()).expect("from lowers");
    let to = compile_theme("to-blue-500", &table, &Config::default()).expect("to lowers");

    let bg_val = value_of(&decls_of(&bg), Prop::Background);
    let from_val = value_of(&decls_of(&from), Prop::GradientFrom);
    let to_val = value_of(&decls_of(&to), Prop::GradientTo);

    assert_eq!(bg_val, from_val, "from-red-500 agrees with bg-red-500");
    assert!(
        matches!(to_val, Val::Rgba(_) | Val::Tok { .. } | Val::ModePair(_)),
        "the stop lowers colour-form: {to_val:?}"
    );
    for pair in decls_of(&from) {
        assert_ne!(pair.0, Prop::BackgroundImage, "no BackgroundImage fold");
    }
}

#[test]
fn via_lowers_onto_its_own_longhand() {
    let table = baked_table();
    let via = compile_theme("via-white", &table, &Config::default()).expect("via lowers");
    let pairs = decls_of(&via);
    let via_val = value_of(&pairs, Prop::GradientVia);
    assert!(
        matches!(via_val, Val::Rgba(_) | Val::Tok { .. } | Val::ModePair(_)),
        "the via stop lowers colour-form: {via_val:?}"
    );
}

#[test]
fn direction_and_stops_coexist_in_one_program() {
    // The clobber regression: `bg-linear-to-r from-red-500 to-blue-500`
    // must carry ALL THREE families — the direction row's
    // `background-image` Kw AND both stops — where the fold let the
    // last stop replace the direction in the per-longhand cascade.
    let table = baked_table();
    let compiled = compile_theme(
        "bg-linear-to-r from-red-500 to-blue-500",
        &table,
        &Config::default(),
    )
    .expect("the string lowers");
    let pairs = decls_of(&compiled);

    let direction = value_of(&pairs, Prop::BackgroundImage);
    assert!(matches!(direction, Val::Kw(_)), "the direction row's Kw");
    value_of(&pairs, Prop::GradientFrom);
    value_of(&pairs, Prop::GradientTo);
}

#[test]
fn stop_positions_lower_onto_the_position_longhands() {
    let table = baked_table();
    let compiled = compile_theme("from-20% via-30% to-80%", &table, &Config::default())
        .expect("the positions lower");
    let pairs = decls_of(&compiled);

    assert_eq!(value_of(&pairs, Prop::GradientFromPosition), Val::Pct(20.0));
    assert_eq!(value_of(&pairs, Prop::GradientViaPosition), Val::Pct(30.0));
    assert_eq!(value_of(&pairs, Prop::GradientToPosition), Val::Pct(80.0));
}

#[test]
fn the_alpha_modifier_rides_a_stop() {
    // `from-red-500/50` — the modifier rides colour-form decls only,
    // the same contract the colour roots run; the value agrees with
    // `bg-red-500/50`'s.
    let table = baked_table();
    let bg = compile_theme("bg-red-500/50", &table, &Config::default()).expect("bg lowers");
    let from = compile_theme("from-red-500/50", &table, &Config::default()).expect("from lowers");

    assert_eq!(
        value_of(&decls_of(&bg), Prop::Background),
        value_of(&decls_of(&from), Prop::GradientFrom),
        "the stop's alpha bake agrees with the colour root's"
    );
}

#[test]
fn the_themeless_refusal_stays_typed() {
    // No table rides: the 270-entry colour table still exceeds the u8
    // keyword slot, the refusal names it (the documented boundary), and
    // NOTHING folds onto BackgroundImage either.
    let compiled = compile("from-red-500", &Config::default()).expect("compile is total");
    assert!(
        !compiled.diagnostics.is_empty(),
        "the themeless miss is a typed diagnostic: {:?}",
        compiled.diagnostics
    );
}

// ---------------------------------------------------------------------------
// The CSS emit arm
// ---------------------------------------------------------------------------

#[test]
fn the_css_arm_renders_the_v4_custom_property_forms() {
    let table = baked_table();
    let css = emit_css_theme(
        "bg-linear-to-r from-red-500 to-blue-500 from-20%",
        &table,
        &Config::default(),
    )
    .expect("the sheet emits")
    .to_string();

    assert!(
        css.contains("--tw-gradient-from:"),
        "the from var renders: {css}"
    );
    assert!(
        css.contains("--tw-gradient-to:"),
        "the to var renders: {css}"
    );
    assert!(
        css.contains("--tw-gradient-from-position: 20%"),
        "the position var renders: {css}"
    );
    assert!(
        css.contains("background-image: linear-gradient(to right"),
        "the direction row's text renders: {css}"
    );
}

// ---------------------------------------------------------------------------
// The interp arm
// ---------------------------------------------------------------------------

#[test]
fn the_interp_arm_resolves_the_stops() {
    // The interp's effective declarations carry all three families
    // under one env — the emit (c) form of the coexistence pin.
    let table = baked_table();
    let compiled = compile_theme(
        "bg-linear-to-r from-red-500 to-blue-500",
        &table,
        &Config::default(),
    )
    .expect("the string lowers");
    let interped = interp_theme(&compiled.program, &table, &aeolus_compile::Env::default())
        .expect("the walk resolves");

    let props: Vec<Prop> = interped.decls.iter().map(|decl| decl.prop).collect();
    assert!(props.contains(&Prop::BackgroundImage), "{props:?}");
    assert!(props.contains(&Prop::GradientFrom), "{props:?}");
    assert!(props.contains(&Prop::GradientTo), "{props:?}");
}

// ---------------------------------------------------------------------------
// The plan bridge composes — against the v0.x oracle's own gradient
// ---------------------------------------------------------------------------

/// The gradient slots of both generations agree: shape, angle, stops
/// (colours within the quantization tolerance, positions patched or
/// defaulted alike). The differential harness's gradient arm — the
/// comparator the battery did not carry, because the fold had nothing
/// honest to compare.
fn assert_gradients_agree(
    v1: Option<aeolus_core::Gradient>,
    v0: Option<aeolus_core::Gradient>,
    context: &str,
) {
    let (Some(v1), Some(v0)) = (v1, v0) else {
        panic!("{context}: the gradient slots disagree — V1 {v1:?} vs V0 {v0:?}");
    };
    let (v1_stops, v0_stops) = (v1.stops(), v0.stops());
    assert!(
        colors_agree(&v1_stops.from.color, &v0_stops.from.color)
            && colors_agree(&v1_stops.to.color, &v0_stops.to.color),
        "{context}: the stop colours disagree — V1 {v1_stops:?} vs V0 {v0_stops:?}"
    );
    assert_eq!(
        v1_stops.from.pos_pct, v0_stops.from.pos_pct,
        "{context}: the from position disagrees"
    );
    assert_eq!(
        v1_stops.via.is_some(),
        v0_stops.via.is_some(),
        "{context}: the via presence disagrees"
    );
    if let (Some(v1_via), Some(v0_via)) = (v1_stops.via, v0_stops.via) {
        assert!(
            colors_agree(&v1_via.color, &v0_via.color),
            "{context}: the via colour disagrees"
        );
    }
    match (v1, v0) {
        (
            aeolus_core::Gradient::Linear { angle_deg: a1, .. },
            aeolus_core::Gradient::Linear { angle_deg: a0, .. },
        ) => assert_eq!(a1, a0, "{context}: the linear angle disagrees"),
        (aeolus_core::Gradient::None, aeolus_core::Gradient::None) => {}
        (aeolus_core::Gradient::Radial { .. }, aeolus_core::Gradient::Radial { .. }) => {}
        (aeolus_core::Gradient::Conic { .. }, aeolus_core::Gradient::Conic { .. }) => {}
        (v1, v0) => panic!("{context}: the gradient shapes disagree — V1 {v1:?} vs V0 {v0:?}"),
    }
}

#[test]
fn the_bridge_composes_and_agrees_with_the_v0_oracle() {
    // The differential oracle, gradient arm: both generations resolve
    // the SAME string against the SAME theme CSS, the V1 stop longhands
    // compose into the plan slot, and the slots agree shape for shape.
    let theme = TailwindTheme::from_theme_css(TAILWIND_DEFAULT_THEME).unwrap();
    let resolver = aeolus_core::Resolver::new(&theme);
    let source = ThemeSource::from_css(TAILWIND_DEFAULT_THEME, None).unwrap();
    let tokens = TokenTable::baked(&source).unwrap();

    let strings = [
        "bg-linear-to-r from-red-500 to-blue-500",
        "bg-linear-to-tl from-red-500 to-blue-500",
        "bg-linear-to-b from-red-500 from-20% to-blue-500 to-80%",
        "bg-linear-to-r from-red-500 via-white to-blue-500",
        "bg-linear-to-r from-red-500",
        "bg-radial from-red-500 to-blue-500",
        "bg-conic from-red-500 to-blue-500",
        "bg-none",
    ];
    for class in strings {
        let sheet = emit_plan_theme(class, &tokens, &Config::default())
            .unwrap_or_else(|e| panic!("{class}: the emit refused: {e:?}"));
        assert!(
            sheet.diagnostics.is_empty(),
            "{class}: V1 lowered with diagnostics {:?}",
            sheet.diagnostics
        );
        let resolution = resolver.resolve(class).unwrap();
        assert!(
            resolution.warnings.is_empty(),
            "{class}: the v0.x oracle warned {:?}",
            resolution.warnings
        );
        assert_gradients_agree(
            sheet.plan.base.background_gradient,
            resolution.plan.base.background_gradient,
            class,
        );
    }
}

#[test]
fn the_bridge_still_reports_the_uncomposable_text() {
    // The bare `bg-linear` row's text is the `var(--tw-gradient-linear)`
    // placeholder — no direction, nothing to compose; the typed note
    // stays (the honest boundary the note has always named).
    let table = baked_table();
    let sheet = emit_plan_theme("bg-linear", &table, &Config::default()).expect("the sheet emits");
    assert!(
        sheet
            .notes
            .iter()
            .any(|note| matches!(note.kind, PlanNoteKind::GradientRow)),
        "the placeholder text still reports: {:?}",
        sheet.notes
    );
}
