# 32 — The media-feature remainder

Slice: the preference/capability tier's last two v4.1.17 static
variants — `inverted-colors:` (`@media (inverted-colors: inverted)`,
the v4.1 addition) and `noscript:` (`@media (scripting: none)`) — the
[`Pref`] set's final remainder. The family rides the media-feature
tier's own shape: the typed bits, the `Env` frame facts, the at-rule
texts, the typed plan boundary, the CLI probe flags. Evidence file per
the workspace convention: the RED spec written first, the GREEN run
that satisfies it, the bench and the consumer examples. One
red→green cycle:

* **M1** (the whole landing): the two bits join the closed [`Pref`]
  set (`Pref::InvertedColors`/`Pref::NoScript`, `source()` naming the
  canonical spelling), the fold arms (the closed spelling set
  extends), the `Env` carries TWO facts — `inverted_colors` the user
  agent's inverted-colours answer (the accessibility answer the
  `forced_colors` bit rides beside), `scripting_none` the scripting
  feature's `none` value (`enable`/`initial-only` have no v4.1.17
  registration — the spelling set stays closed) — the interp answers
  with no probe seam and no sourceless note (an unset frame fact is
  the QUIET miss a media query that does not hold is), the CSS arm
  renders the upstream at-rule texts in BOTH walls (the built-in
  segment wall and the alias path's `apply_cond_shape` — the
  `MediaPref` table grows the two texts), the metamorphic checker's
  conjunctive env carries the facts, the GPUI and floem walks share
  the interp verdicts verbatim, the plan bridge keeps the typed
  `PlanNoteKind::MediaFeature` boundary (no new note kind — the v0.x
  Atom set carries width bounds only, the media-feature boundary's
  own shape), and the CLI's `expand --inverted-colors`/`--noscript`
  flags assert the facts (one flag per FACT, the `--portrait`
  precedent).

## The differential evidence

The tailwindcss v4.1.17 source
(`packages/tailwindcss/src/variants.ts`), the media-features block —
the two spellings are the media-feature tier's whole remainder (the
preference, capability and orientation forms landed 2026-11):

```text
staticVariant('inverted-colors', ['@media (inverted-colors: inverted)'])
staticVariant('noscript',        ['@media (scripting: none)'])
```

Upstream's remainder this landing deliberately does NOT carry:
`starting:` (`@starting-style` — the transition's first frame, an
at-rule whose `Env` reading is not frame state) and the of-type
structural trio (`first-of-type:`/`last-of-type:`/`only-of-type:` —
the `Structural` enum's next slice). The media-feature tier is now
COMPLETE: every `@media`-shaped static variant v4.1.17 registers is a
typed bit.

## RED → GREEN

* RED: `crates/aeolus-compile/tests/media_remainder.rs` (6 pins,
  written first) — the fold refuses both spellings
  (`E0277`/`E0599` compile failures against the un-landed bits).
* GREEN: the same 6 pins + `tests/cli.rs`'s remainder battery (2)
  pass against the landed bits; the workspace battery re-measures at
  the gates.

## Bench

`cargo bench -p aeolus-compile --bench preferences` (divan,
2026-11-16, shared sandbox; medians in
reports/08-benchmarks/BENCHMARKS.md) — the remainder rides the same
env-field read the family's walk carries (~80 ns median over the
two-layer program).

## Consumer examples

* `cargo run -p aeolus-compile --example preferences` — the remainder
  walks beside the family's fire/resting/complement forms.
* `cargo run -p aeolus-cli --example expand_preferences` — the
  `--inverted-colors` flag's answer beside the family flags.
* `aeolus expand --noscript "noscript:mt-2"` — the CLI face itself.
