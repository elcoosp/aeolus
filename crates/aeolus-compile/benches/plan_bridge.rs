//! The plan-lowering arm's microbenchmarks (divan, stable — the
//! workspace bench idiom): the §7 migration seam's walk, class string
//! in, v0.x [`StylePlan`](aeolus_core::StylePlan) out. The per-class
//! costs the backend re-target story cares about: the FULL shadcn
//! Button string the other arm benches key on (the plan shape the five
//! backends consume), the theme-less walk over the same string (the
//! token boundaries' cost), and the single-utility floor.
//!
//! The bench bodies `expect` on the emit: a class string always
//! produces a plan (per-class reports ride the artifact — never a
//! whole-string failure), so the only `Err` is the empty input the
//! benches never pass.
//!
//! Numbers land in `reports/08-benchmarks/BENCHMARKS.md` when the gate
//! environment runs them (see `reports/10-tdd-evidence/22-plan-bridge.md`
//! for the red-to-green record).

use aeolus_compile::{Config, emit_plan, emit_plan_theme};
use aeolus_theme::{ThemeSource, TokenTable};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The shadcn v4 Button table's core classes — the string the
/// differential pins end to end (the full 33-class table carries the
/// arbitrary `[&_svg]` selectors and the `has-[>svg]` descendant form,
/// which stay the plan arm's typed notes today; the CORE classes are
/// the ones the backends lower into native properties).
const BUTTON_CORE_CLASSES: &str = "inline-flex shrink-0 items-center justify-center gap-2 rounded-md text-sm font-medium whitespace-nowrap outline-none focus-visible:ring-2 disabled:pointer-events-none disabled:opacity-50 bg-primary text-primary-foreground hover:bg-primary/90 h-9 px-4 py-2";

/// The baked-tier table over the embedded shadcn palettes (the same
/// seed the interp benches key on).
fn baked_table() -> TokenTable {
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    TokenTable::baked(&source).expect("default-seeded source builds")
}

/// The themed walk over the Button core: the full pipeline (lex →
/// fold → lookup → value pass → slot write → bucket route) with token
/// resolution riding every colour declaration.
#[divan::bench]
fn plan_theme_button_core(bencher: Bencher) {
    let table = baked_table();
    bencher.bench(|| {
        emit_plan_theme(BUTTON_CORE_CLASSES, &table, &Config::default())
            .expect("non-empty input emits")
    });
}

/// The theme-less walk over the same string: the colour tokens stop at
/// the typed boundary (their diagnostics ride the artifact) — the
/// boundary's cost, for the record.
#[divan::bench]
fn plan_themeless_button_core(bencher: Bencher) {
    bencher.bench(|| {
        emit_plan(BUTTON_CORE_CLASSES, &Config::default()).expect("non-empty input emits")
    });
}

/// The single-utility floor: one spacing class through the whole walk
/// (the per-class cost the multi-class strings amortize).
#[divan::bench]
fn plan_single_utility(bencher: Bencher) {
    bencher.bench(|| emit_plan("p-4", &Config::default()).expect("non-empty input emits"));
}
