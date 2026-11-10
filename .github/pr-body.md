# feat: p2-ir-cn-merge — the ir-level cn() merge

## Milestone — P2's third slice: the `cn()` engine at IR level

The V1 pipeline's first two seams landed with one overlap story left
open: `compile()` emits the raw cascade (the v0.11.0 contract,
byte-identical), and the overlap a user's override re-sets was
visible only as `Program::base_conflicts()`'s bits. This change lands
the slice the plan's §3.2 row names ("hosts the `cn()` engine at IR
level") and §3.3 mechanizes ("mask algebra + copy"):

```rust
let base = compile("p-4 bg-primary", &Config::default())?;
let user = compile("px-8 bg-red-500", &Config::default())?;
let merged = cn(&[&base, &user])?; // rightmost wins per longhand
```

`aeolus_compile::cn(&[&Compiled])` folds same-condition layers by
`Cond` equality, concatenates declarations left-then-right, and keeps
the LAST occurrence per longhand — `cn(compile("p-4"),
compile("px-8"))` is the four-longhand cascade `p-4 px-8` resolves
to, the result comes out conflict-free (`base_conflicts()` empty),
specificity-sorted and interned as the dynamic form. The differential
contract is RED-pinned: merging resolves exactly what compiling the
concatenated string resolves to, and the declarations the merge
removes are precisely the conflict bits the raw concatenation names.
The audited `tw_merge` conflict table stays at the string level
(`aeolus_core::cn`, the CLI's `cn` command) — the longhand fan-outs
ARE the groups at this level. The merge is idempotent and
associative (pinned), diagnostics concatenate in input order with
their original spans, and theme-tier token values merge with the
rightmost alpha riding the `Tok` slot.

### What changed

- **`cn()` + `CompileError::NoInputs`** (the empty slice is a caller
  bug, same family as the empty string; the `miette` feature wires
  `aeolus::compile::cn_no_inputs` code + help). `tracing` spans on
  the merge (inputs, layers, diagnostics) — the workspace's opt-in
  pattern, OFF by default.
- **Micro-debt cleanup (CONTRIBUTING §2/§3), both verified by
  reading, no toolchain needed:**
  - `variant::fold` returned `Result<Cond, String>` — a stringly
    typed miss on the seam the diagnostics layer consumes. It now
    returns `Result<Cond, VariantMiss>` (a struct carrying the
    refusing segment, the same typed shape every other stage reports
    through). The rendered diagnostics are unchanged (pinned by the
    existing variant suites).
  - `ThemeError`'s T1 budget refusal was smuggled through the parse
    arm: a fabricated `ParseError::new("the dark pair surface
    exceeds the IR's u16 ModePair slot")` inside the build loop — a
    build boundary dressed as a parse failure, against the crate's
    own "parse errors are carried verbatim, never stringly
    re-wrapped" contract. It now has its own typed variant,
    `ThemeError::PairOverflow { pairs }`, with the always-on
    `Display` + source-less `Error` chain and the
    `aeolus::theme::pair_overflow` miette code + help. The
    `baked()`/`runtime()` rustdocs no longer claim the error is
    impossible.
- **Consumer example**: `crates/aeolus-compile/examples/cn_merge.rs`
  (the twelfth runnable example) walks the whole story — raw cascade
  vs merge, the conflict bits before/after, diagnostics
  passthrough, the theme-tier merge. The bench family gains
  `cn_button_surface_with_override_cold` (divan, stable, no nightly
  gating): the shadcn Button surface merged with a user override.
- **Docs drift, everywhere it was found:**
  - The README's version badge and the status paragraph still said
    **v0.10.0** after the v0.11.0 release (the release commit touches
    the manifests only); both now say v0.11.0 and record the cn
    merge's landed reality.
  - The CHANGELOG carried the git-cliff `[0.11.0]` block ABOVE the
    file title plus an `[Unreleased]` still describing the
    already-released theme pass — consolidated into one `[0.11.0]`
    section (the drift pattern the v0.10.0 pass fixed, recurred by
    the release tooling); the new `[Unreleased]` is this change.
  - The hot-path bullet's test counts were re-counted from the tree
    by `#[test]` attribute: 544 (552 with the instrumentation
    features) — the previous pass printed 535/542 while the tree
    carried 531/538, so the recount also fixes that stale delta.
  - The README's "1.88 gates core/macro/cli" claim had NO CI job
    behind it (`msrv-core` checked only core+macro; the CLI sat in
    neither MSRV job). `msrv-core` now checks `-p aeolus-cli`.
  - The architecture plan's P2 status block and §8 row record the cn
    merge's landed reality (and the PairOverflow boundary); the
    consumer-examples block lists the new example (twelve).
- **TDD order preserved; gates capture at handoff.** The suites were
  authored FIRST as the RED specs (12 cn-side tests: the merge, the
  differential contract, condition folding, honest reports, the
  algebra; + 2 theme-side pins for the PairOverflow surface, 1 of
  them gated), the implementations second, the bench and the
  consumer example last. This handoff's sandbox has no Rust
  toolchain, so the RED→GREEN capture and the gate outputs below run
  with the first cargo invocation in the gate environment; no
  `Cargo.lock` or manifest change is involved (no new dependencies,
  no version bump in the manifests — ship's `[version]` owns the
  bump), so the run stays `--locked`-clean.

### Gates (static verification in the no-toolchain sandbox; executable capture at handoff)

- `cargo fmt --all --check` — **not run here** (no cargo; the task
  forbids toolchain execution in this sandbox). Authored to rustfmt
  style; capture at handoff.
- `cargo clippy --workspace --all-targets --all-features -- -D
  warnings -W pedantic` — **not run here, and statically the command
  as written is the WRONG shape**: rustc resolves command-line lint
  specs positionally ("flags on the right take precedence", rustc
  book; groups expand at their position), and cargo turns the
  `[lints]` table into flags placed BEFORE the `--` extras (clippy
  has no knowledge of the lint table — rust-clippy#11237, resolved as
  cargo#12918). A trailing `-W pedantic` therefore re-arms the whole
  group OVER the manifest's per-lint allows (`cast_possible_truncation`
  & co.) — the previous pr-body's "a per-lint exception always
  outranks its group regardless of flag order" was WRONG, and this
  handoff corrects it. The workspace's pedantic gate is carried by
  `[lints.clippy]` itself (`pedantic = { level = "warn", priority =
  -1 }` + the rationale-carrying allows), which is RFC 3389's
  higher-precedence surface; the executable gate is therefore
  `cargo clippy --workspace --all-targets --all-features -- -D
  warnings`, where the `warnings` group's order-ignoring exception
  (rustc book: "the `warnings` group ... applies to all lints that
  would otherwise warn within the entity") makes every
  manifest-warned pedantic violation FAIL — stricter than a trailing
  `-W pedantic`, which only re-warns. "Moving `-W pedantic` before
  `--`" is not a cargo invocation (cargo's rustc passthrough is the
  `--` surface); the manifest-owned form is the correction.
- Allowance grep (RUN here, it is not a toolchain execution):
  `grep -rnE '#!?\[(allow|expect)\(|cfg_attr\([^)]*_allow\(' src/
  crates/ tests/ benches/ examples/ build.rs xtask/` → **empty** (the
  new suites and example included; zero allowances in code, all
  exceptions in the manifest tables with rationale).
- `cargo test --workspace --all-features` — **not run here**. The net
  new is 13 ungated tests (12 cn-side + 1 theme-side Display pin)
  plus 1 gated (the PairOverflow miette pin): the suite count moves
  531 → 544 without features (552 with), re-counted by `#[test]`
  attribute (see the drift notes: the previous docs pass's 535/542
  was stale). Every test's inputs were cross-checked against the
  committed tables by hand (the spacing math: 4×0.25 = 1rem,
  8×0.25 = 2rem; `Prop::Background` for the colour token; the
  `Cond` specificity ranks probe 2 < media 4; the shadcn `primary`
  token).
- `cargo test --benches --all-features` — **not run here**. All
  benches in the workspace remain divan and stable; the new bench
  adds no nightly gating. **No nightly-only benches exist** — the
  only nightly surface is the `cargo-fuzz` job (six targets), which
  is CI-gated and not a bench.
- Feature matrix: **no mutually-exclusive features exist anywhere in
  the workspace** (cli/data/compile/theme carry the additive
  `tracing`/`miette` optional-dep switches; macro's
  `verbose-diagnostics` is additive; muda-stub's gtk/libxdo/serde are
  stub-surface flags) — so `--all-features` is safe and there is no
  `ci-bundle` to assemble. The one deliberately non-portable wiring
  remains `[patch.crates-io] muda = { path = "crates/muda-stub" }` —
  the dev-sandbox stub that must never ship (PUBLISHING.md),
  unchanged here.
- Cargo.lock: **unchanged** — no new dependencies (the example, the
  bench and the suites consume the crates' existing dependency
  edges), so `cargo test --locked` resolves before the first update.

### Per-file history lives on the branch

Merging is `--rebase` with **one commit per logical change** landed as
Conventional-Commit commits on `feat/p2-ir-cn-merge` (the test pins,
the merge feat, the variant-miss refactor, the theme fix, the bench,
the examples feat, the docs alignment, the CI gate, the pr-body
chore) — the grouping is declared in `ship.toml` at the repo root, so
the per-file history survives the rebase onto `origin/main`; the
review shows the same tree this PR body describes. **No commits have
been made on the base branch for this work** — the whole change ships
as the branch's commits authored by ship from the dirty tree it was
handed.

### Verify locally

```sh
cargo run -p aeolus-compile --example cn_merge
cargo test --workspace --all-features
cargo test --benches --all-features
cargo bench -p aeolus-compile
```
