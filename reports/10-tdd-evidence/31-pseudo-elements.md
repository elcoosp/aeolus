# 31 — The pseudo-element render-target tier

Slice: the pseudo-element spellings' answer — `before:`/`after:`/
`marker:`/`first-letter:`/`first-line:`/`file:`/`backdrop:`/
`details-content:`, the tier beside the `selection:`/`placeholder:`
pair the interp arm's [`PseudoTarget`] seam carried since the
render-target row landed. The seam grows the render-target ENUM, the
typed `Cond` surface, the fold's closed spelling set, the walks, the
CSS arm's heads, the plan bridge's boundary, the LSP labels, the CLI's
`--pseudo` consumer face and the closed-set surface (`ALL`/`from_name`/
`spelling`). Evidence file per the workspace convention: the RED spec
written first, the GREEN run that satisfies it, the bench and the
consumer examples. One red→green cycle:

* **M1** — the tier end to end: the eight bare conditions join the
  `Cond` surface (the `selection:` shape, the probe tier, specificity
  2, NOT an at-rule — a pseudo-element head is a selector conjunct on
  the `&`), the fold arms (the CLOSED pseudo-elements + state blocks
  set extends), the element walk's exclusion (the
  [`InterpNote::PseudoElement`] note covers the whole tier — the
  render-target contract), the pseudo cascades resolve per target
  through the same `walk_pseudo` (the condition-equality walk; the
  enum's `cond()` grew ten arms), the CSS arm renders the upstream
  selector texts (`::first-letter` through `::details-content`;
  `marker:` the FOUR-selector disjunction the upstream registration
  carries — one rule, the alternatives comma-joined;
  `before:`/`after:` the functional bodies' injected
  `content: var(--tw-content)` line riding FIRST, the per-rule `""`
  fallback standing in for the at-root `@property --tw-content`
  registration the per-candidate emitter cannot carry — the documented
  divergence, the initial value upstream registers), the plan bridge
  LANDS `file:` (the v0.x `PseudoElement::FileSelectorButton` the
  surface already carries — the selection/placeholder arm's own shape)
  and reports the other seven through the typed
  `PlanNoteKind::PseudoElement` boundary (the media-feature boundary's
  sibling), the metamorphic checker's note set + conjunctive env + the
  condition labels grow the tier, the GPUI and floem walks keep every
  pseudo layer out of the element cascade (`false` / the exclusion
  report), the four plan-lane backends' boundary channels render the
  new note, the LSP hover labels the tier, and the CLI's `expand
  --pseudo <target>` face answers the named render target's own
  cascade beside the element answer (the closed set's parse —
  unknown spellings are usage errors).

## The differential evidence

The tailwindcss v4.1.17 source
(`packages/tailwindcss/src/variants.ts`), the pseudo-elements + state
blocks — the eight spellings are the tier's whole remainder (the pair
`selection`/`placeholder` landed with the interp arm):

```text
staticVariant('first-letter', ['&::first-letter'])
staticVariant('first-line',   ['&::first-line'])
staticVariant('marker',       ['& *::marker', '&::marker',
                               '& *::-webkit-details-marker',
                               '&::-webkit-details-marker'])
staticVariant('selection',    ['& *::selection', '&::selection'])   // landed
staticVariant('file',         ['&::file-selector-button'])
staticVariant('placeholder',  ['&::placeholder'])                   // landed
staticVariant('backdrop',     ['&::backdrop'])
staticVariant('details-content', ['&::details-content'])
// before/after: the FUNCTIONAL bodies —
//   styleRule('&::before', [contentProperties(),            // at-root @property
//                           decl('content', 'var(--tw-content)'),
//                           ...v.nodes])                    // the utility's own
//   (the same for '&::after')
```

The v0.x oracle's own surface names the file-selector part
(`PseudoElement::FileSelectorButton`, the `file:` spelling) — the plan
bridge's landing arm reads the same surface. Upstream's `link` is NOT
a v4.1.17 variant (checked, absent from `variants.ts`) — the spelling
stays out, evidence 27's own boundary discipline. The remaining
upstream remainder the tier deliberately does NOT carry:
`first-of-type:`/`last-of-type:`/`only-of-type:` (the structural
family's of-type forms — the `Structural` enum's next slice),
`starting:` (`@starting-style`, the at-rule whose `Env` reading is
the transition's first frame — a different shape than the frame-state
tier) and the `nth-of-type-*` functional grammar (the fold's `nth-`
walk is the child-index grammar).

## RED → GREEN

* RED: `crates/aeolus-compile/tests/pseudo_elements.rs` (15 pins,
  written first) — the fold refuses every spelling
  (`E0599: no variant … named Before found for enum Cond` is the
  capture: the suite does not COMPILE against the un-landed tier),
  the CLI battery's `--pseudo` face is an unknown-flag usage error,
  the GPUI `tw!` macro refuses the spellings at expansion.
* GREEN: the same 15 pins + the CLI battery (4) + the GPUI pins (3) +
  the floem pins (3) pass against the landed tier;
  `cargo test --workspace --all-features` = 1796 passed / 0 failed
  (the pre-handoff honest sum 1771 + the tier's 25).

## Bench

`cargo bench -p aeolus-compile --bench pseudo_targets` (divan,
2026-11-16, shared sandbox; medians in
reports/08-benchmarks/BENCHMARKS.md) — the walk is target-generic, the
tier rides the same per-pass cost the pair carried (~54–65 ns median
over the nine-layer mixed program).

## Consumer examples

* `cargo run -p aeolus-compile --example pseudo_targets` — the tier's
  walks (the element cascade's exclusions, the closed set one cascade
  each, the themed form).
* `cargo run -p aeolus-cli --example expand_pseudo` — the `--pseudo`
  face's answer, the whole set.
* `aeolus expand --pseudo before "before:p-8 p-4"` — the CLI face
  itself (the element answer, the notes, the target's own cascade).
