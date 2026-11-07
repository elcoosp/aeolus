# TDD evidence — v0.6 upstream ceiling (`reports/10-tdd-evidence/08-upstream-ceiling.md`)

Plan: `AEOLUS_V0.6.md`. Every batch below was written RED first
(failing against the pre-v0.6 code), then GREEN. Capture evidence lives
in `tools/capture_gradients.mjs` (real tailwindcss 4.1.17 output,
re-runnable).

## Batch 1 — upstream reconnaissance (§3, no code)

| step | command | result |
|---|---|---|
| floem pin vs HEAD | `git ls-remote https://github.com/lapce/floem.git HEAD` | `1351ffb162faeb8be983f1301d718a0d5fb59c27` == the pin |
| xilem pin vs HEAD | clone + `git rev-list 9f504ae..origin/main --count` | 3 commits |
| core API delta | `git diff 9f504ae..dc49687 -- masonry_core xilem_masonry` | **empty** — pin stays |
| crates.io | release listing re-check | `masonry*` 0.4.0 (2025-10-29), `floem` 0.2.0 (2024-11-14) — publishing stays blocked |

## Batch 2 — gradient shapes + stops (§1.2)

RED: `cargo test -p aeolus-core --test gradients` failed to compile
(`background_gradient`, `Gradient`, `GradientStop` did not exist).

GREEN after the IR + parser + composition landed. Pinned semantics
(21 tests in `crates/aeolus-core/tests/gradients.rs`):

- `linear_corners_are_css_angles` — to-t…to-tl = 0…315 (capture:
  `--tw-gradient-position: to right` ≡ 90deg).
- `linear_numeric_and_arbitrary_angles` — `bg-linear-45`, `bg-linear-[200deg]`;
  bare `bg-linear` warns Unknown.
- `gradient_to_is_the_deprecated_alias` — `bg-gradient-to-r` ≡ `bg-linear-to-r`.
- `radial_and_conic_shapes` — `bg-radial`, `bg-conic[-45]`.
- `bg_none_clears_like_a_value` — same-bucket clear + `dark:bg-none`
  clears only the dark layer.
- `stops_compose_with_v4_defaults` / `missing_to_stop_is_transparent` /
  `via_composes_between_from_and_to` — the `@property` initial values.
- `position_patches_patch_only_the_position` — `from-20%` keeps the
  color, composes alone.
- `stop_opacity_modifier_uses_the_v4_color_mix_shape` — `/50` alpha.
- `stops_without_a_shape_compose_nothing` — v4's unset position.
- `solid_and_gradient_coexist` — both slots set, either class order.
- diffs/lanes: `gradient_changes_diff_through_the_cosmetic_lane`;
  conflicts: `cn_keeps_solid_and_gradient_but_merges_per_group`.

## Batch 3 — the verification emitter (§1.4)

`emitter_linear_shape_is_the_v4_var_machinery`,
`emitter_stop_classes_set_the_captured_chains` (incl. the via
`--tw-gradient-via-stops` routing), `emitter_bg_none_is_verbatim`,
`currentcolor_stops_report_precisely`. RED: the emitter had no gradient
arms. GREEN: the captured per-class declaration shapes.

## Batch 4 — masonry native gradient lowering (§1.5)

RED: `cargo test -p aeolus-xilem --test lowering` — no
`Background::Gradient` in the lowered set.

GREEN (`crates/aeolus-xilem/tests/lowering.rs`, 7 new tests):

- `linear_gradient_angle_conversion` — to-t→π, to-r→π/2, to-b→0,
  bg-linear-45→π−45° (the `a = π − θ` derivation from
  `get_peniko_linear_for_rect`).
- `gradient_lowers_natively_with_v4_stop_defaults` — ZERO drops,
  oklab interpolation, Pad, offsets 0/1.
- `via_and_position_patches_land_in_the_stop_offsets` — 0.2/0.5/0.8.
- `gradient_wins_over_solid_in_the_single_brush` + the recorded note.
- `radial_lowers_as_documented_circle` — CircleTo(FarthestCorner) +
  the ellipse→circle note.
- `conic_reports_the_upstream_sweep_todo`; `bg_none_lowers_to_no_gradient_property`.

## Batch 5 — the floem unwired surface (§2)

RED: `cargo test -p aeolus-floem --test lowering` — `OverflowX` unset
(default Visible), `Selectable` default true, no grid templates.

GREEN (8 new tests): `overflow_lowers_to_per_axis_taffy_props`
(hidden + auto → Scroll), `select_none_lowers_to_the_selectable_prop`,
`whitespace_maps_the_single_text_overflow_prop` (normal wraps; nowrap
= no-op; pre reports), `content_align_lowers_to_taffy_align_content`,
`grid_lowers_templates_and_placement_natively` (repeat(4, 1fr);
`Line { start: Line(3), end: Span(2) }`; bare span →
`Line { Span(2), Auto }`), `border_style_lowers_to_kurbo_stroke_patterns`
(dashes [6,6]; round-cap dotted; border-none zeroes; double reports),
`gradients_report_with_the_sharpened_reason`,
`slot_coverage_upgrades_are_consistent_with_the_lowering`.

## Regression caught by the batch-5 work

`overflow-hidden overflow-x-auto` replaced the whole overflow slot
(last class wins) — CSS patches per axis. Fixed in `finalize` with an
explicit per-axis merge; pinned by
`overflow_slots_carry_per_axis_kinds` (core) and
`overflow_lowers_to_per_axis_taffy_props` (floem).

## Final battery (2026-11-07)

`cargo test --workspace` — 221 passed / 0 failed.
`clippy --workspace --all-targets -- -D warnings`, `fmt --check`,
`RUSTDOCFLAGS=-D warnings cargo doc --workspace --no-deps`,
`cargo package -p aeolus-core -p aeolus-macro`, `cargo deny check
licenses` — all green.
