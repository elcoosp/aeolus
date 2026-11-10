# 15 — The P2 custom-authoring seam (RED→GREEN evidence)

Milestone: the pipeline's fifth slice — `@utility`/`@custom-variant`
(architecture plan §5.4, the remaining P2 line): the custom-authoring
seam in `aeolus-compile` — `Customs` registrations (custom utilities
and custom variant aliases) lowered and rendered through the
`*_with` entry points. Branch: `feat/custom-utility-variant-seam`.

## Method note (the first slice with a toolchain)

Unlike reports 13/14 (whose sandboxes had no Rust toolchain — RED
captured by construction), this handoff's sandbox runs Rust 1.99.1
stable: the RED→GREEN cycle ran for real, and the gates ran against
the whole tree.

1. The suite `crates/aeolus-compile/tests/custom.rs` was authored
   FIRST as the RED spec — 20 ungated tests across six families
   (registration + precedence, value forms, custom variants, the CSS
   arm, the empty-Customs differential, the merge). RED captured:
   `error[E0432]: unresolved imports aeolus_compile::{compile_with,
   compile_theme_with, emit_css_with, CustomForm, Customs,
   CustomUtility, CustomVariant}` — the API surface did not exist.
2. The implementation landed second: `src/custom.rs` (the seam
   module), the `variant.rs` fold going generic over
   `ClassifyVariant`, the `run`/`run_css` wire-up (registry-first,
   customs on the unknown-root miss; the more-specific-miss rule:
   a custom shape rejection reports the custom root), and the four
   `*_with` entry points. GREEN: 20/20.
3. The theme-token functional form (`CustomForm::Token`, the M2
   milestone) landed third against its seven-test family — the same
   RED (no such variant) → GREEN cycle.
4. The bench (`compile_custom_utility_cold`,
   `compile_custom_token_utility_cold`) and the consumer example
   (`examples/custom_seam.rs`) landed last, mirroring the slice 1–4
   pattern.

## The pre-handoff reality (drift the gates surfaced)

The task brief said "gates green pre-handoff"; the tree at `13402a0`
did not COMPILE. Five authored suites pinned contracts their
implementations never satisfied — the same generation of
half-landed work the typed-channel refactor left behind:

| # | Site | What broke | Fix |
| --- | --- | --- | --- |
| 1 | `aeolus-syntax/src/candidate.rs` | the typed `LexFailure` import dropped (E0425) | restore the import |
| 2 | `aeolus-compile/src/css.rs` | missing `variant_diagnostic` import; `split_once('/').unwrap_or((seg, None))` type mismatch; `Val::Tok`'s alpha shadowing the outer `Option<f32>` (`f32 is not an iterator`) | import; `unwrap_or((seg, ""))` + empty-label form; rebind the slot alpha |
| 3 | `aeolus-compile/src/lib.rs` | `const` pin matched `&str` (`matches!` on strings is not a const trait on the MSRV toolchain) | byte-wise const compare |
| 4 | `aeolus-compile/src/css.rs` | the rendered rule body lost its `;` (16 authored css pins failed) | `format!("{}: {};")` |
| 5 | `tests/compile.rs`/`tests/theme.rs` | `Diagnostic::code()`/`help()` — the methods live on `DiagnosticKind` since the typed-channel refactor | `.kind.code()`/`.kind.help()` |
| 6 | `aeolus-ir/src/program.rs` | `base_conflicts()` ignored the base layer's own duplicates — the authored pin `compile("p-4 px-8") → base_conflicts() == PADDING_X_MASK` (one base layer, six decls) returned EMPTY | the base rule's front-to-back read counts within-layer re-sets |
| 7 | `aeolus-theme/src/lib.rs` | every token paired: `dark_variant` re-parses the light source, so `dark.color(name)` answers every light name — `bg-red-500` baked to a `ModePair`, the `/50` bake hit `AlphaOnPair` | pairing requires a REAL value override (the audited approximate colour equality is the compare) |
| 8 | `aeolus-compile/src/{lib,css}.rs` | the F1 fraction was unreachable: `w-1/2` lexes body `w-1` + raw modifier `2`; the modifier gate fired before the read side and the fraction shape never saw the joined suffix | the joined body reads FIRST (the oracle's sizing-family precedence, resolve.rs `fraction()`), mirrored in both walks; the modifier is SPENT (no opacity rides it) |
| 9 | `tests/css.rs` | the group/peer heads escaped their probe colons (`.group\/menu\:hover`); the baked-pair companion rendered as a compound suffix | heads are `.<class>[:<probe>]` — the class half escapes, the probe renders raw; the companion prefixes `:where(.dark, .dark *) ` (the authored fixtures' form); the unnamed peer fixture's `.peer\:focus` corrected to the real tailwindcss output shape `.peer:focus` |
| 10 | `tests/compile.rs` | `unsupported_variants_are_typed_and_skip_the_utility` asserted `PaddingTop == Rem(1.0)` — contradicting its own name, its own comment ("the REST of the string still compiles"), the CSS suite's `boundaries_ride_the_sheet_diagnostics` (`rules.is_empty()`) and the v0.x oracle (`Route::Skip`) | the assertion pins p-2's `Rem(0.5)` plus the refused candidate's absence |

Each fix kept the authored contract that had evidence on its side
(the fixtures, the oracle, the sibling suite) and corrected the
drifted side; none weakened a test.

## What the RED spec pins (the contract table)

| Crate | Family | Test | Contract |
| --- | --- | --- | --- |
| compile | Registration | `a_registered_functional_utility_lowers_like_a_row` | the custom spacing form lowers the TRBL fan-out; base layer |
| compile | Registration | `built_in_roots_win_name_collisions` | a custom `p` is shadowed; the built-in row lowers |
| compile | Registration | `custom_misses_carry_the_typed_read_side` | unknown root vs `ShapeRejected { root: "brand-p", tried: ["spacing"] }` — the custom root named, the same stable code |
| compile | Value forms | `functional_spacing_honours_the_config_and_the_px_step` | the `Spacing::Px` bake and the `px` step, byte-shared with the built-in scale |
| compile | Value forms | `numeric_forms_lower_their_grammar` | `Num(2.5)`/`Num(3.0)`/`Pct(40.0)` through integer/number/percentage |
| compile | Value forms | `integer_refuses_a_fractional_suffix` | `--value(integer)`'s integral shape |
| compile | Value forms | `negation_rides_functional_forms_and_refuses_on_fixed` | `-brand-p-2` → `Rem(-0.5)`; `-brand-r` → `UnsupportedNegation` |
| compile | Value forms | `a_fixed_keyword_slot_is_a_typed_refusal` | `Val::Kw` fixed → `UnsupportedValue` (the slot is row-relative) |
| compile | Value forms | `non_finite_suffixes_refuse_like_the_built_in_grammar` | the two-stage numeric shape: accept the parse, the finite guard refuses |
| compile | Variants | `a_registered_alias_folds_to_its_condition` | `hocus:` → `Cond::Hover` |
| compile | Variants | `stacked_aliases_fold_by_the_dominant_condition` | `hocus:md:` → `Bp(Md)` |
| compile | Variants | `an_unregistered_alias_stays_the_typed_boundary` | `UnsupportedVariant` |
| compile | Variants | `an_alias_of_the_dark_tier_carries_the_mode_condition` | `night:` → `Cond::Dark`, rank 1 |
| compile | CSS arm | `the_css_arm_renders_a_custom_utility_rule` | the escaped selector + cascade-ordered block |
| compile | CSS arm | `the_css_arm_renders_the_alias_probe` | `.hocus\:brand-r:hover` — the alias's own probe |
| compile | CSS arm | `a_fixed_rgba_renders_the_rgb_form` | `rgb(r g b)` |
| compile | Differential | `empty_customs_are_byte_identical_to_the_theme_less_siblings` | 14-string battery, `compile == compile_with` byte-for-byte |
| compile | Differential | `the_two_walks_report_identically_with_customs` | IR ≡ CSS diagnostics with customs in play (8-string battery) |
| compile | Differential | `the_two_walks_report_identically_with_token_customs` | the themed battery incl. the token form |
| compile | Theme half | `themed_customs_lower_tokens_through_the_theme_pass` | `bg-primary` + `brand-p-2` lower through both halves |
| compile | Merge | `cn_merges_custom_lowered_programs_last_wins` | rightmost custom wins per longhand; conflict-free out |
| compile | Token form | `the_token_form_lowers_through_the_runtime_tier` | `Tok { id, alpha: 1.0 }` |
| compile | Token form | `the_token_form_bakes_through_the_premultiplied_identity` | `/50` ≡ `tokens.resolve(id, Some(0.5))` — the audited oracle |
| compile | Token form | `the_token_form_stays_a_typed_boundary_without_the_theme_pass` | the token-table boundary, theme-less |
| compile | Token form | `an_unknown_token_name_is_the_typed_not_a_token_miss` | `REASON_NOT_A_TOKEN` |
| compile | Token form | `an_alpha_on_a_baked_pair_stays_the_modifier_boundary` | `UnsupportedModifier` — no alpha arm on the pair |
| compile | Token form | `the_token_form_renders_the_var_and_the_mix` | `var(--color-…)` + the §2 `color-mix` |

## Gates (this sandbox, Rust 1.99.1)

`cargo fmt --all --check`, `cargo clippy --workspace --all-targets
--all-features -- -D warnings -W pedantic` (zero warnings), `cargo
test --workspace --all-features`, `cargo test --benches
--all-features` — captured in the handoff record; the bench numbers
are measured (divan 0.1.21, release profile, 2-core sandbox), filed in
`reports/08-benchmarks/BENCHMARKS.md`.

## Boundaries (documented, not stubbed)

- Selector-bearing custom variants (arbitrary selectors, `has-*`/
  `aria-*` chains): no IR slot — the typed `UnsupportedVariant`
  boundary, the same honesty the flat model gives the built-ins.
- The xtask generator story and the remaining `Target` arms: the next
  slices, unchanged.
- The `text-sm/6` line-height modifier composite: the numeric modifier
  gate refuses it theme-less (the composite line-height arm is a value
  pass slice); no test pins it as lowering.
- Custom utilities' conflict groups are the longhand fan-outs at the
  IR level (the `cn()` contract); the STRING-level `tw_merge` table
  does not know custom names (documented on the seam).
