# The Aeolus Book

The single ordered reader over Aeolus's documentation — the P5 phase's
**docs book** deliverable (`docs/AEOLUS_V1_ARCHITECTURE.md` §8's P5 row
and §5.8's governance line). The tree's other docs stay where they are:
the architecture plan of record, the migration guide, the generated
registry and conformance pages, the reports. The book organizes them
into one navigable whole and includes the generated pages verbatim, so
there is exactly one source for everything and no copied prose to drift.

## Build

```sh
cargo install mdbook
mdbook build docs/book
```

The rendered HTML lands in `docs/book/book/` (gitignored — the source
is the artifact, the render is disposable).

## Gate

The book carries the same drift discipline as every other committed
artifact:

```sh
cargo xtask gen-book      # regenerate the generated chapters (the CLI reference)
cargo xtask verify-book   # the full gate: chapter bytes + structural contract
cargo test -p xtask       # the same pins, in plain `cargo test`
```

`verify-book` checks both halves:

1. **The generated chapters** byte-match their regeneration. Today one:
   the CLI reference, rendered from `aeolus-cli`'s own `USAGE` — the
   page is the binary's help text, never a parallel claim.
2. **The structural contract**: every `SUMMARY.md` entry resolves to a
   chapter file, every `{{#include}}` target exists (mdbook resolves
   them at build time with `create-missing = false` — a moved page is a
   build error, not a silently-generated stub), and `book.toml` carries
   the `[book]` table with the `title`/`src` keys the build needs.

## What is hand-written and what is generated

Hand-authored (the prose): `introduction.md`, `pipeline.md`, `ir.md`,
`themes.md`, `variants.md`, `backends.md`, `tooling.md`,
`performance.md`. Generated (the seam): `cli-reference.md`. Included
verbatim from their committed pages (the single-source rule):
`registry.md`, `conformance.md`, `migration.md`.
