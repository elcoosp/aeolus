//! The parity wave 1's microbenchmarks (divan, stable — the workspace
//! bench idiom): the GPUI walk's new arms — the native clamp/variant
//! longhands, the between-child routing (the state the wrapper lane
//! consumes) and the wrapper's own composition+patch pass — beside the
//! plain-walk floors they appended to.

use divan::Bencher;

use aeolus_gpui::prelude::*;

fn main() {
    divan::main();
}

/// The native clamp longhand's walk arm (the text-style slot + the
/// overflow-hidden compose).
#[divan::bench]
fn line_clamp_walk(bencher: Bencher) {
    let program = tw!("line-clamp-2 bg-primary");
    bencher.bench(|| gpui::StyleRefinement::default().tw(program));
}

/// The font-variant-numeric tags' walk arm (the additive
/// `FontFeatures` composition — three families stacking).
#[divan::bench]
fn variant_numeric_walk(bencher: Bencher) {
    let program = tw!("tabular-nums ordinal slashed-zero bg-primary");
    bencher.bench(|| gpui::StyleRefinement::default().tw(program));
}

/// The between-child families' walk route (the ten longhands into the
/// state accumulator the wrapper lane consumes — the plain walk's own
/// routing cost).
#[divan::bench]
fn space_divide_routing_walk(bencher: Bencher) {
    let program = tw!("space-x-4 divide-y-2 divide-red-500 divide-dashed bg-primary");
    bencher.bench(|| gpui::StyleRefinement::default().tw(program));
}

/// The wrapper's full pass: the ONE walk taking both outputs (base
/// styles onto the element, between-child parts into the state) — the
/// `tw_space` lane's own cost.
#[divan::bench]
fn wrapper_state_pass(bencher: Bencher) {
    let program = tw!("flex space-x-4 divide-y divide-red-500 divide-dashed");
    bencher.bench(|| {
        let wrapper = gpui::div().tw_space_env(program, Env::default());
        let state = wrapper.state();
        let _ = (
            state.margin(aeolus_gpui::Axis::X),
            state.border(aeolus_gpui::Axis::Y),
        );
    });
}

/// The plain-walk floor the wave-1 arms appended beside (the same
/// string minus the wave-1 classes — the delta the three arms above
/// explain).
#[divan::bench]
fn plain_walk_floor(bencher: Bencher) {
    let program = tw!("bg-primary flex");
    bencher.bench(|| gpui::StyleRefinement::default().tw(program));
}
