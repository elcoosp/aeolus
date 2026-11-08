//! The v0.2 condition IR: [`Atom`], [`Cond`], [`Rule`].
//!
//! Everything the v0.1 buckets cannot express — compound variants
//! (`dark:hover:`), `group-*`/`peer-*` over data/aria, `has-*`, `not-*`,
//! structural selectors, container queries, exotic media — lands here as a
//! conjunction of [`Atom`]s evaluated by the backend-agnostic `Probe`
//! evaluator (plan §2).

use crate::ast::{Media, State, Structural};

/// One conjunct of a rule condition.
#[derive(Clone, Debug, PartialEq)]
pub enum Atom {
    /// The element itself is in the described situation (`hover:`,
    /// `data-[state=open]:`, `first:`).
    On(Test),
    /// `not-*` — the element is NOT in the described situation.
    Not(Test),
    /// A `.group[/label]` ancestor matches `Test`.
    Group(Test, Option<Box<str>>),
    /// A `.peer[/label]` sibling matches `Test`.
    Peer(Test, Option<Box<str>>),
    /// A descendant of the element matches the test (`has-checked:`,
    /// and — v0.7 §2 — `has-data-[…]:`/`has-aria-[…]:`/`has-first:`).
    Has(Test),
    /// A media query (`dark:`, `print:`, `motion-reduce:`, ...).
    Media(Media),
    /// Viewport width window (`sm:`–`2xl:`, `min-*`, `max-*`).
    Viewport {
        /// Inclusive lower bound in px.
        min: Option<f32>,
        /// Exclusive upper bound in px.
        max: Option<f32>,
    },
    /// Container query window (`@sm:`, `@max-md:`, `@min-[400px]:`, named).
    Container {
        /// Inclusive lower bound in px.
        min: Option<f32>,
        /// Exclusive upper bound in px.
        max: Option<f32>,
        /// Named container (`@md/name`).
        name: Option<Box<str>>,
    },
}

/// What an [`Atom`] tests.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Test {
    /// A pseudo-state of the element itself.
    State(State),
    /// `data-*` attribute (name, optional value).
    Data(Box<str>, Option<Box<str>>),
    /// `aria-*` attribute (name, optional value).
    Aria(Box<str>, Option<Box<str>>),
    /// A structural (DOM-position) test.
    Structural(Structural),
    /// An element-kind test (`[a&]`, `[p]` — v0.3 §1.2): the element "is
    /// a" tag. Native widgets have no element kind; the default probe
    /// answer is "no", so the rule simply never fires there — the same
    /// semantics CSS has when a selector does not match. A backend that
    /// models element kinds (e.g. a web backend) answers honestly.
    Tag(Box<str>),
    /// The element's own pseudo-element rendering (`selection:`,
    /// `placeholder:`, `file:` — v0.7 §1): the style applies to that
    /// rendering while it exists. Not a runtime state — the default probe
    /// answer is "no", so the rule never fires on a backend that does not
    /// model the rendering (the same semantics CSS has when a selector
    /// does not match). Backends with the surface (e.g. text-selection
    /// colors on a text-input wrapper) answer honestly via `Probe::test`.
    Pseudo(crate::ast::PseudoElement),
}

impl Atom {
    /// Tailwind v4's variant order, encoded so `dark:` and breakpoints beat
    /// `hover:` when both match, as in CSS (plan §2.1). The scale mirrors
    /// the legacy buckets: states ~10, container ~108, viewport 110,
    /// dark 140, other media 130/150.
    pub fn order(&self) -> u16 {
        match self {
            Atom::On(t) => match t {
                Test::Structural(_) => 8,
                Test::State(_) => 10,
                // Element-kind tests rank with the state band: they are
                // tests about the element itself, and arbitrary variants
                // that are byte-identical to named ones ([&:hover] ≡
                // hover:) must share their band so the cascade order is
                // stable (v0.3 §1.2; ties keep source order).
                Test::Tag(_) => 10,
                // Pseudo-element renderings rank with the state band too:
                // v4 emits them in the same variant order.
                Test::Pseudo(_) => 10,
                // aria/data come after states in v4's variant order.
                Test::Data(_, _) | Test::Aria(_, _) => 14,
            },
            Atom::Not(_) => 10,
            Atom::Group(_, _) | Atom::Peer(_, _) | Atom::Has(_) => 10,
            Atom::Container { .. } => 108,
            Atom::Viewport { .. } => 110,
            Atom::Media(Media::Dark) => 140,
            Atom::Media(Media::Print) => 150,
            Atom::Media(_) => 130,
        }
    }
}

/// A conjunction of [`Atom`]s — every atom must hold for the rule to apply.
#[derive(Clone, Debug, PartialEq)]
pub struct Cond(pub Vec<Atom>);

impl Cond {
    /// Cascade rank: `(max atom order, atom count)`. Ties keep source order
    /// (stable sort in the evaluator).
    pub fn rank(&self) -> (u16, u8) {
        let max = self.0.iter().map(Atom::order).max().unwrap_or(0);
        (max, self.0.len().min(255) as u8)
    }
}

/// One conditional rule: `when` holds ⇒ overlay `style` (at `when.rank()`).
#[derive(Clone, Debug, PartialEq)]
pub struct Rule {
    /// The condition conjunction.
    pub when: Cond,
    /// The style to overlay while the condition holds.
    pub style: crate::plan::ResolvedStyle,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Media, PseudoElement, State, Structural};

    /// The `Atom::order` band table is the cascade contract (v4 variant
    /// order): structural < states/tag/pseudo < data-aria < container <
    /// viewport < media < dark < print. Pin every band so a reorder is a
    /// deliberate, reviewed decision — the evaluator, the CSS emitter and
    /// every stack lowering depend on these exact numbers.
    #[test]
    fn order_bands_match_the_v4_cascade_contract() {
        let structural = Atom::On(Test::Structural(Structural::First));
        let state = Atom::On(Test::State(State::Hover));
        let tag = Atom::On(Test::Tag("a".into()));
        let pseudo = Atom::On(Test::Pseudo(PseudoElement::Selection));
        let data = Atom::On(Test::Data("state".into(), Some("open".into())));
        let aria = Atom::On(Test::Aria("expanded".into(), None));
        assert_eq!(structural.order(), 8);
        assert_eq!(state.order(), 10);
        assert_eq!(tag.order(), 10);
        assert_eq!(pseudo.order(), 10);
        assert_eq!(data.order(), 14);
        assert_eq!(aria.order(), 14);
        assert_eq!(Atom::Not(Test::State(State::Hover)).order(), 10);
        assert_eq!(Atom::Group(Test::State(State::Hover), None).order(), 10);
        assert_eq!(Atom::Peer(Test::State(State::Hover), None).order(), 10);
        assert_eq!(Atom::Has(Test::State(State::Hover)).order(), 10);
        assert_eq!(Atom::Container { min: None, max: None, name: None }.order(), 108);
        assert_eq!(Atom::Viewport { min: None, max: None }.order(), 110);
        assert_eq!(Atom::Media(Media::MotionReduce).order(), 130);
        assert_eq!(Atom::Media(Media::Dark).order(), 140);
        assert_eq!(Atom::Media(Media::Print).order(), 150);
        // Band ordering itself (the contract, not just the numbers).
        assert!(structural.order() < state.order());
        assert!(state.order() < data.order());
        assert!(data.order() < Atom::Container { min: None, max: None, name: None }.order());
        assert!(Atom::Container { min: None, max: None, name: None }.order()
            < Atom::Viewport { min: None, max: None }.order());
        assert!(Atom::Media(Media::MotionReduce).order() < Atom::Media(Media::Dark).order());
        assert!(Atom::Media(Media::Dark).order() < Atom::Media(Media::Print).order());
    }

    /// `Cond::rank` is `(max atom order, atom count)` — the max wins so a
    /// `dark:hover:` conjunction sorts with dark, not with hover.
    #[test]
    fn cond_rank_takes_the_max_atom_order() {
        let dark_hover = Cond(vec![
            Atom::Media(Media::Dark),
            Atom::On(Test::State(State::Hover)),
        ]);
        assert_eq!(dark_hover.rank(), (140, 2));
        let single = Cond(vec![Atom::On(Test::State(State::Hover))]);
        assert_eq!(single.rank(), (10, 1));
        // The empty conjunction ranks at 0 — it can never occur from the
        // parser (a rule always has ≥1 atom), but it must not panic.
        assert_eq!(Cond(vec![]).rank(), (0, 0));
    }

    /// Pseudo-element tests are distinguishable from states with the same
    /// display name (`selection:` vs a hypothetical `selection` state) —
    /// the merge-key machinery relies on the typed distinction.
    #[test]
    fn pseudo_tests_are_their_own_arms() {
        let pseudo = Test::Pseudo(PseudoElement::Selection);
        assert_ne!(pseudo, Test::State(State::Hover));
        assert_eq!(pseudo, Test::Pseudo(PseudoElement::Selection));
        // All three v0.7 pseudo-elements construct and differ.
        assert_ne!(
            Test::Pseudo(PseudoElement::Placeholder),
            Test::Pseudo(PseudoElement::FileSelectorButton)
        );
    }
}
