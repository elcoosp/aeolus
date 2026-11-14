//! The CSS Color 4 gamut-mapping seam (architecture plan §9's colour-gamut
//! risk row — "Gamut-map once (CSS Color 4 chroma reduction); document" —
//! and §3.2's aeolus-theme row's own "oklch→sRGB gamut-mapped once"
//! contract). [`Color::to_srgba`] stays the v0.x oracle's clamp conversion
//! (the golden corpus's ground truth); [`Color::gamut_map`] is the NEW
//! out-of-gamut answer: reduce the OkLCh chroma to the largest in-gamut
//! value, hue and lightness untouched — the CSS Color 4 §13 algorithm.
//!
//! The pins:
//!
//! * **The identity contract** — every in-gamut colour maps to ITSELF:
//!   the early-out path runs the same matrix math as the clamp
//!   conversion, so all existing palette pins hold byte for byte;
//! * **Hue and lightness hold** — an out-of-gamut colour's mapped result
//!   keeps the input's OkLCh hue and L (the clamp path shifts hue
//!   badly); only the chroma comes down;
//! * **The mapped result is in gamut** — the sRGBA a backend receives
//!   round-trips back through `from_srgb` to (nearly) the mapped colour,
//!   never NaN, for the whole (L, C, H) input space including the fuzz
//!   extremes;
//! * **Mapping beats clamping** — the mapped result keeps strictly more
//!   chroma than the clamp path's (the reason the algorithm exists);
//! * **Determinism** — the same input maps to the same bytes every time
//!   (the bake runs once per token; the static form freezes the result).

use aeolus_core::Color;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// OkLCh polar reading of a parsed colour (the fields are public).
fn polar(c: Color) -> (f32, f32, f32) {
    let chroma = (c.a * c.a + c.b * c.b).sqrt();
    let mut hue = c.b.atan2(c.a).to_degrees();
    if hue < 0.0 {
        hue += 360.0;
    }
    (c.l, chroma, hue)
}

/// The shorter arc between two hue readings.
fn hue_gap(a: f32, b: f32) -> f32 {
    (a - b).abs().min(360.0 - (a - b).abs())
}

/// The clamp path's effective sRGB result, read back into Oklab — the
/// "what the old code produced" baseline.
fn clamp_roundtrip(c: Color) -> Color {
    let [r, g, b, a] = c.to_srgba();
    Color::from_srgb(r, g, b, a)
}

/// The mapped path's effective sRGB result, read back into Oklab — the
/// "what a backend actually receives" reading.
fn mapped_roundtrip(c: Color) -> Color {
    let [r, g, b, a] = c.gamut_map().to_srgba();
    Color::from_srgb(r, g, b, a)
}

/// A clearly out-of-gamut green: at L 0.5 the sRGB gamut's chroma ceiling
/// sits near 0.15; 0.35 overshoots it hard.
fn out_of_gamut_green() -> Color {
    Color::oklch(0.5, 0.35, 145.0, 1.0)
}

// ---------------------------------------------------------------------------
// 1. The identity contract
// ---------------------------------------------------------------------------

#[test]
fn in_gamut_colors_map_to_themselves() {
    // The audited palettes (shadcn's oklch source, the Tailwind hexes)
    // are gamut-safe — every one must ride the early-out path and land
    // on the clamp conversion's EXACT bytes.
    let in_gamut = [
        "#336699",
        "#0ea5e9",
        "rgb(251 44 54)",
        "rgba(0, 0, 0, 0.5)",
        "oklch(0.6370 0.2370 25.331)", // the v4 default theme's red-500
        "oklch(0.5482 0.2349 317.2986)", // a mid-L fuchsia-class oklch
        "oklch(0.9702 0.0000 0.0000)", // near-white gray
        "oklch(0.5 0.05 200 / 0.7)",   // in-gamut with alpha
    ];
    for text in in_gamut {
        let color = Color::parse(text).unwrap_or_else(|e| panic!("{text}: {e}"));
        let mapped = color.gamut_map();
        assert_eq!(
            mapped.to_srgba(),
            color.to_srgba(),
            "in-gamut {text} must convert unchanged"
        );
    }
}

#[test]
fn the_shadcn_palette_maps_to_itself_token_for_token() {
    // The theme bake's own inputs: a scan of the embedded light palette's
    // oklch tokens stays byte-identical (the bake's output pins hold).
    let palette = [
        "oklch(0.985 0 0)",            // background
        "oklch(0.1411 0.005 285.823)", // foreground (dark-ish)
        "oklch(0.7040 0.04 256.788)",  // muted-foreground-class
        "oklch(0.9702 0 0)",           // a gray scale step
    ];
    for text in palette {
        let color = Color::parse(text).unwrap_or_else(|e| panic!("{text}: {e}"));
        assert_eq!(
            color.gamut_map().to_rgba8(),
            color.to_rgba8(),
            "palette token {text} must bake to the same bytes"
        );
    }
}

// ---------------------------------------------------------------------------
// 2. Hue and lightness hold
// ---------------------------------------------------------------------------

#[test]
fn out_of_gamut_chroma_reduces_hue_and_lightness_hold() {
    let input = out_of_gamut_green();
    let mapped = input.gamut_map();
    let (l_in, c_in, h_in) = polar(input);
    let (l_out, c_out, h_out) = polar(mapped);
    assert_eq!(l_out, l_in, "lightness holds");
    let hue_gap = (h_out - h_in).abs().min(360.0 - (h_out - h_in).abs());
    assert!(hue_gap < 0.1, "hue holds: in {h_in} out {h_out}");
    assert!(
        c_out < c_in - 0.05,
        "chroma came down: in {c_in} out {c_out}"
    );
    assert_eq!(mapped.alpha, 1.0, "alpha passes through");
}

#[test]
fn the_clamp_path_shifts_hue_the_mapping_does_not() {
    // The documented v1 limitation, quantified: the clamp path's hue
    // drifts far from the input's; the mapping's does not.
    let input = out_of_gamut_green();
    let (_, _, h_in) = polar(input);
    let (_, _, h_clamped) = polar(clamp_roundtrip(input));
    let (_, _, h_mapped) = polar(mapped_roundtrip(input));
    let drift = |h: f32| (h - h_in).abs().min(360.0 - (h - h_in).abs());
    assert!(
        drift(h_mapped) < 0.1,
        "mapped hue holds (drift {})",
        drift(h_mapped)
    );
    assert!(
        drift(h_clamped) > 1.0,
        "the clamp path's hue drift is real (drift {})",
        drift(h_clamped)
    );
}

// ---------------------------------------------------------------------------
// 3. The mapped result is in gamut
// ---------------------------------------------------------------------------

#[test]
fn mapped_results_roundtrip_inside_the_gamut() {
    // A backend receives sRGBA; reading it back into Oklab must land
    // (nearly) on the mapped colour — the in-gamut guarantee, public
    // surface only. The clamp path CANNOT satisfy this for out-of-gamut
    // inputs (its hue moves).
    let inputs = [
        Color::oklch(0.5, 0.35, 145.0, 1.0),
        Color::oklch(0.2, 0.3, 300.0, 1.0),
        Color::oklch(0.9, 0.2, 90.0, 1.0),
        Color::oklch(0.5, 0.5, 30.0, 1.0),
        Color::oklch(0.75, 0.28, 25.0, 0.5),
    ];
    for input in inputs {
        let mapped = mapped_roundtrip(input);
        let target = input.gamut_map();
        let (l_t, c_t, h_t) = polar(target);
        let (l_m, c_m, h_m) = polar(mapped);
        assert!(
            (l_m - l_t).abs() < 0.01,
            "L holds through the backend bytes: {l_m} vs {l_t}"
        );
        assert!(
            (c_m - c_t).abs() < 0.02,
            "chroma holds through the backend bytes: {c_m} vs {c_t}"
        );
        let gap = (h_m - h_t).abs().min(360.0 - (h_m - h_t).abs());
        assert!(
            gap < 2.0,
            "hue holds through the backend bytes: {h_m} vs {h_t}"
        );
    }
}

#[test]
fn extremes_stay_finite_and_in_gamut() {
    // The finiteness contract to_srgba carries (the fuzz findings) must
    // hold on the mapped path too: any parsed value maps to a finite,
    // channel-legal sRGBA.
    let extremes = [
        Color::oklch(3.0, 0.1, 30.0, 1.0),   // L beyond the range
        Color::oklch(-1.0, 0.5, 200.0, 1.0), // negative L
        Color::oklch(0.5, 3.3e30, 1.0, 1.0), // the chroma flood (f32-huge)
        Color::oklch(0.5, 0.9, 999.0, 1.0),  // hue beyond a turn
        Color::oklch(1.0, 0.9, 145.0, 1.0),  // white + chroma
        Color::oklch(0.0, 0.9, 145.0, 1.0),  // black + chroma
        Color::oklch(0.5, 0.0, 145.0, 1.0),  // gray: no chroma to reduce
    ];
    for input in extremes {
        let [r, g, b, a] = input.gamut_map().to_srgba();
        for (name, channel) in [("r", r), ("g", g), ("b", b), ("a", a)] {
            assert!(
                channel.is_finite() && (0.0..=1.0).contains(&channel),
                "{input:?} maps to legal {name}={channel}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 4. Hue holds across the wheel
// ---------------------------------------------------------------------------

#[test]
fn mapped_hue_holds_across_the_wheel() {
    // Six out-of-gamut probes around the hue wheel (each clearly past
    // its ray's gamut ceiling): the mapped answer's hue and lightness
    // hold through the backend bytes, chroma strictly down.
    let probes = [
        (0.5, 0.35, 25.0),
        (0.5, 0.35, 80.0),
        (0.5, 0.35, 145.0),
        (0.5, 0.35, 210.0),
        (0.5, 0.35, 300.0),
        (0.75, 0.28, 145.0),
    ];
    for (l, c, h) in probes {
        let input = Color::oklch(l, c, h, 1.0);
        let mapped = mapped_roundtrip(input);
        let (l_out, c_out, h_out) = polar(mapped);
        let (_, c_in, h_in) = polar(input);
        assert!(
            (l_out - l).abs() < 0.02,
            "oklch({l} {c} {h}): lightness holds ({l_out})"
        );
        assert!(
            hue_gap(h_out, h_in) < 1.0,
            "oklch({l} {c} {h}): hue holds ({h_out} vs {h_in})"
        );
        assert!(
            c_out < c_in,
            "oklch({l} {c} {h}): chroma came down ({c_out} vs {c_in})"
        );
    }
}

// ---------------------------------------------------------------------------
// 5. Determinism
// ---------------------------------------------------------------------------

#[test]
fn the_mapping_is_deterministic() {
    let input = out_of_gamut_green();
    let first = input.gamut_map().to_rgba8();
    for _ in 0..8 {
        assert_eq!(
            input.gamut_map().to_rgba8(),
            first,
            "same input, same bytes"
        );
    }
}

// ---------------------------------------------------------------------------
// 6. The clamp oracle stays the clamp oracle
// ---------------------------------------------------------------------------

#[test]
fn to_srgba_keeps_its_clamp_contract() {
    // The v0.x oracle's conversion is UNCHANGED (the differential
    // triangle's ground truth): out-of-gamut inputs still clamp —
    // the negative linear channel clips to exactly 0 (the documented
    // v1 limitation), while the mapped path lands on the boundary.
    let input = out_of_gamut_green();
    let [r, g, _, _] = input.to_srgba();
    assert_eq!(r, 0.0, "the clamp conversion clips the negative channel");
    let [r_mapped, g_mapped, _, _] = input.gamut_map().to_srgba();
    assert_eq!(
        r_mapped, 0.0,
        "the mapped point sits ON the r=0 gamut face for this hue"
    );
    assert!(
        g_mapped < g,
        "the chroma reduction shows in the surviving channel ({g_mapped} vs {g})"
    );
    assert_ne!(
        input.to_srgba(),
        input.gamut_map().to_srgba(),
        "out-of-gamut inputs take different paths by design"
    );
}
