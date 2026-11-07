//! The oracle-fidelity headline (v0.7, plan `AEOLUS_V0.7.md` §7).
//!
//! The shadcn v4 class tables shipped in `crates/aeolus-shadcn` ARE the
//! project's oracle ("the tables are the oracle; fix the resolver" —
//! CONTRIBUTING.md). After v0.7 every parser/IR gap they expose is closed:
//! resolving them yields warnings ONLY from the documented v0.3 §1.3
//! Probe-model boundary — selectors that style OTHER elements
//! (`[&_svg]`, `[&>svg]`, `*:data-…`) and class-identity self-tests
//! ([`.border-b`]) — never from a missing variant/utility.

use aeolus_core::*;
use plan::Slot;

fn theme() -> TailwindTheme {
    TailwindTheme::from_theme_css(shadcn_css::LIGHT).unwrap()
}

/// The only warnings allowed on the shipped tables after v0.7: arbitrary
/// selectors that style other elements and class/id self-tests — the
/// documented v0.3 §1.3 boundary of the Probe model.
fn is_documented_boundary(w: &Warning) -> bool {
    match w {
        Warning::Unsupported { class, reason } => {
            let class = class.as_str();
            let element_targeting = class.starts_with("[&_")
                || class.starts_with("[&>")
                || class.starts_with("[&:")
                || class.starts_with("*:")
                || class.starts_with("[.")
                || class.starts_with("has-[>");
            element_targeting
                && (reason.contains("styles other elements")
                    || reason.contains("class/id selector tests")
                    || reason.contains("has-[selector]"))
        }
        _ => false,
    }
}

const ORACLE_TABLES: &[&str] = &[
    "BUTTON_BASE",
    "INPUT_BASE",
    "LABEL_BASE",
    "CARD",
    "CARD_HEADER",
    "CARD_TITLE",
    "CARD_DESCRIPTION",
    "CARD_CONTENT",
    "CARD_FOOTER",
    "BADGE_BASE",
    "SEPARATOR_BASE",
    "SEPARATOR_HORIZONTAL",
    "SEPARATOR_VERTICAL",
    "SKELETON_BASE",
    "ALERT_BASE",
    "ALERT_TITLE",
    "ALERT_DESCRIPTION",
];

fn oracle_string(name: &str) -> String {
    use aeolus_shadcn::classes as c;
    match name {
        "BUTTON_BASE" => c::BUTTON_BASE.to_string(),
        "INPUT_BASE" => c::INPUT_BASE.to_string(),
        "LABEL_BASE" => c::LABEL_BASE.to_string(),
        "CARD" => c::CARD.to_string(),
        "CARD_HEADER" => c::CARD_HEADER.to_string(),
        "CARD_TITLE" => c::CARD_TITLE.to_string(),
        "CARD_DESCRIPTION" => c::CARD_DESCRIPTION.to_string(),
        "CARD_CONTENT" => c::CARD_CONTENT.to_string(),
        "CARD_FOOTER" => c::CARD_FOOTER.to_string(),
        "BADGE_BASE" => c::BADGE_BASE.to_string(),
        "SEPARATOR_BASE" => c::SEPARATOR_BASE.to_string(),
        "SEPARATOR_HORIZONTAL" => c::SEPARATOR_HORIZONTAL.to_string(),
        "SEPARATOR_VERTICAL" => c::SEPARATOR_VERTICAL.to_string(),
        "SKELETON_BASE" => c::SKELETON_BASE.to_string(),
        "ALERT_BASE" => c::ALERT_BASE.to_string(),
        "ALERT_TITLE" => c::ALERT_TITLE.to_string(),
        "ALERT_DESCRIPTION" => c::ALERT_DESCRIPTION.to_string(),
        other => panic!("unknown oracle table {other}"),
    }
}

/// Captures the fidelity ledger for the release notes (40 warnings across
/// the shipped tables before v0.7 — every one a parser/IR gap).
#[test]
fn fidelity_ledger() {
    let t = theme();
    let r = Resolver::new(&t);
    let mut total = 0;
    let mut boundary = 0;
    for name in ORACLE_TABLES {
        let res = r.resolve(&oracle_string(name)).unwrap();
        for w in &res.warnings {
            total += 1;
            if is_documented_boundary(w) {
                boundary += 1;
            } else {
                panic!("non-boundary warning in {name}: {w:?}");
            }
        }
    }
    println!(
        "FIDELITY LEDGER: {total} warnings remain, all documented boundary ({boundary} boundary)"
    );
    assert!(total == boundary);
}

#[test]
fn shipped_shadcn_tables_resolve_warning_free_up_to_the_boundary() {
    let t = theme();
    let r = Resolver::new(&t);
    let mut violations = Vec::new();
    for name in ORACLE_TABLES {
        let s = oracle_string(name);
        let res = r.resolve(&s).unwrap();
        for w in &res.warnings {
            if !is_documented_boundary(w) {
                violations.push(format!("{name}: {w:?}"));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "oracle-fidelity violated — these warnings are parser/IR gaps, not documented boundaries:\n{}",
        violations.join("\n")
    );
}

/// The variant/axis compositions of the shipped components (base + variant
/// + size + user classes) resolve with the same contract.
#[test]
fn component_compositions_resolve_warning_free_up_to_the_boundary() {
    use aeolus_shadcn::classes::*;
    let t = theme();
    let r = Resolver::new(&t);
    let strings = [
        button_classes(ButtonVariant::Default, ButtonSize::Default),
        button_classes(ButtonVariant::Destructive, ButtonSize::Lg),
        button_classes(ButtonVariant::Outline, ButtonSize::Icon),
        badge_classes(BadgeVariant::Default),
        badge_classes(BadgeVariant::Outline),
        alert_classes(AlertVariant::Default),
        alert_classes(AlertVariant::Destructive),
    ];
    for s in &strings {
        let res = r.resolve(s).unwrap();
        for w in &res.warnings {
            assert!(
                is_documented_boundary(w),
                "composition `{s}` produced a non-boundary warning: {w:?}"
            );
        }
    }
}

/// Fidelity is not just warning-free: the NEW typed rules must actually
/// carry the styles the tables ask for — the Input's selection/placeholder
/// colors land as probe-answered rules, and the Card header's grid template
/// lands in the plan.
#[test]
fn new_rules_carry_the_requested_styles() {
    let t = theme();
    let r = Resolver::new(&t);
    // Input: selection:text-primary-foreground → a Selection rule with the
    // text color; placeholder:text-muted-foreground → a Placeholder rule.
    let res = r.resolve(aeolus_shadcn::classes::INPUT_BASE).unwrap();
    let has_selection = res.plan.rules.iter().any(|rule| {
        rule.when.0.contains(&cond::Atom::On(cond::Test::Pseudo(
            ast::PseudoElement::Selection,
        )))
    });
    let has_placeholder = res.plan.rules.iter().any(|rule| {
        rule.when.0.contains(&cond::Atom::On(cond::Test::Pseudo(
            ast::PseudoElement::Placeholder,
        )))
    });
    assert!(has_selection, "input ships a ::selection rule");
    assert!(has_placeholder, "input ships a ::placeholder rule");
    // Card header: grid-rows-[auto_auto] → the template tracks.
    let res = r.resolve(aeolus_shadcn::classes::CARD_HEADER).unwrap();
    let g = res
        .plan
        .base
        .extras
        .grid
        .as_ref()
        .expect("card header grid");
    assert!(g.template_rows.is_some(), "grid-rows-[auto_auto] lands");
    assert!(g.auto_rows.is_some(), "auto-rows-min lands");
    assert_eq!(
        g.cols, None,
        "card header has no numeric grid-cols; the conditional one is a template"
    );
    // The container declaration lands with its name.
    assert_eq!(
        res.plan.base.extras.container_type,
        Some(plan::ContainerType {
            name: Some("card-header".into())
        }),
        "@container/card-header lands"
    );
    // Slot coverage: the new slots are enumerable for diffing/capabilities.
    let slots = res.plan.base.slots();
    assert!(
        slots.contains(&Slot::ContainerType),
        "container slot present"
    );
}
