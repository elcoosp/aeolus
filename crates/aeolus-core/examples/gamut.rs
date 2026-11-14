//! The CSS Color 4 gamut-mapping seam, walked end to end (architecture
//! plan §9's colour-gamut risk row, §3.2's aeolus-theme row's
//! "oklch→sRGB gamut-mapped once").
//!
//! Run with `cargo run -p aeolus-core --example gamut`.
//!
//! The story: a theme token or arbitrary value that overshoots the sRGB
//! gamut used to convert by channel-clamping — the hue shifted, the
//! colour went muddy, and the web browser's own CSS Color 4 rendering
//! disagreed with the native one. `Color::gamut_map` walks the chroma
//! down the colour's own hue ray to the gamut boundary instead: hue and
//! lightness hold, the answer is in gamut by construction, and every
//! in-gamut colour (the whole default palette) converts bit-identically
//! to the clamp oracle it sits beside.

use aeolus_core::Color;

fn main() {
    // The shadcn `--destructive` token: the real default palette's
    // out-of-gamut row (the linear red channel overshoots 1.27).
    let destructive = Color::parse("oklch(0.577 0.245 27.325)").expect("the token parses");

    println!("token: {}", destructive.to_oklch_string());
    println!();
    println!("clamp oracle (to_srgba — unchanged, the v0.x differential's ground truth):");
    let clamped = destructive.to_rgba8();
    println!("  rgba8 {clamped:?}  as Oklab: {}", read_back(clamped));
    println!();
    println!("CSS Color 4 mapped (gamut_map — the bake's answer):");
    let mapped = destructive.gamut_map().to_rgba8();
    println!("  rgba8 {mapped:?}  as Oklab: {}", read_back(mapped));
    println!();

    // The identity contract: in-gamut colours convert unchanged, so
    // every existing palette pin holds.
    let in_gamut = Color::parse("oklch(0.637 0.237 25.331)").expect("red-500 parses");
    println!("in-gamut token {}", in_gamut.to_oklch_string());
    println!(
        "  clamp {:?} == mapped {:?}: {}",
        in_gamut.to_rgba8(),
        in_gamut.gamut_map().to_rgba8(),
        in_gamut.to_rgba8() == in_gamut.gamut_map().to_rgba8()
    );
}

/// Read packed sRGB bytes back into Oklab and render them OkLCh-style —
/// the "what a backend actually receives" reading.
fn read_back(bytes: [u8; 4]) -> String {
    let [r, g, b, a] = bytes;
    let color = Color::from_srgb(
        f32::from(r) / 255.0,
        f32::from(g) / 255.0,
        f32::from(b) / 255.0,
        f32::from(a) / 255.0,
    );
    color.to_oklch_string()
}
