//! v0.7 plan algebra (AEOLUS_V0.7.md §1): `StylePlan::overlay` composes two
//! resolved plans — a component's base plan with a user-override plan — with
//! the same later-wins semantics `cn()` gives class strings, but AFTER
//! resolution: theme-independent, cache-friendly, diff-integrated.
//!
//! RED phase: written before the implementation; every test here failed to
//! compile (no `StylePlan::overlay`) when first run (evidence in
//! `reports/10-tdd-evidence/09-portable-plans.md`).

use aeolus_core::*;

fn theme() -> TailwindTheme {
    TailwindTheme::from_theme_css(shadcn_css::LIGHT).unwrap()
}

/// Resolve two class strings into plans in one step.
fn plans(base: &str, over: &str) -> (StylePlan, StylePlan) {
    let t = theme();
    let r = Resolver::new(&t);
    let a = r.resolve(base).unwrap().plan;
    let b = r.resolve(over).unwrap().plan;
    (a, b)
}

#[test]
fn base_slot_composes_later_wins() {
    let (base, over) = plans("p-4 bg-primary rounded-md", "p-8");
    let merged = base.overlay(&over);
    assert_eq!(
        merged.base.padding,
        Some(Padding {
            top: 32.0,
            right: 32.0,
            bottom: 32.0,
            left: 32.0,
            set: EdgeSet::ALL
        })
    );
    // Untouched slots survive from the base plan.
    assert!(merged.base.background.is_some(), "bg-primary survives");
    assert!(merged.base.corner_radius.is_some(), "rounded-md survives");
    // `self` is unchanged (overlay is by &self).
    let (base2, _) = plans("p-4 bg-primary rounded-md", "p-8");
    assert_ne!(base2, merged);
}

#[test]
fn layer_merge_by_condition() {
    let (base, over) = plans("bg-primary hover:bg-red-500", "px-2 hover:px-8");
    let merged = base.overlay(&over);
    assert_eq!(merged.layers.len(), 1, "one hover layer after merge");
    let hover = &merged.layers[0];
    assert_eq!(hover.condition, VariantCondition::Hover);
    // v0.8 set-mask semantics: the layer records WHICH edges px-8
    // addresses — {right, left} — with 0.0 placeholders elsewhere. See
    // tests/per_edge_layers.rs for the full contract.
    assert_eq!(
        hover.style.padding,
        Some(Padding {
            top: 0.0,
            right: 32.0,
            bottom: 0.0,
            left: 32.0,
            set: EdgeSet {
                top: false,
                right: true,
                bottom: false,
                left: true
            }
        })
    );
    assert!(hover.style.background.is_some(), "hover bg survives");

    // A layer present only in the override is appended.
    let (base, over) = plans("p-4", "hover:bg-red-500 focus:px-4");
    let merged = base.overlay(&over);
    let conds: Vec<_> = merged.layers.iter().map(|l| l.condition).collect();
    assert!(conds.contains(&VariantCondition::Hover));
    assert!(conds.contains(&VariantCondition::Focus));
    // Ascending rank order (the v4 cascade contract).
    let mut ranks: Vec<u8> = merged.layers.iter().map(|l| l.condition.rank()).collect();
    let sorted = ranks.clone();
    ranks.sort_unstable();
    assert_eq!(ranks, sorted, "layers stay rank-ordered after merge");
}

#[test]
fn responsive_and_dark_compose_per_bucket() {
    let (base, over) = plans("md:p-4 dark:bg-red-500", "md:p-8 lg:p-2 dark:px-2");
    let merged = base.overlay(&over);
    let md = merged.responsive.get(&Breakpoint::Md).expect("md bucket");
    assert_eq!(
        md.padding,
        Some(Padding {
            top: 32.0,
            right: 32.0,
            bottom: 32.0,
            left: 32.0,
            set: EdgeSet::ALL
        })
    );
    assert!(merged.responsive.contains_key(&Breakpoint::Lg));
    let dark = merged.dark.as_ref().expect("dark bucket");
    assert!(
        dark.padding.is_some(),
        "dark:px-2 lands in the merged dark bucket"
    );
    assert!(
        dark.background.is_some(),
        "dark:bg-red-500 survives the merge"
    );
}

#[test]
fn group_peer_and_rules_merge() {
    // group layers merge by id; rules merge by condition equality.
    let (base, over) = plans(
        "group-hover/menu:bg-red-500 data-[state=open]:p-4",
        "group-hover/menu:px-8 aria-[busy=true]:bg-blue-500",
    );
    let merged = base.overlay(&over);
    assert_eq!(
        merged.group.len(),
        1,
        "group-hover/menu merges, not duplicates"
    );
    // Legacy group labels are `{label}{state}` (resolve.rs Route::Group).
    assert_eq!(&*merged.group[0].0, "menuhover");
    assert!(merged.group[0].1.background.is_some());
    assert!(merged.group[0].1.padding.is_some());
    assert_eq!(merged.rules.len(), 2, "two distinct rule conditions");
    // The data-[state=open] rule keeps its style; aria rule is appended.
    let has_data = merged
        .rules
        .iter()
        .any(|r| r.style.padding.is_some() && r.style.background.is_none());
    assert!(has_data, "data rule survives");
}

#[test]
fn breakpoints_and_container() {
    let (base, over) = plans("p-4", "p-8");
    let merged = base.overlay(&over);
    // Breakpoint tables are theme-derived: the override's table wins.
    assert_eq!(merged.breakpoints, over.breakpoints);
    // `@md:` routes to `rules` since v0.2 (the container map is a legacy
    // opt-in bucket) — the overlay merges the two rules by Cond equality.
    let (base, over) = plans("@md:p-4", "@md:p-8");
    let merged = base.overlay(&over);
    assert_eq!(merged.rules.len(), 1, "same container condition merges");
    assert_eq!(
        merged.rules[0].style.padding,
        Some(Padding {
            top: 32.0,
            right: 32.0,
            bottom: 32.0,
            left: 32.0,
            set: EdgeSet::ALL
        })
    );
}

#[test]
fn overlay_is_diff_integrated() {
    // The diff between the merged plan and the base reports exactly the
    // override slots — the repaint/relayout lanes still work on composition.
    let (base, over) = plans("p-4 bg-primary", "p-8");
    let merged = base.overlay(&over);
    let d = plan::diff(&base.base, &merged.base);
    assert_eq!(d.changed, vec![Slot::Padding]);
    assert!(d.needs_layout(), "padding is geometric");
    let (_, over2) = plans("p-4", "bg-red-500");
    let merged2 = base.overlay(&over2);
    let d2 = plan::diff(&base.base, &merged2.base);
    assert_eq!(d2.changed, vec![Slot::Background]);
    assert!(!d2.needs_layout(), "bg-only change is cosmetic");
}

#[test]
fn overlay_with_empty_override_is_identity() {
    let (base, over) = plans("p-4 bg-primary hover:bg-red-500", "p-4");
    let merged = base.overlay(&over);
    // The override re-applies the same padding; every slot value is equal.
    assert_eq!(merged.base, base.base);
    assert_eq!(merged.layers.len(), base.layers.len());
}

#[test]
fn idempotent_overlay() {
    let (base, over) = plans("p-4 bg-primary hover:bg-red-500 md:px-2", "p-8 bg-red-500");
    let once = base.overlay(&over);
    let twice = once.overlay(&over);
    assert_eq!(once, twice, "overlay is idempotent (override dominates)");
}
