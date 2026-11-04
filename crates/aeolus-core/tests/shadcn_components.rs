//! End-to-end tests: the real Shadcn v4 class strings (verbatim from
//! shadcn-ui/ui @ HEAD 2026-11, apps/v4/registry/new-york-v4/ui) resolve
//! into the expected `StylePlan` shapes.

use aeolus_core::plan::AlignItems;
use aeolus_core::*;
use std::sync::Arc;

const THEME: &str = include_str!("../data/shadcn-light.css");

fn theme() -> TailwindTheme {
    TailwindTheme::from_theme_css(THEME).unwrap()
}

/// Button base classes (button.tsx:7).
const BUTTON_BASE: &str = "inline-flex shrink-0 items-center justify-center gap-2 rounded-md text-sm font-medium whitespace-nowrap transition-all outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50";

/// Button `default` variant (button.tsx:11).
const BUTTON_DEFAULT_VARIANT: &str = "bg-primary text-primary-foreground hover:bg-primary/90";

#[test]
fn button_base_resolves() {
    let r = Resolver::new(&theme()).resolve(BUTTON_BASE).unwrap();
    let base = &r.plan.base;

    // inline-flex items-center justify-center
    let layout = base.layout.expect("layout set");
    assert_eq!(layout.display, Some(Display::InlineFlex));
    assert_eq!(layout.align_items, Some(AlignItems::Center));
    assert_eq!(layout.justify_content, Some(JustifyContent::Center));

    // gap-2 -> 8px
    assert_eq!(base.gap, Some(Gap(8.0)));
    // rounded-md -> calc(0.625rem - 2px) = 8px
    assert_eq!(
        base.corner_radius,
        Some(CornerRadius {
            top_left: 8.0,
            top_right: 8.0,
            bottom_right: 8.0,
            bottom_left: 8.0
        })
    );
    // text-sm -> 14px + line-height 20px
    let text = &base.text;
    assert!(
        text.contains(&(TextProperty::FontSize, TextValue::Px(14.0))),
        "text={text:?}"
    );
    assert!(
        text.contains(&(TextProperty::LineHeight, TextValue::Px(20.0))),
        "text={text:?}"
    );
    // font-medium -> 500
    assert!(text.contains(&(TextProperty::FontWeight, TextValue::Weight(500))));
    // outline-none
    assert_eq!(
        base.outline,
        Some(Outline {
            width: 0.0,
            color: None,
            none: true
        })
    );

    // focus-visible:border-ring + focus-visible:ring-[3px] + focus-visible:ring-ring/50
    let fv = r
        .plan
        .layers
        .iter()
        .find(|l| l.condition == VariantCondition::FocusVisible)
        .expect("focus-visible layer");
    assert_eq!(fv.style.ring.as_ref().map(|r| r.width), Some(3.0));
    let ring_color = fv
        .style
        .ring
        .as_ref()
        .and_then(|r| r.color)
        .expect("ring color");
    // ring-ring/50 -> 50% mix of --ring (oklch 0.708 0 0)
    assert!((ring_color.alpha - 0.5).abs() < 1e-4);
    assert!((ring_color.l - 0.708).abs() < 1e-3);

    // disabled:pointer-events-none + disabled:opacity-50
    let dis = r
        .plan
        .layers
        .iter()
        .find(|l| l.condition == VariantCondition::Disabled)
        .expect("disabled layer");
    assert_eq!(dis.style.pointer_events, Some(PointerEvents::None));
    assert_eq!(dis.style.opacity, Some(Opacity(0.5)));

    // transition-all is recognized-and-dropped with a warning (honesty rule)
    assert!(
        r.warnings
            .iter()
            .any(|w| matches!(w, Warning::Unsupported { class, .. } if class == "transition-all")),
        "warnings={:?}",
        r.warnings
    );
}

#[test]
fn button_default_variant_resolves() {
    let r = Resolver::new(&theme())
        .resolve(BUTTON_DEFAULT_VARIANT)
        .unwrap();
    assert_eq!(
        r.warnings,
        Vec::new(),
        "variant classes must be fully supported"
    );
    let base = &r.plan.base;
    let bg = base.background.expect("bg-primary");
    assert!((bg.0.l - 0.205).abs() < 1e-3, "primary L");
    let fg = base.content_color.expect("text-primary-foreground");
    assert!((fg.0.l - 0.985).abs() < 1e-3);

    // hover:bg-primary/90 -> same hue at 90% alpha
    let hover = r
        .plan
        .layers
        .iter()
        .find(|l| l.condition == VariantCondition::Hover)
        .expect("hover layer");
    let hbg = hover.style.background.expect("hover bg");
    assert_eq!(hbg.0.l, bg.0.l);
    assert!((hbg.0.alpha - 0.9).abs() < 1e-4);
}

/// Button `outline` variant (button.tsx:14-15) — exercises dark: + arbitrary.
#[test]
fn button_outline_variant() {
    let r = Resolver::new(&theme()).resolve("border bg-background shadow-xs hover:bg-accent hover:text-accent-foreground dark:bg-input/30").unwrap();
    let base = &r.plan.base;
    // border -> 1px on all edges
    assert_eq!(
        base.border_width,
        Some(BorderWidth {
            top: 1.0,
            right: 1.0,
            bottom: 1.0,
            left: 1.0
        })
    );
    assert_eq!(
        base.background.map(|b| b.0.l),
        Some(1.0),
        "bg-background = white"
    );
    assert!(base.shadow.is_some(), "shadow-xs");
    // hover:bg-accent
    let hover = r
        .plan
        .layers
        .iter()
        .find(|l| l.condition == VariantCondition::Hover)
        .expect("hover");
    assert!((hover.style.background.as_ref().unwrap().0.l - 0.97).abs() < 1e-3);
    assert!(hover.style.content_color.is_some());
    // dark:bg-input/30 lands in the dark bucket
    let dark = r.plan.dark.as_ref().expect("dark layer");
    let d = dark.background.expect("dark bg");
    // input = oklch(0.922 0 0) at 30%
    assert!((d.0.l - 0.922).abs() < 1e-3);
    assert!((d.0.alpha - 0.3).abs() < 1e-4);
}

/// Input base (input.tsx) — min-w-0, w-full, placeholder via arbitrary variants.
#[test]
fn input_base_resolves() {
    let r = Resolver::new(&theme()).resolve("h-9 w-full min-w-0 rounded-md border border-input bg-transparent px-3 py-1 text-base shadow-xs outline-none disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50 md:text-sm").unwrap();
    let base = &r.plan.base;
    assert_eq!(
        base.dimensions,
        Some(Dimensions {
            width: Some(SizedValue::Full),
            height: Some(SizedValue::Px(36.0))
        })
    );
    assert_eq!(
        base.min_dimensions,
        Some(Dimensions {
            width: Some(SizedValue::Px(0.0)),
            height: None
        })
    );
    assert_eq!(
        base.padding,
        Some(Padding {
            top: 4.0,
            right: 12.0,
            bottom: 4.0,
            left: 12.0
        })
    );
    let bi = base.border_color.expect("border-input");
    assert!((bi.0.l - 0.922).abs() < 1e-3);
    assert_eq!(base.background, Some(Background(Color::TRANSPARENT)));
    // md:text-sm -> responsive bucket
    let md = r.plan.responsive.get(&Breakpoint::Md).expect("md bucket");
    assert!(
        md.text
            .contains(&(TextProperty::FontSize, TextValue::Px(14.0)))
    );
    // file:/placeholder:/selection: are arbitrary/functional variants -> warnings
    // (the base string above omits them; assert no unexpected warnings)
    assert!(r.warnings.is_empty(), "warnings={:?}", r.warnings);
}

/// Label (label.tsx) — group-data / peer-disabled arbitrary variants warn.
#[test]
fn label_resolves_with_documented_gaps() {
    let r = Resolver::new(&theme()).resolve("flex items-center gap-2 text-sm leading-none font-medium select-none group-data-[disabled=true]:pointer-events-none peer-disabled:cursor-not-allowed peer-disabled:opacity-50").unwrap();
    let base = &r.plan.base;
    assert_eq!(
        base.layout.as_ref().and_then(|l| l.display),
        Some(Display::Flex)
    );
    assert_eq!(base.gap, Some(Gap(8.0)));
    assert!(
        base.text
            .contains(&(TextProperty::FontSize, TextValue::Px(14.0)))
    );
    assert!(
        base.text
            .contains(&(TextProperty::FontWeight, TextValue::Weight(500)))
    );
    assert!(
        base.text
            .contains(&(TextProperty::Select, TextValue::Keyword("select-none")))
    );
    // group-data-[disabled=true] warns (structural variant); peer-disabled
    // layers RESOLVE into the peer bucket.
    assert_eq!(r.warnings.len(), 1, "warnings={:?}", r.warnings);
    assert!(!r.plan.peer.is_empty(), "peer bucket populated");
}

/// Card (card.tsx) — flex flex-col gap-6 rounded-xl border py-6.
#[test]
fn card_base_resolves() {
    let r = Resolver::new(&theme())
        .resolve(
            "flex flex-col gap-6 rounded-xl border bg-card py-6 text-card-foreground shadow-sm",
        )
        .unwrap();
    let base = &r.plan.base;
    let layout = base.layout.unwrap();
    assert_eq!(layout.display, Some(Display::Flex));
    assert_eq!(layout.flex_direction, Some(FlexDirection::Column));
    assert_eq!(base.gap, Some(Gap(24.0)));
    assert_eq!(
        base.corner_radius,
        Some(CornerRadius {
            top_left: 14.0,
            top_right: 14.0,
            bottom_right: 14.0,
            bottom_left: 14.0
        })
    );
    assert_eq!(
        base.padding,
        Some(Padding {
            top: 24.0,
            right: 0.0,
            bottom: 24.0,
            left: 0.0
        })
    );
    assert!((base.background.as_ref().unwrap().0.l - 1.0).abs() < 1e-3);
    assert!(base.shadow.is_some());
}

/// Badge (badge.tsx) — rounded-full, w-fit, py-0.5.
#[test]
fn badge_base_resolves() {
    let r = Resolver::new(&theme()).resolve("inline-flex w-fit shrink-0 items-center justify-center gap-1 overflow-hidden rounded-full border border-transparent px-2 py-0.5 text-xs font-medium whitespace-nowrap").unwrap();
    let base = &r.plan.base;
    assert_eq!(
        base.dimensions,
        Some(Dimensions {
            width: Some(SizedValue::FitContent),
            height: None
        })
    );
    assert_eq!(
        base.corner_radius,
        Some(CornerRadius {
            top_left: 9999.0,
            top_right: 9999.0,
            bottom_right: 9999.0,
            bottom_left: 9999.0
        })
    );
    assert_eq!(
        base.padding,
        Some(Padding {
            top: 2.0,
            right: 8.0,
            bottom: 2.0,
            left: 8.0
        })
    );
    assert!(
        base.text
            .contains(&(TextProperty::FontSize, TextValue::Px(12.0)))
    );
    // overflow-hidden warns (outside v1 scope)
    assert!(
        r.warnings
            .iter()
            .any(|w| matches!(w, Warning::Unsupported { class, .. } if class == "overflow-hidden"))
    );
}

/// Separator (separator.tsx).
#[test]
fn separator_resolves() {
    let r = Resolver::new(&theme())
        .resolve("shrink-0 bg-border")
        .unwrap();
    assert_eq!(r.warnings, Vec::new());
    let bg = r.plan.base.background.expect("bg-border");
    assert!((bg.0.l - 0.922).abs() < 1e-3);
    assert_eq!(
        r.plan.base.layout.as_ref().and_then(|l| l.flex_shrink),
        Some(0.0)
    );
}

/// Skeleton (skeleton.tsx) — animate-pulse is a documented no-op.
#[test]
fn skeleton_resolves() {
    let r = Resolver::new(&theme())
        .resolve("animate-pulse rounded-md bg-accent")
        .unwrap();
    let base = &r.plan.base;
    assert_eq!(
        base.corner_radius,
        Some(CornerRadius {
            top_left: 8.0,
            top_right: 8.0,
            bottom_right: 8.0,
            bottom_left: 8.0
        })
    );
    assert!((base.background.as_ref().unwrap().0.l - 0.97).abs() < 1e-3);
    assert!(
        r.warnings
            .iter()
            .any(|w| matches!(w, Warning::Unsupported { class, .. } if class == "animate-pulse"))
    );
}

/// Alert (alert.tsx) — relative + grid-cols warn; bg-card/text-card-foreground resolve.
#[test]
fn alert_resolves() {
    let r = Resolver::new(&theme()).resolve("relative grid w-full grid-cols-[0_1fr] items-start gap-y-0.5 rounded-lg border px-4 py-3 text-sm").unwrap();
    let base = &r.plan.base;
    assert_eq!(
        base.layout.as_ref().and_then(|l| l.display),
        Some(Display::Grid)
    );
    assert_eq!(
        base.dimensions,
        Some(Dimensions {
            width: Some(SizedValue::Full),
            height: None
        })
    );
    assert_eq!(
        base.corner_radius,
        Some(CornerRadius {
            top_left: 10.0,
            top_right: 10.0,
            bottom_right: 10.0,
            bottom_left: 10.0
        })
    );
    assert_eq!(
        base.padding,
        Some(Padding {
            top: 12.0,
            right: 16.0,
            bottom: 12.0,
            left: 16.0
        })
    );
    assert!(
        base.text
            .contains(&(TextProperty::FontSize, TextValue::Px(14.0)))
    );
    // relative / grid-cols-[0_1fr] / gap-y warn
    assert_eq!(r.warnings.len(), 3, "warnings={:?}", r.warnings);
}

/// `cn()` conflict resolution composes with shadcn usage: user classes win.
#[test]
fn cn_composes_button() {
    let merged = cn(&[
        "inline-flex items-center justify-center gap-2 rounded-md text-sm",
        "bg-primary text-primary-foreground hover:bg-primary/90",
        "px-8 py-8 rounded-full", // user overrides
    ]);
    assert!(
        merged.contains("rounded-full"),
        "user radius wins: {merged}"
    );
    assert!(
        !merged.contains("rounded-md"),
        "base radius removed: {merged}"
    );
    assert!(merged.contains("px-8"), "user padding wins: {merged}");
    assert!(merged.contains("py-8"));
    assert!(!merged.contains("items-center justify-center gap-2 rounded-md text-sm bg-primary text-primary-foreground hover:bg-primary/90px-8"));
}

/// Unknown utilities produce did-you-mean suggestions (DX requirement).
#[test]
fn diagnostics_suggest() {
    let r = Resolver::new(&theme()).resolve("bg-primar p-4").unwrap();
    assert_eq!(r.plan.base.padding.as_ref().unwrap().top, 16.0);
    match &r.warnings[0] {
        Warning::Unknown { class, suggestion } => {
            assert_eq!(class, "bg-primar");
            assert_eq!(
                suggestion.as_deref(),
                Some("primary"),
                "did-you-mean primary"
            );
        }
        other => panic!("expected Unknown warning, got {other:?}"),
    }
}

/// The full cache path: steady state is one hash lookup.
#[test]
fn plan_cache_hits() {
    let cache = PlanCache::new();
    let theme = theme();
    let a = cache
        .resolve(&theme, "p-4 bg-primary hover:bg-primary/90")
        .unwrap();
    let b = cache
        .resolve(&theme, "p-4 bg-primary hover:bg-primary/90")
        .unwrap();
    assert!(Arc::ptr_eq(&a, &b), "second resolve must hit the cache");
    assert_eq!(cache.len(), 1);
    // theme version bump invalidates
    let dark = theme
        .dark_variant(".dark { --background: oklch(0.145 0 0); }")
        .unwrap();
    let c = cache
        .resolve(&dark, "p-4 bg-primary hover:bg-primary/90")
        .unwrap();
    assert!(!Arc::ptr_eq(&a, &c));
    assert_eq!(cache.len(), 2);
}

/// Negative margins and the important flag parse.
#[test]
fn negatives_and_important() {
    let r = Resolver::new(&theme()).resolve("-mt-2 p-4!").unwrap();
    assert_eq!(
        r.plan.base.margin,
        Some(Margin {
            top: -8.0,
            right: 0.0,
            bottom: 0.0,
            left: 0.0
        })
    );
    assert_eq!(r.plan.base.padding.map(|p| p.top), Some(16.0));
    assert_eq!(r.plan.base.padding.map(|p| p.left), Some(16.0));
}
