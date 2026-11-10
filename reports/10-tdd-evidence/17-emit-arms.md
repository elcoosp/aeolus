# 17 — The emit arms: the interpreter and the static-data form

Slice: the plan §3.1 emit (c) **interp** arm and emit (b) **static
data** arm (the `Target` enum named them the day slice 4 landed; both
stayed named-but-unarmed until this record). Evidence file per the
workspace convention: the RED spec written first, the GREEN run that
satisfies it, the benches and the consumer examples.

## M1 — the interpreter seam

**RED (spec written first, run against the 0.13.0 seam):**

```text
$ cargo test -p aeolus-compile --test interp
error[E0432]: unresolved imports `aeolus_compile::Env`, `aeolus_compile::InterpError`,
              `aeolus_compile::InterpNote`, `aeolus_compile::Interped`,
              `aeolus_compile::TokenSource`, `aeolus_compile::interp`, `aeolus_compile::interp_theme`
$ cargo test -p aeolus-theme --test theme
error[E0599]: no method named `resolve_rgba` found for struct `TokenTable`
```

**GREEN (the seam implemented, every pin satisfied):**

```text
$ cargo test -p aeolus-compile --all-features
test result: ok. 23 passed  (tests/interp.rs — the contract battery)
test result: ok. 27 passed  (tests/compile.rs — the pre-existing suite, unchanged)
test result: ok. 13 passed  (tests/dyn_cache.rs)
test result: ok. 12 passed  (tests/css.rs — the CSS arm, diagnostics still EQUAL)
test result: ok. 11 passed  (tests/statics.rs — M2 below)
test result: ok. 12 passed  (tests/theme.rs)
test result: ok. 10/25/25…  (custom, doc tests, unit tests)
$ cargo test -p aeolus-theme --all-features
test result: ok. 21 passed  (3 new resolve_rgba pins in tests/theme.rs)
```

What the battery pins (`crates/aeolus-compile/tests/interp.rs`):

1. **The cascade** — a fired layer wins per longhand over every earlier
   layer; within a layer the later declaration wins (`p-4 px-8` → four
   paddings, right values); the resolved declarations come out in `Prop`
   cascade order; `hover:px-8` evicts ONLY left/right (the per-edge
   story at IR level); the walk is deterministic and pure.
2. **Probes answer from the Env** — element states, group/peer, the
   viewport (min forms fire AT the boundary, max forms strictly below
   it, pinned at the exact rem values), the container context (absent =
   the layer never matches AND is reported).
3. **The render-target exclusion** — `::selection`/`::placeholder`
   layers are excluded from the element's own cascade and reported as
   typed notes (never silently folded in, never an error).
4. **The TokenSource seam** — baked pairs resolve by the env mode; the
   runtime tier resolves through the one-load-one-multiply (the interp's
   answer equals `TokenTable::resolve_rgba`'s, the audited primitive);
   a host source wires in and the interp consumes ITS answers (mode
   riding the question); a host's failures surface typed.
5. **The typed boundaries** — the theme-less interp refuses token values
   naming the longhand (`ThemeRequired`); the built-in source maps the
   theme pass's refusals (`UnknownToken`/`UnknownPair`/`AlphaOnPair`/
   `AlphaNotFinite`) onto the interp's own set; every variant carries
   the stable code + help contract; the `miette` feature exposes the
   same strings.
6. **Composition** — the DynCache loop (compile once, interp per frame)
   keeps the cached slices pointer-stable; keywords pass through
   (`Val::Kw` is the codegen crate's business); the touched mask
   mirrors the resolved longhands.

**The theme-pass primitive the seam needed** (`crates/aeolus-theme/src/lib.rs`):
`TokenTable::resolve_rgba(id, alpha, dark)` — the T2 tier's "one load
from `TokenTable` + one multiply" as a mode-aware public primitive, so
every host that resolves programs against a live table shares the
audited math. RED: `no method named resolve_rgba`; GREEN: 3 new pins
(the active-half multiply, the baked literal premultiply, the
unconditional alpha-on-pair refusal, the typed refusals).

## M2 — the static-data emit arm

**RED (spec written first, run against the interp-era seam):**

```text
$ cargo test -p aeolus-compile --test statics
error[E0432]: unresolved import `aeolus_compile::emit_static`
error[E0599]: no variant ... named `Static` found for enum `Target`
error[E0599]: no associated function ... `emit_static` found for struct `Compiled`
```

**GREEN:**

```text
$ cargo test -p aeolus-compile --test statics
test result: ok. 11 passed
$ cargo test -p aeolus-compile --doc
… all doctests ran (the module's doc test compiles and RUNS the
emitted shape against the IR)
```

What the suite pins (`crates/aeolus-compile/tests/statics.rs`): the
byte-stable text (verbatim golden for `p-4`, the empty program, whole
floats keeping `.0`), the totality of the emission arms (every `Val`
variant through a hand-built program, every `Cond` form through the
REAL pipeline — one 22-class battery walks Base through
`MaxContainer(Xl2)`), the mask DERIVED at construction (`Layer::new`,
never a literal), the non-finite defensive arms (`f32::NAN`/
`INFINITY`/`NEG_INFINITY` constructors, never `unreachable!`), the
`Target::Static` arm, and the `Tok`/`ModePair` slots riding verbatim
(the static form is allowed to carry them; the interpreter resolves
them).

## Benches

```text
$ cargo bench -p aeolus-compile --bench interp
interp_baked_button_resting   679 ns   (median)
interp_baked_button_busy      886 ns
interp_runtime_button_busy    919 ns
interp_themeless_button_busy  273 ns
$ cargo bench -p aeolus-compile --bench statics
emit_static_button            5.59 µs
compile_uncached_button      21.4  µs  (the floor beside it)
```

Numbers filed in `reports/08-benchmarks/BENCHMARKS.md` (emit-arms run).

## Consumer examples

```sh
cargo run -p aeolus-compile --example interp         # compile once, interp per env flip
cargo run -p aeolus-compile --example static_emit    # freeze the program into the static item
```

## Honest boundaries surfaced (all documented in the module docs)

* the named group/peer label has no IR slot — the Env answers the
  probe, not the label (the CSS emit arm renders the label; the interp
  documents the same boundary the fold makes);
* `Val::Kw` rides through — the interp resolves THEME dynamism, the
  codegen crate owns the keyword names (plan §3.3);
* per-pseudo-element render targets (`::selection`'s own cascade) are
  reported as notes, resolved by their own future slice;
* the static emitter's one fixed choice is the `PROGRAM` item name —
  hosts rename or wrap; the Rust-TOKENS emit arm (emit (a)) stays the
  P3 codegen crate's surface.
