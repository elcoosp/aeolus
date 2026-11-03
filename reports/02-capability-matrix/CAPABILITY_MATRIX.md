# Backend capability matrix (deliverable 2)

Sources: `reports/sections/*.md` (audited at recorded commits). "✅ native"
= the framework itself provides the primitive so Aeolus lowers into it with
zero per-view work. "⚠️ fallback" = Aeolus must do per-view work (documented
fallback path). "❌" = not representable; reported via `LoweringReport`.

## Cascade model per framework

| framework | cascade primitive | hover/focus/active/disabled | responsive | dark | container queries | group/peer | theme hot-swap | typography |
|---|---|---|---|---|---|---|---|---|
| **Masonry/Xilem** `9f504ae` | 4-level: local props → `PropertyStack`(+`Selector`) → `DefaultProperties` → static default; relevance-gated cache | ✅ `Selector` flags (hover/active/disabled/focus) | ❌ no viewport signal (resize_observer only) | ❌ per-widget; host `set_default_properties` | ❌ | ❌ (has_hovered exists but not class-scoped) | ✅ `RenderRoot::set_default_properties` | ⚠️ parley `StyleSet` per widget; no FontSize/FontWeight properties |
| **Floem** `1351ffb` | style stack → class rules → inherited; nested selector maps; disabled > active > focus > hover | ✅ 10 `StyleSelector`s incl. focus-visible/focus-within/selected/drag | ✅ `min_window_width`/responsive selectors | ✅ `dark_mode` selector + OS theme events | ❌ | ❌ | ✅ reactive class rules + `set_global_theme` | ✅ font size/weight/family props |
| **Iced** `8ff48c4` (0.15.0-dev) | `Catalog` per widget family: (class, `Status`) → `Style`; **no cascade** | ⚠️ Status enums (button lacks Focused) | ❌ | ⚠️ closed Theme enum + `Custom(Seed)` | ❌ | ❌ | ✅ `Program::theme()` re-queried per update | ⚠️ font is a widget construction prop, outside Catalog |
| **Vizia** (see sections) | real CSS engine (cssparser/selectors), specificity-sorted | ✅ 20+ pseudo-classes | ⚠️ | ✅ | ❌ | ❌ | ✅ `add_stylesheet` live | ✅ |
| **Slint** | DSL property bindings; `states when`; no cascade | ⚠️ states | ⚠️ via globals | ✅ `Palette.color-scheme` | ❌ | ❌ | ✅ global swap | ✅ (DSL) |
| **GPUI-CE** | `StyleRefinement` + interactivity closures; no cascade | ✅ rich state closures | ⚠️ | ✅ `WindowAppearance` | ❌ | ❌ | ⚠️ | ✅ |
| **Freya** | Rust builders + `StyleState`; no cascade | ⚠️ flags | ❌ | ✅ hot-swappable `Theme` state | ❌ | ❌ | ✅ `theme.set` | ✅ (skia text) |
| **Dioxus/Leptos** (web) | real CSS (class attr) | ✅ (browser) | ✅ (media queries) | ✅ | ✅ | ✅ | ✅ | ✅ |
| **egui** | immediate-mode `Style`; no cascade; `StyleProvider`+`Classes` plugin = programmatic rules | ⚠️ per-widget visuals each frame | ⚠️ | ✅ trivial (replace Style) | ❌ | ❌ | ✅ trivial | ✅ |
| **Ribir** | Rust DSL + class-transformers (`Theme.classes`, `style_class!`) | ⚠️ hover/pressed flags | ❌ | ✅ Theme hot-writable | ❌ | ❌ | ✅ | ✅ |

## Aeolus slot support per v1 backend

| StylePlan slot | aeolus-xilem (fallback widget) | aeolus-floem | notes |
|---|---|---|---|
| background | ✅ `Background::Color` | ✅ `Brush::Solid` | gradients parse, dropped+reported |
| content_color / text colors | ✅ `ContentColor` | ✅ `TextColor` | |
| padding / margin | ✅ per-side Padding / ❌ margin | ✅ per-side both | Masonry has no margin property |
| border_width / border_color | ✅ (uniform width) / ✅ | ✅ per-side both | |
| corner_radius | ✅ single (max of 4) | ✅ per-corner | |
| dimensions (fixed) | ✅ `Dim::Fixed` | ✅ `LengthAuto::Pt` | fraction/full dropped+reported |
| gap / layout (flex) | ✅ gap / ❌ layout slot | ✅ gap / ✅ flex props | Xilem fallback centers via flag |
| shadow | ✅ (no spread/inset) | ✅ (spread + signed offsets) | Masonry upstream TODO |
| opacity | ❌ (dropped) | ✅ `opacity` | |
| ring / outline / cursor / pointer-events | ❌ | ❌ (ring) / ⚠️ | reported |
| text (font size, weight…) | ❌ parley per-widget | ✅ font_size; ⚠️ others | Q5a gap on Masonry side |
| hover/focus/focus-visible/active/disabled | ✅ Selector flags | ✅ selector maps | |
| focus-within | ❌ (reported) | ✅ `focus_within` | Masonry Selector lacks the flag |
| dark | ❌ (reported; host-level swap) | ✅ `dark_mode` | |
| responsive sm..2xl | ❌ (reported) | ✅ `min_window_width` | |
| container (@*) | ❌ | ❌ | both reported |
| group-*/peer-* | ❌ (patch proposal #1) | ❌ | no ancestor/sibling state anywhere |

## Theme switching (Q6 summary)

- Masonry: `RenderRoot::set_default_properties` invalidates every widget's
  property cache; `WindowView::with_default_properties` for per-window.
- Floem: reactive — swap class rules / `set_global_theme`; dark toggles
  restyle the whole tree via the `DarkMode` selector dirty bit.
- Iced: `Program::theme()` is re-queried every update cycle (no restart).
- egui: replace `Style` at frame start. Slint: global property. Vizia:
  `add_stylesheet` at runtime. Freya: `theme.set(...)`.
- Aeolus contract: `TailwindTheme::absorb_css` (bumps `version`, drop the
  `PlanCache`) — explicit, no CSS-variable pretending.
