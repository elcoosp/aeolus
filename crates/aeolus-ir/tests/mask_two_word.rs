//! The mask's two-word growth contract (the filter layer's seam): the
//! longhand list outgrew one `u128` word when the CSS filter family
//! joined the IR, so [`PropMask`] carries TWO words (the plan §3.3
//! sketch's `u128×k` shape). The algebra is word-blind — these pins
//! walk the boundary a one-word implementation cannot cross.
//!
//! Written FIRST (the RED spec): the suite fails until the mask grows
//! past 128 bits and the filter longhands join the cascade tail.

use aeolus_ir::{Prop, PropMask};

#[test]
fn the_longhand_list_crosses_the_one_word_boundary() {
    // The filter layer's ten longhands push the list past 128: the
    // one-word budget's own assertion (`prop_mask_fits_u128`) is the
    // thing being retired — the two-word mask is the migration.
    assert!(
        Prop::ALL.len() > 128,
        "the list has {} longhands; the two-word seam is only wired when the list overflows u128",
        Prop::ALL.len()
    );
}

#[test]
fn a_bit_above_128_round_trips_through_the_mask() {
    // The first filter longhand sits at discriminant 121 — the SECOND
    // word's bit 0 under the `u128×k` layout. `insert`/`contains` must
    // route it through the high word.
    let p = Prop::ALL[121];
    let mask = PropMask::EMPTY.insert(p);
    assert!(mask.contains(p), "the high word's first bit must set/read");
    assert!(!mask.contains(Prop::ALL[0]));
}

#[test]
fn the_algebra_is_word_blind() {
    // One low prop, one high prop: union/intersect/difference/covers
    // must behave exactly as they do inside one word.
    let low = Prop::Display;
    let high = Prop::ALL[121];
    let a = PropMask::EMPTY.insert(low);
    let b = PropMask::EMPTY.insert(high);

    let both = a.union(b);
    assert!(
        both.contains(low) && both.contains(high),
        "union spans words"
    );
    assert_eq!(both.len(), 2, "len counts both words");

    assert!(both.intersect(a).contains(low));
    assert!(!both.intersect(a).contains(high));

    let diff = both.difference(a);
    assert!(
        !diff.contains(low) && diff.contains(high),
        "difference spans words"
    );

    assert!(b.covers(PropMask::EMPTY.insert(high)));
    assert!(!a.covers(b), "low does not cover high");
}

#[test]
fn for_each_walks_low_word_first_then_high() {
    // Cascade order is discriminant order: the walk must yield every
    // set longhand low-word-first, each exactly once.
    let high = Prop::ALL[121];
    let mask = PropMask::EMPTY
        .insert(Prop::Display)
        .insert(Prop::PaddingTop)
        .insert(high);
    let mut seen = Vec::new();
    mask.for_each(|p| seen.push(p));
    assert_eq!(seen, vec![Prop::Display, Prop::PaddingTop, high]);
}

#[test]
fn of_builds_from_the_row_target_slices_across_words() {
    // The registry's fan-out slices are the mask's own input; a slice
    // mixing low and high discriminants builds the two words.
    let props = &[Prop::Display, Prop::ALL[121], Prop::ALL[130]];
    let mask = PropMask::of(props);
    for &p in props {
        assert!(mask.contains(p), "{p:?} must ride the mask");
    }
    assert_eq!(mask.len(), props.len() as u32);
}
