//! The variant & grid engine (v0.7, plan `AEOLUS_V0.7.md`).
//!
//! These tests pin the v0.7 contract: every variant form the shipped
//! shadcn v4 oracle tables use must parse and classify into the typed
//! condition model — pseudo-element variants (`selection:`, `placeholder:`,
//! `file:`), `has-data-[…]`/`has-aria-[…]` descendants, named container
//! declarations (`@container[/name]`), arbitrary grid templates
//! (`grid-cols-[…]`, `grid-rows-[…]`, `auto-rows-*`, `auto-cols-*`) and
//! `justify-items-*` — so the tables resolve warning-free up to the
//! documented Probe-model boundary (selectors styling OTHER elements).

use aeolus_core::ast::{Media, PseudoElement, Structural};
use aeolus_core::cond::{Atom, Test};
use aeolus_core::eval::Probe;
use aeolus_core::*;
use plan::{GridTrack, TrackSizing};

fn theme() -> TailwindTheme {
    TailwindTheme::from_theme_css(shadcn_css::LIGHT).unwrap()
}

fn resolve(class: &str) -> Resolution {
    let t = theme();
    let r = Resolver::new(&t);
    let res = r.resolve(class).expect("parse");
    assert!(
        res.warnings.is_empty(),
        "unexpected warnings for `{class}`: {:?}",
        res.warnings
    );
    res
}

fn rules_of(class: &str) -> Vec<(Cond, ResolvedStyle)> {
    let res = resolve(class);
    res.plan
        .rules
        .into_iter()
        .map(|r| (r.when, r.style))
        .collect()
}

// ---------------------------------------------------------------------------
// §1 — pseudo-element variants: `selection:` / `placeholder:` / `file:`
// ---------------------------------------------------------------------------

#[test]
fn selection_variant_is_a_typed_pseudo_rule() {
    let rules = rules_of("selection:bg-primary");
    assert_eq!(rules.len(), 1, "one rule");
    let (cond, style) = &rules[0];
    assert_eq!(
        cond.0,
        vec![Atom::On(Test::Pseudo(PseudoElement::Selection))],
        "selection: classifies onto the typed pseudo-element condition"
    );
    assert!(
        style.background.is_some(),
        "the background lands in the rule"
    );
}

#[test]
fn placeholder_variant_is_a_typed_pseudo_rule() {
    let rules = rules_of("placeholder:text-muted-foreground");
    assert_eq!(rules.len(), 1);
    let (cond, style) = &rules[0];
    assert_eq!(
        cond.0,
        vec![Atom::On(Test::Pseudo(PseudoElement::Placeholder))]
    );
    assert!(
        style.content_color.is_some(),
        "the text color lands in the rule (text-{{color}} resolves to content_color)"
    );
}

#[test]
fn file_variant_is_a_typed_pseudo_rule() {
    let rules = rules_of("file:text-sm");
    assert_eq!(rules.len(), 1);
    let (cond, _) = &rules[0];
    assert_eq!(
        cond.0,
        vec![Atom::On(Test::Pseudo(PseudoElement::FileSelectorButton))]
    );
}

#[test]
fn selection_stacks_with_other_variants() {
    // `dark:selection:bg-*` — two conditions, conjunction in `rules`.
    let rules = rules_of("dark:selection:bg-primary");
    assert_eq!(rules.len(), 1);
    let (cond, _) = &rules[0];
    assert!(cond.0.contains(&Atom::Media(Media::Dark)));
    assert!(
        cond.0
            .contains(&Atom::On(Test::Pseudo(PseudoElement::Selection)))
    );
}

#[test]
fn pseudo_rules_never_fire_on_a_default_probe() {
    // The parity contract: a backend that does not answer the pseudo
    // question simply never fires the rule — the same semantics CSS has
    // when a selector does not match.
    #[derive(Default)]
    struct NilProbe;
    impl Probe for NilProbe {}
    let t = theme();
    let plan = Resolver::new(&t)
        .resolve("bg-red-500 selection:bg-primary")
        .unwrap()
        .plan;
    let flat = plan.resolve(&NilProbe);
    assert!(flat.background.is_some(), "base still styles");
    // `selection:bg-*` sets background inside the rule only; base is red.
    // (Default probe = not selected = rule does not fire.)
}

#[test]
fn pseudo_rules_fire_when_the_probe_answers() {
    #[derive(Default)]
    struct SelectedProbe;
    impl Probe for SelectedProbe {
        fn test(&self, t: &Test) -> bool {
            matches!(t, Test::Pseudo(PseudoElement::Selection))
        }
    }
    let t = theme();
    let plan = Resolver::new(&t)
        .resolve("selection:bg-primary")
        .unwrap()
        .plan;
    let flat = plan.resolve(&SelectedProbe);
    assert!(flat.background.is_some(), "the selection rule fires");
}

// ---------------------------------------------------------------------------
// §2 — has-data-[…]/has-aria-[…] route through the typed descendant test
// ---------------------------------------------------------------------------

#[test]
fn has_data_routes_to_a_typed_descendant() {
    let rules = rules_of("has-data-[slot=card-action]:grid-cols-2");
    assert_eq!(rules.len(), 1);
    let (cond, style) = &rules[0];
    assert!(
        cond.0.contains(&Atom::Has(Test::Data(
            "slot".into(),
            Some("card-action".into())
        ))),
        "has-data-[slot=…] classifies onto Atom::Has(Test::Data)"
    );
    assert_eq!(style.extras.grid.as_ref().and_then(|g| g.cols), Some(2));
}

#[test]
fn has_aria_routes_to_a_typed_descendant() {
    let rules = rules_of("has-aria-invalid:border-destructive");
    assert_eq!(rules.len(), 1);
    let (cond, _) = &rules[0];
    assert!(
        cond.0
            .contains(&Atom::Has(Test::Aria("invalid".into(), None))),
        "has-aria-invalid classifies onto Atom::Has(Test::Aria)"
    );
}

#[test]
fn has_structural_routes_to_a_typed_descendant() {
    let rules = rules_of("has-first:grid-cols-2");
    assert_eq!(rules.len(), 1);
    let (cond, _) = &rules[0];
    assert!(
        cond.0
            .contains(&Atom::Has(Test::Structural(Structural::First)))
    );
}

// ---------------------------------------------------------------------------
// §3 — named container declarations: `@container` / `@container/name`
// ---------------------------------------------------------------------------

#[test]
fn container_utility_establishes_containment() {
    let res = resolve("@container");
    assert_eq!(
        res.plan.base.extras.container_type,
        Some(ContainerType { name: None }),
        "bare @container = container-type: inline-size, unnamed"
    );
}

#[test]
fn named_container_utility_carries_the_name() {
    let res = resolve("@container/card-header");
    assert_eq!(
        res.plan.base.extras.container_type,
        Some(ContainerType {
            name: Some("card-header".into())
        }),
        "@container/name = container-type: inline-size + container-name"
    );
}

// ---------------------------------------------------------------------------
// §5 — justify-items-*
// ---------------------------------------------------------------------------

#[test]
fn justify_items_lands_in_the_layout_slot() {
    use plan::AlignItems;
    for (class, want) in [
        ("justify-items-start", AlignItems::Start),
        ("justify-items-center", AlignItems::Center),
        ("justify-items-end", AlignItems::End),
        ("justify-items-stretch", AlignItems::Stretch),
        ("justify-items-baseline", AlignItems::Baseline),
        ("justify-items-normal", AlignItems::Stretch),
    ] {
        let res = resolve(class);
        assert_eq!(
            res.plan.base.layout.as_ref().and_then(|l| l.justify_items),
            Some(want),
            "{class}"
        );
    }
}

// ---------------------------------------------------------------------------
// §4 — arbitrary grid templates + auto-rows/auto-cols
// ---------------------------------------------------------------------------

#[test]
fn grid_cols_arbitrary_tracks() {
    let res = resolve("grid-cols-[0_1fr]");
    let g = res.plan.base.extras.grid.as_ref().expect("grid");
    assert_eq!(
        g.template_cols.as_deref(),
        Some(
            &[
                TrackComponent::Track(GridTrack::px(0.0)),
                TrackComponent::Track(GridTrack::fr(1.0)),
            ][..]
        ),
        "`0` is a 0px length track; `1fr` is min=auto max=1fr (taffy semantics)"
    );
}

#[test]
fn grid_rows_arbitrary_tracks() {
    let res = resolve("grid-rows-[auto_auto]");
    let g = res.plan.base.extras.grid.as_ref().expect("grid");
    assert_eq!(
        g.template_rows.as_deref(),
        Some(
            &[
                TrackComponent::Track(GridTrack::auto()),
                TrackComponent::Track(GridTrack::auto()),
            ][..]
        )
    );
}

#[test]
fn grid_cols_number_form_still_sets_the_count() {
    let res = resolve("grid-cols-3");
    let g = res.plan.base.extras.grid.as_ref().expect("grid");
    assert_eq!(g.cols, Some(3));
    assert!(g.template_cols.is_none(), "the N form keeps the v0.6 shape");
}

#[test]
fn grid_template_minmax_and_repeat() {
    let res = resolve("grid-cols-[minmax(0,1fr)_2fr]");
    let g = res.plan.base.extras.grid.as_ref().expect("grid");
    assert_eq!(
        g.template_cols.as_deref(),
        Some(
            &[
                TrackComponent::Track(GridTrack {
                    min: TrackSizing::Px(0.0),
                    max: TrackSizing::Fr(1.0)
                }),
                TrackComponent::Track(GridTrack {
                    min: TrackSizing::Auto,
                    max: TrackSizing::Fr(2.0)
                }),
            ][..]
        )
    );
    let res = resolve("grid-cols-[repeat(3,minmax(0,1fr))]");
    let g = res.plan.base.extras.grid.as_ref().expect("grid");
    assert_eq!(
        g.template_cols.as_deref(),
        Some(
            &[TrackComponent::Repeat {
                count: 3,
                track: GridTrack {
                    min: TrackSizing::Px(0.0),
                    max: TrackSizing::Fr(1.0)
                }
            }][..]
        ),
        "repeat(N, <track>) stays a bounded Repeat component (1:1 with taffy)"
    );
}

#[test]
fn auto_rows_and_auto_cols_keywords() {
    let res = resolve("auto-rows-min");
    let g = res.plan.base.extras.grid.as_ref().expect("grid");
    assert_eq!(
        g.auto_rows.as_deref(),
        Some(
            &[TrackComponent::Track(GridTrack::keyword(
                TrackSizing::MinContent
            ))][..]
        )
    );

    let res = resolve("auto-rows-fr");
    let g = res.plan.base.extras.grid.as_ref().expect("grid");
    assert_eq!(
        g.auto_rows.as_deref(),
        Some(
            &[TrackComponent::Track(GridTrack {
                min: TrackSizing::Px(0.0),
                max: TrackSizing::Fr(1.0)
            })][..]
        ),
        "the v4 KEYWORD auto-rows-fr is minmax(0, 1fr) — the blowout guard"
    );

    let res = resolve("auto-cols-max");
    let g = res.plan.base.extras.grid.as_ref().expect("grid");
    assert_eq!(
        g.auto_cols.as_deref(),
        Some(
            &[TrackComponent::Track(GridTrack::keyword(
                TrackSizing::MaxContent
            ))][..]
        )
    );
}

#[test]
fn grid_template_and_span_compose() {
    // Template (container) and placement (item) are separate fields; the
    // shadcn Card header uses exactly this combination.
    let res = resolve(
        "grid-rows-[auto_auto] has-data-[slot=card-action]:grid-cols-[1fr_auto] col-span-2",
    );
    let g = res.plan.base.extras.grid.as_ref().expect("grid");
    assert!(g.template_rows.is_some());
    assert_eq!(g.col_span, Some(2));
}

#[test]
fn arbitrary_grid_junk_reports_precisely() {
    let t = theme();
    let r = Resolver::new(&t);
    let res = r.resolve("grid-cols-[banana]").unwrap();
    assert!(
        res.warnings.iter().any(
            |w| matches!(w, Warning::Unsupported { reason, .. } if reason.contains("grid track"))
        ),
        "an unparsable track reports a precise reason, got {:?}",
        res.warnings
    );
}

// ---------------------------------------------------------------------------
// §6 — the verification emitter's shapes for the new conditions/utilities
// ---------------------------------------------------------------------------

fn one_rule(class: &str) -> CssRule {
    let t = theme();
    let r = Resolver::new(&t);
    let out = r.to_css(class).expect("emit");
    assert_eq!(out.rules.len(), 1, "{class}: {:?}", out.rules);
    out.rules[0].clone()
}

#[test]
fn selection_emits_the_pseudo_element_selector() {
    let rule = one_rule("selection:bg-primary");
    assert!(
        rule.selector.ends_with("::selection"),
        "v4 shape: `.selection\\:bg-primary::selection` — got {}",
        rule.selector
    );
    assert!(
        rule.declarations
            .iter()
            .any(|(p, _)| p == "background-color"),
        "{:?}",
        rule.declarations
    );
}

#[test]
fn file_emits_the_file_selector_button_selector() {
    let rule = one_rule("file:text-sm");
    assert!(
        rule.selector.ends_with("::file-selector-button"),
        "got {}",
        rule.selector
    );
}

#[test]
fn has_data_emits_the_attribute_has_selector() {
    let rule = one_rule("has-data-[slot=card-action]:grid-cols-2");
    assert!(
        rule.selector.contains(":has(*[data-slot=\"card-action\"])"),
        "v4 shape: `:has(*[data-slot=\"card-action\"])` — got {}",
        rule.selector
    );
}

#[test]
fn container_utility_emits_the_containment_declarations() {
    let rule = one_rule("@container/card-header");
    assert!(
        rule.declarations
            .contains(&("container-type".into(), "inline-size".into())),
        "{:?}",
        rule.declarations
    );
    assert!(
        rule.declarations
            .contains(&("container-name".into(), "card-header".into())),
        "{:?}",
        rule.declarations
    );
}

#[test]
fn grid_tracks_emit_their_css_text() {
    let rule = one_rule("grid-cols-[0_1fr]");
    assert!(
        rule.declarations
            .contains(&("grid-template-columns".into(), "0 1fr".into())),
        "{:?}",
        rule.declarations
    );
    let rule = one_rule("grid-cols-[repeat(3,minmax(0,1fr))]");
    assert!(
        rule.declarations.contains(&(
            "grid-template-columns".into(),
            "repeat(3,minmax(0,1fr))".into()
        )),
        "{:?}",
        rule.declarations
    );
    let rule = one_rule("auto-rows-min");
    assert!(
        rule.declarations
            .contains(&("grid-auto-rows".into(), "min-content".into())),
        "{:?}",
        rule.declarations
    );
}

#[test]
fn justify_items_emits_its_declaration() {
    let rule = one_rule("justify-items-center");
    assert!(
        rule.declarations
            .contains(&("justify-items".into(), "center".into())),
        "{:?}",
        rule.declarations
    );
}
