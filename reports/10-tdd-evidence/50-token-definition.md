# 50 — The token goto-definition face

The wire's `textDocument/definition` — the editor's ctrl-click on a
theme-token class answered. The engine answered what a class means
(the hover), what it lowers to (the swatches, the hints, the legend)
and where else it is spelled (the highlights, the references, the
rename); nothing answered WHERE A TOKEN LIVES — the jump a
Tailwind-class editor makes into the theme file. The landing answers
it from the compiler's own data: the candidate under the cursor (the
shared locator), the compile walk's clean verdict, the utility half
decomposed onto the registry's colour-group roots (the colour
presentations' composition walked backwards — the same rows, the same
roots, never a parallel matcher), the remainder verified through the
riding table (`TokenTable::id`), and the site the theme pass's own
provenance scan answers.

## The seam

`TokenProvenance` lands in `crates/aeolus-theme` (`src/provenance.rs`):
the name → declaration-site map built over the SAME text
`ThemeSource::from_css` parsed, one scan, the reader's own names the
single source of truth. The scan's honesty contract is the
no-silent-failures rule applied to positions:

* a token the sheet does not declare (the seeded palette names) answers
  NO origin — never a guessed span;
* a key mentioned inside a comment, a quoted string or a `var()`
  reference is not a declaration (an offset-true exclusion walk —
  comments and quoted strings are excluded WITHOUT being stripped, so
  every position stays byte-true);
* the prefixed `--color-{name}` form wins over the bare shadcn
  `--{name}` form — the reader's own `color()` lookup order;
* the boundary rules hold: a longer key (`--primary` inside
  `--primary-foreground`) and a `var(--color-primary)` value are not
  the declaration;
* the dark sheet's overrides ride the light site (the re-parsed
  variant re-answers the light names — the light declaration is the
  canonical site);
* the built-in palettes carry NO origins (the compiler's own data — no
  file to jump to, no origin answered);
* positions ride the LSP's own column model (zero-based line, UTF-16
  column, the key's own width) — the theme source is the authority for
  its own positions, and a host relays them verbatim.

The scan rides the source (`ThemeSource::provenance`) and the table it
bakes (`TokenTable::origin`, the `Arc` sharing the map across clones —
the ids contract untouched, the equality fast-path untouched: provenance
is not an answer-shape fact).

## The wire

`AeolusEngine::definition(uri, line, character) -> Option<EngineDefinition>`
answers ONE site (the light sheet's canonical declaration) — the wire
maps the Option onto the protocol's own single `Location` shape, `null`
the miss, a URI the `Uri` parser rejects the same null (the publish
half's own rule). `EngineConfig::theme_origin` names the origin URI
(the file the table was parsed from); `Session::with_theme_loader`
composes the loader's URI in automatically (the watched file IS the
definition's target — one loader, one file), and the watched-file
retheme re-notes it (idempotent). The face stays dark until a load
lands: the built-in shadcn bake is no file, and its table carries no
origins either way.

The host seam rides beside the config path: `TokenOrigins` (the
`ThemeLoader` precedent — the host's own statement of where a resource
lives, the built-in a consumer of the seam, never its definition),
wired through `AeolusEngine::with_token_origins`. The store rides
FIRST; the config's provenance path is the fallback; the compile gate
rides ahead of both. The built-in `ProvenanceOrigins` wraps a
host-owned table + URI for the embedding host.

## RED → GREEN

* RED: `crates/aeolus-theme/tests/provenance.rs` (13 pins, written
  first — the missing `provenance()`/`origin()` accessors and the
  `TokenOrigin`/`TokenProvenance` types WERE the capture: the byte-true
  site, the bare shadcn key, the prefixed form's precedence, the seeded
  token's dark, the comment/quoted/var-reference non-declarations, the
  prefix boundary, the dark sheet's light site, the built-ins'
  emptiness, the table's ride (baked and runtime), the UTF-16 column
  over the emoji neighbour, the purity pin).
* RED: `crates/aeolus-lsp/tests/definition.rs` (18 pins, written first
  — the missing `definition` method, `EngineDefinition` and
  `TokenOrigins` WERE the capture: the declaration site's URI and
  range, the variant-prefixed class, the refused class, the non-token
  class, the seeded token's dark, the theme-less boundary, the unnamed
  origin, the dark-pair class, the unknown URI, the purity pin, the
  host store's precedence, the store's compile gate, the wire's
  capability advertisement, the single-Location answer, the
  outside-candidate null, the pre-handshake gate, the malformed null).
* GREEN: the scan in `crates/aeolus-theme/src/provenance.rs`, the
  accessors in `ThemeSource`/`TokenTable`, the engine face + the
  `TokenOrigins` module in `crates/aeolus-lsp`, the wire arm + the
  `definitionProvider` capability in `server.rs`.

## Bench

`definition_walk` joins `crates/aeolus-lsp/benches/lsp.rs`: the cursor's
token class through the shared locator, the compile gate (the pooled
walk's miss path), the colour-root decomposition and the provenance
lookup — the cost one ctrl-click on a theme-token class pays.

## Example

`crates/aeolus-lsp/examples/lsp_definition.rs` — the theme file's table
baked with the origin URI named, the bare and variant-prefixed classes
answering their declaration sites, the non-token and seeded classes
answering the honest dark, the closed URI, headless.

## Boundaries kept

* the definition answers the LIGHT sheet's site; a dark override rides
  the same site (the token's canonical declaration);
* the answer set is CLOSED by the compile walk + the table + the scan:
  a class the pipeline cannot lower, a root the registry does not name
  a colour group for, a token the file does not declare — all `None`,
  never a guessed span;
* a host store overrides the provenance path wholesale; the compile
  gate stays ahead of every source.
