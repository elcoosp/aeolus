# 36 — The dyn-cache eviction face

The observability seam's answer to the question the storage slice's
boundedness story always implied: WHAT DID THE CAP COST ME? The built-in
[`TwoGenStore`] makes two kinds of leave-taking decisions the observer
could not see — the ROTATION that drops the whole `previous` generation
one step further back, and the STALE-TWIN clear (a fresh store of a key
whose previous-generation twin still sits behind the rotation, the config
flip-flop's cleanup). Both now ride the observer's `on_evict` under the
typed [`EvictReason`] (`Rotation`/`StaleTwin`) and the SAME composed-key
vocabulary the probe/store events report (the identity prefix and the
input verbatim, never a hash). The wire-up is the seam's own
composition: [`DynStore::attach_observer`] (a default no-op trait
method — the trait is the seam, the built-in store a consumer of it, the
`SuggestionSurface` precedent) reaches the store from
[`DynCache::with_observer`], so ONE `CacheStats` handle spans the
cache's hit/miss/store stream AND the store's evictions without the host
wiring anything twice. The callbacks fire AFTER the store's write lock
releases — the dropped keys collect under the lock, the events fire
outside it — so a host observer can consult the cache it watches
without deadlocking it. The session totals grow the answer:
`CacheSnapshot.evictions` (the cap's pressure number). A promotion is a
MOVE between generations — nothing leaves, and no event rides. The
observation-free default keeps the rotation's allocation-free shape
(the dropped-keys report only exists beside a wired observer), and the
differential pins hold every contract byte-identical with and without
the face.

* **M1** (the whole landing): the `EvictReason` type + the
  `DynObserver::on_evict` channel (default no-op — implementors opt
  in; the `Arc<O>` blanket forwards), the `DynStore::attach_observer`
  wire-up + the `TwoGenStore` reporting (rotation drops in the map's
  own walk order, the stale twin after, both after the lock drops), the
  `CacheStats`/`CacheSnapshot` evictions counter, the
  `EvictReason` re-exports from `aeolus-compile` AND `aeolus-rt` (the
  face stays nameable on the runtime surface).
* The neighbouring suite literals grew the field (`evictions: 0`) —
  the exact-total pins keep their strength, the expected snapshots
  answer the new slot with the session's own zero.

## RED → GREEN

* RED: `crates/aeolus-compile/tests/dyn_eviction.rs` (7 pins, written
  first — the unresolved `EvictReason` import, the missing
  `on_evict`/`attach_observer` members and the missing
  `CacheSnapshot.evictions` field ARE the capture: the rotation's
  dropped generation rides the events through the cache's own
  wire-up, the stale twin's clear at the raw store seam, the
  promotion's quiet, the built-in totals' math, the replace-in-place
  arm's non-eviction, the observation-free default byte-identical
  beside the resident bounds, the shared stats exact under four
  threads of rotation churn).
* GREEN: the same pins pass against the landed face; the neighbouring
  suites re-run unchanged (`tests/dyn_observer.rs`'s 16 pins — the two
  snapshot literals gaining `evictions: 0` — and `tests/dyn_cache.rs`'s
  8 rotation/promotion pins, the rt crate's `cell_observer` and
  `observability_face` suites likewise).

## Bench

`cargo bench -p aeolus-compile --bench observer` (divan, release): the
face's cost on the paths it rides — the HIT path unchanged beside the
existing arms (863 ns observation-free / 924 ns stats / 1.04 µs
worst-case recording, medians), the ROTATION path's new arms **1.17 µs**
observation-free / **1.39 µs** with the worst-case recording observer
(medians) — the eviction report's cost is the events, and only beside
an observer; the snapshot read stays ~0.8 ns with its new field.

## Consumer demo

`cargo run -p aeolus-compile --example cache_eviction` — the cap-4
churn's stream (rotation drops typed and in order), the fixed hot
string's promotions staying quiet beside the churn, and the raw store
seam's stale-twin clear through `attach_observer` — headless.
