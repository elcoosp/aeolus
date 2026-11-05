# Changelog

All notable changes to Aeolus are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
