# Aeolus 1.0 — Zero-Runtime Architecture Plan

> **Goal.** Turn Aeolus from "a Tailwind v4 interpreter that runs inside your UI loop" into
> "a Tailwind v4 _compiler_ whose output is the code you would have written by hand".
> First consumer: **`aeolus-gpui`**, replacing the Tailwind half of `gpui-rsx` with real v4 semantics.
> Breaking the API is allowed and expected (→ `1.0`).

**P1 status (2026-11).** Phase P1 — the data-driven core — has landed
as the `aeolus-syntax` (zero-copy lexer, differentially pinned to the
v0.x parser over the golden corpus), `aeolus-data` (registry-as-data,
178 rows, group names cross-checked against the tw_merge oracle, plus
the typed `lookup_detailed`/`LookupError` read seam) and `aeolus-ir`
(longhand `Prop`/`Val`/`Decl`/`Layer`/`Program` — 110 longhands — +
`PropMask` bitset algebra) workspace crates, git-only while the
pipeline assembles. The §8 row's "registry + generator" generator
story (xtask-emitted phf/trie) stayed in P2: the committed tables ARE
the P1 source of truth, as §3.2 always said. P2 (`aeolus-compile`),
P3 (codegen/GPUI) and `xtask` remain planned; the sketches below are
their contracts, not their implementations.

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
| C. **Whole-project scan + one generated file** | Scan sources, emit one global artifact, dedupe globally     | Tailwind v4 _Oxide_ scanner, UnoCSS on-demand                     | **Optional later** (`cargo aeolus build`) for global dedupe + CSS emission                                                              |
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
| `aeolus-cli` / `aeolus-lsp`            | check / sort / explain / expand / bench; LSP                                                                             | Reuses `aeolus-compile` as-is                                                                                                       |
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
`aeolus-shadcn` collapses into a set of `variants!` declarations — F6 disappears.

**(3) Dynamic strings — the explicit slow lane.**

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
4. **CSS-first compatibility**: `@theme`, `@utility` (incl. functional `--value()`), `@custom-variant`, `@source` — same file as web.
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
| **P1 – Data-driven core** | `aeolus-syntax`, `aeolus-data` registry + generator, `aeolus-ir`; port top ~150 utilities (LANDED 2026-11 as the three git-only crates — 178 registry rows; the generator moved to P2)                                                                                                         | New compiler == old `Resolver` == golden corpus on those utilities              | 3–4     |
| **P2 – Full pipeline**    | `aeolus-compile`, `aeolus-theme` (T0–T2), IR-level `cn()`, `@utility`/`@custom-variant`                                                                                                            | Full 193-entry corpus + property/fuzz suites green; `/N` identity property test | 3       |
| **P3 – GPUI static**      | `aeolus-codegen-gpui`, `aeolus-macros` (`tw!`, `cn!`, `variants!`, `theme!`), `aeolus-gpui` (`TwExt`)                                                                                              | `iai` parity with hand-written; 0-alloc/0-lock contract tests; demo app         | 3       |
| **P4 – Env variants**     | `AeolusRoot`/`Env`, `dark:`, breakpoints, group/state bits, aria/data                                                                                                                              | Conformance score published for GPUI; diagnostics for unsupported variants      | 2       |
| **P5 – Ecosystem**        | gpui-rsx integration PR, LSP/CLI, gpui-component bridge, docs book, public benchmarks, `1.0-rc`                                                                                                    | Third-party app migrated; migration guide; CI perf gates                        | 2–3     |
| **P6 – Breadth**          | Other backends on `Program`, ring/gradient polyfills, container queries, motion layer, optional `cargo aeolus build` (global dedupe + CSS)                                                         | Per-feature                                                                     | ongoing |

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
