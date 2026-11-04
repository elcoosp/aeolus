//! Class-table integrity: every shipped v4 class string must parse through
//! the aeolus resolver, and the essential styling slots must land.

use aeolus_core::{Resolver, cn, shadcn_default};
use aeolus_shadcn::classes::*;

fn resolver() -> Resolver<'static> {
    let theme = shadcn_default().unwrap();
    Resolver::new(Box::leak(Box::new(theme)))
}

/// All v1 class strings parse without hard errors.
#[test]
fn all_class_strings_parse() {
    let resolver = resolver();
    let strings = [
        BUTTON_BASE.to_string(),
        button_classes(ButtonVariant::Default, ButtonSize::Default),
        button_classes(ButtonVariant::Destructive, ButtonSize::Sm),
        button_classes(ButtonVariant::Outline, ButtonSize::Lg),
        INPUT_BASE.to_string(),
        LABEL_BASE.to_string(),
        CARD.to_string(),
        CARD_HEADER.to_string(),
        CARD_TITLE.to_string(),
        CARD_DESCRIPTION.to_string(),
        CARD_CONTENT.to_string(),
        CARD_FOOTER.to_string(),
        badge_classes(BadgeVariant::Default),
        badge_classes(BadgeVariant::Outline),
        format!("{SEPARATOR_BASE} {SEPARATOR_HORIZONTAL}"),
        format!("{SEPARATOR_BASE} {SEPARATOR_VERTICAL}"),
        SKELETON_BASE.to_string(),
        alert_classes(AlertVariant::Default),
        alert_classes(AlertVariant::Destructive),
        ALERT_TITLE.to_string(),
        ALERT_DESCRIPTION.to_string(),
    ];
    for s in &strings {
        let r = resolver
            .resolve(s)
            .unwrap_or_else(|e| panic!("failed to parse {s:?}: {e:?}"));
        // Unresolvable classes must produce warnings, never panics.
        let _ = &r.warnings;
    }
}

#[test]
fn button_default_variant_resolves_essential_slots() {
    let resolver = resolver();
    let r = resolver
        .resolve(&button_classes(ButtonVariant::Default, ButtonSize::Default))
        .unwrap();
    let base = &r.plan.base;
    // bg-primary + text-primary-foreground + rounded-md + px-4 py-2 + h-9.
    assert!(base.background.is_some(), "bg-primary");
    assert!(base.content_color.is_some(), "text-primary-foreground");
    let pad = base.padding.unwrap();
    assert_eq!((pad.left, pad.top), (16.0, 8.0), "px-4 py-2");
    // h-9 → 36px height.
    let dims = base.dimensions.unwrap();
    assert!(matches!(
        dims.height,
        Some(aeolus_core::plan::SizedValue::Px(px)) if px == 36.0
    ));
    // hover:bg-primary/90 present as a layer.
    assert!(
        r.plan
            .layers
            .iter()
            .any(|l| l.condition == aeolus_core::plan::VariantCondition::Hover),
        "hover layer present"
    );
    // disabled:opacity-50 — opacity slot inside the disabled layer (dropped
    // by backends that lack opacity, but PARSED and carried).
    let disabled = r
        .plan
        .layers
        .iter()
        .find(|l| l.condition == aeolus_core::plan::VariantCondition::Disabled)
        .expect("disabled layer");
    assert!(
        disabled.style.opacity.is_some(),
        "disabled:opacity-50 parsed"
    );
}

#[test]
fn outline_button_uses_border_input() {
    let resolver = resolver();
    let r = resolver
        .resolve(&button_classes(ButtonVariant::Outline, ButtonSize::Default))
        .unwrap();
    assert!(r.plan.base.border_width.is_some(), "border");
    assert!(r.plan.base.shadow.is_some(), "shadow-xs");
    // Light mode: no explicit border color (CSS currentColor default).
    // Dark mode: `dark:border-input` sets it — and parses through.
    let dark = r.plan.dark.as_ref().expect("outline has dark overrides");
    assert!(dark.border_color.is_some(), "dark:border-input");
}

#[test]
fn destructive_button_targets_real_tokens() {
    // v4 destructive uses text-white (no destructive-foreground in v4 themes).
    let resolver = resolver();
    let r = resolver
        .resolve(&button_classes(
            ButtonVariant::Destructive,
            ButtonSize::Default,
        ))
        .unwrap();
    let base = &r.plan.base;
    let bg = base.background.unwrap().0;
    let destructive = resolver
        .resolve("bg-destructive")
        .unwrap()
        .plan
        .base
        .background
        .unwrap()
        .0;
    assert_eq!(bg, destructive, "bg-destructive");
    // dark:bg-destructive/60 layer exists.
    assert!(r.plan.dark.is_some(), "dark layer present");
}

#[test]
fn skeleton_uses_accent_token() {
    let resolver = resolver();
    let r = resolver.resolve(SKELETON_BASE).unwrap();
    let bg = r.plan.base.background.unwrap().0;
    let accent = resolver
        .resolve("bg-accent")
        .unwrap()
        .plan
        .base
        .background
        .unwrap()
        .0;
    assert_eq!(
        bg, accent,
        "skeleton bg-accent resolves to the theme accent"
    );
}

#[test]
fn user_class_wins_over_component_default() {
    let resolver = resolver();
    let merged = cn(&[
        &button_classes(ButtonVariant::Default, ButtonSize::Default),
        "bg-red-500",
    ]);
    let r = resolver.resolve(&merged).unwrap();
    let red = resolver
        .resolve("bg-red-500")
        .unwrap()
        .plan
        .base
        .background
        .unwrap()
        .0;
    assert_eq!(
        r.plan.base.background.unwrap().0,
        red,
        "user bg-red-500 wins"
    );
}
