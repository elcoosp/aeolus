# TDD evidence — test-pyramid hardening (`10-test-pyramid.md`)

The milestone: rebuild the pyramid's UNIT base, add the missing E2E
(process) layer, and turn the plan-algebra laws into property tests. The
workflow follows `CONTRIBUTING.md` §1 — but this round the properties
did the TDD work: **the property suite found four real defects**, each
fixed RED → GREEN below.

## The pyramid before / after

| layer | before | after |
|---|---|---|
| Unit (`#[cfg(test)]` in src) | 68 tests, 0 in parser/plan/eval/selector/cond/resolve/caps/ast | 101 tests — every core module carries a suite |
| Integration (tests/) | 190+ tests, rich | + `pyramid_properties.rs` (4 laws × 256 cases) |
| E2E (process) | none — `run()` called in-process only | `crates/aeolus-cli/tests/process.rs`, 9 tests via the real binary |
| Property (proptest) | `properties.rs` (4), `snapshot_roundtrip.rs` (1) | + plan-algebra laws over full-pipeline plans |
| Fuzz | 6 targets | unchanged |
| Golden corpus | 193 entries | unchanged |
| Bench | `hot.rs` + cli `pipeline.rs` | + `benches/snapshot.rs` (5 benches, report 08) |

## §1 The four defects the property suite caught (RED → GREEN)

1. **`diff` collapsed the text family onto one bit** (`plan.rs`).
   `Slot::index()` mapped every `Slot::Text(_)` to 18, so the bitset xor
   reported a text change as `Text(FontSize)` — the canonical first
   property — and `slot_differs(Text(FontSize))` on the shared-bit walk
   MISSED a `FontWeight`-only change entirely (e.g. `font-medium` →
   `font-bold` never diffed). RED: `overlay_changes_are_contained`
   counterexample `base = ""`, `over = "font-medium"`. GREEN: per-property
   indices (`Slot::Text(p) => 18 + p.index()`), the extras family
   renumbered, `TextProperty::index()` added and pinned against
   `TEXT_PROPS`. Regression tests:
   `diff_reports_the_actual_text_property_changed`,
   `text_property_indices_match_the_canonical_table`.
2. **The snapshot dropped the v0.7 grid templates** (`snapshot.rs`).
   The writer/reader (restored from the pre-v0.7 tree) serialized only
   `cols/rows/col_span/row_span/col_start/row_start` —
   `template_cols/template_rows/auto_rows/auto_cols` were silently
   dropped, violating the artifact no-silent-failures rule. RED:
   `snapshot_roundtrip_is_the_identity`, counterexample
   `has-data-[slot=card-action]:grid-cols-[1fr auto]`. GREEN: `write_tracks`
   /`read_tracks` (track/repeat components, min/max sizing, keywords),
   strict single-key objects, unknown fields rejected.
3. **The snapshot dropped `Layout.justify_items`** (v0.7 §5). RED: same
   law, counterexample `justify-items-start`. GREEN: the layout object
   grew the `justify_items` field (writer + reader, `AlignItems` reuse).
4. **The snapshot dropped `Extras.container_type`** (v0.7 §3). RED: same
   law, counterexample `@container/card-header`. GREEN: `container_type`
   `{name}` written/read strictly (null / named / unnamed).

Every fix kept the strict-reader contract: unknown keys and malformed
shapes are typed `SnapshotError`s with dotted paths, never defaults.

## §2 The snapshot schema version is honest (1 → 2)

The IR gained `Atom::Has(Test)` (v0.7 §2) and `Test::Pseudo` (§1); the
`has` field moved from a bare state string to a full test object and
`pseudo` joined the test grammar. A version-1 reader cannot read these
artifacts and a version-2 reader rejects version-1 `has` shapes — so
`PLAN_SNAPSHOT_VERSION` is 2, the header error pins the new constant, and
the round-trip tests pin `"version":2` bytes.

## §3 The new unit suites (characterization → pinning)

The six pipeline modules had integration coverage but no focused unit
tests — every ordering, band and routing decision was pinned only
END-to-end. The new suites pin the internals at the type level:

- `cond.rs` (3): the `Atom::order` band table (the cascade contract),
  `Cond::rank` max-atom semantics, `Test::Pseudo` as its own arm.
- `ast.rs` (3): pseudo-element names/CSS suffixes, structural canonical
  names, breakpoint index/name lockstep.
- `caps.rs` (2): the `supports()` routing table (flag-backed slots vs the
  negotiated extras family), `report_unsupported` totality.
- `eval.rs` (8): the default probe answers "no" everywhere, per-arm
  routing, `in_window` `[min, max)` boundaries, `split_group_label`
  longest-suffix, rank-beats-source-order (focus over hover), dark beats
  the state band, responsive uses the plan's OWN table, and
  `resolve_rules_only` bucket exclusion.
- `selector.rs` (5): the documented v0.3 §1.3 boundary (lateral,
  supports, An+B, relative, ancestor, class selectors, bare `&`) with
  exact error strings; the typed self-test forms; `single_test`
  one-test-only; nth forms.
- `parser.rs` (5): the named-variant table (states, pseudo-elements,
  media, bounds, containers, Custom fallback), token shapes (negative,
  arbitrary, raw modifier, honesty errors), per-class totality of
  `parse_class_string`.
- `plan.rs` (10): `ResolvedStyle::overlay` per-slot later-wins + text
  per-property merge; plan-level overlay (layers by condition,
  responsive/dark union, rules by conjunction, breakpoints-wins,
  idempotence); `diff` slots + lanes; the two §1 regression tests.

Two suite-internal expectation bugs were corrected during GREEN (the
implementation was right): the default-probe test carried a tautology,
and the responsive test assumed the v4 breakpoints on a DEFAULT plan —
the derived default is `[0; 5]`; real plans carry the theme's table.

## §4 The property laws (full-pipeline generation)

`tests/pyramid_properties.rs` generates class strings from a curated
alphabet of REAL utilities (95 entries covering every plan region —
including the v0.7 surface), resolves them against the shadcn theme, and
checks the laws over the resulting PLANS (256 cases each):

1. **Overlay idempotence** — `p.overlay(o).overlay(o) == p.overlay(o)`.
2. **Per-slot later-wins / change containment** — every slot
   `diff(base, merged)` reports is a slot the override defines.
3. **Snapshot round-trip identity** —
   `from_snapshot_json(to_snapshot_json(p)) == p`, byte-stable.

Laws 1–3 run through the REAL pipeline (parser → resolver → plan), so a
regression anywhere upstream fails the law — that is how §1's defects
surfaced.

## §5 The process e2e layer

`crates/aeolus-cli/tests/process.rs` spawns the REAL binary
(`CARGO_BIN_EXE_aeolus`) — the layer `tests/cli.rs` (in-process `run()`)
cannot pin: `main()`'s delegation, OS exit codes, real pipes. Pinned:
stdout is a v2 snapshot artifact (parseable), warnings go to stderr and
stay non-fatal, byte-determinism across processes, the exit contract
(0 / 1 findings / 2 usage-IO-theme), `diff` lanes, `cn` merge stability,
`check` exit 1 on findings / 0 clean, theme selection + error paths, and
the large-class-list case carrying every layer and the v0.7 shapes.

## §6 What the milestone changed in production code

Exactly two source files (plus the restored wiring): `plan.rs` (the diff
bit fix, `TextProperty::index`) and `snapshot.rs` (the three lost v0.7
fields + schema version 2). No MSRV change, no new dependencies, no
backend changes. The full workspace suite is green after each fix.
