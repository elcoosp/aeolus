//! The shadow tier's microbenchmark (divan, stable — the workspace bench
//! idiom): the per-frame cost of the §3.7 shadow row's native tier — the
//! walk applying a `tw!`-compiled shadow program (the preset bands
//! composed into GPUI's `Vec<BoxShadow>` through the `ShadowState`
//! accumulator, beside the ring's bands), and the accumulator's own
//! composition as the unit surface.

use divan::Bencher;

use aeolus_gpui::prelude::*;

#[divan::bench]
fn shadow_walk(bencher: Bencher) {
    // The shadcn card corpus: the two-band lg preset, the recolour and
    // the text colour the `currentColor` band would read — the whole
    // `shadow-*` machinery in one walk.
    let program = tw!("shadow-lg shadow-red-500 text-primary-foreground bg-primary");
    bencher.bench(|| gpui::StyleRefinement::default().tw(program));
}

#[divan::bench]
fn shadow_composition(bencher: Bencher) {
    // The unit surface: the accumulator the walk feeds, composed into
    // the v4 band list (the recolour over the preset's geometry).
    use aeolus_ir::{Prop, Val};
    bencher.bench(|| {
        let mut state = aeolus_gpui::ShadowState::default();
        state.apply(Prop::BoxShadow, Val::Kw(4), "var(--tw-shadow-lg)");
        state.apply(Prop::ShadowColor, Val::Rgba([251, 44, 54, 255]), "");
        state.shadows()
    });
}

#[divan::bench]
fn shadow_parse(bencher: Bencher) {
    // The lane's own parse: the closed-world preset reference resolved
    // through the audited band texts into GPUI's box-shadow list.
    bencher.bench(|| aeolus_gpui::parse_shadow_bands("var(--tw-shadow-lg)"));
}

fn main() {
    divan::main();
}
