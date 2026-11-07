//! Lowering correctness for the Floem backend.
//!
//! Floem's `Style::apply_selectors(&[StyleSelector])` is the public
//! cascade-evaluation entry point, so these tests resolve the lowered style
//! under simulated widget states and assert the *effective* property values —
//! the same thing a running app would compute.

use floem::peniko::{Brush, Color as PenikoColor};
use floem::style::unit::Length;
use floem::style::{
    Background, LineHeight, LineHeightValue, PaddingLeft, StyleSelector, TextColor,
};

use aeolus_core::color::Color;
use aeolus_core::{Resolver, shadcn_default};
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
    let r = resolver.resolve("p-4 bg-primary rounded-md").unwrap();
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
    assert!(
        report.is_empty(),
        "focus-within and disabled must map: {report:?}"
    );

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
    let r = resolver
        .resolve("bg-primary dark:bg-black text-primary dark:text-white")
        .unwrap();
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
    assert!(
        !whats.iter().any(|w| w.starts_with("responsive")),
        "{report:?}"
    );
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

/// Phase-2 audit item: is floem's `Length::Pt` a CSS point (1/72 inch) or a
/// logical pixel? Verdict from floem @ 1351ffb `src/style/unit.rs`: **Pt is
/// the logical-pixel unit** — "Floem treats absolute lengths as logical
/// layout points rather than device pixels" and `Length::to_taffy` passes
/// the value through unscaled (`Length::Pt(v) => LengthPercentage::length(v)`);
/// the renderer applies the window scale factor later, like every UI toolkit.
///
/// So `p-4` = 16 logical px = 16 device px on a 96dpi (scale-factor 1)
/// display. This test PINS that behavior; if a floem upgrade ever turned Pt
/// into a true CSS point, `p-4` would render 16pt = 21.33px and both
/// asserts below would fail.
#[test]
fn pt_is_logical_pixel_not_css_point() {
    // (a) End-to-end: the lowered `p-4` padding survives the taffy
    //     conversion as exactly 16 logical px (NOT 16 CSS pt = 21.33px).
    let resolver = resolver();
    let r = resolver.resolve("p-4").unwrap();
    let (style, _) = lower(&r.plan);
    let pad: Length = style.get(PaddingLeft);
    let cx = floem::style::FontSizeCx::new(16.0, 16.0);
    assert_eq!(
        pad.to_taffy(&cx),
        taffy::style::LengthPercentage::length(16.0),
        "p-4 must reach taffy as 16 logical px"
    );

    // (b) A real taffy layout at scale 1: a 64px box with this padding has
    //     a 16px resolved padding per side and a 32px content box
    //     (a CSS-point regression would give 21.33px / 42.67px - 64px...).
    let mut tree: taffy::TaffyTree = taffy::TaffyTree::new();
    let leaf = tree
        .new_leaf(taffy::style::Style {
            size: taffy::prelude::Size {
                width: taffy::prelude::length(64.0_f32),
                height: taffy::prelude::length(64.0_f32),
            },
            padding: taffy::prelude::Rect {
                left: pad.to_taffy(&cx),
                right: pad.to_taffy(&cx),
                top: pad.to_taffy(&cx),
                bottom: pad.to_taffy(&cx),
            },
            ..Default::default()
        })
        .unwrap();
    tree.compute_layout(
        leaf,
        taffy::prelude::Size {
            width: taffy::prelude::AvailableSpace::Definite(200.0),
            height: taffy::prelude::AvailableSpace::Definite(200.0),
        },
    )
    .unwrap();
    let layout = tree.layout(leaf).unwrap();
    assert!(
        (layout.padding.left - 16.0).abs() < 1e-3,
        "layout at 96dpi → 16px pad; got {}",
        layout.padding.left
    );
    assert!(
        (layout.content_size.width - 32.0).abs() < 1e-3,
        "64px box minus 2x16px pad → 32px content; got {}",
        layout.content_size.width
    );
}

/// Arbitrary selector variants (v0.3 §1.2): `[&:hover]` is byte-identical
/// CSS to `hover:` in v4.1.17, so it canonicalizes onto the same NATIVE
/// hover selector map; compound forms land in `rules` and ride the
/// reactive probe path (`FloemProbe` + `resolve_rules_only`).
#[test]
fn arbitrary_self_hover_lowers_native_and_compound_via_probe() {
    use aeolus_floem::FloemProbe;

    let resolver = resolver();

    // [&:hover] ≡ hover: — the native lane.
    let r = resolver.resolve("bg-primary [&:hover]:bg-red-500").unwrap();
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    assert!(
        r.plan.rules.is_empty(),
        "[&:hover] canonicalizes onto the hover layer bucket"
    );
    let (style, report) = lower(&r.plan);
    assert!(report.is_empty(), "{report:?}");
    let hovered = style.apply_selectors(&[StyleSelector::Hover]);
    let hover_bg = brush_rgba(hovered.get(Background).as_ref().unwrap()).unwrap();
    let expected = Color::oklch(0.637, 0.237, 25.331, 1.0).to_rgba8();
    assert_eq!(hover_bg, expected, "[&:hover]:bg-red-500 resolves on hover");

    // [&:hover:focus] — a conjunction the buckets cannot express: a rule,
    // emulated through the probe.
    let r = resolver.resolve("[&:hover:focus]:bg-black").unwrap();
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    assert_eq!(r.plan.rules.len(), 1, "the compound lands in plan.rules");
    let both = FloemProbe {
        hovered: true,
        focused: true,
        ..Default::default()
    };
    let extra = r.plan.resolve_rules_only(&both);
    assert_eq!(
        extra.background.as_ref().unwrap().0.to_rgba8(),
        [0, 0, 0, 255],
        "[&:hover:focus] fires when both states hold"
    );
    let idle = r.plan.resolve_rules_only(&FloemProbe::default());
    assert!(idle.background.is_none(), "…and not otherwise");
}

/// v0.5 §1.4 (AEOLUS_V0.5.md): the `--leading-*` token form is
/// font-size-RELATIVE — Floem lowers it to the NATIVE
/// `LineHeightValue::Normal(multiplier)` (resolved against the element's
/// font-size at paint, exactly CSS's unitless line-height); absolute
/// forms stay `Pt`. Letter-spacing keeps its honest drop report.
#[test]
fn leading_tokens_lower_to_native_multipliers() {
    let r = resolver().resolve("leading-tight").unwrap();
    let (style, report) = lower(&r.plan);
    assert_eq!(
        style.get(LineHeight),
        LineHeightValue::Normal(1.25),
        "leading-tight must reach Floem as the font-size-relative multiplier"
    );
    assert!(report.dropped.is_empty(), "{report:?}");

    // Absolute forms stay absolute.
    let r = resolver().resolve("leading-6").unwrap();
    let (style, _) = lower(&r.plan);
    assert_eq!(style.get(LineHeight), LineHeightValue::Pt(24.0));
    let r = resolver().resolve("leading-[3px]").unwrap();
    let (style, _) = lower(&r.plan);
    assert_eq!(style.get(LineHeight), LineHeightValue::Pt(3.0));

    // `tracking-*` keeps the honest drop (no Floem letter-spacing prop);
    // the relative em stays in the plan.
    let r = resolver().resolve("tracking-tight").unwrap();
    let (_style, report) = lower(&r.plan);
    assert!(
        report
            .dropped
            .iter()
            .any(|(what, _)| what.contains("letter-spacing")),
        "tracking-tight must be reported, never silent: {report:?}"
    );
}

// ---------------------------------------------------------------------------
// v0.6 (AEOLUS_V0.6.md §2) — the unwired native surface at the pinned rev
// (which IS latest floem main): overflow, selectable, whitespace,
// align_content, grid, border dashes.
// ---------------------------------------------------------------------------

use aeolus_core::TailwindTheme;

fn v4_resolver() -> Resolver<'static> {
    let theme = TailwindTheme::default();
    Resolver::new(Box::leak(Box::new(theme)))
}

#[test]
fn overflow_lowers_to_per_axis_taffy_props() {
    use floem::style::{OverflowX, OverflowY};
    use floem::taffy::style::Overflow;

    let r = v4_resolver()
        .resolve("overflow-hidden overflow-x-auto")
        .unwrap();
    let (style, report) = lower(&r.plan);
    assert!(report.is_empty(), "{report:?}");
    assert_eq!(
        style.get(OverflowX),
        Overflow::Scroll,
        "overflow-auto → Scroll (documented)"
    );
    assert_eq!(
        style.get(OverflowY),
        Overflow::Hidden,
        "overflow-hidden keeps y"
    );
}

#[test]
fn select_none_lowers_to_the_selectable_prop() {
    use floem::style::Selectable;

    let r = v4_resolver().resolve("select-none").unwrap();
    let (style, report) = lower(&r.plan);
    assert!(report.is_empty(), "{report:?}");
    assert!(!style.get(Selectable));

    let r = v4_resolver().resolve("select-text").unwrap();
    let (style, report) = lower(&r.plan);
    assert!(report.is_empty(), "{report:?}");
    assert!(style.get(Selectable));

    // No selection-mode prop: precise report, not silence.
    let r = v4_resolver().resolve("select-all").unwrap();
    let (_style, report) = lower(&r.plan);
    assert!(
        report
            .dropped
            .iter()
            .any(|(w, rsn)| w.contains("select-all") && rsn.contains("selection-mode")),
        "{report:?}"
    );
}

#[test]
fn whitespace_maps_the_single_text_overflow_prop() {
    use floem::style::{TextOverflow, TextOverflowProp};

    // normal → the native Wrap form.
    let r = v4_resolver().resolve("whitespace-normal").unwrap();
    let (style, report) = lower(&r.plan);
    assert!(report.is_empty(), "{report:?}");
    assert_eq!(
        style.get(TextOverflowProp),
        TextOverflow::Wrap {
            overflow_wrap: floem::text::OverflowWrap::Normal,
            word_break: floem::text::WordBreakStrength::Normal,
        }
    );

    // nowrap is floem's DEFAULT text flow — a genuine no-op (no drop).
    let r = v4_resolver().resolve("whitespace-nowrap").unwrap();
    let (_style, report) = lower(&r.plan);
    assert!(
        report.is_empty(),
        "nowrap must be a documented no-op: {report:?}"
    );

    // pre* needs whitespace preservation — precise report.
    let r = v4_resolver().resolve("whitespace-pre").unwrap();
    let (_style, report) = lower(&r.plan);
    assert!(
        report
            .dropped
            .iter()
            .any(|(w, rsn)| w.contains("whitespace-pre") && rsn.contains("preservation")),
        "{report:?}"
    );
}

#[test]
fn content_align_lowers_to_taffy_align_content() {
    use floem::style::AlignContentProp;

    let r = v4_resolver().resolve("content-between").unwrap();
    let (style, report) = lower(&r.plan);
    assert!(report.is_empty(), "{report:?}");
    assert_eq!(
        style.get(AlignContentProp),
        Some(floem::style::AlignContent::SpaceBetween)
    );

    // CSS `align-content: normal` resolves to stretch for flex containers.
    let r = v4_resolver().resolve("content-normal").unwrap();
    let (style, report) = lower(&r.plan);
    assert!(report.is_empty(), "{report:?}");
    assert_eq!(
        style.get(AlignContentProp),
        Some(floem::style::AlignContent::Stretch)
    );

    // taffy has no baseline align-content — reported.
    let r = v4_resolver().resolve("content-baseline").unwrap();
    let (_style, report) = lower(&r.plan);
    assert!(
        report
            .dropped
            .iter()
            .any(|(w, _)| w.contains("content-baseline")),
        "{report:?}"
    );
}

#[test]
fn grid_lowers_templates_and_placement_natively() {
    use floem::style::{GridColumn, GridTemplateColumns};

    let r = v4_resolver().resolve("grid grid-cols-4").unwrap();
    let (style, report) = lower(&r.plan);
    assert!(report.is_empty(), "{report:?}");
    let templates = style.get(GridTemplateColumns);
    assert_eq!(templates.len(), 1, "one repeat() component");
    let floem::taffy::style::GridTemplateComponent::Repeat(rep) = &templates[0] else {
        panic!("expected a repeat component: {templates:?}");
    };
    assert_eq!(rep.count, floem::taffy::style::RepetitionCount::Count(4));
    assert_eq!(rep.tracks.len(), 1, "one 1fr track");

    // Item placement: col-span-2 col-start-3 → Line { start: Line(3), end: Span(2) }.
    let r = v4_resolver().resolve("col-span-2 col-start-3").unwrap();
    let (style, report) = lower(&r.plan);
    assert!(report.is_empty(), "{report:?}");
    use floem::taffy::prelude::{TaffyGridLine, TaffyGridSpan};
    let placement = style.get(GridColumn);
    assert_eq!(
        placement.start,
        floem::taffy::style::GridPlacement::from_line_index(3)
    );
    assert_eq!(
        placement.end,
        floem::taffy::style::GridPlacement::from_span(2)
    );

    // Bare span → Line { start: Span(2), end: Auto }.
    let r = v4_resolver().resolve("col-span-2").unwrap();
    let (style, _) = lower(&r.plan);
    let placement = style.get(GridColumn);
    assert_eq!(
        placement.start,
        floem::taffy::style::GridPlacement::from_span(2)
    );
    assert_eq!(placement.end, floem::taffy::style::GridPlacement::Auto);
}

#[test]
fn border_style_lowers_to_kurbo_stroke_patterns() {
    use floem::style::{BorderLeft, BorderTop};

    // border-2 border-dashed → dashed strokes at 2px.
    let r = v4_resolver().resolve("border-2 border-dashed").unwrap();
    let (style, report) = lower(&r.plan);
    assert!(report.is_empty(), "{report:?}");
    let stroke = style.get(BorderLeft);
    assert_eq!(stroke.width, 2.0);
    assert_eq!(
        stroke.dash_pattern.as_ref(),
        &vec![6.0, 6.0],
        "the documented dash pattern"
    );
    let top = style.get(BorderTop);
    assert_eq!(top.dash_pattern.as_ref(), &vec![6.0, 6.0]);

    // border-dotted → round-cap dot pattern.
    let r = v4_resolver().resolve("border-2 border-dotted").unwrap();
    let (style, _) = lower(&r.plan);
    let stroke = style.get(BorderLeft);
    assert_eq!(stroke.dash_pattern.as_ref(), &vec![0.01, 4.0]);
    assert_eq!(stroke.start_cap, floem::peniko::kurbo::Cap::Round);

    // border-none zeroes the strokes even with a width class (CSS semantics).
    let r = v4_resolver().resolve("border-2 border-none").unwrap();
    let (style, report) = lower(&r.plan);
    assert!(report.is_empty(), "{report:?}");
    assert_eq!(style.get(BorderLeft).width, 0.0);

    // double has no kurbo primitive — reported.
    let r = v4_resolver().resolve("border-double").unwrap();
    let (_style, report) = lower(&r.plan);
    assert!(
        report
            .dropped
            .iter()
            .any(|(w, _)| w.contains("border-double")),
        "{report:?}"
    );
}

#[test]
fn gradients_report_with_the_sharpened_reason() {
    let r = v4_resolver()
        .resolve("bg-linear-to-r from-red-500 to-blue-500")
        .unwrap();
    let (_style, report) = lower(&r.plan);
    assert!(
        report
            .dropped
            .iter()
            .any(|(w, rsn)| w.contains("gradient") && rsn.contains("geometry")),
        "the drop must carry the precise geometry reason: {report:?}"
    );
    // bg-none lowers to nothing, silently-correctly (the CSS `none` value).
    let r = v4_resolver().resolve("bg-none").unwrap();
    let (style, report) = lower(&r.plan);
    assert!(report.is_empty(), "{report:?}");
    assert!(style.get(Background).is_none());
}

#[test]
fn slot_coverage_upgrades_are_consistent_with_the_lowering() {
    use aeolus_core::Slot;
    use aeolus_floem::{Coverage, slot_coverage};

    let r = v4_resolver().resolve("overflow-hidden select-none whitespace-normal grid grid-cols-2 content-between border-dashed").unwrap();
    let cov = slot_coverage(&r.plan.base);
    for (slot, coverage) in cov {
        match slot {
            Slot::Overflow
            | Slot::Grid
            | Slot::AlignContent
            | Slot::BorderStyle
            | Slot::Text(aeolus_core::TextProperty::Select)
            | Slot::Text(aeolus_core::TextProperty::Whitespace) => {
                assert_eq!(coverage, Coverage::Native, "{slot:?}");
            }
            _ => {}
        }
    }
}
