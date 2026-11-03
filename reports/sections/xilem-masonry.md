# AEOLUS audit — Xilem / Masonry properties, state & styling surface

- Repo: `linebender/xilem` @ `main` (shallow clone), `/home/z/my-project/repos/xilem`
- Verified commit: `9f504ae71aa2db1f64c2ecb7ee10cc877d9fc405` ("perf: Allow early-exiting `clear_layout_flags` (#1861)")
- Date of audit: 2026 (file headers in tree say 2025/2026; workspace layout matches the post-split
  crate structure: `masonry_core` = engine, `masonry` = widgets/theme re-exporting `masonry_core`
  as `masonry::core` (`masonry/src/lib.rs:223`), `xilem_masonry` = view layer).
- Method: read-only. All `file:line` citations below were actually read at the above commit.
- Important structural note vs. the audit brief: the crates have been reshuffled. Widgets now live in
  `masonry/src/widgets/` (not `masonry_core/src/widgets/`), and property *types* live in
  `masonry_core/src/properties/` + `masonry/src/properties/` (there are none in `property_set.rs`).

---

## Q1 — PropertyStack layer registration path from a view

**Short answer: a Xilem view CANNOT register a property-stack layer. The `prop` view sets a *local
property* (a `PropertySet` entry on the child widget), not a stack layer. Stack layers are created
only by the theme (`DefaultProperties::insert_stack`), by the host via `RenderRoot::insert_property_stack`,
or by attaching a whole pre-registered stack id to a `NewWidget` with `with_property_stack`.**

### Construction and shape

- `PropertyStack` is `Vec<(Selector, PropertySet)>` — `masonry_core/src/core/property_stack.rs:20-23`.
  There are **no layer "kinds"** (no `Default`/`Fallback`/`State` variants). Every layer is a
  `(Selector, PropertySet)` pair; "default/fallback" behavior comes from (a) layers with an empty
  always-matching `Selector`, (b) `DefaultProperties`, and (c) `Property::static_default()`.
- Layer APIs: `push_layer` (`property_stack.rs:64-66`), `insert_layer` (`:81-88`),
  `remove_layer` (`:76-78`), `get_layer_mut` (`:71-73`), `get_layers` (`:91-93`).
- `insert_property` does **NOT FOUND** anywhere in the repo; the only symbol is
  `RenderRoot::insert_property_stack` (`masonry_core/src/app/render_root.rs:1066-1068`), which
  inserts a whole stack and returns its id. Value-level inserts are `PropertySet::insert`
  (`property_set.rs:41-43`), `PropertiesMut::insert` (`properties_mut.rs:56-58`) and
  `WidgetMut::insert_prop` (`widget_mut.rs:95-101`).
- `PropertyArena` is **not** a per-widget map: it is `HashMap<PropertyStackId, PropertyStack>` plus
  an `Arc<DefaultProperties>` (`masonry_core/src/core/property_arena.rs:10-15`). The widget→stack
  link is a single `Option<PropertyStackId>` stored in `WidgetState.property_stack_id`
  (`widget_state.rs:265`) and carried on `NewWidget.property_stack_id` (`widget_pod.rs:42`), set via
  `NewWidget::with_property_stack` (`widget_pod.rs:181-185`). `PropertyArena::get(id, widget_type)`
  falls back to `default_properties.stack_for_widget(widget_type)` (`property_arena.rs:38-46`), and
  `DefaultProperties` keeps per-widget-type default stacks (`default_properties.rs:22, 48-50, 56-58`).

### What the `prop` view actually does (lifecycle)

- Build time: `Prop::build` inserts the value into the **child's `NewWidget.properties` local set** —
  `xilem_masonry/src/view/prop.rs:35-42` (`child_pod.new_widget.properties.insert(self.property.clone())`).
  The value reaches the arena when the parent registers the child: `RegisterCtx::register_child`
  destructures `NewWidget { properties, property_stack_id, classes, .. }` and stores them on the
  `WidgetArenaNode` (`masonry_core/src/core/contexts.rs:2090-2142`).
- Update (rebuild) time: `Prop::rebuild` diffs `self.property != prev.property` (plus a guard against
  stacked `Prop` views via `ViewCtx::mark_prop_changed`/`prop_has_changed`,
  `xilem_masonry/src/view_ctx.rs:88-125`) and calls `element.insert_prop(...)` (`prop.rs:63-66`),
  i.e. `WidgetMut::insert_prop` → `core_property_changed` + `widget.property_changed`
  (`widget_mut.rs:95-101`).
- So: layers are never added by views; only local props (which **shadow the entire stack**, see
  resolution order below). Confirmed by grep: `PropertyStack`, `with_property_stack`,
  `insert_property_stack`, `Selector`, `property_stack_id` have **zero** occurrences in
  `xilem_masonry/`.
- The `Style` extension trait (`xilem_masonry/src/style.rs:21-159`: `.background()`, `.border()`,
  `.padding()`, `.dims()`, `.color()`, …) is pure sugar over `WidgetView::prop`
  (`xilem_masonry/src/widget_view.rs:73-86`), which builds `Prop` views. The repo even has a
  compile-stress test for deeply-chained style views (`xilem/stress_tests/property_stack.rs:14-25`) —
  note the name refers to *view* stacking, not Masonry `PropertyStack`.

### Where stack layers CAN be added today

1. Theme time: `DefaultProperties::insert_stack::<W>(stack)` (`default_properties.rs:48-50`), used
   extensively in `masonry/src/theme.rs` (e.g. Button at `theme.rs:84-103`).
2. Host time (render root, outside the widget tree): `RenderRoot::insert_property_stack`
   (`render_root.rs:1065-1068`), `replace_property_stack` (`:1032-1042`),
   `remove_property_stack` (`:1054-1063`) — each invalidates resolution and re-runs rewrite passes;
   plus `RenderRoot::set_default_properties` for whole-tree theme swap (`:425-449`).
3. Widget-code time: `MutateCtx::set_property_stack(stack_id)` (`contexts.rs:353-359`) sets
   `request_update_props`, `needs_update_props`, invalidates the cache and swaps the id.

### Selector & evaluation

- `Selector` is **not** an enum with `Widget`/`If`/`And` variants — **NOT FOUND** as such. It is a
  struct: `classes: HashSet<String>` + four `Option<bool>` pseudo-flags `is_hovered`, `is_active`,
  `is_disabled`, `has_focus_target` (`masonry_core/src/core/selector.rs:12-19`), with builders
  `classes()` (`:28-34`), `with_hovered` (`:37-40`), `with_active` (`:43-46`), `with_disabled`
  (`:49-52`), `with_focused` (`:55-58`). `matches()` is a subset test on classes plus flag equality
  (`:61-69`); no ancestor/descendant/sibling combinators exist.
- Evaluated against the widget's own `ClassSet` (`class_set.rs:8-14`: classes + the same four bools).
- `property_cache.rs` evaluation: `PropertyStack::resolve` walks layers in reverse (top of vec wins —
  shadowing), calls `cache.extend_relevant(selector)` on every layer it passes
  (`property_stack.rs:128-137`; `property_cache.rs:27-34` records which class strings and pseudo-flags
  influenced *any* cached resolution), then caches `TypeId -> Option<layer index>` in
  `PropertyCache.entries` (`property_stack.rs:134, 139`). `cached_index` returns nothing while
  `invalidated` is set (`property_cache.rs:40-46`). Relevance gating = the cache remembers *which*
  classes/flags mattered, so irrelevant class churn doesn't re-resolve (used in
  `passes/update.rs:1164-1179`).
- Final resolution order (`properties_ref.rs:32-47`, mirrored in `properties_mut.rs:32-47`):
  **1. local `PropertySet` → 2. property stack → 3. default map for widget type → 4. `Property::static_default()`.**
  A local prop set by a view therefore beats every stack layer.

---

## Q2 — group-hover / peer mapping

**What exists today**

- The flags exist and are exactly what the audit brief hoped for, but with *descendant* semantics:
  - `WidgetState.has_hovered` — `masonry_core/src/core/widget_state.rs:249` ("In the hovered path,
    starting from window and ending at the hovered widget. Descendants of the hovered widget are not
    in the hovered path."); `is_hovered` at `:251`; `has_active` at `:255`; `is_active` at `:257`;
    `has_focus_target` at `:261`.
  - Context accessors on all context types: `is_hovered` `contexts.rs:1333-1335`, `has_hovered`
    `:1340-1342`, `is_active` `:1351-1353`, `has_active` `:1358-1360`, `has_focus_target` `:1388-1390`.
- `has_hovered` means "this widget is an *ancestor of* (or is) the hovered widget". That is
  CSS-`:hover`-on-self-and-descendants — **not** group-hover (which needs "a *marked ancestor* is
  hovered" — a weaker condition that also fires for ancestors of the group container, and stronger in
  that it must be scoped to a designated "group").
- Hover is computed per pointer pass as a path, not propagated via `merge_up`: `run_update_pointer_pass`
  builds `next_hovered_path = get_id_path(root, next_hovered_widget)` (`passes/update.rs:965-967`,
  stored back at `:1090`), diffs against `prev_hovered_path` (`:970-973`), then for each widget in
  either path sets `has_hovered` and fires `Update::ChildHoveredChanged` (`:989-1000`; flag on
  `WidgetState` via `hovered_set.contains`). The hovered widget itself gets
  `class_diff.is_hovered = Some(..)` + `request_update_props`/`needs_update_props` (`:1023-1037`).
  Active is symmetrical (`:863-937`, `Update::ChildActiveChanged` at `:898`). Focus:
  `update_focused_status_of` sets `class_diff.has_focus_target` (`:711-725`).
  `WidgetState::merge_up` (`widget_state.rs:370-384`) propagates only `needs_*` flags, **not**
  hover/active status.
- Classes: `ClassSet` is per-widget only (`class_set.rs:8-14`); `add_class`/`remove_class`
  (`contexts.rs:1675-1690`) mutate the widget's **own** set and queue a props update. There is no API
  to query an ancestor's class set, and `Selector::matches` (`selector.rs:61-69`) tests only the
  widget's own `ClassSet`. **No ancestor/sibling/peer machinery exists.**
- Widget-state naming scheme (`widget_state.rs:33-37`): `is_xxx` = self, `has_xxx` = self-or-descendant.

**Minimal upstream patch shape for group/peer**

1. `masonry_core/src/core/class_set.rs`: add `is_group_hovered: bool` (or `hovered_groups:
   HashSet<String>` for named groups) to `ClassSet` and `ClassSetDiff`, applied in `ClassSet::apply`
   (`class_set.rs:37-56`).
2. `masonry_core/src/core/selector.rs`: add `with_group_hovered(name)`; extend `matches` (`:61-69`).
3. `masonry_core/src/core/property_cache.rs`: add `relevant_is_group_hovered` next to
   `relevant_is_hovered` (`property_cache.rs:17-20`) and record it in `extend_relevant` (`:27-34`).
4. `masonry_core/src/passes/update.rs`: in the hover branch of `run_update_pointer_pass`
   (`:940-1037`), when the hovered path changes, walk the id paths (the arena already supports
   ancestor walks, see `get_id_path` `:25-36` and `is_ancestor_of` `:38-47`) and set
   `class_diff.is_group_hovered` on every widget that has a `.group`-classed widget among its
   ancestors. Because `has_hovered` already fires `Update::ChildHoveredChanged` on the whole ancestor
   chain (`:997-998`), the cheapest variant piggybacks there: the widget checks
   `ctx`-side ancestor classes. Peer: needs a designated sibling — a `WidgetTag`-style handle or a
   `peer` class resolved through the parent's children (`children` lists already exist in
   `WidgetArenaNode`); set the peer's `class_diff` from the same pass.
5. No changes needed to `property_stack.rs` resolution — selectors just gain a new predicate.

---

## Q5a — Text / typography properties

- `FontSize` / `FontWeight` / `LetterSpacing` as Masonry `Property` types: **NOT FOUND**. Grep across
  `masonry_core/src` for `FontSize|FontWeight|LetterSpacing` matches only nothing; the only text-style
  types are parley aliases: `pub type StyleProperty = parley::StyleProperty<'static, BrushIndex>` and
  `pub type StyleSet = parley::StyleSet<BrushIndex>` (`masonry_core/src/core/text.rs:29-32`).
- Widgets get font settings through parley, not properties: `Label` stores `styles: StyleSet`
  (`masonry/src/widgets/label.rs:52`), initialized with `StyleSet::new(theme::TEXT_SIZE_NORMAL)` and
  `default_text_styles` (`label.rs:179-183`; `theme.rs:43` `TEXT_SIZE_NORMAL = 15.0`;
  `theme.rs:416-419` inserts `LineHeight(1.2)` and `GenericFamily::SystemUi`).
- Styling API: `Label::with_style(StyleProperty)` (`label.rs:203`), builder-style
  `with_style`/`remove_style` at `:203-226`, `WidgetMut`-level `Label::insert_style`/`retain_styles`
  (`label.rs:286-322`; usage `label.rs:701`, `:847` `StyleProperty::FontSize(20.0)`). Font size is
  documented as reachable only via parley: "To change the font size, use `with_style`, setting
  `StyleProperty::FontSize`" (`label.rs:180`).
- What the property system *does* cover for text: `ContentColor` (`masonry/src/properties/content_color.rs:26-30`,
  theme default `theme.rs:317`) and `LineBreaking` (`masonry/src/properties/line_breaking.rs:10-19`;
  `impl UsesProperty<LineBreaking> for Label` at `label.rs:474`; consumed in measure/`max_advance`
  `label.rs:517-547`).
- Global font changes have a dedicated `Update::FontsChanged` broadcast pass
  (`passes/update.rs:1196-1249`) — the natural hook point if `FontSize` became a property.

---

## Q9 — Environment snapshot (xilem_core)

- Structure: `Environment` is a slot array with a type-index map: `slots: Vec<Slot>`,
  `free_slots: Vec<u32>`, `types: HashMap<TypeId, u32>` (`xilem_core/src/environment.rs:58-66`).
  One slot per `Resource` type (not scoped per subtree — scopes are emulated by `Provides` swapping
  values in/out around child build/rebuild/teardown/message).
- Writing: `provides(...)` (`environment.rs:156-172`) creates a `Provides` view; `build` stores an
  `EnvironmentItem` into the slot and swaps the old value back out afterwards (`:216-260`); `rebuild`
  and `message` do the same swap dance (`:262-294`, `:324-355`).
- Reading: `with_context(...)` (`:376-390`) — `build` looks up the slot, registers the current view
  path as a **change listener** in `value.change_listeners: Vec<Option<Arc<[ViewId]>>>`
  (`:34`, registration `:466-501`), then calls the user's closure with `downcast_mut::<Context>()`
  (`:461-503`). `rebuild` re-reads the value and rebuilds the child view (`:516-550`).
- Listener mechanism: **known-broken / non-functional.** The field comment says it plainly:
  `// TODO: The change listeners currently aren't ever notified.` (`environment.rs:32-33`), and in
  `WithContext::build`: `// Note also that there is currently no way to trigger these views!`
  (`:478`). A `Rebuild` message type exists (`:18-22`) and `WithContext::message` handles it by
  returning `MessageResult::RequestRebuild` (`:578-585`), but **nothing in the repo sends `Rebuild`**.
  Listener GC is also a TODO (`:558-562`), and `queued_rebuilds` is commented out (`:64-65`).
- `memoize` interplay: **does not correctly re-run on env change.** `Memoize::rebuild` only re-invokes
  `init_view` when `prev.data != self.data` or the `dirty` flag is set (`views/memoize.rs:126-136`);
  `dirty` is only set by `MessageResult::RequestRebuild` (`:150-152`) — which the env can never send
  (see above). The docs say it explicitly, twice: "Note that it is not currently fully supported to
  mutate `Resource` values. In particular, if these values are read inside memoising views (such as
  `memoize`), the values won't be correctly updated. We intend to address this limitation in the
  future." (`environment.rs:150-153` and again `:370-373`).
- Practical consequence for Aeolus: env values ARE re-read on every rebuild that reaches a
  `with_context` (fresh `child_view` each rebuild, `environment.rs:539`), so *un-memoized* subtrees
  see updates; any `memoize` between the `provides` and the reader freezes the old value, and
  changing an env value alone never schedules a rebuild.

---

## Q10 — Four upstream gaps

### 1. `has_hovered`-style ancestor state flags (group-hover)
- Reality: `has_hovered` **exists** (`widget_state.rs:249`, accessor `contexts.rs:1340-1342`,
  propagation via hovered-path diffing `passes/update.rs:965-1037` + `Update::ChildHoveredChanged`
  `core/events.rs:116-121`). What's missing is *class-scoped* group state (and any peer/sibling
  state). Patch: see Q2 — `class_set.rs` + `selector.rs` + `property_cache.rs` +
  `passes/update.rs` (~150 LOC total, plus tests in `masonry/src/tests/update.rs`).

### 2. Window-size "class flag" (responsive breakpoints)
- Reality: widgets have **no** viewport-size signal. The root widget is laid out against the window
  size (`passes/layout.rs:548-556` uses `root.get_kurbo_size()`), `RenderRootState.scale_factor`
  exists (`render_root.rs:172, 216`; accessor `contexts.rs:1300-1301`) and `WindowEvent::Rescale`
  (`render_root.rs:512-513`) rescales, but there is no `window_size` in `GlobalState` and no update
  event for window resize. The only responsive primitive is the `resize_observer` view
  (`xilem_masonry/src/view/resize_observer.rs:55-70`), which wraps a child in a
  `widgets::ResizeObserver` with `Dimensions::MAX` (`:105`) and surfaces `element.ctx.content_box().size()`
  through a `LayoutChanged` message (`:162-166`) — documented for exactly this use (`:17-23`, example
  `:39-53`).
- Patch: add `window_size: Size` to `RenderRootState`/`GlobalState` set by the host; emit
  `Update::WindowResized(Size)` (or set a pseudo-class / run a relevance-gated re-resolution) from a
  new branch in `passes/update.rs`; optionally add `Selector::min_width(px)` for breakpoint layers.
  Smaller alternative: keep using `resize_observer` at the root and branch in app state (works today,
  no patch).

### 3. `FontSize` / `FontWeight` as Masonry properties
- Reality: NOT FOUND (see Q5a). Typography is per-widget parley `StyleSet` state.
- Patch: new files `masonry_core/src/properties/font_size.rs`, `font_weight.rs` (impl `Property` with
  `static_default`, mirroring `corner_radius.rs:17-24`); register in
  `masonry_core/src/properties/mod.rs:6-24`; in `core_property_changed` (`properties/mod.rs:29-42`)
  route them to `ctx.request_layout()` (font size affects measurement) or to a
  `run_update_fonts_pass`-style invalidation; implement `property_changed` in `Label`
  (`masonry/src/widgets/label.rs:486-489` already shows the pattern for `LineBreaking`) to translate
  the property into `self.styles.insert(StyleProperty::FontSize(..))`; add defaults in
  `masonry/src/theme.rs` (e.g. `properties.insert::<Label, _>(FontSize(15.0))`).

### 4. View-side API to register property-stack layers on a child widget
- Reality: **does not exist** (zero `PropertyStack` references in `xilem_masonry/`; `Prop` only sets
  local props — Q1). Masonry-side plumbing is all present: `NewWidget::with_property_stack`
  (`widget_pod.rs:181-185`), `MutateCtx::set_property_stack` (`contexts.rs:353-359`),
  `RenderRoot::insert_property_stack` (`render_root.rs:1065-1068`) and the whole
  `replace/remove_property_stack` lifecycle (`:1032-1063`).
- Patch shape:
  1. `xilem_masonry/src/pod.rs`: add `Pod::with_property_stack(id)` forwarding to
     `new_widget.with_property_stack(id)` (one-liner; works for build-time).
  2. A way for views to obtain a `PropertyStackId`: `ViewCtx` currently holds only widget_map /
     proxy / runtime / environment (`view_ctx.rs:14-25`) and never touches a `RenderRoot`. Options:
     (a) put an `Arc<Mutex<PropertyArena>>` handle (or a `RenderRoot` handle) in `ViewCtx` and add
     `ViewCtx::insert_property_stack(stack) -> PropertyStackId`; or (b) attach *additional* layers at
     the `NewWidget` level: extend `NewWidget` with a list of `(Selector, PropertySet)` overlay
     layers that `register_child` pushes into the arena inline (`contexts.rs:2090-2142`) — this
     avoids needing `RenderRoot` access during build.
  3. A `styled_layer(selector, props)` wrapper view whose `build` creates the stack and marks the
     pod, and whose `rebuild` diffs the layer set via
     `RenderRoot::replace_property_stack` (needs the RenderRoot handle) or via a new
     `MutateCtx::mutate_property_stack` (requires making `property_arena` mutable in
     `MutateCtx` — today contexts only hold `&'a PropertyArena`, `contexts.rs:64-149`).

---

## Masonry's property / cascade model (summary)

A widget's effective value for property `P` is a 4-level cascade: the widget's **local**
`PropertySet` (set via `NewWidget::with_props` `widget_pod.rs:163-173` or `WidgetMut::insert_prop`
`widget_mut.rs:95-101`), then the **property stack** (the `PropertyStack` named by
`WidgetState.property_stack_id`, resolved top-down with shadowing against the widget's own
`ClassSet` — `property_stack.rs:107-141`, `selector.rs:61-69`), then **default properties** for the
widget type (`default_properties.rs:40-45`, populated by themes like `masonry/src/theme.rs`), and
finally the type's `static_default` (`property.rs:20-42`). Resolution is cached per widget in
`PropertyCache.entries` (`property_cache.rs:11-23`), and the cache records *relevance*: which class
strings and which pseudo-flags (`is_hovered/is_active/is_disabled/has_focus_target`) influenced any
cached answer (`property_cache.rs:15-34`), so `update_props_for_widget` can skip work when an
irrelevant class flips (`passes/update.rs:1164-1179`, `:1121-1148`). Recompute is triggered by:
class changes (`add_class`/`remove_class`, `contexts.rs:1675-1690`), status transitions that write
`class_diff` (hover `update.rs:1023-1037`, active `:923-937`, focus `:719-725`, disabled
`:340-342`), stack reattachment (`contexts.rs:353-359`), stack replacement/removal
(`render_root.rs:1032-1063`), and whole-tree default-property swaps (`render_root.rs:425-449`).
The CSS-likeness is intentional; the docs describe the same cascade at
`masonry_core/src/doc/masonry_concepts.md:188-197`.

## Widget update / paint flow for style changes (summary)

Setting a property does **not** rebuild anything. `WidgetMut::insert_prop` inserts locally and calls
`core_property_changed` + `Widget::property_changed` (`widget_mut.rs:95-101`). The framework's
`core_property_changed` maps property types to invalidations: `Dimensions`, `BoxShadow`,
`BorderWidth`, `CornerRadius`, `Padding` → `ctx.request_layout()`; `Background`, `BorderColor` →
`ctx.request_pre_paint()` (`masonry_core/src/properties/mod.rs:29-42`); anything else falls through
to the widget's own `property_changed` (default no-op `core/widget.rs:247`), where widgets request
their own flags (e.g. `Label` re-requests layout for `LineBreaking` via
`LineBreaking::prop_changed`, `masonry/src/widgets/label.rs:486-489`). Layout consumes
`BorderWidth`/`Padding`/`Dimensions` via `props.get` (`passes/layout.rs:348-349, 400-402`) and
derives `border_box_insets`; paint is a three-phase `pre_paint`/`paint`/`post_paint` sequence gated
by `request_pre_paint`/`request_paint`/`request_post_paint` flags
(`passes/paint.rs:99-141, 337-338`), where the default `pre_paint`
(`core/widget.rs:380-387`) paints shadow + background + border **from properties for every widget**
(`core/widget_paint.rs:48-62, 65-119`). Class-driven changes (hover/active/focus) route through the
`update_props` pass described above and fire `Widget::property_changed` per affected cached type
(`update.rs:1121-1148`).

## Button: how hover styling works today (summary)

Masonry's own `Button` implements **no** styling in the widget: its `update` is empty
(`masonry/src/widgets/button.rs:161-167`), its `paint` is empty (`:215-221`), and its
`property_changed` is a no-op (`:173`). All visuals come from the theme's default property set +
stack: base props (`Padding`, `CornerRadius`, `BorderWidth`, `Background` ZYNC_800, `BorderColor`
ZYNC_700) at `masonry/src/theme.rs:77-83`, then a `PropertyStack` with layers
`Selector::new().with_hovered(true) → BorderColor ZYNC_500` (`theme.rs:85-89`),
`.with_focused(true) → BorderColor FOCUS_COLOR` (`:90-93`), `.with_active(true) → Background
ZYNC_700` (`:94-97`), `.with_disabled(true) → Background BLACK` (`:98-102`), registered via
`insert_stack::<Button>` (`:102`). The core default `pre_paint` resolves those props through the
class set (which gets `is_hovered` etc. from the update pass) and paints them
(`core/widget_paint.rs:48-62`). So it *is* a property-stack-with-`Selector` design — but the
selector is the flat struct of Q1, not `Selector::If`. Interactivity still uses raw flags:
`if ctx.is_active() && ctx.is_hovered()` to submit the press (`button.rs:118-121`).

## masonry_concepts.md — key styling concepts (summary)

- **Classes** (`doc/masonry_concepts.md:17-22`): arbitrary string labels per widget, analogous to CSS
  classes, "mostly used for property selection".
- **Widget status ≈ pseudo-classes** (`:25-41`): hovered / active / pointer capture / text focus /
  disabled / stashed; status changes call `Widget::update`.
- **Property fallback cascade** (`:188-197`): local properties → property stack (layers gated by
  `Selector`) → per-widget-type defaults; first match wins.
- **Properties are low-verification** (`:179-186`): no compile-time checks that a property is
  supported; the opt-in `UsesProperty` trait (`core/property.rs:44-53`) exists for external crates
  that want static verification.
- **Box model** (`:199-206`): content-box / border-box / paint-box / bounding-box, with padding and
  border resolved from properties (see `widget_state.rs:94-108`).

## External widget / Aeolus applicability (summary)

The property system is fully public and usable from outside `masonry_core`: `Property` is a plain
trait (`Default + Clone + Send + Sync + 'static` + `static_default`, `property.rs:20-42`), there is
no registration authority (any crate can define a property), no `impl_property` macro exists (**NOT
FOUND**; the only macros in masonry_core are `debug_panic` `util.rs:15`,
`impl_context_method` `contexts.rs:42`, `impl_props_from_tuple` `property_set.rs:64`), and `masonry`
re-exports the engine as `masonry::core` (`masonry/src/lib.rs:223`). A new widget simply reads
`props.get::<P>(cache)` in its measure/layout/paint methods, and a theme (external or internal)
provides defaults + stacks via `DefaultProperties::insert/insert_stack`. Concretely,
`lower(plan) -> PropertyStack` can produce today: `PropertyStack::push_layer(Selector::new(),
PropertySet::one(Background::Color(c))`-style layers using the existing types below; pseudo-class
layers via `Selector::new().with_hovered(true)` etc.; breakpoints would need gap #2, group-hover gap
#1, typography gap #3. Note `PropertySet` also accepts tuples of properties via
`impl_props_from_tuple!` (`property_set.rs:64-98`).

### All property types defined (table)

Core (masonry_core/src/properties/):

| Property | Kind | Definition |
|---|---|---|
| `Background` | enum: `Color(AlphaColor<Srgb>)` \| `Gradient(Gradient)` | `background.rs:18-21` (impl `:25-31`) |
| `BorderColor` | `{ color: AlphaColor<Srgb> }` | `border_color.rs:14-16` (impl `:24-31`) |
| `BorderWidth` | `{ width: Length }` | `border_width.rs:15-17` (impl `:22-29`) |
| `BoxShadow` | `{ color, offset: Point, blur_radius: Length }` | `box_shadow.rs:25-36` (impl `:38-47`) |
| `CornerRadius` | `{ radius: Length }` | `corner_radius.rs:13-15` (impl `:17-24`) |
| `Dimensions` | `{ width: Dim, height: Dim }` (Auto/Stretch/Fixed/Ratio/…) | `dimensions.rs:19-22` (impl `:24-29`) |
| `Padding` | `{ left, right, top, bottom: Length }` | `padding.rs:13-22` (impl `:24-29`) |

Widget-level (masonry/src/properties/):

| Property | Kind | Definition |
|---|---|---|
| `AnimationDuration` | `{ seconds }` | `animation.rs:9` |
| `AutoHideScrollBar` | `bool` | `portal.rs:8` |
| `BackwardColor` | `AlphaColor<Srgb>` | `step_input.rs:35` |
| `BarColor` | `AlphaColor<Srgb>` | `progress_bar.rs:12` |
| `CaretColor` | `{ color }` | `selection.rs:14` |
| `CheckmarkColor` | `{ color }` | `checkmark.rs:12` |
| `CheckmarkStrokeWidth` | `{ width }` | `checkmark.rs:28` |
| `Collapsible` | `bool` | `scroll_bar.rs:8` |
| `ContentColor` | `{ color }` | `content_color.rs:26` |
| `ForwardColor` | `AlphaColor<Srgb>` | `step_input.rs:60` |
| `Gap` | `{ cross/main: Length }` | `gap.rs:22` |
| `HeatColor` | `AlphaColor<Srgb>` | `step_input.rs:87` |
| `LineBreaking` | enum WordWrap/Clip/Overflow | `line_breaking.rs:10` |
| `ObjectFit` | enum Fill/Contain/Cover/ScaleDown/None | `object_fit.rs:14` |
| `PlaceholderColor` | `{ color }` | `placeholder_color.rs:12` |
| `SelectionColor` | `{ color }` | `selection.rs:35` |
| `StepInputStyle` | enum | `step_input.rs:11` |
| `ThumbColor` | `Color` | `slider.rs:32` |
| `ThumbRadius` | `Length` | `slider.rs:12` |
| `TrackColor` | `{ active, inactive }` | `track.rs:32` |
| `TrackThickness` | `Length` | `track.rs:12` |

Supporting value types (not properties): `Gradient`, `GradientShape`, `RadialGradientShape`,
`RadialGradientExtent` (`masonry_core/src/properties/types/gradient.rs:22,70,101,118`);
`MainAxisAlignment`/`CrossAxisAlignment` (`masonry/src/properties/types/alignment.rs:9,38`).
**Total: 28 types implementing `Property` (7 core + 21 widget-level).**

---

## Capability matrix inputs

| aspect | masonry/xilem value | citation |
|---|---|---|
| cascade model | 4-level: local `PropertySet` → `PropertyStack` (per-widget stack id, `Selector`-gated layers, top-down shadowing) → `DefaultProperties` per widget type → `Property::static_default`; relevance-gated per-widget cache | `masonry_core/src/core/properties_ref.rs:32-47`, `property_stack.rs:107-141`, `property_arena.rs:38-46`, `property_cache.rs:11-46` |
| hover/focus/active/disabled state access | Full: `is_hovered`/`has_hovered`/`is_active`/`has_active`/`is_focus_target`/`has_focus_target`/`is_disabled` on all contexts; pseudo-flags flow into `ClassSet` for `Selector` matching; `Update::{Hovered,Active,Focus,Disabled}Changed` events | `contexts.rs:1333-1390`, `widget_state.rs:249-261`, `passes/update.rs:989-1037`, `core/events.rs:75-146` |
| responsive breakpoints | No viewport signal; only widget-level `resize_observer` view reporting child content-box size on `LayoutChanged` | `xilem_masonry/src/view/resize_observer.rs:55-70,162-166`; window size used only for root layout `masonry_core/src/passes/layout.rs:548-556`; `window_size` query: NOT FOUND |
| container queries | NOT FOUND (no ancestor-size query; `resize_observer` is the only approximation; no container pseudo-class) | `xilem_masonry/src/view/resize_observer.rs:14-24` |
| dark mode | No class/media-query system; achievable only by whole-tree `set_default_properties` (host) or `WindowView::with_default_properties` (Arc-identity swap in rebuild) | `masonry_core/src/app/render_root.rs:416-449`, `xilem/src/window_view.rs:66-73,127-134` |
| theme hot-swap | Yes at runtime: `RenderRoot::set_default_properties` invalidates every widget's property cache and re-fires `property_changed`; per-window via WindowView | `render_root.rs:425-449`, `xilem/src/window_view.rs:127-134`, `xilem/src/app.rs:164-169` |
| typography | parley `StyleProperty`/`StyleSet` per text widget only; `FontSize`/`FontWeight`/`LetterSpacing` properties NOT FOUND; props cover `ContentColor`, `LineBreaking` | `masonry_core/src/core/text.rs:29-32`, `masonry/src/widgets/label.rs:52,179-183,203`, `label.rs:474` |
| colors (alpha/gradient in `Background`?) | Yes: `Background::Color(AlphaColor<Srgb>)` (alpha-capable) and `Background::Gradient(Gradient)` (linear/radial, CSS-style), rendered via peniko brush | `masonry_core/src/properties/background.rs:18-21,47-52`, `properties/types/gradient.rs:70,22,101,118` |
| box-shadow | Yes: `BoxShadow { color, offset, blur_radius }`; no inset/spread yet (upstream TODO) | `masonry_core/src/properties/box_shadow.rs:10-16,25-36` |
| border-radius | Yes: `CornerRadius { radius: Length }` (single radius, not per-corner) | `masonry_core/src/properties/corner_radius.rs:13-15` |
| padding/margin/gap/layout props | `Padding` (4 edges), `Gap` (cross/main), `Dimensions` (Dim: Auto/Stretch/Min/Max/Fit/Fixed/Ratio); **margin NOT FOUND** (use parent gap/spacers) | `properties/padding.rs:13-22`, `masonry/src/properties/gap.rs:22`, `masonry_core/src/properties/dimensions.rs:19-22` |
| view-side styling API | Yes for values: `WidgetView::prop(P)` + `Style` trait (`.background()`, `.border()`, …) → local props at build/rebuild; NO for stack layers (no `PropertyStack` API in xilem_masonry) | `xilem_masonry/src/widget_view.rs:73-86`, `view/prop.rs:35-66`, `style.rs:21-159`; stack API NOT FOUND (grep: 0 hits) |

---

## Auditor's cross-check notes

- The brief's assumption "views can register property layers" is **false for stack layers** and true
  only for single values: `Prop` writes into `NewWidget.properties` (view build, `prop.rs:35-42`) or
  through `WidgetMut::insert_prop` (view rebuild, `prop.rs:65`), which lands in the *local* property
  set and shadows the entire stack. Per-widget `PropertyStackId` assignment exists on `NewWidget`
  (`widget_pod.rs:181-185`) but no xilem view wrapper uses it.
- `Selector` in `masonry/src/widgets/selector.rs` is a *widget* (a segmented button, aliased
  `SelectorButton` in `theme.rs:186`) — unrelated to the styling `Selector`.
- Hover styling relies on the update pass writing `class_diff.is_hovered` for the hovered widget
  (`passes/update.rs:1023-1037`); ancestors only get `has_hovered` + `Update::ChildHoveredChanged`,
  **not** a class flag, so an ancestor-styled hover currently requires widget code, not properties.
