# 45 — The selection ranges

The expand-selection walk (the wire's
`textDocument/selectionRange`) — the editor's next expansion step
answered from the locator the engine already owns. Every cursor-keyed
answer the engine serves (hover, highlights, references,
prepare-rename) starts at the same place: the candidate span under
the cursor, located by `candidate_span_at` over the extraction seam's
strings. The expand-selection gesture asks the question one level up
from the same locator — "what CONTAINS the cursor's selection, and
what contains that?" — and nothing answered it: the editor's
selection grew only as far as its own double-click heuristic. The
landing adds the answer type and the walk: one parent chain per
requested position, request order kept (the protocol's own 1:1
contract). The chain is the locator's own shape: a cursor inside a
candidate answers the candidate's byte-true span with the class
string's own content span riding as its parent (the extractor's
trimmed hit — the quotes stay the editor's territory, no invented
level above the string); a cursor inside the string but between
classes answers the content span ALONE (no candidate level, no
invented inner range — the honest boundary the chain's two levels
carry); a cursor outside any string answers the protocol's own
minimal range — the position's zero-width caret, no parent — and the
zero-width caret is also the unknown URI's answer (one per requested
position: the contract holds wherever the cursor was). The wire half
advertises `selectionRangeProvider` (the plain `true` form) and the
`textDocument/selectionRange` arm maps the chains through the
protocol's own `parent` key — the `wire_selection` recursion bottoms
at the chain's head — and the answer's length IS the request's at the
wire (the 1:1 contract the multi-cursor pins ride). Byte-truth rides
the engine's own position model: every range in UTF-16 pairs, the
multi-byte neighbour counting its pairs, not its bytes.

* **M2** (the whole landing): the `EngineSelectionRange` answer type,
  the `selection_ranges` walk (the locator's two-level read, one scan
  per position), the wire arm + capability + the `wire_selection`
  mapping, the red-first suite, the bench arm, the consumer example,
  the README/architecture-doc/changelog rows.

## The shape evidence

The walk is the locator's own read extended one level: the same
`scan_rust_strings` hit lookup, the same `parse_class_string` candidate
fold, the candidate half shared verbatim with `candidate_span_at`'s
body (the hover's shared walk — no parallel matcher), with the content
span answering as the chain's head where the candidate level stops.
One scan per position (multi-cursor requests ride in request order;
a three-cursor pin covers the three chain shapes — two-level, content
alone, caret). The zero-width caret is built at the REQUESTED position
in the wire's own units — no byte round-trip can widen it — so an
out-of-range cursor answers a shape the editor can render, never a
guessed range.

## RED → GREEN

* RED: `crates/aeolus-lsp/tests/selection_ranges.rs` (11 pins,
  written first — the unresolved `EngineSelectionRange` import and the
  missing `selection_ranges` method WERE the capture: the two-level
  chain's columns, the between-classes content span, the zero-width
  caret, the 1:1 multi-cursor order, the unknown URI's carêts, the
  byte-true UTF-16 pairs over the accented neighbour, and the wire
  half's nested quads/capability advertisement/pre-handshake gate/
  malformed null/length parity).
* GREEN: the same 11 pins pass against the landed walk; the lsp
  crate's full suite re-measured green (engine 59, engine_cache 5,
  engine_walk 4, references 12, rename_workspace 14, selection_ranges
  11, server 62, transport_observer 7, workspace_symbols 12); the
  full workspace gate set re-measured post-landing (fmt, clippy
  `--workspace --all-targets --all-features -- -D warnings` zero
  warnings, `cargo test --workspace --all-features`, benches).

## Bench

`crates/aeolus-lsp/benches/lsp.rs` (divan, stable):
`selection_ranges_walk` ~841 ns median over the bench's own document
— a two-cursor request (the candidate chain beside the caret), the
locator's scan per position the cost a multi-cursor expand pays per
invocation.

## Files

`crates/aeolus-lsp/src/lib.rs` (the answer type),
`crates/aeolus-lsp/src/engine.rs` (the walk),
`crates/aeolus-lsp/src/server.rs` (the arm, the capability, the
`wire_selection` mapping), `tests/selection_ranges.rs` (new),
`benches/lsp.rs` (the arm),
`examples/lsp_selection_ranges.rs` (new, the three-cursor demo),
the README (LSP rows + example index), the architecture doc P5 row,
the changelog.
