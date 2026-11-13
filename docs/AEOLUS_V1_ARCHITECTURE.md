# Aeolus 1.0 — Zero-Runtime Architecture Plan

> **Goal.** Turn Aeolus from "a Tailwind v4 interpreter that runs inside your UI loop" into
> "a Tailwind v4 _compiler_ whose output is the code you would have written by hand".
> First consumer: **`aeolus-gpui`**, replacing the Tailwind half of `gpui-rsx` with real v4 semantics.
> Breaking the API is allowed and expected (→ `1.0`).

**P1 status (2026-11).** Phase P1 — the data-driven core — has landed
as the `aeolus-syntax` (zero-copy lexer, differentially pinned to the
v0.x parser over the golden corpus), `aeolus-data` (registry-as-data,
184 rows, group names cross-checked against the tw_merge oracle, plus
the typed `lookup_detailed`/`LookupError` read seam) and `aeolus-ir`
(longhand `Prop`/`Val`/`Decl`/`Layer`/`Program` — 110 longhands — +
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
bounded map without touching the pipeline. The seam surfaced
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
`text-[#0f0f0f]` the colour row's). The honest boundaries stay
typed, all test-pinned: multi-token composites (`calc()`, underscore
joins, url/image values), the arbitrary-PROPERTY form, keyword values
on the row-relative `Val::Kw` slots, and the wider v4 hint table
(url/image/angle/time/ratio/position and friends) — the codegen
host's surface. P3 (codegen/GPUI) landed 2026-11: `aeolus-codegen`
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
`decide` semantics at the host); container queries remain the
documented backend boundary.

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
| `aeolus-data`                          | **Utility registry as data**: one row per utility (pattern, value kind, longhand setters, conflict group, v4 source ref) | Generated by `xtask` into committed Rust (phf/trie). Also generates: docs, linter tables, class-sort order, test fixtures. Kills F8 |
| `aeolus-ir`                            | `Prop`, `Val`, `Decl`, `Layer`, `Program`, `Cond`, `Mode`, `Bp`                                                          | `#[repr]`, `const`-constructible, no heap in the static form                                                                        |
| `aeolus-theme`                         | `@theme`/`:root`/`.dark` CSS-first config → `ThemeSource` → (baked consts \| runtime `TokenTable`)                       | Folds `calc()` + `--spacing` at build time; oklch→sRGB gamut-mapped once                                                            |
| `aeolus-compile`                       | The pipeline above: `compile(&str, &ThemeSource, &Target) -> Result<Program, Diagnostics>`                               | Hosts the `cn()` engine at IR level                                                                                                 |
| `aeolus-codegen`                       | `Program → TokenStream`; backend-neutral `Emit` trait                                                                    | Library, used by macros **and** by third parties (gpui-rsx)                                                                         |
| `aeolus-codegen-gpui`                  | GPUI target: capability table, polyfills, version shims                                                                  | Generated from the pinned GPUI `Styled` API by `xtask` (gpui-rsx's trick)                                                           |
| `aeolus-macros`                        | The only proc-macro crate: `tw!`, `cn!`, `variants!`, `theme!`                                                           | Thin shell over `aeolus-codegen*`                                                                                                   |
| `aeolus-rt`                            | Tiny runtime: `Env`, `TokenTable`, `DynCache`, `Program` interpreter                                                     | No locks on the UI path (UI thread is single-threaded)                                                                              |
| `aeolus-gpui`                          | `TwExt`, `AeolusRoot`, `Env` plumbing, gpui-component theme bridge, re-exports                                           | The crate users depend on                                                                                                           |
| `aeolus-cli`                            | check / sort / explain / expand (headless; the LSP is P5's row, the benches are cargo benches beside the crates)          | Reuses `aeolus-compile` as-is                                                                                                       |
| `aeolus-{xilem,floem,iced,egui,slint}` | Re-targeted onto `Program` (codegen where the API allows, interpreter otherwise)                                         | Migrated one by one; see §7                                                                                                         |
| `aeolus-legacy`                        | Frozen 0.9 `StylePlan` + `Resolver`, plus `Program → StylePlan` adapter                                                  | Lets old backends keep working during migration                                                                                     |

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

### 3.5 Variants map to _where_ the cost is paid

| Variant                                                 | GPUI mapping                                                                                     | Cost                                                                                                                                                |
| ------------------------------------------------------- | ------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| `hover:` `active:` `focus:` `focus-visible:` `in-focus` | native `.hover/.active/.focus/.focus_visible/.in_focus(\|s\| …)`                                 | GPUI stores it boxed (`Option<Box<StyleRefinement>>` in `Interactivity`): 1 small alloc per stateful element — GPUI-imposed (see §6 upstream track) |
| `group-hover/name:` `group-active/name:`                | `.group(..)` + `.group_hover(..)`                                                                | same                                                                                                                                                |
| `dark:`                                                 | `Mode` select from `Env`                                                                         | branch-free index                                                                                                                                   |
| `sm: md: lg: …`                                         | `Bp` bit from `Env` (viewport width), emitted as `.when(env.bp >= MD, …)`                        | 1 compare                                                                                                                                           |
| `disabled:` `aria-*:` `data-*:`                         | typed state bits passed with the element (`.tw_state(State{disabled,…})`), **not** string probes | 1 mask test                                                                                                                                         |
| `@container` / `@md:`                                   | needs element bounds ⇒ custom `ContainerQuery` element wrapper (layout-phase)                    | Phase 6                                                                                                                                             |
| `has-*` `peer-*` `*:`                                   | **diagnosed**, with reason + suggested alternative (honest failure, kept)                        | —                                                                                                                                                   |

**`Env`** is a `Copy` 4-byte struct (`dark`, `bp`, `reduced_motion`, `contrast`). Because GPUI renders on one thread,
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

`tw!` expands to a hash-named ZST implementing `StyleProgram` (monomorphised, `#[inline(always)]`):

```rust
{ struct __Tw_9f3a; impl ::aeolus_gpui::StyleProgram for __Tw_9f3a {
    #[inline(always)] fn apply<E: Styled>(mut e: E, env: Env) -> E {
        e = e.flex().items_center().gap(rems(0.5)).p(rems(1.0)).rounded(rems(0.375))
             .bg(THEME_PRIMARY[env.dark as usize]).text_color(THEME_PRIMARY_FG[env.dark as usize]);
        e = e.hover(|s| s.bg(PRIMARY_90[env.dark as usize]));
        if env.bp >= Bp::MD { e = e.p(rems(1.5)); }
        e } }
  __Tw_9f3a }
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
`aeolus-shadcn` collapses into a set of `variants!` declarations — F6 disappears. (The composition half **landed 2026-11**: `aeolus_compile::overlay` — the pipeline-owned form of the sketch's `Program::overlay` method, the IR staying heap-free — composes two programs rightmost-wins per longhand, conflict-free out; `cn()` delegate-folds through the same walk, and the frozen⊕runtime shape is the `program_overlay` example's story.)

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
> the macro's frozen program for the same string. The sketch's `Rc<Program>`
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

### 3.7 GPUI capability matrix (to be **generated**, not hand-written)

`xtask gen-gpui-caps` diffs the pinned GPUI's `Styled`/`StyleRefinement` surface against Aeolus' `Prop` set and emits
`aeolus-codegen-gpui/src/caps_generated.rs` + a docs table. CI fails on drift. Expected shape (to verify at pin time):

| Tailwind area                                                                                                                                                                        | GPUI                                        | Plan                                                                             |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------- | -------------------------------------------------------------------------------- |
| spacing, sizing, flex, gap, rounded (per-corner), border width/colour/style, opacity, cursor, overflow, position/inset, text size/weight/colour/align/decoration/truncate/line-clamp | native                                      | direct                                                                           |
| `shadow-*` (multi-layer v4 presets)                                                                                                                                                  | `box_shadow: Vec<BoxShadow>`                | direct; note: 1 `Vec` alloc/element (upstream track)                             |
| `ring-*`                                                                                                                                                                             | none                                        | **polyfill** as spread-only `BoxShadow` (v4 semantics: ring = box-shadow spread) |
| `bg-linear-*`                                                                                                                                                                        | `Fill` linear gradient (2 stops)            | native for 2 stops; ≥3 stops/`radial`/`conic` → diagnostic                       |
| `grid-cols-*` etc.                                                                                                                                                                   | partial `GridTemplate`                      | native subset; rest diagnosed                                                    |
| `outline-*`, `z-*`, transforms, filters, `backdrop-blur`                                                                                                                             | no/partial                                  | diagnosed; polyfill wrappers backlog                                             |
| `transition-*`, `animate-*`                                                                                                                                                          | `with_animation` (needs element id + state) | Phase 6 "motion layer"                                                           |

---

## 4. Performance contract

All **targets** unless marked _measured_. Phase 0 replaces them with real numbers.

| Metric                                 | Today                                        | Target                                                                                  | How it's enforced                                                                            |
| -------------------------------------- | -------------------------------------------- | --------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------- |
| Static class application               | ≈405 ns warm _(measured, README)_ + lowering | **= hand-written builder chain** (instruction count within ±2 %; ideally identical asm) | `iai-callgrind` golden: `tw!(s)` vs hand-written; CI gate                                    |
| Locks on static path                   | 1 exclusive `RwLock::write` per call (F2)    | **0**                                                                                   | code review + loom-free by construction                                                      |
| Heap allocs/element (static)           | ≥1 (Arc/Vec/clone paths; unmeasured)         | **0** (+GPUI's boxed hover/shadow)                                                      | extend `alloc_contract.rs` to the `.tw()` path                                               |
| Parse/hash at runtime (static)         | yes                                          | **none**                                                                                | grep-able: no `aeolus_syntax` symbols in release binary of a static-only app (`cargo-bloat`) |
| Cold dynamic resolve (33-class button) | 4.6 µs _(measured)_                          | ≤1.5 µs                                                                                 | criterion                                                                                    |
| Warm dynamic (`ClassCell` hit)         | ≈405 ns                                      | ≤20 ns                                                                                  | criterion                                                                                    |
| `tw!` expansion cost                   | n/a                                          | ≤50 µs/string                                                                           | `-Zmacro-stats`, `cargo build --timings`                                                     |
| Binary growth per call site            | n/a                                          | ≤ hand-written + 5 % ; `size` profile outlines identical programs (ICF-friendly)        | `cargo-bloat` diff                                                                           |
| Compile time of a 500-`tw!` crate      | n/a                                          | ≤ +2 s vs hand-written                                                                  | CI timing job                                                                                |

Code-size knob in `aeolus.toml`: `profile = "speed" | "size"` (inline vs one outlined `fn` per distinct program).

---

## 5. Correctness & standard-setting strategy

1. **Spec + conformance first.** Publish the IR and the utility registry as a versioned spec. Grow the golden corpus from
   193 entries to **thousands** by mining Tailwind's own utility test cases and regenerating through the real compiler
   (`tools/capture_*.mjs` already show the method). Publish a per-backend **conformance score** (WPT / caniuse-style badge).
2. **Differential testing triangle:** new compiler ⟷ old `Resolver` (kept in `aeolus-legacy` as a second oracle) ⟷ real Tailwind.
   Any disagreement is a bug in one of the three, and is triaged.
3. **Fuzz** the new lexer/parser/compile stages with the existing six targets ported, plus a "compile ⇒ interpret ⇒ equals emit" metamorphic target.
4. **CSS-first compatibility**: `@theme`, `@utility` (incl. functional `--value()`), `@custom-variant`, `@source` — same file as web. (The `@source` registration landed 2026-11, the last of the four named forms: `aeolus_theme::source_directives` types the three statements — `inline("…")` literals, walked `"path"`s, pruned `not "path"`s — source-ordered, over a string-aware comment walk (a quoted `**/*.rs` glob is payload), malformed forms as the typed `ThemeError::Source`; the registrations ride `ThemeSource::from_css` (`ThemeSource::sources`) and `aeolus check`'s `--theme <css>` consults them — inline candidates join the lint, walked paths join the scan set relative to the CSS's directory, explicit files survive the prune.)
5. **Tooling parity with web Tailwind:** class sorting (official order from the registry), `explain` (class → Rust it emits),
   LSP hover/diagnostics, `cargo aeolus check` in CI.
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
   keep compiling while the new IR matures — **no big-bang**.
2. Re-target backends in this order (value × feasibility): **GPUI (new)** → Floem (builder-style, codegen-friendly) →
   Xilem/Masonry (property stacks) → Iced (style fns; codegen into `const` `StyleBook`s) → egui → Slint (already codegen-ish: emits `.slint`).
3. Delete when superseded: `resolve.rs` match walls, `StylePlan`'s `Vec/BTreeMap` shape, `PlanCache` (replaced by `DynCache`),
   per-backend `lower()` duplication (shared via `aeolus-codegen::Emit`).
4. Keep: `Warning`/`LoweringReport` → becomes `Diagnostics` with spans + reason codes; snapshot JSON becomes a _debug_ serialisation of `Program`.

---

## 8. Roadmap (effort in focused person-weeks; adjust to your pace)

| Phase                     | Deliverable                                                                                                                                                                                        | Exit criteria                                                                   | Est.    |
| ------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------- | ------- |
| **P0 – Prove it**         | Bench harness; hand-write the ideal GPUI output for 5 representative strings; compare (a) current runtime, (b) gpui-rsx static, (c) builder chain, (d) cloned `static` refinement, (e) outlined fn | Real numbers replace §4 targets; pick emit strategy; confirm GPUI pin           | 1       |
| **P1 – Data-driven core** | `aeolus-syntax`, `aeolus-data` registry + generator, `aeolus-ir`; port top ~150 utilities (LANDED 2026-11 as the three P1 git-only crates — joined since by the P2 git-only crates `aeolus-compile` and `aeolus-theme`; 184 registry rows; the generator moved to P2)                                                                                                         | New compiler == old `Resolver` == golden corpus on those utilities              | 3–4     |
| **P2 – Full pipeline**    | `aeolus-compile`, `aeolus-theme` (T0–T2), IR-level `cn()`, `@utility`/`@custom-variant` — **the compile seam landed 2026-11** (`aeolus-compile`: lex → registry lookup → longhand lowering → cascade-sorted `Program`, typed spanned diagnostics, the spacing config knob), **the theme pass landed 2026-11** (`aeolus-theme`: CSS-first `ThemeSource` → tiered `TokenTable`s over the audited v0.x reader; `compile_theme` lowers colour tokens and `/opacity` modifiers, the §2 premultiplied identity), **the IR-level `cn()` merge landed 2026-11** (`aeolus_compile::cn(&[&Compiled])`: same-condition layers fold by `Cond` equality, last per longhand wins, conflict-free out — test-pinned against compiling the concatenation) and **the Target half's CSS arm landed 2026-11** (`emit_css`/`emit_css_theme`: the §3.1 emit (d) rendered in-pipeline — the row tables' keyword text, tiered token forms, baked dark-pair companions, v4 media/container queries, the typed lexer channel; the custom-authoring seam landed 2026-11 (`Customs`: registered custom utilities — fixed, functional numeric/spacing and the `--value(color)` theme-token form — plus custom variant aliases onto the typed `Cond` surface, through the `*_with` entry points; the dyn-cache seam landed 2026-11 (`DynCache`: the content-keyed, bounded, thread-safe cache over the compile surface — the leak-pool dedupe, hits hand back the same interned slices, the `DynStore` storage seam, built-in `TwoGenStore` two-generation rotation; the interp arm landed 2026-11 (`interp`/`interp_theme`: the emit (c) resolved-declaration form under a probe `Env` — the `TokenSource` token seam, the mode-aware `resolve_rgba` one-load-one-multiply primitive, typed exclusion notes); the static-data arm landed 2026-11 (`emit_static`: the emit (b) form — the program frozen into a byte-stable `static` item over `Layer::new` constructors); the ARBITRARY-VALUE UNIT GRAMMAR landed 2026-11 (the value pass's last named P2 slice: units/hints/colours/var-token references through the audited parser and bake, the sibling-row class gate on the rows' own groups, typed boundaries for the composites/property form/keyword slots); the RUST-TOKENS emit arm landed 2026-11 (`emit_rust`/`emit_rust_theme`: the emit (a) form — one embeddable `Layer::new` expression per lowered class, keyword slots resolved to their row text and token slots to their table names, the sheet Display assembling the static item, the backend method calls staying the P3 codegen's surface); the ATTRIBUTE-PROBE VARIANT FOLD landed 2026-11 (the has-/data-/aria- families onto the typed `Cond` surface: `Cond::Data`/`Cond::Aria`/`Cond::Has` with interned payload slots, `group-has-*`/`peer-has-*` on the parent tier, the v0.x oracle's `has_atom`/`parse_attr`/`attr_test` grammar as the differential source — the corpus's own rendered forms (`[data-state="open"]`, `[aria-busy="true"]`, `:has(*:focus)`, `:has(*[data-slot="card-action"])`) are the pins — and the interp's `AttributeUnqueried` note carrying the §3.5 state-bits story); the xtask GENERATOR STORY landed 2026-11 as the LAST named P2 slice (`codegen::GeneratedArtifact`: the roots-index phf the read side walks, the committed docs page, the xtask driver's gen/verify drift gate) | Full 193-entry corpus + property/fuzz suites green; `/N` identity property test | 3       |
| **P3 – GPUI static**      | `aeolus-codegen-gpui`, `aeolus-macros` (`tw!`, `cn!`, `variants!`, `theme!`), `aeolus-gpui` (`TwExt`) — **landed 2026-11**; the §3.6 surface (3)'s dynamic lane (`aeolus_rt::ClassCell`, `TwExt::tw_dyn`, the `keyword_alignment` seam) landed 2026-11                                                                         | `iai` parity with hand-written; 0-alloc/0-lock contract tests; demo app         | 3       |
| **P4 – Env variants**     | `AeolusRoot`/`Env`, `dark:`, breakpoints, group/state bits, aria/data — **frame Env + state layers landed 2026-11; the attribute-bits host state seam (`tw_probes`, the `AttrProbe` trait) landed 2026-11**                                         | Conformance score published for GPUI; diagnostics for unsupported variants      | 2       |
| **P5 – Ecosystem**        | gpui-rsx integration PR, LSP/CLI, gpui-component bridge, docs book, public benchmarks, `1.0-rc`                                                                                                    | Third-party app migrated; migration guide; CI perf gates                        | 2–3     |
| **P6 – Breadth**          | Other backends on `Program`, ring/gradient polyfills, container queries, motion layer, optional `cargo aeolus build` (global dedupe + CSS — **the build half landed 2026-11**: `aeolus_compile::scan`/`build`, the CLI `build` command, the approach C row)                                                         | Per-feature                                                                     | ongoing |

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
| **Colour gamut**: oklch → sRGB (GPUI `Hsla`) clips P3 colours                                      | Gamut-map once (CSS Color 4 chroma reduction); document; revisit if GPUI gains wide-gamut                                   |
| **Static/dynamic divergence**                                                                      | One compiler for both; metamorphic fuzz target (compile ⇒ interpret == emit)                                                |
| **Scope creep** (5 backends + CSS)                                                                 | Critical path is GPUI only; everything else behind the `Program` boundary                                                   |
| **Maintainer bandwidth**                                                                           | Registry-as-data means contributors add a _row_, not a 200-line match arm                                                   |
| Open: does `variants!` need slot-based multi-part components (`cva` "compound variants", `slots`)? | Decide in P3 with the shadcn Card/Alert tables                                                                              |
| Open: static `Rem` vs window rem-size in non-GPUI backends                                         | Per-backend `rem_px` constant, default 16                                                                                   |

---

## 10. "Done" definition for 1.0

- `tw!("…")` on GPUI compiles to code indistinguishable in cost from hand-written builders, enforced by CI.
- The same `app.css` that styles a Tailwind v4 web app themes a GPUI app, light/dark and breakpoints included.
- ≥ 95 % conformance on the expanded real-Tailwind corpus for utilities GPUI can express; every other class yields a precise, actionable diagnostic.
- shadcn components are `const` tables.
- gpui-rsx can call Aeolus for its `class=` handling (or users use `aeolus-gpui` alone).
- Public, reproducible benchmarks and a spec other frameworks can implement.
