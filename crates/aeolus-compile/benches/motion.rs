//! The motion layer's microbenchmarks (divan, stable — the workspace
//! bench idiom): the plan P6 row's families through the pipeline's own
//! surfaces. The measured costs are the per-frame shapes a host runs —
//! the compile of the motion families (the registry walk over the five
//! new roots) beside the compile floor, and the interp of a
//! motion-mixed program (the transition quartet + the animation slot
//! riding the per-longhand cascade) beside the element cascade.
//!
//! The bench bodies `expect` on the walks: a compiled motion program
//! resolves under any env (errors are caller bugs — unknown token ids,
//! non-finite alphas — and none of those can come out of the pipeline).
//!
//! Numbers land in `reports/08-benchmarks/BENCHMARKS.md` when the gate
//! environment runs them.

use aeolus_compile::{Config, DynCache, Env, compile, interp, interp_theme};
use aeolus_theme::{ThemeSource, TokenTable};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The motion-mixed string: one of every family — the transition set,
/// the duration/ease/delay scales and the animation preset — the
/// five-longhand program the interp benches resolve.
const MOTION_CLASSES: &str = "transition-colors duration-300 ease-in-out delay-150 animate-pulse";

/// The compile floor: the shadcn Button core the other benches key on.
const BUTTON_CLASSES: &str = "inline-flex items-center justify-center gap-2 rounded-md \
     text-sm font-medium bg-primary text-primary-foreground hover:bg-primary/90";

/// The runtime-tier table over the shadcn source (the themed bench's
/// token forms).
fn runtime_table() -> TokenTable {
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    TokenTable::runtime(&source).expect("default-seeded source builds")
}

/// Compile once (outside the measured loop) — the interp's input is the
/// interned program, exactly the cached-loop shape.
fn motion_program() -> aeolus_compile::Compiled {
    let cache = DynCache::new();
    cache
        .compile(MOTION_CLASSES, &Config::default())
        .expect("non-empty input compiles")
}

#[divan::bench]
fn compile_motion_families(bencher: Bencher) {
    let config = Config::default();
    bencher.bench(|| compile(MOTION_CLASSES, &config).expect("non-empty input compiles"));
}

/// The same compile WITHOUT the motion families — the floor the cost
/// reads against (the five new roots' walks ride the same registry
/// read side every other family takes).
#[divan::bench]
fn compile_button_floor(bencher: Bencher) {
    let config = Config::default();
    bencher.bench(|| compile(BUTTON_CLASSES, &config).expect("non-empty input compiles"));
}

#[divan::bench]
fn interp_motion_program(bencher: Bencher) {
    let program = motion_program();
    let env = Env::default();
    bencher.bench(|| interp(&program.program, &env).expect("no theme forms in the program"));
}

/// The themed interp: the animation slot rides the same walk with a
/// token source supplied (the T2 tier's one-load-one-multiply).
#[divan::bench]
fn interp_motion_program_theme(bencher: Bencher) {
    let program = motion_program();
    let table = runtime_table();
    let env = Env::default();
    bencher.bench(|| interp_theme(&program.program, &table, &env).expect("the themed resolve"));
}
