# 37 — The LSP engine rides the dyn-cache

The P5 row's latency half: the editor's answers recompile every
candidate from scratch on every keystroke — the `analyze` walk, the
hover's single-candidate compile, the code-action fold and the shared
per-candidate walker all called the free `compile`/`compile_theme`
directly — while the compile crate's bounded pool (`DynCache`, report
16) existed precisely for the repeat-compile shape an editor session
lives in. The engine now holds ONE pool (the default `TwoGenStore`
bound): a private `compiled()` helper routes all four call sites
through it — the themed walk when the config carries a table, the
theme-less boundary otherwise, the same entry points the CLI runs,
never a parallel matcher — so a keystroke that leaves the strings alone
rides the pool instead of re-lowering, and a didChange misses only its
NEW strings. The identity is the engine's own config, fixed at
construction, so one identity slot spans the session (the table rides
the themed slot's own identity, the same `(config, table)` fingerprint
the compile seam keys on). The observability face joins the engine's
own constructor: `AeolusEngine::with_observer` chains after `new` and
wires the pool (the eviction face's store wire-up riding M1's
`attach_observer` composition), so one `CacheStats` handle totals the
whole editor session — and the face is NAMEABLE from the crate a host
actually consumes: `aeolus-lsp` re-exports
`DynObserver`/`CacheStats`/`CacheSnapshot`/`EvictReason`/`NoObserver`
(the rt crate's re-export precedent). The cache never changes a
decision — the differential pins hold the answers byte-identical to the
uncached walk.

* **M2** (the whole landing): the engine's pool field + the one
  `compiled()` helper over all four compile sites, the
  `with_observer` constructor, the face re-exports on `aeolus-lsp`.

## RED → GREEN

* RED: `crates/aeolus-lsp/tests/engine_cache.rs` (5 pins, written
  first — the unresolved `aeolus_lsp::{CacheSnapshot, CacheStats}`
  imports and the missing `with_observer` method ARE the capture: the
  keystroke repeat's hit set (3 misses then 3 hits on the fixture's
  three ScanHits), the cached answers byte-identical to the uncached
  walk, the didChange missing only its new spelling, the hover's
  single-candidate compile riding the same pool, the observation-free
  default answering unchanged beside the wired totals).
* GREEN: the same pins pass against the landed ride; the lsp crate's
  existing suites re-run unchanged (the engine's 48 + the server's 51,
  the answers deterministic over the document map).

## Bench

`cargo bench -p aeolus-lsp --bench lsp` (divan, release): the ride's
latency half — `analyze_document` (the cold shape: a fresh engine per
call) **229.6 µs** median against the NEW `analyze_warm_pool` arm (the
same engine re-analyzing the unchanged document) **1.42 µs** median —
the keystroke repeat is ~160× cheaper, the pool's own hit path; the
hover/cursor arms ride the same pool beside (2.83 µs hover).

## Consumer demo

`cargo run -p aeolus-lsp --example lsp_cache` — the didOpen walk, the
unchanged-document re-analyze (every string a hit), the didChange
missing only its new spelling and the hover hitting the entry analyze
stored — one stats handle printing the session's totals, headless.
