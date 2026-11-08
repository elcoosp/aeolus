//! The backend-agnostic evaluator — the parity engine (plan §2.2).
//!
//! A backend answers questions about the element's current situation by
//! implementing [`Probe`] (every method defaults to "no", so a backend
//! implements only what it has). [`StylePlan::resolve`] then flattens the
//! plan into one [`ResolvedStyle`] for that situation, with candidate order
//! = cascade order (v4 variant order, stable ties keep source order).
//!
//! Both backends call the same evaluator: the Masonry fallback widget
//! resolves per state at pre-paint; Floem calls it from a reactive style
//! closure. Neither needs the framework to understand `group-hover`,
//! `dark:hover`, `data-[state=open]` or `@md` — they only have to answer
//! `Probe`.

use crate::ast::{Media, State};
use crate::cond::{Atom, Rule, Test};
use crate::plan::{ResolvedStyle, StylePlan};

/// Which related element a probe question is about.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Rel {
    /// A `.group` ancestor.
    Group,
    /// A `.peer` sibling.
    Peer,
}

/// A backend answers questions about the element's current situation.
///
/// Every method defaults to "no", so a backend implements only what it has
/// (plan §2.2). Backends that cannot answer a question honestly (e.g. no
/// knowledge of ancestor state) leave the default and the corresponding
/// rule simply does not fire — the same semantics CSS has when a selector
/// does not match.
pub trait Probe {
    /// Does the element itself satisfy `t`? (states, data/aria attrs,
    /// structural position).
    fn test(&self, _t: &Test) -> bool {
        false
    }

    /// Does the related element (`Rel::Group` ancestor / `Rel::Peer`
    /// sibling, optionally labeled) satisfy `t`?
    fn related(&self, _rel: Rel, _label: Option<&str>, _t: &Test) -> bool {
        false
    }

    /// Does any descendant satisfy the state (`has-*` variants)?
    fn descendant_state(&self, _s: crate::ast::State) -> bool {
        false
    }

    /// Is the media query active (`dark`, `print`, `motion-reduce`, ...)?
    fn media(&self, _m: Media) -> bool {
        false
    }

    /// The current viewport width in px, when known.
    fn viewport_width(&self) -> Option<f32> {
        None
    }

    /// The current container width in px for the (optionally named)
    /// container, when known.
    fn container_width(&self, _name: Option<&str>) -> Option<f32> {
        None
    }
}

/// Is `w` inside the `[min, max)` window? `None` width never matches —
/// an unknown window cannot honestly satisfy a width query.
pub fn in_window(w: Option<f32>, min: Option<f32>, max: Option<f32>) -> bool {
    match w {
        None => false,
        Some(w) => min.is_none_or(|m| w >= m) && max.is_none_or(|m| w < m),
    }
}

impl Atom {
    /// Evaluate this atom against the probe.
    pub fn matches(&self, p: &impl Probe) -> bool {
        match self {
            Atom::On(t) => p.test(t),
            Atom::Not(t) => !p.test(t),
            Atom::Group(t, l) => p.related(Rel::Group, l.as_deref(), t),
            Atom::Peer(t, l) => p.related(Rel::Peer, l.as_deref(), t),
            Atom::Has(s) => p.descendant_state(*s),
            Atom::Media(m) => p.media(*m),
            Atom::Viewport { min, max } => in_window(p.viewport_width(), *min, *max),
            Atom::Container { min, max, name } => {
                in_window(p.container_width(name.as_deref()), *min, *max)
            }
        }
    }
}

impl StylePlan {
    /// Flatten the plan for one situation. Candidate order = cascade order:
    /// legacy layers first (state < viewport < dark, per [`Atom::order`]),
    /// then `rules` ranked by their condition's rank; stable sort keeps
    /// source order
    /// for ties.
    pub fn resolve(&self, p: &impl Probe) -> ResolvedStyle {
        let mut hits: Vec<((u16, u8), &ResolvedStyle)> = Vec::with_capacity(8);

        for l in &self.layers {
            let state = State::from(l.condition);
            if p.test(&Test::State(state)) {
                // The tiebreak is the condition's canonical v4 rank
                // (hover < focus < focus-visible < focus-within < active <
                // disabled < visited — `VariantCondition::rank`), NOT source
                // order: Tailwind sorts the generated CSS by variant order,
                // so a focused+hovered element resolves the FOCUS layer even
                // when the class string lists `hover:` first. Same order the
                // stack lowering and `resolved_for_state` use.
                hits.push((
                    (Atom::On(Test::State(state)).order(), l.condition.rank() + 1),
                    &l.style,
                ));
            }
        }
        for (bp, s) in &self.responsive {
            if p.viewport_width()
                .is_some_and(|w| w >= self.breakpoints[bp.index()])
            {
                hits.push(((110, 1), s));
            }
        }
        if let Some(d) = &self.dark
            && p.media(Media::Dark)
        {
            hits.push(((140, 1), d));
        }
        for (label, s) in &self.group {
            // Legacy label is "{label}{state}" (e.g. "hover", "menufocus");
            // split it back so the probe answers the same question the
            // router asked (plan §4.3 helper).
            if let Some((l, st)) = split_group_label(label)
                && p.related(Rel::Group, l, &Test::State(st))
            {
                hits.push(((10, 1), s));
            }
        }
        for (label, s) in &self.peer {
            if let Some((l, st)) = split_group_label(label)
                && p.related(Rel::Peer, l, &Test::State(st))
            {
                hits.push(((10, 1), s));
            }
        }
        for r in &self.rules {
            if r.when.0.iter().all(|a| a.matches(p)) {
                hits.push((r.when.rank(), &r.style));
            }
        }
        hits.sort_by_key(|(rank, _)| *rank); // stable: ties keep source order

        let mut out = self.base.clone();
        for (_, s) in hits {
            out.overlay(s);
        }
        out
    }

    /// Evaluate ONLY the v0.2 `rules` (plus legacy group/peer buckets) —
    /// the pieces native selector maps cannot express. Used by the Floem
    /// reactive path where layers/dark/responsive lower natively and the
    /// closure re-applies the emulated remainder (plan §4.2).
    pub fn resolve_rules_only(&self, p: &impl Probe) -> ResolvedStyle {
        let mut out = ResolvedStyle::default();
        for (label, s) in &self.group {
            if let Some((l, st)) = split_group_label(label)
                && p.related(Rel::Group, l, &Test::State(st))
            {
                out.overlay(s);
            }
        }
        for (label, s) in &self.peer {
            if let Some((l, st)) = split_group_label(label)
                && p.related(Rel::Peer, l, &Test::State(st))
            {
                out.overlay(s);
            }
        }
        let mut hits: Vec<((u16, u8), &Rule)> = self
            .rules
            .iter()
            .filter(|r| r.when.0.iter().all(|a| a.matches(p)))
            .map(|r| (r.when.rank(), r))
            .collect();
        hits.sort_by_key(|(rank, _)| *rank);
        for (_, r) in hits {
            out.overlay(&r.style);
        }
        out
    }
}

/// Split a legacy group/peer label (`"hover"`, `"menufocus"`) into
/// `(label, state)`. Returns `None` for unknown state suffixes.
pub fn split_group_label(label: &str) -> Option<(Option<&str>, crate::ast::State)> {
    // Longest-suffix match over the state names.
    const SUFFIXES: &[(&str, crate::ast::State)] = &[
        ("focus-visible", crate::ast::State::FocusVisible),
        ("focus-within", crate::ast::State::FocusWithin),
        ("placeholder-shown", crate::ast::State::PlaceholderShown),
        ("hover", crate::ast::State::Hover),
        ("focus", crate::ast::State::Focus),
        ("active", crate::ast::State::Active),
        ("disabled", crate::ast::State::Disabled),
        ("visited", crate::ast::State::Visited),
        ("target", crate::ast::State::Target),
        ("checked", crate::ast::State::Checked),
        ("invalid", crate::ast::State::Invalid),
        ("valid", crate::ast::State::Valid),
    ];
    for (suffix, state) in SUFFIXES {
        if let Some(prefix) = label.strip_suffix(suffix) {
            let l = if prefix.is_empty() {
                None
            } else {
                Some(prefix)
            };
            return Some((l, *state));
        }
    }
    None
}

use crate::plan::VariantCondition;

impl From<VariantCondition> for crate::ast::State {
    fn from(c: VariantCondition) -> Self {
        match c {
            VariantCondition::Hover => crate::ast::State::Hover,
            VariantCondition::Active => crate::ast::State::Active,
            VariantCondition::Disabled => crate::ast::State::Disabled,
            VariantCondition::Focus => crate::ast::State::Focus,
            VariantCondition::FocusVisible => crate::ast::State::FocusVisible,
            VariantCondition::FocusWithin => crate::ast::State::FocusWithin,
            VariantCondition::Visited => crate::ast::State::Visited,
        }
    }
}
