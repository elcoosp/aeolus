# Q1–Q10 — answered from source (deliverable 1, part 2)

All answers carry `file:line` citations in the section reports. Commit
`9f504ae` = linebender/xilem, `1351ffb` = lapce/floem, `8ff48c4` = iced.

**Q1 — Xilem `PropertyStack` layer registration path from a view?**
Views **cannot** register stack layers. The `prop` view writes a *local
property* into the child's `NewWidget.properties` at build
(`xilem_masonry/src/view/prop.rs:35-42`) or via `WidgetMut::insert_prop` at
rebuild; local props **shadow the entire stack**
(`properties_ref.rs:32-47`). Stack layers exist only at: theme time
(`DefaultProperties::insert_stack::<W>`, default_properties.rs:48-50), host
time (`RenderRoot::insert_property_stack`, render_root.rs:1066-1068), or
widget time (`MutateCtx::set_property_stack`, contexts.rs:353-359);
`NewWidget::with_property_stack` (widget_pod.rs:181-185) attaches a
pre-registered stack id at build. `Selector` is a flat struct (classes +
hover/active/disabled/focus flags), not an enum (selector.rs:12-19).
Grep: `xilem_masonry/` has **zero** references to `PropertyStack`.

**Q2 — group-hover/peer mapping & the `has_hovered` upstream patch.**
`WidgetState.has_hovered` already exists (self-or-descendant hovered,
widget_state.rs:249) — CSS `:hover`-on-ancestors, NOT class-scoped
group-hover. No ancestor/sibling machinery exists. Minimal patch (~150 LOC):
add `is_group_hovered` to `ClassSet`/`ClassSetDiff` (class_set.rs), a
`with_group_hovered(name)` to `Selector` (selector.rs), a relevance flag in
`PropertyCache` (property_cache.rs:17-34), and in the hover branch of
`run_update_pointer_pass` (passes/update.rs:965-1037) mark group-classed
ancestors — piggybacking on the existing hovered-path diff +
`Update::ChildHoveredChanged` chain. Peer needs a designated-sibling handle
resolved through the parent's children in the same pass. (sections/
xilem-masonry.md §Q2.)

**Q3 — does floem-css have a programmatic rule API?**
Yes — and floem-css does not exist in the repo at all; floem 0.2 ships a
mature CSS-like engine that is fully programmatic: `Style::hover/focus/
focus_visible/focus_within/active/disabled/dark_mode/selected/drag/
file_hover/class/responsive` nested-map builders (style/mod.rs:2609-2753),
attached via `Decorators::style(|s| s.apply(rule))` (decorator.rs:80-95) or
class rules that flow down via `class_context` (cx.rs:98-104); per-prop
`Transition{duration, easing}` (transition.rs). Consequence: a `StylePlan`
lowers to Floem with **zero view-side code** — implemented and tested
(reports/sections/floem-backend-notes.md).

**Q4 — can Iced's `Catalog` be generated from a `StylePlan`?**
Yes, mechanically: `impl button::Catalog for AeolusCatalog` over a static
`(class-key, Status) → button::Style` map is directly supported
(button.rs:551-560). Cost: one Catalog impl per widget family (~20) **plus**
one `.class(key)`/style closure per styled widget instance — no
zero-view-work cascade. `button::Style` = background (Color|LinearGradient),
text color, `Border` (per-corner radius), single `Shadow`, snap. Status
coverage: button `Active/Hovered/Pressed/Disabled` (no Focused);
focus-within/group/peer have no state access. Per-side padding, gap, font,
width/height and cursor live OUTSIDE Catalog as construction properties —
a Tailwind lowering must inject those at view time. (sections/iced.md.)

**Q5 — typography per framework?**
Masonry: no FontSize/FontWeight/LetterSpacing properties (NOT FOUND);
text styling is per-widget parley `StyleSet`
(`masonry_core/src/core/text.rs:29-32`, label.rs:52,179-226) plus
`ContentColor`/`LineBreaking` properties; a global `Update::FontsChanged`
pass exists as the natural hook for a future property
(update.rs:1196-1249). Floem: first-class `FontSize/FontWeight` props.
Iced: `Font` is a construction property. Aeolus carries text props in the
plan and each backend negotiates (capability matrix).

**Q6 — theme hot-reload capabilities?**
Masonry: `RenderRoot::set_default_properties` invalidates every widget's
property cache and re-fires `property_changed` (render_root.rs:425-449);
per-window via `WindowView::with_default_properties`. Floem: reactive —
swap class rules or `set_global_theme`; dark flips restyle the tree via the
`DarkMode` dirty bit (window/handle.rs:440-465). Iced: `Program::theme()`
re-queried every update cycle (winit/window/state.rs:221). egui/Slint/
Vizia/Freya/Ribir: replace Style/global/globals/add_stylesheet/theme state
— see the matrix. Aeolus exposes the same contract: swap tokens via
`TailwindTheme::absorb_css` (version bump) and drop the `PlanCache`.

**Q7 — reuse of tailwind-ast / tw_merge?**
`tailwind-ast` is MPL-2.0 (whole tailwind-rs workspace) — **no vendoring**;
coverage overlaps ours, so we keep our parser and use its grammar docs as
reference only. `tw_merge` (MIT, 0.1.x) is the `cn()` **behavioral oracle**:
272 conflict-group ids (359 match arms) + 46-entry parent→child table in
`crates/tw_merge/tw_merge/src/core/merge/{get_collision_id.rs,
get_collisions.rs}`; reverse scan with a `HashSet<Collision{important,
variants, id}>`, variant-aware, `!` in key, negatives not in key,
unparseable classes pass through. Our trie implements the same semantics;
parity pinned by tests. ironframe: `core::parse()` is a stub
(src/core.rs:33-39) — its value is the `tests/fixtures/complex_parity/`
golden corpus (classes vs genuine tailwindcss v4.1.17 output), which is our
CI parity oracle input. crepuscularity (ISC) validated our typed-token +
OKLCH→sRGB approach. (sections/precedents.md.)

**Q8 — Shadcn v1 minimal component set?**
v1 = Button, Input, Label, Card, Badge, Separator, Skeleton, Alert (per
briefing deliverable 6; frequency analysis in the brief). All 13 candidate
components confirmed present in `apps/v4/registry/new-york-v4/ui/` at
`4aaea19`; the 8 v1 class tables are shipped **verbatim** in
`crates/aeolus-shadcn/src/classes.rs` with integrity tests.

**Q9 — is an Xilem `Environment` theme snapshot viable?**
No, not today. `Environment` is a TypeId→slot map; the change-listener API
is known-broken ("The change listeners currently aren't ever notified",
environment.rs:32-33, :478) and `memoize` does not re-run on env change
(memoize.rs:126-136); the docs say so twice (environment.rs:150-153, :370-373).
Un-memoized subtrees DO re-read env values on rebuild, but changing an env
value alone never schedules a rebuild. Aeolus decision: theme changes are
explicit (swap tokens → bump version → rebuild the affected views), not
Environment-driven. (sections/xilem-masonry.md §Q9.)

**Q10 — upstream contribution strategy (four gaps)?**
1. **group/peer state flags** — patch shape in Q2; smallest
   highest-value win; proof test in `masonry/src/tests/update.rs`.
2. **window-size class flag** — add `window_size` to `RenderRootState`/
   `GlobalState`, emit `Update::WindowResized` and let breakpoint layers be
   relevance-gated; smaller alternative: root `resize_observer` + app-state
   branch (works today, no patch). (xilem-masonry.md Q10.2.)
3. **FontSize/FontWeight properties** — new property files mirroring
   corner_radius.rs, route in `core_property_changed` (properties/mod.rs:
   29-42) to layout, translate to `StyleProperty` in `Label::property_changed`
   (pattern exists for LineBreaking, label.rs:486-489), defaults in theme.rs.
4. **view-side stack registration** — `Pod::with_property_stack(id)` +
   either a `PropertyArena` handle in `ViewCtx` or NewWidget overlay layers
   materialized in `RegisterCtx::register_child` (contexts.rs:2090-2142);
   after it lands, `lower()` output attaches directly and the `StyledWidget`
   fallback retires. (xilem-masonry.md Q10.4; detailed shapes in
   07-upstream-patches/PATCHES.md.)
