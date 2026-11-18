# 38 — The `--width` viewport fact on `expand`

The breakpoint families' CLI face. The interp's `Env.viewport_rem` slot
has answered the `sm:`…`2xl:` min forms (AT and above their
breakpoint's rem) and the `max-*:` complements (strictly below) since
the P4 env landing, and every OTHER frame fact the `Env` carries — the
preference media bits, the capability tier, the direction bit — already
had a `--flag` on the CLI's `expand` face asserting it. The width was
the one fact a CLI user could not assert: `expand "md:p-8"` could
never answer the layer, because no flag carried the viewport. The
landing is the wire-up the interaction-state handoff's shape: one
VALUE flag, `--width <rem>`, parsed through the `--container` arm's
honest f32 window (finite, ≥ 0 — a malformed, negative or non-finite
spelling is a usage error, exit 2, never a half-answered layer), fed
into the same `Env` the other fact flags build, and the interp's own
AT/strictly-below boundary rules answer from there (no new interp arm,
no dispatch axis — the flags are the facts).

* **M1** (the whole landing): the `--width rem` flag end to end —
  `Args` field, the parse arm, the allow-list entry, the `Env`
  construction, the `USAGE` rows, the red-first battery, the bench,
  the consumer example, the README rows.

## The semantics evidence

The v4.1.17 default theme's own breakpoint rems (the IR's `Bp::rem`):
sm 40, md 48, lg 64, xl 80, 2xl 96. The boundary rules are the interp's
pinned ones: `Cond::Bp`/`Cond::Min` fire at `viewport_rem >= bp.rem()`,
`Cond::Max` fires strictly below. The resting env stays the zero-width
viewport (no min form fires — the quiet miss a below-breakpoint
viewport is), and the cascade composition is the sort order's own
slots: at the md width the responsive tier ranks AFTER the state tier,
so `md:p-8` beats `hover:p-12` for the padding longhand while below it
the hover fact wins — the width fact changes the winner, the battery
pins both readings.

## RED → GREEN

* RED: `crates/aeolus-cli/tests/cli.rs`'s breakpoint battery (7 pins,
  written first) — `expand --width 40 sm:p-8` et al. riding the
  unknown-flag exit 2 (the flag did not exist), the two honest-refusal
  pins green before the impl (the allow-list closed set already
  refused the spelling).
* GREEN: the same pins pass against the landed flag; the full
  aeolus-cli suite re-measures at the gates (136 pins in the cli.rs
  battery beside the form/element/widget/composition suites).

## Bench

`cargo bench -p aeolus-cli --bench expand -- expand_viewport_facts`
(divan, release): **140.4 ns** median — the base/state/breakpoint mix
(`p-4 md:p-8 hover:p-12 xl:p-16 max-md:p-2`) walked at the asserted
width, the min forms firing, the max form on its strictly-below edge,
the unasserted tier waiting: the env-field read the whole fact tier
shares (the interaction-state bench's own cost shape, 139.6 ns
fastest).

## Consumer examples

`crates/aeolus-cli/examples/expand_viewport.rs` (the headless
read-outs: the resting zero-width env, `--width 48` at the md
boundary, `--width 39.9` strictly below it, the width-and-hover
composition where the breakpoint tier out-ranks the state tier):

```sh
cargo run -p aeolus-cli --example expand_viewport
```

## Docs

README's breakpoint capability row names the CLI face beside the
`Env` slot, the examples index carries the demo row, the USAGE text
names the flag, and the interp's `Env` doc comment drops its own
drift (the "no breakpoint matches" wording — the max forms DO read
the complement at the resting env, the example's first read-out
demonstrates it).
