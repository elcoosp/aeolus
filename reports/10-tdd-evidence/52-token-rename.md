# 52 — The token-rename face

The command surface's second command (`workspace/executeCommand`'s
`aeolus.renameToken`) — the editor's first THEME-TOKEN identity walk.
The rename face answered the textual question (`textDocument/rename`:
one edit per textual occurrence of the cursor's candidate), the
definition face answered where a token LIVES (`textDocument/definition`:
the theme file's own declaration site) — and nothing answered the
question between them: "rename the TOKEN". `bg-primary` and
`dark:text-primary` are different spellings and the same token; a
workspace rename of `--color-primary` → `--color-brand` means every
colour-group spelling whose remainder the table carries, across every
open document, with the variant prefixes and `/opacity` modifier tails
preserved byte-true.

## The shape evidence

The identity is the compiler's own vocabulary, factored ONCE: the
definition face's decomposition walk (the variant stack's tail — the
last `:` segment — the `/opacity` modifier stripped, decomposed onto the
registry's colour-group roots, the remainder verified through
`TokenTable::id`) gained the token's OWN byte span beside its name
(`token_span_at` in the engine, `token_under` now its name-only
consumer — no parallel matcher). The fold rides the same walk per
candidate over every open document: candidates whose decomposition names
the SAME token ride one edit over the token's own span (the root stays,
the prefix stays, the modifier tail stays — only the token's bytes move).
The determinism is the references fold's own: URI order across the map,
document order within each.

Every miss is honest: a non-token class, a cursor outside any candidate,
a theme-less config (the identity IS the table), an empty new name (the
textual rename's own validity clause — never a deletion walk) all answer
the empty fold. The NEW spelling is the editor's business (a token the
theme file is about to declare): the fold keys on the OLD token's
identity, the diagnostics channel telling the truth about what the new
spellings lower the moment the didChange lands. At the wire the command
rides the CLOSED command table beside `aeolus.sortDocument` (the table
stays closed — two commands, both advertised), the arguments the
rename request's own shape ([uri, line, character, newName]), a missing
or mistyped argument the protocol's own null, and the answer the rename
face's own `changes` map — the SAME fold the rename request serializes,
its keys riding the engine's own URI order (the byte-determinism fix
below lands on both faces at once).

## RED → GREEN

* RED: `crates/aeolus-lsp/tests/token_rename.rs` (14 pins, written
  first — the missing `rename_token` API WAS the capture: the
  advertised two-command table, the workspace edit map (both carrying
  documents, URI order), the byte-true span pin, the splice-through
  pin (`hover:bg-brand/90`, `border-brand`, `text-brand
  dark:text-brand`), the engine/wire parity, the non-token cursor, the
  cursor outside any candidate, the theme-less config, the empty name,
  the name the table does not carry, the unknown URI, the closed
  table's null for an unknown command, the malformed arguments' null,
  the pre-handshake gate).
* GREEN: `token_span_at` + `AeolusEngine::rename_token` in
  `crates/aeolus-lsp/src/engine.rs` (`token_under` the delegated
  consumer), the `aeolus.renameToken` command arm + the two-command
  `executeCommandProvider` in `crates/aeolus-lsp/src/server.rs`, the
  rename request arm folded onto the same `changes_map`.
* The pinned contract the upgrade touched: the execute-command battery's
  one-command table pin reads the two-command table now
  (`crates/aeolus-lsp/tests/execute_command.rs` — the battery's own
  file pins the fold, this one keeps the closed-table shape).

## Bench

`rename_token_fold` joins `crates/aeolus-lsp/benches/lsp.rs`: the
decomposition walk over the cursor's candidate plus the fold over every
open document (one scan per document, each candidate's utility half
decomposed onto the colour-group roots — the cost one rename invocation
pays).

## Example

`crates/aeolus-lsp/examples/lsp_token_rename.rs` — the command answering
the two-document map (four usages of one token across three roots and
two variant prefixes), the spliced result printed beside, the honest
misses (the non-token cursor, the unknown command, the mistyped
argument), headless.

## Boundaries kept

* the identity is the TABLE's own membership (`TokenTable::id` through
  the colour-group rows) — a spelling that decomposes to a token the
  theme file does not declare answers no definition and no fold (the
  honest dark), and a candidate that decomposes to a DIFFERENT token
  stays out of the fold (`bg-primary` never takes `primary-foreground`
  along — the remainder is the full name);
* the theme file's own declarations are NOT the fold's: the definition
  face jumps there, the editor renames there — the fold renames the
  class spellings over the open documents (the engine holds no theme
  text, the honest boundary);
* the arbitrary `bg-(--primary)` / `bg-[var(--primary)]` reference
  forms stay outside the fold (the colour-group decomposition is the
  audited surface; the bracketed forms are the value pass's own path);
* the wire's `textDocument/rename` contract is untouched — still the
  textual fold (a spelling rename, not a token rename); the token
  identity rides the COMMAND face only.
