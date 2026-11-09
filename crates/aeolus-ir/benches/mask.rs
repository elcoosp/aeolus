//! Mask-algebra microbenchmarks (divan, stable) — the bit operations the
//! plan §3.3 promises ("`p-4 ⊕ px-8` is just bit operations") measured at
//! the cold paths a compile stage and the `cn()` merge actually hit.

use aeolus_ir::{Cond, Decl, Layer, Program, Prop, PropMask, Val};
use divan::Bencher;

const PADDING: &[Prop] = &[
    Prop::PaddingTop,
    Prop::PaddingRight,
    Prop::PaddingBottom,
    Prop::PaddingLeft,
];
const PX: &[Prop] = &[Prop::PaddingRight, Prop::PaddingLeft];

fn main() {
    divan::main();
}

/// Build the four-edge mask from a registry target slice (the per-row
/// `PropMask::of` the compile stage runs once per utility).
#[divan::bench]
fn mask_of_trbl(bencher: Bencher) {
    bencher.bench(|| PropMask::of(PADDING));
}

/// The overlay merge `p-4 ⊕ px-8` — the `cn()` inner loop.
#[divan::bench]
fn mask_union_overlay(bencher: Bencher) {
    let p4 = PropMask::of(PADDING);
    let px8 = PropMask::of(PX);
    bencher.bench(|| p4.union(px8));
}

/// Dominance check (`p-4` covering `px-8`) — the eviction test.
#[divan::bench]
fn mask_covers(bencher: Bencher) {
    let p4 = PropMask::of(PADDING);
    let px8 = PropMask::of(PX);
    bencher.bench(|| p4.covers(px8));
}

/// Full walk of a dense mask (every longhand bit) — the emit-side
/// declaration iteration.
#[divan::bench]
fn mask_for_each_dense(bencher: Bencher) {
    bencher.bench(|| {
        // A throwaway sink per iteration; the walk itself is the cost
        // under measurement.
        PropMask::of(Prop::ALL).for_each(|_| ());
    });
}

/// Layer/program assembly from declarations (the pipeline's output step).
#[divan::bench]
fn program_assembly(bencher: Bencher) {
    let decls: &'static [Decl] = Box::leak(
        vec![
            Decl {
                prop: Prop::PaddingTop,
                val: Val::Rem(1.0),
            },
            Decl {
                prop: Prop::PaddingRight,
                val: Val::Rem(2.0),
            },
            Decl {
                prop: Prop::PaddingBottom,
                val: Val::Rem(1.0),
            },
            Decl {
                prop: Prop::PaddingLeft,
                val: Val::Rem(2.0),
            },
        ]
        .into_boxed_slice(),
    );
    bencher.bench(|| {
        let base = Layer::new(Cond::Base, decls);
        let hover = Layer::new(Cond::Hover, decls);
        let layers: &'static [Layer] = Box::leak(vec![base, hover].into_boxed_slice());
        Program::new(layers)
    });
}
