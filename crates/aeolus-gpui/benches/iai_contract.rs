//! The §4 instruction-count golden (the iai-callgrind gate): the SAME
//! five representative strings the stable `perf_contract` wall-clock
//! harness pins, measured as instruction counts under callgrind — the
//! ±3% baseline the nightly CI job compares against. The stable
//! benches stay the wall-clock half; this target runs ONLY on nightly
//! with valgrind (the iai-callgrind contract), never locally on stable.
//!
//! ```sh
//! cargo +nightly bench -p aeolus-gpui --bench iai_contract
//! ```

use aeolus_gpui::prelude::*;
use iai_callgrind::{black_box, library_benchmark, library_benchmark_group, main};

#[library_benchmark]
fn the_walk_button() -> (bool, bool) {
    aeolus_gpui::set_env(Default::default()); // the frame-Env escape hatch (plan §3.5's debug-assert keys on it)
    let program = aeolus_gpui::tw!(
        "inline-flex items-center justify-center gap-2 rounded-md px-4 py-2 font-medium border border-red-500 bg-red-500 text-red-500"
    );
    let style = gpui::StyleRefinement::default().tw(program);
    (
        black_box(style.padding.top.is_some()),
        black_box(style.background.is_some()),
    )
}

#[library_benchmark]
fn the_walk_layout() -> (bool, bool) {
    aeolus_gpui::set_env(Default::default()); // the frame-Env escape hatch (plan §3.5's debug-assert keys on it)
    let program = aeolus_gpui::tw!("w-full h-10 px-3 py-2 border border-red-500 rounded-lg");
    let style = gpui::StyleRefinement::default().tw(program);
    (
        black_box(style.padding.top.is_some()),
        black_box(style.background.is_some()),
    )
}

#[library_benchmark]
fn the_walk_typo() -> (bool, bool) {
    aeolus_gpui::set_env(Default::default()); // the frame-Env escape hatch (plan §3.5's debug-assert keys on it)
    let program =
        aeolus_gpui::tw!("font-bold underline decoration-wavy decoration-2 decoration-red-500");
    let style = gpui::StyleRefinement::default().tw(program);
    (
        black_box(style.padding.top.is_some()),
        black_box(style.background.is_some()),
    )
}

#[library_benchmark]
fn the_walk_grid() -> (bool, bool) {
    aeolus_gpui::set_env(Default::default()); // the frame-Env escape hatch (plan §3.5's debug-assert keys on it)
    let program = aeolus_gpui::tw!("grid grid-cols-3 gap-4 h-32");
    let style = gpui::StyleRefinement::default().tw(program);
    (
        black_box(style.padding.top.is_some()),
        black_box(style.background.is_some()),
    )
}

#[library_benchmark]
fn the_walk_state() -> (bool, bool) {
    aeolus_gpui::set_env(Default::default()); // the frame-Env escape hatch (plan §3.5's debug-assert keys on it)
    let program =
        aeolus_gpui::tw!("hover:bg-red-500/90 focus:ring-2 focus:ring-red-500 bg-red-500");
    let style = gpui::StyleRefinement::default().tw(program);
    (
        black_box(style.padding.top.is_some()),
        black_box(style.background.is_some()),
    )
}

library_benchmark_group!(
    name = the_perf_contract;
    benchmarks =
        the_walk_button,
        the_walk_layout,
        the_walk_typo,
        the_walk_grid,
        the_walk_state
);

main!(library_benchmark_groups = the_perf_contract);
