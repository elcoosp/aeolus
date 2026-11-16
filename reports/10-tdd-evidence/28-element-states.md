# 28 — The element-state family

Slice: the element-state variants' answer — `target:`/`indeterminate:`/
`autofill:`/`read-only:`, the element-state tier beside the validation
quartet (the v0.x `State` surface ALREADY carries the four forms — the
family closes the v0.x/V1 gap; the honest-failure boundary carried
forward from evidence 27: `enabled:`/`required:`/`optional:`/
`in-range:`/`out-of-range:` stay outside — a non-form element matches
NEITHER half, so one `Env` bit cannot answer them honestly — and
`open:`/`inert:`/`default:` stay the v4.1.17 spellings the v0.x
surface has no slot for, the direction-family precedent's own
boundary). Evidence file per the workspace convention: the RED spec
written first, the GREEN run that satisfies it, the bench and the
consumer example. Two red→green cycles, one per milestone:

* **M1** — the family at the pipeline level: the four bare states
  join the `Cond` surface (`Cond::Target`/`Cond::Indeterminate`/
  `Cond::Autofill`/`Cond::ReadOnly` — the `hover:` shape, the probe
  tier, specificity 2, NOT an at-rule), the fold arms (the CLOSED
  state/forms-block set extends), the `state_pseudo` table the `has-`
  inner grammar reads (one grammar, three consumers), the Env facts
  (ONE fact per question — `url_target`, `indeterminate`,
  `autofilled`; `read-only:` rides the editability fact's COMPLEMENT,
  `read_write`, the `validation_invalid` precedent: CSS's own reading
  for a non-editable element IS `:read-only`, so the resting env
  answers the `read-only:` query — an unset fact the QUIET miss a
  selector that does not match is, never an exclusion note), the CSS
  arm's upstream pseudo-classes (the v4.1.17 source's own
  `staticVariant` texts — `:target`, `:read-only`, no at-rule head;
  BOTH the built-in segment wall and the alias path's
  `apply_cond_shape` render them), the metamorphic checker's
  conjunctive env (the one-fact states bake the bit true, the
  read-only complement bakes the editability bit false), the GPUI and
  floem walks (the same env semantics verbatim), the four plan-lane
  backends' probe paths honestly keep the facts off, the LSP labels
  the family, and the plan bridge LANDS the state slots (the v0.x
  `State` surface's own target/indeterminate/autofill/read-only
  forms — the `checked:` precedent, no note rides). The completion
  surface carries the four spellings (`KNOWN_VARIANTS` beside the
  two-sided audit).
* **M2** — the CLI face: `expand --target`/`--indeterminate`/
  `--autofill`/`--read-write`, the element-state facts' headless
  flags (the validation flags' sibling — element state needs no facts
  row, the `--dark` precedent). One flag per FACT: `--target` the
  fragment-target fact, `--indeterminate` the checkbox/radio answer,
  `--autofill` the filled-form answer, `--read-write` the
  editability fact (`read-only:` is its resting-env complement, it
  takes no flag). Unset is the quiet miss a selector that does not
  match is.

## The differential evidence

The tailwindcss v4.1.17 source
(`packages/tailwindcss/src/variants.ts`), the state/forms blocks'
`staticVariant` calls — the four spellings are the family's whole
one-fact set (the matches-neither spellings stay out on evidence
27's own reasoning; `read-write` has NO v4.1.17 registration at all):

```text
staticVariant('target', ['&:target'])
staticVariant('indeterminate', ['&:indeterminate'])
staticVariant('autofill', ['&:autofill'])
staticVariant('read-only', ['&:read-only'])
```

## RED (spec written first)

```text
$ cargo test -p aeolus-compile --test element_states
error[E0599]: no variant, associated function, or constant named `Target`
              found for enum `aeolus_ir::Cond`
error[E0599]: no variant, associated function, or constant named `Indeterminate`
              found for enum `aeolus_ir::Cond`
error[E0599]: no variant, associated function, or constant named `Autofill`
              found for enum `aeolus_ir::Cond`
error[E0599]: no variant, associated function, or constant named `ReadOnly`
              found for enum `aeolus_ir::Cond`
(12 pins, exit 101 — the fold refuses every element-state spelling)

$ cargo test -p aeolus-cli --test cli expand_
test result: FAILED. 36 passed; 5 failed
(expand_target_flag_answers_the_layers,
expand_without_element_state_flags_is_a_quiet_miss,
expand_indeterminate_flag_answers_the_layers,
expand_autofill_flag_answers_the_layers,
expand_read_only_answers_the_complement — the flags are not in the
fold's allow-list yet)
```

## GREEN (the battery satisfies the spec)

```text
$ cargo test -p aeolus-compile --test element_states
test result: ok. 12 passed; 0 failed

$ cargo test -p aeolus-cli --test cli
test result: ok. 102 passed; 0 failed
```

## Bench (divan, release, 2-core sandbox, 2026-11-16)

```text
$ cargo bench -p aeolus-compile --bench element_states
interp_element_state_button        median 138.7 ns   (the fire path)
interp_element_state_default_env   median 50.11 ns   (the resting path)
```

## Consumer examples

```text
$ cargo run -p aeolus-compile --example element_states
(the fire, the resting complement, the quiet miss, the CSS head — headless)
$ cargo run -p aeolus-cli --example expand_element_states
(the fact-bearing form, the resting form, the complement form — headless)
```
