## [Unreleased]

### 🚀 Features

- *(data)* The registry's sort-order read seam (plan §5.5's "class sorting (official order from the registry)"): `registry::sort_index` answers a utility BODY's canonical order position — the FIRST matching row's index, under the same longest-prefix + shape rules `lookup` applies, `None` on a typed miss (no root, rejected shape, never a guessed position) — and `registry::row_count()` names the boundary the custom registrations sort after. The rows' own order IS the canonical utility progression; the pre-index linear scan stays as the differential oracle, battery-pinned over every root and value shape
- *(compile)* The class-sort seam (`sort`/`sort_with`): one read-side walk orders a class string into the registry's progression — base classes first by row index, variant buckets after the base by the cascade specificity the IR itself sorts layers by (`Cond::specificity`, base < dark < probes < parent < media < container), registered custom utilities after every registry row in registration order (the registry wins name collisions; empty registrations are byte-identical to `sort`) — and classes the seam cannot ORDER (lex failures, unsupported variants, registry/custom misses) ride the source-order tail with the SAME typed, coded diagnostics the compile walk reports. The sort is stable (equal keys keep source order), idempotent, and total on any non-empty input; the value pass and the theme pass are deliberately NOT consulted (order is a shape-level fact — a themeless token boundary sorts exactly like its lowered sibling)
- *(compile)* The interp arm's winner attribution: `Interped::conds` rides parallel to `decls` — the condition each effective declaration's winner came from, the cascade's winner attribution the row-relative `Val::Kw` annotation texts render from (a host reading `decls[i]` beside `conds[i]` knows WHICH condition won the longhand without re-walking the layers). Additive field; every existing read site is unchanged
- *(cli)* The `expand` command (§3.2's CLI row, the interp arm's human-readable form): a class string's EFFECTIVE declarations under one probe `Env` — the per-longhand cascade, the keyword slots resolved to their WINNER's row text through the `(cond, prop)` annotation table (an unfired layer's text can never shadow the winner's), the honest exclusions as typed notes, `--theme shadcn|shadcn-dark|<path>` lowering tokens through the V1 table (the themed walk rides `interp_theme`), `--dark` lifting the mode tier, `--json` carrying the declarations array plus notes. The command joins `aeolus-ir` (workspace-internal) for the `Val`/`Prop` render types
- *(cli)* The `sort` command (§3.2's CLI row, §5.5's parity line): `aeolus sort [--json] <classes...>` renders the registry-ordered string, the typed misses as exit-1 findings (the sorted string still renders — the honest tail), `--json` carrying the sorted string plus the diagnostics array. The CLI joins nothing new: the command rides the workspace-internal `aeolus-compile` join the `explain` command established
- *(gpui)* The attribute-bits host state seam (P4's last named piece, plan §3.5's "typed state bits passed WITH the element"): `TwExt::tw_probes`/`tw_env_probes` walk the base + environment layers with the element's attribute bits in hand — the GPUI half of the interp arm's `interp_probes` entry point. `data-*:`/`aria-*:` layers answer per CSS selector semantics (a presence spelling fires on any value, a value spelling on equality, a miss stays CSS-quiet); the sourceless walk keeps them quiet exactly as before (the interp's `AttributeUnqueried` note has no GPUI channel), and the `has-*` descendant shapes stay the plan's diagnosed boundary with or without bits. The seam is the trait: the built-in `AttrState` bit table joins the prelude beside `AttrBit`/`AttrKind`/`AttrProbe`, and a host `AttrProbe` impl answers the same walk
- *(cli)* The `explain` command (plan §5.5's tooling-parity line, "class → Rust it emits"): per-class Rust fragments the Target::Rust arm renders (one embeddable `Layer::new` expression per lowered class, keyword/token slots annotated), every miss as the SAME typed, coded diagnostics the pipeline reports (exit 1 findings), `--theme shadcn|shadcn-dark|<path>` lowering colour tokens through the V1 token table (the themeless emit keeps the token boundary honest — the diagnostic names the lowering path). The CLI joins `aeolus-compile` (workspace-internal; the no-external-crate discipline holds)

### 🧪 Testing

- *(data)* Pin the sort-order seam (`tests/registry.rs`: `sort_index` vs the linear-scan oracle over a battery built from every row's own value shapes, the canonical-progression chain, the typed misses)
- *(compile)* Pin the sort seam end to end (`tests/sort.rs`: registry order, the variant buckets and the stacked-variant fold, the honest tail per miss mode (unknown root, shape rejection, lex failure, unsupported variant) with spans and stable codes, the customs-after-registry order and the collision win, stability, idempotence, whitespace normalization, the empty-input error — RED first: the capture was the `no sort_index/sort/sort_with` compile failure plus the seven CLI command failures)
- *(compile)* Pin the winner attribution (the interp suites' effective-declaration contracts run unchanged; the `conds` parallel rides every existing pin)
- *(cli)* Pin the `expand` command contract (`tests/cli.rs`: the effective declarations, the keyword slots' winner texts, the resting-env probe semantics, `--dark`, `--theme`, the themeless token boundary as exit 1, the `::selection` note, `--json`, unknown flags, the missing-string usage error)
- *(cli)* Measure `expand` (`benches/expand.rs`: the themeless effective walk, the themed walk, the keyword-heavy render)
- *(cli)* Pin the `sort` command contract (`tests/cli.rs`: the registry order, the variant buckets, misses as exit-1 findings with the coded diagnostic, the `--json` shape, unknown flags, the missing-string and empty-string usage errors)
- *(gpui)* Pin the host state seam end to end (`tests/attrs.rs`: the value/presence spellings, the verbatim aria bracket form, the sourceless quiet, the has- boundary, the dark+attrs composition, the custom-probe impl)
- *(gpui)* Measure the seam (`benches/attrs.rs`: the sourceless walk, the bit-miss walk, the bit-hit walk over the shadcn Button string's own `aria-invalid` layers)
- *(cli)* Pin `explain` (`tests/cli.rs`: one fragment per lowered class, misses as exit-1 diagnostics, the `--theme` path, the themeless token boundary, unknown flags, the missing-string usage error)
- *(cli)* Measure `explain` (`benches/explain.rs`: the themeless emit, the themed emit, the diagnostics path)

### 📚 Documentation

- *(data)* File the sort-order read seam under its landed form (the crate, `rows` and `registry` docs no longer list the class-sort order as future generator work)
- *(readme)* Add the `sort` and `expand` commands to the CLI crate row, the shell walkthrough, the example list and the status paragraph; mark the §5.5 class-sorting half and the interp winner attribution landed in the architecture index
- *(architecture)* Mark the class-sorting half of the §5.5 tooling-parity line landed (the P2 status paragraph)
- *(readme)* Align badge/status to v0.19.0, the six-backend crate table (xtask joins), the GPUI tagline, the `explain` command and the new examples
- *(architecture)* Mark the attribute-bits host state seam landed (the P4 row and the remaining-boundaries line)
- *(ci)* Correct the workflow header's member count (19 members, six backend crates)
- *(changelog)* Give the shipped 0.19.0 its release header (the items sat in Unreleased since the tag) and remove the stale 0.15-era `Unreleased` block

### 💼 Other

- *(codegen)* Remove the dead `supports` stub (the capability tables live on the target crates — `aeolus-codegen-gpui`'s own table — the crate-level default had no consumer)
- *(core)* File the conflict.rs compile-time story under its landed form (the V1 pipeline, not the once-planned pre-interned-ID trie)

## [0.19.0] - 2026-11-11

### 🚀 Features

- *(compile)* The plan-lowering emit arm (`Target::Plan`, architecture plan §7's migration seam): `emit_plan`/`emit_plan_theme`/`emit_plan_with`/`emit_plan_theme_with` lower the V1 pipeline's artifact into the v0.x `StylePlan` the five backend crates already consume — the registry's Prop surface written slot-for-slot (the per-edge `EdgeSet` masks, the per-corner `CornerSet`, the composite `text-sm` px pair, the runtime tokens through the theme pass's table), the fold's conditions routed into the v0.x buckets (the seven classic states into `layers`, `dark:` into `dark`, `sm:`/`min-*:` into `responsive`, `group-*:`/`peer-*:` into the label vecs, the attribute probes and `max-*:`/container/`has-*` forms into `rules` over the v0.x `Atom` conjunction), and every longhand without a v0.x slot (the gradient and shadow preset rows, the slot-less families, the unmappable `has-*` fragments) reported as a typed `PlanNote` with a stable code and help text
- *(compile)* `lower::suffix_of` joins the `pub(crate)` surface beside `kw_annotations` (the plan arm's per-prop spellings read the accepted suffix)

### 🧪 Testing

- *(compile)* Pin the plan bridge end to end (`tests/plan_bridge.rs`): the DIFFERENTIAL oracle — every battery class's plan must agree slot-for-slot with the v0.x `Resolver` fed the same theme CSS (the P1 exit criterion's plan-level form), over the v4-default theme battery (spacing/margins/sizing/radii/colours/borders/typography/layout/position/ring/outline/transforms/grid-numeric) and the shadcn token battery, the multi-class cascade strings, the bucket-routing table, the same-condition layer merge, the typed notes, the theme-less diagnostics parity with `compile`, the customs pass-through and the determinism contract
- *(bench)* Measure the plan arm (`benches/plan_bridge.rs`: the themed shadcn Button core walk, the theme-less boundary path, the single-utility floor)

### 📚 Documentation

- *(readme)* Align the badge and status to v0.18.0 (the drift the 0.18.0 release left) and file the plan bridge
- *(architecture)* Mark the five-arm `Target` surface in the workspace manifest comment and the CI member count (the xtask join the 14-member line predated)

### 💼 Other

- *(core)* Give the five bare `unreachable!()` panics in the resolver their guard reasons (the repo's own convention — every other site carries one)

## [0.18.0] - 2026-11-11

### 🚀 Features

- *(data)* The `codegen::GeneratedArtifact` seam — the generator story's one trait every committed derived artifact implements, with `RootsIndex` (the seeded-FNV perfect-hash over the rows' distinct roots) and `RegistryDocs` (the markdown audit page) as the landing artifacts
- *(data)* The generated roots index wired into the registry read side (`lookup`/`lookup_detailed` walk the phf probe; the pre-index linear scan stays as the documented `lookup_linear` differential oracle)
- *(xtask)* The generator driver (`cargo xtask gen-tables`/`verify-tables`/`list`, the `run(args, out, err) -> code` contract, `--root` for test-only temp roots) and the `cargo xtask` alias
- *(compile)* The attribute-probe interp surface (plan §3.5's typed state bits): `AttrProbe` (the one question a host answers), the built-in `AttrState` bit table, and the `interp_probes`/`interp_theme_probes` entry points — probe layers fire, answered misses stay CSS-quiet, the descendant probes stay the diagnosed boundary

### 📚 Documentation

- *(readme)* Align the badge and status to v0.17.0 (the drift the 0.16/0.17 releases left) and file the generator story and the probe surface
- *(architecture)* Mark the xtask generator story landed (the P2 row, the remaining-slices line, the P1 status paragraph)
- *(changelog)* Give the shipped 0.16.0 and 0.17.0 their release headers (the items sat in Unreleased since the tags)
- *(data)* The `codegen` module docs (the seam, the determinism contract, the artifacts that grow on it) and the registry read side's index/oracle story
- *(compile)* The `interp` module docs (the probe seam's firing semantics, the quiet-miss rule, the has- diagnosed boundary) and the note's two unqueried cases

### 🧪 Testing

- *(data)* Pin the generator contracts (`tests/codegen.rs`: the seam list, the committed-bytes drift pins, the index's partition/reachability/load-factor invariants, the differential battery over roots/composites/adversarial misses and a deterministic 4000-body fuzz, the typed-miss parity)
- *(data)* Battery the index against the linear oracle in the lookup bench (hit, multi-dash hit, shape-rejected miss)
- *(xtask)* Pin the driver's exit-code contract and the write/verify cycle against a temp root (`tests/driver.rs`)
- *(compile)* Pin the probe seam end to end (`tests/probes.rs`: the firing semantics, the CSS-quiet miss, the sourceless back-compat notes, the has- boundary, the cascade order, the themed combo, the custom-alias composition, the ergonomic half, purity)
- *(bench)* Measure the probe seam (`benches/interp.rs`: themed+probes Button, mixed probe program, the sourceless note path)

### 💼 Other

- *(data)* Demonstrate the generator story (`examples/codegen_artifacts.rs`)
- *(compile)* Demonstrate the probe seam (`examples/probes.rs`)
- *(ci)* Fix the push branch filter (the `ain]` typo that disabled the push-to-main gate) and gate `xtask` in the 1.88 MSRV job

## [0.17.0] - 2026-11-11

### 🚀 Features

- *(compile)* The attribute-probe variant fold — the `data-*`/`aria-*`/`has-*` spellings onto the typed `Cond` surface (`Cond::Data`/`Cond::Aria`/`Cond::Has` with interned payloads, the v0.x oracle's `has_atom`/`parse_attr`/`attr_test` grammar, the parent-relative `group-has-*`/`peer-has-*` on the parent tier)
- *(ir)* The attribute-probe `Cond` variants (`Data`/`Aria`/`Has` on the state tier, `GroupHas`/`PeerHas` on the parent tier, payload equality over text not pointers)

### 📚 Documentation

- *(compile)* The `variant` module docs (the attribute-probe families' fold grammar and the differential oracle) and the interp note's state-bits story
- *(ir)* The probe conds' payload/storage contracts on the `Cond` docs
- *(readme)* File the attribute-probe fold in the status note (v0.16.0) and the crate table
- *(architecture)* Mark the attribute-probe variant fold landed in the P2 row
- *(benchmarks)* File the attribute-probe-fold run numbers
- *(evidence)* File the attribute-probe-fold red-to-green record (M1 element-own probes, M2 parent-relative has-probes)

### 🧪 Testing

- *(compile)* Pin the attribute-probe contracts end to end (`tests/attr_variants.rs`: the fold's one-condition identities, the grammar's honest refusals, the corpus selector forms, the interp notes, the payload emission, the merge/custom generality, the parent-relative heads)
- *(compile)* Move the unsupported-variant pins to the new boundary (`tests/compile.rs`, `tests/cn.rs`: the bare `has-[x]` form) and match the attribute note in the interp finder (`tests/interp.rs`)
- *(bench)* Measure the attribute-probe fold (`benches/attr_variants.rs`)

### 💼 Other

- *(compile)* Demonstrate the attribute-probe fold (`examples/attr_variants.rs`)

## [0.16.0] - 2026-11-11

### 🚀 Features

- *(compile)* The rust-tokens emit arm module (`emit_rust`/`emit_rust_theme` and the custom-seam siblings: one embeddable `Layer::new` expression per lowered class, keyword slots resolved to their row text, token slots to their table names)
- *(compile)* Arm `Target::Rust` — the plan §3.1 emit (a) form, the `Target` surface's last arm
- *(compile)* The row-table keyword annotation pass (`lower::kw_annotations`) and the shared `val_literal` float contract

### 📚 Documentation

- *(compile)* The `rust` module docs (the in-pipeline rationale, the compiled-shape doc test) and the statics/interp cross-references
- *(readme)* Align the badge and status to 0.15.0 (the drift the grammar release left) and name the rust-tokens arm
- *(architecture)* Mark the rust-tokens emit arm landed
- *(changelog)* File the rust-tokens arm
- *(benchmarks)* File the rust-tokens-arm run numbers
- *(evidence)* File the rust-tokens red-to-green record (M2: has-/data-/aria- folding identified and deferred)

### 🧪 Testing

- *(compile)* Pin the rust-tokens contracts end to end (`tests/rust_tokens.rs`: the verbatim fragment and sheet, determinism, the keyword/composite/bare annotations, the token-name annotations, the 22-cond totality battery, the diagnostics-equal battery, the custom differential)
- *(bench)* Measure the rust-tokens emit (`benches/rust_tokens.rs`)

### 💼 Other

- *(compile)* Demonstrate the rust-tokens arm (`examples/rust_tokens.rs`)

## [0.15.0] - 2026-11-10

### 🚀 Features

- *(syntax)* Export the v4 type-hint table
- *(syntax)* Re-export the hint table from the crate root
- *(compile)* The arbitrary-value unit grammar module
- *(compile)* Lower the arbitrary groups in both value forms
- *(compile)* Thread the arbitrary group through the ir walk
- *(compile)* Thread the arbitrary group through the css walk
- *(bench)* Measure the arbitrary-value grammar
- *(compile)* Demonstrate the arbitrary-value grammar

### 📚 Documentation

- *(data)* Name the landed unit grammar in the shape contract
- *(readme)* Align the badge, status and table to the grammar
- *(architecture)* Mark the arbitrary-value unit grammar landed
- *(changelog)* File the grammar and the drift consolidation
- *(benchmarks)* File the arbitrary-grammar run numbers
- *(evidence)* File the arbitrary-values red-to-green record

### 🧪 Testing

- *(compile)* Pin the arbitrary-value contracts end to end
- *(compile)* Flip the w-[32px] pin to the lowering contract

### ⚙️ Miscellaneous Tasks

- *(release)* V0.15.0

### 💼 Other

- *(compile)* The colour-parser dep and the grammar bench
- *(deps)* Lock the aeolus-core edge for the compile seam
# Changelog

All notable changes to Aeolus are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.14.0] - 2026-11-10

### 🚀 Features

- *(theme)* The mode-aware token rgba resolve primitive
- *(compile)* The program interpreter seam module
- *(compile)* The static-data emit arm module
- *(compile)* Wire the interp and static arms into the compile seam
- *(bench)* Measure the interpreter arm paths
- *(bench)* Measure the static-data emit
- *(compile)* Demonstrate the interpreter arm
- *(compile)* Demonstrate the static-data emit

### 📚 Documentation

- *(readme)* Align the status, crate table, examples and counts
- *(architecture)* Mark the interp and static-data arms landed
- *(changelog)* File the emit arms and the drift fixes
- *(evidence)* File the emit-arms red-to-green record
- *(benchmarks)* File the emit-arms run numbers

### 🧪 Testing

- *(theme)* Pin the concrete rgba resolve and its refusals
- *(compile)* Pin the interpreter contracts end to end
- *(compile)* Pin the static-data emit contracts end to end

### ⚙️ Miscellaneous Tasks

- *(build)* Repair the stale P2 slice comment in the manifest
- *(release)* V0.14.0

### 💼 Other

- *(compile)* Bench the interp and static arms in the manifest
## [0.13.0] - 2026-11-10

### 🚀 Features

- *(compile)* The dyn-cache store seam and bounded two-gen cache
- *(compile)* Wire the dyn cache into the compile seam
- *(bench)* Measure the dyn cache hit and miss paths
- *(compile)* Demonstrate the dyn cache
- *(egui)* Demonstrate the frame and widget-visuals lowering
- *(floem)* Demonstrate the style lowering
- *(iced)* Demonstrate the container and button styles
- *(slint)* Demonstrate the snippet generation

### 📚 Documentation

- *(compile)* Align the custom-seam header to the token form
- *(readme)* Align the crate table and examples to the dyn cache
- *(architecture)* Mark the dyn-cache seam landed
- *(changelog)* File the dyn cache and the drift fixes
- *(benchmarks)* File the dyn-cache bench numbers
- *(evidence)* File the dyn-cache red-to-green record

### 🧪 Testing

- *(compile)* Pin the dyn-cache contracts end to end

### ⚙️ Miscellaneous Tasks

- *(release)* V0.13.0

### 💼 Other

- *(compile)* Bench the dyn cache in the manifest
## [0.12.0] - 2026-11-10

### 🚀 Features

- *(compile)* Wire the ir-level cn merge
- *(bench)* Measure the cn merge
- *(examples)* Demonstrate the ir-level cn merge
- *(compile)* Wire the css target arm
- *(bench)* Measure the css target
- *(examples)* Demonstrate the css target
- *(compile)* The custom-authoring seam module
- *(compile)* Thread the ClassifyVariant seam through the fold
- *(compile)* Wire the customs read side and the with entry points
- *(compile)* Bench the custom seam cold paths
- *(compile)* Demonstrate the custom-authoring seam

### 🐛 Bug Fixes

- *(theme)* Give the pair overflow its typed refusal
- *(syntax)* Restore the typed lex-failure import
- *(syntax)* Import VERSION in the inline version-pin test
- *(ir)* Count the base rule's own re-sets in base_conflicts
- *(ir)* Repair the crate-tests doc link under -D warnings
- *(theme)* Pair a token only on a real dark override
- *(compile)* The value pass alpha shadow and the token form
- *(compile)* Finish the css walk refactor and the render contract
- *(syntax)* Rustfmt the error display and its doc link
- *(compile)* Drop the raw-string hashes clippy denies

### 📚 Documentation

- Align every surface to the cn merge and sweep the drift
- Align every surface to the css target and the drift
- *(compile)* Align the pipeline example header to the slices
- *(changelog)* File the custom seam and the breakage record
- *(readme)* Align the crate table and status to slice 5
- *(architecture)* Mark the custom-authoring seam landed
- *(benchmarks)* File the v1 compile-seam bench numbers
- *(evidence)* File the custom-seam red-to-green record

### 🚜 Refactor

- *(compile)* Type the variant-fold miss

### 🎨 Styling

- *(syntax)* Apply rustfmt to the lexer example
- *(theme)* Apply rustfmt to the tiers example
- *(compile)* Apply rustfmt to the css bench
- *(compile)* Apply rustfmt to the cn example
- *(compile)* Apply rustfmt to the theme example

### 🧪 Testing

- *(compile)* Pin the ir-level cn contracts
- *(syntax)* Pin the typed degraded-lexer channel
- *(compile)* Pin the css target contracts
- *(compile)* Pin the custom seam contracts end to end
- *(compile)* Repair the diagnostic accessor and variant pins
- *(compile)* Pin the probe heads and the dark companion fixtures
- *(compile)* Repair the theme-seam accessor and config pins
- *(compile)* Drop the unused import under -D warnings
- *(syntax)* Drop the unused LexFailure import
- *(theme)* Rustfmt the suite and pin the /0 clamp

### ⚙️ Miscellaneous Tasks

- Gate the cli's 1.88 msrv claim
- Pr body
- *(release)* V0.12.0
- Gitignore
- Gate the v1 crates' 1.88 msrv and sweep the header drift

## [0.12.0] — 2026-11-10 — the P2 slice 3: the IR-level `cn()` merge

Release summary (the git-cliff block, filed in place): features — the
compile seam hosts the ir-level cn merge, the merge bench, the consumer
example; fixes — the theme pair overflow's typed refusal; docs — the
cn-merge surface alignment and the drift sweep; refactors — the typed
variant-fold miss; tests — the ir-level cn contract pins; misc — the
CLI's 1.88 MSRV gate, the pr body, the release itself.

### The P2 slice 3 — the IR-level `cn()` merge

The third slice of phase P2 (architecture plan §3.2/§3.3): the compile
seam hosts the `cn()` engine at IR level — `aeolus_compile::cn(&[&Compiled])`
merges already-compiled programs RIGHTMOST WINS per longhand. The
mechanics are the longhand IR's own algebra ("mask algebra + copy",
§3.3): same-condition layers fold by `Cond` equality, declarations
concatenate left-then-right, the per-longhand LAST occurrence wins, and
the result comes out conflict-free (`base_conflicts()` empty),
specificity-sorted and interned as the dynamic form. The differential
contract is test-pinned: merging resolves exactly what compiling the
concatenated string resolves to, and the declarations the merge removes
are precisely the overlap `Program::base_conflicts()` names on the raw
cascade (which stays byte-identical to v0.11.0 — the MERGE is what
dedupes). The audited `tw_merge` conflict table stays at the string
level (`aeolus_core::cn`, the CLI's `cn` command); the longhand
fan-outs ARE the groups at this level. The merge is idempotent and
associative (pinned), diagnostics concatenate in input order with
their original spans, and theme-tier token values merge with the
rightmost alpha riding the `Tok` slot.

- **Added (compile):** `cn(&[&Compiled]) -> Result<Compiled,
  CompileError>` with `tracing` spans on the merge and the new
  `CompileError::NoInputs` boundary (the empty slice is a caller bug;
  the `miette` feature wires its code + help). `examples/cn_merge.rs`
  demonstrates the whole story (raw cascade vs merge, the conflict
  bits, diagnostics passthrough, the theme-tier merge), and the bench
  family gains `cn_button_surface_with_override_cold` (divan, stable).
- **Changed (compile, refactor):** the variant-fold miss is TYPED —
  `variant::fold` returns `Result<Cond, VariantMiss>` (a struct
  carrying the refusing segment) instead of a bare `String`. The
  rendered diagnostics are unchanged (pinned by the existing variant
  suites); the seam now reports through the same typed shape every
  other stage uses.
- **Changed (theme, fix):** the T1 budget refusal is honest —
  `ThemeError::PairOverflow { pairs }` is its own variant instead of a
  fabricated `ParseError` smuggled through the parse arm (the crate's
  own contract: parse errors are carried verbatim, never re-wrapped).
  Always-on `Display` + source-less `Error` chain; the `miette`
  feature carries `aeolus::theme::pair_overflow` code + help.
- **Changed (docs):** the drift sweep — the README's version badge and
  status paragraph still said v0.10.0 after the v0.11.0 release (the
  release commit touches the manifests only, the badge follows the
  next docs pass); the CHANGELOG carried the git-cliff `[0.11.0]`
  block ABOVE the file title plus an `[Unreleased]` still describing
  the already-released theme pass — consolidated into one `[0.11.0]`
  section, the new `[Unreleased]` is this change; the hot-path bullet's
  test counts were re-counted from the tree by `#[test]` attribute
  (544; 552 with the instrumentation features — the previous pass
  printed 535/542 while the tree carried 531/538); the consumer-examples
  block lists the new cn example (twelve runnable examples); and the
  README's "1.88 gates core/macro/cli" claim has its CI job back (the
  `msrv-core` job now checks `-p aeolus-cli`, which it had silently
  stopped covering).

## [0.11.0] — 2026-11-10 — the P2 theme pass: the theme half of the compile signature

Release summary (the git-cliff block, filed in place): features — the
tiered token table over the audited theme parser, the compile seam's
theme half, the theme tiers + theme seam consumer examples, the
token-table bench; fixes — the compile bench builds without a `Default`
`Compiled`; docs — the theme-pass RED-to-GREEN evidence record and the
drift sweep; tests — the tier contracts + alpha identity pins, the
theme-seam + non-finite pins.

The second slice of phase P2 (architecture plan §3.2/§3.4): `aeolus-theme`,
the CSS-first theme pass — the SAME `@theme`/`:root`/`.dark` file a web
Tailwind v4 project uses resolves into a tiered `TokenTable`, and
`compile_theme(&str, &TokenTable, &Config)` lowers colour tokens and
`/opacity` modifiers through it. This resolves the boundary the compile
seam discovered and pinned (the 270-entry colour table does not fit the
IR's u8 `Val::Kw` slot): tokens ride the u16 `Val::Tok` slot, tiered
exactly per the plan's §3.4 table — T0 literals, T1 light/dark const
pairs, T2 runtime slots with the audited premultiplied `/N` identity
(one multiply). The theme-less `compile` keeps the v0.10.0 contract
byte-for-byte: colour tokens and modifiers stay typed boundaries.

- **Added (theme):** the crate. `ThemeSource::from_css`/`shadcn`/
  `shadcn_dark` parse through the audited v0.x reader (the interim
  oracle, `dark_variant` propagating `@theme inline` chains into the
  dark variant); `TokenTable::baked`/`runtime` snapshot the colours with
  sorted-name stable ids (they ride compiled `Program`s — a contract,
  test-pinned); `resolve(id, alpha)` emits the tier's `Val` and refuses
  typed on unknown ids, non-finite alphas and an alpha on a T1 pair
  slot. Two OFF-by-default instrumentation features land with it, the
  workspace's pattern: `tracing` (build/resolve spans) and `miette`
  (code + help on `ThemeError`/`ResolveError`). Benched
  (`benches/tokens.rs`: the build, the runtime and baked hot paths) and
  demonstrated (`examples/theme_tiers.rs`).
- **Added (compile):** the theme half. `compile_theme` takes the plan
signature's second argument; the value pass routes the oversized colour
keyword table through the supplied `TokenTable` (tier-aware), the
`/opacity` modifier normalizes once per candidate (`"90"` → 0.9,
`"[0.37]"` → 0.37) and rides COLOUR-form decls only — a modifier on a
non-colour lowering or on a baked dark pair stays the typed
`UnsupportedModifier` boundary, never a half-applied alpha. Two new
reasons join the diagnostic set (`not-a-token`, `non-finite`); the
theme-less path is byte-identical to v0.10.0 (pinned).
- **Fixed (bench):** `benches/compile.rs` could not build — four
`unwrap_or_default()` calls on `Result<Compiled, _>` while `Compiled`
has no `Default` (an "empty compile" no string produced is not a value
the seam fabricates). The bodies `expect` like the examples do;
`cargo test --benches` builds again.
- **Fixed (compile):** the f32 grammar's non-finite spellings
(`p-nan`, `z-inf`, `w-1e40`) are legal PARSES and lowered SILENTLY
into the IR as `NaN`/`inf` values — a no-silent-failures violation.
The value pass now refuses them typed (`REASON_NON_FINITE`) on every
numeric arm (spacing, the negated forms, fractions, percents, numbers
and the px bake's overflow), with the rest of the string still
compiling; test-pinned end to end.
- **Added (examples):** the theme seam's consumer demonstrations —
`crates/aeolus-theme/examples/theme_tiers.rs` (a source → tiers end
to end) and `crates/aeolus-compile/examples/theme_seam.rs` (the same
string theme-less vs. runtime-tier vs. baked-tier).
- **Changed (docs):** the drift pass — the README's fuzz bullet said
CI fuzzes "four of them" while CI sweeps all six (fixed in v0.10.0);
CONTRIBUTING's fuzz smoke listed 4 of the 6 targets; the CHANGELOG
carried TWO `[0.10.0]` headers (the git-cliff block plus the prose
section) and an `[Unreleased]` still describing the already-released
compile seam — consolidated here; the architecture plan's P2 status
and §8 row record the theme pass's landed reality; the crate table,
the examples block and the test counts follow the tree.

## [0.10.0] — 2026-11-10 — the P1 consumer handoff, part two + the P2 compile seam

### The P2 seam — the compile pipeline's first slice

The first slice of phase P2 (architecture plan §3.1/§3.2): `aeolus-compile`,
the pure function `compile(&str, &Config) -> Result<Compiled, CompileError>`
that runs class strings through the three P1 crates end to end — the
zero-copy lexer, the typed registry read side, the longhand value pass —
and emits cascade-sorted `Program`s with the IR's masks as data. Everything
a class does not lower to (importance, modifiers, colour tokens, arbitrary
values, untyped variants) comes back as a spanned, coded, helped
`Diagnostic` — never a silent drop.

- **Added (compile):** the seam. The registry semantics are pinned at
  `REGISTRY_VERSION = "1"` by a const assert (the value pass audits the
  rows' meaning); the spacing config knob lands with it (rem by default,
  the px compat config baking against `rem_px`, plan §3.3); the
  flat-`Cond` variant fold (dominant condition, source order breaks
  ties — cond.rs's own contract, now implemented); benched
  (`benches/compile.rs`: single utility, variant stack, the shadcn Button
  surface, the typed miss) and demonstrated
  (`examples/compile_pipeline.rs`). Two OFF-by-default instrumentation
  features land with it, exactly the CLI's v0.9.1 / registry's P1
  pattern: `tracing` (spans on the pipeline stages) and `miette` (code +
  help on `CompileError`; the always-on `code()`/`help()` strings live
  on `DiagnosticKind`).
- **Discovered and pinned (ir):** the seam surfaced the first concrete IR
  budget: the colour keyword table is 270 entries and `Val::Kw` is a u8
  slot — colour tokens lower through the theme pass's `TokenTable`
  (plan §3.4 T2), typed and reported, never truncated.
- **Removed (syntax):** `Candidate::signed_body` — dead code (zero
  callers across the workspace) whose doc contradicted its own
  implementation: it returned the sign-STRIPPED body while claiming to
  restore the sign; the sign rides the `negative` flag by design
  (`Candidate::negative`).
- **Fixed (v1):** the `VERSION` consts in `aeolus-syntax` and
  `aeolus-ir` derive from the manifests now (`env!("CARGO_PKG_VERSION")`)
  — the hardcoded `"0.1.0"` drifted three releases behind the
  workspace; each crate pins the derivation with a test.
- **Changed (docs):** the drift pass — the README's version badge says
  0.10.0 (it still said 0.7.0), the status paragraph moved to v0.10.0
  with the P2 seam noted, the consumer-examples block lists all nine
  examples (it still listed four), the test count is the real one (509;
  513 with the instrumentation features), the crate table carries
  `aeolus-compile`, the architecture plan records the P2 seam's landed
  reality beside P1's, and the rustdocs stopped contradicting the code
  (`prop.rs`'s "96 longhands" → 110; `hash_classes`' "used by both cache
  and interner" → what actually uses it; the `tw!` span-preservation
  promise → the honest nightly-only ceiling).

### The P1 consumer handoff, part two: typed registry seam, examples, drift alignment

- **Fixed (build):** the v0.9.0 release bump left `[workspace.dependencies]`'s
  intra-workspace pins at 0.8.0 while `[workspace.package]` moved to 0.9.0
  — cargo enforces the version requirement against path dependencies, so
  the workspace stopped resolving (`failed to select a version for the
  requirement aeolus-core = "^0.8.0"`). The pins are back in lockstep,
  with a manifest comment explaining the contract and the release
  process (ship.sh) updating both blocks together from now on.
- **Fixed (ci):** the fuzz sweep runs all SIX fuzz targets —
  `parse_variant` and `probe_eval` were wired in `fuzz/Cargo.toml` but
  absent from the CI loop.
- **Added (data):** `registry::lookup_detailed` + `LookupError` — the
  typed "did you mean" seed the crate docs promise: identical rows to
  `lookup` on a hit (pinned), and on a miss a TYPED failure mode —
  `UnknownRoot` vs `ShapeRejected { root, tried }` (the deduplicated
  value-shape labels that were tried, `AnyOf` arms flattened). Two
  OFF-by-default instrumentation features land with it, exactly the
  CLI's v0.9.1 pattern: `tracing` (debug spans on the read paths) and
  `miette` (`Diagnostic` code + help text). The default graph stays
  dependency-light (aeolus-ir only). Benched (`benches/lookup.rs`:
  silent hit, detailed hit, both misses).
- **Added (examples):** consumer demonstrations for the crates that had
  none — `crates/aeolus-syntax/examples/lex_candidates.rs` (the
  zero-copy lexer's candidates + spans, and a typed miss),
  `crates/aeolus-data/examples/registry_lookup.rs` (rows, masks,
  conflict groups, the typed failures), and
  `crates/aeolus-cli/examples/theme_spec.rs` (the `--theme` seam as a
  library: `load_theme`, the typed `ThemeSpecError` contract).
- **Changed (hygiene):** the guarded unwraps on reachable public parse
  paths are gone — `eval_calc`'s shunting-yard pop rides its own guard,
  the bare-operator token is a let-else returning a parse error, and
  `Color::parse`'s 3/4-digit hex doubling has an explicit error arm
  (CONTRIBUTING §2: no unwraps on reachable public-API paths). A new
  test pins the calc()-edge typed-error contract (operator needs an
  operand / ends with an operator — errors, never panics).
- **Changed (docs):** the drift pass — the README's status paragraph
  says v0.9.0 (it still said v0.7.0); the CHANGELOG cuts `[0.9.0]` and
  `[0.8.0]` (the merged v0.8 + v0.9 story had lived under
  `[Unreleased]` since its tags); the P1 section's counts are the real
  ones (`Prop` = 110 longhands, the registry = 178 rows — was "95" and
  "~150"); the V1 architecture plan's P1 status and §8 row record the
  landed reality (178 rows in-tree, the xtask generator explicitly P2).

### V1 phase P1 — the data-driven core lands

The first slice of the zero-runtime track (`docs/AEOLUS_V1_ARCHITECTURE.md`
§8, phase P1) is now in-tree, git-only (`publish = false`), zero new
external dependencies:

- **Added `aeolus-ir`:** the longhand IR — `#[repr(u16)] Prop` (110
  longhands, discriminant order = cascade/sort order), the `PropMask`
  u128 bitset (`union`/`intersect`/`difference`/`covers` — the v0.8
  per-edge set-mask idea generalized to the whole surface:
  `p-4 ⊕ px-8` and layer dominance are bit operations), the 12-byte
  `Copy` `Val`, `Cond`/`Bp` layer conditions with the v4-cascade
  specificity ranks, and `Decl`/`Layer`/`Program` assembly. Law-tested
  (`tests/ir_laws.rs`: mask algebra idempotence/absorption/disjoint
  TRBL edges, value size budget, stable specificity sort, the
  base-conflict projection), benched (`benches/mask.rs`), and
  demonstrated for consumers (`examples/mask_algebra.rs`).
- **Added `aeolus-data`:** the utility registry as data — one auditable
  row per utility pattern (178 rows: root, `ValueKind` shape, longhand
  targets, `tw_merge` conflict group, v4 source ref, negation flag),
  the committed v4 scale tables (text sizes, weights, radii, tracking,
  leading, container widths, z, opacity, border/outline steps, the
  full default colour palette + shadcn tokens), longest-prefix
  disambiguation lookup (`rounded-tl-lg` → `rounded-tl`, `min-w-0` →
  `min-w`, the `text-*` font-size/colour/align triple split), and
  `generate_docs()` proving the one-source-of-truth claim. The registry
  tests cross-check every group name against the v0.x
  `collision_group` oracle AND sweep the golden corpus: every
  oracle-grouped corpus class must find a registry row carrying that
  group.
- **Added `aeolus-syntax`:** the zero-copy lexer — class strings into
  borrowed `Candidate`s with byte spans (variant segmentation,
  negation, importance, body, arbitrary form/hint/value/property flag,
  raw `/modifier`), zero dependencies, typed `LexError` spans, honest
  per-class degradation. The differential harness pins the lexer to
  the v0.x `aeolus-core::parser` oracle — the failure/Success split,
  variant segmentation (canonical space), negation, importance, body,
  arbitrary and modifier — over a 59-string tricky battery AND every
  golden-corpus class; the one documented divergence is the v0.x
  inline caps (8 variants / 10 elements), which the lexer
  deliberately drops. Benched (`benches/lex.rs`: bare, stacked,
  calc-arbitrary, whole-string sweep).
- **Workspace:** the three crates join `[workspace.members]` (git-only
  while the pipeline assembles, plan §3.2); `aeolus-ir` joins
  `[workspace.dependencies]` for `aeolus-data`. Pedantic clippy via the
  workspace `[lints]` table, zero `#[allow]` in the new crates.

P2 (the `aeolus-compile` pipeline), P3 (GPUI codegen) and `xtask`
remain planned — nothing in the v0.x surface changes: every v0.9.1
test still passes untouched.

### v0.9.1 — consumer handoff

- **Added:** runnable consumer examples — `crates/aeolus-core/examples/`
  (`resolve`, `plan_algebra`, `cn_merge`) and
  `crates/aeolus-xilem/examples/styled_label` (plan → native Masonry
  `PropertyStack` + the `.tw()` view sugar, headlessly). The README's
  Usage section now lists them.
- **Added (opt-in):** the `aeolus-cli` `--theme` seam returns the typed
  `ThemeSpecError` (unknown spec / unreadable file / unparseable CSS) —
  Display text unchanged, so the `run()` stderr contract is preserved.
  Two OFF-by-default instrumentation features land with it: `miette`
  (structured `Diagnostic` help text on the typed errors) and `tracing`
  (debug spans around command dispatch and the per-file `check` loop);
  the DEFAULT build keeps the zero-dependency discipline.
- **Changed (docs):** the v0.9-in-tree reality is now stated everywhere:
  the README's corpus count is 193 in both places (the emitter bullet
  said 157), the Status deferred-items list no longer claims arbitrary
  grid templates are still reported (they LANDED with v0.9), the
  Learn-more section links `AEOLUS_V0.8.md`/`AEOLUS_V0.9.md`/the V1
  architecture plan (and fixes the `docs/release-notes/…` link targets),
  the CLI rustdoc's plan-of-record reference points to `AEOLUS_V0.7.md`
  §3 (was `AEOLUS_V0.9.md`), and the V1 architecture doc's evidence
  header records the re-verification of its F-findings at the v0.9 tree.
- **Changed (hygiene):** `muda-stub`'s crate-level `#![allow(dead_code)]`
  and `#![allow(clippy::new_without_default)]` moved into its own
  `[lints]` table with rationale — the workspace-wide "zero `#[allow]`
  attributes in `crates/*/src`" grep claim holds again.
- **Benches:** `theme_spec_load_builtin` and `theme_spec_error_render`
  in the CLI's `benches/pipeline.rs` (the `--theme` seam, once per
  process).

## [0.9.0] — 2026-11-09 — v0.8 + v0.9: per-edge set-mask, variant engine & test pyramid (merged)

### Fixed

- **Per-edge cascade semantics** (v0.8 §1, `AEOLUS_V0.8.md`): the four
  per-edge families (`Padding`/`Margin`, `BorderWidth`, `CornerRadius`)
  now carry a set-mask (`EdgeSet`/`CornerSet`) recording WHICH edges the
  class string addressed. The v0.7 finalize materialized unset edges as
  `0.0`, so `p-4 hover:px-8` lost `padding-top: 16px` on hover (every
  backend applied the layer's slot wholesale) and plan algebra composed
  `p-4 ⊕ px-8` to `{0, 32, 0, 32}`. The mask rides through `finalize`,
  `ResolvedStyle::overlay` (EDGE-WISE merge for the four families —
  `p-4 ⊕ px-8` → `{16, 32, 16, 32}`), the probe flatten
  (`StylePlan::resolve`), the new
  `ResolvedStyle::materialize_unset_edges_from(&base)` (xilem stack
  entries and slint state blocks materialize before lowering), floem's
  per-field application (gated by the mask), and iced/egui's overlay
  bookkeeping (`padding_set`, `corner_set`, margin sets).
- **The test-pyramid property suite caught four real defects** (all
  RED → GREEN, evidence in `reports/10-tdd-evidence/12-test-pyramid.md`):
  - `diff` collapsed the text family onto one bitset bit — a
    `FontWeight`-only change was reported as `Text(FontSize)` or missed
    entirely. Every text property now has its own bit
    (`Slot::Text(p) => 18 + p.index()`).
  - Plan snapshots silently dropped the grid templates
    (`template_cols/rows`, `auto_rows/cols`), `Layout.justify_items` and
    `Extras.container_type`. All four serialize strictly now (track /
    repeat components, sizing keywords, named containers).
### Changed

- **Snapshot schema v2** (`PLAN_SNAPSHOT_VERSION` 1 → 2): the masks are
  part of the artifact — `padding`/`border_width`/`margin` carry a
  required `"set"` object, `corner_radius` a corner `"set"`. v1
  documents fail with `SnapshotError::UnsupportedVersion(1)` (loudly, by
  design); the strict reader rejects malformed masks with dotted paths
  (`$.base.padding.set.left`).
- **Snapshot schema v3** (`PLAN_SNAPSHOT_VERSION` 2 → 3, the
  variant-engine merge): grid track templates
  (`template_cols`/`template_rows`/`auto_rows`/`auto_cols`),
  `container_type`, `layout.justify_items`, the `pseudo` test kind and
  full `has` test objects (`Atom::Has(Test)`) serialize strictly — a
  strict superset of the v2 set-mask payload; old readers fail loudly
  instead of guessing (no silent drops, the artifact rule).
- **Pedantic clippy, workspace-wide**: `[workspace.lints.clippy]`
  arms `pedantic = warn` with per-lint exceptions (each with its
  rationale, in the root manifest); every member opts in via
  `[lints] workspace = true`; ZERO `#[allow]` attributes remain in the
  workspace crates (the two pre-existing clippy allows were converted).

### Added

- `EdgeSet`/`CornerSet` in the IR and the public API;
  `ResolvedStyle::materialize_unset_edges_from`.
- `tests/per_edge_layers.rs` (11 tests) pinning finalize masks, edge-wise
  composition (base + layers + corners + margins/borders), materialize,
  diff exactness, idempotence, and the v2 snapshot round-trip/strictness;
  per-backend partial-edge tests in all five lowering suites.
- `plan_overlay_partial_edges` bench (~318 ns median — the mask walk is
  free against the whole-slot overlay).
- `reports/10-tdd-evidence/10-per-edge-set-mask.md` (RED→GREEN evidence,
  including the stashed-impl backend runs and what the cycle caught).
- **Unit suites for the hollow modules** (parser, plan, eval, selector,
  cond, ast, caps — 33 new tests): the cascade band table, the
  named-variant table, the selector boundary, `supports()` routing,
  per-slot overlay, per-property diff, `in,max)` windows and the
  longest-suffix group labels are pinned at the type level.
- **Plan-algebra property laws** (`tests/pyramid_properties.rs`,
  proptest, 256 cases/law): overlay idempotence, change containment and
  snapshot round-trip identity, generated through the REAL pipeline
  (class strings → resolver → plan).
- **Process-level CLI e2e** (`crates/aeolus-cli/tests/process.rs`, 9
  tests): the real binary, real exit codes, byte-determinism, stdout-
  is-artifact / stderr-is-diagnostics discipline, the 0/1/2 exit
  contract.
- **`benches/snapshot.rs`**: the artifact path and the plan overlay on
  the rich Button plan — report 08.

## [0.8.0] — 2026-11-09

The per-edge set-mask slice of the merged story above (the two releases
shipped one merged narrative — see [0.9.0]; the tags cut at
`chore(release): v0.8.0` and `chore(release): v0.9.0`).

## [0.7.0] — 2026-11-08

The **oracle-fidelity** release (engine plan: `AEOLUS_V0.9.md`): a fidelity
audit resolved every shipped shadcn v4 class table against the v0.6
resolver and counted **40 warnings — every one a parser/IR gap**. v0.7
closes them: the same tables now produce **13 warnings, all the
documented v0.3 §1.3 Probe-model boundary** (selectors styling OTHER
elements), pinned by `crates/aeolus-shadcn/tests/oracle_fidelity.rs`.

### Added

- **Pseudo-element variants** (v0.9 §1): `selection:`, `placeholder:`,
  `file:` parse as `NamedVariant::Pseudo` and classify onto the typed
  `Test::Pseudo(PseudoElement)` condition — NOT runtime states but the
  element's own `::selection`/`::placeholder`/`::file-selector-button`
  renderings, answered via `Probe::test` (default "no": the rule never
  fires where the rendering does not exist). The shadcn Input table's
  ten unknown-variant warnings are gone; `tw!` accepts the three names.
- **`has-data-[…]:`/`has-aria-[…]:`/`has-<structural>:` descendants**
  (v0.9 §2): `Atom::Has` generalizes from `Has(State)` to `Has(Test)`;
  `Probe::descendant(&Test)` (default: state-only) keeps old trait
  impls compiling; the arbitrary-selector classifier maps `:has(...)`
  onto the same typed set. Combinator forms (`has-[>svg]`) keep their
  precise report.
- **Named container declarations** (v0.9 §3): `@container[/name]` —
  `Extras.container_type: Option<ContainerType>` (slot
  `ContainerType`, lane Geometric), emitter reproduces v4's
  `container-type: inline-size` + `container-name` verbatim.
- **Arbitrary grid templates + implicit tracks** (v0.9 §4):
  `GridTrack { min, max }` / `TrackSizing` / `TrackComponent` —
  `grid-cols-[0_1fr]` (bare `1fr` = `minmax(auto, 1fr)`, bare `0` = the
  raw length), `grid-rows-[auto_auto]`, `minmax()`, bounded
  `repeat(N, <track>)`, and the `auto-rows-*`/`auto-cols-*` keywords
  (`auto-rows-fr` is `minmax(0, 1fr)` — v4's blowout guard, pinned
  against the real compiler). Templates are CANONICAL over the v0.6
  numeric counts; Floem lowers everything NATIVELY
  (`GridTemplateComponent::Single/Repeat`, `grid_auto_rows/columns`).
- **`justify-items-*`** (v0.9 §5): `Layout.justify_items`
  (start/center/end/stretch/baseline, `normal` → stretch); Floem
  native, others report.
- **Plan algebra** (`crates/aeolus-core/src/plan.rs`):
  `StylePlan::overlay(&self, over: &StylePlan) -> StylePlan` composes two
  RESOLVED plans later-wins-per-slot — the IR-level mirror of `cn()`.
  Layers merge by `VariantCondition` and stay rank-ordered;
  `responsive`/`container` buckets overlay per breakpoint;
  `dark` overlays per slot; `group`/`peer` merge by id; `rules` merge by
  `Cond` equality (one rule per conjunction); the override's breakpoint
  table wins. Idempotent, and diff-integrated: `diff(&base.base,
  &merged.base)` reports exactly the override's slots, so backends keep
  their repaint-vs-relayout lanes across composition. Pinned by 8 tests
  (`tests/plan_algebra.rs`).
- **Canonical plan snapshots** (`crates/aeolus-core/src/snapshot.rs`):
  `StylePlan::to_snapshot_json()` — canonical, byte-stable, dependency-free
  JSON (fixed schema order, compact, shortest round-trip floats) — and
  `StylePlan::from_snapshot_json(&str)` — a STRICT reader: unknown fields,
  unknown enum names, duplicate keys, wrong `format`/`version` headers,
  truncation and trailing content are typed `SnapshotError`s with dotted
  paths (`$.base.extras.grid.cols`), never silent defaults. The JSON
  parser is hand-rolled (depth-limited, full escapes incl. surrogate
  pairs); non-finite floats encode as `"NaN"`/`"inf"`/`"-inf"` so the
  writer is total; a closed keyword table keeps
  `TextValue::Keyword(&'static str)` sound. Pinned by 9 tests
  (`tests/snapshot_roundtrip.rs`), including a 256-string deterministic
  random round-trip.
- **The headless CLI** (`crates/aeolus-cli`, binary `aeolus`,
  `publish = false`): `resolve` (the plan snapshot JSON is the default
  output; `--summary` for a human digest; warnings non-fatal on stderr),
  `css` (the verification emitter), `diff` (base-layer diff with
  cosmetic/geometric lanes, `--json`), `cn` (conflict merge), and `check`
  — the class-string linter: documented extractor heuristic (quoted
  strings kept when ≥ half the tokens resolve warning-free), warning
  findings, `cn()` conflict attribution (dropped token + winning token),
  `--fail-on unknown|unsupported|conflict|any|none` exit codes for CI,
  `--json` output. `--theme shadcn|shadcn-dark|default|<path>`. Zero
  dependencies beyond `aeolus-core`; the whole program is the library
  `run(args, out, err) -> code` (process-free testing). Pinned by 12
  tests (`crates/aeolus-cli/tests/cli.rs`).
- **The Slint backend** (`crates/aeolus-slint`): the fifth framework
  backend, chosen from the capability audit's candidate list (vizia's
  skia toolchain and GTK-family system dependencies being out of reach
  for headless sandboxes; Slint 1.18 is the most-used remaining
  pure-Rust framework). The lowering targets Slint's own value types
  and the generated-code path the audit prescribes for frameworks
  without runtime rule injection:
  - Native paint parity — the strongest of the five backends:
    gradients as real `Brush` values
    (`LinearGradientBrush`/`RadialGradientBrush::new_circle`/
    `ConicGradientBrush`; slint's angle convention matches CSS),
    per-corner radii (`border-top-left-radius` …), drop shadows WITH
    spread, and inset shadows via the `inner-shadow-*` surface.
  - `SlintStyle`/`SlintText` — typed element-property payload
    (`Brush` fills, `SlintLength` px/percent geometry, `clip`,
    `visible`, `opacity`, `mouse-cursor` via `BuiltInMouseCursor`).
  - Generated `.slint` code: `SlintStyle::snippet` (property
    bindings), `SlintText::snippet` (the `Text`-item block),
    `StyleBook::to_states_snippet` / `_focused` (`states [ … ]` gated
    on `TouchArea` `has-hover`/`pressed`/`enabled` and — with a named
    `FocusScope` — `has-focus`).
  - `StyleBook` + `SlintProbe`: the book keeps every state layer
    natively (hover/active/disabled/focus all resolve from probe
    flags), `resolve_at` keeps the full group/peer/responsive/
    container parity path, and the honest `LoweringReport` covers the
    rest (ring, layout structure, transforms, transitions, margins…).
  - 36 tests: lowering (19), parity (8), and snippet suites (9) whose
    generated `.slint` is compiled by slint's own interpreter
    headless — every emitted property name, keyword and gradient
    grammar is proven valid against Slint 1.18 itself. Clippy clean.
  - Licensing: the slint stack's `GPL-3.0-only` /
    `LicenseRef-Slint-Royalty-free-2.0` / `LicenseRef-Slint-Software-3.0`
    branches are allow-listed in `deny.toml` for the git-only,
    `publish = false` backend crate (slint's royalty-free grant covers
    use/redistribution; the workspace itself stays MIT OR Apache-2.0).
- **`aeolus-shadcn::slint`** — the Slint wrapper module mirroring the
  iced/egui shape: `button_book` and the style-only books
  (`badge_book`, `label_book`, the card family, `alert_book`) handing
  out `Arc<StyleBook>`s whose snippets assemble real interactive
  shadcn Buttons in `.slint`.
- **Benchmarks**: `plan_overlay_button`, `snapshot_write_button`,
  `snapshot_read_button`, `snapshot_roundtrip_button` (core
  `benches/hot.rs`); `resolve_input_cold` (the 32-class Input table
  with its pseudo-element variants) and `resolve_card_header_cold`
  (container + templates + has-data); `benches/snapshot.rs` (the
  artifact path); the CLI `pipeline.rs` suite.
- `reports/10-tdd-evidence/09-portable-plans.md` — the RED→GREEN evidence
  for plan algebra/snapshots/CLI, including what the cycle caught.
- `reports/10-tdd-evidence/11-variant-engine.md` — the RED→GREEN
  evidence for the variant & grid engine.

### Documented (discovered, pinned, deferred)

- **Per-edge layer semantics** (v0.7 §1.4): per-edge families materialize
  unset edges as 0.0 at finalize, so a `hover:px-8` layer carries
  `Padding { top: 0, right: 32, bottom: 0, left: 32 }` and the zeroed
  edges REPLACE the base's edges when the layer applies (CSS would keep
  `p-4`'s top/bottom). The per-edge set-mask fix ripples through every
  backend lowering — deferred (v0.7 §6), with the behavior pinned by
  `tests/plan_algebra.rs::layer_merge_by_condition` instead of unknown.
  (FIXED in the v0.8 milestone above — kept for the audit trail.)

### Changed

- **cn() gains the grid-template conflict groups** (v0.9 §4):
  `grid-template-columns/rows`, `grid-auto-rows/columns` and
  `container-type` — the later template evicts the earlier ACROSS the
  numeric/arbitrary boundary (`grid-cols-3` + `grid-cols-[0_1fr]` keeps
  only the arbitrary; previously BOTH were kept — a tw_merge-parity gap
  the milestone's own tests caught).
- The verification emitter reproduces v4's star forms for
  `:has()`/`:not()` inners (`:has(*[data-slot="x"])`,
  `:not(*[data-state="open"])` — the bare-attr `:not` shape was never
  corpus-pinned and was wrong); new declaration shapes for tracks,
  auto-tracks, justify-items, container declarations; verbatim
  arbitrary-value serialization (`0`, `repeat(3,minmax(0,1fr))`).
- **The golden corpus grows 176 → 193 entries**, all generated by the
  REAL tailwindcss 4.1.17 (`tools/gen_golden.mjs`): pseudo-element
  variants (self half — v4's `& *::selection` descendant rule is a
  documented Probe-model normalization), has-attr descendants, named
  containers, arbitrary templates, implicit tracks, justify-items,
  `md:selection:bg-*`.
- README, the how-it-works diagram and the capability table now list
  five backends; CI's `cargo check -p` gate includes `aeolus-slint`;
  the variant-engine capabilities join the matrix.

## [0.6.0] — 2026-11-07

The **upstream-ceiling** release (plan: `AEOLUS_V0.6.md`): a
reconnaissance of the latest GitHub floem and masonry (`git ls-remote`
2026-11-07) found that the floem pin `1351ffb` IS latest `main` HEAD,
and xilem upstream moved `9f504ae → dc49687` with an EMPTY
`masonry_core`/`xilem_masonry` diff — so every capability below is
consumed at the pinned revs with ZERO upstream changes. The headline:
the v0.1–v0.5 audits missed that Masonry has carried a layout-invariant
`Background::Gradient` all along; gradients lower natively now, and the
Floem backend wires the native surface it never used.

### Added

- **Gradient backgrounds in the IR** (v0.6 §1): `ResolvedStyle` gains
  `background_gradient: Option<Gradient>` — the composed
  `background-image` (`Gradient::Linear { angle_deg, stops }` /
  `Radial` / `Conic` / `None`), kept SEPARATE from the solid
  `background` exactly as CSS's `background-color` + `background-image`
  coexist. Parser: `bg-linear-to-{t,tr,r,br,b,bl,l,tl}` (CSS angles),
  `bg-linear-<angle>`, arbitrary `bg-linear-[<angle>deg]`, the
  deprecated `bg-gradient-to-*` alias, `bg-radial`, `bg-conic[-<angle>]`,
  `bg-none`; stops `from-`/`via-`/`to-` with theme/shadcn tokens,
  arbitrary values and `/opacity` modifiers (`color-mix(in oklab, …)`,
  v4's shape), plus `%` position patches (`from-20%` patches ONLY the
  position). Composition semantics pinned against the REAL tailwindcss
  4.1.17 output (`tools/capture_gradients.mjs`): stop-position defaults
  are the `@property` initial values (0%/50%/100%), missing from/to
  stops are `#0000` (transparent), interpolation is `in oklab`, stops
  without a shape compose nothing. `Slot::Gradient(GradientPart)` diffs
  through the cosmetic lane; `cn()` gains the `bg-image` split and
  per-edge from/via/to color+position groups.
- **Masonry gradient lowering, native** (v0.6 §1.5): `Background::Gradient`
  (layout-invariant `GradientShape`, `ColorSpaceTag::Oklab` = v4's
  default `in oklab`, `Extend::Pad`, sRGB stops at fraction offsets).
  The linear angle conversion `a = π − θ` is DERIVED from masonry's
  `get_peniko_linear_for_rect` paint math (its angle 0 points to the
  BOTTOM) and unit-pinned. Radial lowers as circle-farthest-corner with
  the documented ellipse→circle note (upstream `#[non_exhaustive]`
  TODO); conic reports the upstream vello sweep-offset TODO.
- **The Floem unwired surface** (v0.6 §2, all native at the pinned rev):
  `overflow-*`/`overflow-x-*`/`overflow-y-*` → taffy
  `overflow_x`/`overflow_y` (`overflow-auto` → `Scroll`, documented);
  `select-none`/`select-text` → the inherited `Selectable` prop;
  `whitespace-normal` → `TextOverflow::Wrap` and `whitespace-nowrap` →
  floem's default (a documented no-op — floem never wraps by default);
  `content-*` → `align_content` (`normal` → `Stretch` per CSS, baseline
  reported — taffy has none); `grid-cols-N`/`grid-rows-N` →
  `grid_template_*` repeat(N, 1fr), `col/row-span` + `col/row-start` →
  `grid_column`/`grid_row` placement (`N / span M` composition);
  `border-dashed`/`border-dotted`/`border-none`/`border-hidden` → kurbo
  `Stroke` dash patterns/zeroed strokes on the four side borders
  (`border-double` reports — no double-line primitive). The parity
  ledger (`slot_coverage`) upgrades Overflow, Select, Whitespace,
  AlignContent, Grid and BorderStyle Reported → Native.
- **The verification emitter reproduces v4's per-class gradient shapes**
  (v0.6 §1.4): the captured `--tw-gradient-*` var machinery
  (`--tw-gradient-position`, `--tw-gradient-{from,via,to}`,
  `--tw-gradient-{from,via,to}-position`, the two `--tw-gradient-stops`
  chains) — unit-pinned against `tools/capture_gradients.mjs` output.
- `AEOLUS_V0.6.md` — the v0.6 plan of record;
  `reports/10-tdd-evidence/08-upstream-ceiling.md` — the RED→GREEN
  evidence.
- Overflow per-axis merge fix: `overflow-hidden overflow-x-auto` now
  keeps `y = hidden` (CSS: `overflow-*` sets both axes, `overflow-x-*`
  patches one) — previously the last class replaced the whole slot.

### Changed

- The golden corpus is UNCHANGED (176 entries): gradient parity needs
  cross-rule `--tw-gradient-*` var composition (three separate rules per
  gradient) — a harness model boundary, documented as such (v0.6 §4.3,
  the v0.5 §2.3 precedent), not a coverage gap.
- Capability matrix corrections (v0.6 §3): Masonry's gradient row
  upgrades from "gradients dropped" to native; the uniform-border-width,
  single-radius and no-spread-shadow claims re-verified against the
  pinned sources and re-documented.

### Upstream re-verification (§3)

- floem: pin `1351ffb162faeb8be983f1301d718a0d5fb59c27` == latest
  `main` HEAD (2026-11-07).
- masonry/xilem: upstream `9f504ae → dc49687` = 3 commits (text-widget
  tests, example interaction tests, vello dep removal);
  `git diff 9f504ae..dc49687 -- masonry_core xilem_masonry` is empty —
  the pin stays.
- crates.io: `masonry`/`masonry_core`/`masonry_testing` 0.4.0
  (2025-10-29) and `floem` 0.2.0 (2024-11-14) remain the newest
  releases — publishing the backends stays blocked, as do the other
  three upstream deferrals (v0.6 §5).

## [0.5.0] — 2026-11-07

The **element-relative typography + allocation contract** release (plan:
`AEOLUS_V0.5.md`): the last two follow-ups the reports name as open and
in-repo-doable — the benchmark report's allocation counters ("or the
project writes its own global allocator counting harness", deliberately
out of scope for v0.4) and the element-relative typography gap (v0.4
§2.3 kept `tracking-*`/bare `leading-*` out of the corpus). The four
upstream-blocked deferrals stay deferred (§5 of the plan, re-verified
against crates.io 2026-11-07: no published masonry/floem release is
compatible with the pinned revs).

### Added

- **Element-relative typography in the IR** (v0.5 §1): `TextValue` gains
  `TrackingEm(f32)` (the `--tracking-*` token em) and
  `LineHeightRatio(f32)` (the `--leading-*` unitless ratio) — the token
  forms of `tracking-*` / `leading-*` / `text-*/*`-with-named-leading
  now carry the RELATIVE unit the class names, exactly what v4's theme
  tokens hold, instead of a 16px pre-multiplication the element's real
  font-size cannot use. Absolute forms are unchanged (arbitrary
  `tracking-[7px]` / `leading-[3px]`, spacing-multiple `leading-6` and
  `text-sm/6`, token-paired `text-sm` line-heights). Multiplying by an
  element's font-size is the renderer's cascade duty (v0.5 §1.6).
- **Corpus: element-relative typography entries** (v0.5 §2): the golden
  corpus grows 157 → **176 entries** — the six `tracking-*` tokens, the
  negative form (`-tracking-wide`, v4's `calc(var(--tracking-wide) * -1)`),
  arbitrary absolutes (`tracking-[7px]`, `leading-[3px]`), the six named
  `leading-*` tokens + `leading-none`, named text-size leading modifiers
  (`text-base/tight`, `text-lg/loose`), and two shadcn-theme entries.
  All 176 at zero-warning parity with the real tailwindcss 4.1.17 output.
- **The allocation contract** (`tests/alloc_contract.rs`, v0.5 §3): a
  counting global allocator + mutex-serialized measured sections pin
  report 08's structural claims as machine-independent numbers —
  exact-zero claims pinned exactly, clone-shaped claims as measured
  upper bounds. divan 0.1.21 re-verified as unable to carry
  per-benchmark alloc counters (`IntoCounter` is a fixed per-iteration
  value, not a sampled delta) — the contract test IS the harness the
  report asked for.
- `AEOLUS_V0.5.md` — the v0.5 plan of record.

### Fixed

- **`diff()` allocated on EVERY call** (caught by the allocation
  contract on day one — the report-08 claim "no alloc" was wrong):
  `bits_of`/`diff` routed through `slots()`, whose scratch
  `Vec::with_capacity(40)` allocated per call, and `slots()` itself was
  re-walked in the shared-bits compare. Slot presence now walks ONE
  `for_each_present_slot!` source; `diff` walks shared bits directly.
  The rebuild path is allocation-free when nothing changed — pinned
  EXACTLY — and `diff_unchanged` dropped 238.8 → 139.8 ns (report 08).
- **`-tracking-wide` ignored the negative marker**: the token arm of
  `tracking` never applied `t.negative` (the arbitrary-px arm did). The
  token em now negates, matching v4's `calc(var(--tracking-wide) * -1)`
  shape; pinned by its corpus entry.
- The parity harness's calc evaluator now folds em-valued calcs
  unit-preservingly (`calc(0.025em * -1)` → `-0.025em`) and no longer
  eats a unit-leading `e` as an exponent marker (`0.025em` parsed as
  `0.025e` before).

### Changed

- **Floem**: the `--leading-*` token form lowers to Floem's NATIVE
  `LineHeightValue::Normal(multiplier)` — font-size-relative at paint,
  exactly CSS's unitless line-height (the capability-matrix row upgrades
  from absolute-pt to element-relative). Absolute forms stay `Pt`.
  `tracking-*` keeps the honest drop report (no Floem letter-spacing
  prop at the pinned rev); the relative em stays in the plan.
- The v0.4 §2.3 corpus scope note is REWRITTEN, not deleted
  (AEOLUS_V0.5.md §2.3): element-relative values ARE pinned — as the
  relative units the plan carries; element-level composition remains the
  renderer's cascade duty, documented as a model boundary.

### Deferred to 0.6 (all upstream-blocked, re-verified 2026-11-07)

Unchanged from 0.4 §6 / 0.3 §4: publishing the backends (crates.io's
`masonry`/`masonry_core` 0.4.0, published 2025-10-29, predates the
pinned `linebender/xilem @ 9f504ae` by a year — the `LenReq`/`SizeDef`
layout model and `layers`/`imaging` surfaces the backends consume exist
in no release; `floem` 0.2.0, published 2024-11-14, predates the pinned
`1351ffb` by two years), view-side property-stack registration, Masonry
typography via native properties, Masonry transitions/animations.

## [0.4.0] — 2026-11-07

The **parity proof** release (plan: `AEOLUS_V0.4.md`): the pipeline
(parse → theme → plan) is now verified end-to-end against the REAL
`tailwindcss` v4.1.17 compiler — the golden-corpus follow-up the design
report (§7.4) and the source-audit Q&A (Q7) named — plus the benchmarks
report's allocation-counter follow-up (re-tested, still unavailable in
divan 0.1.x; documented). No new utility families, no backend changes, no
MSRV bump: v0.4 makes what already ships *verifiable*, then measures it.

### Added

- **The verification CSS emitter** (`crates/aeolus-core/src/css.rs`,
  v0.4 §1): `Resolver::to_css` derives Tailwind-v4-equivalent CSS from
  the SAME resolution backends consume — one `CssRule` per class token,
  v4's one-rule-per-candidate shape. Selectors mirror v4's templates
  (escaped class names, state/data/aria/structural pseudo-classes,
  `:not(...)`, `:has(...)` — star forms, group/peer descendant templates
  `&:is(:where(.group):hover *)`, `@media`/`@container` wrappers with
  v4's canonical `(width >= 48rem)` conditions); values mirror v4's
  shapes (`calc(var(--spacing) * n)`, oklch colors,
  `color-mix(in oklab, …)` for opacity modifiers, composed `box-shadow`
  layers for rings and multi-layer shadow presets, `calc(infinity * 1px)`
  for the full-radius sentinel). A verification/debugging tool — not a
  styling path; backends keep consuming `StylePlan` directly, and
  unsupported classes keep their warnings.
- **The golden corpus** (`tools/gen_golden.mjs` +
  `crates/aeolus-core/tests/fixtures/golden.corpus`, v0.4 §2): 157
  single-class entries across two theme contexts (the verbatim v4 default
  palette and the Shadcn neutral light theme with its `@theme inline`
  var chains), generated by the REAL tailwindcss 4.1.17 CLI. Covers
  spacing/margin, sizing (fractions/arbitrary/viewport/fit), colors
  (palette, shadcn tokens, opacity modifiers, bracket alpha, hex),
  borders/radii (logical edges), ring/outline px semantics, multi-layer
  shadows + shadow colors, typography, flex/grid layout, position/inset/
  overflow/aspect, transforms, transitions, cursor/pointer-events/select,
  all seven classic states, breakpoints (min/max), `dark:`,
  group/peer (incl. labels), `data-*`/`aria-*`, structural, `has-*`,
  `not-*`, container queries, v0.3 arbitrary variants, compounds
  (`md:hover:*`, `dark:hover:*`), `!important` (both v4 spellings) and
  negatives.
- **The parity harness** (`tests/parity_golden.rs`, v0.4 §3): both sides
  normalize through ONE pipeline — `var()` substitution from the golden
  `:root`/`@property` tables, `calc()` evaluation (rem→px, mixed units),
  `color-mix` evaluation, color canonicalization (numeric Oklab compare),
  `--tw-*` machinery composition into final `box-shadow` values, unitless
  `line-height` × same-rule `font-size` — plus the documented model
  normalizations (hover-capability wrappers dropped, `@supports`
  modern-browser override merged, initial `border/outline-style` dropped).
  Fails on ANY resolution warning in the corpus (the coverage-guard
  contract extended to the corpus).
- Benchmarks: `css_write_button` (the 33-class Button through the
  emitter) — numbers in report 08.
- Emitter unit tests for the corners the corpus cannot reach (`Test::Tag`
  selector prefixing, escaping corners, viewport units from token text,
  warning honesty, inert ring variants).
- `AEOLUS_V0.4.md` — the v0.4 plan of record.

### Fixed (caught by the corpus — the parity proof paying off immediately)

- **`-top-2`/`-translate-y-1` lost the sign**: the numeric arms of
  `inset_value`/`transform_size` ignored `t.negative` (margins handled
  it; insets and translations did not). Negative insets/translations now
  carry their sign.
- **`rounded-t-full` (and every `rounded-{edge}-full`) resolved to 6px**:
  the `--radius-full` token is absent from the v4 theme (it is
  `calc(infinity * 1px)`), so the lookup fell through to the 6px default.
  Full-radius tokens now resolve to the 9999px sentinel (emitted as
  `calc(infinity * 1px)`), and `rounded-{edge}-none` resets to 0.

### Deferred to 0.5 (all upstream-blocked, tracked in `AEOLUS_V0.4.md` §6)

Unchanged from 0.4 §6 / 0.3 §4: publishing the backends, view-side
property-stack registration, Masonry typography via native properties,
Masonry transitions/animations.

## [0.3.0] — 2026-11-06

The **arbitrary selector variants** release (plan: `AEOLUS_V0.3.md`) — the
last core-language gap from the 0.2 deferred list, plus the post-0.2.0
audit round (a full-repo sweep for gaps, panic paths and semantic
divergences from Tailwind v4.1.17, verified against the `tailwindcss`
source, not the docs alone). Every v4 arbitrary
variant form now either resolves to typed probe conditions or fails with a
per-form precise `Unsupported` reason; the blanket "arbitrary/structural
selectors have no native equivalent" report is gone. Semantics verified
against the `tailwindcss` v4.1.17 source (`candidate.ts`, `compile.ts`,
`variants.ts`), not the docs alone.

### Fixed (post-0.2.0 audit round)

- **Theme parser panics on unbalanced custom-rule blocks**: CSS ending in
  `@utility foo {` or `@custom-variant x {` with no closing brace sliced
  past the end (`rest[close + 1..]`) — fuzz-reachable via
  `theme_from_css`. The malformed tail is now ignored, matching
  `extract_declarations`' contract.
- **Theme resolution was declaration-ORDER dependent** (CSS custom
  properties resolve at use time): an `@theme inline { --color-primary:
  var(--primary); }` block preceding its `:root` source failed with
  `unsupported color function "var"`, and a `calc()` referencing a var
  whose own value is a later `calc()` failed with `calc() references
  unknown token`. Bare-var color values now defer and the deferred
  calc()/var() passes run to a fixpoint; truly-unresolvable references
  keep the precise-error contract.
- **Ring/outline/underline-offset widths were spacing multiples, not px**
  (v4.1.17 semantics, verified in `utilities.ts`): `ring-2` resolved 8px
  (correct: 2px), bare `ring` 4px (correct: 1px — `--default-ring-width`
  fallback), `ring-offset-2` 8px (correct: 2px), bare `outline` 2px
  (correct: 1px — `--default-outline-width` fallback), `outline-offset-2`
  8px (correct: 2px), `underline-offset-4` 16px (correct: 4px). Negative
  offsets (`-outline-offset-2`, `-underline-offset-2`) now carry their
  sign. Only the spacing families keep `--spacing` multiples.
- **State-layer cascade ignored the canonical variant order**:
  `StylePlan::resolve` ranked legacy state layers by source order, so
  `focus:bg-blue-500 hover:bg-red-500` resolved the HOVER layer on a
  focused+hovered element (Tailwind sorts generated CSS by variant order →
  focus wins). Layers now rank by `VariantCondition::rank` — the same
  order the stack lowering and `resolved_for_state` already used.
- **Floem silently dropped viewport/fit-content sizes**: `apply_dim`
  no-op'd `w-screen`/`h-dvh`/`w-fit` shapes without a report (the
  no-silent-failures contract). They are now `LoweringReport` entries, and
  `slot_coverage` classifies text props exactly as `lower_text` maps them.

### Added

- **Arbitrary selector variants classify into typed atoms**
  (`selector.rs`, v0.3 §1.2): the typed subset is exactly what constrains
  THE ELEMENT — the only thing the `Probe` model answers about:
  - self pseudo-classes (`[&:hover]`, `[:hover]`, `[&:hover:focus]`,
    `[p:hover]`) → `Atom::On(Test::State(_))`; `[&:hover]` generates
    byte-identical CSS to `hover:` in v4, so it canonicalizes onto the
    same legacy layer bucket (native selector lowering on Floem for
    free);
  - negation (`[&:not(:hover)]`, `not-[&:hover]`) → `Atom::Not(_)`;
  - self attributes (`[&[data-state=open]]`, `[&[aria-busy=true]]`) →
    `Test::Data` / `Test::Aria` (identical plans to the named forms);
  - structural self tests (`[&:nth-child(3)]`, `[&:first-child]`,
    `[&:empty]`) → `Test::Structural`;
  - descendant-state selectors (`[&:has(:focus)]`, `has-[:focus]`) →
    `Atom::Has(_)`;
  - **element-kind tests** (`[a&]`, `[a]`, `[p&:hover]`) → the new
    `Test::Tag` — the rule lands typed in `plan.rules`; native backends
    answer "no" (the rule never fires — the same semantics CSS has when
    a selector does not match) and a backend that models element kinds
    answers honestly;
  - arbitrary at-rules (`[@media(print)]`,
    `[@media(prefers-color-scheme:dark)]`, `[@media(min-width:900px)]`,
    the v4 canonical `[@media(width>=900px)]`, orientation/reduced-
    motion/contrast/forced-colors/pointer features, `and`-conjunctions)
    → `Atom::Media` / `Atom::Viewport`;
    `[@container(min-width:400px)]` → `Atom::Container`;
  - `group-[&:hover]` / `peer-[&:hover]` inners classify like self
    selectors; `not-*` inners must be a single self test (nested
    negation and compounds are reported — a `Cond` is a conjunction).
- **Per-form precise reports** (v0.3 §1.3) for everything the Probe
  model cannot ask: selectors styling OTHER elements (`[&_svg]`,
  `[&>*]`, `[&+&]`, `*`, `**`), pseudo-elements (`[&::before]`),
  class/id self tests (`[.x]`, `[.x&]`), `@supports`, disjunctions
  (`,` lists, `:is(a,b)`, `or` media), An+B structural forms
  (`nth-[2n+1]`, `:nth-child(2n+1)`), bare non-data/aria attributes,
  unknown media features and pseudo-classes, relative selectors
  (`[>img]` standalone), the deprecated at-rule-with-selector form.
- Tailwind's underscore decoding inside arbitrary selectors
  (`decodeArbitraryValue`: `_` → space, `\_` escapes) —
  `[@media(min-width:900px)_and_(orientation:landscape)]` works.
- Regression suites: `tests/arbitrary_variants.rs` (30 tests: the typed
  subset, evaluator equivalence with the named forms, cn() raw-variant
  keys, per-form precise reports), truth-table `Test::Tag` row, and
  backend parity tests pinning `[&:hover]` native lowering (Floem
  selector maps) + compound-rule evaluation through both probes.
- **Legacy comma color syntax**: `Color::parse` documented
  `rgb(r, g, b, a)` but only the space form parsed — `rgb(251, 44, 54)`
  failed with `bad number "251,"`. Common in arbitrary values
  (`bg-[rgba(0,0,0,0.5)]`); comma forms (with optional alpha component,
  numbers or percentages) parse now.
- **Missing `cn()` conflict groups** (tw_merge parity): logical inset
  edges `start-*`/`end-*`, the six grid-line families
  (`col/row-span/start/end`), and the shadow/outline splits —
  `shadow-md` + `shadow-destructive` and `outline-2` + `outline-primary`
  no longer knock each other out (preset-width vs color are independent
  groups, plus `outline-style`).
- `AEOLUS_V0.2.md` — the v0.2 plan of record (referenced by this README
  since 0.2.0 but never committed).
- Regression suites: `tests/px_semantics.rs` (px-vs-spacing semantics +
  canonical state cascade), theme order-independence and unbalanced-block
  tests, legacy comma color tests, and per-family `cn()` merge tests.

### Changed

- **`cond::Test::Tag(Box<str>)`** — additive variant on the public
  condition enum (0.x minor bump): exhaustive `Test` matches need a new
  arm (the in-repo probes answer "no", the honest default).
- The resolver routes arbitrary variants through
  `Resolver::atoms_of` (a conjunction list) instead of one `atom_of`
  atom; `variant_test` accepts single-self-test arbitrary inners.
- `has-[…]` accepts the `[:state]` selector form (`has-[:focus]` ≡
  `has-focus:`); other selector forms keep the precise report.
- README's deferred list shrinks to the upstream-blocked items
  (arbitrary selector variants shipped here); the capability matrix
  gains the arbitrary-variant row and the stale ring/outline/opacity/
  cursor rows were refreshed against the actual backends.
- The `space-*` warning no longer claims the resolver "emulates" it as
  gap — it reports precisely that `gap-*` is the supported equivalent.
- README's deferred list moved to v0.3 (group/peer layers, structured
  variants, container queries, responsive/dark on Masonry and Floem
  transitions shipped in 0.2.0).

### Deferred to 0.4 (all upstream-blocked, tracked in `AEOLUS_V0.3.md` §4)

- Publishing the backends — needs crates.io releases of the pinned
  Masonry/Xilem and Floem revs.
- View-side property-stack registration (retires the fallback widget) —
  upstream gap #4.
- Masonry typography via native properties — upstream patch proposal #3.
- Transitions/animations on the Masonry backend — needs Masonry-side
  animation machinery (Floem has per-set transitions).

## [0.2.0] — 2026-11-05

The "serious Tailwind v4 compiler" release (plan: `AEOLUS_V0.2.md`): the
core owns **all** semantics — including compound variants, group/peer,
data/aria, structural selectors and container queries — and backends only
map them, through the new `Probe` parity evaluator.

### Fixed (correctness — the F1–F7 silent-mis-style bugs)

- **F1** fractions: `w-1/2` → `Fraction(0.5)` (was 4px). The raw `/modifier`
  is the fraction denominator for sizing utilities.
- **F2** arbitrary lengths: `p-[7px]` → 7px (was 28px). Arbitrary values are
  absolute, never `--spacing` multiples; `(--var)`/`[var(--x)]` resolve
  through the theme.
- **F3** `dark:hover:*` lands in `rules` as `[Media(Dark), On(Hover)]` (was:
  the `dark` half silently dropped, hover applied in LIGHT mode). Shipped
  Shadcn Ghost/Outline hovers are correct now; regression-tested per
  variant.
- **F4** `md:hover:*` — same fix for breakpoint compounds.
- **F5** `rounded-s-lg`/`rounded-e-*`/`rounded-ss-*` map to the logical
  start/end edges (was: all four corners).
- **F6** `text-sm/6` resolves the line-height modifier (6 × spacing);
  `text-sm/tight` uses `--leading-tight`.
- **F7** `bg-primary/[0.37]` is a raw alpha fraction (was: rejected); 
  `bg-primary/(--o)` resolves through the theme.
- `cn()` merge keys use the EXACT canonical variant prefix — `hover:` and
  `group-hover:` no longer knock each other out, `md:` no longer collides
  with `@md:`.
- `PlanCache` is keyed by the class STRING (per-thread L1 + shared L2) — a
  hash collision can no longer return the wrong plan; entries are
  version-stamped and previous-generation hits are promoted.
- Multi-layer v4 shadows parse fully (`--shadow-sm` = 2 layers; v0.1
  truncated at the first comma).
- `@md` is the `--container-md` size (448px), not the `sm` viewport
  breakpoint; `@xs`/`@3xl` and `@min-[400px]` resolve from the container
  scale.

### Added

- **The parity engine** (`eval.rs`): `Probe` + `StylePlan::resolve` /
  `resolve_rules_only` — a backend answers questions about the element's
  situation and the SAME evaluator flattens the plan for both backends.
  `Atom`/`Test`/`Cond`/`Rule` model the full v4 variant grammar
  (`not-*` recursive, `has-*`, `data-*`, `aria-*`, structural, media,
  viewport windows, named container queries).
- **No-alloc token shape**: `ClassToken::variants` is a stack-inlined
  `Inline<Variant, 8>`, `elements` a `Segments<'a, 10>` (deref to slices);
  the raw `body`/`modifier` slices replace `join("-")` on the hot path.
- **Full v4 default theme** (`data/tailwind-default-theme.css`, diffed
  against tailwindcss v4.1.17): `--container-*`, multi-layer `--shadow-*`,
  `--text-*` + paired line heights, `--font-weight-*`, `--tracking-*`,
  `--leading-*`, `--radius-xs..4xl`, `--blur-*`, `--perspective-*`,
  `--ease-*`, `--animate-*`, default transition duration.
- **`@utility` / `@custom-variant`**: class-string expansion (depth ≤ 4)
  and name→`Media::Dark` mapping from CSS.
- **Utility coverage** (plan §3.2): `mx-auto`, `max-w-{container}`, viewport
  sizes (`w-screen`/`h-dvh` as `SizedValue::Viewport`), `gap-x/y`,
  `space-*` (gap emulation), `basis-*`, `order-*`, `content-*`,
  `grid-cols/rows`, spans/starts, `aspect-*`, `absolute/relative/...`,
  `inset-*` (incl. logical start/end), `z-*`, `overflow-*`, `visible/
  invisible`, `italic/uppercase/underline/truncate/line-clamp/font-sans/
  tabular-nums/text-ellipsis`, `transition-*/duration-*/ease-*/delay-*`,
  `animate-*`, `scale/rotate/translate/skew`, `shadow-{color}`,
  `inset-shadow-*`, `ring-inset`, `outline-offset-*`, `border-{style}`,
  logical `border-s/e`, the full `cursor-*` keyword set and `select-text/all`.
- **A coverage guard test**: every v4 utility family must resolve to a
  typed slot or a precise `Unsupported` — `Unknown` fails CI (plan §5.3).
- **Floem parity** (plan §4.2): align/self/wrap/display, gap axes, auto
  margins, `absolute()`, insets, z-index, aspect-ratio, cursor,
  pointer-events, typography (weight/style/align/line-height/overflow),
  transform scale/rotate/translate, ring→outline+outline_color, per-set
  `Transition`s, and the `tw_plan` reactive path
  (`FloemProbe` + `plan.resolve_rules_only`) that makes compound variants,
  group/peer and container queries work without an upstream patch.
  Theme-aware breakpoints replace `Breakpoint::default_px`.
- **Masonry/Xilem widening** (plan §4.3): `TwEnv` (dark, viewport/container
  width, group/peer scope via `WidgetMut::set_env`) feeds
  `plan.resolve(&probe)` in measure/layout/pre-paint; margins measured and
  inset; per-side borders and per-corner radii painted with Kurbo geometry;
  ring/outline stroked; shadow presets render every layer; opacity is
  multiplied into the chrome colors (children-not-faded documented).
- **Hot path** (plan §3.4): 8-bytes-per-round `FastHasher` with a
  length-tagged tail + final fold; string-keyed two-tier cache with
  promotion; `(ptr,len)`-keyed `resolve_static`; O(1) bitmask diff.
  Measured (1-core sandbox): cached hit 424ns → **76ns**, unchanged diff
  763ns → **235ns**, cold resolve 7.54µs → **5.49µs** (with the v0.2 rule
  machinery included). Parser is ~0.9µs slower per Button string — the
  richer v4 grammar costs, documented honestly.
- **MSRV split** (F9): `aeolus-core`/`aeolus-macro` stay at **1.88**
  (verified with a real 1.88 toolchain); `aeolus-shadcn` and the backends
  declare **1.96** (Masonry 0.4's requirement). CI grows two MSRV jobs.

### Changed

- `tw!` keeps its compile-time typo check: unknown variant *names* are
  rejected statically (the runtime parser accepts them as `Custom` for
  `@custom-variant` resolution).
- `Warning::Unsupported` texts name the parity lane ("emulated by the
  reactive probe path" / "reported, never silent (plan §7)").
- `TailwindTheme::default()` seeds the full v4 defaults (v0.1 seeded
  nothing; `from_theme_css` seeded colors only).

## [0.1.0] — 2026-11-04

The first release-candidate version: the Tailwind v4 → native-Rust style
compiler goes from research prototype to publishable. **Publishable today:
`aeolus-core` and `aeolus-macro`.** The three backend crates stay
`publish = false` (git-pinned framework dependencies; see
`reports/09-publishing/PUBLISHING.md`).

### Added

- **Phase 1 — publishability**
  - Backend manifests reference the audited frameworks by pinned git rev
    (`xilem @ 9f504ae`, `floem @ 1351ffb`) instead of sibling `repos/`
    checkouts; the workspace builds and `cargo test -p aeolus-core -p
    aeolus-macro` is green with `repos/` deleted.
  - `LICENSE-MIT` / `LICENSE-APACHE` at the repo root (referenced by the
    `MIT OR Apache-2.0` license field, previously absent).
  - `#![deny(missing_docs)]` on `aeolus-core` — every public item documented.
  - `crates/muda-stub` (dev-sandbox-only, not a member, not publishable)
    documented as the sanctioned muda/gtk workaround for Linux floem builds.
- **Phase 2 — correctness (TDD, each with a RED-then-GREEN test)**
  - `calc()` evaluation now honors **operator precedence** (shunting-yard):
    `calc(4px + var(--r) * 2)` evaluates as `4 + r*2`, and parenthesized /
    nested-`var()` expressions parse (previously left-to-right AND a
    parse error on parens).
  - `Color` equality is **approximate (~1e-4 per Oklab component)** —
    exact f32 equality caused spurious diffs and false cache misses.
  - Per-side border widths are **reported** (`LoweringReport` entry
    `border-width:per-side`) when flattened for Masonry's uniform
    `BorderWidth`, and per-side border classes now **compose**
    (`border-t border-b` keeps both edges).
  - `cn()` classifies the missing conflict families from the audited
    tw_merge table: `justify-items/self`, `align-content`, `place-*`,
    `flex-wrap`, `basis`, `order`, `position`, `inset`, `z-index`,
    `overflow-*`, `overscroll-*`, `text-align`, `object-*`, `aspect`,
    `columns`, `visibility`, `isolation`, `resize`, `font-style`,
    `text-transform`, `text-decoration`, `vertical-align` — e.g.
    `cn(["justify-items-start","justify-items-center"])` now merges.
  - Cache locks are **poison-tolerant** (`expect("lock")` removed; the
    guarded maps carry no cross-lock invariants — documented).
  - `Interner` and `PlanCache` are **bounded** (two-generation rotation,
    default 4096/generation, ≤ 2×cap memory; zero dependencies; LRU
    feature deliberately skipped).
- **Phase 3 — Track A: real interactive widgets**
  - `StyledButton` (`aeolus-xilem::button`): a real widget — pointer
    capture + active/hovered release semantics, Space/Enter keyboard
    activation, accesskit `Click`, disabled state, per-state plan
    painting — plus the `styled_button` view with callback routing.
  - `aeolus_shadcn::xilem::button(resolver, id, text, variant, size,
    class, on_press)`: the Shadcn Button is now a REAL button (the old
    API produced a styled label that ignored clicks — signature changed
    intentionally to take the callback).
  - `aeolus_shadcn::xilem::input(resolver, value, on_changed, class)`:
    Masonry `TextInput` styled from the Shadcn Input class table with the
    controlled-value contract. badge/label/card/alert remain explicitly
    style-only.
- **Phase 4 — hardening**
  - Four libFuzzer targets (`fuzz/`): `parse_class_string`, `color_parse`,
    `cn`, `theme_from_css` — each ran clean 60s in release+ASAN
    (1.19M / 7.26M / 540K / 6.3K execs).
  - Property tests (`proptest`, dev-only): `cn` idempotence, parser
    roundtrip, OKLab↔sRGB roundtrip (tight in-gamut, finite everywhere),
    opacity hue preservation.
  - CI (`.github/workflows/ci.yml`): fmt, clippy `-D warnings`, test
    matrix (linux/macos/windows), docs `-D warnings`, package gate,
    `cargo deny` licenses, 60s fuzz smoke (main + weekly).
  - `deny.toml` with the license allow list enumerated from the
    dependency graph.
  - Runnable crate-level doc example in `aeolus-core`.
- **Phase 5 — release prep**
  - `CHANGELOG.md`, `SECURITY.md`, `CONTRIBUTING.md`, rewritten
    `README.md`, `reports/09-release/RELEASE_CHECKLIST.md`, `v0.1.0` tag.

### Fixed

- `Color::to_srgba` clamps linear components (documented but unimplemented):
  out-of-gamut Oklab previously produced **NaN** through `powf` on negative
  linear components (found by the property suite).
- Fuzz findings (each fixed TDD-first):
  - `parse_hex` byte-sliced multibyte strings of the right byte length
    (panic mid-char on `#abᐁ`); ASCII-gated now.
  - `cn()` leaked caller-internal whitespace (`cn("\n")` echoed `"\n"`)
    and broke idempotence; output is now always a normalized word sequence.
  - `extract_declarations` sliced past the end for CSS ending in an
    unbalanced block; the malformed tail is ignored.
  - Absurd alpha components (`1e95`) parsed to f32 inf; alpha clamps to
    [0,1] per CSS.
  - Huge hue (`oklch(3 3 1.9e83)`) turned NaN in `cos` argument reduction;
    hue clamps to ±360°.
  - Chroma overflow (`3.3e40`) overflowed the Oklab→linear cubes to inf;
    Oklab components clamp to their physically meaningful ranges at
    conversion.
- Duplicate `grow`/`shrink` conflict arms (unreachable-pattern warnings).

### Deferred to 0.2

- **Publishing the backends**: blocked on upstream crates.io releases of
  the pinned Masonry/Xilem and Floem commits, and on floem making its
  Linux muda/gtk dependency optional (`reports/09-publishing/`).
- Group/peer conditional layers (`group-hover:`) — needs the Masonry
  class-scoped group-state patch (proposal in `reports/07-upstream-patches/`).
- View-side property-stack registration (upstream gap #4) — retires the
  `StyledWidget` fallback wrapper.
- Typography properties in Masonry (`FontSize`/`FontWeight` upstream
  patch proposal #3) and responsive/dark layers on the Xilem backend.
- Container queries (`@*` variants) on any backend.
- Opacity / ring / outline / cursor / pointer-events slots on the Masonry
  backend (reported as dropped today, never silently ignored).
- Transition/animation support (recognized-and-dropped with warnings).
- Structured variants (`has-[…]`, `data-[…]`, `aria-*`, `[&_svg]`) —
  parser extension beyond the v1 scope decision.

[0.1.0]: https://github.com/aeolus-rs/aeolus/releases/tag/v0.1.0
