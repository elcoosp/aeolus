# Backend capability matrix (deliverable 2)

Sources: `reports/sections/*.md` (audited at recorded commits). "✅ native"
= the framework itself provides the primitive so Aeolus lowers into it with
zero per-view work. "⚠️ fallback" = Aeolus must do per-view work (documented
fallback path). "❌" = not representable; reported via `LoweringReport`.

## Cascade model per framework

> v0.8 §1 note: the per-edge families (`p-*`/`m-*`/`border-*`/`rounded-*`)
> carry a set-mask in the IR; conditional layers no longer wipe the base's
> masked-off edges on ANY backend (`p-4 hover:px-8` keeps
> `padding-top: 16px` everywhere). Xilem/Slint materialize layers against
> the base before lowering; Floem applies per-field gated by the mask;
> Iced/egui merge the lowered payload edge-wise.

| framework | cascade primitive | hover/focus/active/disabled | responsive | dark | container queries | group/peer | theme hot-swap | typography |
|---|---|---|---|---|---|---|---|---|
| **Masonry/Xilem** `9f504ae` (= upstream `dc49687` core, v0.6 §3.2) | 4-level: local props → `PropertyStack`(+`Selector`) → `DefaultProperties` → static default; relevance-gated cache | ✅ `Selector` flags (hover/active/disabled/focus) | ❌ no viewport signal (resize_observer only) | ❌ per-widget; host `set_default_properties` | ❌ | ❌ (has_hovered exists but not class-scoped) | ✅ `RenderRoot::set_default_properties` | ⚠️ parley `StyleSet` per widget; no FontSize/FontWeight properties |
| **Floem** `1351ffb` (= latest `main` HEAD, v0.6 §3.1) | style stack → class rules → inherited; nested selector maps; disabled > active > focus > hover | ✅ 10 `StyleSelector`s incl. focus-visible/focus-within/selected/drag | ✅ `min_window_width`/responsive selectors | ✅ `dark_mode` selector + OS theme events | ❌ | ❌ | ✅ reactive class rules + `set_global_theme` | ✅ font size/weight/family props |
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
| background | ✅ `Background::Color` | ✅ `Brush::Solid` | v0.6: the solid is the `background-color` declaration; gradients are a separate slot |
| background_gradient (v0.6) | ✅ `Background::Gradient` — NATIVE (layout-invariant `GradientShape`, oklab interpolation; linear exact, radial = documented circle normalization, conic reported: upstream vello sweep TODO) | ❌ reported — peniko `Brush::Gradient` needs concrete geometry; `paint_bg` fills as-is (no rect adaptation) | the v0.1–v0.5 audits missed the property at the pin; linear angle conversion `a = π − θ` unit-pinned |
| content_color / text colors | ✅ `ContentColor` | ✅ `TextColor` | |
| padding / margin | ✅ per-side Padding / ❌ margin | ✅ per-side both | Masonry has no margin property |
| border_width / border_color | ✅ (uniform width) / ✅ | ✅ per-side both | |
| corner_radius | ✅ single (max of 4) | ✅ per-corner | |
| dimensions (fixed) | ✅ `Dim::Fixed` | ✅ `LengthAuto::Pt` | fraction/full dropped+reported |
| gap / layout (flex) | ✅ gap / ❌ layout slot | ✅ gap / ✅ flex props | Xilem fallback centers via flag |
| shadow | ✅ (no spread/inset) | ✅ (spread + signed offsets) | Masonry upstream TODO |
| opacity | ⚠️ multiplied into the painted chrome colors (children not faded — documented) | ✅ `opacity` | |
| ring / outline | ✅ stroked (`border_box.inflate(offset + width/2)`) | ✅ ring → `outline` + `outline_color` | Masonry has no spread; reported |
| cursor / pointer-events | ❌ (reported) | ✅ `cursor` map / ✅ `pointer-events-none` | |
| overflow (v0.6) | ✅ via `plan.resolve(&probe)` (no property) | ✅ `overflow_x`/`overflow_y` (v0.6 §2.1; auto → Scroll documented) | Masonry has no overflow property |
| grid template/placement (v0.6) | ✅ via `plan.resolve(&probe)` | ✅ `grid_template_*` + `grid_column`/`grid_row` (v0.6 §2.5) | floem's full grid surface was unwired before v0.6 |
| arbitrary grid templates + auto-rows/cols (v0.7) | ✅ via `plan.resolve(&probe)` | ✅ NATIVE — `Single`/`Repeat` track components + `grid_auto_rows/columns` (v0.9 §4) | templates are CANONICAL over the numeric counts; the `auto-rows-fr` keyword is `minmax(0, 1fr)` (corpus-pinned) |
| justify-items (v0.7) | ❌ reported (no Masonry property) | ✅ NATIVE taffy `justify_items` (v0.9 §5) | `normal` → stretch (CSS) |
| pseudo-element variants — `selection:`/`placeholder:`/`file:` (v0.7) | ✅ typed `Test::Pseudo` rules via `plan.resolve(&probe)` (probe default "no"; a wrapper with the surface answers) | ✅ typed rules via the reactive probe path (`FloemProbe`); no native `::selection` prop at the pin | v4's `selection:` ALSO emits a descendant rule (`& *::selection`) — outside the Probe model, a documented emitter normalization |
| has-data/has-aria descendants (v0.7) | ✅ `Atom::Has(Test)` rules via probe | ✅ typed rules via the reactive probe path (v0.9 §2) | combinator forms (`has-[>svg]`) stay reported |
| @container[/name] declaration (v0.7) | ✅ typed slot via `plan.resolve(&probe)` | ✅ typed slot; the reactive path answers `container_width` | the NAME is carried; name-matching ancestors is app-level (the probe answers for the app-supplied container) |
| align-content (v0.6) | ✅ via `plan.resolve(&probe)` | ✅ `align_content` (v0.6 §2.4; baseline reported) | |
| select-none/text (v0.6) | ❌ (reported) | ✅ the inherited `Selectable` prop (v0.6 §2.2) | Masonry: no selection property |
| whitespace (v0.6) | ❌ parley per-widget | ✅ via the single `TextOverflow` prop (normal → `Wrap`, nowrap = floem default no-op, `pre*` reported — v0.6 §2.3) | |
| border-style (v0.6) | ❌ (reported) | ✅ dashed/dotted/none/hidden → kurbo `Stroke` patterns (v0.6 §2.6; double reported) | dash lengths are a documented normalization |
| text (font size, weight…) | ❌ parley per-widget | ✅ font_size; ✅ line-height (v0.5: token leading lowers to the NATIVE `LineHeightValue::Normal` font-size multiplier); ⚠️ others (letter-spacing has no Floem prop — reported) | Q5a gap on Masonry side |
| hover/focus/focus-visible/active/disabled | ✅ Selector flags | ✅ selector maps | |
| focus-within | ❌ (reported) | ✅ `focus_within` | Masonry Selector lacks the flag |
| dark | ✅ via `plan.resolve(&probe)` (`TwEnv.dark`, v0.2) | ✅ `dark_mode` | Masonry lowering is env-fed; host-level swap also works |
| responsive sm..2xl | ✅ via `plan.resolve(&probe)` (`TwEnv` viewport width, v0.2) | ✅ `min_window_width` | |
| container (@*) | ✅ via `rules` + `plan.resolve(&probe)` (`TwEnv` container width) | ✅ via the reactive `tw_plan` path | both emulated through the Probe (v0.2) |
| group-*/peer-* | ✅ via `TwEnv` group/peer scope + probe | ✅ via the reactive `tw_plan` path (group-aware probe at the call site) | no native ancestor/sibling state anywhere |
| arbitrary selector variants (`[&:hover]`, `[a&]`, `[@media(print)]`) | ✅ typed subset via `rules` + probe (v0.3) | ✅ typed subset; `[&:hover]` ≡ `hover:` lowers NATIVELY into the selector maps (v0.3) | other forms report a per-form precise reason (`selector.rs`) |

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
