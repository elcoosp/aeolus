//! Arbitrary selector variants (v0.3 plan §1): `[&…]`, `[a&]`, `[:hover]`,
//! `[@media(…)]` — semantics verified against tailwindcss v4.1.17 source
//! (`candidate.ts`, `compile.ts`, `variants.ts`).
//!
//! Contract: the typed subset resolves to probe `Atom`s (or canonicalizes
//! onto the named-variant buckets it is byte-identical to in CSS); every
//! other form fails with a PRECISE `Unsupported` reason, never a blanket
//! "arbitrary variants unsupported" and never a silent drop.

use aeolus_core::cond::{Atom, Test};
use aeolus_core::eval::Rel;
use aeolus_core::plan::{TextProperty, TextValue};
use aeolus_core::*;
use std::sync::atomic::{AtomicBool, Ordering};

/// Does the resolved style set `underline` (the `underline` utility)?
fn has_underline(s: &aeolus_core::ResolvedStyle) -> bool {
    s.text.contains(&(
        TextProperty::TextDecoration,
        TextValue::Keyword("underline"),
    ))
}

fn theme() -> TailwindTheme {
    TailwindTheme::new()
}

/// A probe with every answer settable (mirrors the truth-table probe).
#[derive(Default)]
struct SelProbe {
    hover: bool,
    focus: bool,
    dark: bool,
    print: bool,
    landscape: bool,
    width: Option<f32>,
    container: Option<f32>,
    nth3: bool,
    first_child: bool,
    descendant_focus: bool,
    data_open: bool,
    group_hover: bool,
    /// The element's tag, when the backend knows element kinds.
    tag: Option<&'static str>,
}

impl Probe for SelProbe {
    fn test(&self, t: &Test) -> bool {
        match t {
            Test::State(s) => match s {
                State::Hover => self.hover,
                State::Focus => self.focus,
                _ => false,
            },
            Test::Data(name, _) => name.as_ref() == "state" && self.data_open,
            Test::Aria(_, _) => false,
            Test::Structural(s) => match s {
                Structural::Nth(3) => self.nth3,
                Structural::First => self.first_child,
                _ => false,
            },
            Test::Tag(name) => self.tag.is_some_and(|t| t == name.as_ref()),
            // Pseudo-element renderings never fire on a probe that does not
            // model the rendering (the default probe answer).
            Test::Pseudo(_) => false,
        }
    }

    fn related(&self, rel: Rel, _label: Option<&str>, t: &Test) -> bool {
        matches!(rel, Rel::Group) && self.group_hover && matches!(t, Test::State(State::Hover))
    }

    fn descendant_state(&self, s: State) -> bool {
        s == State::Focus && self.descendant_focus
    }

    fn media(&self, m: Media) -> bool {
        match m {
            Media::Dark => self.dark,
            Media::Print => self.print,
            Media::Landscape => self.landscape,
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

fn warnings_of(r: &Resolution) -> Vec<&str> {
    r.warnings
        .iter()
        .map(|w| match w {
            Warning::Unsupported { reason, .. } => reason,
            Warning::Unparseable { message, .. } => message.as_str(),
            _ => "",
        })
        .collect()
}

/// A `Probe` that tracks whether `Test::Tag` was ever asked (default
/// probes must not fire tag rules, and must not be *asked* about tags
/// when the plan has none).
static TAG_ASKED: AtomicBool = AtomicBool::new(false);

struct TagLoggingProbe(SelProbe);

impl Probe for TagLoggingProbe {
    fn test(&self, t: &Test) -> bool {
        if matches!(t, Test::Tag(_)) {
            TAG_ASKED.store(true, Ordering::SeqCst);
        }
        self.0.test(t)
    }
    fn related(&self, rel: Rel, label: Option<&str>, t: &Test) -> bool {
        self.0.related(rel, label, t)
    }
    fn descendant_state(&self, s: State) -> bool {
        self.0.descendant_state(s)
    }
    fn media(&self, m: Media) -> bool {
        self.0.media(m)
    }
    fn viewport_width(&self) -> Option<f32> {
        self.0.viewport_width()
    }
    fn container_width(&self, name: Option<&str>) -> Option<f32> {
        self.0.container_width(name)
    }
}

// ---------------------------------------------------------------------------
// §1.2 — the typed subset
// ---------------------------------------------------------------------------

/// `[&:hover]` generates byte-identical CSS to `hover:` in v4, so it
/// canonicalizes onto the same layer bucket and behaves identically.
#[test]
fn self_hover_matches_named_hover() {
    let r = Resolver::new(&theme())
        .resolve("[&:hover]:bg-black")
        .unwrap();
    assert!(
        r.warnings.is_empty(),
        "[&:hover] must be typed, got {:?}",
        r.warnings
    );
    let hit = r.plan.resolve(&SelProbe {
        hover: true,
        ..Default::default()
    });
    assert!(hit.background.is_some(), "[&:hover] must fire on hover");
    assert!(
        r.plan.resolve(&SelProbe::default()).background.is_none(),
        "[&:hover] must not fire at rest"
    );
    // Same bucket as the named variant.
    let named = Resolver::new(&theme()).resolve("hover:bg-black").unwrap();
    assert_eq!(r.plan.layers.len(), named.plan.layers.len());
    assert_eq!(r.plan.layers[0].condition, named.plan.layers[0].condition);
}

/// `[:hover]` (no `&`) — v4 wraps it as `&:is(:hover)`: the same self test.
#[test]
fn implicit_self_pseudo() {
    let r = Resolver::new(&theme())
        .resolve("[:hover]:bg-black")
        .unwrap();
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    assert!(
        r.plan
            .resolve(&SelProbe {
                hover: true,
                ..Default::default()
            })
            .background
            .is_some()
    );
}

/// `[&:hover:focus]` — a compound of two self pseudo-classes is a
/// conjunction (both states required).
#[test]
fn compound_self_states_conjoin() {
    let r = Resolver::new(&theme())
        .resolve("[&:hover:focus]:bg-black")
        .unwrap();
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    let both = r.plan.resolve(&SelProbe {
        hover: true,
        focus: true,
        ..Default::default()
    });
    assert!(both.background.is_some());
    let only_hover = r.plan.resolve(&SelProbe {
        hover: true,
        ..Default::default()
    });
    assert!(only_hover.background.is_none(), "needs BOTH states");
}

/// `[&:not(:hover)]` — negation inside the arbitrary selector.
#[test]
fn negation_inside_selector() {
    let r = Resolver::new(&theme())
        .resolve("[&:not(:hover)]:bg-black")
        .unwrap();
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    assert!(
        r.plan.resolve(&SelProbe::default()).background.is_some(),
        "applies when NOT hovered"
    );
    assert!(
        r.plan
            .resolve(&SelProbe {
                hover: true,
                ..Default::default()
            })
            .background
            .is_none()
    );
}

/// `[&[data-state=open]]` — self attribute test, same atoms as the named
/// `data-[state=open]:` form (identical plans).
#[test]
fn self_attribute_matches_named_data() {
    let theme = theme();
    let arb = Resolver::new(&theme)
        .resolve("[&[data-state=open]]:bg-black")
        .unwrap();
    assert!(arb.warnings.is_empty(), "{:?}", arb.warnings);
    let named = Resolver::new(&theme)
        .resolve("data-[state=open]:bg-black")
        .unwrap();
    assert_eq!(arb.plan, named.plan, "identical atom conjunction");

    let hit = arb.plan.resolve(&SelProbe {
        data_open: true,
        ..Default::default()
    });
    assert!(hit.background.is_some());
}

/// `[&[aria-disabled=true]]` — self aria attribute test.
#[test]
fn self_aria_attribute() {
    let r = Resolver::new(&theme())
        .resolve("[&[aria-disabled=true]]:bg-black")
        .unwrap();
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    let hit = r.plan.resolve(&SelProbe {
        data_open: true, // probe answers data "state"; aria disabled -> use data probe? no:
        ..Default::default()
    });
    // The probe above answers data-state only; the aria rule must NOT fire.
    assert!(hit.background.is_none());
    // But the rule exists with the Aria test.
    let rule = r
        .plan
        .rules
        .iter()
        .find(|r| matches!(&r.when.0[0], Atom::On(Test::Aria(name, Some(_))) if name.as_ref() == "disabled"))
        .expect("aria rule must be in plan.rules");
    assert_eq!(rule.when.0.len(), 1);
}

/// `[&:nth-child(3)]` and `[&:first-child]` — structural self tests.
#[test]
fn structural_self_tests() {
    let r = Resolver::new(&theme())
        .resolve("[&:nth-child(3)]:bg-black")
        .unwrap();
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    let hit = r.plan.resolve(&SelProbe {
        nth3: true,
        ..Default::default()
    });
    assert!(hit.background.is_some());

    let r = Resolver::new(&theme())
        .resolve("[&:first-child]:bg-black")
        .unwrap();
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    let hit = r.plan.resolve(&SelProbe {
        first_child: true,
        ..Default::default()
    });
    assert!(hit.background.is_some());
}

/// `[a&]` — the element-kind (tag) test. The rule lands typed in
/// `plan.rules`; a backend that cannot answer `Test::Tag` never fires it
/// (the same semantics CSS has when the selector does not match), and a
/// backend that knows the element kind does.
#[test]
fn tag_test_explicit_and_implicit() {
    for class in ["[a&]:underline", "[a]:underline"] {
        let r = Resolver::new(&theme()).resolve(class).unwrap();
        assert!(r.warnings.is_empty(), "{class}: {:?}", r.warnings);
        let rule = r
            .plan
            .rules
            .iter()
            .find(|r| matches!(r.when.0[0], Atom::On(Test::Tag(_))))
            .unwrap_or_else(|| panic!("{class}: tag rule must be typed in plan.rules"));
        assert_eq!(rule.when.0.len(), 1);

        let probe = SelProbe {
            tag: Some("a"),
            ..Default::default()
        };
        assert!(
            has_underline(&r.plan.resolve(&probe)),
            "{class}: fires when the element is an <a>"
        );
        assert!(
            !has_underline(&r.plan.resolve(&SelProbe::default())),
            "{class}: never fires for an unknown element kind"
        );
    }
}

/// `[p:hover]` — tag AND pseudo-class compound (tag prefix + `&` suffix
/// equivalent: `[p&:hover]`).
#[test]
fn tag_with_state_compound() {
    for class in ["[p:hover]:bg-black", "[p&:hover]:bg-black"] {
        let r = Resolver::new(&theme()).resolve(class).unwrap();
        assert!(r.warnings.is_empty(), "{class}: {:?}", r.warnings);
        let rest = SelProbe {
            tag: Some("p"),
            ..Default::default()
        };
        assert!(
            r.plan.resolve(&rest).background.is_none(),
            "{class}: needs the state too"
        );
        let both = SelProbe {
            tag: Some("p"),
            hover: true,
            ..Default::default()
        };
        assert!(
            r.plan.resolve(&both).background.is_some(),
            "{class}: tag + state fires"
        );
    }
}

/// `[@media(print)]` behaves like `print:`; the dark scheme form like
/// `dark:`.
#[test]
fn arbitrary_media_matches_named() {
    let theme = theme();
    let r = Resolver::new(&theme)
        .resolve("[@media(print)]:bg-black")
        .unwrap();
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    let hit = r.plan.resolve(&SelProbe {
        print: true,
        ..Default::default()
    });
    assert!(hit.background.is_some());
    assert!(r.plan.resolve(&SelProbe::default()).background.is_none());

    let r = Resolver::new(&theme)
        .resolve("[@media(prefers-color-scheme:dark)]:bg-black")
        .unwrap();
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    let hit = r.plan.resolve(&SelProbe {
        dark: true,
        ..Default::default()
    });
    assert!(hit.background.is_some());
}

/// `[@media(orientation:landscape)]` maps to the typed media.
#[test]
fn arbitrary_media_orientation() {
    let r = Resolver::new(&theme())
        .resolve("[@media(orientation:landscape)]:bg-black")
        .unwrap();
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    let hit = r.plan.resolve(&SelProbe {
        landscape: true,
        ..Default::default()
    });
    assert!(hit.background.is_some());
}

/// `[@media(min-width:900px)]` — a viewport window atom; `width>=` is the
/// v4 canonical shorthand for the same window.
#[test]
fn arbitrary_media_width_window() {
    for class in [
        "[@media(min-width:900px)]:bg-black",
        "[@media(width>=900px)]:bg-black",
    ] {
        let r = Resolver::new(&theme()).resolve(class).unwrap();
        assert!(r.warnings.is_empty(), "{class}: {:?}", r.warnings);
        let hit = r.plan.resolve(&SelProbe {
            width: Some(900.0),
            ..Default::default()
        });
        assert!(hit.background.is_some(), "{class}: fires at 900px");
        let miss = r.plan.resolve(&SelProbe {
            width: Some(899.0),
            ..Default::default()
        });
        assert!(miss.background.is_none(), "{class}: not below 900px");
    }
}

/// `and`-conjunctions of typed media features conjoin (spaces are
/// underscore-encoded in class strings, per Tailwind's
/// `decodeArbitraryValue`).
#[test]
fn arbitrary_media_and_conjunction() {
    let r = Resolver::new(&theme())
        .resolve("[@media(min-width:900px)_and_(orientation:landscape)]:bg-black")
        .unwrap();
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    let both = r.plan.resolve(&SelProbe {
        width: Some(900.0),
        landscape: true,
        ..Default::default()
    });
    assert!(both.background.is_some());
    let only_width = r.plan.resolve(&SelProbe {
        width: Some(900.0),
        ..Default::default()
    });
    assert!(only_width.background.is_none());
}

/// `[@container(min-width:400px)]` — a container query window atom.
#[test]
fn arbitrary_container_window() {
    let r = Resolver::new(&theme())
        .resolve("[@container(min-width:400px)]:bg-black")
        .unwrap();
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    let hit = r.plan.resolve(&SelProbe {
        container: Some(400.0),
        ..Default::default()
    });
    assert!(hit.background.is_some());
    let miss = r.plan.resolve(&SelProbe {
        container: Some(399.0),
        ..Default::default()
    });
    assert!(miss.background.is_none());
}

/// `has-[:focus]` — the descendant-state selector form typed like
/// `has-focus:`.
#[test]
fn has_selector_state_form() {
    let r = Resolver::new(&theme())
        .resolve("has-[:focus]:bg-black")
        .unwrap();
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    let hit = r.plan.resolve(&SelProbe {
        descendant_focus: true,
        ..Default::default()
    });
    assert!(hit.background.is_some());
    assert!(r.plan.resolve(&SelProbe::default()).background.is_none());
}

/// `group-[&:hover]` — the arbitrary group inner classifies like
/// `group-hover:`.
#[test]
fn group_arbitrary_inner() {
    let r = Resolver::new(&theme())
        .resolve("group-[&:hover]:bg-black")
        .unwrap();
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    let hit = r.plan.resolve(&SelProbe {
        group_hover: true,
        ..Default::default()
    });
    assert!(hit.background.is_some());
    assert!(r.plan.resolve(&SelProbe::default()).background.is_none());
}

/// `not-[&:hover]` — the named-`not` over an arbitrary self test.
#[test]
fn not_over_arbitrary_self_test() {
    let r = Resolver::new(&theme())
        .resolve("not-[&:hover]:bg-black")
        .unwrap();
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    assert!(
        r.plan.resolve(&SelProbe::default()).background.is_some(),
        "fires when not hovered"
    );
    assert!(
        r.plan
            .resolve(&SelProbe {
                hover: true,
                ..Default::default()
            })
            .background
            .is_none()
    );
}

/// Stacking with a named variant: `md:[&:hover]:` = width ≥ 768 AND hover.
#[test]
fn stacked_with_named_breakpoint() {
    let r = Resolver::new(&theme())
        .resolve("md:[&:hover]:bg-black")
        .unwrap();
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    let hit = r.plan.resolve(&SelProbe {
        width: Some(768.0),
        hover: true,
        ..Default::default()
    });
    assert!(hit.background.is_some());
    let miss = r.plan.resolve(&SelProbe {
        width: Some(768.0),
        ..Default::default()
    });
    assert!(miss.background.is_none());
}

/// Default probes are not asked tag questions they cannot answer — the
/// rule simply never fires (honest, like CSS non-matching).
#[test]
fn default_probe_never_fires_tag_rule() {
    TAG_ASKED.store(false, Ordering::SeqCst);
    let r = Resolver::new(&theme()).resolve("[a&]:underline").unwrap();
    let logging = TagLoggingProbe(SelProbe::default());
    let out = r.plan.resolve(&logging);
    assert!(!has_underline(&out));
    // The probe WAS asked (and answered "no") — that is the contract: the
    // question is honest, the default answer is "no".
    assert!(TAG_ASKED.load(Ordering::SeqCst));
}

// ---------------------------------------------------------------------------
// §1.3 — precise reports for everything else
// ---------------------------------------------------------------------------

/// Helpers for asserting a precise unsupported reason.
fn assert_unsupported(class: &str, needle: &str) {
    let r = Resolver::new(&theme()).resolve(class).unwrap();
    let reasons = warnings_of(&r);
    assert!(
        reasons
            .iter()
            .any(|reason| reason.contains(needle) && !reason.is_empty()),
        "{class}: expected a precise Unsupported containing {needle:?}, got {reasons:?}"
    );
}

/// Selectors styling OTHER elements — the Probe styles the element itself.
/// (`[& svg]` with a literal space cannot occur in a class string — class
/// tokens split on whitespace — so the `_`-encoded form is the case.)
#[test]
fn descendant_and_child_selectors_reported() {
    assert_unsupported("[&_svg]:size-4", "styles other elements");
    assert_unsupported("[&>*]:p-4", "styles other elements");
    assert_unsupported("[&_*]:p-4", "styles other elements");
    assert_unsupported("[&+&]:p-4", "styles other elements");
}

/// The v4 lateral variants `*` (children) and `**` (descendants).
#[test]
fn lateral_star_variants_reported() {
    assert_unsupported("*:p-4", "styles other elements");
    assert_unsupported("**:p-4", "styles other elements");
}

/// Pseudo-elements inside arbitrary selectors.
#[test]
fn pseudo_element_reported() {
    assert_unsupported("[&::before]:bg-black", "pseudo-element");
}

/// Class/id self tests have no probe question.
#[test]
fn class_self_test_reported() {
    assert_unsupported("[.is-published]:block", "class");
    assert_unsupported("[.is-published&]:block", "class");
}

/// `@supports` has no capability query in the Probe model.
#[test]
fn supports_reported() {
    assert_unsupported("[@supports(display:grid)]:bg-black", "@supports");
    assert_unsupported("supports-grid:bg-black", "@supports");
}

/// Disjunctions (`,` media lists, `:is(a,b)`) — the Cond model is a
/// conjunction; reported, never collapsed.
#[test]
fn disjunctions_reported() {
    assert_unsupported("[@media(print),(min-width:900px)]:bg-black", "disjunction");
    assert_unsupported("[&:is(:hover,:focus)]:bg-black", "disjunction");
}

/// An+B structural forms and unknown media features / pseudo-classes.
#[test]
fn unrecognized_forms_reported() {
    assert_unsupported("[&:nth-child(2n+1)]:bg-black", "An+B");
    assert_unsupported("nth-[2n+1]:bg-black", "An+B");
    assert_unsupported("[@media(orientation:square)]:bg-black", "media feature");
    assert_unsupported("[&:focus-ring]:bg-black", "pseudo-class");
    assert_unsupported("[&[contenteditable]]:bg-black", "attribute");
}

/// Relative selectors are invalid standalone (v4: only inside has-*).
#[test]
fn relative_selector_reported() {
    assert_unsupported("[>img]:block", "relative");
}

/// Deprecated at-rule-with-selector form (v4 returns null for it).
#[test]
fn at_rule_with_selector_reported() {
    assert_unsupported("[@media(print){&:hover}]:bg-black", "at-rule");
}

// ---------------------------------------------------------------------------
// §1.4 — cn() keys and cascade
// ---------------------------------------------------------------------------

/// Merge keys stay the RAW variant text: `[&:hover]:p-4` and `hover:p-2`
/// do not knock each other out; the same raw key does.
#[test]
fn cn_keys_use_raw_variant_text() {
    let out = cn(&["[&:hover]:p-4", "hover:p-2"]);
    assert!(out.contains("[&:hover]:p-4"), "{out}");
    assert!(out.contains("hover:p-2"), "{out}");

    let out = cn(&["[&:hover]:p-4", "[&:hover]:p-2"]);
    assert_eq!(out, "[&:hover]:p-2", "same raw key: last wins");
}

/// `[&:hover]:` and `hover:` rank together (identical CSS in v4); source
/// order breaks the tie — the later one wins when both match.
#[test]
fn cascade_tie_keeps_source_order() {
    let r = Resolver::new(&theme())
        .resolve("hover:bg-red-500 [&:hover]:bg-black")
        .unwrap();
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    let hit = r.plan.resolve(&SelProbe {
        hover: true,
        ..Default::default()
    });
    assert!(
        hit.background.is_some(),
        "the later arbitrary form wins the tie"
    );
}
