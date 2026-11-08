//! Lowering correctness: the generated `PropertyStack` must equal a
//! hand-written stack for the demo classes, and `cn()` must resolve
//! conflicts rightmost-wins.

use masonry::core::{PropertySet, PropertyStack, Selector};
use masonry::layout::Length;
use masonry::properties::{Background, BorderColor, CornerRadius, Padding};

use aeolus_core::color::Color;
use aeolus_core::{Resolver, shadcn_default};
use aeolus_xilem::{lower, to_masonry_color};

/// Hand-written Masonry stack for `p-4 bg-primary hover:bg-primary/90 rounded-md`
/// with the shadcn theme — the "what the developer would have written" oracle.
fn handwritten() -> PropertyStack {
    let primary = Color::oklch(0.205, 0.0, 0.0, 1.0);
    let mut stack = PropertyStack::new();
    // Base: bg-primary + p-4 + rounded-md (radius 0.625rem - 2px = 8px).
    let mut base = PropertySet::new();
    base.insert(Background::Color(to_masonry_color(primary)));
    base.insert(Padding::all(Length::px(16.0)));
    base.insert(CornerRadius::all(Length::px(8.0)));
    stack.push_layer(Selector::new(), base);
    // Hover: bg-primary/90 — same hue, alpha 0.9 (oklab color-mix semantics).
    let hover_bg = Color::oklch(0.205, 0.0, 0.0, 0.9);
    stack.push_layer(
        Selector::new().with_hovered(true),
        PropertySet::one(Background::Color(to_masonry_color(hover_bg))),
    );
    stack
}

#[test]
fn lower_matches_handwritten() {
    let theme = shadcn_default().unwrap();
    let resolver = Resolver::new(&theme);
    let resolution = resolver
        .resolve("p-4 bg-primary hover:bg-primary/90 rounded-md")
        .unwrap();
    let (stack, report) = lower(&resolution.plan);
    assert!(report.dropped.is_empty(), "unexpected drops: {report:?}");

    let expected = handwritten();
    let got = stack.get_layers();
    let want = expected.get_layers();
    assert_eq!(got.len(), want.len(), "layer count differs");

    for (i, ((gs, gp), (ws, wp))) in got.iter().zip(want.iter()).enumerate() {
        assert_eq!(gs, ws, "layer {i}: selector differs");
        assert_eq!(
            gp.get::<Background>().map(to_rgba),
            wp.get::<Background>().map(to_rgba),
            "layer {i}: Background differs"
        );
        assert_eq!(
            gp.get::<Padding>(),
            wp.get::<Padding>(),
            "layer {i}: Padding differs"
        );
        assert_eq!(
            gp.get::<CornerRadius>(),
            wp.get::<CornerRadius>(),
            "layer {i}: CornerRadius differs"
        );
        assert_eq!(
            gp.get::<BorderColor>(),
            wp.get::<BorderColor>(),
            "layer {i}: BorderColor differs"
        );
    }
}

fn to_rgba(bg: &Background) -> Option<masonry::peniko::color::Rgba8> {
    match bg {
        Background::Color(c) => Some(c.to_rgba8()),
        Background::Gradient(_) => None,
    }
}

#[test]
fn cn_rightmost_wins() {
    // px-2 (8px) then px-4 (16px): the rightmost class must win the conflict.
    let merged = aeolus_core::cn(&["px-2", "px-4"]);
    let theme = shadcn_default().unwrap();
    let resolver = Resolver::new(&theme);
    let resolution = resolver.resolve(&merged).unwrap();
    let padding = resolution.plan.base.padding.unwrap();
    assert_eq!(
        (padding.left, padding.right),
        (16.0, 16.0),
        "cn({merged:?}) must keep px-4"
    );
}

#[test]
fn hover_alpha_is_color_mix_semantics() {
    // Tailwind v4 `bg-primary/90` = color-mix(in oklab, primary 90%, transparent):
    // chroma AND alpha scale by 0.9; for an achromatic token alpha is the
    // observable change.
    let theme = shadcn_default().unwrap();
    let resolver = Resolver::new(&theme);
    let r = resolver.resolve("bg-primary hover:bg-primary/90").unwrap();
    let base = r.plan.base.background.unwrap().0;
    let hover = r.plan.layers[0].style.background.unwrap().0;
    assert!((base.alpha - 1.0).abs() < 1e-4);
    assert!((hover.alpha - 0.9).abs() < 1e-4);
    assert_eq!(hover.a, base.a);
    assert_eq!(hover.b, base.b);
}

#[test]
fn focus_and_disabled_layers_map_to_selectors() {
    let theme = shadcn_default().unwrap();
    let resolver = Resolver::new(&theme);
    let r = resolver
        .resolve(
            "bg-primary focus:bg-red-500 active:bg-black disabled:bg-gray-300 focus-within:ring-2",
        )
        .unwrap();
    let (stack, report) = lower(&r.plan);
    let selectors: Vec<_> = stack.get_layers().iter().map(|(s, _)| s.clone()).collect();
    assert!(selectors.contains(&Selector::new().with_focused(true)));
    assert!(selectors.contains(&Selector::new().with_active(true)));
    assert!(selectors.contains(&Selector::new().with_disabled(true)));
    // focus-within has no Selector flag today: it must be REPORTED, not dropped.
    assert!(
        report
            .dropped
            .iter()
            .any(|(what, _)| what == "focus-within"),
        "focus-within must be reported: {report:?}"
    );
}

#[test]
fn unrepresentable_conditions_are_reported() {
    let theme = shadcn_default().unwrap();
    let resolver = Resolver::new(&theme);
    let r = resolver
        .resolve("bg-primary group-hover:bg-red-500 peer-focus:text-primary dark:bg-black md:p-8 @lg:p-12")
        .unwrap();
    let (_, report) = lower(&r.plan);
    let whats: Vec<_> = report.dropped.iter().map(|(w, _)| w.as_str()).collect();
    assert!(whats.iter().any(|w| w.starts_with("group:")), "{report:?}");
    assert!(whats.iter().any(|w| w.starts_with("peer:")), "{report:?}");
    assert!(whats.contains(&"dark"), "{report:?}");
    assert!(
        whats.iter().any(|w| w.starts_with("responsive:")),
        "{report:?}"
    );
    // v0.2: `@lg` routes into `rules` (evaluated by StyledWidget's probe)
    // instead of the legacy container bucket — nothing is dropped.
    assert!(!r.plan.container.is_empty() || !r.plan.rules.is_empty());
    assert!(
        r.plan.rules.iter().any(|rule| matches!(
            rule.when.0.first(),
            Some(aeolus_core::Atom::Container { .. })
        )),
        "the @lg rule must be a Container atom: {:?}",
        r.plan.rules
    );
}

#[test]
fn default_palette_available() {
    // The v4 default palette ships with every theme; user tokens override it.
    let theme = shadcn_default().unwrap();
    assert!(theme.color("black").is_some());
    assert!(theme.color("blue-500").is_some());
    assert!(theme.color("red-500").is_some());
    // shadcn token wins over any palette entry of the same name.
    let primary = theme.color("primary").unwrap();
    assert!((primary.l - 0.205).abs() < 1e-3);
    // Percentage lightness form parses: oklch(63.7% 0.237 25.331).
    let red = theme.color("red-500").unwrap();
    assert!((red.l - 0.637).abs() < 1e-3);
}

#[test]
fn shadow_spread_is_dropped_not_panicking() {
    let theme = shadcn_default().unwrap();
    let resolver = Resolver::new(&theme);
    let r = resolver.resolve("shadow-sm").unwrap();
    let (_, report) = lower(&r.plan);
    // shadow-sm parses (single representative layer kept by the theme).
    assert!(r.plan.base.shadow.is_some());
    let _ = report;
}

/// Per-side border widths (`border-t border-b`) are silently flattened to
/// `BorderWidth::all(bw.top)` — the honest-degradation contract requires a
/// LoweringReport entry whenever a class feature is not representable.
#[test]
fn per_side_border_width_is_reported_not_silently_dropped() {
    let theme = shadcn_default().unwrap();
    let resolver = Resolver::new(&theme);
    let resolution = resolver.resolve("border-t border-b").unwrap();
    let bw = resolution
        .plan
        .base
        .border_width
        .expect("per-side widths parsed");
    assert_eq!(bw.top, 1.0);
    assert_eq!(bw.bottom, 1.0);
    assert_eq!(bw.left, 0.0);
    assert_eq!(bw.right, 0.0);

    let (_stack, report) = lower(&resolution.plan);
    assert!(
        report.dropped.iter().any(|(what, reason)| {
            what == "border-width:per-side" && reason.contains("uniform")
        }),
        "expected a border-width:per-side drop entry, got {report:?}"
    );
}

// ---------------------------------------------------------------------------
// v0.6 gradients (AEOLUS_V0.6.md §1.5) — masonry's layout-invariant
// `Background::Gradient` consumed natively, no upstream change.
// ---------------------------------------------------------------------------

use aeolus_core::TailwindTheme;
use aeolus_core::plan::GRADIENT_FROM_POS_PCT;
use aeolus_xilem::{LoweringReport, lower_resolved};
use masonry::layout::UnitPoint;
use masonry::peniko::color::ColorSpaceTag;
use masonry::properties::types::{GradientShape, RadialGradientShape};

fn v4_theme() -> TailwindTheme {
    TailwindTheme::default()
}

fn plan_of(classes: &str) -> aeolus_core::StylePlan {
    let theme = v4_theme();
    let resolver = Resolver::new(&theme);
    let resolution = resolver.resolve(classes).unwrap();
    assert!(
        resolution.warnings.is_empty(),
        "unexpected warnings: {:?}",
        resolution.warnings
    );
    resolution.plan
}

#[test]
fn linear_gradient_angle_conversion() {
    // `a = π − θ` (derived from masonry's `get_peniko_linear_for_rect`;
    // see the fn docs). bg-linear-to-r = CSS 90° → π/2 (end at right).
    let theme = v4_theme();
    let resolver = Resolver::new(&theme);
    for (class, rad) in [
        ("bg-linear-to-t", std::f64::consts::PI),
        ("bg-linear-to-r", std::f64::consts::FRAC_PI_2),
        ("bg-linear-to-b", 0.0),
        ("bg-linear-45", std::f64::consts::PI - 45f64.to_radians()),
    ] {
        let resolution = resolver.resolve(class).unwrap();
        let mut report = LoweringReport::default();
        let set = lower_resolved(&resolution.plan.base, &mut report);
        let bg = set.get::<Background>().expect("gradient background");
        let masonry::properties::Background::Gradient(g) = bg else {
            panic!("{class} did not lower to Background::Gradient");
        };
        let masonry::properties::types::GradientShape::Linear { angle } = g.shape else {
            panic!("{class} is not a linear shape");
        };
        assert!((angle - rad).abs() < 1e-9, "{class}: {angle} vs {rad}");
        assert_eq!(
            g.interpolation_cs,
            ColorSpaceTag::Oklab,
            "v4 default in oklab"
        );
        assert_eq!(g.extend, masonry::peniko::Extend::Pad);
    }
}

#[test]
fn gradient_lowers_natively_with_v4_stop_defaults() {
    let plan = plan_of("bg-linear-to-r from-red-500 to-blue-500");
    let (stack, report) = lower(&plan);
    assert!(
        report.dropped.is_empty(),
        "gradients are native at the pin; got {report:?}"
    );
    let bg = stack.get_layers()[0].1.get::<Background>().unwrap();
    let Background::Gradient(g) = bg else {
        panic!("not a gradient");
    };
    let stops = &g.stops.0;
    assert_eq!(stops.len(), 2);
    assert!((stops[0].offset - GRADIENT_FROM_POS_PCT / 100.0).abs() < 1e-6);
    assert!((stops[1].offset - 1.0).abs() < 1e-6, "to default = 100%");
    // The stops carry the resolved sRGB colors.
    let red = to_masonry_color(plan.base.background_gradient.unwrap().stops().from.color);
    assert_eq!(
        stops[0]
            .color
            .to_alpha_color::<masonry::peniko::color::Srgb>(),
        red
    );
}

#[test]
fn via_and_position_patches_land_in_the_stop_offsets() {
    let plan = plan_of("bg-linear-to-b from-red-500 via-yellow-400 from-20% to-blue-500 to-80%");
    let mut report = LoweringReport::default();
    let set = lower_resolved(&plan.base, &mut report);
    let Background::Gradient(g) = set.get::<Background>().unwrap() else {
        panic!("not a gradient");
    };
    let stops = &g.stops.0;
    assert_eq!(stops.len(), 3, "from + via + to");
    assert!((stops[0].offset - 0.2).abs() < 1e-6, "from-20% patch");
    assert!((stops[1].offset - 0.5).abs() < 1e-6, "via default 50%");
    assert!((stops[2].offset - 0.8).abs() < 1e-6, "to-80% patch");
}

#[test]
fn gradient_wins_over_solid_in_the_single_brush() {
    // CSS layers background-image over background-color; masonry's single
    // Background property takes the gradient and the coexistence is recorded.
    let plan = plan_of("bg-red-500 bg-linear-to-r from-red-500 to-blue-500");
    let mut report = LoweringReport::default();
    let set = lower_resolved(&plan.base, &mut report);
    let bg = set.get::<Background>().unwrap();
    assert!(matches!(bg, Background::Gradient(_)), "{bg:?}");
    assert!(
        report
            .dropped
            .iter()
            .any(|(w, r)| w == "background-color under gradient" && r.contains("layers")),
        "expected the single-brush note, got {report:?}"
    );
}

#[test]
fn radial_lowers_as_documented_circle() {
    let plan = plan_of("bg-radial from-red-500 to-blue-500");
    let mut report = LoweringReport::default();
    let set = lower_resolved(&plan.base, &mut report);
    let Background::Gradient(g) = set.get::<Background>().unwrap() else {
        panic!("not a gradient");
    };
    assert!(matches!(
        g.shape,
        GradientShape::Radial { shape: RadialGradientShape::CircleTo(_), center }
            if center == UnitPoint::CENTER
    ));
    assert!(
        report
            .dropped
            .iter()
            .any(|(w, r)| w == "bg-radial shape" && r.contains("ellipse")),
        "expected the ellipse→circle normalization note, got {report:?}"
    );
}

#[test]
fn conic_reports_the_upstream_sweep_todo() {
    let plan = plan_of("bg-conic-45 from-red-500 to-blue-500");
    let mut report = LoweringReport::default();
    let set = lower_resolved(&plan.base, &mut report);
    assert!(set.get::<Background>().is_none(), "conic does not lower");
    assert!(
        report
            .dropped
            .iter()
            .any(|(w, r)| w == "bg-conic-*" && r.contains("vello")),
        "expected the vello sweep TODO reason, got {report:?}"
    );
}

#[test]
fn bg_none_lowers_to_no_gradient_property() {
    let plan = plan_of("bg-none");
    let mut report = LoweringReport::default();
    let set = lower_resolved(&plan.base, &mut report);
    assert!(set.get::<Background>().is_none());
    assert!(report.dropped.is_empty());
}

// ---------------------------------------------------------------------------
// v0.8 §1.2: per-edge set-mask materialization at the stack boundary
// ---------------------------------------------------------------------------

/// Stack entries replace WHOLE properties, so a partial-edge layer is
/// materialized against the base before lowering: `p-4 hover:px-8` keeps
/// `padding-top/bottom: 16px` on hover (the CSS cascade) instead of the
/// 0.0 placeholders wiping the base's edges.
#[test]
fn partial_edge_layers_materialize_from_base() {
    use masonry::layout::Length;

    let theme = shadcn_default().unwrap();
    let resolver = Resolver::new(&theme);
    let resolution = resolver.resolve("p-4 hover:px-8").unwrap();
    let (stack, report) = lower(&resolution.plan);
    assert!(report.dropped.is_empty(), "unexpected drops: {report:?}");

    let layers = stack.get_layers();
    assert_eq!(layers.len(), 2, "base + hover");
    let hover = layers[1]
        .1
        .get::<Padding>()
        .expect("the hover entry carries the materialized padding");
    assert_eq!(
        hover,
        &Padding {
            left: Length::px(32.0),
            right: Length::px(32.0),
            top: Length::px(16.0),
            bottom: Length::px(16.0),
        },
        "hover keeps the base's top/bottom (16px) and takes px-8's left/right"
    );

    // The base entry is untouched.
    let base = layers[0].1.get::<Padding>().expect("base padding");
    assert_eq!(
        base,
        &Padding {
            left: Length::px(16.0),
            right: Length::px(16.0),
            top: Length::px(16.0),
            bottom: Length::px(16.0),
        },
    );
}
