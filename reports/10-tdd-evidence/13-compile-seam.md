# 13 — The P2 compile seam (RED→GREEN evidence)

Milestone: the pipeline's first slice — `aeolus-compile`, the pure
function `compile(&str, &Config) -> Result<Compiled, CompileError>`
(architecture plan §3.1/§3.2, phase P2). Branch: `feat/p2-compile-seam`.

## Method note (honesty first)

This handoff's sandbox has **no Rust toolchain**: the RED→GREEN capture
in the strict sense (failing test output pasted, then the same tests
passing) runs with the gates in the consumer environment, pre-handoff.
What WAS done here, in TDD order:

1. The suite `crates/aeolus-compile/tests/compile.rs` was authored
   FIRST, as the RED spec: 23 ungated + 2 feature-gated tests pinning
   the lowering, the program shape, the variant fold and every honest
   boundary. It does not compile against a stub crate (the API surface
   it imports — `compile`, `Compiled`, `DiagnosticKind`, `Config`,
   `Spacing`, `CompileError` — did not exist before step 2), which is
   this slice's RED state by construction.
2. The implementation landed second (`src/lib.rs`, `src/variant.rs`,
   `src/lower.rs`), written strictly against the suite and the three
   P1 crates' public APIs (no registry/lowering shortcuts).
3. The bench (`benches/compile.rs`, divan, stable) and the consumer
   example (`examples/compile_pipeline.rs`) landed last, mirroring the
   data/syntax crates' P1 pattern.

Gate capture (`cargo fmt --all --check`, `cargo clippy --workspace
--all-targets --all-features -- -D warnings`, `cargo test --workspace
--all-features`, `cargo test --benches --all-features`) is the first
cargo invocation in the gate environment; the `Cargo.lock` entry for
`aeolus-compile` is already in place so the run is `--locked`-clean.

## What the RED spec pins (the contract table)

| Family | Test | Contract |
| --- | --- | --- |
| Value pass | `p4_lowers_to_the_trbl_rem_fanout` | `p-4` → the four padding longhands at `Rem(1.0)`, TRBL order, mask mirrors, `touched()` exact |
| Value pass | `px_config_bakes_pixels_at_the_configured_rem` | the §3.3 spacing knob: `px` config bakes `n × 0.25 × rem_px`; the `px` step is one absolute pixel in both modes |
| Value pass | `negated_spacing_negates_the_value` | `-mt-2` → `MarginTop Rem(-0.5)` |
| Value pass | `fraction_lowers_to_a_percentage` | `w-1/2` → `Width Pct(50.0)` |
| Value pass | `number_scale_is_unitless` | `z-10` → `Num(10.0)`; `opacity-50` → `Num(50.0)` (the per-root scale factor is the codegen's) |
| Value pass | `keywords_lower_to_the_row_table_slot` | `flex-col` → `Val::Kw(slot)` — the slot indexes the ROW's own table (pinned against `tables::FLEX_DIRECTIONS`, not a magic number) |
| Value pass | `bare_roots_lower_to_slot_zero` | `hidden` → `Display Kw(0)` |
| Value pass | `size_keywords_resolve_through_the_anyof_arm` | `w-full` resolves through the row's AnyOf keyword arm, first-accept wins |
| Value pass | `composite_rows_fan_the_pair_across_their_targets` | `text-lg` → `FontSize Kw(i)` + `LineHeight Kw(i)`, one TEXT_SIZES slot |
| Program shape | `base_and_conditional_layers_sort_by_specificity` | base + hover + md sort by specificity; masks are data (`PropMask::of(decls)`) |
| Program shape | `same_condition_duplicates_surface_as_base_conflicts` | `p-4 px-8` emits the raw 4+2 cascade; the overlap is EXACTLY `base_conflicts()` = the px mask (the IR-level `cn()` merge is the next P2 slice, and the bits say so) |
| Variant fold | `stacked_variants_fold_to_the_dominant_condition` | `hover:focus:` → Focus (later segment wins the tie); `md:hover:` → Bp(Md) (media outranks probe) |
| Variant fold | `named_group_variants_fold_to_the_probe` | `group-hover/menu:` → GroupHover; the NAME has no IR slot yet (cond.rs's documented threading) |
| Boundaries | `unsupported_variants_are_typed_and_skip_the_utility` | `has-data-[x]:p-4 p-2` → one spanned diagnostic (the segment's own bytes 0..12), the rest of the string still lowers |
| Boundaries | `importance_is_a_typed_boundary` | `p-4!` → `UnsupportedImportant`, nothing lowered |
| Boundaries | `modifiers_are_a_typed_boundary` | `bg-red-500/50` → `UnsupportedModifier`, help names the theme pass |
| Boundaries | `colour_tokens_report_the_theme_pass` | `bg-red-500` → `UnsupportedValue` with the TokenTable reason (see the discovered budget below) |
| Boundaries | `arbitrary_values_report_the_value_pass` | `w-[32px]` → `UnsupportedValue` with the value-pass reason |
| Boundaries | `negation_the_row_refuses_is_typed` | `-p-4` → `UnsupportedNegation` (the registry's `negative` flag is the legality oracle) |
| Boundaries | `unknown_roots_carry_the_lookup_error` | the miss is spanned (`4..17`) and the rest compiles |
| Boundaries | `shape_rejections_carry_the_tried_labels` | `p-red` → `ShapeRejected { root: "p", tried ⊇ ["spacing"] }` |
| Boundaries | `lex_failures_are_spanned_and_degrade` | `p--4 p-2` → spanned `Lex` diagnostic, `p-2` still lowers |
| Boundaries | `empty_input_is_a_typed_error` | `""`/whitespace → `CompileError::Empty` (the only whole-string failure) |
| Gated | `compile_error_carries_miette_help` (`miette`) | `CompileError` carries code `aeolus::compile::empty_input` + help |
| Gated | `compile_with_tracing_spans_keeps_the_contract` (`tracing`) | spans are no-ops without a subscriber; the contract is unchanged |

## Discovered while lowering (the evidence this slice exists to catch)

1. **The colour table does not fit the IR's keyword slot.**
   `tables::COLOR_KEYWORDS` is 270 entries; `Val::Kw` is a `u8` slot
   (255 max). Truncating was never on the table (no-silent-failures);
   the seam reports `UnsupportedValue` with a reason that names the
   theme pass's `TokenTable` (plan §3.4 T2) as the lowering home. This
   is the first concrete IR budget the pipeline has surfaced, and it is
   now test-pinned instead of latent.
2. **The named group/peer label has nowhere to ride.** `cond.rs`
   documents the compile-stage fold threading `group-hover/menu`'s
   name; the flat `Cond` has no name slot. The fold lands (the probe is
   typed and tested); the name threading is honestly deferred to the
   codegen slice, documented in the fold's rustdoc and the test.
3. **The degraded-lexer pairs lost their spans.** `parse_class_string`
   reports `(raw, message)` pairs; the seam re-derives the byte span
   with the SAME sequential find the lexer uses to span its tokens, so
   editor squiggles stay exact even for repeated tokens.

## Boundaries (documented, not stubbed)

- Theme tiers (`Val::Tok`, T0–T2), IR-level `cn()`, `@utility`/
  `@custom-variant`, the xtask generator: next P2 slices; the crate
  docs say so next to every seam that touches them.
- Importance (`!`) and modifiers (`/50`): typed diagnostics; the codegen
  and theme passes own them.
- Bench numbers: authored, not measured (no toolchain here); they land
  in `reports/08-benchmarks/BENCHMARKS.md` with the gate run.
