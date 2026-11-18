# 49 — The workspace-wide pull

The wire's `workspace/diagnostic` (LSP 3.17) — the pull channel
generalized over the whole document map. The document pull answered
per document; a client syncing a whole workspace re-sent one request
per open file with no cache contract to ride — the spec's workspace
pull exists for exactly that, and the server advertised
`workspaceDiagnostics: false`. The landing flips it honestly:
`AeolusEngine::workspace_diagnostics(&[(&str, &str)])` answers EVERY
open document — URI order across the map (the cross-document answers'
own determinism contract) — each report riding its RESULT IDENTITY,
the content-keyed 64-bit FNV-1a over the document's exact text. The
identity's design decision: the findings are pure over the text, so
the CONTENT is the identity — the same text answers the same id
whatever the version tag rides (the equal-content re-save the
byte-neutral no-op the retheme fast-path keeps), a text change a new
one. A previous pair whose id matches answers
`EngineWorkspaceOutcome::Unchanged` — the client's cache is the
report, nothing recomputes (the pool's cheap path skipped wholesale);
a stale id, or a uri the previous list never named, answers the full
report (the engine never caches, the pull channel's own contract).

## The shape evidence

The engine carries the identity and the outcome; the wire carries the
JSON. lsp-types 0.97 has no typed workspace-report shape (the
workspace pull never standardized its structs into the crate), so the
items ride the `json!` form the workspace-symbol answer keeps — the
crate's own wire precedent for protocol shapes the typed crate lags:
`full` items re-list the findings through the shared `wire_diagnostic`
(one render, every transport), `unchanged` items echo the identity and
carry no findings key. The params' `previousResultIds` walks off the
raw JSON with the spec's own required shape (the array each of whose
entries carries `uri` and `value`; a missing key or a shapeless entry
is malformed — the protocol's own null). The capability flip is the
milestone's advertised half: `DiagnosticOptions::workspace_diagnostics`
goes `true`, and the server battery's standing pin re-pins with it —
the re-pin IS the contract upgrade (the no-workspace-pull comment the
old pin carried named the exact boundary the landing crosses).

## RED → GREEN

* RED: `crates/aeolus-lsp/tests/workspace_diagnostics.rs` (14 pins,
  written first — the missing `workspace_diagnostics` method and the
  missing `EngineWorkspaceOutcome` type WERE the capture: the
  URI-order map walk, the full report's findings beside the identity,
  the content-keyed identity (same text, same id, across URIs), the
  text change a new identity beside the equal-content change keeping
  it, the matching id the unchanged verdict, the stale id the
  recompute, the unopened previous entries riding unanswered, the
  purity pin, and the wire's items/unchanged echo/capability
  advertisement/pre-handshake gate/malformed null). The cycle taught
  the identity once: a version-only didChange (same text, new tag)
  answers the SAME id under the content-keyed contract — the stale-id
  and change pins re-fixture onto text changes, and the equal-content
  pin (`an_equal_content_change_keeps_the_identity`) landed beside them
  to pin the no-op half deliberately.
* GREEN: the same 14 pins pass against the landed walk; the lsp
  crate's full suite re-measured green (235 pins across the batteries,
  the folding face's 12 and the pull's 14 among them).

## Bench

`crates/aeolus-lsp/benches/lsp.rs` (divan, release, divan 0.1.21,
median of 100): `workspace_diagnostics_pull` ~1.67 µs over one open
document (the fingerprint beside the pooled analyze — the document
pull's own ~2.4 µs walk the workspace pull rides), and
`workspace_diagnostics_unchanged` ~266 ns — the quiet pull, the
fingerprint matching and the recompute skipped wholesale (the
retheme fast-path's own ratio at the pull channel).

## Files

* `crates/aeolus-lsp/src/engine.rs` — `workspace_diagnostics` (the
  URI-order map walk, the identity, the outcome) and `fnv1a` (the
  dependency-free fingerprint).
* `crates/aeolus-lsp/src/lib.rs` — `EngineWorkspaceDiagnostic` +
  `EngineWorkspaceOutcome` (the wire-free answer types), the
  crate-doc bullet.
* `crates/aeolus-lsp/src/server.rs` — the `workspace/diagnostic` arm,
  the `previous_result_ids` param walk, the
  `workspace_diagnostics: true` flip.
* `crates/aeolus-lsp/tests/workspace_diagnostics.rs` — the 14-pin RED
  battery.
* `crates/aeolus-lsp/tests/server.rs` — the standing
  `workspaceDiagnostics` pin re-pinned (`false` → `true`).
* `crates/aeolus-lsp/benches/lsp.rs` — `workspace_diagnostics_pull`,
  `workspace_diagnostics_unchanged`.
* `crates/aeolus-lsp/examples/lsp_workspace_diagnostics.rs` — the
  headless consumer drive (the first pull, the quiet pull, the
  one-didChange pull).
