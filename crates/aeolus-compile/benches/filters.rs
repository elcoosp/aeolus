//! The filter layer's microbenchmarks (divan, stable — the workspace
//! bench idiom): the plan §3.7 backlog row's families through the
//! pipeline's own surfaces. The measured costs are the per-frame
//! shapes a host runs — the compile of the filter families (the
//! registry walk over the ten new roots) beside the compile floor, the
//! CSS emit of the var-chain form (the primitive's var line + the
//! composed chain, the render the browser cascade consumes) and the
//! interp of a filter-mixed program (the primitive longhands riding
//! the per-longhand cascade) beside the element cascade.
//!
//! The bench bodies `expect` on the walks: a compiled filter program
//! resolves under any env (errors are caller bugs — unknown token ids,
//! non-finite alphas — and none of those can come out of the pipeline).
//!
//! Numbers land in `reports/08-benchmarks/BENCHMARKS.md` when the gate
//! environment runs them.

use aeolus_compile::{Config, DynCache, Env, compile, emit_css, interp};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The filter-mixed string: one of every primitive family plus the
/// composed reset — the longhand set the interp benches resolve.
const FILTER_CLASSES: &str = "blur-md brightness-110 contrast-125 grayscale invert \
     hue-rotate-30 saturate-150 sepia drop-shadow-md filter-none";

/// The compile floor: the shadcn Button core the other benches key on.
const BUTTON_CLASSES: &str = "inline-flex items-center justify-center gap-2 rounded-md \
     text-sm font-medium bg-primary text-primary-foreground hover:bg-primary/90";

/// Compile once (outside the measured loop) — the interp's input is the
/// interned program, exactly the cached-loop shape.
fn filter_program() -> aeolus_compile::Compiled {
    let cache = DynCache::new();
    cache
        .compile(FILTER_CLASSES, &Config::default())
        .expect("non-empty input compiles")
}

#[divan::bench]
fn compile_filter_families(bencher: Bencher) {
    let config = Config::default();
    bencher.bench(|| compile(FILTER_CLASSES, &config).expect("non-empty input compiles"));
}

/// The same compile WITHOUT the filter families — the floor the cost
/// reads against (the ten new roots' walks ride the same registry
/// read side every other family takes).
#[divan::bench]
fn compile_button_floor(bencher: Bencher) {
    let config = Config::default();
    bencher.bench(|| compile(BUTTON_CLASSES, &config).expect("non-empty input compiles"));
}

/// The CSS emit of the var-chain form: each primitive rule renders its
/// var line beside the composed chain — the stylesheet bytes the
/// browser cascade consumes.
#[divan::bench]
fn emit_css_filter_families(bencher: Bencher) {
    let config = Config::default();
    bencher.bench(|| emit_css(FILTER_CLASSES, &config).expect("the sheet"));
}

#[divan::bench]
fn interp_filter_program(bencher: Bencher) {
    let program = filter_program();
    let env = Env::default();
    bencher.bench(|| interp(&program.program, &env).expect("no theme forms in the program"));
}
