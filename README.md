# Aeolus

**Aeolus is a Tailwind CSS v4 → native-Rust style compiler.** It parses
Tailwind class strings, resolves theme tokens, variants, opacity modifiers
and conflict groups, and lowers the result into a typed IR —
[`StylePlan`](crates/aeolus-core) — that native UI frameworks consume
directly, without a browser and without CSS.

```rust,ignore
use aeolus_core::{TailwindTheme, Resolver, cn};

let theme = TailwindTheme::from_theme_css(
    ":root { --primary: oklch(0.205 0 0); --radius: 0.625rem; }",
)?;
let resolver = Resolver::new(&theme);
let plan = resolver.resolve("p-4 bg-primary hover:bg-primary/90 rounded-md")?;
// plan.base.background  = Some(primary)
// plan.base.padding     = 16px per edge
// plan.layers           = [hover layer at 90% alpha]   <- real OKLCH color-mix
```

Status (2026-11): **v0.1.0** — `aeolus-core` and `aeolus-macro` are
publishable crates; the backend crates are feature-complete for their
backends but stay `publish = false` (see *What's in v0.1 vs v0.2*).

## Crates

| crate | what it is | publishes? |
|---|---|---|
| [`aeolus-core`](crates/aeolus-core) | parser, theme registry, `StylePlan` IR, `cn()` conflict resolution, bounded caches — **zero dependencies** | ✅ |
| [`aeolus-macro`](crates/aeolus-macro) | `tw!` / `tw_cn!` compile-time class validation (syn-free) | ✅ |
| [`aeolus-xilem`](crates/aeolus-xilem) | Masonry/Xilem backend: `StylePlan` → `PropertyStack` + fallback widget + **real interactive `StyledButton`** | ❌ pinned git deps |
| [`aeolus-floem`](crates/aeolus-floem) | Floem backend: `StylePlan` → nested selector maps, transitions, responsive + dark | ❌ pinned git deps |
| [`aeolus-shadcn`](crates/aeolus-shadcn) | Shadcn v4 class tables (verbatim) + wrappers: real `button`/`input`, style-only badge/label/card/alert | ❌ depends on backends |

All crates are `MIT OR Apache-2.0` (`LICENSE-MIT`, `LICENSE-APACHE`).

## What's in v0.1 — honestly

**Done and tested (87 tests):**

- Tailwind v4 subset parser (the Shadcn v4 component surface): variants
  (state, breakpoint, container, group/peer, stacking), arbitrary values,
  opacity modifiers as OKLCH color-mix, `!important`, negatives.
- Theme registry: `@theme` / `:root` / `.dark` CSS, verbatim v4 default
  palette seeding, typed tokens, `calc()` with **real operator
  precedence**, dark-variant derivation, version-tracked theme swaps.
- `StylePlan` IR: base + conditional layers + responsive/container/dark/
  group/peer buckets; typed slots with cosmetic/geometric lanes for
  two-lane invalidation; `diff()`.
- `cn()`: tw_merge-compatible conflict resolution (the audited 272-group
  table drives classification), approximate equality where floats demand
  it, total and idempotent on ANY input.
- Masonry/Xilem backend: lowered `PropertyStack`s equal hand-written
  stacks (pinned by test), two-lane fallback widget with repaint-only
  hover restyle (pinned by `TestHarness`), and a **real interactive
  button** (pointer, Space/Enter, accesskit, disabled — acceptance tests
  click it).
- Floem backend: full-plan lowering into Floem's programmatic style
  engine, evaluated through its public cascade evaluator in tests.
- Shadcn v1 (8 components): verbatim v4 class tables with integrity
  tests; `button()` is a real widget, `input()` is a real text input.
- Hardening: 4 fuzz targets (60s clean each), property tests, CI gates,
  bounded caches, poison-tolerant locks, `#![deny(missing_docs)]`.

**Deferred to v0.2 (tracked in CHANGELOG):** publishing the backends
(needs upstream releases), group/peer conditional layers (needs a Masonry
patch — proposal in `reports/07-upstream-patches/`), view-side stack
registration (retires the fallback widget), Masonry typography/responsive/
dark slots, container queries, transitions/animations (recognized and
dropped **with warnings**), structured variants (`has-[…]`, `data-[…]`).

The full capability matrix per framework lives in
[`reports/02-capability-matrix/CAPABILITY_MATRIX.md`]; the honest-limitation
contract (every drop is reported, never silent) in the design report.

## Building

Stable Rust 1.88+ (MSRV). No sibling checkouts needed — backend framework
deps resolve from pinned git revs.

```sh
git clone https://github.com/aeolus-rs/aeolus && cd aeolus
cargo test --workspace          # the whole suite
cargo clippy --workspace --all-targets -- -D warnings
cargo doc --workspace --no-deps # RUSTDOCFLAGS="-D warnings" is CI's gate
cargo bench -p aeolus-core      # hot-path numbers (report 08)
```

Packaging (publishable set):

```sh
cargo package -p aeolus-core -p aeolus-macro --allow-dirty --no-verify
```

Fuzzing (nightly; targets in `fuzz/`):

```sh
cargo +nightly fuzz run parse_class_string -- -max_total_time=60
cargo +nightly fuzz run color_parse        -- -max_total_time=60
cargo +nightly fuzz run cn                 -- -max_total_time=60
cargo +nightly fuzz run theme_from_css     -- -max_total_time=60
```

## Architecture in one paragraph

`aeolus-core` is the semantic layer: class strings → tokens →
theme-resolved `StylePlan`. Backends lower the plan into their framework's
native cascade — Masonry `PropertyStack` + `Selector` flags (with an
honest per-state fallback widget until upstream lands view-side stack
registration), Floem's nested selector maps (zero view-side work), and a
shared Shadcn class-table crate on top. Every backend negotiates its
capabilities (`BackendCapabilities`) and reports everything it cannot
express (`LoweringReport`, `Warning`) — the no-silent-failures rule.

## Reports

The `reports/` directory is the project's research memory and stays
current with the code: source audit + Q&A (01), capability matrix (02),
core design (03), Shadcn v1 (06), upstream patch proposals (07),
benchmarks (08), publishing decision (09), release checklist (09-release),
and per-phase TDD evidence (10).

## Contributing

Read [`CONTRIBUTING.md`](CONTRIBUTING.md) — TDD-first (failing test → RED
→ fix → GREEN), no silent failures, conventional commits. Security:
[`SECURITY.md`](SECURITY.md).
