//! The attribute-bits host state seam (architecture plan §3.5's
//! "typed state bits passed WITH the element" row, P4's last named
//! piece): `data-*:`/`aria-*:` layers answer through the interp arm's
//! [`aeolus_rt::AttrProbe`] seam when the host passes the element's
//! attribute bits — the GPUI half of the `interp_probes` entry point.
//!
//! The pins:
//! * the value spelling (`data-[state=open]:`) fires on equality, the
//!   presence spelling (`data-open:`) on any value — the corpus's own
//!   CSS semantics (`[data-state="open"]`, `[data-open]`);
//! * a sourceless walk keeps the layers quiet (the interp's
//!   `AttributeUnqueried` note has no GPUI channel, so the CSS
//!   selector-miss skip stands);
//! * the descendant shapes (`has-*`/`group-has-*`/`peer-has-*`) are NOT
//!   attribute bits — they stay the diagnosed boundary (quiet here),
//!   with or without a source, exactly like the interp;
//! * the seam is the trait, not the built-in: a host `AttrProbe` impl
//!   answers the same walk.

use aeolus_gpui::prelude::*;
use gpui::{Display, rems};

fn bits() -> [AttrBit<'static>; 2] {
    [
        AttrBit {
            kind: AttrKind::Data,
            name: "state",
            value: "open",
        },
        AttrBit {
            kind: AttrKind::Aria,
            name: "busy",
            value: "true",
        },
    ]
}

#[test]
fn data_value_layer_fires_on_a_matching_bit() {
    let program = tw!("data-[state=open]:p-4");
    let rows = bits();
    let state = AttrState::new(&rows);
    let style = gpui::StyleRefinement::default().tw_probes(program, &state);
    assert_eq!(style.padding.top, Some(rems(1.0).into()));
}

#[test]
fn data_value_layer_stays_quiet_on_a_different_value() {
    let program = tw!("data-[state=open]:p-4");
    let state = AttrState::new(&[AttrBit {
        kind: AttrKind::Data,
        name: "state",
        value: "closed",
    }]);
    let style = gpui::StyleRefinement::default().tw_probes(program, &state);
    assert_eq!(style.padding.top, None);
}

#[test]
fn data_presence_layer_fires_on_any_value() {
    let program = tw!("data-open:p-4");
    // The presence test answers on ANY value — the corpus's `[data-open]`.
    let state = AttrState::new(&[AttrBit {
        kind: AttrKind::Data,
        name: "open",
        value: "whatever",
    }]);
    let style = gpui::StyleRefinement::default().tw_probes(program, &state);
    assert_eq!(style.padding.top, Some(rems(1.0).into()));
}

#[test]
fn data_presence_layer_stays_quiet_without_the_bit() {
    let program = tw!("data-open:p-4");
    let state = AttrState::new(&[AttrBit {
        kind: AttrKind::Data,
        name: "other",
        value: "whatever",
    }]);
    let style = gpui::StyleRefinement::default().tw_probes(program, &state);
    assert_eq!(style.padding.top, None);
}

#[test]
fn aria_busy_layer_fires_on_the_true_bit() {
    // Bare `aria-busy:` folds to the "true" value (the corpus's
    // `[aria-busy="true"]`), so the "true" bit answers it.
    let program = tw!("aria-busy:hidden");
    let rows = bits();
    let state = AttrState::new(&rows);
    let style = gpui::StyleRefinement::default().tw_probes(program, &state);
    assert_eq!(style.display, Some(Display::None));
}

#[test]
fn aria_bracket_form_compares_the_verbatim_value() {
    let program = tw!("aria-[sort=asc]:p-4");
    let asc = AttrState::new(&[AttrBit {
        kind: AttrKind::Aria,
        name: "sort",
        value: "asc",
    }]);
    let style = gpui::StyleRefinement::default().tw_probes(program, &asc);
    assert_eq!(style.padding.top, Some(rems(1.0).into()));

    let desc = AttrState::new(&[AttrBit {
        kind: AttrKind::Aria,
        name: "sort",
        value: "desc",
    }]);
    let style = gpui::StyleRefinement::default().tw_probes(program, &desc);
    assert_eq!(style.padding.top, None);
}

#[test]
fn sourceless_walk_keeps_attribute_layers_quiet() {
    // The documented sourceless semantics: without a probe source the
    // element-own layers skip (CSS selector-miss), and the plain `.tw`
    // walk stays exactly what it was before the seam.
    let program = tw!("data-open:p-4");
    let style = gpui::StyleRefinement::default().tw(program);
    assert_eq!(style.padding.top, None);
}

#[test]
fn has_layers_stay_quiet_even_with_bits() {
    // The descendant shapes are not element-own attribute bits — the
    // plan §3.5 row keeps them the diagnosed boundary, with or without
    // a source (the interp reports them as notes; the GPUI walk skips).
    let program = tw!("has-focus:p-4 group-has-data-[slot=card-action]:mt-2");
    let rows = bits();
    let state = AttrState::new(&rows);
    let style = gpui::StyleRefinement::default().tw_probes(program, &state);
    assert_eq!(style.padding.top, None);
    assert_eq!(style.margin.top, None);
}

#[test]
fn dark_and_attribute_layers_compose_in_one_walk() {
    let program = tw!("dark:p-4 data-[state=open]:mt-2");
    let rows = bits();
    let state = AttrState::new(&rows);
    let env = Env {
        dark: true,
        ..Env::default()
    };
    let style = gpui::StyleRefinement::default().tw_env_probes(program, env, &state);
    assert_eq!(style.padding.top, Some(rems(1.0).into()));
    assert_eq!(style.margin.top, Some(rems(0.5).into()));
}

#[test]
fn custom_probe_impl_answers_the_walk() {
    // The seam is the trait, not the built-in: a host probe with owned
    // state answers the same walk (the "host implements the ONE
    // question itself" story the AttrProbe docs name).
    struct OwnedBit {
        name: &'static str,
        value: String,
    }

    impl AttrProbe for OwnedBit {
        fn probe(&self, kind: AttrKind, name: &str) -> Option<&str> {
            if kind == AttrKind::Data && name == self.name {
                Some(&self.value)
            } else {
                None
            }
        }
    }

    let program = tw!("data-[state=open]:p-4");
    let probe = OwnedBit {
        name: "state",
        value: "open".to_string(),
    };
    let style = gpui::StyleRefinement::default().tw_probes(program, &probe);
    assert_eq!(style.padding.top, Some(rems(1.0).into()));
}
