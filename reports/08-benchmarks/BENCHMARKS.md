# Benchmarks — hot-path report (deliverable 8)

Run: `cargo bench -p aeolus-core` (divan 0.1.21), release profile,
2-core shared CI sandbox (Linux). Source: `crates/aeolus-core/benches/hot.rs`.
Date: 2026-11-03. Treat absolute numbers as environment-specific; the ratios
are the point.

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
