# 34 — The `starting:` variant

`@starting-style`, the transition's first frame (the tailwindcss
v4.1.17 `variants.ts` `staticVariant('starting', ['@starting-style'])`
registration — the media-feature tier's at-rule sibling whose `Env`
reading the pseudo-element landing deliberately deferred: "a different
shape than the frame-state tier"). The landing resolves the shape: the
at-rule WRAPS the rule (the `@media` form, the tier's specificity 4)
and the fact IS frame state the host answers — ONE affirmative
`Env::starting_style` bit (the element's first-render gate beside the
transition), an unset bit the QUIET miss a frame the element is not in
is (the media facts' own reading, no probe seam and no sourceless
note), the CSS arm renders the bare `@starting-style` head in both
walls, the plan bridge reports the v0.x boundary through the typed
`PlanNoteKind::StartingStyle` note, the metamorphic checker's
conjunctive env bakes the fact so the compile == CSS == interp triangle
agrees, the GPUI/floem walks share the interp verdicts verbatim, the
four plan-lane backends carry the env literal, the LSP labels the
family, and the CLI's `expand --starting` flag asserts the fact (one
flag per FACT, the `--portrait` precedent).

* **M2** (the whole landing): the `starting:` variant end to end —
  IR arm, fold spelling, env fact, interp verdict, CSS at-rule head,
  plan note, metamorphic arm, gpui/floem/egui/iced/slint/xilem wiring,
  LSP label, CLI flag, bench, examples, docs.

## The differential evidence

```
staticVariant('starting', ['@starting-style']),
```

The at-rule is the registration's own text — no selector suffix
renders, the media tier's render shape. The `Env` fact is the host's
gate: a frame either IS the element's starting-style frame or is not,
so one affirmative bit answers and the resting env stays the quiet
miss (the `motion-safe:` complement reading has no half here — the
spelling set stays closed at one).

## RED → GREEN

* RED: `crates/aeolus-compile/tests/starting.rs` (8 pins, written
  first) + `crates/aeolus-cli/tests/cli.rs`'s starting battery (2) +
  the GPUI/floem walk pins (3 + 3) — compile failures against the
  un-landed `Cond::Starting` arm and the `Env` field (`E0599`/`E0609`).
* GREEN: the same pins pass against the landed arm; the metamorphic
  battery and the workspace suite re-measure at the gates.

## Bench

`cargo bench -p aeolus-compile --bench starting` (divan, release):
`interp_starting_fire` **127.7 ns** median (the fact rides), 
`interp_starting_resting` **80.7 ns** (the resting quiet-miss path) —
the env-field read the whole state tier shares (the form-state
family's own cost shape).

## Consumer examples

`examples/starting.rs` (the compile crate's four-way demo: the fold,
the resting quiet miss, the fired frame, the CSS head, the plan note)
and `examples/expand_starting.rs` (the CLI face beside the resting
walk) — `cargo run -p aeolus-compile --example starting`,
`cargo run -p aeolus-cli --example expand_starting`.
