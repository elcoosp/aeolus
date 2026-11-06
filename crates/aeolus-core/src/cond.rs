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
    /// A descendant of the element matches the state (`has-checked:`).
    Has(State),
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
