//! The element-bounds wrapper's microbenchmarks (divan, stable — the
//! workspace bench idiom): the walk consuming the measured hand-off
//! beside the built-in static table, and the publish/refresh
//! handshake's own per-frame cost (the converged prepaint the wrapper
//! runs every frame).
//!
//! The measured hand-off adds one hash lookup per named container
//! layer over the static table's linear scan — and the wrapper's
//! converged publish is a lookup-and-compare (no allocation, no
//! re-render). The arms pin the cost split: the static-table walk, the
//! measured-handle walk (the same answers, the handle the wrapper
//! publishes into) and the stable publish (the every-frame case where
//! the width did not change). (`bench_local` for the measured arms —
//! the handle is `!Sync` by construction, the ClassCell precedent.)

use aeolus_gpui::prelude::*;
use divan::Bencher;

fn main() {
    divan::main();
}

/// The card surface the wrapper benchmarks wear: the base card plus
/// two named container-query layers (`@md/card:` min form,
/// `@max-lg/card:` max form) so the walk has named work to do through
/// the hand-off.
fn card_program() -> StaticProgram {
    tw!("flex flex-col gap-2 rounded-md border p-4 @md/card:p-6 @max-lg/card:p-2")
}

/// The measured hand-off with the card container already published —
/// the steady state of a wrapper that has measured once and converges.
fn measured_card() -> MeasuredContainers {
    let measured = MeasuredContainers::new();
    measured.publish("card", 48.0);
    measured
}

#[divan::bench]
fn tw_containers_static_table(bencher: Bencher) {
    let program = card_program();
    let bounds = [ContainerBound {
        name: "card",
        rem: 48.0,
    }];
    let card = ContainerState::new(&bounds);
    bencher.bench(|| gpui::StyleRefinement::default().tw_containers(program, &card));
}

#[divan::bench]
fn tw_containers_measured_handle(bencher: Bencher) {
    let program = card_program();
    let measured = measured_card();
    bencher.bench_local(|| gpui::StyleRefinement::default().tw_containers(program, &measured));
}

#[divan::bench]
fn publish_stable_width(bencher: Bencher) {
    // The wrapper's every-frame prepaint in the converged case: the
    // same width published again is not a change — a lookup and a
    // compare, no re-render asked for.
    let measured = measured_card();
    bencher.bench_local(|| measured.publish("card", 48.0));
}
