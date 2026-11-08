//! Lowering correctness for the Iced backend.
//!
//! Iced's `container::Style` / `button::Style` are plain value structs, so
//! these tests assert the *lowered values* — the same structs a running
//! app paints from — plus the honest report contract for everything Iced
//! 0.14 cannot express.

use iced_core::mouse::Interaction;
use iced_core::{Background, Color, Gradient, Length};
use iced_widget::button::Status;

use aeolus_core::color::Color as TwColor;
use aeolus_core::{Resolver, shadcn_default};
use aeolus_iced::{IcedStyle, StyleBook, lower_resolved};

fn resolver() -> Resolver<'static> {
    let theme = shadcn_default().unwrap();
    Resolver::new(Box::leak(Box::new(theme)))
}

/// Resolve + lower, tolerating only the report entries the classes asked
/// for (kept strict so new silent drops cannot slip in).
fn tw(resolver: &Resolver<'static>, classes: &str) -> StyleBook {
    let resolution = resolver.resolve(classes).unwrap();
    let (book, report) = StyleBook::new(&resolution.plan);
    assert!(
        report
            .dropped
            .iter()
            .all(|(w, _)| w.starts_with("typography")
                || w.starts_with("rules")
                || w.starts_with("responsive")
                || w.starts_with("Focus")),
        "unexpected drops in {classes:?}: {report:?}"
    );
    book
}

fn solid(bg: &Background) -> Color {
    match *bg {
        Background::Color(c) => c,
        Background::Gradient(g) => panic!("expected a solid color, got gradient {g:?}"),
    }
}

fn rgba(c: Color) -> [u8; 4] {
    [
        (c.r * 255.0).round() as u8,
        (c.g * 255.0).round() as u8,
        (c.b * 255.0).round() as u8,
        (c.a * 255.0).round() as u8,
    ]
}

#[test]
fn base_layer_lowers_exactly() {
    let resolver = resolver();
    let book = tw(&resolver, "p-4 bg-primary rounded-md");

    // bg-primary = oklch(0.205 0 0) ≈ #171717.
    let bg = solid(book.base.container.background.as_ref().unwrap());
    assert_eq!(rgba(bg), [23, 23, 23, 255], "bg-primary must be #171717");

    // p-4 = 16px on every side (widget-argument hint: Iced carries
    // padding on the constructor, not the cascade).
    let padding = book.base.widget.padding.unwrap();
    assert_eq!(
        (padding.top, padding.right, padding.bottom, padding.left),
        (16.0, 16.0, 16.0, 16.0)
    );

    // rounded-md = 0.625rem - 2px = 8px, per-corner native.
    assert_eq!(book.base.container.border.radius.top_left, 8.0);
    assert_eq!(book.base.container.border.radius.bottom_right, 8.0);
}

#[test]
fn button_statuses_map_state_layers() {
    let resolver = resolver();
    let book = tw(
        &resolver,
        "bg-primary hover:bg-red-500 active:bg-black disabled:bg-blue-500",
    );
    let light = iced_core::Theme::Light;

    // Idle: primary.
    let idle = book.button_style(Status::Active, &light);
    assert_eq!(
        rgba(solid(idle.background.as_ref().unwrap())),
        [23, 23, 23, 255]
    );

    // Hovered: red-500 = oklch(63.7% 0.237 25.331).
    let hovered = book.button_style(Status::Hovered, &light);
    let expected = TwColor::oklch(0.637, 0.237, 25.331, 1.0).to_rgba8();
    assert_eq!(
        rgba(solid(hovered.background.as_ref().unwrap())),
        expected,
        "hover must resolve red-500"
    );

    // Pressed: the active layer.
    let pressed = book.button_style(Status::Pressed, &light);
    assert_eq!(
        rgba(solid(pressed.background.as_ref().unwrap())),
        [0, 0, 0, 255]
    );

    // Disabled: the disabled layer (blue-500 = oklch(62.3% 0.214 259.815)).
    let disabled = book.button_style(Status::Disabled, &light);
    let expected = TwColor::oklch(0.623, 0.214, 259.815, 1.0).to_rgba8();
    assert_eq!(
        rgba(solid(disabled.background.as_ref().unwrap())),
        expected,
        "disabled:bg-blue-500 = the v4 token"
    );
}

#[test]
fn dark_layer_resolves_on_dark_theme() {
    let resolver = resolver();
    let book = tw(
        &resolver,
        "bg-primary dark:bg-black text-primary dark:text-white",
    );

    // Light theme: primary.
    let light = book.container_style(&iced_core::Theme::Light);
    assert_eq!(rgba(solid(&light.background.unwrap())), [23, 23, 23, 255]);

    // Dark theme: the dark layer overlays.
    let dark = book.container_style(&iced_core::Theme::Dark);
    assert_eq!(rgba(solid(&dark.background.unwrap())), [0, 0, 0, 255]);
    assert_eq!(
        rgba(dark.text_color.unwrap()),
        [255, 255, 255, 255],
        "dark:text-white"
    );
}

#[test]
fn theme_is_dark_detects_by_palette_luminance() {
    assert!(!aeolus_iced::theme_is_dark(&iced_core::Theme::Light));
    assert!(aeolus_iced::theme_is_dark(&iced_core::Theme::Dark));
    // A custom theme with a dark background answers honestly too.
    let custom = iced_core::Theme::custom(
        "dark-custom",
        iced_core::theme::Palette {
            background: Color::BLACK,
            ..iced_core::theme::Palette::DARK
        },
    );
    assert!(aeolus_iced::theme_is_dark(&custom));
}

#[test]
fn linear_gradient_angle_conversion() {
    let resolver = resolver();
    // CSS 90° (to right): the gradient line's END points right.
    let book = tw(&resolver, "bg-linear-to-r from-red-500 to-blue-500");
    let bg = book.base.container.background.as_ref().unwrap();
    let lin = match bg {
        Background::Gradient(Gradient::Linear(lin)) => *lin,
        other => panic!("expected a native linear gradient, got {other:?}"),
    };
    // DERIVED FROM iced_core/src/angle.rs::Radians::to_distance:
    // direction = (sin θ, −cos θ), so CSS 90° (end right) ⇒ θ = π/2.
    assert!(
        (lin.angle.0 - std::f32::consts::FRAC_PI_2).abs() < 1e-6,
        "bg-linear-to-r must be π/2, got {:?}",
        lin.angle
    );

    // from/to land at the v4 default stop positions 0% / 100%.
    let stops: Vec<(f32, [u8; 4])> = lin
        .stops
        .into_iter()
        .flatten()
        .map(|s| (s.offset, rgba(s.color)))
        .collect();
    assert_eq!(stops.len(), 2);
    assert!((stops[0].0 - 0.0).abs() < 1e-6);
    assert!((stops[1].0 - 1.0).abs() < 1e-6);
    // red-500 → blue-500 in oklch, converted at the backend boundary.
    assert_eq!(
        stops[0].1,
        TwColor::oklch(0.637, 0.237, 25.331, 1.0).to_rgba8()
    );
}

#[test]
fn gradient_coexistence_reports_and_takes_the_gradient() {
    let resolver = resolver();
    let r = resolver
        .resolve("bg-red-500 bg-linear-to-r from-red-500 to-blue-500")
        .unwrap();
    let mut report = aeolus_iced::LoweringReport::default();
    let style = lower_resolved(&r.plan.base, &mut report);
    match style.container.background.as_ref().unwrap() {
        Background::Gradient(Gradient::Linear(_)) => {}
        other => panic!("the gradient must win the single Background slot: {other:?}"),
    }
    assert!(
        report
            .dropped
            .iter()
            .any(|(w, _)| w == "background-color under gradient"),
        "the coexistence must be recorded: {report:?}"
    );
}

#[test]
fn radial_and_conic_report_honestly() {
    let resolver = resolver();
    for classes in [
        "bg-radial from-red-500 to-blue-500",
        "bg-conic from-red-500 to-blue-500",
    ] {
        let r = resolver.resolve(classes).unwrap();
        let mut report = aeolus_iced::LoweringReport::default();
        let _ = lower_resolved(&r.plan.base, &mut report);
        assert!(
            report.dropped.iter().any(|(w, _)| w.starts_with("bg-")),
            "{classes} must report (Iced 0.14 Gradient is linear-only): {report:?}"
        );
    }
}

#[test]
fn opacity_bakes_into_every_painted_alpha() {
    let resolver = resolver();
    let r = resolver
        .resolve("bg-primary text-primary-foreground opacity-50")
        .unwrap();
    let mut report = aeolus_iced::LoweringReport::default();
    let style = lower_resolved(&r.plan.base, &mut report);
    let bg = solid(style.container.background.as_ref().unwrap());
    assert!(
        (bg.a - 0.5).abs() < 1e-3,
        "background alpha must halve, got {}",
        bg.a
    );
    let fg = style.container.text_color.unwrap();
    assert!(
        (fg.a - 0.5).abs() < 1e-3,
        "text alpha must halve (CSS opacity composites the whole element)"
    );
}

#[test]
fn per_side_border_reports() {
    let resolver = resolver();
    let r = resolver.resolve("border-t-2 border-black").unwrap();
    let mut report = aeolus_iced::LoweringReport::default();
    let style = lower_resolved(&r.plan.base, &mut report);
    assert_eq!(style.container.border.width, 2.0, "top edge survives");
    assert!(
        report
            .dropped
            .iter()
            .any(|(w, _)| w == "border-width:per-side"),
        "the flattening must be recorded: {report:?}"
    );
}

#[test]
fn ring_margin_and_responsive_report() {
    let resolver = resolver();
    let r = resolver
        .resolve("ring-2 ring-red-500 m-4 sm:bg-black")
        .unwrap();
    let (book, report) = StyleBook::new(&r.plan);
    assert!(report.dropped.iter().any(|(w, _)| w == "ring"));
    assert!(report.dropped.iter().any(|(w, _)| w == "margin"));
    assert!(
        report
            .dropped
            .iter()
            .any(|(w, _)| w.starts_with("responsive")),
        "no window-size signal inside a Catalog closure: {report:?}"
    );
    // The ring only landed in the report — nothing natively lowered.
    assert!(book.base.container.background.is_none());
}

#[test]
fn text_hints_lower_to_constructor_args() {
    let resolver = resolver();
    let book = tw(
        &resolver,
        "text-sm font-medium leading-tight font-mono text-left italic whitespace-nowrap",
    );
    let t = &book.base.text;
    assert_eq!(t.font_size, Some(14.0), "text-sm = 0.875rem = 14px");
    assert_eq!(
        t.weight,
        Some(iced_core::font::Weight::Medium),
        "font-medium = 500"
    );
    assert_eq!(
        t.line_height,
        Some(iced_core::text::LineHeight::Relative(1.25)),
        "leading-tight = --leading-tight = 1.25"
    );
    assert_eq!(t.family, Some(iced_core::font::Family::Monospace));
    assert_eq!(t.align, Some(iced_core::text::Alignment::Left));
    assert_eq!(t.font_style, Some(iced_core::font::Style::Italic));
    assert_eq!(t.wrapping, Some(iced_core::text::Wrapping::None));
}

#[test]
fn widget_hints_lower_to_constructor_args() {
    let resolver = resolver();
    let book = tw(
        &resolver,
        "w-16 h-8 gap-2 cursor-pointer pointer-events-none",
    );
    let w = &book.base.widget;
    assert_eq!(w.width, Some(Length::Fixed(64.0)), "w-16 = 4rem = 64px");
    assert_eq!(w.height, Some(Length::Fixed(32.0)));
    assert_eq!(w.spacing, Some(8.0), "gap-2 = 8px");
    assert_eq!(w.cursor, Some(Interaction::Pointer));
    assert_eq!(w.interactive, Some(false), "pointer-events-none");
}

#[test]
fn cn_rightmost_wins() {
    let resolver = resolver();
    let book = tw(&resolver, "bg-red-500 bg-black");
    assert_eq!(
        rgba(solid(book.base.container.background.as_ref().unwrap())),
        [0, 0, 0, 255]
    );
}

#[test]
fn full_width_normalizes_to_fill() {
    let resolver = resolver();
    let book = tw(&resolver, "w-full h-px");
    assert_eq!(book.base.widget.width, Some(Length::Fill));
    assert_eq!(book.base.widget.height, Some(Length::Fixed(1.0)));
}

#[test]
fn empty_plan_is_a_noop() {
    let (book, report) = StyleBook::new(&aeolus_core::StylePlan::default());
    assert!(report.is_empty());
    assert_eq!(book.base, IcedStyle::default());
    assert!(book.layers.is_empty());
    assert!(book.dark.is_none());
}

#[test]
fn focus_conditions_report_for_the_probe_path() {
    let resolver = resolver();
    let r = resolver
        .resolve("bg-primary focus-visible:bg-red-500 focus:bg-black")
        .unwrap();
    let (book, report) = StyleBook::new(&r.plan);
    // Both conditions reported (Iced 0.14 has no Focused Status).
    assert_eq!(
        report
            .dropped
            .iter()
            .filter(|(w, _)| w.starts_with("Focus"))
            .count(),
        2,
        "focus + focus-visible must report: {report:?}"
    );
    // The book keeps no such layers natively — resolve_at serves them.
    assert!(book.layers.is_empty());
}

// ---------------------------------------------------------------------------
// v0.8 §1.2: per-edge set-mask merge in the Catalog cascade
// ---------------------------------------------------------------------------

/// [`IcedStyle::overlay`] (the accumulation the Catalog closures do per
/// frame) merges the padding hint EDGE-WISE through the v0.8 set-mask:
/// `p-4` ⊕ `hover:px-8` keeps top/bottom 16px instead of the 0.0
/// placeholders wiping them.
#[test]
fn partial_edge_layers_overlay_edge_wise() {
    use aeolus_iced::LoweringReport;

    let resolver = resolver();
    let base_plan = resolver.resolve("p-4").unwrap().plan;
    let layer_plan = resolver.resolve("hover:px-8").unwrap().plan;

    let mut report = LoweringReport::default();
    let mut base = lower_resolved(&base_plan.base, &mut report);
    let mut layer_report = LoweringReport::default();
    let layer = lower_resolved(&layer_plan.layers[0].style, &mut layer_report);

    base.overlay(&layer);
    let padding = base.widget.padding.expect("merged padding hint");
    assert_eq!(
        (padding.top, padding.right, padding.bottom, padding.left),
        (16.0, 32.0, 16.0, 32.0),
        "hover:px-8 overrides left/right only; top/bottom keep p-4"
    );
}
