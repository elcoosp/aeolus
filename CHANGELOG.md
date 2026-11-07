# Changelog

All notable changes to Aeolus are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **The Slint backend** (`crates/aeolus-slint`): the fifth framework
  backend, chosen from the capability audit's candidate list (vizia's
  skia toolchain and GTK-family system dependencies being out of reach
  for headless sandboxes; Slint 1.18 is the most-used remaining
  pure-Rust framework). The lowering targets Slint's own value types
  and the generated-code path the audit prescribes for frameworks
  without runtime rule injection:
  - Native paint parity — the strongest of the five backends:
    gradients as real `Brush` values
    (`LinearGradientBrush`/`RadialGradientBrush::new_circle`/
    `ConicGradientBrush`; slint's angle convention matches CSS),
    per-corner radii (`border-top-left-radius` …), drop shadows WITH
    spread, and inset shadows via the `inner-shadow-*` surface.
  - `SlintStyle`/`SlintText` — typed element-property payload
    (`Brush` fills, `SlintLength` px/percent geometry, `clip`,
    `visible`, `opacity`, `mouse-cursor` via `BuiltInMouseCursor`).
  - Generated `.slint` code: `SlintStyle::snippet` (property
    bindings), `SlintText::snippet` (the `Text`-item block),
    `StyleBook::to_states_snippet` / `_focused` (`states [ … ]` gated
    on `TouchArea` `has-hover`/`pressed`/`enabled` and — with a named
    `FocusScope` — `has-focus`).
  - `StyleBook` + `SlintProbe`: the book keeps every state layer
    natively (hover/active/disabled/focus all resolve from probe
    flags), `resolve_at` keeps the full group/peer/responsive/
    container parity path, and the honest `LoweringReport` covers the
    rest (ring, layout structure, transforms, transitions, margins…).
  - 36 tests: lowering (19), parity (8), and snippet suites (9) whose
    generated `.slint` is compiled by slint's own interpreter
    headless — every emitted property name, keyword and gradient
    grammar is proven valid against Slint 1.18 itself. Clippy clean.
  - Licensing: the slint stack's `GPL-3.0-only` /
    `LicenseRef-Slint-Royalty-free-2.0` / `LicenseRef-Slint-Software-3.0`
    branches are allow-listed in `deny.toml` for the git-only,
    `publish = false` backend crate (slint's royalty-free grant covers
    use/redistribution; the workspace itself stays MIT OR Apache-2.0).
- **`aeolus-shadcn::slint`** — the Slint wrapper module mirroring the
  iced/egui shape: `button_book` and the style-only books
  (`badge_book`, `label_book`, the card family, `alert_book`) handing
  out `Arc<StyleBook>`s whose snippets assemble real interactive
  shadcn Buttons in `.slint`.

### Changed

- README, the how-it-works diagram and the capability table now list
  five backends; CI's `cargo check -p` gate includes `aeolus-slint`.

## [0.6.0] — 2026-11-07

The **upstream-ceiling** release (plan: `AEOLUS_V0.6.md`): a
reconnaissance of the latest GitHub floem and masonry (`git ls-remote`
2026-11-07) found that the floem pin `1351ffb` IS latest `main` HEAD,
and xilem upstream moved `9f504ae → dc49687` with an EMPTY
`masonry_core`/`xilem_masonry` diff — so every capability below is
consumed at the pinned revs with ZERO upstream changes. The headline:
the v0.1–v0.5 audits missed that Masonry has carried a layout-invariant
`Background::Gradient` all along; gradients lower natively now, and the
Floem backend wires the native surface it never used.

### Added

- **Gradient backgrounds in the IR** (v0.6 §1): `ResolvedStyle` gains
  `background_gradient: Option<Gradient>` — the composed
  `background-image` (`Gradient::Linear { angle_deg, stops }` /
  `Radial` / `Conic` / `None`), kept SEPARATE from the solid
  `background` exactly as CSS's `background-color` + `background-image`
  coexist. Parser: `bg-linear-to-{t,tr,r,br,b,bl,l,tl}` (CSS angles),
  `bg-linear-<angle>`, arbitrary `bg-linear-[<angle>deg]`, the
  deprecated `bg-gradient-to-*` alias, `bg-radial`, `bg-conic[-<angle>]`,
  `bg-none`; stops `from-`/`via-`/`to-` with theme/shadcn tokens,
  arbitrary values and `/opacity` modifiers (`color-mix(in oklab, …)`,
  v4's shape), plus `%` position patches (`from-20%` patches ONLY the
  position). Composition semantics pinned against the REAL tailwindcss
  4.1.17 output (`tools/capture_gradients.mjs`): stop-position defaults
  are the `@property` initial values (0%/50%/100%), missing from/to
  stops are `#0000` (transparent), interpolation is `in oklab`, stops
  without a shape compose nothing. `Slot::Gradient(GradientPart)` diffs
  through the cosmetic lane; `cn()` gains the `bg-image` split and
  per-edge from/via/to color+position groups.
- **Masonry gradient lowering, native** (v0.6 §1.5): `Background::Gradient`
  (layout-invariant `GradientShape`, `ColorSpaceTag::Oklab` = v4's
  default `in oklab`, `Extend::Pad`, sRGB stops at fraction offsets).
  The linear angle conversion `a = π − θ` is DERIVED from masonry's
  `get_peniko_linear_for_rect` paint math (its angle 0 points to the
  BOTTOM) and unit-pinned. Radial lowers as circle-farthest-corner with
  the documented ellipse→circle note (upstream `#[non_exhaustive]`
  TODO); conic reports the upstream vello sweep-offset TODO.
- **The Floem unwired surface** (v0.6 §2, all native at the pinned rev):
  `overflow-*`/`overflow-x-*`/`overflow-y-*` → taffy
  `overflow_x`/`overflow_y` (`overflow-auto` → `Scroll`, documented);
  `select-none`/`select-text` → the inherited `Selectable` prop;
  `whitespace-normal` → `TextOverflow::Wrap` and `whitespace-nowrap` →
  floem's default (a documented no-op — floem never wraps by default);
  `content-*` → `align_content` (`normal` → `Stretch` per CSS, baseline
  reported — taffy has none); `grid-cols-N`/`grid-rows-N` →
  `grid_template_*` repeat(N, 1fr), `col/row-span` + `col/row-start` →
  `grid_column`/`grid_row` placement (`N / span M` composition);
  `border-dashed`/`border-dotted`/`border-none`/`border-hidden` → kurbo
  `Stroke` dash patterns/zeroed strokes on the four side borders
  (`border-double` reports — no double-line primitive). The parity
  ledger (`slot_coverage`) upgrades Overflow, Select, Whitespace,
  AlignContent, Grid and BorderStyle Reported → Native.
- **The verification emitter reproduces v4's per-class gradient shapes**
  (v0.6 §1.4): the captured `--tw-gradient-*` var machinery
  (`--tw-gradient-position`, `--tw-gradient-{from,via,to}`,
  `--tw-gradient-{from,via,to}-position`, the two `--tw-gradient-stops`
  chains) — unit-pinned against `tools/capture_gradients.mjs` output.
- `AEOLUS_V0.6.md` — the v0.6 plan of record;
  `reports/10-tdd-evidence/08-upstream-ceiling.md` — the RED→GREEN
  evidence.
- Overflow per-axis merge fix: `overflow-hidden overflow-x-auto` now
  keeps `y = hidden` (CSS: `overflow-*` sets both axes, `overflow-x-*`
  patches one) — previously the last class replaced the whole slot.

### Changed

- The golden corpus is UNCHANGED (176 entries): gradient parity needs
  cross-rule `--tw-gradient-*` var composition (three separate rules per
  gradient) — a harness model boundary, documented as such (v0.6 §4.3,
  the v0.5 §2.3 precedent), not a coverage gap.
- Capability matrix corrections (v0.6 §3): Masonry's gradient row
  upgrades from "gradients dropped" to native; the uniform-border-width,
  single-radius and no-spread-shadow claims re-verified against the
  pinned sources and re-documented.

### Upstream re-verification (§3)

- floem: pin `1351ffb162faeb8be983f1301d718a0d5fb59c27` == latest
  `main` HEAD (2026-11-07).
- masonry/xilem: upstream `9f504ae → dc49687` = 3 commits (text-widget
  tests, example interaction tests, vello dep removal);
  `git diff 9f504ae..dc49687 -- masonry_core xilem_masonry` is empty —
  the pin stays.
- crates.io: `masonry`/`masonry_core`/`masonry_testing` 0.4.0
  (2025-10-29) and `floem` 0.2.0 (2024-11-14) remain the newest
  releases — publishing the backends stays blocked, as do the other
  three upstream deferrals (v0.6 §5).

## [0.5.0] — 2026-11-07

The **element-relative typography + allocation contract** release (plan:
`AEOLUS_V0.5.md`): the last two follow-ups the reports name as open and
in-repo-doable — the benchmark report's allocation counters ("or the
project writes its own global allocator counting harness", deliberately
out of scope for v0.4) and the element-relative typography gap (v0.4
§2.3 kept `tracking-*`/bare `leading-*` out of the corpus). The four
upstream-blocked deferrals stay deferred (§5 of the plan, re-verified
against crates.io 2026-11-07: no published masonry/floem release is
compatible with the pinned revs).

### Added

- **Element-relative typography in the IR** (v0.5 §1): `TextValue` gains
  `TrackingEm(f32)` (the `--tracking-*` token em) and
  `LineHeightRatio(f32)` (the `--leading-*` unitless ratio) — the token
  forms of `tracking-*` / `leading-*` / `text-*/*`-with-named-leading
  now carry the RELATIVE unit the class names, exactly what v4's theme
  tokens hold, instead of a 16px pre-multiplication the element's real
  font-size cannot use. Absolute forms are unchanged (arbitrary
  `tracking-[7px]` / `leading-[3px]`, spacing-multiple `leading-6` and
  `text-sm/6`, token-paired `text-sm` line-heights). Multiplying by an
  element's font-size is the renderer's cascade duty (v0.5 §1.6).
- **Corpus: element-relative typography entries** (v0.5 §2): the golden
  corpus grows 157 → **176 entries** — the six `tracking-*` tokens, the
  negative form (`-tracking-wide`, v4's `calc(var(--tracking-wide) * -1)`),
  arbitrary absolutes (`tracking-[7px]`, `leading-[3px]`), the six named
  `leading-*` tokens + `leading-none`, named text-size leading modifiers
  (`text-base/tight`, `text-lg/loose`), and two shadcn-theme entries.
  All 176 at zero-warning parity with the real tailwindcss 4.1.17 output.
- **The allocation contract** (`tests/alloc_contract.rs`, v0.5 §3): a
  counting global allocator + mutex-serialized measured sections pin
  report 08's structural claims as machine-independent numbers —
  exact-zero claims pinned exactly, clone-shaped claims as measured
  upper bounds. divan 0.1.21 re-verified as unable to carry
  per-benchmark alloc counters (`IntoCounter` is a fixed per-iteration
  value, not a sampled delta) — the contract test IS the harness the
  report asked for.
- `AEOLUS_V0.5.md` — the v0.5 plan of record.

### Fixed

- **`diff()` allocated on EVERY call** (caught by the allocation
  contract on day one — the report-08 claim "no alloc" was wrong):
  `bits_of`/`diff` routed through `slots()`, whose scratch
  `Vec::with_capacity(40)` allocated per call, and `slots()` itself was
  re-walked in the shared-bits compare. Slot presence now walks ONE
  `for_each_present_slot!` source; `diff` walks shared bits directly.
  The rebuild path is allocation-free when nothing changed — pinned
  EXACTLY — and `diff_unchanged` dropped 238.8 → 139.8 ns (report 08).
- **`-tracking-wide` ignored the negative marker**: the token arm of
  `tracking` never applied `t.negative` (the arbitrary-px arm did). The
  token em now negates, matching v4's `calc(var(--tracking-wide) * -1)`
  shape; pinned by its corpus entry.
- The parity harness's calc evaluator now folds em-valued calcs
  unit-preservingly (`calc(0.025em * -1)` → `-0.025em`) and no longer
  eats a unit-leading `e` as an exponent marker (`0.025em` parsed as
  `0.025e` before).

### Changed

- **Floem**: the `--leading-*` token form lowers to Floem's NATIVE
  `LineHeightValue::Normal(multiplier)` — font-size-relative at paint,
  exactly CSS's unitless line-height (the capability-matrix row upgrades
  from absolute-pt to element-relative). Absolute forms stay `Pt`.
  `tracking-*` keeps the honest drop report (no Floem letter-spacing
  prop at the pinned rev); the relative em stays in the plan.
- The v0.4 §2.3 corpus scope note is REWRITTEN, not deleted
  (AEOLUS_V0.5.md §2.3): element-relative values ARE pinned — as the
  relative units the plan carries; element-level composition remains the
  renderer's cascade duty, documented as a model boundary.

### Deferred to 0.6 (all upstream-blocked, re-verified 2026-11-07)

Unchanged from 0.4 §6 / 0.3 §4: publishing the backends (crates.io's
`masonry`/`masonry_core` 0.4.0, published 2025-10-29, predates the
pinned `linebender/xilem @ 9f504ae` by a year — the `LenReq`/`SizeDef`
layout model and `layers`/`imaging` surfaces the backends consume exist
in no release; `floem` 0.2.0, published 2024-11-14, predates the pinned
`1351ffb` by two years), view-side property-stack registration, Masonry
typography via native properties, Masonry transitions/animations.

## [0.4.0] — 2026-11-07

The **parity proof** release (plan: `AEOLUS_V0.4.md`): the pipeline
(parse → theme → plan) is now verified end-to-end against the REAL
`tailwindcss` v4.1.17 compiler — the golden-corpus follow-up the design
report (§7.4) and the source-audit Q&A (Q7) named — plus the benchmarks
report's allocation-counter follow-up (re-tested, still unavailable in
divan 0.1.x; documented). No new utility families, no backend changes, no
MSRV bump: v0.4 makes what already ships *verifiable*, then measures it.

### Added

- **The verification CSS emitter** (`crates/aeolus-core/src/css.rs`,
  v0.4 §1): `Resolver::to_css` derives Tailwind-v4-equivalent CSS from
  the SAME resolution backends consume — one `CssRule` per class token,
  v4's one-rule-per-candidate shape. Selectors mirror v4's templates
  (escaped class names, state/data/aria/structural pseudo-classes,
  `:not(...)`, `:has(...)` — star forms, group/peer descendant templates
  `&:is(:where(.group):hover *)`, `@media`/`@container` wrappers with
  v4's canonical `(width >= 48rem)` conditions); values mirror v4's
  shapes (`calc(var(--spacing) * n)`, oklch colors,
  `color-mix(in oklab, …)` for opacity modifiers, composed `box-shadow`
  layers for rings and multi-layer shadow presets, `calc(infinity * 1px)`
  for the full-radius sentinel). A verification/debugging tool — not a
  styling path; backends keep consuming `StylePlan` directly, and
  unsupported classes keep their warnings.
- **The golden corpus** (`tools/gen_golden.mjs` +
  `crates/aeolus-core/tests/fixtures/golden.corpus`, v0.4 §2): 157
  single-class entries across two theme contexts (the verbatim v4 default
  palette and the Shadcn neutral light theme with its `@theme inline`
  var chains), generated by the REAL tailwindcss 4.1.17 CLI. Covers
  spacing/margin, sizing (fractions/arbitrary/viewport/fit), colors
  (palette, shadcn tokens, opacity modifiers, bracket alpha, hex),
  borders/radii (logical edges), ring/outline px semantics, multi-layer
  shadows + shadow colors, typography, flex/grid layout, position/inset/
  overflow/aspect, transforms, transitions, cursor/pointer-events/select,
  all seven classic states, breakpoints (min/max), `dark:`,
  group/peer (incl. labels), `data-*`/`aria-*`, structural, `has-*`,
  `not-*`, container queries, v0.3 arbitrary variants, compounds
  (`md:hover:*`, `dark:hover:*`), `!important` (both v4 spellings) and
  negatives.
- **The parity harness** (`tests/parity_golden.rs`, v0.4 §3): both sides
  normalize through ONE pipeline — `var()` substitution from the golden
  `:root`/`@property` tables, `calc()` evaluation (rem→px, mixed units),
  `color-mix` evaluation, color canonicalization (numeric Oklab compare),
  `--tw-*` machinery composition into final `box-shadow` values, unitless
  `line-height` × same-rule `font-size` — plus the documented model
  normalizations (hover-capability wrappers dropped, `@supports`
  modern-browser override merged, initial `border/outline-style` dropped).
  Fails on ANY resolution warning in the corpus (the coverage-guard
  contract extended to the corpus).
- Benchmarks: `css_write_button` (the 33-class Button through the
  emitter) — numbers in report 08.
- Emitter unit tests for the corners the corpus cannot reach (`Test::Tag`
  selector prefixing, escaping corners, viewport units from token text,
  warning honesty, inert ring variants).
- `AEOLUS_V0.4.md` — the v0.4 plan of record.

### Fixed (caught by the corpus — the parity proof paying off immediately)

- **`-top-2`/`-translate-y-1` lost the sign**: the numeric arms of
  `inset_value`/`transform_size` ignored `t.negative` (margins handled
  it; insets and translations did not). Negative insets/translations now
  carry their sign.
- **`rounded-t-full` (and every `rounded-{edge}-full`) resolved to 6px**:
  the `--radius-full` token is absent from the v4 theme (it is
  `calc(infinity * 1px)`), so the lookup fell through to the 6px default.
  Full-radius tokens now resolve to the 9999px sentinel (emitted as
  `calc(infinity * 1px)`), and `rounded-{edge}-none` resets to 0.

### Deferred to 0.5 (all upstream-blocked, tracked in `AEOLUS_V0.4.md` §6)

Unchanged from 0.4 §6 / 0.3 §4: publishing the backends, view-side
property-stack registration, Masonry typography via native properties,
Masonry transitions/animations.

## [0.3.0] — 2026-11-06

The **arbitrary selector variants** release (plan: `AEOLUS_V0.3.md`) — the
last core-language gap from the 0.2 deferred list, plus the post-0.2.0
audit round (a full-repo sweep for gaps, panic paths and semantic
divergences from Tailwind v4.1.17, verified against the `tailwindcss`
source, not the docs alone). Every v4 arbitrary
variant form now either resolves to typed probe conditions or fails with a
per-form precise `Unsupported` reason; the blanket "arbitrary/structural
selectors have no native equivalent" report is gone. Semantics verified
against the `tailwindcss` v4.1.17 source (`candidate.ts`, `compile.ts`,
`variants.ts`), not the docs alone.

### Fixed (post-0.2.0 audit round)

- **Theme parser panics on unbalanced custom-rule blocks**: CSS ending in
  `@utility foo {` or `@custom-variant x {` with no closing brace sliced
  past the end (`rest[close + 1..]`) — fuzz-reachable via
  `theme_from_css`. The malformed tail is now ignored, matching
  `extract_declarations`' contract.
- **Theme resolution was declaration-ORDER dependent** (CSS custom
  properties resolve at use time): an `@theme inline { --color-primary:
  var(--primary); }` block preceding its `:root` source failed with
  `unsupported color function "var"`, and a `calc()` referencing a var
  whose own value is a later `calc()` failed with `calc() references
  unknown token`. Bare-var color values now defer and the deferred
  calc()/var() passes run to a fixpoint; truly-unresolvable references
  keep the precise-error contract.
- **Ring/outline/underline-offset widths were spacing multiples, not px**
  (v4.1.17 semantics, verified in `utilities.ts`): `ring-2` resolved 8px
  (correct: 2px), bare `ring` 4px (correct: 1px — `--default-ring-width`
  fallback), `ring-offset-2` 8px (correct: 2px), bare `outline` 2px
  (correct: 1px — `--default-outline-width` fallback), `outline-offset-2`
  8px (correct: 2px), `underline-offset-4` 16px (correct: 4px). Negative
  offsets (`-outline-offset-2`, `-underline-offset-2`) now carry their
  sign. Only the spacing families keep `--spacing` multiples.
- **State-layer cascade ignored the canonical variant order**:
  `StylePlan::resolve` ranked legacy state layers by source order, so
  `focus:bg-blue-500 hover:bg-red-500` resolved the HOVER layer on a
  focused+hovered element (Tailwind sorts generated CSS by variant order →
  focus wins). Layers now rank by `VariantCondition::rank` — the same
  order the stack lowering and `resolved_for_state` already used.
- **Floem silently dropped viewport/fit-content sizes**: `apply_dim`
  no-op'd `w-screen`/`h-dvh`/`w-fit` shapes without a report (the
  no-silent-failures contract). They are now `LoweringReport` entries, and
  `slot_coverage` classifies text props exactly as `lower_text` maps them.

### Added

- **Arbitrary selector variants classify into typed atoms**
  (`selector.rs`, v0.3 §1.2): the typed subset is exactly what constrains
  THE ELEMENT — the only thing the `Probe` model answers about:
  - self pseudo-classes (`[&:hover]`, `[:hover]`, `[&:hover:focus]`,
    `[p:hover]`) → `Atom::On(Test::State(_))`; `[&:hover]` generates
    byte-identical CSS to `hover:` in v4, so it canonicalizes onto the
    same legacy layer bucket (native selector lowering on Floem for
    free);
  - negation (`[&:not(:hover)]`, `not-[&:hover]`) → `Atom::Not(_)`;
  - self attributes (`[&[data-state=open]]`, `[&[aria-busy=true]]`) →
    `Test::Data` / `Test::Aria` (identical plans to the named forms);
  - structural self tests (`[&:nth-child(3)]`, `[&:first-child]`,
    `[&:empty]`) → `Test::Structural`;
  - descendant-state selectors (`[&:has(:focus)]`, `has-[:focus]`) →
    `Atom::Has(_)`;
  - **element-kind tests** (`[a&]`, `[a]`, `[p&:hover]`) → the new
    `Test::Tag` — the rule lands typed in `plan.rules`; native backends
    answer "no" (the rule never fires — the same semantics CSS has when
    a selector does not match) and a backend that models element kinds
    answers honestly;
  - arbitrary at-rules (`[@media(print)]`,
    `[@media(prefers-color-scheme:dark)]`, `[@media(min-width:900px)]`,
    the v4 canonical `[@media(width>=900px)]`, orientation/reduced-
    motion/contrast/forced-colors/pointer features, `and`-conjunctions)
    → `Atom::Media` / `Atom::Viewport`;
    `[@container(min-width:400px)]` → `Atom::Container`;
  - `group-[&:hover]` / `peer-[&:hover]` inners classify like self
    selectors; `not-*` inners must be a single self test (nested
    negation and compounds are reported — a `Cond` is a conjunction).
- **Per-form precise reports** (v0.3 §1.3) for everything the Probe
  model cannot ask: selectors styling OTHER elements (`[&_svg]`,
  `[&>*]`, `[&+&]`, `*`, `**`), pseudo-elements (`[&::before]`),
  class/id self tests (`[.x]`, `[.x&]`), `@supports`, disjunctions
  (`,` lists, `:is(a,b)`, `or` media), An+B structural forms
  (`nth-[2n+1]`, `:nth-child(2n+1)`), bare non-data/aria attributes,
  unknown media features and pseudo-classes, relative selectors
  (`[>img]` standalone), the deprecated at-rule-with-selector form.
- Tailwind's underscore decoding inside arbitrary selectors
  (`decodeArbitraryValue`: `_` → space, `\_` escapes) —
  `[@media(min-width:900px)_and_(orientation:landscape)]` works.
- Regression suites: `tests/arbitrary_variants.rs` (30 tests: the typed
  subset, evaluator equivalence with the named forms, cn() raw-variant
  keys, per-form precise reports), truth-table `Test::Tag` row, and
  backend parity tests pinning `[&:hover]` native lowering (Floem
  selector maps) + compound-rule evaluation through both probes.
- **Legacy comma color syntax**: `Color::parse` documented
  `rgb(r, g, b, a)` but only the space form parsed — `rgb(251, 44, 54)`
  failed with `bad number "251,"`. Common in arbitrary values
  (`bg-[rgba(0,0,0,0.5)]`); comma forms (with optional alpha component,
  numbers or percentages) parse now.
- **Missing `cn()` conflict groups** (tw_merge parity): logical inset
  edges `start-*`/`end-*`, the six grid-line families
  (`col/row-span/start/end`), and the shadow/outline splits —
  `shadow-md` + `shadow-destructive` and `outline-2` + `outline-primary`
  no longer knock each other out (preset-width vs color are independent
  groups, plus `outline-style`).
- `AEOLUS_V0.2.md` — the v0.2 plan of record (referenced by this README
  since 0.2.0 but never committed).
- Regression suites: `tests/px_semantics.rs` (px-vs-spacing semantics +
  canonical state cascade), theme order-independence and unbalanced-block
  tests, legacy comma color tests, and per-family `cn()` merge tests.

### Changed

- **`cond::Test::Tag(Box<str>)`** — additive variant on the public
  condition enum (0.x minor bump): exhaustive `Test` matches need a new
  arm (the in-repo probes answer "no", the honest default).
- The resolver routes arbitrary variants through
  `Resolver::atoms_of` (a conjunction list) instead of one `atom_of`
  atom; `variant_test` accepts single-self-test arbitrary inners.
- `has-[…]` accepts the `[:state]` selector form (`has-[:focus]` ≡
  `has-focus:`); other selector forms keep the precise report.
- README's deferred list shrinks to the upstream-blocked items
  (arbitrary selector variants shipped here); the capability matrix
  gains the arbitrary-variant row and the stale ring/outline/opacity/
  cursor rows were refreshed against the actual backends.
- The `space-*` warning no longer claims the resolver "emulates" it as
  gap — it reports precisely that `gap-*` is the supported equivalent.
- README's deferred list moved to v0.3 (group/peer layers, structured
  variants, container queries, responsive/dark on Masonry and Floem
  transitions shipped in 0.2.0).

### Deferred to 0.4 (all upstream-blocked, tracked in `AEOLUS_V0.3.md` §4)

- Publishing the backends — needs crates.io releases of the pinned
  Masonry/Xilem and Floem revs.
- View-side property-stack registration (retires the fallback widget) —
  upstream gap #4.
- Masonry typography via native properties — upstream patch proposal #3.
- Transitions/animations on the Masonry backend — needs Masonry-side
  animation machinery (Floem has per-set transitions).

## [0.2.0] — 2026-11-05

The "serious Tailwind v4 compiler" release (plan: `AEOLUS_V0.2.md`): the
core owns **all** semantics — including compound variants, group/peer,
data/aria, structural selectors and container queries — and backends only
map them, through the new `Probe` parity evaluator.

### Fixed (correctness — the F1–F7 silent-mis-style bugs)

- **F1** fractions: `w-1/2` → `Fraction(0.5)` (was 4px). The raw `/modifier`
  is the fraction denominator for sizing utilities.
- **F2** arbitrary lengths: `p-[7px]` → 7px (was 28px). Arbitrary values are
  absolute, never `--spacing` multiples; `(--var)`/`[var(--x)]` resolve
  through the theme.
- **F3** `dark:hover:*` lands in `rules` as `[Media(Dark), On(Hover)]` (was:
  the `dark` half silently dropped, hover applied in LIGHT mode). Shipped
  Shadcn Ghost/Outline hovers are correct now; regression-tested per
  variant.
- **F4** `md:hover:*` — same fix for breakpoint compounds.
- **F5** `rounded-s-lg`/`rounded-e-*`/`rounded-ss-*` map to the logical
  start/end edges (was: all four corners).
- **F6** `text-sm/6` resolves the line-height modifier (6 × spacing);
  `text-sm/tight` uses `--leading-tight`.
- **F7** `bg-primary/[0.37]` is a raw alpha fraction (was: rejected); 
  `bg-primary/(--o)` resolves through the theme.
- `cn()` merge keys use the EXACT canonical variant prefix — `hover:` and
  `group-hover:` no longer knock each other out, `md:` no longer collides
  with `@md:`.
- `PlanCache` is keyed by the class STRING (per-thread L1 + shared L2) — a
  hash collision can no longer return the wrong plan; entries are
  version-stamped and previous-generation hits are promoted.
- Multi-layer v4 shadows parse fully (`--shadow-sm` = 2 layers; v0.1
  truncated at the first comma).
- `@md` is the `--container-md` size (448px), not the `sm` viewport
  breakpoint; `@xs`/`@3xl` and `@min-[400px]` resolve from the container
  scale.

### Added

- **The parity engine** (`eval.rs`): `Probe` + `StylePlan::resolve` /
  `resolve_rules_only` — a backend answers questions about the element's
  situation and the SAME evaluator flattens the plan for both backends.
  `Atom`/`Test`/`Cond`/`Rule` model the full v4 variant grammar
  (`not-*` recursive, `has-*`, `data-*`, `aria-*`, structural, media,
  viewport windows, named container queries).
- **No-alloc token shape**: `ClassToken::variants` is a stack-inlined
  `Inline<Variant, 8>`, `elements` a `Segments<'a, 10>` (deref to slices);
  the raw `body`/`modifier` slices replace `join("-")` on the hot path.
- **Full v4 default theme** (`data/tailwind-default-theme.css`, diffed
  against tailwindcss v4.1.17): `--container-*`, multi-layer `--shadow-*`,
  `--text-*` + paired line heights, `--font-weight-*`, `--tracking-*`,
  `--leading-*`, `--radius-xs..4xl`, `--blur-*`, `--perspective-*`,
  `--ease-*`, `--animate-*`, default transition duration.
- **`@utility` / `@custom-variant`**: class-string expansion (depth ≤ 4)
  and name→`Media::Dark` mapping from CSS.
- **Utility coverage** (plan §3.2): `mx-auto`, `max-w-{container}`, viewport
  sizes (`w-screen`/`h-dvh` as `SizedValue::Viewport`), `gap-x/y`,
  `space-*` (gap emulation), `basis-*`, `order-*`, `content-*`,
  `grid-cols/rows`, spans/starts, `aspect-*`, `absolute/relative/...`,
  `inset-*` (incl. logical start/end), `z-*`, `overflow-*`, `visible/
  invisible`, `italic/uppercase/underline/truncate/line-clamp/font-sans/
  tabular-nums/text-ellipsis`, `transition-*/duration-*/ease-*/delay-*`,
  `animate-*`, `scale/rotate/translate/skew`, `shadow-{color}`,
  `inset-shadow-*`, `ring-inset`, `outline-offset-*`, `border-{style}`,
  logical `border-s/e`, the full `cursor-*` keyword set and `select-text/all`.
- **A coverage guard test**: every v4 utility family must resolve to a
  typed slot or a precise `Unsupported` — `Unknown` fails CI (plan §5.3).
- **Floem parity** (plan §4.2): align/self/wrap/display, gap axes, auto
  margins, `absolute()`, insets, z-index, aspect-ratio, cursor,
  pointer-events, typography (weight/style/align/line-height/overflow),
  transform scale/rotate/translate, ring→outline+outline_color, per-set
  `Transition`s, and the `tw_plan` reactive path
  (`FloemProbe` + `plan.resolve_rules_only`) that makes compound variants,
  group/peer and container queries work without an upstream patch.
  Theme-aware breakpoints replace `Breakpoint::default_px`.
- **Masonry/Xilem widening** (plan §4.3): `TwEnv` (dark, viewport/container
  width, group/peer scope via `WidgetMut::set_env`) feeds
  `plan.resolve(&probe)` in measure/layout/pre-paint; margins measured and
  inset; per-side borders and per-corner radii painted with Kurbo geometry;
  ring/outline stroked; shadow presets render every layer; opacity is
  multiplied into the chrome colors (children-not-faded documented).
- **Hot path** (plan §3.4): 8-bytes-per-round `FastHasher` with a
  length-tagged tail + final fold; string-keyed two-tier cache with
  promotion; `(ptr,len)`-keyed `resolve_static`; O(1) bitmask diff.
  Measured (1-core sandbox): cached hit 424ns → **76ns**, unchanged diff
  763ns → **235ns**, cold resolve 7.54µs → **5.49µs** (with the v0.2 rule
  machinery included). Parser is ~0.9µs slower per Button string — the
  richer v4 grammar costs, documented honestly.
- **MSRV split** (F9): `aeolus-core`/`aeolus-macro` stay at **1.88**
  (verified with a real 1.88 toolchain); `aeolus-shadcn` and the backends
  declare **1.96** (Masonry 0.4's requirement). CI grows two MSRV jobs.

### Changed

- `tw!` keeps its compile-time typo check: unknown variant *names* are
  rejected statically (the runtime parser accepts them as `Custom` for
  `@custom-variant` resolution).
- `Warning::Unsupported` texts name the parity lane ("emulated by the
  reactive probe path" / "reported, never silent (plan §7)").
- `TailwindTheme::default()` seeds the full v4 defaults (v0.1 seeded
  nothing; `from_theme_css` seeded colors only).

## [0.1.0] — 2026-11-04

The first release-candidate version: the Tailwind v4 → native-Rust style
compiler goes from research prototype to publishable. **Publishable today:
`aeolus-core` and `aeolus-macro`.** The three backend crates stay
`publish = false` (git-pinned framework dependencies; see
`reports/09-publishing/PUBLISHING.md`).

### Added

- **Phase 1 — publishability**
  - Backend manifests reference the audited frameworks by pinned git rev
    (`xilem @ 9f504ae`, `floem @ 1351ffb`) instead of sibling `repos/`
    checkouts; the workspace builds and `cargo test -p aeolus-core -p
    aeolus-macro` is green with `repos/` deleted.
  - `LICENSE-MIT` / `LICENSE-APACHE` at the repo root (referenced by the
    `MIT OR Apache-2.0` license field, previously absent).
  - `#![deny(missing_docs)]` on `aeolus-core` — every public item documented.
  - `crates/muda-stub` (dev-sandbox-only, not a member, not publishable)
    documented as the sanctioned muda/gtk workaround for Linux floem builds.
- **Phase 2 — correctness (TDD, each with a RED-then-GREEN test)**
  - `calc()` evaluation now honors **operator precedence** (shunting-yard):
    `calc(4px + var(--r) * 2)` evaluates as `4 + r*2`, and parenthesized /
    nested-`var()` expressions parse (previously left-to-right AND a
    parse error on parens).
  - `Color` equality is **approximate (~1e-4 per Oklab component)** —
    exact f32 equality caused spurious diffs and false cache misses.
  - Per-side border widths are **reported** (`LoweringReport` entry
    `border-width:per-side`) when flattened for Masonry's uniform
    `BorderWidth`, and per-side border classes now **compose**
    (`border-t border-b` keeps both edges).
  - `cn()` classifies the missing conflict families from the audited
    tw_merge table: `justify-items/self`, `align-content`, `place-*`,
    `flex-wrap`, `basis`, `order`, `position`, `inset`, `z-index`,
    `overflow-*`, `overscroll-*`, `text-align`, `object-*`, `aspect`,
    `columns`, `visibility`, `isolation`, `resize`, `font-style`,
    `text-transform`, `text-decoration`, `vertical-align` — e.g.
    `cn(["justify-items-start","justify-items-center"])` now merges.
  - Cache locks are **poison-tolerant** (`expect("lock")` removed; the
    guarded maps carry no cross-lock invariants — documented).
  - `Interner` and `PlanCache` are **bounded** (two-generation rotation,
    default 4096/generation, ≤ 2×cap memory; zero dependencies; LRU
    feature deliberately skipped).
- **Phase 3 — Track A: real interactive widgets**
  - `StyledButton` (`aeolus-xilem::button`): a real widget — pointer
    capture + active/hovered release semantics, Space/Enter keyboard
    activation, accesskit `Click`, disabled state, per-state plan
    painting — plus the `styled_button` view with callback routing.
  - `aeolus_shadcn::xilem::button(resolver, id, text, variant, size,
    class, on_press)`: the Shadcn Button is now a REAL button (the old
    API produced a styled label that ignored clicks — signature changed
    intentionally to take the callback).
  - `aeolus_shadcn::xilem::input(resolver, value, on_changed, class)`:
    Masonry `TextInput` styled from the Shadcn Input class table with the
    controlled-value contract. badge/label/card/alert remain explicitly
    style-only.
- **Phase 4 — hardening**
  - Four libFuzzer targets (`fuzz/`): `parse_class_string`, `color_parse`,
    `cn`, `theme_from_css` — each ran clean 60s in release+ASAN
    (1.19M / 7.26M / 540K / 6.3K execs).
  - Property tests (`proptest`, dev-only): `cn` idempotence, parser
    roundtrip, OKLab↔sRGB roundtrip (tight in-gamut, finite everywhere),
    opacity hue preservation.
  - CI (`.github/workflows/ci.yml`): fmt, clippy `-D warnings`, test
    matrix (linux/macos/windows), docs `-D warnings`, package gate,
    `cargo deny` licenses, 60s fuzz smoke (main + weekly).
  - `deny.toml` with the license allow list enumerated from the
    dependency graph.
  - Runnable crate-level doc example in `aeolus-core`.
- **Phase 5 — release prep**
  - `CHANGELOG.md`, `SECURITY.md`, `CONTRIBUTING.md`, rewritten
    `README.md`, `reports/09-release/RELEASE_CHECKLIST.md`, `v0.1.0` tag.

### Fixed

- `Color::to_srgba` clamps linear components (documented but unimplemented):
  out-of-gamut Oklab previously produced **NaN** through `powf` on negative
  linear components (found by the property suite).
- Fuzz findings (each fixed TDD-first):
  - `parse_hex` byte-sliced multibyte strings of the right byte length
    (panic mid-char on `#abᐁ`); ASCII-gated now.
  - `cn()` leaked caller-internal whitespace (`cn("\n")` echoed `"\n"`)
    and broke idempotence; output is now always a normalized word sequence.
  - `extract_declarations` sliced past the end for CSS ending in an
    unbalanced block; the malformed tail is ignored.
  - Absurd alpha components (`1e95`) parsed to f32 inf; alpha clamps to
    [0,1] per CSS.
  - Huge hue (`oklch(3 3 1.9e83)`) turned NaN in `cos` argument reduction;
    hue clamps to ±360°.
  - Chroma overflow (`3.3e40`) overflowed the Oklab→linear cubes to inf;
    Oklab components clamp to their physically meaningful ranges at
    conversion.
- Duplicate `grow`/`shrink` conflict arms (unreachable-pattern warnings).

### Deferred to 0.2

- **Publishing the backends**: blocked on upstream crates.io releases of
  the pinned Masonry/Xilem and Floem commits, and on floem making its
  Linux muda/gtk dependency optional (`reports/09-publishing/`).
- Group/peer conditional layers (`group-hover:`) — needs the Masonry
  class-scoped group-state patch (proposal in `reports/07-upstream-patches/`).
- View-side property-stack registration (upstream gap #4) — retires the
  `StyledWidget` fallback wrapper.
- Typography properties in Masonry (`FontSize`/`FontWeight` upstream
  patch proposal #3) and responsive/dark layers on the Xilem backend.
- Container queries (`@*` variants) on any backend.
- Opacity / ring / outline / cursor / pointer-events slots on the Masonry
  backend (reported as dropped today, never silently ignored).
- Transition/animation support (recognized-and-dropped with warnings).
- Structured variants (`has-[…]`, `data-[…]`, `aria-*`, `[&_svg]`) —
  parser extension beyond the v1 scope decision.

[0.1.0]: https://github.com/aeolus-rs/aeolus/releases/tag/v0.1.0
