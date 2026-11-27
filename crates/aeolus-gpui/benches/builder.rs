//! The emit (a) END FORM's runtime microbenchmark (divan, stable — the
//! workspace bench idiom): the §4 performance contract's target face —
//! the emitted builder-chain application against the walk that the
//! frozen program rides, both beside the hand-written builder calls
//! the plan's parity harness pins. The chain IS the hand-written
//! chain's shape (the compile-time method calls), so its per-frame
//! cost is the §4 target's "hand-written builder chain" cell, measured
//! over the same representative strings.
//!
//! The parity (chain == walk, field by field) rides the setup pins in
//! `tests/builder_parity.rs` — the measurement below is only fair
//! because the two lanes land the same refinement.

use divan::Bencher;

use aeolus_gpui::prelude::*;

fn main() {
    divan::main();
}

/// The frozen program the walk lane rides.
#[divan::bench]
fn walk_button(bencher: Bencher) {
    let program = tw!(
        "inline-flex items-center justify-center gap-2 p-4 rounded-md bg-primary text-primary-foreground"
    );
    bencher.bench(|| {
        set_env(Default::default()); // the frame-Env escape hatch (plan §3.5's debug-assert keys on it)
        gpui::StyleRefinement::default().tw(program)
    });
}

/// The emitted chain: `tw_builder!`'s compile-time method calls
/// applied through [`TwExt::tw_chain`] — no walk, no dispatch, the
/// §4 target's own shape.
#[divan::bench]
fn chain_button(bencher: Bencher) {
    bencher.bench(|| {
        gpui::StyleRefinement::default().tw_chain(tw_builder!("inline-flex items-center justify-center gap-2 p-4 rounded-md bg-primary text-primary-foreground"))
    });
}

/// The hand-written builder calls (the §4 parity harness's third lane)
/// — the chain's cost ceiling and the walk's target, on one harness.
/// The colours are the button bake's own bytes (the parity pins'
/// rendered-bytes comparison).
#[divan::bench]
fn hand_button(bencher: Bencher) {
    bencher.bench(|| {
        gpui::StyleRefinement::default()
            .flex()
            .items_center()
            .justify_center()
            .gap_x(gpui::rems(0.5))
            .gap_y(gpui::rems(0.5))
            .pt(gpui::rems(1.0))
            .pr(gpui::rems(1.0))
            .pb(gpui::rems(1.0))
            .pl(gpui::rems(1.0))
            .rounded_tl(gpui::rems(0.375))
            .rounded_tr(gpui::rems(0.375))
            .rounded_br(gpui::rems(0.375))
            .rounded_bl(gpui::rems(0.375))
            .bg(gpui::Hsla::from(gpui::rgba(0x1717_17FF)))
            .text_color(gpui::Hsla::from(gpui::rgba(0xFAFA_FAFF)))
    });
}
