//! Gamut-conversion benchmarks: the clamp oracle beside the CSS Color 4
//! chroma reduction (plan §9's mitigation), the in-gamut early-out beside
//! the out-of-gamut binary search. The bake runs the mapped conversion
//! ONCE per token, so the absolute cost is amortised — these numbers are
//! for the relative story (the early-out's identity path is the plain
//! conversion's cost; only out-of-gamut tokens pay the search).
//!
//! Run with `cargo bench -p aeolus-core --bench gamut` (divan).

use aeolus_core::Color;
use divan::Bencher;

fn main() {
    divan::main();
}

/// The in-gamut path: the shadcn palette's own tokens (the identity
/// contract's early-out).
const IN_GAMUT: &str = "oklch(0.637 0.237 25.331)";

/// The out-of-gamut path: a clearly past-the-ceiling green.
const OUT_OF_GAMUT: &str = "oklch(0.5 0.35 145)";

#[divan::bench]
fn clamp_conversion_in_gamut(b: Bencher) {
    let color = Color::parse(IN_GAMUT).expect("parses");
    b.bench(|| color.to_srgba());
}

#[divan::bench]
fn gamut_map_in_gamut(b: Bencher) {
    let color = Color::parse(IN_GAMUT).expect("parses");
    b.bench(|| color.gamut_map().to_srgba());
}

#[divan::bench]
fn clamp_conversion_out_of_gamut(b: Bencher) {
    let color = Color::parse(OUT_OF_GAMUT).expect("parses");
    b.bench(|| color.to_srgba());
}

#[divan::bench]
fn gamut_map_out_of_gamut(b: Bencher) {
    let color = Color::parse(OUT_OF_GAMUT).expect("parses");
    b.bench(|| color.gamut_map().to_srgba());
}

#[divan::bench]
fn bake_byte_packed_out_of_gamut(b: Bencher) {
    // The theme bake's own shape: the packed rgba8 the token tables
    // freeze.
    let color = Color::parse(OUT_OF_GAMUT).expect("parses");
    b.bench(|| color.gamut_map().to_rgba8());
}
