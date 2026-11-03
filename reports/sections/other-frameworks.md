# AEOLUS Section Audit — Other Frameworks (vizia, slint, gpui-ce, freya, dioxus, leptos, egui, ribir)

- **Repos root:** `/home/z/my-project/repos/` (shallow clones of default branch, latest commit).
- **HEADs recorded** (all verified via `git rev-parse HEAD`).
- `leptos` and `ribir` arrived as empty/incomplete clones (no HEAD); re-cloned with
  `git clone --depth 1 --single-branch` from `https://github.com/leptos-rs/leptos` and
  `https://github.com/RibirX/Ribir` — both succeeded.
- `MANIFEST.txt` updated: `leptos` + `ribir` entries appended (they had no matching lines)
  with hashes and current UTC date; all other entries already matched, left untouched.
- Per framework we answer: (1) styling model, (2) cascade model, (3) responsive /
  container queries / dark mode, (4) theme hot-swap, (5) typography & key style values,
  (6) external restyling entry point for an AEOLUS-style backend.

| repo | HEAD |
|---|---|
| vizia | `9231260010d57380681d6bd2eca05b2b756463cb` |
| slint | `84972235182156b338756e0224aee65aa178d91c` |
| gpui-ce | `bc577476a0eb55173ef985585cfb487f75ab1cab` |
| freya | `9017e49c5632ae6eae6f0c0233f05fd7a8ffea10` |
| dioxus | `7dfd07eb7a7ef130f49de043327e762e24cd484e` |
| leptos | `37fcf7d72eb1bf5664ae906cfa5d19bc0a2334d9` |
| egui | `fc9d66dda742d722eedaa8684d23f86369cf8e5f` |
| ribir | `f9d9f5b09fe2d1ff8130f3211833831949319d5d` |

---

## 1. vizia — real CSS engine in Rust

### 1.1 Styling model
- Two ways to style, per the module docs at `crates/vizia_core/src/style/mod.rs:1-61`:
  - **Inline**: view modifiers, e.g. `Element::new(cx).background_color(Color::red())`
    (`style/mod.rs:13`); sets per-entity `inline_data` (`style/mod.rs:3506-3516`).
  - **Shared**: CSS text rules, applied to views via `.class("foo")` (`style/mod.rs:22`).
- CSS is parsed with Servo `cssparser` + `selectors`: `StyleSheet::parse`
  (`crates/vizia_style/src/stylesheet.rs:17-44`) → `CssRuleList` of
  `Style | Property | Keyframes | Ignored` (`vizia_style/src/rules/mod.rs:13-19`).
- Declarations become typed `Property` values (`vizia_style/src/property.rs`, ~290 lines;
  ~60 value modules under `vizia_style/src/values/`).
- Central store: `Style` struct with per-entity style storages
  (`crates/vizia_core/src/style/mod.rs:198`).

### 1.2 Cascade
- Genuine cascade: matched rules collected per entity with
  `rule.selector.specificity()` (`crates/vizia_core/src/systems/style.rs:1221,1260`),
  sorted by specificity desc, tie-broken by rule order
  (`systems/style.rs:1230-1231`).
- Inline properties override shared rules (`style/mod.rs:15,29`).
- Selectors: element name, `.class`, descendant combinators, `:has()`
  (`vizia_style/src/selector.rs:66-70`); pseudo-elements exist too
  (`vizia_style/src/pseudoelement.rs`).
- Runtime class toggling: `EventContext::toggle_class(name, applied)`
  (`crates/vizia_core/src/context/mod.rs:650`).

### 1.3 Responsive / container queries / dark mode
- No `@media`, no container queries — `CssRule` enum has no media variant
  (`vizia_style/src/rules/mod.rs:13-19`). NOT FOUND.
- Dark mode: `Environment.theme_mode` (user) vs OS-detected `system_theme_mode`,
  resolved by `effective_theme()` (`crates/vizia_core/src/environment.rs:107-122`).
- Resolution mechanism: root gets `.dark` class toggled at startup
  (`context/mod.rs:880-886`); default theme CSS keys off it.
- Default themes embedded: `DEFAULT_THEME = include_str!("resources/themes/default_theme.css")`
  (`context/mod.rs:77`).

### 1.4 Theme hot-swap
- Yes, full pipeline: `EventContext::reload_styles()` removes rules, re-concatenates
  every registered stylesheet string, re-parses (`Style::parse_theme`,
  `style/mod.rs:1142-1180`), then flags relayout/restyle for all entities
  (`crates/vizia_core/src/context/event.rs:835-862`).
- Triggers: `Context::add_stylesheet()` at any time (`context/mod.rs:850-856`);
  `EnvironmentEvent::SetThemeMode` (`environment.rs:133`); F5 key handler
  (`crates/vizia_core/src/events/event_manager.rs`, F5 → reload_styles);
  file-loaded stylesheets hot-reload via F5 (`style/mod.rs:52`).

### 1.5 Typography & key style values
- Fonts: size (incl. keywords), weight, width, slant, family
  (`vizia_style/src/values/font_size.rs`, `font_weight.rs`, `font_width.rs`,
  `font_slant.rs`, `font_family.rs`).
- Text extras: letter-spacing, line-height, text-decoration, text-stroke
  (`values/letter_spacing.rs`, `line_height.rs`, `text_decoration.rs`,
  `text_stroke.rs`).
- Radius: `corner-radius`, `outline-radius` (in test CSS `stylesheet.rs:62,109-110`).
- Shadows: `outer-shadow`, `inner-shadow` (`values/shadow.rs`; `stylesheet.rs:72,102`).
- Opacity (`values/opacity.rs`), transitions + `@keyframes` (`rules/keyframes.rs`).
- CSS custom properties: `Property` rules + `Variable` resolution with typed
  fallbacks (`style/mod.rs:1710-1919`).

### 1.6 External restyling entry point
- **Strongest fit for AEOLUS**: any crate with access to `Context` can call
  `add_stylesheet(css_string)` at runtime — arbitrary CSS text is parsed and
  live-applied with specificity resolution and pseudo-class matching
  (`context/mod.rs:850-856`).
- Programmatic alternatives: `EventContext::toggle_class` (`context/mod.rs:650`);
  custom-property rules for token injection; `Style::parse_theme` is the parse+insert
  core (`style/mod.rs:1142`).
- Caveat: pseudo-class matching is driven by vizia's input system; an external backend
  cannot invent new pseudo-classes without patching (custom pseudo-class enum variant
  exists: `PseudoClass::Custom` at `vizia_style/src/pseudoclass.rs:36`).

---

## 2. slint — declarative DSL + property graph (no CSS)

### 2.1 Styling model
- `.slint` DSL: elements carry typed properties set by *bindings*; the runtime is a
  dependency-tracked property graph.
- Core machinery: `Property<T>` with `dirty: Cell<bool>`
  (`internal/core/properties.rs:456-457, 902`), `mark_dirty` +
  `mark_dependencies_dirty` propagation (`properties.rs:817-875`);
  two-way bindings generated by the compiler.
- Item visuals are plain properties: `Rectangle.border_radius`
  (`internal/core/items.rs:452,576`), `Text.font_size/weight/letter_spacing`
  (`internal/core/items/text.rs:50-61`).
- Whole widget look-and-feel is a compile-time-selected style
  (cupertino/material/cosmic/…) implemented as `.slint` under
  `internal/compiler/widgets/`. Runtime does not parse CSS.

### 2.2 Cascade
- No cascade, no selectors. State styling via `states [...]` blocks:
  `states [ foo when bar : { x:y; } ]` (`internal/compiler/parser/element.rs:915-918`),
  with `in`/`out` animate transitions (`element.rs:930-932`).
- States are property-condition driven; TouchArea exposes explicit state
  properties `pressed`, `has-hover` (`internal/core/items/input_items.rs:35-44`),
  set by the input system (`input_items.rs:104,130,147`).

### 2.3 Responsive / dark mode
- No media queries in the language (NOT FOUND); size-adaptive UI is written with
  layouts plus `states when <width comparison>` conditions.
- Dark mode: `ColorScheme { Unknown, Dark, Light }`
  (`internal/common/enums.rs:579-587`); platform sets
  `PlatformContext.color_scheme` (`internal/core/context.rs:50,416`).
- Widget styles read the scheme through globals:
  `internal/compiler/widgets/cupertino/color-scheme.slint:4-5`.

### 2.4 Theme hot-swap
- Yes for colors/tokens: widget styles expose `in-out` globals —
  `global Palette { ... in-out property <ColorScheme> color-scheme <=> ...; }`
  (`internal/compiler/widgets/cupertino/style-base.slint:22-35`).
- Rust mutates via `ComponentHandle::global()` accessor
  (`api/rs/slint/lib.rs:143,195-198`); the property graph re-evaluates dependents.
- The widget *style* itself (cupertino vs material) is fixed at build time
  (`internal/backends/selector`), not hot-swappable.

### 2.5 Typography & key style values
- `font-size`, `font-weight`, `letter-spacing` per text item
  (`items/text.rs:50-61`); root `default-font-size` (`items/text.rs:247`).
- `border-radius` (`items.rs:452,576`).
- `drop-shadow-blur/color/offset-x/offset-y/spread` on Rectangle — docs explicitly
  model CSS `filter: drop-shadow()` (`internal/compiler/builtin_elements.rs:646-664`).
- `opacity` on any element via the built-in Opacity item
  (`internal/core/items.rs:856-859`).

### 2.6 External restyling entry point
- Only global property writes: an external crate can `.set()` an exported global
  (e.g. a token-bus global) at runtime and all bound properties restyle.
- No runtime rule injection, no CSS parsing, no class toggling.
- AEOLUS mapping: emit a generated global with `in` properties and bindings, or
  drive `Palette.color-scheme`; cannot express selectors/states externally.

---

## 3. gpui-ce — Tailwind-flavored Rust refinements

### 3.1 Styling model
- Fluent, Tailwind-like Rust API. `Styled` trait exposes
  `fn style(&mut self) -> &mut StyleRefinement` (`crates/gpui/src/styled.rs:25-27`)
  plus macro-generated utilities (`styled.rs:36-44`).
- Full `Style` is a `#[derive(Refineable)]` struct ("The CSS styling that can be
  applied to an element via the `Styled` trait", `crates/gpui/src/style.rs:228-231`);
  per-element styling is a `StyleRefinement` layered on top via `refine()`
  (`styled.rs:30-34`; `impl Styled for StyleRefinement` at `style.rs:414-418`).
- Conditional building via `FluentBuilder::map/when`
  (`crates/gpui/src/util.rs:11-25`).
- Filters/backdrop-blur utilities (`styled.rs:81-117`).

### 3.2 Cascade
- No selector cascade; instead per-state style closures stored on `Interactivity`
  (`crates/gpui/src/elements/div.rs:2794-2868`):
  `base_style`, `focus_style`, `in_focus_style`, `focus_visible_style`,
  `hover_style`, `group_hover_style`, `active_style`, `group_active_style`,
  `disabled_style`, `drag_over_style` (`div.rs:2813-2827`).
- Setters: `hover_style` (`div.rs:1022`), `group_hover_style` (`1038`),
  `focus_style` (`1537`), `active_style` (`1900`), `disabled_style` (`931`),
  `transitions` (`942-950`).
- Traits: `InteractiveElement` (`div.rs:916`), `StatefulInteractiveElement`
  (`div.rs:1606`); state resolved from hitboxes at paint time.

### 3.3 Responsive / dark mode
- No container queries; no breakpoint helpers (NOT FOUND). Viewport-reactive styling
  is done with `.when(window bounds …)` closures.
- Dark mode: `App::window_appearance()` / `set_window_appearance()`
  (`crates/gpui/src/app.rs:1446-1460`); `Window::appearance()`
  (`window.rs:2843`); appearance-change plumbing (`window.rs:2828`).
- Default colors switch on appearance: `Colors::for_appearance()`
  (`crates/gpui/src/colors.rs:37-56`).

### 3.4 Theme hot-swap
- Implicitly yes: elements are re-built every frame, so any state change restyles
  on next paint. A `Global` theme object mutated via `cx.set_global` is the
  idiomatic theme bus.
- Dark mode hot-switches via `set_window_appearance` (`app.rs:1460`).
- No stylesheet reload concept (nothing to reload).

### 3.5 Typography & key style values
- `TextStyle` with `font_size`, `line_height`, `font_weight` etc.
  (`style.rs:652-680`); `TextStyleRefinement` for per-element overrides
  (`style.rs:826`).
- Shadows: `box_shadow_style_methods!` macro (`styled.rs:44`).
- Opacity: `.opacity(f32)` (`styled.rs:970`).
- Corner radius via macro helpers + `rounded_smoothing` (`styled.rs:1131`).

### 3.6 External restyling entry point
- No rule engine and no stylesheet API.
- Options: (a) register/mutate a `Global` theme read inside style closures;
  (b) generate components that apply `StyleRefinement`s programmatically.
- The state closures (`hover_style` etc.) are the only "rule" mechanism, attached
  at element construction — AEOLUS must emit Rust, not data.

---

## 4. freya — Rust builder elements + state-based theme

### 4.1 Styling model
- Skia-rendered Rust builder elements. Core style API writes typed structs:
  `BackgroundExt::background` → `StyleState`
  (`crates/freya-core/src/elements/extensions.rs:956-977`);
  `StyleExt::style_mut/style` for whole-struct replacement (`extensions.rs:966-978`).
- `StyleState { background, corner_radius, borders, shadows }`
  (`crates/freya-core/src/data.rs:101-106`); `TextStyleState`
  (`data.rs:127-141`).
- Components (Button, Card, …) pull values from a `Theme` object, overridable
  per instance with theme partials (`crates/freya-components/src/button.rs:27-40,
  205-211`).
- `freya-css` / `FreyaCN` / Tailwind integration: NOT FOUND anywhere in the repo.

### 4.2 Cascade
- None. Interaction styling is hand-written per component: Button tracks
  `hovering`/`enabled` state and picks `hover_background` vs border colors
  (`button.rs:255-311`).
- Theme color sheet carries `focus`/`active`/`disabled` tokens consumed by
  components (`crates/freya-components/src/theming/component_themes.rs:95-98`).

### 4.3 Responsive / container queries / dark mode
- No media/container queries (NOT FOUND).
- Dark mode: `light_theme()` / `dark_theme()` constructors
  (`crates/freya-components/src/theming/themes.rs:630,638`); selection is manual
  (`examples/theme_switcher.rs`), no OS detection found in `freya-winit`/`freya-core`.

### 4.4 Theme hot-swap
- Yes: theme is a reactive `State<Theme>` in context.
- Hooks: `use_init_theme`, `use_provide_theme`, `use_theme`
  (`crates/freya-components/src/theming/hooks.rs:15-50`).
- Canonical example: `theme.set(if is_light { light_theme() } else { dark_theme() })`
  (`examples/theme_switcher.rs:20-37`).

### 4.5 Typography & key style values
- `TextStyleState`: font_size, weight, width, slant, letter-spacing, features,
  text-decoration, text-shadows (`data.rs:127-141`).
- `StyleState`: corner_radius, borders, shadows (`data.rs:101-106`);
  `opacity` (`data.rs:93`; `RectElement` setter `rect.rs:722`).
- Style value modules under `crates/freya-core/src/style/` (`shadow.rs`,
  `font_size.rs`, `gradient.rs`, `letter_spacing.rs`, …).

### 4.6 External restyling entry point
- Mutate the provided `State<Theme>` from any code holding a context hook; theme
  carries per-component maps (`Theme::get/set`, `component_themes.rs:13-37`).
- Or set explicit style values on elements (`.background(...)`, `StyleExt`).
- No rule language; AEOLUS would emit a theme object + component partials.

---

## 5. dioxus — HTML attributes; CSS lives outside the framework

### 5.1 Styling model
- HTML-first: RSX emits elements with attributes; `class` is a real HTML class
  attribute with conditional-class syntax
  (`packages/html/src/attribute_groups.rs:151-193`).
- Web: attributes reach the real DOM via the interpreter —
  `packages/interpreter/src/js/set_attribute.js`: `style` namespace →
  `node.style.setProperty`, others → `setAttribute`.
- Native: renderer is blitz ("Native renderer for Dioxus based on blitz",
  `packages/native/Cargo.toml:6`; `blitz-dom`/`blitz-paint` deps at
  `Cargo.toml:73-76`).
- Native mapping: `style:` namespace attrs → inline style properties
  (`packages/native-dom/src/mutation_writer.rs:497-505`); `class` becomes a DOM
  attribute consumed by blitz/stylo's real CSS engine (external crate).

### 5.2 Cascade
- None of its own — delegated entirely to CSS (browser on web; stylo inside blitz
  on native). Selector matching, pseudo-classes, specificity all external.

### 5.3 Responsive / dark mode
- Via CSS only; nothing in-repo (NOT FOUND). Media queries and
  `prefers-color-scheme` handled by the host engine.

### 5.4 Theme hot-swap
- Reactivity re-renders with new class/style attributes.
- `Document` abstraction supports runtime head manipulation:
  `Document::create_head_element` and `eval`
  (`packages/document/src/document.rs:64-74`), implemented for native-dom too
  (`packages/native-dom/src/dioxus_document.rs:155`) — a `<style>` element can be
  created/updated at runtime on both targets.

### 5.5 Typography & key style values
- Not modeled by dioxus itself — whatever CSS provides (NOT FOUND in-repo; no
  style value types in the workspace).

### 5.6 External restyling entry point
- Yes, pragmatically: `document().eval(js)` on web; construct/update `<style>`
  nodes via `Document::create_head_element`; inline `style:` attributes on any
  element (`mutation_writer.rs:497-505`).
- Untyped — no rule API; native selector support depends on blitz (external).

---

## 6. leptos — reactive classes over real CSS

### 6.1 Styling model
- Same philosophy as dioxus: view layer emits real HTML/SVG with class/style
  attributes.
- `class()` helper returns a `Class` attribute
  (`tachys/src/html/class.rs:12-25`); supports reactive toggles — `rebuild`
  applies/removes classes on signal change (`class.rs:75-81`).
- Renderers in-repo: DOM (`tachys/src/renderer/dom.rs:468 class_list`),
  sledgehammer, mock — web only.

### 6.2 Cascade
- None — standard browser CSS cascade applies externally; no selector engine.

### 6.3 Responsive / dark mode
- External CSS (media queries, `prefers-color-scheme`); NOT FOUND in repo.
- No native renderer in-repo (the "cabin" ecosystem is external; no references).

### 6.4 Theme hot-swap
- Signals re-drive class/style attributes; no framework theming layer.

### 6.5 Typography & key style values
- External CSS. NOT FOUND.

### 6.6 External restyling entry point
- Insert a `<style>` node via a view or DOM interop; that is the whole story —
  no API beyond the DOM.

---

## 7. egui — immediate mode, per-frame style reads (+ experimental theme plugins)

### 7.1 Styling model
- Immediate mode: one global `Style` (`crates/egui/src/style.rs:244-342`) holding
  `Visuals` (`style.rs:1099`), `Spacing` (`style.rs:386`), `text_styles`.
- Per-`Ui` / per-widget overrides via `StyleModifier` closures
  (`style.rs:194-230`; `Ui::style_mut` at `crates/egui/src/ui.rs:406`).
- Widgets read the style fresh each frame: Button resolves an `AtomLayoutStyle`
  through `ui.widget_style(...)` (`crates/egui/src/widgets/button.rs:348-353`).

### 7.2 Cascade
- No cascade. State visuals chosen at paint time:
  `Widgets { noninteractive, inactive, hovered, active, open }`
  (`style.rs:1360-1380`).
- `Widgets::style(response)` picks active when pressed/focused/clicked, hovered
  when hovered/highlighted (`style.rs:1383-1394`).
- Selection visuals via `Style::interact_selectable` (`style.rs:360-369`).

### 7.3 Responsive / dark mode
- No media/container queries; point-based sizing with
  `Context::set_pixels_per_point` scaling.
- Dark mode: two full `Style`s kept (`dark_style`/`light_style`,
  `crates/egui/src/context.rs:2647-2648`), selected by
  `Context::set_theme(ThemePreference)` (`context.rs:2594`); system preference
  detection (`context.rs:2969-2970`).

### 7.4 Theme hot-swap
- Trivial: `set_style_of` / `style_mut_of` / `set_visuals_of` replace or mutate the
  active `Style`/`Visuals` any time (`context.rs:2661-2704`); takes effect next
  frame. Documented intent at `style.rs:236-238`.

### 7.5 Typography & key style values
- `text_styles: BTreeMap<TextStyle, FontId>` + `override_font_id`
  (`style.rs:249-280`).
- `CornerRadius` on widgets/windows (`style.rs:1418,1629`).
- `Shadow { offset, blur, spread, color }` (`crates/epaint/src/shadow.rs:10`).
- Opacity is per-color alpha (Color32), not a per-widget property.

### 7.6 External restyling entry point
- Classic route: `Context::set_style_of(theme, style)` / `set_visuals_of`
  (`context.rs:2674, 2691`).
- **Experimental theme-plugin system** (`experimental_theme` feature):
  `StyleProvider` computes a widget style from
  `StyleArgs { classes, state, style, stack }` every frame
  (`crates/egui/src/theme/style_provider.rs:6-17`; resolution in
  `theme/mod.rs:26-48`).
- Register with `Context::add_widget_theme`; hot-replace with
  `replace_widget_theme` — doc says "This allow to live edit a theme"
  (`context.rs:2458, 2473-2482`).
- Widgets carry `Classes` (`crates/egui/src/class/classes.rs:15-17`) — effectively
  a programmatic class+state rule engine, the closest existing analogue to an
  external AEOLUS injection point.

---

## 8. ribir — Rust DSL with a class-transformer system

### 8.1 Styling model
- Declarative Rust DSL: `fn_widget!` / `rdl!` blocks with built-in style fields —
  one file per field under `core/src/builtin_widgets/`: `background.rs`,
  `border.rs`, `radius.rs`, `box_shadow.rs`, `opacity.rs`, `padding.rs`,
  `margin.rs`, `text_style.rs`.
- The `Declare` macro wraps widgets in `FatObj` to accept these fields
  (`macros/src/lib.rs:56-74`).
- No CSS text parser anywhere; "css" appears only in doc comments
  (`core/src/builtin_widgets/class.rs:3`).
- Toolchain: stable Rust now — edition 2024, `rust-version = "1.92.0"`
  (`Cargo.toml:53-59`); not nightly-only anymore.

### 8.2 Cascade
- No selector cascade; a two-level class system instead:
  - `class_names!` declares names; the `Class` widget resolves a name to a
    `ClassImpl` widget-transformer from `Theme.classes`
    (`core/src/builtin_widgets/class.rs:1-9, 51-55`).
  - `style_class!` builds a transformer from builtin-field declarations
    (`class.rs:60-71`); `named_style_impl!` defines named styles
    (`class.rs:86-100`).
- Widget states are flags on `MixBuiltin`: `is_hovered()` (`mix_builtin.rs:147`),
  `is_pointer_pressed()` (`153`), tracing setup (`446-456`), plus a `Disabled`
  builtin (`disabled.rs`) and focus scopes.

### 8.3 Responsive / dark mode
- No media/container queries (NOT FOUND).
- Responsive behavior is user-written against reactive state; window size is
  readable via `Window::size()` (`core/src/window.rs:303`) and fed through
  ribir's `pipe` data-flow.
- Dark mode: no OS detection found; app picks `material` or `ribir_slim`
  (top-level `themes/` dir).

### 8.4 Theme hot-swap
- Yes: `Theme` is a `StateWriter` provided down the tree
  (`core/src/builtin_widgets/theme.rs:104-139`); `Theme::write_of(ctx)`
  (`theme.rs:107`) mutates it at runtime and dependents (palette readers, class
  providers) rebuild.
- Startup: `App::with_app_theme(...)` (`ribir/src/app.rs:363,377`).

### 8.5 Typography & key style values
- `Theme { palette, typography_theme, classes, fonts... }`
  (`core/src/builtin_widgets/theme.rs:68-87`).
- `TypographyTheme` (`theme/typography_theme.rs`); `TextStyle` builtin
  (`text_style.rs`).
- Radius (`radius.rs`), box shadow (`box_shadow.rs`), opacity (`opacity.rs`);
  transitions/animations (`core/src/animation.rs:1-12`).

### 8.6 External restyling entry point
- Programmatic only: insert into `theme.classes` at runtime — doc example
  `theme.classes.insert(RED_BORDER, style_class! { border: ... })`
  (`class.rs:18-28`).
- Or compose `style_class!` / `named_style_impl!` transformers; no rule
  language, no CSS parsing. AEOLUS must emit Rust closures.

---

## Capability matrix

| Framework | Cascade model | States supported | Responsive / container q. | Dark mode | Hot-swap | Typography | Shadows | External rule injection (AEOLUS fit) |
|---|---|---|---|---|---|---|---|---|
| **vizia** | Real CSS cascade: specificity sort + rule order (`systems/style.rs:1221-1231`); inline > shared | Full pseudo-class set: hover/active/over/focus(-visible/-within)/disabled/checked/valid/... (`pseudoclass.rs:7-37`) | No `@media`, no container queries (`rules/mod.rs:13-19`) | OS detect + `dark` root class (`environment.rs:107-122`, `context/mod.rs:884`) | Yes — `add_stylesheet`/`reload_styles` re-parses CSS live (`event.rs:835-862`); F5 reload | font size/weight/width/slant/family, letter-spacing, line-height, text-stroke | `outer/inner-shadow` (`values/shadow.rs`) | **Excellent** — inject raw CSS strings at runtime (`context/mod.rs:850-856`), full selector+state matching |
| **slint** | None — typed property bindings + `states when` (`parser/element.rs:915-932`) | TouchArea pressed/has-hover (`input_items.rs:35-44`); author-defined `when` conditions | No; `states when width…` hand-written | `ColorScheme` enum fed by platform (`enums.rs:579-587`, `context.rs:416`) | Yes via `in-out` globals e.g. `Palette.color-scheme` (`style-base.slint:34`; `global()` `lib.rs:195`); widget style fixed at build | font-size/weight/letter-spacing (`items/text.rs:50-61`); default-font-size | `drop-shadow-*` on Rectangle (`builtin_elements.rs:646-664`) | Weak — set exported global properties only; no runtime rules |
| **gpui-ce** | None; state-closure refinements on `Interactivity` (`div.rs:2813-2827`) | hover, group-hover, active, group-active, focus, in-focus, focus-visible, disabled, drag-over (`div.rs:1022,1038,1537,1900,931,2825`) | No container queries; `.when(window…)` closures only | `WindowAppearance` light/dark/vibrant (`app.rs:1446-1460`; `colors.rs:37-42`) | Implicit — per-frame re-eval; `Global` theme mutation + `set_window_appearance` | TextStyle size/weight/line-height/features (`style.rs:652-680`) | box-shadow macros (`styled.rs:44`); filters/backdrop blur (`styled.rs:81-117`) | Medium — programmatic `StyleRefinement` + `Global` themes; no rule matching |
| **freya** | None — per-component hand-written state logic (`button.rs:255-311`) | hover/enabled per component; focus/active/disabled tokens in `ColorsSheet` (`component_themes.rs:95-98`) | No (NOT FOUND) | `light_theme()`/`dark_theme()` manual (`themes.rs:630,638`); no OS detect | Yes — reactive `State<Theme>`, `theme.set(...)` (`hooks.rs:15-50`, `theme_switcher.rs:33-36`) | TextStyleState size/weight/width/slant/features (`data.rs:127-141`) | `StyleState.shadows` + text shadows (`data.rs:105,134`) | Medium — mutate reactive `State<Theme>` / per-instance theme partials (`button.rs:205`) |
| **dioxus** | Delegated to CSS (browser web; stylo via blitz native — external) | Delegated to CSS engine (external) | Delegated to CSS (NOT FOUND in-repo) | Delegated to CSS / host | Reactive attrs re-render; `Document::create_head_element`/`eval` for runtime `<style>` (`document.rs:64-74`) | Not modeled in-repo (CSS external) | Not modeled in-repo | Yes but untyped — inject `<style>` node / inline `style:` attrs (`mutation_writer.rs:497-505`) |
| **leptos** | None — plain classes on real DOM (`tachys/src/html/class.rs:14`) | None in-framework (browser CSS) | External CSS (NOT FOUND) | External CSS | Signals re-drive class attrs; no theme layer | Not modeled (CSS external) | Not modeled | Weak — emit/replace `<style>` via views or DOM interop |
| **egui** | No cascade; per-state `WidgetVisuals` chosen at paint (`style.rs:1383-1394`) | noninteractive/inactive/hovered/active/open + focus, selected, disabled, highlighted (`style.rs:1360-1380`, `360-369`) | None (point scaling only) | Dual dark/light `Style`s + `set_theme` (`context.rs:2594, 2647-2648`) | Trivial — replace `Style`/`Visuals` anytime (`context.rs:2661-2704`) | `text_styles` map + `override_font_id` (`style.rs:249-280`) | `Shadow { offset, blur, spread, color }` (`epaint/src/shadow.rs:10`) | **Good** — `set_style_of` (`context.rs:2674`) + experimental `StyleProvider` plugins keyed by `Classes`+state, hot-replaceable (`context.rs:2458,2473`) |
| **ribir** | No selectors; `Class` name → widget-transformer from `Theme.classes` (`class.rs:51-71`) | hover/pressed flags on `MixBuiltin` (`mix_builtin.rs:147,153`), disabled, focus | No media queries; reactive pipes on `Window::size()` (`window.rs:303`) | No OS detect; app picks material/ribir_slim | Yes — `Theme` is a `StateWriter`; `Theme::write_of` mutates live (`theme.rs:104-139`) | TypographyTheme + TextStyle builtin (`theme.rs:70`, `text_style.rs`) | `box_shadow.rs`, `radius.rs` | Medium — insert `style_class!` transformers into `Theme.classes` at runtime (`class.rs:18-28`); no CSS parser |

### Cross-cutting observations for AEOLUS

- **Only vizia has a genuine CSS pipeline** (parse → selector match → specificity
  sort → pseudo-class states → custom properties). It is the natural first backend
  target: AEOLUS-generated CSS text can be fed to `add_stylesheet` unchanged.
- **egui's experimental `StyleProvider`** (registered per `WidgetStyle` type,
  computing style from `Classes` + `WidgetState` every frame, hot-replaceable) is
  the only other framework-side hook that accepts arbitrary programmatic style
  logic per class/state at runtime.
- **gpui-ce / freya / ribir / slint** require *generated code or global property
  writes* rather than data-driven rules: AEOLUS must lower a StylePlan into
  builder calls / class-transformers / globals for these.
- **dioxus / leptos** shift all cascade and state work to CSS itself; on the web
  AEOLUS output is literally CSS classes; on native (dioxus+blitz) inline
  `style:` attributes and `<style>` elements reach stylo.
- **State coverage varies sharply**: vizia (20+ pseudo-classes) > gpui-ce
  (hover/active/focus/focus-visible/disabled/drag-over incl. group variants) >
  egui (5 visual states) > slint/ribir/freya (hover/pressed/disabled,
  author-managed).
