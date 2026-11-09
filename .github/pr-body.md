# feat: v1-p1-data-driven-core — the V1 zero-runtime data-driven core (phase P1)

## Milestone — V1 phase P1: the data-driven core

The tree carries the v0.8 per-edge set-mask and the v0.9 variant & grid
engine with the v0.9.1 consumer handoff (examples, typed theme errors)
already merged. This change lands the FIRST slice of the zero-runtime
track (`docs/AEOLUS_V1_ARCHITECTURE.md` §8, phase P1 — "the data-driven
core"): three net-new git-only crates that give Aeolus the pieces the
1.0 compiler pipeline assembles from — the zero-copy lexer, the utility
registry as data, and the longhand IR with mask algebra. Nothing in the
v0.x surface changes: every existing test passes untouched, no new
external dependencies anywhere, `publish = false` while the pipeline
comes together (plan §3.2).

### What changed

- **`aeolus-ir`** (net-new, zero-dep): the longhand IR —
  `#[repr(u16)] Prop` (95 longhands; discriminant order = cascade/sort
  order), the `PropMask` u128 bitset (`union`/`intersect`/
  `difference`/`covers`/`for_each` — the v0.8 per-edge set-mask idea
  generalized to the whole longhand surface: `p-4 ⊕ px-8` and layer
  dominance are bit operations, plan §3.3), the 12-byte `Copy` `Val`
  (Px/Rem/Pct/Num/Rgba/Tok/ModePair/Kw), `Cond`/`Bp` layer conditions
  with the v4-cascade specificity ranks, and `Decl`/`Layer`/`Program`
  assembly (layer 0 = base, stable specificity sort, the
  base-conflict projection). Laws pinned in `tests/ir_laws.rs` (mask
  algebra idempotence/absorption/dominance, disjoint TRBL edges, the
  u128 budget, value size, stable sort, conditional layers never evict
  base declarations), benched in `benches/mask.rs`, demonstrated for
  consumers in `examples/mask_algebra.rs`.
- **`aeolus-data`** (net-new, dep: `aeolus-ir`): the utility registry
  as data — ~150 auditable rows (root, `ValueKind` shape, longhand
  targets in TRBL/TL-TR-BR-BL fan-out order, `tw_merge` conflict group,
  negation flag, v4 source ref), the committed v4 scale tables
  (`TEXT_SIZES` with the calculated leading pairs, weights, radii,
  tracking, leading ratios, container widths, z levels, opacity,
  border/outline/ring-offset steps, the full default colour palette +
  shadcn tokens, gradient shapes), longest-prefix disambiguation
  (`rounded-tl-lg` → `rounded-tl`, `min-w-0` → `min-w`, the `text-*`
  font-size/colour/align triple split, `from-20%` → from-pos vs
  `from-red-500` → from-color), and `generate_docs()` rendering the
  audit page from the same rows (the one-source-of-truth claim, plan
  §3.2). `tests/registry.rs` cross-checks every sampled group name AND
  sweeps the golden corpus: every `collision_group`-grouped corpus
  class must find a registry row carrying that group — the tw_merge
  quirk (`border-dashed` under `border-color`, `flex-1` under
  `flex-direction`, `ring-inset` under `ring-color`) is registry
  reality, not a comment.
- **`aeolus-syntax`** (net-new, zero-dep): the zero-copy lexer — class
  strings into borrowed `Candidate`s with byte spans (bracket-aware
  variant segmentation, negation, importance both spellings, body,
  arbitrary form/hint/value/property flag, RAW `/modifier`), typed
  `LexError`s with spans, honest per-class degradation for class
  STRINGS. `tests/differential.rs` pins the lexer to the v0.x
  `aeolus-core::parser` oracle over a 59-string tricky battery AND
  every golden-corpus class: failure/Success split, variant
  segmentation (compared in canonical space through the oracle's own
  `parse_variant`), negation, importance, body, arbitrary and modifier
  must agree. One documented divergence: the v0.x inline caps (8
  variants / 10 elements) are deliberately dropped. `tests/shapes.rs`
  pins the spans/slices/errors contract; `benches/lex.rs` measures the
  cold paths (bare 15 ns, stacked 80 ns, calc-arbitrary 47 ns,
  whole-string 813 ns).
- **Docs drift fixed for P1:** the README crate table grows the three
  V1 rows, the Status deferred-items list now says P1 is IN the tree
  (P2/P3/xtask/GPUI remain planned), the Learn-more V1 entry drops
  "nothing built yet", the CHANGELOG grows the P1 section, and the V1
  architecture doc's header records the P1 status beside its evidence
  header.
- **Micro-debt/hygiene:** `muda-stub`'s `[lints]` comment now names the
  two allowances it carries (dead_code, new_without_default) — keeping
  the workspace's zero-`#[allow]` grep claim precise after the P1
  additions.

### Gates (all green on the dirty tree, pre-handoff)

- `cargo fmt --all --check` — clean.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` —
  zero warnings. Pedantic is enforced through the workspace
  `[lints.clippy]` table (`pedantic = warn`, per-lint exceptions with
  rationale), denied by `-D warnings`; the new crates opt in with
  `[lints] workspace = true` and carry zero `#[allow]`/`#[expect]`.
- `#![allow|expect|cfg_attr(allow)]` grep over all `.rs` in `crates/` —
  **empty**.
- `cargo test --workspace --all-features` — **481 passed, 0 failed**
  (446 pre-P1 + 35 net-new: 15 in `aeolus-ir`, 9 in `aeolus-data`,
  11 in `aeolus-syntax`).
- `cargo test --workspace --benches` + `cargo bench` runs (divan,
  stable — no nightly-only benches exist; the `cargo-fuzz` targets are
  CI-gated nightly and are not benches): the P1 `mask`/`lex` suites
  compile and run green alongside the existing benches
  (`mask_union_overlay` ≈ 0.4 ns, `lex_bare` ≈ 15 ns).

### Per-file history lives on the branch

Merging is `--rebase` with **one commit per logical change** landed as
Conventional-Commit commits on `feat/v1-p1-data-driven-core` (per-crate
feat commits for ir → data → syntax, test/bench/docs commits, the
workspace wiring, the release chore), so the per-file history survives
the rebase onto `origin/main`; the review shows the same tree this PR
body describes. **No commits have been made on the base branch for this
work** — the whole change ships as the branch's commits authored by
`ship.sh` from the dirty tree it was handed.

### Verify locally

```sh
cargo run -p aeolus-ir --example mask_algebra
cargo test -p aeolus-ir -p aeolus-data -p aeolus-syntax
cargo test --workspace --all-features
```
