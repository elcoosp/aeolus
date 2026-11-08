# aeolus-core design (deliverable 3)

The code IS the reference (`crates/aeolus-core/`, zero dependencies,
edition 2024, MSRV 1.88, rust-version pinned in Cargo.toml). This document
explains the shape and the why, with pointers into the source.

v0.3 note: arbitrary selector variants classify in `selector.rs` (§8
below, plan of record `AEOLUS_V0.3.md`).

v0.5 note: element-relative typography (§10) and the allocation
contract (§11) — plan of record `AEOLUS_V0.5.md`; the golden corpus is
176 entries as of v0.5.

## 1. Grammar (parser.rs, ast.rs)

Class token grammar (v4 subset, the Shadcn v4 component surface):

```text
class       := [-] variant-list utility [opacity] [arbitrary]
variant     := named | breakpoint | container | grouped | arbitrary-variant
named       := "hover" | "focus" | "focus-visible" | "focus-within"
             | "active" | "disabled" | "dark" | "visited"
grouped     := ("group"|"peer") ["-" label] "-" named
utility     := prefix ["-" name-list]        // bg, p, rounded, shadow, …
opacity     := "/" number                     // v4 oklab color-mix
arbitrary   := "[" value "]"                  // [#0ea5e9], [13px], …
```

- Variants stack left-to-right (`md:hover:bg-*`): the last variant is the
  outermost condition; parse is right-to-left over `:` separators
  (`parse_token`, parser.rs).
- Unknown variants (`has-[>svg]:`, `data-[state=open]:`, `aria-*:`) are
  **skipped with a warning**, never fatal (`parse_class_string` returns
  `(tokens, errors)`); the `tw!` proc macro (`aeolus-macro`) turns the same
  errors into compile errors for static strings — hard contract at compile
  time, soft at runtime.
- Opacity is v4-correct: `bg-primary/90` = color-mix(in oklab, primary 90%,
  transparent) = premultiplied scale of chroma AND alpha
  (`Color::mix_with_transparent`, color.rs) — tested.

## 2. StylePlan IR (plan.rs)

Exactly the §3.1 locked shape: `base: ResolvedStyle` + ordered
`layers: Vec<ConditionalLayer { condition, style }>` + `responsive/container:
BTreeMap<Breakpoint, ResolvedStyle>` + `dark: Option<…>` + `group/peer` maps.
`ResolvedStyle` is a struct of typed `Option` slots (no dynamic maps) so
backends get exhaustive matching; every slot carries `Slot::lane()` =
Cosmetic|Geometric, mirroring Masonry's `core_property_changed` lanes, so
backends can pick the cheapest invalidation via `diff()` + `StyleDiff::needs_layout()`.
`VariantCondition::rank()` fixes the v4 cascade order
(hover < focus < focus-visible < focus-within < active < disabled < visited).

## 3. Theme registry (theme.rs)

- `TailwindTheme::from_theme_css(css)` — CSS-first: parses `@theme`,
  `@theme inline|static`, `:root`/`.dark` variable blocks; `@property`
  declarations pass through. Every theme is **seeded with the verbatim v4
  default palette** (`data/tailwind-default-palette.css`, from
  tailwindcss v4.1.17 `packages/tailwindcss/theme.css`); user declarations
  merge on top (same key → later wins).
- Typed tokens: `Color` (oklab+alpha), `Length(px)`, `FontSize{size,line_height}`,
  `BoxShadowSpec`, breakpoints. Type inference is **value-first**
  (`value_is_color` sniffs oklch/rgb/hsl/hex) so app tokens like
  `--chart-1` work without name lists; `--color-*` / `--text-*` / `--shadow-*`
  prefixes remain as fallbacks for `var()` indirection.
- `calc(var(--radius) - 2px)` is deferred until deps exist, then evaluated
  left-to-right (documented limitation for exotic expressions).
- Runtime theme change is explicit: `absorb_css` (merge + bump `version`) —
  callers drop their `PlanCache`; no CSS-variable pretending (mission §3.5).
- `shadcn_default()` / `shadcn_dark()` ship the neutral light/dark tokens
  **verbatim** from `shadcn-ui/ui @ 4aaea19 apps/v4/app/legacy-themes.css`
  (`.theme-neutral` + its `@variant dark`), including chart/sidebar tokens;
  upstream dropped `--destructive-foreground` and so do we (v4 destructive
  variants use `text-white`).

## 4. Conflict resolution (conflict.rs)

`cn(&["px-2","px-4"]) -> "px-4"`: classes are parsed to tokens, keyed by
`(important, variant-set, conflict-group)`; scanning right-to-left, a later
class in the same key evicts the earlier one; sub-edge survivors keep parents
(`pl-4` survives `px-4` is FALSE — `px-4` evicts `px-*` children; `pl-4`
after `p-4` survives because it is narrower). Group ids follow the tw_merge
semantics (272 groups audited in sections/precedents.md — including the
`text-{size}` vs `text-{color}` disambiguation), with `!` in the key and
negatives normalized. Unparseable input is returned verbatim (best-effort).
Parity is pinned by `tests/shadcn_components.rs` (merge assertions on real
Shadcn strings).

## 5. Cache & interning (cache.rs)

- `Interner`: string → id (hit 387 ns).
- `PlanCache`: keyed by `(interned id | FxHash of string, theme.version)`;
  cold resolve ~4.6 µs, warm hit ~404 ns (report 08).
- `CachedPlan` carries the `Resolution` and the theme version it was built
  against — a theme `absorb_css` invalidates by version bump, not by
  scanning.

## 6. Capabilities (caps.rs)

`BackendCapabilities { properties: PropertySupport, cascade: CascadeModel,
responsive, container_queries, dark_mode }` with
`CascadeModel::{Native{selector_kind}, StateClosures, ViewLevel, None}` —
the contract `lower()` implementations negotiate against (values from the
capability matrix, report 02). The lowering rule from the brief holds:
unsupported utility → `tracing::warn!` in debug, never a panic, never a
silent zero.

## 7. Test plan (tests/)

1. **Unit**: every core module carries an in-src suite (the v0.7
   test-pyramid hardening rebuilt the base: parser/plan/eval/selector/
   cond/ast/caps were integration-only before). Pinned at the type level:
   the `Atom::order` band table, the named-variant table, the selector
   boundary error strings, `supports()` routing, per-slot overlay and
   per-property diff bits. Color round-trips, opacity premultiplication,
   theme parsing (shadcn tokens incl. percentage-lightness oklch),
   breakpoint tokens, spacing default.
2. **Component parity**: the 8 v1 Shadcn class tables must parse, land their
   essential slots, merge with user classes last-wins
   (`tests/shadcn_components.rs`, 14 tests + aeolus-shadcn's 6).
3. **cn() parity oracle**: tw_merge behavior is the oracle (group table
   documented in sections/precedents.md); core merge tests mirror its cases.
4. **Golden corpus (CI parity) — SHIPPED in v0.4** (`AEOLUS_V0.4.md`):
   our own corpus (`tools/gen_golden.mjs`) is generated by the REAL
   tailwindcss 4.1.17 CLI (157 single-class entries, two themes) and
   pinned through the verification CSS emitter (`src/css.rs`) in
   `tests/parity_golden.rs`. The ironframe fixtures stayed out — its
   `core::parse` is a stub and our corpus is generated, version-pinned,
   and re-runnable instead (§9 below).

## 8. Arbitrary selector classification (selector.rs, v0.3)

`Variant::Arbitrary` stays RAW in the AST (the v0.2 §1.2 rule: syntax in
the parser, semantics in the resolver); `selector.rs` is that semantics.
`classify(raw)` maps the v4.1.17 arbitrary-variant grammar (verified
against `candidate.ts`/`compile.ts`/`variants.ts`, not the docs) onto the
`Probe` condition model:

- The typed subset is exactly what constrains THE ELEMENT — the only
  thing a `Probe` answers about. A selector without `&` is wrapped as
  `&:is(sel)` by v4 itself, so even the implicit forms (`[p]`) are self
  tests; the classifier reads them as conjunctions of state / data /
  aria / structural / tag atoms, `:not(...)` as `Atom::Not`, `:has(...)`
  as `Atom::Has`, and `[@media(...)]` / `[@container(...)]` at-rules as
  `Media` / `Viewport` / `Container` atoms (underscore-decoded first).
- `[&:hover]` generates byte-identical CSS to `hover:`, so the
  single-classic-state form canonicalizes onto the legacy layer buckets —
  Floem lowers it into the NATIVE hover selector map with zero extra
  work (pinned by `tests/lowering.rs`).
- Element-kind tests (`[a&]`, `[a]`) become `Test::Tag` — a question
  native backends cannot answer, so the rule simply never fires there
  (the same semantics CSS has when a selector does not match); a backend
  that models element kinds answers honestly. Tag ranks with the state
  band in `Atom::order` so arbitrary forms identical to named ones share
  their cascade position.
- Everything else fails with a per-form precise reason: selectors
  styling other elements (`[&_svg]`, `[&>*]`, `*`, `**`), pseudo-elements,
  class/id self tests, `@supports`, disjunctions (`,` lists, `:is(a,b)`,
  `or` media — a `Cond` is a conjunction), An+B forms, bare non-data/aria
  attributes, unknown pseudo-classes and media features, standalone
  relative selectors, and the deprecated at-rule-with-selector form.
- `not-*`/`group-*`/`peer-*` inners classify through `single_test`
  (exactly one self test; nested negation and compounds are reported —
  the wrappers hold ONE `Test`).

## 9. The verification CSS emitter (css.rs, v0.4)

`Resolver::to_css` answers "what did the compiler understand?" — it
derives Tailwind-v4-equivalent CSS from the SAME resolution backends
consume (`route` + `apply_utility`), one `CssRule` per class token, v4's
one-rule-per-candidate shape. It is a verification and debugging tool,
not a styling path: backends keep consuming `StylePlan` directly, and
classes the resolver cannot apply keep their warnings.

- Selectors mirror v4's templates: escaped class names (everything
  outside `[a-zA-Z0-9_-]` backslash-escaped), state pseudo-classes,
  data/aria attribute tests (`aria-*` bare means `"true"`), structural
  pseudo-classes, `:not(*:hover)` / `:has(*:focus)` star forms, group/peer
  descendant templates (`&:is(:where(.group):hover *)`), and
  `@media`/`@container` wrappers with v4's canonical `(width >= 48rem)`
  conditions (rem reconstructed from the theme px / 16).
- Values mirror v4's shapes: spacing utilities emit
  `calc(var(--spacing) * n)` (the token text restores the scale number
  the IR resolved to px), colors emit oklch (numeric-compare in the
  harness), opacity modifiers emit `color-mix(in oklab, …)` (the mixed
  color is un-mixed by scaling chroma back), rings and shadow presets
  emit composed `box-shadow` layers, `rounded-full` emits
  `calc(infinity * 1px)` (the plan keeps the finite 9999px sentinel),
  viewport units come from the token text (`h-screen` → `100vh`,
  `h-dvh` → `100dvh` — the plan stores the abstract `Viewport`).
- The golden corpus (`tests/parity_golden.rs` + `tools/gen_golden.mjs`)
  pins the emitter — and therefore the whole pipeline — against the REAL
  tailwindcss 4.1.17: 157 entries, two themes, zero-warning resolution
  asserted per entry, both sides normalized through one pipeline
  (documented model normalizations: hover-capability wrappers dropped,
  `@supports` modern-browser overrides merged, initial
  border/outline-style dropped, unitless line-height × same-rule
  font-size).

## 10. Element-relative typography (v0.5)

`AEOLUS_V0.5.md` §1 reverses the v0.4 §2.3 corpus exclusion: the token
forms of `tracking-*` / bare `leading-*` / named `text-*/*` leading
resolve to the RELATIVE unit the class names — `TextValue::TrackingEm`
(the `--tracking-*` em) and `TextValue::LineHeightRatio` (the
`--leading-*` unitless ratio) — not a 16px pre-multiplication.

- WHY: `tracking-tight` at the 16px base is −0.4px, but the element's
  real font-size is whatever the cascade resolved (`text-sm` → 14px →
  −0.35px). The compiler cannot know which font-size an element ends up
  with; carrying the em/ratio and letting the renderer multiply is the
  only exact model — and it is exactly what v4 emits
  (`letter-spacing: -0.025em`, `line-height: 1.25`).
- WHAT STAYS ABSOLUTE: arbitrary values (`tracking-[7px]`,
  `leading-[3px]`), the spacing-multiple number forms (`leading-6` =
  `calc(var(--spacing) * 6)`, `text-sm/6`) and the token-paired
  line-heights of bare `text-{size}` (the pairing is part of the size
  token itself — v0.4 behavior, corpus-pinned).
- Backends: Floem lowers the ratio to its native
  `LineHeightValue::Normal(multiplier)` (font-size-relative at paint);
  letter-spacing keeps its honest drop report there (no property at the
  pinned rev). The Masonry fallback applies no text properties (the Q5
  gap) — unchanged.
- The corpus pins these as relative units (176 entries): both sides emit
  the same relative text; the harness's only new machinery is folding
  em-valued `calc()`s unit-preservingly (`calc(0.025em * -1)` →
  `-0.025em`) and no longer eating a unit-leading `e` as an exponent.

## 11. The allocation contract (v0.5)

`AEOLUS_V0.5.md` §3 closes report 08's follow-up: a counting global
allocator (`tests/alloc_contract.rs`) pins the hot path's allocation
behavior as machine-independent numbers. Exact-zero claims (diff of
equal plans, intern hit, warm cache hit — an `Arc` bump, it turns out,
not a plan clone) are pinned exactly; clone-shaped claims as measured
upper bounds; `cn()` and the cold resolve are counted for report 08.
The contract caught `diff()` allocating on every call (the slot walk
routed through `slots()`' scratch `Vec`) — slot presence now walks one
`for_each_present_slot!` source, and the rebuild path is
allocation-free when nothing changed.

## 12. The upstream ceiling (v0.6)

`AEOLUS_V0.6.md` consumes everything the latest GitHub floem and masonry
expose WITHOUT an upstream change — verified, not assumed: the floem pin
`1351ffb` IS latest `main` HEAD (`git ls-remote`), and xilem upstream
moved `9f504ae → dc49687` with an EMPTY `masonry_core`/`xilem_masonry`
diff (3 commits: tests + a vello dep removal).

- **Gradients, the audit-missed capability.** Masonry has carried a
  layout-invariant `Background::Gradient` all along
  (`properties/types/gradient.rs`: angle-based `GradientShape`, oklab
  `interpolation_cs`, peniko `ColorStops`) — the v0.1–v0.5 audits
  reported gradients as dropped on BOTH backends; that row was stale.
  The IR therefore gains the composed `background_gradient`
  (`Gradient::Linear/Radial/Conic/None`) as a SEPARATE slot from the
  solid `background`, exactly mirroring CSS's `background-color` +
  `background-image` coexistence (tw_merge's separate bg-color/bg-image
  groups; the emitter reproduces v4's per-class `--tw-gradient-*` var
  shapes verbatim). Composition semantics are pinned against the REAL
  tailwindcss 4.1.17 (`tools/capture_gradients.mjs`): stop positions
  default to the `@property` initial values (0/50/100%), missing
  from/to stops are `#0000`, interpolation is `in oklab`, stops without
  a shape compose nothing, `bg-none` clears like a value.
- **Model boundary, not a corpus gap.** Gradient parity needs cross-rule
  `--tw-gradient-*` var composition (three separate rules per gradient
  in v4's output); the golden corpus stays at 176 entries and the
  boundary is documented here — the v0.5 §2.3 precedent (relative units
  pinned as units, element-resolved composition out).
- **The unwired Floem surface.** The pinned rev (= HEAD) exposes
  per-axis `overflow_x`/`overflow_y`, the full grid surface
  (`grid_template_*`, `grid_column`/`grid_row` placement), `align_content`,
  the inherited `Selectable` prop, `TextOverflow::Wrap`, and kurbo
  `Stroke` dashes on the border strokes (`paint_border` strokes with the
  full `Stroke`). All wired in v0.6; six parity-ledger slots upgrade
  Reported → Native; every normalization (auto→Scroll, normal→Stretch,
  ellipse→circle, dash lengths, single-brush solid+gradient) is a
  documented drop-record or unit-pinned choice, never a silent fail.
- **The masonry angle conversion** is derived from masonry's paint math,
  not its doc comments: `get_peniko_linear_for_rect` puts angle 0 at the
  BOTTOM (`end = center + (sin a, cos a)·len/2`), so CSS angle θ maps to
  `a = π − θ` — pinned per corner direction.

## 13. Oracle fidelity — the variant & grid engine (v0.7)

`AEOLUS_V0.9.md` closes every parser/IR gap the shipped shadcn oracle
tables expose: a 2026-11-08 audit counted 40 warnings across the tables;
after v0.7 the count is 13 and every one is the documented v0.3 §1.3
Probe-model boundary (selectors styling OTHER elements). The headline
contract lives in `crates/aeolus-shadcn/tests/oracle_fidelity.rs`.

- **Pseudo-element variants** (`selection:`, `placeholder:`, `file:` —
  §1) are typed `Test::Pseudo(PseudoElement)` conditions, NOT runtime
  states: they style a sub-rendering of the element. `Probe::test`
  answers them (default "no" — the rule never fires where the rendering
  does not exist, the same semantics CSS has when a selector does not
  match), so `selection:bg-primary` rides the `rules` path with zero new
  bucket machinery. The real 4.1.17 emits BOTH `&::selection` and the
  descendant `& *::selection`; the self half is corpus-pinned, the
  descendant half is outside the Probe model by construction.
- **`Atom::Has` generalizes to `Has(Test)`** (§2): `has-data-[…]`/
  `has-aria-[…]`/`has-first:` classify through the same `variant_test`
  the `group-*` wrappers use; `Probe::descendant(&Test)` (default:
  state-only) keeps old trait impls compiling. Combinator forms
  (`has-[>svg]`) keep the precise report — the child-edge is real
  information the typed descendant question does not carry.
- **Named containers** (§3): `@container[/name]` is a UTILITY in v4 —
  `Extras.container_type` carries `container-type: inline-size` + the
  optional name; conditions already carried names since v0.2.
- **Track templates** (§4): `GridTrack { min, max }` mirrors CSS
  `minmax()`; a bare `1fr` is `minmax(auto, 1fr)` (taffy's `from_fr`),
  but the `auto-rows-fr` KEYWORD is `minmax(0, 1fr)` (v4's blowout guard
  — the first capture proved CSS intuition wrong, so it is pinned).
  Templates are CANONICAL over the v0.6 numeric counts; the template
  families are `cn()` conflict groups that evict across the
  numeric/arbitrary boundary. Floem lowers everything natively
  (`GridTemplateComponent::Single/Repeat`, `grid_auto_rows/columns`).
- **`justify-items-*`** (§5): one resolver arm the cn() tables already
  anticipated; `normal` → stretch (CSS), Floem native.
- **Corpus 193** (§6): every new shape is single-rule and IN; the
  selection descendant half is a documented harness normalization, as is
  comma-spacing (v4 emits arbitrary values verbatim and keywords pretty).

## 14. Portable plans (v0.7)

`AEOLUS_V0.7.md` makes the IR a first-class artifact instead of a private
resolver↔backend contract. Three additions, all zero-dependency:

- **Plan algebra** (`StylePlan::overlay`, plan.rs). `cn()` composes class
  STRINGS (theme-aware, parse-based); `overlay` composes RESOLVED plans:
  later-wins per slot, layers merge by condition and stay rank-ordered,
  responsive/dark/group/peer buckets overlay per key, rules merge by
  `Cond` equality, the override's breakpoint table wins. Idempotent, and
  diff-integrated — `diff(base, merged)` reports exactly the override
  slots, so the repaint/relayout lanes survive composition. The runtime
  story: a component's cached plan gains per-instance overrides without a
  theme reference or a re-resolve. The per-edge layer flattening that
  v0.7 pinned and deferred (§1.4/§6) is FIXED by v0.8 §1 (see §15).
- **Canonical snapshots** (`snapshot.rs`). `to_snapshot_json` emits fixed-
  schema-order, compact, shortest-round-trip JSON — byte-stable for equal
  plans. `from_snapshot_json` is STRICT (unknown fields/enum names,
  duplicate keys, wrong header, truncation, trailing content → typed
  `SnapshotError` with a dotted path) — no silent defaults, the artifact
  counterpart of the no-silent-failures rule. The JSON parser is
  hand-rolled (depth-limited, full escapes incl. surrogate pairs); the
  non-finite encoding (`"NaN"`/`"inf"`/`"-inf"`) keeps the writer total.
  The closed `KEYWORDS` table is how `TextValue::Keyword(&'static str)`
  round-trips soundly.
- **The headless surface** (`aeolus-cli`). `run(args, out, err) -> code`
  is the whole program (process-testable): `resolve` emits the snapshot
  artifact by default, `css` drives the verification emitter, `diff`
  reports lane-aware base diffs, `cn` merges, `check` lints source files
  with a documented extractor heuristic (quoted strings kept when ≥ half
  the tokens resolve warning-free) and cn()-conflict attribution. Exit
  contract: 0 success, 1 findings, 2 usage/theme/IO.

## 15. The per-edge set-mask (v0.8)

The four per-edge families (`Padding`/`Margin`, `BorderWidth`,
`CornerRadius`) carry a set-mask (`EdgeSet`/`CornerSet`) beside their
`f32` values: `false` = "the class string never named this edge" — the
0.0 is a placeholder, never a cascade input. The v0.7 finalize destroyed
that information (every edge materialized as 0.0), which made a
`hover:px-8` layer wipe the base's top/bottom on every backend and made
`p-4 ⊕ px-8` compose to `{0, 32, 0, 32}`.

Semantics, all in the core so every consumer shares one model:

- `finalize` records the mask from its `Option`-per-edge accumulation.
- `ResolvedStyle::overlay` merges the four families EDGE-WISE (an edge
  the override addresses wins; otherwise the destination keeps its
  value) — `StylePlan::overlay` and the probe flatten
  (`StylePlan::resolve`) route through it, fixing composition and the
  runtime cascade with one primitive. Idempotence and the
  diff-integration contract survive with partial edges.
- `ResolvedStyle::materialize_unset_edges_from(&base)` fills a partial
  layer's masked-off edges from the base — the primitive for cascades
  that replace whole properties. Xilem's stack builder and Slint's book
  builder call it; Floem applies per-field gated by the mask; Iced and
  egui carry the mask in their lowered payload bookkeeping and merge
  edge/corner-wise in `overlay`.
- The snapshot schema bumps to v2: masks serialize as required `"set"`
  objects, the strict reader rejects malformed ones with dotted paths,
  v1 documents fail with `UnsupportedVersion(1)` (loudly, by design).
  (The variant-engine merge later bumps v2 → v3 for the track templates,
  `container_type`, `justify_items` and the `pseudo`/`has` test shapes —
  a strict superset of the v2 payload; see §13.)

Pinned by `tests/per_edge_layers.rs` (11 tests) plus the per-backend
partial-edge tests; evidence in
`reports/10-tdd-evidence/10-per-edge-set-mask.md`; the plan of record is
`AEOLUS_V0.8.md` §1.
