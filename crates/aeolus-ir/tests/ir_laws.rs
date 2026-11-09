//! The IR laws (architecture plan §3.3 "why this shape" pinned as tests).
//!
//! Three families:
//! 1. **Mask algebra** — the bitset laws that make `cn()` a bit
//!    operation: idempotence, commutativity, absorption, and the TRBL
//!    disjointness the per-edge model relies on.
//! 2. **Value laws** — the 12-byte budget, `Copy`, and the token/number
//!    classification contract.
//! 3. **Program laws** — specificity sorting is stable, layer masks are
//!    data-not-cache, and the base-conflict projection is exactly the
//!    bits a later layer re-sets.

use aeolus_ir::{Bp, Cond, Decl, Layer, Program, Prop, PropMask, Val};

// ---------------------------------------------------------------------------
// 1. Mask algebra
// ---------------------------------------------------------------------------

const PADDING: PropMask = PropMask::of(&[
    Prop::PaddingTop,
    Prop::PaddingRight,
    Prop::PaddingBottom,
    Prop::PaddingLeft,
]);
const PADDING_X: PropMask = PropMask::of(&[Prop::PaddingRight, Prop::PaddingLeft]);
const PADDING_T: PropMask = PropMask::of(&[Prop::PaddingTop]);
const MARGIN: PropMask = PropMask::of(&[
    Prop::MarginTop,
    Prop::MarginRight,
    Prop::MarginBottom,
    Prop::MarginLeft,
]);

#[test]
fn mask_of_round_trips_the_longhand_list() {
    // Every single-longhand mask round-trips through of/contains/len.
    for (i, &p) in Prop::ALL.iter().enumerate() {
        let m = PropMask::of(&[p]);
        assert!(m.contains(p), "of(All[{i}]) must set the bit");
        assert_eq!(m.len(), 1);
        assert_eq!(m.to_vec(), vec![p]);
    }
}

#[test]
fn mask_union_is_idempotent_commutative_absorbing() {
    // Idempotence: a ⊕ a == a.
    assert_eq!(PADDING.union(PADDING), PADDING);
    // Commutativity: a ⊕ b == b ⊕ a.
    assert_eq!(PADDING_X.union(PADDING_T), PADDING_T.union(PADDING_X));
    // Absorption: a ⊕ (a ⊓ b) == a.
    assert_eq!(PADDING.union(PADDING.intersect(PADDING_X)), PADDING);
    // The plan's flagship example: p-4 ⊕ px-8 covers every padding edge.
    assert_eq!(PADDING.union(PADDING_X), PADDING);
    // And stays disjoint from margin (no accidental cross-family bits).
    assert_eq!(PADDING.union(MARGIN).len(), PADDING.len() + MARGIN.len());
}

#[test]
fn mask_difference_and_dominance_model_eviction() {
    // px-8 evicts exactly the x-edges of p-4: the y-edges survive.
    assert_eq!(
        PropMask::of(&[Prop::PaddingTop, Prop::PaddingBottom]),
        PADDING.difference(PADDING_X)
    );
    // Dominance: p-4 covers px-8, never the other way round.
    assert!(PADDING.covers(PADDING_X));
    assert!(!PADDING_X.covers(PADDING));
    // Empty is the identity of union and the zero of intersect.
    assert_eq!(PropMask::EMPTY.union(PADDING), PADDING);
    assert!(PropMask::EMPTY.intersect(PADDING).is_empty());
}

#[test]
fn mask_for_each_walks_discriminant_order() {
    // The walk must yield cascade order regardless of insertion order.
    let m = PropMask::of(&[Prop::PaddingLeft, Prop::Display, Prop::ZIndex]);
    let walked: Vec<Prop> = m.to_vec();
    let mut sorted = walked.clone();
    sorted.sort_by_key(|p| *p as u16);
    assert_eq!(walked, sorted, "for_each must yield discriminant order");
    // And match the ALL slice's relative order.
    let ranks = |p: Prop| Prop::ALL.iter().position(|&q| q == p).expect("in ALL");
    assert!(walked.windows(2).all(|w| ranks(w[0]) < ranks(w[1])));
}

#[test]
fn trbl_edges_are_pairwise_disjoint_masks() {
    // The per-edge model (v0.8 §1): the four edge masks share no bits.
    let t = PropMask::of(&[Prop::PaddingTop]);
    let r = PropMask::of(&[Prop::PaddingRight]);
    let b = PropMask::of(&[Prop::PaddingBottom]);
    let l = PropMask::of(&[Prop::PaddingLeft]);
    assert!(t.intersect(r).is_empty());
    assert!(t.intersect(b).is_empty());
    assert!(t.intersect(l).is_empty());
    assert!(r.intersect(b).is_empty());
    assert!(r.intersect(l).is_empty());
    assert!(b.intersect(l).is_empty());
    // And their union is exactly the shorthand's fan-out.
    assert_eq!(t.union(r).union(b).union(l), PADDING);
}

#[test]
fn prop_mask_fits_u128_and_props_are_unique() {
    // The bitset budget: the longhand list must stay within u128.
    assert!(Prop::ALL.len() <= 128, "Prop grew past the PropMask budget");
    // And the ALL table must be duplicate-free with dense discriminants.
    for (i, &p) in Prop::ALL.iter().enumerate() {
        assert_eq!(p as u16, i as u16, "Prop::ALL[{i}] discriminant drifted");
        assert_eq!(Prop::ALL.iter().filter(|&&q| q == p).count(), 1);
    }
}

// ---------------------------------------------------------------------------
// 2. Value laws
// ---------------------------------------------------------------------------

#[test]
fn val_numeric_view_classifies_units() {
    assert_eq!(Val::Num(10.0).as_num(), Some(10.0));
    assert_eq!(Val::Px(16.0).as_num(), Some(16.0));
    assert_eq!(Val::Rem(1.0).as_num(), Some(1.0));
    assert_eq!(Val::Pct(50.0).as_num(), Some(50.0));
    assert_eq!(Val::Kw(3).as_num(), None);
    assert_eq!(Val::Rgba([0, 0, 0, 255]).as_num(), None);
}

#[test]
fn val_token_classification_drives_the_zero_alloc_contract() {
    // Runtime-tier values are exactly the Tok/ModePair arms.
    assert!(Val::Tok { id: 1, alpha: 1.0 }.is_token());
    assert!(Val::ModePair(2).is_token());
    assert!(!Val::Rgba([0, 0, 0, 255]).is_token());
    assert!(!Val::Rem(1.0).is_token());
    assert!(!Val::Kw(0).is_token());
}

// ---------------------------------------------------------------------------
// 3. Program laws
// ---------------------------------------------------------------------------

const P4: Layer = Layer {
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
    mask: PropMask::of(&[
        Prop::PaddingTop,
        Prop::PaddingRight,
        Prop::PaddingBottom,
        Prop::PaddingLeft,
    ]),
};

const PX8: Layer = Layer {
    cond: Cond::Base,
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
    mask: PropMask::of(&[Prop::PaddingRight, Prop::PaddingLeft]),
};

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
    mask: PropMask::of(&[Prop::PaddingRight, Prop::PaddingLeft]),
};

/// `mt-4`: a single-edge margin layer (widens the touched set).
const MT4: Layer = Layer {
    cond: Cond::Base,
    decls: &[Decl {
        prop: Prop::MarginTop,
        val: Val::Rem(1.0),
    }],
    mask: PropMask::of(&[Prop::MarginTop]),
};

static HOVER_FIRST: [Layer; 2] = [HOVER_PX8, P4];
static BASE_PAIR: [Layer; 2] = [PX8, P4];
static BASE_WITH_MARGIN: [Layer; 2] = [P4, MT4];

#[test]
fn layer_new_derives_the_mask() {
    // The dynamic-form constructor must agree with the static literals.
    let derived = Layer::new(
        Cond::Base,
        &[
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
    );
    assert_eq!(derived, P4);
    assert_eq!(derived.props(), P4.props());
}

#[test]
fn program_sort_is_stable_and_specificity_ordered() {
    // Deliberately unsorted: hover before base.
    let unsorted = Program::new(&HOVER_FIRST);
    assert!(!unsorted.is_sorted());
    let fixed: &'static [Layer] = Box::leak(unsorted.sorted().into_boxed_slice());
    let sorted = Program::new(fixed);
    assert_eq!(sorted.layers[0].cond, Cond::Base, "base sorts to layer 0");
    assert_eq!(sorted.layers[1].cond, Cond::Hover);
    assert!(sorted.is_sorted());
}

#[test]
fn program_sort_keeps_source_order_within_a_specificity_rank() {
    // Two base layers keep their source order (stable sort) — the cascade
    // contract for equal-specificity layers.
    let p = Program::new(&BASE_PAIR);
    let fixed = p.sorted();
    assert_eq!(fixed[0], PX8, "first base layer stays first");
    assert_eq!(fixed[1], P4);
}

#[test]
fn program_touched_is_the_mask_union() {
    let p = Program::new(&[P4, HOVER_PX8]);
    assert_eq!(p.touched(), PADDING);
    // A margin layer widens the touched set.
    let p2 = Program::new(&BASE_WITH_MARGIN);
    assert_eq!(
        p2.touched(),
        PADDING.union(PropMask::of(&[Prop::MarginTop]))
    );
}

#[test]
fn program_base_conflicts_are_exactly_the_overlap_bits() {
    // px-8 (base) re-sets the x-edges p-4 (base) already set.
    let p = Program::new(&[P4, PX8]);
    assert_eq!(p.base_conflicts(), PADDING_X);
    // hover:px-8 does NOT conflict with base p-4 (different conditions —
    // the hover rule only wins where it fires).
    let p2 = Program::new(&[P4, HOVER_PX8]);
    assert!(p2.base_conflicts().is_empty());
    // No base layer → no conflicts.
    assert!(Program::new(&[]).base_conflicts().is_empty());
}

#[test]
fn condition_specificity_matches_the_v4_cascade_ranks() {
    // base < dark < probe states < parent-relative < media < container.
    let order = [
        Cond::Base,
        Cond::Dark,
        Cond::Hover,
        Cond::Disabled,
        Cond::Selection,
        Cond::GroupHover,
        Cond::PeerFocus,
        Cond::Bp(Bp::Md),
        Cond::Max(Bp::Sm),
        Cond::Container(Bp::Md),
        Cond::MaxContainer(Bp::Sm),
    ];
    for w in order.windows(2) {
        assert!(
            w[0].specificity() <= w[1].specificity(),
            "{:?} must not outrank {:?}",
            w[0],
            w[1]
        );
    }
    // Equal-rank families tie (stable sort keeps their source order).
    assert_eq!(Cond::Hover.specificity(), Cond::Focus.specificity());
    assert_eq!(
        Cond::Bp(Bp::Sm).specificity(),
        Cond::Bp(Bp::Xl2).specificity()
    );
    // The canonical names round-trip for the diagnostics surface.
    assert_eq!(Cond::Bp(Bp::Md).source_text(), Some("md"));
    assert_eq!(Cond::MaxContainer(Bp::Sm).source_text(), Some("@max-sm"));
    assert_eq!(Cond::Hover.source_text(), Some("hover"));
    assert_eq!(Cond::Base.source_text(), None);
}
