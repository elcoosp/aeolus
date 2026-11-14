//! The backdrop-filter layer's microbenchmarks (divan, stable — the
//! workspace bench idiom): the plan §3.7 backlog row's second half
//! through the pipeline's own surfaces. The measured costs are the
//! per-frame shapes a host runs — the compile of the backdrop families
//! (the registry walk over the ten new roots) beside the compile
//! floor, the CSS emit of the prefixed var-chain form (the primitive's
//! var line beside BOTH composed properties, the render the browser
//! cascade consumes) and the interp of a backdrop-mixed program (the
//! primitive longhands riding the per-longhand cascade).
//!
//! The bench bodies `expect` on the walks: a compiled backdrop program
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

/// The backdrop-mixed string: one of every primitive family plus the
/// composed reset — the longhand set the interp benches resolve.
const BACKDROP_CLASSES: &str = "backdrop-blur-md backdrop-brightness-90 backdrop-contrast-125 \
     backdrop-grayscale backdrop-invert backdrop-hue-rotate-30 backdrop-opacity-50 \
     backdrop-saturate-150 backdrop-sepia backdrop-filter-none";

/// The compile floor: the shadcn Button core the other benches key on.
const BUTTON_CLASSES: &str = "inline-flex items-center justify-center gap-2 rounded-md \
     text-sm font-medium bg-primary text-primary-foreground hover:bg-primary/90";

/// Compile once (outside the measured loop) — the interp's input is the
/// interned program, exactly the cached-loop shape.
fn backdrop_program() -> aeolus_compile::Compiled {
    let cache = DynCache::new();
    cache
        .compile(BACKDROP_CLASSES, &Config::default())
        .expect("non-empty input compiles")
}

#[divan::bench]
fn compile_backdrop_families(bencher: Bencher) {
    let config = Config::default();
    bencher.bench(|| compile(BACKDROP_CLASSES, &config).expect("non-empty input compiles"));
}

/// The same compile WITHOUT the backdrop families — the floor the cost
/// reads against (the ten new roots' walks ride the same registry read
/// side every other family takes).
#[divan::bench]
fn compile_button_floor(bencher: Bencher) {
    let config = Config::default();
    bencher.bench(|| compile(BUTTON_CLASSES, &config).expect("non-empty input compiles"));
}

/// The CSS emit of the prefixed var-chain form: each primitive rule
/// renders its var line beside BOTH composed properties — the
/// stylesheet bytes the browser cascade consumes.
#[divan::bench]
fn emit_css_backdrop_families(bencher: Bencher) {
    let config = Config::default();
    bencher.bench(|| emit_css(BACKDROP_CLASSES, &config).expect("the sheet"));
}

#[divan::bench]
fn interp_backdrop_program(bencher: Bencher) {
    let program = backdrop_program();
    let env = Env::default();
    bencher.bench(|| interp(&program.program, &env).expect("no theme forms in the program"));
}
