# Changelog

All notable changes to Aeolus are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
