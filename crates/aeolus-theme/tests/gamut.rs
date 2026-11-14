//! The theme bake's gamut-mapping wire-up (architecture plan §3.2's
//! aeolus-theme row — "oklch→sRGB gamut-mapped once" — and §9's colour-
//! gamut risk row). The bake is where the ONE conversion happens: every
//! T0 literal, T1 dark pair and T2 runtime answer flows through it, so
//! the CSS Color 4 chroma reduction rides the bake, not the backends.
//!
//! The pins:
//!
//! * the REAL embedded palette carries out-of-gamut tokens (the shadcn
//!   `--destructive: oklch(0.577 0.245 27.325)` overshoots the sRGB
//!   gamut) — the bake maps them: the baked bytes keep the token's hue
//!   and lightness and stay more saturated than the clamp path's;
//! * in-gamut tokens bake byte-identically to the clamp conversion
//!   (the identity contract — every existing palette pin holds);
//! * the `/alpha` runtime answers mix FIRST, then map (the CSS reading:
//!   `color-mix` runs in Oklab, the display conversion maps) — for
//!   in-gamut tokens the mix is already in gamut and the answer is the
//!   unmixed bytes the existing pins hold.

use aeolus_core::Color;
use aeolus_theme::{ThemeSource, TokenTable};

/// OkLCh polar reading of a parsed colour.
fn polar(c: Color) -> (f32, f32, f32) {
    let chroma = (c.a * c.a + c.b * c.b).sqrt();
    let mut hue = c.b.atan2(c.a).to_degrees();
    if hue < 0.0 {
        hue += 360.0;
    }
    (c.l, chroma, hue)
}

fn hue_gap(a: f32, b: f32) -> f32 {
    (a - b).abs().min(360.0 - (a - b).abs())
}

/// The shadcn destructive token — the real embedded palette's
/// out-of-gamut row (linear-red ≈ 1.27 at the clamp conversion).
const DESTRUCTIVE: &str = "oklch(0.577 0.245 27.325)";

#[test]
fn baked_out_of_gamut_tokens_gamut_map() {
    let source = ThemeSource::shadcn().expect("embedded palette parses");
    let table = TokenTable::baked(&source).expect("bakes");
    let id = table.id("destructive").expect("destructive is a token");
    let baked = table.light_rgba(id).expect("baked bytes");

    let input = Color::parse(DESTRUCTIVE).expect("the token's own spelling");
    let (l_in, c_in, h_in) = polar(input);

    // The mapped answer, through the backend bytes.
    let [r, g, b, a] = {
        let [r, g, b, a] = baked;
        [
            f32::from(r) / 255.0,
            f32::from(g) / 255.0,
            f32::from(b) / 255.0,
            f32::from(a) / 255.0,
        ]
    };
    let received = Color::from_srgb(r, g, b, a);
    let (l_out, c_out, h_out) = polar(received);
    assert!(
        (l_out - l_in).abs() < 0.02,
        "lightness holds through the bake: {l_out} vs {l_in}"
    );
    assert!(
        hue_gap(h_out, h_in) < 2.0,
        "hue holds through the bake: {h_out} vs {h_in}"
    );
    assert!(
        c_out <= c_in,
        "chroma never grows past the input's: {c_out} vs {c_in}"
    );

    // And the clamp path genuinely differs (the mapping fired — this is
    // what changed, byte-wise, on the one out-of-gamut palette token).
    let clamped = input.to_rgba8();
    assert_ne!(
        baked, clamped,
        "the destructive bake is mapped, not clamped"
    );
}

#[test]
fn baked_in_gamut_tokens_stay_clamp_identical() {
    let source = ThemeSource::shadcn().expect("embedded palette parses");
    let table = TokenTable::baked(&source).expect("bakes");
    // The identity contract through the bake: an in-gamut token's baked
    // bytes ARE the clamp conversion's (the early-out runs the same
    // matrix math, so every existing palette pin holds).
    for name in ["background", "foreground", "primary", "muted-foreground"] {
        let id = table
            .id(name)
            .unwrap_or_else(|| panic!("{name} is a token"));
        let baked = table.light_rgba(id).expect("baked bytes");
        let parsed = source
            .light()
            .color(name)
            .unwrap_or_else(|| panic!("{name} reads back"));
        assert_eq!(
            baked,
            parsed.to_rgba8(),
            "{name} bakes to the clamp conversion's bytes"
        );
    }
}

#[test]
fn the_alpha_modifier_mixes_first_then_maps() {
    let source = ThemeSource::shadcn().expect("embedded palette parses");
    let table = TokenTable::runtime(&source).expect("runtime tier builds");
    let id = table.id("destructive").expect("destructive is a token");
    let half = table.resolve_rgba(id, 0.5, false).expect("the /50 answer");

    let input = Color::parse(DESTRUCTIVE).expect("parses");
    // Mix in Oklab (the audited premultiplied identity), THEN map —
    // the CSS reading. The mix halves the chroma; at 0.1225 the token
    // is back inside the gamut, so the answer IS the mix's own bytes.
    let expected = input.mix_with_transparent(50.0).to_rgba8();
    assert_eq!(half, expected, "the /50 answer mixes first, maps after");

    // And the unmapped mix stays inside the gamut (the mix reduced the
    // chroma below the boundary — assert via the round-trip reading).
    let [r, g, b, a] = half;
    let received = Color::from_srgb(
        f32::from(r) / 255.0,
        f32::from(g) / 255.0,
        f32::from(b) / 255.0,
        f32::from(a) / 255.0,
    );
    let (l_in, _, h_in) = polar(input.mix_with_transparent(50.0));
    let (l_out, _, h_out) = polar(received);
    assert!((l_out - l_in).abs() < 0.02, "lightness holds");
    assert!(hue_gap(h_out, h_in) < 2.0, "hue holds");
}

#[test]
fn a_host_theme_with_an_out_of_gamut_token_bakes_mapped() {
    // A user CSS the way §3.4 promises it works: the same app.css a web
    // Tailwind project uses. An out-of-gamut `--color-brand` maps at
    // the bake; a web browser shows the CSS Color 4 mapped rendering —
    // the two agree now.
    let source = ThemeSource::from_css(":root { --color-brand: oklch(0.45 0.32 145); }", None)
        .expect("the css parses");
    let table = TokenTable::baked(&source).expect("bakes");
    let id = table.id("brand").expect("brand is a token");
    let baked = table.light_rgba(id).expect("bytes");

    let input = Color::parse("oklch(0.45 0.32 145)").expect("parses");
    assert_ne!(
        baked,
        input.to_rgba8(),
        "the out-of-gamut brand token is mapped, not clamped"
    );
    let [r, g, b, a] = baked;
    let received = Color::from_srgb(
        f32::from(r) / 255.0,
        f32::from(g) / 255.0,
        f32::from(b) / 255.0,
        f32::from(a) / 255.0,
    );
    let (l_in, c_in, h_in) = polar(input);
    let (l_out, c_out, h_out) = polar(received);
    assert!((l_out - l_in).abs() < 0.02, "lightness holds");
    assert!(hue_gap(h_out, h_in) < 2.0, "hue holds");
    assert!(
        c_out > c_in - 0.25 && c_out < c_in,
        "chroma reduced onto the boundary: {c_out} vs {c_in}"
    );
}
