# 24 — The preference families

Slice: the media-feature tier's named spellings — `motion-safe:`,
`motion-reduce:`, `contrast-more:`, `contrast-less:`, `forced-colors:`,
`print:` — the at-rule tier beside the media forms. Evidence file per
the workspace convention: the RED spec written first, the GREEN run
that satisfies it, the bench and the consumer example. One red→green
cycle:

* the typed bit: `Cond::Media(Pref)` (plan §3.5's "typed state bits,
  not string probes" row; `Pref::source` carries the canonical v4
  spelling), the fold arms (the CLOSED static-variant set), the Env
  facts (`reduced_motion`/`more_contrast`/`less_contrast`/
  `forced_colors`/`print_media` — frame state, the tier `dark:` rides),
  `CssAt::MediaPref` (the v4.1.17 source's own `staticVariant` texts,
  pinned verbatim), the metamorphic checker's conjunctive env (the
  three-walk agreement applies — no probe seam, no sourceless note),
  the GPUI/Floem runtime walks (the same env semantics), the LSP hover
  label, the plan bridge's `PlanNoteKind::MediaFeature` note, and the
  CLI face (`expand`'s five preference flags — the `--dark`
  precedent).

## The differential evidence

The tailwindcss v4.1.17 source
(`packages/tailwindcss/src/variants.ts`), the `staticVariant` calls:

```text
staticVariant('motion-safe',   ['@media (prefers-reduced-motion: no-preference)'])
staticVariant('motion-reduce', ['@media (prefers-reduced-motion: reduce)'])
staticVariant('contrast-more', ['@media (prefers-contrast: more)'])
staticVariant('contrast-less', ['@media (prefers-contrast: less)'])
staticVariant('print',         ['@media print'])
staticVariant('forced-colors', ['@media (forced-colors: active)'])
```

## RED (spec written first)

```text
$ cargo test -p aeolus-compile --test preferences
error: could not compile `aeolus-compile` (test "preferences") due to 15 previous errors
error[E0432]: unresolved import `aeolus_ir::Pref`
error[E0599]: no variant named `Media` found for enum `Cond`
error[E0609]: no field `reduced_motion` on type `&mut Env`
error[E0599]: no variant named `MediaFeature` found for enum `PlanNoteKind`
```

## GREEN

```text
$ cargo test -p aeolus-compile --test preferences
test result: ok. 10 passed; 0 failed

$ cargo test --workspace --all-features
TOTAL passed: 1671 failed: 0

$ cargo test -p aeolus-cli --test cli
test result: ok. 86 passed; 0 failed (the three preference pins beside)
```

## Bench (divan, release, shared sandbox — medians of 100 samples)

```text
$ cargo bench -p aeolus-compile --bench preferences
interp_preferences_button        median 109.1 ns  (the fire path)
interp_preferences_default_env   median 56.8 ns   (the quiet-miss path)
```

## Example

`cargo run -p aeolus-compile --example preferences` — the fire under
the reduced-motion + print facts, the complement answer under a
no-preference user agent.

## The boundaries kept

* NO new probe seam: the preference facts are frame state — the `Env`
  answers, an unset bit is the QUIET miss a media query that does not
  hold is, never an exclusion note (the `dark:` tier's own semantics);
* `motion-safe:` is the COMPLEMENT of `motion-reduce:` (the feature's
  two v4 values share one fact — `prefers-reduced-motion` has no third
  value in v4);
* the plan bridge keeps the honest note — the v0.x `Atom::Viewport`
  form carries width bounds only; the style does not land;
* the probe paths (`env_for_probe` in the backends) honestly keep the
  facts off — the user agent's answers ride no element probe surface.
