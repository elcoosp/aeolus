# Aeolus v0.4 Plan (plan of record)

The v0.3 plan (`AEOLUS_V0.3.md`) shipped in 0.3.0. This document is the
v0.4 plan of record; every `v0.4 §X` citation in code and reports refers
here.

Goal: **prove the compiler end-to-end against the real thing.** The
design report (§7.4) and the source-audit Q&A (Q7) name the follow-up:
a golden-corpus parity test of class → CSS against genuine `tailwindcss`
v4.1.17 output. v0.4 ships that proof: the verification CSS emitter, the
corpus generator, and the parity harness — plus the benchmarks report's
follow-up (allocation counters on the hot-path benches). No new utility
families, no backend changes, no MSRV bump: v0.4 makes what already ships
*verifiable*, then measures it.

## §1 — The verification CSS emitter (`crates/aeolus-core/src/css.rs`)

- §1.1 `Resolver::to_css(classes) -> Result<Vec<CssRule>, …>` — one
  [`CssRule`] per class token, exactly v4's one-rule-per-candidate shape:
  `{ selector, media, container, declarations }`. The emitter reuses the
  resolver's own routing (`route`) and application (`apply_utility`) —
  the CSS is derived from the SAME resolution backends consume, so parity
  here pins the whole pipeline (parse → theme → plan).
- §1.2 Declarations map the resolved slots to the canonical v4 property
  names and value shapes: `SizedValue::Scale(n)` emits
  `calc(var(--spacing) * n)`, absolute px emit `…px`, opacity modifiers
  emit `color-mix(in oklab, <oklch> P%, transparent)` (v4's modern form),
  shadows/rings emit the composed `box-shadow` layers, per-edge families
  emit the per-edge properties the token names (`px-*` →
  `padding-inline`, `border-t` → `border-top-width`, …).
- §1.3 Selectors mirror v4's templates: escaped class name
  (`.hover\:bg-x\/90`), state suffixes (`:hover`, `:focus-visible`),
  `data-*`/`aria-*` attribute tests, `:not(...)`, `:has(...)`,
  structural pseudo-classes, group/peer descendant forms
  (`&:is(:where(.group):hover *)`), and at-rule wrappers for media
  (`@media (prefers-color-scheme: dark)`), breakpoints
  (`@media (width >= 48rem)`) and containers
  (`@container (width >= 28rem)`). The atom → CSS mapping lives beside
  the `Atom` definitions it mirrors (cond.rs); `Test::Tag` emits the
  typed rule but the selector kind has no v4 output to compare against
  and is excluded from the corpus.
- §1.4 Honest scope: the emitter is a VERIFICATION and debugging tool
  (what did the compiler understand?), not a styling target — backends
  still consume `StylePlan` directly. CSS it cannot express does not
  exist here: unsupported classes keep failing with the same warnings the
  resolver already produces, and `to_css` surfaces them.

## §2 — The golden corpus (`tools/gen_golden.mjs`, `tests/fixtures/golden/`)

- §2.1 `tools/gen_golden.mjs` generates the corpus with the REAL
  compiler: `tailwindcss` + `@tailwindcss/cli` **4.1.17** (the pinned
  audit version). Input per theme: one HTML soup of the corpus classes;
  output: the utilities-layer rules split per escaped class name, plus
  the `:root`/`@property` blocks needed to resolve `var()` references.
  The committed fixture (`crates/aeolus-core/tests/fixtures/golden/`)
  records the tailwind version; regenerating is `bun tools/gen_golden.mjs`.
- §2.2 Two theme contexts: `default` (the verbatim v4 default palette
  aeolus seeds) and `shadcn` (the neutral light theme, `@theme inline`
  var chains included — golden var() substitution must resolve
  transitively). Every corpus class must resolve in aeolus with ZERO
  warnings; the harness fails on any warning (the coverage-guard
  contract extended to the corpus).
- §2.3 Corpus scope: single-class entries spanning the supported surface
  — spacing/margin, sizing (fractions, arbitrary, viewport, fit),
  colors (palette, shadcn tokens, opacity modifiers, arbitrary hex,
  bracket alpha), borders/radii (incl. logical edges), ring/outline px
  semantics, multi-layer shadows + shadow colors, typography
  (sizes with paired line-heights, weights, styles, alignment),
  layout (flex/grid/gap axes/basis/order), position/inset/z/overflow/
  aspect, transforms, transitions (sets/duration/ease/delay), opacity,
  cursor/pointer-events/select; variants: the seven classic states,
  breakpoints (min/max), `dark:`, group/peer (incl. labels), `data-*`,
  `aria-*`, structural (`first:`/`last:`/`odd:`/…), `has-*`, `not-*`,
  container queries, arbitrary variants from the v0.3 typed subset
  (`[&:hover]`, `[@media(print)]`), compounds (`md:hover:*`,
  `dark:hover:*`), `!important` and negatives.
  Element-relative values (`tracking-*`, bare `leading-*`) stay OUT:
  the IR resolves them at the 16px base and the corpus only pins
  element-independent (px-exact) semantics — documented, not hidden.

## §3 — The parity harness (`crates/aeolus-core/tests/parity_golden.rs`)

- §3.1 Test-only normalizer, no new runtime dependencies: parse the
  golden block (recursive: nested rules, `@media`, `@container`,
  `@supports`), substitute `var()` from the golden `:root` + `@property`
  initial values to a fixpoint, evaluate `calc()`, convert rem→px
  (1rem = 16px), compose the `--tw-*` shadow/ring machinery into final
  `box-shadow` values, evaluate `color-mix(in oklab, X P%, transparent)`
  numerically, and parse both sides' colors with `Color::parse`
  (approximate ~1e-4 comparison per Oklab component).
- §3.2 Documented normalizations (differences that are model choices,
  not bugs): the `@media (hover: hover)` wrapper tailwind adds to
  `hover:` is dropped (the Probe model has no hover-capability concept);
  `border-style: var(--tw-border-style)` initial-value noise (`solid`)
  is dropped unless a style class sets it; unitless `line-height`
  ratios are multiplied by the `font-size` declared in the same rule;
  `calc(1/2 * 100%)` fractions compare as percentages.
- §3.3 Failure output is a diff (golden vs ours, per declaration) —
  the TDD loop for emitter work runs on it.

## §4 — Benchmarks (the report-08 follow-up)

- §4.1 `css_write_button` — emit CSS for the 33-class Button string;
  the parity emitter is verification tooling, its cost must be known.
- §4.2 Wire divan allocation counters (`AllocCount`) on the hot-path
  benches if the tracker compiles cleanly in the current toolchain
  (the report-08 note); otherwise re-test and document the outcome
  honestly.
- §4.3 Re-run `cargo bench -p aeolus-core`; record numbers + deltas in
  `reports/08-benchmarks/BENCHMARKS.md`.

## §5 — Quality gates

- §5.1 The full v0.3 battery stays green: `cargo test --workspace`,
  `clippy -D warnings`, `fmt --check`, `doc -D warnings`,
  `cargo package -p aeolus-core -p aeolus-macro`.
- §5.2 New tests: `tests/parity_golden.rs` (the corpus is the test
  data), plus unit tests in `css.rs` for the escaping and atom→CSS
  mapping corners the corpus cannot reach (`Test::Tag`).
- §5.3 No backend/manifest/MSRV changes; version → 0.4.0; docs: README
  status, CHANGELOG, DESIGN §9 (emitter + corpus), report 08 numbers,
  TDD evidence `reports/10-tdd-evidence/06-parity.md` (RED→GREEN
  captures per family batch).

## §6 — Upstream-blocked (stay deferred, re-documented)

Unchanged from v0.3 §4: publishing the backends, view-side property-
stack registration, Masonry typography via native properties,
Masonry transitions/animations.
