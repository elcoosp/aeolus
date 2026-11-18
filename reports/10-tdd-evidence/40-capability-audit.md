# 40 — The capability audit face

The conformance score's project-sized twin. The §5.1 seam published
the backend-wide answer — `Conformance::of` over `Prop::ALL`, the
committed `docs/CONFORMANCE.md` page, the score the xtask driver pins —
and the macro's `audit` gate refuses per program at expansion time,
but nothing answered the project's own question: "which of MY classes
degrade, on which longhand, and why". The observed set was the seam
nobody read. The landing adds it at the codegen layer, backend-neutral
and generic over `Capability`: `ClassAudit::of` tiers the distinct
longhands one compiled program actually sets — first-seen cascade
order, the bitset dedupe, the worst-tier gate reading — with the
boundary reason column riding the SAME one-source-of-truth discipline
the tiers do (`Capability::reason` is the provided method; the GPUI
impl reads the generated `caps_generated.rs` rows its tier impl
already reads — one partition, four renderings now). The CLI's
`aeolus audit` command is the report face: the scan seam's strings
through the themed V1 compile walk into the tiered rows, the
non-native classes named with the capability's own typed boundary
reasons, the observed-set score in the summary, `--fail-on` the CI
gate keyed on the tier (report-only by default — a project that
chooses the polyfill tier is not a failure), `--json` the machine
form. Correctness stays `check`'s job: the audit reports the tiers of
what lowered — the two channels never blur.

* **M1** (the whole landing): the codegen seam (`audit` module —
  `PropAudit`/`ClassAudit`/`AuditReport`), the `Capability::reason`
  provided method, `Caps`' reason override reading the generated table,
  the CLI command (`cmd_audit` + the `audit_string` library seam), the
  red-first suites in all three crates, the bench, the consumer
  example, the README rows, the architecture doc's landed note.

## The shape evidence

The rows are a FUNCTION of the capability's own predicates — the
honesty contract the conformance module documents, pointed at a
project: `tier` answers the emit gates' own verdicts, `reason` answers
the generated table's refused rows (`""` → `None`), so an audit can
never drift from what the emitter actually does. Dedupe is the
`PropMask` bitset's own walk: a longhand two layers set keeps ONE row
(the per-longhand ceiling, never the per-declaration count). Score
arithmetic mirrors `Conformance::score` over the OBSERVED rows
(expressible = native + polyfill), and the empty observation set
answers the vacuous full 100% — NaN is never a published number. The
report is per DISTINCT class string with the first occurrence keeping
the `path:line:col` attribution; the text form names only the
non-native classes (the good news rides the summary line), the JSON
form carries every class.

## RED → GREEN

* RED: `crates/aeolus-codegen/tests/audit.rs` (6 pins, written first —
  the compile failure before the `audit` module existed IS the
  capture: first-seen cascade order + dedupe, the worst-tier ranking +
  the empty program's `None`, the score arithmetic over the observed
  set, the empty set's vacuous full, the report worst aggregation, the
  provided-default `None`, the deterministic Display) +
  `crates/aeolus-codegen-gpui/tests/caps_reason.rs` (3 pins — the
  `Caps.reason` compile failure WAS the capture: the refused-row
  reason parity, the tier-table agreement, the table's `Prop::ALL`
  shape) + `crates/aeolus-cli/tests/audit.rs` (7 pins — the
  unknown-command failures WERE the capture: the per-class text report
  with the generated table's own reasons, the `--fail-on` tier gate,
  the native-only source passing every gate, the JSON shape, the
  themed token walk, the usage grammar, the prose-only file's vacuous
  full).
* GREEN: the same pins pass against the landed face — codegen 6/6,
  codegen-gpui 3/3, cli 7/7; the gate set re-measured post-landing
  (fmt, clippy `--workspace --all-targets --all-features -- -D
  warnings` zero warnings, `cargo test --workspace --all-features`,
  `cargo test --workspace --benches`).

## Bench

`cargo bench -p aeolus-cli --bench audit` (divan, release):
`audit_button_themed` **8.90 µs** median (the Button slice's real
shape — the themed compile walk dominating the tiered-row read,
the ring family riding its polyfill tier) and `audit_refusals_themed`
**1.76 µs** median (the refusal-heavy battery — every row a verdict
plus the generated table's boundary reason).

## Consumer examples

`crates/aeolus-cli/examples/audit.rs` (the library seam
`audit_string` driven over the battery's own four class stories —
native, the ring polyfill half, the z-index and blur refusals — the
score line and the deterministic rendered table headless):

```sh
cargo run -p aeolus-cli --example audit
```

## Docs

README's tooling block and tool table name the audit face (the report
form, the tier gate, the JSON form), the examples index carries the
demo row, and the architecture doc's §5.1 paragraph files the landed
note beside the conformance score's own (the seam, the provided
`reason` method, the CLI command — one partition, four renderings).
