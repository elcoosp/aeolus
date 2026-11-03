# Xilem backend prototype — implementation notes (deliverable 4)

Status: **compiles and all tests green** (`cargo test -p aeolus-xilem`:
12 passed). Code: `crates/aeolus-xilem/` (lib + widget + view, tests in
`tests/{lowering,hover,view_sugar}.rs`). Audited framework commit:
`linebender/xilem @ 9f504ae71aa2db1f64c2ecb7ee10cc877d9fc405`.

## What was built

| piece | file | purpose |
|---|---|---|
| `lower(plan) -> (PropertyStack, LoweringReport)` | `src/lib.rs` | native-cascade path: base layer + per-condition layers gated by `Selector::{with_hovered,with_active,with_disabled,with_focused}` |
| `lower_resolved(style) -> PropertySet` | `src/lib.rs` | slot→property mapping incl. oklab→sRGB conversion (`to_masonry_color`) |
| `LoweringReport` | `src/lib.rs` | every condition/slot Masonry cannot express is **reported, never silently dropped** (group/peer/dark/responsive/container/typography/margin/ring/…) |
| `StyledWidget` | `src/widget.rs` | documented fallback: plan in a field, per-state resolution at `pre_paint`, two-lane invalidation via `aeolus_core::diff` + `Slot::lane()` |
| `Tw` view + `TwViewExt::{tw, tw_centered, plan}` | `src/view.rs` | `.tw(&resolver, "p-4 bg-primary hover:bg-primary/90")` sugar; wraps any `WidgetView` |

## Lowering table (StylePlan slot → Masonry)

| plan slot | Masonry property | note |
|---|---|---|
| `background` | `Background::Color(AlphaColor<Srgb>)` | oklab→linear-sRGB→sRGB, straight alpha |
| `content_color` | `ContentColor` | |
| `padding` | `Padding { left,right,top,bottom }` | per-side, exact |
| `border_color` | `BorderColor` | |
| `border_width` | `BorderWidth::all(w)` | Masonry is uniform; Tailwind default too |
| `corner_radius` | `CornerRadius::all(max corner)` | Masonry has one radius; max keeps clipping conservative |
| `dimensions (Px/Scale)` | `Dimensions::new(Dim::Fixed, Dim::Fixed)` | Fraction/Full dropped+reported |
| `gap` | `Gap::new` | |
| `shadow` | `BoxShadow { color, offset, blur_radius }` | spread/inset dropped (upstream TODO, box_shadow.rs:10-16) |
| Hover/Active/Disabled/Focus(Visible) layers | `Selector` flag layers | pushed in ascending `VariantCondition::rank` so later (higher-rank) layers shadow |
| FocusWithin / Visited / dark / responsive / container / group / peer | — | `LoweringReport.dropped` |

## Proof results (all passing)

1. **`lower_matches_handwritten`** — `.tw("p-4 bg-primary hover:bg-primary/90
   rounded-md")` lowers to a `PropertyStack` equal (layer count, selectors,
   per-layer property values) to the hand-written stack a Masonry developer
   would produce (Padding 16px, primary #171717, hover alpha 0.9, radius 8px).
   This pins "same visuals as handwritten styles + hover layer".
2. **`hover_restyle_is_paint_only`** (TestHarness + `Recorder`) — pointer move
   onto the widget produces exactly `Update::HoveredChanged(true)` +
   `PrePaint`/`Paint` records and **zero** `Measure`/`Layout` records; the
   rendered image changes. Combined with the fact that the hover path runs
   entirely inside widget code (no view rebuild exists in a raw harness),
   this proves widget-level zero-view-rework with repaint-only restyling.
3. **`tw_view_sugar_builds_a_styled_widget_tree`** — headless
   `ViewCtx::new(noop_proxy, tokio)` builds the `Tw` view, the pod is
   attached to a real masonry tree, plan travels through the wrapper, and a
   mouse move flips the rendered pixels.
4. **`cn_rightmost_wins` / `cn_wins_rightmost`** — `cn(["px-2","px-4"])` →
   `px-4` (16px horizontal padding).

## Honest caveats

- The fallback resolves **colors at paint time** but **geometry (padding)
  from the base layer at layout time**; hover-driven padding changes request
  relayout via the diff lane, which is correct but re-runs layout (rare in
  practice).
- The stack path (`lower`) is tested at the value level; attaching a lowered
  stack to an arbitrary child from a *view* still requires upstream gap #4
  (view-side stack registration). Today's attachment paths:
  `DefaultProperties::insert_stack::<W>` (theme time) or
  `RenderRoot::insert_property_stack` (host time).
- `AeolusPlan`-as-property was tried and replaced by a plain field +
  `WidgetMut::set_plan` (KISS); the property route is unnecessary once gap #4
  lands.
- Fixed during bring-up (bugs the tests caught): `request_paint_only()` does
  not trigger `pre_paint` — cosmetic lane must call `request_pre_paint()`
  (matches `core_property_changed`'s lane for `Background`).
