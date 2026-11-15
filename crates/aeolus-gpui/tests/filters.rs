//! The GPUI filter tier's POLYFILL half (architecture plan §3.7's
//! filter row — "the polyfill-wrapper backlog is the remaining half of
//! this row"): the seven colour-matrix filter longhands the registry
//! lowers into (`brightness-*`/`contrast-*`/`grayscale-*`/
//! `hue-rotate-*`/`invert-*`/`saturate-*`/`sepia-*`) compose into a
//! [`FilterState`](aeolus_gpui::FilterState) and recolour the element's
//! own declared colours (the CSS shorthand's sRGB matrices, Filter
//! Effects Level 1), the `drop-shadow-*` presets compose onto GPUI's
//! own `Vec<BoxShadow>` (the pinned `BoxShadow` carries the blur
//! radius the function's third token names), and the bare `filter` /
//! `filter-none` reset arm clears the chain — instead of the silent
//! no-op the un-mapped `_ => style` arm used to be.
//!
//! The spatial half stays the reported boundary: `blur-*` has no GPUI
//! slot and the `backdrop-*` mirrors paint BEHIND the element — the
//! walk skips them (the documented skip, the static gate refuses them
//! typed), the CSS/interp arms carry the real output.
//!
//! The pins walk real compiled programs through the SAME path a
//! running app's `div()` consumes and inspect the applied
//! [`gpui::StyleRefinement`] headless. The programs come from
//! [`ClassCell`] — the same compiler, no capability gate — and the
//! expected colours are the spec matrices' own bytes on the registry
//! palette (each pin quotes its derivation: the f32 op order the
//! accumulator runs, sRGB non-premultiplied, alpha untouched).
//!
//! Palette ground truth: the registry's `red-500` bake is `#fb2c36`
//! (251, 44, 54) — the ring suite's colour.

use aeolus_compile::Config;
use aeolus_gpui::prelude::*;
use aeolus_rt::ClassCell;
use gpui::{px, rgba};

/// The registry's `red-500` bake as the [`gpui::Hsla`] the walk emits —
/// the same `rgba` construction the accumulators run.
fn red_500() -> gpui::Hsla {
    rgba(0xfb2c_36ff).into()
}

/// The `grayscale(1)` bytes on red-500 — the spec luminance row
/// (0.2126/0.7152/0.0722): 0.2126·251 + 0.7152·44 + 0.0722·54 = 88.73
/// → 89. The alpha byte rides untouched (the shorthand's matrices are
/// colour-only).
fn red_gray() -> gpui::Hsla {
    rgba(0x5959_59ff).into()
}

fn program(classes: &str) -> StaticProgram {
    let source = aeolus_rt::ThemeSource::shadcn().expect("the built-in shadcn source parses");
    let table = aeolus_rt::TokenTable::baked(&source).expect("the T0/T1 bake succeeds");
    ClassCell::new()
        .get_theme(classes, &table, &Config::default())
        .expect("the filter demo string lowers clean")
}

fn background_of(classes: &str) -> gpui::Background {
    let style = gpui::StyleRefinement::default().tw(program(classes));
    let Some(gpui::Fill::Color(background)) = style.background else {
        panic!("the solid background reaches the refinement: {classes}");
    };
    background
}

#[test]
fn grayscale_recolours_the_solid_background() {
    assert_eq!(
        background_of("bg-red-500 grayscale").as_solid(),
        Some(red_gray())
    );
    // The identity amount never repainted: `grayscale-0` composes the
    // identity matrix — the bake rides unchanged.
    assert_eq!(
        background_of("bg-red-500 grayscale-0").as_solid(),
        Some(red_500())
    );
}

#[test]
fn grayscale_part_way_interpolates_the_luminance_row() {
    // grayscale(0.5): halfway between the bake and the luminance row —
    // r' = 0.5·251 + 0.5·88.73 = 169.87 → 170, g' = 0.5·44 + 0.5·88.73
    // = 66.37 → 66, b' = 0.5·54 + 0.5·88.73 = 71.37 → 71.
    assert_eq!(
        background_of("bg-red-500 grayscale-50").as_solid(),
        Some(rgba(0xaa42_47ff).into())
    );
}

#[test]
fn each_colour_matrix_land_its_spec_bytes() {
    // invert(1): c' = 1 − c — the complement channels (4, 211, 201).
    assert_eq!(
        background_of("bg-red-500 invert").as_solid(),
        Some(rgba(0x04d3_c9ff).into())
    );
    // brightness(0.5): c' = c·a — (126, 22, 27).
    assert_eq!(
        background_of("bg-red-500 brightness-50").as_solid(),
        Some(rgba(0x7e16_1bff).into())
    );
    // brightness(2): the clamp catches the overshoot — (255, 88, 108).
    assert_eq!(
        background_of("bg-red-500 brightness-200").as_solid(),
        Some(rgba(0xff58_6cff).into())
    );
    // contrast(0): c' = 0.5 — the mid-gray pivot (128, 128, 128).
    assert_eq!(
        background_of("bg-red-500 contrast-0").as_solid(),
        Some(rgba(0x8080_80ff).into())
    );
    // saturate(0): the spec's own desaturation row (0.213/0.715/0.072)
    // — (89, 89, 89) on this bake.
    assert_eq!(
        background_of("bg-red-500 saturate-0").as_solid(),
        Some(red_gray())
    );
    // sepia(1): the spec's fixed matrix — (143, 127, 99).
    assert_eq!(
        background_of("bg-red-500 sepia").as_solid(),
        Some(rgba(0x8f7f_63ff).into())
    );
    // hue-rotate(90deg): the spec's cos/sin matrix — (54, 116, 0).
    assert_eq!(
        background_of("bg-red-500 hue-rotate-90").as_solid(),
        Some(rgba(0x3674_00ff).into())
    );
}

#[test]
fn the_chain_composes_in_the_var_canonical_order() {
    // `grayscale invert`: v4's var order puts invert AFTER grayscale —
    // the complement of the gray, not the gray of the complement
    // (166, 166, 166). The cascade's last-write-wins per primitive and
    // the canonical application order together make the source order
    // of the two classes irrelevant: the flipped spelling reads the
    // same bytes.
    assert_eq!(
        background_of("bg-red-500 grayscale invert").as_solid(),
        Some(rgba(0xa6a6_a6ff).into())
    );
    assert_eq!(
        background_of("bg-red-500 invert grayscale").as_solid(),
        Some(rgba(0xa6a6_a6ff).into())
    );
}

#[test]
fn the_text_and_border_slots_recolour_alongside_the_background() {
    // The chain recolours EVERY colour the walk composes — the text
    // colour (Prop::Color) and the border colour ride the same
    // matrices (CSS filters apply to the element's rendered paint).
    let style = gpui::StyleRefinement::default()
        .tw(program("bg-red-500 text-red-500 border-red-500 grayscale"));
    let Some(gpui::Fill::Color(background)) = style.background else {
        panic!("the solid background reaches the refinement");
    };
    assert_eq!(background.as_solid(), Some(red_gray()));
    assert_eq!(style.text.color, Some(red_gray()));
    assert_eq!(style.border_color, Some(red_gray()));
}

#[test]
fn drop_shadow_presets_compose_onto_the_box_shadow_list() {
    // drop-shadow-md: `drop-shadow(0 3px 3px rgb(0 0 0 / 0.12))` — the
    // offset/blur tokens land on GPUI's own band (no spread, not
    // inset), the alpha byte the audited text's rounding (0.12·255 =
    // 30.6 → 31).
    let style = gpui::StyleRefinement::default().tw(program("drop-shadow-md"));
    let shadows = style.box_shadow.expect("the drop shadow band lands");
    assert_eq!(shadows.len(), 1, "one band: {shadows:?}");
    assert_eq!(shadows[0].offset, gpui::point(px(0.0), px(3.0)));
    assert_eq!(shadows[0].blur_radius, px(3.0));
    assert_eq!(shadows[0].spread_radius, px(0.0));
    assert!(!shadows[0].inset);
    assert_eq!(shadows[0].color, rgba(0x0000_001f).into());
}

#[test]
fn bare_drop_shadow_composes_its_two_default_bands() {
    // The bare `drop-shadow` utility: v4's `--drop-shadow` token is the
    // TWO-band composition (0 1px 2px 0.1, 0 1px 1px 0.06) — both
    // bands land, in the token's own order.
    let style = gpui::StyleRefinement::default().tw(program("drop-shadow"));
    let shadows = style.box_shadow.expect("the default bands land");
    assert_eq!(shadows.len(), 2, "the token's two bands: {shadows:?}");
    assert_eq!(shadows[0].blur_radius, px(2.0));
    assert_eq!(shadows[0].color, rgba(0x0000_001a).into());
    assert_eq!(shadows[1].blur_radius, px(1.0));
    assert_eq!(shadows[1].color, rgba(0x0000_000f).into());
}

#[test]
fn drop_shadow_bands_ride_beside_the_shadow_presets() {
    // The shadow tier's preset bands and the filter's drop-shadow
    // bands compose into ONE list — the shadow bands first (the
    // `--tw-shadow` machinery), the drop-shadow bands after. The
    // recolour (`shadow-red-500`) recolours the shadow tier's bands
    // only (v4's `--tw-shadow-colored` reaches `--tw-shadow`, never
    // the drop-shadow functions).
    let style =
        gpui::StyleRefinement::default().tw(program("shadow-lg shadow-red-500 drop-shadow-md"));
    let shadows = style.box_shadow.expect("both tiers compose");
    assert_eq!(
        shadows.len(),
        3,
        "two preset bands + the drop band: {shadows:?}"
    );
    assert_eq!(shadows[0].color, red_500(), "the recoloured preset band");
    assert_eq!(shadows[2].color, rgba(0x0000_001f).into(), "the drop band");
}

#[test]
fn the_matrix_recolours_the_composed_shadow_and_ring_bands() {
    // CSS filters recolour the element's painted shadows too: the
    // shadow tier's recolour lands red-500, the matrix grays it
    // (89, 89, 89). The ring band (spread-only) rides the same read.
    let style = gpui::StyleRefinement::default().tw(program("shadow-lg shadow-red-500 grayscale"));
    let shadows = style.box_shadow.expect("the preset bands land");
    for shadow in &shadows {
        assert_eq!(shadow.color, red_gray(), "the grayscaled band");
    }
    let style = gpui::StyleRefinement::default().tw(program("ring-2 ring-red-500 grayscale"));
    let shadows = style.box_shadow.expect("the ring band lands");
    assert_eq!(shadows[0].color, red_gray(), "the grayscaled ring band");
}

#[test]
fn the_matrix_recolours_the_gradient_stops() {
    // The gradient's stops are the element's paint: from-red-500 grays
    // to the luminance row, to-white stays white (the luminance row
    // maps white to white). The composed Background compares EQUAL to
    // the one the walk would compose from the transformed stops —
    // angle, positions and colour space ride unchanged.
    let style = gpui::StyleRefinement::default()
        .tw(program("bg-linear-to-r from-red-500 to-white grayscale"));
    let Some(gpui::Fill::Color(background)) = style.background else {
        panic!("the gradient reaches the refinement");
    };
    let expected = gpui::linear_gradient(
        90.0,
        gpui::linear_color_stop(gpui::Hsla::from(rgba(0x5959_59ff)), 0.0),
        gpui::linear_color_stop(gpui::Hsla::from(rgba(0xffff_ffff)), 1.0),
    );
    assert_eq!(background, expected, "{background:?}");
}

#[test]
fn the_reset_arm_clears_the_chain_and_the_bare_root_redeclares_it() {
    // `filter-none` is the reset the cascade reads — the bake rides
    // unfiltered, the way v4's own cascade reads the reset after the
    // primitives.
    assert_eq!(
        background_of("bg-red-500 grayscale filter-none").as_solid(),
        Some(red_500())
    );
    // The bare `filter` root is v4's chain rule re-declared (the
    // all-empty var chain): the primitives' vars persist beside it,
    // so the chain stays live — the grayscale rides.
    assert_eq!(
        background_of("bg-red-500 grayscale filter").as_solid(),
        Some(red_gray())
    );
    // A chain rule AFTER the reset in the SOURCE re-arms it — but the
    // pipeline's registry order freezes the composed root AFTER the
    // primitives (the `filter` row is the family's tail), so the reset
    // is the winning `filter` decl regardless of the source order —
    // the same last-wins read the interp's cascade and the CSS rule
    // order run (one semantics, three walks).
    assert_eq!(
        background_of("bg-red-500 filter-none grayscale").as_solid(),
        Some(red_500())
    );
    // The reset clears the drop-shadow bands too (the same chain).
    let style = gpui::StyleRefinement::default().tw(program("drop-shadow-md filter-none"));
    assert!(style.box_shadow.is_none(), "the reset cleared the band");
}

#[test]
fn the_spatial_and_backdrop_halves_stay_the_documented_skip() {
    // The walk skips the spatial and backdrop longhands (the static
    // gate refuses them typed; a cell-compiled program reaching the
    // walk degrades to the skip, never a wrong paint): the bake rides
    // UNfiltered, no band lands.
    assert_eq!(
        background_of("bg-red-500 blur-md").as_solid(),
        Some(red_500())
    );
    assert_eq!(
        background_of("bg-red-500 backdrop-brightness-50").as_solid(),
        Some(red_500())
    );
    let style = gpui::StyleRefinement::default().tw(program("backdrop-blur-md"));
    assert!(
        style.box_shadow.is_none(),
        "the backdrop family paints nothing"
    );
}

#[test]
fn the_unit_surface_composes_the_chain_alone() {
    // The accumulator's own surface (the ring bench's unit pin): the
    // seven matrix longhands record, the drop-shadow bands record, and
    // the chain recolours an arbitrary colour — the same bytes the
    // walk pins assert.
    use aeolus_ir::{Prop, Val};

    let mut state = FilterState::default();
    state.apply(Prop::FilterGrayscale, Val::Num(100.0), "");
    assert_eq!(
        state.shade(red_500()),
        Some(red_gray()),
        "grayscale alone: the luminance row"
    );
    state.apply(Prop::FilterInvert, Val::Num(100.0), "");
    assert_eq!(
        state.shade(red_gray()),
        Some(rgba(0xa6a6_a6ff).into()),
        "then invert: the complement of the gray"
    );
    assert_eq!(
        state.shade(red_500()),
        Some(rgba(0xa6a6_a6ff).into()),
        "the composed chain: grayscale then invert, source-independent"
    );

    let mut bands = FilterState::default();
    bands.apply(
        Prop::FilterDropShadow,
        Val::Kw(0),
        "drop-shadow(0 3px 3px rgb(0 0 0 / 0.12))",
    );
    let shadows = bands.drop_shadows().expect("the audited preset parses");
    assert_eq!(shadows.len(), 1);
    assert_eq!(shadows[0].blur_radius, px(3.0));
}

#[test]
fn the_empty_chain_is_an_identity() {
    // No filter class in the program: the walk composes NOTHING new —
    // the refinement equals the one the pre-filter walk produced (the
    // empty-registration byte-identity the custom seam pins, on this
    // tier).
    let style = gpui::StyleRefinement::default().tw(program("bg-red-500"));
    let Some(gpui::Fill::Color(background)) = style.background else {
        panic!("the solid background reaches the refinement");
    };
    assert_eq!(background.as_solid(), Some(red_500()));
    assert!(style.box_shadow.is_none());
}
