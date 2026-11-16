# 27 — The validation states

Slice: the form-state variants' answer — `valid:`/`invalid:`/
`user-valid:`/`user-invalid:`, the element-state tier beside
`hover:`/`checked:` (the v0.x `State` surface ALREADY carries the four
forms — the family closes the v0.x/V1 gap; the honest-failure
boundary: `read-write:` stays outside, no such spelling exists in the
pinned v4.1.17 source). Evidence file per the workspace convention:
the RED spec written first, the GREEN run that satisfies it, the bench
and the consumer example. Two red→green cycles, one per milestone:

* **M1** — the family at the pipeline level: the four bare states
  join the `Cond` surface (`Cond::Valid`/`Cond::Invalid`/
  `Cond::UserValid`/`Cond::UserInvalid` — the `hover:` shape, the
  probe tier, specificity 2, NOT an at-rule), the fold arms (the
  CLOSED forms-block set extends), the Env facts (the constraint pair
  rides ONE fact — `validation_invalid` answers `invalid:` on the
  bit, `valid:` its COMPLEMENT, the `rtl:`/`ltr:` precedent: CSS's
  own reading for a pristine field is `:valid`, the resting env
  answers the `valid:` query — plus the interaction gate the HTML
  spec's `:user-*` pseudo-classes require, `user_validated`; BOTH
  `user-` spellings stay quiet until the gate is set, past it the
  SAME constraint fact answers), the CSS arm's upstream pseudo-classes
  (the v4.1.17 source's own `staticVariant` texts — `:valid`,
  `:user-invalid`, no at-rule head), the metamorphic checker's
  conjunctive env (the complement bakes the bit false, the gate rides
  WITH the constraint fact), the GPUI and floem walks (the same env
  semantics verbatim), the four plan-lane backends' probe paths
  honestly keep the facts off, the LSP labels the family, and the
  plan bridge LANDS the state slots (the v0.x `State` surface's own
  forms block — the `checked:` precedent, no note rides). The
  `has-` inner grammar answers the states beside `checked:` (one
  grammar, three consumers).
* **M2** — the CLI face: `expand --invalid`/`--user-valid`/
  `--user-invalid`, the validation facts' headless flags (the
  capability flags' sibling — element state needs no facts row, the
  `--dark` precedent). One flag per SPELLING: `--invalid` the
  failing-constraints fact (`valid:` is its resting-env complement,
  it takes no flag), `--user-valid` the gate WITH the holding fact,
  `--user-invalid` the gate WITH the failing fact. Unset is the
  quiet miss a selector that does not match is.

## The differential evidence

The tailwindcss v4.1.17 source
(`packages/tailwindcss/src/variants.ts`), the forms block's
`staticVariant` calls — the four spellings are the family's whole
`user-`-carrying set (`required`/`optional`/`in-range`/`out-of-range`
stay outside this slice: the required/optional pair is not one
complement over every element — a non-form element matches NEITHER,
so one `Env` bit cannot answer it honestly; `read-write` has NO v4.1.17
registration at all):

```text
staticVariant('valid', ['&:valid'])
staticVariant('invalid', ['&:invalid'])
staticVariant('user-valid', ['&:user-valid'])
staticVariant('user-invalid', ['&:user-invalid'])
```

## RED (spec written first)

```text
$ cargo test -p aeolus-compile --test validation
error[E0599]: no variant, associated function, or constant named `Valid`
              found for enum `aeolus_ir::Cond`
error[E0599]: no variant, associated function, or constant named `Invalid`
              found for enum `aeolus_ir::Cond`
error[E0599]: no variant, associated function, or constant named `UserValid`
              found for enum `aeolus_ir::Cond`
error[E0599]: no variant, associated function, or constant named `UserInvalid`
              found for enum `aeolus_ir::Cond`
(11 pins, exit 101 — the fold refuses every validation spelling)

$ cargo test -p aeolus-cli --test cli expand_
test result: FAILED. 33 passed; 3 failed
(expand_invalid_flag_answers_the_layers, expand_valid_answers_the_complement,
expand_user_flags_answer_past_the_gate — the flags are not in the
fold's allow-list yet)
```

## GREEN (the battery satisfies the spec)

```text
$ cargo test -p aeolus-compile --test validation
test result: ok. 11 passed; 0 failed

$ cargo test -p aeolus-cli --test cli
test result: ok. 97 passed; 0 failed

$ cargo test -p aeolus-compile -p aeolus-ir -p aeolus-core
(all suites ok — the two-sided completion audit pins the four
spellings beside the fold arms)
```

## Bench (divan, release, 2-core sandbox, 2026-11-16)

```text
$ cargo bench -p aeolus-compile --bench validation
interp_validation_button        median 185.8 ns   (the fire path)
interp_validation_default_env   median 46.97 ns   (the resting path)
```

## Consumer examples

```text
$ cargo run -p aeolus-compile --example validation
(the fire, the complement, the gated pair, the CSS head — headless)
$ cargo run -p aeolus-cli --example expand_validation
(the fact-bearing form, the resting form, the gate-only form — headless)
```
