# 30 — The form-state family

Slice: the form-state variants' answer — `enabled:`/`required:`/
`optional:`/`in-range:`/`out-of-range:`, the matches-neither tier
beside the widget-state family. The family closes the boundary
evidence 27 documented — a non-form element matches NEITHER half of
each pair, so ONE `Env` bit cannot answer them honestly — with the
TWO-FACT shape: a MEMBERSHIP fact per universe (`form_associated`,
`range_bounded`) beside the pair's shared state bit (`required_field`,
`range_in`), the v0.x `State` surface's own five slots (landed with
the surface itself) bridging through the V1 `Cond` surface's five new
conditions (`Cond::Enabled`/`Cond::Required`/`Cond::Optional`/
`Cond::InRange`/`Cond::OutOfRange`). Evidence file per the workspace
convention: the RED spec written first, the GREEN run that satisfies
it, the bench and the consumer example. Two red→green cycles, one per
milestone:

* **M1** — the family at the pipeline level: the five bare states
  join the `Cond` surface (the `hover:` shape, the probe tier,
  specificity 2, NOT an at-rule), the fold arms (the CLOSED
  state/forms block set extends), the `state_pseudo` table the
  `has-` inner grammar reads (one grammar, three consumers —
  `has-enabled:` → `*:enabled`), the Env facts (FOUR —
  `form_associated` the form-membership fact the HTML spec's
  form-associated list names, the gate EVERY form pair reads;
  `required_field` the `required` attribute the pair splits on —
  `required:` on the bit, `optional:` its complement WITHIN the
  universe, the `validation_invalid` precedent, the attribute bit
  alone never fires; `range_bounded` the bounded-universe membership
  the host keeps off while no position answer exists, the
  pristine-empty reading; `range_in` the value's position —
  `out-of-range:` its complement within the universe), the CSS arm's
  upstream pseudo-classes in BOTH walls (the v4.1.17 source's own
  `staticVariant` texts — `:enabled` through `:out-of-range`, no
  at-rule head), the metamorphic checker's conjunctive env (the
  membership bakes true beside the bit, or beside its complement
  false), the plan bridge LANDS the state slots (the v0.x surface
  already carries the five forms — no note rides), the GPUI and floem
  walks (the same env semantics verbatim), the four plan-lane
  backends' probe paths honestly keep the facts off, the LSP labels
  the family, and the completion surface carries the five spellings
  (`KNOWN_VARIANTS` beside the two-sided audit).
* **M2** — the CLI face: `expand --form-associated`/`--required`/
  `--range-bounded`/`--range-in`, the form-state facts' headless
  flags (the widget-state flags' sibling). One flag per FACT, the
  `--target` precedent: `--form-associated` the form-membership fact,
  `--required` the required-attribute fact, `--range-bounded` the
  bounded-universe membership, `--range-in` the value's position.
  `optional:` answers within the membership, `out-of-range:` within
  the bounds — no complement flag rides. Unset is the quiet miss a
  selector that does not match is.

## The differential evidence

The tailwindcss v4.1.17 source
(`packages/tailwindcss/src/variants.ts`), the state/forms blocks'
`staticVariant` calls — the five spellings are the family's whole
set (the plain pseudo-class heads are the upstream selectors' own
shape, pinned verbatim; the matches-neither reading rides the host's
membership facts, the selectors themselves carry no attribute hooks
to distinguish):

```text
staticVariant('enabled', ['&:enabled'])
staticVariant('required', ['&:required'])
staticVariant('optional', ['&:optional'])
staticVariant('in-range', ['&:in-range'])
staticVariant('out-of-range', ['&:out-of-range'])
```

## RED (spec written first)

```text
$ cargo test -p aeolus-compile --test form_states
error[E0599]: no variant or associated item named `Enabled` found
              for enum `aeolus_ir::Cond`
error[E0599]: no variant or associated item named `Required` found
              for enum `aeolus_ir::Cond`
error[E0599]: no variant or associated item named `Optional` found
              for enum `aeolus_ir::Cond`
error[E0599]: no variant or associated item named `InRange` found
              for enum `aeolus_ir::Cond`
error[E0599]: no variant or associated item named `OutOfRange` found
              for enum `aeolus_ir::Cond`
error[E0560]: struct `Env` has no field named `form_associated`
error[E0560]: struct `Env` has no field named `required_field`
error[E0560]: struct `Env` has no field named `range_bounded`
error[E0560]: struct `Env` has no field named `range_in`
(exit 101 — 35 compile errors: the fold refuses every form-state
spelling as unsupported-variant — the boundary evidence 27 kept — and
the Env carries none of the membership/state facts the two-fact shape
needs)

$ cargo test -p aeolus-cli --test cli form_state
test result: FAILED. 1 passed; 1 failed
(expand_form_state_flags_mix_with_the_facts — the flags are not in
the fold's allow-list yet, the walk answers exit 2)
```

## GREEN (the battery satisfies the spec)

```text
$ cargo test -p aeolus-compile --test form_states
test result: ok. 15 passed; 0 failed

$ cargo test -p aeolus-cli --test cli
test result: ok. 113 passed; 0 failed
```

## Bench (divan, release, 2-core sandbox, 2026-11-16)

```text
$ cargo bench -p aeolus-compile --bench form_states
interp_form_state_button         median 160.4 ns   (the fire path)
interp_form_state_default_env    median 26.91 ns   (the resting path)
```

## Consumer examples

```text
$ cargo run -p aeolus-compile --example form_states
(the fire, the resting quiet miss, the bounded complement, the CSS
head — headless)
$ cargo run -p aeolus-cli --example expand_form_states
(the fact-bearing form, the resting form, the bounded-complement form
— headless)
```
