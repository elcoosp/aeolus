//! Lowering correctness for the Floem backend.
//!
//! Floem's `Style::apply_selectors(&[StyleSelector])` is the public
//! cascade-evaluation entry point, so these tests resolve the lowered style
//! under simulated widget states and assert the *effective* property values —
//! the same thing a running app would compute.

use floem::peniko::{Brush, Color as PenikoColor};
use floem::style::unit::Length;
use floem::style::{Background, PaddingLeft, StyleSelector, TextColor};

use aeolus_core::color::Color;
use aeolus_core::{shadcn_default, Resolver};
use aeolus_floem::lower;

fn resolver() -> Resolver<'static> {
    let theme = shadcn_default().unwrap();
    Resolver::new(Box::leak(Box::new(theme)))
}

fn brush_rgba(b: &Brush) -> Option<[u8; 4]> {
    match b {
        // peniko 0.6 has no `[u8; 4]` compare for Rgba8 — use raw components.
        Brush::Solid(c) => Some(c.components.map(|v| (v * 255.0).round() as u8)),
        _ => None,
    }
}

#[test]
fn base_layer_lowers_exactly() {
    let resolver = resolver();
    let r = resolver
        .resolve("p-4 bg-primary rounded-md")
        .unwrap();
    let (style, report) = lower(&r.plan);
    assert!(report.is_empty(), "unexpected drops: {report:?}");

    // bg-primary = oklch(0.205 0 0) ≈ #171717.
    let bg = style.get(Background);
    let rgba = brush_rgba(bg.as_ref().unwrap()).unwrap();
    assert_eq!(rgba, [23, 23, 23, 255], "bg-primary must be #171717");

    // p-4 = 16px on every side.
    assert_eq!(style.get(PaddingLeft), Length::Pt(16.0));

    // rounded-md = 0.625rem - 2px = 8px.
    assert_eq!(
        style.get(floem::style::BorderTopLeftRadius),
        Length::Pt(8.0)
    );
}

#[test]
fn hover_resolves_through_selectors() {
    let resolver = resolver();
    let r = resolver.resolve("bg-primary hover:bg-red-500").unwrap();
    let (style, report) = lower(&r.plan);
    assert!(report.is_empty(), "{report:?}");

    // Idle: primary.
    let idle = style.clone().apply_selectors(&[]);
    let idle_bg = brush_rgba(idle.get(Background).as_ref().unwrap()).unwrap();
    assert_eq!(idle_bg, [23, 23, 23, 255]);

    // Hovered: red-500 = oklch(63.7% 0.237 25.331).
    let hovered = style.apply_selectors(&[StyleSelector::Hover]);
    let hover_bg = brush_rgba(hovered.get(Background).as_ref().unwrap()).unwrap();
    let expected = Color::oklch(0.637, 0.237, 25.331, 1.0).to_rgba8();
    assert_eq!(hover_bg, expected, "hover must resolve red-500");
}

#[test]
fn focus_within_and_disabled_map_to_selectors() {
    let resolver = resolver();
    let r = resolver
        .resolve("bg-primary focus-within:bg-black disabled:bg-red-500")
        .unwrap();
    let (style, report) = lower(&r.plan);
    assert!(report.is_empty(), "focus-within and disabled must map: {report:?}");

    let disabled = style.clone().apply_selectors(&[StyleSelector::Disabled]);
    let disabled_bg = brush_rgba(disabled.get(Background).as_ref().unwrap()).unwrap();
    // v4 red-500 is oklch(0.637 0.237 25.331); compare against our own
    // CSS-Color-4 conversion (simple gamut clamp), not a v3 hex memory.
    let expected = Color::oklch(0.637, 0.237, 25.331, 1.0).to_srgba();
    let expected: [u8; 4] = expected.map(|v| (v * 255.0).round() as u8);
    assert_eq!(disabled_bg, expected, "disabled:bg-red-500 = the v4 token");

    let within = style.apply_selectors(&[StyleSelector::FocusWithin]);
    let within_bg = brush_rgba(within.get(Background).as_ref().unwrap()).unwrap();
    assert_eq!(within_bg, [0, 0, 0, 255], "focus-within:bg-black = #000");
}

#[test]
fn dark_layer_resolves_in_dark_mode() {
    let theme = shadcn_default().unwrap();
    let theme = theme.dark_variant(aeolus_core::shadcn_css::DARK).unwrap();
    let resolver = Resolver::new(&theme);
    let r = resolver.resolve("bg-primary dark:bg-black text-primary dark:text-white").unwrap();
    let (style, report) = lower(&r.plan);
    assert!(
        !report.dropped.iter().any(|(w, _)| w == "dark"),
        "dark must map to dark_mode(): {report:?}"
    );

    let dark = style.apply_selectors(&[StyleSelector::DarkMode]);
    let dark_bg = brush_rgba(dark.get(Background).as_ref().unwrap()).unwrap();
    assert_eq!(dark_bg, [0, 0, 0, 255], "dark:bg-black");

    // Inherited text color prop flips to white (#fff) in dark mode.
    let fg: Option<PenikoColor> = dark.get(TextColor);
    assert_eq!(
        fg.map(|c| c.components.map(|v| (v * 255.0).round() as u8)),
        Some([255, 255, 255, 255]),
        "dark:text-white"
    );
}

#[test]
fn group_and_peer_are_reported_not_silently_dropped() {
    let resolver = resolver();
    let r = resolver
        .resolve("bg-primary group-hover:bg-red-500 peer-focus:text-black md:p-8")
        .unwrap();
    let (_, report) = lower(&r.plan);
    let whats: Vec<_> = report.dropped.iter().map(|(w, _)| w.as_str()).collect();
    assert!(whats.iter().any(|w| w.starts_with("group:")), "{report:?}");
    assert!(whats.iter().any(|w| w.starts_with("peer:")), "{report:?}");
    // md:p-8 lowers into a min-window-width responsive rule — no drop.
    assert!(!whats.iter().any(|w| w.starts_with("responsive")), "{report:?}");
}

#[test]
fn transitions_attach_when_requested() {
    let resolver = resolver();
    let r = resolver.resolve("bg-primary").unwrap();
    // `lower` without transitions must not fail; with transitions the
    // builders run (smoke test — transition maps are internal to Style).
    let (_, _) = lower(&r.plan);
    let (_, report) = aeolus_floem::lower_with_transitions(&r.plan, Some(150.0));
    assert!(report.is_empty());
}

#[test]
fn oklab_alpha_semantics_match_v4() {
    // `bg-primary/50` mixes with transparent in oklab: chroma AND alpha scale.
    let resolver = resolver();
    let r = resolver.resolve("bg-primary/50").unwrap();
    let bg = r.plan.base.background.unwrap().0;
    assert!((bg.alpha - 0.5).abs() < 1e-4);
    let (style, _) = lower(&r.plan);
    let rgba = brush_rgba(style.get(Background).as_ref().unwrap()).unwrap();
    assert_eq!(rgba[3], 128, "50% alpha straight through to peniko");
}
