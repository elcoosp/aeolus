# Aeolus v0.8 Plan (plan of record) — §1: the per-edge set-mask

The v0.7 plan (`AEOLUS_V0.7.md`) shipped in 0.7.0. This document is the
v0.8 plan of record; every `v0.8 §X` citation in code and reports refers
here. **§1 is LANDED in this tree** (TDD evidence:
`reports/10-tdd-evidence/10-per-edge-set-mask.md`); later sections are
recorded-deferred as usual.

Goal: **fix the cascade semantics of the per-edge families** — the first
item of the v0.7 §6 deferred list, and the last known correctness defect
in the IR (v0.7 §1.4).

## §1 — Per-edge set-mask (LANDED)

### §1.1 The defect

`finalize` materialized unset edges as `0.0`, so a partial family in a
conditional layer carried a fully-populated struct:
`hover:px-8` → `Padding { top: 0, right: 32, bottom: 0, left: 32 }`.
Two consequences:

- **Layer application wiped the base.** `p-4 hover:px-8` lost
  `padding-top: 16px` on hover — CSS keeps it. Every backend applied the
  layer's slot wholesale (Masonry's `PropertyStack` replaces whole
  properties; Iced/egui books replaced whole payload structs; Floem's
  selector entries set all four fields; Slint re-bound all four edges in
  the state block).
- **Plan algebra composed wrong.** `p-4 ⊕ px-8` produced
  `{0, 32, 0, 32}` — the zeros are indistinguishable from explicit
  `pt-0 pb-0` in a maskless IR, so no post-hoc merge could fix it. The
  information "which edges did the class string address" was destroyed at
  `finalize`.

### §1.2 The model

Every per-edge family carries a set-mask alongside its `f32` values:

- `EdgeSet { top, right, bottom, left }` on `Padding` (= `Margin`) and
  `BorderWidth`;
- `CornerSet { top_left, top_right, bottom_right, bottom_left }` on
  `CornerRadius`.

`false` = "this bucket/layer/plan never named that edge": the `f32` is
the documented `0.0` placeholder, never a cascade input. `true` = the
edge carries a real value (which may legitimately be `0.0` — `pt-0`).
This is the same shape the codebase already had for `AutoEdges`
(`mx-auto`), applied to the four edge families.

### §1.3 Semantics

- **`finalize`** records the mask from its `Option`-per-edge accumulation
  (the Options ARE the mask). Whole families (`p-4`) end `ALL`; partial
  families (`px-8`, `rounded-t-md`) end with only their edges set.
- **`ResolvedStyle::overlay`** merges the four families EDGE-WISE: an
  edge the override addresses wins; an edge it does not address keeps
  the destination's value. `p-4 ⊕ px-8` → `{16, 32, 16, 32}` with a full
  mask (the CSS cascade). Every other slot keeps the v0.7
  `Some`-replaces semantics. All of `StylePlan::overlay` (base, layers,
  responsive/dark/group/peer/rules buckets) and the probe flatten
  (`StylePlan::resolve`) route through it, so composition and runtime
  resolution are fixed by the same primitive. Idempotence and the
  diff-integration contract (v0.7 §1.3) re-pinned with partial edges.
- **`ResolvedStyle::materialize_unset_edges_from(&base)`** — the
  layer-application primitive for cascades that replace WHOLE
  properties: fills a partial layer's masked-off edges from the base
  plan (values and mask), leaves `None` slots alone. Backends call it
  before lowering a layer.
- **Per-field cascades** (Floem's taffy fields, Iced/egui payload
  structs) don't need materialization: the lowered output carries the
  mask (`WidgetHints::padding_set`, egui `Presence.{inner,outer}_margin_set`,
  `corner_set`, iced `Presence.radius_set`) and their `overlay` merges
  edge/corner-wise directly.

### §1.4 Backend application (v0.8 §1.2)

| Backend   | Mechanism                                                                                     |
|-----------|-----------------------------------------------------------------------------------------------|
| Xilem     | `lower()` materializes each layer against `plan.base` before pushing the stack entry           |
| Floem     | `apply_resolved` sets padding/margin/border/radius per edge, gated by the mask                 |
| Iced      | `WidgetHints::overlay` + `IcedStyle::overlay` merge the padding hint / radius per edge         |
| egui      | `EguiStyle::overlay` merges `inner_margin`/`outer_margin` per edge and `CornerRadius` per corner |
| Slint     | `StyleBook::new` materializes layers/dark before lowering; state blocks re-bind flattened values |

### §1.5 Snapshot schema v2

The masks are part of the artifact: `padding`/`border_width`/`margin`
objects gain a required `"set"` object (four booleans), `corner_radius`
gains `"set"` with the four corners. `PLAN_SNAPSHOT_VERSION` bumps 1 → 2;
v1 documents fail with `SnapshotError::UnsupportedVersion(1)` — old
readers fail loudly, per the v0.7 §2.3 contract. The writer stays
canonical and byte-stable; the strict reader rejects a malformed mask
with a dotted path (`$.base.padding.set.left`).

### §1.6 Contracts pinned by `tests/per_edge_layers.rs` (11 tests)

finalize masks (whole + partial + per-corner + `pt-0`); edge-wise base
overlay (both orders); partial-edge layers carry the mask; the flatten
path keeps base edges under hover; `materialize_unset_edges_from`
(inheritance, totality, `None` slots untouched, identity on total
styles); diff exactness; idempotence with partial edges; margin/border
edge-wise; corner-wise composition; snapshot v2 round-trip of masks;
strict reader mask rejection.

### §1.7 Quality gates

- `cargo test --workspace` — 359 tests green (the full battery).
- `cargo clippy --all-targets --all-features -- -D warnings` — 0 with
  `[workspace.lints.clippy] pedantic = warn` + per-crate
  `[lints] workspace = true`: pedantic is armed workspace-wide for the
  first time; the sanctioned per-lint exceptions (with rationale) live in
  the root manifest, and there are ZERO `#[allow]` attributes in the
  workspace crates (the two pre-existing clippy allows were converted).
- `cargo fmt --all -- --check` — 0.
- New bench `plan_overlay_partial_edges` (~309 ns, same cost class as the
  whole-slot overlay) — the mask walk is free.

### §1.8 Deferred (recorded honestly)

- `justify-items-*`/`justify-self-*`/`place-*`/`grid-flow-*`/`auto-rows-*`
  /`auto-cols-*` (carried from v0.7 §6): recognized-but-unsupported, taffy
  exposes the surface at the floem pin — a backend milestone.
- Snapshot → `PlanCache` persistence (carried from v0.7 §6): the artifact
  format is v2 and strict; the generation semantics for multi-theme apps
  are still to be designed.
- Fuzz targets `plan_snapshot`/`extract` (carried from v0.7 §5): the
  invariants stay pinned by `snapshot_never_panics_on_fuzzish_input` and
  the depth-limit test.
