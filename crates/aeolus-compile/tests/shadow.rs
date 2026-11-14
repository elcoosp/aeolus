//! The plan bridge's shadow slot (the §3.7 shadow row's plan half, the
//! architecture plan §7 migration seam): the `shadow-*` families the
//! registry lowers into `Prop::BoxShadow`/`Prop::ShadowColor` land in
//! the v0.x plan's own shadow slots — `base.shadow` (the dominant
//! layer), `extras.shadow_rest` (the multi-layer rest) — instead of the
//! typed note the preset text used to take.
//!
//! The semantics are the ORACLE'S OWN (the differential rule "new
//! compiler == old resolver", pinned against `Resolver` below): the
//! preset replaces the slot wholesale (last preset wins, order-insensitive
//! against the recolour — the v0.x bucket's scalar-extras pass runs
//! after the shadow-colour patch), the `shadow-{color}` recolour patches
//! the layers ALREADY in the slot (the `shadow-none` clear), and a
//! recolour without a preset composes nothing. The bands come from the
//! audited v4 default theme — the same texts the CSS emit arm renders
//! and the GPUI lane quotes (one source of truth across the three
//! consumers, pinned against the oracle's own theme reader).
//!
//! The inset placeholder (`shadow-inner`'s `inset 0 0 0 0 currentColor`)
//! keeps the typed note: its `currentColor` has no plan answer (the
//! oracle's own shadow-inner errors — v0.x carries the real inset
//! families on `--inset-shadow-*`), so the honest report stands.

use aeolus_compile::{Config, PlanNoteKind, emit_plan_theme};
use aeolus_core::{Resolver, TAILWIND_DEFAULT_THEME, TailwindTheme};
use aeolus_theme::{ThemeSource, TokenTable};

fn tokens() -> TokenTable {
    let source = ThemeSource::from_css(TAILWIND_DEFAULT_THEME, None).unwrap();
    TokenTable::baked(&source).unwrap()
}

fn oracle() -> Resolver<'static> {
    // The theme leaks into the resolver's borrowed parameter — the
    // test's own static, fine for a pinned fixture.
    let theme: &'static TailwindTheme = Box::leak(Box::new(
        TailwindTheme::from_theme_css(TAILWIND_DEFAULT_THEME).unwrap(),
    ));
    Resolver::new(theme)
}

/// The audited v4 shadow preset bands (the theme file's own values, as
/// the oracle's theme reader parses them).
fn oracle_bands(name: &str) -> Vec<aeolus_core::plan::BoxShadow> {
    let theme = TailwindTheme::from_theme_css(TAILWIND_DEFAULT_THEME).unwrap();
    let specs = theme.shadows(name);
    assert!(!specs.is_empty(), "--shadow-{name} parses");
    specs
        .iter()
        .map(|spec| aeolus_core::plan::BoxShadow {
            offset_x: spec.offset_x,
            offset_y: spec.offset_y,
            blur: spec.blur,
            spread: spec.spread,
            color: spec.color,
            inset: spec.inset,
        })
        .collect()
}

#[test]
fn the_shadow_preset_lands_in_the_plan_slots() {
    let sheet = emit_plan_theme("shadow-lg", &tokens(), &Config::default()).unwrap();
    assert!(sheet.diagnostics.is_empty(), "{:?}", sheet.diagnostics);
    assert!(
        sheet.notes.is_empty(),
        "the preset maps now: {:?}",
        sheet.notes
    );
    let bands = oracle_bands("lg");
    assert_eq!(
        sheet.plan.base.shadow.as_ref().expect("the dominant layer"),
        &bands[0],
        "the first band lands in base.shadow"
    );
    assert_eq!(
        sheet.plan.base.extras.shadow_rest,
        bands[1..],
        "the rest land in shadow_rest"
    );
}

#[test]
fn the_full_preset_scale_lands() {
    for (class, name) in [
        ("shadow-2xs", "2xs"),
        ("shadow-xs", "xs"),
        ("shadow-sm", "sm"),
        ("shadow-md", "md"),
        ("shadow-xl", "xl"),
        ("shadow-2xl", "2xl"),
    ] {
        let sheet = emit_plan_theme(class, &tokens(), &Config::default()).unwrap();
        assert!(sheet.diagnostics.is_empty(), "{class}");
        assert!(sheet.notes.is_empty(), "{class}: {:?}", sheet.notes);
        let bands = oracle_bands(name);
        assert_eq!(
            sheet
                .plan
                .base
                .shadow
                .as_ref()
                .unwrap_or_else(|| panic!("{class}")),
            &bands[0],
            "{class} dominant band"
        );
        assert_eq!(
            sheet.plan.base.extras.shadow_rest,
            bands[1..],
            "{class} rest"
        );
    }
}

#[test]
fn the_clear_lands_the_v4_transparent_band() {
    let sheet = emit_plan_theme("shadow-none", &tokens(), &Config::default()).unwrap();
    assert!(sheet.diagnostics.is_empty());
    assert!(sheet.notes.is_empty(), "{:?}", sheet.notes);
    // The oracle's own shadow-none shape: one zero layer, transparent.
    let clear = sheet.plan.base.shadow.expect("the clear lands");
    assert_eq!(
        (clear.offset_x, clear.offset_y, clear.blur, clear.spread),
        (0.0, 0.0, 0.0, 0.0)
    );
    assert_eq!(clear.color, aeolus_core::Color::TRANSPARENT);
    assert!(sheet.plan.base.extras.shadow_rest.is_empty());
}

#[test]
fn the_last_preset_wins_the_cascade() {
    // `shadow-sm shadow-lg`: the later preset replaces the earlier one
    // wholesale (the per-slot cascade the oracle's scalar-extras pass
    // runs).
    let sheet = emit_plan_theme("shadow-sm shadow-lg", &tokens(), &Config::default()).unwrap();
    let bands = oracle_bands("lg");
    assert_eq!(sheet.plan.base.shadow.as_ref().expect("lg"), &bands[0]);
    assert_eq!(sheet.plan.base.extras.shadow_rest, bands[1..]);
}

#[test]
fn the_recolour_does_not_survive_a_preset() {
    // The ORACLE'S OWN semantics (pinned both orders): the shadow
    // colour patches the layers already in the slot, and the preset's
    // wholesale replace runs after — so a preset + recolour pair
    // carries the plain preset bands. (v4's `--tw-shadow-colored` is
    // the GPUI lane's own v4-true composition; the plan is the
    // oracle-shaped artifact, the divergence documented, not hidden.)
    for class in ["shadow-lg shadow-red-500", "shadow-red-500 shadow-lg"] {
        let sheet = emit_plan_theme(class, &tokens(), &Config::default()).unwrap();
        assert!(sheet.diagnostics.is_empty(), "{class}");
        assert!(sheet.notes.is_empty(), "{class}: {:?}", sheet.notes);
        let bands = oracle_bands("lg");
        assert_eq!(
            sheet.plan.base.shadow.as_ref().expect("{class}"),
            &bands[0],
            "{class}: the plain preset, the recolour not applied"
        );
    }
}

#[test]
fn the_recolour_patches_the_clear() {
    // `shadow-none shadow-red-500` (both orders): the transparent band
    // is already in the slot when the colour patches it — the oracle's
    // own observable shape.
    for class in ["shadow-none shadow-red-500", "shadow-red-500 shadow-none"] {
        let sheet = emit_plan_theme(class, &tokens(), &Config::default()).unwrap();
        assert!(sheet.diagnostics.is_empty(), "{class}");
        assert!(sheet.notes.is_empty(), "{class}: {:?}", sheet.notes);
        let clear = sheet.plan.base.shadow.expect("{class}");
        assert_eq!(
            (clear.offset_x, clear.offset_y, clear.blur, clear.spread),
            (0.0, 0.0, 0.0, 0.0)
        );
        let expected = oracle()
            .resolve("shadow-none shadow-red-500")
            .unwrap()
            .plan
            .base
            .shadow
            .expect("the oracle's recoloured clear");
        // The recolour agrees within the baked-table quantization
        // (the bridge's colour rides the theme pass's [u8; 4] bake —
        // the harness's own ±0.02/axis tolerance).
        assert!(
            (clear.color.l - expected.color.l).abs() <= 0.02
                && (clear.color.a - expected.color.a).abs() <= 0.02
                && (clear.color.b - expected.color.b).abs() <= 0.02
                && (clear.color.alpha - expected.color.alpha).abs() <= 0.02,
            "{class} recolour: V1 {:?} vs V0 {:?}",
            clear.color,
            expected.color
        );
    }
}

#[test]
fn a_recolour_without_a_preset_composes_nothing() {
    // v4's `--tw-shadow-color` colours an EMPTY shadow: the slot stays
    // empty (the oracle's own shape), the class carries no note.
    let sheet = emit_plan_theme("shadow-red-500", &tokens(), &Config::default()).unwrap();
    assert!(sheet.diagnostics.is_empty(), "{:?}", sheet.diagnostics);
    assert!(sheet.notes.is_empty(), "{:?}", sheet.notes);
    assert!(sheet.plan.base.shadow.is_none());
}

#[test]
fn the_variant_layers_carry_the_preset() {
    // `hover:shadow-lg`: the variant bucket's own shadow — the same
    // slot routing every other longhand takes.
    let sheet = emit_plan_theme("hover:shadow-lg", &tokens(), &Config::default()).unwrap();
    assert!(sheet.diagnostics.is_empty());
    assert!(sheet.notes.is_empty());
    let bands = oracle_bands("lg");
    let layer = sheet.plan.layers.first().expect("the hover bucket lowers");
    assert_eq!(
        layer.style.shadow.as_ref().expect("the bucket's preset"),
        &bands[0]
    );
}

#[test]
fn the_inset_placeholder_keeps_the_typed_note() {
    // `shadow-inner`'s `inset 0 0 0 0 currentColor`: the plan has no
    // `currentColor` answer (the oracle's own shadow-inner errors), so
    // the honest note stands — the CSS arm renders the real output.
    let sheet = emit_plan_theme("shadow-inner", &tokens(), &Config::default()).unwrap();
    assert!(sheet.diagnostics.is_empty(), "the class itself lowered");
    assert_eq!(sheet.notes.len(), 1, "one note for the placeholder");
    assert!(matches!(sheet.notes[0].kind, PlanNoteKind::ShadowPreset));
}

#[test]
fn the_shadow_families_agree_with_the_v0_resolver() {
    // The differential battery's shadow rows: the bridge's plan and the
    // oracle's plan agree slot for slot on the whole preset scale, the
    // clear, and the recolour interactions.
    let resolver = oracle();
    for class in [
        "shadow-2xs",
        "shadow-xs",
        "shadow-sm",
        "shadow-md",
        "shadow-lg",
        "shadow-xl",
        "shadow-2xl",
        "shadow-none",
        "shadow-lg shadow-red-500",
        "shadow-red-500 shadow-lg",
        "shadow-none shadow-red-500",
        "shadow-red-500 shadow-none",
        "shadow-sm shadow-lg",
        "hover:shadow-lg",
        "dark:shadow-2xl",
    ] {
        let sheet = emit_plan_theme(class, &tokens(), &Config::default())
            .unwrap_or_else(|e| panic!("{class}: the emit refused: {e:?}"));
        assert!(
            sheet.diagnostics.is_empty(),
            "{class}: V1 lowered with diagnostics {:?}",
            sheet.diagnostics
        );
        let resolution = resolver.resolve(class).unwrap();
        assert_plans_agree(&sheet.plan, &resolution.plan, class);
    }
}

/// The shadow slots of two plans agree: the dominant layer field-wise
/// (colours within the baked-table quantization tolerance), the
/// multi-layer rest, the inset set.
fn assert_plans_agree(v1: &aeolus_core::StylePlan, v0: &aeolus_core::StylePlan, class: &str) {
    let check = |a: &aeolus_core::plan::ResolvedStyle,
                 b: &aeolus_core::plan::ResolvedStyle,
                 bucket: &str| {
        match (&a.shadow, &b.shadow) {
            (None, None) => {}
            (Some(x), Some(y)) => {
                assert_eq!(x.offset_x, y.offset_x, "{class} {bucket} shadow offset_x");
                assert_eq!(x.offset_y, y.offset_y, "{class} {bucket} shadow offset_y");
                assert_eq!(x.blur, y.blur, "{class} {bucket} shadow blur");
                assert_eq!(x.spread, y.spread, "{class} {bucket} shadow spread");
                assert_eq!(x.inset, y.inset, "{class} {bucket} shadow inset");
                assert!(
                    (x.color.l - y.color.l).abs() <= 0.02
                        && (x.color.a - y.color.a).abs() <= 0.02
                        && (x.color.b - y.color.b).abs() <= 0.02
                        && (x.color.alpha - y.color.alpha).abs() <= 0.02,
                    "{class} {bucket} shadow colour: V1 {x:?} vs V0 {y:?}"
                );
            }
            (x, y) => panic!("{class} {bucket} shadow presence disagrees: {x:?} vs {y:?}"),
        }
        assert_eq!(
            a.extras.shadow_rest.len(),
            b.extras.shadow_rest.len(),
            "{class} {bucket} shadow_rest count"
        );
        for (i, (x, y)) in a
            .extras
            .shadow_rest
            .iter()
            .zip(b.extras.shadow_rest.iter())
            .enumerate()
        {
            assert_eq!(
                (x.offset_x, x.offset_y, x.blur, x.spread, x.inset),
                (y.offset_x, y.offset_y, y.blur, y.spread, y.inset),
                "{class} {bucket} shadow_rest [{i}] geometry"
            );
        }
        assert_eq!(
            a.extras.inset_shadows.len(),
            b.extras.inset_shadows.len(),
            "{class} {bucket} inset_shadows count"
        );
    };
    check(&v1.base, &v0.base, "base");
    for (i, (a, b)) in v1.layers.iter().zip(v0.layers.iter()).enumerate() {
        check(&a.style, &b.style, &format!("layer {i}"));
    }
}
