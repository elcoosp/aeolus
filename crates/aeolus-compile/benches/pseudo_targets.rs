//! The pseudo-element render-target microbenchmarks (divan, stable —
//! the workspace bench idiom): the plan §3.5 render-target row's
//! RESOLUTION half, the scoped walk the [`PseudoTarget`] arm adds. The
//! measured cost is the per-frame pseudo-cascade resolve: one pass over
//! the interned layers, skipping everything that is not the target's
//! own — the walk a `::selection`/`::placeholder` render path would
//! run beside the element cascade.
//!
//! The bench bodies `expect` on the walk: a compiled program resolves
//! under any target (errors are caller bugs — unknown token ids,
//! non-finite alphas — and none of those can come out of the pipeline).
//!
//! Numbers land in `reports/08-benchmarks/BENCHMARKS.md` when the gate
//! environment runs them.

use aeolus_compile::{Config, DynCache, Env, PseudoTarget, interp_pseudo, interp_pseudo_theme};
use aeolus_theme::{ThemeSource, TokenTable};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The pseudo-mixed string: one `::selection` rule, one `::placeholder`
/// rule, a base declaration and a hover layer — the four-layer program
/// the target walk scopes against.
const PSEUDO_CLASSES: &str =
    "selection:bg-[#eab308] placeholder:text-[#ef4444] p-4 hover:bg-[#b91c1c]";

/// The runtime-tier table over the shadcn source (the themed bench's
/// token forms).
fn runtime_table() -> TokenTable {
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    TokenTable::runtime(&source).expect("default-seeded source builds")
}

/// Compile once (outside the measured loop) — the interp's input is the
/// interned program, exactly the cached-loop shape.
fn pseudo_program(table: &TokenTable) -> aeolus_compile::Compiled {
    let cache = DynCache::new();
    cache
        .compile_theme(PSEUDO_CLASSES, table, &Config::default())
        .expect("non-empty input compiles")
}

/// The selection target over the mixed program: exactly one layer
/// participates — one pass, three skips, one cascade.
#[divan::bench]
fn interp_pseudo_selection(bencher: Bencher) {
    let table = runtime_table();
    let compiled = pseudo_program(&table);
    bencher.bench(|| {
        interp_pseudo(&compiled.program, PseudoTarget::Selection).expect("the program resolves")
    });
}

/// The placeholder target over the same program — the OTHER pseudo's
/// cascade (the scoped-empty path's cost, for the record).
#[divan::bench]
fn interp_pseudo_placeholder(bencher: Bencher) {
    let table = runtime_table();
    let compiled = pseudo_program(&table);
    bencher.bench(|| {
        interp_pseudo(&compiled.program, PseudoTarget::Placeholder).expect("the program resolves")
    });
}

/// The themed form over the same program: the selection layer's token
/// forms resolve through the source beside the target (the DynCache
/// loop's shape on the pseudo render path).
#[divan::bench]
fn interp_pseudo_theme_selection(bencher: Bencher) {
    let table = runtime_table();
    let compiled = pseudo_program(&table);
    let env = Env::default();
    bencher.bench(|| {
        interp_pseudo_theme(&compiled.program, &table, &env, PseudoTarget::Selection)
            .expect("the program resolves")
    });
}
