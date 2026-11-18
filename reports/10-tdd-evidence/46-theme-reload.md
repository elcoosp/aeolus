# 46 — The theme reload

The theme lifecycle (the watched-files wire half over the engine's
retheme seam) — the editor's `app.css` save button re-themes a live
session. Every engine answer the session serves rides the table the
config fixed AT CONSTRUCTION: a designer rebranding the palette meant
restarting the server, the one lifecycle boundary the engine's faces
never crossed. The landing adds the seam at every layer: the compile
crate's pool gains its invalidation face (`DynStore::invalidate_all`,
the built-in store reporting one typed `EvictReason::Invalidated` per
retired key after its lock drops; `DynCache::clear` the cache-level
face — a clear retires the memory, never a decision, the next probe of
a cleared string a miss that re-lowers to the same bytes); the engine
gains `retheme` (the config's token slot swaps wholesale, the equality
fast-path keeping the no-op save cheap: a candidate table whose tier,
names and resolved colours equal the riding one answers
`RethemeOutcome::Unchanged` and touches NOTHING — the pool keeps
serving, the answers stay byte-identical); and the wire half drives it
(`Session::with_theme_loader` wiring ONE `ThemeLoader` — the host's own
statement of where the next table comes from, `FileThemeLoader` the
disk form that reads fresh every load, no cache between loads since the
notification IS the freshness signal): the
`workspace/didChangeWatchedFiles` event naming the loader's URI
re-bakes and re-themes, a failed load keeping the riding table (the
typed `ThemeError` lane — nothing changed, nothing republishes), a
real swap republishing EVERY open document's diagnostics in URI order.
The binary face rides the same loader: `aeolus-lsp --theme <path>`
bakes the startup table through the loader's OWN first read (one
construction path; a bad file is a startup error naming its
`ThemeError` before the transport starts) and the same loader watches
the session — `run_stdio_with_theme` the loop's themed twin.

## The shape evidence

The invalidation is the store's OWN decision reported through the
observer faces' existing vocabulary: the wholesale leave collects under
the store's write lock (what left — the previous generation's keys
first, the current's after; no meaningful order exists BETWEEN the two
maps) and fires after the lock drops, the rotation's own
deadlock-freedom rule. The typed `Invalidated` reason is deliberately
distinct from `Rotation` (a host counting drops reads the deliberate
retirement, not churn) and from `StaleTwin` (no key was rewritten —
every key left, none of them stale under their own identity). The
engine's equality fast-path rides the theme table's public getters
(tier, names in id order, the resolved rgba per id beside the pair
surface): two tables equal under it lower to byte-identical programs,
which is the contract the no-op save's `Unchanged` needs — and the
tier rides FIRST, because a baked and a runtime table over the same
css carry the same colours but answer different `Val` shapes. The
document map rides untouched through a swap (the answers re-derive
lazily from the strings, the same purity every other answer keeps);
the engine's new `documents()` accessor is the wire half's republish
walk (and the host embedding the engine's own push channel set).

## RED → GREEN

* RED: `crates/aeolus-compile/tests/dyn_cache_clear.rs` (3 pins,
  written first — the missing `invalidate_all`/`clear` methods and the
  `EvictReason::Invalidated` variant WERE the capture: the wholesale
  leave empties both generations and the decisions keep flowing, every
  leave riding the typed reason once, the cache-level clear keeping
  the answers byte-identical with the re-lowering the new miss) and
  `crates/aeolus-lsp/tests/theme_reload.rs` (11 pins — the unresolved
  `ThemeLoader`/`FileThemeLoader`/`RethemeOutcome` imports and the
  missing `retheme` method WERE the capture: the swap retires the pool
  and the new table answers, the equal-content retheme the no-op the
  pool survives, the themeless boundary staying honest, the disk
  loader's read, the watched-files event re-theming the session beside
  the no-op save answering nothing, the unrelated file riding
  silently, the loader-less session staying quiet, the failed load
  keeping the last good table, and the swap republishing every open
  document in URI order).
* GREEN: the same 14 pins pass against the landed seam; the touched
  crates' full suites re-measured green (aeolus-lsp engine 52,
  engine_cache 5, engine_walk 4, references 12, rename_workspace 14,
  selection_ranges 11, theme_reload 11, server 62, transport_observer
  7, workspace_symbols 12; aeolus-compile's batteries including
  dyn_eviction 7 and dyn_cache; aeolus-theme's four suites). The
  `EvictReason::Invalidated` variant joined the eviction face's
  exhaustive matches (the test's translator and the `cache_eviction`
  example — the typed vocabulary's own consumers).

## Bench

`crates/aeolus-lsp/benches/lsp.rs` (divan, release, divan 0.1.21,
median of 100): `theme_bake` ~38.8 µs (the loader's own
`from_css` + `baked` — the read every re-theme pays), `retheme_swap`
~120.5 µs (the whole honest lifecycle: the bake, the pool
invalidation and the document's strings re-lowered under the new
table), `retheme_unchanged` ~119.8 µs (the no-op save: the bake
beside the equality fast-path — the invalidation and the recompile
are what the fast-path SKIPS, the bake itself the reload's dominant
cost).

## Files

* `crates/aeolus-compile/src/dyn_cache.rs` — `DynStore::invalidate_all`,
  `TwoGenStore`'s typed wholesale leave, `DynCache::clear`, the
  `EvictReason::Invalidated` variant.
* `crates/aeolus-compile/tests/dyn_cache_clear.rs` — the 3-pin RED
  battery.
* `crates/aeolus-theme/src/lib.rs` — no new API (the existing
  `tier()` getter is the equality's first read).
* `crates/aeolus-lsp/src/theme_loader.rs` — the `ThemeLoader` seam,
  `FileThemeLoader`, the `Arc` sharing blanket.
* `crates/aeolus-lsp/src/lib.rs` — `RethemeOutcome`, the re-exports.
* `crates/aeolus-lsp/src/engine.rs` — `retheme`, `tokens_equal`,
  `documents()`.
* `crates/aeolus-lsp/src/server.rs` — `Session::with_theme_loader`,
  the `workspace/didChangeWatchedFiles` arm, `run_stdio_with_theme`.
* `crates/aeolus-lsp/tests/theme_reload.rs` — the 11-pin RED battery.
* `crates/aeolus-lsp/benches/lsp.rs` — `theme_bake`, `retheme_swap`,
  `retheme_unchanged`.
* `crates/aeolus-lsp/examples/lsp_theme_reload.rs` — the headless
  consumer drive (a host's own loader, the rebrand save, the no-op
  save, the failure lane).
* `crates/aeolus-lsp/src/bin/aeolus-lsp.rs` — the `--theme <path>`
  flag (the startup bake through the loader's own first read).
