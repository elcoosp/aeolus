# Benchmarks — hot-path report (deliverable 8)

Run: `cargo bench -p aeolus-core` (divan 0.1.21), release profile,
2-core shared CI sandbox (Linux). Source: `crates/aeolus-core/benches/hot.rs`.
Date: 2026-11-03, re-run 2026-11-04 on the v0.1.0 release branch (below).
Treat absolute numbers as environment-specific; the ratios are the point.

The benchmarked button string is 33 classes (the full shadcn Button base
plus variant utilities):

```
inline-flex shrink-0 items-center justify-center gap-2 rounded-md text-sm
font-medium whitespace-nowrap outline-none focus-visible:ring-[3px]
focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50
bg-primary text-primary-foreground hover:bg-primary/90
```

## Results (median of 100 samples)

| benchmark | median | notes |
|---|---|---|
| `parse_button` (33 classes) | **1.26 µs** | lexer+parser only, no theme |
| `resolve_button_cold` | **4.64 µs** (3.7 Mitem/s) | full parse + theme + conflict + plan build |
| `resolve_cached_hit` | **405 ns** | warm steady state = one hash lookup + plan clone |
| `diff_unchanged` | **520 ns** | full `ResolvedStyle` diff (all slots equal) |
| `intern_hit` | 386 ns | string interner hit path |
| `cn_button_merge` (34 classes, conflicts) | 3.34 µs | conflict trie over base+override |
| `cn_no_conflicts` (6 classes) | 1.49 µs | |
| `color_mix_opacity` | 0.22 ns | oklab premultiplied mix (scalar) |
| `theme_parse_shadcn` | 201 µs | one-time; ~270 tokens incl. default palette |

## v0.1.0 re-run (2026-11-04, same sandbox, release profile)

After the Phase-2/4 correctness work (calc precedence, approximate color
equality, bounded caches, fuzz fixes) the hot path is UNCHANGED:

| benchmark | v0.1.0 median | vs 2026-11-03 | note |
|---|---|---|---|
| `resolve_cached_hit` | **405 ns** | = | two-generation lookup added zero overhead |
| `resolve_button_cold` | **4.6 µs** (3.7 Mitem/s) | = | shunting-yard calc only runs at theme parse |
| `diff_unchanged` | **505 ns** | ≈4% faster | approximate `Color` eq short-circuits on tolerance |
| `intern_hit` | **386 ns** | = | |
| `parse_button` | 1.23 µs | ≈ | |
| `cn_button_merge` | 3.2 µs | ≈ | new conflict families classify in the same match |
| `theme_parse_shadcn` | 231 µs | +15% | accepted: calc() precedence + ASCII/parity checks are cold-path only, paid once per theme load |
| `color_mix_opacity` | 0.22 ns | = | |

Verdict: the production-hardening work did not regress the hot paths the
brief pins (§3.6); theme parsing grew but stays a one-time cost.

## Steady-state claims (per mission brief §3.6)

- **Static path (`tw!`)**: parse + resolve + conflict all happen at compile
  time; runtime is a cache lookup → measured **~405 ns** warm, with zero
  parsing and zero theme work on the path.
- **Dynamic path**: after warmup the `PlanCache` returns the identical
  `Resolution` (same ~405 ns class); the cold 4.6 µs resolve is paid once per
  distinct class string per theme version.
- **Rebuild path**: `diff(prev, next)` is 520 ns for a fully-populated
  33-class plan and returns `StyleDiff::changed` (empty here) — slot-level
  lanes (`Slot::lane()`) decide cosmetic (pre-paint) vs geometric
  (layout) invalidation, mirroring Masonry's own
  `core_property_changed` lanes.
- **Theme swap**: explicit (swap token map, bump version, drop cache) —
  the 201 µs parse is per theme load, not per resolve.

## Allocation note

divan's default output here does not surface per-benchmark allocation
counts; the structural claims hold from the code: the cached-hit path clones
one `Resolution` (plan `Vec`s), the diff path allocates only when slots
actually changed (`Vec<Slot>` empty case = no alloc), and `cn()` allocates
one output `String`. A follow-up should wire divan's alloc counters
(`#[divan::bench(counters = [AllocCount])]`) once the tracker compiles
cleanly in this toolchain.

## v0.4 re-run (2026-11-07, 1-core sandbox, release profile, divan 0.1.21)

After the verification emitter landed (`AEOLUS_V0.4.md` §4). The hot
paths are unchanged in shape; the sandbox is slower than the 2026-11-03
2-core shared runner, so compare RATIOS within a run, not across runs.

| benchmark | v0.4 median | note |
|---|---|---|
| `resolve_cached_hit` | **78.6 ns** | steady state unchanged |
| `diff_unchanged` | **238.8 ns** | |
| `resolve_button_cold` | **5.76 µs** (2.9 Mitem/s) | 3 corpus-caught resolver fixes (negative insets/translations, full-radius) did not regress the path |
| `parse_button` | 3.25 µs | |
| `intern_hit` | 54.8 ns | |
| `cn_button_merge` | 4.58 µs | |
| `cn_no_conflicts` | 2.41 µs | |
| `theme_parse_shadcn` | 99.0 µs | one-time |
| `color_mix_opacity` | 0.185 ns | |
| **`css_write_button`** (new) | **13.4 µs** | the 33-class Button through the verification emitter: parse + resolve + route + 33 v4-shaped rules. Verification tooling, NOT on the styling hot path (backends consume the plan) |

## v0.5 re-run (2026-11-07, 2-core sandbox, release profile, divan 0.1.21)

After the element-relative typography and the allocation fix
(`AEOLUS_V0.5.md` §1/§3). Same sandbox class as the v0.4 run; compare
RATIOS within a run.

| benchmark | v0.5 median | vs v0.4 | note |
|---|---|---|---|
| `diff_unchanged` | **139.8 ns** | **−42%** | the zero-alloc diff (v0.5 §3) removed two malloc/free pairs per call |
| `resolve_cached_hit` | 76.8 ns | = | |
| `resolve_button_cold` | 5.79 µs (2.9 Mitem/s) | = | typography tokens resolve to stored constants — no added work |
| `parse_button` | 3.27 µs | = | |
| `intern_hit` | 53.4 ns | = | |
| `cn_button_merge` | 4.33 µs | = | |
| `cn_no_conflicts` | 2.22 µs | = | |
| `theme_parse_shadcn` | 114.1 µs | ≈ | one-time |
| `color_mix_opacity` | 0.219 ns | = | |
| `css_write_button` | 13.4 µs | = | the leading/tracking arms emit the relative unit directly |

Allocation behavior per operation: see the v0.5 allocation table below
(`tests/alloc_contract.rs` — the report-08 follow-up closed there).

### The report-08 allocation-counter follow-up — CLOSED in v0.5

Re-tested per `AEOLUS_V0.4.md` §4.2: divan **0.1.21 still ships no
`AllocCount` counter** (`divan::counter::{ItemsCount}` only) — and the
re-verification against the vendored 0.1.21 source settles WHY: an
`IntoCounter` is a fixed per-iteration VALUE for throughput display,
not a sampled delta, so a global allocation counter cannot be expressed
through it. The path the original note named as the alternative —
"the project writes its own global allocator counting harness" — is what
v0.5 shipped (`AEOLUS_V0.5.md` §3): `tests/alloc_contract.rs`, a
counting `GlobalAlloc` + mutex-serialized measured sections, pinning
the claims as machine-independent numbers. The contract caught a real
miss on day one: `diff()` allocated on EVERY call (the slot walk routed
through `slots()`' scratch `Vec`), contradicting this report's
structural claim — fixed in v0.5 (one `for_each_present_slot!` source;
`diff` walks shared bits directly), which is also why the
`diff_unchanged` number above dropped 238.8 → 139.8 ns.

## v0.5 allocation table (rustc 1.99.1, x86_64-linux, debug profile)

From `tests/alloc_contract.rs` (exact pins where marked; bounds absorb
rustc-version churn):

| operation | allocations | bytes | contract |
|---|---|---|---|
| `diff_unchanged` (33-class Button) | **0** | **0** | pinned EXACTLY |
| `resolve_cached_hit` (warm) | **0** | **0** | pinned EXACTLY (an `Arc` refcount bump — the report's "plan clone" wording was wrong all along) |
| `intern_hit` (warm) | **0** | **0** | pinned EXACTLY |
| `diff_changed` (8 slots) | 2 | 24 | the changed-slot `Vec` growth |
| `cn_button_merge` (33 + 4 classes) | 61 | 46,218 | bounded ≤ 128 / ≤ 64 KiB |
| `resolve_button_cold` | 82 | 44,112 | bounded ≤ 160 / ≤ 64 KiB |
| `theme_parse_shadcn` (~270 tokens) | 1,953 | 170,301 | one-time per theme load |

The zero-alloc steady state is the §3.6 claim, now measured: the static
path (`tw!`) and the dynamic warm path allocate NOTHING per resolve;
the rebuild path allocates nothing when nothing changed and one `Vec`
growth when slots actually changed.

## v0.7 run (2026-11-08, 2-core sandbox, release profile, divan 0.1.21)

After the variant & grid engine (`AEOLUS_V0.7.md` §8): pseudo-element
variants, has-attr descendants, named containers, arbitrary grid
templates, implicit tracks, justify-items. Same sandbox class as the
v0.5 run; compare RATIOS within a run.

| benchmark | v0.7 median | vs v0.5 | note |
|---|---|---|---|
| `diff_unchanged` | 154.8 ns | ≈ | zero-alloc diff untouched |
| `resolve_cached_hit` | 79.0 ns | = | |
| `resolve_button_cold` | 6.19 µs (2.7 Mitem/s) | ≈ | the Button string exercises none of the new surfaces |
| `parse_button` | 3.32 µs | = | |
| `intern_hit` | 53.5 ns | = | |
| `cn_button_merge` | 4.31 µs | = | the new conflict groups classify in the same match |
| `cn_no_conflicts` | 2.19 µs | = | |
| `theme_parse_shadcn` | 104.9 µs | ≈ | one-time |
| `color_mix_opacity` | 0.218 ns | = | |
| `css_write_button` | 13.2 µs | = | |
| **`resolve_input_cold`** (new) | **11.5 µs** (2.4 Mitem/s) | — | the 32-class Input table: pseudo-element variants (`selection:`/`placeholder:`/`file:` ×10), responsive, dark |
| **`resolve_card_header_cold`** (new) | **3.35 µs** (2.7 Mitem/s) | — | named container + arbitrary row template + auto-rows + conditional grid-cols |

Verdict: the variant-engine work did not regress the pinned hot paths
(the Button string's numbers are unchanged in shape), and the NEW
surfaces resolve at the same per-class throughput as the old ones
(~2.4–2.7 Mitem/s cold). The allocation contract's exact-zero pins
(diff, intern hit, warm cache hit) held unchanged
(`tests/alloc_contract.rs`, part of the 340-test workspace suite).

## Test-pyramid run (2026-11-08, 2-core sandbox, bench profile, divan 0.1.21)

The artifact path got its own bench file (`benches/snapshot.rs`, the
test-pyramid milestone): the snapshot writer/reader and the plan-level
overlay on a RICH plan — the Button table with its state layers plus the
v0.7 shapes (`selection:`, `has-checked:`, `@container/card-header`,
`md:p-8`, `dark:`). The reader is a hand-rolled strict parser; it reads
the whole artifact and reconstructs every slot.

| benchmark | median | note |
|---|---|---|
| `snapshot_write_rich` | 15.7 µs | fixed-schema-order writer, shortest-float form |
| `snapshot_read_rich` | 422 µs | strict reader: per-field paths, per-string allocs, typed errors |
| `snapshot_roundtrip_rich` | 433 µs | write + read — the artifact contract |
| `overlay_rich` | 649 ns | plan-level later-wins composition (per-slot merge) |
| `diff_text_only_change` | 243 ns | the regression path: a non-first text property change (per-property bits now reach it) |

Honesty notes: (1) the reader is ~27× the writer — it is a cold,
correctness-first path (artifact import, tests, tooling), not a per-frame
path; the hot path stays `resolve_cached_hit` + `diff` (both pinned
zero-alloc in `tests/alloc_contract.rs`). (2) The bench profile is the
same one `cargo bench` drives; treat numbers as relative, not absolute.
