//! The wider v4 hint table's microbenchmarks (divan, stable — the
//! workspace bench idiom): the plan §3.1 value pass's hint line through
//! the pipeline's own surfaces. The measured costs are the per-frame
//! shapes a host runs — the compile of the hinted battery (the hint
//! arms ride the same classify walk every other value form takes)
//! against the compile floor, the CSS emit of the unit-reconstructed
//! forms and the interp of a hint-mixed program.
//!
//! The bench bodies `expect` on the walks: a hinted program resolves
//! under any env (the Num/Verbatim slots carry no theme forms).
//!
//! Numbers land in `reports/08-benchmarks/BENCHMARKS.md` when the gate
//! environment runs them.

use aeolus_compile::{Config, DynCache, Env, compile, emit_css};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The hint-mixed string: one hinted form per landed family — the
/// longhand set the interp bench resolves.
const HINT_CLASSES: &str = "rotate-[angle:45deg] duration-[time:2s] \
     object-[position:50%_25%] font-[family-name:\"Inter_Sans\"] \
     text-[absolute-size:larger] content-[\"hello_world\"]";

/// The compile floor: the shadcn Button core the other benches key on.
const BUTTON_CLASSES: &str = "inline-flex items-center justify-center gap-2 rounded-md \
     text-sm font-medium bg-primary text-primary-foreground hover:bg-primary/90";

/// Compile once (outside the measured loop) — the interp's input is the
/// interned program, exactly the cached-loop shape.
fn hint_program() -> aeolus_compile::Compiled {
    let cache = DynCache::new();
    cache
        .compile(HINT_CLASSES, &Config::default())
        .expect("non-empty input compiles")
}

#[divan::bench]
fn compile_hint_battery(bencher: Bencher) {
    let config = Config::default();
    bencher.bench(|| compile(HINT_CLASSES, &config).expect("non-empty input compiles"));
}

/// The same compile WITHOUT the hints — the floor the cost reads
/// against (the hint arms ride the same classify walk every other
/// arbitrary form takes).
#[divan::bench]
fn compile_button_floor(bencher: Bencher) {
    let config = Config::default();
    bencher.bench(|| compile(BUTTON_CLASSES, &config).expect("non-empty input compiles"));
}

#[divan::bench]
fn css_emit_hint_battery(bencher: Bencher) {
    let config = Config::default();
    bencher.bench(|| emit_css(HINT_CLASSES, &config).expect("the CSS walk is total"));
}

#[divan::bench]
fn interp_hint_battery(bencher: Bencher) {
    let compiled = hint_program();
    let env = Env::default();
    bencher.bench(|| compiled.interp(&env).expect("the base situation resolves"));
}
