# 18 — The arbitrary-value unit grammar

Slice: the value pass's last named P2 slice (the boundary every doc
carried — `tests/compile.rs`'s own pin said `w-[32px]` reports — and
the registry rows always ACCEPTED the shape). Evidence file per the
workspace convention: the RED spec written first, the GREEN run that
satisfies it, the benches and the consumer example.

## M1 — the grammar

**RED (spec written first, run against the 0.14.0 seam):**

```text
$ cargo test -p aeolus-compile --test arbitrary
error[E0432]: unresolved import `aeolus_core`
  --> crates/aeolus-compile/tests/arbitrary.rs:39:5
   |
39 | use aeolus_core::Color;
   | ^^^^^^^^^^^ use of unresolved module or unlinked crate `aeolus_core`
```

(The battery cannot even name the audited colour parser — the grammar
does not exist.)

**GREEN (the grammar implemented, every pin satisfied):**

```text
$ cargo test -p aeolus-compile
test result: ok. 38 passed  (tests/arbitrary.rs — the contract battery)
test result: ok. 23 passed  (tests/compile.rs — the pre-existing suite;
                             the w-[32px] boundary pin flipped to the
                             lowering contract, in place)
test result: ok. 12 passed  (tests/css.rs — the CSS arm, diagnostics
                             still EQUAL over the shared batteries)
test result: ok. 25 passed  (tests/custom.rs — the customs differential)
test result: ok. 27 passed  (tests/interp.rs)
test result: ok. 13 passed  (tests/dyn_cache.rs)
test result: ok. 22 passed  (tests/theme.rs)
test result: ok. 11 passed  (tests/statics.rs, tests/cn.rs, doc tests…)
```

What the battery pins (`crates/aeolus-compile/tests/arbitrary.rs`):

1. **Units** — `pl-[7px]`/`h-[13px]`/`rounded-[3px]` (the golden
   corpus's own classes, real-compiler ground truth) lower onto the
   rows' longhand fan-outs as `Val::Px`; `p-[1.5rem]` lowers into the
   FIRST-CLASS `Val::Rem` slot (plan §3.3); `w-[50%]` → `Pct`;
   `opacity-[0.37]`/`z-[13]` → `Num` where the row's own shape contract
   carries a `Number` arm — `w-[13]` refuses typed (the row is the
   legality oracle, the v0.x oracle-consistent reading).
2. **The lexer's sign split** — `w-[-32px]` keeps the value's own sign
   (`Px(-32)`); `-w-[32px]` rides the row-legal negation gate (`w`
   refuses; `-mt-[3px]` flips).
3. **The sibling-row class gate** — `text-[13px]` lowers the font-size
   row ONLY (no line-height, no colour-row noise: the audited v0.x
   oracle's reading, the override fan is `FontSize` alone);
   `text-[#0f0f0f]` lowers the colour row ONLY. The gate is
   DATA-DRIVEN (the rows' own tw_merge groups + numeric arms), with
   drift-catch tests wiring every colour row into `COLOUR_GROUPS` and
   every numeric row into the scalar rule (the `NUMBER_FORMS` audit
   pattern).
4. **Type hints** — `length:`/`percentage:`/`number:`/`color:`
   constrain the parse (`length:0` is the CSS-sanctioned zero length;
   the `number:` hint is the caller's explicit assertion); a MISMATCHED
   hint (`w-[length:#fff]`) and a no-IR-slot hint (`w-[angle:45deg]`)
   refuse typed, never silently re-interpreted. The hint table IS the
   lexer's (`aeolus_syntax::HINTS`, now exported — one source of truth).
5. **Colours** — hex spellings parse through the audited
   `aeolus_core::Color` (the `color_parse` fuzzer's own target); the
   `/opacity` modifier bakes through the audited premultiplied identity
   (`mix_with_transparent`, byte-identical to the token path's
   literals); a modifier on a scalar arbitrary refuses (never
   half-applied).
6. **var() references** — `bg-[var(--color-primary)]` lowers to the
   SAME tiered value `bg-primary` produces (test-pinned equality); the
   v4.1 `(--var)` shorthand is the same path; the theme-less default
   refuses naming `compile_theme`; an unknown name refuses against
   `TokenTable::names()`.
7. **The typed boundaries** — multi-token composites (`calc()`,
   underscore joins), the arbitrary-PROPERTY form, keyword values on
   the row-relative `Val::Kw` slots, deg units off the `Deg` rows,
   non-finite spellings — every refusal carries the stable code
   (`aeolus::compile::unsupported_value`) and the updated help text.
8. **The two walks agree** — `compile` and `emit_css` report EQUAL
   diagnostics over a 24-class battery (the load-bearing differential);
   the CSS arm renders the corpus's forms (`padding-left: 7px`,
   `background-color: #0ea5e9` verbatim, `1.5rem`, `width: 50%`) and
   the baked alpha in the space-separated form.
9. **Composition** — `cn()` merges arbitrary values rightmost-wins per
   longhand; the theme-less interp resolves the concrete values (no
   `ThemeRequired` boundary); `emit_static` carries them as valid f32
   literals; the grammar is deterministic.

## The design decisions (the why)

* **The IR is unchanged.** The grammar lowers into the EXISTING
  `Val` slots — no new variants, the static form and the interpreter
  consume the output as-is. What the IR cannot carry refuses typed
  (the honest boundary set grew NARROWER, never wider).
* **The sibling-row gate is data, not a table of classes invented per
  root.** A row takes a value class iff its conflict group is a colour
  group (the audited tw_merge names) or its own shape contract carries
  a numeric arm — the registry is the oracle, and two drift-catch
  tests wire the audit into CI.
* **Bare numbers ride the row's `Number` arm.** `z-[13]` lowers,
  `w-[13]` refuses — the same legality oracle the spacing scale uses,
  and the v0.x oracle's own reading. The `number:` hint is the
  explicit contract that overrides (v4 emits the value verbatim).
* **`text-[13px]` fans `FontSize` alone** (the oracle's reading; the
  composite (size, line-height) pair rides the NAMED sizes). One
  override entry, one audit test.
* **`rem` stays `rem`.** The v0.x oracle bakes rem→px at 16; the V1 IR
  has a first-class `Rem` slot and plan §3.3 says why (the window's
  rem size). Documented divergence, IR-faithful.
* **`opacity-[0.37]` reads verbatim** (`Num(0.37)`, CSS `0.37` ==
  `37%`) — the v0.x oracle's `/100` on the arbitrary form is a bug the
  corpus does not pin; the divergence is deliberate and documented.
* **Modifiers stay the theme pass's.** The theme-less default refuses
  `/50` exactly like it refuses it on colour tokens — the themed forms
  (`compile_theme`/`emit_css_theme`) lower it through the audited bake.

## The benches and the example

* `benches/arbitrary.rs` — the battery against the scaled floor
  (~150 ns/class classify cost; the numbers in
  `reports/08-benchmarks/BENCHMARKS.md`).
* `examples/arbitrary_values.rs` — the end-to-end walk: theme-less vs
  themed lowering, the diagnostics, the CSS arm.

```sh
cargo run -p aeolus-compile --example arbitrary_values
cargo bench -p aeolus-compile --bench arbitrary
```

## What this slice deliberately is NOT

The multi-token composite grammar (`calc()`, underscore joins,
url/image values, shadow/filter lists), the arbitrary-PROPERTY form,
keyword values on the row-relative `Val::Kw` slots, and the wider v4
hint table — the codegen host's surface (plan §3.2's
`aeolus-codegen` row), each refusing typed with the reason naming it.
The Rust-tokens emit arm, the has-/data-/aria- variant folding, P3
(GPUI codegen) and `xtask` remain the planned slices.
