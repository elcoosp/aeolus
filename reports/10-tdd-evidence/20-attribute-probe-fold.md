# 20 — The attribute-probe variant fold

Slice: the variant surface's next P2 slice — the `data-*`/`aria-*`/
`has-*` attribute-probe spellings fold onto the typed `Cond` surface
(the boundary the variant docs carried as "what does NOT fold" since
the compile seam landed). Evidence file per the workspace convention:
the RED spec written first, the GREEN run that satisfies it, the
benches and the consumer example. Two milestones shipped in one red→
green cycle per milestone:

* **M1** — the element-own probes: `Cond::Data`/`Cond::Aria`/`Cond::Has`
  (the fold grammar, the CSS arm's selector forms, the interp's typed
  exclusion note, the static/rust arms' payload emission, the custom
  seam's generic aliasing);
* **M2** — the parent-relative has-probes: `Cond::GroupHas`/
  `Cond::PeerHas` (the same inner grammar on the parent tier, the CSS
  arm's conjunctive `.group:has(...)`/`.peer:has(...) ~` heads, the
  label form riding the head only).

The differential oracle is IN this tree: the v0.x resolver's own
`has_atom`/`parse_attr`/`attr_test` (aeolus-core), pinned against the
real tailwindcss v4.1.17 output in the golden corpus —
`data-[state=open]` → `[data-state="open"]`, bare `aria-busy:` →
`[aria-busy="true"]`, `has-focus:` → `:has(*:focus)`,
`has-data-[slot=card-action]:` → `:has(*[data-slot="card-action"])`.

## M1 — the element-own attribute probes

**RED (spec written first, run against the 0.16.0 seam):**

```text
$ cargo test -p aeolus-compile --test attr_variants
error[E0432]: unresolved import `aeolus_ir::Cond`
  → no variant `Data` / `Aria` / `Has` found for enum `Cond`
error[E0432]: unresolved import `aeolus_compile::InterpNote`
  → no variant `AttributeUnqueried` found
```

(The battery imports the typed surface it pins; at HEAD the probe
variants do not exist — `cond.rs` carries no `Data`/`Aria`/`Has` — and
`has-data-[x]:p-4` is a documented `UnsupportedVariant` refusal pin.)

**GREEN (the fold implemented, every pin satisfied):**

```text
$ cargo test -p aeolus-compile --all-features
test result: ok. 40 passed  (tests/attr_variants.rs — the contract battery)
test result: ok. 25 passed  (tests/compile.rs — the pins moved to the new boundary)
test result: ok. 12 passed  (tests/cn.rs — the merge report pin moved likewise)
test result: ok. 23 passed  (tests/interp.rs — the note finder matches the new note)
test result: ok. 12 passed  (tests/css.rs — the CSS arm, diagnostics still EQUAL)
test result: ok. 25 passed  (tests/custom.rs — the alias seam stays generic)
test result: ok. 38 passed  (tests/arbitrary.rs — the value pass untouched)
test result: ok. 13 passed  (tests/dyn_cache.rs) / 12 (theme) / 11 (statics)
                            / 19 (rust_tokens) / 12 (cn) …
$ cargo test --workspace --all-features          # 80 suites, 0 failures
$ cargo test --benches --all-features            # every bench compiles and runs
```

What the battery pins (`crates/aeolus-compile/tests/attr_variants.rs`):

1. **The fold** — `data-[state=open]:`/`data-open:` →
   `Cond::Data { name, value }` (the bracketed presence spelling and
   the bare spelling are ONE condition), `aria-busy:`/`aria-[sort=asc]:`
   → `Cond::Aria` (the bare spelling normalizes to the `"true"` value
   the corpus renders), `has-focus:`/`has-[:focus]:`/`has-data-[…]:`/
   `has-aria-…:` → `Cond::Has` (the star-prefixed `:has()` fragment).
2. **The flat fold still applies** — stacked variants keep the
   dominant condition by the IR's specificity ranks (the probe is the
   state tier 2, the parent probe tier 3, dark tier 1), equal ranks
   stay separate layers in source order.
3. **The grammar refuses honestly** — empty names/values, quotes and
   backslashes in payloads, compound/structural `has-` inners,
   `has-[x]` (names no state and no data-/aria- compound), and the
   parent×attribute compositions (`group-data-*`, `peer-aria-*`) all
   come back typed `UnsupportedVariant` with the segment's own span,
   stable code and help text — never silent.
4. **The CSS arm renders the corpus's selector forms** — the attribute
   test compounding on the escaped class (`.data-\[state\=open\]\:p-4[data-state="open"]`),
   the presence form bare, the `:has()` fragment star-prefixed, the
   probe riding both rules of a baked dark-pair companion, the probe
   compounding AFTER the pseudo suffixes in a source stack, the
   `@media` wrap carrying the probe.
5. **The interpreter arm reports, never fires** — attribute layers are
   excluded under the element-state `Env` as typed
   `InterpNote::AttributeUnqueried { cond, props }` notes (the plan
   §3.5 state-bits surface is the follow-up); base layers still fire
   beside them.
6. **The static-data and rust-tokens arms emit the payload conds** —
   the derived `Debug` IS the constructor text
   (`aeolus_ir::Cond::Data { name: "open", value: None }`), the
   statics arm's documented name-table contract.
7. **The merge and the custom seam stay generic** — `cn` folds
   same-probe layers and keeps disjoint ones; a registered
   `CustomVariant` may target the new conds; the two walks' diagnostics
   stay EQUAL over a 21-input battery with probes in play.

## M2 — the parent-relative has-probes

**RED→GREEN (same file, second milestone):** the `group-has-*`/
`peer-has-*` pins fail against the M1-only seam (folded as unknown
parent prefixes) and pass once `Cond::GroupHas`/`Cond::PeerHas` land:
the fold rides the parent tier with the SAME inner grammar the
element-own arm uses (one grammar, three consumers — the refusal sets
cannot drift), the label form folds to the same probe (the label is
the CSS head's decoration, the group-hover precedent), the parent
probe outranks the element probe in the dominant-condition fold, and
the CSS arm renders the conjunctive heads
(`.group:has(*:focus) .class`, `.group\/menu:has(*:focus) …` with both
class halves escaped, `.peer:has(*[data-slot="x"]) ~ .class` with the
sibling combinator). Compound inners (`group-has-[p:hover]`) refuse
typed; the interp notes the parent probes like the element probes.

## Micro-benchmarks

```text
$ cargo bench -p aeolus-compile --bench attr_variants
compile_base         556 ns   (one bare utility — the no-variant floor)
compile_state_fold  1.23 µs   (one state segment — the established path)
compile_probe_fold  4.06 µs   (four probe segments, four payload interns)
```

Numbers filed in `reports/08-benchmarks/BENCHMARKS.md`
(attribute-probe-fold run). The probe fold sits on par with the state
fold per segment — the delta over the base floor is the four variant
classes themselves, not the probe grammar; the dyn-cache dedupes the
warm repeat.

## Consumer example

```sh
cargo run -p aeolus-compile --example attr_variants   # fold → CSS → interp notes → static head
```

## Honest boundaries surfaced (all documented in the module docs)

* the interpreter's `Env` answers element STATES, not DOM probes — the
  attribute layers are excluded and REPORTED; the typed state-bits
  surface (plan §3.5) is the follow-up that makes them fire;
* the parent×attribute compositions (`group-data-*`, `peer-aria-*`)
  stay typed refusals — only the has- family composes on the parent
  tier today;
* the arbitrary-variant form and the structural `has-` inners (no
  `first:` surface in the compile stage yet) stay the typed
  `UnsupportedVariant` boundary;
* `Cond::source_text` returns `None` for the payload conds (the
  segment text lives in the payload, no const table reconstructs it) —
  the diagnostic surfaces carry the payload when a reader needs the
  probe text;
* payloads intern into the leak pool per folded segment — the IR's
  documented dynamic-form storage, the same cost the program's own
  slices pay; the dyn-cache dedupes the warm repeat.
