//! The attribute-probe fold's microbenchmarks (divan, stable — the
//! workspace bench idiom): the data-/aria-/has- fold against the state
//! fold and the base compile floor. The probe payloads intern into the
//! leak pool per folded segment (the IR's documented dynamic-form
//! storage), so the interesting number is the COLD fold — one compile
//! per distinct probe spelling is the intended call pattern, and the
//! dyn-cache benches cover the warm repeat.
//!
//! Numbers land in `reports/08-benchmarks/BENCHMARKS.md` when the gate
//! environment runs them (see `reports/10-tdd-evidence/19-rust-tokens-arm.md`
//! for the red-to-green record).

use aeolus_compile::Config;
use divan::Bencher;

fn main() {
    divan::main();
}

/// The compile floor: one bare utility (the base layer, no fold).
const BASE: &str = "p-4";

/// The state fold: the established rank-2 probe path.
const STATE: &str = "hover:bg-primary";

/// The corpus's own attribute-probe spellings (the golden corpus's
/// `data-[state=open]:bg-green-500`, the bare aria form, the has-
/// family's three shapes) — one string, three probe segments, three
/// payload interns.
const PROBES: &str = "data-[state=open]:bg-green-500 aria-busy:opacity-50 has-focus:ring-2 has-data-[slot=card-action]:grid-cols-2";

#[divan::bench]
fn compile_base(bencher: Bencher) {
    let config = Config::default();
    bencher.bench(|| aeolus_compile::compile(BASE, &config).expect("compiles"));
}

#[divan::bench]
fn compile_state_fold(bencher: Bencher) {
    let config = Config::default();
    bencher.bench(|| aeolus_compile::compile(STATE, &config).expect("compiles"));
}

#[divan::bench]
fn compile_probe_fold(bencher: Bencher) {
    let config = Config::default();
    bencher.bench(|| aeolus_compile::compile(PROBES, &config).expect("compiles"));
}
