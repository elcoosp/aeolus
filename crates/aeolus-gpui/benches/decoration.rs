//! The decoration tier's microbenchmark (divan, stable — the workspace
//! bench idiom): the walk applying a `tw!`-compiled decoration program
//! (the colour/thickness/style longhands through the pinned
//! `text_decoration_*` methods and the walk's per-leg patches), and
//! the full family composed in one walk beside the line-only floor.

use divan::Bencher;

use aeolus_gpui::prelude::*;

#[divan::bench]
fn decoration_walk(bencher: Bencher) {
    // The §3.7 decoration tier's own corpus: line, style, thickness and
    // colour — all four longhands in one walk.
    let program = tw!("underline decoration-wavy decoration-4 decoration-red-500 bg-primary");
    bencher.bench(|| gpui::StyleRefinement::default().tw(program));
}

#[divan::bench]
fn line_only_floor(bencher: Bencher) {
    // The pre-decoration floor: the line longhand alone (the walk arm
    // the family appended its three longhands beside).
    let program = tw!("underline bg-primary");
    bencher.bench(|| gpui::StyleRefinement::default().tw(program));
}

#[divan::bench]
fn decoration_integer_thickness_walk(bencher: Bencher) {
    // The bare-integer thickness form (the v4 isPositiveInteger
    // grammar's own arm) with the line-through leg beside — the two-leg
    // patch the colour/thickness arms compose.
    let program = tw!("line-through decoration-13 decoration-red-500");
    bencher.bench(|| gpui::StyleRefinement::default().tw(program));
}

fn main() {
    divan::main();
}
