# 43 — The pooled-path walk channel

The walk face's channel threaded through the dyn cache — the pooled
path the LSP engine rides. `compile_observed` landed with the walk
face (report 41) and had no production consumer: the engine exposed
the CACHE face only (`with_observer` → the pool's hit/miss/store
stream), and a host that wanted the walk's stage events had to bypass
the pool and call the free function — losing the cache's whole reason
for existing (the per-keystroke repeats answering from resident
entries). The landing gives the declared seam its host: the cache
gains `compile_observed` / `compile_theme_observed` (mirroring the
free functions, the cache methods' stated contract), and the engine
gains `with_walk_observer` (the `with_observer` shape) with
`compiled()` threading the wired observer through the pool per
answer. The rules kept exactly: the observer is a per-call `Arc`
argument into the internal `run`'s observer slot (no key change, no
decision change — an observed and an unobserved session pool into the
SAME slots, the byte-identity differential pins it); a MISS lowers
through the observed walk (the stage events stream exactly as the free
entry reports them); a HIT answers from the pool and streams NOTHING —
no walk ran, the channel reports lowerings, not answers, which is the
cache's economics kept intact. The engine holds ONE walk slot (the
builders' replace semantics, the cache face's own); a host that wants
two channels composes host-side — the trait's no-op defaults make the
composite two forwards (the demo's `Telemetry` pair). The pool face's
`with_observer` rides beside unchanged: one host, two vocabularies
over one session (the cache face answers "what did the pool decide",
the walk face "what did the walk do when the pool missed").

* **M2** (the whole landing): the two `DynCache` entries, the
  `AeolusEngine` wire-up (the `walk` slot, the manual `Debug` — the
  observer is a host handle, not printable state, the cache Debug's
  own rule — and the four-arm `compiled()` dispatch over
  tokens×observer), the red-first suites (compile + lsp), the bench,
  the consumer example, the README/architecture-doc/changelog rows.

## The shape evidence

The events reach the observer ONLY through the miss path's
`crate::run(..., Some(&*observer))` closure — the `cached` helper's
hit path returns before the closure runs, so the channel's silence on
a hit is structural, not a promise. The themed variant keys the SAME
`DynIdentity::themed` fingerprint the plain themed entry probes, so
the face adds a channel, never a key. The engine's four-arm dispatch
(tokens × observer) keeps every existing call path byte-identical
(the `None`/`None` arm IS the old two-line body). The engine test
battery pins the session story end to end: three ScanHits → three
walks; the unchanged re-analyze → silence; the didChange's swapped
spelling → exactly one new walk (the kept strings hit); and the
analyzed diagnostics equal to the plain engine's — the face never
changes an answer at the surface a host reads.

## RED → GREEN

* RED: `crates/aeolus-compile/tests/pooled_walk.rs` (5 pins, written
  first — the "no `compile_observed` entry on `DynCache`" compile
  failure IS the capture: the observed miss's stage totals, the hit's
  silence, the byte-identity, the one handle spanning the free walk
  AND the pooled walk, the themed variant) +
  `crates/aeolus-lsp/tests/engine_walk.rs` (4 pins, the "no
  `with_walk_observer`" compile failure WAS the capture).
* GREEN: the same pins pass against the landed channel — 5/5 + 4/4;
  the compile and lsp crates' existing batteries re-measured (the
  cache face's, the walk face's, the engine's contracts and pool pins
  all untouched-green); the full workspace gate set re-measured
  post-landing (fmt, clippy `--workspace --all-targets --all-features
  -- -D warnings` zero warnings, `cargo test --workspace
  --all-features`, benches).

## Bench

`crates/aeolus-compile/benches/pooled_walk.rs` (divan, stable), over
the walk face's bench's own shadcn Button string, medians:
`pooled_observed_hit` ~395 ns (the pool answers; the channel costs
nothing on a hit — the cache face's own number beside an observer),
`pooled_observed_miss` ~5.4 µs (the full observed walk + the pool's
store), `pooled_theme_observed_miss` ~5.8 µs (the themed walk, the
table's tier consults riding the theme face beside).

## Files

`crates/aeolus-compile/src/dyn_cache.rs` (the two entries),
`crates/aeolus-lsp/src/engine.rs` (the engine wire-up),
`tests/pooled_walk.rs`, `crates/aeolus-lsp/tests/engine_walk.rs`,
`benches/pooled_walk.rs` + the manifest row,
`crates/aeolus-lsp/examples/lsp_walk_observer.rs`, the README (LSP
rows + example index), the architecture doc §3.4 note, the changelog.
