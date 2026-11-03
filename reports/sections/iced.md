# AEOLUS Section Audit — iced (Catalog system)

- **Repo:** `/home/z/my-project/repos/iced` (iced-rs/iced, shallow clone)
- **Commit:** `8ff48c42b0d5188223501ea1ccc506fd5beaf36a`
- **Branch:** `master`
- **Workspace version:** `0.15.0-dev` (`Cargo.toml:170`); **MSRV:** `rust-version = "1.93"` (`Cargo.toml:178`), edition 2024 (`Cargo.toml:172`). No 0.14 tag exists in this snapshot; the in-tree version targets 0.15-dev (0.14 is already superseded on master).
- Scope: core `Theme`, widget `Catalog`/`Style`/`Status` system, style value types, render path, theme hot-swap.

---

## 1. Architecture in 5 bullets

1. **Workspace layout:** `core/` (`iced_core`: `Widget` trait, `Theme`, style primitives like `Length`/`Padding`/`Border`), `runtime/` (`UserInterface` + widget `Tree` diffing, `runtime/src/user_interface.rs:29-38`), `program/` (the Elm `Program` trait, `program/src/lib.rs:27-108`), `widget/` (widget library incl. all `Catalog`s), `graphics/` + `wgpu/` + `tiny_skia/` (renderers), `winit/` (window shell), `src/` (`iced` facade with `application(...)` builder).
2. **Elm loop:** `Program { State, Message, Theme, Renderer, Executor }` (`program/src/lib.rs:27-41`); `update` → `Task<Message>`, `view` → widget tree; `iced::application(...).theme(...)` wiring in `src/application.rs:410-420`.
3. **Element/widget tree:** `view` returns an `Element`; persistent per-widget state lives in a `Tree` (each widget supplies `tag()`/`state()`, e.g. `widget/src/button.rs:222-228`), reconciled via `diff` (`button.rs:230-236`).
4. **Style pipeline:** each widget instance stores a *class* (`class: Theme::Class<'a>`, `button.rs:81`) which its `Catalog` resolves into a plain-data `Style` (`Background`/`Border`/`Shadow`/`Color`) **at draw time**, then lowered to a renderer `Quad` (`core/src/renderer.rs:101-113`) via `fill_quad` (`core/src/renderer.rs:55`).
5. **Renderer trait:** `core/src/renderer.rs:16-97` — quad fills, layers, transformations; backends: `wgpu` (shader pipeline incl. `wgpu/src/shader/quad/shadow.wgsl`) and CPU `tiny_skia`; primitive-level damage tracking limits repaint regions (`graphics/src/damage.rs:5-27`).

---

## 2. Q4 — can a StylePlan generate a Catalog?

### 2.1 Exact `Catalog` trait shape (button family)

`widget/src/button.rs:551-560`:

```rust
pub trait Catalog {
    type Class<'a>;                                    // per-widget style class
    fn default<'a>() -> Self::Class<'a>;               // class used if none set
    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style;
}
```

- Companion alias: `pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme, Status) -> Style + 'a>` (`button.rs:563`).
- Stock impl: `impl Catalog for Theme { type Class<'a> = StyleFn<'a, Self>; ... fn style(...) { class(self, status) } }` (`button.rs:565-575`) — i.e. **the default "class" is itself a boxed style closure**.
- Built-in style fns: `primary`/`secondary`/`success`/`warning`/`danger`/`text`/`background`/`subtle` (`button.rs:578-707`), all reading `theme.palette()`.
- Per-widget class setter: `Button::style(closure)` (`button.rs:190-196`, requires `Theme::Class<'a>: From<StyleFn>`) or `Button::class(...)` gated on `feature = "advanced"` (`button.rs:199-204`).
- Same shape repeats in **20 widget families** (`grep "trait Catalog"` → 20 files under `widget/src/`). Variations:
  - `container::Catalog`: **no `Status`** — `fn style(&self, class) -> Style` (`widget/src/container.rs:476-485`); `StyleFn = Box<dyn Fn(&Theme) -> Style>` (`container.rs:488`).
  - `checkbox::Catalog` adds a third method `fn selection(&self) -> Color` (`widget/src/checkbox.rs:551-563`).
  - `text_input::Catalog` matches button shape (`widget/src/text_input.rs:551-560`).

### 2.2 What is in a `button::Style`? (`widget/src/button.rs:468-480`)

| field | type | notes |
|---|---|---|
| `background` | `Option<Background>` | `Background::Color(Color)` or `Background::Gradient(Gradient)` (`core/src/background.rs:6-12`) |
| `text_color` | `Color` | cascaded to children via `renderer::Style { text_color }` (`button.rs:401-403`) |
| `border` | `Border` | `{ color: Color, width: f32, radius: Radius }` (`core/src/border.rs:6-15`) |
| `shadow` | `Shadow` | `{ color: Color, offset: Vector, blur_radius: f32 }` (`core/src/shadow.rs:5-14`) — **single** shadow |
| `snap` | `bool` | pixel-grid snapping; default `renderer::CRISP` (`button.rs:499`, `core/src/renderer.rs:13`) |

### 2.3 Feasibility of `impl Catalog for AeolusCatalog`

**Yes — fully supported.** `Catalog` is a plain public trait; `Button<'a, Message, W, Theme>` is generic over `Theme: Catalog` (`button.rs:71-74`), and `Program::Theme` only requires `theme::Base` (`program/src/lib.rs:35`, trait at `core/src/theme.rs:228-249`). A downstream crate can make `AeolusCatalog` its app theme type and back every widget-family `Catalog` with a static `(class-key, Status) → Style` map. Caveat: there is **no zero-view-work cascade** — styling a widget requires (a) one `Catalog` impl per widget family (~20), and (b) **each styled widget instance must carry its class key at construction time** (`.class(key)` or `.style(closure)`), because the class lives in the widget struct and is resolved per draw. Per-widget cost: exactly one builder call (`.class(K)`) plus a hash/lookup in `Catalog::style` at draw time; the view closure itself is unchanged otherwise.

Sketch:

```rust
use std::sync::Arc;
use iced::{widget::button, Theme as _}; // theme::Base is the Program-level trait

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Key { Primary, Danger, Ghost }          // Aeolus class keys

pub struct AeolusCatalog { plan: Arc<StylePlan> } // static map (family, Key, Status) -> Style

impl iced::theme::Base for AeolusCatalog {        // program/src/lib.rs:35, core/src/theme.rs:228
    fn default(preference: iced::theme::Mode) -> Self { /* ... */ }
    fn mode(&self) -> iced::theme::Mode { iced::theme::Mode::None }
    fn base(&self) -> iced::theme::Style { /* background_color + text_color */ }
    fn seed(&self) -> Option<iced::theme::palette::Seed> { None }
    fn name(&self) -> &str { "Aeolus" }
}

impl button::Catalog for AeolusCatalog {          // mirrors button.rs:551-560
    type Class<'a> = Key;
    fn default<'a>() -> Key { Key::Primary }
    fn style(&self, class: &Key, status: button::Status) -> button::Style {
        self.plan.button(*class, status)          // O(1) map lookup; falls back to defaults
    }
}
// ... repeat per widget family used (container, text_input, checkbox, ...)

// Widget construction (the unavoidable per-widget cost):
let b = button("Save").class(Key::Primary);       // needs feature = "advanced" (button.rs:199-204)
// or, without `advanced`, the default StyleFn route (button.rs:190-196):
let b = button("Save").style(move |_t, status| plan.button(Key::Primary, status));
```

### 2.4 Default Catalog impls & stock classes

- Every family ships `impl Catalog for Theme { type Class<'a> = StyleFn<'a, Self>; ... }` dispatching to the boxed closure (`button.rs:565-575`, `container.rs:496-506`, `text_input.rs:567-577`, `checkbox.rs:570-584`). This means **with the stock `iced::Theme`, the per-widget "class key" is literally a boxed closure**, and `container::StyleFn` even implements `From<Style>` so a constant style can be assigned directly (`container.rs:490-494`).
- Stock button classes: `primary`, `secondary`, `success`, `warning`, `danger`, `text`, `background`, `subtle` (`button.rs:578-707`); stock container classes: `transparent`, `rounded_box`, `bordered_box`, `dark`, `primary`, … (`container.rs:509-587`). They all derive from `Theme::palette()` swatches, so they re-style automatically under a different palette.
- The default button class resolves to `primary` (`button.rs:568-570`); disabled styling is applied by `disabled()` alpha-scaling background and text (`button.rs:718-726`).
- Class keys are hashed/compared per draw only through the `Status` match — classes themselves are zero-cost enums or boxed closures chosen by the integrator.

### 2.5 Status computation (exact code)

- **button** (`button.rs:342-354`, inside `Widget::update`): `on_press.is_none() → Status::Disabled`; else cursor over bounds → `Pressed` if internal `State.is_pressed` else `Hovered`; else `Active`. On status change → `shell.request_redraw()` (`button.rs:356-360`). **No `Focused` variant — keyboard focus never restyles a button.**
- **text_input** (`text_input.rs:370-380`): `Disabled` if `on_input.is_none()`; `Focused { is_hovered }` if `state.input.is_focused()`; `Hovered` if cursor over; else `Active` (`text_input.rs:521-533`).
- **checkbox** (`checkbox.rs:345-357`): `Disabled { is_checked }` if `on_toggle.is_none()`; `Hovered { is_checked }` if cursor over; else `Active { is_checked }` (`checkbox.rs:519-535`).

---

### 2.6 Redraw semantics around Status

`Button` caches `status: Option<Status>` (set only on `RedrawRequested`, `button.rs:356-357`) and requests a redraw whenever the computed status diverges (`button.rs:358-360`). `text_input` does the same with `last_status` (`text_input.rs:382-395`). Consequence for AEOLUS: a plan-driven Catalog gets fresh `(class, status)` queries automatically on every state transition — no widget recreation needed for hover/press/disable restyling, only for class-key changes.

## 3. Status/state mapping

| widget family | `Status` variants | citation |
|---|---|---|
| `button` | `Active`, `Hovered`, `Pressed`, `Disabled` | `widget/src/button.rs:451-462` |
| `text_input` | `Active`, `Hovered`, `Focused { is_hovered }`, `Disabled` | `widget/src/text_input.rs:521-533` |
| `checkbox` | `Active { is_checked }`, `Hovered { is_checked }`, `Disabled { is_checked }` | `widget/src/checkbox.rs:519-535` |
| `toggler` | `Active/Hovered/Disabled { is_toggled }` | `widget/src/toggler.rs:479-495` |
| `pick_list` | `Active`, `Hovered`, `Opened { is_hovered }`, `Disabled` | `widget/src/pick_list.rs:844-856` |
| `scrollable` | `Active/Hovered/Dragged {…}` + scrollbar hover/disabled flags | `widget/src/scrollable.rs:3450-3470` |
| `container` | **none** — `style(&self, class)` only | `widget/src/container.rs:484` |

- **Lossless Tailwind mappings:** `hover:` → `Hovered`; `disabled:` → `Disabled`; `active:` → `Active`/`Pressed` (button "active" Tailwind is usually mousedown → `Pressed`); `focus:` → only `text_input::Focused` (buttons, checkboxes, togglers have **no** focus styling state); `checked:` → `is_checked` payloads (checkbox/toggler).
- **Not representable:** `focus-within:` (no status on containers; no ancestor→descendant style propagation), `group-hover:`/`peer-*:` (no sibling/ancestor state access anywhere — status is computed locally from `cursor.is_over(layout.bounds())` inside each widget's `update`), `first:/last:`, `visited:`. Closest structural feature: the `hover` widget (`widget/src/hover.rs:20-38`) shows a *second widget* when the base is hovered — layout-level, not style-level. `Themer` (`widget/src/themer.rs:17-53`) can override background/text-color for a subtree but not per-status styling.

---

## 4. Style value coverage table

| property | supported? | exact type | citation |
|---|---|---|---|
| background solid | yes | `Background::Color(Color)` (`Color { r,g,b,a: f32 }`, `core/src/color.rs:20`) | `core/src/background.rs:6-12` |
| background gradient | yes | `Background::Gradient(Gradient)`; only `Gradient::Linear { angle: Radians, stops: [Option<ColorStop>; 8] }` (8 stops max) | `core/src/gradient.rs:9-12, 29-50` |
| text color | yes | `Color`; cascades via `renderer::Style { text_color }` | `button.rs:473`, `core/src/renderer.rs:128-131`, `container.rs:316-318` |
| border width/color | yes | `Border { color: Color, width: f32, radius }` — single uniform width, no per-side border | `core/src/border.rs:6-15` |
| border radius (per-corner) | yes | `Radius { top_left, top_right, bottom_right, bottom_left: f32 }` | `core/src/border.rs:81-90` |
| shadow | yes, **single** | `Shadow { color, offset: Vector, blur_radius: f32 }` — one shadow per quad; no spread, no shadow list | `core/src/shadow.rs:5-14`, consumed at `button.rs:377-389` |
| multiple shadows | **no** | NOT FOUND — no `Vec<Shadow>` in any Style | — |
| padding | yes (per-side) but **widget property, not in Style** | `Padding { top, right, bottom, left: f32 }`, set via `.padding(...)` | `core/src/padding.rs:36-45`, `button.rs:130-133` |
| width/height | yes (widget property) | `Length::{Fit, Fill, FillPortion(u16), Shrink, Fixed(f32), Bounded{…}, Fluid(…)}` | `core/src/length.rs:6-42`, `button.rs:118-127` |
| gap | partial | `container` has **no gap**; `Column`/`Row` have uniform `spacing: f32` (`.spacing(Pixels)`) | `widget/src/column.rs:33, 81-83`; none in `container.rs` |
| font size | yes (text widget property) | `Option<Pixels>`; default 16.0 in renderer settings | `core/src/renderer.rs:171-175, 190-192` |
| font weight/props | yes | `Font { family: Family, weight: Weight(Thin..Black), stretch: Stretch, style: Normal/Italic/Oblique }` — per text, not in Catalog Style | `core/src/font.rs:6-15, 159-196` |
| opacity | partial | only `Image::opacity` / `Svg::opacity` (`f32`); no global widget opacity in Style; emulate with `Color::scale_alpha` | `widget/src/image.rs:132-137`, `widget/src/svg.rs:143-148`, `button.rs:718-726` |
| cursor (CSS cursor) | no in Style | cursor is `mouse::Interaction` returned from `mouse_interaction` (`Pointer` for enabled hovered button) | `button.rs:410-425` |

---

## 5. Theme hot-swap (Q6, iced side)

- **Theme type:** closed enum of 22 built-ins + `Custom(Arc<Custom>)` — `core/src/theme.rs:14-61`. Extensibility = `Theme::custom(name, palette::Seed)` producing the `Custom` variant (`theme.rs:91-103`); the palette itself is generated from a 6-color `Seed` into `Palette { background, primary, secondary, success, warning, danger, is_dark }` (`core/src/theme/palette.rs:8-23, 152-165`). `theme.extend(...)`: **NOT FOUND** — the 0.13 "user-extensible theme" mechanism is gone; a truly custom theme means implementing `theme::Base` for your own type (`theme.rs:228-249`).
- **Where stored:** per window, in the winit shell state: `theme: Option<P::Theme>`, `default_theme: P::Theme`, `style: theme::Style` (`winit/src/window/state.rs:23-26`); refreshed **on every view cycle**: `self.theme = program.theme(window_id); self.style = program.style(self.theme())` (`state.rs:220-222`). `Program::theme` default is `None` → fall back to `default_theme` built via `theme::Base::default(system_theme)` (`state.rs:56-58`, `state.rs:123-127`).
- **Reactive, no restart:** because `theme()` is re-queried each update and `view` re-runs each frame, mutating app state (e.g. `Message::ThemeChanged`) swaps the theme instantly — see the styling example: `.theme(Styling::theme)` (`examples/styling/src/main.rs:9-12`), state field `theme: Option<Theme>` (`main.rs:16-22`), cycling `Theme::ALL` (`main.rs:48-67`).
- **Flow to widgets:** window state theme → `UserInterface::draw(&mut renderer, &theme, &style, cursor)` (`runtime/src/user_interface.rs:557`) → `Widget::draw(..., theme: &Theme, style: &renderer::Style, ...)` → `theme.style(&self.class, status)` (`button.rs:375`, `container.rs:305`, `text_input.rs:409`, `checkbox.rs:396-401`). Subtree re-theming via `Themer` widget (`widget/src/themer.rs:125-158`).
- **Dark/light detection:** `theme::Mode { None, Light, Dark }` (`theme.rs:206-215`); `Theme::mode()` reads `palette().is_dark` (`theme.rs:277-283`; `palette.rs:22`, oklch `l < 0.6` at `palette.rs:587-589`). System preference: winit `WindowEvent::ThemeChanged` → `subscription::Event::SystemThemeChanged(mode)` (`winit/src/lib.rs:1020-1027`; `futures/src/subscription.rs:32`); Linux via `mundy` (`winit/src/lib.rs:497-509`); `ICED_THEME` env var override (`theme.rs:256-265`).

---

## 6. Render path: where style is resolved & applied

1. `Widget::update` recomputes `Status` on every event; a change fires `shell.request_redraw()` (`button.rs:342-360`, `text_input.rs:370-395`).
2. Per frame, `UserInterface::draw` walks the tree; each widget's `draw` resolves style **then**: `let style = theme.style(&self.class, self.status.unwrap_or(Status::Disabled))` (`button.rs:375`).
3. The `Style` is immediately lowered: `renderer.fill_quad(Quad { bounds, border, shadow, snap }, background)` (`button.rs:377-389`; shared helper `container::draw_background`, `container.rs:376-394`); children draw with the cascaded `renderer::Style { text_color }` (`button.rs:397-407`, `container.rs:312-326`).
4. **Granularity:** styling is immediate-mode — the whole tree is re-drawn every frame and style resolution happens per widget per frame; there is **no retained per-widget style cache**. What limits repaint work is primitive-level damage grouping (`graphics/src/damage.rs:5-27, 45-60`), so a style change on one widget repaints (at least) that widget's damage region, not the OS window. A theme swap repaints everything because every widget re-resolves `Catalog::style` with the new `&Theme`.

---

## 7. Capability matrix inputs

| aspect | iced value | citation |
|---|---|---|
| cascade model | per-widget `Class` + `StyleFn` closures; only `text_color` cascades down (via `renderer::Style`); no selector engine, no inheritance of other props | `button.rs:81,190-196,563`; `container.rs:316-318` |
| hover mapping | lossless: `Status::Hovered` from `cursor.is_over` | `button.rs:344-351` |
| focus mapping | partial: only `text_input::Status::Focused{is_hovered}`; buttons/checkboxes have no focus style state | `text_input.rs:370-380`; absent in `button.rs:451-462` |
| active/pressed mapping | `Pressed` (mouse-down) + `Active`; checkbox/toggler `is_checked` payload | `button.rs:459,347-348`; `checkbox.rs:521-529` |
| disabled mapping | `Status::Disabled` derived from missing handler (`on_press/on_toggle/on_input.is_none()`), alpha-scaled default styles | `button.rs:342-343,718-726` |
| responsive breakpoints | `Responsive` widget: `view: Box<dyn Fn(Size) -> W>` re-invoked with available size on layout — manual breakpoints possible, none built-in | `widget/src/responsive.rs:13-34` |
| container queries | NOT FOUND (Responsive is the closest analog) | — |
| dark mode | `theme::Mode` + `palette.is_dark` + system detection (winit/mundy) + `ICED_THEME` env | `theme.rs:206-215,277-283`; `winit/src/lib.rs:1020-1027` |
| theme hot-swap | reactive: `Program::theme` re-queried every update cycle (`state.rs:221`); no restart needed | `program/src/lib.rs:93-95`; `winit/src/window/state.rs:220-222` |
| typography | `Font{family,weight,stretch,style}` + `Pixels` size + `LineHeight`; per-widget, not theme-styled | `core/src/font.rs:6-15`; `core/src/renderer.rs:171-179` |
| shadows (multiple?) | single `Shadow{color,offset,blur_radius}` per quad; no list/spread | `core/src/shadow.rs:5-14` |
| per-corner radius | yes: `Radius{top_left,top_right,bottom_right,bottom_left}` | `core/src/border.rs:81-90` |
| gradients | linear only, 8 color stops, angle in Radians | `core/src/gradient.rs:9-12,45-50` |
| programmatic styling entry point | `impl Catalog` per family + `.class()` (feature `advanced`) or `.style(closure)` (default `StyleFn`) | `button.rs:190-204,551-575` |
| per-widget cost of styling | 1 class assignment per styled widget instance + O(1) map lookup per draw; Catalog impl per widget family (~20) | `button.rs:81,375` |

---

## 8. Renderer-side lowering notes (style → GPU)

- A resolved `Style` collapses into one `Quad { bounds, border, shadow, snap }` + a `Background` (`core/src/renderer.rs:101-113, 55`); the wgpu backend renders quads through dedicated shaders — `solid`, `gradient`, `snap`, and `shadow` variants (`wgpu/src/shader/quad/{solid,gradient,snap,shadow}.wgsl`), so border radii, gradients, and the single drop shadow are all first-class GPU paths.
- The quad shader pipeline means: per-corner radius is fully honored for backgrounds and borders; gradient backgrounds interpolate along `angle` with up to 8 stops; the shadow is drawn from the same quad geometry (offset + blur), which is why only one shadow per widget is expressible.
- `snap` (default `renderer::CRISP`, `core/src/renderer.rs:13`, feature `crisp`) controls pixel-grid snapping — relevant for AEOLUS when translating fractional Tailwind px values.

---

## 9. AEOLUS takeaways

1. **Q4 verdict:** a `StylePlan` *can* generate a Catalog — `impl button::Catalog for AeolusCatalog` with `type Class<'a> = Key` over a static `(Key, Status) → button::Style` map is a clean, supported integration; but it costs one `Catalog` impl per widget family **and** one `.class(key)`/`.style(closure)` call per styled widget instance at view time — no zero-view-work cascade exists.
2. **Theme hot-swap verdict:** fully reactive and cheap — theme is re-queried from `Program::theme` every update (`winit/src/window/state.rs:221`), widgets re-resolve `Catalog::style` per frame; `iced::Theme` is a closed enum (22 built-ins + `Custom(Arc<Custom>)`), with `theme::Base` as the open escape hatch for an `AeolusTheme`.
3. **Biggest surprise:** the old `theme.extend`/custom-theme-struct mechanism is gone — themes are either closed-enum variants, generated `Custom` palettes from a 6-color `Seed`, or a foreign `theme::Base` type; also buttons have no `Focused` styling state at all, and 20 widget families each define their own incompatible `Catalog`/`Status`/`Style` trio (container has no `Status`; checkbox's Catalog has an extra `selection()` method), so a generic Aeolus lowering must be per-family by design.
