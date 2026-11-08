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

    /// Does any descendant satisfy the state (`has-checked:`)?
    fn descendant_state(&self, _s: crate::ast::State) -> bool {
        false
    }

    /// Does any descendant satisfy the test (`has-*` variants — v0.9 §2
    /// generalizes them to data/aria/structural inners)? Defaults to
    /// answering only the state form through [`Probe::descendant_state`];
    /// a backend that can answer attribute tests of its children overrides
    /// this.
    fn descendant(&self, t: &Test) -> bool {
        match t {
            Test::State(s) => self.descendant_state(*s),
            _ => false,
        }
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
            Atom::Has(t) => p.descendant(t),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::State;
    use crate::cond::Cond;

    /// A scripted probe: answers from a fixed table, recording nothing.
    struct Scripted {
        tests: Vec<Test>,
        group: bool,
        peer: bool,
        descendant_state: Option<State>,
        media: Vec<Media>,
        viewport: Option<f32>,
        container: Option<f32>,
    }

    impl Probe for Scripted {
        fn test(&self, t: &Test) -> bool {
            self.tests.contains(t)
        }
        fn related(&self, rel: Rel, _label: Option<&str>, t: &Test) -> bool {
            match rel {
                Rel::Group => self.group && matches!(t, Test::State(_)),
                Rel::Peer => self.peer && matches!(t, Test::State(_)),
            }
        }
        fn descendant_state(&self, s: State) -> bool {
            self.descendant_state == Some(s)
        }
        fn media(&self, m: Media) -> bool {
            self.media.contains(&m)
        }
        fn viewport_width(&self) -> Option<f32> {
            self.viewport
        }
        fn container_width(&self, _name: Option<&str>) -> Option<f32> {
            self.container
        }
    }

    fn on(state: State) -> Atom {
        Atom::On(Test::State(state))
    }

    /// The bare `Test` form, for the `Not`/`Has` wrappers.
    fn ts(state: State) -> Test {
        Test::State(state)
    }

    /// The default probe answers "no" everywhere — a backend implements
    /// only what it has, and unimplemented questions never fire rules.
    #[test]
    fn default_probe_answers_no_to_everything() {
        struct Nil;
        impl Probe for Nil {}
        let p = Nil;
        assert!(!on(State::Hover).matches(&p));
        // `not-hover` against the default probe: test()=false ⇒ Not fires.
        assert!(Atom::Not(ts(State::Hover)).matches(&p));
        assert!(!Atom::Has(ts(State::Checked)).matches(&p));
        assert!(!Atom::Media(Media::Dark).matches(&p));
        assert!(
            !Atom::Viewport {
                min: None,
                max: None
            }
            .matches(&p)
        );
        assert!(
            !Atom::Container {
                min: None,
                max: None,
                name: None
            }
            .matches(&p)
        );
        // The descendant default answers only the state form (v0.9 §2).
        assert!(!p.descendant(&Test::Data("slot".into(), None)));
        assert!(!p.descendant(&Test::State(State::Checked)));
    }

    /// `Atom::matches` routes each arm to its probe question.
    #[test]
    fn atom_matches_routes_each_arm() {
        let p = Scripted {
            tests: vec![Test::State(State::Hover)],
            group: false,
            peer: false,
            descendant_state: None,
            media: vec![Media::Dark],
            viewport: Some(800.0),
            container: None,
        };
        assert!(on(State::Hover).matches(&p));
        assert!(!on(State::Focus).matches(&p));
        assert!(Atom::Not(ts(State::Focus)).matches(&p));
        assert!(Atom::Media(Media::Dark).matches(&p));
        assert!(!Atom::Media(Media::Print).matches(&p));
        assert!(
            Atom::Viewport {
                min: Some(640.0),
                max: Some(1024.0)
            }
            .matches(&p)
        );
        assert!(
            !Atom::Viewport {
                min: Some(1024.0),
                max: None
            }
            .matches(&p)
        );
    }

    /// `in_window` is `[min, max)`: min inclusive, max exclusive, and an
    /// unknown width never matches (an unknown window cannot honestly
    /// satisfy a width query).
    #[test]
    fn in_window_is_min_inclusive_max_exclusive() {
        assert!(in_window(Some(640.0), Some(640.0), Some(1024.0)));
        assert!(!in_window(Some(1024.0), Some(640.0), Some(1024.0)));
        assert!(in_window(Some(1023.999), Some(640.0), Some(1024.0)));
        assert!(in_window(Some(5000.0), Some(640.0), None));
        assert!(in_window(Some(100.0), None, Some(640.0)));
        assert!(!in_window(None, None, None));
        assert!(!in_window(None, Some(0.0), None));
    }

    /// `split_group_label` longest-suffix match: `menufocus-visible` must
    /// split as `(menu, focus-visible)`, NOT `(menufocus, visible)`-ish.
    #[test]
    fn split_group_label_is_longest_suffix() {
        assert_eq!(split_group_label("hover"), Some((None, State::Hover)));
        assert_eq!(
            split_group_label("menufocus"),
            Some((Some("menu"), State::Focus))
        );
        assert_eq!(
            split_group_label("menufocus-visible"),
            Some((Some("menu"), State::FocusVisible))
        );
        assert_eq!(
            split_group_label("xplaceholder-shown"),
            Some((Some("x"), State::PlaceholderShown))
        );
        assert_eq!(split_group_label("nosuchstate"), None);
    }

    /// Cascade order (the parity engine's core): a focused+hovered element
    /// resolves the FOCUS layer even when the class string listed `hover:`
    /// first — rank beats source order within the state band.
    #[test]
    fn resolve_prefers_focus_over_hover_regardless_of_source_order() {
        let mut plan = StylePlan::default();
        let hover_bg = ResolvedStyle {
            background: Some(crate::plan::Background(crate::color::Color {
                l: 0.0,
                a: 0.0,
                b: 0.0,
                alpha: 1.0,
            })),
            ..Default::default()
        };

        let focus_ring = ResolvedStyle {
            ring: Some(crate::plan::Ring {
                width: 3.0,
                color: None,
                offset_width: 0.0,
                offset_color: None,
                inset: false,
            }),
            ..Default::default()
        };

        // Source order: hover FIRST, focus second.
        plan.layers.push(crate::plan::ConditionalLayer {
            condition: crate::plan::VariantCondition::Hover,
            style: hover_bg.clone(),
        });
        plan.layers.push(crate::plan::ConditionalLayer {
            condition: crate::plan::VariantCondition::FocusVisible,
            style: focus_ring.clone(),
        });
        let p = Scripted {
            tests: vec![Test::State(State::Hover), Test::State(State::FocusVisible)],
            group: false,
            peer: false,
            descendant_state: None,
            media: vec![],
            viewport: None,
            container: None,
        };
        let out = plan.resolve(&p);
        assert!(out.background.is_some(), "hover layer applies");
        assert!(out.ring.is_some(), "focus-visible layer applies");
        // The reverse source order must produce the SAME result: rank
        // ordering makes the flatten source-order independent within a band.
        let mut flipped = StylePlan::default();
        flipped.layers.push(crate::plan::ConditionalLayer {
            condition: crate::plan::VariantCondition::FocusVisible,
            style: focus_ring.clone(),
        });
        flipped.layers.push(crate::plan::ConditionalLayer {
            condition: crate::plan::VariantCondition::Hover,
            style: hover_bg.clone(),
        });
        assert_eq!(
            out,
            flipped.resolve(&p),
            "cascade rank must beat source order"
        );
    }

    /// Dark beats the state band (140 > 10): a dark+hover element in dark
    /// mode resolves the dark bucket over the hover layer.
    #[test]
    fn resolve_dark_beats_the_state_band() {
        let mut plan = StylePlan::default();
        let hover_bg = ResolvedStyle {
            background: Some(crate::plan::Background(crate::color::Color {
                l: 0.0,
                a: 0.0,
                b: 0.0,
                alpha: 1.0,
            })),
            ..Default::default()
        };

        let dark_bg = ResolvedStyle {
            background: Some(crate::plan::Background(crate::color::Color {
                l: 1.0,
                a: 1.0,
                b: 1.0,
                alpha: 1.0,
            })),
            ..Default::default()
        };

        plan.layers.push(crate::plan::ConditionalLayer {
            condition: crate::plan::VariantCondition::Hover,
            style: hover_bg,
        });
        plan.dark = Some(dark_bg);
        let p = Scripted {
            tests: vec![Test::State(State::Hover)],
            group: false,
            peer: false,
            descendant_state: None,
            media: vec![Media::Dark],
            viewport: None,
            container: None,
        };
        let out = plan.resolve(&p);
        assert_eq!(
            out.background,
            Some(crate::plan::Background(crate::color::Color {
                l: 1.0,
                a: 1.0,
                b: 1.0,
                alpha: 1.0
            }))
        );
        // Out of dark mode only the hover layer fires.
        let light = Scripted {
            tests: vec![Test::State(State::Hover)],
            group: false,
            peer: false,
            descendant_state: None,
            media: vec![],
            viewport: None,
            container: None,
        };
        let out = plan.resolve(&light);
        assert_eq!(
            out.background,
            Some(crate::plan::Background(crate::color::Color {
                l: 0.0,
                a: 0.0,
                b: 0.0,
                alpha: 1.0
            }))
        );
    }

    /// Responsive buckets use the plan's OWN breakpoint table (`sm:` is 640
    /// only because the table says so) — a lowered width wins by px, not by
    /// name.
    #[test]
    fn resolve_responsive_uses_the_plan_breakpoint_table() {
        let mut plan = StylePlan {
            breakpoints: [640.0, 768.0, 1024.0, 1280.0, 1536.0],
            ..Default::default()
        };

        let sm = ResolvedStyle {
            opacity: Some(crate::plan::Opacity(0.5)),
            ..Default::default()
        };

        plan.responsive.insert(crate::ast::Breakpoint::Sm, sm);
        let wide = Scripted {
            tests: vec![],
            group: false,
            peer: false,
            descendant_state: None,
            media: vec![],
            viewport: Some(639.0),
            container: None,
        };
        assert!(
            plan.resolve(&wide).opacity.is_none(),
            "639 < sm(640): no hit"
        );
        let at = Scripted {
            tests: vec![],
            group: false,
            peer: false,
            descendant_state: None,
            media: vec![],
            viewport: Some(640.0),
            container: None,
        };
        assert_eq!(
            plan.resolve(&at).opacity,
            Some(crate::plan::Opacity(0.5)),
            "min is inclusive"
        );
    }

    /// `resolve_rules_only` touches ONLY rules + group/peer buckets — the
    /// Floem reactive path relies on layers/dark/responsive NOT leaking in.
    #[test]
    fn resolve_rules_only_excludes_buckets() {
        let mut plan = StylePlan::default();
        let hover_bg = ResolvedStyle {
            background: Some(crate::plan::Background(crate::color::Color {
                l: 0.0,
                a: 0.0,
                b: 0.0,
                alpha: 1.0,
            })),
            ..Default::default()
        };

        plan.layers.push(crate::plan::ConditionalLayer {
            condition: crate::plan::VariantCondition::Hover,
            style: hover_bg,
        });
        let dark_bg = ResolvedStyle {
            opacity: Some(crate::plan::Opacity(0.25)),
            ..Default::default()
        };

        plan.dark = Some(dark_bg);
        let rule_style = ResolvedStyle {
            cursor: Some(crate::plan::Cursor::Pointer),
            ..Default::default()
        };

        plan.rules.push(Rule {
            when: Cond(vec![Atom::On(ts(State::Checked))]),
            style: rule_style,
        });
        let p = Scripted {
            tests: vec![Test::State(State::Checked), Test::State(State::Hover)],
            group: false,
            peer: false,
            descendant_state: None,
            media: vec![Media::Dark],
            viewport: None,
            container: None,
        };
        let out = plan.resolve_rules_only(&p);
        assert!(out.background.is_none(), "layers must not leak");
        assert!(out.opacity.is_none(), "dark must not leak");
        assert_eq!(
            out.cursor,
            Some(crate::plan::Cursor::Pointer),
            "matching rule fires"
        );
    }
}
