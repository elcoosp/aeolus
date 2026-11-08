# Aeolus v0.7 Plan (plan of record)

The v0.6 plan (`AEOLUS_V0.6.md`) shipped in 0.6.0. This document is the
v0.7 plan of record; every `v0.7 §X` citation in code and reports refers
here.

Goal: **make the `StylePlan` IR portable and composable, and give the
project a headless consumption surface.** Through v0.6 the IR was the
private contract between the resolver and the five GUI backends; this
release turns it into a first-class artifact:

1. `StylePlan` values COMPOSE (`overlay`) — plan-level algebra that mirrors
   `cn()` but works on RESOLVED plans, theme-independent.
2. `StylePlan` values PERSIST (`to_snapshot_json` / `from_snapshot_json`) —
   a canonical, byte-stable, dependency-free JSON artifact with a STRICT
   reader (unknown fields are errors, never defaults).
3. A new crate, `aeolus-cli`, consumes all of it headlessly: resolve class
   strings into plan snapshots, emit verification CSS, diff two resolutions
   across the cosmetic/geometric lanes, merge with `cn()`, and LINT class
   strings inside source files.

No backend changes, no parser changes, no MSRV change, no new
dependencies anywhere — `aeolus-cli` keeps the core's zero-dependency
discipline (hand-rolled arg parsing, std I/O only), and `snapshot.rs` is
a hand-rolled canonical JSON writer/reader.

## §1 — Plan algebra (`StylePlan::overlay`)

- §1.1 `StylePlan::overlay(&self, over: &StylePlan) -> StylePlan` composes
  two resolved plans, later-wins per slot — the IR-level mirror of `cn()`:
  - `base`: [`ResolvedStyle::overlay`] — `Some` slots of `over.base`
    replace, unset slots keep the base value (the v0.2 slot semantics).
  - `layers`: SAME [`VariantCondition`] merges per slot; conditions only
    the override has are appended; the result stays in ascending
    [`VariantCondition::rank`] order (the cascade contract).
  - `responsive`/`container`: per-bucket overlay, union of buckets.
  - `dark`: per-slot overlay when both plans carry a dark bucket.
  - `group`/`peer`: merged by id, new ids appended (base-first order).
  - `rules`: merged by [`Cond`] equality — the same conjunction is ONE
    rule; new conditions append in override order.
  - `breakpoints`: the override's table wins (theme-derived; mixing plans
    from different theme versions is documented best-effort).
- §1.2 Why: composition today happens on class STRINGS (`cn()` before
  resolve), which needs the theme and the parser at the merge site.
  Plan-level overlay works AFTER resolve: a component's cached plan can
  gain per-instance overrides at runtime without re-resolving, re-parsing,
  or holding a theme reference. It is also the natural primitive for
  theming wrappers (base plan + dark-plan overlays).
- §1.3 Contracts pinned by `tests/plan_algebra.rs` (8 tests): later-wins
  per slot; layer merge by condition; rank-ordered layers after merge;
  responsive/dark/group/peer/rules bucket composition; breakpoint-table
  override; `overlay` is idempotent (`p.overlay(o).overlay(o) ==
  p.overlay(o)`); and the DIFF integration —
  `diff(&base.base, &merged.base)` reports exactly the override's slots,
  so backends keep their repaint-vs-relayout lanes when composing.
- §1.4 Discovered and DOCUMENTED, not fixed (the v0.6 pattern for honest
  deferrals): while pinning layer-merge semantics we confirmed that
  per-edge families (`p-*`, `m-*`, `border-*`) materialize unset edges as
  `0.0` at finalize, so a `hover:px-8` layer carries
  `Padding { top: 0, right: 32, bottom: 0, left: 32 }`. Applied as a
  layer, the zeroed edges REPLACE the base's edges — CSS would keep
  `padding-top: 16px` for `p-4 hover:px-8`. The per-edge set-mask needed
  to fix this is an IR model change that ripples through every backend's
  lowering; it stays out of v0.7 and is recorded as the first item of
  §6 (deferred). The behavior is now PINNED by tests instead of unknown.

## §2 — Canonical plan snapshots (`snapshot.rs`)

- §2.1 `StylePlan::to_snapshot_json()` — canonical JSON: fixed field order
  (the schema order IS the compatibility contract), compact separators,
  Rust's shortest round-trip float formatting, no map-order drift
  (`responsive` is a `BTreeMap`, `layers`/`rules` are `Vec`s). Equal plans
  produce byte-identical documents.
- §2.2 `StylePlan::from_snapshot_json(&str) -> Result<StylePlan,
  SnapshotError>` — STRICT: unknown fields, unknown enum names, duplicate
  keys, wrong `format`/`version` headers, truncated documents and trailing
  content are all typed errors with a dotted path
  (`$.base.extras.grid.cols`). A reader never silently drops or defaults
  a field — the no-silent-failures rule, applied to the artifact format.
- §2.3 Header: `{"format":"aeolus-plan","version":1,...}`. Other versions
  fail with `SnapshotError::UnsupportedVersion(n)` — new IR fields bump
  the schema version, old readers fail LOUDLY.
- §2.4 Every IR type is covered: all `ResolvedStyle` slots (including
  `background_gradient` with composed stops, multi-layer shadows, the
  full text table), all extras, and the v0.2 rules IR (`Atom`/`Cond`/
  `Test` — states, data/aria, structural, tag, media, viewport/container
  windows, group/peer labels). `TextValue::Keyword(&'static str)` maps
  through a closed keyword table (the exact set `resolve.rs` produces).
- §2.5 Non-finite floats (never produced by the resolver — `rounded-full`
  is the finite 9999px sentinel) are encoded as `"NaN"`/`"inf"`/`"-inf"`
  strings and decode symmetrically: the writer is total, no panics on
  hand-built plans.
- §2.6 The parser is a hand-rolled recursive-descent JSON subset with a
  depth limit (hostile-input guard), full string escapes including
  surrogate pairs, and exact duplicate-key detection. `from_snapshot_json`
  is a natural fuzz target (see §5).
- §2.7 Contracts pinned by `tests/snapshot_roundtrip.rs` (9 tests,
  including a deterministic pseudo-random round-trip over 256 class
  strings and a kitchen-sink plan that touches every bucket and most slot
  payloads): `from(to(p)) == p`; byte-stability across resolutions; header
  shape; malformed-input totality (no panics); gradient round-trips;
  cache-vs-resolver snapshot agreement.

## §3 — `aeolus-cli`, the headless surface

- §3.1 New workspace crate `crates/aeolus-cli` (binary `aeolus`,
  `publish = false` alongside the backends). Zero dependencies beyond
  `aeolus-core`; the whole program is the library `run(args, out, err) ->
  exit_code`, so the suite drives it without spawning processes.
- §3.2 Subcommands:
  - `resolve [--theme <spec>] [--summary] <classes...>` — the DEFAULT
    output is the plan snapshot JSON (§2 is the artifact); `--summary`
    prints the human slot/layers/responsive/dark digest; warnings print to
    stderr and stay non-fatal (exit 0).
  - `css [--theme <spec>] <classes...>` — the v0.4 verification emitter's
    output (v4 per-class declaration shapes).
  - `diff [--json] <A> <B>` — resolve both strings, diff the base layers,
    report changed slots WITH their lanes (cosmetic vs geometric) and the
    resulting invalidation (`repaint-only` vs `relayout`).
  - `cn <groups...>` — the conflict merge.
  - `check [--theme <spec>] [--fail-on unknown|unsupported|conflict|any|
    none] [--json] <files...>` — the linter: extracts quoted strings
    (single/double/Rust raw `r#"…"#`), keeps a string as a candidate when
    at least half of its tokens resolve warning-free (the documented
    "looks like Tailwind" heuristic), and reports every remaining warning
    as a finding plus `cn()` conflicts (token dropped, with the winning
    token attributed). Exit 1 when findings match `--fail-on` (default
    `unknown` — typos fail CI), exit 2 on usage/theme/IO errors.
- §3.3 `--theme` accepts `shadcn` (default), `shadcn-dark`, `default`
  (the verbatim Tailwind v4 default theme), or a path to theme CSS.
- §3.4 Contracts pinned by `crates/aeolus-cli/tests/cli.rs` (12 tests):
  snapshot-artifact output parses back; `--summary` shape; warnings on
  stderr with exit 0; theme specs and precise theme errors; CSS shapes;
  lane-aware diff (human + JSON); cn() eviction semantics; the check
  pipeline (findings, fail-on modes, JSON output, prose immunity);
  usage/version/exit-code contract.

## §4 — Quality gates

- §4.1 The full battery stays green: `cargo test --workspace`,
  `clippy --workspace --all-targets -D warnings`, `fmt --check`,
  `doc -D warnings`, `cargo package -p aeolus-core -p aeolus-macro`,
  `cargo deny check licenses`.
- §4.2 Benchmarks (divan): the core `benches/hot.rs` gains
  `plan_overlay_button`, `snapshot_write_button`, `snapshot_read_button`,
  `snapshot_roundtrip_button`; the CLI crate gains `benches/pipeline.rs`
  (extraction over a 1k-line source buffer, lint of the 33-class Button
  string, snapshot read/write/round-trip). Numbers in
  `reports/08-benchmarks/BENCHMARKS.md`.
- §4.3 Version → 0.7.0 (workspace manifest + the two publish-ready crates'
  version refs); docs: README (crate table + usage + features),
  CHANGELOG, DESIGN §13, `AEOLUS_V0.7.md` (this file), TDD evidence in
  `reports/10-tdd-evidence/09-portable-plans.md`.

## §5 — Fuzz targets (recommended follow-up)

`from_snapshot_json` and the extractor are the two new untrusted-input
surfaces. The `fuzz/` workspace should gain `plan_snapshot` (writer→reader
round-trip invariant: either it parses back equal, or it errors — never a
panic) and `extract` (no panics on arbitrary bytes). Not wired in v0.7 —
the fuzz workspace pins its own toolchain — but the invariants are already
pinned by `snapshot_never_panics_on_fuzzish_input` and the depth-limit
test.

## §6 — Deferred (recorded honestly)

- **Per-edge layer semantics** (§1.4): `p-4 hover:px-8` layers carry
  zeroed unset edges; the correct model is a per-edge set-mask in
  `Padding`/`Margin`/`BorderWidth` (or per-edge `Option<f32>`), which
  ripples through every backend. Deferred with a pinned test.
- **`justify-items-*`/`justify-self-*`/`place-*`/`grid-flow-*`/`auto-rows-
  *`/`auto-cols-*`**: recognized-but-unsupported in the resolver today
  (precise reasons, per the coverage guard). Taffy exposes the native
  surface at the floem pin; wiring it is a v0.8-scale backend milestone,
  not part of the v0.7 portable-plan goal.
- **Snapshot → `PlanCache` persistence**: the artifact format is ready
  (canonical, versioned, strict); a `PlanCache::load_snapshot` API needs
  the cache's generation semantics designed against multi-theme apps.
