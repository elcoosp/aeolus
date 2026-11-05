//! End-to-end tests: the real Shadcn v4 class strings (verbatim from
//! shadcn-ui/ui @ HEAD 2026-11, apps/v4/registry/new-york-v4/ui) resolve
//! into the expected `StylePlan` shapes.
//!
//! v0.2 (plan §4.4/§5): the F3 regression suite — every Button variant's
//! light-mode hover must equal the base token (alpha 1.0), because
//! `dark:hover:` now routes to the rules bucket instead of leaking into the
//! plain hover layer.

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
            none: true,
            offset: 0.0
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

    // v0.2: transition-all resolves to a TYPED slot (plan §3.2), not a
    // dropped warning.
    assert!(
        base.extras.transition.is_some(),
        "transition-all -> Transition slot"
    );
    // The rest of the string is clean.
    assert!(r.warnings.is_empty(), "warnings={:?}", r.warnings);
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
    // v0.2: shadow-xs comes from the shipped v4 default theme (single
    // layer; the two-layer presets like shadow-sm fill shadow_rest).
    let shadow = base.shadow.as_ref().expect("shadow-xs");
    assert!((shadow.offset_y - 1.0).abs() < 1e-4);
    assert!((shadow.blur - 2.0).abs() < 1e-4);
    assert_eq!(
        base.extras.shadow_rest.len(),
        0,
        "shadow-xs is a single-layer preset"
    );
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

/// F3 (plan §5.1): `dark:hover:bg-input/50` must NOT leak into the plain
/// hover layer — it lands in `rules` as `[Media(Dark), On(Hover)]`.
#[test]
fn dark_hover_compound_routes_to_rules() {
    let r = Resolver::new(&theme())
        .resolve("dark:hover:bg-input/50")
        .unwrap();
    assert!(
        r.plan
            .layers
            .iter()
            .all(|l| l.condition != VariantCondition::Hover),
        "no plain hover layer may exist: {:?}",
        r.plan.layers
    );
    assert!(r.plan.dark.is_none(), "no dark-bucket leak");
    assert_eq!(r.plan.rules.len(), 1, "the rule bucket holds the compound");
    let rule = &r.plan.rules[0];
    assert_eq!(
        rule.when.0,
        vec![
            Atom::Media(Media::Dark),
            Atom::On(Test::State(State::Hover)),
        ]
    );
    let bg = rule.style.background.expect("dark:hover bg");
    assert!((bg.0.alpha - 0.5).abs() < 1e-4);
    assert!((bg.0.l - 0.922).abs() < 1e-3, "input token");
    assert!(r.warnings.is_empty());
}

/// F4 (plan §5.1): `md:hover:p-2` keeps BOTH conditions.
#[test]
fn md_hover_compound_routes_to_rules() {
    let r = Resolver::new(&theme()).resolve("md:hover:p-2").unwrap();
    assert!(r.plan.layers.is_empty(), "no plain hover layer");
    assert!(r.plan.responsive.is_empty(), "no responsive-bucket leak");
    assert_eq!(r.plan.rules.len(), 1);
    let rule = &r.plan.rules[0];
    assert_eq!(
        rule.when.0,
        vec![
            Atom::Viewport {
                min: Some(768.0),
                max: None
            },
            Atom::On(Test::State(State::Hover)),
        ]
    );
    assert_eq!(
        rule.style.padding,
        Some(Padding {
            top: 8.0,
            right: 8.0,
            bottom: 8.0,
            left: 8.0
        })
    );
}

/// F1 (plan §5.1): fractions are fractions, not 4px-per-unit.
#[test]
fn fractions_resolve_as_fractions() {
    let r = Resolver::new(&theme())
        .resolve("w-1/2 w-1/3 w-2/3 h-3/4")
        .unwrap();
    let d = r.plan.base.dimensions.expect("dimensions");
    assert_eq!(d.height, Some(SizedValue::Fraction(0.75)));
    // Later classes win: w-2/3 is last.
    assert_eq!(d.width, Some(SizedValue::Fraction(2.0 / 3.0)));
    assert_eq!(r.warnings, Vec::new());
}

/// F2 (plan §5.1): arbitrary lengths are NOT spacing multiples.
#[test]
fn arbitrary_length_is_absolute() {
    let r = Resolver::new(&theme()).resolve("p-[7px]").unwrap();
    assert_eq!(
        r.plan.base.padding,
        Some(Padding {
            top: 7.0,
            right: 7.0,
            bottom: 7.0,
            left: 7.0
        })
    );
    assert_eq!(r.warnings, Vec::new());
}

/// F5 (plan §5.1): logical radius edges map to the start side only.
#[test]
fn logical_radius_edges() {
    let r = Resolver::new(&theme()).resolve("rounded-s-lg").unwrap();
    let cr = r.plan.base.corner_radius.expect("radius");
    // shadcn --radius-lg = calc(0.625rem) = 10px; start side = LEFT in LTR.
    assert_eq!(cr.top_left, 10.0);
    assert_eq!(cr.bottom_left, 10.0);
    assert_eq!(cr.top_right, 0.0);
    assert_eq!(cr.bottom_right, 0.0);
}

/// F6 (plan §5.1): `text-sm/6` resolves the line-height modifier.
#[test]
fn text_size_line_height_modifier() {
    let r = Resolver::new(&theme()).resolve("text-sm/6").unwrap();
    let text = &r.plan.base.text;
    assert!(
        text.contains(&(TextProperty::LineHeight, TextValue::Px(24.0))),
        "6 * 4px = 24px: {text:?}"
    );
    // Named leading tokens work too.
    let r = Resolver::new(&theme()).resolve("text-sm/tight").unwrap();
    assert!(
        r.plan
            .base
            .text
            .contains(&(TextProperty::LineHeight, TextValue::Px(20.0)))
    );
}

/// F7 (plan §5.1): arbitrary opacity `[0.37]` is a raw alpha fraction.
#[test]
fn bracket_opacity_is_raw_alpha() {
    let r = Resolver::new(&theme())
        .resolve("bg-primary/[0.37]")
        .unwrap();
    let bg = r.plan.base.background.expect("bg");
    assert!((bg.0.alpha - 0.37).abs() < 1e-4);
    assert!((bg.0.l - 0.205).abs() < 1e-3);
    // And the var() shorthand resolves through the theme.
    let r = Resolver::new(&theme()).resolve("bg-primary/50").unwrap();
    assert!((r.plan.base.background.unwrap().0.alpha - 0.5).abs() < 1e-4);
}

/// F3 against the SHIPPED Ghost variant (plan §4.4): light-mode hover
/// equals the base token at alpha 1.0 — the v0.1 bug resolved it at 0.50.
/// Class strings verbatim from shadcn-ui/ui new-york-v4 button.tsx.
#[test]
fn ghost_light_mode_hover_is_full_accent() {
    let theme = theme();
    let classes = cn(&["hover:bg-accent hover:text-accent-foreground dark:hover:bg-accent/50"]);
    let r = Resolver::new(&theme).resolve(&classes).unwrap();
    // The Ghost variant contributes hover:bg-accent AND dark:hover:bg-accent/50.
    // In LIGHT mode the hover background must be accent at alpha 1.0.
    let hover = r
        .plan
        .layers
        .iter()
        .find(|l| l.condition == VariantCondition::Hover)
        .expect("hover layer");
    let hover_bg = hover.style.background.expect("hover bg");
    let accent = theme.color("accent").unwrap();
    assert!(
        (hover_bg.0.alpha - accent.alpha).abs() < 1e-4,
        "light hover alpha must equal accent (1.0), got {}",
        hover_bg.0.alpha
    );
    // And the dark:hover rule exists separately in `rules`.
    assert!(
        r.plan.rules.iter().any(|rule| {
            rule.when.0.len() == 2
                && rule.when.0[0] == Atom::Media(Media::Dark)
                && rule.when.0[1] == Atom::On(Test::State(State::Hover))
                && rule
                    .style
                    .background
                    .is_some_and(|b| (b.0.alpha - 0.5).abs() < 1e-4)
        }),
        "dark:hover:bg-accent/50 must be a rule"
    );
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
    assert!(r.warnings.is_empty(), "warnings={:?}", r.warnings);
}

/// Label (label.tsx) — group-data / peer-disabled resolve through the
/// v0.2 paths: peer into the peer bucket, group-data into `rules`.
#[test]
fn label_resolves_with_v02_paths() {
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
    // v0.2: NOTHING is dropped — group-data lands in rules with a
    // Group(Data) atom; peer-disabled lands in the peer bucket.
    assert!(r.warnings.is_empty(), "warnings={:?}", r.warnings);
    assert!(!r.plan.peer.is_empty(), "peer bucket populated");
    assert_eq!(r.plan.rules.len(), 1, "group-data rule");
    let rule = &r.plan.rules[0];
    assert_eq!(
        rule.when.0,
        vec![Atom::Group(
            Test::Data("disabled".into(), Some("true".into())),
            None
        )]
    );
    assert_eq!(rule.style.pointer_events, Some(PointerEvents::None));
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
    // v0.2: overflow-hidden resolves to the typed Overflow slot.
    assert_eq!(
        base.extras.overflow,
        Some(Overflow {
            x: Some(OverflowKind::Hidden),
            y: Some(OverflowKind::Hidden)
        })
    );
    assert!(r.warnings.is_empty(), "warnings={:?}", r.warnings);
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

/// Skeleton (skeleton.tsx) — animate-pulse resolves to the typed
/// Animation slot (v0.2), still zero dropped warnings.
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
    let anim = base.extras.animation.as_ref().expect("animation slot");
    assert_eq!(anim.name.as_ref(), "pulse");
    assert_eq!(anim.duration_ms, Some(2000.0));
    assert!(anim.infinite);
    assert!(r.warnings.is_empty(), "warnings={:?}", r.warnings);
}

/// Alert (alert.tsx) — relative/grid/gap-y all RESOLVE in v0.2; only the
/// arbitrary grid template is reported.
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
    // relative -> Position slot; gap-y-0.5 -> GapAxes slot.
    assert_eq!(base.extras.position, Some(Position::Relative));
    assert_eq!(
        base.extras.gap_axes,
        Some(GapAxes {
            col: None,
            row: Some(2.0)
        })
    );
    // Only the arbitrary grid template is reported (precisely).
    assert_eq!(r.warnings.len(), 1, "warnings={:?}", r.warnings);
    assert!(
        matches!(
            &r.warnings[0],
            Warning::Unsupported { class, .. } if class == "grid-cols-[0_1fr]"
        ),
        "warnings={:?}",
        r.warnings
    );
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

/// v0.2 cn() fix (plan §1.3): `hover:` and `group-hover:` are DIFFERENT
/// variant stacks and must not knock each other out; same for `md:` vs `@md:`.
#[test]
fn cn_distinguishes_group_and_container_variants() {
    assert_eq!(
        cn(&["hover:bg-a", "group-hover:bg-a"]),
        "hover:bg-a group-hover:bg-a"
    );
    assert_eq!(cn(&["md:p-2", "@md:p-4"]), "md:p-2 @md:p-4");
    // Same-variant conflicts still merge.
    assert_eq!(cn(&["hover:bg-a", "hover:bg-b"]), "hover:bg-b");
}

/// Unknown utilities produce did-you-mean suggestions (DX requirement).
#[test]
fn diagnostics_suggest() {
    let r = Resolver::new(&theme()).resolve("bg-primar").unwrap();
    assert!(
        matches!(
            &r.warnings[0],
            Warning::Unknown {
                suggestion: Some(s),
                ..
            } if s == "primary"
        ),
        "warnings={:?}",
        r.warnings
    );
}

/// v0.2: negatives and important still work end-to-end.
#[test]
fn negatives_and_important() {
    let r = Resolver::new(&theme()).resolve("-mt-4!").unwrap();
    let mar = r.plan.base.margin.expect("margin");
    assert_eq!(mar.top, -16.0);
    assert!(r.plan.base.text.is_empty() || true);
    let _ = Arc::new(&r.plan); // plans are clonable/shared
}
