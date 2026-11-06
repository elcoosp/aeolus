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

Status (2026-11): **v0.3.0** — the v0.2 plan (`AEOLUS_V0.2.md`) shipped in
0.2.0; **v0.3** (`AEOLUS_V0.3.md`) closes the last core-language gap:
arbitrary selector variants (`[&:hover]`, `[a&]`, `[@media(print)]`, …)
resolve to typed probe conditions or fail with a per-form precise report
— semantics verified against the tailwindcss v4.1.17 source, plus the
post-0.2.0 audit fixes (theme order-independence, px semantics, `cn()`
groups). `aeolus-core`/`aeolus-macro` stay publishable; backends stay
`publish = false` (git-pinned deps).

## Crates

| crate | what it is | publishes? |
|---|---|---|
| [`aeolus-core`](crates/aeolus-core) | parser, theme registry, `StylePlan` IR, `cn()` conflict resolution, bounded caches — **zero dependencies** | ✅ |
| [`aeolus-macro`](crates/aeolus-macro) | `tw!` / `tw_cn!` compile-time class validation (syn-free) | ✅ |
| [`aeolus-xilem`](crates/aeolus-xilem) | Masonry/Xilem backend: `StylePlan` → `PropertyStack` + fallback widget + **real interactive `StyledButton`** | ❌ pinned git deps |
| [`aeolus-floem`](crates/aeolus-floem) | Floem backend: `StylePlan` → nested selector maps, transitions, responsive + dark | ❌ pinned git deps |
| [`aeolus-shadcn`](crates/aeolus-shadcn) | Shadcn v4 class tables (verbatim) + wrappers: real `button`/`input`, style-only badge/label/card/alert | ❌ depends on backends |

All crates are `MIT OR Apache-2.0` (`LICENSE-MIT`, `LICENSE-APACHE`).

## What's in v0.3 — honestly

**Done and tested (166 tests):**

- Everything listed under v0.2 below, PLUS the post-0.2.0 audit round
  (theme resolution order-independence, px-vs-spacing semantics for
  ring/outline widths, canonical state-layer cascade, legacy comma color
  syntax, missing `cn()` merge groups).
- **Arbitrary selector variants** (`AEOLUS_V0.3.md`, the last core-language
  gap): `[&:hover]` ≡ `hover:` (canonicalized onto the same native layer),
  `[&:hover:focus]` conjunctions, `[&:not(:hover)]`, `[&[data-state=open]]`,
  `[&:nth-child(3)]`, `[&:has(:focus)]`, `has-[:focus]`, `group-[&:hover]`,
  `not-[&:hover]`, element-kind tests (`[a&]`, `[a]` → the new
  `Test::Tag`), and arbitrary at-rules (`[@media(print)]`,
  `[@media(min-width:900px)]`, `[@media(width>=900px)]`,
  `[@media(prefers-color-scheme:dark)]`, orientation/motion/contrast/
  pointer features, `and`-conjunctions, `[@container(min-width:…)]`) —
  all typed; every other form (`[&_svg]`, `[&>*]`, `[&::before]`, `[.x]`,
  `@supports`, disjunctions, An+B) reports a PRECISE per-form reason.

<details><summary><b>What was in v0.2</b></summary>
- **Correctness (F1–F7)**: fractions (`w-1/2` = 50%, not 4px), absolute
  arbitrary lengths (`p-[7px]` = 7px), compound variants (`dark:hover:*`
  and `md:hover:*` keep BOTH conditions — the shipped Shadcn Ghost/Outline
  hovers are correct), logical radius edges (`rounded-s-lg`), `text-sm/6`
  line-height modifiers, bracket/var opacity (`[0.37]`, `(--o)`), exact
  `cn()` variant keys, collision-proof string-keyed plan cache.
- **The parity engine**: `Probe` + `StylePlan::resolve` — backends answer
  questions about the element's situation; ONE evaluator flattens the plan
  for both. Compound variants, group/peer, `data-*`/`aria-*`, `not-*`,
  `has-*`, structural selectors and container queries all flow through it.
- **Full v4 default theme** (diffed against tailwindcss v4.1.17):
  `--container-*`, multi-layer `--shadow-*`, text scales with paired line
  heights, weights, tracking, leading, radii, easings, animations.
- **Utility coverage guard**: every v4 utility family resolves to a typed
  slot or a precise `Unsupported` — `Unknown` fails CI.
- **Floem**: the v0.1 lowering gaps are closed (align/self/wrap/display,
  gap axes, auto margins, position/insets/z-index, aspect ratio, cursor,
  pointer events, typography, transforms, ring→outline, per-set
  transitions) plus a reactive `tw_plan` path for everything native
  selectors cannot express.
- **Masonry/Xilem**: `TwEnv` + `plan.resolve(&probe)` in the fallback
  widget (dark, responsive, container, group/peer), margins, per-side
  borders, per-corner radii, ring/outline stroking, multi-layer shadows.
- **MSRV split (F9)**: core/macro verified on real Rust **1.88**; shadcn +
  backends declare **1.96**; CI runs both gates.

<details><summary><b>What was in v0.1</b></summary>

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

</details>

**Deferred to 0.4 — all upstream-blocked (tracked in `AEOLUS_V0.3.md` §4
and the CHANGELOG):** publishing the backends (needs upstream crates.io
releases), view-side property-stack registration (upstream gap #4 —
retires the fallback widget), Masonry typography via native properties
(upstream patch proposal #3), and transitions/animations on the Masonry
backend (Floem has per-set transitions; Masonry drops them with
warnings).

The full capability matrix per framework lives in
[`reports/02-capability-matrix/CAPABILITY_MATRIX.md`]; the honest-limitation
contract (every drop is reported, never silent) in the design report; the
v0.3 plan of record in [`AEOLUS_V0.3.md`].

## Building

MSRV (v0.2, F9 fix): `aeolus-core`/`aeolus-macro` build on stable Rust
**1.88**; `aeolus-shadcn` and the backend crates (`aeolus-xilem`,
`aeolus-floem`) need **1.96+** (Masonry 0.4's `rustc` requirement). No
sibling checkouts needed — backend framework deps resolve from pinned git
revs.

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
