# TDD evidence 11 — the variant & grid engine (v0.7)

Plan of record: `AEOLUS_V0.9.md`. Every behavioral change below was
written test-first: the failing test ran RED, then the implementation
turned it GREEN. Commands were run in the development sandbox
(rustc 1.99.1, Linux); outputs are abridged to the decisive lines.

## 1. The fidelity audit — the RED that started the milestone

The oracle-fidelity probe (a temporary test over the shipped tables,
deleted after the milestone) resolved every shadcn v4 class table against
the v0.6 resolver:

```
== button_default: 4 warnings      ([&_svg]:… ×3, has-[>svg]:px-3)
== input: 10 warnings              (selection: ×2, file: ×7, placeholder: ×1)
== card_header: 5 warnings         (@container/card-header, auto-rows-min,
                                    grid-rows-[auto_auto], has-data-[…], [.border-b])
== badge_default: 2 warnings       ([&>svg]:… ×2)
== alert_destructive: 7 warnings   (grid-cols-[0_1fr], has-[>svg]:… ×2,
                                    [&>svg]:… ×3, *:data-[slot=…]:…)
== alert_description: 2 warnings   (justify-items-start UNKNOWN, [&_p]:…)
TOTAL warnings across shipped tables: 40
```

Every warning class except the `[&…]`/`[.border-b]` boundary set is a
parser/IR gap — the plan of record's table.

## 2. Pseudo-element variants (v0.9 §1)

RED (compile failure = the typed model does not exist yet):

```
error[E0432]: unresolved imports `plan::GridTrack`, `plan::PseudoElement`, `plan::TrackSizing`
error[E0599]: no variant ... named `Pseudo` found for enum `Test`
```

GREEN (`cargo test -p aeolus-core --test variant_engine`):

```
test selection_variant_is_a_typed_pseudo_rule ... ok
test placeholder_variant_is_a_typed_pseudo_rule ... ok
test file_variant_is_a_typed_pseudo_rule ... ok
test selection_stacks_with_other_variants ... ok
test pseudo_rules_never_fire_on_a_default_probe ... ok
test pseudo_rules_fire_when_the_probe_answers ... ok
test result: ok. 19 passed; 0 failed
```

The real-compiler pin (the emitter test asserted `::selection`, the
corpus pinned the declaration shape):

```
.selection\:bg-primary {
  & *::selection { background-color: var(--color-primary); }   ← descendant: normalization
  &::selection { background-color: var(--color-primary); }     ← self: corpus-pinned
}
```

## 3. has-data routing (v0.9 §2)

RED: `has_data_routes_to_a_typed_descendant` failed with the old report
(`unknown has-<state> variant`). GREEN: the rule classifies onto
`Atom::Has(Test::Data("slot", Some("card-action")))` with zero warnings,
and the emitter reproduces v4's star form — captured:
`&:has(*[data-slot="card-action"])`, `&:not(*[data-state="open"])` (the
latter fixed the emitter's bare-attr `:not` shape, never corpus-pinned
before).

## 4. Named containers, justify-items (v0.9 §3/§5)

RED: `container_utility_establishes_containment` /
`named_container_utility_carries_the_name` /
`justify_items_lands_in_the_layout_slot` (compile failures — no slot, no
arm; `justify-items-start` resolved as `Unknown` with a misleading
suggestion). GREEN: `Extras.container_type` + `Layout.justify_items`
land; the real-compiler capture matched the emitter byte-for-byte:

```
.\@container\/card-header { container-type: inline-size; container-name: card-header; }
.justify-items-start { justify-items: start; }
```

## 5. Grid templates (v0.9 §4)

RED: `grid_cols_arbitrary_tracks` … `grid_template_and_span_compose` +
the floem lowering suite (`grid_arbitrary_templates_lower_to_taffy_tracks`
et al.). Two REAL-COMPILER findings during GREEN worth recording:

1. `auto-rows-fr` emits **`minmax(0, 1fr)`** — the blowout guard — not
   the CSS `1fr` shorthand (`minmax(auto, 1fr)`). First implementation
   stored the shorthand; the corpus caught it:

```
default:auto-rows-fr [.auto-rows-fr ] grid-auto-rows:
  golden "minmax(0, 1fr)" vs ours "1fr"
```

2. v4 emits arbitrary values VERBATIM (`repeat(3,minmax(0,1fr))`, the
   raw `0`) but keyword forms pretty (`minmax(0, 1fr)`) — comma spacing
   became a documented harness normalization.

GREEN (floem, native taffy surface at the pin):

```
test grid_arbitrary_templates_lower_to_taffy_tracks ... ok
test grid_arbitrary_templates_beat_the_numeric_counts ... ok
test auto_rows_and_cols_lower_to_the_implicit_track_props ... ok
test justify_items_lowers_to_the_native_prop ... ok
test result: ok. 22 passed; 0 failed
```

The v0.6 expectation that changed intentionally (never deleted —
updated to the new capability): `shadcn_components.rs::alert_resolves`
asserted `grid-cols-[0_1fr]` reports precisely; it now asserts the
template lands (`[0px, minmax(auto,1fr)]`, zero warnings).

## 6. cn() conflict groups (v0.9 §4)

RED: `grid_template_families_conflict` — `cn(&["grid-cols-3",
"grid-cols-2"])` kept BOTH (no group existed). GREEN after adding the
four template groups + `container-type`:

```
test conflict::tests::grid_template_families_conflict ... ok
```

## 7. The headline (v0.7 §7)

`crates/aeolus-shadcn/tests/oracle_fidelity.rs`, GREEN:

```
FIDELITY LEDGER: 13 warnings remain, all documented boundary (13 boundary)
test shipped_shadcn_tables_resolve_warning_free_up_to_the_boundary ... ok
test component_compositions_resolve_warning_free_up_to_the_boundary ... ok
test new_rules_carry_the_requested_styles ... ok
```

40 → 13, all 13 the documented boundary (`[&_svg]`, `[&>svg]`, `*:data-…`,
`[.border-b]`, `has-[>svg]`).

## 8. Final gates (2026-11-08)

- `cargo test --workspace` — 340 passed, 0 failed (incl. the 193-entry
  golden corpus).
- `cargo clippy --workspace --all-targets -- -D warnings` — clean
  (3 lints caught and fixed: collapsed if-let, redundant float guard,
  f64-literal fallbacks in the floem tests).
- `cargo fmt` — applied; `RUSTDOCFLAGS="-D warnings" cargo doc` — clean.
- `cargo package -p aeolus-core -p aeolus-macro --allow-dirty --no-verify`
  — packaged.
- Benchmarks: `cargo bench -p aeolus-core` — recorded in report 08's
  v0.7 section; the exact-zero allocation pins held unchanged.
