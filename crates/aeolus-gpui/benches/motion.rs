//! The motion lane's microbenchmarks (divan, stable — the workspace
//! bench idiom): the frozen-shorthand parse, the animation build and
//! the cascade winner walk the lane's entries run per apply.
//!
//! The parse is one pass over a five-token text; the winner walk is
//! one pass over the interned layers — the same cost class the base
//! walk's condition matching runs. The three arms pin the cost split:
//! the shorthand parse (per distinct text), the built `Animation` (the
//! parse plus one `Rc` easing allocation) and the winner walk (per
//! apply, the cell-compiled program the lane consumes).

use aeolus_compile::Config;
use aeolus_gpui::animation_for;
use aeolus_gpui::animation_keyword;
use aeolus_gpui::parse_animation;
use aeolus_gpui::prelude::*;
use aeolus_rt::ClassCell;
use divan::Bencher;

fn main() {
    divan::main();
}

/// The five v4 default-theme animate shorthands verbatim (the
/// `ANIMATE_PRESETS` texts the registry freezes) — the parse bench's
/// corpus.
const PRESETS: [&str; 5] = [
    "spin 1s linear infinite",
    "ping 1s cubic-bezier(0, 0, 0.2, 1) infinite",
    "pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite",
    "bounce 1s infinite",
    "none",
];

#[divan::bench]
fn parse_shorthands(bencher: Bencher) {
    bencher.bench(|| {
        for text in PRESETS {
            let _ = parse_animation(text);
        }
    });
}

#[divan::bench]
fn build_animation(bencher: Bencher) {
    bencher.bench(|| {
        for text in PRESETS {
            let _ = animation_for(text);
        }
    });
}

#[divan::bench]
fn winner_walk(bencher: Bencher) {
    // The lane's winner selection over the cell-compiled motion
    // string: one pass over the layers, the last matching `Animation`
    // declaration answering.
    let cell = ClassCell::new();
    let program = cell
        .get("animate-spin dark:animate-pulse", &Config::default())
        .expect("the cell compiles the motion string");
    let dark = Env {
        dark: true,
        ..Env::default()
    };
    bencher.bench(|| animation_keyword(&program, &dark));
}
