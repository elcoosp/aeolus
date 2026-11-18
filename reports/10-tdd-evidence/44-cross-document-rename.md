# 44 — The cross-document rename

The rename face's workspace upgrade — the references fold's third
consumer. The rename face landed per-document (report rows in the
rename handoff: "one edit per textual occurrence ... the
per-document boundary kept: the map is the scope"), and the boundary
was honest then — `textDocument/references` had not landed. After the
cross-document answers landed (report rows in the references and
workspace-symbol handoffs), the per-document rename became the ONE
answer left speaking a narrower scope than its protocol shape:
`textDocument/rename` answers a `WorkspaceEdit` whose `changes` table
is keyed by URI — the protocol's own workspace-edit contract — and
the wire half was flattening every edit onto the REQUESTING URI's key
alone, occurrences in the other open documents riding nowhere. The
landing lifts the boundary: `AeolusEngine::rename` answers
`Vec<EngineRenameEdit>` (the edit beside the URI of the document that
carries it), the references' own fold generalized onto the rename
edits — the cursor's candidate located ONCE through the shared
locator (`candidate_span_at`, the hover's own walk), one scan per
open document (`scan_rust_strings` + `parse_class_string`, never a
parallel matcher), each textual match riding its document's URI
beside its byte-true UTF-16 range and the new spelling. URI order
across the documents (sorted, whatever the open order was — the
answer is pure over the hash map), document order within each. The
identity stays TEXTUAL: a refused class's occurrences edit exactly
like a lowered one's (no compile verdict riding — the diagnostics
channel tells the truth about the NEW spelling the moment the
didChange lands), and a variant-prefixed sibling (`hover:flex-col`
beside `flex-col`) is a DIFFERENT spelling, renamed whole. The guards
keep their shapes: an empty new name answers NO edits anywhere (the
deletion walk never runs — the protocol's validity clause), an
unknown cursor URI and a cursor outside any candidate answer empty.
The wire half answers the FULL changes map — one key per carrying
document, a URI the `Uri` parser rejects skipping its key (the
references half's own rule) — and the outside-candidate answer's
shape re-pins from the single-document one-key-with-an-empty-list to
the cross-document honest empty: no edits anywhere, no keys.

* **M1** (the whole landing): the `EngineRenameEdit` answer type, the
  `rename` walk generalized over the map, the wire arm's grouped
  changes map, the red-first suites (the new battery + the re-pinned
  contract pins), the bench arms, the consumer examples (the face's
  own demo beside the workspace one), the README/architecture-doc/
  changelog rows.

## The shape evidence

The walk is the references method's own body read at one level
deeper — the same URI sort, the same one-scan-per-document loop, the
same byte-true range construction — with the edit's new text riding
where the reference rode nothing. The requesting document locates the
candidate ONCE through the shared locator and every other document
reuses the raw text: no document's answer depends on the open order
(the reverse-open determinism pin), no occurrence answers twice (the
cursor's own occurrence rides exactly once), and the closed document
drops out of the walk the same way it drops out of the references'
(the map IS the scope). The wire arm groups by URI in the answer's
own order — the edit lists preserve the engine's document order
within each key, the parsed `Uri` keys the same rule the publish
half's quad rides.

## RED → GREEN

* RED: `crates/aeolus-lsp/tests/rename_workspace.rs` (14 pins,
  written first — the unresolved `EngineRenameEdit` import and the
  missing `uri` field WERE the capture: the cross-document ride, the
  URI-order determinism over a reverse-opened map, the cursor's own
  occurrence, the variant-prefixed sibling, the refused ride, the
  unknown-URI/outside-candidate empties, the empty-name guard's
  map-wide quiet, the closed-document drop, the byte-true UTF-16
  pairs over an accented neighbour, and the wire half's full-changes
  map/pre-handshake gate/malformed null/empty-map shape/refused
  shape).
* GREEN: the same 14 pins pass against the landed walk; the renamed
  contract pins re-measured (`tests/engine.rs`'s rename battery grew
  the URI assertions; `tests/server.rs`'s outside-candidate pin
  re-pinned to the empty map — the re-pins ARE the contract upgrade);
  the lsp crate's full suite re-measured green (engine 59,
  engine_cache 5, engine_walk 4, references 12, rename_workspace 14,
  selection_ranges 11, server 62, transport_observer 7,
  workspace_symbols 12); the full workspace gate set re-measured
  post-landing (fmt, clippy `--workspace --all-targets --all-features
  -- -D warnings` zero warnings, `cargo test --workspace
  --all-features`, benches).

## Bench

`crates/aeolus-lsp/benches/lsp.rs` (divan, stable), medians over the
bench's own document: `rename_walk` ~1.6 µs (the one-document scope,
beside the references arm's own walk) and `rename_eight_documents`
~7.1 µs (the map's scaling story — one scan per document beside the
URI sort, the references arm's own eight-document set ~6.9 µs riding
beside: the SAME walk shape answers both). The gesture's cost scales
with the OPEN SET, not the document's length — the honest shape for a
workspace edit.

## Files

`crates/aeolus-lsp/src/lib.rs` (the answer type),
`crates/aeolus-lsp/src/engine.rs` (the generalized walk),
`crates/aeolus-lsp/src/server.rs` (the grouped changes map),
`tests/rename_workspace.rs` (new), `tests/engine.rs` +
`tests/server.rs` (the re-pinned contract pins),
`benches/lsp.rs` (the two arms),
`examples/lsp_rename.rs` (the type-shape catch-up) +
`examples/lsp_workspace_rename.rs` (new, the three-document demo),
the README (LSP rows + example index), the architecture doc P5 row,
the changelog.
