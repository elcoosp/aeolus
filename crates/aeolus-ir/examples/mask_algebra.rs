//! Consumer demonstration: the plan §3.3 promise that per-edge overlay
//! (`p-4 ⊕ px-8`) and conditional dominance (`hover:px-8` vs `p-4`) are
//! bit operations on `PropMask` — no tables, no per-backend machinery.
//! The whole program is `const`-constructible: the static form is a
//! `static` item end to end.
//!
//! Run: `cargo run -p aeolus-ir --example mask_algebra`

use aeolus_ir::{Cond, Decl, Layer, Program, Prop, PropMask, Val};

const PADDING: &[Prop] = &[
    Prop::PaddingTop,
    Prop::PaddingRight,
    Prop::PaddingBottom,
    Prop::PaddingLeft,
];
const PADDING_X: &[Prop] = &[Prop::PaddingRight, Prop::PaddingLeft];

/// `p-4`: every padding edge, `1rem` (the default 4 × 0.25rem step).
const BASE: Layer = Layer {
    cond: Cond::Base,
    decls: &[
        Decl {
            prop: Prop::PaddingTop,
            val: Val::Rem(1.0),
        },
        Decl {
            prop: Prop::PaddingRight,
            val: Val::Rem(1.0),
        },
        Decl {
            prop: Prop::PaddingBottom,
            val: Val::Rem(1.0),
        },
        Decl {
            prop: Prop::PaddingLeft,
            val: Val::Rem(1.0),
        },
    ],
    mask: PropMask::of(PADDING),
};

/// `hover:px-8`: the x-edges only, `2rem`, under a hover probe.
const HOVER_PX8: Layer = Layer {
    cond: Cond::Hover,
    decls: &[
        Decl {
            prop: Prop::PaddingRight,
            val: Val::Rem(2.0),
        },
        Decl {
            prop: Prop::PaddingLeft,
            val: Val::Rem(2.0),
        },
    ],
    mask: PropMask::of(PADDING_X),
};

static LAYERS: [Layer; 2] = [BASE, HOVER_PX8];

fn main() {
    // The registry rows for `p` (all four edges) and `px` (x-edges only):
    let p4 = PropMask::of(PADDING);
    let px8 = PropMask::of(PADDING_X);
    println!("p-4   sets {:?}", names(p4));
    println!("px-8  sets {:?}", names(px8));

    // Overlay merge: the union IS the merged longhand set.
    println!("\np-4 ⊕ px-8  → {:?}", names(p4.union(px8)));
    println!("px-8 evicts from p-4: {:?}", names(p4.difference(px8)));

    // Conditional layer: hover:px-8 does not evict base p-4 — it wins
    // only where it fires (specificity ordering + condition identity).
    let program = Program::new(&LAYERS);
    println!("\nprogram layers:");
    for layer in program.layers {
        println!(
            "  {:>14?} (rank {}) → {:?}",
            layer.cond,
            layer.cond.specificity(),
            names(layer.mask)
        );
    }
    println!("\nbase conflicts: {:?}", program.base_conflicts().to_vec());
    println!("touched longhands: {}", program.touched().len());
}

/// Bit longhand names in cascade order — the audit view.
fn names(mask: PropMask) -> Vec<&'static str> {
    mask.to_vec().into_iter().map(Prop::name).collect()
}
