//! The grid auto-flow family's microbenchmarks (divan, stable — the
//! workspace bench idiom): the plan §3.7 grid row's auto-flow half
//! through the pipeline's own surfaces. The measured costs are the
//! per-frame shapes a host runs — the compile of the family (the
//! keyword-row walk over the five slots) against the compile floor, the
//! CSS emit of the value form and the interp of a flow-mixed program.
//!
//! The bench bodies `expect` on the walks: a compiled flow program
//! resolves under any env (keyword slots carry no theme forms).
//!
//! Numbers land in `reports/08-benchmarks/BENCHMARKS.md` when the gate
//! environment runs them.

use aeolus_compile::{Config, DynCache, Env, compile, emit_css, interp};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The flow-mixed string: the whole keyword scale — the longhand set
/// the interp bench resolves.
const FLOW_CLASSES: &str = "grid-flow-row grid-flow-col grid-flow-dense \
     grid-flow-row-dense grid-flow-col-dense";

/// The compile floor: the shadcn Button core the other benches key on.
const BUTTON_CLASSES: &str = "inline-flex items-center justify-center gap-2 rounded-md \
     text-sm font-medium bg-primary text-primary-foreground hover:bg-primary/90";

/// Compile once (outside the measured loop) — the interp's input is the
/// interned program, exactly the cached-loop shape.
fn flow_program() -> aeolus_compile::Compiled {
    let cache = DynCache::new();
    cache
        .compile(FLOW_CLASSES, &Config::default())
        .expect("non-empty input compiles")
}

#[divan::bench]
fn compile_grid_flow_families(bencher: Bencher) {
    let config = Config::default();
    bencher.bench(|| compile(FLOW_CLASSES, &config).expect("non-empty input compiles"));
}

/// The same compile WITHOUT the family — the floor the cost reads
/// against (the keyword walk rides the same registry read side every
/// other family takes).
#[divan::bench]
fn compile_button_floor(bencher: Bencher) {
    let config = Config::default();
    bencher.bench(|| compile(BUTTON_CLASSES, &config).expect("non-empty input compiles"));
}

/// The CSS emit of the value form: each class's rule renders the v4
/// `grid-auto-flow` value the cascade consumes.
#[divan::bench]
fn emit_css_grid_flow_families(bencher: Bencher) {
    let config = Config::default();
    bencher.bench(|| emit_css(FLOW_CLASSES, &config).expect("the sheet"));
}

#[divan::bench]
fn interp_grid_flow_program(bencher: Bencher) {
    let program = flow_program();
    let env = Env::default();
    bencher.bench(|| interp(&program.program, &env).expect("no theme forms in the program"));
}
