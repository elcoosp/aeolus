# TDD evidence — 10 · the per-edge set-mask (v0.8 §1)

Cycle: RED (`tests/per_edge_layers.rs` written first) → implement →
GREEN → bench. Toolchain: stable 1.99.1, clippy 0.1.99.

## RED — the new suite does not compile, and the pinned defect is visible

`cargo test -p aeolus-core --test per_edge_layers` (before the
implementation):

```text
error[E0422]: cannot find struct, variant or union type `EdgeSet` in this scope
error[E0422]: cannot find struct, variant or union type `CornerSet` in this scope
error[E0609]: no field `set` on type `aeolus_core::Padding`
error[E0609]: no field `set` on type `aeolus_core::CornerRadius`
error[E0433]: cannot find type `EdgeSet` in this scope
```

The same run demonstrated the defect the milestone removes: the v0.7
suite Pinned `hover:px-8` as `Padding { 0, 32, 0, 32 }`
(`tests/plan_algebra.rs` `layer_merge_by_condition`), and plan algebra
composed `p-4 ⊕ px-8` to `{0, 32, 0, 32}` — CSS keeps `16`.

## RED — each backend's layer application wipes the base (stash-run)

The backend tests were written BEFORE the backend implementations. Each
was captured against the stashed implementation (`git stash push
<crate>/src/lib.rs`), i.e. the pre-milestone lowering:

- **aeolus-xilem** (`--test lowering partial_edge`):

  ```text
  assertion `left == right` failed: hover keeps the base's top/bottom (16px) and takes px-8's left/right
    left: Padding { left: 32px, right: 32px, top: 0px, bottom: 0px }
   right: Padding { left: 32px, right: 32px, top: 16px, bottom: 16px }
  ```

- **aeolus-floem**:

  ```text
  assertion `left == right` failed: the top edge keeps p-4 — the layer's 0.0 placeholder must NOT apply
    left: Pt(0.0)
   right: Pt(16.0)
  ```

- **aeolus-iced**:

  ```text
  assertion `left == right` failed: hover:px-8 overrides left/right only; top/bottom keep p-4
    left: (0.0, 32.0, 0.0, 32.0)
   right: (16.0, 32.0, 16.0, 32.0)
  ```

- **aeolus-egui**:

  ```text
  assertion `left == right` failed: hover:px-8 overrides left/right only; top/bottom keep p-4
    left: (0, 32, 0, 32)
   right: (16, 32, 16, 32)
  ```

- **aeolus-slint** (the `padding-top: 0px;` placeholder binding):

  ```text
  thread 'partial_edge_states_materialize_from_base' panicked at crates/aeolus-slint/tests/lowering.rs
  test result: FAILED. 0 passed; 1 failed
  ```

## GREEN — all suites pass after the implementation

- `cargo test -p aeolus-core --test per_edge_layers` → **11 passed**
  (after three of MY OWN test-authoring bugs were fixed against the
  implementation's correct output: `rounded-lg` is 10px in the shadcn
  theme, a mis-ordered border tuple, and a snapshot tamper that broke
  the JSON envelope — the cycle catching the test, not the code).
- Full workspace: `cargo test --workspace` → **359 passed, 0 failed**
  (core 247 incl. the new suite; xilem/floem/iced/egui/slint/shadcn/
  macro/cli suites unchanged or extended; the two pre-existing zeroed-
  edge pins in `plan_algebra.rs` now pin the v0.8 mask semantics).

## Gates

- `cargo fmt --all -- --check` → 0.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  → 0, with `[workspace.lints.clippy] pedantic = { level = "warn",
  priority = -1 }` and `[lints] workspace = true` in every member —
  pedantic armed workspace-wide for the first time, ZERO `#[allow]`
  attributes in the workspace crates (the two pre-existing clippy allows
  moved to the manifest with rationale). What the cycle caught here: the
  pre-existing `assert!(… || true)` tautology in
  `shadcn_components::negatives_and_important` (a real test bug — now
  asserts what was intended), a pre-existing `#[allow(clippy::…)]` pair
  contrary to the CONTRIBUTING §3 no-allow rule (now manifest
  exceptions), and `Segments`' manual `Clone`/`Copy` impls.

## Bench

`cargo bench -p aeolus-core --bench hot -- plan_overlay` (divan, release,
this sandbox — directional, not CI numbers):

| bench | median |
|---|---|
| `plan_overlay_button` (v0.7 whole-slot) | ~318 ns |
| `plan_overlay_partial_edges` (v0.8 edge-wise) | ~318 ns |

The mask walk costs nothing measurable over the slot walk. Snapshot
costs are unchanged by the mask fields (write ~8.4 µs, read ~166 µs,
round-trip ~183 µs for the Button plan).
