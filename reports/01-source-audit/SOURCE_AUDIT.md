# Source audit — repos, commits, style models (deliverable 1)

Every claim lives in the section reports with `file:line` citations; this
document is the index and the per-repo style-model summary. Commit hashes
were read from the local clones (`git rev-parse HEAD`), never from memory.

## Commit table (audited 2026-11-03)

| repo | commit (default branch, latest) | section report |
|---|---|---|
| linebender/xilem | `9f504ae71aa2db1f64c2ecb7ee10cc877d9fc405` | sections/xilem-masonry.md |
| lapce/floem | `1351ffb162faeb8be983f1301d718a0d5fb59c27` | sections/floem.md |
| iced-rs/iced | `8ff48c42b0d5188223501ea1ccc506fd5beaf36a` (master, workspace 0.15.0-dev, MSRV 1.93) | sections/iced.md |
| vizia | `9231260010d57380681d6bd2eca05b2b756463cb` | sections/other-frameworks.md |
| slint-ui/slint | `84972235182156b338756e0224aee65aa178d91c` | sections/other-frameworks.md |
| DioxusLabs/dioxus | `7dfd07eb7a7ef130f49de043327e762e24cd484e` | sections/other-frameworks.md |
| leptos-rs/leptos | `37fcf7d7…` (re-cloned; MANIFEST.txt updated) | sections/other-frameworks.md |
| emilk/egui | `fc9d66dda742d722eedaa8684d23f86369cf8e5f` | sections/other-frameworks.md |
| RibirX/Ribir | `f9d9f5b…` (re-cloned; stable Rust now) | sections/other-frameworks.md |
| longbridge/gpui-ce | `bc577476a0eb55173ef985585cfb487f75ab1cab` | sections/other-frameworks.md |
| marc2332/freya | `9017e49c5632ae6eae6f0c0233f05fd7a8ffea10` | sections/other-frameworks.md |
| andaja/icss | `b3d21d6fd7b6f10299dec242063db8ea9d3a2479` | sections/precedents.md |
| tw_merge (crates) | `4f5a516e76d75a15a587ab2169352cf34fd318d0` | sections/precedents.md |
| encre-css | `ae2870ac444bf6c9aebe9abb8b6494a05209a776` | sections/precedents.md |
| crepuscularity | `eb60f0f90b01f691b00b57053652ea6e7dd7d50e` | sections/precedents.md |
| twill | `e529d3e136afd943e72491f868b01150f739f793` | sections/precedents.md |
| ironframe | `b7c151a1680ab779523342716aa4747694ea2c56` | sections/precedents.md |
| tailwind-rs (tailwind-ast) | `511a19042b062fd10a8423b19ca33855499ea859` | sections/precedents.md |
| gpui-rsx | `51837497c669b9a2d00adb7bb0be583137025f35` | sections/precedents.md |
| floem-tailwind / floem-shadcn | `3b165092…` / `ae2f61ba…` (anti-examples) | sections/precedents.md |
| shadcn-ui/ui (official) | `4aaea19d9cff3e0908e12a8c1d504b8c5ec34610` (2026-11-02) | sections/precedents.md |
| tailwindcss v4.1.17 theme.css | fetched raw, verbatim palette | crates/aeolus-core/data/tailwind-default-palette.css |

Full hash list also in `repos/MANIFEST.txt` (working tree of the audit).

## One-paragraph style models

- **Masonry (xilem's engine)**: a 4-level cascade — widget-local
  `PropertySet` → per-widget `PropertyStack` of `(Selector, PropertySet)`
  layers (Selector = class set + hover/active/disabled/focus flags) →
  per-widget-type `DefaultProperties` → `Property::static_default`. 28
  property types; core `pre_paint` paints shadow/background/border for every
  widget from properties. Property changes invalidate by lane (layout vs
  pre-paint) with a relevance-gated per-widget cache. Xilem views can insert
  *local properties* (`Prop` view) but **cannot attach stack layers** — the
  audit's headline correction to the briefing's premise.
- **Floem**: a full CSS-like engine with no CSS text: `Style` maps with 10
  nested `StyleSelector` maps (hover/focus/focus-visible/focus-within/
  active/disabled/dark_mode/selected/drag/file_hover), structural
  (nth-child) selectors, responsive width selectors, class contexts flowing
  down the tree, per-prop `Transition`s, and a style cache with merge-id
  diffing — the closest native analogue to CSS of the set.
- **Iced**: per-widget-family `Catalog` traits resolving `(Class, Status) →
  Style` — state closures, not a cascade; `Program::theme()` re-queried per
  update makes hot-swap trivial; padding/font/width live outside Catalog as
  construction props.
- **Vizia**: an actual CSS engine (cssparser + selectors crate) with
  specificity-sorted rule application and runtime `add_stylesheet` — a
  strong future Aeolus target.
- **Slint / GPUI-CE / Freya / Ribir / egui**: DSL property bindings with
  `states when`; Tailwind-fluent `StyleRefinement` + state closures;
  Rust builders with hot-swappable `Theme` state; class-transformers;
  immediate-mode per-frame `Style` (egui's experimental `StyleProvider` +
  `Classes` plugin is a real programmatic rule engine).
- **Dioxus/Leptos (web)**: delegate to browser CSS; native renderers are
  external (blitz/stylo).
- **Precedents**: tw_merge = cn() oracle (272 groups, reverse scan, no trie);
  ironframe = golden-fixture oracle (parse() is a stub); tailwind-ast =
  MPL-2.0 (no vendoring); crepuscularity = ISC, typed-token + OKLCH ideas;
  encre-css = CssType hint idea; icss = CSS parsing engine; floem-tailwind/
  floem-shadcn = anti-examples (method-only, no class parsing, no v4).
