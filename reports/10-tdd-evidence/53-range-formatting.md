# 53 — The range-formatting face

The sort seam's SELECTION-SCOPED form (`textDocument/rangeFormatting`,
LSP 3.12). The formatting face answered the whole document
(`textDocument/formatting`: one edit per out-of-order class string, the
canonical join as the new text) — and an editor that formats a
SELECTION, a line or the block the cursor sits in had no answer that
respects the range: the whole-document walk reorders strings the
selection never touched, a partial selection over one string had no
word-level answer. The landing opens the range honestly: the SAME walk
(the engine's own `format` walk, no parallel sort) restricted to the
class strings the requested half-open byte range TOUCHES — a string
whose span intersects the range rides its WHOLE-string edit (the sort
seam reorders the string, never a fragment), a string wholly outside
the range stays out.

## The shape evidence

The answer is the engine's `format_range`: the document's strings
filtered by span intersection with the requested range (the positions
converted once, the engine's own byte-true offset walk), then the
formatting face's own boundary list, unchanged: an already-canonical
string answers NO edit wherever the range sits (the no-op contract a
selection-scoped format-on-save rides), the whitespace normalization is
the canonical join's own, the misses never block the edit and never
ride this channel (the diagnostics' story), and the edit's range is the
string CONTENT's span, byte-true in the LSP's UTF-16 pairs. The range
itself answers honestly: a reversed range (an end before its start) is
no selection and answers the empty list, a position past the document's
last line likewise (no invented walk), and the whole-document range IS
the formatting request's own answer, byte for byte — the differential
pin the battery keeps. At the wire the capability rides beside the
formatting provider (`documentRangeFormattingProvider`, the plain
`true` form — the answer is the engine's own walk, no options object),
the params' formatting options are accepted and left to the
shape-level walk (the canonical order is a registry fact, not an
indentation choice — the formatting arm's own rule), and the malformed
params answer the protocol's own null.

## RED → GREEN

* RED: `crates/aeolus-lsp/tests/format_range.rs` (11 pins, written
  first — the missing arm WAS the capture: the advertised provider,
  the whole-document range matching the formatting answer byte for
  byte, the one-string range, the partial-overlap pin (the middle of a
  string still names the string), the two-string range, the
  already-canonical range's empty, the range outside any string, the
  reversed range, the unknown URI, the pre-handshake gate, the
  malformed null).
* GREEN: `AeolusEngine::format_range` in
  `crates/aeolus-lsp/src/engine.rs` (the intersection boundary beside
  the format walk's own edges), the `textDocument/rangeFormatting`
  arm + the `documentRangeFormattingProvider` capability in
  `crates/aeolus-lsp/src/server.rs`.

## Bench

`format_range_slice` joins `crates/aeolus-lsp/benches/lsp.rs`: the
range over the second string of the bench document — the intersection
boundary, the sort walk, the wire mapping and the edit serialization
(the cost one selection-scoped format-on-save pays).

## Example

`crates/aeolus-lsp/examples/lsp_range_format.rs` — the range over one
string, the partial selection, the two-string range, the honest
empties (the canonical line, the reversed range) and the whole-document
parity read-out (the formatting answer byte for byte), headless.

## Boundaries kept

* the intersection is span-level, never token-level: a selection over
  the string's middle reorders the WHOLE string — the sort seam's own
  contract (the canonical join is the string's new text; a fragment
  sort would be a different, unshipped answer);
* the misses are not this answer's channel (the diagnostics publish
  them, push and pull — the formatter carries edits only);
* the theme-less walk is deliberate (the order is a shape-level fact —
  the sort module's own boundary), so a range-scoped sort needs no
  table;
* the wire's whole-document `textDocument/formatting` contract is
  untouched; the range form restricts, never replaces, it.
