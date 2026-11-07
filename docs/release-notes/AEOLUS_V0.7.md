# Aeolus v0.7 Plan (plan of record)

The v0.6 plan (`AEOLUS_V0.6.md`) shipped in 0.6.0; the Slint backend
landed on top of it (the `Unreleased` section of `CHANGELOG.md`). This
document is the v0.7 plan of record; every `v0.7 §X` citation in code and
reports refers here.

Goal: **oracle fidelity — the shipped shadcn v4 class tables resolve
warning-free.** A 2026-11-08 fidelity audit resolved every shipped table
string (Button, Input, Label, Card family, Badge, Separator, Skeleton,
Alert) against the v0.6 resolver and counted **40 warnings**. Every one
was a parser/IR gap, not a documented boundary:

| gap | count | example from the tables |
|---|---|---|
| pseudo-element variants unknown to the parser | 10 | `selection:bg-primary`, `placeholder:text-muted-foreground`, `file:text-sm` |
| arbitrary grid templates outside the typed IR | 4 | `grid-cols-[0_1fr]`, `grid-rows-[auto_auto]`, `auto-rows-min` |
| `has-data-[…]` routing missing | 1 | `has-data-[slot=card-action]:grid-cols-[1fr_auto]` |
| named container declarations | 1 | `@container/card-header` |
| `justify-items-*` unapplied | 1 | `justify-items-start` (cn() classified it; the resolver had no arm) |

v0.7 closes all five. After v0.7 the same tables produce **13 warnings,
all of them the documented v0.3 §1.3 Probe-model boundary** — arbitrary
selectors that style OTHER elements (`[&_svg]`, `[&>svg]`,
`*:data-[slot=…]`) and class-identity self-tests (`[.border-b]`), plus
the `has-[>svg]` combinator form. The headline contract is pinned by
`crates/aeolus-shadcn/tests/oracle_fidelity.rs`.

No backend-manifest changes, no MSRV bump, no upstream changes
(both pins remain: floem `1351ffb` = `main` HEAD, xilem `dc49687`).

## §1 — Pseudo-element variants (`selection:`, `placeholder:`, `file:`)

v4 names three pseudo-element renderings as plain variants. They are NOT
runtime states — `selection:` styles the element's own selected-text
rendering, `placeholder:` the placeholder rendering, `file:` the
`<input type=file>` button part. The typed model keeps them
distinguishable from `State`s:

- `ast::PseudoElement { Selection, Placeholder, FileSelectorButton }`
  with `name()` (merge keys) and `css()` (`::selection`, ...).
- `NamedVariant::Pseudo(PseudoElement)` — the parser table gains the
  three names (they were `Custom` before, i.e. compile errors for `tw!`
  and runtime warnings).
- `Test::Pseudo(PseudoElement)` — a new `cond::Test` arm ranking with the
  state band (`Atom::order` = 10, the state band — v4 sorts them in the
  same variant order).
- Routing: single-`Pseudo` conditions land in `rules` (they are not
  classic bucket states), evaluated by `Probe::test(&Test::Pseudo(…))`.
  The DEFAULT probe answer is "no" — the rule never fires where the
  rendering does not exist, exactly the semantics CSS has when a selector
  does not match. A backend with the surface (text-selection colors on a
  text-input wrapper) answers honestly and the rule applies.
- `cn()`: the canonical name rides the existing variant-key machinery —
  `selection:bg-a selection:bg-b` merges correctly with zero new table
  entries.

Semantics pinned against the REAL tailwindcss 4.1.17 output (captured
2026-11-08 with the `tools/gen_golden.mjs` method): `selection:bg-red-500`
emits BOTH `&::selection` (self) and `& *::selection` (descendants). The
self half is corpus-pinned; the descendant half is outside the Probe
model (v0.3 §1.3) and is a DOCUMENTED NORMALIZATION in the parity harness
(§6).

## §2 — `has-data-[…]` / `has-aria-[…]` / `has-<structural>`

`Atom::Has` generalizes from `Has(State)` to `Has(Test)`: the inner of a
`has-*` wrapper is parsed as a variant and mapped through the existing
`variant_test` — so `has-data-[slot=card-action]` becomes
`Has(Test::Data("slot", Some("card-action")))`, `has-aria-invalid`
becomes `Has(Test::Aria("invalid", None))`, `has-first` becomes
`Has(Test::Structural(First))`, and the state forms are unchanged.

- `Probe::descendant(&Test)` (new, default impl) answers the general
  form; it delegates to the existing `descendant_state` for state tests,
  so v0.x trait implementations keep compiling. The arbitrary-selector
  classifier (`selector.rs`) maps `:has(...)` inners onto the same typed
  set (previously states only).
- Combinator selectors (`has-[>svg]`) KEEP their precise report — the
  child-edge is real information the typed descendant question does not
  carry, and the shadcn tables' `has-[>svg]:px-3` stays honestly
  reported.
- The emitter reproduces v4's star form exactly (captured):
  `:has(*[data-slot="card-action"])`, `:not(*[data-state="open"])`.

## §3 — Named container declarations (`@container[/name]`)

`@container` (bare) and `@container/name` are UTILITY classes in v4:
they establish an inline-size containment context (the named form adds
`container-name`). The IR gains `Extras.container_type:
Option<ContainerType { name: Option<Box<str>> }>` (slot
`Slot::ContainerType`, lane Geometric — containment constrains layout);
the parser routes the class through the `/modifier` slot the body split
already produces. The emitter emits `container-type: inline-size` +
`container-name` when named — byte-matching the captured v4 output.
`cn()` classifies the family as one `container-type` group (later wins).
CONDITIONS (`@md/name:`) already carried names since v0.2 (`Atom::
Container { name }`); nothing changes for them.

## §4 — Arbitrary grid templates + implicit tracks

`Grid` gains the CANONICAL template slots (the numeric `cols`/`rows`
counts stay for v0.6 consumers; backends MUST prefer templates when
present — the numeric form is v4's `repeat(N, minmax(0, 1fr))`
shorthand):

```rust
pub struct GridTrack { pub min: TrackSizing, pub max: TrackSizing } // minmax() pair
pub enum TrackSizing { Px(f32), Percent(f32), Fr(f32), Auto, MinContent, MaxContent }
pub enum TrackComponent { Track(GridTrack), Repeat { count: u16, track: GridTrack } }
// Grid: template_cols/template_rows/auto_rows/auto_cols: Option<Vec<TrackComponent>>
```

- Parser semantics (bounded: ≤ 16 tracks, repeat count ≤ 64):
  `grid-cols-[0_1fr]` → `[0px, minmax(auto, 1fr)]` (a bare `1fr` is the
  CSS shorthand `minmax(auto, 1fr)`; bare unitless lengths must be `0`);
  `auto_auto` → keywords; `minmax(a,b)` pairs; `repeat(N, <single
  track>)` stays a bounded component (1:1 with taffy's
  `GridTemplateComponent::Repeat`); junk tracks report precisely.
  `auto-rows-min|max|fr|auto` map to min-content/max-content/
  **minmax(0, 1fr)**/auto — the `fr` KEYWORD is the blowout-guard
  `minmax(0, 1fr)`, NOT the `1fr` shorthand (pinned against the real
  compiler after the first capture disagreed with CSS intuition);
  `auto-cols-*` symmetric; arbitrary templates accepted for both.
- Floem lowers all of it NATIVELY at the pin (taffy 0.10 surface:
  `grid_template_columns/rows` take `Vec<GridTemplateComponent>` with
  `Single(TrackSizingFunction)`/`Repeat(GridTemplateRepetition)`, and
  `grid_auto_rows/columns` take plain `Vec<TrackSizingFunction>`).
  Templates are canonical in the lowering: `grid-cols-4
  grid-cols-[auto_auto]` lowers the two auto tracks.
- `cn()`: the template families become conflict groups
  (`grid-template-columns`/`rows`, `grid-auto-rows`/`columns`) — the
  later template evicts the earlier ACROSS the numeric/arbitrary
  boundary (`grid-cols-3` + `grid-cols-[0_1fr]` keeps only the
  arbitrary). Rows/cols/placement never collide.
- Emitter: templates emit v4's verbatim serializer (`0 1fr`,
  `repeat(3,minmax(0,1fr))` — no space after commas, the raw `0`), the
  keyword forms emit pretty (`minmax(0, 1fr)`); the parity harness
  normalizes comma spacing (§6).

## §5 — `justify-items-*`

`Layout.justify_items: Option<AlignItems>` (slot `Layout`, lane
Geometric). Values: start/center/end/stretch/baseline,
`first|last-baseline` → baseline, and `normal` → stretch (CSS resolves
`normal` to stretch for grid/flex containers, same choice as
`content-normal`). Floem lowers to the native taffy `justify_items`
(present at the pin); Masonry/egui/iced/slint report (no property).
`cn()` already classified the family (a v0.2-era fix); only the
resolver arm was missing.

## §6 — Emitter + golden corpus (193 entries)

- New selector shapes, all pinned against the real 4.1.17:
  `&::selection` / `&::placeholder` / `&::file-selector-button` suffixes,
  `:has(*[data-slot="…"])` star forms for `has-data-*`/`has-aria-*`
  (and the matching `:not(*…)`), the container declarations.
- New declaration shapes: `container-type`/`container-name`,
  `grid-template-columns/rows` from tracks, `grid-auto-rows/columns`,
  `justify-items`.
- Corpus: 176 → **193 entries** — the v0.6 §4.3 gradient exclusion
  precedent stays (cross-rule `--tw-gradient-*` composition remains a
  harness model boundary), but every v0.7 class shape is single-rule and
  IN: pseudo-element variants (self half), has-attr descendants, named
  containers, arbitrary templates, implicit tracks, justify-items, the
  compound `md:selection:bg-*`. One new documented normalization:
  v4's `selection:` descendant rule (`& *::selection`) is skipped
  (Probe-model boundary), and comma spacing normalizes (serializer, not
  semantics).

## §7 — The oracle-fidelity contract (the release's own test)

`crates/aeolus-shadcn/tests/oracle_fidelity.rs`: every shipped table
string AND every component composition (base + variant + size + user
classes) resolves with warnings ONLY from the documented boundary set
(element-targeting selectors, class-identity tests, `has-[>svg]`). The
ledger: **40 → 13, all 13 boundary.** `new_rules_carry_the_requested_styles`
additionally asserts the styles the tables asked for actually landed
(the Input's selection/placeholder colors as rules, the Card header's
row template, auto-rows, container name).

## §8 — Benchmarks

divan, release profile, Linux sandbox (2-core shared — compare RATIOS
within a run, not across runs). The hot paths are UNCHANGED in shape:

| benchmark | v0.7 median | vs v0.5 |
|---|---|---|
| `resolve_cached_hit` | 79.0 ns | = (76.8 ns) |
| `diff_unchanged` | 154.8 ns | ≈ (139.8 ns) |
| `resolve_button_cold` | 6.19 µs (2.7 Mitem/s) | ≈ (5.79 µs) |
| `parse_button` | 3.32 µs | = |
| `intern_hit` | 53.5 ns | = |
| `cn_button_merge` | 4.31 µs | = |
| `theme_parse_shadcn` | 104.9 µs | ≈ |
| **`resolve_input_cold`** (new) | **11.5 µs** (2.4 Mitem/s) | the 32-class Input table incl. pseudo-element variants |
| **`resolve_card_header_cold`** (new) | **3.35 µs** (2.7 Mitem/s) | container + templates + has-data |

The allocation contract (`tests/alloc_contract.rs`) holds unchanged —
the exact-zero pins (diff, intern hit, warm cache hit) are untouched by
the new surfaces.

## §9 — Quality gates

`cargo test --workspace` (340 tests), `clippy -D warnings`, `fmt
--check`, `doc -D warnings`, `cargo package -p aeolus-core
-p aeolus-macro` all green. RED→GREEN evidence captured in
`reports/10-tdd-evidence/09-variant-engine.md`.

## §10 — Upstream-blocked (unchanged, re-documented)

The four deferrals stay: publishing the backends (no compatible
crates.io releases), view-side property-stack registration, native
Masonry typography, Masonry transitions — re-verified against the pins
(2026-11-08): floem `1351ffb` is still `main` HEAD; xilem still
`dc49687`.
