# feat: p2-compile-seam — the lex-registry-IR compile seam, typed diagnostics

## Milestone — P2's first slice: the compile pipeline's first seam

The V1 data-driven core (phase P1: `aeolus-syntax`, `aeolus-data`,
`aeolus-ir`) landed git-only; this change lands the first slice of P2
(architecture plan §3.1/§3.2): **`aeolus-compile`**, the pure function

```rust
compile(&str, &Config) -> Result<Compiled, CompileError>
```

that runs class strings through the three P1 crates end to end — the
zero-copy lexer, the typed registry read side, the longhand value pass —
and emits cascade-sorted `Program`s with the IR's masks as data. The
theme and target halves of the plan's signature (`aeolus-theme` T0–T2,
the IR-level `cn()` merge, `@utility`/`@custom-variant`, the xtask
generator) are the remaining P2 slices; every seam this change touches
says so in its rustdoc. Nothing in the v0.x surface changes and no
default dependency graph grows (the new crate's default graph is
aeolus-syntax + aeolus-data + aeolus-ir only).

### What changed

- **`aeolus-compile`** (git-only, `publish = false`): the five-stage
  pipeline — lex (borrowed candidates, byte spans), variant fold (the
  flat-`Cond` model: dominant condition, source order breaks ties —
  `cond.rs`'s own contract, now implemented), registry read (the typed
  `lookup_detailed` miss modes, spanned), value pass (spacing rem/px
  configs per plan §3.3, fractions, numbers, row-table keyword slots),
  assembly (stable specificity sort, `Layer::new`-derived masks,
  interned as the IR's documented dynamic form). The registry semantics
  are pinned by a const assert at `REGISTRY_VERSION = "1"` — a registry
  semantics bump must re-audit the value pass, and the build says so.
- **Every class that does not lower comes back typed**: `Diagnostic`
  carries the token's byte span, a stable `aeolus::compile::*` code and
  a help text (CONTRIBUTING §2 applied to the pipeline). Boundaries:
  importance (`!`), `/modifiers`, colour tokens, arbitrary values,
  untyped variants (`has-`, `[a&]`, …) — each its own variant, never a
  dropped class. Two OFF-by-default instrumentation features land with
  the crate, exactly the CLI's v0.9.1 / registry's P1 pattern:
  `tracing` (spans on the stages) and `miette` (code + help on
  `CompileError`; the always-on `code()`/`help()` strings live on
  `DiagnosticKind`).
- **The seam surfaced the first concrete IR budget**: the colour
  keyword table is **270 entries** and `Val::Kw` is a **u8 slot** —
  colour tokens lower through the theme pass's `TokenTable`
  (plan §3.4 T2), typed and reported, never truncated. Also surfaced:
  the named group/peer label has no IR slot yet (the probe folds, the
  name threading is the codegen's), and the lexer's degraded pairs lost
  their spans (the seam re-derives them with the lexer's own sequential
  find). All three are test-pinned, documented in
  `reports/10-tdd-evidence/13-compile-seam.md`.
- **TDD order preserved; gates capture at handoff.** The suite
  (`tests/compile.rs`, 23 ungated + 2 feature-gated) was authored FIRST
  as the RED spec, the implementation second, the bench
  (`benches/compile.rs`: single utility, variant stack, the shadcn
  Button surface, the typed miss — divan, stable) and the consumer
  example (`examples/compile_pipeline.rs`) last. This handoff's sandbox
  has no Rust toolchain, so the RED→GREEN capture and the gate outputs
  below run with the first cargo invocation in the gate environment;
  the `Cargo.lock` entry for `aeolus-compile` is already in place so
  that run is `--locked`-clean.
- **Micro-debt cleanup (CONTRIBUTING §2/§3):**
  - `Candidate::signed_body` removed — dead code (zero callers across
    the workspace) whose doc contradicted its own implementation (it
    returned the sign-STRIPPED body while claiming to restore the sign;
    the sign rides `Candidate::negative` by design).
  - The `VERSION` consts in `aeolus-syntax` and `aeolus-ir` derive from
    the manifests now (`env!("CARGO_PKG_VERSION")`) — the hardcoded
    `"0.1.0"` had drifted three releases behind the workspace; each
    crate pins the derivation with a test.
- **Docs drift, everywhere it was found:** the README's version badge
  said **0.7.0** (three releases stale), the status paragraph said
  **v0.9.0** after the v0.10.0 release commit, the consumer-examples
  block listed 4 of the tree's 8 examples, and the test-count claim was
  stale (now the real 509; 513 with the instrumentation features); the
  crate table carries `aeolus-compile`. The CHANGELOG cuts **[0.10.0]**
  — the release commit shipped without its section — and this change
  gets its own `[Unreleased]`. The architecture plan records the P2
  seam's landed reality beside P1's (a P2 status block + the §8 row).
  Rustdocs stopped contradicting the code: `prop.rs`'s "96 longhands" →
  110 (and "32 bits" of headroom → 18), `hash_classes`' "used by both
  cache and interner" → what actually uses it, and the `tw!`
  span-preservation promise → the honest nightly-only ceiling (V1 plan
  §9 documents the same constraint).

### Gates (static verification in the no-toolchain sandbox; executable capture at handoff)

- `cargo fmt --all --check` — **not run here** (no cargo; the task
  forbids toolchain execution in this sandbox). Authored to rustfmt
  style; capture at handoff.
- `cargo clippy --workspace --all-targets --all-features -- -D
  warnings` — **not run here**. Pedantic is enforced through the
  workspace `[lints.clippy]` table (`pedantic = warn`, priority −1,
  per-lint exceptions with rationale) denied by `-D warnings` — the
  manifest-driven gate is the one that preserves this repo's documented
  lint contract (a literal `-W pedantic` on the command line both
  deprecation-errors on current clippy and would override the
  manifest's rationale-backed exceptions). Verified statically: the
  `[lints.clippy]` table is intact, and the
  `#!?\[(allow|expect)\(|cfg_attr\([^)]*_allow\(` grep over `crates/`
  is **empty** (the new crate included; zero allowances in code, all
  exceptions in the manifest tables with rationale).
- `cargo test --workspace --all-features` — **not run here**. The net
  new is 25 tests (23 ungated + 2 gated) in `aeolus-compile` plus the
  2 version-pin tests (syntax, ir): the suite count moves 482 → 509
  (513 with the instrumentation features). Every test's registry rows,
  tables and prop targets were cross-checked against the committed
  rows/tables by hand (TRBL orders, keyword slots, the 270-entry
  colour table).
- `cargo test --benches --all-features` — **not run here**. All benches
  in the workspace remain divan and stable; the new bench adds no
  nightly gating. **No nightly-only benches exist** — the only nightly
  surface is the `cargo-fuzz` job, which is CI-gated and not a bench.
- Feature matrix: **no mutually-exclusive features exist anywhere in
  the workspace** (cli/data/compile carry the additive
  `tracing`/`miette` optional-dep switches; macro's
  `verbose-diagnostics` is additive) — so `--all-features` is safe and
  there is no `ci-bundle` to assemble.
- Cargo.lock: the `aeolus-compile` member entry is in place
  (version 0.10.0, deps + optionals + dev-deps, alphabetical order
  preserved) so `cargo test --locked` resolves before the first
  update.

### Per-file history lives on the branch

Merging is `--rebase` with **one commit per logical change** landed as
Conventional-Commit commits on `feat/p2-compile-seam` (the compile-seam
feat carrying its workspace wiring, lock entry and bench, the test pin,
the examples feat, the dead-code refactor, the VERSION fix, the docs
alignment, the gitignore chore, the pr-body chore) — the grouping is
declared in `ship.toml` at the repo root, so the per-file history
survives the rebase onto `origin/main`; the review shows the same tree
this PR body describes. **No commits have been made on the base branch
for this work** — the whole change ships as the branch's commits
authored by ship from the dirty tree it was handed.

### Verify locally

```sh
cargo run -p aeolus-compile --example compile_pipeline
cargo test --workspace --all-features
cargo test --benches --all-features
cargo bench -p aeolus-compile
```
