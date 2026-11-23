//! The frame `Env`'s debug-assert microbenchmarks (divan, stable — the
//! workspace bench idiom): the cost the §3.5/§9 mitigation rides on
//! the read the backends make every walk.
//!
//! The assert is DEBUG-only (the marker's reader compiles out in
//! release — the empty `mark_frame` body inlines away), so these
//! numbers are the RELEASE shape's floor: the TLS `Env` load the
//! `.tw()` path always paid, plus the `set_env` escape hatch's one
//! mark + store. The debug build's own cost is the assert's business
//! (a debug-only branch on the thread-local marker).
//!
//! Numbers land in `reports/08-benchmarks/BENCHMARKS.md` when the
//! bench environment runs them.
use aeolus_gpui::env;
use aeolus_gpui::prelude::{Env, set_env};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The read the walk path makes per application, on a marked thread —
/// the release shape (no assert branch) and the debug shape's baseline
/// both key on this call.
#[divan::bench]
fn env_read(bencher: Bencher) {
    set_env(Env::default());
    bencher.bench(env);
}

/// The escape hatch itself: one mark + one TLS store — the cost a
/// test or a headless walk pays once per thread.
#[divan::bench]
fn set_env_call(bencher: Bencher) {
    bencher.bench(|| set_env(Env::default()));
}
