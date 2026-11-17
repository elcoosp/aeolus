//! The style face's microbenchmarks (divan, stable — the workspace
//! bench idiom): the `StaticProgram::style*` methods, the free walk
//! entries' delegates on the host surface. The delegation cost is the
//! number the face exists to bound — the host-surface call must be the
//! free walk's own cost, the slices threaded once by the type.
//!
//! Numbers land in `reports/08-benchmarks/BENCHMARKS.md` when the gate
//! environment runs them.

use aeolus_compile::{Config, Env, PseudoTarget, program_style, program_style_pseudo};
use aeolus_rt::{ClassCell, StaticProgram};
use aeolus_theme::{ThemeSource, TokenTable};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The baked shadcn LIGHT table (T0's shape) — the freeze walk's own
/// construction.
fn baked_table() -> TokenTable {
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    TokenTable::baked(&source).expect("the baked shadcn table builds")
}

/// Build a static program through the dynamic lane's own compile (the
/// ClassCell is the runtime path's constructor): the shadcn Button
/// string grown by a selection layer — one program, both faces.
fn frozen(classes: &str) -> StaticProgram {
    let cell = ClassCell::new();
    cell.get(classes, &Config::default())
        .expect("the cell compiles what lowers")
}

/// The shadcn v4 Button core grown by a selection recolour — one
/// program, the element walk and the pseudo-target cascade both
/// answer it.
const CORPUS: &str = "inline-flex items-center justify-center gap-2 rounded-md bg-primary h-9 px-4 py-2 selection:bg-primary/90";

/// The element walk through the host surface's method.
#[divan::bench]
fn style_face_element(bencher: Bencher) {
    let program = frozen(CORPUS);
    let env = Env::default();
    bencher.bench(|| program.style(&env));
}

/// The element walk through the free entry the method delegates to:
/// the delegation-parity pin's other half (the same slices, threaded
/// by hand).
#[divan::bench]
fn style_free_element(bencher: Bencher) {
    let program = frozen(CORPUS);
    let env = Env::default();
    bencher.bench(|| program_style(program.layers, program.keywords, &env));
}

/// The pseudo-target cascade through the host surface's method.
#[divan::bench]
fn style_face_pseudo(bencher: Bencher) {
    let program = frozen(CORPUS);
    bencher.bench(|| program.style_pseudo(PseudoTarget::Selection));
}

/// The pseudo-target cascade through the free entry.
#[divan::bench]
fn style_free_pseudo(bencher: Bencher) {
    let program = frozen(CORPUS);
    bencher
        .bench(|| program_style_pseudo(PseudoTarget::Selection, program.layers, program.keywords));
}

/// The themed pseudo cascade: the runtime-tier slots resolve through
/// the source (T2's one load + one multiply on the host face).
#[divan::bench]
fn style_face_pseudo_themed(bencher: Bencher) {
    let cell = ClassCell::new();
    let table = baked_table();
    let program = cell
        .get_theme(CORPUS, &table, &Config::default())
        .expect("the themed cell compiles");
    let env = Env::default();
    bencher.bench(|| program.style_pseudo_theme(PseudoTarget::Selection, &table, &env));
}
