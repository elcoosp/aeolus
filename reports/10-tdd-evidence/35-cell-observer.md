# 35 — The view-state cell's observability face

The dynamic-strings slow lane answers the observability question the
bounded pool already answers (report 16's seam, the handoff the dyn-cache
observability face filed): [`ClassCell::with_observer`] wires a
host-implemented [`DynObserver`] — the SAME trait the pool's
`DynCache::with_observer` carries — and the cell's decisions ride it in
order (miss, store, hit) under the composed key the [`DynStore`] vocabulary
already probes. The composed key gained its public composer
(`aeolus_compile::session_key`): the identity prefix (the config
fingerprint and the table/bag addresses, `\u{1}`-separated) followed by
the input verbatim — never a hash, the collision-safety rule — so the pool
and the cells report ONE vocabulary and one built-in [`CacheStats`] handle
spans every slot the app wires it into (the app-wide budget's shape). The
cell stays the UI-thread surface (`!Sync`, the plan's own note); the
handle is the `Send + Sync` half. `NoObserver` stays the
every-constructor default: the observer adds a channel, it never changes
a decision (the differential suites pin every contract byte-identical
with and without one).

* **M1** (the whole landing): the cell's `with_observer` constructor +
  the wire-up through the one `swap` path (hit before the counter bump,
  miss before the lowering, store carrying the entry's diagnostic count
  — the cache's own order), the key composed ONCE per call through
  `session_key`, the per-cell counters unchanged beside the stream.
* **M2** (the host surface's nameability): `DynObserver`, `NoObserver`,
  `CacheStats`, `CacheSnapshot` and `session_key` re-exported from
  `aeolus_rt` — a host consuming the runtime crate names the face
  WITHOUT adding `aeolus-compile` to its manifest (the re-export block's
  missing entries, the probe-face handoff's precedent).

## RED → GREEN

* RED: `crates/aeolus-rt/tests/cell_observer.rs` (9 pins, written first —
  the unresolved `session_key` import and the missing `with_observer`
  method ARE the capture: the decision order miss→store→hit, the
  identity's key split between the theme-less and themed sessions, the
  built-in snapshot's math, one stats object spanning the pool AND two
  cells, the cell-and-cache one-vocabulary pin, the caller bug missing
  and never storing with the last good program standing, the store
  event's diagnostic count, the observation-free defaults beside the
  every-entry-point stream, the shared stats exact across four threads
  of per-thread cells). `crates/aeolus-rt/tests/observability_face.rs`
  (3 pins, RED-first — the unresolved `aeolus_rt::{DynObserver,
  NoObserver, CacheStats, CacheSnapshot, session_key}` imports ARE the
  capture: the nameability, the shared handle through the runtime
  surface's own names, the composed-key vocabulary's shape and its
  byte-identity contracts).
* GREEN: the same pins pass against the landed seam; the neighbouring
  suites re-run unchanged (`tests/class_cell.rs`'s 9 differential pins,
  the probe/style-face suites, the compile crate's own dyn-cache
  batteries — the observer never changes a decision).

## Bench

`cargo bench -p aeolus-rt --bench class_cell` (divan, release): the seam's
cost ON the hot path it rides — `cell_hit` **241.3 ns** median baseline /
`cell_hit_stats_observer` **214.8 ns** / `cell_hit_recording_observer`
**272.0 ns** (the composed-key `format!` bounded within the run's noise;
the worst-case recording observer ~+30 ns beside the baseline), the miss
lanes unchanged (`cell_miss_rotating` 1.44 µs, `cell_theme_miss` 1.79 µs).

## Consumer examples

`crates/aeolus-rt/examples/cell_observer.rs` (headless): the 60-frame
loop with the pool and the cell sharing one `CacheStats` handle (the
snapshot beside the per-slot counters), one stats object across two view
cells, and the streaming observer over a mixed battery with the
diagnostic counts printing as they happen.
