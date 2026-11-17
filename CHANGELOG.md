## [Unreleased]

### 🚀 Features — the program lane's pseudo-target face (M1)

- **`program_style_pseudo`** / **`program_style_pseudo_theme`** — the interp arm's own-cascade contract grown onto the FROZEN static program: the two slices the `tw!` macro emits resolve one [`PseudoTarget`]'s own cascade into the v0.x [`ResolvedStyle`], the `(style, walk)` shape the element walk answers. Where the pseudo cascades resolved only through `interp_pseudo`/`interp_pseudo_theme` (the heap `Program` the compile lane holds — a host holding the macro's frozen form had no face for the render targets at all), the program lane answers them now: the verdict is CONDITION EQUALITY (the interp arm's own `walk_pseudo` — only the layers carrying the target's condition participate, later fired wins per longhand, no `Env` and no probe seams consult, the flat fold's own reading), the fold runs through the SAME [`PlanWriter`] the element walk writes through (the row texts indexing the flattened annotations across ALL layers, the slot boundaries reporting typed, never silent), and the render-target contract holds both halves: the element walk's [`InterpNote::PseudoElement`] note reports the target layers on ITS side, the target walk stays SILENT about the element's own layers and the sibling targets. The theme forms ride the walk's two faces: the themeless entry refuses [`Val::Tok`]/[`Val::ModePair`] PER LONGHAND into [`StyleWalk::refusals`] (the style fold's report shape, the reason naming the lane that works — `program_style_pseudo_theme`), the themed entry resolves them through one [`TokenSource`] (the `env` riding the pair selection, never the firing). The host surface carries the face: `aeolus-rt` re-exports the entries beside `PseudoTarget`/`interp_pseudo`/`interp_pseudo_theme` and `program_style_struct` (the re-export block's missing entries), the walk module's docs diagram grows the target's own walk beside the element walk's. Tests: `crates/aeolus-compile/tests/style_walk_pseudo.rs` (9 pins, RED-first — the suite does not compile against the pre-seam walk, the missing entries ARE the capture: the selection cascade's later-beat winner, the target-side silence, the whole closed set headless, the themeless refusal's per-longhand report, the themed resolution, the interp-arm differential agreement, the flattened-index reading both orders, the target kw's slot boundary). Bench: `crates/aeolus-compile/benches/style_walk_pseudo.rs` (the selection fire path ~44 ns median beside the sibling quiet path ~48 ns, the themed resolution, the runtime-tier refusal report) — join with `[[bench]] style_walk_pseudo`. Example: `crates/aeolus-compile/examples/style_walk_pseudo.rs`.

### 🚀 Features — the host surface's own style face (M2)

- **`StaticProgram::style`** / **`style_with`** / **`style_pseudo`** / **`style_pseudo_theme`** on `crates/aeolus-rt` — the walk entries as METHODS on the frozen type the macro emits, the free functions' delegates (one walk, two spellings, the differential suite pins the answers byte-equal), so a host threads no slices and no imports beyond the `aeolus_rt` surface it already consumes. The face's return type rides the re-export too: `ResolvedStyle` is nameable from `aeolus-rt` for the first time (the crate grows the `aeolus-core` dependency pin beside it — the walk entries' own shape, not a second style type). Tests: `crates/aeolus-rt/tests/program_style_face.rs` (5 pins, RED-first — the methods do not exist yet, the compile failure IS the capture: the element/method agreement, the seam threading, both pseudo faces, the themed resolution through a real `ClassCell`-built program). Bench: `crates/aeolus-rt/benches/style_face.rs` (the method beside the free entry it delegates to — element ~200 ns vs ~174 ns, pseudo ~36 ns vs ~45 ns, the delegation parity the face exists to bound) — join with `[[bench]] style_face`. Example: `crates/aeolus-rt/examples/style_face.rs`.

### 🛠 Fixed — docs drift (this handoff)

- `docs/AEOLUS_V1_ARCHITECTURE.md` — the §7 re-target paragraph names the program lane's pseudo-target face and the host surface's style face beside the landed backend walks (the doc's re-target story predated both seams).
- `README.md` — the variant table's pseudo-element row names the program-lane face beside `interp_pseudo`; the crate map's `aeolus-rt` row names the `style*` face; the example indexes file the two new demos; the workspace battery count re-measured post-handoff.

### 🧹 Micro-debt (this handoff)

- `crates/aeolus-rt/tests/class_cell.rs` — the compile surface's FOURTH form gains its pin: `ClassCell::get_theme_with` had no consumer and no coverage anywhere in the workspace (the cell's own doc claims "the whole compile surface cell-shaped", the full-surface test exercised three of the four entrances); the new test pins the table+bag identity, the registered keyword text, the hit path and the content-miss swap.

### 🎁 Consumer examples (additive)

- `crates/aeolus-compile/examples/style_walk_pseudo.rs` — the pseudo-target face headless: the element walk's excluded targets beside the selection/placeholder cascades, the quiet path, the runtime tier's themeless refusal beside the themed form.
- `crates/aeolus-rt/examples/style_face.rs` — the host surface's own face headless: both walks through the methods, the delegation parity beside the free entries.

### 🚀 Features — the program lane's full probe seam (M1)

- **`program_style_probes_containers_struct_descendants_supports`** — the §7 style walk grows the interp arm's whole probe surface (the `interp_probes_containers_struct_descendants_supports` entry's own naming, the program lane's `*_with` token tail kept). Where the walk threaded only `AttrProbe` + `StructProbe` + `TokenSource` (the named-container, `has-*` and `@supports` layers reporting unqueried forever — the walk's own docs named the gap "the natural follow-up"), the widest entry threads all five probe seams into the SAME `decide` verdicts: a named container fires by the AT-and-above / strictly-below boundary rules the interp pins (a container the source does not know is the QUIET miss a selector that does not match is), a `has-*` layer fires on the subtree's YES, a `supports-*` layer fires on the capability's YES, each answered miss CSS-quiet, each sourceless form keeping the typed boundary it always carried ([`InterpNote::ContainerUnqueried`]/`DescendantUnqueried`/`SupportsUnqueried` — the pre-seam shape, unchanged for callers that pass no source). One `decide`, two walks, no third verdict path — the narrow entries are the widest form with the seams they do not name riding `None` (the equality pin). Tests: `tests/style_walk_probes.rs` (7 pins, RED-first — the suite does not compile against the pre-seam walk, the missing entry IS the capture), the neighbouring suites green (`tests/style_walk.rs`, `tests/structural.rs`, `tests/containers.rs`, `tests/descendants.rs`, `tests/supports.rs`). Bench: `benches/style_walk.rs` (the sourceless report path the backends ride beside the full-probe fire path and the attr-wired ceiling) — join with `[[bench]] style_walk`. Examples: `crates/aeolus-compile/examples/style_walk_probes.rs`. The host surface: `aeolus-rt` re-exports the entry beside `program_style`/`program_style_with` (and grows the `Descendant*` seam types its re-export block lacked).

### 🚀 Features — the probe seams ride the plan-lane backends (M2)

- **`lower_program_probes_containers_struct_descendants_supports`** on the four shared-walk plan-lane backends — xilem, egui, iced and slint (the floem/gpui walks already answer the full surface; these four inherited the walk's pre-seam shape). The widest entry threads the container/structural/descendant/supports seams through the shared walk and keeps each backend's report translation unchanged: a host whose layout KNOWS the answers (a measured `@container` element, a tracked subtree, a capability query) gets those layers to FIRE in the lowered style, the answered miss the quiet skip, the sourceless form the honest report drop it always was. Tests: `crates/aeolus-xilem/tests/program_probes.rs`, `crates/aeolus-egui/tests/program_probes.rs`, `crates/aeolus-iced/tests/program_probes.rs`, `crates/aeolus-slint/tests/program_probes.rs` (three pins each — the sourceless report, the wired fire, the answered quiet miss — RED-first against the missing entry).

### 🛠 Fixed — docs drift (this handoff)

- `docs/AEOLUS_V1_ARCHITECTURE.md` — the §3.7 shadow row's impossible conformance progression dropped (`59.50% → 61.16%` is no state of the 146-prop table; the drift-gated score is the generated page's own record, `docs/CONFORMANCE.md`).
- `README.md` — the example index's missing `suggest` row added (the seam's consumer example predates the index row; every other indexed example verified on disk), the program lane's new demo beside it.
- `README.md` — the workspace battery count re-measured post-handoff (the honest run's sum below).

### 🧹 Micro-debt (this handoff)

- `crates/aeolus-theme/src/source.rs` — the comment walk's guard-proven step no longer `expect`s (`rest.chars().next()` behind `while !rest.is_empty()`), the let-else + `unreachable!` naming the guard — the no-panic contract's audited form the resolve walks carry.

### 🎁 Consumer examples (additive)

- `crates/aeolus-compile/examples/style_walk_probes.rs` — the program lane's full probe walk headless: the sourceless reports, the seams' fires, the answered quiet miss, the narrow-entries equality pin.

### 🚀 Features — the parent-relative composition tier (M1)

- **`group-focus-visible:`/`group-focus-within:`/`group-active:`/`group-disabled:`/`group-visited:`/`group-checked:`** and the peer half **`peer-focus-visible:`/`peer-focus-within:`/`peer-active:`/`peer-disabled:`/`peer-visited:`/`peer-checked:`** fold — the parent-relative compositions over the FULL probe-state set. Where only the hover/focus pair folded, the tailwindcss v4.1.17 `group`/`peer` compound mechanisms' whole eligible set folds now (the compound table composes with every StyleRules-eligible `staticVariant` — `active`/`visited`/`disabled`/`checked`/`focus-visible`/`focus-within`). Each composition is its own typed `Cond` probe and its own `Env` frame fact (a `.group` ancestor / `.peer` sibling IN the composed state); the unset bit stays the QUIET miss a frame the composition does not hold in (the media facts' own reading — never a note, never a panic). The CSS arm renders the composed state on the relation's own head (`.group:active`, the sibling `.peer:disabled ~`); the plan bridge lands the layer in the same labelled group/peer buckets the landed pair rides; the metamorphic checker sets and reconstructs each spelling; the LSP hover labels the tier; gpui/floem answer the composition facts from the frame env, iced/egui/slint/xilem carry them honestly off (the probe surfaces' documented boundary). Tests: `tests/compile.rs` (the fold + label form + rank pins), `tests/interp.rs` (the env facts + the tie rule), `tests/css.rs` (the heads + the labelled head), `tests/plan_bridge.rs` (the bucket routing), `tests/statics.rs`/`tests/rust_tokens.rs` (the closed-set arms), `crates/aeolus-ir/tests/ir_laws.rs` (rank + source round-trip), `crates/aeolus-gpui/tests/compositions.rs` + `crates/aeolus-floem/tests/compositions.rs` (the frame-fact walks) — RED-first against the diagnosed-refusal boundary. Bench: `benches/compositions.rs` (the fire path, the resting quiet-miss path, the fold's own cost) — join with `[[bench]] compositions`. Examples: `crates/aeolus-compile/examples/group_peer_states.rs`.

### 🚀 Features — the composition tier's CLI face (M2)

- **`expand --group-focus-visible/--group-focus-within/--group-active/--group-disabled/--group-visited/--group-checked`** and the peer half **`--peer-focus-visible/--peer-focus-within/--peer-active/--peer-disabled/--peer-visited/--peer-checked`** — the composition facts ride the CLI face headless (the `expand` flag battery's own shape, the fact flags the interp `Env` answers from). The usage text carries the twelve flags; the closed set parses and the fired layer lands in the effective declarations, the unset fact the quiet miss. Tests: `crates/aeolus-cli/tests/expand_group_peer.rs` (the fact pins + the resting face + the closed flag set). Example: `crates/aeolus-cli/examples/expand_group_peer.rs`.

### 🛠 Fixed — docs drift (this handoff)

- `README.md` — the example index's glued duplicate comment split (`expand_form_states` carries the form-state facts line, `expand_starting` its own, both blocks), the duplicate `pseudo_targets` row dropped, the "every crate ships runnable examples" claim scoped to the xtask driver exception.
- `docs/AEOLUS_V1_ARCHITECTURE.md` — the crate table's `aeolus-compile` sketch signature aligned with the landed surface (`compile(&str, &Config) -> Result<Compiled, CompileError>`, the five emit arms named).
- `README.md` — the workspace battery count re-measured post-handoff (the honest run's sum below).

### 🧹 Micro-debt (this handoff)

- `crates/aeolus-ir/src/val.rs` — the dead `Val::as_verbatim` accessor removed (no consumer anywhere in the workspace; the CSS arm renders `Val::Verbatim` through its own match wall).
- `crates/xtask/src/lib.rs` — the swallowed `create_dir_all` failure now carries its context (`prepare failed:` report line beside the write failure's own).

### 🎁 Consumer examples (additive)

- `crates/aeolus-compile/examples/suggest.rs` — the suggestion seam (`suggest_for`/`SuggestionSurface`) end to end: the registry, the variant set, the raw surface walk, the registered custom.

### 🚀 Features — the of-type structural tier (M1)
- `aeolus-ir` + `aeolus-compile` + `aeolus-core` + the CLI: the structural family's OF-TYPE tier lands — the five v4.1.17 spellings `first-of-type:`/`last-of-type:`/`only-of-type:`, `nth-of-type-N:` and `nth-last-of-type-N:` (the differential evidence the tailwindcss v4.1.17 SOURCE's own structural block again — `packages/tailwindcss/src/variants.ts`, pinned verbatim: `staticVariant('first-of-type', ['&:first-of-type'])` through `staticVariant('only-of-type', ['&:only-of-type'])`, plus the `functional('nth-of-type', …)`/`functional('nth-last-of-type', …)` registrations). The five spellings fold onto the typed [`Structural`] enum's five new arms (the child-index family's own 1-based semantics read in the TYPE-relative dimension, the longest-prefix-first fold order keeping `nth-last-of-type-` out of the `nth-` arms — the `nth-last-`/`nth-` order's own discipline), the [`StructFacts`] seam value grows `type_index`/`type_sibling_count` beside the child-index fields (what "same type" means is the HOST's own reading — the seam carries the answer, never a tag string; `StructFacts::new` keeps its child-index shape and `with_type` carries both dimensions, the of-type tests quietly missing until a host supplies the type facts), the interp answers through the same `structural_fires` verdicts (a source-answered miss is CSS-quiet, the sourceless forms keep the `StructuralUnqueried` note), the CSS arm renders the upstream selector texts (`:first-of-type` through `:nth-last-of-type(N)` — the child-index family's own grammar in the type dimension), the static emit names the typed tests, the plan bridge LANDS the tier on the oracle's own grown `ast::Structural` table (the parser's spellings, the canonical texts, the CSS render and the snapshot writer/strict reader in lockstep; the `nth-last-of-type-*` spelling keeps the oracle's last-flag divergence pinned — the `nth-last-2:` precedent's sibling), the CLI's `expand --child-of-type i/n` facts flag answers the dimension (the `--child` flag's sibling — the two dimensions ride their own facts, an independent answer never derived), and the completion surface (KNOWN_VARIANTS + the two-sided audit) catches up — the of-type trio beside the structural spellings, AND the pseudo-element tier's eight render-target spellings the pre-tier landing left out. Tests `tests/of_type.rs` (11 pins, RED-first) + `tests/cli.rs`'s of-type battery (2 pins, RED-first); bench `benches/struct.rs`'s of-type half (161.1 ns hit / 114.9 ns miss median); examples `examples/struct_probe.rs` grows the tier demos (sections 7-8).

### 🚀 Features — the `starting:` variant (M2)
- `aeolus-ir` + `aeolus-compile` + the backends + the CLI: the `starting:` variant lands — `@starting-style`, the transition's first frame (the differential evidence the tailwindcss v4.1.17 SOURCE's own state block: `staticVariant('starting', ['@starting-style'])`, pinned verbatim — the media-feature tier's at-rule sibling whose `Env` reading the pseudo-element landing deliberately deferred, resolved as the frame-state shape the landing deferred was never: the fact IS frame state). The spelling folds onto the typed `Cond::Starting` arm (the at-rule tier's specificity 4 — the block WRAPS the rule, the `@media` shape, never a selector conjunct), the `Env` carries ONE affirmative frame fact (`starting_style` — the host's first-render gate beside the transition; an unset bit the QUIET miss a frame the element is not in is, the media facts' own reading — no probe seam and no sourceless note), the CSS arm renders the bare `@starting-style` head in BOTH walls (no payload — the render cannot drift the fold), the plan bridge reports the typed `PlanNoteKind::StartingStyle` boundary (the v0.x plan surface has no starting-style slot, the media-feature boundary's sibling), the metamorphic checker's conjunctive env bakes the fact so the compile == CSS == interp triangle agrees, the GPUI/floem walks share the interp verdicts verbatim, the four plan-lane backends carry the env literal, the LSP labels the family, and the CLI's `expand --starting` flag asserts the fact (one flag per FACT, the `--portrait` precedent — the expand usage block's drifted flag list catching up beside it). Tests `tests/starting.rs` (8 pins, RED-first) + `tests/cli.rs`'s starting battery (2 pins, RED-first) + the GPUI/floem walk pins (3 + 3, RED-first); bench `benches/starting.rs` (127.7 ns fire / 80.7 ns resting median); examples `examples/starting.rs` + `examples/expand_starting.rs`.

### 🚀 Features — the pseudo-element render-target tier (M1)
- `aeolus-compile` + `aeolus-ir` + the CLI: the pseudo-element render-target tier lands — the eight spellings `before:`/`after:`/`marker:`/`first-letter:`/`first-line:`/`file:`/`backdrop:`/`details-content:` beside the `selection:`/`placeholder:` pair (the differential evidence the tailwindcss v4.1.17 SOURCE's own pseudo-elements + state blocks again — `packages/tailwindcss/src/variants.ts`, pinned verbatim: `staticVariant('first-letter', ['&::first-letter'])` through `staticVariant('details-content', ['&::details-content'])`, plus the `before`/`after` FUNCTIONAL bodies `styleRule('&::before', [contentProperties(), decl('content', 'var(--tw-content)'), …nodes])`). The eight spellings fold onto bare typed conditions (`Cond::Before`/`Cond::After`/`Cond::Marker`/`Cond::FirstLetter`/`Cond::FirstLine`/`Cond::FileSelector`/`Cond::Backdrop`/`Cond::DetailsContent`, `source()` naming the canonical spelling, the probe tier's specificity 2 — a pseudo-element head is a selector conjunct on the `&`, NOT an at-rule): a pseudo-element styles a DIFFERENT render target than the element's own cascade, so the [`PseudoTarget`] render-target enum grows the eight arms beside the pair it carried with the closed-set surface (`ALL`, `from_name`, `spelling`), the element walks EXCLUDE every pseudo layer with the [`InterpNote::PseudoElement`] note (the render-target contract — never element state, never an error), and the pseudo cascades resolve per target through the SAME condition-equality walk (`interp_pseudo`/`interp_pseudo_theme`, `Compiled::interp_pseudo` delegating). The CSS arm renders the upstream selector texts in BOTH walls (the built-in segment wall and the alias path's `apply_cond_shape` — `::before` through `::details-content`; `marker:` the FOUR-selector disjunction one rule carries, `& *::marker`/`&::marker`/the `-webkit-details-marker` pair, comma-joined alternatives that each ride the conjunctive stack; `before:`/`after:` the functional bodies' injected `content: var(--tw-content)` line riding FIRST, the per-rule `""` fallback standing in for the at-root `@property --tw-content` registration the per-candidate emitter cannot carry — the documented divergence, the initial value upstream registers). The plan bridge LANDS `file:` (the v0.x `PseudoElement::FileSelectorButton` the surface already carries — the selection/placeholder arm's own shape) and reports the other seven through the typed `PlanNoteKind::PseudoElement` boundary (the media-feature boundary's sibling — the class stays visible, never silently dropped). The metamorphic checker's `PseudoTarget` notes, condition labels and conjunctive env grow the tier; the GPUI and floem walks keep every pseudo layer out of the element cascade (`false` / the exclusion report); the four plan-lane backends' boundary channels render the new note; the LSP hover labels the tier. The CLI face: `expand --pseudo <target>` answers the NAMED render target's own cascade beside the element answer (the closed set's parse — `selection|placeholder|before|after|marker|first-letter|first-line|file|backdrop|details-content`; unknown spellings are usage errors, exit 2; the JSON form carries the `"pseudo"` object only when the flag rides). Upstream's remainder the tier deliberately does NOT carry: `link:` (NOT a v4.1.17 variant — absent from `variants.ts`), the of-type structural trio, `starting:` (`@starting-style` — the transition's first frame, a different shape than the frame-state tier) and the `nth-of-type-*` grammar (the `nth-` walk is the child-index grammar). Tests `tests/pseudo_elements.rs` (15 pins, RED-first — the suite does not compile against the un-landed tier, the fold's typed refusal the capture), `tests/cli.rs`'s pseudo battery (4 pins, RED-first), the GPUI exclusion pins (`tests/pseudo.rs`, 3) and the floem exclusion pins (`tests/pseudo.rs`, 3); bench `benches/pseudo_targets.rs` (the nine-layer mixed program: the tier's targets resolve at the same per-pass cost the pair carried, ~54–65 ns median); examples `examples/pseudo_targets.rs` (the tier's walks, one cascade each) + `examples/expand_pseudo.rs` (the `--pseudo` face's answer).

### 🚀 Features — the media-feature remainder (M2)
- `aeolus-compile` + `aeolus-ir` + the CLI: the media-feature remainder lands — the preference/capability tier's last two v4.1.17 static variants, `inverted-colors:` (`@media (inverted-colors: inverted)`, the v4.1 addition) and `noscript:` (`@media (scripting: none)`; the scripting feature's own static spelling — `enable`/`initial-only` have no v4.1.17 registration, the spelling set stays closed). The two spellings fold onto the typed [`Pref`] bits (`Cond::Media(Pref::InvertedColors)`/`Cond::Media(Pref::NoScript)`, `source()` naming the canonical text, the at-rule tier's specificity 4), the `Env` carries TWO facts (`inverted_colors` the user agent's inverted-colours answer — the accessibility answer the `forced_colors` bit rides beside — and `scripting_none`), the interp answers with no probe seam and no sourceless note (an unset frame fact is the QUIET miss a media query that does not hold is), the CSS arm renders the upstream at-rule texts in BOTH walls (`MediaPref`'s table grows the two texts — no selector suffix renders, the at-rule tier), the metamorphic checker's conjunctive env carries the facts, the GPUI and floem walks share the interp verdicts verbatim, the plan bridge keeps the typed `PlanNoteKind::MediaFeature` boundary (no new note kind — the v0.x Atom set carries width bounds only), and the CLI's `expand --inverted-colors`/`--noscript` flags assert the facts (one flag per FACT, the `--portrait` precedent; unset is the quiet miss). With this landing the media-feature tier is COMPLETE — every `@media`-shaped static variant v4.1.17 registers is a typed bit. Tests `tests/media_remainder.rs` (6 pins, RED-first) + `tests/cli.rs`'s remainder battery (2 pins, RED-first); bench `benches/preferences.rs` (the remainder walk 80.38 ns median, the family beside it); examples `examples/preferences.rs` + `examples/expand_preferences.rs` grow the remainder's faces.

### 🛠 Fixed — docs drift (this handoff)
- `README.md`: the workspace battery count re-measures against the actual `--all-features` run (the claimed 1771 did not move with this handoff's pins — the honest run's sum is 1804, the claim now, dated the pseudo-element handoff); the variant table's `selection:`/`placeholder:` row grows into the whole render-target tier the tier landing carried; the media-feature row names the remainder's two spellings; the examples index files the tier's two demos (`pseudo_targets` re-run, the new `expand_pseudo`) and the remainder's flags ride the preference demos' comments.

### 🧹 Micro-debt (this handoff)
- `crates/aeolus-compile/src/interp.rs`: `PseudoTarget` derives `Hash` — every sibling enum the render-target condition carries (`Cond`, `Structural`, `Pref`, `Direction`) hashes, the render-target enum did not, so a host keying a per-target cache (the DynCache precedent) could not key the target beside the program. The derive is additive; equality semantics unchanged.

### 🚀 Features — the form-state family (M1)
- `aeolus-compile` + `aeolus-ir` + the backends: the form-state family lands — the matches-neither static variants `enabled:`/`required:`/`optional:`/`in-range:`/`out-of-range:` beside the widget-state trio (the differential evidence the tailwindcss v4.1.17 SOURCE's own state/forms blocks again — `packages/tailwindcss/src/variants.ts`, pinned verbatim: `staticVariant('enabled', ['&:enabled'])`, `staticVariant('required', ['&:required'])`, `staticVariant('optional', ['&:optional'])`, `staticVariant('in-range', ['&:in-range'])`, `staticVariant('out-of-range', ['&:out-of-range'])`; the family CLOSES the matches-neither boundary evidence 27 documented — a non-form element matches NEITHER half of each pair, so ONE `Env` bit cannot answer them honestly — with the TWO-FACT shape: a MEMBERSHIP fact per universe beside the pair's shared state bit). The five spellings fold onto bare typed conditions (`Cond::Enabled`/`Cond::Required`/`Cond::Optional`/`Cond::InRange`/`Cond::OutOfRange`, `source()` naming the canonical spelling, the probe tier's specificity 2 — selector conjunctions, not at-rules): the form-own answers are ELEMENT STATE the host answers, so the `Env` carries FOUR facts — `form_associated` (the form-membership fact the HTML spec's form-associated list names, the gate EVERY form pair reads; `enabled:` fires on the membership beside the unset `disabled` bit), `required_field` (the `required` attribute the required/optional pair splits on — the attribute bit alone never fires; `required:` on the bit, `optional:` its complement WITHIN the universe, the `validation_invalid` precedent), `range_bounded` (the bounded-universe membership fact — the element tests its value against declared min/max bounds AND holds a bounds answer; the host keeps the fact off while no position answer exists, the pristine-empty reading) and `range_in` (the value's position within the bounds — `in-range:` on the bit, `out-of-range:` its complement within the universe); the unset membership is the QUIET miss a selector that does not match is (never an exclusion note). Every match wall the fold's tier-2 arm feeds follows: the CSS arm renders the upstream pseudo-classes in BOTH walls (the built-in segment wall and the alias path's `apply_cond_shape` — `:enabled` through `:out-of-range` beside `:target`, no at-rule ever renders), the `has-` inner grammar answers the spellings beside `target:` (one grammar, three consumers — `has-enabled:` folds to `*:enabled`), the metamorphic checker's conjunctive env carries the two-fact shape (the membership bakes true beside the bit, or beside its complement false — the `valid:`/`read-only:` arms' own reading), the GPUI and floem walks read the same Env semantics verbatim, the four plan-lane backends' probe paths honestly keep the facts off, the LSP labels the family, and the plan bridge LANDS the state slots — the v0.x `State` surface already carries the five forms (the landed slots evidence 27 documented at the boundary), so every layer buckets into its `Test::State(...)` rule (the `checked:` precedent, no note rides) and the snapshot's strict reader round-trips the names. The completion surface carries the five spellings (`KNOWN_VARIANTS` beside the two-sided audit). Tests `tests/form_states.rs` (15 pins, RED-first), bench `benches/form_states.rs` (the env-answer walk 160.4 ns median, the resting-env path 26.91 ns), example `examples/form_states.rs` (the fire, the resting quiet miss, the bounded complement, the CSS head — headless).

### 🚀 Features — the form-state family's CLI face (M2)
- `aeolus-cli`: `expand --form-associated/--required/--range-bounded/--range-in`, the form-state facts' headless flags (the widget-state flags' sibling — element state needs no facts row, the `--dark` precedent): the flags set the Env bits, the walk answers the form-state layers from them; unset flags are the quiet miss (never a reported exclusion), ONE flag per FACT and no complement flag rides (the `--target` precedent — `--form-associated` the form-membership fact every form pair gates on, `--required` the required-attribute fact, `--range-bounded` the bounded-universe membership fact, `--range-in` the value's position fact; `optional:` answers within the membership, `out-of-range:` within the bounds). The allow list gates the flags to `expand` alone, and the Env build threads them beside the widget-state bits. Tests `tests/cli.rs`'s form-state battery (6 pins, RED-first — the firing answer, the membership gate the attribute flag alone never passes, the within-universe complements both ways, the quiet miss, the facts' mix), example `examples/expand_form_states.rs` (the fact-bearing form beside the resting and bounded-complement forms, headless through the same `run` entry point the binary drives).

### 🛠 Fixed — docs drift (this handoff)
- `README.md`: the quickstart block's five duplicated example lines retire (`descendants`/`supports`/`preferences`/`capabilities` repeated beside the family rows, `widget_states` listed twice); the all-examples details block files the six state-family demos the per-crate index never named (`descendants`/`supports`/`preferences`/`capabilities`/`element_states`/`widget_states`) beside the family's own demos; the variant table files the form-state row; the workspace battery count re-measures against the actual `--all-features` run (the claimed 1750 did not move with the last handoffs' pins — the honest run's sum is 1771, the claim now); the feature-flags section names what CI actually runs (the default-feature suite — the all-features battery is the workspace gate, and there is no conflict-feature bundle to gate: every flag composes).

### 🧹 Micro-debt (this handoff)
- `crates/aeolus-cli/src/lib.rs`: the `expand` path's `keyword_texts` expect retires — the call sat on a `Result` whose "only the empty input errors it" contract was documented but not type-enforced; the typed failure now renders through the sibling walk's own error channel (stderr + exit 2), an honest-failure path instead of a panic the wording had to argue cannot happen.
- `.github/workflows/ci.yml`: the `msrv-core` job names the four 1.88 crates it never checked (`aeolus-rt`/`aeolus-codegen`/`aeolus-codegen-gpui`/`aeolus-lsp` — each declares `rust-version = 1.88` while the job's "gated here, not just claimed" comment covered only ten of the fourteen).
- `crates/aeolus-cli/benches/pipeline.rs`: the file header carries the "divan, stable — the workspace bench idiom" wording every other bench file documents (the one header that predated the convention).

### 🎁 Consumer examples (additive)
- `crates/aeolus-cli`: `examples/expand_preferences.rs` — the preference families' consumer demo (`expand --reduced-motion/--more-contrast/--forced-colors/--print` — the media-feature facts the preference layers answer from; the flags existed since the preference landing with no example beside them, the example index's own gap).

### 🚀 Features — the widget-state family (M1)
- `aeolus-compile` + `aeolus-ir` + `aeolus-core` + the backends: the widget-state family lands — the one-fact widget-state static variants `open:`/`inert:`/`default:` beside the element-state quartet (the differential evidence the tailwindcss v4.1.17 SOURCE's own state/forms blocks again — `packages/tailwindcss/src/variants.ts`, pinned verbatim: `staticVariant('open', ['&:is([open], :popover-open, :open)'])`, `staticVariant('inert', ['&:is([inert], [inert] *)'])`, `staticVariant('default', ['&:default'])`; the family CLOSES the v0.x-surface boundary the element-state landing documented — the gap closes in the OTHER direction: the core `State` surface GROWS the three slots — `State::Open`/`State::Inert`/`State::Default` through the parser's spelling table, the resolver's `state_suffix`/`state_name` walls, the selector wall and the snapshot's strict reader — AND the V1 `Cond` surface grows the three conditions). The three spellings fold onto bare typed conditions (`Cond::Open`/`Cond::Inert`/`Cond::Default`, `source()` naming the canonical spelling, the probe tier's specificity 2 — selector conjunctions, not at-rules): every family question is AFFIRMATIVE (the `target:` shape — a non-open element matches neither `:open`'s arms, a non-form element misses `:default`, a non-inert subtree misses `:is([inert], [inert] *)`), so the `Env` carries ONE fact per question — `opened` (the disclosure/popover answer), `inert` (the frozen-subtree answer the host computes for every element inside (or carrying) an `[inert]` root, the compound head's two arms sharing the one per-element fact), `form_default` (the form group's default-control answer beside `checked`) — and no complement rides; the unset bit is the QUIET miss a selector that does not match is. Every match wall the fold's tier-2 arm feeds follows: the CSS arm renders the upstream selector texts in BOTH walls (the built-in segment wall and the alias path's `apply_cond_shape` — the compound `:is(...)` heads verbatim beside `:target`, no at-rule ever renders), the `has-` inner grammar answers the states beside `target:` (one grammar, three consumers — `has-open:` folds to `*:is([open], :popover-open, :open)`, the v4 mechanism rendering the inner's `&` as `*`), the metamorphic checker's conjunctive env carries the facts (the one-fact states bake the bit true), the GPUI and floem walks read the same Env semantics verbatim, the four plan-lane backends' probe paths honestly keep the facts off, the LSP labels the family, and the plan bridge LANDS the state slots — the family's own v0.x forms now exist, so every layer buckets into its `Test::State(...)` rule (the `checked:` precedent, no note rides) and the snapshot's strict reader round-trips the names. The completion surface carries the three spellings (`KNOWN_VARIANTS` beside the two-sided audit). Tests `tests/widget_states.rs` (13 pins, RED-first), bench `benches/widget_states.rs` (the env-answer walk 156.9 ns median, the resting-env path 26.06 ns), example `examples/widget_states.rs` (the fire, the resting quiet miss, the CSS head — headless).

### 🚀 Features — the widget-state family's CLI face (M2)
- `aeolus-cli`: `expand --open/--inert/--default`, the widget-state facts' headless flags (the element-state flags' sibling — element state needs no facts row, the `--dark` precedent): the flags set the Env bits, the walk answers the widget-state layers from them; unset flags are the quiet miss (never a reported exclusion), every family question is AFFIRMATIVE so one flag per FACT and no complement rides (the `--target` precedent — `--open` the disclosure/popover fact, `--inert` the frozen-subtree fact the compound head answers from, `--default` the form group's default-control fact). The allow list gates the flags to `expand` alone, and the Env build threads them beside the element-state bits. Tests `tests/cli.rs`'s widget-state battery (5 pins, RED-first — the firing answers, the quiet miss, the independent facts' mix), example `examples/expand_widget_states.rs` (the fact-bearing form beside the resting and mix forms, headless through the same `run` entry point the binary drives).

### 🛠 Fixed — docs drift (this handoff)
- `README.md`: both example indexes file the family's demos AND the five compile-side demos the per-crate index had skipped (`descendants`/`supports`/`preferences`/`capabilities`/`element_states` — present in the quickstart block, absent from the all-examples list) plus the `cn` face's demo (the command had zero index coverage; `examples/cn.rs` was never listed); the variant table files the widget-state row and the architecture plan's §3.5 map files the validation-quartet row it had skipped (the element-state row referenced "the validation quartet" beside, the map had no row for it).

### 🚀 Features — the element-state family (M1)
- `aeolus-compile` + `aeolus-ir` + the backends: the element-state family lands — the one-fact element-state static variants `target:`/`indeterminate:`/`autofill:`/`read-only:` beside the validation quartet (the differential evidence the tailwindcss v4.1.17 SOURCE's own state/forms blocks again — `packages/tailwindcss/src/variants.ts`, pinned verbatim: `staticVariant('target', ['&:target'])` and the three siblings; the family is the whole one-fact set the v0.x `State` surface carries — `enabled:`/`required:`/`optional:`/`in-range:`/`out-of-range:` stay the matches-neither boundary evidence 27 documented, a non-form element matches NEITHER half so one `Env` bit cannot answer them honestly, and `open:`/`inert:`/`default:` are the v4.1.17 spellings the v0.x surface has no slot for). The four spellings fold onto bare typed conditions (`Cond::Target`/`Cond::Indeterminate`/`Cond::Autofill`/`Cond::ReadOnly`, `source()` naming the canonical spelling, the probe tier's specificity 2 — selector conjunctions, not at-rules): the element-own answers are ELEMENT STATE the host answers, so the `Env` carries ONE fact per question — `url_target` (the fragment-target answer), `indeterminate` (the checkbox/radio answer beside `checked`), `autofilled` (the browser's filled-form answer) — and `read-only:` rides the editability fact's COMPLEMENT (`read_write`, the `validation_invalid` precedent: CSS's own reading for a non-editable element IS `:read-only`, so the resting env answers the `read-only:` query; `read-write:` has no v4.1.17 registration, the spelling stays out); the unset bit is the QUIET miss a selector that does not match is. Every match wall the fold's tier-2 arm feeds follows: the CSS arm renders the upstream pseudo-classes in BOTH walls (the built-in segment wall and the alias path's `apply_cond_shape` — the `:pseudo` head beside `:valid`, no at-rule ever renders), the `has-` inner grammar answers the states beside `checked:` (one grammar, three consumers — `has-target:` folds to `*:target`), the metamorphic checker's conjunctive env carries the facts (the one-fact states bake the bit true, the read-only complement bakes the editability bit false), the GPUI and floem walks read the same Env semantics verbatim, the four plan-lane backends' probe paths honestly keep the facts off, the LSP labels the family, and the plan bridge LANDS the state slots — the v0.x `State` surface already carries the four forms, so every layer buckets into its `Test::State(...)` rule (the `checked:` precedent, no note rides). The completion surface carries the four spellings (`KNOWN_VARIANTS` beside the two-sided audit). Tests `tests/element_states.rs` (12 pins, RED-first), bench `benches/element_states.rs` (the env-answer walk 138.7 ns median, the resting-env path 50.11 ns), example `examples/element_states.rs` (the fire, the resting complement, the quiet miss, the CSS head — headless).

### 🚀 Features — the element-state family's CLI face (M2)
- `aeolus-cli`: `expand --target/--indeterminate/--autofill/--read-write`, the element-state facts' headless flags (the validation flags' sibling — element state needs no facts row, the `--dark` precedent): the flags set the Env bits, the walk answers the element-state layers from them; unset flags are the quiet miss (never a reported exclusion), `read-only:` fires on the resting env's own complement answer and takes no flag (one flag per FACT, the `--invalid` precedent — `--target` the fragment-target fact, `--indeterminate` the checkbox/radio answer, `--autofill` the filled-form answer, `--read-write` the editability fact the complement reads). The allow list gates the flags to `expand` alone, and the Env build threads them beside the validation bits. Tests `tests/cli.rs`'s element-state battery (5 pins, RED-first — the firing answer, the quiet miss, the indeterminate/autofill facts, the read-only complement both ways), example `examples/expand_element_states.rs` (the fact-bearing form beside the resting and complement forms, headless through the same `run` entry point the binary drives).

### 🛠 Fixed — docs drift (this handoff)
- `README.md`: the workspace battery count re-measures against the actual `--all-features` run (1715 → 1732 — the claimed count did not move with the family's 17 pins; the honest run's sum is the claim now); the variant table files the element-state row and both example indexes file the family's demos AND the sort face's demo (the `sort` command had zero example coverage beside the other ten).

### 🧹 Micro-debt (this handoff)
- `crates/aeolus-compile/src/plan.rs`: the `PlanWriter::number` helper's dead receiver retires — the method took `&self`, documented nothing with it, and discarded it (`let _ = self;`) since the bare-number fold reads no lowering state; the helper becomes an associated function (the suggest fold's dead-`tokens` precedent, the eight `self.number(...)` call sites and the two multi-line shapes follow).

### 🎁 Consumer examples (additive)
- `crates/aeolus-cli`: `examples/sort.rs` — the `sort` face's consumer demo (the canonical registry order, the `--json` machine form beside the miss channel — headless through the same `run` entry point the binary drives); the command had zero example coverage beside the other ten.

### 🚀 Features — the validation states (M1)
- `aeolus-compile` + `aeolus-ir` + the backends: the form-state family lands — the element-state static variants `valid:`/`invalid:`/`user-valid:`/`user-invalid:` beside the `hover:`/`checked:` tier (the differential evidence the tailwindcss v4.1.17 SOURCE's own forms block again — `packages/tailwindcss/src/variants.ts`, pinned verbatim: `staticVariant('valid', ['&:valid'])` and the three siblings; v4.1.17 has NO `read-write` registration — the spelling stays out, the family's whole set is these four). The four spellings fold onto bare typed conditions (`Cond::Valid`/`Cond::Invalid`/`Cond::UserValid`/`Cond::UserInvalid`, `source()` naming the canonical spelling, the probe tier's specificity 2 — selector conjunctions, not at-rules): the constraint state is ELEMENT STATE the host answers, so the `Env` carries TWO facts — `validation_invalid` (the pair's two v4 values share one fact, the `rtl:`/`ltr:` precedent; CSS's own reading for a pristine field is `:valid`, so the resting env answers the `valid:` query) and `user_validated` (the interaction gate the HTML spec's `:user-*` pseudo-classes require — BOTH `user-` spellings stay quiet until the gate is set, past it the SAME constraint fact answers); the unset bit is the QUIET miss a selector that does not match is. Every match wall the fold's tier-2 arm feeds follows: the CSS arm renders the upstream pseudo-classes (the `:pseudo` head beside `:checked`, no at-rule ever renders), the metamorphic checker's conjunctive env carries the facts (the complement bakes the bit false, the gate rides WITH the constraint fact), the GPUI and floem walks read the same Env semantics verbatim, the four plan-lane backends' probe paths honestly keep the facts off, the LSP labels the family, the `has-` inner grammar answers the states beside `checked:` (one grammar, three consumers), and the plan bridge LANDS the state slots — the v0.x `State` surface already carries the four forms, so every layer buckets into its `Test::State(...)` rule (the `checked:` precedent, no note rides). The completion surface carries the four spellings (`KNOWN_VARIANTS` beside the two-sided audit). Tests `tests/validation.rs` (11 pins, RED-first), bench `benches/validation.rs` (the env-answer walk 185.8 ns median, the resting-env path 46.97 ns), example `examples/validation.rs` (the fire, the complement, the gated pair, the CSS head — headless).

### 🚀 Features — the validation states' CLI face (M2)
- `aeolus-cli`: `expand --invalid/--user-valid/--user-invalid`, the validation facts' headless flags (the capability flags' sibling — element state needs no facts row, the `--dark` precedent): the flags set the Env bits, the walk answers the validation layers from them; unset flags are the quiet miss (never a reported exclusion), `valid:` fires on the resting env's own complement answer and takes no flag (one flag per SPELLING, the `--rtl` precedent — `--invalid` the failing-constraints fact, `--user-valid` the gate WITH the holding fact, `--user-invalid` the gate WITH the failing fact). The allow list gates the flags to `expand` alone, and the Env build threads them beside the direction bits. Tests `tests/cli.rs`'s validation battery (4 pins, RED-first — the firing answer, the quiet miss, the valid complement both ways, the gate's own reading), example `examples/expand_validation.rs` (the fact-bearing form beside the resting and gate-only forms, headless through the same `run` entry point the binary drives).

### 🛠 Fixed — docs drift (this handoff)
- `README.md`: the workspace battery count re-measures against the actual static count (the claimed 1700 did not reconcile — the honest count claims itself); the ClassCell quickstart snippet reads the real API (`get(&str, &Config) -> Result<StaticProgram, _>` — the theme-bearing `get_theme` is the other face, and the `Result` is no longer silently dropped in the sample); the `aeolus-syntax` row states the `no_std + alloc` goal as a goal (the lexer's build is std today — the error impls carry `std::error::Error`); the validation rows and the example indexes file the family's demos.

### 🧹 Micro-debt (this handoff)
- `crates/aeolus-compile/src/suggest.rs`: `suggest_for`'s `tokens` parameter retires — the function accepted the theme pass's table, documented it as a live input, and discarded it (`let _ = tokens;`) since the composed fold moved onto the registry's own audited tables; the signature drops the dead parameter (the LSP engine's call site and the suggestion suite's six call shapes follow).

### 🎁 Consumer examples (additive)
- `crates/aeolus-cli`: `examples/cn.rs` — the `cn` face's consumer demo (conflict groups, the parent-eviction shape, the shadcn-shaped modifier call — headless through the same `run` entry point the binary drives); the command had zero example coverage beside the other ten commands.

### 🚀 Features — the direction family (M1)
- `aeolus-compile` + `aeolus-ir` + the backends: the text-direction family lands — the selector-side static variants `ltr:`/`rtl:` beside the media-feature tier (the differential evidence the tailwindcss v4.1.17 SOURCE's own `staticVariant` calls again — `packages/tailwindcss/src/variants.ts`, pinned verbatim: `staticVariant('ltr', ['&:where(:dir(ltr), [dir="ltr"], [dir="ltr"] *)'])` and the `rtl` sibling — the family renders as SELECTOR conjunctions, not at-rules). The two spellings fold onto the typed `Cond::Dir` payload (`Direction::Ltr`/`Direction::Rtl`, `source()` naming the canonical spelling): the direction is FRAME STATE, so the `Env` carries ONE fact (`direction_rtl`) with no probe seam and no sourceless note — `rtl:` on the bit, `ltr:` its COMPLEMENT (the `portrait:`/`landscape:` precedent — the feature's two v4 values share one fact, and CSS's own initial `direction` value is ltr, so the resting env honestly answers the `ltr:` query the way it answers `landscape:`); the unset bit is the QUIET miss a selector that does not match is. Every match wall the fold's tier-2 arm feeds follows: the CSS arm renders the upstream selectors (the probe tier's head, the `direction_probe` helper deriving both `:where(...)` texts from the bit — no at-rule head ever renders), the metamorphic checker's conjunctive env carries the fact (the complement bakes the bit false), the GPUI and floem walks read the same Env semantics verbatim, the four plan-lane backends' probe paths honestly keep the fact off, the codegen's frozen tokens name the typed bit, the LSP labels the family, and the plan bridge keeps the new `Direction` note's honest boundary (the v0.x surface has no text-direction slot). The completion surface carries the two spellings (`KNOWN_VARIANTS` beside the two-sided audit). Tests `tests/direction.rs` (10 pins, RED-first), bench `benches/direction.rs` (the env-answer walk 130.5 ns median, the resting-env complement path 46.6 ns), example `examples/direction.rs` (the fire, the complement, the quiet miss, the CSS head — headless).

### 🚀 Features — the direction family's CLI face (M2)
- `aeolus-cli`: `expand --rtl`, the direction fact's headless flag (the capability flags' sibling — frame state needs no facts row, the `--dark` precedent): the flag sets the Env bit, the walk answers the `rtl:`/`ltr:` layers from it; unset flags are the quiet miss (never a reported exclusion), `ltr:` fires on the resting env's own complement answer and takes no flag (one flag per FACT, the `--portrait` precedent). The allow list gates the flag to `expand` alone, and the Env build threads it beside the capability bits. Tests `tests/cli.rs`'s direction battery (3 pins, RED-first — the firing answer, the quiet miss, the complement both ways), example `examples/expand_direction.rs` (the fact-bearing form beside the resting form, headless through the same `run` entry point the binary drives).

### 🛠 Fixed — docs drift (this handoff)
- `README.md`: the workspace battery re-measures against the actual `--all-features` run (1687 → 1700 — the claimed number did not reconcile with any count method, the honest run's sum is the claim now); the examples indexes file the three new demos AND the two on disk the details list never named (`program_components` — the shadcn section the details block lacked entirely — and `lsp_document_highlights` beside its engine answer); the capability rows catch up with the direction family; "per-backend conformance" names the page's real scope (GPUI — the page never covered the other backends).
- `docs/AEOLUS_V1_ARCHITECTURE.md`: §3.5's variant table files the direction row (the family's landing beside the capability tier's); the `Env` sketch names the direction fact.
- `reports/08-benchmarks/BENCHMARKS.md`: the three "(2026-11, this handoff)" sections stamp their real run dates (the supports and preference runs 2026-11-15, the capability run 2026-11-16 — the evidence commits' own dates); the direction run files its medians.

### 🧹 Micro-debt (this handoff)
- `crates/aeolus-compile/src/lib.rs`: the registry-semantics pin's `const fn registry_version_is_pinned` was referenced only from the anonymous `const _` assert — a newer rustc's `dead_code` fired on it and the tree did not pass `-D warnings` (the pre-handoff tree carried the warning; the byte-walk moved inside the assert's own const block, the audit gate unchanged).

### 🎁 Consumer examples (additive)
- `crates/aeolus-lsp`: `examples/lsp_document_highlights.rs` — the click-to-find highlights' consumer face (`documentHighlight` driven headless through the same messages a client drives: the exact occurrences in document order, the refused class highlighting like a lowered one, the outside-candidate empty); the engine's tenth answer had zero example coverage beside it.

### 🚀 Features — the media-feature capability tier (M1)
- `aeolus-compile` + `aeolus-ir` + the backends: the media-feature tier COMPLETES — the input-capability families `pointer-fine:`/`pointer-coarse:`/`pointer-none:`/`any-pointer-fine:`/`any-pointer-coarse:`/`any-pointer-none:` and the orientation forms `portrait:`/`landscape:` land (the preference families' sibling; the differential evidence the tailwindcss v4.1.17 SOURCE's own `staticVariant` calls again — `packages/tailwindcss/src/variants.ts`, pinned verbatim: `@media (pointer: fine)` … `@media (any-pointer: none)`, `@media (orientation: portrait)`, `@media (orientation: landscape)` — v4.1.17 carries NO `hover-hover`/`hover-none` spelling, the plain `hover:` variant already wraps `(hover: hover)`). The eight spellings fold onto the SAME typed `Cond::Media` payload the preferences ride (`Pref::PointerFine`…`Pref::AnyPointerNone`, `Pref::Portrait`/`Pref::Landscape`, `source()` naming the canonical spelling): the capability facts are FRAME STATE, so the `Env` carries them with no probe seam and no sourceless note — six pointer bits asserted independently (a hit on one never implies another; the host asserts what it knows, the unset bit the QUIET miss a media query that does not hold is) and ONE `portrait` bit whose complement answers `landscape:` (the `motion-safe:` precedent — the feature's two v4 values share one fact, the orientation query always holding an answer the way reduced-motion does). Every match wall the fold's tier-4 arm feeds follows: the CSS arm renders the pinned texts (`CssAt::MediaPref`), the metamorphic checker's conjunctive env carries the facts (the complement bakes the bit false), the GPUI/floem runtime walks read the same Env semantics verbatim (the five plan-lane backends' probe paths honestly keep the facts off), the codegen's frozen tokens name the typed bit, the LSP labels the family, and the plan bridge keeps the `MediaFeature` note's honest boundary (the v0.x surface has no capability slot). The completion surface carries the eight spellings. Tests `tests/capabilities.rs` (12 pins, RED-first), bench `benches/capabilities.rs` (the env-answer walk 111.1 ns median, the quiet-miss path 54.1 ns), example `examples/capabilities.rs` (the fire, the complement, the quiet miss — headless).

### 🚀 Features — the capability tier's CLI face (M2)
- `aeolus-cli`: `expand --pointer-fine/--pointer-coarse/--pointer-none/--any-pointer-fine/--any-pointer-coarse/--any-pointer-none/--portrait`, the capability tier's headless face (the preference flags' sibling — frame state needs no facts row, the `--dark` precedent): the flags set the Env bits, the walk answers the `pointer-*`/`any-pointer-*`/`portrait`/`landscape` layers from them; unset flags are the quiet miss (never a reported exclusion), `landscape:` fires on the resting env's own complement answer. The allow list gates the seven flags to `expand` alone, and the Env build threads them beside the preference bits. Tests `tests/cli.rs`'s capability battery (4 pins, RED-first — the firing answer, the quiet miss, the landscape complement both ways), example `examples/expand_capabilities.rs` (the fact-bearing form beside the resting form, headless through the same `run` entry point the binary drives).

### 🛠 Fixed — docs drift (this handoff)
- `README.md`: the workspace battery re-measures (1629 → 1687; the count line had not moved with the two at-rule milestones' 42 pins); the examples indexes file the two new demos AND the one on disk the details list never named (`expand_supports`); the capability rows and the example index catch up with the tier.
- `docs/AEOLUS_V1_ARCHITECTURE.md`: the crate map's CLI row lists the ELEVEN commands (the `fmt` file face's row predated it); §8's P3 row names the macro crate `aeolus-macro` (not `aeolus-macros`); §3.5's probe-state row drops the `in-focus` sketch spelling (no v4 spelling folds it); the §3.5 `Env` sketch no longer claims a 4-byte shape with a two-field media half — the Env carries one field per frame-state question (the preference and capability facts included); the §3.5 capability row and §8's P4 row file the tier's landing.
- `reports/02-capability-matrix/CAPABILITY_MATRIX.md`: the slot-support table's header no longer claims "per v1 backend" over the two columns it carries (xilem, floem) — the header names its own scope.

### 🧹 Micro-debt (this handoff)
- `crates/aeolus-core/src/resolve.rs`: `named_atom`'s guard+unwrap double-check reads each fact once (`if let` bindings — the workspace's only non-test `.unwrap()` sites retire).
- `crates/aeolus-codegen-gpui/src/lib.rs`: five Debug-name→TokenStream folds (`format!("{x:?}").parse().expect(…)` over prop/breakpoint/structural/preference/condition) retire into one `ident_tokens` helper — the parse contract named once, `what` carrying the frozen unit's context.

### 🎁 Consumer examples (additive)
- `crates/aeolus-shadcn`: `examples/program_components.rs` — the non-button program tables' consumer face (`BadgePrograms`, `AlertPrograms`, `BasePrograms`: the input/card shapes beside the badge/alert variants, effective declarations through the session's token table); the tables had zero example coverage beside `program_tables`' Button demo.

### 🚀 Features — the supports-variant seam (M1)
- `aeolus-compile`: the `supports-[…]` family's answer lands — the `@supports` condition seam, the at-rule tier's second member beside the media forms (tailwindcss v4.1.17's own conditional-rule order `@media`/`@supports`/`@container` maps onto the flat tier model's 4 < 5; the family's only v4 spelling is the bracketed arbitrary form, and the differential evidence is the v4.1.17 SOURCE's own normalization — `packages/tailwindcss/src/variants.ts`'s `functional('supports', …)` arm — the corpus carries no supports class). `Cond::Supports` carries the RENDERED condition text: function-prefixed inners (`selector(:hover)`, `(width_>_400px)`) pass through verbatim with `and`/`or`/`not` connectives spaced and whitespace collapsed, colon-bearing declarations wrap in parens (`display:grid` → `(display:grid)`), bare features expand to the declaration probe (`display` → `(display: var(--tw))`, the source's own shorthand words), and the v4 arbitrary-value convention rides the fold (`_` spells the space a class string cannot carry). The honesty tightening: an empty inner, braces, `;`, backslash escapes and unbalanced parens refuse — a condition no `@supports` grammar accepts never folds, the typed unsupported-variant diagnostic is the report (`supports_condition` is the one-grammar public face the CLI fact flag reuses). The interp arm's FIFTH member: `SupportsProbe` asks the ONE question (does the render target satisfy this condition? — the capability set is NOT frame state, the structural seam's own reading), `SupportsState` is the built-in static-fact table (presence IS the answer), the dedicated `InterpNote::SupportsUnqueried` note keeps the sourceless boundary, and the entry-point matrix completes its fifth dimension (`interp_supports` through the FULL `interp_probes_containers_struct_descendants_supports`, 32 new free functions + the 32 `Compiled` methods, every one with its themed form). The CSS arm renders `@supports <condition>` heads from the SAME payload (`CssAt::Supports`, the sheet's at-rule grouping carrying it), the static-program token emitter carries the interned payload, the GPUI runtime walk keeps the family the quiet skip its no-note-channel semantics dictate, the LSP hover labels the family, and the plan bridge keeps the v0.x surface's honest note (`PlanNoteKind::SupportsCondition`, the v0.x surface has no at-rule slot) with the backends' report translates beside. Tests `tests/supports.rs` (25 pins, RED-first), bench `benches/supports.rs` (the capability-table hit 114.8 ns median over the supports-mixed Button slice, the sourceless note path 43.4 ns beside it), example `examples/supports.rs` (the fire, the answered miss, the sourceless note — headless).

### 🚀 Features — the supports seam's CLI face + the preference families (M2)
- `aeolus-cli`: `expand --supports <condition>`, the supports seam's headless face (the `--has` flag's sibling): repeatable rows normalized through the fold's OWN grammar (`supports_condition` — one grammar, two consumers, the bare `display:grid` spelling and the paren-wrapped `(display:grid)` land the SAME payload the variant folded), borrowed into the built-in `SupportsState` table the walk consults; a condition no grammar accepts is a CLI usage error, exit 2 (the refusal names the grammar). The dispatch carries the full five-seam matrix, and the usage text documents the flag. Tests `tests/cli.rs`'s `--supports` battery (4 pins, RED-first), bench `benches/expand.rs`'s `expand_supports_facts` (median 229.9 ns), example `examples/expand_supports.rs`.
- `aeolus-compile` + `aeolus-ir` + the backends: the media-feature preference families land — `motion-safe:`/`motion-reduce:`/`contrast-more:`/`contrast-less:`/`forced-colors:`/`print:`, the at-rule tier beside the media forms (the differential evidence the v4.1.17 source's own `staticVariant` calls, pinned verbatim: `@media (prefers-reduced-motion: no-preference)`, `@media (prefers-reduced-motion: reduce)`, `@media (prefers-contrast: more)`, `@media (prefers-contrast: less)`, `@media (forced-colors: active)`, `@media print`). `Cond::Media` carries the TYPED `Pref` bit (plan §3.5's "typed state bits, not string probes" row) — the preference facts are FRAME STATE (the user agent's own media-query answers, the tier `dark:` rides): the `Env` carries one fact per question, the plain walk answers with no probe seam and no sourceless note (an unset bit is the QUIET miss a media query that does not hold is — `motion-safe:` the COMPLEMENT of `motion-reduce:`), the CSS arm renders the pinned texts (`CssAt::MediaPref`), the metamorphic checker's conjunctive env carries the facts, the GPUI/Floem runtime walks read the same Env semantics, and the plan bridge keeps the v0.x surface's honest note (`PlanNoteKind::MediaFeature`). The completion surface carries the six spellings. Tests `tests/preferences.rs` (10 pins, RED-first), bench `benches/preferences.rs` (the env-answer walk 109.1 ns median, the quiet-miss path 56.8 ns), example `examples/preferences.rs`; the CLI face rides `expand`'s five preference flags (the `--dark` precedent — frame state needs no facts row) with its battery (3 pins, RED-first).

### 🛠 Fixed — docs drift (this handoff)
- `README.md`: the Examples index files the three demos this handoff lands AND the one on disk the index never named (`caps_table` — the generated GPUI capability table); the capability table rows catch up with the two at-rule families.
- `crates/aeolus-lsp/src/engine.rs`: three rustdoc links named the PRIVATE `compiled_candidates`/`lowered_markdown` items (a newer rustdoc's `-D warnings` fails the workspace doc build on them) — the spans are plain code text now.
- `crates/aeolus-ir/src/cond.rs`: the supports/preferece docs' cross-crate links (`crate::Env`, `Val::Kw`) resolved outside the crate's doc scope — plain code text now.

### 🧹 Micro-debt (this handoff)
- `crates/aeolus-core/tests/parity_golden.rs`: the colour-table's catch-all `unreachable!()` carried no message — it now carries its context (a fixture colour outside the four-name set is a fixture bug, not a resolve question; the CHANGELOG 0.19.0 pass's own convention).

### 🚀 Features — the descendant-probe seam (M1)
- `aeolus-compile`: the `has-*` family's answer lands — the seam the `AttributeUnqueried` note's descendant case carried as a diagnosed boundary since the attribute-probe fold. The interp arm's FOURTH member (the `AttrProbe`/`ContainerProbe`/`StructProbe` precedent, trait + built-in + entry points + the composed matrix): `DescendantKind` names the three `:has()` scopes the typed `Cond` surface folds (the element's own subtree, the `.group` ancestor's, the `.peer` sibling's — the label forms ride the CSS head only, so one scope answers per family), `DescendantProbe` asks the ONE question (does THIS scope's subtree contain a descendant matching the fragment? — the fragment is the rendered `:has()` inner payload `Cond::Has` carries verbatim, so a host compares like against like), and `DescendantState` is the built-in static-fact table (presence IS the answer — the zero-allocation per-frame form the `AttrState` bits take). A YES fires the layer; a source-answered NO is a QUIET skip (a `:has()` selector that does not match, never an exclusion); the sourceless forms keep the reported boundary under the DEDICATED `InterpNote::DescendantUnqueried` note (the per-seam convention the container and structural notes follow — the blanket attribute spelling retires for the family, the metamorphic checker's note vocabulary translated beside). The entry-point matrix completes: `interp_descendants` and the `*_probes`/`*_containers`/`*_struct` compositions through the FULL `interp_probes_containers_struct_descendants`, every one with its `_theme` form (16 new entry points, the same `*_with` convention), and the ergonomic half rides `Compiled` (16 new methods, `interp_descendants` through the full themed walk). Tests `tests/descendants.rs` (15 pins, RED-first — the fire, the answered-miss quiet skip, the data/aria compounds asking the same seam, the sourceless note's cond/props/text, the group/peer scope discrimination (a group answer never leaks into the peer question), the attribute/container/structural compositions, the FULL themeless and themed walks over one string, the matrix pairs and triples, the `Compiled` method face, the built-in table's presence semantics), bench `benches/descendants.rs` (the fact-table hit 257.9 ns median over the Button string's descendant layers, the sourceless note path 100 ns beside it — the consult shape the attribute seam's ~26 ns/layer verdicts set), example `examples/descendants.rs` (the fire, the scope discrimination, the sourceless note — headless).

### 🚀 Features — the descendant-probe seam's CLI face (M2)
- `aeolus-cli`: `expand --has kind=fragment`, the descendant-probe seam's headless face (the `--container name=rem` flag's sibling): repeatable rows naming the `:has()` scope (`element` — the element's own subtree, `group` — the `.group` ancestor's, `peer` — the `.peer` sibling's, the [`DescendantKind`] spellings) and the rendered `:has()` inner fragment the condition carries, borrowed into the built-in [`DescendantState`] table the walk consults. Without any, `expand` keeps the sourceless shape — descendant layers report the dedicated `DescendantUnqueried` note instead of answering. The typed input gate: a missing value, a `kind=fragment` shape miss, an unknown kind (the error names the three scopes) and an empty fragment are CLI usage errors, exit 2 — the `--container` arm's own refusal discipline. The dispatch carries the full four-seam matrix (theme × containers × structural × descendants — the `*_descendants` forms the M1 seam landed), and the usage text documents the flag beside `--container`/`--child`. Tests `tests/cli.rs`'s `--has` battery (4 pins, RED-first — the sourceless note, the firing answer over `--has element=*:focus`, the scope discrimination (a group answer never rides the peer question, the answered miss quiet), the malformed/unknown-kind refusals), bench `benches/expand.rs`'s `expand_descendant_facts` (median 256 ns — the same walk the interp bench keys on), example `examples/expand_descendants.rs` (the sourceless note, the firing answer, the scope discrimination — headless through the same `run` entry point the binary drives).

### 🛠 Fixed — docs drift (this handoff)
- `Cargo.toml`: the `repository` URL read `aeolus-rs/aeolus` while every consumer-facing page (the README's badges, the git-dependency snippets) says `elcoosp/aeolus` — the manifest now reads the real home.
- `README.md`: a "Feature flags" section documents the three additive flags the manifests self-document only (`tracing`, `miette` across the pipeline crates, `verbose-diagnostics` on the macro); the `has-*` capability rows and both examples indexes catch up with the seam and its CLI face; the workspace battery re-measures (1610 → 1629).
- `docs/AEOLUS_V1_ARCHITECTURE.md`: the §3.5 `has-*` row moves from "diagnosed, honest failure kept" to the seam's landing; the `*:` arbitrary-variant half stays the diagnosed boundary.
- `reports/08-benchmarks/BENCHMARKS.md`: the descendant-seam section files the run's medians (the fact-table hit 257.9 ns, the sourceless note path 100 ns, the CLI face's walk 256 ns).

### 🧹 Micro-debt (this handoff)
- `crates/aeolus-core/src/cache.rs`: the FxHasher's infallible chunk cast carried a bare `.unwrap()` — it now carries its context (`expect` naming the chunks_exact(8) invariant, the comment beside; the lint exception's rationale stands in the workspace manifest).

### 🚀 Features — the class-sort seam's editor face (M1)
- `aeolus-lsp`: the format-on-save answer lands — `textDocument/formatting`, the class-sort seam's editor face (the §5.5 "class sorting" line's third surface, beside `sort`'s literal-string form and the new CLI file face). The engine's `format(uri)` walks the extraction seam's strings and rides ONE edit per class string whose order is not yet the registry's canonical one — the sort seam's own canonical join as the new text ([`EngineTextEdit`]: the string CONTENT's byte-true UTF-16 range, never the quotes around it), the edit list exactly the diff an editor applies. The walk is deliberately THEME-LESS (`aeolus_compile::sort` — the order is a shape-level fact, the sort module's own boundary: a themeless token boundary sorts exactly like its lowered sibling, so the formatter needs no table), and three boundaries the answer keeps honestly: an already-canonical string answers NO edit (the no-op contract a format-on-save rides), the canonical join normalizes the string's whitespace run to single spaces (the seam's documented canonical form), and the classes the seam cannot order ride the TAIL in source order and never block the edit — but the misses themselves are NOT this answer's channel (the diagnostics publish them, push and pull; the formatter carries edits only). The wire half advertises `document_formatting_provider` (the plain `true` form — the answer is the engine's whole-document walk) and maps the protocol's `DocumentFormattingParams` onto `TextEdit`s (the formatting options accepted and left to the shape-level walk: the canonical order is a registry fact, not an indentation choice). Tests `tests/engine.rs`'s format battery (6 pins, RED-first — the canonical-order edit with its byte-true content-span range, the UTF-16 column truth over the multi-byte `é` fixture, the clean-document no-op and the no-strings empty, the whitespace-run normalization, the honest tail (known classes reorder, the tail rides source order, no findings riding), purity + the unknown-URI/closed-document gates) and `tests/server.rs`'s wire battery (5 pins — the capability advertisement, the decoded edit over the fixture (line 1, column 13 → 40, the canonical `newText`), the clean document's empty array, the before-initialize quiet drop, the malformed-params null). Bench `benches/lsp.rs`'s `format_walk` (median 1.86 µs over the three-string fixture — the themeless sort walk, an order under any format-on-save budget), example `examples/lsp_format.rs` (the edits, the no-op string, the whitespace normalization, applied headless through the same messages a client drives).

### 🚀 Features — the class-sort seam's file face (M2)
- `aeolus-cli`: `aeolus fmt`, the ELEVENTH command — `sort`'s batch form and the §5.5 line's CLI face: read ONE file, walk its class strings through the extraction seam, rewrite every string whose order is not yet the registry's canonical one with the sort seam's own canonical join, the rest of the file byte-for-byte. Default: print the rewritten file to stdout (a shell redirects; the file itself is never touched). `--check`: print NO rewrite — one `path:line:col "before" -> "after"` line per out-of-order string, the one-based line and UTF-16 column the string content's own start (the lint convention's caret form, the engine's own conversion shape) — and exit 1 when any string needs an edit: the CI drift gate a format-on-save pre-commit rides. The misses NEVER block (the formatter's job is formatting): a class the seam cannot order rides the tail in source order and reports the SAME typed, coded lines `sort` reports, on stderr, exit 0 — the lint gate is `check` (the two channels' own split). Usage errors exit 2 (exactly one positional file — a formatter rewrites one file; unknown flags; unreadable file naming the path). Tests `tests/cli.rs`'s fmt battery (10 pins, RED-first — the in-place rewrite beside the byte-identical canonical string, the rest-of-the-file byte-for-byte walk (comment, prose string, structure untouched), the clean-file no-op with quiet stderr, the --check report's position and before→after pair with the drift exit, the clean --check's silence, the miss channel (exit 0, the coded lines on stderr), the exactly-one-file gate, the unknown-flag and unreadable-file refusals, the documented command). Bench `benches/fmt.rs` (the whole-file walk 2.65 µs over the three-string fixture, the single-string sort at the Button slice's real shape 4.24 µs), example `examples/fmt.rs` (the drift report beside the rewrite, headless in-process).

### 🧹 Micro-debt (this handoff)
- `crates/aeolus-lsp/src/engine.rs`: the per-candidate compile scaffold retires into the shared `compiled_candidates` walker — the colour swatches, the semantic legend and the inlay hints each carried the SAME scan → per-candidate compile → absolute-span loop, three parallel scaffolds the shared-locator precedent says should be one walk (the walker folds each answer's boundaries through its own closure; the boundary rustdocs stay the answers'). The bench medians hold (the swatch 4.7 µs, the legend 5.0 µs, the hints 11.3 µs — same numbers, one walk).
- `crates/aeolus-cli/tests/cli.rs`: the fmt battery's temp-dir helper takes the caller's own label — the helper's `line!()` named every caller alike, and parallel test threads shared one directory (the race fired under `--all-features`; the check battery's pid+line pattern, label-spelled).

### 🛠 Fixed — docs drift (this handoff)
- `README.md`: the tooling block, the tooling table (`aeolus fmt`'s row) and the LSP row catch up with the two new faces (the formatting answer, the fmt command); the crate map's CLI row lists the eleven commands; the examples index files the two demos (`lsp_format`, `fmt`); the workspace battery re-measures (1589 → 1610).
- `docs/AEOLUS_V1_ARCHITECTURE.md`: the §5.5 tooling-parity line names the class-sort seam's two remaining faces (the editor's formatting answer, the CLI's `fmt` file face with its `--check` gate).
- `reports/08-benchmarks/BENCHMARKS.md`: the LSP table gains `format_walk` (re-run 2026-11-16) and a new class-sort file-face section files the fmt benches.

### 🚀 Features — the did-you-mean suggestion seam (M1)
- `aeolus-compile` (+ `aeolus-lsp`): the "did you mean" surface the data crate's docs always promised lands as the miss-side SEAM — `aeolus_compile::suggest`. The v0.x oracle answers unknown classes with a bounded Levenshtein walk over a hand-kept prefix list (`aeolus-core`'s `suggest`); the V1 pipeline's typed diagnostics carried only the prose help. The new module is the trait seam (the `DynStore`/`AttrProbe` precedent): `SuggestionSurface` names the contract — `source()` (the honesty tag) and `nearest(token, cap)` (the audited spellings within the bounded distance, the surface's own canonical order the rank) — and the built-ins implement it over their own tables (`RegistrySpellings` over the rows' roots and keyword/composite spellings, `TokenSpellings` over the config's own table names, `VariantSpellings` over the closed `KNOWN_VARIANTS` set, `CustomSpellings` over the registered customs — the host vocabulary the built-ins cannot know, the trait's reason to exist). Every emitted suggestion is a spelling the pipeline LOWERS (the audited tables are the sources, never a guessed word — the completion's own one-source-of-truth rule at the miss side; the battery pins recompilability). The diagnostic-keyed composition (`suggest_for`) keys on the typed miss: the utility half's folds answer `UnknownRoot`/`ShapeRejected`/`UnsupportedValue` with the registry and customs, the variant half answers `UnsupportedVariant` with the closed set (the SINGLE-segment stack only — a composed stack has no honest single-spelling edit), and the whole-token refusals (lex, importance, modifier, negation) answer empty (the help text is the story). The corrections compose onto the FULL token: the variant prefix rides back on and the raw's own `/modifier` tail survives (the user's `/50` is not dropped by the fix). The composed fold rides the registry alone for the colour half — the audited keyword list IS the registry's own table (the token surface dedupes onto it; the honest finding `bg-primary-1` refuses at the shape gate — a host token beyond the audited list never lowers — is pinned as the audit-gate test). The bounded distance is the plain Levenshtein with the early abandon (the oracle's own algorithm, bounded — the length pre-gate, the per-row minimum; cap 2, one typo or the transposition the plain distance counts twice). Tests `tests/suggest.rs` (20 pins, RED-first — the registry sibling, the recompile-clean contract, the sibling keyword, the variant-prefix and modifier-tail composition, the themeless and non-colour boundaries, the single-segment variant boundary, the whole-token refusals, the far-miss silence, the customs surface, the ranked/capped/deterministic fold, the distance cap, the host surface folding beside the built-ins, the registry collision win, the token surface's own names, the unaudited name never suggesting). Bench `benches/suggest.rs` (the registry sweep 1.66 ms, the composed fold 1.68 ms, the variant fold 5.1 µs, the two-surface raw fold 1.45 ms — the COLD miss path only, the oracle's own precedent), and the editor face: `aeolus-lsp`'s `code_actions(uri, line, character)` (the cursor's candidate through the seam, each finding riding the diagnostic it repairs, the CLI-shaped title and the corrected token) with the wire half's `textDocument/codeAction` (`CodeActionProviderCapability`, the QUICKFIX kind, the edit over the finding's own range, the `wire_diagnostic` mapping shared with the push channel). Engine battery 5 pins, wire battery 6 pins (RED-first), bench `code_actions_walk` 1.67 ms, example `examples/lsp_code_actions.rs` (the keyword fix, the variant fix with the composed range, headless).

### 🚀 Features — the LSP document highlights (M2)
- `aeolus-lsp`: the editor's click-to-find lands — `textDocument/documentHighlight` (LSP 3.16's occurrences). The engine's `document_highlights(uri, line, character)` keys on the candidate under the cursor through the SHARED locator (`candidate_span_at` — the extraction seam's strings and the lexer's own candidate spans, one implementation the hover and the highlights walk; the hover's inline copy retires), and answers every candidate whose text equals the cursor's, its byte-true range, document order. The highlight is TEXTUAL, not semantic: a refused class's occurrences highlight exactly like a lowered one's (the editor's own click-to-find, no compile verdict riding), and the cursor's own occurrence rides the answer (the wire contract the editor renders). The wire half advertises `documentHighlightProvider` and maps the answer onto the protocol's `DocumentHighlight` (no kind — a plain highlight, the honest shape). Tests `tests/engine.rs`'s highlight battery (3 pins — the exact occurrences with the variant-prefixed neighbour excluded, the refused class highlighting like a lowered one, the outside-candidate/unknown-URI empties) and `tests/server.rs`'s wire battery (3 pins — the capability advertisement, the decoded occurrence range, the quiet drop). Bench `benches/lsp.rs`'s `document_highlights_walk` (median 1.4 µs — the shared locator plus the equality walk, an order under the hover's own cost).

### 🛠 Fixed — docs drift (this handoff)
- `README.md`: the tooling row's LSP line and the examples index catch up with the ninth and tenth requests (the quick fixes and the document highlights), and the workspace battery count re-measures with the seam's suites.
- `docs/AEOLUS_V1_ARCHITECTURE.md`: the §5.5 LSP line names the suggestion seam and the two new editor answers.
- `reports/08-benchmarks/BENCHMARKS.md`: the LSP table gains the two new walks and the suggestion seam's own run (the compile crate's miss-path folds).

### 🧹 Micro-debt (this handoff)
- `crates/aeolus-lsp/src/engine.rs`: the hover's inline candidate-location walk retires into the shared `candidate_span_at` locator — the ONE walk the cursor-keyed answers (hover, quick fixes, highlights) share, the keyword-alignment precedent (the LSP's own no-parallel-matcher rule).
- `crates/aeolus-cli/src/lib.rs`: the `Finding.from` field retires — constructed at all three sites, read by none (`render()` prints `token: detail`, the JSON keys on `token`/`kind`/`detail`, the file attribution rides `cmd_check`'s tuple) — the publish=false crate's dead weight, gone.

### 🚀 Features — the generated capability table (M2)
- `xtask` (+ `aeolus-codegen-gpui`): the §3.7 generator slice lands — the sketch's `caps_generated.rs` is in-tree and drift-gated at last. The driver's new `gpui-caps` artifact renders the per-longhand verdict table from the backends' OWN capability impls (`Caps` — the publish-the-emitter's-verdicts rule the conformance page rides) into `crates/aeolus-codegen-gpui/src/caps_generated.rs`, the committed module the CRATE now consumes (`aeolus_codegen_gpui::PROP_CAPABILITIES` — one `PropCapability` row per IR longhand in `Prop::ALL`'s own cascade order: the tier plus, for a refused longhand only, the typed boundary reason the emit gate's per-prop answer carries — the text-dependent verdicts say so honestly, their reason naming the annotation the verdict is the ROW TEXT's). One partition, three renderings, all pinned equal: the `Capability` impl, the generated `docs/CONFORMANCE.md` page, and the committed table. The wire-up: the crate's `tests/caps_generated.rs` battery (5 pins, RED-first — the row-for-row `Prop::ALL` coverage in order, the table IS the impl's verdicts, the conformance partition agreement, the refused-reason/expressible-empty contract, the §3.7 rows' typed boundaries: ring polyfills, the motion row's own text, the z-index fallback, the direction row's annotation refinement) fails first; the driver's battery grows the focused pair (`gen-gpui-caps` / `verify-gpui-caps`, 4 pins — the listed path, the temp-root write + focused clean verify, the drift exit code over a corrupted byte, the REAL tree's read-only clean check) and the global walk grows the fourth artifact (the clean/missing counts the battery pins, updated beside). The gates' honesty stands: no CI job exists, the drift gate is the test battery and the pre-commit `verify-gpui-caps` walk (the verify-tables story kept); the sketch's SURFACE-DIFF half stays the hand-maintained rows' duty at pin time — the architecture doc's §3.7 says so now.

### 🚀 Features — the LSP inline hints (M1)
- `aeolus-lsp`: the editor's SEVENTH answer lands — `textDocument/inlayHint` (LSP 3.17's inline answers). The engine's `inlay_hints(uri)` walks the extraction seam's strings per candidate through the SAME compile entry points the CLI and the macros run (the colour-swatch walk's own shape — one candidate per call, the spans from the lexer), and every candidate that lowers cleanly rides ONE hint at its END position: the CSS arm's own declarations compacted to the wire's single-line label (the sheet's first rule body, each declaration's trailing `;` folded into the `"; "` join — a paraphrase never enters the label), the hover's markdown riding as the tooltip. The hover's lowered rendering is the shared `lowered_markdown` now — one answer shape at two surfaces (the wire-diagnostic mapping's own single-source rule), so the inline label and the cursor hover can never disagree about what a class lowers to. The refused candidates never hint: their story is the diagnostics' and the semantic legend's (a refused class has no CSS to show, the honest boundary), and the theme-less config answers the same (the CSS arm renders the lowered class either way). The wire half advertises `inlayHintProvider` (no resolve round-trip — the answer is final, the completion's own contract) and maps each hint onto the protocol's `InlayHint` (the class's end position, the string label, the type-annotation kind, the markdown tooltip); the params' visible range is accepted and answered document-wide — no incremental model, the pull diagnostics' own contract. Tests `tests/engine.rs`'s hint battery (3 pins, RED-first — the lowered CSS at the class's end with the single-line label and the markdown tooltip, the themed walk's resolved bytes, purity + the unknown-URI/clean/prose/refused boundaries + the theme-less walk) and `tests/server.rs`'s wire battery (4 pins — the capability without the resolve round-trip, the decoded hint over the two-candidate fixture (one hint, the refused candidate absent, the end position, kind 1, the markdown tooltip), the before-initialize quiet drop, the malformed-params null). Bench `benches/lsp.rs`'s `inlay_hints_walk` (median 11.6 µs over the three-string fixture — the per-candidate compile plus the CSS arm's render, an order under the editor's own framing budget), example `examples/lsp_inlay_hints.rs` (the two hints, the refusal's diagnostics, headless through the same messages a client drives).

### 🚀 Features — the LSP semantic tokens (M1)
- `aeolus-lsp`: the editor's FIFTH answer lands — `textDocument/semanticTokens/full` (LSP 3.16's highlighter feed). The engine's `semantic_tokens(uri)` walks the extraction seam's strings per candidate through the SAME compile entry points the CLI and the macros run (the document-colour walk's own shape — one candidate per call, the spans from the lexer), classifying each candidate's outcome onto the wire's two-entry legend: a clean lower is the protocol's own `class` type, a typed refusal the server's custom `refused` type (an editor that does not know the custom entry renders neutral — the honest degradation). The answer rides in document order (the delta encoder's own precondition, pinned), and the wire half encodes the protocol's delta form (the first token's deltas absolute, the later ones relative to the previous token's start, the length the column arithmetic — a candidate never spans a line, the lexer's own whitespace split). Tests `tests/engine.rs`'s semantic-token battery (4 pins, RED-first — the classification over the lowered/refused fixture with byte-true ranges, the document order, purity + the unknown-URI/clean/prose boundaries) and `tests/server.rs`'s wire battery (5 pins — the legend + full advertisement, the decoded deltas over the two-candidate fixture, the clean document's empty array, the before-initialize quiet drop, the malformed-params null). Bench `benches/lsp.rs`'s `semantic_tokens_walk` (median 4.77 µs over the three-string fixture), example `examples/lsp_semantic_tokens.rs` (the legend, the decoded highlights).

### 🚀 Features — the LSP colour presentations (M2)
- `aeolus-lsp`: the `colorProvider` capability's paired half lands — `textDocument/colorPresentation` (the seventh request; the capability advertised the swatches but never answered the picker's spellings). The engine's `colour_presentations(rgba)` walks the config's own table BACKWARDS: the resolves run ONCE per (token, step) over the audited `OPACITY_STEPS` scale, and an EXACT light-tier match spells on every colour-group root the registry names (the completion source's own surface — the same gate that makes a suggested token class a lowered token class); the bare spelling IS the 100% step, the `/opacity` modifier rides only below it. The answer carries no document argument — the spellings ride the config's table, the one engine question the document map does not gate — and the theme-less config and an unmatched colour answer the honest empty (never a guessed spelling). The wire maps the protocol's 0..1 fractions back onto the bytes (the swatches' channel-for-channel mapping, inverted) and rides each spelling as the edit over the range the picker names. Tests `tests/engine.rs`'s presentation battery (4 pins, RED-first — the differential round trip from the swatch, the /50 round trip through the baked table, the theme-less/unmatched boundaries, purity + dedupe) and `tests/server.rs`'s wire battery (4 pins — the swatch→presentation round trip with the edit over the requested range, the unmatched empty, the quiet drop, the malformed null). Bench `benches/lsp.rs`'s `colour_presentations_walk` (median 493 µs — the resolves once per token × step, the row walk only spells), example `examples/lsp_colour_presentations.rs` (the swatch, the spellings, the honest empty).

### 🚀 Features — the macro demo's theme fixture (filed late)
- `aeolus-macro`: the `variants` example demonstrates the whole macro surface — `variants!`'s cva compound/slots tables, `tw_cn!` and `theme!` with the `theme.css` fixture riding the example dir (the demo landed 2026-11-15; this entry files it — the index listed the example before the changelog did).

### 🛠 Fixed — docs drift (this handoff)
- `CHANGELOG.md`: the doubled `## [Unreleased]` header dedupes.
- `README.md`: the MSRV badge's blanket "1.96 backends" names the split's truth (the five framework backends at 1.96, GPUI at the core's own 1.88); the "All examples" details block regains the seven examples the quickstart block listed but the index dropped (`backdrop_filters`, `filters`, `grid_flow`, `hints`, `container_query`, `decoration`, `struct`) and files the two new demos.
- `.github/workflows/ci.yml`: the `msrv-core` job checks `aeolus-gpui` beside the other 1.88 crates — the crate declares `rust-version = "1.88"` but rode neither MSRV job.
- `crates/xtask/src/lib.rs`: the `verify-tables` help stops calling itself "the CI gate" — no such job exists; the gate is the test battery that pins the drift (`write_mode_populates_a_temp_root_and_verify_stays_clean`, `verify_reports_drift_with_the_drift_exit_code`).

### 🧹 Micro-debt (this handoff)
- `crates/aeolus-lsp/src/server.rs`: the `Session.shutdown` flag stops being dead state — a request after the client shut down rides the same quiet-drop contract the unknown methods keep (a repeated `shutdown` still answers null: a client waiting on it must not hang), test-pinned in `tests/server.rs`.
- `crates/aeolus-lsp/tests/engine.rs`: the vestigial `hit` computation in the UTF-16 range test retires — the pin's real assertion is the refused range's columns; the discarded `let _ = (hit, …)` binding was scaffolding.

### 🚀 Features — the plan bridge's consumer face (M1)
- `aeolus-cli`: the §7 migration seam's CLI half lands — `aeolus plan`, the tenth command, lowers a class string through the V1 pipeline's `Target::Plan` arm ([`emit_plan`]/[`emit_plan_theme`]) into the v0.x `StylePlan` the five backend crates consume, and prints the SAME canonical snapshot JSON `resolve` emits — one artifact, two compilers, both headless. The themed half rides the BAKED tiering (the bridge's own colour story: the baked sRGB values convert through the audited `Color` parser into the plan's Oklab slots; the shared `--theme` seam splits into `theme_source_or_report` + the per-command table form — `runtime` for `explain`/`build`, `baked` for `plan`), the themeless half keeps the colour tokens as typed boundaries (the pipeline's honest default), `--summary` renders the slot summary, `--json` embeds the snapshot verbatim beside both report arrays, and `--fail-on` maps the diagnostics' TYPED kinds onto the check-style exit codes (default `none`, the emit-first semantics `build` has). The two honest channels never silence: every per-class miss rides stderr as the coded diagnostic (the `explain` shape), every longhand that lowered into the IR but has no v0.x slot rides as the plan arm's typed note — notes are the boundary channel, they report and never fail (the interp arm's `InterpNote` precedent). The shared `Args` bag grows the `--fail-on` value flag (allow-gated per command, the typed refusal for a bad mode). Tests `tests/cli.rs`'s `plan` battery (10 pins, RED-first — the themed bridge snapshot with the hover layer, the canonical artifact across class order, the differential slot-for-slot agreement with `resolve`'s own snapshot, the summary with the slot-boundary note on stderr, the themeless token boundary keeping the background slot honestly unset while `p-4` lowers, the `shape_rejected` code on stderr, the JSON envelope with the empty arrays and the `slot_unsupported` note channel, the fail-on gate on the typed kinds (`unknown` fires on the shape-rejection half of the unknown family), the usage errors (no classes, the `default` spec's typed refusal, unknown flags, unreadable theme CSS), and the documented command). Bench `benches/plan.rs` (the themeless bridge, the themed bridge's colour conversion, the diagnostics path — the shadcn Button slice the pipeline benches key on), example `examples/plan.rs` (the snapshot + notes + the two lanes' background slot side by side). The USAGE drift fixes ride the same file: `expand`'s line gains the implemented `--child`/`--empty` flags it never listed, and the FAIL-ON line stops claiming one default — `unknown (default)` → the per-command truth (`check unknown; build/plan none`, `conflict` marked check-only).

### 🚀 Features — the LSP pull diagnostics (M2)
- `aeolus-lsp`: the editor's FIFTH question lands — `textDocument/documentDiagnostic` (LSP 3.17's pull model) beside the push channel. The engine's `analyze` walk IS the report (one walk, two transports — the pulled findings carry the SAME codes and ranges the didOpen/didChange publishes carry), and the state machine answers the request with the protocol's `Full` report shape (no result ID: the engine never caches, so every pull recomputes — the honest contract a refresh or a newly focused file costs). The capability half rides `initialize` (`diagnosticProvider`: per document, no inter-file model, no workspace pull — the document map is the answer's scope), and the wire mapping is now ONE function both channels share (`wire_diagnostic` — the publish half's own mapping lifted to the single source, the keyword-alignment precedent). Tests `tests/server.rs`'s pull battery (6 pins, RED-first — the capability with its two `false` scopes, the full report with the WARNING tier's severity/source/code/range, the clean document's empty report, the pull-mirrors-push parity (same length, same codes, channel to channel), the before-initialize quiet drop, the malformed-params null) and `tests/engine.rs`'s answer-shape pin (the analyze walk IS the pull's source: one coded finding, byte-true range, the empty answer). Bench `benches/lsp.rs`'s `document_diagnostics_pull` (the whole state-machine path — params parse, analyze, wire mapping, report serialization — over the synthetic document), example `examples/lsp_diagnostics.rs` (the capability, the pulled report, the clean document's empty answer, headless through the same messages a client drives).

### 🛠 Fixed — docs drift (this handoff)
- `crates/aeolus-data/src/rows.rs`: the `ROWS` doc's self-contradictory count catches up with the decoration landing (184/209 → **213** rows, the decoration family's three rows named; the audit-count pointer now names `registry::row_count()`).
- `crates/aeolus-data/src/codegen.rs`: the roots-index doc's pre-decoration counts catch up with the generated artifact beside it (154 roots / 178 rows → **182 roots / 213 rows**, matching `ROOT_ENTRIES`' own length).
- `crates/aeolus-ir/src/prop.rs`: the module header still said the mask is "one `u128` bitset … 128 bits cover the full v4 longhand set" — the filter/backdrop layer pushed the surface past one word (`PropMask([u128; 2])` since); the header now names the two-word form and why it grew.
- `reports/08-benchmarks/BENCHMARKS.md`: the lookup story's seed/row prose catches up with the regenerated index (154 roots / 178 rows → 182 / 213, the recorded run's row set marked as of its date).
- `docs/AEOLUS_V1_ARCHITECTURE.md`: the §3.2 table's stale rows catch up — the CLI row lists the ten commands (check/sort/explain/expand → resolve/css/diff/cn/check/explain/sort/expand/build/plan), `aeolus-macros` → `aeolus-macro` with `tw_cn!` in the list, and the xtask row's artifact over-claim ("linter tables, class-sort order, test fixtures") names the three artifacts the generator actually ships; §3.7's phantom generator claim is honest now — `xtask gen-gpui-caps` + `caps_generated.rs` + "CI fails on drift" were DESIGNED but never landed (no such file, no such gate): the section says so and names the hand-maintained tables as today's state, keeping the sketch as the planned slice; §4's cold/warm "How it's enforced" cells stop naming criterion (no criterion in the tree — divan is the bench idiom everywhere).
- `.github/workflows/ci.yml`: the test matrix's muda comment was wrong in both directions — the `[patch.crates-io]` is unconditional (real muda never builds on any target); the comment now says what is true (floem's own Linux gate makes the patch inert elsewhere).
- `CONTRIBUTING.md`: the local fuzz gate lists the seventh target CI sweeps (`compile_interp_emit` — SIX → SEVEN).

### 🧹 Micro-debt (this handoff)
- `crates/aeolus-rt/src/lib.rs`: two dead re-exports retire — `interp_theme_probes_containers` and `program_style_struct` rode the host-seam wall with zero `aeolus_rt::` consumers in the workspace (the compile tests import them from `aeolus_compile` directly); a re-export nobody names is scaffolding, not surface.
### 🚀 Features — the LSP completion seam (M1)
- `aeolus-compile` (+ `aeolus-lsp`): the editor half's completion lands — the engine's THIRD question ("what could go here?") beside diagnostics and hover, and the compile crate's own audited spelling surface with it. `aeolus-compile` exposes `KNOWN_VARIANTS` (the closed, CONCRETE variant spelling set `classify_builtin` answers as-is — 42 entries across the probe states, the mode, the parent probes with the named-label demo, the structural bare spellings, the breakpoints, the min/max and container forms with the `@md/card` demo, and the attribute-probe families' demo spellings; the open payload grammars are represented, not enumerated) and publishes `COLOUR_GROUPS` (the colour gate the value pass audits, now the editor's source of truth too) — the two-sided drift test pins the surface (`variant::completion_tests`: every entry folds, no duplicates, and every base spelling the fold's match walls accept is present). The engine's `complete(uri, line, character)` answers the fragment at the typing edge from the compiler's OWN audited sources, never a guess: the registry's bare roots (prefix match — the root IS the class), the keyword/composite tables (root + `-` + suffix, the typed remainder filtering), the colour roots' theme tokens THROUGH the compile walk's own gate (a suggested token class is a lowered token class; the T1 tier's detail names its dark pair), the audited default-step scale (the `opacity` root's own table — the previously-unconsumed audited surface gets its consumer), and the closed variant set. The dynamic scales stay unnamed (the spacing multiples, the fractions — every number is a legal spelling, no closed table exists; the no-invented-data rule the pipeline keeps). The wire half advertises `completionProvider` (the grammar's own `-`/`:` trigger characters, no resolve round-trip) and answers a final `CompletionList` whose items carry the protocol kinds (utility `Property`, token `Value`, variant `Keyword`) and the registry's canonical order as the sort key. Two boundaries the popup keeps honestly: the extraction seam's prose pre-filter applies (a string with no `-`/`:` token is not a class string — the popup fires where the pipeline looks), and the WORD is not lex-validated (a half-typed `flex-` is not a legal candidate yet — the popup must answer while typing; the lexical gate belongs to the compile walk).
- Tests `aeolus-lsp/tests/engine.rs`'s completion battery (9 pins, RED-first — the variant fragment from the closed set, the keyword spellings with their longhand details, the theme tokens through the default bake, the bare roots and their fixture's prose-filter boundary, the empty utility half's bounded answer, rank order strictness + dedupe + purity, the theme-less config's keyword-only answer, the empty answers outside the pipeline) and `tests/server.rs`'s wire battery (4 pins — the capability with its trigger characters, the `CompletionList` round-trip with kinds/detail/sortText, the before-initialize quiet drop, the malformed-params null). Audit `crates/aeolus-compile/src/variant.rs`'s `completion_tests` (the fold-covers-set and set-covers-fold pins). Bench `crates/aeolus-lsp/benches/lsp.rs`'s `completion_fragment` (the typing-edge path beside analyze and hover), example `crates/aeolus-lsp/examples/lsp_completion.rs` (the variant, token and keyword fragments beside the honest empty answer).

### 🚀 Features — the LSP colour-swatch seam (M2)
- `aeolus-lsp`: the engine's FOURTH question ("what does the editor paint here?") — `document_colours(uri)`, the `textDocument/documentColor` surface. The walk is the per-candidate compile (the SAME entry points the CLI and the macros run, one candidate per call — the program carries no per-candidate attribution, the spans come from the lexer), and each swatch is the FIRST colour declaration in cascade order resolved through the config's own table: the IR's three table-resolvable colour slots, one arm each — the baked literal (`Val::Rgba`), the runtime token (`Val::Tok`, the alpha modifier riding the same premultiplied bake the token path lowers), the T1 pair slot (`Val::ModePair`, the light half). The keyword slots stay unnamed (`transparent`/`currentColor` are CSS keywords, not swatches — the honest boundary), the theme-less config answers empty for every token class (the compile walk's own boundary), and the wire maps the resolved bytes onto the protocol's rgba `Color` channel for channel. The server advertises `colorProvider` and answers the swatch array; the engine carries none of the wire types (the crate's own split).
- Tests `aeolus-lsp/tests/engine.rs`'s swatch battery (5 pins — the token slots resolve opaque light-tier swatches with `p-4` and `bg-transparent` staying quiet, the ranges are the class spans byte-true, one token on two roots is ONE swatch, purity over the document map, the empty answers outside the pipeline) and `tests/server.rs`'s wire battery (4 pins — the `colorProvider` capability, the `ColorInformation` round-trip with the 0..1 channel fractions and the baked alpha, the quiet drop, the malformed-params null). Bench `crates/aeolus-lsp/benches/lsp.rs`'s `document_colours_walk`, example `crates/aeolus-lsp/examples/lsp_colours.rs` (the token/modifier/secondary swatches beside the keyword and theme-less boundaries).

### 🛠 Fixed — docs drift (this handoff)
- `README.md`: the pipeline diagram's registry-row count catches up with the decoration landing (210 → 213 rows — the crate guide, the phase table and the docs page already carried 213); the tooling table's LSP row, the crate map, the P5 phase row and the examples index carry the two new answers (completion, colour swatches) and their demos.
- `docs/AEOLUS_V1_ARCHITECTURE.md`: the P1 status row's stale count catches up (210 → 213 rows, the decoration family named); the crate-surface row and the P5 prose row file the two seams' landed notes (the audited sources, the boundaries, the wire mapping).
- The `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` gate (CONTRIBUTING's own local-gate list) regains green: three pre-existing broken intra-doc links — `crates/aeolus-compile/src/metamorphic.rs` (the report field's `[InterpError]` link never resolved: the name is the crate root's re-export, not the module's scope — now the qualified path), `crates/aeolus-gpui/src/shadow.rs` (`parse_shadow_bands`' docs linked the module-private `parse_bands` — now prose) and `crates/aeolus-core/src/color.rs` (`gamut_map`'s three bare `to_srgba` links — now the qualified `Color::to_srgba` sibling-method form).

### 🧹 Micro-debt (this handoff)
- `crates/aeolus-lsp/src/engine.rs`: the dead public accessor `AeolusEngine::config()` retires — zero callers in the workspace (the server renders its capabilities from the static surface, a host reads `EngineConfig` where it constructs it), and a public API nobody calls is scaffolding the editor half's docs kept alive for nothing. The config remains the crate's real surface (`EngineConfig::new`/`Default`, the `tokens`/`compile` knobs the docs point at).
- `crates/aeolus-data/src/tables.rs`: the audited `OPACITY_STEPS` table's consumer lands — the completion's default-step source reads it (the M1 wire-up; the table itself is untouched, the dead-data label retires).
### 🚀 Features — the P0 performance-contract harness (M2)
- `aeolus-gpui` (+ the docs): the §4 performance contract's STABLE measurement half lands — `benches/perf_contract.rs`, the P0 phase's exit criteria (the README row's own "replace §4 targets"), on the workspace bench idiom (divan, stable; the `iai-callgrind` instruction-count golden stays the NIGHTLY-ONLY half — flagged, not run here). The harness measures five representative strings (the button core, a layout stack, a typography stack riding the decoration family, a grid, a stateful surface) across the lanes the §4 table names: the hand-written builder chains (the §4 row's own "hand-written" half, rebuilt per iteration), the `tw!` static-program walk under the default `Env`, the freeze-once-clone-per-element host pattern (`clone()` of the walked refinement), the v0.x lane's own static/dynamic paths (`PlanCache::resolve_static` — the ptr-keyed path, the L1 hit, the fresh-L2 shape — the lane's thread-local L1 persists the fresh cache by design, documented in the bench), and the V1 dynamic lane's warm/cold shapes (`ClassCell`, the UI-thread shape under a thread-local). The FAIRNESS contract is the design's centre: the harness's setup PINS the parity before anything is measured — walk == hand-written, field by field, per string (the same gpui calls, the frozen palette's own baked constant extracted once so both chains carry one identical colour) — a diverging chain fails the harness at startup, never an unfair comparison. The measured medians (divan, release, 2026-11): the button string's walk **1.07 µs** against the hand chain's **84 ns** (layout 830/69 ns, typography 319/58 ns, grid 327/45 ns), the clone **70 ns**, the ring polyfill's walk 217 ns, the V1 cell warm hit **908 ns** (hit + walk), the cold compile **9.4 µs**, the v0.x lane's static path 45 ns and warm hit 51 ns — the walk's delta is the data-driven form's own bounded dispatch (per-decl match arms beside the cond checks and the polyfill states' compose), sub-microsecond and constant per string, the numbers the §4 table's unmeasured cells now carry (the walk/hand delta is the honest P0 finding: the ±2 % instruction-count gate stays the nightly iai's contract, the stable half's wall-clock numbers stand beside it in the doc).

### 🛠 Fixed — the walk's row-text slot resolution (the harness's own finding)
- `crates/aeolus-gpui/src/lib.rs`: the parity pins caught a REAL walk gap — the frozen program's length slots that ride `Val::Kw`/`Val::Num` silently no-oped: `border-2` (the integer px scale), the bare `border` (the row's 1px default), `rounded-md` (the radius scale's `0.375rem`), `text-sm` (the composite's `0.875rem` primary) and `w-full` (the sized keyword's `100%`) never reached the walk's absolute/definite/length readers, which matched `Val::Rem`/`Val::Px` only. The NEW `row_length` reader resolves the row texts the frozen program carries beside the keyword slots — the plain px/rem/% spellings — into the length forms (the bare integers read as the px scales they are, v4's own reading); the helpers take the keyword (the call sites updated — the drift gate the exhaustive helper signatures carry). The documented limit stays: the CALC texts (the composite line-height pair) are the CSS arm's own output — the walk does not evaluate expressions, the field stays unset, never a guessed value. Tests `tests/row_text_slots.rs` (RED-first — the integer/bare border widths, the radius and text-size row texts, the calc limit, the spacing-scale regression guard).

### 🛠 Fixed — docs drift (this handoff)
- `README.md`: the P0 phase row files the harness's landing (the stable half, the measured numbers, the nightly remainder) — the 🎯 "targets pending measurement" placeholder retires.
- `docs/AEOLUS_V1_ARCHITECTURE.md`: the §4 performance contract files the P0 status (the harness, the parity-pins' fairness contract, the measured medians, the nightly iai flag) and the table's static-application/cold/warm rows carry the measured cells beside the targets.

### 🚀 Features — the decoration family (M1)
- `aeolus-compile` (+ the workspace's data/IR/oracle surface): the §3.7 typography surface's DECORATION family lands — the `decoration-{style}`/`decoration-{thickness}`/`decoration-{color}` rows lower onto THREE appended `Prop` longhands (143 → 146: `TextDecorationStyle`, `TextDecorationThickness`, `TextDecorationColor`, the append-only tail keeping the stored masks valid) through the same value pass every family walks. The audited v4.1.17 grammar is the contract: the static style utilities (`decoration-solid`…`decoration-wavy`), the static thickness keywords (`decoration-auto`, `decoration-from-font`), and the functional root whose bare value is a NON-NEGATIVE INTEGER thickness — the v4 `isPositiveInteger` round-trip rule (integers only, the spelling parsing back to itself: `decoration-2`/`decoration-0`/`decoration-13` lower as pixel `Num`s, `decoration-2.5` and `decoration-02` are not utilities) — else a colour token through the tiered table, the `/opacity` modifier riding. The integer grammar is its own shape: the NEW `ValueKind::PositiveInt` (the `Number` sibling's stricter twin — `u64` parse plus the round-trip check, so `+2`/`02`/`2.5` refuse at the read side), and the arbitrary bare-number path keeps refusing (`decoration-[13]` — the real compiler infers no number type for the form). The arbitrary values split by type the way the real compiler infers them: length/percentage → thickness, colour → colour (the sibling-row gate's own split — `decoration-[3px]` is the thickness row's alone, `decoration-[#0f0f0f]` the colour row's, `COLOUR_GROUPS` growing the audit entry the drift test enforces). The tw_merge oracle's `collision_group` grows the three arms beside the rows (`decoration-style`/`decoration-thickness`/`decoration-color` — three properties that COEXIST, each group self-conflicting, the registry group strings oracle-checked like every other family's). The plan bridge reports the three longhands through the typed `SlotUnsupported` notes (the v0.x text surface carries the LINE slot only — the underline-offset precedent), the floem walk reports them at the pinned rev, and the metamorphic triangle pins compile == CSS == interp over the family (the CSS arm rendering `text-decoration-style/thickness/color` verbatim).
- `aeolus-gpui` (+ `aeolus-codegen-gpui`'s capability surface): the family rides the pinned `Styled` surface's OWN methods — `decoration-{color}` through `text_decoration_color` (the underline leg's get-or-insert, the pinned method's own semantics) plus the strikethrough leg's only-if-present patch (no pinned method; v4's own `text-decoration-color` reading — the colour alone never creates a line-through), `decoration-{thickness}` generalized past the five named `text_decoration_0..8` methods to the whole integer scale (the same get-or-insert underline leg; the strikethrough leg patched when the line-through composes), and `solid`/`wavy` through `text_decoration_solid`/`text_decoration_wavy`. The capability tier moves the three Refused → Native with the per-decl verdicts staying typed: the `double`/`dotted`/`dashed` style keywords refuse (GPUI renders a single solid line — the row-text verdict, the shadow preset gate's pattern), the `auto`/`from-font` keywords and the rem/percent thickness forms refuse on the absolute-px slot (the val-dependent gate beside the per-prop tiers). The published score rises 58.04% → 58.90% (50.00% native + 8.90% polyfilled over 146 longhands), the conformance page regenerating through the drift gate.
- Tests `aeolus-compile/tests/decoration.rs` (22 pins, RED-first — the style/thickness/colour lowers with IR + CSS + theme tokens + interp + `cn()` merge + sort + metamorphic + static/rust-tokens freezes, the plan bridge's `SlotUnsupported` notes, the typed boundary set (non-integer/zero-padded thicknesses, negation, unknown style names, non-tokens, the bare-number arbitrary), the sibling rows' arbitrary split, the modifier bake), `aeolus-data/tests/registry.rs`'s group samples grown beside the oracle arms (the six decoration classes oracle-checked), `aeolus-codegen-gpui/tests/gate.rs`'s decoration battery (the expressible surface audits clean; the unrenderable style/thickness forms refuse at the gate), and `aeolus-gpui/tests/decoration.rs` (7 walk pins — the wavy/solid forms, the integer thickness on both legs, the colour on both legs, the colour-alone never creating a line-through, the full family through the macro). Bench `crates/aeolus-compile/benches/decoration.rs` + `crates/aeolus-gpui/benches/decoration.rs` (the sibling-row compile beside the Button floor, the four-property CSS emit, the cached warm path, the four-longhand walk beside the line-only floor, the integer-thickness two-leg patch), example `crates/aeolus-gpui/examples/decoration.rs` (the full family, the two-leg reading, the bare-integer scale).

### 🛠 Fixed — docs drift (this handoff)
- `README.md`: the registry-row and longhand counts catch up with the decoration landing (210 rows / 143 props → 213 rows / 146 props across the utility-semantics table, the phase table and the crate guide); the open-boundaries list files the family's remaining typed refusals (the single-line style keywords, the keyword/percentage thickness forms); the examples index carries the decoration demo.
- `docs/AEOLUS_V1_ARCHITECTURE.md`: the P1 status line files the landed counts (210 rows / 143 longhands → 213 rows / 146 longhands, the decoration family named) and the §3.7 matrix's typography row files the decoration half's landed note (the grammar, the oracle arms, the walk's per-leg semantics, the boundaries, the score).
- `docs/REGISTRY.md`/`docs/CONFORMANCE.md`: the generated pages regenerate through the drift gate over the decoration rows and the 146-longhand surface (`cargo xtask gen-tables`).

### 🧹 Micro-debt (this handoff)
- `crates/aeolus-core/src/resolve.rs`: the v0.x oracle's `decoration` refusal carried the generic catch-all text — the family keeps its own arm now, the message naming the lane that works (`decoration-{color,thickness,style}` lower through the V1 pipeline; the v0.x plan shape carries the line slot only — the plan bridge reports the rest), the honest-failure convention the blur/divide arms established.

### 🚀 Features — the wider v4 hint table (M1)
- `aeolus-compile` (+ the workspace's data/IR surface): the value pass's WIDER V4 HINT TABLE lands — the README open-boundary's own line. The eight hints the landed unit grammar named as refusals lower onto the rows whose slots carry the shape: `angle:` parses the `deg` spelling onto the degree rows (`rotate`/`skew-x`/`skew-y`, the `deg_row` oracle the free grammar already consulted), `time:` parses `ms`/`s` onto the millisecond rows (`duration`/`delay`, the `s` bakes ×1000 — the slots ARE milliseconds), `position:` lowers the decoded text onto the object-position row, `family-name:`/`generic-name:` onto the font-family row (the weight sibling shares the root and skips silently — the family is not a weight), `absolute-size:`/`relative-size:` onto the font-size row's own fan, and `string:` onto the content root — lowering onto the NEW `Prop::Content` longhand (143 longhands, the mask budget's two-word form unchanged): the CSS `content` property the `content-["…"]` spelling writes (v4's own reading; the root's keyword half stays `align-content`, the tw_merge oracle's group name, via the `targets_of` override the font-size fan established). The free grammar rides the same shapes where the row's own form carries them: the `ms`/`s` time units lower on the time rows (the `deg` precedent — the motion battery's old `duration-[2s]` refusal retires for the landed semantics) and the QUOTED-STRING form lowers on the content root (the branch leads the composite walk, so an underscore inside a string is the string's own space, never a track list). The eligibility tables are the design's centre (the `COLOUR_GROUPS` audit pattern): `POSITION_GROUPS`/`FAMILY_GROUPS`/`SIZE_KEYWORD_GROUPS`/`STRING_ROOTS` gate each hint, a miss is the silent sibling-row skip, and the run's catch-all renders one boundary when no row of the root takes the class. The CSS arm's arbitrary-Num path reconstructs the unit suffix from the row's OWN number form (`rotate-[45deg]` renders `45deg`, `duration-[time:2s]` renders `2000ms` — the keyword path's oracle, the two CSS channels agreeing; the pre-existing bare-number drift the unit rows carried retires). The verbatim hints' text rides the composite slot's own contract (underscores are spaces, the author owns the CSS validity); a non-`deg` angle spelling and a unitless time are the typed hint mismatches. The registry rows grew the arbitrary arms the hints need (the `font` family, `duration`, `delay` rows — the docs page regenerating through the drift gate), the plan bridge reports the content longhand through its verbatim short-circuit (the `CompositeValue` note naming `Prop::Content`), and the GPUI emit gate refuses the new longhand per prop (the default tier — the capability tables and the conformance page regenerating, `48.95% + 9.09%` over 143 longhands — the published score reading 58.04% over the grown surface).
- Tests `aeolus-compile/tests/hints.rs` (29 pins, RED-first — the angle/time/position/family/generic/size/string lowers with IR + CSS + interp + merge + sort + metamorphic + static/rust-tokens freezes, the free `2s`/quoted-string forms, the content longhand + plan note + keyword-half pin, the typed mismatch set (non-deg, unitless time, non-degree rows, non-family rows), the keyword annotation seam over the grown font row), `aeolus-compile/tests/motion.rs`'s boundary battery rewritten for the landed semantics (the time lowers, the ease curve stays the shape rejection), `aeolus-compile/tests/arbitrary.rs`'s hint-refusal pin redrawn (the catch-all unit boundary when no row carries the hint, the hint mismatch stays typed), and `aeolus-codegen-gpui/tests/emit.rs`'s content pin (the per-prop refusal, the tiers' partition). Bench `crates/aeolus-compile/benches/hints.rs` (the six-family hinted battery beside the Button floor, the unit-form CSS emit, the hint-mixed interp), example `crates/aeolus-compile/examples/hints.rs` (every landed hint, the typed boundaries, the merge).

### 🛠 Fixed — docs drift (this handoff)
- `README.md`: the IR's longhand count catches up with the content longhand (142 → 143 props across the utility-semantics table and the phase table); the open-boundaries list re-draws the arbitrary line (the hint table landed — the arbitrary-property form stays the typed refusal); the examples index carries the hints demo.
- `docs/AEOLUS_V1_ARCHITECTURE.md`: the P1 status line files the landed count (142 → 143 longhands) and the §3.1 value-pass note files the hint table's landed note (the eight hints, the content longhand, the boundaries that stay).
- `docs/REGISTRY.md`/`docs/CONFORMANCE.md`: the generated pages regenerate through the drift gate over the grown arbitrary arms and the 143-longhand surface (`cargo xtask gen-tables`).
- `crates/aeolus-ir/src/prop.rs`: the mask budget doc's stale 141-longhand figure retires for the real surface.

### 🧹 Micro-debt (this handoff)
- `crates/aeolus-macro`: the declared-but-dead `verbose-diagnostics` feature lands its named behaviour — the macro's compile-error rendering gains the two-mode `render_diagnostics` seam (the DEFAULT one-line form the rustc window shows inline; the FEATURE renders each typed refusal multi-line with the stable machine code and help text, the `DiagnosticKind::code`/`help` seam's editor parity inside the compile error), the feature table carries its rationale doc, and the plain-fn unit battery pins both renderers (the same refusal set, the code/help lines under the feature) — the one workspace feature flag that no `cfg` referenced now works in both modes.

### 🚀 Features — the filter tier's GPUI polyfill half (M1)
- `aeolus-gpui` (+ the capability surface): the §3.7 filter row's REMAINING half lands — the polyfill wrappers the pipeline half's landing left as the row's backlog. GPUI's style surface carries no `filter` property, but the element's own colour slots ARE the paint a CSS filter recolours, so the NEW `aeolus_gpui::FilterState` accumulator (the `RingState`/`ShadowState`/`GradState` precedent, `Copy` and last-write-wins like every other state) composes the nine polyfillable filter longhands the registry lowers into: the seven colour-matrix functions (`brightness-*`/`contrast-*`/`grayscale-*`/`hue-rotate-*`/`invert-*`/`saturate-*`/`sepia-*`) apply the CSS Filter Effects Level 1 shorthand's fixed sRGB matrices to the element's own declared colours — background, text, border (recorded beside the walk's inline application, re-applied filtered after it, the cascade's own last-write order), the composed ring/shadow band colours and the gradient stops (`GradState::background_with`, the composition mapping the stops at paint time) — alpha riding untouched, the matrices applied in v4's var-canonical order (brightness → contrast → grayscale → hue-rotate → invert → saturate → sepia), and the drop-shadow presets (`drop-shadow-*` + the bare root's two-band `--drop-shadow` token) map onto GPUI's own `Vec<BoxShadow>` bands (offset, blur radius, colour, no spread, not inset — the pinned `BoxShadow` carries the blur radius the function's third token names), appended AFTER the shadow tier's bands in v4's canonical order (the drop-shadow var last, so the matrices never recolour its own colour). The bare `filter` root is v4's chain rule re-declared (the primitives' vars persist beside it) and `filter-none` is the reset the cascade reads — the pipeline's registry order freezes the composed root AFTER the primitives, so the reset wins regardless of source order, the same last-wins read the interp's cascade and the CSS rule order run (one semantics, three walks). The amounts ride the registry's own forms: the `FnPct` bare-integer percent rows through the same pragmatic read the opacity mapping runs, the `FnDeg` degrees raw, the bare 100% defaults parsed from their composed texts. The colour math is exact on `Rgba`'s f32 components with the lattice rounding at the byte boundary (gpui's own `u32::from(Rgba)` truncates — the CSS byte the spec math rounds to), and the empty chain is a byte-identical identity (the empty-registration discipline every seam pins). The SPATIAL half stays the reported boundary: `blur-*` blurs the element's rendered output and the `backdrop-*` mirrors paint BEHIND it — no GPUI slot, the walk skips them (the documented no-op arms), the static gate refuses them typed, the CSS/interp arms carry the real output. The capability surface moves the nine longhands Refused → Polyfill (`aeolus-codegen-gpui::polyfilled`, the per-decl gate passing the closed-world forms — the registry's own `DROP_SHADOW_PRESETS`/`DROP_SHADOW_DEFAULT`/`FILTER_CHAIN` tables the one source of truth — and refusing the annotation-less band gate rather than guessing): the published score rises 52.11% → 58.45% (49.30% native + 9.15% polyfilled over 142 longhands), the conformance page regenerating through the drift gate.
- Tests `aeolus-gpui/tests/filters.rs` (14 pins, RED-first — the spec matrices' own bytes on the registry palette for each of the seven functions, the halfway interpolation, the canonical-order composition (source-order-independent), the text/border slots riding beside the background, the drop-shadow preset band + the bare root's two bands, the bands beside the shadow tier's presets, the matrix recolour over the composed shadow/ring bands and the gradient stops, the reset arm and the registry-order freeze, the spatial/backdrop skip, the unit surface, the empty-chain identity) and `aeolus-gpui/tests/filters_static.rs` (2 pins — the macro gate passes the matrix chain + the reset arm and the drop-shadow presets, the frozen programs carrying the longhands), the emit battery's filter boundary test re-drawn as the tier split (`the_filter_tier_splits_the_polyfill_half_from_the_spatial_refusal`: nine polyfill, blur + the ten backdrop refusals, the keyword-less band gate refusing), and the conformance pins over the 13-slot polyfill tier. Bench `crates/aeolus-gpui/benches/filters.rs` (the seven-matrix chain walk, the drop-shadow band walk beside the shadow tier's, the empty-chain identity floor, the accumulator's own shade), example `crates/aeolus-gpui/examples/filters.rs` (the full chain over a coloured surface, the band list, the reset arm, the reported boundary).

### 🛠 Fixed — docs drift (this handoff)
- `README.md`: the IR's longhand count catches up with the filter/backdrop layers' own landing (the utility-semantics table's **121 props** figure — line 38 — retires for the landed **142 props**, the count the crate guide and the conformance page already carried); the GPUI capability row and the examples index carry the filter polyfill half (the three pipeline-side filter/backdrop/grid-flow demos the index had never listed beside them) and the macro surface's own demo (`frozen_program`, the example the crate had none of).
- `docs/CONFORMANCE.md`: the generated page regenerates through the drift gate over the moved tier (`cargo xtask gen-tables`).
- `crates/aeolus-codegen-gpui/tests/emit.rs`: the tier-split rewrite retires the duplicated doc-comment lines the old filter test carried (the same sentence twice — the rewrite leaves one).

### 🧹 Micro-debt (this handoff)
- `crates/aeolus-floem/src/program.rs`: the colour-resolve helper's refusal channel carried a bare `Result<[u8; 4], String>` — the workspace's typed-error convention reaches it as the new `ColourRefusal` enum (the value form, the source's answer, the interp's own typed `InterpError` carried instead of stringified, the two no-source lane texts), the report text rendering at the drop boundary only.
- `crates/aeolus-macro`: the one workspace crate without a consumer example grows `examples/frozen_program.rs` (the macro surface is the last "missing consumer example" the handoff audit named) — the frozen program a `tw!` expansion evaluates to, printed layer by layer beside the row texts, the `cn!` compile-time merge's winners, and the capability gate's refusal story; the `aeolus-rt` dev-dependency resolves the expansion's `::aeolus_rt::…` absolute paths.

### 🚀 Features — the composite arbitrary-value grammar (M1)
- `aeolus-compile` (+ the workspace's data/IR surface): the §3.1 value pass's composite boundary lands as a first-class, kind-gated grammar — the sized functions (`calc()`/`min()`/`max()`/`clamp()`) lower on the single-target scalar rows, the underscore-joined track lists and the track functions (`repeat()`/`minmax()`/`fit-content()`) on the grid-template/auto rows, the image functions (`url()`/`image()`/`image-set()`/the gradients) on the bg-image row and the bare `n/d` ratios on the aspect row, through the IR's NEW `Val::Verbatim(&'static str)` slot — the decoded text (the underscores ARE spaces, `\_` a literal underscore, the var branch leading so a custom-property name's underscore stays the name's own byte) interning into the leak pool the conds already ride, the static form staying a `static` item end to end. The kind gate is the design's centre: a composite refuses TYPED on every row whose slots cannot carry its shape — the shorthand fans (one value per longhand), the filter/backdrop handles (one value per handle, the §3.7 row's own boundary kept), the shadow lists — plus the malformed functions (unbalanced parens, the new `REASON_ARBITRARY_MALFORMED` boundary), the negative flag (no composite form; the CSS walk refuses identically so the two channels' diagnostics stay equal) and the opacity modifier (the colour-form post-check, now pinned over the verbatim slot). The CSS arm renders the text as typed (v4's own arbitrary contract: the author owns the CSS validity), the interp answers under any env, the IR-level `cn()` merge folds the slot per longhand (last wins, the same fold), the metamorphic triangle agrees on the text (both sides render it), the plan bridge reports the value (the NEW `PlanNoteKind::CompositeValue { prop, text }` note, the `SlotUnsupported` precedent — the plan slot left unset, never a silent zero), the annotation alignment answers `""` (the value carries its own text, the numbers' documented convention), the GPUI emit gate refuses per VALUE (the new `decl_unsupported_reason` val-keyed verdict beside the per-prop tiers; the conformance ceiling stays per-longhand, so no page drift), and the backends' unmapped-value reports carry it (the floem walk's report pin lands with the battery). The hinted forms ride the same slot (`length:` takes the sized functions, `url:` wraps the bare path, `image:`/`ratio:` decode), the integer hint parses whole-number syntax, and the wider hint refusals re-word to the covered set.
- Tests `aeolus-compile/tests/composites.rs` (30 pins, RED-first — the sized functions, the track lists + decode, the image row with its silent colour sibling, the bare ratio, the hinted length/url/image forms, the typed boundary set (shorthand fans, filter handles, shadow lists, malformed, negative, modifier), the CSS/interp/`cn()`/metamorphic/static/rust-tokens/sort/alignment surfaces, the plan note), bench `benches/composites.rs` (the five-shape compile against the Button floor, the verbatim CSS emit, the composite-mixed interp), example `examples/composites.rs` (the decode, the boundaries, the merge, the frozen text). The unit battery's own composite strings walk the landed semantics (the `multi_token_composites_lower_through_the_verbatim_slot` pin replaces the blanket refusal), the floem static-walk battery pins the honest report (`composite_values_are_reported_never_dropped`), and the GPUI emit battery pins the val-keyed refusal (`the_composite_values_stay_the_reported_gpui_boundary`).

### 🚀 Features — the grid auto-flow family (M2)
- `aeolus-compile` (+ the workspace's data/IR surface): the §3.7 grid row's SECOND boundary half lands — the auto-TRACK lists rode the composite grammar's verbatim slot (`grid-cols-[auto_1fr_auto]`, `auto-cols-[minmax(0,1fr)_auto]`), and the auto-FLOW keyword family joins the registry: `grid-flow-row`/`grid-flow-col`/`grid-flow-dense`/`grid-flow-row-dense`/`grid-flow-col-dense` lower onto the NEW `Prop::GridAutoFlow` longhand (142 longhands, the mask budget's two-word form unchanged) through the SAME keyword walk every family takes — the row's own table carrying the v4.1 values verbatim (`col` renders `column`, the dense pairs the two-value form), the CSS arm rendering the value form, the interp answering, the static/rust-tokens arms freezing the slots, and the plan bridge reporting the longhand honestly (the v0.x plan carries grid templates/starts but no auto-flow slot — the `SlotUnsupported` note, the grid-auto-rows precedent; the floem walk reports the same boundary). The GPUI emit gate refuses per longhand by the default tier (no slot at the pinned surface), the conformance page regenerating through the drift gate.
- Tests `aeolus-compile/tests/grid_flow.rs` (9 pins, RED-first — the keyword scale, the annotation texts, the CSS value form, the variant conds, the interp/static/rust-tokens arms, the sort order, the plan note), bench `benches/grid_flow.rs` (the five-slot compile against the Button floor, the value-form CSS emit, the flow-mixed interp), example `examples/grid_flow.rs` (the scale, the texts, the plan notes beside the composite note).

### 🛠 Fixed — docs drift (this handoff)
- `README.md`: the registry-row and longhand counts catch up with the two lands (209 rows / 141 longhands → 210 rows / 142 longhands across the utility-semantics row, the crate diagram, the phase table and the crate guide); the open-boundaries list re-draws the composite line (the grammar landed — the arbitrary-property form and the angle/time/position/family-name hints stay the typed refusals); the examples index carries the two new demos.
- `docs/AEOLUS_V1_ARCHITECTURE.md`: the P1 status line and the §8 P1 row file their landed counts (189 rows / 121 longhands → 210 rows / 142 longhands, the motion/filter/grid-flow lineage named); the §3.1 value-pass note files the composite grammar's landed note (the verbatim slot, the kind gate, the boundaries that stay).
- `docs/CONFORMANCE.md`/`docs/REGISTRY.md`: the generated pages regenerate through the drift gate over the new longhand and row (`cargo xtask gen-tables`).

### 🧹 Micro-debt (this handoff)
- `crates/aeolus-ir/src/lib.rs`: the IR's size-contract test retires the plan sketch's 12-byte `Val` figure for the landed shape (the composite slot's `&'static str` payload is the conds' own interned-slices discipline — `Copy`, no owned memory, the static form still a `static` item; the pin now walks the real 24/32 bytes with the rationale in place, so a future owned variant fails `assert_copy` before it fails the size pin).

### 🚀 Features — the CSS filter layer (M1)
- `aeolus-compile` (+ the workspace's data/IR surface): the §3.7 backlog row's pipeline half lands — the `blur-*`/`brightness-*`/`contrast-*`/`grayscale-*`/`hue-rotate-*`/`invert-*`/`saturate-*`/`sepia-*`/`drop-shadow-*`/`filter` families lower through the SAME typed pipeline as every other registry family. The IR carries ten appended longhands (`FilterBlur`…`FilterDropShadow` + the composed `Filter` slot — the `--tw-blur`-style var-reference names, the `ShadowColor` precedent); `aeolus-data` carries the ten rows with the v4.1.17 default theme's own texts quoted verbatim from the pinned upstream (`--blur-*`/`--drop-shadow-*` tokens, the bare-integer percent forms, the `drop-shadow` two-layer default) and the composed `cssFilterValue` chain as the shared `FILTER_CHAIN` const. The value pass renders the family through the per-root form table (the `NumForm::FnPct`/`FnDeg` fn-wrapped forms — `brightness-110` → `brightness(110%)`), the CSS arm renders the v4 var-chain emission form (each primitive's var line beside the composed chain — the identical text in every rule, so the cascade never loses a component), the IR-level `cn()` merge folds the primitives per longhand, the interp answers them, the sort seam orders them by the rows' own registry order, the static/rust-tokens arms freeze them, and the plan bridge reports the family honestly (the v0.x oracle refuses filters outright — the `SlotUnsupported` note is the differential contract, the grid-auto-rows precedent). The metamorphic triangle pins compile == CSS == interp with the chain lines riding the NEW typed `MetamorphicNote::FilterCompose` note, and the fn-wrapped numeric comparison (`brightness(110%)` vs the interp's bare `110`) joins `value_agrees`. The honest boundaries stay typed: bare `brightness`/`contrast`/`saturate`/`hue-rotate` (no v4 default — the theme key's absence is a shape refusal), the `grayscale`/`invert`/`sepia` bare roots carrying the v4 100% default, `hue-rotate`'s negative flag (`-hue-rotate-30` → `hue-rotate(-30deg)`), the single-value arbitrary forms (`blur-[2px]` → `blur(2px)`, the `EXTRA_SCALAR_GROUPS` audit surface growing), and the composite arbitrary values refusing (the codegen host's surface). The `drop-shadow-{color}` recolour arm is the documented boundary (the shadow tier's colour machinery reaches it; the size/color two-var interplay stays out of the registry). The GPUI emit gate refuses the family per longhand with the reason pointing at the CSS arm (no filter slot at the pinned surface), the conformance page regenerating through the drift gate.
- `aeolus-ir`: the mask grows past one word — `PropMask` is the plan §3.3 sketch's own `u128×k` form at k=2 (the low word holds discriminants 0..128, the high word the rest; the algebra is word-blind, `for_each` walks low-word-first, the budget test retires `prop_mask_fits_u128` for `prop_mask_fits_two_words`), and the new `tests/mask_two_word.rs` pins a one-word implementation cannot pass (the boundary round-trip, the word-blind union/intersect/difference/covers, the cascade-order walk, the row-target slice build).
- Tests `aeolus-compile/tests/filters.rs` (21 pins, RED-first — the scale slots, the bare defaults, the percent/degree forms, the negative flag, the drop-shadow presets + the two-layer bare default, the composed reset, the arbitrary singles, the composite refusal, the annotation texts, the var-chain CSS form, the `cn()` fold, the interp, the plan notes, the metamorphic agreement, the static freeze, the rust-tokens text, the sort order), bench `benches/filters.rs` (the family compile against the Button floor, the var-chain CSS emit, the interp), example `examples/filters.rs`.

### 🚀 Features — the backdrop-filter layer (M2)
- `aeolus-compile` (+ the workspace's data/IR surface): the §3.7 backlog row's second half — the `backdrop-blur-*`/`backdrop-brightness-*`/`backdrop-contrast-*`/`backdrop-grayscale-*`/`backdrop-hue-rotate-*`/`backdrop-invert-*`/`backdrop-opacity-*`/`backdrop-saturate-*`/`backdrop-sepia-*`/`backdrop-filter` families ride the SAME seam the filter layer walked. The IR carries ten more appended longhands (`BackdropBlur`…`BackdropSepia` + the composed `BackdropFilter` slot; 141 longhands total); `aeolus-data` carries the ten rows — the SAME value grammar the filter half takes (v4's own theme fallback chains: `backdrop-blur` reads `--backdrop-blur` then `--blur`, so the scale table is shared), the backdrop-only ninth primitive (`backdrop-opacity-*`, the chain's own var with no filter-half counterpart), and the composed `cssBackdropFilterValue` chain as the shared `BACKDROP_FILTER_CHAIN` const. The CSS arm renders v4's own emission order — each primitive's var line beside BOTH composed properties (`-webkit-backdrop-filter` beside `backdrop-filter`), the composed backdrop rows expanding into the prefixed pair. The boundaries stay typed: bare `backdrop-brightness`/`backdrop-contrast`/`backdrop-saturate`/`backdrop-hue-rotate`/`backdrop-opacity` (no v4 default), the 100% bare defaults (`backdrop-grayscale`/`backdrop-invert`/`backdrop-sepia`), `backdrop-hue-rotate`'s negative flag, the single-value arbitrary forms, and the GPUI lane refusing per longhand (the conformance page regenerating through the drift gate).
- Tests `aeolus-compile/tests/backdrop_filters.rs` (12 pins, RED-first — the shared scale slots, the percent forms, the 100% defaults, the ninth primitive, the negative flag, the composed reset + bare chain, the arbitrary singles, the prefixed var-chain CSS form, the reset/off texts, the `cn()` fold, the interp), bench `benches/backdrop_filters.rs` (the family compile against the Button floor, the prefixed var-chain CSS emit, the interp), example `examples/backdrop_filters.rs`.

### 🛠 Fixed — docs drift (this handoff)
- `README.md`: the registry-row and longhand counts catch up with the two lands (189 rows / 121 longhands → 209 rows / 141 longhands across the utility-semantics row, the crate diagram, the phase table and the crate guide); the workspace battery count re-measured (1279 → 1348 tests, 1289 → 1358 with all features enabled).
- `docs/AEOLUS_V1_ARCHITECTURE.md`: the §3.7 backlog row files its landed note (the filter + backdrop-filter pipeline halves, the twenty longhands, the two-word mask, the var-chain CSS forms, the typed boundaries, the GPUI refusal with the polyfill wrappers remaining the row's backlog); the §8 P6 row files the same landed note beside the motion layer's.
- `.github/workflows/ci.yml`: the header's workspace-member count caught up with the P5 LSP surface (19 → 20 members — the lsp crate landed after the comment was written).

### 🧹 Micro-debt (this handoff)
- `crates/aeolus-floem/src/program.rs`: the motion arm's drop reason carried a stray run of whitespace mid-sentence (`ride                  the CSS emit arm`) — the string reads as one line again (cosmetic; the walk's report text only).
- `crates/aeolus-ir/tests/ir_laws.rs`: the mask budget test retires the one-word name (`prop_mask_fits_u128` → `prop_mask_fits_two_words`) — the assertion the test makes grew with the seam (256-bit budget over the 141-longhand surface), not just its name.

### 🚀 Features — the custom-authoring CSS-file form (M1)
- `aeolus-compile`: the §5.4 "same file as web" line's file form lands as a first-class, public, test-pinned seam — `customs_from_css(css, &Config) -> Result<Customs, CustomCssError>` walks the `@custom-variant name (selector);` statements and the `@utility name { @apply …; }` composite bodies a project's theme CSS carries onto the SAME typed `Customs` surface the API form registers (no parallel semantics, no legacy read; the statement walk mirrors the `@source` seam's shape — comments stripped first, every malformed form a typed refusal naming the statement). The composite body compiles AT REGISTRATION: the apply classes run through the same pipeline (the registration-order customs resolving, a later `@utility` may apply an earlier custom root), a variant-carrying body refuses (the flat `Cond` model composes the CANDIDATE's own condition, never the body's), a body with any compile diagnostic refuses (the honest-failure contract reaches registration), and the compiled declarations + their parallel keyword texts intern — the NEW `CustomValue::Applied(AppliedBody)` form, the multi-decl shape (four radii + two paddings, different `Val`s per longhand) the single-`Val` fixed/functional forms cannot hold. The audited selector table folds the `.dark` family (the legacy reader's own containment rule — the differential source) and the built-in state spellings onto the typed `Cond` surface; every other selector refuses. The variant aliases ride the annotation seam too: `keyword_texts_with(input, customs, config)` folds the aliases' candidate conds and hands out the applied bodies' registered row texts, `ClassCell::get_with`/`get_theme_with` align through the SAME registrations they compile (empty bag byte-identical to the theme-less call), and the CSS arm renders the applied body per stored declaration (the row-relative `Val::Kw` slots rendering their registration-time text — `@apply hidden` renders `display: none`), the plan bridge and the rust-tokens arm carrying the same texts.
- Tests `aeolus-compile/tests/custom_css.rs` (18 pins, RED-first — the applied-vs-body differential parity, the cond composition, the collision rule, the alignment with the registered texts, the empty-customs byte-identity, the CSS render, the modifier/negation refusals, the alias folds, the selector/block/bare/raw/variant-carrying/unknown-class/empty-body typed refusal set, the additive contract, the registration-order resolve, the spacing-knob ride, the audit surface), `aeolus-rt/tests/class_cell.rs` gains the applied-freeze pin (the registered `none` lands on the frozen keyword slice), bench `benches/custom_css.rs` (the registration walk, the applied hit beside the expanded built-ins — the splice is ~2× cheaper than re-lowering — and the empty-bag annotation floor), example `examples/custom_css.rs` (the sheet registers, the dark-mode splice, the aligned texts, the CSS render).

### 🚀 Features — the functional star form (M2)
- `aeolus-compile`: the same §5.4 row's functional half — the `@utility brand-pad-* { padding: --spacing(n); }` star bodies register through the SAME `CustomForm` grammar the typed API takes (one suffix fills ONE form; mixed forms, literal CSS values, unknown property names, unknown value forms and the empty value refuse typed with the reason named), the CSS property names map onto the IR longhand targets (the exact longhand names 1:1, the audited `padding`/`margin`/`border-radius`/`inset` shorthands fanning in the registry rows' own order), and the differential pins the file form compiling EXACTLY as the equivalent `CustomUtility::functional` registration — the spacing config's px bake, the integer/number shapes, the token form's tiered resolve (the `/opacity` modifier riding the theme pass's premultiplied identity) and the applied/functional forms coexisting in one sheet.
- Tests `aeolus-compile/tests/custom_css_functional.rs` (11 pins, RED-first — the typed registration, the API-form differential, the integer/number shapes and the shape rejection, the shorthand fans, the px bake, the token tier with its modifier ride, the mixed-form/literal/unknown-property/unknown-form/empty-value/star-less refusals, the coexistence), bench `benches/custom_css.rs` gains the functional star compile, example `examples/custom_css.rs` demonstrates the star body beside the token path.

### 🛠 Fixed — docs drift (this handoff)
- `docs/AEOLUS_V1_ARCHITECTURE.md`: the §5.4 CSS-first line files its landed note (the file form's both halves, the applied shape, the refusal set).
- `README.md`: the custom-authoring row and the examples index catch up with the two lands.

### 🧹 Micro-debt (this handoff)
- `crates/aeolus-compile/src/metamorphic.rs`: the rustc 1.100 clippy sweep's silent-drop cleanups — the three translated `MetamorphicNote` pushes lost their `;`, the `value_agrees` numeric probe kept a single-pattern `match` instead of the `if let` it means, and `keyword_map` passed the Copy `Config` by reference (all style, zero behaviour; the gate is clean under `-D warnings` again).
- `crates/aeolus-compile/tests/metamorphic.rs`: the stale `Env` import and the two `assert_eq!(…, true)` bool-assert forms (`assert!` is the established idiom the workspace's own convention names).

### 🚀 Features — the metamorphic seam (M1)
- `aeolus-compile`: the §5.3 differential line's named target lands as a first-class, public, test-pinned seam — `metamorphic_check`/`metamorphic_check_with`/`metamorphic_check_theme`/`metamorphic_check_theme_with` walk the THREE pipeline answers over one class string (the compile, the CSS emit, the interp) and report, per probe situation, whether the interp's effective declarations equal the CSS rules the env fires: cascade last-wins on BOTH sides (the IR's same-condition dedupe against the CSS rules' source order), the diagnostics channels byte-equal (the two walks' shared typed surfaces, plan §5.2's triangle rule), the whole-string sheet the per-candidate parts (the compositional identity), the dark mode walked beside every situation so baked pairs answer both halves. The walk is built FROM the pipeline's own parts — the same lexer walk, the same variant classifier, the interp's own `decide` verdicts, the CSS arm's own `rgba_text` colour form, the annotation seam's row texts — so the checker cannot drift from the walks it checks. Every documented divergence is a typed note (`MetamorphicNote`): `StackedVariantFold` (the CSS renders the stack conjunctively, the IR folds to the dominant condition — the checker walks the env every conjunct holds), `PseudoTarget` (the render-target boundary, both sides), `TokenDynamic` (the CSS keeps the `var()`/`color-mix` reference, the interp answers the probe's bytes — counted, never a text lie), and the sourceless-probe notes translated from the interp's own. The IR's unit-less `Num` slot compares numeric-normalized against the CSS row's unit text (`500` ⟷ `500ms`), the arbitrary literal's verbatim hex passthrough normalizes against the baked `rgb(r g b)` bytes, and the one row whose CSS property name overrides the IR longhand's (`stroke-2` → `stroke-width`) maps back so both sides key identically.
- Tests `aeolus-compile/tests/metamorphic.rs` (15 pins, RED-first — the diagnostics-equality battery over lowered/refused/boundary strings, the base-situation decl-for-decl agreement, the cascade across same-situation candidates, the hover/breakpoint/stacked conjunctive envs, the fold note, the pseudo-target exclusion, the themed pairs in both modes, the runtime tokens as counted notes, the `/50` modifier bake identity, the compositional identity, the report rendering, determinism, the empty-input error, the customs seam), bench `benches/metamorphic.rs` (the themed/themeless Button three-walk beside the single-utility floor), example `examples/metamorphic.rs` (the per-situation agreement report across the three tiers), fuzz target `fuzz/fuzz_targets/compile_interp_emit.rs` (the seventh: panic freedom + diagnostics equality + compositional identity on EVERY input, full agreement for every clean lowering) with the CI sweep updated.

### 🚀 Features — the P5 LSP (M2)
- `aeolus-lsp` (the P5 row's LSP hover/diagnostics, the §5.5 tooling-parity half): the headless ENGINE answers from the same pipeline the CLI runs — the `scan_rust_strings` extraction seam through the V1 compile walk (the themed walk over the built-in shadcn bake by default, a host CSS through the same `ThemeSource` the CLI's `--theme <path>` accepts; the theme-less boundary rides `compile`), per-location typed diagnostics carried into the LSP's UTF-16 line/character model with their stable machine codes and help text (`EngineDiagnostic`), the hover keyed by the candidate under the cursor (the lexer's own candidate spans): a lowered class renders its folded condition and the CSS arm's own declaration texts, a refused class renders the typed refusal (code + help), a cursor outside any candidate string answers None (`EngineHover`). The position conversions (bytes ⟷ UTF-16 pairs) are owned ONCE in the engine — a host never re-derives the column model. The SERVER half is the stdio JSON-RPC state machine (`Session` + `handle_message`, testable message-by-message without a pipe: initialize capabilities, didOpen/didChange/didClose publish diagnostics with the protocol's own clear-on-close contract, hover requests answer from the engine, shutdown/exit close cleanly) and `bin/aeolus-lsp` is the transport's thin shell (one thread per side, no async runtime).
- Tests `crates/aeolus-lsp/tests/engine.rs` (12 pins, RED-first — the lifecycle, determinism, the UTF-16 column shift over a multi-byte identifier, the stable codes, the lowered/refused/None hover triad, the host-CSS token resolution), `tests/server.rs` (9 pins — the initialize capabilities, the didOpen/didChange/didClose publishes, the hover answers, the protocol gate before initialize, the shutdown/exit contract, the unknown-method quiet drop), bench `benches/lsp.rs` (the analyze walk and the hover resolution over a synthetic document), example `examples/lsp_hover.rs` (the engine headless: the diagnostics publish and the three hover shapes).

### 🛠 Fixed — stale pins the shadow tier left behind (this handoff)
- `crates/aeolus-compile/tests/style_walk.rs`: the boundary test still pinned the pre-shadow-slot behavior (`shadow-sm` reported as a boundary) — the preset resolves into the style's OWN shadow slot now; the pin walks the landed semantics (the first band paints, the `shadow-inner` inset placeholder keeps the typed note).
- `crates/aeolus-{iced,slint,xilem}/tests/*`: the three backend `unmapped_longhands_are_reported_never_dropped` rows still asserted the preset family as a DROPPED longhand — the plan bridge resolves it now (egui's row was the only one updated with the tier). The pins walk the landed semantics per backend: the first band paints (iced's `container.shadow`/slint's `drop_shadow`/xilem's painted bands), the extra preset layers ride the `shadow layers 2+` note where the backend paints one band, and `box-shadow` no longer drops. Floem's row keeps its positive pin WITH the accurate comment: floem's two lanes split the tier honestly — the plan path paints the parsed bands (`apply_box_shadows`), the V1 program walk keeps the walk-level boundary (the preset text is a row-relative `Val::Kw`, no shadow-text parser rides the walk) and reports it.

### 🧹 Micro-debt (this handoff)
- `fuzz/fuzz_targets/probe_eval.rs`: the target had not compiled since the `Test::Pseudo` variant joined the probe surface (a non-exhaustive match — the CI fuzz sweep would have failed to build). The arm answers through the target's own contract (every answer flips deterministically — a fourth bit channel), and the target compiles again.
- `.github/workflows/ci.yml`: the fuzz job's target list gains the seventh target (`compile_interp_emit`) and the header count catches up (the comment still said six).

### 🚀 Features — the GPUI shadow tier (M1)
- `aeolus-gpui`: the §3.7 shadow row's GPUI half lands as the NATIVE tier — `aeolus_gpui::shadow` parses the closed-world preset texts the registry's shadow row lowers into (the seven `var(--tw-shadow-{name})` references through the audited band table, the `0 0 #0000` clear, the `inset 0 0 0 0 currentColor` placeholder — the pinned `gpui-pre` `BoxShadow` carries the `inset` flag) into GPUI's own `Vec<BoxShadow>`, the `ShadowState` accumulator (the `RingState` precedent, `Copy`, riding the interaction closures) composes the winner preset's bands beside the ring's in ONE `shadow()` call (v4's own list order — the ring bands first, first-paints-on-top), the `shadow-{color}` recolour applies v4-true (`--tw-shadow-colored`'s replace, the `/opacity` modifier riding the theme bake's colour), the `currentColor` band reads the walk's text colour, and `SHADOW_PRESET_TEXTS` quotes the audited v4 default theme verbatim (a differential pin walks the table against `tailwind-default-theme.css` — the same drift discipline the registry tables run). The dead `apply_ring` folded into the merged compose.
- `aeolus-codegen-gpui`: the capability tables move `Prop::BoxShadow`/`Prop::ShadowColor` NATIVE — **59.50% → 61.16% expressible** (70 native + 4 polyfill over 121 longhands) — the per-decl emit gate reads the registry's OWN `SHADOW_PRESETS` texts (one source of truth; `aeolus-data` rides along dependency-light), `lower_class` accepts the families, and the conformance page regenerates through the drift gate. `tw!("shadow-lg shadow-red-500")` compiles where it used to refuse at expansion.
- Tests `aeolus-gpui/tests/shadow.rs` (11 pins, RED-first — the audited-text differential, the preset scale, the clear, the inset-in-text-colour compose, the cascade, the v4-true recolour with its `/50` bake read from the compile, the empty-recolor contract, the ring+shadow one-list merge, the honest `None`s), `aeolus-codegen-gpui/tests/shadow.rs` (6 pins, RED-first — the native verdicts, the closed-world per-decl gate, the audit-empty verdict, `lower_class`, the frozen emit, the conformance partition), bench `benches/shadow.rs` (the walk, the compose, the parse), example `examples/shadow.rs` (the card: lg + recolour, the hover wholesale replace, the inset placeholder, the clear).

### 🚀 Features — the plan bridge's shadow slot (M2)
- `aeolus-compile`: the §7 migration seam's shadow half — the `Prop::BoxShadow`/`Prop::ShadowColor` decls land in the v0.x plan's OWN shadow slots instead of the typed note: the preset references resolve through the ORACLE'S OWN theme reader (`TailwindTheme::default()`'s audited `--shadow-*` tokens — the same bands the CSS arm renders and the GPUI lane quotes) into `base.shadow` (dominant layer) + `extras.shadow_rest` (the rest), the `shadow-none` clear lands its transparent band, and the `shadow-{color}` recolour patches at `finish` in the ORACLE'S OWN pass order (the preset's wholesale replace beats the recolour wherever the source order put the classes — the oracle's observable semantics, differentially pinned both orders; the recolour without a preset composes nothing, v4's empty-shadow colour). The `ShadowParts` accumulator (the `GradParts` precedent) carries the composed form candidate-to-candidate per bucket; `style_walk`'s program lane rides the same writer (per-layer, the fired layers being independent buckets). The inset placeholder alone keeps the re-scoped `PlanNoteKind::ShadowPreset` note (its `currentColor` has no plan answer — the oracle's own shadow-inner errors). The five plan-path backends style real shadow bands from V1 output (xilem paints `paint_box_shadow`, floem applies `apply_box_shadows` — the painters the v0.x lane already runs).
- Tests `aeolus-compile/tests/shadow.rs` (10 pins, RED-first — the preset scale through the oracle reader, the clear, the cascade, the recolour interactions BOTH orders, the variant buckets, the inset note, the differential battery slot-for-slot against `Resolver` including `hover:shadow-lg`/`dark:shadow-2xl`), the plan-bridge differential battery gains the fifteen shadow rows and the comparator grows the shadow/shadow_rest/inset sections, bench `benches/shadow.rs` (the preset resolution, the recolour compose, the cascade beside the oracle's own resolve), example `examples/shadow_plan.rs` (the slots printed, the differential walk).

### 🛠 Fixed — docs drift (this handoff)
- `README.md`: the P3 roadmap row and the open-boundary list catch up with the shadow tier's both halves; the examples index lists the two new demos (`shadow`, `shadow_plan`); the test battery re-derived.
- `docs/AEOLUS_V1_ARCHITECTURE.md`: the §3.7 shadow row files its landed note (both halves, the pass-order semantics, the score movement).

### 🧹 Micro-debt (this handoff)
- `crates/aeolus-gpui/src/lib.rs`: the dead `apply_ring` application point folded into `apply_polyfills`' merged compose (the shadow tier's one-list merge absorbed it — zero callers left).
- `crates/aeolus-gpui/src/lib.rs`: the `is_ring_prop` doc comment re-scoped for the shadow tier's flip — `ring-inset`'s bare `inset` text stays OUTSIDE the closed-world preset forms, so the emit gate keeps refusing it (the honest-failure contract unchanged where the tier did not reach).

### 🚀 Features — the CSS Color 4 gamut mapping (M1)
- `aeolus-core`: `Color::gamut_map` — the plan §9 colour-gamut row's mitigation ("gamut-map once; document"), the CSS Color 4 §13 algorithm as a public seam: an out-of-gamut colour walks its chroma down its OWN hue ray to the largest in-gamut value (a ~20-step binary search, deterministic, `f32`-exact), hue and lightness hold through the backend bytes, alpha carries, and any parsed value stays finite (the fuzz findings hold on the mapped path too). The identity contract is the load-bearing half: a colour that never left the gamut returns UNCHANGED — bit-identical to the clamp oracle `to_srgba` (which stays untouched: the v0.x differential triangle's ground truth) — so every existing palette pin holds byte for byte.
- `aeolus-theme`: the bake runs the mapping (plan §3.2's aeolus-theme row's own "oklch→sRGB gamut-mapped once" contract, until now a documented limitation): every T0 literal, T1 dark pair and T2 runtime answer converts mix-first-then-map — the CSS reading (`color-mix` runs in Oklab, the display conversion maps). The REAL default palette carries out-of-gamut rows — shadcn's `--destructive: oklch(0.577 0.245 27.325)` overshoots linear-red 1.27 — and its baked bytes now sit ON the gamut boundary hue-true instead of channel-clipped hue-shifted; in-gamut tokens bake byte-identically.
- `aeolus-compile`: the arbitrary-value colour pass runs the same mapping after the alpha mix, so a literal agrees with its token spelling (the multi-token `oklch(…)` spelling stays the documented composite boundary — out-of-gamut reachability lives on the token path; the hex/`rgb()` in-gamut forms keep the audited conversion's exact bytes).
- Tests `aeolus-core/tests/gamut.rs` (9 pins, RED-first — the identity contract over palette and hex spellings, the hue/lightness holds through the backend bytes across the wheel, the in-gamut round-trip, the fuzz extremes, determinism, and the clamp oracle's unchanged contract), `aeolus-theme/tests/gamut.rs` (4 pins — the real palette's out-of-gamut token maps, in-gamut tokens stay clamp-identical, the `/alpha` mix order, a host CSS with an out-of-gamut `--color-brand`), `aeolus-compile/tests/gamut.rs` (3 pins — the token lowering through `compile_theme`, the in-gamut arbitrary identity, the audited alpha bake), bench `aeolus-core/benches/gamut.rs` (clamp beside chroma reduction, in-gamut early-out and out-of-gamut search), example `examples/gamut.rs` (the destructive token: clamp `[231, 0, 11]` hue-shifted beside mapped `[228, 0, 22]` hue-true).

### 🚀 Features — the GPUI grid-placement + flex-basis tier (M2)
- `aeolus-codegen-gpui`: the §3.7 grid row's "native subset; rest diagnosed" finished for every longhand the pinned `gpui-pre` surface carries a slot for — `supports()` takes `FlexBasis`, `GridColumnStart/End`, `GridRowStart/End` NATIVE (the pinned surface carries `flex_basis`, `col_start`/`col_end`, `row_start`/`row_end`), the emit gate and the third-party `lower_class` seam accept their classes, and the conformance page regenerates through the drift gate: **55.37% → 59.50% expressible** (68 native + 4 polyfill over 121 longhands).
- `aeolus-data`: the four placement rows gain the negative flag — v4's `-col-start-2`/`-row-end-1` from-end line forms (the roots index and the registry docs page regenerate through the same gate; the span rows stay positive — spans are).
- `aeolus-gpui`: the walk applies the tier — the `GridState` accumulator (the `RingState`/`GradState` precedent) records the four start/end longhands' cascade winners and composes at the walk's end: an equal POSITIVE (start, end) pair — the registry's span fingerprint, ONE `col-span-3` fanning onto both longhands — composes GPUI's `Span` placement (the per-decl Line mapping would read a degenerate zero-width range, never Tailwind's `span 3 / span 3`); every other value maps 1:1 onto the line slots, negative lines included; an axis the program never mentions stays `Auto`. The state threading carries the placement through the interaction closures beside ring/grad (`hover:col-start-2` composes with the base's placement the way the cascade reads), and `FlexBasis` lands on the `flex_basis` slot in all three value shapes (spacing rem, fraction percentage, arbitrary percentage).
- Tests `aeolus-codegen-gpui/tests/placement.rs` (5 pins, RED-first — the tier flags, the emit gate and `lower_class` accepting the ten spellings, the audit-empty verdict, the conformance partition reading the same tables, the frozen emit), `aeolus-gpui/tests/placement.rs` (6 pins, RED-first — line placements on the `GridLocation` ranges, the span compose, the cascade overwrite, the negative from-end lines, the basis family's three shapes, the unmentioned-axis Auto), bench `benches/placement.rs` (line placements / span compose / basis family), example `examples/placement.rs` (the dashboard shell: `Line(2) .. Line(9)`, the `Span(2)` compose, `flex_basis 4rem`, the cascade, the boundary).

### 🧹 Micro-debt (this handoff)
- `crates/aeolus-core/src/color.rs`: `to_srgba`'s doc comment carried the pre-M1 framing ("no gamut mapping beyond clamping in v1; Tailwind's own palette is gamut-safe") — both halves stale now: the mapped conversion exists beside the clamp oracle, and the palette claim was false (the real default palette carries out-of-gamut rows — fuchsia-500 and shadcn's `--destructive` both overshoot sRGB). Rewritten for the two-conversion reality.
- `crates/aeolus-compile/examples/dyn_store.rs`: the `DynStore` storage seam's missing consumer example lands (a host's own 2-slot FIFO store through `DynCache::with_store` — the compile surface, the pointer-stable dedupe and the `resident()` observability riding unchanged) — every other P2 seam had a demo.

### 🚀 Features — the element-bounds container wrapper (M1)
- `aeolus-rt`: `MeasuredContainers` — the layout-phase hand-off (plan §3.5's `@container` row, the "who MEASURES the widths" half the docs kept assigning to the host): a shared, interior-mutable `ContainerProbe` (the `ClassCell`'s `Rc<RefCell<…>>` shape, `!Sync` by construction) the measuring element publishes real widths into and every walk reads through the seam's ONE question; `publish` answers whether the measurement CHANGED (the re-render handshake), a re-publish overwrites, `clear` resets on unmount.
- `aeolus-gpui`: `TwContainerExt::tw_container(name, &MeasuredContainers)` — the stateless measuring element (no per-element state, the name doubles as the element id, the child receives every phase unchanged): at prepaint it publishes its own assigned box's width in rem (`container_rem`, the `begin_frame` viewport arithmetic's sibling; a degenerate rem size is NO measurement) and asks for a re-render only on a real delta — a stable width never schedules a frame, a resize asks for exactly one. The wrapper never styles, the walk never measures; the frame beat is GPUI's own (the styles a frame builds read the previous frame's measurement — the animation lane's beat).
- Tests `aeolus-rt/tests/measured_containers.rs` (6 pins, RED-first) + `aeolus-gpui/tests/container_query.rs` (8 pins, RED-first — the walk-through-the-handle pins ride the built-in `ContainerState` table as the differential), bench `aeolus-gpui/benches/container_query.rs` (static-table walk 828ns / measured-handle walk 897ns / stable publish 13ns), example `examples/container_query.rs` (publish → walk → resize beat → interp differential, SAME verdicts).

### 🚀 Features — the shadcn program tables (M2)
- `aeolus-shadcn`: the closed-world PROGRAM TABLES land (plan §3.6's surface (2), §10's "shadcn components are `const` tables" — the F6 finding gone): `ButtonPrograms` (one interned cell per (variant, size) combo), `BadgePrograms`/`AlertPrograms` (per variant) and `BasePrograms` (input, label, the card family, the separators, skeleton, alert title/description) compile the VERBATIM v4 strings through the V1 pipeline ONCE per closed-world combo — a repeat call is the SAME interned `StaticProgram` (pointer-pinned), never the per-call `String` build plus a re-resolve. The token table is the handle's (the built-in `ThemeSource::shadcn` bake or a host's own, the `ClassCell` identity rule); the first compile's diagnostics ride the table and pin the documented boundary — every refusal is the arbitrary-variant/`has-*` family (`UnsupportedVariant`), never an unknown root, never a silent drop.
- Tests `aeolus-shadcn/tests/program.rs` (8 pins, RED-first — all-combos compile, interning, the table's layers ARE the canonical string's layers, token resolution, the documented-refusal battery), bench `benches/program.rs` (the F6 arms side by side; run via `cargo test -p aeolus-shadcn --benches` — the git-pinned iced chain refuses release builds without a renderer feature, the pre-existing reason the crate carried no benches), example `examples/program_tables.rs` (string path vs table hit, the interning proof, the refusals, the effective declarations).

### 🛠 Fixed — docs drift (this handoff)
- `README.md`: the `@md/card:` capability row and the container-measurement open boundary catch up with the measuring wrapper (`tw_container`); the P6 status row names it; the `aeolus-rt` and `aeolus-shadcn` crate-map rows gain `MeasuredContainers` and the program tables; the examples index lists the three new demos (`container_query`, `struct`, `program_tables`).
- `docs/AEOLUS_V1_ARCHITECTURE.md`: the §3.5 `@container` row, §3.6's surface (2) note, §7's landed note, §8's P3/P6 rows and §9's shadcn risk row file their rows for the wrapper and the tables.

### 🧹 Micro-debt (this handoff)
- `crates/aeolus-gpui/src/lib.rs`: the dead `Env as Environment` / `StaticProgram as Program` root aliases removed — zero consumers in-tree (tests, benches, examples, docs), the prelude re-exports both under their real names.
- `crates/aeolus-gpui/examples/struct.rs`: the structural seam's missing consumer example lands (the only probe-family member without one).
- `crates/aeolus-compile/src/style_walk.rs`: the broken intra-doc link resolves its crate path (`Cond::Structural` → `aeolus_ir::Cond::Structural` — the doc gate caught the pre-existing drift).

### 🚀 Features — the named-container walk half (M1)
- `aeolus-rt`: the bounds seam re-exported beside the probe seams (`ContainerProbe`/`ContainerState`/`ContainerBound`) and the `interp_containers` entry-point family joined the host surface.
- `aeolus-gpui`: the walk answers NAMED container queries through the seam — `tw_containers`/`tw_env_containers` and the `*_probes`/`*_struct` composed pairings (the interp's `interp_probes_containers_struct` matrix mirrored); `cond_matches` carries the seam arm, the verdicts are the interp `decide` semantics shared verbatim (AT/strictly-below boundaries, unknown-name quiet miss, sourceless quiet skip), the unnamed forms keep the `Env`'s nearest-container answer, and the element-bounds WRAPPER stays the host's half.
- Tests `aeolus-gpui/tests/containers.rs` (11 pins, RED-first — the differential pin walks `interp_containers` beside the walk), bench `benches/containers.rs` (sourceless 605ns / bounds-miss 603ns / bounds-hit 834ns), example `examples/containers.rs`.

### 🚀 Features — the GPUI motion lane (M2)
- `aeolus-gpui`: the §3.7 motion row's GPUI half lands as the STATEFUL lane — `aeolus_gpui::motion` parses the frozen v4 animate shorthand (`spin 1s linear infinite`) into `AnimSpec` (first duration f64-exact, easing keyword or `cubic-bezier(...)` with spaces, `infinite`, finite counts folding to one-shot, unknown functions never stealing the name), the CSS `--ease-*` curves solve by deterministic bisection, `animation`/`animation_for` build the `gpui::Animation`, `animation_keyword` answers the cascade's last matching `Animation` declaration under the probe env, and `pulse_animator` is the one `Styled`-expressible preset built in.
- `aeolus-gpui`: `TwAnimExt` — `tw_anim`/`tw_anim_env` and the dynamic `tw_anim_dyn`/`tw_anim_theme_dyn` entrances through the ungated `ClassCell` (last-good degradation) — the host's element id and animator beside the lane's parse; `animate-none` wraps the element statically, the animator never called; the `tw!` macro's capability gate keeps refusing motion classes (the static emit boundary stands, its reason now pointing at the lane).
- Tests `aeolus-gpui/tests/motion.rs` (14 pins, RED-first — grammar, curves, cascade winner, the lane wraps), bench `benches/motion.rs` (parse 360ns / build 412ns / winner walk 10ns), example `examples/motion.rs`.

### 🛠 Fixed — docs drift
- `README.md`: the Quickstart MSRV line now matches the manifests (1.88 for the compiler set, CLI and GPUI lane; 1.96 for the five framework backends + shadcn); the test battery re-derived (1144, 1154 instrumented); the missing consumer examples listed (`lower_class`, the two new demos).
- `docs/AEOLUS_V1_ARCHITECTURE.md`: the §3.5 container row and §3.7 motion row (plus the P6 roadmap line) catch up with the walk half and the lane.

### 🧹 Micro-debt
- `CHANGELOG.md`: the doubled `## [0.12.0]` heading — the second block was the same release's prose summary — demoted to a `### Release summary` subsection.
- `CONTRIBUTING.md`: the "StylePlan is the IR" guardrail was the v0.x framing — rewritten for the V1 `Copy` `Program` IR and the `lower_program` contract.
- `.gitignore`: the handoff-ceremony ignore pair carried three times — collapsed to one commented block.

### 🚀 Features — the structural-variant probe fold (M1)
- `aeolus-ir`: the `Structural` enum and `Cond::Structural` — the element's own DOM-position probe joins the typed surface (specificity band 2, the attribute-probe tier).
- `aeolus-compile`: the fold accepts `first:`/`last:`/`odd:`/`even:`/`only:`/`empty:`/`nth-N:`/`nth-last-N:` (the oracle's `parse_nth` grammar); the `StructProbe` seam + built-in `StructFacts`/`StructFacts::UNKNOWN`; the `interp_*_struct` entry-point family; `program_style_struct`; `InterpNote::StructuralUnqueried`; `structural_fires` exported as the shared verdict fn; the CSS arm renders the v4 positional selectors; the plan bridge maps onto the oracle's `Test::Structural` (differential battery; `nth-last-N` pinned as the oracle's own parser gap).
- `aeolus-codegen-gpui`: the static emit renders `Cond::Structural(Structural::…)`.
- `aeolus-rt`/`aeolus-gpui`: the seam re-exported; `tw_struct`/`tw_env_struct`/`tw_probes_struct`/`tw_env_probes_struct` answer the structural layers through the same verdicts.
- Backends: floem's walk reports the structural boundary honestly; the four program-lane backends translate the new note; the CLI's `expand` gains `--child i/n` + `--empty`.
- Tests `tests/structural.rs` (17 pins, RED-first), `aeolus-gpui/tests/struct.rs` (9 pins), bench `benches/struct.rs`, example `examples/struct_probe.rs`.

### 🚀 Features — the macro capability gate (M2)
- `aeolus-codegen-gpui`: `audit(program, keywords)` — the `Emit` impl's own `unsupported_reason` verdicts over a frozen program; `Unsupported` re-exported.
- `aeolus-macro`: `tw!`/`cn!`/`tw_cn!`/`variants!` run the gate before freezing — a class the GPUI target cannot express (motion families, the gradient via stop, the `BoxShadow`/`PointerEvents` slots) is a compile error naming the class and the emitter's reasons, instead of a silent runtime no-op.
- Tests `tests/gate.rs` (RED-first), bench `benches/gate.rs`, example `examples/gate.rs`.

### 🛠 Fixed — docs drift
- `SECURITY.md`: the supported-version table (0.12.x → 0.19.x) and the published-crate scope.
- `README.md`: the test count re-derived mechanically, the missing consumer examples (`struct_probe`, `gate`).
- `CONTRIBUTING.md`: the 1.88/1.96 MSRV split the CI actually gates is stated.

### 🚀 Features

- *(ir)* The motion longhands (the plan P6 row, the IR half): `Prop::TransitionProperty`/`TransitionDuration`/`TransitionTimingFunction`/`TransitionDelay`/`Animation` append at the cascade tail (116 → 121 longhands — the plan doc's own 110 was the stale figure this pass corrects; the mask budget test still asserts the u128 fit) — the `transition-*`/`duration-*`/`ease-*`/`delay-*`/`animate-*` families get their own typed slots, the row-table keyword texts riding where only the pipeline can read them
- *(data)* The motion rows (the plan P6 row, the registry half): `transition` (the v4 default set + the six set spellings), `duration`/`delay` (millisecond numbers), `ease` (the four v4 default curves) and `animate` (the four v4 default-theme presets + the `none` clear) — the keyword texts are the v0.x oracle's OWN render strings (`transition_property_list`'s lists, the `--ease-*`/`--animate-*` token values), exported as pub consts so the CSS arm renders and the plan bridge maps back from ONE source of truth; the five roots carry registry-local groups (the tw_merge oracle leaves them ungrouped — the transforms/fill/stroke precedent) and the roots index + registry docs page regenerated through the xtask drift gate
- *(compile)* The motion layer lowers end to end: the value pass fans the five families through the same row-driven walk (the `duration-*`/`delay-*` numbers rendering `{n}ms` through the new `NumForm::Ms`), the plan bridge folds the transition quartet into the oracle's ONE `Transition` extras slot (seeded with the resolver's own defaults — set `Default`, 150ms, no ease, 0ms delay — whichever longhand lands first) and parses the animation shorthand quirk-faithfully (the whitespace-cut `cubic-bezier(…)` fragment takes the resolver's own `InOut` fallback, so `animate-ping` reads the same on both sides), the CSS arm renders the v4 property forms, the interp/rust-tokens/static arms ride the generic surfaces, and `tests/motion.rs` pins the shapes, the boundaries (arbitrary timing values, negation, unknown preset names — typed refusals, never silent) and the `cn()` merge (two sets collide on `transition-property`, the rightmost survives); `tests/plan_bridge.rs`'s differential battery grew the eighteen motion classes, every one slot-for-slot against the v0.x resolver
- *(codegen-gpui)* The motion longhands are the reported capability boundary (the §3.7 transition/animate row's GPUI half): no transition slot on the style surface, `with_animation` needs an element id + state the frozen static program does not name — `unsupported_reason` names each refusal per longhand, the emit gate and the third-party `lower_class` seam both report (the CSS/interp/plan arms carry the real output)
- *(floem)* The backend walk reports the motion longhands honestly (no transition/animation surface at the pinned rev — the CSS emit arm's boundary, the plan path's own report shape)
- *(codegen)* The per-backend conformance score (the plan §5.1 publishing row): the `conformance` module — the `Capability` trait (one tiered verdict per IR longhand: native/polyfill/refused) and the `Conformance` report (the TOTAL, DISJOINT partition over `Prop::ALL`, the expressible-score arithmetic with the strict native-only share beside it, and the deterministic audit table the committed page renders) — the partition totality test-pinned so a capability table that forgets a longhand cannot inflate the score
- *(codegen-gpui)* The `Caps` capability impl reading the SAME `supports`/`polyfilled` tables the emit gates run — the published score is the emitter's verdicts, never a parallel claim (the per-declaration text-dependent refinements stay at the gates; the score is the per-longhand ceiling); the report pins: the ring family is the whole polyfill tier, the five motion longhands all refuse
- *(xtask)* The committed `docs/CONFORMANCE.md` page joins the GeneratedArtifact seam (`conformance-docs` — the third artifact beside the roots index and the registry docs page): rendered from the capability impls, written by `cargo xtask gen-tables`, drift-pinned by `cargo xtask verify-tables`
- *(codegen)* The conformance seam's benchmarks (`benches/conformance.rs`: the report walk and the rendered table) and the `aeolus-codegen-gpui` `conformance` example (a registered backend beside a hand-rolled third-party target, both scores rendered)
- *(compile)* The motion-layer benchmarks (`benches/motion.rs`: the five-family compile against the Button floor, the motion-mixed interp themeless and themed) and the `examples/motion.rs` demo walking every arm end to end

- *(compile)* The named-container BOUNDS seam (plan §3.5's `@container` row, the resolution half the named-container fold left open): the `ContainerProbe` trait (one question — what width does the container NAME carry?) with the built-in `ContainerState` static table (the `AttrState` precedent) and the `interp_containers`/`interp_theme_containers`/`interp_probes_containers`/`interp_theme_probes_containers` entry points (the `*_probes` convention; the `Compiled` convenience forms ride along). A named layer asks the seam for ITS container: a hit fires by the same AT/strictly-below boundary rules the viewport breakpoints obey, a container the source does not know is a QUIET miss (a CSS selector that does not match, no note), and the sourceless forms keep the `ContainerUnqueried` note; the unnamed forms keep answering from `Env::container_rem` beside the seam
- *(cli)* The `expand` command grew `--container name=rem` (repeatable): the named containers' widths ride the `ContainerProbe` seam — the headless form of the bounds answer; a malformed, non-finite or negative bound is a CLI input error (exit 2)
- *(compile)* The pseudo-element RENDER TARGETS (the interp arm's own-slice note, resolved): the `PseudoTarget` selector (`Selection`/`Placeholder`) and the `interp_pseudo`/`interp_pseudo_theme` entry points (the `Compiled` convenience forms ride along) — a pseudo-element styles a DIFFERENT render target, so its cascade resolves in its OWN walk: only the layers carrying the target's condition participate, the element's own layers never do (the element walk's `PseudoElement` note stays the element-side half of the same contract), and the answer is scoped and silent about the rest
- *(macro)* The `variants!` slot form (plan §9's open question, decided — the tailwind-variants shape, the compound-variant list's sibling): a `slots:` map declares one base per PART of a component (a card's root/header/title) and generates a `{Name}Slot` enum (default: the first declared slot) plus a `style_slot(slot, …)` dispatch — each part is its OWN `StaticProgram`. A group value or a compound `class:` that is a STRING applies to every slot; a brace `{ slot: "classes" }` applies to the slots it names (an unknown slot name, a per-slot brace in a table without `slots:`, a duplicate slot, an empty map and a compound on a group-less table are typed grammar errors); a `base:` beside `slots:` applies to every slot (prepended). The arms compile per (slot, combo) at expansion time, so the runtime cost stays the match dispatch the table always had (the slots bench: 0.7 ns, the pure dispatch)
- *(ir)* The named-container condition (plan §3.5's `@container` row, the v0.9 oracle's `@md/name` form): `Cond::Container`/`Cond::MaxContainer` carry the container's NAME (`Container { bp, name }` — the name interns into the same leak pool the attribute probes use). The fold used to split the label off and DROP it, merging `@md/card:` into `@md:` — two different containers collapsing into one cascade slot; the named and unnamed spellings are different conditions now
- *(compile)* The named-container fold threads the label: `@md/card:` folds to `Container { bp: Md, name: Some("card") }`, the empty name (`@md/:`) refuses typed (the unknown-variant refusal, never a silent unnamed fold), and the CSS arm renders the v4 form `@container card (min-width: 48rem)` (the named head, the unnamed form unchanged); the plan bridge fills the v0.x oracle's own `Atom::Container.name`, so the plan path renders the same named query the fold carried
- *(core)* The v0.x parser routes known-breakpoint named container spellings (`@md/card`) through the custom path's own split, so the oracle's `Atom::Container.name` carries the name it was BUILT to carry (the `@4xl/name` forms already did; the `@md/name` forms dropped the label at the named-variant parse — the differential triangle's disagreement, fixed in the oracle's favour)
- *(ir)* The gradient stop longhands (plan §3.7's `bg-linear-*` row, the IR half): `Prop::GradientFrom`/`GradientVia`/`GradientTo` plus the three `-position` slots — v4's OWN `--tw-gradient-*` custom-property set. The registry's stop rows folded onto `Prop::BackgroundImage`, the shorthand-shaped slot where the last stop REPLACED the gradient direction in the per-longhand cascade (`bg-linear-to-r from-primary` rendered `background-image: #171717`); the stops are their own longhands now, the per-edge family's precedent (F9's own lesson)
- *(compile)* The plan bridge composes gradients: the direction row's composed text parses into the shape through the ONE public read (`linear_gradient_angle` — 0° = to top, clockwise, the CSS convention), the stop slots accumulate per bucket with the v0.x oracle's last-wins semantics (a hoisted accumulator adopted per candidate — a `from-*` before or after its `bg-linear-to-*` composes the same gradient, the per-bucket reset keeps `hover:` stops home), `bg-none` lands as `Gradient::None`, and the un-composable placeholder texts keep the `GradientRow` note. The style fold shares the writer, so the FIVE plan-path backends get real gradients through the `background_gradient` slot they already lower
- *(codegen-gpui)* The capability gate's per-declaration half: `unsupported_reason` refines the per-prop tiers where the verdict is the ROW TEXT's — the direction longhand expresses exactly the `linear-gradient(to …)` directions and `bg-none` (the radial/conic/placeholder texts refuse typed, a text-less decl refuses rather than guessing), and the via stop refuses on the slot's own shape (GPUI's gradient carries two stops). The two-stop family (`GradientFrom`/`GradientTo` + their positions) joins the NATIVE tier; the emit and `lower_class` gates run the refinement over the keyword slice the pipeline freezes
- *(gpui)* The §3.7 gradient lane: `GradState` accumulates the gradient longhands the walk sees (the direction row's text recovering the angle — the annotation a bare `Val::Kw` cannot carry) and composes the linear gradient in ONE `bg()` call per walk (the `TwStateExt` closures seed the base walk's state, so `hover:from-blue-500` composes with the base's stops). The honest boundaries: a via stop paints NOTHING rather than a flattened three-stop misreading, the radial/conic/placeholder texts stay no-paint, and the missing edges compose v4's `@property` transparent initials so a lone `from-*` still fades
- *(data)* The stop rows retarget onto the new longhands (`from`/`via`/`to` colour rows → `GradientFrom`/`Via`/`To`, the percent rows → the position slots; the tw_merge groups unchanged) — the roots index and the registry docs page regenerated through the xtask drift gate
- *(macro)* The `variants!` compound-variant list (plan §9's open question, decided — the cva shape): a `compound: [ … ]` LIST of brace entries applies extra classes to every combo that matches ALL of an entry's `group: key` constraints (any declared group, not just `variant`/`size`), resolved at EXPANSION time — the entries fold into the cartesian match arms the render already emits, so the runtime cost stays the match dispatch the table always had; an unknown group key or a compound on a group-less table is a typed grammar error, and the plain grammar (base/groups/defaults) is byte-unchanged

### 🧹 Micro-debt

- *(compile)* The style fold's `concrete` helper returned a hand-prose `String` refusal — the typed `ConcreteRefusal` enum (no source wired vs the source's own typed failure) replaces it, machine-checkable, the public `(prop, reason)` report shape unchanged
- *(compile)* The CSS arm's `classify_segments` returned the refused segment as a bare `String` — it returns the IR fold's own typed `VariantMiss` now (the payload the spanned diagnostic renders)
- *(macro)* The unused `aeolus-core` dependency left the manifest (the compile walk is the V1 pipeline; nothing in the crate references `aeolus_core` — the stale "the zero-dep core lexer is still what `extract_*` avoids copying" comment went with it)
- *(readme)* The Features section's test counts were two milestones stale (849/859 → 1053/1063)
- *(readme)* The Features section's test counts were one milestone stale again (1053/1063 → 1064/1074 — the slot pins and the named-container pins joined the suite)
- *(compile)* `keyword_texts`' `config` param was a silent `let _ = config;` — the ignore is now the documented contract (signature parity with the compile surface; the spacing knob changes VALUES, never a keyword table's text)
- *(readme)* The Features section's test counts were one milestone stale again (1086/1096 → 1103/1113 — the motion pins, the motion emit-gate refusals and the differential battery additions joined the suite)
- *(xtask)* The driver tests' artifact-count pins grew with the list (`2` → `3`): the committed conformance page joins the seam the same one-impl-one-entry way the docs promise — the counts are the seam's growth, updated beside it

### 🐛 Bug Fixes

- *(core)* The v0.x named-variant parse dropped the `/label` on known container breakpoints before the atom's own name field could carry it — `@md/card` and `@4xl/card` disagreed inside the oracle itself; both carry the name now (see the feature entries)
- *(docs)* The architecture plan's `aeolus-legacy` row, the §5.2 oracle mention and the §7 freeze step described a frozen-copy crate that never existed — the v0.x surface stays in-tree in `aeolus-core` (the differential oracle) and `emit_plan` is the migration adapter; the drift is filed in place
- *(compile)* The gradient-stop cascade clobber: `bg-linear-to-r from-primary to-transparent` rendered `background-image: #171717` — the stop classes folded onto the direction's `BackgroundImage` longhand and the interp's per-longhand winner was the LAST stop's colour. The stop longhands split the family; the direction text and the stops coexist in one program (the regression is a named pin)

### 🧪 Testing

- *(compile)* Pin the container-bounds seam (`tests/container_bounds.rs`: the named min form AT the boundary and quiet below, the named max form strictly below, the unknown-container quiet miss against the sourceless reported exclusion, per-name independence, the unnamed-answer regression, the probe-composed full walk, the themed form, the `Compiled` form, a host impl answering through the trait — RED first: the `ContainerProbe`/`ContainerState`/`interp_containers` E0432s)
- *(compile)* Measure the container seam (`benches/container_bounds.rs`: the hit, the quiet miss and the themed form over a container-mixed program — 157–218 ns per resolve, the two-row table scan the seam adds)
- *(compile)* Demonstrate the container seam (`examples/container_bounds.rs`: sourceless reporting, the answered boundary and the quiet miss over one program)
- *(cli)* Pin the `--container` flag (`tests/cli.rs`: the sourceless report, the answered boundary, the answered quiet miss and the malformed-bound exit 2)
- *(compile)* Pin the pseudo-element targets (`tests/pseudo_targets.rs`: the selection/placeholder cascades, the other-target empty answer, the element layers never participating, the element walk's parity note, the value differential, the themed form, the `Compiled` form — RED first: the `PseudoTarget`/`interp_pseudo` E0432/E0599s)
- *(compile)* Measure the pseudo targets (`benches/pseudo_targets.rs`: the scoped walk over a pseudo-mixed program, both targets and the themed form — 47–52 ns per resolve, one pass three skips)
- *(compile)* Demonstrate the pseudo targets (`examples/pseudo_targets.rs`: the element notes beside both targets' cascades and the themed form)
- *(codegen)* Demonstrate the `Emit` seam (`examples/text_emit.rs`: a text target refusing per-longhand, the capability report in cascade order, nothing dropped silently)
- *(gpui)* Pin the slot form (`tests/variants.rs`: the per-slot dispatch, the string-vs-brace group values, the compound class in both forms, the group-less slotted table and the first-slot default — RED first: the `style_slot`/`CardSlot` E0599s)
- *(gpui)* Measure the slot dispatch (`benches/slots.rs`: the generated `style_slot()` match over the 3-slot × 3×3 combo cartesian — 0.7 ns, the pure dispatch)
- *(gpui)* Demonstrate the slot form (`examples/slots.rs`: the card's root/header/title parts, the string-vs-brace values)
- *(compile)* Pin the named-container fold (`tests/containers.rs`: the named/unnamed condition split, the max form, the empty-name refusal, the CSS `@container card (min-width: …)` head, the plan bridge's name fill DIFFERENTIAL against the v0.x oracle, the interp's unnamed-answer/named-report split, the static arm's Debug contract — RED first: the `Cond::Container` `bp`/`name` E0559s/E0026s)
- *(compile)* Measure the container fold (`benches/attr_variants.rs`: the named+unnamed corpus beside the state/probe folds — 1.2 µs cold over three segments, the intern cost the probe payloads pay)
- *(compile)* Demonstrate the named-container fold (`examples/named_containers.rs`: the fold, the CSS head, the bridge's atom, the interp's note)
- *(compile)* The statics emission-arm pin carries the named payload (`Container { bp: Xl, name: Some("card") }` through the IR's own Debug contract)
- *(compile)* Pin the gradient pipeline (`tests/gradient.rs`: the stop/token differential (`from-red-500` agrees with `bg-red-500`'s value — no absolute hex), the direction+stops coexistence pin, the position longhands, the alpha ride, the themeless typed refusal, the CSS `--tw-gradient-*` var forms, the interp arm's three-family resolution, THE DIFFERENTIAL — the bridge's composed gradient agrees with the v0.x oracle's over the linear/corner/position/via/radial/conic/none corpus, and the placeholder text still reports — RED first: the missing `Prop::GradientFrom` compile failures)
- *(codegen-gpui)* Pin the per-declaration gate (`tests/gradient.rs`: the native tier pins, the 2-stop program emits with its stop longhands frozen, the via/radial/conic/placeholder refusals carry their reasons, `bg-none` and direction-only programs emit, the missing annotation refuses)
- *(gpui)* Pin the gradient lane (`tests/grad.rs`: the two-stop composition at the CSS angle, the direction map (0/90/135/180/270/315), the from-only transparent to-stop, the position patches riding the stops, the via skip, `bg-none`, the accumulator unit surface, the direction-without-stops nothing)
- *(compile)* The plan-bridge note pin now reads the composed form (`bg-linear-to-r` lands in the Gradient slot, no note — the slot-for-note trade the stop longhands bought)
- *(gpui)* Measure the gradient lane (`benches/grad.rs`: the walk applying a `tw!`-compiled two-stop program over the solid+gradient corpus, the accumulator's own composition)
- *(gpui)* Pin the compound variants (`tests/variants.rs`: the entry rides only the matching combos, the one-group constraint across the other groups, the constraint keys naming any declared group, the base-only table unchanged — RED first: the `compound` expects a string or `{ … }` grammar refusal)
- *(gpui)* Measure the compound dispatch (`benches/variants.rs`: the generated `style()` match over the 3×3 combos with two compound arms — 0.7ns, the pure dispatch)

### 📚 Documentation

- *(readme/architecture/changelog)* File the gradient lane (§3.7's `bg-linear-*` row and the P6 ring/gradient note as landed — the gradient half of "ring/gradient polyfills", the README features/status lines, the example list)

### 🚀 Features

- *(floem)* The V1 static-program lane (plan §7's re-target, the second backend after GPUI): `lower_program`/`lower_program_with` walk a `tw!`-frozen `StaticProgram` into a Floem `Style` under the interp's `Env` probe semantics — the backend's OWN walk (one arm per registry longhand onto the pinned-rev surface: taffy layout, kurbo border strokes with the dash patterns the `border-style` longhand painted earlier preserved, peniko brushes, the grid repeat form; every longhand without a Floem slot reported per prop, never a silent zero), the `REM_PX` bake (§9's per-backend constant, default 16), the keyword texts the walk reads back through `StaticProgram::keyword`, and the `AttrProbe`/`TokenSource` seams wired on the `*_with` entry point (an unanswered probe is reported, an unresolvable token refuses into the report)
- *(xilem)* The V1 static-program lane (the third backend): the pipeline's SHARED walk composes with the widget's own resolution — `lower_program`/`lower_program_with` ride `aeolus_rt::program_style_with` (the interp's `decide` verdicts + the plan bridge's slot writer, so the string-side plan and the program-side style cannot drift) into the per-state `ResolvedStyle` the `StyledWidget` already consumes, `env_for_state` converts the `TwEnv` plus the four Masonry context state flags into the probe `Env` (widths baked through `REM_PX`, the focus flag feeding BOTH focus conditions — the plan path's documented normalization), and the walk's typed report (`InterpNote`s, `PlanNoteKind` boundaries, theme refusals) translates onto the crate's `LoweringReport`
- *(iced)* The V1 static-program lane (the fourth backend): the same shared walk composed with the crate's own mapping — `lower_program`/`lower_program_with` resolve a frozen program into the `IcedStyle` the Catalog closures paint (`program_style_with` → `ResolvedStyle` → `lower_resolved`, so a capability bump re-audits ONE backend mapping), `env_for_probe` converts the `IcedProbe` the parity path already keeps into the walk's `Env` (widths through `REM_PX`, focus feeding both focus conditions per the probe's own normalization, the unanswerable conditions honestly off), and the walk's typed report lands beside the lowering's own drops on the one `LoweringReport` channel
- *(egui)* The V1 static-program lane (the fifth backend): the same composition — `lower_program`/`lower_program_with` resolve a frozen program into the `EguiStyle` the immediate-mode frame paints (`program_style_with` → `ResolvedStyle` → `lower_resolved`), `env_for_probe` converts the `EguiProbe` (built per frame from the `Response` or the context) into the walk's `Env`, and the walk's typed report joins the lowering's drops — the per-frame re-walk immediate mode already performs, now over the frozen form
- *(slint)* The V1 static-program lane (the SIXTH backend — plan §7's re-target is complete): the same composition — `lower_program`/`lower_program_with` resolve a frozen program into the `SlintStyle` the element properties and generated snippets paint (`program_style_with` → `ResolvedStyle` → `lower_resolved`, so a capability bump re-audits ONE backend mapping), `env_for_probe` converts the `SlintProbe` the `TouchArea`/`FocusScope` flags feed into the walk's `Env` (widths through `REM_PX`, focus feeding both focus conditions per the probe's own normalization, the checked flag riding, the unanswerable conditions honestly off), and the walk's typed report joins the lowering's drops on the one `LoweringReport` channel
- *(gpui)* The §3.7 ring polyfill: `RingState` accumulates the four `--tw-ring-*` longhands the walk sees (the frozen row text recovering bare `ring`'s 1px — v4's `currentColor` default falls back to the element's own text colour from the SAME cascade) and composes the v4 box-shadow list — the offset band over the ring band, spread-only — applied in ONE `shadow()` call per walk (the `TwStateExt` closures seed the base walk's state, so `hover:ring-4` composes with the base's ring colour); `ring-0` leaves no layers behind
- *(codegen-gpui)* The capability table's polyfill tier: `polyfilled`/`expressible` beside `supports` (native stays honest), and the `Emit`/`lower_class` gates accept a program whose only non-native longhands are the ring family — `tw!("ring-2 ring-red-500")` emits where the third-party seam used to refuse

### 🐛 Bug Fixes

- *(compile)* The sibling-row gate's non-arbitrary half: a shape miss on a row whose ROOT another row already took stays quiet — bare `ring`/`border`/`outline` carry a colour-sibling row whose arbitrary arm accepts the bare root but has no group to lower, and that refusal shadowed the class the width row lowered (the macro path turned `tw!("ring")` into a compile error; the interp arm rendered it fine)
- *(slint)* The `content_color` slot (where every `text-{color}` class lands on the V1 plan bridge and the program lane) now feeds the Text item's colour — the slot the egui/iced/floem/xilem lowerings already read; the typography surface's own colour entry wins when both carry one

### 🧪 Testing

- *(floem)* Pin the program lane to the plan path (`tests/static_program.rs`: the differential against the plan bridge's own lowering, the keyword-text sufficiency, the honest report)
- *(xilem)* Pin the program lane to the plan path (`tests/program.rs`: the base layer, the `REM_PX` bake, the state layers under `env_for_state`, THE DIFFERENTIAL — the walk's per-state `ResolvedStyle` agrees with the widget's `resolved_for_state` over the shadcn button corpus, dark and wide, the token refusal/resolution through the `TokenSource` seam, the attr-probe seam, the honest report — RED first: the capture was the missing `lower_program`/`env_for_state` imports)
- *(iced)* Pin the program lane to the plan path (`tests/program.rs`: the base layer, the `REM_PX` bake, the state layers and the opacity bake under `env_for_probe`, the probe conversion, THE DIFFERENTIAL — the walk's per-probe `IcedStyle` agrees with `plan.resolve` + `lower_resolved` over the shadcn button corpus, the `StyleBook` payload agreement per theme, dark and wide, the token refusal/resolution, the attr-probe seam, the honest report — RED first: the capture was the missing `lower_program`/`env_for_probe` imports)
- *(egui)* Pin the program lane to the plan path (`tests/program.rs`: the same battery against the `EguiStyle` — the `Frame` slots, the hints, THE DIFFERENTIAL per probe combination, the `StyleBook::resolve_at` payload agreement, dark and wide, the token refusal/resolution, the attr-probe seam, the honest report; the composite `text-sm` row's line-height half stays egui's own reported boundary, the corpus rides `font-medium` — RED first: the capture was the missing `lower_program`/`env_for_probe` imports)
- *(slint)* Pin the program lane to the plan path (`tests/program.rs`: the base layer with the padding/brush/radius/text-colour slots, the `REM_PX` bake, the state layers with Slint's NATIVE opacity, the probe conversion, THE DIFFERENTIAL per probe combination over the shadcn button corpus, the `StyleBook::resolve_at` payload agreement, dark and wide, the token refusal/resolution, the attr-probe seam, the honest report — RED first: the capture was the missing `lower_program`/`env_for_probe` imports)
- *(gpui)* Pin the ring polyfill (`tests/ring.rs`: the one-band composition, the v4 offset-over-ring list order, the `currentColor` fallback through the text colour, `ring-0`'s clean clear, the accumulator unit surface, the cascade winner under a hovering env)
- *(codegen-gpui)* Pin the polyfill tier (the ring longhands refuse nothing, the native table stays honest, the emit and `lower_class` take the ring program)

### 📚 Documentation

- *(readme/architecture/changelog)* File the four backends' static-program lanes under their landed form (the §7 re-target note and the P6 row, the README crate table rows, the example list, the status paragraph — the Floem/Xilem/Iced/egui lanes had landed in code without their doc filing)
- *(readme/architecture/changelog)* File the slint lane (§7's re-target note now reads COMPLETE, the P6 row) and the §3.7 ring polyfill (the capability-table row, the P3 row, the README crate rows and example list) as landed

### 🚀 Features

- *(compile)* The IR-level overlay seam (plan §3.6's surface (2) note — "User overrides go through `cn!` which resolves at compile time when static, or by `Program::overlay` (mask + copy) when not" — the non-compile-time composition path): `aeolus_compile::overlay(&Program, &Program)` composes two programs rightmost-wins per longhand — same-condition layers fold by `Cond` equality, the result conflict-free (`base_conflicts()` empty) and specificity-sorted (stable, first-seen ties), total on empty sides — and `cn()` delegate-folds through the SAME walk (one implementation, the keyword-alignment precedent; the merge keeps its diagnostic channel and its N-ary fold, and the fold starts from the empty program so a single overlapping input still normalizes — every existing pin held); `Compiled::overlay` is the ergonomic half (the compile's reports ride), `aeolus_rt::overlay` the runtime re-export. The sketch's "mask + copy" landed as the Prop-discriminant sort + last-wins (the discriminant IS the cascade order); the masks still answer the conflict-freedom contract, test-pinned
- *(compile)* The project-build seam (plan §2's approach C — "whole-project scan + one generated artifact, dedupe globally" — and the P6 row's `cargo aeolus build`): the `scan` module types the extraction strategy (`ClassSource`, one question — "what class strings does this file name?"; the built-in `RustStrings` walks the quoted/raw single-line string literals with the documented dash/colon pre-filter, `scan` resolves every hit's file/line/col/offset location) and `build`/`build_with` assemble the whole-project artifact — the GLOBAL dedupe (one candidate compiles ONCE no matter how many sites say it; identical rules collapse by `(selector, media, body)` identity), the registry-ordered rules (one `sort_with` walk over the candidates' join decides the order — the cascade reads rule order, so `p-4` before `px-8` is semantics, not formatting; each candidate rides its head, the minimum `(bucket, row)` among its own classes, the honest tail stable first-seen), the self-contained theme vars block (a supplied `TokenTable` renders `:root` light values + `.dark` overrides through the emitter's own `rgba_text` forms and the `--color-` prefix the rules' `var(--color-…)` references use), and the honest diagnostics at EVERY location naming the offender (the dedupe must not hide where a miss lives) — the artifact is differential with the per-string CSS sheets (byte-equal rules, only the global walk added)
- *(cli)* The `build` command (the disk-walking form of the compile seam): files and directories in (explicit files ride verbatim, the directory walk carries the documented extension filter and skips `target`/`node_modules`/`.git`/`dist`/`.venv`, hidden entries and symlinks — deterministic sorted scan order), ONE stylesheet out — `--theme shadcn|shadcn-dark|<path>` lowering colour tokens through the V1 table and rendering the vars block, `--out <path>` writing the artifact (stdout without it), `--fail-on unknown|unsupported|any|none` mapping the typed codes onto the check-style exit codes, `--json` carrying the summary object (files, candidates, rules, the diagnostics per location; under `--json` the artifact rides `--out` only, findings ride the object — stderr stays quiet). Without `--theme` the colour tokens stay typed boundaries per location, the honest default
- *(cli)* `check`'s extractor delegates to the build seam's walk (`aeolus_compile::scan_rust_strings`): ONE byte-walk implementation answers both consumers now (the `check` linter and the `build` artifact) — the keyword-alignment precedent, third copy retired; the extractor's doc comment no longer describes a resolver-based filter the code never ran (drift fixed)

- *(theme)* The `@source` registration seam (plan §5.4's CSS-first compatibility line, item 4 — the last of the four named forms): `aeolus_theme::source_directives` types the three statements — `inline("…")` literals, walked `"path"`s, pruned `not "path"`s — source-ordered over a string-aware comment walk (a quoted `**/*.rs` glob is payload, never a comment opener), malformed statements (unterminated string, unrecognized payload, the bare form) as the typed `ThemeError::Source` — never a guessed registration; the registrations ride `ThemeSource::from_css` (`ThemeSource::sources`)
- *(cli)* `check` consults the theme CSS's `@source` registrations (a `--theme <path>` file): inline candidates join the lint attributed to the theme CSS, walked paths join the scan set relative to the theme CSS's own directory, `not` paths prune the registrations — a file the operator passed explicitly survives the prune (the operator's list wins); the built-in specs carry no CSS file and keep exactly the pre-`@source` surface
- *(rt)* The dynamic lane's view-state cell (plan §3.6's surface (3), the third user-facing surface): `aeolus_rt::ClassCell` — ONE slot keyed on the class string's exact content plus the compile identity (config + table/bag address, the `DynCache` collision-safety rule, never hash-only), UI-thread interior mutability (`RefCell`, `!Sync` via the `PhantomData<Rc<()>>` marker — no locks, no atomics, the plan's own note), holding the interned `StaticProgram` the compile stages already leak (the macro's frozen form, runtime-built). A hit hands back the SAME interned slices pointer-stable; the four `get`/`get_theme`/`get_with`/`get_theme_with` entry points mirror the whole compile surface; per-class misses ride the typed `Diagnostic` channel; the empty input errors and leaves the last good program standing (errors never cached); `hits()`/`misses()` are the observability counters
- *(compile)* The keyword-alignment seam: `keyword_alignment(&Compiled, &[ClassKeywords])` renders one row text per flattened declaration in layer-major order — the single implementation the `tw!` freeze, the `variants!` emitter, the codegen target's `lower_class` and the dynamic cell all walk (the alignment used to be hand-rolled in the macro, the codegen-gpui crate and the `static_walk` example; three copies, one seam now)
- *(gpui)* The dynamic lane's GPUI half: `TwExt::tw_dyn`/`tw_dyn_env`/`tw_dyn_theme`/`tw_dyn_theme_env` compile a dynamic class string through the cell and apply it under the frame `Env` through the SAME walk the static programs take — a failed compile degrades to the last good program (never a panic per frame, never a silent reset; the cell's diagnostics carry the reason), and `ClassCell` joins the prelude
- *(data)* The registry's sort-order read seam (plan §5.5's "class sorting (official order from the registry)"): `registry::sort_index` answers a utility BODY's canonical order position — the FIRST matching row's index, under the same longest-prefix + shape rules `lookup` applies, `None` on a typed miss (no root, rejected shape, never a guessed position) — and `registry::row_count()` names the boundary the custom registrations sort after. The rows' own order IS the canonical utility progression; the pre-index linear scan stays as the differential oracle, battery-pinned over every root and value shape
- *(compile)* The class-sort seam (`sort`/`sort_with`): one read-side walk orders a class string into the registry's progression — base classes first by row index, variant buckets after the base by the cascade specificity the IR itself sorts layers by (`Cond::specificity`, base < dark < probes < parent < media < container), registered custom utilities after every registry row in registration order (the registry wins name collisions; empty registrations are byte-identical to `sort`) — and classes the seam cannot ORDER (lex failures, unsupported variants, registry/custom misses) ride the source-order tail with the SAME typed, coded diagnostics the compile walk reports. The sort is stable (equal keys keep source order), idempotent, and total on any non-empty input; the value pass and the theme pass are deliberately NOT consulted (order is a shape-level fact — a themeless token boundary sorts exactly like its lowered sibling)
- *(compile)* The interp arm's winner attribution: `Interped::conds` rides parallel to `decls` — the condition each effective declaration's winner came from, the cascade's winner attribution the row-relative `Val::Kw` annotation texts render from (a host reading `decls[i]` beside `conds[i]` knows WHICH condition won the longhand without re-walking the layers). Additive field; every existing read site is unchanged
- *(cli)* The `expand` command (§3.2's CLI row, the interp arm's human-readable form): a class string's EFFECTIVE declarations under one probe `Env` — the per-longhand cascade, the keyword slots resolved to their WINNER's row text through the `(cond, prop)` annotation table (an unfired layer's text can never shadow the winner's), the honest exclusions as typed notes, `--theme shadcn|shadcn-dark|<path>` lowering tokens through the V1 table (the themed walk rides `interp_theme`), `--dark` lifting the mode tier, `--json` carrying the declarations array plus notes. The command joins `aeolus-ir` (workspace-internal) for the `Val`/`Prop` render types
- *(cli)* The `sort` command (§3.2's CLI row, §5.5's parity line): `aeolus sort [--json] <classes...>` renders the registry-ordered string, the typed misses as exit-1 findings (the sorted string still renders — the honest tail), `--json` carrying the sorted string plus the diagnostics array. The CLI joins nothing new: the command rides the workspace-internal `aeolus-compile` join the `explain` command established
- *(gpui)* The attribute-bits host state seam (P4's last named piece, plan §3.5's "typed state bits passed WITH the element"): `TwExt::tw_probes`/`tw_env_probes` walk the base + environment layers with the element's attribute bits in hand — the GPUI half of the interp arm's `interp_probes` entry point. `data-*:`/`aria-*:` layers answer per CSS selector semantics (a presence spelling fires on any value, a value spelling on equality, a miss stays CSS-quiet); the sourceless walk keeps them quiet exactly as before (the interp's `AttributeUnqueried` note has no GPUI channel), and the `has-*` descendant shapes stay the plan's diagnosed boundary with or without bits. The seam is the trait: the built-in `AttrState` bit table joins the prelude beside `AttrBit`/`AttrKind`/`AttrProbe`, and a host `AttrProbe` impl answers the same walk
- *(cli)* The `explain` command (plan §5.5's tooling-parity line, "class → Rust it emits"): per-class Rust fragments the Target::Rust arm renders (one embeddable `Layer::new` expression per lowered class, keyword/token slots annotated), every miss as the SAME typed, coded diagnostics the pipeline reports (exit 1 findings), `--theme shadcn|shadcn-dark|<path>` lowering colour tokens through the V1 token table (the themeless emit keeps the token boundary honest — the diagnostic names the lowering path). The CLI joins `aeolus-compile` (workspace-internal; the no-external-crate discipline holds)

### 🧪 Testing

- *(compile)* Pin the overlay seam (`tests/overlay.rs`: last-wins per longhand (`p-4 ⊕ px-8`, the overlap exactly `base_conflicts()`'s bits), the same-condition fold and the disjoint layers, the specificity resort independent of composition order, THE DIFFERENTIAL (`cn(&[a,b,c])` == `overlay(overlay(a,b),c)`, the pairwise fold groups associatively), idempotence and associativity, the honest interning story (each composition interns its result — the `cn` story, no cache over the composition surface), the empty-side totality, the frozen⊕runtime composition (§3.6's use case), the `Compiled::overlay` ergonomic half with its diagnostic ride — RED first: the capture was the missing `overlay`/`Compiled::overlay` compile failures)
- *(compile)* Pin the project-build seam (`tests/build.rs`: the extraction walk with its one-based line/col/offset locations, the raw-string and single-quote forms, the prose pre-filter, the scan determinism, the global dedupe across files, the sort-seam-derived rule ORDER (the differential form: the artifact's global order is the sort seam's own decision, never re-derived), the variant-bucket candidate order, the self-contained vars block with its `.dark` overrides and its vars-before-rules reading order, the themeless token boundary per location, the per-location diagnostics of a twice-named offender, the byte-equality differential against `emit_css_theme`, the at-rule grouping, the customs registration with the registry winning the collision, the empty-scan empty artifact, the vars-only emission — RED first: the capture was the missing `scan`/`build`/`RustStrings` compile failures)
- *(cli)* Pin the `build` command contract (`tests/cli.rs`: the stdout artifact with its cross-file dedupe, the `--theme` vars block and token lowering, `--out` writing beside the summary line, the `--json` summary object with the typed codes and quiet stderr, `--fail-on` mapping unknown/unsupported/any, the walk skipping `target/`, explicit files ignoring the extension filter, the usage errors — RED first: the capture was the unknown-command compile failure)
- *(compile)* Measure the overlay seam (`benches/overlay.rs`: the base ⊕ override composition over the shadcn Button surface, the frozen⊕runtime shape, and the `cn` fold that delegate-walks the seam)
- *(compile)* Measure the build seam (`benches/build.rs`: the extraction walk over the synthetic 8-file project, the themed global artifact, the themeless floor — the dedupe's saving is the story: ~9 unique candidates compile, not the 72 occurrences)

- *(theme)* Pin the `@source` seam (`tests/sources.rs`: the three forms in source order, the typed distinctness, absence scanning to nothing, the malformed forms' typed errors, the carried form on `ThemeSource::sources`, comments and bodies never leaking, the quoted glob surviving the comment walk — RED first: the capture was the missing `source_directives`/`sources` compile failures plus the three `check` command failures)
- *(cli)* Pin the `check` wiring (`tests/cli.rs`: the inline candidate linted and attributed to the theme CSS, the walked path joining the scan set, the `not` prune with the explicit-file survival, the malformed directive as exit 2, the built-in theme's unchanged surface)
- *(theme)* Measure the scan (`benches/tokens.rs`: the three-form statement walk, the clean-file floor over the embedded palette)
- *(rt)* Pin the cell end to end (`tests/class_cell.rs`: the pointer-stable hit and the counters, the content+identity slot key (a second live table is a miss), the string-change swap, the Empty error keeping the last good program standing (the miss counted, the error never cached), the per-class diagnostics channel and its clean-recompile clear, the fresh cell's empty program, the themed lane resolving the token where the themeless lane reports the boundary)
- *(compile)* Pin the alignment seam (`tests/alignment.rs`: layer-major totality, duplicate `(cond, prop)` pairs consuming in source order (`hidden flex block` keeps the class order), non-keyword values aligning empty, the per-layer variant buckets, the refused class aligning nothing — RED first: the capture was the missing `keyword_alignment`/`ClassCell`/`tw_dyn` compile failures across the three suites)
- *(gpui)* Pin the dynamic half (`tests/dyn.rs`: THE DIFFERENTIAL — the cell's themed program is byte-equal to the `tw!` macro's frozen program; `tw_dyn` applying to a `StyleRefinement`; the token boundary reported; the last-good degradation; `tw_dyn_env` lifting the dark layer)
- *(rt)* Measure the lane (`benches/class_cell.rs`: the content-compare hit, the render-side program read, the rotating themeless and themed misses — the full pipeline per miss)
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

- *(compile/rt/readme/architecture)* File the overlay seam under its landed form (the §3.6 surface (2) landed note, the README compile row and status paragraph and example list, the crate docs' seam story, the `rt` docs' seam list, the `program_overlay` example)
- *(readme/architecture)* File the project-build seam under its landed form (the §2 approach C row, the P6 roadmap row, the README CLI row and shell walkthrough and example list and status paragraph, the crate docs' seam story)
- *(readme)* Correct the `check` walkthrough line (the command takes named FILES — a directory is a usage error; the drift showed `check src/`)
- *(compile)* File the build seam under its landed form (the crate docs' project-build paragraph beside the Target arms)
- *(theme/readme/architecture)* File the `@source` seam under its landed form (the §5.4 item's landed note, the README index and example list, the `check` USAGE footer, the `theme_sources` example)
- *(rt)* File the dynamic lane under its landed form (the crate docs' seam list; the `ClassCell` module docs carry the sketch-to-landed delta — exact content over the sketch's `hash: u64`, the interned `StaticProgram` over the sketch's `Rc<Program>`, the `!Sync` marker in type-system form)
- *(architecture/readme)* Mark the §3.6 surface (3) landed (the P3 roadmap row, the §3.6 landed-note, the crate table, the example list and the README architecture index)
- *(rt)* Walk the `static_walk` example through the alignment seam (the hand-rolled queue the example carried is the seam now)
- *(data)* File the sort-order read seam under its landed form (the crate, `rows` and `registry` docs no longer list the class-sort order as future generator work)
- *(readme)* Add the `sort` and `expand` commands to the CLI crate row, the shell walkthrough, the example list and the status paragraph; mark the §5.5 class-sorting half and the interp winner attribution landed in the architecture index
- *(architecture)* Mark the class-sorting half of the §5.5 tooling-parity line landed (the P2 status paragraph)
- *(readme)* Align badge/status to v0.19.0, the six-backend crate table (xtask joins), the GPUI tagline, the `explain` command and the new examples
- *(architecture)* Mark the attribute-bits host state seam landed (the P4 row and the remaining-boundaries line)
- *(ci)* Correct the workflow header's member count (19 members, six backend crates)
- *(changelog)* Give the shipped 0.19.0 its release header (the items sat in Unreleased since the tag) and remove the stale 0.15-era `Unreleased` block

### 💼 Other

- *(xilem)* Give the program lane its measurement and its consumer demo (`benches/program.rs` — the button walk, the hover-frame shape and the plan path's resolution as the differential baseline, the divan idiom the Floem/Iced/egui lanes landed with — and the headless `examples/program.rs` walkthrough; the lane's first landing shipped neither)
- *(iced)* Document the bench profile boundary on the bench registration (`iced_renderer` 0.14 compile_errors in release without a renderer feature — its own `#[cfg(not(debug_assertions))]` gate — so the lane's microbenchmark runs under `cargo test --benches`/`cargo bench --profile dev`, the headless no-renderer discipline the crate already documents)
- *(floem/iced/egui/xilem)* File the lanes' divan benches under `[[bench]]` registrations with the dev-dep joins (`aeolus-compile`, `aeolus-theme`, `divan`)
- *(gpui)* Remove the dead `TwDiv` alias (no consumer in any crate, the README or the docs — the examples name `Div` directly)
- *(cli)* Remove the dead statement in the `expand` render's keyword arm (`value_text`'s no-op `_ = kw;` line)
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

### Release summary — 0.12.0, the P2 slice 3: the IR-level `cn()` merge

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
