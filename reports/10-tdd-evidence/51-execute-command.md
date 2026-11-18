# 51 — The command surface

The wire's `workspace/executeCommand` (LSP 3.16) — the server's command
surface, the first wire capability the client can DRIVE rather than
ask. The server answered requests and streamed publishes; an editor
keybinding, a refactoring menu or a code-action's follow-up had no
extension point to invoke — the protocol's command channel existed for
exactly that, and the server advertised nothing. The landing opens it
honestly: a CLOSED, advertised command table (`executeCommandProvider`
carrying `aeolus.sortDocument` — the class-sort seam's editor face,
already landed as the `textDocument/formatting` request, invoked here
as a command), the same engine answer the formatting request returns,
byte for byte.

## The shape evidence

The command table is closed and the capability is its contract:
one command, the sort face, no resolve round-trip (the edits are
final — the formatting request's own contract). The dispatch rides the
state machine's own vocabulary: the unknown command and the missing
argument answer the protocol's own null (the quiet rule every
unhandled request keeps — no invented command runs, no guessed edit
ships); the malformed params the same. The known command parses the
arguments' FIRST string as the URI (the command's own single-argument
shape) and hands it to `AeolusEngine::format` — the sort seam's
canonical join over every out-of-order class string, one `TextEdit`
per string, the already-canonical strings answering nothing (the no-op
contract a keybinding's second press rides).

## RED → GREEN

* RED: `crates/aeolus-lsp/tests/execute_command.rs` (8 pins, written
  first — the missing `executeCommandProvider` capability and the
  missing `workspace/executeCommand` arm WERE the capture: the
  advertised command table, the edits matching the engine's own
  `format` answer, the already-canonical document's empty list, the
  unknown URI's empty list, the unknown command's null, the missing
  argument's null, the pre-handshake gate, the malformed null).
* GREEN: the `execute_command_provider` capability + the
  `workspace/executeCommand` arm in `crates/aeolus-lsp/src/server.rs`
  (the typed `ExecuteCommandParams` parse, the closed table's match,
  the argument's URI, the shared `TextEdit` mapping).

## Bench

`execute_command_sort` joins `crates/aeolus-lsp/benches/lsp.rs`: the
command round trip through the state machine — the params parse, the
sort walk, the wire mapping and the edit serialization (the cost one
keybinding invocation pays).

## Example

`crates/aeolus-lsp/examples/lsp_execute_command.rs` — the command
answering the two out-of-order strings, the closed table's nulls (the
unknown command, the missing argument), the already-canonical
document's empty list, headless.

## Boundaries kept

* the table stays closed: a new command is a new capability, never a
  stringly fallthrough;
* the command answers the engine's own answer — no parallel edit shape,
  no second sort implementation;
* a command the table does not know is the protocol's own null, the
  state machine's quiet vocabulary, never a fabricated result.
