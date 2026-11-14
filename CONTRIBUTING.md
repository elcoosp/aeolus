# Contributing to Aeolus

Thank you for considering a contribution. Aeolus is a Tailwind v4 →
native-Rust style compiler; the code IS the reference (see
`reports/03-aeolus-core-design/DESIGN.md`), so the bar for changes is
"tests prove the behavior, docs explain the why".

## The non-negotiables

### 1. Test-driven development

- **Write the failing test FIRST.** Run it, capture the RED output, then
  implement, then capture GREEN.
- Every behavioral change has a test. No exceptions.
- If a test reveals a design flaw, fix the design **and** update the
  corresponding `reports/*` document, then re-write the test.
- **Never delete a failing test to make the suite green. Fix the code.**
- One commit per logical change, [Conventional
  Commits](https://www.conventionalcommits.org/), and the commit body
  explains **why** (the diff already shows what).

For fuzz findings, the workflow is: reduce the corpus input to a unit test
in the same file as the affected code (RED), fix (GREEN), and keep the
fuzz target asserting the invariant that caught it. Phase-4 examples live
in `reports/10-tdd-evidence/04-hardening.md`.

### 2. No silent failures

The project's own contract (`LoweringReport`, `Warning::*`) requires every
dropped feature to be visible:

- new capability → remove the matching report entry **and** update the
  capability matrix (`reports/02-capability-matrix/`);
- new limitation → add a report entry or a `Warning`;
- no `unwrap()`/`expect()`/`panic!()` on reachable public-API paths in
  library code — return `Result` or document the panic in rustdoc.
  (Poison-tolerant lock access is the one sanctioned pattern; see
  `cache.rs`'s `read_map`/`write_map` for the documented justification.)

### 3. No shortcuts

- No `#[allow(...)]` to bypass `cargo clippy -- -D warnings`.
- Do not weaken tests to make them pass.
- Do not ship `muda-stub` as a published artifact (it is dev-sandbox-only;
  see `reports/09-publishing/PUBLISHING.md`).
- Do not bump the MSRV (1.88 for the core/CLI/pipeline crates, 1.96 for the backend crates — the CI gates both) without updating **every** manifest's
  `rust-version`.

### 4. Git hygiene

- Branches off `main`; PRs target `main`.
- Squash fixup commits before merge.
- Tags follow `vMAJOR.MINOR.PATCH`.

## Architectural guardrails

- **`aeolus-core` stays zero-dependency.** Small utilities: write them
  (see `FxHasher`, the shunting-yard `calc()` evaluator, the
  two-generation caches). Property/dev tooling may be dev-dependencies
  (`proptest`, `divan`).
- **The V1 IR is `aeolus-ir`'s `Copy` `Program`** (longhand `Decl`s,
  `PropMask` overlay algebra); the v0.x `StylePlan` + `Resolver` stay
  in `aeolus-core` as the differential oracle and the runtime lane.
  A new backend lowering (`lower_program`) consumes the frozen
  `StaticProgram` and must map every `Prop` or report it onto its
  `LoweringReport` — never a silent skip.
- **The Shadcn class tables are verbatim** from the upstream registry
  (commit pinned in `reports/06-shadcn-v1/SHADCN_V1.md`). Do not "fix"
  them to make tests pass — the tables are the oracle; fix the resolver.
- Do not rewrite the parser from scratch; fix specific issues with tests.
- Update the reports when reality changes: `reports/` is the project's
  memory (audit, capability matrix, design, benchmarks, publishing).

## Local gates (all must be green before a PR)

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
cargo test --doc -p aeolus-core
cargo package -p aeolus-core -p aeolus-macro --allow-dirty --no-verify
cargo deny check licenses
```

Fuzz (short local smoke; CI runs 60s per target over all SIX):

```sh
cargo +nightly fuzz run parse_class_string -- -max_total_time=60
cargo +nightly fuzz run color_parse       -- -max_total_time=60
cargo +nightly fuzz run cn                -- -max_total_time=60
cargo +nightly fuzz run theme_from_css    -- -max_total_time=60
cargo +nightly fuzz run parse_variant     -- -max_total_time=60
cargo +nightly fuzz run probe_eval        -- -max_total_time=60
```

The backend crates resolve their framework deps from pinned git revs —
no `repos/` checkout is needed.

## Reporting bugs / security

Bugs: GitHub issues with a reproducing class string or CSS snippet.
Security: see `SECURITY.md` (private advisory / email, 7-day ack).

## License

By contributing you agree your work is released under the repository's
dual license (MIT OR Apache-2.0).
