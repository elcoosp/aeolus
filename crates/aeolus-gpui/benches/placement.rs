//! The grid-placement + flex-basis tier's microbenchmarks (divan,
//! stable — the workspace bench idiom): the walk applying a placement
//! program beside the span-pair compose and the basis-family value
//! shapes.
//!
//! The placement longhands ride the walk's grid accumulator (the
//! `RingState` precedent): the per-declaration cost is one `Option`
//! store, the compose is two range writes at the walk's end — the
//! arms pin the split between the line placements (one compose each)
//! and the span fingerprint (the equal-pair reading).
//!
//! Run with `cargo bench -p aeolus-gpui --bench placement`.

use aeolus_gpui::prelude::*;
use divan::Bencher;

fn main() {
    divan::main();
}

/// The dashboard-shell surface a placement string wears: the grid
/// template beside placement lines and a span, plus the basis family.
fn shell_program() -> StaticProgram {
    tw!("grid grid-cols-12 col-start-2 col-end-9 row-span-2 basis-16")
}

#[divan::bench]
fn tw_line_placements(bencher: Bencher) {
    let program = tw!("col-start-2 col-end-9 row-start-3 row-end-6");
    bencher.bench(|| gpui::StyleRefinement::default().tw(program));
}

#[divan::bench]
fn tw_span_compose(bencher: Bencher) {
    let program = shell_program();
    bencher.bench(|| gpui::StyleRefinement::default().tw(program));
}

#[divan::bench]
fn tw_basis_family(bencher: Bencher) {
    let program = tw!("basis-16 basis-1/2 basis-[30%]");
    bencher.bench(|| gpui::StyleRefinement::default().tw(program));
}
