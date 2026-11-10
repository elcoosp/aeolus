# 14 — The P2 theme pass (RED→GREEN evidence)

Milestone: the pipeline's second slice — `aeolus-theme`, the CSS-first
theme pass, plus the compile signature's theme half
`compile_theme(&str, &TokenTable, &Config)` (architecture plan §3.2/§3.4,
phase P2). Branch: `feat/p2-theme-pass`.

## Method note (honesty first)

This handoff's sandbox has **no Rust toolchain**: the RED→GREEN capture
in the strict sense (failing test output pasted, then the same tests
passing) runs with the gates in the consumer environment, pre-handoff —
the same discipline report 13 recorded. What WAS done here, in TDD
order:

1. The suite `crates/aeolus-theme/tests/theme.rs` was authored FIRST,
   as the RED spec: 14 ungated + 2 feature-gated tests pinning the
   sources, the id contract, the tiers and the alpha identity. It does
   not compile against a stub crate (the API surface it imports —
   `ThemeSource`, `TokenTable`, `Tier`, `ThemeError`, `ResolveError` —
   did not exist before step 2), which is this slice's RED state by
   construction. The crate's inline id-order test and the compile-side
   suite (`crates/aeolus-compile/tests/theme.rs`, 11 ungated + 1
   feature-gated) were authored in the same pass, before the seams they
   pin existed.
2. The implementation landed second (`crates/aeolus-theme/src/lib.rs`,
   then the compile seam's `compile_theme`/`run` split and the value
   pass's token routing in `src/lower.rs`), written strictly against
   the suites and the audited v0.x oracle's public API
   (`TailwindTheme`/`Color`/`shadcn_css` — no parser forks).
3. The bench (`crates/aeolus-theme/benches/tokens.rs`, divan, stable)
   and the two consumer examples (`theme_tiers`, `theme_seam`) landed
   last, mirroring the P1/P2-slice-1 pattern.

Gate capture (`cargo fmt --all --check`, `cargo clippy --workspace
--all-targets --all-features -- -D warnings -W pedantic`,
`cargo test --workspace --all-features`, `cargo test --benches
--all-features`) is the first cargo invocation in the gate environment;
the `Cargo.lock` entries for `aeolus-theme` (member) and
`aeolus-compile` (its new dep edge) are already in place so the run is
`--locked`-clean.

## What the RED spec pins (the contract table)

| Crate | Family | Test | Contract |
| --- | --- | --- | --- |
| theme | Sources | `shadcn_source_builds_a_token_table` | the embedded palettes build; `primary`/`background`/`red-500` are tokens; the default-seeded surface is 270+ |
| theme | Sources | `parse_failures_are_typed_and_carry_the_source` | an unresolvable `var()` chain is `ThemeError::Parse`, Display says `theme CSS` |
| theme | Sources | `source_dark_variant_resolves_the_dark_overrides` | `.dark` overrides re-resolve against the dark palette (`dark_variant` propagation) |
| theme | Ids | `ids_are_sorted_name_order` (inline) | names sorted; `id(name) == index`; `name(id)` inverts |
| theme | Ids | `ids_survive_rebuilds_of_the_same_source` | baked/runtime tables agree per name; the inverse round-trips |
| theme | Tiers | `tokens_without_dark_bake_to_literals` | T0: `Val::Rgba`, `is_token() == false` |
| theme | Tiers | `tokens_with_dark_form_mode_pairs` | T1: `Val::ModePair(slot)`; `pair_at(slot)` == (light, dark) bytes |
| theme | Tiers | `runtime_tier_carries_id_and_unit_alpha` | T2: `Tok { id, alpha: 1.0 }` |
| theme | Tiers | `runtime_alpha_is_carried_not_baked` | `/37` rides the slot (`alpha ≈ 0.37`), nothing bakes |
| theme | Alpha | `alpha_bakes_through_the_premultiplied_identity` | the `/50` bake == `Color::parse(token).mix_with_transparent(50)` — the audited oracle |
| theme | Alpha | `alpha_bounds_are_the_audited_clamp` | `/0` zeroes the alpha byte; `/100` and `/250` clamp to the unmodified token; channels intact |
| theme | Alpha | `alpha_is_monotone_in_the_modifier` | the alpha byte grows with `/10 → /90` |
| theme | Refusals | `unknown_names_and_ids_are_typed` | `id(miss) == None`; `resolve(9999)` → `UnknownId` |
| theme | Refusals | `non_finite_alphas_refuse` | `NaN`/`inf` alphas → `AlphaNotFinite`, never a NaN literal |
| theme | Refusals | `alpha_on_a_baked_dark_pair_is_a_typed_boundary` | T1 + alpha → `AlphaOnPair`; the runtime tier is the documented alternative |
| theme | Gated | `theme_errors_carry_miette_code_and_help` | `aeolus::theme::parse` / `aeolus::theme::alpha_on_pair` + help |
| theme | Gated | `building_and_resolving_under_tracing_keeps_the_contract` | spans are no-ops; the contract is unchanged |
| compile | Theme half | `colour_tokens_lower_through_the_runtime_table` | `bg-red-500 text-primary p-2` → `Tok` decls + the spacing fan-out, zero diagnostics |
| compile | Theme half | `alpha_modifier_becomes_the_token_alpha` | `bg-primary/90` → `Tok { id, alpha ≈ 0.9 }` |
| compile | Theme half | `baked_alpha_uses_the_premultiplied_bake` | the seam's literal == the table's own resolve (the oracle underneath) |
| compile | Tiers | `baked_tier_literals_and_dark_pairs` | `bg-red-500` bakes; `dark:bg-primary` forms the Dark layer's `ModePair` |
| compile | Tiers | `alpha_on_a_baked_dark_pair_is_typed` | `dark:bg-primary/90` → `UnsupportedModifier`, nothing lowered |
| compile | Tiers | `modifier_on_a_non_colour_lowering_is_still_typed` | `text-sm/6` and `bg-primary/[nan]` stay the boundary; `p-4` lowers beside |
| compile | Refusals | `unknown_colour_names_report_the_token_table` | shape-legal but table-absent → `UnsupportedValue` naming `TokenTable::names()` |
| compile | Refusals | `suffixes_outside_the_registry_stay_lookup_misses` | `bg-not-a-token` → the lookup's `ShapeRejected` (the theme pass does not shadow the registry) |
| compile | Silent path | `the_silent_path_is_unchanged_without_a_theme` | the v0.10.0 pins byte-for-byte: colour-token reason, modifier boundary, px config, `Empty` |
| compile | Finite guard | `non_finite_suffixes_are_typed_not_lowered` | `p-nan z-inf w-1e40` → three typed refusals; `p-2` lowers; no non-finite `Val` |
| compile | Finite guard | `negated_and_percent_non_finite_forms_are_typed` | `-mt-nan`, `from-nan%`, and the px bake's overflow refuse; the IR stays finite |
| compile | Gated | `compile_theme_spans_keep_the_contract` | spans are no-ops; the contract is unchanged |

## Discovered while landing (the evidence this slice exists to catch)

1. **The registry's colour table and the theme's token surface overlap
   but are not equal.** `COLOR_KEYWORDS` (270) carries the v4 palette
   AND the shadcn app names (`primary`, `background`, …) — so
   `bg-primary` is shape-legal even to a theme that does not define it.
   The not-a-token miss (`REASON_NOT_A_TOKEN`) is therefore reachable
   only through a source that lacks a registry-legal name (pinned with
   the bare-seed table), and a name the REGISTRY lacks
   (`bg-not-a-token`) dies at the lookup as `ShapeRejected` — the theme
   pass must not shadow the registry read side (pinned).
2. **The alpha never half-applies.** A modifier on a multi-decl
   fan-out (`text-sm/6`'s two longhands) or on a T1 pair refuses the
   whole row (`Reject::Modifier` → the typed boundary with the raw
   modifier text); only all-colour fan-outs carry it. The clamp lives
   in the audited bake, the normalize step only rejects non-finite —
   one source of truth for the bounds.
3. **`Compiled` has no `Default`, so the bench never built.** The
   slice-1 bench used `unwrap_or_default()` on
   `Result<Compiled, CompileError>` — a compile error `cargo test
   --benches` would have caught at the gate. The bodies `expect` (a
   non-empty string always produces a `Compiled`; only the empty input
   hard-fails), and the bench doc says why no `Default` is deliberate.
4. **`f32::from_str` is not a value grammar.** `nan`, `inf`,
   `infinity` (any case) and overflow-to-inf parses are Ok; the spacing
   and number shape contracts accepted them, so `p-nan` lowered
   SILENTLY into the IR as `Val::Rem(NaN)`. The finite guard is now
   typed on every numeric arm, parse and product alike.

## Boundaries (documented, not stubbed)

- The IR-level `cn()` merge, `@utility`/`@custom-variant`, the `Target`
  half of the plan §3.2 signature and the xtask generator: the next P2
  slices; the crate docs say so next to every seam that touches them.
- The v0.x theme reader is the INTERIM oracle: the xtask build-time
  bake (plan §3.4's tracked-path story) replaces the runtime parse; the
  tier model this slice pins is what stays.
- An alpha on a T1 pair is a typed refusal (the runtime tier is the
  answer), and `Val::ModePair`'s u16 slot budget is guarded at build.
- Bench numbers: authored, not measured (no toolchain here); they land
  in `reports/08-benchmarks/BENCHMARKS.md` with the gate run.
