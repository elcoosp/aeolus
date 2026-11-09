# feat: p1-consumer-examples — typed registry seam, consumer examples, drift alignment

## Milestone — the P1 consumer handoff, part two

The V1 data-driven core (phase P1: `aeolus-syntax`, `aeolus-data`,
`aeolus-ir`) landed git-only in the previous change; this change closes
the P1 story so P2 (the `aeolus-compile` pipeline) starts from an
honest, resolvable, consumer-ready tree — nothing in the v0.x surface
changes and no default dependency graph grows.

### What changed

- **Fixed the build breakage the v0.9.0 release shipped.** The release
  bump moved `[workspace.package]` to 0.9.0 but left the
  `[workspace.dependencies]` intra-workspace pins at 0.8.0; cargo
  enforces the version requirement against path dependencies, so every
  `aeolus-core = { workspace = true }` member failed resolution
  (`failed to select a version for the requirement aeolus-core =
  "^0.8.0"`) — the tree at HEAD did not build at all. The pins are back
  in lockstep (0.9.0), the manifest carries a comment stating the
  contract, `Cargo.lock` records the members at 0.9.0, and `ship.sh`'s
  BUMP step now updates both blocks together so a release can not
  strand them again.
- **`aeolus-data` grows the typed read seam** the crate docs promise:
  `registry::lookup_detailed` returns exactly `lookup`'s rows on a hit
  (pinned by tests) and, on a miss, a typed
  `LookupError::UnknownRoot` / `LookupError::ShapeRejected { root,
  tried }` — the deduplicated value-shape labels that were tried, with
  `AnyOf` arms flattened to their inner kinds (the RED test caught the
  first implementation labelling every row "any-of"). Two OFF-by-default
  instrumentation features land with it, exactly the CLI's v0.9.1
  pattern: `tracing` (debug spans on the read paths) and `miette`
  (`Diagnostic` code + help text: `aeolus::registry::unknown_root` /
  `::shape_rejected`). The default graph stays dependency-light
  (aeolus-ir only). Benched in `benches/lookup.rs` (silent hit, detailed
  hit, both misses — median 327.8 ns on the detailed hit vs 531.8 ns
  silent, misses 387–434 ns).
- **The three crates that had no consumer entry point get runnable
  demonstrations:** `crates/aeolus-syntax/examples/lex_candidates.rs`
  (borrowed candidates + byte spans, honest per-class degradation, the
  typed `LexError` misses), `crates/aeolus-data/examples/registry_lookup.rs`
  (rows, longest-prefix disambiguation, conflict groups, the typed
  misses, `generate_docs()`), and
  `crates/aeolus-cli/examples/theme_spec.rs` (the `--theme` seam as a
  library: `load_theme` + the typed `ThemeSpecError` contract).
- **Micro-debt cleanup (CONTRIBUTING §2: no unwraps on reachable
  public-API paths):** `eval_calc`'s shunting-yard pop now rides the
  same guard that scoped the loop, the bare-operator token is a
  let-else returning a parse error, and `Color::parse`'s 3/4-digit hex
  doubling has an explicit error arm instead of two provably-safe
  unwraps. A new test pins the calc()-edge typed-error contract
  (`calc(2px + )` / `calc(* 4px)` / `calc(2px + * 3px)` are parse
  errors, never panics) and the sane spellings still resolve.
- **CI drift:** the fuzz smoke loop sweeps all SIX `fuzz/Cargo.toml`
  targets — `parse_variant` and `probe_eval` were wired as targets but
  absent from the CI loop.
- **Docs drift, everywhere it was found:** the README status paragraph
  says **v0.9.0** (it still said v0.7.0 while claiming P1 was in-tree);
  the CHANGELOG cuts `[0.9.0]` and `[0.8.0]` (the merged v0.8 + v0.9
  story had lived under `[Unreleased]` since its tags) and this change
  gets its own `[Unreleased]` section; the P1 story's counts are the
  real ones — `Prop` = **110 longhands** (was "95") and the registry =
  **178 rows** (was "~150") — in the CHANGELOG, the architecture plan's
  P1 status block, the §8 roadmap row (with the generator explicitly
  moved to P2) and `rows.rs`'s own rustdoc; the evidence header's
  "workspace version still 0.7.0" is annotated with when that stopped
  being true (the v0.8.0/v0.9.0 tags).

### Gates (all green on the dirty tree, pre-handoff)

- `cargo fmt --all --check` — clean.
- `cargo clippy --workspace --all-targets --all-features -- -D
  warnings` — **exit 0, zero warnings**. Pedantic is enforced through
  the workspace `[lints.clippy]` table (`pedantic = warn`, per-lint
  exceptions with rationale) denied by `-D warnings` — proven live: the
  gate rejected `match_wildcard_for_single_variants` in the new tests
  until they named the variants. (Clippy 1.99 makes a literal `-W
  pedantic` on the command line a deprecation ERROR, and `-W
  clippy::pedantic` overrides the manifest's rationale-backed allows —
  186 of them, all deliberate — so the manifest-driven gate is the one
  that preserves this repo's documented lint contract.)
- `#![allow|expect|cfg_attr(allow)]` grep over `crates/` — **empty**.
- `cargo test --workspace --all-features` — **486 passed, 0 failed**
  (the pre-change suite plus the net-new: 3 registry-seam tests + the
  miette contract test + the calc-edge pin).
- `cargo test --benches --all-features` — green (all benches are divan
  and stable; **no nightly-only benches exist** — the only nightly
  surface is the `cargo-fuzz` job, which is CI-gated and not a bench).

### Per-file history lives on the branch

Merging is `--rebase` with **one commit per logical change** landed as
Conventional-Commit commits on `feat/p1-consumer-examples` (the
build-pin fix, the CI fix, the core de-unwrap refactor, the data seam
feat with its tests + bench, the examples feat, the docs alignment), so
the per-file history survives the rebase onto `origin/main`; the review
shows the same tree this PR body describes. **No commits have been made
on the base branch for this work** — the whole change ships as the
branch's commits authored by `ship.sh` from the dirty tree it was
handed.

### Verify locally

```sh
cargo run -p aeolus-syntax --example lex_candidates
cargo run -p aeolus-data --example registry_lookup
cargo run -p aeolus-cli --example theme_spec
cargo test --workspace --all-features
```
