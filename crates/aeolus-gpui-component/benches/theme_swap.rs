//! The swap path's microbenchmark (the M2 batch's bench arm): the
//! theme hot-swap's own costs — the absorb (the merge + the table
//! rebuild), the post-swap style resolve (the ClassCell HIT + the
//! re-resolution) and the pre-swap resolve beside them. The no-re-parse
//! contract is the benchmark's subject: the post-swap resolve must ride
//! the compiled program, never the compiler.

use divan::Bencher;

use aeolus_gpui_component::{AeolusTheme, Env};
use aeolus_rt::ClassCell;
use gpui::Styled;

const LIGHT: &str = include_str!("../examples/fixtures/light.css");
const DARK: &str = include_str!("../examples/fixtures/dark.css");

/// The shadcn Button's semantic surface (the demo string the pins ride).
const BUTTON: &str = "bg-primary text-primary-foreground hover:bg-primary/90 px-4 py-2 rounded-md text-sm font-medium";

fn main() {
    divan::main();
}

/// The pre-swap resolve: the compile (miss) then the hit — the dynamic
/// lane's own shape.
#[divan::bench]
fn style_resolve_pre_swap(bencher: Bencher) {
    let theme = AeolusTheme::from_css(LIGHT, None).expect("the sheet parses");
    let cell = ClassCell::new();
    let env = Env::default();
    bencher.bench(|| theme.style(gpui::StyleRefinement::default(), &cell, BUTTON, env));
}

/// The absorb itself: the merge, the in-place table rebuild, the mirror
/// re-projection and the §3.4 drops (the version bump rides).
#[divan::bench]
fn absorb_css(bencher: Bencher) {
    bencher.bench(|| {
        let mut theme = AeolusTheme::from_css(LIGHT, None).expect("the sheet parses");
        theme.absorb_css(DARK).expect("the swap merges");
    });
}

/// The post-swap resolve: the SAME cell and string after the swap — the
/// ClassCell hit (no re-parse) plus the walk-time resolution against the
/// swapped values, at the steady state (the warm-up arm compiles once
/// and swaps once OUTSIDE the measured loop). The contract's own arm:
/// this must cost the resolution, not the compiler.
#[divan::bench]
fn style_resolve_post_swap(bencher: Bencher) {
    let mut theme = AeolusTheme::from_css(LIGHT, None).expect("the sheet parses");
    let cell = ClassCell::new();
    let env = Env::default();
    let warm = theme.style(gpui::StyleRefinement::default(), &cell, BUTTON, env);
    theme.absorb_css(DARK).expect("the swap merges");
    theme.set_dark(true);
    assert!(
        warm.background.is_some(),
        "the warm resolve carried the token"
    );
    bencher.bench(|| theme.style(gpui::StyleRefinement::default(), &cell, BUTTON, env));
}
