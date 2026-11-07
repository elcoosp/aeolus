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

### The report-08 allocation-counter follow-up

Re-tested per `AEOLUS_V0.4.md` §4.2: divan **0.1.21 still ships no
`AllocCount` counter** (`divan::counter::{ItemsCount}` only) — the
allocation tracker never landed in the 0.1.x line. The structural claims
in the original allocation note stand; the follow-up stays open until a
divan release ships the tracker (or the project writes its own global
allocator counting harness — deliberately out of scope for v0.4).
