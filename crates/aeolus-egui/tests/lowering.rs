//! Lowering correctness for the egui backend.
//!
//! egui's `Frame` is a plain value struct, so these tests assert the
//! *lowered values* — the same struct a running app paints from — plus
//! the honest report contract for everything egui 0.37 cannot express.

use egui::{CursorIcon, TextWrapMode};

use aeolus_core::{Resolver, shadcn_default};
use aeolus_egui::{EguiStyle, StyleBook, lower_resolved};

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
                || w.starts_with("container:")
                || w.starts_with("Disabled")
                || w.starts_with("Focus")
                || w.starts_with("Visited")
                || w == "layout"),
        "unexpected drops in {classes:?}: {report:?}"
    );
    book
}

fn rgba(c: egui::Color32) -> [u8; 4] {
    let [r, g, b, a] = c.to_array();
    [r, g, b, a]
}

#[test]
fn base_layer_lowers_exactly() {
    let resolver = resolver();
    let book = tw(&resolver, "p-4 bg-primary rounded-md");

    // bg-primary = oklch(0.205 0 0) ≈ #171717.
    assert_eq!(rgba(book.base.frame.fill), [23, 23, 23, 255]);

    // p-4 = CSS padding = egui inner_margin, 16px per side.
    let m = book.base.frame.inner_margin;
    assert_eq!((m.top, m.right, m.bottom, m.left), (16, 16, 16, 16));

    // rounded-md = 8px, per-corner native (u8 points).
    assert_eq!(book.base.frame.corner_radius.nw, 8);
    assert_eq!(book.base.frame.corner_radius.se, 8);

    // No stroke without a border class.
    assert_eq!(book.base.frame.stroke, egui::Stroke::NONE);
}

#[test]
fn border_is_the_frame_stroke() {
    let resolver = resolver();
    let book = tw(&resolver, "border border-input rounded-md");
    // `border` = 1px, border-input = the shadcn input token.
    assert!((book.base.frame.stroke.width - 1.0).abs() < 1e-4);
    assert_ne!(book.base.frame.stroke.color, egui::Color32::TRANSPARENT);
    assert_eq!(book.base.frame.corner_radius.nw, 8);
}

#[test]
fn margin_is_outer_margin() {
    let resolver = resolver();
    // egui's own docs: outer_margin is "similar to what is called margin
    // in CSS" — the one backend where m-* is NATIVE.
    let book = tw(&resolver, "m-4 bg-black");
    let m = book.base.frame.outer_margin;
    assert_eq!(
        (m.top, m.right, m.bottom, m.left),
        (16, 16, 16, 16),
        "m-4 = 4 spacing units = 1rem = 16px"
    );
    assert_eq!(book.base.frame.inner_margin, egui::Margin::ZERO);
}

#[test]
fn state_layers_land_on_the_book_and_widget_visuals() {
    let resolver = resolver();
    let book = tw(&resolver, "bg-primary hover:bg-red-500 active:bg-black");

    // Hover + active lower natively into the widget-visual cascade.
    assert_eq!(book.layers.len(), 2);
    assert_eq!(
        book.layers.iter().map(|(c, _)| *c).collect::<Vec<_>>(),
        vec![
            aeolus_core::VariantCondition::Hover,
            aeolus_core::VariantCondition::Active
        ]
    );

    // inactive slot takes the base.
    let mut visuals = egui::Visuals::default().widgets.inactive;
    aeolus_egui::apply_widget_visuals(&mut visuals, &book.base);
    assert_eq!(rgba(visuals.bg_fill), [23, 23, 23, 255]);

    // hovered slot takes base + hover layer.
    let mut hovered_slot = egui::Visuals::default().widgets.inactive;
    let mut merged = book.base.clone();
    merged.overlay(&book.layers[0].1);
    aeolus_egui::apply_widget_visuals(&mut hovered_slot, &merged);
    assert_eq!(
        rgba(hovered_slot.bg_fill),
        aeolus_core::color::Color::oklch(0.637, 0.237, 25.331, 1.0).to_rgba8(),
        "hover must resolve red-500"
    );
}

#[test]
fn disabled_and_focus_conditions_resolve_via_probe_path() {
    let resolver = resolver();
    let r = resolver
        .resolve("bg-primary disabled:opacity-50 focus-visible:bg-red-500")
        .unwrap();
    let (book, report) = StyleBook::new(&r.plan);
    // Both conditions REPORTED at lower time (egui 0.37 has no disabled /
    // focused WidgetVisuals slots)...
    assert!(
        report
            .dropped
            .iter()
            .any(|(w, _)| w.starts_with("Disabled"))
    );
    assert!(report.dropped.iter().any(|(w, _)| w.starts_with("Focus")));
    assert!(book.layers.is_empty());
    // ...but resolve_at serves them per frame from the response state.
    let ctx = egui::Context::default();
    let _ = ctx; // resolution itself needs no pass
    let mut probe = aeolus_egui::EguiProbe {
        dark: false,
        disabled: true,
        ..Default::default()
    };
    let _ = &mut probe;
    let style = book.resolve_at(&probe);
    // opacity-50 dims the base fill (CSS opacity composites the element).
    assert!((style.frame.fill.a() as f32 / 255.0 - 0.5).abs() < 0.01);
}

#[test]
fn dark_resolves_for_ctx() {
    let resolver = resolver();
    let book = tw(
        &resolver,
        "bg-primary dark:bg-black text-primary dark:text-white",
    );
    let ctx = egui::Context::default();
    ctx.set_theme(egui::Theme::Light);

    let light = book.resolve_for_ctx(&ctx);
    assert_eq!(rgba(light.frame.fill), [23, 23, 23, 255]);
    // `text-primary` IS the primary token (#171717); the button tables use
    // `text-primary-foreground` for the near-white.
    assert_eq!(rgba(light.text.color.unwrap()), [23, 23, 23, 255]);

    ctx.set_theme(egui::Theme::Dark);
    let dark = book.resolve_for_ctx(&ctx);
    assert_eq!(rgba(dark.frame.fill), [0, 0, 0, 255]);
    assert_eq!(rgba(dark.text.color.unwrap()), [255, 255, 255, 255]);
}

#[test]
fn cursor_maps_across_the_v4_keyword_set() {
    let resolver = resolver();
    let book = tw(&resolver, "cursor-pointer");
    assert_eq!(book.base.cursor, Some(CursorIcon::PointingHand));

    let book = tw(&resolver, "cursor-not-allowed");
    assert_eq!(book.base.cursor, Some(CursorIcon::NotAllowed));

    let book = tw(&resolver, "cursor-zoom-in");
    assert_eq!(book.base.cursor, Some(CursorIcon::ZoomIn));
}

#[test]
fn typography_hints_lower() {
    let resolver = resolver();
    let book = tw(
        &resolver,
        "text-sm font-semibold font-mono italic underline line-through text-red-500",
    );
    let t = &book.base.text;
    let id = t.font_id.as_ref().unwrap();
    assert_eq!(id.size, 14.0, "text-sm = 0.875rem = 14px");
    assert_eq!(id.family, egui::FontFamily::Monospace);
    assert_eq!(
        t.weight,
        Some(600),
        "font-semibold = 600 → RichText::strong"
    );
    assert_eq!(t.italics, Some(true));
    // text-decoration-line is ONE property: `underline line-through`
    // resolves last-wins to line-through (the plan dedupes per property).
    assert_eq!(t.underline, None);
    assert_eq!(t.strikethrough, Some(true));

    // A lone underline lowers on its own.
    let book = tw(&resolver, "underline");
    assert_eq!(book.base.text.underline, Some(true));
    assert_eq!(book.base.text.strikethrough, None);
    assert_eq!(
        rgba(t.color.unwrap()),
        aeolus_core::color::Color::oklch(0.637, 0.237, 25.331, 1.0).to_rgba8()
    );

    // no-underline clears both decorations natively.
    let book = tw(&resolver, "underline no-underline");
    assert_eq!(book.base.text.underline, Some(false));
    assert_eq!(book.base.text.strikethrough, Some(false));
}

#[test]
fn wrap_select_and_sizing_hints_lower() {
    let resolver = resolver();
    let book = tw(
        &resolver,
        "whitespace-nowrap select-none gap-2 w-16 h-8 min-w-32",
    );
    assert_eq!(book.base.wrap_mode, Some(TextWrapMode::Extend));
    assert_eq!(book.base.selectable, Some(false));
    assert_eq!(
        book.base.item_spacing,
        Some(egui::Vec2::splat(8.0)),
        "gap-2 = 8px"
    );
    assert_eq!(book.base.width, Some(64.0));
    assert_eq!(book.base.height, Some(32.0));
    assert_eq!(
        book.base.min_size,
        Some(egui::Vec2::new(128.0, 0.0)),
        "min-w-32 = 128px"
    );
}

#[test]
fn opacity_bakes_into_every_painted_alpha() {
    let resolver = resolver();
    let r = resolver
        .resolve("bg-primary text-primary-foreground opacity-50")
        .unwrap();
    let mut report = aeolus_egui::LoweringReport::default();
    let style = lower_resolved(&r.plan.base, &mut report);
    assert!(
        (style.frame.fill.a() as f32 / 255.0 - 0.5).abs() < 0.01,
        "fill alpha must halve, got {}",
        style.frame.fill.a()
    );
    let fg = style.text.color.unwrap();
    assert!(
        (fg.a() as f32 / 255.0 - 0.5).abs() < 0.01,
        "text alpha must halve (CSS opacity composites the whole element)"
    );
}

#[test]
fn gradients_report_honestly() {
    let resolver = resolver();
    let r = resolver
        .resolve("bg-linear-to-r from-red-500 to-blue-500")
        .unwrap();
    let mut report = aeolus_egui::LoweringReport::default();
    let _ = lower_resolved(&r.plan.base, &mut report);
    assert!(
        report
            .dropped
            .iter()
            .any(|(w, _)| w.starts_with("bg-linear")),
        "Frame fills are solid; gradients must report: {report:?}"
    );

    // Solid + gradient coexistence reports the flattening too.
    let r = resolver
        .resolve("bg-red-500 bg-linear-to-r from-red-500 to-blue-500")
        .unwrap();
    let mut report = aeolus_egui::LoweringReport::default();
    let _ = lower_resolved(&r.plan.base, &mut report);
    assert!(
        report
            .dropped
            .iter()
            .any(|(w, _)| w == "background-color under gradient"),
        "{report:?}"
    );
}

#[test]
fn ring_reports_outline_takes_the_free_stroke() {
    let resolver = resolver();
    // ring always reports (single stroke slot).
    let r = resolver.resolve("ring-2 ring-red-500").unwrap();
    let mut report = aeolus_egui::LoweringReport::default();
    let _ = lower_resolved(&r.plan.base, &mut report);
    assert!(report.dropped.iter().any(|(w, _)| w == "ring"));

    // outline takes the stroke slot when no border class competes.
    let r = resolver.resolve("outline-2 outline-red-500").unwrap();
    let mut report = aeolus_egui::LoweringReport::default();
    let style = lower_resolved(&r.plan.base, &mut report);
    assert!(report.is_empty(), "{report:?}");
    assert!((style.frame.stroke.width - 2.0).abs() < 1e-4);

    // …but reports when border-* already owns the stroke.
    let r = resolver
        .resolve("border border-black outline-2 outline-red-500")
        .unwrap();
    let mut report = aeolus_egui::LoweringReport::default();
    let _ = lower_resolved(&r.plan.base, &mut report);
    assert!(report.dropped.iter().any(|(w, _)| w == "outline"));
}

#[test]
fn cn_rightmost_wins() {
    let resolver = resolver();
    let book = tw(&resolver, "bg-red-500 bg-black");
    assert_eq!(rgba(book.base.frame.fill), [0, 0, 0, 255]);
}

#[test]
fn empty_plan_is_a_noop() {
    let (book, report) = StyleBook::new(&aeolus_core::StylePlan::default());
    assert!(report.is_empty());
    assert_eq!(book.base, EguiStyle::default());
    assert!(book.layers.is_empty());
    assert!(book.dark.is_none());
}

#[test]
fn resolve_for_response_applies_state_layers_in_rank_order() {
    let resolver = resolver();
    let book = tw(&resolver, "bg-black hover:bg-red-500 active:bg-white");

    // egui's own doc-test UI harness drives a real Response through a
    // pass — but the state→style selection is what we pin, so drive it
    // through the probe equivalent instead (same code path resolve_for
    // uses for condition matching).
    let mut probe = aeolus_egui::EguiProbe {
        hovered: true,
        ..Default::default()
    };
    let hovered = book.resolve_at(&probe);
    assert_eq!(
        rgba(hovered.frame.fill),
        aeolus_core::color::Color::oklch(0.637, 0.237, 25.331, 1.0).to_rgba8()
    );

    probe.hovered = false;
    probe.active = true;
    let active = book.resolve_at(&probe);
    assert_eq!(rgba(active.frame.fill), [255, 255, 255, 255]);
}

#[test]
fn shadcn_button_string_lowers_end_to_end() {
    // The shadcn Default-variant button table, verbatim.
    const DEFAULT_BUTTON: &str = "inline-flex items-center justify-center gap-2 rounded-md text-sm font-medium \
whitespace-nowrap bg-primary text-primary-foreground hover:bg-primary/90";
    let resolver = resolver();
    let book = tw(&resolver, DEFAULT_BUTTON);
    // Interactive surface: base + hover lowered, typography carried.
    assert_eq!(book.layers.len(), 1);
    assert_eq!(book.base.text.weight, Some(500));
    assert_eq!(book.base.text.font_id.as_ref().unwrap().size, 14.0);
    assert_eq!(rgba(book.base.frame.fill), [23, 23, 23, 255]);
}

// ---------------------------------------------------------------------------
// v0.8 §1.2: per-edge set-mask merge in the Frame cascade
// ---------------------------------------------------------------------------

/// [`EguiStyle::overlay`] (the accumulation `apply_widget_visuals_cascade`
/// and the book resolvers do) merges `inner_margin`/`outer_margin` EDGE-WISE
/// through the v0.8 set-mask: `p-4` ⊕ `hover:px-8` keeps top/bottom 16px
/// instead of the 0.0 placeholders wiping them.
#[test]
fn partial_edge_layers_overlay_edge_wise() {
    use aeolus_egui::LoweringReport;

    let resolver = resolver();
    let base_plan = resolver.resolve("p-4").unwrap().plan;
    let layer_plan = resolver.resolve("hover:px-8").unwrap().plan;

    let mut report = LoweringReport::default();
    let mut base = lower_resolved(&base_plan.base, &mut report);
    let mut layer_report = LoweringReport::default();
    let layer = lower_resolved(&layer_plan.layers[0].style, &mut layer_report);

    base.overlay(&layer);
    let m = base.frame.inner_margin;
    assert_eq!(
        (m.top, m.right, m.bottom, m.left),
        (16, 32, 16, 32),
        "hover:px-8 overrides left/right only; top/bottom keep p-4"
    );
}
