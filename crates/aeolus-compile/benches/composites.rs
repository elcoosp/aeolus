//! The composite grammar's microbenchmarks (divan, stable — the
//! workspace bench idiom): the value pass's verbatim slot through the
//! pipeline's own surfaces. The measured costs are the per-frame shapes
//! a host runs — the compile of the composite families (the kind gate's
//! decode + balance + eligibility walk beside the registry read side)
//! against the compile floor, the CSS emit of the verbatim rule text
//! and the interp of a composite-mixed program (the verbatim longhands
//! riding the per-longhand cascade).
//!
//! The bench bodies `expect` on the walks: a compiled composite program
//! resolves under any env (the verbatim slot is env-independent — no
//! theme forms ride the fixture).
//!
//! Numbers land in `reports/08-benchmarks/BENCHMARKS.md` when the gate
//! environment runs them.

use aeolus_compile::{Config, DynCache, Env, compile, emit_css, interp};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The composite-mixed string: one of every eligible shape — the sized
/// function, the track list, the track function, the image function and
/// the bare ratio — the longhand set the interp bench resolves.
const COMPOSITE_CLASSES: &str = "w-[calc(100%-2rem)] grid-cols-[auto_1fr_auto] \
     grid-rows-[minmax(0,1fr)_2fr] bg-[url(/img/hero.png)] aspect-[16/9]";

/// The compile floor: the shadcn Button core the other benches key on.
const BUTTON_CLASSES: &str = "inline-flex items-center justify-center gap-2 rounded-md \
     text-sm font-medium bg-primary text-primary-foreground hover:bg-primary/90";

/// Compile once (outside the measured loop) — the interp's input is the
/// interned program, exactly the cached-loop shape.
fn composite_program() -> aeolus_compile::Compiled {
    let cache = DynCache::new();
    cache
        .compile(COMPOSITE_CLASSES, &Config::default())
        .expect("non-empty input compiles")
}

#[divan::bench]
fn compile_composite_families(bencher: Bencher) {
    let config = Config::default();
    bencher.bench(|| compile(COMPOSITE_CLASSES, &config).expect("non-empty input compiles"));
}

/// The same compile WITHOUT the composites — the floor the cost reads
/// against (the verbatim slot's walk rides the same registry read side
/// every other family takes).
#[divan::bench]
fn compile_button_floor(bencher: Bencher) {
    let config = Config::default();
    bencher.bench(|| compile(BUTTON_CLASSES, &config).expect("non-empty input compiles"));
}

/// The CSS emit of the verbatim rule text: each eligible row renders
/// its decoded text as typed — the stylesheet bytes the browser
/// cascade consumes.
#[divan::bench]
fn emit_css_composite_families(bencher: Bencher) {
    let config = Config::default();
    bencher.bench(|| emit_css(COMPOSITE_CLASSES, &config).expect("the sheet"));
}

#[divan::bench]
fn interp_composite_program(bencher: Bencher) {
    let program = composite_program();
    let env = Env::default();
    bencher.bench(|| interp(&program.program, &env).expect("no theme forms in the program"));
}
