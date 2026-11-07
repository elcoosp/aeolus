# Aeolus v0.2 Plan (plan of record)

This is the plan the v0.2.0 release implemented. It is the document every
`plan §X.Y` citation in the code and reports refers to. Status markers are
per the 0.2.0 changelog; the quality gates (tests, clippy `-D warnings`,
docs `-D warnings`, fuzz targets) were green at release.

Goal: turn the v0.1 research prototype into a **serious Tailwind v4
compiler** — close the silent-mis-style bug class, move ALL variant
semantics into the core behind a backend-agnostic evaluator, ship the full
v4 default theme, cover the Shadcn v4 utility surface with typed slots or
precise reports, and make the hot path cache-driven — without breaking the
v0.1 public API (additive only).

## §0 — Correctness: the F-bugs (silent mis-styles)

Every fix lands with a RED-then-GREEN test.

| id | bug | fix |
|---|---|---|
| F1 | `w-1/2` resolved as `4px` (modifier read as a spacing value) | the raw `/modifier` is the fraction denominator for sizing utilities → `SizedValue::Fraction(0.5)` |
| F2 | `p-[7px]` resolved as `28px` (arbitrary values treated as spacing multiples) | arbitrary lengths are absolute px; `(--var)`/`[var(--x)]` resolve through the theme |
| F3 | `dark:hover:*` dropped the `dark` half (hover applied in LIGHT mode) | compound variants land in `rules` as `[Media(Dark), On(Hover)]` |
| F4 | `md:hover:*` — same class of bug for breakpoint compounds | same routing fix |
| F5 | `rounded-s-lg` painted all four corners | logical start/end edges map to the correct corners |
| F6 | `text-sm/6` ignored the line-height modifier | `text-sm/6` = 6 × spacing; `text-sm/tight` = `--leading-tight` |
| F7 | `bg-primary/[0.37]` rejected; `(--o)` unhandled | bracketed decimals are raw alpha fractions; var modifiers resolve via the theme |
| F9 | single MSRV blocked the backends | split: core/macro on 1.88, backends on 1.96 |

Additional correctness contracts: exact `cn()` variant keys (§1.3),
collision-proof string-keyed plan cache (§3.4.4), multi-layer shadow
parsing (§3.3), `@md` = `--container-md` (§3.3).

## §1 — Grammar (parser.rs, ast.rs)

- §1.1 No-alloc token shape: `ClassToken::variants` is a stack-inlined
  `Inline<Variant, 8>`; `elements` a `Segments<'a, 10>`; the raw
  `body`/`modifier` slices replace `join("-")`.
- §1.2 The `/modifier` is kept RAW (`"2"`, `"[0.37]"`, `"6"`, `"(--o)"`) —
  interpretation belongs to the utility, not the parser (fixes F1/F6/F7 at
  the source).
- §1.3 `cn()` merge keys use the EXACT canonical variant prefix —
  `hover:` vs `group-hover:` and `md:` vs `@md:` never collide.
- §1.4 Full v4 variant grammar: states, media, breakpoints, `min-*`/`max-*`,
  `@container` forms, `group-*`/`peer-*` (labels), `has-*`, `not-*`
  (recursive), `data-*`, `aria-*`, structural (`first`…`nth-*`),
  pseudo-elements, arbitrary `[&…]`. Unknown names parse as
  `NamedVariant::Custom` at runtime; the `tw!` macro keeps the strict
  compile-time check.

## §2 — StylePlan IR (plan.rs)

- §2.1 Routing: a token with EXACTLY one classic variant keeps going to the
  v0.1 bucket (base / layer / responsive / dark / group / peer);
  **everything else** lands in `StylePlan::rules` as a `Cond` conjunction —
  nothing is dropped, nothing collapses into the wrong bucket.
- §2.2 The parity engine (eval.rs): backends answer questions via `Probe`
  (test/related/descendant_state/media/viewport_width/container_width);
  `StylePlan::resolve` flattens the plan for one situation with candidate
  order = cascade order (states < group/peer < container < viewport < media
  < print; stable ties keep source order; state layers rank by the
  canonical v4 condition order). `resolve_rules_only` exposes the emulated
  lane for backends that lower the rest natively.

## §3 — Theme registry & utilities (theme.rs, resolve.rs)

- §3.1 Parser-side raw modifiers (see §1.2); the resolver interprets per
  utility family.
- §3.2 Utility coverage: every v4 utility family resolves to a typed slot
  or a precise `Unsupported` — `Unknown` fails CI (the coverage-guard
  test). Adds `mx-auto`, `max-w-{container}`, viewport sizes, `gap-x/y`,
  `space-*` (reported — gap is the supported equivalent), `basis-*`,
  `order-*`, `content-*`, grid cols/rows/spans/starts, `aspect-*`,
  positioning, `inset-*` (logical edges), `z-*`, `overflow-*`,
  `visible/invisible`, the typography set (`italic`…`line-clamp`,
  `font-sans`, `tabular-nums`, `text-ellipsis`), `transition-*`/
  `duration-*`/`ease-*`/`delay-*`, `animate-*`, transforms, `shadow-{color}`,
  `inset-shadow-*`, `ring-inset`, `outline-offset-*`, `border-{style}`,
  logical `border-s/e`, the full `cursor-*` set, `select-*`.
- §3.3 Theme: the FULL v4 default theme ships in every theme (diffed
  against tailwindcss v4.1.17) — `--container-*`, multi-layer `--shadow-*`,
  `--text-*` + paired line heights, weights, tracking, leading, radii,
  easings, animations, default transition duration. `@utility` expansion
  (depth ≤ 4) and `@custom-variant` name→meaning extraction. `calc()` with
  real operator precedence; deferred resolution is ORDER-INDEPENDENT (an
  `@theme inline` var chain may precede its `:root` source; calc→calc
  chains resolve to a fixpoint — unresolvable references error precisely).
- §3.4 Hot path: (§3.4.1) slice, never join; (§3.4.2) stack-buffer key
  building; (§3.4.3) 8-bytes-per-round `FastHasher` with length-tagged tail
  + avalanche; (§3.4.4) string-keyed two-tier cache (per-thread L1 + shared
  L2, version-stamped, previous-generation hits promoted) — a hash
  collision can never return the wrong plan; (§3.4.5) O(1) bitmask
  `diff()`; (§3.4.6) `(ptr,len)`-keyed `resolve_static` for `&'static str`
  call sites.
- §3.5 Dark-variant derivation re-resolves `@theme inline` var chains
  against the `.dark` palette.

## §4 — Backends

- §4.1 Parity contract: every slot is NATIVE, EMULATED (with a mechanism),
  or REPORTED (`LoweringReport` + debug warnings) — never silently dropped.
  Capabilities negotiate via `BackendCapabilities`.
- §4.2 Floem: close the v0.1 lowering gaps (align/self/wrap/display, gap
  axes, auto margins, `absolute()`, insets, z-index, aspect-ratio, cursor,
  pointer-events, typography, transforms, ring→outline, per-set
  transitions) + the reactive `tw_plan` path (`FloemProbe` +
  `resolve_rules_only`) so compound variants, group/peer, data/aria and
  container queries work without an upstream patch. Theme-aware
  breakpoints replace `Breakpoint::default_px`.
- §4.3 Masonry/Xilem: `TwEnv` (dark, viewport/container width, group/peer
  scope via `WidgetMut::set_env`) feeds `plan.resolve(&probe)` in
  measure/layout/pre-paint; margins measured and inset; per-side borders
  and per-corner radii painted with Kurbo geometry; ring/outline stroked;
  multi-layer shadows rendered; opacity multiplied into the chrome colors
  (children-not-faded documented).

## §5 — Quality gates

- §5.1 `cargo test --workspace` green; `cargo clippy --workspace
  --all-targets -- -D warnings` clean; `RUSTDOCFLAGS="-D warnings" cargo
  doc` clean; `cargo fmt --check` clean.
- §5.2 Fuzz targets (`fuzz/`) run clean: `parse_class_string`,
  `color_parse`, `cn`, `theme_from_css`, `parse_variant`, `probe_eval`.
- §5.3 The coverage-guard test: every v4 utility family → typed slot or
  precise `Unsupported`; `Unknown` fails CI.
- §5.4 Property tests: `cn` idempotence, parser roundtrip, OKLab↔sRGB
  roundtrip, opacity hue preservation.
- §5.5 Publishing gates: `cargo package -p aeolus-core -p aeolus-macro`
  (backends stay `publish = false` — git-pinned deps).
- §5.6 Macro contract: `tw!` rejects unknown variant NAMES at compile time
  (typo check) while the runtime parser accepts them for
  `@custom-variant` resolution.

## §6 — Performance budgets (1-core sandbox, report 08)

Cached hit ≤ 100 ns (measured 76 ns), unchanged diff ≤ 300 ns (measured
235 ns), cold resolve within 1.5× of the v0.1 baseline. The parser may
cost ~1 µs more per Button string — the richer v4 grammar, documented
honestly.

## §7 — The honesty contract (no silent failures)

Every drop is reported: `Warning::Unsupported` with a specific reason,
`LoweringReport` entries on the backends, debug-build warnings for
capability-negotiated slots. A class is never silently mis-styled, a
condition never silently ignored, a value shape never silently coerced.
