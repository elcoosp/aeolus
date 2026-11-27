# Introduction

Aeolus is a Tailwind v4 class compiler for native-Rust GUI toolkits. A
class string goes in one end and comes out the other as the style you
would have written by hand: a frozen `static` program the runtime maps
onto the toolkit's own fields, verification CSS, effective declarations
under a probe environment, or a plan the interpreter backends lower.
There is no parser, no hash table and no lock on the hot path — the
compile-time work is kept instead of thrown away.

Two contracts shape everything else. The first is **fidelity**: the
semantics are pinned against the real tailwindcss v4.1.17 through a
2992-entry golden corpus mined from upstream's own utility tests, and
the utility registry (247 rows over 160 longhands) is data, not code —
a contributor adds a row, the generator re-renders the committed
artifacts, and the drift gates fail a stale regeneration in plain
`cargo test`. The second is **honesty**: every class that does not
lower comes back as a typed, spanned diagnostic with a stable code and
a help text — never a silent drop. The same contract reaches the
backends (a longhand a target cannot express is reported, per
declaration, with a reason) and the docs (a boundary is stated, never
papered over).

## What the workspace looks like

The pipeline is a pure function over three git-only core crates:
`aeolus-syntax` (the zero-copy, `no_std`-capable lexer), `aeolus-data`
(the registry as auditable rows) and `aeolus-ir` (the longhand
`Program` IR — `Copy` types, `const`-constructible, no heap in the
static form). `aeolus-compile` assembles them into the pipeline the
plan's §3.1 sketched, `aeolus-theme` resolves CSS-first theme files
into tiered token tables, and `aeolus-macro` exposes the compile-time
front door (`tw!`, `cn!`, `tw_cn!`, `variants!`, `theme!`). The runtime
half is `aeolus-rt` (the frozen `StaticProgram`, the dynamic-lane
`ClassCell`, the bounded `DynCache`, the observability faces) and
`aeolus-gpui` (the native GPUI lane) beside the five interpreter
backends (`aeolus-xilem`, `aeolus-floem`, `aeolus-iced`,
`aeolus-egui`, `aeolus-slint`). Tooling shares the same pipeline:
`aeolus-cli` (twelve headless commands), `aeolus-lsp` (the editor
surface over the same engine) and `xtask` (the committed-artifact
driver).

The v0.x resolver stays in `aeolus-core` as the runtime lane and the
differential oracle — the migration ran without a big bang, and the
triangle (new compiler ⟷ old resolver ⟷ real Tailwind) remains the
correctness strategy's spine.

## Where to read next

Work through the pipeline chapters in order — [the V1
pipeline](pipeline.md), [the IR](ir.md), [themes](themes.md),
[variants](variants.md) — then the consumption half: [the
backends](backends.md), [the tooling](tooling.md), and the reference
part (performance, registry, conformance, CLI, migration). The
architecture plan (`docs/AEOLUS_V1_ARCHITECTURE.md`) stays the plan of
record; the book summarizes it and links rather than restates.
