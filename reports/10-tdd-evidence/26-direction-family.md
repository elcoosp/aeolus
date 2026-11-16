# 26 — The direction family

Slice: the text-direction variants' answer — `ltr:`/`rtl:`, the
selector-side static-variant family beside the media-feature tier (the
v0.x surface has no direction slot; the family was unnamed in the
diagnosed rows — the `not-*` compound and the `*:` arbitrary-variant
half stay their own boundaries). Evidence file per the workspace
convention: the RED spec written first, the GREEN run that satisfies
it, the bench and the consumer example. Two red→green cycles, one per
milestone:

* **M1** — the family at the pipeline level: the typed
  `Direction` bit joins the `Cond` surface as `Cond::Dir` (one
  condition, the payload the bit — the selector-side tier the probe
  states ride, specificity 2, NOT an at-rule), the fold arms (the
  CLOSED static-variant set extends), the Env fact
  (`direction_rtl` — frame state, the tier the preference and
  capability bits ride; `ltr:` is the COMPLEMENT, the
  `portrait:`/`landscape:` precedent — CSS's own initial `direction`
  value is ltr, the resting env answers the `ltr:` query the way it
  answers `landscape:`), the CSS arm's upstream selector heads (the
  v4.1.17 source's own `staticVariant` texts, pinned verbatim —
  `:where(:dir(rtl), [dir="rtl"], [dir="rtl"] *)`, no at-rule head),
  the metamorphic checker's conjunctive env (the complement bakes the
  bit false), the GPUI runtime walk (the same env semantics verbatim),
  the plan-lane backends ride the interp verdicts, the codegen's
  frozen tokens name the typed bit, the LSP labels the family, and the
  plan bridge keeps the new `Direction` note's honest boundary (the
  v0.x surface has no direction slot).
* **M2** — the CLI face: `expand --rtl`, the direction fact's headless
  flag (the capability flags' sibling — frame state needs no facts
  row, the `--dark` precedent; `ltr:` is the resting env's complement
  answer, it takes no flag).

## The differential evidence

The tailwindcss v4.1.17 source
(`packages/tailwindcss/src/variants.ts`), the `staticVariant` calls —
the two spellings are the family's whole set (v4.1.17 renders them as
SELECTOR conjunctions, not at-rules):

```text
staticVariant('ltr', ['&:where(:dir(ltr), [dir="ltr"], [dir="ltr"] *)'])
staticVariant('rtl', ['&:where(:dir(rtl), [dir="rtl"], [dir="rtl"] *)'])
```

## RED (spec written first)

```text
$ cargo test -p aeolus-compile --test direction
error[E0432]: unresolved import `aeolus_ir::Direction`
error[E0599]: no variant or associated item named `Dir` found for enum `Cond`
error[E0560]: struct `Env` has no field named `direction_rtl`
error[E0599]: no variant named `Direction` found for enum `PlanNoteKind`

$ cargo test -p aeolus-cli --test cli expand_rtl
test expand_rtl_flag_answers_the_layers ... FAILED
(unknown flag "--rtl" for this command — exit 2)
```

## GREEN

```text
$ cargo test -p aeolus-compile --test direction
test result: ok. 10 passed; 0 failed

$ cargo test -p aeolus-cli --test cli
test result: ok. 93 passed; 0 failed (the three direction pins beside)
```

## Bench (divan, release, shared sandbox — medians of 100 samples)

```text
$ cargo bench -p aeolus-compile --bench direction
interp_direction_button        median 130.5 ns  (the fire path)
interp_direction_default_env   median 46.6 ns   (the resting-env complement beside the quiet-miss path)
```

## Examples

`cargo run -p aeolus-compile --example direction` — the fire under the
rtl fact, the resting env's complement answer (`ltr:` fires), the
quiet miss, and the CSS head the arm renders. `cargo run -p aeolus-cli
--example expand_direction` — the CLI face through the same `run`
entry point the binary drives (the fact-bearing form beside the
resting form).

## The boundaries kept

* NO new probe seam: the direction fact is frame state — the `Env`
  answers, an unset bit is the QUIET miss a selector that does not
  match is, never an exclusion note (the capability tier's own
  semantics, the preference families', the `dark:` tier's);
* ONE fact for the family: `rtl:` on the bit, `ltr:` its complement
  (the feature's two v4 values share one fact — the orientation query
  always holding an answer, the `motion-safe:` precedent; CSS's own
  initial `direction` value is ltr, so the resting env honestly
  answers the `ltr:` query);
* the head is SELECTOR-side: `:where(:dir(rtl), …)` rides the
  element's own selector (the probe tier, specificity 2) — no
  `@media` head renders, the family is not an at-rule (the
  differential evidence's own shape);
* the plan bridge keeps the honest note — the v0.x `Atom` set has no
  text-direction slot; the style does not land (the `Direction` note
  the media-feature and supports notes keep) with the backends' report
  translates beside;
* the fold stays CLOSED: the two spellings join `KNOWN_VARIANTS` and
  the completion audit's expected list (every entry folds, every fold
  is listed — the two-sided drift catch).
