# 22 — The descendant-probe seam

Slice: the interp arm's FOURTH seam member — the `has-*`/
`group-has-*`/`peer-has-*` family answers through a host question,
the boundary the `AttributeUnqueried` note's descendant case carried
as a diagnosed boundary since the attribute-probe fold (evidence file
20) landed. Evidence file per the workspace convention: the RED spec
written first, the GREEN run that satisfies it, the benches and the
consumer examples. Two milestones, one red→green cycle each:

* **M1** — the descendant-probe seam: `DescendantKind` (the three
  `:has()` scopes the typed `Cond` surface folds), `DescendantProbe`
  (the one question), `DescendantState` (the built-in static-fact
  table), the dedicated `InterpNote::DescendantUnqueried` note (the
  blanket attribute spelling retires for the family), the
  `*_descendants` entry-point matrix (16 free functions + the 16
  `Compiled` methods, the full composed matrix included) and the
  backends' note arms (xilem/iced/egui);
* **M2** — the CLI face: `expand --has kind=fragment` (the
  `--container` flag's sibling), the four-seam dispatch matrix, the
  typed input gate (exit 2 over a missing value, a shape miss, an
  unknown kind, an empty fragment), the usage text.

The CSS semantics the pins encode, the `:has()` reading: a YES fires
the layer; a source-answered NO is a QUIET skip (a `:has()` selector
that does not match, never an exclusion); the sourceless forms keep
the reported exclusion (the dedicated note, the per-seam convention
the container and structural notes follow). The scope discriminates
the question — the element's own subtree, the `.group` ancestor's,
the `.peer` sibling's — the label forms ride the CSS head only, so
one scope answers per family.

## M1 — the descendant-probe seam

**RED (spec written first, run against the 0.19.0 seam):**

```text
$ cargo test -p aeolus-compile --test descendants
error[E0432]: unresolved imports `DescendantFact` / `DescendantKind`
  / `DescendantState` — no such types in `aeolus_compile`
error[E0599]: no method named `interp_descendants` /
  `interp_theme_descendants` / `interp_probes_descendants` …
  found for struct `Compiled`
error: could not compile `aeolus-compile` (test "descendants")
  due to 13 previous errors
```

(The battery imports the typed surface it pins; at HEAD the seam
types and entry points do not exist, and `has-focus:p-8` under the
plain walk reports the blanket `AttributeUnqueried` spelling.)

**GREEN (the seam implemented, every pin satisfied):**

```text
$ cargo test -p aeolus-compile --test descendants --all-features
test result: ok. 15 passed; 0 failed
  (the fire, the answered-miss quiet skip, the data/aria compounds,
   the sourceless note's cond/props/text, the group/peer scope
   discrimination, the compositions, the FULL walks, the matrix
   pairs/triples, the Compiled method face, the built-in table)

$ cargo test -p aeolus-compile --all-features
39 test binaries, all green (the moved pins: the group-has note in
tests/attr_variants.rs, the style fold's descendant note in
tests/style_walk.rs, the note finders in tests/interp.rs /
tests/probes.rs / tests/container_bounds.rs / tests/structural.rs)
```

**Benches (the seam's own slice of the per-frame story, the CLI face beside):**

```text
$ cargo bench -p aeolus-compile --bench descendants
descendants                   fastest   │ median    │ samples
├─ interp_descendants_button  255.7 ns  │ 257.9 ns  │ 100  (the fact table hits, a layer fires)
╰─ interp_sourceless_button   99.01 ns  │ 100 ns    │ 100  (the note path, the before number)

$ cargo bench -p aeolus-cli --bench expand
├─ expand_descendant_facts    254.7 ns  │ 256 ns    │ 100  (the --has face's walk)
```

(The medians file into `reports/08-benchmarks/BENCHMARKS.md`'s
descendant-seam section — the attribute seam's ~26 ns/layer consult
shape's sibling, ~50 ns/layer here with one layer firing.)

**Examples (headless, in-process):**

```text
$ cargo run -p aeolus-compile --example descendants
fired (has-focus wins the cascade):
  PaddingTop -> Rem(2.0) …
scope discrimination:
  group-has-focus fired: PaddingTop -> Rem(0.5)
  peer-has-focus missed quietly: PaddingBottom present? false
sourceless walk:
  the Has("*:focus") layer never matched: no DescendantProbe source
  rides — the subtree's :has() facts are not frame state (padding …)

$ cargo run -p aeolus-cli --example expand_descendants
── the sourceless walk: the layer reports (no facts ride) ──
note: the Has("*:focus") layer never matched: …
── --has element=*:focus: the subtree answers, the layer fires ──
padding-top = 2rem …
── the scope discriminates: the group answer never rides the peer ──
padding-top = 0.5rem
```
