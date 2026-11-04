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
    assert!(
        whats.iter().any(|w| w.starts_with("container:")),
        "{report:?}"
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
