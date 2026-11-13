//! The ring polyfill's microbenchmark (divan, stable — the workspace
//! bench idiom): the per-frame cost of the §3.7 polyfill — the walk
//! applying a `tw!`-compiled ring program (the `--tw-ring-*`
//! accumulator composed into GPUI's `Vec<BoxShadow>`), and the
//! accumulator's own composition as the unit surface.

use divan::Bencher;

use aeolus_gpui::prelude::*;

#[divan::bench]
fn ring_walk(bencher: Bencher) {
    // The shadcn focus-ring corpus: width, colour, offset width and
    // offset colour — all four `--tw-ring-*` longhands in one walk.
    let program = tw!("ring-2 ring-red-500 ring-offset-2 ring-offset-white bg-primary");
    bencher.bench(|| gpui::StyleRefinement::default().tw(program));
}

#[divan::bench]
fn ring_composition(bencher: Bencher) {
    // The unit surface: the accumulator the walk feeds, composed into
    // the v4 box-shadow list.
    use aeolus_ir::{Prop, Val};
    bencher.bench(|| {
        let mut state = aeolus_gpui::RingState::default();
        state.apply(Prop::RingOffsetWidth, Val::Num(2.0), "");
        state.apply(Prop::RingOffsetColor, Val::Rgba([255, 255, 255, 255]), "");
        state.apply(Prop::RingWidth, Val::Num(2.0), "");
        state.apply(Prop::RingColor, Val::Rgba([251, 44, 54, 255]), "");
        state.shadows()
    });
}

fn main() {
    divan::main();
}
