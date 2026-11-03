# AEOLUS Section Audit — Floem Style System

- **Repo:** `/home/z/my-project/repos/floem` (lapce/floem @ main, shallow clone)
- **HEAD:** `1351ffb162faeb8be983f1301d718a0d5fb59c27` — "fix: forward post_paint through Box<dyn View> (#1089)"
- **Version:** floem 0.2.0, edition 2024, rustc ≥ 1.91 (`Cargo.toml:28-44`)
- **Scope note:** there is **NO `floem-css` sub-crate in this repo** (workspace members: `renderer`, `skia`, `vello`, `vger`, `tiny_skia`, `reactive`, `editor-core`, `examples/*`, `ui-events-winit`, `test` — `Cargo.toml:3-14`). `rg floem-css|floem_css` → 0 matches. The audit therefore covers the in-tree programmatic style system, which is extensive: `src/style/{mod,selectors,props,values,theme,transition,cx,recalc,cache,components,custom,unit}.rs`, `src/view/{mod,state,id}.rs`, `src/views/decorator.rs`, `src/style.rs` → NOT FOUND (it's `src/style/mod.rs`).

---

## 1. Architecture in 5 bullets

1. **View tree + id system:** every widget is a `View` trait object behind a `ViewId` (global `VIEW_STORAGE` slotmap-ish registry, `src/view/id.rs:1088`); each view owns a `ViewState` (`src/view/state.rs:321-415`) holding a taffy `NodeId`, an `ElementId` in an `understory_box_tree` box tree for hit-testing/visibility (`src/view/state.rs:423-428`, `Cargo.toml:117-121`).
2. **Layout:** taffy 0.9.2 (flexbox + grid) per node; the resolved style is pushed into taffy via `Style::to_taffy_style()` (`src/style/mod.rs:3628-3705`) and `taffy().set_style(...)` (`src/style/cx.rs:571-588`).
3. **Rendering:** multiple renderer backends — vger (default feature), vello, skia, tiny-skia — all wgpu-based (`Cargo.toml:148-152`, vendored `vello/`, `vger/`, `skia/`, `tiny_skia/`); paint via peniko brushes/kurbo paths (`src/style/mod.rs:132-133`).
4. **Events:** capture-then-bubble dispatch (`src/view/mod.rs:993-1078`), hover/focus/active tracked by `understory_event_state` (`src/window/state.rs:16`), winit bridge crate `ui-events-winit`; `HeadlessHarness` enables windowless interaction tests (`src/headless.rs:1-30`).
5. **Reactivity:** first-party signals/effects crate `floem_reactive` (`reactive/`); style closures are `UpdaterEffect`s that re-run on signal change and patch one slot of the view's style stack (`src/views/decorator.rs:80-95`).

## 2. Q3 — Programmatic rule API

**Verdict: YES — fully programmatic; there is no CSS text parsing anywhere in the repo.** "Rules" are `Style` maps (`Rc<FxHashMap<StyleKey, Rc<dyn Any>>>`, `src/style/mod.rs:861-888`) with nested sub-maps for selectors and classes. Everything a CSS rule would express is built by Rust builder calls.

### 2.1 Selector construction — all variants

Pseudo-class enum `StyleSelector` (`src/style/selectors.rs:27-38`):
`Hover`, `Focus`, `FocusVisible`, `FocusWithin`, `Disabled`, `DarkMode`, `Active`, `Dragging`, `Selected`, `FileHover`.

Bitmask `StyleSelectors: u16` (`src/style/selectors.rs:10-23`) adds one extra synthetic flag: `RESPONSIVE` (plus the 10 flags above). `StyleSelector::all()` lists the 10 (`selectors.rs:109-122`). `HoverTargetClass` etc. exist as classes (`theme.rs:26`).

Structural selectors (`selectors.rs:88-106`): `FirstChild`, `LastChild`, `NthChild(NthChild)` with `NthChild::{odd, even, exact, an_plus_b}` (`selectors.rs:41-85`).

Parameterized responsive selectors (`src/style/mod.rs:592-612`): `ScreenSize(ScreenSize)`, `MinWidth(Pt)`, `MaxWidth(Pt)`, `WidthRange { min, max }`.

Builder methods that attach a nested style map (`Style::selector` at `mod.rs:2609-2613` wraps `style(Style::default())`):
- `Style::hover` `mod.rs:2626`, `focus` `:2631`, `focus_visible` `:2636`, `focus_within` `:2642`, `first_child` `:2647`, `last_child` `:2652`, `nth_child` `:2657`, `odd` `:2662`, `even` `:2666`, `selected` `:2672`, `drag` `:2677`, `disabled` `:2682`, `dark_mode` `:2687`, `file_hover` `:2692`, `active` `:2697`.
- Responsive: `Style::responsive` `:2702`, `min_window_width` `:2709`, `max_window_width` `:2720`, `window_width_range` `:2731`.
- Class rules: `Style::class::<C>(class, |s| ...)` `mod.rs:2749`; child-widget targeting helper `Style::custom_style_class::<CS>` `mod.rs:2773`.

### 2.2 Attaching a rule to a widget

- **Direct style:** `Decorators::style(|s| ...)` pushes onto the view's `StyleStack` (`src/views/decorator.rs:80-95`); repeated calls stack, later wins (`src/view/state.rs:265-296`; doc `mod.rs:42-48`).
- **Widget-internal style:** `View::view_style()` (`src/view/mod.rs:956-958`) — stack index 0, overridden by decorators.
- **Widget class:** every widget type has a class (e.g. `style_class!(pub ButtonClass)`, `src/views/button.rs:5-8`); instances get it via `.class(ButtonClass)` (`decorator.rs:151-155`, `Button::new` at `button.rs:96-100`). Class **definitions** live in any `Style` and flow down the tree through `class_context` (`cx.rs:98-104`; propagation `Style::apply_only_class_maps`, `mod.rs:985-999`; per-child dirty marking `cx.rs:387-409`). So `ancestor.style(|_| my_button_rule.clone())` + `button("x")` restyles every descendant button — exactly the shadcn "component variants" pattern (live example: `default_theme()` buttons `theme.rs:551-574`; themes example `examples/themes/src/main.rs:23-58` uses `.class(Button, move |_| blue_button)`).
- **View type class:** `View::view_class()` (`view/mod.rs:960-962`) auto-applies a class per widget type (`cx.rs:245-258`).
- **Reactive class add/remove:** `Decorators::class_if` (`decorator.rs:158-174`), `ViewId::add_class`/`remove_class` (`view/id.rs:1074-1085`).

### 2.3 Specificity / cascade ordering

`StyleCx::style_view` (`src/style/cx.rs:241-653`) + `ViewState::compute_combined` (`src/view/state.rs:507-553`):

1. Base = stacked styles: `view_style` (index 0) then each `.style()` call in order, later overriding (`state.rs:282-290`).
2. `resolve_nested_maps` (`mod.rs:645-674`): **class rules first** (`resolve_classes`, `mod.rs:676-693`), then the view's own style is applied **over** the class style (`class_style.apply(view_style)`, `mod.rs:668-669`) — so *inline/direct style > class rule*. No numeric specificity; order + nesting depth is the whole model ("deeper/more nested maps take precedence", doc `mod.rs:72-78`).
3. Within one style, nested selector maps resolve in a fixed loop (`resolve_selectors`, `mod.rs:704-843`): structural (`:first-child` etc.) → responsive → active breakpoint (`screen_size_bp_to_key`, `mod.rs:768`) → `dark_mode` (`:773`) → **`disabled` short-circuits the rest** (`:779-783`; hover/focus/active/selected only apply when not disabled) → `selected` → `hover` → `file_hover` → `focus` → `focus_within` → `focus_visible` (keyboard nav only, `:816-819`) → `active` (mouse or keyboard, `:820-834`).
4. Inherited props merge last: `computed_style = inherited_context.apply(direct)` (`cx.rs:412-414`); inheritance is a per-prop flag (`prop!` macro, `props.rs:490-520`; e.g. `TextColor` `mod.rs:1970`, `FontSize` `:1990`).
5. The `Theme` prop (`prop!(pub Theme: DesignSystem { inherited })`, `theme.rs:377-379`) is itself an inherited prop, so theme values cascade like CSS custom properties; deferred resolution via `ContextRef::def`/`ThemeExpr` (`theme.rs:381-449`, `mod.rs:260-304`).

### 2.4 Transitions — yes, first-class

- `Transition { duration: Duration, easing: Rc<dyn Easing> }` (`src/style/transition.rs:110-113`); constructors `Transition::{new, linear, ease_in_out, spring}` (`transition.rs:256-274`); easings `Linear`, `Bezier` (cubic), `Spring` in `src/animate/easing.rs` + `src/animate/mod.rs`.
- Attach per property: `Style::transition(Background, Transition::linear(100.millis()))` (`mod.rs:2602-2607`); auto-generated `transition_<prop>()` builders for `{tr}`-flagged props (`define_builtin_props!` at `mod.rs:1700-1714`; most visual props carry `tr`).
- Interpolation: `StylePropValue::interpolate` vtable (`props.rs:229-253`), driven by `StylePropReader::read` on extraction (`props.rs:371-404`) and `TransitionState::step` (`transition.rs:60-88`); each in-flight frame re-schedules a style pass (`StyleReason::transition`, `cx.rs:491-494`, `mod.rs` export `mod.rs:192`). Easing previews even have a debug view (`transition.rs:116-252`).
- Limitations: per-property (not per-layer) duration/easing; transitions run on value change detected at extraction time — equivalent to CSS `transition: <prop> <dur> <ease>`. Custom props opt in by implementing `StylePropValue::interpolate`.

### 2.5 Conclusion for `aeolus-floem`

A StylePlan (base + hover/focus/active/disabled/dark layers) lowers **entirely without touching view code**, using exactly these API calls:
`Style::new()` → prop builders (`.background/.padding/.border_radius/.width...`) → layer builders `.hover(...) .focus(...) .active(...) .disabled(...) .dark_mode(...)` (`mod.rs:2626-2699`) → transitions `.transition(Prop, Transition)` (`mod.rs:2602`) → attach with `Decorators::style(|s| s.apply(rule.clone()))` (`decorator.rs:80`) or publish as a class rule `.class(Class, |s| ...)` (`mod.rs:2749`) + `.class(Class)` on widgets (`decorator.rs:151`).

```rust
// Lowering: `bg-primary hover:bg-primary/90 rounded-md p-4` (radius-md = 6px, p-4 = 16px)
use std::time::Duration;
use floem::prelude::*;
use floem::style::{Style, Transition, Background, StyleClass};
use floem::unit::DurationUnitExt;

style_class!(pub AeolusBtn); // optional: reusable class = "component" layer

fn lower(plan: &StylePlan) -> Style {
    let mut s = Style::new();
    // base: `bg-primary` — theme-aware so a later dark swap still resolves correctly
    if let Some(_c) = &plan.base.background {
        s = s.with_theme(|s, t| s.background(t.primary()));          // theme.rs:462
    }
    // `rounded-md`
    s = s.border_radius(6.0);                                        // mod.rs:1911-1917
    // `p-4`
    s = s.padding(16.0);                                             // mod.rs:3024
    // hover layer: `hover:bg-primary/90`
    if plan.hover.background.is_some() {
        s = s.hover(|s| s.with_theme(|s, t| s.background(t.def(|t| t.primary().with_alpha(0.9)))));
    }
    s.transition(Background, Transition::ease_in_out(150.millis()))  // mod.rs:2602
}

// Option A (direct, no view-code coupling): wrap any view
let v = button("Save").style(move |s| s.apply(lower(&plan)));

// Option B (class rule, attach once on an ancestor; flows down via class_context):
let rule = Style::new().class(AeolusBtn, move |s| s.apply(lower(&plan)));
let root = root_view.style(move |_| rule.clone());
let btn  = button("Save").class(AeolusBtn);                        // decorator.rs:151
```

Caveats: hover/active are element-local (`InteractionState`, `cx.rs:39-70`); Tailwind's `group-hover:` has **no** equivalent (no ancestor-conditioned selectors — NOT FOUND). `disabled` layers work via the inherited `Disabled`/`set_disabled` prop (`mod.rs:2192`) which suppresses hover/focus/active styling (`mod.rs:779-783`).

## 3. StylePlan → floem property coverage

All built-in props are declared in `define_builtin_props!` (`src/style/mod.rs:1725-2196`); types below.

| Tailwind-ish property | floem prop (struct) | type | line | notes |
|---|---|---|---|---|
| background / bg-* | `Background` | `Option<Brush>` {tr} | `mod.rs:1975` | Brush = solid/gradient/image (peniko) |
| text color | `TextColor` (builder `.color`) | `Option<Color>` {tr, inherited} | `mod.rs:1970` | |
| border width | `BorderLeft/Top/Right/Bottom` | `Stroke` {nocb, tr} | `mod.rs:1893-1899` | `.border(1.0)` builder `mod.rs:2956` |
| border color | `BorderLeftColor` … ×4 | `Option<Brush>` {tr} | `mod.rs:1902-1908` | `.border_color` `mod.rs:2947` |
| border radius | `BorderTopLeftRadius` … ×4 | `Length` {tr} | `mod.rs:1911-1917` | `.border_radius` `mod.rs:459` |
| padding | `PaddingLeft/Top/Right/Bottom` | `Length` {tr} | `mod.rs:1920-1926` | `.padding` `mod.rs:3024`; % variants |
| margin | `MarginLeft/Top/Right/Bottom` | `LengthAuto` {tr} | `mod.rs:1929-1935` | `.margin` `mod.rs:3080` |
| width/height (+min/max) | `Width`,`Height`,`MinWidth`,`MinHeight`,`MaxWidth`,`MaxHeight` | `LengthAuto` {tr} | `mod.rs:1750-1775` | px/%/auto; `*_pct` helpers |
| gap | `ColGap`,`RowGap` | `Length` {tr} | `mod.rs:2059-2064` | `.gap` `mod.rs:2848` |
| flex-direction | `FlexDirectionProp` | `FlexDirection` | `mod.rs:1780` | `.flex_row/.flex_col` `mod.rs:3550` |
| justify-* | `JustifyContentProp`,`JustifyItemsProp`,`JustifySelf`,`AlignItemsProp`,`AlignContentProp`,`AlignSelf` | taffy enums | `mod.rs:1805-1870` | full flex+grid alignment |
| font-size | `FontSize` | `f64` {tr, inherited} | `mod.rs:1990` | em/pt units via `unit.rs` |
| font-weight | `FontWeight` | `Option<FontWeightProp>` {inherited} | `mod.rs:2000` | |
| font-family | `FontFamily` | `Option<String>` {inherited} | `mod.rs:1995` | |
| text color | `TextColor` | (above) | `mod.rs:1970` | same prop |
| cursor | `Cursor` | `Option<CursorStyle>` | `mod.rs:1965` | enum `components.rs:44` (Pointer, Text, …) |
| box-shadow | `BoxShadowProp` | `SmallVec<[BoxShadow; 3]>` {tr} | `mod.rs:1985` | **shadows exist**: `BoxShadow{color,x_offset,y_offset,blur_radius,spread}` `components.rs:71`; multi-shadow supported (`theme.rs:523-544`) |
| opacity | `Opacity` | `f32` {tr} | `mod.rs:2182` | 0.0-1.0 |
| display | `DisplayProp` | `Display` (taffy) | `mod.rs:1729` | Flex/None/Grid (`mod.rs:144-147`) |
| visibility | **no standalone prop** | — | — | visibility = `Display::None` + hidden phase machinery (`cx.rs:530-588`, `state.rs Visibility`); `overflow_x/y` for clipping `mod.rs:2090-2093` |

Also available: `Transform`/`ScaleX/Y`/`TranslateX/Y`/`Rotation` {tr} (`mod.rs:2101-2176`), `AspectRatio` (`:2041`), `LineHeight` {inherited} (`:2036`), `TextAlign`/`TextOverflow` (`:2026-2031`), `Outline`/`OutlineColor` {tr} (`:1875-1880`), `ZIndex` (`:1960`), `InsetLeft/...` (`:1938-1947`), `PointerEvents` {inherited} (`:1952`), `ScrollbarWidth` (`:2087`), `ObjectFit/ObjectPosition` (`:2048-2054`), focusability `Focusable` (`:2195`). Missing for a full Tailwind map: backdrop-filter/blur, CSS filters, text-shadow, letter-spacing (NOT FOUND).

### 3.1 Value plumbing details relevant to a lowering backend

- **StyleValue tri-state per prop:** `Val` / `Animated` / `Context` / `Unset` (`src/style/values.rs`; `StyleMapValue` downcast in `mod.rs:1066-1077`). `unset_*()` builders (`mod.rs:1677-1693`) let a lower layer explicitly cancel an inherited/outer value (Tailwind `hover:bg-transparent` → `.unset_background()` in the hover layer).
- **`StyleValue::Base` semantics in `apply`:** applying a style with `Base` for a prop leaves the underlying value untouched — i.e. layers compose additively, which maps 1:1 to how a StylePlan's conditional layers should merge (`mod.rs:1412-1421`).
- **Merge identity:** every Style mutation bumps a `merge_id`; `apply_mut` short-circuits when ids match, and combined ids are hashed — this is how `class_maps_eq` detects which classes changed without deep compares (`mod.rs:205-218`, `:1001-1003`, `:1023-1058`).
- **Deferred context values:** `Style::set_context`/`ContextRef::def` store closures resolved only when a prop is read, with re-entrancy protection for same-prop reads (`mod.rs:2486-2599`). This is what makes theme tokens (`t.primary()`) behave like CSS variables rather than baked colors, and what a lowering backend should emit for any design-token-derived value.
- **Length units:** `Length`/`LengthAuto`/`Pct`/`Pt`/`Em`/`Lh` (`src/style/unit.rs`, exports `mod.rs:193-196`) with `UnitExt` (`.pct()`, `.px()`); padding is `Length` (no auto), margins are `LengthAuto`. Percentages resolve at taffy layout time (`mod.rs:2383-2428`).
- **Per-view cacheability:** styles containing structural selectors or context values are excluded from the global resolved-style cache (`state.rs:312-317`, `mod.rs:1011-1021`) — a lowering backend that bakes concrete colors instead of context closures gets better cache hits.

## 4. Cascade model — how the final style is computed & recomputed

**Resolution (compile-time order):** base stack (`view_style` → successive `.style()`) → class rules (ancestor `class_context` + widget's own class list + `view_class`) applied *under* the view's own style → nested selector maps resolved in the fixed priority loop of §2.3 (disabled > selected > hover > file_hover > focus > focus-within > focus-visible > active; structural/responsive/dark applied first in-loop, `mod.rs:734-835`) → inherited props merged from parent's `computed_style` (`cx.rs:412-414`). Per-call stack precedence is proven in tests: `test/tests/style_computation.rs:52-60` ("Later .style() calls should override earlier ones").

**Runtime invalidation (what happens when `hover` flips):**
1. Pointer movement → hit-test path diff → `HoverEvent::Enter/Leave` → `mark_style_dirty_selector(target, StyleSelector::Hover)` (`src/event/dispatch.rs:2001-2056`, specifically 2015/2039). Active: `dispatch.rs:1308,1328,1341`; focus-within: `:1392-1397`.
2. `mark_style_dirty_selector` produces a targeted `StyleReason` (`src/window/state.rs:1392-1396`; `recalc.rs:115-125`) — **only the affected element(s) restyle**, not the tree; views without matching selector bits are skipped (`cx.rs:247-278`, intersection check `:261-267`).
3. Re-resolution consults a global style cache keyed by (style-content hash, InteractionState, breakpoint, classes, class_context) (`cx.rs:286-345`; `src/style/cache.rs`), so repeated hover enter/leave on an unchanged style is a cache hit.
4. Prop extractors re-read the computed style (`cx.rs:452-495`); a changed background → `request_paint` only (`cx.rs:642-644`); changed layout props → taffy `set_style` + relayout (`cx.rs:577-588`). **No widgets are rebuilt** — hover is pure restyle+repaint, and only transitions/anim frames keep re-scheduling style passes (`cx.rs:491-494`).
5. Cross-element states (`disabled`, `selected`, dark, responsive) propagate deliberately: `PROPAGATING_FLAGS` (`selectors.rs:155-158`) and per-selector interest registries `responsive_selector_views` / `disabled_selector_views` / `selected_selector_views` (`window/state.rs:140-142`, `update_selector_interest` `:1254-1266`, `mark_descendants_with_selector_dirty` `:1291-1310`).

**Selector application priority inside `resolve_selectors`** (each entry = nested map applied over the accumulating style when its state bit is on; `mod.rs:726-840`):

| order | applied when | gate |
|---|---|---|
| 1 | `:first-child` / `:last-child` / `:nth-child(an+b)` | `child_index`/`sibling_count` (`cx.rs:761-790`) |
| 2 | parametric responsive (`MinWidth`/`MaxWidth`/range) | `window_width` (`mod.rs:747-754`) |
| 3 | active breakpoint map (`xs`..`xxl`) | `window_state.screen_size_bp` (`mod.rs:768`) |
| 4 | `dark_mode` map | `is_dark_mode` (`mod.rs:773`) |
| 5 | `disabled` map — **excludes 6-10** | `is_disabled \|\| style.get(Disabled)` (`mod.rs:779-783`) |
| 6 | `selected` map | `is_selected \|\| style.get(Selected)` (`mod.rs:785`) |
| 7 | `hover` map | `is_hovered` (`mod.rs:792`) |
| 8 | `file_hover` map | `is_file_hover` (`mod.rs:798`) |
| 9 | `focus` / `focus_within` maps | `is_focused` / `is_focus_within` (`mod.rs:805-814`) |
| 10 | `focus_visible` + `active` (keyboard) or `active` (mouse) | `using_keyboard_navigation` (`mod.rs:816-834`) |

The loop repeats until fixpoint (max depth 20, `mod.rs:723-840`) so a hover layer may itself contain responsive/dark nested maps. Note the asymmetries a backend must respect: `Active` requires `!using_keyboard_navigation` for mouse presses and vice-versa (`mod.rs:816-834`); `FocusVisible` requires keyboard navigation; `Disabled` comes from an inherited prop (`set_disabled`, `mod.rs:2192`) so a parent can disable a whole subtree and the `disabled` maps of descendants fire (`cx.rs:513-521`).

**Cost of a hover flip:** one element re-runs `compute_combined` (map merge + nested-map resolution over a small `FxHashMap`) or hits the style cache; extraction + paint follow. Benchmarks exist for exactly this path (`benches/style_computation.rs`, `benches/animation.rs`). There is no global recompute, no diffing of view trees, and no re-creation of any `View` object.

## 5. Other capability inputs

- **Responsive breakpoints: YES.** `Style::responsive(ScreenSize, …)`, `min_window_width`, `max_window_width`, `window_width_range` (`mod.rs:2702-2746`); default Tailwind-like `GridBreakpoints` xs<576 / sm<768 / md<992 / lg<1200 / xl<1400 / xxl≥1400 (`src/layout/responsive.rs:30-56`); source of truth is `window_state.root_size.width` (`cx.rs:805`); resize marks responsive-interested descendants dirty (`cx.rs:161-166`, `window/state.rs:1239`, `:1312-1321`). Parametric widths stored as rules inside the style (`mod.rs:1248-1261`).
- **Container queries: NOT FOUND** (window/root width only).
- **Dark mode: YES, two mechanisms.** (1) `Style::dark_mode(|s| …)` selector — `is_dark_mode` from the winit `Theme` (`window/state.rs:504-505`), toggled by `set_global_theme`/`set_theme`/`toggle_*` actions (`src/action.rs:87-117`); `WindowHandle::set_theme` rebuilds the default theme style and marks the root `DarkMode` selector dirty → whole-tree restyle (`src/window/handle.rs:440-465`). (2) Inherited `Theme: DesignSystem` prop (`theme.rs:377-379`) with light/dark presets (`theme.rs:128-159`) read lazily via `with_theme`/`ThemeExpr` (`theme.rs:451-474`) — values re-resolve on restyle without re-creating style objects.
- **Theme hot-swap: feasible & demonstrated.** `AppConfig::set_global_theme` (`app/mod.rs:80-84`), OS theme change events (`window/handle.rs:217-221`), and the `examples/themes` example swaps entire class-defined palettes at runtime via reactive `.class(...)` rules (`examples/themes/src/main.rs:23-120`). Per-subtree theming is possible by setting the `Theme` prop anywhere (`Style::theme(ds)`).
- **Restyling built-in widgets externally: YES.** (a) class rules from any ancestor (`.class(ButtonClass, …)`, §2.2); (b) `Style::custom_style_class::<SomeWidgetCustomStyle>()` targets a child widget's class (`mod.rs:2755-2777`); (c) `CustomStyle`/`CustomStylable` traits give per-widget styling DSLs incl. selectors (`src/style/custom.rs:36-120`, `:397`); (d) window-level default-theme replacement (`WindowHandle.default_theme`, `window/handle.rs:184-185`, `:333`). Widgets consume props via `prop_extractor!` structs in `style_pass` (`src/views/label.rs:483-492`; extractor macro `props.rs:522-580`).
- **Testing surface:** `HeadlessHarness` simulates pointer/keyboard and asserts styles headlessly (`src/headless.rs:1-60`), plus dedicated style tests (`test/tests/style_computation.rs`, `transitions.rs`, `responsive_selectors.rs`, `style_cache.rs`, `theme_inherited_props.rs`).

### Reactive `.style()` closures

`Decorators::style` allocates a stack slot, then creates an `UpdaterEffect::new(|| style(Style::new()), |s| view_id.update_style(offset, s))` and pushes the initial value (`decorator.rs:80-95`). Because the closure runs inside an effect scope, any signal read inside is tracked; on change the effect re-runs and `ViewId::update_style` overwrites **only that slot** and requests a full style recalc for the view (`view/id.rs:1087-1093`). The `StyleStack` recomputes lazily from the first dirty index and caches partial merges + a content hash (`state.rs:240-318`, `style()` at `:265`). Caveat documented on the trait: track signals at the top level of the closure (`decorator.rs:32-34`). `ExprStyle` variants allow the same closures inside `hover(...)`/`with_theme(...)` (`mod.rs:306-410`).

### floem-tailwind / floem-shadcn

Not present in this repo and not referenced anywhere (`rg "floem-tailwind|floem_tailwind|shadcn"` → 0 matches; also 0 for `floem-css`). This repo only *implies* what such crates would build on: the class-context system + selector layers + inherited `Theme: DesignSystem` prop are precisely the substrate for a Tailwind-like utility layer or a shadcn-like component-variant library, and `default_theme()` (`theme.rs:550-780`) is already a hand-written "shadcn-style" stylesheet in pure Rust. README advertises CSS-like transitions for any interpolatable property, including in classes (`README.md:52`).

### 5.1 Gaps & risks for `aeolus-floem`

- **`group-hover:` / `peer-*:` / descendant combinators: NOT FOUND.** Selectors are self-state + class-lookup only; there is no way to say "style me when *my child* is hovered" or "when a sibling has class X". Workaround: set the conditional classes/styles from the parent's reactive style closure.
- **Arbitrary variants like `first-letter`, `::placeholder`** are modeled ad hoc: placeholder styling exists via `PlaceholderTextClass` (`theme.rs:15`) and custom props via `prop!` (`props.rs:490-520`) — but only where the widget author added an extractor.
- **Per-layer transition timing:** floem attaches a transition to a *property* (a single duration/easing per prop per style), not to a (layer, prop) pair. Lowering `hover:duration-150` style plans means the hover map itself carries the transition (`theme.rs:569` does exactly this on the base style).
- **Specificity is structural, not numeric:** order of `.style()` calls matters (`state.rs:265-296`). A lowering backend that emits one merged base style + one merged layer per state is immune to this, but users mixing generated styles with hand-written `.style()` calls can get ordering surprises.
- **`Display::None` vs `visibility`**: hiding during a transition uses the visibility-phase machinery keyed off animations (`cx.rs:530-588`); a plain Tailwind `invisible` has no direct prop (closest: `.opacity(0.)` or `unset` display).

## 6. Capability matrix inputs

| aspect | floem value | citation |
|---|---|---|
| cascade model | fixed order: style stack (later > earlier) > class rules > inherited props; nested selector maps resolved in-loop, depth-first-wins; no numeric specificity | `mod.rs:645-674`, `state.rs:265-296`, doc `mod.rs:37-78` |
| hover/focus/active/disabled via selectors | YES — 10 `StyleSelector` variants + structural (:first/:last/:nth-child) + responsive; disabled suppresses others | `selectors.rs:27-38`, `mod.rs:2626-2699`, `mod.rs:779-835` |
| responsive | YES — breakpoints (xs-xxl) + parametric min/max/range window-width selectors | `mod.rs:2702-2746`, `responsive.rs:30-56` |
| container queries | NOT FOUND (window-width only) | — |
| dark mode | YES — `dark_mode()` selector + inherited `Theme: DesignSystem` prop; OS theme + manual actions | `mod.rs:2687`, `theme.rs:377-379`, `action.rs:87-117` |
| theme hot-swap | YES — global action, window set_theme, per-subtree Theme prop; demonstrated in examples/themes | `app/mod.rs:80-84`, `window/handle.rs:440-465`, `examples/themes/src/main.rs` |
| typography props | YES — size/weight/style/family/line-height/align/overflow, all inheritable; no letter-spacing | `mod.rs:1990-2036` |
| shadows | YES — multi box-shadow prop w/ color/offset/blur/spread, transitionable | `mod.rs:1985`, `components.rs:71`, `theme.rs:523-544` |
| border-radius | YES — 4-corner `Length` props + builder, transitionable | `mod.rs:1911-1917`, `:459-468` |
| programmatic rule API | YES — `Style::hover/focus/active/disabled/dark_mode/class(...)` builders + `Decorators::style` attach + class-context flow-down; no CSS text parsing exists | `mod.rs:2609-2753`, `decorator.rs:80-95`, `cx.rs:98-104` |
| transition support | YES — per-prop `Transition{duration, easing}` (Linear/Bezier/Spring), interpolated at extraction, frame-scheduled | `transition.rs:110-113`, `:256-274`, `props.rs:371-404`, `cx.rs:491-494` |
