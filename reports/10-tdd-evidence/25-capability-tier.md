# 25 — The media-feature capability tier

Slice: the media-feature tier's REMAINING v4.1.17 spellings beside the
preference families — the input-capability forms `pointer-fine:`,
`pointer-coarse:`, `pointer-none:`, `any-pointer-fine:`,
`any-pointer-coarse:`, `any-pointer-none:` and the orientation forms
`portrait:`/`landscape:`. Evidence file per the workspace convention:
the RED spec written first, the GREEN run that satisfies it, the bench
and the consumer example. Two red→green cycles, one per milestone:

* **M1** — the tier at the pipeline level: the typed bits join the
  existing `Pref` payload of `Cond::Media` (no new condition family,
  no new probe seam — the same at-rule tier the preferences ride),
  the fold arms (the CLOSED static-variant set extends), the Env facts
  (`pointer_fine`/`pointer_coarse`/`pointer_none`/`any_pointer_fine`/
  `any_pointer_coarse`/`any_pointer_none`/`portrait` — frame state,
  the tier `dark:` and the preference bits ride), `CssAt::MediaPref`
  (the v4.1.17 source's own `staticVariant` texts, pinned verbatim),
  the metamorphic checker's conjunctive env (the complement bakes the
  bit false), the GPUI/Floem runtime walks (the same env semantics
  verbatim), the four plan-lane backends' probe paths (the facts
  honestly off), and the plan bridge's existing `MediaFeature` note.
* **M2** — the CLI face: `expand`'s seven capability flags
  (`--pointer-fine` … `--any-pointer-none`, `--portrait` — the
  preference flags' sibling, the `--dark` precedent; `landscape:` is
  the resting env's complement answer, it takes no flag).

## The differential evidence

The tailwindcss v4.1.17 source
(`packages/tailwindcss/src/variants.ts`), the `staticVariant` calls —
the eight spellings are the tier's whole remainder (v4.1.17 carries NO
`hover-hover`/`hover-none` spelling: the plain `hover:` variant already
wraps `@media (hover: hover)`):

```text
staticVariant('pointer-none',        ['@media (pointer: none)'])
staticVariant('pointer-coarse',      ['@media (pointer: coarse)'])
staticVariant('pointer-fine',        ['@media (pointer: fine)'])
staticVariant('any-pointer-none',    ['@media (any-pointer: none)'])
staticVariant('any-pointer-coarse',  ['@media (any-pointer: coarse)'])
staticVariant('any-pointer-fine',    ['@media (any-pointer: fine)'])
staticVariant('portrait',            ['@media (orientation: portrait)'])
staticVariant('landscape',           ['@media (orientation: landscape)'])
```

## RED (spec written first)

```text
$ cargo test -p aeolus-compile --test capabilities
error[E0599]: no variant, associated function, or constant named `PointerFine` found for enum `Pref`
error[E0599]: no variant, associated function, or constant named `PointerCoarse` found for enum `Pref`
error[E0599]: no variant, associated function, or constant named `PointerNone` found for enum `Pref`
error[E0599]: no variant, associated function, or constant named `AnyPointerFine` found for enum `Pref`
...

$ cargo test -p aeolus-cli --test cli expand_capability
test expand_capability_flags_answer_the_layers ... FAILED
(unknown flag "--pointer-fine" for this command — exit 2)
```

## GREEN

```text
$ cargo test -p aeolus-compile --test capabilities
test result: ok. 12 passed; 0 failed

$ cargo test -p aeolus-cli --test cli
test result: ok. 90 passed; 0 failed (the four capability pins beside)
```

## Bench (divan, release, shared sandbox — medians of 100 samples)

```text
$ cargo bench -p aeolus-compile --bench capabilities
interp_capabilities_button        median 111.1 ns  (the fire path)
interp_capabilities_default_env   median 54.1 ns   (the quiet-miss path beside the complement)
```

## Examples

`cargo run -p aeolus-compile --example capabilities` — the fire under
the pointer-fine + portrait facts, the resting env's complement answer
(`landscape:flex` fires). `cargo run -p aeolus-cli --example
expand_capabilities` — the CLI face through the same `run` entry point
the binary drives (the fact-bearing form beside the resting form).

## The boundaries kept

* NO new probe seam and NO new condition family: the capability facts
  are frame state — the `Env` answers, an unset bit is the QUIET miss
  a media query that does not hold is, never an exclusion note (the
  preference families' own semantics, the `dark:` tier's);
* the six pointer bits assert INDEPENDENTLY (a hit on one never
  implies another — the host asserts what it knows, the way
  `more_contrast` and `forced_colors` already coexist);
* `landscape:` is the COMPLEMENT of `portrait:` (the feature's two v4
  values share one fact — the orientation query always holding an
  answer the way reduced-motion does, the `motion-safe:` precedent);
* the plan bridge keeps the honest note — the v0.x `Atom::Viewport`
  form carries width bounds only; the style does not land (the
  `MediaFeature` note the preferences keep);
* the plan-lane probe paths (`env_for_state` in the four backends)
  honestly keep the facts off — the user agent's answers ride no
  element probe surface.
