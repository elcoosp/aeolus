# Aeolus v0.6 Plan (plan of record)

The v0.5 plan (`AEOLUS_V0.5.md`) shipped in 0.5.0. This document is the
v0.6 plan of record; every `v0.6 §X` citation in code and reports refers
here.

Goal: **consume the upstream ceiling — everything the latest GitHub
floem and masonry expose, without a single upstream change.** The
reconnaissance (2026-11-07) checked both pinned frameworks against their
`main` HEADs and re-audited the vendored sources for capabilities the
backends report as dropped:

- **floem**: the pinned rev `1351ffb` **is** latest `main` HEAD
  (`git ls-remote https://github.com/lapce/floem HEAD` → `1351ffb…`,
  byte-identical). The vendored source exposes a native surface the
  backend never wires: per-axis `overflow_x`/`overflow_y` (taffy
  `Overflow`), a full grid surface (`grid_template_columns`/`rows`,
  `grid_column`/`grid_row` placement, `grid_auto_flow`), the
  `AlignContent` prop, the inherited `Selectable` prop, the
  `TextOverflow::Wrap { overflow_wrap, word_break }` form, and kurbo
  `Stroke` dashes on the per-side border strokes (`paint_border` strokes
  with the full `Stroke`, dashes included).
- **masonry/xilem**: upstream moved `9f504ae → dc49687` — exactly 3
  commits (`17b8e85` text-widget tests, `f5c6f30` example interaction
  tests, `dc49687` vello dep removal); `git diff 9f504ae..dc49687 --
  masonry_core xilem_masonry` is **empty**. The pin stays. But the audit
  missed a capability the pinned rev has had all along: the
  layout-invariant `Background::Gradient` (`properties/types/gradient.rs`
  — angle-based `GradientShape::Linear/Radial/Sweep`,
  `interpolation_cs: ColorSpaceTag`, `ColorStops`). Linear and radial
  gradients lower NATIVELY today; the capability matrix's "gradients
  parse, dropped+reported" row is stale on the masonry side.

No backend-manifest changes, no MSRV bump, no upstream patches. The four
upstream-blocked deferrals (v0.4 §6) stay deferred and are re-verified in
§5.

## §1 — Gradient backgrounds (core IR + emitter; masonry native)

- §1.1 The IR keeps the v4 model: `background-color` and
  `background-image` are SEPARATE declarations. `ResolvedStyle.background`
  stays the solid (`Background::Solid` — the tuple struct becomes an
  additive enum: `Solid(Color)` | `Linear { … }` | `Radial { … }` is
  NOT used for the slot; see §1.2) and a new
  `ResolvedStyle.background_gradient: Option<Gradient>` carries the
  composed image:
  ```rust
  pub enum Gradient {
      Linear { angle_deg: f32, stops: GradientStops }, // CSS convention: 0 = to top, clockwise
      Radial { stops: GradientStops },                 // center 50%/50%, circle farthest-corner (§1.5)
      Conic { from_angle_deg: f32, stops: GradientStops },
  }
  pub struct GradientStops { pub from: GradientStop, pub via: Option<GradientStop>, pub to: GradientStop }
  pub struct GradientStop { pub color: Color, pub pos_pct: Option<f32> }
  ```
  `pos_pct: None` = the v4 `@property` default (from 0%, via 50%, to
  100% — captured verbatim from tailwindcss 4.1.17, `tools/capture_gradients.mjs`).
  Backends with a single background brush lower `background_gradient`
  preferentially and record the solid coexistence honestly (§1.6).
- §1.2 Parser (v4.1.17-pinned semantics): `bg-linear-to-{t,tr,r,br,b,bl,l,tl}`
  → CSS angles 0/45/90/135/180/225/270/315; `bg-linear-<angle>` and
  arbitrary `bg-linear-[<angle>deg]` → the angle; the deprecated v4 alias
  `bg-gradient-to-*` resolves identically; `bg-radial` → `Radial`;
  `bg-conic`/`bg-conic-<angle>` → `Conic { from_angle_deg }`; `bg-none`
  clears the image. Stops: `from-{color}`, `via-{color}`, `to-{color}`
  (theme + shadcn tokens + arbitrary + `/opacity` modifiers via the
  existing `with_opacity` machinery — v4 emits
  `color-mix(in oklab, var(--color-x) P%, transparent)`, exactly the
  shape the resolver already produces); `from-{P}%`/`via-{P}%`/`to-{P}%`
  patch ONLY the stop position (v4: `--tw-gradient-from-position: 20%`);
  `from-*`/`to-*` keyword colors `transparent` resolve, `current` is
  reported (no `currentColor` in the color IR). Missing `to` stop =
  transparent @100%, missing `from` = transparent @0% (v4 `@property`
  initial values `#0000` — captured). Composition at finalize: shape +
  stops → `background_gradient`; stops without a shape compose nothing
  (v4: `--tw-gradient-position` unset → no image).
- §1.3 Slots/diffs: `Slot::Gradient(GradientPart)` with
  `GradientPart::{Shape, From, Via, To}` — lane `Cosmetic` (paint-only
  on both backends); the `slot_diff` arms compare the new extras fields;
  `cn()` conflict groups: `bg-linear-*`/`bg-radial`/`bg-conic*`/`bg-none`
  share the background-image group (later wins), `from-*`/`via-*`/`to-*`
  conflict per edge (later wins); solid `bg-{color}` and a gradient
  coexist (different CSS properties — matches tw_merge's separate
  bg-color/bg-image groups).
- §1.4 The verification emitter (css.rs) emits the composed declaration
  `background-image: linear-gradient(<angle>deg in oklab, <from> <pos>, [<via>
  <pos>,] <to> <pos>)` / `radial-gradient(circle farthest-corner in oklab, …)`
  alongside `background-color` when a solid coexists. DOCUMENTED
  NORMALIZATION: v4 splits the same information across `--tw-gradient-*`
  custom properties on three separate rules; the emitter composes at plan
  time because the IR composes at plan time (the golden corpus stays OUT
  — §2.3).
- §1.5 Backends. Masonry (the headline): `Background::Gradient(Gradient)`
  — `GradientShape::Linear { angle }` takes CSS-convention radians
  directly (masonry's paint math `get_peniko_linear_for_rect` implements
  the CSS spec formula; angle 0 = to top, clockwise positive — verified
  against the source, NOT the doc comment's wording on the Sweep variant);
  `interpolation_cs: Oklab` (v4's default `in oklab`); `Extend::Pad`;
  stops in sRGB with fraction offsets (`pos_pct / 100`). Radial lowers to
  `Radial { center: UnitPoint::CENTER, shape: CircleTo(FarthestCorner) }`
  with the ellipse→circle normalization recorded (masonry's
  `RadialGradientShape` has no ellipse radii — upstream `#[non_exhaustive]`
  TODO). Conic is REPORTED with the precise upstream reason (masonry's
  sweep paint carries a vello angle-offset TODO — CSS from-angles outside
  vello's 0..2π cull the gradient). Floem: gradients stay REPORTED, with
  the sharpened reason (peniko `Brush::Gradient` needs concrete gradient
  geometry; `paint_bg` fills the brush as-is — no rect adaptation, so a
  "to right" gradient would need per-view size knowledge, which the
  zero-view-work model forbids).
- §1.6 Single-brush reality: when a gradient AND a solid coexist, Masonry
  lowers the gradient and records
  `("background-color under gradient", "CSS layers background-image over background-color; the single Background property takes the gradient")`.
  Gradient-only or solid-only plans are unchanged.

## §2 — Floem: the unwired native surface (all at the pinned rev = HEAD)

- §2.1 `overflow-*` → `overflow_x`/`overflow_y` (`taffy::style::Overflow`):
  Visible/Hidden/Clip/Scroll map 1:1; `overflow-auto` lowers to
  `Overflow::Scroll` (taffy has no auto; the reserved-scrollbar difference
  is recorded in the lowering table, not a drop).
- §2.2 `select-none`/`select-text` → the inherited `Selectable` prop
  (`selectable(false)` / `selectable(true)`) — labels read it
  (`views/label.rs`). `select-all`/`select-auto` keep precise drop
  reports (no selection-mode prop).
- §2.3 `whitespace-*` → Floem's single `TextOverflow` prop, by CSS
  semantics: `whitespace-normal` → `TextOverflow::Wrap { overflow_wrap:
  Normal, word_break: Normal }` (native); `whitespace-nowrap` is a
  NO-OP — Floem's default `TextOverflow::NoWrap(Clip)` already means
  "never wrap" (documented, not a drop); `pre`/`pre-line`/`pre-wrap`
  keep precise reports (no whitespace-preservation control; the nowrap
  half is the Floem default). `text-overflow` classes lower as before;
  when `whitespace-normal` and `text-{ellipsis,clip}` coexist the LAST
  text slot wins the single prop (mechanical later-wins, documented).
- §2.4 `content-*` → `align_content` (taffy `AlignContent`): Start→
  FlexStart, Center, End→FlexEnd, Between/Around/Evenly/Stretch 1:1,
  Normal→Stretch (CSS `align-content: normal` resolves to stretch for
  flex containers — taffy's default), Baseline→ reported (taffy has no
  baseline align-content).
- §2.5 Grid: `grid-cols-N`/`grid-rows-N` → `grid_template_columns/rows`
  = `vec![GridTemplateComponent::Repeat(GridTemplateRepetition { count:
  RepetitionCount::Count(N), tracks: vec![TrackSizingFunction::from_fr(1.0)],
  line_names: vec![] })]`; `col-span-N`/`row-span-N` →
  `grid_column`/`grid_row` = `Line { start: GridPlacement::Span(N), end:
  Auto }`; `col-start-N`/`row-start-N` → `Line { start:
  GridPlacement::Line(N), end: Auto }`; start+span coexisting →
  `Line { start: Line(N), end: Span(span) }` (CSS `grid-column: N / span M`).
  Arbitrary templates (`grid-cols-[…]`) stay reported.
- §2.6 `border-dashed`/`border-dotted` → kurbo `Stroke` dashes on the
  four side strokes (`with_dashes(0., [6., 6.])`; dotted =
  round-cap `[0.01, w]` dot pattern) — `paint_border` strokes with the
  full `Stroke`, so the pattern is native at the pinned rev. The dash
  pattern is a DOCUMENTED NORMALIZATION (CSS dash lengths are UA-styled).
  `border-none`/`border-hidden` → zero-width side strokes (Tailwind
  preflight keeps border-width 0 — the widths already carry the truth);
  `border-double` keeps its report (no double-stroke primitive).
- §2.7 The parity ledger (`slot_coverage`) upgrades: Overflow, Select,
  Whitespace (normal/nowrap), AlignContent (non-baseline), Grid, and
  BorderStyle (dashed/dotted/none/hidden) move Reported → Native; the
  capability matrix rows are rewritten accordingly.

## §3 — Upstream re-verification (evidence, pins unchanged)

- §3.1 floem: pin `1351ffb` == latest `main` HEAD (2026-11-07,
  `git ls-remote`); the backend now consumes the surface audited in §2
  — documented per item with source citations.
- §3.2 masonry/xilem: upstream `9f504ae..dc49687` = 3 commits, zero
  `masonry_core`/`xilem_masonry` changes; the pin stays. The stale
  capability-matrix claim is corrected: `Background::Gradient` exists at
  the pin and is now consumed (§1.5). `BorderWidth` is still uniform,
  `CornerRadius` still single-radius, `BoxShadow` still lacks
  spread/inset — verified against the pinned sources and re-documented.
- §3.3 crates.io re-check (2026-11-07): `masonry`/`masonry_core`/
  `masonry_testing` 0.4.0 (2025-10-29) and `floem` 0.2.0 (2024-11-14)
  remain the newest releases — publishing the backends stays blocked.

## §4 — Quality gates

- §4.1 The full battery stays green: `cargo test --workspace`,
  `clippy -D warnings`, `fmt --check`, `doc -D warnings`,
  `cargo package -p aeolus-core -p aeolus-macro`, `cargo deny check licenses`.
- §4.2 New tests (RED→GREEN, captured in
  `reports/10-tdd-evidence/08-upstream-ceiling.md`): gradient
  parse/composition unit tests (the v4.1.17-captured semantics of §1.2),
  emitter shape tests, the masonry angle-conversion + stop-offset tests,
  the floem overflow/grid/selectable/whitespace/content/border-dash
  lowering tests, and the slot_coverage upgrades.
- §4.3 The golden corpus (176 entries) is UNCHANGED: gradient parity
  needs cross-rule `--tw-gradient-*` var composition (three separate
  rules per gradient), which is a harness model boundary, not a coverage
  gap — same precedent as v0.5 §2.3, now documented in DESIGN §12. The
  captured real-compiler output lives in
  `tools/capture_gradients.mjs` (evidence, re-runnable).
- §4.4 No manifest/MSRV changes; version → 0.6.0; docs: README status,
  CHANGELOG, DESIGN §12, capability matrix gradient/overflow/grid rows,
  `AEOLUS_V0.6.md` (this file).

## §5 — Upstream-blocked (stay deferred, re-documented)

Re-verified 2026-11-07 (§3): publishing the backends (no compatible
crates.io release), view-side property-stack registration (no
`Pod::with_property_stack` at `dc49687`), Masonry typography via native
properties (no `FontSize`/`FontWeight` property upstream; the Q5 parley
per-widget gap), Masonry transitions/animations (no property-animation
infrastructure). Unchanged from v0.4 §6 / v0.3 §4 / v0.5 §5.
