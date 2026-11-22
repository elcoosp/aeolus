//! The named-group dispatch's microbenchmark (the wave's M1b half):
//! the label extraction the state walk performs (the native
//! `.group_hover(label, …)` wiring's own work) beside the plain
//! unnamed-fold walk — the label dispatch's marginal cost over the
//! boolean fold.
use divan::Bencher;

use aeolus_gpui::prelude::*;

#[divan::bench]
fn unnamed_fold_walk(bencher: Bencher) {
    // The wave-1 baseline: the flat boolean fold, the env's unnamed
    // fact answering.
    let program = tw!("group-hover:p-4 group-active:bg-red-500 p-8");
    bencher.bench(|| gpui::StyleRefinement::default().tw(program));
}

#[divan::bench]
fn named_label_walk(bencher: Bencher) {
    // The named form: the env walk stays quiet (no group registry),
    // the label extraction (the state walk's own dispatch work) rides
    // `tw_state`'s native closure wiring.
    let program = tw!("group-hover/menu:p-4 group-active/nav:bg-red-500 p-8");
    bencher.bench(|| gpui::StyleRefinement::default().tw(program));
}

fn main() {
    divan::main();
}
