# Aeolus 1.0 — Zero-Runtime Architecture Plan

> **Goal.** Turn Aeolus from "a Tailwind v4 interpreter that runs inside your UI loop" into
> "a Tailwind v4 _compiler_ whose output is the code you would have written by hand".
> First consumer: **`aeolus-gpui`**, replacing the Tailwind half of `gpui-rsx` with real v4 semantics.
> Breaking the API is allowed and expected (→ `1.0`).

**P1 status (2026-11).** Phase P1 — the data-driven core — has landed
as the `aeolus-syntax` (zero-copy lexer, differentially pinned to the
v0.x parser over the golden corpus), `aeolus-data` (registry-as-data,
213 rows since the decoration family joined the tail, group names
cross-checked against the tw_merge oracle, plus
the typed `lookup_detailed`/`LookupError` read seam) and `aeolus-ir`
(longhand `Prop`/`Val`/`Decl`/`Layer`/`Program` — 146 longhands, the
decoration half's three the latest append-only block — +
`PropMask` bitset algebra) workspace crates, git-only while the
pipeline assembles. The §8 row's "registry + generator" generator
story (xtask-emitted phf/trie) landed in P2 2026-11: the committed tables ARE
the P1 source of truth (as §3.2 always said) and the generator renders FROM
them — the roots index and the docs page are derived, drift-pinned artifacts.

**P2 status (2026-11).** The pipeline's first seam is in-tree:
`aeolus-compile` — the §3.1 pure function as
`compile(&str, &Config) -> Result<Compiled, CompileError>` — runs
class strings through the P1 crates end to end (lex → registry lookup
→ longhand lowering) and emits cascade-sorted `Program`s with typed,
spanned diagnostics for every class that does not lower. The THEME
half of the §3.2 signature is in-tree too (slice 2): `aeolus-theme`
resolves the §3.4 CSS-first source (`@theme`/`:root`/`.dark`, the same
file a web project uses) into tiered `TokenTable`s — T0 literals
(`Val::Rgba`), T1 light/dark const pairs (`Val::ModePair`), T2 runtime
slots (`Val::Tok { id, alpha }`) — over the audited v0.x theme reader,
and `compile_theme(&str, &TokenTable, &Config)` lowers colour tokens
(the 270-entry table the u8 `Kw` slot could not hold) and `/opacity`
modifiers (the §2 premultiplied identity, one multiply) through it.
The IR-level `cn()` merge is in-tree as well (slice 3, this §3.2
row's "hosts the `cn()` engine at IR level" + this §3.3 note's "mask
algebra + copy"): `aeolus_compile::cn(&[&Compiled])` folds
same-condition layers by `Cond` equality and keeps the LAST
occurrence per longhand — the differential contract is test-pinned
(merging resolves like compiling the concatenation; the removed
declarations are exactly `Program::base_conflicts()`'s bits, which
come out empty), the audited `tw_merge` table stays at the string
level, and the merge is idempotent and associative.
The custom-authoring seam (plan §5.4's `@utility`/`@custom-variant`
line) landed 2026-11 as P2 slice 5: `aeolus_compile::Customs` register
custom utilities (fixed literals, functional `integer`/`number`/
`percentage`/`spacing` forms and the `--value(color)` theme-token form
through the tiered tables — modifier and all) and custom variant
aliases onto the typed `Cond` surface, and the `*_with` entry points
lower and render them through the same typed read side, value pass,
diagnostics and emit paths as the built-in surface. The registry wins
name collisions, the fold is generic over the `ClassifyVariant` seam
trait, empty registrations are byte-identical to the plain entry
points (pinned), and both walks share the refusal set (the
diagnostics-equal batteries pin it with customs in play).
The LAST named P2 slice — the xtask
generator story — landed 2026-11: `aeolus-data`'s `codegen::GeneratedArtifact` seam renders the committed
artifacts from the rows (the roots index the read side consults, the
`docs/REGISTRY.md` audit page), `crates/xtask` drives them
(`gen-tables`/`verify-tables`), the pre-index linear scan stays as the
documented differential oracle, and the drift pins run in plain
`cargo test`. The Target half's four emit forms are ALL armed now:
the CSS arm landed 2026-11 as P2 slice 4 — see the P1/P2 rows and the
crate docs — the other two emit forms landed 2026-11 as well: the
INTERPRETER arm, plan §3.1 emit (c): `interp`/`interp_theme` resolve a
compiled `Program` to its EFFECTIVE declarations under one probe `Env`
— the per-longhand cascade the specificity sort encodes, theme
dynamism through the `TokenSource` seam (the built-in `TokenTable`
answers with the T2 tier's one-load-one-multiply through the
mode-aware `TokenTable::resolve_rgba` primitive; a host source wires
in through the same two questions), the honest exclusions reported as
typed notes (`InterpNote`) — and the STATIC-DATA arm, emit (b):
`emit_static` freezes the program into the IR's OWN static form, one
compilable byte-stable `static` item over `Layer::new` constructors
(masks derive at construction, every `Val`/`Prop`/`Cond` variant has
an emission arm, the text is deterministic) — and the RUST-TOKENS arm,
emit (a), landed 2026-11 as the LAST arm of the `Target` surface:
`emit_rust`/`emit_rust_theme` render one embeddable `Layer::new`
expression per lowered class (the per-class fragment the `tw!`-style
macro's unit), the keyword slots resolved to their ROW text and the
runtime token slots to their TABLE names — the two annotations a bare
`Program` cannot recover, which is why the arm renders IN the pipeline
(the CSS arm's stated rationale); the sheet's `Display` assembles the
compilable static item, layers specificity-sorted, while the fragments
stay per-class in source order. The backend METHOD calls the emit (a)
end form implies (`FlexDirection::Column`, builder chains) remain the
P3 codegen crate's surface. The dyn-cache slice
landed 2026-11 as P2 slice 6: `aeolus_compile::DynCache` is the
leak-pool dedupe the compile seam's storage note promised — a
content-keyed (never hash-only), bounded (two-generation rotation
with promotion, the v0.x `PlanCache`'s own design), thread-safe cache
over `compile`/`compile_theme`/`compile_with`/`compile_theme_with`;
a hit hands back the SAME interned slices the miss pooled
(pointer-identical, test-pinned), the config/table/registrations
identity is exact equality (addresses, the v0.x `(ptr, len)`
precedent — one table and one bag per session), and the storage is a
SEAM (`DynStore`, built-in `TwoGenStore`) so a host swaps in its own
bounded map without touching the pipeline. The cache's observability
face landed 2026-11 beside the storage seam: a host-supplied
`DynObserver` (`DynCache::with_observer`) sees every decision in
order — hit/miss/store, the composed key the store itself probes, the
store event carrying the entry's diagnostic count — with the built-in
`CacheStats` collector (`snapshot`/`hit_rate`, one shareable handle
spanning caches) the ready-made aggregation and `NoObserver` the
every-constructor default; the differential pins every contract
byte-identical with and without an observer, and the bench set pins
the seam's cost on the hot path it rides. The face's EVICTION HALF
landed 2026-11: the built-in store's leave-takings — the rotation's
discarded generation and the stale twin a fresh store clears — ride the
observer's `on_evict` under the typed `EvictReason`
(`Rotation`/`StaleTwin`) and the SAME composed-key vocabulary, the
wire-up reaching the store through `DynStore::attach_observer` (the
trait is the seam, the built-in store a consumer of it — the
`SuggestionSurface` precedent), the callbacks firing AFTER the store's
write lock releases so a host observer can consult the cache it watches
without deadlocking it, the `CacheStats`/`CacheSnapshot` session totals
growing the evictions count (the cap's pressure number), a promotion a
move and never an event, and the observation-free rotation keeping its
allocation-free shape (the report only exists beside a wired observer).
The seam surfaced
honest boundaries, all test-pinned: the IR's u8 `Val::Kw` slot cannot
index the 270-entry colour table (RESOLVED by the theme pass's u16
`Tok` slot; the T1 u16 `ModePair` budget is its own typed
`ThemeError::PairOverflow` refusal), the flat `Cond` model folds
stacked variants to the dominant condition (the named group/peer
label has no IR slot yet — the CSS emit arm renders it from the
candidate's segments, which is why the emit lives in the pipeline),
and the f32 grammar's non-finite spellings
are typed refusals, never IR literals. The §5.5 TOOLING-PARITY LINE'S CLASS-SORTING HALF
landed 2026-11: `aeolus_data::registry::sort_index` (the canonical order
position — the FIRST matching row's index; the pre-index linear scan
stays as the differential oracle) composes with the variant fold into
`aeolus_compile::sort`/`sort_with` (registry order, the variant buckets
by the cascade specificity the layers sort by, registered customs after
every registry row in registration order) and the CLI's `sort` renders
it headless — unsortable classes ride the source-order tail with the
pipeline's own typed diagnostics. The CLI row's `expand` command landed
beside it: the interp arm's human-readable form (the EFFECTIVE
declarations under one probe `Env`, `--theme`/`--dark` flags, typed
notes), riding a new interp surface — each winning declaration now
carries its winner's condition (`Interped::conds`, parallel to `decls`)
— the cascade's winner attribution the row-relative `Val::Kw` texts
render from. The ARBITRARY-VALUE UNIT
GRAMMAR landed 2026-11 (the value pass's last named P2 slice): the
bracket/paren groups the registry rows always accepted now LOWER —
px/rem/%/deg units into the IR's own slots (rem first-class, §3.3),
colours through the audited `aeolus_core::Color` parser, `/opacity`
through the audited premultiplied bake, `var()`/`(--var)` references
through the theme pass's token path, the
`length:`/`percentage:`/`number:`/`color:` hints constraining the
parse (the lexer's own `HINTS` table — one source of truth, now
exported), and the sibling-row class gate keyed on the rows' own
tw_merge groups keeping the diagnostics quiet where another row of
the root takes the class (`text-[13px]` is the font-size row's alone;
`text-[#0f0f0f]` the colour row's). The COMPOSITE ARBITRARY-VALUE GRAMMAR
landed 2026-11 (the value pass's verbatim slot): the sized functions
(`calc()`/`min()`/`max()`/`clamp()`) lower on the single-target scalar
rows, the underscore-joined track lists and track functions
(`repeat()`/`minmax()`/`fit-content()`) on the grid-template/auto rows,
the image functions (`url()`/`image()`/the gradients) on the bg-image
row and the bare `n/d` ratios on the aspect row — the decoded text (the
underscores ARE spaces, `\_` a literal underscore) riding the IR's
`Val::Verbatim` slot (the conds' own interned-slices discipline, the
static form staying a `static` item end to end), the kind gate refusing
TYPED on every row whose slots cannot carry the shape (the shorthand
fans, the filter/backdrop handles, the shadow lists) plus the malformed
functions, the negative flag and the opacity modifier, the CSS arm
rendering the text as typed, the interp and the `cn()` merge answering
it, the plan bridge reporting it (the `CompositeValue` note, the
`SlotUnsupported` precedent) and the metamorphic triangle agreeing on
the text. The honest boundaries stay typed, all test-pinned: the
arbitrary-PROPERTY form and the keyword values on the row-relative
`Val::Kw` slots (the WIDER V4 HINT TABLE landed 2026-11: `angle:`/`time:`/
`position:`/`family-name:`/`generic-name:`/`absolute-size:`/`relative-size:`/
`string:` lower onto the rows whose slots carry the shape — the degree and
millisecond number forms, the object-position/font-family/font-size rows and
the content root's NEW `Prop::Content` longhand, the CSS `content` property
the `content-["…"]` spelling writes — with the free `ms`/`s` time units
riding the deg precedent and the CSS arm's arbitrary-Num path reconstructing
the unit from the row's own number form; the keyword-value slots and the
arbitrary-property form stay the typed refusals). P3 (codegen/GPUI) landed 2026-11: `aeolus-codegen`
(the backend-neutral `Emit` seam and capability report),
`aeolus-codegen-gpui` (the static-program token emitter, the GPUI
capability table and the `lower_class` third-party seam),
`aeolus-rt` (the host-facing `StaticProgram`/`Env`/`TokenTable`/
`DynCache` re-exports and `theme_from_css`), `aeolus-gpui`
(`TwExt`/`TwStateExt`, the frame `Env`, `AeolusRoot` and the prelude)
and the reworked `aeolus-macro` (`tw!`/`cn!`/`tw_cn!`/`variants!`/
`theme!` compile the V1 pipeline and emit the frozen static program —
the input string is never emitted back). P4's dark/breakpoint Env
plumbing and state layers are in the same pass, and the attribute-bits
host state seam landed 2026-11 — `tw_probes`/`tw_env_probes` answer the
`data-*`/`aria-*` layers through the `AttrProbe` trait (the interp's
`decide` semantics at the host); container queries' walk half landed
in the same pass, and the element-bounds measuring wrapper landed
2026-11 beside it (see §3.5's row).

**Status of evidence.** Everything in §1 comes from reading `elcoosp/aeolus` at the
v0.9-in-tree rev (`47ecfba`, 2026-11 — the v0.8 set-mask + the variant & grid engine
landed; the workspace version AT THAT REV was still 0.7.0 — it released as the
v0.8.0/v0.9.0 tags days later) and `wsafight/gpui-rsx` (v0.12.1), plus GPUI's
`style.rs` / `styled.rs` / `elements/div.rs` on `main`. The code-level findings were
**re-verified against that tree by the v0.9.1 consumer-handoff**: F1 (`tw!` still emits
the input string back), F2 (`resolve_static` still takes the exclusive lock on a hit),
F6 (`button_classes() -> String` per call) and F8 (`resolve.rs` is 4022 lines) all still
hold; F3's numbers remain the README's own quotes. **I could not build or benchmark
anything** (no Rust toolchain in the analysis sandbox). Every number
labelled _measured_ is quoted from Aeolus' own README; every number labelled _target_ is a proposal that
Phase 0 must validate before anything else is built. Claims about GPUI internals should be re-checked
against the GPUI revision you pin.

---

## 1. Diagnosis: where the runtime cost actually lives

### 1.1 The compile-time work is thrown away

`aeolus-macro::tw!` already runs the **full** parser at compile time… then emits the input string back
unchanged (`out = format!("\"{}\"", literal…)`, with a comment: _"Later phases replace this with interned IDs"_).
So every `tw!("p-4 bg-primary")` is validated once by rustc and **re-parsed, re-resolved and re-cached at
runtime** on every process start. That is the single biggest architectural mistake: the information exists at
compile time and is discarded.

### 1.2 The runtime path, as it exists today

```
&str ─▶ expand_utilities (allocs a String)
     ─▶ parse_class_string (Vec<ClassToken>)
     ─▶ Bucket/BTreeMap per layer ─▶ finalize() (4k-line string-`match` walls in resolve.rs)
     ─▶ StylePlan { Vec layers, 2×BTreeMap, Vec group/peer/rules, ResolvedStyle{ Vec text, Extras{Vec…} } }
     ─▶ PlanCache: L1 thread-local map / L2 RwLock<FastMap<Arc<str>,…>> / Arc<CachedPlan>
     ─▶ backend lower(&StylePlan) ─▶ native style   (again, at runtime, per call)
```

Concrete findings:

| #   | Finding                                                                                                                                                                                                                   | Where                          |
| --- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------ |
| F1  | `tw!` discards its compile-time result                                                                                                                                                                                    | `aeolus-macro/src/lib.rs`      |
| F2  | `PlanCache::resolve_static` takes `static_map.write()` — an **exclusive lock even on a cache hit**                                                                                                                        | `cache.rs` `resolve_static`    |
| F3  | Warm hit ≈ **405 ns** _(measured, their README)_; cold 33-class shadcn Button ≈ **4.6 µs** _(measured)_. Per element, per render. 1 000 elements at 120 fps ⇒ ~0.4 ms/frame (~5 % of an 8.3 ms budget) in _lookups alone_ | README                         |
| F4  | Theme tokens are `HashMap<String, Token>` queried by name (`theme.color("primary")`)                                                                                                                                      | `theme.rs`                     |
| F5  | The IR is heap-shaped: `Vec`, `BTreeMap`, `Box<str>`, `Option<Vec<TrackComponent>>`; ~25 `Option<…>` slots in `ResolvedStyle`. Pointer-chasing, `PartialEq` diffs, `clone()` to apply                                     | `plan.rs`                      |
| F6  | shadcn components build a **`String`** per call (`button_classes() -> String`) and then resolve it                                                                                                                        | `aeolus-shadcn/src/classes.rs` |
| F7  | Backends re-lower the plan each call (`aeolus-floem::tw` → `lower(plan)` + `style.clone()`), five times over, ~1.3–1.6 k lines each                                                                                       | `aeolus-*/src/lib.rs`          |
| F8  | Utility semantics live in code (`match name { … }`), not data → no perfect hashing, no generated docs/linter/sorter from one source of truth                                                                              | `resolve.rs` (4 022 lines)     |
| F9  | Per-edge cascade needed an `EdgeSet`/`CornerSet` set-mask patch (v0.8) because the IR stores _shorthand-shaped_ slots                                                                                                     | `plan.rs`                      |

### 1.3 What to keep (this is the moat)

- Semantics verified against real `tailwindcss` v4.1.17, including the **193-entry golden corpus**.
- `cn()` conflict-group table, `@theme`/`calc()` CSS parser, variant/`Cond` algebra, honest-failure contract
  (`LoweringReport`: never drop a class silently), fuzz targets, allocation-contract tests, snapshot JSON.
- Zero-dependency core.

**The revamp changes _when_ work happens and _what shape the data has_, not what Tailwind means.**

### 1.4 What `gpui-rsx` gets right and wrong (our opening)

- Right: static literals expand to builder calls at compile time; tables generated from GPUI sources
  (`cargo xtask gen-tables`), so API drift is detectable; configurable `gpui-path`; LSP/CLI.
- Wrong/limited (their own docs): numeric classes are **px by default** (`gap-4` = 4 px, not Tailwind's 16 px);
  `dark:` and `md:` are unsupported (warning/error); dynamic strings use a _smaller_ runtime matcher;
  variants limited to hover/focus/focus-visible/active/group-*. No v4 CSS-first theme, no `@utility`,
  no oklch-correct colour pipeline, no `cn()` conflict engine.

**Positioning:** _Aeolus semantics + gpui-rsx-style static delivery_, behind a stable IR.

---

## 2. Design research: four ways to kill the runtime

| Approach                                       | Idea                                                        | Prior art                                                         | Verdict                                                                                                                                 |
| ---------------------------------------------- | ----------------------------------------------------------- | ----------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------- |
| A. **Proc-macro codegen**                      | Class literal → straight-line target code at expansion time | gpui-rsx, StyleX/Panda/Vanilla-Extract ("zero-runtime" CSS-in-JS) | **Chosen for call sites**                                                                                                               |
| B. **`const` data tables**                     | Resolve into `static` data, interpreted by a tiny loop      | `phf`, const lookup tables                                        | **Chosen for tables** (shadcn variants, non-codegen backends) — _not_ a `const fn` parser (no float `powf/cbrt`, painful compile times) |
| C. **Whole-project scan + one generated file** | Scan sources, emit one global artifact, dedupe globally     | Tailwind v4 _Oxide_ scanner, UnoCSS on-demand                     | **LANDED 2026-11** (`aeolus_compile::scan`/`build` + the CLI's `aeolus build`: the `ClassSource` extraction seam, the global dedupe, the registry-ordered rules, the self-contained vars block)                                                                                              |
| D. Runtime + cache                             | What exists today                                           | —                                                                 | **Demote to the dynamic-string fallback only**                                                                                          |

Techniques borrowed (as inspirations — verify details before citing publicly):

- **Longhand declarations + shorthand re-synthesis** (Lightning CSS / Stylo). Store `padding-top`… separately; re-fold
  to `.p()` only at emit time. This _deletes_ `EdgeSet`/`CornerSet` (F9).
- **Style sharing / interning** (Stylo's style-sharing cache): identical declaration lists share one `&'static` blob.
- **Atomic deterministic resolution** (StyleX): conflict resolution is a _compile-time_ property, not a merge at render.
- **Rule table with static fast-path + functional-prefix trie** (UnoCSS, Tailwind v4's `Utilities` registry keyed by root).
- **Perfect hashing / FST** for the static-name registry, generated offline (no runtime dependency).
- **Instruction-count benchmarking** (`iai-callgrind`) so CI can gate performance deterministically, not by noisy wall-clock.
- **Hand-rolled byte lexer** (`logos`-style, but dependency-free) + `memchr`-style whitespace split.

### Key identity that deletes runtime colour maths

Tailwind v4 emits `bg-primary/90` as `color-mix(in oklab, var(--color-primary) 90%, transparent)`.
Mixing with `transparent` in a **premultiplied** space leaves the colour channels unchanged and scales alpha:
`result = (L, a, b, α·0.9)`. Therefore, at runtime, an opacity modifier on a theme token is **one f32 multiply**,
and when the token is known at compile time it is a **literal**. No tables, no per-alpha pyramid.
(Aeolus already pins `/N` behaviour against the real compiler in the golden corpus — keep that as the proof harness;
add a property test for the identity.)

---

## 3. Target architecture

### 3.1 Compiler pipeline (a pure function; no global state)

```
 ┌──────────┐   ┌────────────┐   ┌─────────────┐   ┌──────────────┐   ┌────────────┐
 │ syntax   │──▶│  HIR       │──▶│  MIR        │──▶│  Program     │──▶│  emit      │
 │ lex+parse│   │ candidates │   │ resolved vs │   │ optimised,   │   │ (a) Rust   │
 │ zero-copy│   │ + variants │   │ theme; long-│   │ cascade-     │   │     tokens │
 │          │   │ + utilities│   │ hand decls; │   │ merged, cond-│   │ (b) static │
 │          │   │            │   │ cn() merge  │   │ partitioned  │   │     data   │
 └──────────┘   └────────────┘   └─────────────┘   └──────────────┘   │ (c) interp │
                                                                       │ (d) CSS    │
                                                                       └────────────┘
```

Every stage is deterministic and total (keeps today's fuzz/property discipline).

### 3.2 Crate layout

| Crate                                  | Role                                                                                                                     | Notes                                                                                                                               |
| -------------------------------------- | ------------------------------------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------- |
| `aeolus-syntax`                        | Lexer, candidate parser, AST with spans                                                                                  | zero-dep, `no_std + alloc` capable, borrowed (`&str`)                                                                               |
| `aeolus-data`                          | **Utility registry as data**: one row per utility (pattern, value kind, longhand setters, conflict group, v4 source ref) | Generated by `xtask` into committed Rust (phf/trie). Also generates: the roots index, docs/REGISTRY.md, docs/CONFORMANCE.md — the three committed artifacts the `verify-tables` drift gate pins. Kills F8 |
| `aeolus-ir`                            | `Prop`, `Val`, `Decl`, `Layer`, `Program`, `Cond`, `Mode`, `Bp`                                                          | `#[repr]`, `const`-constructible, no heap in the static form                                                                        |
| `aeolus-theme`                         | `@theme`/`:root`/`.dark` CSS-first config → `ThemeSource` → (baked consts \| runtime `TokenTable`)                       | Folds `calc()` + `--spacing` at build time; oklch→sRGB gamut-mapped once                                                            |
| `aeolus-compile`                       | The pipeline above: `compile(&str, &Config) -> Result<Compiled, CompileError>` (the `Target`'s five emit arms — CSS, Rust tokens, interp, static data, plan — carry their own entry points) | Hosts the `cn()` engine at IR level                                                                                                 |
| `aeolus-codegen`                       | `Program → TokenStream`; backend-neutral `Emit` trait                                                                    | Library, used by macros **and** by third parties (gpui-rsx)                                                                         |
| `aeolus-codegen-gpui`                  | GPUI target: capability table, polyfills, version shims                                                                  | Generated from the pinned GPUI `Styled` API by `xtask` (gpui-rsx's trick)                                                           |
| `aeolus-macro`                         | The only proc-macro crate: `tw!`, `cn!`, `tw_cn!`, `variants!`, `theme!`                                                 | Thin shell over `aeolus-codegen*`                                                                                                   |
| `aeolus-rt`                            | Tiny runtime: `Env`, `TokenTable`, `DynCache`, `Program` interpreter; the observability face (`DynObserver`/`CacheStats`/`NoObserver`/`CacheSnapshot`/`EvictReason`/`session_key`) is nameable here and the `ClassCell` joins the same seam (`with_observer`) | No locks on the UI path (UI thread is single-threaded); the cell is `!Sync`, the observer handle `Send + Sync`                                                                              |
| `aeolus-gpui`                          | `TwExt`, `AeolusRoot`, `Env` plumbing, gpui-component theme bridge, re-exports                                           | The crate users depend on                                                                                                           |
| `aeolus-cli`                            | resolve / css / diff / cn / check / explain / sort / fmt / expand / build / plan / audit (headless; the benches are cargo benches beside the crates)                                                        | Reuses `aeolus-compile` as-is                                                                                                       |
| `aeolus-lsp`                            | the P5 row's LSP surface (landed 2026-11, the completion and swatch answers joined 2026-11): the headless engine answers from the same pipeline — the extraction seam's strings through the V1 compile walk, per-location typed diagnostics with UTF-16 ranges and stable codes, the hover keyed by the candidate under the cursor (the CSS arm's own declaration texts, or the typed refusal), the completion at the typing edge (the registry spellings, the theme tokens, the compile crate's audited closed variant set), the colour swatches (the per-candidate compile resolving the IR's colour slots through the config's own table), the inline hints (every lowered class rides one `inlayHint` at its end — the CSS arm's own declarations compacted, the hover's markdown as the tooltip; a refused class never hints), the quick fixes (the "did you mean" suggestion seam's editor face — every correction a spelling the pipeline lowers), the document highlights (the cursor's class, every textual occurrence), the semantic tokens (`textDocument/semanticTokens` — the lowered/refused split tokenized from the same walk), the colour presentations (`textDocument/colorPresentation` — the swatch seam's per-slot forms) and the diagnostics pull (`textDocument/diagnostic` — the push diagnostics' own report, one shot); the server half is the stdio JSON-RPC state machine (`bin/aeolus-lsp`); the session's compiles ride the bounded pool (`DynCache` — the keystroke repeat is the cheap path, the observable face on `AeolusEngine::with_observer`); the wire half's transport face landed 2026-11 (`Session::with_observer` + `run_stdio_observed`: the state machine's own per-message verdicts — answered/quiet requests, published/exit/silent notifications — stream to a `TransportObserver`, the built-in `TransportStats`/`TransportSnapshot` beside, the outgoing messages byte-identical) | The engine carries none of the wire types; the state machine is testable without a pipe                                             |
| `aeolus-{xilem,floem,iced,egui,slint}` | Re-targeted onto `Program` (codegen where the API allows, interpreter otherwise)                                         | Migrated one by one; see §7                                                                                                         |
| `aeolus-legacy`                        | (never split out — the plan decided against a frozen copy once the re-target landed)                                     | The v0.9 `StylePlan` + `Resolver` stay in-tree in `aeolus-core` as the differential oracle; `emit_plan` in `aeolus-compile` is the `Program → StylePlan` adapter the migration walked through |

### 3.3 The IR (the heart of the revamp)

```rust
// aeolus-ir — sketch, not final
#[repr(u16)] pub enum Prop {            // LONGHANDS only; stable numeric order = cascade/sort order
    PaddingTop, PaddingRight, PaddingBottom, PaddingLeft, Gap_Row, Gap_Col,
    Background, BackgroundImage, Color, FontSize, FontWeight, /* … */
}
#[repr(C)] #[derive(Copy, Clone)]
pub enum Val {                           // 12 bytes, Copy, no heap
    Px(f32), Rem(f32), Pct(f32), Num(f32),
    Rgba([u8; 4]),                       // baked colour (static theme)
    Tok { id: u16, alpha: f32 },         // runtime theme token + opacity modifier
    ModePair(u16),                       // index into a (light, dark) const pair
    Kw(u8),                              // enum keyword (flex-col, uppercase, …)
}
pub struct Decl { pub prop: Prop, pub val: Val }
pub struct Layer { pub cond: Cond, pub decls: &'static [Decl], pub mask: PropMask /* u128×k */ }
pub struct Program { pub layers: &'static [Layer] }   // layer 0 = base, rest sorted by specificity
```

Why this shape:

- **Longhands + set-masks** ⇒ `p-4 ⊕ px-8` and `hover:px-8` are _just bit operations_; no special per-edge machinery.
- **`Val: Copy`, no heap** ⇒ the static form is a `static` item; the dynamic form is one `Box<[Decl]>` interned and shared.
- **`Rem` is first-class.** GPUI has `rems()`; emitting rem-based values honours the window's rem size —
  _more correct than gpui-rsx's px-only default_. Config: `spacing = "tailwind"` (default, 1 unit = 0.25 rem), `"px"` for gpui-rsx compat.
- **Overlay/`cn()` at IR level** = `mask` algebra + copy. The existing conflict table feeds the merge; result is computed at
  compile time for static input.

### 3.4 Theme tiers (pay only for the dynamism you use)

| Tier                    | When                                               | Emitted                | Runtime cost                          |
| ----------------------- | -------------------------------------------------- | ---------------------- | ------------------------------------- |
| **T0** static theme     | `theme!("app.css")` known at compile time          | colour **literals**    | 0                                     |
| **T1** light/dark baked | same, with `.dark` block                           | `[Rgba; 2]` const pair | 1 index by `Env.dark` (branch-free)   |
| **T2** runtime theme    | user-editable themes, gpui-component `ActiveTheme` | `Tok{id, alpha}`       | 1 load from `TokenTable` + 1 multiply |

The theme file is tracked by emitting `const _: &str = include_str!("/abs/path/app.css");` inside the expansion,
so Cargo rebuilds when it changes (stable-Rust substitute for nightly's `tracked_path`).
**Same `app.css` a web Tailwind v4 project uses must work unchanged** — that is a headline feature.

**The theme stage's observability face LANDED 2026-11** (`aeolus_theme::ThemeObserver`): the no-op-default trait
(`on_resolve`/`on_resolve_rgba`, the typed `ResolveOutcome`/`RgbaOutcome` beside the verbatim `ResolveError`),
`NoThemeObserver` the every-constructor default, the `Arc` blanket and the `ThemeStats`/`ThemeSnapshot`
collector the compile crate's two faces keep, wired through `TokenTable::with_observer` — the events ride the
table itself, so a table wired before the themed compile streams the walk's per-declaration consults (the
stage the walk face's events sit above), answer-neutral by the differential pin. **The pooled-path WALK
CHANNEL landed beside it**: `DynCache::compile_observed`/`compile_theme_observed` thread the walk face
through the pool (a miss lowers observed, a hit streams nothing — the channel reports lowerings, not
answers) and `AeolusEngine::with_walk_observer` joins it to the LSP session. **The wire half's
TRANSPORT FACE landed 2026-11** (the loop's own seam beside the engine's two): `TransportObserver`
on `crates/aeolus-lsp` — `on_request(method, RequestOutcome::Answered|QuietDropped)` and
`on_notification(method, NotificationOutcome::Published|Exit|QuietDropped)`, the state machine's
OWN per-message verdicts read off the `Handled` the callers receive, `NoTransportObserver` the
every-constructor default, the `Arc` blanket and the `TransportStats`/`TransportSnapshot` collector
the pool/walk/theme faces keep, wired through `Session::with_observer` (the loop's observed twin
`run_stdio_observed`), answer-neutral by the byte-for-byte differential pin.

### 3.5 Variants map to _where_ the cost is paid

| Variant                                                 | GPUI mapping                                                                                     | Cost                                                                                                                                                |
| ------------------------------------------------------- | ------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| `hover:` `active:` `focus:` `focus-visible:`            | native `.hover/.active/.focus/.focus_visible(\|s\| …)`                                           | GPUI stores it boxed (`Option<Box<StyleRefinement>>` in `Interactivity`): 1 small alloc per stateful element — GPUI-imposed (see §6 upstream track) |
| `group-hover/name:` `group-active/name:`                | `.group(..)` + `.group_hover(..)`                                                                | same                                                                                                                                                |
| `dark:`                                                 | `Env.dark` index (the landed frame fact)                                              | branch-free index                                                                                                                                   |
| `sm: md: lg: …`                                         | `Env.viewport_rem` compare (the landed frame fact), emitted as `.when(env.viewport_rem >= bp.rem(), …)` | 1 compare                                                                                                                                           |
| `disabled:` `aria-*:` `data-*:`                         | typed state bits passed with the element (`.tw_state(State{disabled,…})`), **not** string probes | 1 mask test                                                                                                                                         |
| `@container` / `@md:`                                   | needs element bounds ⇒ custom `ContainerQuery` element wrapper (layout-phase)                    | Phase 6. The NAMED-container fold landed 2026-11 at the pipeline level: `@md/card:` carries the container's NAME on the typed `Cond` (`Container { bp, name }` — two different containers are two different conditions, the silent drop that merged them into one cascade slot is gone), the CSS arm renders the v4 form `@container card (min-width: …)`, the plan bridge fills the v0.x oracle's own `Atom::Container.name` (the oracle's parser now routes known-bp named spellings through the same split), and the interp answers the unnamed question from the Env. The NAMED-container BOUNDS SEAM landed 2026-11 too (the `ContainerProbe` trait — one question, what width does the container NAME carry? — the built-in `ContainerState` static table, the `interp_*_containers` entry points and the CLI's `expand --container name=rem`): a named layer asks the seam for ITS container, a hit fires by the same AT/strictly-below boundary rules the viewport breakpoints obey, a container the source does not know is a CSS-quiet miss, and the sourceless forms keep the `ContainerUnqueried` note — the GPUI WALK HALF landed 2026-11 too: `aeolus_gpui::TwExt::tw_containers` (and the `*_probes`/`*_struct` composed pairings) walks named layers through the same `ContainerProbe` seam the interp consults — the AT/strictly-below verdicts are the interp's own `decide` semantics shared verbatim, a sourceless walk keeps the quiet skip, and the differential pin rides `tests/containers.rs` — and the element-bounds WRAPPER (who MEASURES the widths) landed 2026-11 as the host half itself: `aeolus_gpui::TwContainerExt::tw_container(name, &MeasuredContainers)` is the stateless measuring element — it publishes the width of the box it assigns its child at the layout phase (its own prepaint bounds, in rem through `container_rem`, the `begin_frame` arithmetic's sibling; a degenerate rem size is NO measurement) into `aeolus_rt::MeasuredContainers`, the shared interior-mutable `ContainerProbe` whose clones are the wrapper's publish side and the styled descendants' read side; the publish/refresh handshake answers whether the measurement CHANGED, so a stable width never schedules a frame and a resize asks for exactly one re-render — the wrapper never styles, the walk never measures, and the frame beat is GPUI's own (the styles a frame builds read the previous frame's measurement, the animation lane's beat). |
| `has-*` `group-has-*` `peer-has-*`                      | the descendant-probe seam landed 2026-11: `DescendantKind` names the three `:has()` scopes the typed `Cond` surface folds, `DescendantProbe` asks the ONE question (does THIS scope's subtree contain a descendant matching the rendered `:has()` inner fragment `Cond::Has` carries verbatim?), the built-in `DescendantState` static-fact table answers (presence IS the answer — the `AttrState` bits' zero-allocation form), the `*_descendants` entry points carry the seam (the `*_probes`/`*_containers`/`*_struct` convention, the full composed matrix included, `Compiled`'s ergonomic half beside), a YES fires, a source-answered NO is a CSS-quiet skip, and the sourceless forms keep the dedicated `DescendantUnqueried` note. The `*:` (arbitrary-variant) half stays **diagnosed**, with reason + suggested alternative (honest failure, kept) | `tests/descendants.rs` (15 pins, RED-first)                                                                                                         |
| `supports-*` (`supports-[…]`) | the `@supports` condition seam landed 2026-11: `Cond::Supports` carries the RENDERED condition text the tailwindcss v4.1.17 source's own normalization produces (function-prefixed inners verbatim with `and`/`or`/`not` connectives spaced, colon-bearing declarations wrapped, bare features the `(feature: var(--tw))` probe; the honesty tightening refuses braces, `;`, backslashes and unbalanced parens), `SupportsProbe` asks the ONE question (the capability set is NOT frame state, the structural seam's own reading), the built-in `SupportsState` static table answers, the `*_supports` entry-point matrix carries the seam (the fifth dimension + the `Compiled` face), a YES fires, a source-answered NO is a CSS-quiet skip, the sourceless forms keep the dedicated `SupportsUnqueried` note, the CSS arm renders `@supports <condition>` from the SAME payload, and the plan bridge keeps the v0.x surface's honest note. The compound `not-*` forms stay **diagnosed** | `tests/supports.rs` (25 pins, RED-first) + `tests/cli.rs`'s `--supports` battery (4 pins, RED-first) |
| `pointer-*` `any-pointer-*` `portrait` `landscape` | the media-feature CAPABILITY TIER landed 2026-11 (the preference families' sibling — the remaining v4.1.17 `staticVariant` media spellings, verified against the upstream `variants.ts` source): the eight spellings fold onto the same typed `Cond::Media` payload (`Pref::PointerFine`…`Pref::AnyPointerNone`, `Pref::Portrait`/`Pref::Landscape`), the `Env` carries the frame facts (six pointer bits asserted independently — a hit on one never implies another — and one `portrait` bit whose complement answers `landscape:`, the `motion-safe:` precedent: the orientation feature always holds an answer the way reduced-motion does), the interp answers with no probe seam and no sourceless note (an unset bit is the quiet miss a media query that does not hold is), the CSS arm renders the v4.1.17 texts (`@media (pointer: fine)` … `@media (orientation: landscape)`), the plan bridge keeps the `MediaFeature` note's honest boundary, the GPUI/floem walks share the interp verdicts verbatim, and the CLI's `expand --pointer-fine`/`--portrait` flags assert the same facts the preference flags do | `tests/capabilities.rs` (12 pins, RED-first) + `tests/cli.rs`'s capability battery (4 pins, RED-first) |
| `ltr:` `rtl:` | the DIRECTION FAMILY landed 2026-11 (the media-feature tier's SELECTOR-side sibling — the two static variants upstream renders as selector conjunctions, not at-rules, verified against the upstream `variants.ts` source): the two spellings fold onto the typed `Cond::Dir` payload (`Direction::Ltr`/`Direction::Rtl`), the `Env` carries ONE frame fact (`direction_rtl` — `rtl:` on the bit, `ltr:` its COMPLEMENT, the `portrait:`/`landscape:` precedent: CSS's own initial `direction` value is ltr, so the query always holds an answer), the interp answers with no probe seam and no sourceless note (an unset bit is the quiet miss a selector that does not match is), the CSS arm renders the v4.1.17 selectors (`:where(:dir(rtl), [dir="rtl"], [dir="rtl"] *)` — the probe tier's head, no at-rule), the plan bridge keeps the `Direction` note's honest boundary (the v0.x surface has no text-direction slot), the GPUI walk shares the interp verdicts verbatim, the codegen's frozen tokens name the typed bit, the LSP labels the family, and the CLI's `expand --rtl` flag asserts the fact (the capability flags' sibling) | `tests/direction.rs` (10 pins, RED-first) + `tests/cli.rs`'s direction battery (3 pins, RED-first) |
| `valid:` `invalid:` `user-valid:` `user-invalid:` | the VALIDATION STATES landed 2026-11 (the forms block's constraint quartet, the selector-side tier beside `checked:` — the v4.1.17 `staticVariant` calls verified against the upstream source; the row documents the landed shape — the element-state row below was the recorded boundary note that carried it): the four spellings fold onto bare typed conditions (`Cond::Valid`/`Cond::Invalid`/`Cond::UserValid`/`Cond::UserInvalid`, the probe tier's specificity 2), the `Env` carries ONE constraint fact (`validation_invalid` — `invalid:` on the bit, `valid:` its COMPLEMENT, the `rtl:`/`ltr:` precedent: CSS's own reading for a pristine field is `:valid`, so the resting env answers the `valid:` query) and ONE interaction gate (`user_validated` — the `user-` pair additionally gates on it, the HTML spec's own requirement: BOTH spellings stay quiet until the gate is set, past it the SAME constraint fact answers), the interp answers with no probe seam and no sourceless note (an unset gate is the quiet miss a selector that does not match is), the CSS arm renders the upstream pseudo-classes in BOTH walls (`:valid` … `:user-invalid` — no at-rule head), the plan bridge LANDS the state slots (the v0.x `State` surface already carries the forms — no note rides), the GPUI/floem walks share the interp verdicts verbatim, the `has-` inner grammar answers the states beside `checked:`, the LSP labels the family, and the CLI's `expand --invalid`/`--user-valid`/`--user-invalid` flags assert the facts (one flag per FACT — `valid:` the resting complement, it takes no flag) | `tests/validation.rs` (11 pins, RED-first) + `tests/cli.rs`'s validation battery (4 pins, RED-first) |
| `target:` `indeterminate:` `autofill:` `read-only:` | the ELEMENT-STATE family landed 2026-11 (the state/forms blocks' element-state tier beside the validation quartet — the v4.1.17 `staticVariant` calls verified against the upstream source; the family is the whole ONE-FACT set the v0.x `State` surface carried at its landing — `open:`/`inert:`/`default:` were the v4.1.17 spellings the v0.x surface had no slot for AT THE LANDING — the WIDGET-STATE family closed that boundary beside (the row below), and the matches-neither five — `enabled:`/`required:`/`optional:`/`in-range:`/`out-of-range:`, where a non-form element matches NEITHER half so one `Env` bit cannot answer them honestly (evidence 27's own boundary) — the FORM-STATE family closed with the two-fact shape, the row after): the four spellings fold onto bare typed conditions (`Cond::Target`/`Cond::Indeterminate`/`Cond::Autofill`/`Cond::ReadOnly`, the probe tier's specificity 2), the `Env` carries ONE fact per question (`url_target`, `indeterminate`, `autofilled`) and `read-only:` rides the editability fact's COMPLEMENT (`read_write` — the `validation_invalid` precedent: CSS's own reading for a non-editable element IS `:read-only`, so the resting env answers the `read-only:` query; `read-write:` has no v4.1.17 registration), the interp answers with no probe seam and no sourceless note (an unset bit is the quiet miss a selector that does not match is), the CSS arm renders the upstream pseudo-classes in BOTH walls (`:target` … `:read-only` — no at-rule head), the plan bridge LANDS the state slots (the v0.x `State` surface already carries the four forms — the `checked:` precedent, no note rides), the GPUI/floem walks share the interp verdicts verbatim, the `has-` inner grammar answers the states beside `checked:` (`has-target:` → `*:target`), the LSP labels the family, and the CLI's `expand --target`/`--indeterminate`/`--autofill`/`--read-write` flags assert the facts (one flag per FACT, the `--invalid` precedent — `read-only:` the resting complement, it takes no flag) | `tests/element_states.rs` (12 pins, RED-first) + `tests/cli.rs`'s element-state battery (5 pins, RED-first) |
| `open:` `inert:` `default:` | the WIDGET-STATE family landed 2026-11 (the state/forms blocks' widget-own tier beside the element-state family — the v4.1.17 `staticVariant` calls verified against the upstream `variants.ts` source verbatim; the family CLOSES the v0.x-surface boundary the element-state landing documented, the gap closing in the OTHER direction: the core `State` surface GROWS the three slots AND the V1 `Cond` surface grows the three conditions): the three spellings fold onto bare typed conditions (`Cond::Open`/`Cond::Inert`/`Cond::Default`, the probe tier's specificity 2), every family question is AFFIRMATIVE (the `target:` shape — a non-open element matches neither `:open`'s arms, a non-form element misses `:default`, a non-inert subtree misses `:is([inert], [inert] *)`), so the `Env` carries ONE fact per question (`opened`, `inert`, `form_default`) with no complement riding, the interp answers with no probe seam and no sourceless note (an unset bit is the quiet miss a selector that does not match is), the CSS arm renders the upstream selector texts in BOTH walls (`:is([open], :popover-open, :open)`, `:is([inert], [inert] *)`, `:default` — the compound `:is(...)` heads are the upstream selectors' own shape, no at-rule head), the plan bridge LANDS the state slots (the family's own v0.x forms — no note rides), the snapshot's strict reader round-trips the names, the GPUI/floem walks share the interp verdicts verbatim, the `has-` inner grammar answers the states beside `target:` (`has-open:` → `*:is([open], :popover-open, :open)` — the v4 mechanism rendering the inner's `&` as `*`), the LSP labels the family, and the CLI's `expand --open`/`--inert`/`--default` flags assert the facts (one flag per FACT, the `--target` precedent) | `tests/widget_states.rs` (13 pins, RED-first) + `tests/cli.rs`'s widget-state battery (5 pins, RED-first) |
| `enabled:` `required:` `optional:` `in-range:` `out-of-range:` | the FORM-STATE family landed 2026-11 (the state/forms blocks' matches-neither tier beside the widget-state family — the v4.1.17 `staticVariant` calls verified against the upstream `variants.ts` source verbatim; the family CLOSES the matches-neither boundary evidence 27 documented — a non-form element matches NEITHER half of each pair, so one `Env` bit cannot answer them honestly — with the TWO-FACT shape: a MEMBERSHIP fact per universe beside the pair's shared state bit): the five spellings fold onto bare typed conditions (`Cond::Enabled`/`Cond::Required`/`Cond::Optional`/`Cond::InRange`/`Cond::OutOfRange`, the probe tier's specificity 2), the `Env` carries FOUR facts — `form_associated` (the form-membership fact the HTML spec's form-associated list names, the gate every form pair reads; `enabled:` fires on the membership beside the unset `disabled` bit), `required_field` (the `required` attribute — `required:` on the bit, `optional:` its complement WITHIN the universe, the `validation_invalid` precedent; the attribute bit alone never fires), `range_bounded` (the bounded-universe membership — the host keeps it off while no position answer exists, the pristine-empty reading) and `range_in` (the value's position — `in-range:` on the bit, `out-of-range:` its complement within the universe), the interp answers with no probe seam and no sourceless note (an unset membership is the quiet miss a selector that does not match is), the CSS arm renders the upstream pseudo-classes in BOTH walls (`:enabled` … `:out-of-range` — no at-rule head), the plan bridge LANDS the state slots (the v0.x `State` surface already carries the five forms — the `checked:` precedent, no note rides), the GPUI/floem walks share the interp verdicts verbatim, the `has-` inner grammar answers the spellings beside `target:` (`has-enabled:` → `*:enabled`), the LSP labels the family, and the CLI's `expand --form-associated`/`--required`/`--range-bounded`/`--range-in` flags assert the facts (one flag per FACT, the `--target` precedent — `optional:` answers within the membership, `out-of-range:` within the bounds, neither takes a flag) | `tests/form_states.rs` (15 pins, RED-first) + `tests/cli.rs`'s form-state battery (6 pins, RED-first) |
| `before:` `after:` `marker:` `first-letter:` `first-line:` `file:` `backdrop:` `details-content:` | the PSEUDO-ELEMENT TIER landed 2026-11 (the pseudo-elements + state blocks' render-target remainder beside the `selection:`/`placeholder:` pair — the v4.1.17 `staticVariant` calls and the `before`/`after` functional bodies verified against the upstream `variants.ts` source verbatim): the eight spellings fold onto bare typed conditions (`Cond::Before`/`Cond::After`/`Cond::Marker`/`Cond::FirstLetter`/`Cond::FirstLine`/`Cond::FileSelector`/`Cond::Backdrop`/`Cond::DetailsContent`, the probe tier's specificity 2 — a pseudo-element head is a selector conjunct on the `&`, not an at-rule), the [`PseudoTarget`] render-target enum grows the eight arms beside the pair it carried with the closed-set surface (`ALL`, `from_name`, `spelling` — the CLI's `--pseudo` face and the diagnostics key on it), the element walks EXCLUDE every pseudo layer with the [`InterpNote::PseudoElement`] note (the render-target contract — never element state) and the pseudo cascades resolve per target through the same condition-equality walk (`interp_pseudo`/`interp_pseudo_theme`), the CSS arm renders the upstream selector texts in BOTH walls (`::before` through `::details-content`; `marker:` the FOUR-selector disjunction one rule carries — `& *::marker`, `&::marker`, the `-webkit-details-marker` pair; `before:`/`after:` the functional bodies' injected `content: var(--tw-content)` line riding FIRST, the per-rule `""` fallback standing in for the at-root `@property --tw-content` registration the per-candidate emitter cannot carry — the documented divergence, the initial value upstream registers), the plan bridge LANDS `file:` (the v0.x `PseudoElement::FileSelectorButton` the surface already carries) and reports the other seven through the typed `PlanNoteKind::PseudoElement` boundary (the media-feature boundary's sibling), the metamorphic checker's notes/labels/conjunctive env grow the tier, the GPUI/floem walks keep every pseudo layer out of the element cascade (`false` / the exclusion report), the four plan-lane backends' boundary channels render the note, the LSP labels the tier, and the CLI's `expand --pseudo <target>` face answers the named render target's own cascade beside the element answer (unknown spellings are usage errors). Upstream's remainder the tier deliberately does NOT carry: `link:` (NOT a v4.1.17 variant — absent from `variants.ts`). The of-type structural trio and the `nth-of-type-*` grammar landed 2026-11 as the STRUCTURAL family's OF-TYPE TIER (the row below), and `starting:` landed 2026-11 as the `@starting-style` at-rule variant (the row after — the frame-state reading the tier's landing deferred resolved as ONE affirmative `Env` bit, the `@media` shape's own tier) | `tests/pseudo_elements.rs` (15 pins, RED-first) + `tests/cli.rs`'s pseudo battery (4 pins, RED-first) + the GPUI/floem exclusion pins (3 + 3) |
| `inverted-colors:` `noscript:` | the MEDIA-FEATURE REMAINDER landed 2026-11 (the preference/capability tier's last two v4.1.17 static variants — the media-features block's `staticVariant` calls verified against the upstream `variants.ts` source verbatim: `staticVariant('inverted-colors', ['@media (inverted-colors: inverted)'])`, `staticVariant('noscript', ['@media (scripting: none)'])`; with this row the media-feature tier is COMPLETE — every `@media`-shaped static variant v4.1.17 registers is a typed bit): the two spellings fold onto the typed [`Pref`] bits (`Cond::Media(Pref::InvertedColors)`/`Cond::Media(Pref::NoScript)`, the at-rule tier's specificity 4), the `Env` carries TWO facts (`inverted_colors` the user agent's inverted-colours answer beside the `forced_colors` bit, `scripting_none` the scripting feature's `none` value — `enable`/`initial-only` have no v4.1.17 registration, the spelling set stays closed), the interp answers with no probe seam and no sourceless note (an unset frame fact is the quiet miss a media query that does not hold is), the CSS arm renders the upstream at-rule texts in BOTH walls (the `MediaPref` table grows the two texts — no selector suffix renders), the metamorphic checker's conjunctive env carries the facts, the GPUI/floem walks share the interp verdicts verbatim, the plan bridge keeps the typed `PlanNoteKind::MediaFeature` boundary (no new note kind — the v0.x Atom set carries width bounds only), and the CLI's `expand --inverted-colors`/`--noscript` flags assert the facts (one flag per FACT, the `--portrait` precedent) | `tests/media_remainder.rs` (6 pins, RED-first) + `tests/cli.rs`'s remainder battery (2 pins, RED-first) |
| `first-of-type:` `last-of-type:` `only-of-type:` `nth-of-type-N:` `nth-last-of-type-N:` | the OF-TYPE STRUCTURAL TIER landed 2026-11 (the structural family's type-relative slice — the `staticVariant('first-of-type', ['&:first-of-type'])` trio and the `functional('nth-of-type', …)` pair verified against the upstream `variants.ts` source verbatim): the five spellings fold onto the typed [`Structural`] enum's five new arms (the child-index family's own semantics read in the type-relative dimension, the longest-prefix-first fold order keeping `nth-last-of-type-` out of the `nth-` arms), the [`StructFacts`] seam value grows `type_index`/`type_sibling_count` beside the child-index fields (what "same type" means is the HOST's own reading — the seam carries the answer, never a tag string; `StructFacts::new` keeps its child-index shape, the of-type tests quietly missing until a host supplies the type facts), the interp answers through the same `structural_fires` verdicts, the CSS arm renders `:first-of-type` through `:nth-last-of-type(N)`, the plan bridge LANDS the tier (the v0.x `ast::Structural` table grows the arms — parser, canonical, CSS, snapshot writer/reader in lockstep; the `nth-last-of-type-*` spelling keeps the oracle's last-flag divergence pinned, the `nth-last-2:` precedent), the CLI's `expand --child-of-type i/n` facts flag answers the dimension (the `--child` flag's sibling), and the bench/example/docs rows join the family's | `tests/of_type.rs` (11 pins, RED-first) + `tests/cli.rs`'s of-type battery (2 pins, RED-first) |
| `starting:` | the `@starting-style` AT-RULE VARIANT landed 2026-11 (the transition's first frame — the `staticVariant('starting', ['@starting-style'])` registration verified verbatim; the frame-state reading the pseudo-element landing deliberately deferred, resolved as the media-feature tier's at-rule sibling): the spelling folds onto the typed `Cond::Starting` arm (the at-rule tier's specificity 4 — the block WRAPS the rule, the `@media` shape, never a selector conjunct), the `Env` carries ONE affirmative frame fact (`starting_style` — the host's first-render gate beside the transition; an unset bit the QUIET miss a frame the element is not in is, no probe seam and no sourceless note), the CSS arm renders the bare `@starting-style` head in BOTH walls (no payload, the render cannot drift the fold), the plan bridge reports the typed `PlanNoteKind::StartingStyle` boundary (the v0.x plan surface has no starting-style slot, the media-feature boundary's sibling), the metamorphic checker's conjunctive env bakes the fact so the triangle agrees, the GPUI/floem walks share the interp verdicts verbatim, the four plan-lane backends carry the env literal, the LSP labels the family, and the CLI's `expand --starting` flag asserts the fact (one flag per FACT, the `--portrait` precedent) | `tests/starting.rs` (8 pins, RED-first) + `tests/cli.rs`'s starting battery (2 pins, RED-first) + the GPUI/floem walk pins (3 + 3) |

**`Env`** is a `Copy` struct (the mode, the breakpoint rem, the preference, capability, direction, validation and
element-state facts, the container rem — one field per frame-state question). Because GPUI renders on one thread,
`AeolusRoot` (a wrapper view; or `begin_frame(window, cx)`) writes it once per window render into a thread-local `Cell<Env>`;
`.tw()` reads it with a single TLS load. Zero-argument ergonomics, no hidden locks.
Multi-window is safe because windows render sequentially per thread; add a debug-assert that `begin_frame` ran.

### 3.6 The three user-facing surfaces

**(1) Static — the 95 % case.** Zero parse, zero hash, zero lock, zero alloc (barring GPUI's own boxing).

```rust
use aeolus_gpui::prelude::*;

div().tw(tw!("flex items-center gap-2 p-4 rounded-md bg-primary text-primary-foreground \
              hover:bg-primary/90 dark:shadow-none md:p-6"))
```

`tw!` expands to a frozen `StaticProgram` — a `const` block the macro's compile walk emits
(the layered longhands and keyword tables baked at expansion, applied at runtime under a
`Copy` `Env`; the V1 pipeline's emit (b) static-data arm):

```rust
{ const LAYERS: [::aeolus_rt::Layer; 3] = [/* the specificity-sorted condition groups */];
  const KEYWORDS: ::aeolus_rt::KeywordBook = ::aeolus_rt::KeywordBook::const_default();
  ::aeolus_gpui::StaticProgram::new(LAYERS, KEYWORDS) }
```

**(2) Closed-world components — the shadcn win.** `class-variance-authority`/`tailwind-variants` as a _compile-time_ table:

```rust
variants! { pub Button {
    base: "inline-flex items-center justify-center gap-2 rounded-md text-sm font-medium …",
    variant: { default: "bg-primary text-primary-foreground hover:bg-primary/90",
               destructive: "bg-destructive text-white hover:bg-destructive/90", /* … */ },
    size:    { default: "h-9 px-4 py-2", sm: "h-8 px-3", lg: "h-10 px-6" },
} }
// Button::style(Variant::Default, Size::Sm)  → &'static Program   (index into [Program; V*S]; zero strings at runtime)
```

User overrides go through `cn!` which resolves at compile time when static, or by `Program::overlay` (mask + copy) when not.
`aeolus-shadcn` collapses into a set of `variants!` declarations — F6 disappears. (The composition half **landed 2026-11**: `aeolus_compile::overlay` — the pipeline-owned form of the sketch's `Program::overlay` method, the IR staying heap-free — composes two programs rightmost-wins per longhand, conflict-free out; `cn()` delegate-folds through the same walk, and the frozen⊕runtime shape is the `program_overlay` example's story. The shadcn half landed 2026-11 as the closed-world PROGRAM TABLES (`aeolus-shadcn::program`): the verbatim v4 strings compile through the V1 pipeline once per combo and intern — `ButtonPrograms::program(variant, size)` is a table hit, the per-call `String` build (`button_classes() -> String`, F6) stays only as the string surface the five backend wrappers consume.)

**(3) Dynamic strings — the explicit slow lane.**

> **Landed 2026-11** as the runtime cell + the GPUI wire-up: `aeolus_rt::ClassCell`
> is the single-slot view-state cell ("compiled when the string changes, a
> compare per frame otherwise") — keyed on the string's EXACT CONTENT plus the
> compile identity (config + table/bag address, the `DynCache` collision-safety
> rule, never hash-only), interior-mutable through a `RefCell` with a
> `PhantomData<Rc<()>>` marker making the type `!Sync` (the UI-thread note in
> type-system form), holding the interned `StaticProgram` the compile stages
> already leak. The hit hands back the SAME interned slices (counter-pinned);
> the four `get*` entry points mirror the compile surface; per-class misses
> ride the cell's typed diagnostics; the empty input errors and leaves the
> last good program standing. `TwExt::tw_dyn`/`tw_dyn_env`/`tw_dyn_theme`/
> `tw_dyn_theme_env` apply it under the frame `Env`; a failed compile degrades
> to the last good program (never a panic per frame). The annotation
> alignment became a public seam (`aeolus_compile::keyword_alignment`) — the
> macro freeze, the codegen target and the cell all walk one implementation —
> and the differential pin holds: the cell's themed program is byte-equal to
> the macro's frozen program for the same string. The cell's OBSERVABILITY
> FACE joined 2026-11 (the pool's own seam beside it): `ClassCell::with_observer`
> wires the `DynObserver` the bounded pool carries — hit/miss/store decisions
> in order, under the composed `session_key` vocabulary the `DynStore` probe
> uses (the identity prefix, the input verbatim), the store event carrying the
> entry's diagnostic count — so ONE `CacheStats` handle spans the pool AND
> every view cell the app wires it into (the app-wide budget's shape; the cell
> stays `!Sync`, the handle is `Send + Sync`), with `NoObserver` the
> every-constructor default and the differential pins holding every contract
> byte-identical with and without one. The sketch's `Rc<Program>`
> became the landed IR's interned `&'static` form (the storage note's own
> answer); the bounded pool remains the host's `DynCache`, the cell is the
> per-view fast path beside it.

```rust
// store in view state; compiled when the string changes, a hash compare per frame otherwise
struct Card { classes: ClassCell }        // ClassCell { hash: u64, prog: Rc<Program> }  (UI-thread, !Sync, no RwLock/Arc atomics)
div().tw_dyn(self.classes.get(&self.user_classes, theme))
```

Interner is thread-local and bounded (generation rotation, as today, minus the locks). Full v4 coverage here is
_the same compiler_ as static — **no smaller matcher** (fixes gpui-rsx's capability split).

### 3.7 GPUI capability matrix (generated — the xtask slice landed 2026-11)

The capability tables (`aeolus-codegen-gpui`'s `supports`/`polyfilled`/`expressible` tiers) are HAND-MAINTAINED in
`crates/aeolus-codegen-gpui/src/lib.rs`, and the generator slice this section sketched LANDED 2026-11 (`cargo xtask
gen-gpui-caps` / `verify-gpui-caps`): the per-longhand verdicts render from the backends' OWN capability impls
(`Caps` — the publish-the-emitter's-verdicts rule) into the committed `caps_generated.rs` module the crate consumes
(`PROP_CAPABILITIES`), drift-pinned twice — `tests/caps_generated.rs` (plain `cargo test`, no driver needed) and the
focused `cargo xtask verify-gpui-caps` walk (no CI job exists, the verify-tables honesty kept). The sketch's SURFACE-DIFF
half stays the hand-maintained rows' duty (verified against the pinned GPUI surface at pin time — the walk in
`aeolus-gpui` is that proof); the per-declaration verdicts stay at the emit gates (the table is the per-longhand
ceiling, the docs page's own boundary):

| Tailwind area                                                                                                                                                                        | GPUI                                        | Plan                                                                             |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------- | -------------------------------------------------------------------------------- |
| spacing, sizing, flex, gap, rounded (per-corner), border width/colour/style, opacity, cursor, overflow, position/inset, text size/weight/colour/align/decoration/truncate/line-clamp | native                                      | direct — **the DECORATION HALF landed 2026-11 (the typography surface's own family)**: the `decoration-{color,thickness,style}` rows lower onto THREE appended `Prop` longhands (143 → 146, the append-only tail) through the value pass — the v4.1.17 grammar's own forms (the static style keywords, the static thickness keywords, the functional root whose bare value is the NON-NEGATIVE INTEGER `isPositiveInteger` form — the new `ValueKind::PositiveInt` shape, the round-trip rule `decoration-2.5`/`decoration-02` keep typed — else a colour token, the `/opacity` modifier riding; the arbitrary values split by type, length/percentage → thickness, colour → colour), the tw_merge oracle's `collision_group` growing the three arms beside the rows (three properties that COEXIST, each group self-conflicting), and the GPUI walk riding the pinned `Styled` surface's own methods — `text_decoration_color` and the `text_decoration_0..8` thickness methods on the underline leg (their own get-or-insert semantics), the strikethrough leg patched only when a line-through already composes (v4's own `text-decoration-color`/`-thickness` reading), the `solid`/`wavy` line forms through `text_decoration_solid`/`text_decoration_wavy`. The boundaries stay typed: GPUI renders a single solid line, so `double`/`dotted`/`dashed` refuse per-decl (the row-text verdict, the shadow preset gate's pattern), the `auto`/`from-font`/rem/percent thickness forms refuse on the absolute-px slot, and the v0.x plan shape (the line slot only) keeps the bridge's `SlotUnsupported` notes — the score rising 58.04% → 58.90% (50.00% native + 8.90% polyfilled over 146) |
| `shadow-*` (multi-layer v4 presets)                                                                                                                                                  | `box_shadow: Vec<BoxShadow>`                | direct; note: 1 `Vec` alloc/element (upstream track) — **LANDED 2026-11 (the tier's both halves)**: the GPUI lane composes the presets' audited band texts into GPUI's OWN band list (`aeolus_gpui::ShadowState`, the `RingState` precedent — the pinned `BoxShadow` even carries the `inset` flag the `shadow-inner` placeholder needs, its `currentColor` reading the walk's text colour), the `shadow-{color}` recolour applies v4-true (`--tw-shadow-colored`'s replace, the `/opacity` modifier riding the baked colour), and the capability tables move the pair NATIVE (the emit gates' per-decl verdict reading the registry's own `SHADOW_PRESETS` texts — the conformance score is the generated page's own drift-gated record, `docs/CONFORMANCE.md`); the PLAN half landed beside it: the plan bridge resolves the preset references through the oracle's OWN theme reader into the parsed `BoxShadow` slots (base.shadow + `shadow_rest`), the recolour patching at `finish` in the oracle's own pass order (a preset beats the recolour wherever the source order put the classes — the oracle's observable semantics, differentially pinned; the inset placeholder alone keeps the typed note) |
| `ring-*`                                                                                                                                                                             | none                                        | **polyfill** as spread-only `BoxShadow` (v4 semantics: ring = box-shadow spread) — **LANDED 2026-11**: `aeolus-gpui`'s `RingState` accumulates the four `--tw-ring-*` longhands in the walk and composes the v4 list (offset band over ring band) onto GPUI's `Vec<BoxShadow>`; `aeolus-codegen-gpui`'s `polyfilled`/`expressible` tier lets the emit gates take it |
| `bg-linear-*`                                                                                                                                                                        | `Fill` linear gradient (2 stops)            | native for 2 stops; ≥3 stops/radial/conic → diagnostic — **LANDED 2026-11**: the registry's stop rows lower onto their OWN longhands (v4's `--tw-gradient-from/via/to` + the `-position` set — the shorthand-shaped `BackgroundImage` fold let the last stop replace the direction in the cascade), the plan bridge composes the direction text + stops into the v0.x `Gradient` slot the five plan-path backends already lower, and the GPUI lane composes `linear_gradient(angle, from, to)` (`GradState`, the `RingState` precedent) with the per-declaration emit gate refusing the via stop (two stops), the radial/conic/placeholder texts and the annotation-less decl typed |
| `grid-cols-*` etc.                                                                                                                                                                   | partial `GridTemplate`                      | native subset; rest diagnosed — **the PLACEMENT half landed 2026-11**: the pinned GPUI surface carries `col_start`/`col_end`/`row_start`/`row_end` (the refinement's `GridLocation` line ranges) and `flex_basis`, so `col-start-*`/`col-end-*`/`row-start-*`/`row-end-*`/`basis-*` move refused → NATIVE (the registry's four placement rows gain the negative flag — v4's `-col-start-2` from-end form; the `col-span-*`/`row-span-*` span families ride the same longhands: ONE class fans onto the equal (start, end) pair, the walk's `GridState` accumulator composes GPUI's `Span` placement from the fingerprint — the per-decl Line mapping would read a degenerate zero-width range, never Tailwind's span). The auto-track/auto-flow slots stay the reported boundary; the conformance page regenerates through the same drift gate |
| `outline-*`, `z-*`, transforms, filters, `backdrop-blur`                                                                                                                             | no/partial                                  | diagnosed; polyfill wrappers backlog — **the FILTER + BACKDROP-FILTER HALVES landed 2026-11 (the pipeline half)**: the `blur-*`/`brightness-*`/`contrast-*`/`grayscale-*`/`hue-rotate-*`/`invert-*`/`saturate-*`/`sepia-*`/`drop-shadow-*`/`filter` families and their `backdrop-*` mirrors (the backdrop-only `backdrop-opacity-*` ninth primitive beside them) lower onto TWENTY appended `Prop` longhands (121 → 141, the mask growing to the plan §3.3 sketch's own `u128×k` two-word form) through the same value pass the motion layer walked — the row tables carrying the v4.1.17 default theme's own texts (`--blur-*`/`--drop-shadow-*` tokens, the bare-integer percent forms, the composed `cssFilterValue`/`cssBackdropFilterValue` chains quoted verbatim from the pinned upstream), the CSS arm rendering the v4 var-chain form (each primitive's var line beside the composed chain; the backdrop rules re-declaring BOTH properties, `-webkit-` beside the standard), the IR-level `cn()` merge folding the primitives per longhand, the interp answering them, the plan bridge reporting the family honestly (the v0.x oracle refuses filters outright — the `SlotUnsupported` note is the differential contract) and the metamorphic triangle pinning compile == CSS == interp (the chain lines riding the typed `FilterCompose` note). The boundaries stay typed: bare `brightness`/`backdrop-brightness` (no v4 default), the `drop-shadow-{color}` recolour arm (the shadow tier's colour machinery reaches it; the size/color two-var interplay stays out of the registry), composite arbitrary values (the codegen host's surface), and the GPUI lane's POLYFILL HALF landed 2026-11 (the row's remaining half): the seven colour-matrix longhands, the `drop-shadow-*` presets and the `filter`/`filter-none` reset arm ride the NEW `aeolus_gpui::FilterState` accumulator (the `RingState`/`ShadowState`/`GradState` precedent) — the CSS Filter Effects shorthand's fixed sRGB matrices recolour the element's own colour slots (background, text, border, the composed ring/shadow bands, the gradient stops through `GradState::background_with`), the drop-shadow bands map onto GPUI's own `Vec<BoxShadow>` (the blur radius the third token names), the capability tier moves the nine Refused → Polyfilled (the score 52.11% → 58.45%), and the spatial/backdrop halves keep the typed refusal (no slot blurs the rendered output or paints behind it — the CSS/interp arms carry the real output) |
| `transition-*`, `animate-*`                                                                                                                                                          | `with_animation` (needs element id + state) | **the MOTION LAYER landed 2026-11 (the P6 row, the pipeline half)**: `transition`/`transition-{all,colors,opacity,shadow,transform,none}` lower onto `Prop::TransitionProperty`, `duration-*`/`delay-*` onto the millisecond `Num` slots, `ease-*` onto `TransitionTimingFunction` and `animate-{spin,ping,pulse,bounce,none}` onto `Prop::Animation` — the registry rows carrying the v4 default theme's own texts (the oracle's `transition_property_list` lists, the `--ease-*`/`--animate-*` values), the plan bridge folding the quartet into the oracle's ONE `Transition` extras slot and parsing the animation shorthand quirk-faithfully (the whitespace-cut cubic-bezier fragment takes the resolver's own `InOut` fallback), the CSS arm rendering the v4 property forms, and `tests/plan_bridge.rs`'s differential battery pinning every motion class slot-for-slot against the v0.x resolver; the GPUI half landed 2026-11 as the STATEFUL MOTION LANE: `aeolus_gpui::TwAnimExt` (`tw_anim`/`tw_anim_env`, the dynamic `tw_anim_dyn`/`tw_anim_theme_dyn` entrances through the ungated `ClassCell`) takes the element id and the animator closure from the HOST — the two inputs the frozen static program cannot name — and builds the `gpui::Animation` from the frozen row text (the v4 shorthand `spin 1s linear infinite` parsed into duration, easing and iteration; the CSS `--ease-*` cubic-béziers solved by bisection in `aeolus_gpui::motion`), `animate-none` wrapping the element statically; the STATIC emit keeps its per-longhand refusal (the macro gate's honest boundary, its reason now pointing at the lane), `pulse_animator` is the one `Styled`-expressible preset built in, and the spin/ping/bounce animators stay the host's (GPUI's `Styled` carries no transform slots) |

---

## 4. Performance contract

All **targets** unless marked _measured_. Phase 0 replaces them with real numbers.

**P0 status (2026-11).** The stable half of the measurement contract LANDED: `crates/aeolus-gpui/benches/perf_contract.rs` — the static walk measured BESIDE the hand-written builder chains it replaces (the harness's setup PINS the parity: walk == hand-written, field by field, before anything is measured), the freeze-once-clone host pattern, the v0.x lane's own static/dynamic paths and the V1 dynamic lane's warm/cold shapes, over five representative strings. The measured medians (divan, release, 2026-11): the button string's walk 1.07 µs against the hand chain's 84 ns (layout 830 ns / 69 ns, typography 319 ns / 58 ns, grid 327 ns / 45 ns — the walk's delta is the data-driven form's own dispatch, the per-decl match arms beside the cond checks and the polyfill states' compose, bounded sub-microsecond and CONSTANT per string), the freeze-once clone 70 ns (the host pattern that skips the walk entirely), the ring polyfill's walk 217 ns, the V1 `ClassCell` warm hit 908 ns (hit + walk), the cold compile 9.4 µs, the v0.x lane's static path 45 ns and its warm hit 51 ns (the lane's L1 is thread-local by design, so its fresh-cache shape is the L1-hit-plus-reinsert, not a cold parse). The parity pins' own finding fixed a real walk gap on the way: the frozen program's row-text length slots (`border-2`, `rounded-md`, `text-sm`, `w-full` — the `Val::Kw`/`Val::Num` slots whose row texts carry the px/rem/% spellings) now APPLY through the walk (the `row_length` reader; the CALC texts stay the CSS arm's). The `iai-callgrind` instruction-count golden (`tw!(s)` vs hand-written, the ±2 % gate) remains the NIGHTLY-ONLY half — flagged, not run in this environment; the stable numbers above replace the table's unmeasured cells.

| Metric                                 | Today                                        | Target                                                                                  | How it's enforced                                                                            |
| -------------------------------------- | -------------------------------------------- | --------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------- |
| Static class application               | **1.07 µs walk vs 84 ns hand-written (button; 830/69 layout, 319/58 typography, 327/45 grid — divan medians, measured)** | **= hand-written builder chain** (instruction count within ±2 %; ideally identical asm) | `iai-callgrind` golden: `tw!(s)` vs hand-written — flagged nightly-only, no CI job (the P0 status above); the stable divan half measured 2026-11 (the walk is the data-driven form's own bounded dispatch cost; the clone pattern runs 70 ns for hosts that freeze once) |
| Locks on static path                   | 1 exclusive `RwLock::write` per call (F2)    | **0**                                                                                   | code review + loom-free by construction                                                      |
| Heap allocs/element (static)           | ≥1 (Arc/Vec/clone paths; unmeasured)         | **0** (+GPUI's boxed hover/shadow)                                                      | extend `alloc_contract.rs` to the `.tw()` path                                               |
| Parse/hash at runtime (static)         | yes                                          | **none**                                                                                | grep-able: no `aeolus_syntax` symbols in release binary of a static-only app (`cargo-bloat`) |
| Cold dynamic resolve (33-class button) | 4.6 µs _(measured)_ · **V1: 9.4 µs (compile + walk, divan 2026-11)** | ≤1.5 µs                                                                                 | divan (the workspace bench idiom — no criterion in the tree)                                 |
| Warm dynamic (`ClassCell` hit)         | ≈405 ns · **V1: 908 ns (hit + walk, divan 2026-11)** | ≤20 ns                                                                                  | divan (the workspace bench idiom — no criterion in the tree)                                 |
| `tw!` expansion cost                   | n/a                                          | ≤50 µs/string                                                                           | `-Zmacro-stats`, `cargo build --timings`                                                     |
| Binary growth per call site            | n/a                                          | ≤ hand-written + 5 % ; `size` profile outlines identical programs (ICF-friendly)        | `cargo-bloat` diff                                                                           |
| Compile time of a 500-`tw!` crate      | n/a                                          | ≤ +2 s vs hand-written                                                                  | CI timing job                                                                                |

Code-size knob in `aeolus.toml`: `profile = "speed" | "size"` (inline vs one outlined `fn` per distinct program).

---

## 5. Correctness & standard-setting strategy

1. **Spec + conformance first.** Publish the IR and the utility registry as a versioned spec. **The per-backend conformance SCORE landed 2026-11** (this §5.1 row's publishing half): the `aeolus_codegen::conformance` seam (`Capability` — one tiered verdict per IR longhand — and the `Conformance` report: the partition over `Prop::ALL`, the expressible/native/polyfill score arithmetic, the deterministic audit table), the GPUI impl reading the emit gates' OWN `supports`/`polyfilled` tables (the published score is the emitter's verdicts, never a parallel claim), the committed `docs/CONFORMANCE.md` page the xtask driver renders beside the registry pages (`cargo xtask gen-tables`/`verify-tables` pin its drift — the third artifact on the same seam), and `examples/conformance.rs` walking a registered backend beside a hand-rolled third-party target. **The score's PROJECT-SIZED twin LANDED 2026-11** (the §5.1 per-longhand ceiling applied to a real class set): the `aeolus_codegen::audit` seam — `ClassAudit::of` tiers the distinct longhands one compiled program actually sets (first-seen cascade order, the bitset dedupe) through the capability's own predicates, the `Capability::reason` provided method carrying the generated tables' typed boundary reasons (the GPUI impl reads the same `caps_generated.rs` rows its tier impl does — one partition, four renderings now), the `AuditReport` aggregate with the OBSERVED-set score arithmetic (the empty set answering the vacuous full 100%, never NaN) and the deterministic rendered table, the CLI's `aeolus audit` command the report face (scan → themed compile → tiered rows; `--fail-on refused|polyfill|any|none` the CI gate keyed on the tier, report-only by default; `--json` the machine form) — correctness stays `check`'s job, the two channels never blur. Grow the golden corpus from
   193 entries to **thousands** by mining Tailwind's own utility test cases and regenerating through the real compiler
   (`tools/capture_*.mjs` already show the method). Publish a per-backend **conformance score** (WPT / caniuse-style badge).
2. **Differential testing triangle:** new compiler ⟷ old `Resolver` (in-tree in `aeolus-core` as the second oracle — the planned `aeolus-legacy` frozen copy never split out once the plan bridge landed) ⟷ real Tailwind.
   Any disagreement is a bug in one of the three, and is triaged.
3. **Fuzz** the new lexer/parser/compile stages with the existing six targets ported, plus a "compile ⇒ interpret ⇒ equals emit" metamorphic target. **The metamorphic seam LANDED 2026-11**: `aeolus_compile::metamorphic_check*` walks the three pipeline answers per string — the compile, the CSS emit and the interp — and reports, per probe situation, whether the interp's effective declarations equal the CSS rules the env fires (cascade last-wins on both sides, the diagnostics channels byte-equal, the sheet the per-candidate parts), every documented divergence a typed note (`StackedVariantFold`, `PseudoTarget`, `TokenDynamic`, the sourceless-probe notes); the deterministic battery (`tests/metamorphic.rs`) pins the agreement contracts, the seventh fuzz target (`fuzz/compile_interp_emit.rs`) rides the adversarial half, and the checker is built FROM the pipeline's own parts (the same lexer walk, the same variant classifier, the interp's own `decide`, the CSS arm's own `rgba_text`) so it cannot drift from them.
4. **CSS-first compatibility**: `@theme`, `@utility` (incl. functional `--value()`), `@custom-variant`, `@source` — same file as web. (The `@source` registration landed 2026-11, the last of the four named forms: `aeolus_theme::source_directives` types the three statements — `inline("…")` literals, walked `"path"`s, pruned `not "path"`s — source-ordered, over a string-aware comment walk (a quoted `**/*.rs` glob is payload), malformed forms as the typed `ThemeError::Source`; the registrations ride `ThemeSource::from_css` (`ThemeSource::sources`) and `aeolus check`'s `--theme <css>` consults them — inline candidates join the lint, walked paths join the scan set relative to the CSS's directory, explicit files survive the prune. The custom-authoring statements' FILE form landed 2026-11 as well: `aeolus_compile::customs_from_css` walks the `@custom-variant name (selector);` statements and the `@utility` bodies the SAME theme CSS carries onto the typed `Customs` surface — the `@apply` composite bodies compiling at registration (the `CustomValue::Applied` form: different `Val`s per longhand, the body's keyword texts riding the registration beside the decls, the annotation seam's `keyword_texts_with` and `ClassCell::get_with` reading them), the functional star bodies (`--value(integer|number|percentage|color)`/`--spacing(n)`) registering through the SAME `CustomForm` grammar the API form takes, the audited selector table folding the `.dark` family and the built-in state spellings onto the flat `Cond` surface, and every malformed statement, refused selector, non-`@apply` body and outside-the-grammar value a typed `CustomCssError` — the honest-failure contract reaching registration.)
5. **Tooling parity with web Tailwind:** class sorting (official order from the registry), `explain` (class → Rust it emits),
   LSP hover/diagnostics, `aeolus check` in CI. **The LSP half LANDED 2026-11** (the P5 row's `aeolus-lsp`, the CLI row's deferred deliverable): the headless engine answers hover and diagnostics from the SAME pipeline the CLI runs — the `scan_rust_strings` extraction seam through the V1 compile walk, per-location typed diagnostics carried into the LSP's UTF-16 position model with their stable machine codes, the hover rendering the candidate's folded condition and the CSS arm's own declaration texts (a refusal renders its code + help), the position conversions owned ONCE in the engine — and the server half is the stdio JSON-RPC state machine (`Session` + `handle_message`, testable without a pipe; `bin/aeolus-lsp` the transport's thin shell). **The completion seam LANDED 2026-11** (the engine's third question: the fragment at the typing edge completed from the compiler's OWN audited sources — the registry's bare/keyword/composite spellings, the theme tokens through the same colour gate the compile walk lowers through, the compile crate's audited `KNOWN_VARIANTS` closed set; the dynamic scales stay unnamed — no invented data) **and the swatch seam** (the fourth question: the per-candidate compile resolving the IR's `Rgba`/`Tok`/`ModePair` slots through the config's own table — the keyword slots stay unnamed, a CSS keyword is not a swatch) **and the inline-hint seam** (LSP 3.17's `textDocument/inlayHint`: every lowered class rides one hint at its END — the CSS arm's own declarations compacted to the single-line label, the hover's shared markdown rendering as the tooltip, the refused candidates staying quiet — their story is the diagnostics' and the legend's; no resolve round-trip, the answer final). **The miss side's editor face LANDED 2026-11 too** (the "did you mean" surface the data crate's docs always promised): the suggestion seam (`aeolus_compile::suggest` — the `SuggestionSurface` trait over the audited tables — the registry's rows, the theme pass's names, the closed variant set, the registered customs — the bounded-distance fold, the diagnostic-keyed composition the linter renders, every correction a spelling the pipeline LOWERS) with the editor's quick fixes (`textDocument/codeAction`: the cursor's candidate through the seam, the variant prefix and the `/modifier` tail composed back onto the replacement, the edit over the finding's own range) and the click-to-find highlights (`textDocument/documentHighlight`: the cursor's candidate, every textual occurrence, document order — the shared locator the hover and the highlights walk). **The engine's remaining answers LANDED 2026-11 too** (the wire-protocol half of the same engine: the semantic tokens `textDocument/semanticTokens` — the lowered/refused split tokenized from the same walk, the colour presentations `textDocument/colorPresentation` — the swatch seam's per-slot forms, and the diagnostics pull `textDocument/diagnostic` — the push diagnostics' own report, one shot; each with its headless consumer example beside the engine's). **And the class-sort seam's two remaining faces LANDED 2026-11-16**: the editor's `textDocument/formatting` (one edit per out-of-order class string — the sort seam's own canonical join as the new text, the themeless walk deliberate (the order is a shape-level fact), an already-canonical string answering nothing — the format-on-save no-op) and the CLI's `aeolus fmt` file face (`sort`'s batch form: one file rewritten on stdout, `--check` the drift gate that prints `path:line:col "before" -> "after"` and exits 1; the misses report on stderr and never block — the lint gate is `check`).
6. **Bridges:** `aeolus-gpui-component` mapping `ActiveTheme` colours onto shadcn token names (gpui-component follows shadcn naming —
   _verify at pin time_), so `bg-primary` just works in existing apps.
7. **Same IR → real CSS** (`to_css` already exists): the day Dioxus/Leptos users can share one registry and one conformance suite,
   Aeolus is the Rust-wide Tailwind implementation, not a GPUI add-on.
8. **Governance hygiene:** semver-checks + `cargo-public-api` in CI, MSRV policy, `cargo-vet`/`cargo-deny` (deny.toml exists), docs book, migration guide, reproducible public benchmarks.

### Integrating with `gpui-rsx` (the stated goal)

- Expose `aeolus_codegen_gpui::lower_class(literal, &Config) -> Result<TokenStream, Diagnostics>` and a `resolve_style()`
  equivalent of gpui-rsx's `StyleOperation` (powers its LSP/preview).
- Offer a PR behind a cargo feature (`aeolus`) so `class="…"` routes through Aeolus; keep gpui-rsx's JSX parsing, components, IDs, listeners.
- Config bridge: `spacing = "px"` compat mode and an `rsx.toml → aeolus theme` importer; a migration table for behavioural differences
  (px→rem scale, `dark:`/`md:` now live).
- If upstream declines: ship `aeolus-gpui` standalone (works without RSX) — the macro surface above doesn't depend on it.

---

## 6. Upstream track (GPUI) — optional, but multiplies the win

The remaining per-element allocations are GPUI's: `hover_style` etc. are boxed `StyleRefinement`s, `box_shadow` is a `Vec`,
and `StyleRefinement` is not `const`-constructible. Proposals, in order of plausibility:

1. `box_shadow: Vec<BoxShadow>` → `SmallVec`/`Cow<'static, [BoxShadow]>`.
2. `Option<Box<StyleRefinement>>` → `Option<Cow<'static, StyleRefinement>>` (a `&'static` fast-path for programs emitted as statics).
3. A `Styled::apply_static(&'static StyleRefinement)` fast path (needs `const` constructors on the refinable types).

Do **not** block on these: the design works today and simply gets cheaper if they land.

---

## 7. Migration of the existing workspace

1. Freeze current `0.9` as `aeolus-legacy` (branch + crate). Provide `Program::to_style_plan()` so the five existing backends
   keep compiling while the new IR matures — **no big-bang**. (Resolved in-tree: the v0.x `StylePlan` + `Resolver` stayed in `aeolus-core` as the differential oracle and `emit_plan` is the adapter — no frozen copy crate.)
2. Re-target backends in this order (value × feasibility): **GPUI (new)** → Floem (builder-style, codegen-friendly) →
   Xilem/Masonry (property stacks) → Iced (style fns; codegen into `const` `StyleBook`s) → egui → Slint (already codegen-ish: emits `.slint`).
   The Floem, Xilem, Iced, egui and Slint lanes LANDED 2026-11: each backend's
   `lower_program`/`lower_program_with` walks the frozen `StaticProgram` under
   one probe `Env` into its native payload — Floem through its OWN walk (every
   `Prop` mapped or reported onto the pinned-rev `Style`), Xilem through the
   pipeline's shared `program_style` seam into the per-state `ResolvedStyle`
   the widget already resolves, Iced and egui through the same seam composed
   with the crate's own mapping (`IcedStyle`/`EguiStyle`), Slint through that
   same seam composed into the `SlintStyle` the element properties and
   snippets paint — each carrying its
   `REM_PX` bake, its probe-to-`Env` conversion (`env_for_state`/
   `env_for_probe`), the `AttrProbe`/`TokenSource` seams on the `*_with`
   entry points, the walk's typed report translated onto the crate's
   `LoweringReport`, differential pins against the plan path
   (`tests/program.rs`; floem's differential suite is
   `tests/static_program.rs`), a divan bench and a headless `examples/`
   demo. The egui lane's GRADIENT MESH-PAINTER seam landed 2026-11 (the
   backend ships the painter egui's solid-`Color32` `Frame` lacks: the
   lowering bakes a `GradientPaint` payload onto `EguiStyle::gradient`,
   `gradient_mesh` builds the paint over any rect — linear's banded
   strip exact at every stop, radial's concentric rings over the
   `farthest-corner` radius, conic's angular sectors from the CSS start
   angle, all clipped to the rect — and `paint_gradient` is the
   one-liner the host runs beside the frame; the "gradients need an
   app-side mesh painter" refusal retired). The §7 re-target is COMPLETE:
   every backend consumes the frozen
   program. The program lane's PSEUDO-TARGET FACE landed 2026-11 (the interp
   arm's own-cascade contract on the frozen program: `program_style_pseudo`/
   `program_style_pseudo_theme` resolve one `PseudoTarget`'s own cascade into
   the per-state `ResolvedStyle` through the SAME `PlanWriter` the element
   fold writes through — condition equality is the verdict, the element walk's
   `InterpNote::PseudoElement` note the other half of the render-target
   contract), and the host surface's OWN STYLE FACE landed beside it
   (`aeolus_rt::StaticProgram::style`/`style_with`/`style_pseudo`/
   `style_pseudo_theme` — the walk entries as methods on the frozen type, the
   free entries' delegates, the v0.x `ResolvedStyle` re-exported so the host
   names the answer) The host surface's PROBE FACE landed 2026-11 beside it
   (`StaticProgram::style_struct` and `style_probes_containers_struct_
   descendants_supports` — the structural and widest walk entries the five
   backend walks ride, as methods on the frozen type — and the narrow
   `program_style_containers`/`program_style_descendants`/
   `program_style_supports` entries with their `style_*` method twins, the
   interp arm's per-seam convention carried to the program lane, each narrow
   entry keeping the `*_with` token tail) The pseudo-target face rode the PLAN-LANE BACKENDS 2026-11
   (the render-target contract's backend half: `lower_program_pseudo`/
   `lower_program_pseudo_theme` on xilem, egui, iced, slint and floem — the
   target's own cascade lowers into each backend's native payload through the
   SAME report translation the element walk rides, floem's own walk applying
   the condition-equality verdict over its `apply_decl` mapping), and the
   floem walk's FULL PROBE SURFACE landed beside it (the last backend on the
   lane: `lower_program_probes_containers_struct_descendants_supports` —
   `structural_fires` re-exported from `aeolus-rt` so the walk shares the
   interp's structural verdicts verbatim; the probe-seam handoff's "floem
   already answers" line was drift, the walk demonstrably kept its pre-seam
   shape until this landing) The shadcn PROGRAM TABLES landed 2026-11 beside the lanes (§3.6's surface (2), §10's const-table row — the F6 finding gone): `aeolus-shadcn`'s `program` module interns the verbatim v4 strings through the V1 pipeline once per closed-world combo (`ButtonPrograms` per (variant, size), `BadgePrograms`/`AlertPrograms` per variant, `BasePrograms` per single-string component), the token table riding the handle (the built-in `ThemeSource::shadcn` bake or a host's own), the first compile's diagnostics pinning the documented arbitrary-variant boundary.
3. Delete when superseded: `resolve.rs` match walls, `StylePlan`'s `Vec/BTreeMap` shape, `PlanCache` (replaced by `DynCache`),
   per-backend `lower()` duplication (shared via `aeolus-codegen::Emit`).
4. Keep: `Warning`/`LoweringReport` → becomes `Diagnostics` with spans + reason codes; snapshot JSON becomes a _debug_ serialisation of `Program`.

---

## 8. Roadmap (effort in focused person-weeks; adjust to your pace)

| Phase                     | Deliverable                                                                                                                                                                                        | Exit criteria                                                                   | Est.    |
| ------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------- | ------- |
| **P0 – Prove it**         | Bench harness; hand-write the ideal GPUI output for 5 representative strings; compare (a) current runtime, (b) gpui-rsx static, (c) builder chain, (d) cloned `static` refinement, (e) outlined fn | Real numbers replace §4 targets; pick emit strategy; confirm GPUI pin           | 1       |
| **P1 – Data-driven core** | `aeolus-syntax`, `aeolus-data` registry + generator, `aeolus-ir`; port top ~150 utilities (LANDED 2026-11 as the three P1 git-only crates — joined since by the P2 git-only crates `aeolus-compile` and `aeolus-theme`; 213 registry rows since the P6 motion layer appended the transition quartet and the decoration family and the animation root, the filter/backdrop layers the twenty longhands, and the grid-flow family the auto-flow slot; the generator moved to P2)                                                                                                         | New compiler == old `Resolver` == golden corpus on those utilities              | 3–4     |
| **P2 – Full pipeline**    | `aeolus-compile`, `aeolus-theme` (T0–T2), IR-level `cn()`, `@utility`/`@custom-variant` — **the compile seam landed 2026-11** (`aeolus-compile`: lex → registry lookup → longhand lowering → cascade-sorted `Program`, typed spanned diagnostics, the spacing config knob), **the theme pass landed 2026-11** (`aeolus-theme`: CSS-first `ThemeSource` → tiered `TokenTable`s over the audited v0.x reader; `compile_theme` lowers colour tokens and `/opacity` modifiers, the §2 premultiplied identity), **the IR-level `cn()` merge landed 2026-11** (`aeolus_compile::cn(&[&Compiled])`: same-condition layers fold by `Cond` equality, last per longhand wins, conflict-free out — test-pinned against compiling the concatenation) and **the Target half's CSS arm landed 2026-11** (`emit_css`/`emit_css_theme`: the §3.1 emit (d) rendered in-pipeline — the row tables' keyword text, tiered token forms, baked dark-pair companions, v4 media/container queries, the typed lexer channel; the custom-authoring seam landed 2026-11 (`Customs`: registered custom utilities — fixed, functional numeric/spacing and the `--value(color)` theme-token form — plus custom variant aliases onto the typed `Cond` surface, through the `*_with` entry points; the dyn-cache seam landed 2026-11 (`DynCache`: the content-keyed, bounded, thread-safe cache over the compile surface — the leak-pool dedupe, hits hand back the same interned slices, the `DynStore` storage seam, built-in `TwoGenStore` two-generation rotation; the interp arm landed 2026-11 (`interp`/`interp_theme`: the emit (c) resolved-declaration form under a probe `Env` — the `TokenSource` token seam, the mode-aware `resolve_rgba` one-load-one-multiply primitive, typed exclusion notes); the static-data arm landed 2026-11 (`emit_static`: the emit (b) form — the program frozen into a byte-stable `static` item over `Layer::new` constructors); the ARBITRARY-VALUE UNIT GRAMMAR landed 2026-11 (the value pass's last named P2 slice: units/hints/colours/var-token references through the audited parser and bake, the sibling-row class gate on the rows' own groups, typed boundaries for the composites/property form/keyword slots); the RUST-TOKENS emit arm landed 2026-11 (`emit_rust`/`emit_rust_theme`: the emit (a) form — one embeddable `Layer::new` expression per lowered class, keyword slots resolved to their row text and token slots to their table names, the sheet Display assembling the static item, the backend method calls staying the P3 codegen's surface); the ATTRIBUTE-PROBE VARIANT FOLD landed 2026-11 (the has-/data-/aria- families onto the typed `Cond` surface: `Cond::Data`/`Cond::Aria`/`Cond::Has` with interned payload slots, `group-has-*`/`peer-has-*` on the parent tier, the v0.x oracle's `has_atom`/`parse_attr`/`attr_test` grammar as the differential source — the corpus's own rendered forms (`[data-state="open"]`, `[aria-busy="true"]`, `:has(*:focus)`, `:has(*[data-slot="card-action"])`) are the pins — and the interp's `AttributeUnqueried` note carrying the §3.5 state-bits story); the xtask GENERATOR STORY landed 2026-11 as the LAST named P2 slice (`codegen::GeneratedArtifact`: the roots-index phf the read side walks, the committed docs page, the xtask driver's gen/verify drift gate); the COMPILE-WALK OBSERVABILITY SEAM landed 2026-11 (the dyn-cache face's twin pointed at the walk: `WalkObserver` — the stage boundaries as default-no-op trait methods, `on_lexed`/`on_candidate`/`on_assembled`, the per-candidate verdict a typed `CandidateOutcome` (Lowered{decls, diagnostics} / Missed{diagnostics}), `NoWalkObserver` the constructor default, the `Arc` blanket the cache face keeps, `WalkStats`/`WalkSnapshot` the built-in collector — and `compile_observed` the entry that drives the ONE walk with the observer attached; byte-neutral by differential pin, the face re-exported through aeolus-rt and aeolus-lsp beside the pool's) | Full 193-entry corpus + property/fuzz suites green; `/N` identity property test | 3       |
| **P3 – GPUI static**      | `aeolus-codegen-gpui`, `aeolus-macro` (`tw!`, `cn!`, `variants!` — with the cva compound-variant list landed 2026-11, `theme!`), `aeolus-gpui` (`TwExt`) — **landed 2026-11**; the §3.6 surface (3)'s dynamic lane (`aeolus_rt::ClassCell`, `TwExt::tw_dyn`, the `keyword_alignment` seam) landed 2026-11; the §3.7 ring polyfill (`RingState` → the v4 `BoxShadow` bands, the `polyfilled` capability tier) landed 2026-11; the §3.6 surface (2)'s shadcn const-tables half (`aeolus-shadcn::program`, the F6 fix) landed 2026-11                                         | `iai` parity with hand-written (the instruction-count golden stays the flagged nightly half — the stable divan parity pins measure today); 0-alloc/0-lock contract tests; demo app         | 3       |
| **P4 – Env variants**     | `AeolusRoot`/`Env`, `dark:`, breakpoints, group/state bits, aria/data, the media-feature families (preferences + the capability tier landed 2026-11) — **frame Env + state layers landed 2026-11; the attribute-bits host state seam (`tw_probes`, the `AttrProbe` trait) landed 2026-11**                                         | Conformance score published for GPUI; diagnostics for unsupported variants      | 2       |
| **P5 – Ecosystem**        | gpui-rsx integration PR, LSP/CLI (**the LSP landed 2026-11** — `aeolus-lsp`: the headless engine over the V1 pipeline + the stdio JSON-RPC server; the CLI landed 2026-11 with `check`/`sort`/`fmt`/`explain`/`expand`/`build`/`plan` (and the base lane's `resolve`/`css`/`diff`/`cn`, the capability audit's `audit` joining 2026-11), gpui-component bridge, docs book, public benchmarks, `1.0-rc`                                                                                                    | Third-party app migrated; migration guide; CI perf gates                        | 2–3     |
| **P6 – Breadth**          | Other backends on `Program`, the ring polyfill and the GRADIENT half of ring/gradient landed 2026-11 (see §3.7's rows), container queries (the NAMED-container fold landed 2026-11 at the pipeline level — see §3.5's row — and the named-container BOUNDS SEAM landed 2026-11 beside it: `ContainerProbe`/`ContainerState` + the `interp_*_containers` entry points; the element-bounds WRAPPER landed 2026-11 too — `TwContainerExt::tw_container` + `aeolus_rt::MeasuredContainers`, the measuring half the docs kept assigning to the host), the PSEUDO-ELEMENT RENDER TARGETS landed 2026-11 (the interp arm's own-slice note resolved: `PseudoTarget` + `interp_pseudo`/`interp_pseudo_theme` — the `::selection`/`::placeholder` cascades resolve in their own scoped walk, the parity the CSS arm renders), motion layer (**the pipeline half landed 2026-11** — see §3.7's transition row: the `transition-*`/`duration-*`/`ease-*`/`delay-*`/`animate-*` families lower onto five appended `Prop` longhands through the same value pass, the plan bridge folds them into the oracle's `Transition`/`Animation` extras differentially, the CSS arm renders the v4 forms, and the GPUI half landed as the stateful motion lane (`TwAnimExt`, see §3.7's row)), the FILTER + BACKDROP-FILTER layers (**the pipeline half landed 2026-11** — see §3.7's backlog row: the `blur-*`/…/`drop-shadow-*`/`filter` families and their `backdrop-*` mirrors lower onto twenty appended `Prop` longhands (the mask growing to the sketch's own two-word `u128×k` form), the CSS arm renders the v4 var-chain forms, the metamorphic triangle pins the three answers, and the GPUI lane's POLYFILL HALF landed 2026-11 (see §3.7's row — the `FilterState` accumulator, the matrices over the element's own colour slots, the drop-shadow bands, the score's move; the spatial/backdrop halves stay the typed refusal)), optional `aeolus build` (global dedupe + CSS — **the build half landed 2026-11**: `aeolus_compile::scan`/`build`, the CLI `build` command, the approach C row); the backend re-target half is COMPLETE — **the Floem/Xilem/Iced/egui/Slint static-program lanes landed 2026-11** (see §7's landed note)                                                         | Per-feature                                                                     | ongoing |

Critical path to _"replace gpui-rsx's Tailwind half"_: **P0 → P1 → P2 → P3 → P5** (≈12–14 weeks solo-ish; P4 can overlap P3 late).

---

## 9. Risks & open questions

| Risk                                                                                               | Mitigation                                                                                                                  |
| -------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------- |
| **GPUI API churn** (official `gpui`, Zed git, gpui-kit/component forks)                            | Generate caps from the pinned source; `gpui-path` + feature-gated shims; CI matrix on 2 revs                                |
| **Code bloat** from inlined expansions                                                             | `profile = "size"`, ICF/lld, `cargo-bloat` budget in CI; measure in P0 before committing                                    |
| **Proc-macro error spans inside string literals** (sub-literal spans are nightly-only)             | Per-class diagnostics with the class text in the message + gpui-rsx's `#[deprecated]` warning trick; LSP gives exact ranges |
| **Thread-local `Env` correctness** (multi-window, tests, non-UI threads)                           | `begin_frame` debug-assert; explicit `.tw_in(env, …)` escape hatch                                                          |
| **Float parity host↔target**                                                                       | Emit `f32` via exact bit-patterns/shortest-round-trip literals; test with cross-compile                                     |
| **Colour gamut**: oklch → sRGB (GPUI `Hsla`) clips P3 colours                                      | Gamut-map once (CSS Color 4 chroma reduction); document; revisit if GPUI gains wide-gamut — **LANDED 2026-11**: `aeolus_core::Color::gamut_map` is the §13 binary-search chroma reduction (in-gamut colours return unchanged, bit-identical to the clamp oracle `to_srgba` — every existing pin holds; hue and lightness hold through the backend bytes), and the theme bake + the arbitrary-value colour pass run it so every `Val::Rgba` the pipeline freezes is gamut-legal (the REAL default palette carries out-of-gamut rows — shadcn's `--destructive` overshoots linear-red 1.27, the baked bytes now sit ON the boundary hue-true) |
| **Static/dynamic divergence**                                                                      | One compiler for both; metamorphic fuzz target (compile ⇒ interpret == emit)                                                |
| **Scope creep** (5 backends + CSS)                                                                 | Critical path is GPUI only; everything else behind the `Program` boundary                                                   |
| **Maintainer bandwidth**                                                                           | Registry-as-data means contributors add a _row_, not a 200-line match arm                                                   |
| Open: does `variants!` need slot-based multi-part components (`cva` "compound variants", `slots`)? | The compound-variants half DECIDED and landed 2026-11: a `compound:` LIST applies extra classes to every combo matching ALL of an entry's constraints, folded into the cartesian match arms at expansion time (zero runtime cost beyond the dispatch); the SLOTS half landed 2026-11 as well: a `slots:` map declares one base per part and generates a `{Name}Slot` enum plus a `style_slot(slot, …)` dispatch — a group value or compound `class:` that is a STRING applies to every slot, a brace applies to the slots it names, each (slot, combo) arm compiles at expansion time, so the runtime cost stays the match dispatch the table always had. |
| Open: static `Rem` vs window rem-size in non-GPUI backends                                         | Per-backend `rem_px` constant, default 16                                                                                   |

---

## 10. "Done" definition for 1.0

- `tw!("…")` on GPUI compiles to code indistinguishable in cost from hand-written builders, enforced by CI.
- The same `app.css` that styles a Tailwind v4 web app themes a GPUI app, light/dark and breakpoints included.
- ≥ 95 % conformance on the expanded real-Tailwind corpus for utilities GPUI can express; every other class yields a precise, actionable diagnostic.
- shadcn components are `const` tables. (**Landed 2026-11** as the closed-world program tables — `aeolus-shadcn::program`.)
- gpui-rsx can call Aeolus for its `class=` handling (or users use `aeolus-gpui` alone).
- Public, reproducible benchmarks and a spec other frameworks can implement.
