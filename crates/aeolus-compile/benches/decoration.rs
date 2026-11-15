//! The decoration family's microbenchmarks (divan, stable — the
//! workspace bench idiom): the §3.7 typography surface's decoration
//! half through the pipeline's own surfaces. The measured costs are
//! the shapes a host runs — the compile of the decoration rows (the
//! three-row root's sibling walk: the style keywords, the thickness
//! keywords + the `PositiveInt` integer form, the colour tokens) and
//! the CSS emit of the family (the four property forms the browser
//! cascade consumes), each beside the compile/emit floor the other
//! benches key on.
//!
//! The bench bodies `expect` on the walks: a compiled decoration
//! program resolves under any env (errors are caller bugs — unknown
//! token ids, non-finite alphas — and none of those can come out of
//! the pipeline).
//!
//! Numbers land in `reports/08-benchmarks/BENCHMARKS.md` when the gate
//! environment runs them.

use aeolus_compile::{Config, DynCache, compile, emit_css};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The decoration-mixed string: the style, thickness and colour
/// keywords plus the integer and arbitrary forms — the row set the
/// benches walk.
const DECORATION_CLASSES: &str = "decoration-solid decoration-wavy decoration-auto \
     decoration-from-font decoration-2 decoration-[3px] decoration-[#0f0f0f]";

/// The compile floor: the shadcn Button core the other benches key on.
const BUTTON_CLASSES: &str = "inline-flex items-center justify-center gap-2 rounded-md \
     text-sm font-medium bg-primary text-primary-foreground hover:bg-primary/90";

#[divan::bench]
fn decoration_compile(bencher: Bencher) {
    bencher.bench(|| compile(DECORATION_CLASSES, &Config::default()).expect("compiles"));
}

#[divan::bench]
fn button_compile_floor(bencher: Bencher) {
    bencher.bench(|| compile(BUTTON_CLASSES, &Config::default()).expect("compiles"));
}

/// The CSS emit of the family beside the floor: the four property
/// forms (line, style, thickness, colour) one rule carries.
#[divan::bench]
fn decoration_css_emit(bencher: Bencher) {
    bencher.bench(|| {
        emit_css(
            "underline decoration-wavy decoration-2 decoration-[#0f0f0f]",
            &Config::default(),
        )
        .expect("compiles")
    });
}

#[divan::bench]
fn button_css_emit_floor(bencher: Bencher) {
    bencher.bench(|| emit_css(BUTTON_CLASSES, &Config::default()).expect("compiles"));
}

/// The cached-loop shape: the decoration string through the dyn cache
/// (the per-frame warm path the ClassCell lane rides).
#[divan::bench]
fn decoration_cached_compile(bencher: Bencher) {
    let cache = DynCache::new();
    bencher.bench(|| {
        cache
            .compile(DECORATION_CLASSES, &Config::default())
            .expect("compiles")
    });
}
