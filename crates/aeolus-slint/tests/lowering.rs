//! Lowering correctness for the Slint backend.
//!
//! Slint's element properties are plain typed values, so these tests
//! assert the *lowered values* — the same `Brush`/lengths a running app
//! paints from — plus the honest report contract for everything Slint
//! 1.18 cannot express.

use i_slint_core::graphics::Brush;
use i_slint_core::items::{BuiltInMouseCursor, TextHorizontalAlignment, TextOverflow, TextWrap};

use aeolus_core::color::Color as TwColor;
use aeolus_core::plan::{Cursor, SizedValue};
use aeolus_core::{Resolver, shadcn_default};
use aeolus_slint::{PaddingPx, SlintLength, StyleBook, lower_resolved, to_slint_color};

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
                || w.starts_with("tracking")
                || w.starts_with("font-")
                || w.starts_with("select")
                || w.starts_with("line-")
                || w.starts_with("leading")
                || w.starts_with("Focus")
                || w.starts_with("transition")
                || w.starts_with("text-")
                || w.starts_with("underline")),
        "unexpected drops in {classes:?}: {report:?}"
    );
    book
}

#[test]
fn base_fill_and_radius_lower_natively() {
    let r = resolver();
    let book = tw(&r, "bg-primary rounded-md border border-input");
    let base = &book.base;
    // The lowered fill is a real solid brush (opaque, non-transparent).
    let argb = match &base.background {
        Some(Brush::SolidColor(c)) => c.as_argb_encoded(),
        other => panic!("expected a solid brush, got {other:?}"),
    };
    assert_eq!(argb >> 24, 0xFF, "primary fill is fully opaque");
    // rounded-md = calc(var(--radius) - 2px) = 8px on every corner.
    let radius = base.border_radius.expect("rounded-md lowered");
    assert_eq!(
        (
            radius.top_left,
            radius.top_right,
            radius.bottom_right,
            radius.bottom_left
        ),
        (8.0, 8.0, 8.0, 8.0)
    );
    // border (1px) + border-input color.
    assert_eq!(base.border_width, Some(1.0));
    assert!(base.border_color.is_some());
}

#[test]
fn per_corner_radius_is_native() {
    let r = resolver();
    let book = tw(&r, "rounded-tl-lg rounded-br-sm");
    let radius = book.base.border_radius.expect("per-corner radii lowered");
    // rounded-tl-lg = var(--radius) = 10px; rounded-br-sm = 6px.
    assert_eq!(radius.top_left, 10.0);
    assert_eq!(radius.top_right, 0.0);
    assert_eq!(radius.bottom_right, 6.0);
    assert_eq!(radius.bottom_left, 0.0);
}

#[test]
fn shadows_carry_spread_and_inset_surface() {
    let r = resolver();
    // shadow-xs: 0 1px 2px 0 rgb(0 0 0 / 0.05).
    let book = tw(&r, "shadow-xs");
    let ds = book.base.drop_shadow.expect("drop shadow lowered");
    assert_eq!(ds.offset_x, 0.0);
    assert_eq!(ds.offset_y, 1.0);
    assert_eq!(ds.blur, 2.0);
    assert_eq!(ds.spread, 0.0);
    let a = (ds.color.as_argb_encoded() >> 24) as u8;
    assert!(a < 255, "shadow color carries the 5% alpha, got {a}");

    // inset shadows land on the inner-shadow surface.
    let resolution = r.resolve("inset-shadow-2xs").unwrap();
    let mut report = Default::default();
    let lowered = lower_resolved(&resolution.plan.base, &mut report);
    assert!(
        lowered.inner_shadow.is_some(),
        "inset shadows map to Slint's inner-shadow surface: {report:?}"
    );
}

#[test]
fn gradients_lower_to_native_brushes() {
    let r = resolver();
    let book = tw(&r, "bg-linear-to-r from-red-500 to-blue-500");
    match &book.base.background_gradient {
        Some(Brush::LinearGradient(g)) => {
            // to-r = 90deg, and slint's angle convention matches CSS.
            assert!((g.angle() - 90.0).abs() < 1e-3);
            let stops: Vec<_> = g.stops().collect();
            assert_eq!(stops.len(), 2);
            assert!((stops[0].position - 0.0).abs() < 1e-4);
            assert!((stops[1].position - 1.0).abs() < 1e-4);
        }
        other => panic!("expected a native linear gradient brush, got {other:?}"),
    }

    // radial + conic shapes map too.
    let resolution = r.resolve("bg-radial from-white to-black").unwrap();
    let mut report = Default::default();
    let lowered = lower_resolved(&resolution.plan.base, &mut report);
    assert!(matches!(
        lowered.background_gradient,
        Some(Brush::RadialGradient(_))
    ));
    let resolution = r.resolve("bg-conic from-red-500 to-blue-500").unwrap();
    let lowered = lower_resolved(&resolution.plan.base, &mut report);
    assert!(matches!(
        lowered.background_gradient,
        Some(Brush::ConicGradient(_))
    ));
}

#[test]
fn bg_none_clears_the_gradient_on_overlay() {
    let r = resolver();
    let resolution = r
        .resolve("bg-linear-to-r from-red-500 to-blue-500 bg-none")
        .unwrap();
    let mut report = Default::default();
    let lowered = lower_resolved(&resolution.plan.base, &mut report);
    assert!(
        lowered.background_gradient.is_none(),
        "bg-none clears the image in the composed plan"
    );

    // And a bg-none LAYER clears an accumulated gradient
    // (gradient_cleared is set by lower_resolved when it sees bg-none).
    let mut acc = lowered.clone();
    let resolution = r.resolve("bg-none").unwrap();
    let none_layer = lower_resolved(&resolution.plan.base, &mut report);
    assert!(none_layer.background_gradient.is_none());
    acc.overlay(&none_layer);
    assert!(acc.background_gradient.is_none());
}

#[test]
fn opacity_is_a_native_property() {
    let r = resolver();
    let book = tw(&r, "opacity-50");
    assert_eq!(book.base.opacity, Some(0.5));
}

#[test]
fn overflow_hidden_lowers_to_clip() {
    let r = resolver();
    let book = tw(&r, "overflow-hidden");
    assert_eq!(book.base.clip, Some(true));
}

#[test]
fn typography_lowers_onto_the_text_surface() {
    let r = resolver();
    let resolution = r
        .resolve("text-sm font-semibold tracking-tight whitespace-nowrap truncate")
        .unwrap();
    let mut report = Default::default();
    let style = lower_resolved(&resolution.plan.base, &mut report);
    let t = &style.text;
    assert_eq!(t.font_size, Some(14.0));
    assert_eq!(t.font_weight, Some(600));
    // tracking stays element-relative (em) in the IR — v0.5 §1.1 — so the
    // backend reports it: Slint's letter-spacing is a length and the
    // compiler cannot know which font-size the element ends up with.
    assert!(t.letter_spacing.is_none());
    assert!(
        report.dropped.iter().any(|(w, _)| w == "tracking (em)"),
        "em tracking reported for the call site to resolve: {report:?}"
    );
    assert_eq!(t.wrap, Some(TextWrap::NoWrap));
    assert_eq!(t.overflow, Some(TextOverflow::Elide));
}

#[test]
fn text_alignment_maps() {
    let r = resolver();
    let book = tw(&r, "text-center");
    assert_eq!(
        book.base.text.horizontal_alignment,
        Some(TextHorizontalAlignment::Center)
    );
}

#[test]
fn geometry_percentages_and_scale() {
    let r = resolver();
    let book = tw(&r, "w-full h-10 min-w-20 max-w-96");
    assert_eq!(book.base.width, Some(SlintLength::Percent(100.0)));
    assert_eq!(book.base.height, Some(SlintLength::Px(40.0)));
    assert_eq!(book.base.min_width, Some(SlintLength::Px(80.0)));
    assert_eq!(book.base.max_width, Some(SlintLength::Px(384.0)));
}

#[test]
fn padding_is_a_layout_hint_with_full_fidelity() {
    let r = resolver();
    let book = tw(&r, "px-3 py-1.5");
    let p = book.base.padding.expect("padding carried as a hint");
    assert_eq!(p.left, 12.0);
    assert_eq!(p.right, 12.0);
    assert_eq!(p.top, 6.0);
    assert_eq!(p.bottom, 6.0);
}

#[test]
fn cursor_maps_to_builtin() {
    let r = resolver();
    let book = tw(&r, "cursor-pointer");
    assert_eq!(book.base.mouse_cursor, Some(BuiltInMouseCursor::Pointer));

    // The keywords Slint lacks are reported, not dropped silently.
    let resolution = r.resolve("cursor-zoom-in").unwrap();
    let mut report = Default::default();
    let lowered = lower_resolved(&resolution.plan.base, &mut report);
    assert!(lowered.mouse_cursor.is_none());
    assert!(
        report.dropped.iter().any(|(w, _)| w == "cursor"),
        "zoom-in reported: {report:?}"
    );
}

#[test]
fn cursor_full_set_matches() {
    // Every cursor Slint carries lowers to the identically named variant.
    let pairs = [
        (Cursor::Default, BuiltInMouseCursor::Default),
        (Cursor::None, BuiltInMouseCursor::None),
        (Cursor::Help, BuiltInMouseCursor::Help),
        (Cursor::Pointer, BuiltInMouseCursor::Pointer),
        (Cursor::Progress, BuiltInMouseCursor::Progress),
        (Cursor::Wait, BuiltInMouseCursor::Wait),
        (Cursor::Crosshair, BuiltInMouseCursor::Crosshair),
        (Cursor::Text, BuiltInMouseCursor::Text),
        (Cursor::Alias, BuiltInMouseCursor::Alias),
        (Cursor::Copy, BuiltInMouseCursor::Copy),
        (Cursor::Move, BuiltInMouseCursor::Move),
        (Cursor::NoDrop, BuiltInMouseCursor::NoDrop),
        (Cursor::NotAllowed, BuiltInMouseCursor::NotAllowed),
        (Cursor::Grab, BuiltInMouseCursor::Grab),
        (Cursor::Grabbing, BuiltInMouseCursor::Grabbing),
        (Cursor::ColResize, BuiltInMouseCursor::ColResize),
        (Cursor::RowResize, BuiltInMouseCursor::RowResize),
        (Cursor::NResize, BuiltInMouseCursor::NResize),
        (Cursor::EResize, BuiltInMouseCursor::EResize),
        (Cursor::SResize, BuiltInMouseCursor::SResize),
        (Cursor::WResize, BuiltInMouseCursor::WResize),
        (Cursor::NeResize, BuiltInMouseCursor::NeResize),
        (Cursor::NwResize, BuiltInMouseCursor::NwResize),
        (Cursor::SeResize, BuiltInMouseCursor::SeResize),
        (Cursor::SwResize, BuiltInMouseCursor::SwResize),
        (Cursor::EwResize, BuiltInMouseCursor::EwResize),
        (Cursor::NsResize, BuiltInMouseCursor::NsResize),
        (Cursor::NeswResize, BuiltInMouseCursor::NeswResize),
        (Cursor::NwseResize, BuiltInMouseCursor::NwseResize),
    ];
    for (tw, builtin) in pairs {
        let resolution = Resolver::new(&shadcn_default().unwrap())
            .resolve(match tw {
                Cursor::Default => "cursor-default",
                Cursor::None => "cursor-none",
                Cursor::Help => "cursor-help",
                Cursor::Pointer => "cursor-pointer",
                Cursor::Progress => "cursor-progress",
                Cursor::Wait => "cursor-wait",
                Cursor::Crosshair => "cursor-crosshair",
                Cursor::Text => "cursor-text",
                Cursor::Alias => "cursor-alias",
                Cursor::Copy => "cursor-copy",
                Cursor::Move => "cursor-move",
                Cursor::NoDrop => "cursor-no-drop",
                Cursor::NotAllowed => "cursor-not-allowed",
                Cursor::Grab => "cursor-grab",
                Cursor::Grabbing => "cursor-grabbing",
                Cursor::ColResize => "cursor-col-resize",
                Cursor::RowResize => "cursor-row-resize",
                Cursor::NResize => "cursor-n-resize",
                Cursor::EResize => "cursor-e-resize",
                Cursor::SResize => "cursor-s-resize",
                Cursor::WResize => "cursor-w-resize",
                Cursor::NeResize => "cursor-ne-resize",
                Cursor::NwResize => "cursor-nw-resize",
                Cursor::SeResize => "cursor-se-resize",
                Cursor::SwResize => "cursor-sw-resize",
                Cursor::EwResize => "cursor-ew-resize",
                Cursor::NsResize => "cursor-ns-resize",
                Cursor::NeswResize => "cursor-nesw-resize",
                Cursor::NwseResize => "cursor-nwse-resize",
                _ => continue,
            })
            .unwrap();
        let mut report = Default::default();
        let lowered = lower_resolved(&resolution.plan.base, &mut report);
        assert_eq!(lowered.mouse_cursor, Some(builtin), "{tw:?}");
    }
}

#[test]
fn pointer_events_none_disables() {
    let r = resolver();
    let book = tw(&r, "pointer-events-none");
    assert_eq!(book.base.enabled, Some(false));
}

#[test]
fn per_edge_border_flattens_with_a_report() {
    let r = resolver();
    let resolution = r.resolve("border-t-2").unwrap();
    let mut report = Default::default();
    let lowered = lower_resolved(&resolution.plan.base, &mut report);
    assert_eq!(lowered.border_width, Some(2.0));
    assert!(
        report
            .dropped
            .iter()
            .any(|(w, _)| w == "border-width:per-side"),
        "per-edge border reported honestly: {report:?}"
    );
}

#[test]
fn unexpressible_slots_are_reported() {
    let r = resolver();
    let classes = "ring-3 flex gap-2 m-4 rotate-45 transition-all border-dashed absolute top-2";
    let resolution = r.resolve(classes).unwrap();
    let mut report = Default::default();
    let _ = lower_resolved(&resolution.plan.base, &mut report);
    for what in [
        "ring",
        "layout",
        "gap",
        "margin",
        "transform",
        "transition-*",
        "border-style",
        "position",
        "inset",
    ] {
        assert!(
            report.dropped.iter().any(|(w, _)| w == what),
            "expected a drop for {what:?}: {report:?}"
        );
    }
}

#[test]
fn ring_and_padding_fidelity_on_the_shadcn_button() {
    let r = resolver();
    // The real Button table: focus-visible ring-3 + px/py padding.
    // (layout/ring/line-height drops are the table's own — tolerated here
    // because this test pins the payload, not the ledger)
    let resolution = r
        .resolve(
            "inline-flex items-center rounded-md px-4 py-2 text-sm font-medium focus-visible:ring-3 disabled:opacity-50 hover:bg-primary/90",
        )
        .unwrap();
    let (book, _) = StyleBook::new(&resolution.plan);
    assert_eq!(
        book.base.padding,
        Some(PaddingPx {
            top: 8.0,
            right: 16.0,
            bottom: 8.0,
            left: 16.0
        })
    );
    assert_eq!(book.base.text.font_weight, Some(500));
    assert_eq!(book.base.text.font_size, Some(14.0));
    // The hover and disabled layers exist and carry their payload.
    let hover = book
        .layers
        .iter()
        .find(|(c, _)| matches!(c, aeolus_core::VariantCondition::Hover))
        .expect("hover layer");
    let _ = hover;
    let disabled = book
        .layers
        .iter()
        .find(|(c, _)| matches!(c, aeolus_core::VariantCondition::Disabled))
        .expect("disabled layer");
    assert_eq!(disabled.1.opacity, Some(0.5));
}

#[test]
fn sized_values_stay_honest() {
    let r = resolver();
    // w-fit / h-screen are reported, not faked.
    let resolution = r.resolve("w-fit h-screen").unwrap();
    let mut report = Default::default();
    let lowered = lower_resolved(&resolution.plan.base, &mut report);
    assert!(lowered.width.is_none() && lowered.height.is_none());
    assert!(report.dropped.iter().any(|(w, _)| w == "w:fit"));
    assert!(report.dropped.iter().any(|(w, _)| w == "h:screen"));

    // w-auto is the exact Slint default: a no-op WITHOUT a report.
    let resolution = r.resolve("w-auto").unwrap();
    let mut report = Default::default();
    let lowered = lower_resolved(&resolution.plan.base, &mut report);
    assert!(lowered.width.is_none());
    assert!(!report.dropped.iter().any(|(w, _)| w.starts_with("w:")));

    // cn() conflict resolution: a later w-auto cancels an earlier w-fit.
    let resolution = r.resolve("w-fit w-auto").unwrap();
    let mut report = Default::default();
    let lowered = lower_resolved(&resolution.plan.base, &mut report);
    assert!(lowered.width.is_none());
    assert!(!report.dropped.iter().any(|(w, _)| w == "w:fit"));
    let _ = SizedValue::Auto;
}

#[test]
fn color_conversion_round_trips_through_srgb() {
    let tw = TwColor::from_srgb(
        0x18 as f32 / 255.0,
        0x81 as f32 / 255.0,
        0x0C as f32 / 255.0,
        1.0,
    );
    let sl = to_slint_color(tw);
    let [r, g, b, a] = tw.to_rgba8();
    assert_eq!((sl.as_argb_encoded() >> 24) as u8, a);
    assert_eq!(((sl.as_argb_encoded() >> 16) & 0xFF) as u8, r);
    assert_eq!(((sl.as_argb_encoded() >> 8) & 0xFF) as u8, g);
    assert_eq!((sl.as_argb_encoded() & 0xFF) as u8, b);
}

// ---------------------------------------------------------------------------
// v0.8 §1.2: per-edge set-mask materialization for state re-bindings
// ---------------------------------------------------------------------------

/// Slint state blocks re-bind properties wholesale, so the generated
/// `states [ … ]` snippet materializes a partial-edge layer against the
/// base: `p-4 hover:px-8` emits `padding-top: 16px` (kept) and
/// `padding-right: 32px` (the override) — never the 0.0 placeholder.
#[test]
fn partial_edge_states_materialize_from_base() {
    let r = resolver();
    let book = tw(&r, "p-4 hover:px-8");
    let states = book.to_states_snippet("ta");
    assert!(states.contains("hovered when ta.has-hover"), "{states}");
    assert!(
        states.contains("padding-top: 16px;"),
        "the hovered state keeps p-4's top edge: {states}"
    );
    assert!(
        states.contains("padding-right: 32px;"),
        "the hovered state takes px-8's right edge: {states}"
    );
    assert!(
        !states.contains("padding-top: 0px;"),
        "the 0.0 placeholder must never become a state binding: {states}"
    );

    // resolve_for (the flattened runtime path) agrees.
    let probe = aeolus_slint::SlintProbe {
        has_hover: true,
        ..aeolus_slint::SlintProbe::default()
    };
    let style = book.resolve_for(&probe);
    let padding = style.padding.expect("materialized padding");
    assert_eq!(
        (padding.top, padding.right, padding.bottom, padding.left),
        (16.0, 32.0, 16.0, 32.0),
    );
}
