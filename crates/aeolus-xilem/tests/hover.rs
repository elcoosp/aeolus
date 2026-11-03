//! The hover proof: hovering a `StyledWidget` changes the painted
//! background **without any relayout and without any view rebuild**.
//!
//! Instrumentation: masonry's `Recorder` widget wraps our `StyledWidget` and
//! records every Widget method call. After the harness settles (initial
//! layout flushed), a pointer move onto the widget must produce exactly:
//! - `Update(HoveredChanged(true))` — the state flip, and
//! - `PrePaint`/`Paint` — the repaint,
//! and NOT `Measure`/`Layout` (a relayout would mean the restyle disturbed
//! geometry, violating the two-lane invalidation contract).

use masonry::core::{NewWidget, Widget as _};
use masonry_testing::{Record, Recording, Recorder, TestHarness};

use aeolus_core::{shadcn_default, Resolver};
use aeolus_xilem::widget::{StyledWidget, resolved_for_state};

fn plan() -> aeolus_core::StylePlan {
    let theme = shadcn_default().unwrap();
    let resolver = Resolver::new(&theme);
    resolver
        .resolve("p-4 bg-primary hover:bg-primary/90 rounded-md")
        .unwrap()
        .plan
}

#[test]
fn hover_restyle_is_paint_only() {
    let recording = Recording::default();
    let styled = StyledWidget::new(masonry::widgets::Label::new("Save").prepare())
        .with_plan(plan());
    // The Recorder wraps the StyledWidget, so every StyledWidget method call
    // is recorded. The recorder itself is the root.
    let root = Recorder::new(styled, &recording);
    let mut harness = TestHarness::create(Default::default(), NewWidget::new(root));

    // Settle the initial layout/paint, then flush the recorder.
    let before = harness.render();
    while recording.next().is_some() {}

    // Window is 120x60; the StyledWidget fills it, the label sits inside the
    // 16px padding. Hovering the label therefore hovers the styled box
    // (CSS `:hover` containment semantics — the wrapper uses has_hovered).
    harness.mouse_move(masonry::kurbo::Point::new(60., 30.));

    // The harness paints on demand: render() performs the repaint pass that
    // the hover flip scheduled, then we drain what it recorded.
    let after = harness.render();
    let mut saw_hover_flip = false;
    let mut relayout_records = Vec::new();
    let mut saw_repaint = false;
    while let Some(record) = recording.next() {
        match record {
            Record::Update(masonry::core::Update::HoveredChanged(true)) => {
                saw_hover_flip = true;
            }
            Record::PrePaint | Record::Paint | Record::PostPaint => saw_repaint = true,
            Record::Measure(_) | Record::Layout(_) => relayout_records.push(record),
            _ => {}
        }
    }
    assert!(saw_hover_flip, "expected a HoveredChanged(true) update");
    assert!(saw_repaint, "expected a repaint after the hover flip");
    assert!(
        relayout_records.is_empty(),
        "hover changed only the background; got {relayout_records:?}"
    );

    // And the repaint really changed the pixels (hover effect is visible).
    assert_ne!(before, after, "hover must change the rendered output");
}

#[test]
fn resolved_state_matches_stack_path() {
    // The fallback's per-state resolution must agree with the stack path for
    // the canonical states: base (nothing set) and hover.
    let p = plan();
    let base = resolved_for_state(&p, false, false, false, false);
    let hovered = resolved_for_state(&p, true, false, false, false);

    let base_bg = base.background.as_ref().unwrap().0;
    let hover_bg = hovered.background.as_ref().unwrap().0;
    assert!((base_bg.alpha - 1.0).abs() < 1e-4);
    assert!((hover_bg.alpha - 0.9).abs() < 1e-4);
    // Padding does not change on hover here: the two lanes stay clean.
    assert_eq!(base.padding, hovered.padding);
}

#[test]
fn active_wins_over_hover() {
    // v4 cascade intent: active (rank 4) beats hover (rank 0).
    let theme = shadcn_default().unwrap();
    let resolver = Resolver::new(&theme);
    let p = resolver
        .resolve("bg-primary hover:bg-blue-500 active:bg-black")
        .unwrap()
        .plan;
    let both = resolved_for_state(&p, true, true, false, false);
    let black = aeolus_core::Color::from_srgb(0.0, 0.0, 0.0, 1.0);
    let [r, g, b, _] = both.background.unwrap().0.to_srgba();
    let [br, bg_, bb, _] = black.to_srgba();
    assert!((r - br).abs() < 1e-3 && (g - bg_).abs() < 1e-3 && (b - bb).abs() < 1e-3);
}

