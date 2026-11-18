# 41 — The compile-walk observability seam

The dyn-cache face's twin, pointed at the walk. The pool's
observability seam answered "what did the cache decide" — `DynObserver`
under the composed `session_key`, the `CacheStats` collector, the typed
`EvictReason` — and the walk itself stayed quiet: the `tracing`
feature's spans are the stringly channel a host must subscribe to, the
artifact's diagnostics are the answer-only channel, and a host driving
the walk itself (a build script, a server session's per-keystroke
compiles) had no typed per-stage telemetry. The landing adds it with
the cache face's own rules kept exactly: every trait method a no-op
default (the seam adds a channel, never replaces one), `NoWalkObserver`
the every-constructor default, the `Arc` blanket the cache face keeps,
`WalkStats`/`WalkSnapshot` the built-in collector, and the events the
walk's OWN stage boundaries — `on_lexed(candidates, degraded)`, the
per-candidate `on_candidate(raw, CandidateOutcome)` verdict
(`Lowered { decls, diagnostics }` / `Missed { diagnostics }`), and
`on_assembled(layers)`. The wire-up is the internal `run`'s observer
slot: `None` the every-public-constructor default, `compile_observed`
the one entry that attaches the channel over the whole surface (theme
pass and customs ride their own slots — no four-function family).

* **M2** (the whole landing): the `walk_observer` module (trait +
  outcome enum + default + blanket + collector), the `run` wire-up
  (the per-candidate deltas riding the walk's own counters — a
  diagnostics watermark and the declarations total, never a recount),
  the `compile_observed` entry, the red-first suite, the rt/lsp
  re-exports beside the pool face, the rt nameability pin, the bench,
  the consumer example, the README and architecture-doc rows.

## The shape evidence

The seam is BYTE-NEUTRAL by differential pin — the observed walk's
`Compiled` is the plain compile's artifact (the program's Debug equal,
the diagnostics' set equal), which the suite pins directly and the
bench baseline confirms (the unobserved arm IS the plain `compile`
path). The collector's `diagnostics` counter answers the artifact's
total — the degraded lex failures ARE `Diagnostic`s (the Lex kind), so
`on_lexed` counts them into the same counter: one number for one
truth. The per-candidate outcome rides the walk's own counters (a
diagnostics watermark + the declarations total computed as deltas), so
the events can never disagree with what the artifact carries.

## RED → GREEN

* RED: `crates/aeolus-compile/tests/walk_observer.rs` (6 pins, written
  first — the unresolved-import compile failure IS the capture: the
  collector's stage totals against a known mixed string, the
  per-candidate outcome split in walk order, the multi-layer assembly
  count, the themed walk's same stages, the byte-neutrality
  differential, the shared-handle session).
* GREEN: the same pins pass against the landed seam — 6/6; the rt
  observability face re-measures 4/4 (the nameability pin added); the
  full workspace gate set re-measured post-landing (fmt, clippy
  `--workspace --all-targets --all-features -- -D warnings` zero
  warnings, `cargo test --workspace --all-features`,
  `cargo test --workspace --benches`).

## Bench

`cargo bench -p aeolus-compile --bench walk_observer` (divan,
release), over the Button slice's real shape:
`walk_unobserved` **7.49 µs** · `walk_counted` **7.71 µs** ·
`walk_recorded` **8.00 µs** medians — the seam's overhead story:
~3% for the built-in counters at the stage hooks, the worst-case
mutex-recording channel bounded beside (the per-candidate verdicts
dominate; the stage totals ride twice per walk).

## Consumer examples

`crates/aeolus-compile/examples/walk_observer.rs` (the session totals
over three strings, the custom per-candidate channel's walk-order
verdicts, the byte-neutrality read-out headless):

```sh
cargo run -p aeolus-compile --example walk_observer
```

## Docs

README's dynamic-lane observability paragraph names the walk face
beside the pool's (the stringly `tracing` flag's typed alternative)
and the crate-map rows file the seam; the architecture doc's P2 row
files the landed note beside the dyn-cache's own; the face
re-exports through `aeolus-rt` and `aeolus-lsp` keep the
no-manifest-add discipline the pool face set.
