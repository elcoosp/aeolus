# 16 — The P2 dyn-cache seam (RED→GREEN evidence)

Milestone: the pipeline's sixth slice — architecture plan §3.2's
`DynCache` ("P2's DynCache dedupes the pool later"), the leak-pool
dedupe the compile seam's storage note promised. Branch:
`feat/dyn-cache-seam`.

## Scope

Two milestones, one full red→green cycle each:

* **M1 (feat)** — `DynCache` + the `DynStore` storage seam + the
  built-in `TwoGenStore` (two-generation rotation with promotion), over
  the plain `compile`: content-keyed (never hash-only), config-factored
  keys, bounded resident map, thread-safe, errors never cached.
* **M2 (feat)** — the themed and customized session entry points on the
  same cache: `compile_theme`/`compile_with`/`compile_theme_with`, the
  theme table and registrations joining the identity (exact-equality
  addresses — the v0.x `(ptr, len)` precedent; empty registrations key
  as zero, their byte-identity to the plain entry points being the
  custom seam's differential contract).

Both are net-new capability on the crate's existing conventions: typed
errors (the same `CompileError`), opt-in `tracing` spans on the
hit/miss/store decisions, and the custom seam's trait shape (the fold
generic over `ClassifyVariant`, the cache generic over `DynStore`).

## The cycle

1. The suite `crates/aeolus-compile/tests/dyn_cache.rs` was authored
   FIRST as the RED spec — 9 tests (M1) covering the hit's
   pointer-identical interned slices, the free-compile differential
   battery, the config-fingerprint miss, the 200-string no-collision
   walk, the bounded resident set, the previous-generation promotion,
   the 4-thread shared cache, the host-implemented `DynStore` seam, and
   the errors-never-cached contract. RED captured:
   `error[E0432]: unresolved imports aeolus_compile::DynCache,
   aeolus_compile::DynStore, aeolus_compile::CachedCompiled` — the API
   surface did not exist.
2. The implementation landed second: `src/dyn_cache.rs` (the seam
   module: `DynIdentity`, `CachedCompiled`, `DynStore`, `TwoGenStore`,
   `DynCache`) and the `lib.rs` wire-up (module + re-exports + the
   storage note fulfilled). GREEN: 9/9.
3. M2's RED: four session tests appended (themed hit, cross-table
   slots, customized session, the theme+custom session) — RED captured:
   `error[E0599]: no method named 'compile_theme' found for struct
   'DynCache<S>'`. GREEN: 13/13.

One mid-cycle correction is part of the record: the first flip-flop
pin exposed that one-key-per-string thrashes under alternating configs
(the second config overwrote the first's entry), so the key became the
IDENTITY PREFIX composed with the string — configurations (and, in M2,
tables and bags) own separate resident slots. The promotion pin's
counter expectation was corrected with it (two cold lowers + one
promoted hit = 2 misses, not 1).

## Commands and results

```sh
cargo test -p aeolus-compile --test dyn_cache
#   RED  (M1): error[E0432] unresolved imports DynCache/DynStore/CachedCompiled
#   GREEN(M1): test result: ok. 9 passed; 0 failed
#   RED  (M2): error[E0599] no method named `compile_theme` found for struct `DynCache<S>`
#   GREEN(M2): test result: ok. 13 passed; 0 failed
cargo bench -p aeolus-compile --bench dyn_cache   # numbers in reports/08
cargo run -p aeolus-compile --example dyn_cache   # 60 frames: hits=60 misses=1, same interned slices
cargo test --workspace --all-features             # green end to end
```

## Bench evidence (medians, 2-core sandbox, release profile)

| benchmark | median | ratio |
|---|---|---|
| `compile_uncached_button` | 21.1 µs | 1.0× |
| `dyn_cache_hit_warm_button` | 985 ns | **21× faster** |
| `compile_uncached_single_utility` | 546 ns | 1.0× |
| `dyn_cache_hit_warm_single_utility` | 251 ns | 2.2× faster |
| `dyn_cache_hit_warm_themed` | 261 ns | the theme resolve skipped too |
| `dyn_cache_miss_cold_button` | 25.0 µs | the honest first-call cost (probe + pipeline + store) |

## Docs drift + micro-debt fixed alongside (additive)

* `src/lib.rs`: the crate header still listed `--value(color)` as "the
  next custom-seam slice" (it landed as `CustomForm::Token`) and the
  `UnsupportedValue` help still claimed "the value half is the next P2
  slice" (the value pass landed; the remaining boundary is the
  arbitrary-value unit grammar) — both aligned, and the storage note
  now names the landed `DynCache`.
* `src/custom.rs`: the module header's NOT-list still deferred
  `--value(color)` — aligned to the landed `CustomForm::Token`.
* `README.md`: the crate table and status gained the dyn-cache surface;
  the example list gained the `css_emit`/`custom_seam` lines it was
  missing.
* `docs/AEOLUS_V1_ARCHITECTURE.md`: the P2 status and the P2 table row
  mark slice 6 landed.
