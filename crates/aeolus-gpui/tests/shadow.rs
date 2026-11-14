//! The §3.7 shadow tier's WALK half (architecture plan §3.7's shadow
//! row — "`shadow-*` (multi-layer v4 presets) → `box_shadow:
//! Vec<BoxShadow>` → direct"): the `shadow-*` families the registry
//! lowers into [`Prop::BoxShadow`]/[`Prop::ShadowColor`] compose into
//! GPUI's native `Vec<BoxShadow>` through the [`ShadowState`]
//! accumulator (the `RingState` precedent) instead of the silent no-op
//! the un-mapped `_ => style` arm used to be.
//!
//! The pins walk real `tw!`-compiled programs (the same path a running
//! app's `div()` consumes) and inspect the applied
//! [`gpui::StyleRefinement`] headless, beside unit pins on the lane's
//! own parse and a differential pin against the audited theme file the
//! band texts quote.

use aeolus_gpui::prelude::*;
use gpui::{px, rgba};

/// The v4 default theme file the band texts quote verbatim — the
/// differential pin's ground truth (the same file the CSS emit arm
/// renders `var(--tw-shadow-*)` references from and the plan bridge
/// resolves presets through the oracle's own theme reader).
const DEFAULT_THEME: &str = include_str!("../../aeolus-core/data/tailwind-default-theme.css");

/// The band colour the audited texts carry: pure black at `alpha` —
/// `rgb(0 0 0 / a)` baked through the same `gpui::rgba` construction
/// the parser runs (u8 rounding on the alpha byte, half away from
/// zero).
fn black(alpha: f32) -> gpui::Hsla {
    rgba(u32::from_be_bytes([0, 0, 0, (alpha * 255.0).round() as u8])).into()
}

/// The `--shadow-{name}` value the audited theme file carries (the
/// test-local scanner: token name → value text up to the `;`).
fn theme_value(name: &str) -> &'static str {
    let needle = format!("  --shadow-{name}: ");
    let start = DEFAULT_THEME
        .find(&needle)
        .unwrap_or_else(|| panic!("{needle} missing from the audited theme file"))
        + needle.len();
    let rest = &DEFAULT_THEME[start..];
    let end = rest.find(';').expect("the token terminates");
    // The `&'static str` the include_str! slice hands out.
    let full: &'static str = DEFAULT_THEME;
    let byte_start = start;
    &full[byte_start..byte_start + end]
}

#[test]
fn the_preset_texts_quote_the_audited_theme_file() {
    // The lane's closed-world band texts are the audited theme's own
    // values — one source of truth, pinned byte for byte (a theme-file
    // bump that forgets the lane fails here, the same drift gate the
    // registry tables run).
    for (name, text) in aeolus_gpui::SHADOW_PRESET_TEXTS {
        assert_eq!(*text, theme_value(name), "--shadow-{name} drifts");
    }
}

#[test]
fn shadow_lg_composes_the_two_audited_bands() {
    // `--shadow-lg: 0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px
    // rgb(0 0 0 / 0.1)` — the two layers, first-paints-on-top order
    // preserved.
    let style = gpui::StyleRefinement::default().tw(tw!("shadow-lg"));
    let shadows = style
        .box_shadow
        .expect("the shadow preset lowers into box shadows");
    assert_eq!(shadows.len(), 2, "the lg preset's two bands: {shadows:?}");
    assert_eq!(shadows[0].offset.x, px(0.0));
    assert_eq!(shadows[0].offset.y, px(10.0));
    assert_eq!(shadows[0].blur_radius, px(15.0));
    assert_eq!(shadows[0].spread_radius, px(-3.0));
    assert!(!shadows[0].inset);
    assert_eq!(shadows[0].color, black(0.1));
    assert_eq!(shadows[1].offset.y, px(4.0));
    assert_eq!(shadows[1].blur_radius, px(6.0));
    assert_eq!(shadows[1].spread_radius, px(-4.0));
    assert_eq!(shadows[1].color, black(0.1));
}

#[test]
fn the_full_preset_scale_composes() {
    // The band counts across the audited scale (2xs/xs/2xl carry one
    // band, sm..xl two) with each scale step's own geometry.
    let style = gpui::StyleRefinement::default().tw(tw!("shadow-2xs"));
    assert_eq!(style.box_shadow.expect("2xs").len(), 1);
    let style = gpui::StyleRefinement::default().tw(tw!("shadow-xs"));
    assert_eq!(style.box_shadow.expect("xs").len(), 1);
    let style = gpui::StyleRefinement::default().tw(tw!("shadow-2xl"));
    assert_eq!(style.box_shadow.expect("2xl").len(), 1);
    for class in ["shadow-sm", "shadow-md", "shadow-xl"] {
        let shadows = match class {
            "shadow-sm" => gpui::StyleRefinement::default().tw(tw!("shadow-sm")),
            "shadow-md" => gpui::StyleRefinement::default().tw(tw!("shadow-md")),
            _ => gpui::StyleRefinement::default().tw(tw!("shadow-xl")),
        }
        .box_shadow
        .expect(class);
        assert_eq!(shadows.len(), 2, "{class}: {shadows:?}");
    }
    // Each scale step's own geometry, spot-checked.
    let style = gpui::StyleRefinement::default().tw(tw!("shadow-md"));
    let shadows = style.box_shadow.expect("md");
    assert_eq!(shadows[0].offset.y, px(4.0));
    assert_eq!(shadows[0].blur_radius, px(6.0));
    let style = gpui::StyleRefinement::default().tw(tw!("shadow-xl"));
    let shadows = style.box_shadow.expect("xl");
    assert_eq!(shadows[0].offset.y, px(20.0));
    assert_eq!(shadows[0].blur_radius, px(25.0));
    // The 2xl colour is the audited 0.25 alpha.
    let style = gpui::StyleRefinement::default().tw(tw!("shadow-2xl"));
    let shadows = style.box_shadow.expect("2xl");
    assert_eq!(shadows[0].color, black(0.25));
}

#[test]
fn shadow_none_clears_to_the_v4_transparent_band() {
    // v4's own output: `box-shadow: 0 0 #0000` — one transparent band,
    // the composed surface the cascade reads (a later `shadow-lg`
    // replaces it, an earlier one it replaces).
    let style = gpui::StyleRefinement::default().tw(tw!("shadow-none"));
    let shadows = style
        .box_shadow
        .expect("shadow-none lands the v4 transparent band");
    assert_eq!(shadows.len(), 1);
    assert_eq!(shadows[0].offset.x, px(0.0));
    assert_eq!(shadows[0].blur_radius, px(0.0));
    assert_eq!(shadows[0].color, rgba(0x0000_0000).into(), "#0000");
}

#[test]
fn shadow_inner_composes_the_inset_band_in_the_text_colour() {
    // The pinned `BoxShadow` carries the `inset` flag, so v4's
    // `inset 0 0 0 0 currentColor` placeholder composes natively — the
    // band's `currentColor` reads the element's own text colour, the
    // same cascade the ring's `currentColor` default walks.
    let style = gpui::StyleRefinement::default().tw(tw!("shadow-inner text-red-500"));
    let shadows = style.box_shadow.expect("the inset placeholder composes");
    assert_eq!(shadows.len(), 1);
    assert!(shadows[0].inset, "the inset flag rides");
    assert_eq!(shadows[0].offset.y, px(0.0));
    assert_eq!(shadows[0].color, rgba(0xfb2c_36ff).into(), "currentColor");
}

#[test]
fn the_cascade_is_last_write_wins() {
    // `shadow-sm shadow-lg`: the later preset replaces the earlier one
    // wholesale (the per-longhand cascade every other family runs).
    let style = gpui::StyleRefinement::default().tw(tw!("shadow-sm shadow-lg"));
    let shadows = style.box_shadow.expect("the winner composes");
    assert_eq!(shadows.len(), 2, "the lg preset's two bands, not sm's");
    assert_eq!(shadows[0].offset.y, px(10.0), "lg's geometry");
}

#[test]
fn the_shadow_color_recolors_the_preset_v4_true() {
    // v4's `--tw-shadow-colored` machinery: `shadow-red-500` replaces
    // every band colour with the shadow colour (the band's own black
    // never leaks), the alpha riding the recolour itself. The theme
    // pass bakes the token (`red-500` = #fb2c36, the audited
    // gamut-mapped value) before the walk sees it.
    let style = gpui::StyleRefinement::default().tw(tw!("shadow-lg shadow-red-500"));
    let shadows = style.box_shadow.expect("the recoloured preset composes");
    assert_eq!(shadows.len(), 2, "the bands keep their geometry");
    assert_eq!(shadows[0].offset.y, px(10.0));
    assert_eq!(
        shadows[0].color,
        rgba(0xfb2c_36ff).into(),
        "the recolour replaces the band black"
    );
    assert_eq!(shadows[1].color, rgba(0xfb2c_36ff).into());
}

#[test]
fn the_shadow_color_modifier_rides_the_recolour() {
    // `shadow-red-500/50`: the premultiplied `/opacity` identity bakes
    // the alpha into the token's colour (mix-first-then-map through the
    // gamut, the theme pass's own read) before the walk sees it — the
    // recolour carries THAT baked colour. The expected value reads the
    // same bake the compile freezes (the walk renders the bake, the
    // pin's the alignment between the two halves).
    let source = aeolus_rt::ThemeSource::shadcn().expect("the built-in shadcn source parses");
    let table = aeolus_rt::TokenTable::baked(&source).expect("the built-in bake");
    let compiled = aeolus_compile::compile_theme(
        "shadow-lg shadow-red-500/50",
        &table,
        &aeolus_compile::Config::default(),
    )
    .expect("the shadow families lower");
    let baked = compiled
        .program
        .layers
        .iter()
        .flat_map(|layer| layer.decls.iter())
        .find_map(|decl| match decl.val {
            aeolus_ir::Val::Rgba(rgba_bytes) if decl.prop == aeolus_ir::Prop::ShadowColor => {
                Some(rgba_bytes)
            }
            _ => None,
        })
        .expect("the recolour bakes to a colour");
    let expected = rgba(u32::from_be_bytes(baked)).into();
    let style = gpui::StyleRefinement::default().tw(tw!("shadow-lg shadow-red-500/50"));
    let shadows = style.box_shadow.expect("the recoloured preset composes");
    assert_eq!(shadows[0].color, expected, "the /50 alpha rides the bake");
    assert_eq!(shadows[1].color, expected);
}

#[test]
fn a_shadow_color_without_a_preset_composes_nothing() {
    // v4's `--tw-shadow-color` alone colors an EMPTY shadow — nothing
    // paints, and an empty state never touches the refinement (the
    // `ring-0` contract).
    let style = gpui::StyleRefinement::default().tw(tw!("shadow-red-500"));
    assert!(
        style.box_shadow.is_none(),
        "a recolour without bands paints nothing: {:?}",
        style.box_shadow
    );
}

#[test]
fn ring_bands_and_shadow_bands_compose_one_list() {
    // GPUI's `shadow()` replaces the whole `Vec<BoxShadow>` — the two
    // accumulators compose in ONE call, v4's list order (the ring
    // machinery's bands first, the shadow preset's bands after — the
    // first layer paints on top, so the ring sits over the preset's
    // inner edge exactly as v4's `box-shadow:` list reads).
    let style = gpui::StyleRefinement::default().tw(tw!("shadow-lg ring-2 ring-red-500"));
    let shadows = style.box_shadow.expect("the merged list composes");
    assert_eq!(shadows.len(), 3, "the ring band + the two lg bands");
    assert_eq!(shadows[0].spread_radius, px(2.0), "the ring band first");
    assert_eq!(shadows[0].color, rgba(0xfb2c_36ff).into());
    assert_eq!(shadows[1].offset.y, px(10.0), "the preset's bands after");
    assert_eq!(shadows[1].color, black(0.1));
}

#[test]
fn the_lane_parses_the_closed_world_texts() {
    // The unit surface: the preset references resolve through the
    // audited table, the structural forms parse directly, an unknown
    // text is the honest `None` (the walk has no note channel — the
    // documented skip).
    assert_eq!(
        aeolus_gpui::parse_shadow_bands("var(--tw-shadow-2xl)")
            .expect("the preset reference resolves")
            .len(),
        1
    );
    assert_eq!(
        aeolus_gpui::parse_shadow_bands("0 0 #0000")
            .expect("the clear parses")
            .len(),
        1
    );
    assert!(
        aeolus_gpui::parse_shadow_bands("inset 0 0 0 0 currentColor").is_some(),
        "the inset placeholder parses"
    );
    assert_eq!(
        aeolus_gpui::parse_shadow_bands("definitely not a shadow text"),
        None,
        "an unknown text is the honest skip"
    );
}
