# 47 — The linked editing ranges

The simultaneous-edit face (the wire's
`textDocument/linkedEditingRange`) — the highlights' textual equality
rendered as the client's edit-together ranges. The editor's
multi-cursor class edit had no answer: the rename face moves a
spelling to a NEW one, the highlights answer to a human, but the
protocol's linked-editing gesture — "give me every range that must
change together when I type here" — went unanswered, and nothing else
speaks its contract. The landing adds the answer from the machinery
the engine already owns: the cursor's candidate located once through
the shared `candidate_span_at` walk (the hover's own locator), then
the highlights' textual-equality fold over the extraction seam's
strings — every occurrence of that raw spelling in the REQUESTING
document riding one plain `EngineRange`, the cursor's own included,
document order kept (the extraction seam's scan order). The identity
is TEXTUAL, the rename face's own token rule: a variant-prefixed
sibling (`hover:bg-primary` beside `bg-primary`) is a DIFFERENT
spelling that links alone, and a refused candidate's occurrences
answer exactly like a lowered one's (no compile verdict riding). The
scope stays same-file — the cross-document fold is the RENAME face's
answer, one face one scope; the protocol's same-file contract is what
`linkedEditingRange` names. The wire half advertises
`linked_editing_range_provider` (the plain `true` form) and the
`textDocument/linkedEditingRange` arm maps the answer onto the
protocol's own `LinkedEditingRanges` shape — no `word_pattern` rides
(the ranges ARE the contract; the engine's textual identity needs no
regex re-statement). Byte-truth rides the engine's own position model
(the multi-byte neighbour counting its pairs, not its bytes).

## The shape evidence

The walk is the highlights' body read again with the answer's own
shape: the same `scan_rust_strings` hits, the same
`parse_class_string` candidate fold, the same raw-equality guard —
no parallel scaffold to drift. The empty answers keep their shapes:
an unknown URI, an off-document cursor and a cursor resting between
classes (inside a string but outside any candidate span) all answer
EMPTY — the protocol's own nothing-to-link, distinct from an error.
The wire arm rides the shared `wire_range` conversion (the ONE
mapping every range-shaped answer takes) and the pre-handshake gate
every request arm keeps; the malformed-params answer stays the
protocol's own null (the wire's established empty answer).

## RED → GREEN

* RED: `crates/aeolus-lsp/tests/linked_ranges.rs` (10 pins, written
  first — the missing `linked_ranges` method WAS the capture: the
  plain-spelling occurrences riding in document order with the
  cursor's own among them, the variant-prefixed sibling linking
  alone, the refused candidate riding, the between-classes gap
  answering empty, the unknown URI's empty, the byte-true UTF-16
  columns over the accented neighbour, and the wire half's ranges
  quads/capability advertisement/pre-handshake gate/malformed null).
  The fixtures taught the identity model twice during the cycle: the
  plain cursor's answer counts the PLAIN spellings only (the
  variant-prefixed sibling excluded — the pin re-counted from 3 to
  the honest 2), and the between-classes probe arithmetic had to
  land on the separator itself.
* GREEN: the same 10 pins pass against the landed walk; the lsp
  crate's full suite re-measured green alongside the reload battery
  (the handoff's own counts in report 46).

## Bench

`crates/aeolus-lsp/benches/lsp.rs` (divan, release, divan 0.1.21,
median of 100): `linked_ranges_walk` ~1.27 µs over the bench's own
document — the highlights' scan folded to ranges, the cost a
multi-cursor linked-edit gesture pays per invocation (the locator's
scan plus the string walk, the compile walk's verdict nowhere on the
path — the identity is textual).

## Files

* `crates/aeolus-lsp/src/engine.rs` — `linked_ranges` (the highlights'
  fold, ranges out).
* `crates/aeolus-lsp/src/server.rs` — the
  `textDocument/linkedEditingRange` arm, the
  `linked_editing_range_provider` capability.
* `crates/aeolus-lsp/tests/linked_ranges.rs` — the 10-pin RED battery.
* `crates/aeolus-lsp/benches/lsp.rs` — `linked_ranges_walk`.
* `crates/aeolus-lsp/examples/lsp_linked_ranges.rs` — the headless
  consumer drive (the plain pair, the variant sibling, the refused
  ride, the gap's honest empty).
