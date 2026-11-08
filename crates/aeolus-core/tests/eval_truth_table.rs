//! The evaluator truth table (plan §5.2): for each `Atom` kind, a `Probe`
//! that flips it, asserting the overlay order (`dark:` beats `hover:`,
//! rules rank above plain layers per `Cond::rank`).

use aeolus_core::ast::{Media, State, Structural};
use aeolus_core::cond::Test;
use aeolus_core::eval::{Probe, Rel, in_window};
use aeolus_core::plan::{Opacity, Overflow, OverflowKind};
use aeolus_core::*;

/// A probe with every answer settable.
#[derive(Clone, Copy, Default)]
struct TableProbe {
    hover: bool,
    focus: bool,
    active: bool,
    dark: bool,
    print: bool,
    width: Option<f32>,
    container: Option<f32>,
    first_child: bool,
    data_open: bool,
    group_hover: bool,
    peer_hover: bool,
}

impl Probe for TableProbe {
    fn test(&self, t: &Test) -> bool {
        match t {
            Test::State(s) => match s {
                State::Hover => self.hover,
                State::Focus => self.focus,
                State::Active => self.active,
                _ => false,
            },
            Test::Data(name, _) => name.as_ref() == "state" && self.data_open,
            Test::Aria(_, _) => false,
            Test::Structural(s) => matches!(s, Structural::First) && self.first_child,
            Test::Tag(_) => false,
            // Pseudo-element renderings (`selection:`/`placeholder:`/`file:`)
            // never fire on a probe that does not model the rendering — the
            // default probe answer, same semantics as an unmatched state.
            Test::Pseudo(_) => false,
        }
    }

    fn related(&self, rel: Rel, _label: Option<&str>, t: &Test) -> bool {
        let flag = match rel {
            Rel::Group => self.group_hover,
            Rel::Peer => self.peer_hover,
        };
        flag && matches!(t, Test::State(State::Hover))
    }

    fn media(&self, m: Media) -> bool {
        match m {
            Media::Dark => self.dark,
            Media::Print => self.print,
            _ => false,
        }
    }

    fn viewport_width(&self) -> Option<f32> {
        self.width
    }

    fn container_width(&self, _name: Option<&str>) -> Option<f32> {
        self.container
    }
}

/// `dark:` beats `hover:` when both match (v4 variant order, plan §2.1).
#[test]
fn dark_beats_hover() {
    let theme = TailwindTheme::new();
    let r = Resolver::new(&theme)
        .resolve("hover:opacity-60 dark:opacity-100")
        .unwrap();
    let probe = TableProbe {
        hover: true,
        dark: true,
        ..Default::default()
    };
    let out = r.plan.resolve(&probe);
    assert_eq!(out.opacity, Some(Opacity(1.0)), "dark wins over hover");

    // Only hover: hover wins.
    let probe = TableProbe {
        hover: true,
        ..Default::default()
    };
    let out = r.plan.resolve(&probe);
    assert_eq!(out.opacity, Some(Opacity(0.6)));
}

/// Breakpoint rules (110) beat state layers (10).
#[test]
fn responsive_beats_state_layers() {
    let theme = TailwindTheme::new();
    let r = Resolver::new(&theme)
        .resolve("hover:opacity-60 md:opacity-100")
        .unwrap();
    let probe = TableProbe {
        hover: true,
        width: Some(900.0),
        ..Default::default()
    };
    let out = r.plan.resolve(&probe);
    assert_eq!(out.opacity, Some(Opacity(1.0)));
    // Below md, only hover applies.
    let probe = TableProbe {
        hover: true,
        width: Some(500.0),
        ..Default::default()
    };
    let out = r.plan.resolve(&probe);
    assert_eq!(out.opacity, Some(Opacity(0.6)));
}

/// Compound `dark:hover:` (a rule, rank 140) beats both plain layers.
#[test]
fn compound_dark_hover_wins() {
    let theme = TailwindTheme::new();
    let r = Resolver::new(&theme)
        .resolve("hover:opacity-60 dark:opacity-80 dark:hover:opacity-100")
        .unwrap();
    let probe = TableProbe {
        hover: true,
        dark: true,
        ..Default::default()
    };
    let out = r.plan.resolve(&probe);
    assert_eq!(out.opacity, Some(Opacity(1.0)));
    // dark only (no hover): the dark layer applies.
    let probe = TableProbe {
        dark: true,
        ..Default::default()
    };
    let out = r.plan.resolve(&probe);
    assert_eq!(out.opacity, Some(Opacity(0.8)));
}

/// Every atom kind flips its rule.
#[test]
fn atom_truth_table() {
    let theme = TailwindTheme::new();
    let resolver = Resolver::new(&theme);

    // data-[state=open]
    let r = resolver.resolve("data-[state=open]:bg-black").unwrap();
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    let hit = TableProbe {
        data_open: true,
        ..Default::default()
    };
    assert!(r.plan.resolve(&hit).background.is_some());
    let miss = TableProbe::default();
    assert!(r.plan.resolve(&miss).background.is_none());

    // first:
    let r = resolver.resolve("first:bg-black").unwrap();
    let hit = TableProbe {
        first_child: true,
        ..Default::default()
    };
    assert!(r.plan.resolve(&hit).background.is_some());
    assert!(r.plan.resolve(&TableProbe::default()).background.is_none());

    // group-hover:
    let r = resolver.resolve("group-hover:bg-black").unwrap();
    let hit = TableProbe {
        group_hover: true,
        ..Default::default()
    };
    assert!(r.plan.resolve(&hit).background.is_some());
    assert!(r.plan.resolve(&TableProbe::default()).background.is_none());

    // peer-hover:
    let r = resolver.resolve("peer-hover:bg-black").unwrap();
    let hit = TableProbe {
        peer_hover: true,
        ..Default::default()
    };
    assert!(r.plan.resolve(&hit).background.is_some());

    // not-hover:
    let r = resolver.resolve("not-hover:bg-black").unwrap();
    assert!(r.plan.resolve(&TableProbe::default()).background.is_some());
    let hovered = TableProbe {
        hover: true,
        ..Default::default()
    };
    assert!(r.plan.resolve(&hovered).background.is_none());

    // has-checked: (descendant state — default probe says no)
    let r = resolver.resolve("has-checked:bg-black").unwrap();
    assert!(r.plan.resolve(&TableProbe::default()).background.is_none());

    // @md container query: --container-md = 28rem = 448px (NOT the 768px
    // viewport breakpoint — the v0.1 conflation the plan fixes in §3.3).
    let r = resolver.resolve("@md:bg-black").unwrap();
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    let hit = TableProbe {
        container: Some(448.0),
        ..Default::default()
    };
    assert!(r.plan.resolve(&hit).background.is_some());
    let miss = TableProbe {
        container: Some(447.0),
        ..Default::default()
    };
    assert!(r.plan.resolve(&miss).background.is_none());
}

/// `in_window` is `[min, max)` and never matches an unknown width.
#[test]
fn window_semantics() {
    assert!(!in_window(None, Some(0.0), None));
    assert!(in_window(Some(640.0), Some(640.0), None));
    assert!(!in_window(Some(639.9), Some(640.0), None));
    assert!(!in_window(Some(768.0), Some(640.0), Some(768.0)));
    assert!(in_window(Some(767.9), Some(640.0), Some(768.0)));
}

/// The legacy group bucket labels split back into (label, state).
#[test]
fn split_group_label_roundtrip() {
    assert_eq!(split_group_label("hover"), Some((None, State::Hover)));
    assert_eq!(
        split_group_label("menufocus"),
        Some((Some("menu"), State::Focus))
    );
    assert_eq!(split_group_label("notastate"), None);
}

/// Plans are evaluator-shape, not evaluator-resolved: an empty probe never
/// activates anything beyond the base.
#[test]
fn base_only_with_default_probe() {
    let theme = TailwindTheme::new();
    let r = Resolver::new(&theme)
        .resolve("overflow-hidden bg-black")
        .unwrap();
    let out = r.plan.resolve(&TableProbe::default());
    assert!(out.background.is_some());
    assert_eq!(
        out.extras.overflow,
        Some(Overflow {
            x: Some(OverflowKind::Hidden),
            y: Some(OverflowKind::Hidden)
        })
    );
}
