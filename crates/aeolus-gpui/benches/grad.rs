//! The gradient lane's microbenchmark (divan, stable — the workspace
//! bench idiom): the per-frame cost of the §3.7 native tier — the walk
//! applying a `tw!`-compiled two-stop linear program (the
//! `--tw-gradient-*` accumulator composed into GPUI's `Fill`), and the
//! accumulator's own composition as the unit surface.

use divan::Bencher;

use aeolus_gpui::prelude::*;

#[divan::bench]
fn grad_walk(bencher: Bencher) {
    // The banner corpus: direction, from stop, position patch, to stop
    // — the whole gradient family in one walk.
    let program = tw!("bg-linear-to-r from-red-500 from-20% to-blue-500 bg-primary");
    bencher.bench(|| gpui::StyleRefinement::default().tw(program));
}

#[divan::bench]
fn grad_composition(bencher: Bencher) {
    // The unit surface: the accumulator the walk feeds, composed into
    // the linear gradient.
    use aeolus_ir::{Prop, Val};
    bencher.bench(|| {
        let mut state = aeolus_gpui::GradState::default();
        state.apply(
            Prop::BackgroundImage,
            Val::Kw(0),
            "linear-gradient(to right, var(--tw-gradient-stops))",
        );
        state.apply(Prop::GradientFrom, Val::Rgba([251, 44, 54, 255]), "");
        state.apply(Prop::GradientFromPosition, Val::Pct(20.0), "");
        state.apply(Prop::GradientTo, Val::Rgba([59, 130, 246, 255]), "");
        state.background()
    });
}

fn main() {
    divan::main();
}
