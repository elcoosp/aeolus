//! The descendant-probe seam's microbenchmarks (divan, stable — the
//! workspace bench idiom): the `has-*` family's walk cost, the seam's
//! own slice of the interp arm's per-frame story. The built-in table
//! answers by a linear scan over a handful of facts (one row per
//! matched shape the frame declares), so the per-layer question is the
//! walk's dominant term — the same shape the attribute seam's bench
//! records for its own family.
//!
//! The bench bodies `expect` on the interp: a compiled program resolves
//! under any env (errors are caller bugs — unknown ids, non-finite
//! alphas — and none of those can come out of the pipeline).
//!
//! Numbers land in `reports/08-benchmarks/BENCHMARKS.md` when the gate
//! environment runs them (see `reports/10-tdd-evidence/22-descendant-seam.md`
//! for the red-to-green record).

use aeolus_compile::{
    Config, DescendantFact, DescendantKind, DescendantState, DynCache, Env, interp,
    interp_descendants,
};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The subtree facts the table answers for: the element's own subtree
/// matches the focus and hover shapes and the card-action data shape;
/// the group ancestor's matches focus. The realistic per-frame set —
/// the shapes a card component's interactions actually declare.
const TREE_FACTS: &[DescendantFact<'static>] = &[
    DescendantFact {
        kind: DescendantKind::Element,
        fragment: "*:focus",
    },
    DescendantFact {
        kind: DescendantKind::Element,
        fragment: "*:hover",
    },
    DescendantFact {
        kind: DescendantKind::Element,
        fragment: "*[data-slot=\"card-action\"]",
    },
    DescendantFact {
        kind: DescendantKind::Group,
        fragment: "*:focus",
    },
];

/// The shadcn v4 Button string's descendant half (the
/// `has-[>svg]:px-3` class among the 33 the other benches key on,
/// plus the fold's own data/aria compounds) — the layers the seam
/// answers in the pipeline's real consumer.
const BUTTON_CLASSES: &str =
    "inline-flex items-center gap-2 has-[>svg]:px-3 has-data-[slot=card-action]:p-8 has-focus:p-8";

#[divan::bench]
fn interp_descendants_button(bencher: Bencher) {
    let cache = DynCache::new();
    let compiled = cache
        .compile(BUTTON_CLASSES, &Config::default())
        .expect("non-empty input compiles");
    let table = DescendantState::new(TREE_FACTS);
    let env = Env::default();
    bencher.bench(|| {
        interp_descendants(&compiled.program, &env, &table).expect("the program resolves")
    });
}

/// The same program through the SOURCELESS walk: every descendant
/// layer is the reported exclusion — the note path's cost, for the
/// record (the plain `interp` the backends' pre-seam walks used).
#[divan::bench]
fn interp_sourceless_button(bencher: Bencher) {
    let cache = DynCache::new();
    let compiled = cache
        .compile(BUTTON_CLASSES, &Config::default())
        .expect("non-empty input compiles");
    let env = Env::default();
    bencher.bench(|| interp(&compiled.program, &env).expect("the program resolves"));
}
