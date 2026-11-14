//! The pipeline's gamut-mapping wire-up (architecture plan §9's colour-
//! gamut risk row): every `Val::Rgba` the pipeline freezes — the token
//! literal bake AND the arbitrary-value colour bake — runs the CSS
//! Color 4 chroma reduction, so an out-of-gamut `oklch()` token lands
//! ON the gamut boundary, hue-true, and a literal agrees with its token
//! spelling.
//!
//! The pins:
//!
//! * the TOKEN path through the pipeline (`compile_theme` over the
//!   shadcn source): the out-of-gamut `--destructive` token lowers to
//!   mapped bytes — hue and lightness hold through the backend bytes,
//!   and the clamp conversion's bytes differ;
//! * the ARBITRARY path: an in-gamut literal (hex, `rgb()`) lowers to
//!   the audited conversion's exact bytes — the identity contract (the
//!   multi-token `oklch(...)` spelling with underscore joins stays the
//!   documented composite boundary, so out-of-gamut reachability lives
//!   on the token path);
//! * the `/alpha` modifier on an in-gamut arbitrary keeps the audited
//!   premultiplied bake (mix of an in-gamut colour is in gamut — the
//!   mix-then-map order is invisible there, byte-pinned).

use aeolus_compile::{Config, compile, compile_theme, interp};
use aeolus_core::Color;
use aeolus_ir::{Prop, Val};
use aeolus_theme::{ThemeSource, TokenTable};

/// The base layer's declaration for `prop` (the single-class battery's
/// reading — the same helper shape the arbitrary battery pins with).
fn base_decl(compiled: &aeolus_compile::Compiled, prop: Prop) -> Val {
    let layer = compiled
        .program
        .layers
        .iter()
        .find(|layer| layer.cond == aeolus_ir::Cond::Base)
        .expect("the base layer exists");
    layer
        .decls
        .iter()
        .find(|decl| decl.prop == prop)
        .expect("the longhand is set")
        .val
}

/// The effective sRGBA under one probe env — the backend's reading.
fn effective(compiled: &aeolus_compile::Compiled, prop: Prop) -> [u8; 4] {
    let resolved = interp(&compiled.program, &aeolus_compile::Env::default())
        .expect("resolves under the probe env");
    for decl in &resolved.decls {
        if decl.prop == prop {
            if let Val::Rgba(bytes) = decl.val {
                return bytes;
            }
        }
    }
    panic!("no effective {prop:?} Rgba declaration: {resolved:?}");
}

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

fn baked_table() -> TokenTable {
    let source = ThemeSource::shadcn().expect("embedded palettes parse");
    TokenTable::baked(&source).expect("default-seeded source builds")
}

/// The shadcn destructive token — the real palette's out-of-gamut row
/// (linear-red ≈ 1.27 at the clamp conversion).
const DESTRUCTIVE: &str = "oklch(0.577 0.245 27.325)";

#[test]
fn a_token_literal_lowering_is_mapped() {
    let compiled =
        compile_theme("bg-destructive", &baked_table(), &Config::default()).expect("compiles");
    assert!(
        compiled.diagnostics.is_empty(),
        "{:?}",
        compiled.diagnostics
    );
    let Val::Rgba(rgba) = base_decl(&compiled, Prop::Background) else {
        panic!("an Rgba expected")
    };

    let input = Color::parse(DESTRUCTIVE).expect("the token's own spelling");
    assert_ne!(
        rgba,
        input.to_rgba8(),
        "the out-of-gamut token bakes mapped, not clamped"
    );
    let (l_in, c_in, h_in) = polar(input);
    let [r, g, b, a] = rgba;
    let received = Color::from_srgb(
        f32::from(r) / 255.0,
        f32::from(g) / 255.0,
        f32::from(b) / 255.0,
        f32::from(a) / 255.0,
    );
    let (l_out, c_out, h_out) = polar(received);
    assert!(
        (l_out - l_in).abs() < 0.02,
        "lightness holds: {l_out} vs {l_in}"
    );
    assert!(hue_gap(h_out, h_in) < 2.0, "hue holds: {h_out} vs {h_in}");
    assert!(c_out <= c_in, "chroma never grows: {c_out} vs {c_in}");

    // And the effective declaration under the probe env is the SAME
    // answer (the interp reads the frozen bytes, no second conversion).
    assert_eq!(effective(&compiled, Prop::Background), rgba);
}

#[test]
fn an_in_gamut_arbitrary_lowers_clamp_identical() {
    let compiled = compile("text-[#336699]", &Config::default()).expect("compiles");
    assert!(
        compiled.diagnostics.is_empty(),
        "{:?}",
        compiled.diagnostics
    );
    let Val::Rgba(rgba) = base_decl(&compiled, Prop::Color) else {
        panic!("an Rgba expected")
    };
    assert_eq!(
        rgba,
        Color::parse("#336699").expect("parses").to_rgba8(),
        "in-gamut literals keep the audited conversion's bytes"
    );
    assert_eq!(rgba, [51, 102, 153, 255]);

    // The comma legacy `rgb()` spelling rides the same audited parser.
    let compiled = compile("bg-[rgba(251,44,54)]", &Config::default()).expect("compiles");
    let Val::Rgba(rgba) = base_decl(&compiled, Prop::Background) else {
        panic!("an Rgba expected")
    };
    assert_eq!(
        rgba,
        Color::parse("rgb(251 44 54)").expect("parses").to_rgba8(),
        "the rgb() literal keeps the audited conversion's bytes"
    );
}

#[test]
fn the_alpha_form_keeps_the_audited_bake_for_in_gamut_values() {
    // The themed form lowers (the modifier arithmetic is the theme
    // pass's — the plain compile refuses the modifier, the arbitrary
    // battery's own typed boundary).
    let source = ThemeSource::shadcn().expect("embedded palettes parse");
    let table = TokenTable::runtime(&source).expect("runtime tier builds");
    let compiled =
        compile_theme("text-[#336699]/50", &table, &Config::default()).expect("compiles");
    assert!(
        compiled.diagnostics.is_empty(),
        "{:?}",
        compiled.diagnostics
    );
    let Val::Rgba(rgba) = base_decl(&compiled, Prop::Color) else {
        panic!("an Rgba expected")
    };
    assert_eq!(rgba[3], 128, "the /50 modifier bakes the alpha byte");
    let red = Color::parse("#336699").expect("parses");
    assert_eq!(
        rgba,
        red.mix_with_transparent(50.0).to_rgba8(),
        "in-gamut alpha keeps the audited premultiplied bake (the mix is \
         already in gamut, so mix-then-map is byte-invisible)"
    );
}
