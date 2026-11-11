# 19 — The rust-tokens emit arm

Slice: the plan §3.1 emit (a) **Rust tokens** arm — the LAST arm of the
`Target` surface (the enum named it the day slice 4 landed; it stayed
named-but-unarmed through the CSS/interp/static slices). Evidence file
per the workspace convention: the RED spec written first, the GREEN run
that satisfies it, the benches and the consumer example. M2 of the
session's plan — the has-/data-/aria- variant folding — is identified
and DEFERRED (see the closing section; a partial IR-surface slice does
not ship).

## M1 — the rust-tokens emit arm

**RED (spec written first, run against the 0.15.0 seam):**

```text
$ cargo test -p aeolus-compile --test rust_tokens
error[E0432]: unresolved imports `aeolus_compile::emit_rust`,
              `aeolus_compile::emit_rust_theme`, `aeolus_compile::emit_rust_with`
error[E0599]: no variant, associated function, or constant named `Rust`
              found for enum `Target`
```

**GREEN (the arm implemented, every pin satisfied):**

```text
$ cargo test -p aeolus-compile --all-features
test result: ok. 19 passed  (tests/rust_tokens.rs — the contract battery)
test result: ok. 23 passed  (tests/interp.rs — unchanged)
test result: ok. 11 passed  (tests/statics.rs — unchanged)
test result: ok. 12 passed  (tests/css.rs — the CSS arm, diagnostics still EQUAL)
test result: ok. 27 passed  (tests/compile.rs — unchanged)
test result: ok. 13 passed  (tests/dyn_cache.rs)
test result: ok. 12 passed  (tests/theme.rs)
test result: ok. …          (custom 25, arbitrary 22, cn 11, unit suites, doc tests)
$ cargo test -p aeolus-compile --doc
… all doctests ran (the module's doc test compiles and RUNS the emitted
static item against the IR — masks derived, sorted)
```

What the battery pins (`crates/aeolus-compile/tests/rust_tokens.rs`):

1. **The artifact** — the fragment is a complete, embeddable
   `aeolus_ir::Layer::new(...)` expression (verbatim golden for `p-4`);
   the sheet's `Display` assembles the compilable
   `static PROGRAM: &[aeolus_ir::Layer]` item, layers
   specificity-sorted (stable, source order breaking ties) while the
   fragments themselves stay per-candidate in SOURCE order (the
   `tw!`-style consumer's unit); the text is deterministic (same input,
   same bytes).
2. **The keyword resolution** — the capability the program-only
   emitters structurally lack: `Val::Kw` slots render their ROW text as
   a trailing comment (`flex-col` → `Val::Kw(0) }, // column`), bare
   roots their own value text (`hidden` → `// none`), composite rows
   their per-target texts (`text-sm` → `// 0.875rem` on the font-size
   decl, `// calc(1.25 / 0.875)` on the line-height decl). The slot
   indexes the ROW'S OWN table — a bare `Program` cannot recover the
   text, which is why this arm renders IN the pipeline (the CSS arm's
   stated rationale).
3. **The theme annotations** — runtime `Val::Tok` slots render their
   table name (`bg-primary/50` → `Val::Tok { id: 151, alpha: 0.5 } },
   // primary`, the table's canonical unprefixed key); the baked dark
   pair has no name table and rides verbatim, honestly unannotated.
4. **Totality** — one 22-class battery walks every `Cond` form through
   the REAL pipeline (Base through `MaxContainer(Xl2)`); the px spacing
   config bakes pixels; negation and the `px` step render.
5. **The shared walk** — diagnostics stay EQUAL to `compile`'s over a
   battery (lexer misses, unknown roots, shape rejections, variant
   refusals, importance, theme-less modifiers, the arbitrary boundary
   set, mixed strings); empty `Customs` registrations are
   byte-identical to the plain entry points (the differential), and a
   registered custom utility lowers through the same value pass.
6. **The arm** — `Target::Rust` completes the plan §3.1 surface
   (`Css`/`Rust`/`Interp`/`Static`).

**The annotation pass the arm needed** (`crates/aeolus-compile/src/lower.rs`):
`kw_annotations(row, body)` — the row-table read side mirroring
`row_to_decls`'s arm selection (same suffix rule, same AnyOf
flattening, same acceptance order) but only READING the tables — no
lowering, no refusals; arbitrary candidates never consult it. The
static emitter's `val_literal` became `pub(crate)` so both emitters
share one float contract (the emitted value text cannot drift).

## M2 — identified and DEFERRED: has-/data-/aria- variant folding

The next arch milestone candidate: fold the `has-…:`/`data-…:`/
`aria-…:`/arbitrary-variant segments into the typed surface (today they
report `UnsupportedVariant`). DEFERRED, deliberately: the fold has no
honest IR slot yet — `Cond` is a fixed probe/bp/container enum, the
`Env` answers nine states plus two widths, and the CSS arm renders
these selectors from the candidate's SEGMENTS (which is why the CSS
emit carries them today). Shipping it means a multi-crate IR-surface
design (new condition carrier or attribute slot, new Env probe
questions, interp exclusion notes, CSS conjunctive shapes) with its own
red-green cycle across `aeolus-ir`/`aeolus-compile` — a partial slice
does not ship. The boundary stays typed and honest
(`UnsupportedVariant` carries the segment text and its help names the
next slice).

## Benches

```text
$ cargo bench -p aeolus-compile --bench rust_tokens
emit_rust_button             31.33 µs  (median 31.81 µs)
compile_uncached_button      21.6  µs  (the floor beside it)
compile_cached_button        594.8 ns  (the dyn-cache loop under it)
$ cargo bench -p aeolus-compile --bench statics  (re-run for the ratio)
emit_static_button           4.43 µs
compile_uncached_button      21.3 µs
```

Numbers filed in `reports/08-benchmarks/BENCHMARKS.md`
(rust-tokens-arm run). The emit walk sits ~1.45× over the uncached
compile floor (the annotation pass + the render, one String per
fragment) and stays a build-time artifact cost.

## Consumer example

```sh
cargo run -p aeolus-compile --example rust_tokens   # per-class fragments + the assembled static item
```

## Honest boundaries surfaced (all documented in the module docs)

* the backend METHOD calls (`FlexDirection::Column`, builder chains)
  are the P3 codegen crate's surface — the arm renders the IR's own
  token shape, annotated where only the pipeline can see;
* a `Compiled::emit_rust` convenience does NOT exist and cannot: the
  program alone carries `Val::Kw` slots, the row context is gone once
  the walk ends (the `emit_static`/`interp` convenience shapes do not
  apply);
* the baked dark pair rides without a name annotation (the pair table
  indexes colours, not names — the interp/static arms carry the same
  slot);
* custom utilities carry their own `Val`s verbatim — no row-table
  annotations exist for them;
* the has-/data-/aria- variant folding stays the typed
  `UnsupportedVariant` boundary (M2 above).
