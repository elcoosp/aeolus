# The performance contract

The zero-runtime thesis is measured, not asserted. The stable harness
(`crates/aeolus-gpui/benches/perf_contract.rs`) measures the static
walk beside the hand-written builder chains it replaces and PINS the
parity — walk == hand-written, field by field — before anything is
measured; the freeze-once-clone host pattern, the v0.x lane's
static/dynamic paths and the V1 dynamic lane's warm/cold shapes ride
the same suite. The committed wall-clock gate (`perf.yml`) compares the
walk and the cell-warm path against `baselines.json` and fails a PR on
a >5% walk regression.

The measured medians live in two places, kept deliberately:

- `reports/08-benchmarks/BENCHMARKS.md` — the reproducible benchmark
  page: the environment, the exact commands and the per-suite tables
  (the ratios are the point; absolute numbers are environment-specific).
- `docs/AEOLUS_V1_ARCHITECTURE.md` §4 — the plan's performance
  contract table with the measured cells filled in.

## The allocation and instruction-count halves

The allocation contract tests pin the static path's zero-alloc
discipline (`alloc_contract.rs` and the walk allocation battery), and
the lock-free claim is by construction (no locks on the static path;
the UI thread's `Env` is a TLS cell with a `begin_frame` debug-assert).
The iai-callgrind instruction-count golden (`tw!(s)` vs hand-written,
the ±2% gate) is the nightly-only half — flagged as such, run on the
`iai-nightly` workflow, with the stable divan medians standing in on
stable.

## What to measure before you optimize

`cargo bench -p aeolus-gpui --bench perf_contract` is the first stop;
the compile seams' benches ride their own crates (`cargo bench -p
aeolus-compile`, `-p aeolus-core`, `-p aeolus-data`). The exact
commands and the current numbers are [the benchmark
page](../../../reports/08-benchmarks/BENCHMARKS.md).
