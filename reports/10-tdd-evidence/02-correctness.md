# Phase 2 — correctness bugs (TDD evidence, one entry per item)

Every item below: failing test FIRST, verbatim RED captured, then the fix,
then verbatim GREEN.

---

## 1. Floem `Length::Pt` vs `Length::Px` — verdict: Pt IS pixel-equivalent

Reading `repos/floem/src/style/unit.rs` @ 1351ffb settles the question the
mission posed: `Pt` is documented as "absolute length value in Floem layout
points … Floem treats absolute lengths as logical layout points rather than
device pixels", and `Length::to_taffy` passes the value through unscaled
(`Length::Pt(v) => LengthPercentage::length(v)`). So this is the mission's
second case: **add a comment AND a test that pins the behavior**.

Pinning test: `aeolus-floem/tests/lowering.rs::pt_is_logical_pixel_not_css_point`
— (a) the lowered `p-4` padding converts to `LengthPercentage::length(16.0)`
at taffy level; (b) a real taffy layout resolves 16px padding per side and a
32px content box in a 64px box (96dpi / scale-factor 1).

Mutation-verified RED (temporarily giving `pt()` CSS-point semantics
`px * 72/96` to prove the pin catches the regression, then reverted):

```text
test pt_is_logical_pixel_not_css_point ... FAILED
thread 'pt_is_logical_pixel_not_css_point' panicked at crates/aeolus-floem/tests/lowering.rs:176:5:
assertion `left == right` failed: p-4 must reach taffy as 16 logical px
  left: LengthPercentage(CompactLength(CompactLengthInner { tagged_ptr: 0x4140000000000001 }))
 right: LengthPercentage(CompactLength(CompactLengthInner { tagged_ptr: 0x4180000000000001 }))
test result: FAILED. 0 passed; 1 failed
```

GREEN (mutation reverted, doc comment added at the `pt()` boundary):

```text
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; (aeolus-floem)
```

Justification: a unit-semantics pin at the backend boundary is the cheapest
insurance against a floem upgrade silently rescaling every length.

## 2. `Color::PartialEq` on f32

RED (`color::tests::approximate_equality_ignores_float_noise`):

```text
assertion `left == right` failed: sRGB round-trip noise must not split colors
  left: Color { l: 0.637, a: 0.21421275, b: 0.10139973, alpha: 1.0 }
 right: Color { l: 0.637, a: 0.21421248, b: 0.10139984, alpha: 1.0 }
test result: FAILED. 0 passed; 1 failed
```

Fix: manual `impl PartialEq for Color` with per-component tolerance 1e-4
(below one 8-bit sRGB step; non-transitivity documented as the accepted
tradeoff for diffing/caching). GREEN: `26 passed` (core lib) — including the
companion test that real hue/alpha differences stay unequal.

## 3. Per-side border width silent drop

RED (`aeolus-xilem::per_side_border_width_is_reported_not_silently_dropped`),
run 1 (no report entry) and run 2 (a second bug surfaced: `border-t
border-b` RESET the top edge to 0 because `border_width_family` emitted
explicit `Some(0.0)` for non-target edges):

```text
thread 'per_side_border_width_is_reported_not_silently_dropped' panicked at crates/aeolus-xilem/tests/lowering.rs:181:5:
assertion `left == right` failed
  left: 0.0
 right: 1.0
```

Fix: (a) `border_width_family` emits `None` for non-target edges so edges
compose like padding/margin do; (b) `lower_resolved` adds
`report.drop("border-width:per-side", "Masonry BorderWidth is uniform; using
top edge")` whenever the four edges are not uniform. GREEN: `workspace 64
passed`.

Justification: honesty rule — a flattened feature must appear in the
`LoweringReport`, and per-side classes must compose exactly like every other
edge family.

## 4. `calc()` no operator precedence

RED (`theme::tests::calc_has_operator_precedence` — worse than suspected:
parens did not even tokenize):

```text
thread 'theme::tests::calc_has_operator_precedence' panicked:
called `Result::unwrap()` on an `Err` value:
ParseError { message: "bad calc token \"(4px\"" }
```

(and, before the paren test shape, the left-to-right accumulator would have
computed 28 instead of 24 for `4px + 10px * 2`.)

Fix: shunting-yard evaluator in `theme.rs` — tokenizer (px numbers, glued
parens, unary +/- folding, `0-(group)` rewrite for signed groups), RPN,
stack evaluation; same px/dimensionless unit rule as before. GREEN:

```text
test result: ok. 23 passed; 0 failed; ... (core lib at that commit)
```

Justification: precedence bugs are silent visual bugs; CSS `calc()` subset
with zero deps is ~90 lines and removes the documented limitation.

## 5. `justify-items-center` unclassified

RED (`conflict::tests::grid_alignment_families_merge_independently`):

```text
assertion `left == right` failed
  left: "justify-items-start justify-items-center"
 right: "justify-items-center"
```

Fix: classification arms for the families enumerable from the audited
tw_merge group table (precedents.md §3): `justify-items`, `justify-self`,
`align-content` (`content-*`), `place-content/items/self`, `flex-wrap`
(split from flex-direction), `flex-basis`, `order`, `position`,
`inset-x/y/inset`, `top/right/bottom/left`, `z-index`, `overflow-x/y/overflow`,
`overscroll-behavior` (+ parent→child knockouts), `aspect`, `columns`,
`visibility`, `isolation`, `resize`, `object-fit/object-position`,
`font-style`, `text-transform`, `text-decoration`, `vertical-align`,
`text-align`. GREEN: `24 passed` core lib + `14 passed` shadcn integration
(no regressions in shipped class tables).

## 6. `expect()` on RwLock in cache.rs

RED (`cache::tests::poisoned_lock_does_not_panic_the_caller`):

```text
thread 'poisoned_lock_does_not_panic_the_caller' panicked at crates/aeolus-core/src/cache.rs:146:13:
thread 'poisoned_lock_does_not_panic_the_caller' panicked at crates/aeolus-core/src/cache.rs:72:44:   <- resolve() panicking on the poisoned lock
test result: FAILED. 0 passed; 1 failed
```

Fix: all lock access routed through `read_map`/`write_map` using
`unwrap_or_else(|e| e.into_inner())` with a rustdoc justification (the
guarded maps hold plain resolved values; no cross-lock invariants — the
guard is consistent after a writer panic). GREEN: 29/29 core lib.

## 7. `Interner` unbounded

RED: `Interner::with_cap` did not exist —

```text
error[E0599]: no associated function or constant named `with_cap` found for
struct `cache::Interner` in the current scope
```

Fix: two-generation rotation (current fills to `cap` → becomes previous;
lookups consult both; ≤ 2×cap entries total; evicted strings re-intern to
fresh ids — documented). GREEN: `interner_evicts_beyond_cap` passes.

## 8. `PlanCache` unbounded

RED: same missing-`with_cap` compile error (item 7's capture covers it).

Fix: same two-generation rotation for `PlanCache`; warm previous-generation
entries still hit `Arc`-identically. `PlanCache::with_cap` /
`DEFAULT_CACHE_CAP = 4096` exported. LRU feature deliberately skipped to
keep aeolus-core zero-dep (tradeoff documented in the module docs).
GREEN: `plan_cache_evicts_beyond_cap` passes.

---

Phase totals: 9 new tests (1 pin + 2 color + 1 xilem + 1 calc + 1 conflict +
1 poison + 2 cache); workspace 62 → 71 passing; zero behavioral regressions
in the shipped Shadcn class tables.
