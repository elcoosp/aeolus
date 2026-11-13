<div align="center">
  <img src="https://cdn.jsdelivr.net/gh/elcoosp/aeolus@main/docs/logo.png" alt="Aeolus Logo" width="300"/>
  <p>
    <strong>Tailwind CSS v4, compiled to native UI code</strong><br/>
    Xilem · Floem · Iced · egui · Slint · <strong>GPUI</strong>
  </p>
  <p>
    Class strings in at <em>compile time</em>, frozen typed programs out — no runtime parser, no locks,
    no string matching on the hot path. Semantics verified against <code>tailwindcss</code> v4.1.17,
    the same <code>app.css</code> your web app uses, and a typed diagnostic for every class a backend
    cannot express. Nothing is ever dropped silently.
  </p>
  <p>
    <a href="https://github.com/elcoosp/aeolus/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/elcoosp/aeolus/ci.yml?style=flat-square&label=CI" alt="CI"/></a>
    <a href="CHANGELOG.md"><img src="https://img.shields.io/badge/version-0.19.0-blue?style=flat-square" alt="Version 0.19.0"/></a>
    <a href="LICENSE-MIT"><img src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue?style=flat-square" alt="License: MIT OR Apache-2.0"/></a>
    <a href=".github/workflows/ci.yml"><img src="https://img.shields.io/badge/MSRV-1.88_core_%C2%B7_1.96_backends-blue?style=flat-square" alt="MSRV: 1.88 core, 1.96 backends"/></a>
    <a href=".github/workflows/ci.yml"><img src="https://img.shields.io/badge/tested-Linux_%7C_macOS_%7C_Windows-blue?style=flat-square" alt="Tested on Linux, macOS, and Windows"/></a>
  </p>
</div>

<p align="center">
  <a href="#whats-new-in-v1">What's new</a> • <a href="#quickstart">Quickstart</a> • <a href="#the-pipeline">Pipeline</a> • <a href="#backends">Backends</a> • <a href="#themes">Themes</a> • <a href="#tooling">Tooling</a> • <a href="#status--roadmap">Status</a> • <a href="#learn-more">Learn more</a>
</p>

---

## What's new in V1

The **zero-runtime pipeline is in-tree**: `tw!` no longer validates a string and hands it back —
it compiles it through the full V1 pipeline at macro-expansion time and emits a frozen
`StaticProgram`. The runtime does what hand-written code does: apply data.

|                    | v0.x runtime path                                                                 | **V1 pipeline (landed)**                                                                |
| ------------------ | --------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------- |
| `tw!("p-4 …")`     | validated, then the **input string emitted back** — re-parsed every process start | compiled at expansion → **frozen static program**; the string is never emitted          |
| Static hot path    | parse → resolve → cache (**exclusive lock even on a hit**)                        | **zero parse, zero hash, zero locks** — static data applied under a `Copy` `Env`        |
| IR                 | heap-shaped `StylePlan` (`Vec`/`BTreeMap`/`Box`), per-edge set-mask patches       | `Copy` longhands (**121 props**) + `PropMask` bitsets — `p-4 ⊕ px-8` is bit ops         |
| Theme lookups      | `HashMap<String, Token>` by name                                                  | **tiered tables**: T0 literals · T1 dark pairs · T2 one load + one multiply             |
| `cn()` / overrides | string-level tw_merge at runtime                                                  | **IR-level `cn()` + `overlay`** — rightmost-wins per longhand, computed at compile time |
| Utility semantics  | 4,000-line `match` walls                                                          | **189 registry rows as data** → generated index, docs, sort order, conformance          |
| Backends           | re-lower plans on every call                                                      | all six consume the frozen program; GPUI gets macros, `Env` plumbing and polyfills      |
| Dynamic strings    | one big runtime matcher                                                           | same compiler behind `DynCache` + `ClassCell` — **no capability split**                 |

> [!IMPORTANT]
> **The honesty contract carried over.** Every class that does not lower produces a spanned, coded,
> helped diagnostic — in the macro, the CLI, and every backend walk. Per-backend capability is a
> published [conformance score](docs/CONFORMANCE.md) computed from the emitter's own gates, never a
> parallel claim.

## Quickstart

Rust **1.96+** for the GUI path; `aeolus-core` and the five V1 compiler crates build on **1.88**.
Crates are consumed from git (see [Status](#status--roadmap)):

```toml
[dependencies]
# Static / macro path (flagship):
aeolus-gpui = { git = "https://github.com/elcoosp/aeolus" }
# …or the runtime resolver path:
aeolus-core  = { git = "https://github.com/elcoosp/aeolus" }
aeolus-xilem = { git = "https://github.com/elcoosp/aeolus" }
```

### Static — compile-time (GPUI)

```rust,ignore
use aeolus_gpui::prelude::*;

// Parsed, theme-resolved and merged at expansion time.
// At runtime: static data + a Copy Env. That's it.
div().tw(tw!(
    "flex items-center gap-2 p-4 rounded-md bg-primary text-primary-foreground \
     hover:bg-primary/90 dark:shadow-none md:p-6"
))
```

Closed-world components compile to dispatch tables — shadcn's class tables keep their web vocabulary:

```rust,ignore
variants! { pub Button {
    base:    "inline-flex items-center justify-center gap-2 rounded-md text-sm font-medium",
    variant: { default: "bg-primary text-primary-foreground hover:bg-primary/90",
               destructive: "bg-destructive text-white hover:bg-destructive/90" },
    size:    { default: "h-9 px-4 py-2", sm: "h-8 px-3", lg: "h-10 px-6" },
    // cva `compound:` lists and tailwind-variants `slots:` maps supported —
    // every (slot, combo) arm compiles at expansion time.
}}
// Button::style(Variant::Default, Size::Sm) → &'static program. Zero strings at runtime.
```

### Runtime — the resolver lane (Xilem, Floem, Iced, egui, Slint)

```rust
use aeolus_core::{Resolver, shadcn_default};
use aeolus_xilem::TwViewExt;
use xilem_masonry::view::label;

let theme = shadcn_default().unwrap();       // embedded shadcn light palette
let resolver = Resolver::new(&theme);

let save = label("Save changes").tw(
    &resolver,
    "p-4 bg-primary text-primary-foreground rounded-md hover:bg-primary/90",
);
```

No build step, no browser; a CSS theme file is optional. `.tw` panics on unparseable classes in
debug and warns + drops styling in release — the honest-failure contract, not a silent no-op.
Every backend also has the V1 lane: `lower_program` walks a frozen `StaticProgram` under one probe
`Env` into its native payload, differentially pinned against the plan path.

### Dynamic strings — the explicit slow lane

```rust,ignore
// In view state: compiled when the string changes, a content compare per frame otherwise.
struct Card { classes: ClassCell }   // !Sync, UI-thread, no RwLock/Arc atomics

div().tw_dyn(self.classes.get(&self.user_classes, theme))
```

`ClassCell` keys on exact content + compile identity (never hash-only), hands back the _same_
interned slices on a hit, and degrades to the last good program on a failed compile — never a
panic per frame. Behind it, `DynCache` is the bounded, content-keyed, thread-safe pool
(two-generation rotation, swappable `DynStore`). Full v4 coverage — the same compiler as `tw!`.

<details><summary><b>More usage: shadcn widgets, theme swaps, plan algebra, snapshots</b></summary>

```rust,ignore
// shadcn: real interactive widgets, your class merges last and wins via cn().
aeolus_shadcn::xilem::button(&resolver, "save", "Save",
    ButtonVariant::Default, ButtonSize::Default, "shadow-lg",
    |state: &mut AppState| state.save());
// Same tables: Floem selector maps · Iced StyleBook closures · egui WidgetVisuals · Slint states.

// Resolve without a widget:
let resolution = resolver.resolve("p-4 bg-primary hover:bg-primary/90")?;
assert!(resolution.warnings.is_empty());

// Theme swaps are explicit and version-tracked:
let mut theme = TailwindTheme::from_theme_css(custom_brand_css)?;
theme.absorb_css(dark_mode_overrides)?;   // bumps the version; drop the PlanCache

// v0.x plan algebra + portable artifacts (still the runtime lane's IR):
let merged = base.overlay(&overrides);            // later-wins per slot, diff-integrated
let json = merged.to_snapshot_json();             // canonical, byte-stable
let same = StylePlan::from_snapshot_json(&json)?; // strict reader: unknown fields are errors

// V1 IR-level composition — frozen base ⊕ runtime override:
let merged = aeolus_compile::overlay(&base, &overrides); // rightmost-wins per longhand
```

</details>

## The pipeline

One pure, deterministic function — `compile(&str, &Config)` — with a theme pass beside it.
Every stage is total and fuzzed; the same compiler serves macros, the runtime lane and the CLI.

```mermaid
flowchart LR
    A["class string"] --> S["aeolus-syntax<br/>zero-copy lexer + spans"]
    D["aeolus-data<br/>189 registry rows"] --> C
    T["aeolus-theme<br/>app.css → T0/T1/T2 tiers"] --> C
    S --> C["aeolus-compile<br/>lookup · longhand lowering<br/>variants → Cond · cn()/overlay"]
    C --> P["Program (aeolus-ir)<br/>Copy longhands + PropMask<br/>cascade-sorted layers"]
    P --> R["(a) emit_rust<br/>per-class Layer::new — the tw! unit"]
    P --> B["(b) emit_static<br/>byte-stable static item"]
    P --> I["(c) interp<br/>effective decls under a probe Env"]
    P --> X["(d) emit_css<br/>real v4 stylesheet"]
    B --> BK["6 backends<br/>GPUI · Xilem · Floem · Iced · egui · Slint"]
    P -. "emit_plan bridge" .-> SP["v0.x StylePlan<br/>runtime lane + differential oracle"]
```

| Emit arm            | Entry points                                        | What you get                                                                                               |
| ------------------- | --------------------------------------------------- | ---------------------------------------------------------------------------------------------------------- |
| **(a) Rust tokens** | `emit_rust` / `emit_rust_theme`                     | one embeddable `Layer::new` fragment per class — keyword slots resolved to row text, tokens to table names |
| **(b) Static data** | `emit_static`                                       | the program frozen into the IR's own compilable, byte-stable `static` item                                 |
| **(c) Interpreter** | `interp*` (+ `_probes` / `_containers` / `_pseudo`) | effective declarations under one probe `Env`; exclusions reported as typed notes                           |
| **(d) CSS**         | `emit_css` / `emit_css_theme`                       | v4-faithful stylesheet — verification, debugging, and `aeolus build`                                       |

**Differential triangle:** new compiler ⟷ v0.x `Resolver` (kept in-tree as oracle) ⟷ real
`tailwindcss` v4.1.17 (193-entry golden corpus). Any disagreement is a bug in one of the three.

## Backends

Every backend consumes **both** the v0.x `StylePlan` (runtime lane) and the V1 frozen
`StaticProgram` (`lower_program`, differential-pinned, benched, with a headless example).

| Backend         | Crate          | Lowers to                                               | Highlights                                                                                                  |
| --------------- | -------------- | ------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------- |
| **GPUI** ⭐     | `aeolus-gpui`  | `TwExt` / `TwStateExt` + frame `Env`, `AeolusRoot`      | macro/static flagship; `ring-*` → v4 `BoxShadow` bands; 2-stop linear gradients; `tw_probes` attribute bits |
| Xilem / Masonry | `aeolus-xilem` | `PropertyStack` + per-state `ResolvedStyle`             | **native gradients** (linear exact, radial normalized; conic reported)                                      |
| Floem           | `aeolus-floem` | programmatic `Style`, nested selector maps              | native transitions; full pinned-rev taffy surface (grid, overflow, dashes…)                                 |
| Iced            | `aeolus-iced`  | `StyleBook` → Catalog closures per `Status`/`&Theme`    | linear gradients native; per-corner radius; opacity baked into painted alphas                               |
| egui            | `aeolus-egui`  | `Frame` + `WidgetVisuals` state cascade                 | hover/press restyle via egui's own machinery; shadows with spread native                                    |
| Slint           | `aeolus-slint` | brushes + generated `.slint` bindings & `states` blocks | every emitted binding **compiler-validated in CI**                                                          |

Per-slot support: [capability matrix](reports/02-capability-matrix/CAPABILITY_MATRIX.md) ·
V1 conformance: [docs/CONFORMANCE.md](docs/CONFORMANCE.md) (generated from the emit gates, drift-gated).

## Themes

**The same `app.css` a web Tailwind v4 project uses works unchanged** — that's a headline feature.

```css
/* app.css */
@theme { --color-primary: oklch(0.55 0.2 250); }
.dark  { --color-primary: oklch(0.70 0.15 250); }
```

Pay only for the dynamism you use:

| Tier              | When                                   | Emitted                | Runtime cost                      |
| ----------------- | -------------------------------------- | ---------------------- | --------------------------------- |
| **T0** static     | theme known at compile time (`theme!`) | colour literals        | **0**                             |
| **T1** light/dark | same, with a `.dark` block             | `[Rgba; 2]` const pair | 1 branch-free index by `Env.dark` |
| **T2** runtime    | user-editable themes, live bridges     | `Tok { id, alpha }`    | 1 table load + 1 multiply         |

- **`/opacity` is one multiply, not colour maths.** v4's `color-mix(… transparent)` in premultiplied
  space only scales alpha: `bg-primary/90` → `(L, a, b, α·0.9)`. Baked to a literal when static;
  property-tested against the real compiler.
- `calc()` + `--spacing` folded at build time; oklch → sRGB gamut-mapped **once**.
- `theme!` emits an `include_str!` pin, so editing the CSS triggers a Cargo rebuild.
- **CSS-first authoring, all four forms:** `@theme`, `@utility` (incl. functional `--value()`),
  `@custom-variant`, `@source` (inline literals, walked paths, pruned paths — feeds `check`/`build`).

## Variants & state

`Env` is a `Copy` 4-byte struct written once per frame (`AeolusRoot`) — one TLS read, no locks.

| Family                                                     | V1 mapping                                                            | Status                                                        |
| ---------------------------------------------------------- | --------------------------------------------------------------------- | ------------------------------------------------------------- |
| `hover:` `active:` `focus:` `focus-visible:`               | native state refinements (GPUI) / selector maps / status closures     | ✅                                                            |
| `group-*` / `peer-*` (incl. named, `*-has-*`)              | parent-tier typed `Cond`s                                             | ✅                                                            |
| `dark:`                                                    | T1 pair index / `Env` mode                                            | ✅ branch-free                                                |
| `sm:` … `2xl:`                                             | `Env.bp` compare                                                      | ✅ 1 compare                                                  |
| `data-*:` `aria-*:`                                        | typed `AttrProbe` bits (`tw_probes`), CSS selector semantics          | ✅                                                            |
| `has-*:`                                                   | typed `Cond::Has`; element-side answered                              | ✅ (descendant probes diagnosed)                              |
| `@md/card:` named containers                               | typed `Cond` + `ContainerProbe` bounds seam, CLI `expand --container` | ✅ (element-bounds wrapper = backend boundary)                |
| `selection:` `placeholder:`                                | `PseudoTarget` scoped cascades                                        | ✅                                                            |
| `transition-*` `duration-*` `ease-*` `delay-*` `animate-*` | five motion longhands, v4 default-theme texts                         | ✅ pipeline (GPUI `with_animation` needs id+state — reported) |
| arbitrary selectors, unsupported forms                     | precise per-form reason + suggested alternative                       | 🛑 diagnosed, never silent                                    |

## Tooling

```sh
aeolus check src/ --theme app.css --fail-on unknown   # lint class strings in CI (@source-aware)
aeolus sort "block p-4 mx-2 hover:block"              # canonical registry order
aeolus explain "p-4 flex-col" --theme shadcn          # class → the Rust tokens it emits
aeolus expand "hidden dark:block" --dark              # effective declarations under a probe Env
aeolus build src/ --theme shadcn-dark --out app.css   # whole project → ONE stylesheet
aeolus resolve "p-4 bg-primary" | jq .                # plan snapshot · css · diff · cn also available
```

| Tool                                       | What it does                                                                                                                                                                          |
| ------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `aeolus build`                             | scan → global dedupe → registry-ordered rules + self-contained `:root`/`.dark` vars block; diagnostics at **every** occurrence                                                        |
| `aeolus check`                             | unknown classes with suggestions, unsupported utilities with reasons, `cn()` conflicts with winner attribution; `--fail-on` exit codes                                                |
| `cargo xtask gen-tables` / `verify-tables` | regenerate & drift-gate the committed artifacts: roots index, [docs/REGISTRY.md](docs/REGISTRY.md), [docs/CONFORMANCE.md](docs/CONFORMANCE.md) — a stale row fails plain `cargo test` |

## Correctness & performance

- **Spec-first:** registry + IR published as data; golden corpus generated by the real v4.1.17
  compiler (193 entries, growing by mining Tailwind's own test cases).
- **Differential pins everywhere:** lexer ⟷ v0.x parser, `cn()` ⟷ tw_merge oracle, plan bridge ⟷
  v0.x resolver, index ⟷ linear scan, `ClassCell` ⟷ macro-frozen program (byte-equal).
- **Fuzz + property suites:** six fuzz targets (60 s CI sweeps), `compile ⇒ interpret == emit`
  metamorphic target, `/opacity` identity property test.
- **Allocation contract:** counting-allocator tests pin the hot path (`diff` of equal plans:
  **zero** allocations). Workspace battery: 1,112 tests (1,122 with instrumentation features).

| Metric                                 | v0.x — *measured*¹          | V1 — *target*²                                       |
| -------------------------------------- | --------------------------- | ---------------------------------------------------- |
| Static class application               | ~405 ns warm + lowering     | hand-written-builder parity (±2 % instruction count) |
| Locks / allocs on the static path      | ≥1 exclusive lock, ≥1 alloc | **0 / 0** (+ GPUI's own boxing)                      |
| Cold dynamic resolve (33-class button) | 4.6 µs                      | ≤ 1.5 µs                                             |
| Warm dynamic (`ClassCell` hit)         | ~405 ns                     | ≤ 20 ns                                              |

¹ divan, release profile, Linux CI — method & caveats in [BENCHMARKS.md](reports/08-benchmarks/BENCHMARKS.md).
² Contract from the [V1 plan §4](docs/AEOLUS_V1_ARCHITECTURE.md); the P0 `iai-callgrind`/criterion gates
replace targets with measured numbers before 1.0. Structural guarantees (no parse/hash/locks on the
static path) hold by construction today.

## Status & roadmap

**v0.19.0** (2026-11). `aeolus-core` is publish-ready (`cargo package` is a CI gate); the V1 crates,
backends, macros, runtime, CLI and xtask are **git-only** (`publish = false`) while the pipeline
settles toward 1.0. The v0.x `Resolver`/`StylePlan` stay in-tree as the runtime lane **and** the
differential oracle — no big-bang migration.

| Phase                   | Scope                                                                                                                                             | Status                         |
| ----------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------ |
| **P0** Prove it         | iai/criterion gates vs hand-written builders; replace §4 targets                                                                                  | 🎯 targets pending measurement |
| **P1** Data-driven core | `aeolus-syntax` · `aeolus-data` (189 rows) · `aeolus-ir` (121 longhands)                                                                          | ✅ landed                      |
| **P2** Full pipeline    | compile + theme passes, IR `cn()`, **all four emit arms**, customs, `DynCache`, arbitrary values, attribute fold, xtask generator                 | ✅ landed                      |
| **P3** GPUI static      | `aeolus-codegen(-gpui)`, reworked `aeolus-macro`, `aeolus-rt`, `aeolus-gpui`; `ClassCell` dynamic lane; ring polyfill                             | ✅ landed                      |
| **P4** Env & state      | frame `Env`, dark/breakpoints, state layers, `AttrProbe` host seam                                                                                | ✅ landed                      |
| **P5** Ecosystem        | gpui-rsx integration PR (`lower_class` seam ready), LSP, gpui-component bridge, docs book, public benchmarks, **1.0-rc**                          | 🔜 next                        |
| **P6** Breadth          | 5-backend re-target ✅ · ring + gradient lanes ✅ · motion pipeline ✅ · named containers + bounds seam ✅ · pseudo targets ✅ · project build ✅ | ✅ mostly — see boundaries     |

**Open boundaries — typed, test-pinned, never silent:**

- `calc()` / multi-token composites, arbitrary-property form, the wider v4 hint table → typed refusals
- `has-*` descendant probes → diagnosed with reasons (element-side attribute probes are answered)
- container **measurement** (who computes element widths) → `ContainerProbe` seam landed; the layout wrapper is backend work
- GPUI motion (`with_animation` needs element id + state) and >2-stop / radial / conic gradients → per-longhand refusals
- per-backend gradient gaps (conic on Masonry, radial/conic on Iced, solid fills on egui) → capability matrix

## Crate map

**Compiler** (zero-dep core discipline; MSRV 1.88)

| Crate            | Role                                                                                                                   |
| ---------------- | ---------------------------------------------------------------------------------------------------------------------- |
| `aeolus-syntax`  | zero-copy lexer: borrowed candidates + spans, `no_std + alloc`                                                         |
| `aeolus-data`    | the registry as data: 189 audited rows → generated roots index, docs, sort order                                       |
| `aeolus-ir`      | `Copy` longhand IR: `Prop`/`Val`/`Decl`/`Layer`/`Program` + `PropMask` bitset algebra                                  |
| `aeolus-theme`   | CSS-first `ThemeSource` → tiered `TokenTable`s; `@source` directives                                                   |
| `aeolus-compile` | the pipeline: `compile`/`cn`/`overlay`, 4 emit arms, `Customs`, `DynCache`, `sort`, `scan`/`build`, `emit_plan` bridge |

**Emit & runtime**

| Crate                 | Role                                                                                                                        |
| --------------------- | --------------------------------------------------------------------------------------------------------------------------- |
| `aeolus-codegen`      | backend-neutral `Emit` trait + `Capability`/`Conformance` report                                                            |
| `aeolus-codegen-gpui` | GPUI token emitter, native + polyfill capability tiers, `lower_class` third-party seam                                      |
| `aeolus-macro`        | `tw!` / `cn!` / `tw_cn!` / `variants!` (compound + slots) / `theme!` — syn-free; emits the frozen program, never the string |
| `aeolus-rt`           | host seam: `StaticProgram`, `Env`, `TokenTable`, `DynCache`, `ClassCell`, `theme_from_css`                                  |

**Integration & tooling**

| Crate                               | Role                                                                                            |
| ----------------------------------- | ----------------------------------------------------------------------------------------------- |
| `aeolus-gpui` + five backend crates | see [Backends](#backends) — plan lane + V1 static-program lane each                             |
| `aeolus-shadcn`                     | shadcn `new-york-v4` class tables verbatim + per-backend wrappers                               |
| `aeolus-cli`                        | `resolve` `css` `diff` `cn` `check` `explain` `sort` `expand` `build` — zero external crates    |
| `xtask`                             | `gen-tables` / `verify-tables` drift gates for the committed artifacts                          |
| `aeolus-core`                       | v0.x `Resolver` + `StylePlan`: the runtime lane and the differential oracle — **publish-ready** |

_`crates/muda-stub` is a dev-sandbox patch crate — never a dependency of a publishable crate._

## Examples

The fastest way in — every crate ships runnable, headless examples:

```sh
cargo run -p aeolus-compile --example compile_pipeline  # lex → registry → longhand Program + diagnostics
cargo run -p aeolus-compile --example theme_seam        # tiers + the /opacity identity
cargo run -p aeolus-compile --example project_build     # scan → dedupe → whole-project stylesheet
cargo run -p aeolus-rt --example class_cell             # the dynamic lane: hit/miss counters, honest degradation
cargo run -p aeolus-gpui --example ring                 # ring-* → v4 BoxShadow bands
cargo run -p aeolus-codegen-gpui --example conformance  # the published score, from the emit gates themselves
```

<details><summary><b>All examples, by crate</b></summary>

```sh
# core (runtime lane)
cargo run -p aeolus-core --example resolve
cargo run -p aeolus-core --example plan_algebra
cargo run -p aeolus-core --example cn_merge
# V1 compiler
cargo run -p aeolus-syntax --example lex_candidates
cargo run -p aeolus-data --example registry_lookup
cargo run -p aeolus-data --example codegen_artifacts
cargo run -p aeolus-ir --example mask_algebra
cargo run -p aeolus-theme --example theme_tiers
cargo run -p aeolus-theme --example theme_sources
cargo run -p aeolus-compile --example compile_pipeline
cargo run -p aeolus-compile --example theme_seam
cargo run -p aeolus-compile --example cn_merge
cargo run -p aeolus-compile --example css_emit
cargo run -p aeolus-compile --example rust_tokens
cargo run -p aeolus-compile --example static_emit
cargo run -p aeolus-compile --example interp
cargo run -p aeolus-compile --example custom_seam
cargo run -p aeolus-compile --example dyn_cache
cargo run -p aeolus-compile --example arbitrary_values
cargo run -p aeolus-compile --example attr_variants
cargo run -p aeolus-compile --example probes
cargo run -p aeolus-compile --example plan_bridge
cargo run -p aeolus-compile --example sort_order
cargo run -p aeolus-compile --example project_build
cargo run -p aeolus-compile --example named_containers
cargo run -p aeolus-compile --example container_bounds
cargo run -p aeolus-compile --example pseudo_targets
cargo run -p aeolus-compile --example motion
# codegen & runtime
cargo run -p aeolus-codegen --example text_emit
cargo run -p aeolus-codegen-gpui --example conformance
cargo run -p aeolus-rt --example class_cell
cargo run -p aeolus-rt --example static_walk
cargo run -p aeolus-rt --example program_overlay
# GPUI
cargo run -p aeolus-gpui --example ring
cargo run -p aeolus-gpui --example grad
cargo run -p aeolus-gpui --example variants
cargo run -p aeolus-gpui --example slots
cargo run -p aeolus-gpui --example attr_state
# backends: plan lane + V1 program lane each
cargo run -p aeolus-xilem --example styled_label
cargo run -p aeolus-xilem --example program
cargo run -p aeolus-floem --example lower_style
cargo run -p aeolus-floem --example static_program
cargo run -p aeolus-iced --example card_style
cargo run -p aeolus-iced --example program
cargo run -p aeolus-egui --example panel_style
cargo run -p aeolus-egui --example program
cargo run -p aeolus-slint --example style_snippet
cargo run -p aeolus-slint --example program
# CLI as a library
cargo run -p aeolus-cli --example theme_spec
cargo run -p aeolus-cli --example explain
```

</details>

## Learn more

**Design & specs**

- [V1 architecture](docs/AEOLUS_V1_ARCHITECTURE.md) — the zero-runtime plan of record: pipeline, IR, theme tiers, performance contract
- [docs/REGISTRY.md](docs/REGISTRY.md) — the utility registry audit page (generated from the rows)
- [docs/CONFORMANCE.md](docs/CONFORMANCE.md) — per-backend conformance, from the emitter's own gates
- [Core design](reports/03-aeolus-core-design/DESIGN.md) — the `Probe` parity model and the honesty contract
- [Capability matrix](reports/02-capability-matrix/CAPABILITY_MATRIX.md) — per-slot support, every backend
- [Benchmarks](reports/08-benchmarks/BENCHMARKS.md) — method, numbers, allocation contract
- [CONTRIBUTING.md](CONTRIBUTING.md) — TDD-first workflow, the no-silent-failures bar

<details><summary><b>RED→GREEN evidence (TDD reports)</b></summary>

- [19 — rust-tokens arm](reports/10-tdd-evidence/19-rust-tokens-arm.md) · [18 — arbitrary values](reports/10-tdd-evidence/18-arbitrary-values.md) · [17 — interp & static arms](reports/10-tdd-evidence/17-emit-arms.md)
- [14 — theme pass](reports/10-tdd-evidence/14-theme-pass.md) · [13 — compile seam](reports/10-tdd-evidence/13-compile-seam.md)
- [12 — test pyramid](reports/10-tdd-evidence/12-test-pyramid.md) · [11 — variant engine](reports/10-tdd-evidence/11-variant-engine.md)
- [09 — portable plans](reports/10-tdd-evidence/09-portable-plans.md) · [08 — upstream ceiling](reports/10-tdd-evidence/08-upstream-ceiling.md)

</details>

<details><summary><b>Release history (plans of record)</b></summary>

- [v0.9](docs/release-notes/AEOLUS_V0.9.md) — variant & grid engine (pseudo-elements, `has-*`, named containers, arbitrary grid tracks)
- [v0.8](docs/release-notes/AEOLUS_V0.8.md) — per-edge set-mask cascade fix
- [v0.7](docs/release-notes/AEOLUS_V0.7.md) — plan algebra, canonical snapshots, the CLI
- [v0.6](docs/release-notes/AEOLUS_V0.6.md) — gradient backgrounds, the wired Floem surface
- [v0.5](docs/release-notes/AEOLUS_V0.5.md) — element-relative typography, the allocation contract
- [v0.4](docs/release-notes/AEOLUS_V0.4.md) — verification CSS emitter, golden corpus, parity harness
- [v0.3](docs/release-notes/AEOLUS_V0.3.md) — arbitrary selector variants · [v0.2](docs/release-notes/AEOLUS_V0.2.md) — parity engine, MSRV split

</details>

---

<div align="center">
  <sub>Licensed under MIT OR Apache-2.0 · Semantics pinned to tailwindcss v4.1.17 · Built with a no-silent-failures bar</sub>
</div>
