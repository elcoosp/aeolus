# Tooling

Everything shares one pipeline — the CLI, the LSP and the macros run
the same compile walk, never a parallel matcher.

## The macro surface

`tw!` compiles the class string at expansion and emits the frozen
static program (the input string is never emitted back). The family:
`cn!`/`tw_cn!` merge at compile time, `variants!` builds the
closed-world component tables (with the `compound:` list and the
`slots:` map), `theme!` is the T2 runtime handle, and the neutral
`tw_static!`/`cn_static!`/`variants_static!` entries freeze without a
GPUI gate. The optional `theme: "path",` prefix bakes a host sheet at
expansion; the `aeolus.toml` `profile` knob picks the emit face
(`speed` inline vs `size` outlined) per crate.

## The CLI

Twelve headless commands over the same engine (the [CLI
reference](cli-reference.md) renders the binary's own help text):
`resolve`, `css`, `diff`, `cn`, `check`, `explain`, `sort`, `fmt`,
`expand`, `build`, `plan` and `audit`. The CI-shaped ones: `check` is
the lint gate (`--fail-on unknown`), `fmt --check` gates the canonical
class order, `audit --fail-on` keys on the capability tier.

## The LSP

`aeolus-lsp` answers from the same engine: hover, diagnostics,
completion, colour swatches, inlay hints, quick fixes, document
highlights, semantic tokens (full and range), formatting and
range-formatting, references, rename and token-rename, selection
ranges, linked editing ranges, folding ranges, document and workspace
symbols, goto-definition for theme tokens, `executeCommand`, colour
presentations, workspace diagnostics (the pull channel), incremental
sync, and the theme lifecycle (the watched-file save re-themes the
session). The engine carries none of the wire types; the stdio
JSON-RPC state machine is testable without a pipe, and the observability
faces (engine, walk, transport) stream the session's verdicts.

## The committed artifacts and their driver

`cargo xtask` is the generator story's driver: `gen-tables`/
`verify-tables` over the registry-derived artifacts (the roots index,
the registry page, the six conformance pages, the committed capability
tables), `gen-gpui-caps`/`verify-gpui-caps` over the GPUI capability
module, and `gen-book`/`verify-book` over this book. The drift pins run
in plain `cargo test` — a stale regeneration fails CI before a review
sees it.
