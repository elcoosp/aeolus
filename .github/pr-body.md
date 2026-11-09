# feat: consumer-examples-handoff — v0.9 doc alignment, typed theme errors, runnable examples

## Milestone — v0.9.1 "consumer handoff"

The tree carries the v0.8 per-edge set-mask and the v0.9 variant & grid engine
in-tree, but the surfaces a consumer reads told different stories: the README
quoted both 157 and 193 for the golden corpus and still listed the arbitrary
grid templates as deferred (v0.9 landed them), the Learn-more section linked
neither `AEOLUS_V0.8.md`, `AEOLUS_V0.9.md` nor the V1 architecture plan (and
its older links pointed at non-existent root-level `AEOLUS_V*.md` paths), the
CHANGELOG header only said "v0.8", the CLI rustdoc cited `AEOLUS_V0.9.md` §3
as the CLI plan of record (it is `AEOLUS_V0.7.md` §3), and the V1 architecture
doc's evidence header still claimed a "v0.7.0 tree". This PR aligns every one
of those surfaces to the v0.9-in-tree reality, clears the two micro-debt items
found in the survey, and adds the missing consumer surface — runnable
`examples/` — so a new consumer has working entry points for the first time.
The V1 zero-runtime track (GPUI, `aeolus-syntax`/`aeolus-ir`/codegen, `xtask`)
stays documented as future work; nothing of it exists in this tree.

### What changed

- **examples/** (net-new): `crates/aeolus-core/examples/{resolve,plan_algebra,cn_merge}.rs`
  and `crates/aeolus-xilem/examples/styled_label.rs` — resolve → slots/CSS/snapshot,
  overlay + diff lanes + artifact round-trip, `cn()` conflict merge, and the
  headless Masonry lowering + `.tw()` sugar. README's Usage section lists them.
- **Typed theme-spec errors**: `aeolus_cli::load_theme` now returns
  `ThemeSpecError` (UnknownSpec / BuiltIn / ReadFile / ParseCss) instead of a
  bare `String`. Display text is unchanged, so the `run()` stderr contract
  (pinned by the existing tests) is preserved; bare-word specs that are not
  built-ins now produce a precise "unknown spec" message instead of a
  misleading file-not-found.
- **Opt-in instrumentation seams** (off by default — the default dependency
  graph stays zero-dep): CLI feature `miette` (structured `Diagnostic`
  code+help on `ThemeSpecError`) and `tracing` (debug spans around command
  dispatch and the per-file `check` loop). The `aeolus-xilem` backend already
  carries `tracing`; these bring the headless surface to the same bar without
  taxing default builds.
- **Micro-debt**: `muda-stub`'s crate-level `#![allow(dead_code)]` /
  `#![allow(clippy::new_without_default)]` moved into its own `[lints]` table
  with rationale — the workspace-wide "zero `#[allow]` in `crates/*/src`"
  grep claim holds again (the stub is not a workspace member; its allowances
  are deliberate and now documented where allowances belong).
- **Docs**: README (corpus 193 in both places; deferred-items list updated —
  grid templates LANDED, V1 track explicitly PLANNED-not-present; Learn-more
  links V0.8/V0.9/V1 + fixed `docs/release-notes/…` targets; test count
  re-measured 444/446; runnable-examples section), CHANGELOG (Unreleased
  header covers v0.8+v0.9; v0.9.1 subsection), CLI rustdoc plan-of-record
  reference, V1 architecture evidence header (F1/F2/F6/F8 re-verified at
  `47ecfba`).
- **Benches**: `theme_spec_load_builtin` (~115 µs — the once-per-process
  embedded-palette parse) and `theme_spec_error_render` (~118 ns) in
  `benches/pipeline.rs`.

### Gates (all green on the dirty tree, pre-handoff)

- `cargo fmt --all --check` — clean.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` —
  zero warnings. Pedantic is enforced through the workspace
  `[lints.clippy]` table (`pedantic = warn`, per-lint exceptions with
  rationale), denied by `-D warnings`. Note on the invocation: appending
  `-W pedantic` after `--` (unqualified) hard-errors on
  `renamed-and-removed-lints` under `-D warnings`, and the qualified
  `-W clippy::pedantic` re-arms pedantic lints AFTER cargo emits the
  `[lints]` flags, stomping the documented per-lint exceptions (verified:
  `many_single_char_names` fires on sanctioned code); `-W` before `--` is
  rejected by cargo. The table-driven form is the working pedantic gate —
  verified by the same run erroring on any stomped allow.
- `#![allow|expect|cfg_attr(allow)]` grep over all `.rs` in `crates/`, `fuzz/` —
  **empty**.
- `cargo test --all-features --workspace` — **446 passed, 0 failed**
  (444 under default features; the delta is the feature-gated CLI tests).
- `cargo test --benches` (divan, stable — no nightly-only benches exist;
  the `cargo-fuzz` targets are CI-gated nightly and are not benches) —
  bench binaries compile and run green.
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` and
  `cargo test --doc -p aeolus-core` — clean (CI parity, bonus).

### Per-file history lives on the branch

Merging is `--rebase` with **one commit per logical change** landed as
Conventional-Commit commits on `feat/consumer-examples-handoff` (docs / test /
feat(examples) / refactor(cli) / chore(lints) / chore(release)), so the
per-file history survives the rebase onto `origin/main`; the review shows the
same tree this PR body describes. **No commits have been made on the base
branch for this work** — the whole change ships as the branch's commits
authored by `ship.sh` from the dirty tree it was handed.

### Verify locally

```sh
cargo run -p aeolus-core --example resolve
cargo run -p aeolus-core --example plan_algebra
cargo run -p aeolus-core --example cn_merge
cargo run -p aeolus-xilem --example styled_label
cargo test --all-features --workspace
```
