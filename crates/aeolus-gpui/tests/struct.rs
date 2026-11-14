//! The structural host-position seam (architecture plan §3.5's probe
//! row, the position half): the structural layers (`first:`, `last:`,
//! `odd:`, `even:`, `only:`, `empty:`, `nth-N:`, `nth-last-N:`) answer
//! through the interp arm's [`StructProbe`] seam when the host passes
//! the element's DOM-position facts — the GPUI half of the
//! `interp_struct` entry point, the verdicts the interp's own
//! `structural_fires` renders (one semantics, three walks).
//!
//! The pins:
//! * the facts decide: the first-child facts fire `first:` (and `odd:`
//!   — index 0 IS `:nth-child(odd)`), the last-child facts fire
//!   `last:`; an unknown field quietly misses its test;
//! * a sourceless walk keeps the layers quiet (the interp's
//!   `StructuralUnqueried` note has no GPUI channel, so the CSS
//!   selector-miss skip stands);
//! * `StructFacts::UNKNOWN` is the honest"answered, knows nothing"
//!   value — quiet, not fired;
//! * the seam is the trait, not the built-in: a host `StructProbe`
//!   impl answers the same walk, and the probe+struct composition
//!   rides one walk (`tw_probes_struct`).

use aeolus_gpui::prelude::*;
use gpui::rems;

#[test]
fn the_first_child_facts_fire_the_first_layer() {
    let program = tw!("first:p-4");
    let facts = StructFacts::new(0, 3);
    let style = gpui::StyleRefinement::default().tw_struct(program, &facts);
    assert_eq!(style.padding.top, Some(rems(1.0).into()));
}

#[test]
fn a_middle_child_quietly_skips_the_first_layer() {
    let program = tw!("first:p-4");
    let facts = StructFacts::new(1, 3);
    let style = gpui::StyleRefinement::default().tw_struct(program, &facts);
    assert_eq!(style.padding.top, None);
}

#[test]
fn the_odd_layer_follows_the_one_based_css_parity() {
    let program = tw!("odd:p-4");
    // Index 0: the FIRST child — `:nth-child(odd)` fires.
    let style = gpui::StyleRefinement::default().tw_struct(program, &StructFacts::new(0, 3));
    assert_eq!(style.padding.top, Some(rems(1.0).into()));
    // Index 1: the second child — even, quiet.
    let style = gpui::StyleRefinement::default().tw_struct(program, &StructFacts::new(1, 3));
    assert_eq!(style.padding.top, None);
}

#[test]
fn the_last_child_facts_fire_the_last_layer() {
    let program = tw!("last:p-4");
    let style = gpui::StyleRefinement::default().tw_struct(program, &StructFacts::new(2, 3));
    assert_eq!(style.padding.top, Some(rems(1.0).into()));
    let style = gpui::StyleRefinement::default().tw_struct(program, &StructFacts::new(0, 3));
    assert_eq!(style.padding.top, None);
}

#[test]
fn the_ordinal_is_one_based() {
    let program = tw!("nth-3:p-4");
    // Index 2 of 3: the third child.
    let style = gpui::StyleRefinement::default().tw_struct(program, &StructFacts::new(2, 3));
    assert_eq!(style.padding.top, Some(rems(1.0).into()));
    let style = gpui::StyleRefinement::default().tw_struct(program, &StructFacts::new(0, 3));
    assert_eq!(style.padding.top, None);
}

#[test]
fn unknown_facts_are_a_quiet_miss_not_a_fire() {
    let program = tw!("first:p-4 empty:p-4");
    let style = gpui::StyleRefinement::default().tw_struct(program, &StructFacts::UNKNOWN);
    assert_eq!(
        style.padding.top, None,
        "UNKNOWN facts match no positional test"
    );
}

#[test]
fn a_sourceless_walk_keeps_the_structural_layers_quiet() {
    let program = tw!("first:p-4");
    let style = gpui::StyleRefinement::default().tw_env(program, Env::default());
    assert_eq!(style.padding.top, None);
}

/// The seam is the trait: a host probe (a layout bridge's stand-in)
/// answers the same walk the built-in facts value does.
#[test]
fn a_host_struct_probe_impl_answers_the_same_walk() {
    struct ThirdOfFive;
    impl StructProbe for ThirdOfFive {
        fn structure(&self) -> StructFacts {
            StructFacts::new(2, 5)
        }
    }
    let program = tw!("nth-3:p-4");
    let style = gpui::StyleRefinement::default().tw_struct(program, &ThirdOfFive);
    assert_eq!(style.padding.top, Some(rems(1.0).into()));
}

/// The probe+struct composition rides ONE walk: the attribute bits and
/// the DOM-position facts answer their own layers in one call.
#[test]
fn the_seams_compose_in_one_walk() {
    let program = tw!("data-open:p-4 first:p-2");
    let bits = [AttrBit {
        kind: AttrKind::Data,
        name: "open",
        value: "",
    }];
    let state = AttrState::new(&bits);
    let facts = StructFacts::new(0, 3);
    let style = gpui::StyleRefinement::default().tw_probes_struct(program, &state, &facts);
    // Both layers fire (the bit answers, index 0 is first) — and the
    // LATER structural layer wins the shared padding longhand (the
    // cascade the specificity sort encodes), so every edge is `p-2`.
    assert_eq!(style.padding.top, Some(rems(0.5).into()));
    assert_eq!(style.padding.bottom, Some(rems(0.5).into()));
}
