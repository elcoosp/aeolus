//! The named-container bounds seam at the GPUI walk (architecture plan
//! §3.5's `@container` row, the `ContainerProbe` half): `@md/card:` /
//! `@max-lg/sidebar:` layers answer through the interp arm's
//! [`aeolus_rt::ContainerProbe`] seam when the host passes the named
//! containers' widths — the GPUI half of the `interp_containers` entry
//! point, the same AT/strictly-below boundary rules the viewport
//! breakpoints obey.
//!
//! The pins:
//! * a named min form fires at and above ITS container's width, the
//!   max form strictly below — the interp's `decide` semantics shared
//!   verbatim (the differential pin rides the tests);
//! * a container the source does not know is a CSS-quiet miss, never
//!   an exclusion;
//! * a sourceless walk keeps the named layers quiet (the interp
//!   reports `ContainerUnqueried`; this walk has no note channel);
//! * the UNNAMED forms keep answering from the Env's nearest-container
//!   slot — the seam is for the NAMED question only;
//! * the seam is the trait, not the built-in: a host `ContainerProbe`
//!   impl answers the same walk;
//! * named containers compose with the attribute bits and the
//!   structural facts in one walk (the interp's
//!   `interp_probes_containers_struct` pairing).

use aeolus_compile::{Interped, interp_containers};
use aeolus_gpui::prelude::*;
use gpui::{Display, rems};

#[test]
fn named_min_container_fires_at_and_above_its_width() {
    let program = tw!("@md/card:p-4");
    // `md` is 48rem: AT the boundary fires, above fires, below misses
    // — the min-width boundary the viewport breakpoints pin.
    let card = ContainerState::new(&[ContainerBound {
        name: "card",
        rem: 48.0,
    }]);
    let at = gpui::StyleRefinement::default().tw_containers(program, &card);
    assert_eq!(at.padding.top, Some(rems(1.0).into()));

    let wide = ContainerState::new(&[ContainerBound {
        name: "card",
        rem: 96.0,
    }]);
    let above = gpui::StyleRefinement::default().tw_containers(program, &wide);
    assert_eq!(above.padding.top, Some(rems(1.0).into()));

    let narrow = ContainerState::new(&[ContainerBound {
        name: "card",
        rem: 47.9,
    }]);
    let below = gpui::StyleRefinement::default().tw_containers(program, &narrow);
    assert_eq!(below.padding.top, None);
}

#[test]
fn named_max_container_fires_strictly_below_its_width() {
    let program = tw!("@max-lg/sidebar:hidden");
    // `lg` is 64rem: strictly below fires, AT the boundary misses —
    // the max-width boundary rule.
    let narrow = ContainerState::new(&[ContainerBound {
        name: "sidebar",
        rem: 63.9,
    }]);
    let below = gpui::StyleRefinement::default().tw_containers(program, &narrow);
    assert_eq!(below.display, Some(Display::None));

    let at = ContainerState::new(&[ContainerBound {
        name: "sidebar",
        rem: 64.0,
    }]);
    let boundary = gpui::StyleRefinement::default().tw_containers(program, &at);
    assert_ne!(boundary.display, Some(Display::None));
}

#[test]
fn unknown_container_name_is_a_quiet_miss() {
    // The host knows no such container — the CSS selector that does
    // not match, quiet, never an exclusion.
    let program = tw!("@md/card:p-4");
    let other = ContainerState::new(&[ContainerBound {
        name: "aside",
        rem: 96.0,
    }]);
    let style = gpui::StyleRefinement::default().tw_containers(program, &other);
    assert_eq!(style.padding.top, None);
}

#[test]
fn sourceless_named_container_stays_quiet() {
    // The documented sourceless semantics: without a bounds source the
    // named layers skip (the interp reports `ContainerUnqueried`; this
    // walk has no note channel), and the plain `.tw` walk stays exactly
    // what it was before the seam.
    let program = tw!("@md/card:p-4");
    let style = gpui::StyleRefinement::default().tw(program);
    assert_eq!(style.padding.top, None);
}

#[test]
fn unnamed_container_still_answers_from_the_env() {
    // The seam is for the NAMED question; the nearest-container
    // question keeps its Env slot answer (unchanged behavior).
    let program = tw!("@md:p-4");
    let wide = Env {
        container_rem: Some(50.0),
        ..Env::default()
    };
    let fires = gpui::StyleRefinement::default().tw_env(program, wide);
    assert_eq!(fires.padding.top, Some(rems(1.0).into()));

    let sourceless = Env::default();
    let quiet = gpui::StyleRefinement::default().tw_env(program, sourceless);
    assert_eq!(quiet.padding.top, None);
}

#[test]
fn first_matching_name_answers() {
    // The built-in table answers with the FIRST row for a repeated
    // name (the `ContainerState::container` contract the compile-side
    // pins) — here the first row's width wins.
    let program = tw!("@md/card:p-4");
    let rows = [
        ContainerBound {
            name: "card",
            rem: 48.0,
        },
        ContainerBound {
            name: "card",
            rem: 0.0,
        },
    ];
    let card = ContainerState::new(&rows);
    let style = gpui::StyleRefinement::default().tw_containers(program, &card);
    assert_eq!(style.padding.top, Some(rems(1.0).into()));
}

#[test]
fn custom_probe_impl_answers_the_walk() {
    // The seam is the trait, not the built-in: a host probe with owned
    // state (a layout-phase measurement cache, a component-tree walk)
    // answers the same walk.
    struct Measured {
        widths: std::collections::HashMap<&'static str, f32>,
    }

    impl ContainerProbe for Measured {
        fn container(&self, name: &str) -> Option<f32> {
            self.widths.get(name).copied()
        }
    }

    let program = tw!("@md/card:p-4");
    let probe = Measured {
        widths: [("card", 52.0)].into_iter().collect(),
    };
    let style = gpui::StyleRefinement::default().tw_containers(program, &probe);
    assert_eq!(style.padding.top, Some(rems(1.0).into()));
}

#[test]
fn named_containers_compose_with_probes_and_struct_in_one_walk() {
    // Three seams, one walk: the attribute bits answer the data layer,
    // the container bounds answer the named layer, the structural
    // facts answer the positional layer — the interp's
    // `interp_probes_containers_struct` pairing.
    let program = tw!("data-[state=open]:mt-2 @md/card:p-4 first:w-2");
    let rows = [AttrBit {
        kind: AttrKind::Data,
        name: "state",
        value: "open",
    }];
    let attrs = AttrState::new(&rows);
    let bounds = [ContainerBound {
        name: "card",
        rem: 48.0,
    }];
    let card = ContainerState::new(&bounds);
    let facts = StructFacts::new(0, 3);
    let style = gpui::StyleRefinement::default()
        .tw_probes_containers_struct(program, &attrs, &card, &facts);
    assert_eq!(style.margin.top, Some(rems(0.5).into()));
    assert_eq!(style.padding.top, Some(rems(1.0).into()));
    assert!(style.size.width.is_some());
}

#[test]
fn dark_and_named_container_layers_compose_in_one_walk() {
    // The frame env and the container seam answer their OWN layers:
    // dark mode hides, the named container pads.
    let program = tw!("dark:hidden @md/card:p-4");
    let bounds = [ContainerBound {
        name: "card",
        rem: 48.0,
    }];
    let card = ContainerState::new(&bounds);
    let env = Env {
        dark: true,
        ..Env::default()
    };
    let style = gpui::StyleRefinement::default().tw_env_containers(program, env, &card);
    assert_eq!(style.display, Some(Display::None));
    assert_eq!(style.padding.top, Some(rems(1.0).into()));
}

#[test]
fn min_and_max_forms_compose_by_cascade_when_both_fire() {
    // At 48rem BOTH forms fire (at `md`, still below `lg`) — the same
    // longhand twice, so the LATER layer wins the cascade (the max
    // form's tighter value), exactly the CSS source-order rule.
    let program = tw!("@md/card:p-6 @max-lg/card:p-2");
    let bounds = [ContainerBound {
        name: "card",
        rem: 48.0,
    }];
    let card = ContainerState::new(&bounds);
    let style = gpui::StyleRefinement::default().tw_containers(program, &card);
    assert_eq!(style.padding.top, Some(rems(0.5).into()));
}

#[test]
fn walk_matches_the_interps_containers_arm_differentially() {
    // The differential pin: for the same program + env + bounds, the
    // GPUI walk's verdict is the interp arm's — the fired declaration
    // at a qualifying width, no declaration below it.
    let program = tw!("@md/card:hidden");
    let env = Env::default();

    let bounds = [ContainerBound {
        name: "card",
        rem: 48.0,
    }];
    let card = ContainerState::new(&bounds);
    let interped: Interped = interp_containers(&program.program(), &env, &card)
        .expect("the interp answers the same question");
    assert_eq!(interped.decls.len(), 1);
    let style = gpui::StyleRefinement::default().tw_containers(program, &card);
    assert_eq!(style.display, Some(Display::None));

    let narrow = [ContainerBound {
        name: "card",
        rem: 40.0,
    }];
    let quiet = ContainerState::new(&narrow);
    let missed = interp_containers(&program.program(), &env, &quiet)
        .expect("the interp answers the same question");
    assert!(missed.decls.is_empty());
    let style = gpui::StyleRefinement::default().tw_containers(program, &quiet);
    assert_eq!(style.display, None);
}
