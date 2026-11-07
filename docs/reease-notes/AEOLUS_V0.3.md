# Aeolus v0.3 Plan (plan of record)

The v0.2 plan (`AEOLUS_V0.2.md`) shipped in 0.2.0. This document is the
v0.3 plan of record; every `v0.3 §X` citation in code and reports refers
here. It is scoped by the "Deferred to v0.3" list the 0.2.0 README and
changelog carried, split into what is implementable in-repo and what is
upstream-blocked.

Goal: close the last core-language gap — **arbitrary selector variants**
(`[&_svg]`, `[a&]`, `[@media(…)]`, …) — so that every v4 variant form
either resolves to typed probe conditions or fails with a precise
`Unsupported` reason. Never a blanket "arbitrary variants unsupported",
never a silent drop (§7 of the v0.2 plan).

## §1 — Semantics (verified against tailwindcss v4.1.17 source)

References: `packages/tailwindcss/src/candidate.ts` (arbitrary-variant
parsing), `compile.ts` (`applyVariant`), `variants.ts` (static variant
selectors). The contract below mirrors that source, not the docs alone.

- §1.1 An arbitrary variant's selector positions the element with `&`.
  A selector without `&` (and not starting with `>`/`+`/`~`/`@`) is
  wrapped as `&:is(sel)` — an implicit SELF test (`[p]:` ≡ `a`
  element-kind test, `[.x]:` a class test). Relative selectors
  (`[>img]:`) are invalid standalone; at-rule variants
  (`[@media(…)]:`) must not contain `&`.
- §1.2 The typed subset — selectors that constrain THE ELEMENT (the
  only thing the `Probe` model can answer about):
  - `[&:hover]`, `[:hover]`, `[&:hover:focus]`, `[p:hover]` — self
    pseudo-classes → `Atom::On(Test::State(_))` (compound = conjunction).
  - `[&:not(:hover)]`, `not-[&:hover]` — negation → `Atom::Not(_)`.
  - `[&[data-state=open]]`, `[&[aria-busy=true]]` — self attributes →
    `Test::Data` / `Test::Aria` (bare non-data/aria attributes are
    reported, not guessed).
  - `[&:nth-child(3)]`, `[&:first-child]`, `[&:empty]` — structural →
    `Test::Structural` (An+B forms stay reported).
  - `[&:has(:focus)]`, `has-[:focus]` — descendant state → `Atom::Has(_)`.
  - `[a&]`, `[a]`, `[p:hover]` — element-kind (tag) tests → the new
    `Test::Tag` (native backends cannot answer it and the rule simply
    never fires — the same semantics CSS has when a selector does not
    match).
  - `[@media(print)]`, `[@media(prefers-color-scheme:dark)]`,
    `[@media(min-width:900px)]`, `[@media(width>=900px)]`,
    `[@media(orientation:landscape)]`, the `prefers-reduced-motion` /
    `prefers-contrast` / `forced-colors` / `pointer` features, and
    `and`-conjunctions of those → `Atom::Media` / `Atom::Viewport`.
    `[@container(min-width:400px)]` → `Atom::Container`.
  - `group-[&:hover]` / `peer-[&:hover]` inners classify like self
    selectors and become the group/peer test.
- §1.3 Everything else is REPORTED with a per-form precise reason (the
  honesty contract): selectors styling OTHER elements (`[&_svg]`,
  `[&>*]`, `*`, `**` — the Probe styles the element itself),
  pseudo-elements (`[&::before]`), class/id self tests (`[.x&]`),
  `@supports` (no capability query in the model), disjunctions
  (`,`/`:is(a,b)` — the `Cond` model is a conjunction), An+B structural
  forms, unrecognized media features, unknown pseudo-classes.
- §1.4 Canonicalization: `[&:hover]` generates byte-identical CSS to
  `hover:` in v4, so the single-classic-state form keeps routing onto
  the legacy layer buckets (native lowering on Floem for free). Merge
  (`cn()`) keys stay the RAW variant text — `[&:hover]:p-4` and
  `hover:p-4` do not knock each other out, matching tw_merge.
- §1.5 Underscores decode to spaces (`\_` escapes), as
  `decodeArbitraryValue` does.

## §2 — Implementation shape

- §2.1 `crates/aeolus-core/src/selector.rs` — the zero-dependency
  classifier: raw variant text → `Vec<Atom>` or a precise
  `&'static str` reason. No new dependencies, no new public parser API;
  the parser keeps storing `Variant::Arbitrary(raw)` (§1.2 of the v0.2
  plan: raw syntax in the AST, semantics in the resolver).
- §2.2 `cond::Test::Tag(Box<str>)` — additive variant on the public
  condition enum (0.x minor bump; exhaustive external matches need a
  new arm — documented in the changelog). `Atom::order` ranks it with
  the state band (10); the ordering approximation is documented.
- §2.3 Routing: `route()` classifies `Variant::Arbitrary` per variant
  and appends the atoms (a stack of `md:[&:hover]:` becomes the
  conjunction). `variant_test` (not-/group-/peer- inners) requires a
  single self test. `has_atom` accepts `[:state]` selector forms.
- §2.4 Backends need no changes: the typed rules flow through the
  existing `Probe` paths (Masonry `plan.resolve` in the fallback
  widget; Floem `resolve_rules_only` in the reactive `tw_plan`
  closure). Probes answer `Test::Tag` "no" by default — honest.

## §3 — Quality gates

- §3.1 The v0.2 battery stays green: `cargo test --workspace`,
  `clippy -D warnings`, `fmt --check`, `doc -D warnings`,
  `cargo package -p aeolus-core -p aeolus-macro`, `cargo deny check
  licenses`.
- §3.2 New tests: `tests/arbitrary_variants.rs` (typed subset +
  per-form precise reports + cn() keys + evaluator equivalence with the
  named-equivalent variants), truth-table rows for `Test::Tag`, and
  backend parity tests pinning rule evaluation through both probes.
- §3.3 Fuzz targets unchanged (`parse_variant` already accepted
  arbitrary variant syntax; the classifier must be panic-free on any
  input — it only slices ASCII-checked boundaries and never indexes
  past `len`).

## §4 — Upstream-blocked (stay deferred, re-documented)

| item | blocker |
|---|---|
| publishing the backends | needs crates.io releases of the pinned Masonry/Xilem and Floem revs (`reports/09-publishing/`) |
| view-side property-stack registration (retires the fallback widget) | upstream gap #4 (`reports/07-upstream-patches/PATCHES.md` patch 2) |
| Masonry typography via native properties | upstream patch proposal #3 |
| transitions/animations on the Masonry backend | needs Masonry-side animation machinery (per-set transitions exist on Floem; Masonry drops with warnings) |
