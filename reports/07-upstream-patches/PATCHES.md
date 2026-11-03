# Upstream patch proposals (deliverable 7)

Target: `linebender/xilem @ 9f504ae71aa2db1f64c2ecb7ee10cc877d9fc405`.
These are **proposal diffs with proof-test sketches**, not applied patches —
each shape below is derived from the audited source (citations in
`reports/sections/xilem-masonry.md`) and is intentionally minimal. Patches
are ordered by value/effort ratio.

## Patch 1 — class-scoped group state (`group-hover:`)

*Smallest change, unlocks group-*/peer-style composition for every widget.*

Files (4) ~150 LOC total:

1. `masonry_core/src/core/class_set.rs`
   ```diff
   #[derive(Clone, Debug, Default, PartialEq)]
   pub struct ClassSet {
       pub classes: HashSet<String>,
       pub is_hovered: bool,
       pub is_active: bool,
       pub is_disabled: bool,
       pub has_focus_target: bool,
   +   /// An ancestor marked `.group` is currently hovered. Set by the
   +   /// pointer pass; consumed by `Selector::with_group_hovered`.
   +   pub is_group_hovered: bool,
   }
   ```
   Mirror into `ClassSetDiff` and `ClassSet::apply` (class_set.rs:37-56) so
   flips propagate like the other pseudo flags.
2. `masonry_core/src/core/selector.rs`
   ```diff
   pub struct Selector {
       classes: HashSet<String>,
       is_hovered: Option<bool>,
       is_active: Option<bool>,
       is_disabled: Option<bool>,
       has_focus_target: Option<bool>,
   +   is_group_hovered: Option<bool>,
   }
   + pub fn with_group_hovered(mut self, value: bool) -> Self { … }
   ```
   and extend `matches` (selector.rs:61-69) with the same `is_none_or` arm.
3. `masonry_core/src/core/property_cache.rs` — add
   `relevant_is_group_hovered` next to `relevant_is_hovered`
   (property_cache.rs:17-20) and record it in `extend_relevant` (:27-34) so
   unrelated group flips do not invalidate caches.
4. `masonry_core/src/passes/update.rs` — in the hover branch of
   `run_update_pointer_pass` (update.rs:965-1037): when `next_hovered_path`
   changes, for every widget already visited by the
   `Update::ChildHoveredChanged` chain, set `class_diff.is_group_hovered =
   true` iff any ancestor in `get_id_path` (update.rs:25-36) carries the
   `.group` class; `false` otherwise. Ancestor class lookup uses the arena
   (`is_ancestor_of` already exists, update.rs:38-47).

Proof test (masonry/src/tests/update.rs): parent with `.group` class + a
stack layer `Selector::new().with_group_hovered(true)` → `BackgroundColor`;
hover a descendant; assert the parent's resolved background changed **and**
an unrelated widget's cached property did not re-resolve (relevance check).

Fallback if rejected: Aeolus keeps the widget-level approach (`has_hovered`
is public today) — the fallback `StyledWidget` already styles ancestors
correctly for unmarked containment; only *marked* groups need the patch.

## Patch 2 — view-side property-stack registration (Q10.4)

*Removes the need for Aeolus's fallback widget entirely.*

Files (3):

1. `xilem_masonry/src/pod.rs`:
   ```diff
   impl<W: Widget> Pod<W> {
   +   pub fn with_property_stack(self, id: PropertyStackId) -> Self {
   +       self.new_widget = self.new_widget.with_property_stack(id);
   +       self
   +   }
   }
   ```
   (`NewWidget::with_property_stack` exists, widget_pod.rs:181-185.)
2. `xilem_masonry/src/view_ctx.rs` — hold an
   `Arc<Mutex<PropertyArenaWriter>>` (or a `RenderRoot` handle):
   ```diff
   pub struct ViewCtx {
       widget_map: …, proxy: …, runtime: …, environment: …,
   +   property_arena: PropertyArenaHandle,
   }
   + pub fn insert_property_stack(&mut self, stack: PropertyStack)
   +     -> PropertyStackId { self.property_arena.insert(stack) }
   ```
   (`RenderRoot::insert_property_stack` shows the invalidation duty,
   render_root.rs:1065-1068.)
3. New view `xilem_masonry/src/view/stack_layer.rs` — a wrapper view whose
   `build` inserts the stack and marks the child pod; `rebuild` diffs layers
   and calls `RenderRoot::replace_property_stack` (render_root.rs:1032-1042).

Proof test: build `text("x").stack_layer(&stack)` headlessly (same pattern
as our `view_sugar` test), attach to a `TestHarness`, hover, assert the
stack-gated property applied — i.e. exactly our
`lower_matches_handwritten` values rendered through the REAL cascade.

## Patch 3 — `FontSize`/`FontWeight` properties (Q10.3)

Files: new `masonry_core/src/properties/font_size.rs` / `font_weight.rs`
(`impl Property`, `static_default` mirroring corner_radius.rs:17-24);
register in `masonry_core/src/properties/mod.rs:6-24`; route to
`ctx.request_layout()` in `core_property_changed` (mod.rs:29-42);
translate to parley in `Label::property_changed` (pattern exists for
`LineBreaking`, masonry/src/widgets/label.rs:486-489) via
`self.styles.insert(StyleProperty::FontSize(..))`; theme defaults in
`masonry/src/theme.rs`. `Update::FontsChanged` (update.rs:1196-1249) is the
existing broadcast hook for global font changes.

Proof test: insert `FontSize(20.0)` property on a Label inside a
`TestHarness`; assert measure changes and the parley `StyleSet` picked it
up (compare against `Label::insert_style(StyleProperty::FontSize(20.0))`).

## Patch 4 — window-size class flag (Q10.2)

Minimal: add `window_size: Size` to `RenderRootState`/`GlobalState`, set by
the host on resize; emit `Update::WindowResized(Size)` from a new branch in
`passes/update.rs`; widgets with breakpoint layers register relevance
(`relevant_window_width`) like pseudo flags. Optional sugar:
`Selector::min_width(px)`.
No-patch alternative (works today, documented in the capability matrix):
root `resize_observer` view → app state branch
(xilem_masonry/src/view/resize_observer.rs:55-70).

Proof test: set window size below/above a breakpoint; assert the
`Selector::min_width` layer flips without any widget rebuild.

## What we deliberately do NOT propose

- a CSS engine in Masonry (Selector stays a flat struct — the design is
  intentional and cheap);
- per-property wrapper views (briefing §3.6 forbids them);
- Environment-based theming (Q9: listener machinery is known-broken
  upstream — that is an upstream fix of its own, tracked separately).
