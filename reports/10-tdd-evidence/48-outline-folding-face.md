# 48 — The outline folding face

The wire's `textDocument/foldingRange` — one fold per class string the
extraction seam names. The editor's outline panels fold regions; the
engine answered the outline (`documentSymbol`), the selection chains
and the linked ranges, but nothing answered "what folds here" — and
the extraction seam IS the region model: every class string is a
region the editor can collapse, the multi-line raw-string call sites
the folds a reader actually reaches for (the quoted walk stops at a
newline — the raw-string form the multi-line shape). The landing
answers from the seam alone: one `EngineRange` per extracted hit — the
string's own content span (the extractor's trimmed hit, the quotes the
editor's territory), document order kept, each fold spanning the whole
content (every candidate inside it rides the one range). The fold is a
SHAPE-level answer — the highlights' textual family: no compile
verdict rides, a refused candidate's string folds exactly like a
lowered one's, and the answer set is CLOSED by the extractor's own
pre-filter (a literal with no `-`/`:` token is not a class string, so
it never folds) — the server reports the class strings it knows, the
client renders which folds it wants (the protocol's own contract). The
wire half advertises `folding_range_provider` (the plain `true` form)
and the `textDocument/foldingRange` arm maps the folds onto the
protocol's own `FoldingRange` — the columns riding (the client folds
the exact span), no `kind` riding: a plain region fold, the client's
own default.

## The shape evidence

The walk is the cheapest the engine answers: the extraction seam's
scan and the per-hit range translation, no `compiled()` call anywhere
on the path — the same shape-level discipline the formatter's
themeless walk keeps (a fold is a fact about the TEXT, not the
verdict). The content span reuses the format face's exact territory
(the extractor's trimmed hit), so the fold's columns agree with the
edits' and the selection chains' head — one span model across the
three answers. The wire arm rides the shared `wire_range` conversion
and the pre-handshake gate every request arm keeps; the malformed
answer stays the protocol's own null, and the unknown URI answers the
empty ARRAY (a list-shaped answer keeps the list shape at the wire,
the document pull's own precedent).

## RED → GREEN

* RED: `crates/aeolus-lsp/tests/folding_ranges.rs` (12 pins, written
  first — the missing `folding_ranges` method WAS the capture: one
  fold per class string in document order, the content span's columns,
  the multi-line raw string folding across its lines, the refused
  ride, the prose literal staying out, the unknown URI, the byte-true
  UTF-16 columns over the accented neighbour, the purity pin, and the
  wire's folds/closed-URI empty/capability advertisement/pre-handshake
  gate/malformed null). The cycle taught the fixture twice: the
  multi-line fold's END column is the content's last byte (20, not the
  line's start), and the fold's span assertion rides component
  equality (`EngineRange` carries no ordering — the honest comparison
  names the fields).
* GREEN: the same 12 pins pass against the landed walk; the lsp
  crate's full suite re-measured green alongside the workspace-pull
  battery (report 49).

## Bench

`crates/aeolus-lsp/benches/lsp.rs` (divan, release, divan 0.1.21,
median of 100): `folding_ranges_walk` ~539 ns over the bench's own
document — the extraction scan and the range translation, the compile
verdict nowhere on the path; the cheapest whole-document answer the
engine carries (the outline's per-candidate compile ~9.7 µs beside it,
the shape answer's own ratio).

## Files

* `crates/aeolus-lsp/src/engine.rs` — `folding_ranges` (the extraction
  scan folded to the content spans).
* `crates/aeolus-lsp/src/lib.rs` — the crate-doc bullet beside the
  selection ranges'.
* `crates/aeolus-lsp/src/server.rs` — the
  `textDocument/foldingRange` arm, the `folding_range_provider`
  capability.
* `crates/aeolus-lsp/tests/folding_ranges.rs` — the 12-pin RED
  battery.
* `crates/aeolus-lsp/benches/lsp.rs` — `folding_ranges_walk`.
* `crates/aeolus-lsp/examples/lsp_folding_ranges.rs` — the headless
  consumer drive (the three folds, the raw-string collapse, the
  closed-document empty).
