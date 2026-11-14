//! The named-container bounds seam's microbenchmarks (divan, stable —
//! the workspace bench idiom): the base walk over a card string that
//! carries its own named container-query layers, with and without the
//! container bounds in hand.
//!
//! The walk is one pass over the interned layers either way — the seam
//! adds one bounds lookup per named container layer, no allocation.
//! The three arms pin the cost split: the sourceless walk (named
//! layers skip), the bounds-miss walk (the unknown-name element — the
//! quiet CSS miss) and the bounds-hit walk (the `card` container
//! answers — the layer firing).

use aeolus_gpui::prelude::*;
use divan::Bencher;

fn main() {
    divan::main();
}

/// The card surface a dashboard panel would wear: the base card plus
/// two named container-query layers (`@md/card:` min form, `@max-lg/card:`
/// max form) so the container walk has work to do. (`tw!` compiles a
/// literal, so the program builds in one helper the three benches
/// share.)
fn card_program() -> StaticProgram {
    tw!("flex flex-col gap-2 rounded-md border p-4 @md/card:p-6 @max-lg/card:p-2")
}

/// The named containers' widths: the one container the card string
/// probes (the `card` container a panel layout declares, wide enough
/// for the min form to fire).
fn card_bounds() -> [ContainerBound<'static>; 1] {
    [ContainerBound {
        name: "card",
        rem: 48.0,
    }]
}

#[divan::bench]
fn tw_sourceless(bencher: Bencher) {
    let program = card_program();
    bencher.bench(|| gpui::StyleRefinement::default().tw(program));
}

#[divan::bench]
fn tw_containers_bounds_miss(bencher: Bencher) {
    let program = card_program();
    let bounds: [ContainerBound<'static>; 0] = [];
    let card = ContainerState::new(&bounds);
    bencher.bench(|| gpui::StyleRefinement::default().tw_containers(program, &card));
}

#[divan::bench]
fn tw_containers_bounds_hit(bencher: Bencher) {
    let program = card_program();
    let bounds = card_bounds();
    let card = ContainerState::new(&bounds);
    bencher.bench(|| gpui::StyleRefinement::default().tw_containers(program, &card));
}
