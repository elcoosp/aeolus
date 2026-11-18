# 39 — The LSP `documentSymbol` outline face

The engine's next question. The Aeolus engine answered twelve editor
questions — diagnostics, hover, completion, swatches, legend, picker
spellings, inline hints, quick fixes, highlights, prepareRename +
rename, formatting — and the wire's most-asked remaining capability
was the outline: `textDocument/documentSymbol`, the symbol panel's
feed. The landing adds the thirteenth question to the SAME engine,
over the SAME pipeline: one symbol per candidate class the extraction
seam's strings carry — the spelling the NAME, its byte-true range
(the shared UTF-16 translation), the semantic legend's own verdict
(the `compiled_candidates` walk's diagnostics-list emptiness, no
parallel matcher) and the pipeline's own detail line (a lowered class
renders the compact CSS the inlay hint's label carries — one answer
shape at a third surface; a refused one the stable code the
diagnostics key on). The wire half maps the verdict onto the
protocol's kinds: a lowered class rides `SymbolKind::Class`, a refused
one stays `SymbolKind::String` — the raw spelling, not a compiled
class, and no custom SymbolKind exists to name the boundary (the
highlighter's custom `refused` type is the legend's own extension;
SymbolKind has none).

* **M2** (the whole landing): the engine question
  (`AeolusEngine::document_symbols`), the `EngineSymbol` answer shape,
  the server dispatch arm (`textDocument/documentSymbol`) beside the
  `documentSymbolProvider` capability advertisement, the red-first
  engine battery and wire quads, the bench, the consumer example, the
  README rows.

## The shape evidence

The wire answer is the protocol's own JSON names, spelled directly:
`name`, `detail`, `kind`, `range`, `selectionRange` (the whole
spelling IS the name, so the selection rides the candidate's span).
The typed `lsp_types::DocumentSymbol` carries a deprecated field whose
literal construction trips the workspace's zero-allow bar, so the
mapping spells the protocol's own keys — the wire quads pin the exact
key set and kind numbers (5 / 15) beside, the same test-pinned contract
style the semantic legend's indexes ride. Document order is the
extraction seam's scan order, candidates within a string left to
right — the same ordering contract the legend and the swatches keep.

## RED → GREEN

* RED: `crates/aeolus-lsp/tests/engine.rs`'s symbol battery (4 pins,
  written first — the `E0599` no-method compile failure IS the capture:
  one symbol per candidate in document order, the byte-true ranges and
  the legend's verdict, the lowered-CSS/refused-code detail split, the
  purity and unknown-URI pins) + `crates/aeolus-lsp/tests/server.rs`'s
  wire quads (4 pins: the capability advertisement, the outline
  answer's key set and kinds, the before-initialize quiet drop, the
  malformed-params null).
* GREEN: the same pins pass against the landed face; the engine (57)
  and server (62) suites re-measure at the gates.

## Bench

`cargo bench -p aeolus-lsp --bench lsp -- document_symbols`
(divan, release): `document_symbols_walk` **9.33 µs** median over the
fixture document (two lowered strings, one unknown root — five
candidates, each through the per-candidate compile + the CSS emit the
detail renders): the per-candidate walk the colour/hint faces share,
one more answer off the same scaffold.

## Consumer examples

`crates/aeolus-lsp/examples/lsp_symbols.rs` (the headless outline over
the state machine: initialize → didOpen → the symbol request — the
message sequence a client drives when the panel opens — beside the
no-strings empty answer):

```sh
cargo run -p aeolus-lsp --example lsp_symbols
```

## Docs

README's LSP row and crate map name the outline face beside the
rename/formatting ones, the examples index carries the demo row, and
the crate docs' question list gains the thirteenth bullet (the
engine's own module docs, the list the other faces keep).
