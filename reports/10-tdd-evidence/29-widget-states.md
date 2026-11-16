# 29 — The widget-state family

Slice: the widget-state variants' answer — `open:`/`inert:`/`default:`,
the widget-own tier beside the element-state family. The family closes
the v0.x-surface boundary evidence 28 documented — the gap closing in
the OTHER direction: the v0.x `State` surface had no slot for the
three spellings at the element-state landing, so the family GROWS the
core's state/forms blocks (`State::Open`/`State::Inert`/`State::Default`
— the parser, the resolver's state walls, the selector wall, the
snapshot's strict reader) AND the V1 `Cond` surface grows the three
conditions (`Cond::Open`/`Cond::Inert`/`Cond::Default`). Evidence file
per the workspace convention: the RED spec written first, the GREEN
run that satisfies it, the bench and the consumer example. Two
red→green cycles, one per milestone:

* **M1** — the family at the pipeline level: the three bare states
  join the `Cond` surface (the `hover:` shape, the probe tier,
  specificity 2, NOT an at-rule), every family question is AFFIRMATIVE
  (the `target:` shape: a non-open element matches neither `:open`'s
  arms, a non-form element misses `:default`, a non-inert subtree
  misses `:is([inert], [inert] *)` — no complement rides, the resting
  env answers each with the QUIET miss a selector that does not match
  is, never an exclusion note), the fold arms (the CLOSED state/forms
  block set extends), the `state_pseudo` table the `has-` inner
  grammar reads (one grammar, three consumers), the Env facts (ONE
  fact per question — `opened`, `inert`, `form_default`; `inert:`'s
  compound head shares the ONE per-element fact the host computes for
  every element inside (or carrying) an `[inert]` root), the CSS arm's
  upstream selector texts in BOTH walls (the v4.1.17 source's own
  `staticVariant` texts — the compound `:is(...)` heads verbatim, no
  at-rule head; BOTH the built-in segment wall and the alias path's
  `apply_cond_shape` render them), the v0.x core's own lane (the
  parser's spelling table, the resolver's `state_suffix`/`state_name`
  walls, the selector wall's texts, the snapshot's round trip), the
  metamorphic checker's conjunctive env (the one-fact states bake
  their bit true), the plan bridge LANDS the state slots (the family's
  own v0.x forms — no note rides), the GPUI and floem walks (the same
  env semantics verbatim), the four plan-lane backends' probe paths
  honestly keep the facts off, the LSP labels the family, and the
  completion surface carries the three spellings (`KNOWN_VARIANTS`
  beside the two-sided audit).
* **M2** — the CLI face: `expand --open`/`--inert`/`--default`, the
  widget-state facts' headless flags (the element-state flags'
  sibling). One flag per FACT, the `--target` precedent: `--open` the
  disclosure/popover fact, `--inert` the frozen-subtree fact,
  `--default` the form group's default-control fact. Unset is the
  quiet miss a selector that does not match is.

## The differential evidence

The tailwindcss v4.1.17 source
(`packages/tailwindcss/src/variants.ts`), the state/forms blocks'
`staticVariant` calls — the three spellings are the family's whole
set (the compound `:is(...)` heads are the upstream selectors' own
shape, pinned verbatim; every question is AFFIRMATIVE):

```text
staticVariant('open', ['&:is([open], :popover-open, :open)'])
staticVariant('inert', ['&:is([inert], [inert] *)'])
staticVariant('default', ['&:default'])
```

## RED (spec written first)

```text
$ cargo test -p aeolus-compile --test widget_states
error[E0599]: no variant, associated function, or constant named `Open`
              found for enum `aeolus_ir::Cond`
error[E0599]: no variant, associated function, or constant named `Inert`
              found for enum `aeolus_ir::Cond`
error[E0599]: no variant, associated function, or constant named `Default`
              found for enum `aeolus_ir::Cond`
error[E0599]: no variant, associated function, or constant named `Open`
              found for enum `aeolus_core::State`
(exit 101 — 18 compile errors: the fold refuses every widget-state
spelling and the v0.x surface has no slot for the family; the missing
Env facts are the E0560s beside)

$ cargo test -p aeolus-cli --test cli expand_
test result: FAILED. 41 passed; 3 failed
(expand_open_flag_answers_the_layers,
expand_inert_flag_answers_the_layers,
expand_default_flag_answers_the_layers — the flags are not in the
fold's allow-list yet)
```

## GREEN (the battery satisfies the spec)

```text
$ cargo test -p aeolus-compile --test widget_states
test result: ok. 13 passed; 0 failed

$ cargo test -p aeolus-cli --test cli expand_
test result: ok. 46 passed; 0 failed
```

## Bench (divan, release, 2-core sandbox, 2026-11-16)

```text
$ cargo bench -p aeolus-compile --bench widget_states
interp_widget_state_button        median 156.9 ns   (the fire path)
interp_widget_state_default_env   median 26.06 ns   (the resting path)
```

## Consumer examples

```text
$ cargo run -p aeolus-compile --example widget_states
(the fire, the resting quiet miss, the CSS head — headless)
$ cargo run -p aeolus-cli --example expand_widget_states
(the fact-bearing form, the resting form, the mix form — headless)
```
