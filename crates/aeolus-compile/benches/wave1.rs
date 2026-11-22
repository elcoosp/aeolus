//! The parity wave 1's microbenchmarks (divan, stable — the workspace
//! bench idiom): the compile seam's new emission shapes — the CSS arm's
//! child-selector rules (the between-child families), the clamp
//! composite's chain-append pass, and the plan bridge's wave-1 arms —
//! beside the plain emission floors they appended to.

use divan::Bencher;

use aeolus_compile::{Config, emit_css, emit_plan};

fn main() {
    divan::main();
}

/// The between-child rule's full emission (the selector wrap + the
/// reverse-var machinery lines — the child-selector shape's own cost).
#[divan::bench]
fn space_css_emit(bencher: Bencher) {
    bencher.bench(|| emit_css("space-x-4", &Config::default()));
}

/// The divide family's emission (the width/style/colour arms' composite
/// lines — the biggest child rule the wave appends).
#[divan::bench]
fn divide_css_emit(bencher: Bencher) {
    bencher.bench(|| emit_css("divide-y-2 divide-dashed", &Config::default()));
}

/// The clamp composite's emission (the overflow/-webkit-box chain
/// append beside the clamp value).
#[divan::bench]
fn line_clamp_css_emit(bencher: Bencher) {
    bencher.bench(|| emit_css("line-clamp-2", &Config::default()));
}

/// The sr-only composite's emission (the fifteen-row fan-out — the
/// composite shape's own cost, the truncate precedent's bigger
/// sibling).
#[divan::bench]
fn sr_only_css_emit(bencher: Bencher) {
    bencher.bench(|| emit_css("sr-only", &Config::default()));
}

/// The plain emission floor the wave-1 shapes appended beside (the
/// spacing + colour string the other arms prefix).
#[divan::bench]
fn plain_css_floor(bencher: Bencher) {
    bencher.bench(|| emit_css("p-4 bg-primary", &Config::default()));
}

/// The plan bridge's wave-1 arm (the clamp composite's two-slot mapping
/// onto the v0.x oracle's text vec — the differential's own path).
#[divan::bench]
fn line_clamp_plan_bridge(bencher: Bencher) {
    bencher.bench(|| emit_plan("line-clamp-2", &Config::default()));
}
