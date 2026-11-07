# TDD evidence — v0.7 portable plans (`09-portable-plans.md`)

Every behavioral change in v0.7 has a test written FIRST; the RED state
was captured by running the suite before the implementation existed (for
new modules, the RED state is the compile failure of the test target —
the missing API IS the missing behavior). The workflow follows
`CONTRIBUTING.md` §1.

## §1 Plan algebra (`StylePlan::overlay`) — RED → GREEN

- RED (captured): `cargo test -p aeolus-core --test plan_algebra` →
  `error[E0599]: no method named 'overlay' found for struct
  'aeolus_core::StylePlan'` — 8 tests, all failing to compile because the
  API did not exist.
- GREEN: `StylePlan::overlay` implemented in `crates/aeolus-core/src/plan.rs`
  (layers merged by condition + rank-sorted; responsive/container/dark/
  group/peer per-key overlay; rules merged by `Cond` equality; override's
  breakpoint table wins).
- Suite result: 8/8 pass. Two tests were CORRECTED during GREEN — both
  were test-side expectation errors against pinned pre-existing semantics,
  not implementation bugs, and the corrections are documented in the test
  bodies:
  1. `layer_merge_by_condition` expected a `hover:px-8` layer to carry
     `{32, 32, 32, 32}`; the resolver has ALWAYS materialized unset edges
     as 0.0 at finalize, so the layer carries `{0, 32, 0, 32}`. This
     exposed the per-edge layer-flattening gap (v0.7 §1.4): now pinned by
     the test and deferred honestly instead of silently unknown.
  2. `group_peer_and_rules_merge` expected the legacy group id `"menu"`;
     the v0.1 Group bucket has always used the `{label}{state}` form
     (`"menuhover"`, per `resolve.rs` `Route::Group`). Test updated to
     pin the actual id.

## §2 Canonical snapshots (`snapshot.rs`) — RED → GREEN

- RED (captured): `cargo test -p aeolus-core --test snapshot_roundtrip` →
  `no method named 'to_snapshot_json'`, `no method named
  'from_snapshot_json'`, `cannot find type 'SnapshotError'` — 9 tests,
  all failing to compile.
- GREEN: `crates/aeolus-core/src/snapshot.rs` — hand-rolled canonical JSON
  writer (fixed schema order, shortest round-trip floats, total on
  non-finite values) + strict reader (typed `SnapshotError` with dotted
  paths, depth-limited parser, duplicate-key detection, closed keyword
  table for `TextValue::Keyword(&'static str)`).
- Suite result: 9/9 pass, including:
  - `round_trip_kitchen_sink` — a plan touching every bucket (base,
    layers, responsive, dark, group/peer, rules with data/aria/structural/
    tag/media/viewport/container atoms, gradients incl. position patches)
    round-trips exactly.
  - `round_trip_is_byte_stable` — two resolutions produce byte-identical
    artifacts; re-serializing a parsed snapshot is byte-identical.
  - `rejects_malformed_snapshots` — 8 malformed shapes all produce typed
    errors (never defaults, never panics).
  - `snapshot_never_panics_on_fuzzish_input` — 10 hostile inputs, all
    `Err`, no panic.
  - `proptest_random_plans_round_trip` — 256 deterministic pseudo-random
    class strings (variant stacks, gradients, positions, structural
    variants) resolve, snapshot, and round-trip exactly.
  - `parse_cache_and_snapshot_agree` — the cached path and the cold path
    produce the SAME artifact bytes.

## §3 `aeolus-cli` — RED → GREEN

- RED (captured): `cargo test -p aeolus-cli` before the crate existed →
  the workspace manifest did not reference `crates/aeolus-cli`
  (the "no such crate" state is the missing deliverable).
- GREEN: the crate implements `run(args, out, err) -> code` and the five
  subcommands; 12/12 tests pass, covering: snapshot-artifact output that
  parses back; order-independence of the artifact; `--summary`; warnings
  on stderr with exit 0; theme specs (including a precise file-read
  error); the verification CSS shapes; lane-aware diff (human + JSON);
  `cn()` eviction semantics (px-2 evicted by px-4; p-8 evicts px-4 —
  tw_merge's parent-over-child rule); the check pipeline (findings,
  fail-on modes, JSON output, prose immunity); usage/version/exit codes.
- Two test-side corrections during GREEN, both against verified-correct
  behavior: the emitter emits the `padding:` shorthand for `p-4` (not
  per-side), and `cn("px-4", "p-8")` IS `"p-8"` (parent later in the
  string wins) — both match the pinned v0.2/v0.4 semantics and the
  tw_merge oracle; the tests were rewritten to pin them.

## §4 What the RED→GREEN cycle caught (beyond coverage)

1. The per-edge layer flattening (§1 above) — now a documented, test-pinned
   deferral in `AEOLUS_V0.7.md` §1.4/§6 instead of an unknown.
2. The strict reader's header loop initially rejected body fields
   (`UnknownField { path: "base" }` on an empty plan) — caught by
   `empty_plan_round_trips` before any release artifact existed; fixed by
   separating header collection from the body whitelist.
3. `read_f32` accepted out-of-range f64 numbers (e.g. `1e999` from JSON)
   that silently became infinities in f32 — now a typed error; only the
   documented `"NaN"`/`"inf"`/`"-inf"` string encodings produce non-finite
   values.
