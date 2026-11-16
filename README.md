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
    <a href=".github/workflows/ci.yml"><img src="https://img.shields.io/badge/MSRV-1.88_core_%C2%B7_1.96_framework_backends-blue?style=flat-square" alt="MSRV: 1.88 core and GPUI, 1.96 framework backends"/></a>
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
| IR                 | heap-shaped `StylePlan` (`Vec`/`BTreeMap`/`Box`), per-edge set-mask patches       | `Copy` longhands (**146 props**) + `PropMask` bitsets — `p-4 ⊕ px-8` is bit ops         |
| Theme lookups      | `HashMap<String, Token>` by name                                                  | **tiered tables**: T0 literals · T1 dark pairs · T2 one load + one multiply             |
| `cn()` / overrides | string-level tw_merge at runtime                                                  | **IR-level `cn()` + `overlay`** — rightmost-wins per longhand, computed at compile time |
| Utility semantics  | 4,000-line `match` walls                                                          | **213 registry rows as data** → generated index, docs, sort order, conformance         |
| Backends           | re-lower plans on every call                                                      | all six consume the frozen program; GPUI gets macros, `Env` plumbing and polyfills      |
| Dynamic strings    | one big runtime matcher                                                           | same compiler behind `DynCache` + `ClassCell` — **no capability split**                 |

> [!IMPORTANT]
> **The honesty contract carried over.** Every class that does not lower produces a spanned, coded,
> helped diagnostic — in the macro, the CLI, and every backend walk. GPUI capability is a
> published [conformance score](docs/CONFORMANCE.md) computed from the emitter's own gates, never a
> parallel claim.

## Quickstart

Rust **1.88** for the compiler set, CLI and the GPUI lane; the five framework backends and
`aeolus-shadcn` build on **1.96** (their pinned frameworks' own requirement).
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

let program = self.classes.get(&self.user_classes, &config)?; // Result — never a panic per frame
div().tw_dyn(program)
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
    D["aeolus-data<br/>213 registry rows"] --> C
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
**The metamorphic checker** (`aeolus_compile::metamorphic_check*`) walks the triangle's in-tree
half per string: `interp(compile(s))` must equal the CSS emit's cascade-resolved rules for every
probe situation the string names — the diagnostics channels byte-equal, the sheet the
per-candidate parts, every documented divergence a typed note.

## Backends

Every backend consumes **both** the v0.x `StylePlan` (runtime lane) and the V1 frozen
`StaticProgram` (`lower_program`, differential-pinned, benched, with a headless example).

| Backend         | Crate          | Lowers to                                               | Highlights                                                                                                  |
| --------------- | -------------- | ------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------- |
| **GPUI** ⭐     | `aeolus-gpui`  | `TwExt` / `TwStateExt` + frame `Env`, `AeolusRoot`      | macro/static flagship; `ring-*` → v4 `BoxShadow` bands; the filter chain's colour-matrix polyfill + `drop-shadow-*` bands; 2-stop linear gradients; `tw_probes` attribute bits |
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
  The `@custom-variant`/`@utility` statements register onto the same typed `Customs` surface
  (`customs_from_css`): `@apply` composite bodies compile at registration — the multi-decl
  `CustomValue::Applied` form, the body's keyword texts riding beside the decls — and the functional
  star bodies (`--value(integer|number|percentage|color)`, `--spacing(n)`) ride the same
  `CustomForm` grammar the typed API takes; every malformed statement, refused selector and
  outside-the-grammar value is a typed `CustomCssError`.

## Variants & state

`Env` is a `Copy` 4-byte struct written once per frame (`AeolusRoot`) — one TLS read, no locks.

| Family                                                     | V1 mapping                                                            | Status                                                        |
| ---------------------------------------------------------- | --------------------------------------------------------------------- | ------------------------------------------------------------- |
| `hover:` `active:` `focus:` `focus-visible:`               | native state refinements (GPUI) / selector maps / status closures     | ✅                                                            |
| `group-*` / `peer-*` (incl. named, `*-has-*`)              | parent-tier typed `Cond`s                                             | ✅                                                            |
| `dark:`                                                    | T1 pair index / `Env` mode                                            | ✅ branch-free                                                |
| `sm:` … `2xl:`                                             | `Env.bp` compare                                                      | ✅ 1 compare                                                  |
| `data-*:` `aria-*:`                                        | typed `AttrProbe` bits (`tw_probes`), CSS selector semantics          | ✅                                                            |
| `has-*:` `group-has-*:` `peer-has-*:`                      | typed `Cond::Has` + `DescendantProbe` subtree facts (`tw`-side `*_descendants` walks), CSS `:has()` semantics | ✅                                                            |
| `supports-*:` `supports-[…]:`                              | typed `Cond::Supports` + `SupportsProbe` capability facts (`tw`-side `*_supports` walks), CSS `@supports` semantics | ✅                                                            |
| `motion-*:` `contrast-*:` `forced-colors:` `print:`        | typed `Cond::Media` bits, `Env` preference facts (frame state — no probe seam), CSS media-query semantics | ✅                                                            |
| `pointer-*:` `any-pointer-*:` `portrait:` `landscape:`     | typed `Cond::Media` bits, `Env` capability facts (the same frame-state tier; `landscape:` the `portrait:` complement), CSS media-query semantics | ✅                                                            |
| `ltr:` `rtl:`                                              | typed `Cond::Dir` bit, one `Env` direction fact (`ltr:` the `rtl:` complement), CSS `:where(:dir(…))` selector semantics | ✅                                                            |
| `valid:` `invalid:` `user-valid:` `user-invalid:`          | typed `Cond` states, two `Env` facts (the constraint pair + the `user-` interaction gate), CSS `:pseudo` selector semantics | ✅                                                            |
| `target:` `indeterminate:` `autofill:` `read-only:`        | typed `Cond` states, one `Env` fact each (`read-only:` the editability fact's complement — the resting env answers), CSS `:pseudo` selector semantics | ✅                                                            |
| `open:` `inert:` `default:`                                | typed `Cond` states, one affirmative `Env` fact each (the v0.x `State` surface's own slots — the v0.x-gap closure the element-state landing documented), CSS `:is([open], :popover-open, :open)`/`:is([inert], [inert] *)`/`:default` selector semantics | ✅                                                            |
| `@md/card:` named containers                               | typed `Cond` + `ContainerProbe` bounds seam, CLI `expand --container` | ✅ (`tw_container` measures; the walk consumes)               |
| `selection:` `placeholder:`                                | `PseudoTarget` scoped cascades                                        | ✅                                                            |
| `transition-*` `duration-*` `ease-*` `delay-*` `animate-*` | five motion longhands, v4 default-theme texts                         | ✅ pipeline (GPUI `with_animation` needs id+state — reported) |
| arbitrary selectors, unsupported forms                     | precise per-form reason + suggested alternative                       | 🛑 diagnosed, never silent                                    |

## Tooling

```sh
aeolus check src/ --theme app.css --fail-on unknown   # lint class strings in CI (@source-aware)
aeolus sort "block p-4 mx-2 hover:block"              # canonical registry order
aeolus fmt src/widget.rs                              # one file's class strings into the canonical order (--check = the CI drift gate)
aeolus explain "p-4 flex-col" --theme shadcn          # class → the Rust tokens it emits
aeolus expand "hidden dark:block" --dark              # effective declarations under a probe Env
aeolus expand --has element=*:focus "has-focus:p-8"   # the subtree facts the has-* layers answer from
aeolus build src/ --theme shadcn-dark --out app.css   # whole project → ONE stylesheet
aeolus plan "p-4 hover:px-8" --theme shadcn | jq .    # the V1 lane's plan bridge → the same snapshot JSON
aeolus resolve "p-4 bg-primary" | jq .                # plan snapshot · css · diff · cn also available
```

| Tool                                       | What it does                                                                                                                                                                          |
| ------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `aeolus build`                             | scan → global dedupe → registry-ordered rules + self-contained `:root`/`.dark` vars block; diagnostics at **every** occurrence                                                        |
| `aeolus plan`                              | the V1 lane's plan bridge (`Target::Plan`): the v0.x `StylePlan` the backends consume, as the same canonical snapshot JSON `resolve` prints; slot-boundary notes + per-class misses ride stderr, `--fail-on` keys on the typed codes                        |
| `aeolus check`                             | unknown classes with suggestions, unsupported utilities with reasons, `cn()` conflicts with winner attribution; `--fail-on` exit codes                                                |
| `aeolus fmt`                               | the class-sort seam's file face: one file's class strings into the canonical order (`sort`'s batch form — stdout rewrite, the file untouched); `--check` prints `path:line:col "before" -> "after"` per out-of-order string and exits 1 — the CI drift gate |
| `cargo xtask gen-tables` / `verify-tables` | regenerate & drift-gate the committed artifacts: roots index, [docs/REGISTRY.md](docs/REGISTRY.md), [docs/CONFORMANCE.md](docs/CONFORMANCE.md), the GPUI capability table — a stale row fails plain `cargo test` |
| `aeolus-lsp` (the LSP server)              | editor hover, diagnostics, completion and colour swatches over the same pipeline: `hover` per class (longhand signature or typed refusal with its code), the extraction seam's strings linted per location, `completion` at the typing edge (registry spellings, theme tokens, the closed variant set — the compiler's own audited tables), `documentColor` swatches (the table's resolved light-tier rgba), `documentDiagnostic` pull (LSP 3.17 — the SAME findings the push channel publishes, recomputed on demand), `semanticTokens` (LSP 3.16 — every candidate's compile outcome onto the legend: lowered `class`, refused the custom `refused` type), `colorPresentation` (the picker's spellings — the table walked backwards, every root/token/opacity-step that re-resolves to the picked colour), `inlayHint` (LSP 3.17 — every lowered class rides one hint at its end: the CSS arm's own declarations compacted, the hover's markdown as the tooltip; a refused class never hints), `codeAction` quick fixes (the "did you mean" seam — every correction a spelling the pipeline lowers, the variant prefix and `/modifier` tail composed back on), `documentHighlight` (the cursor's class, every occurrence in the file) and `formatting` (the class-sort seam's editor face — one edit per out-of-order class string, the canonical order as the new text, an already-canonical string answering nothing: the format-on-save no-op); `bin/aeolus-lsp` speaks stdio JSON-RPC |

## Correctness & performance

- **Spec-first:** registry + IR published as data; golden corpus generated by the real v4.1.17
  compiler (193 entries, growing by mining Tailwind's own test cases).
- **Differential pins everywhere:** lexer ⟷ v0.x parser, `cn()` ⟷ tw_merge oracle, plan bridge ⟷
  v0.x resolver, index ⟷ linear scan, `ClassCell` ⟷ macro-frozen program (byte-equal).
- **Fuzz + property suites:** seven fuzz targets (60 s CI sweeps — the §5.3 metamorphic
  `compile ⇒ interpret == emit` target rides the V1 pipeline since 2026-11), `/opacity`
  identity property test, the deterministic metamorphic battery (`tests/metamorphic.rs`).
- **Allocation contract:** counting-allocator tests pin the hot path (`diff` of equal plans:
  **zero** allocations). Workspace battery: 1750 tests with all features enabled (2026-11).

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

## Feature flags

Every flag is additive and off by default — the documented defaults are
the zero-dependency builds (the `--all-features` CI run covers them all):

| Flag | Crates | What it adds |
| --- | --- | --- |
| `tracing` | `aeolus-cli`, `aeolus-compile`, `aeolus-data`, `aeolus-theme` | `debug_span!`/`debug!` events on the pipeline walk (compile, interp, sort, expand); no subscriber is installed — the host owns that |
| `miette` | `aeolus-cli`, `aeolus-compile`, `aeolus-data`, `aeolus-theme` | structured [`miette::Diagnostic`] impls on the typed errors/diagnostics (codes, help text, spans) |
| `verbose-diagnostics` | `aeolus-macro` | multi-line compile-error rendering for `tw!`/`cn!` refusals (the single-line form stays the default) |

## Status & roadmap

**v0.19.0** (2026-11). `aeolus-core` is publish-ready (`cargo package` is a CI gate); the V1 crates,
backends, macros, runtime, CLI and xtask are **git-only** (`publish = false`) while the pipeline
settles toward 1.0. The v0.x `Resolver`/`StylePlan` stay in-tree as the runtime lane **and** the
differential oracle — no big-bang migration.

| Phase                   | Scope                                                                                                                                             | Status                         |
| ----------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------ |
| **P0** Prove it         | iai/criterion gates vs hand-written builders; replace §4 targets                                                                                  | ✅ the stable half landed 2026-11 — `benches/perf_contract` (the walk beside the hand-written chains, parity-pinned; walk 1.07 µs vs hand 84 ns on the button string, clone 70 ns, cell warm 908 ns) — the iai-callgrind instruction-count golden stays nightly |
| **P1** Data-driven core | `aeolus-syntax` · `aeolus-data` (213 rows) · `aeolus-ir` (146 longhands)                                                                          | ✅ landed                      |
| **P2** Full pipeline    | compile + theme passes, IR `cn()`, **all four emit arms**, customs, `DynCache`, arbitrary values, attribute fold, xtask generator                 | ✅ landed                      |
| **P3** GPUI static      | `aeolus-codegen(-gpui)`, reworked `aeolus-macro`, `aeolus-rt`, `aeolus-gpui`; `ClassCell` dynamic lane; ring polyfill; shadow tier (`shadow-*` → GPUI's own `Vec<BoxShadow>`) | ✅ landed                      |
| **P4** Env & state      | frame `Env`, dark/breakpoints, state layers, `AttrProbe` host seam                                                                                | ✅ landed                      |
| **P5** Ecosystem        | gpui-rsx integration PR (`lower_class` seam ready), **LSP landed 2026-11** (`aeolus-lsp`: hover, diagnostics, completion and colour swatches over the same pipeline), gpui-component bridge, docs book, public benchmarks, **1.0-rc** | 🔜 next — the LSP row is in |
| **P6** Breadth          | 5-backend re-target ✅ · ring + gradient lanes ✅ · motion pipeline ✅ · named containers + bounds seam + measuring wrapper ✅ · pseudo targets ✅ · project build ✅ | ✅ mostly — see boundaries     |

**Open boundaries — typed, test-pinned, never silent:**

- composite arbitrary values landed (the IR's verbatim slot: `calc()`, track lists, `url()`, `aspect-[16/9]`) and the wider v4 hint table landed (`angle:` on the degree rows, `time:` on the millisecond rows, `position:`, `family-name:`/`generic-name:`, `absolute-size:`/`relative-size:`, `string:` onto the new `content` longhand — the `content-["…"]` spelling now lowers); the arbitrary-property form (`[mask-type:luminance]`) → the remaining typed refusal
- the `has-*` family answers through the descendant-probe seam (`DescendantProbe`/`DescendantState`, the `*_descendants` walks); the `*:` arbitrary-variant half stays diagnosed with reasons (honest failure, kept)
- container **measurement** (who computes element widths) → the seam landed, the GPUI walk answers named queries through it (`tw_containers`), and the GPUI measuring wrapper landed beside it: `tw_container(name, &MeasuredContainers)` publishes the element box's width at the layout phase, the delta handshake asks for one re-render on a real change
- GPUI motion's static emit keeps its per-longhand refusals (`with_animation` needs element id + animator — the stateful `TwAnimExt` lane applies them at runtime); >2-stop / radial / conic gradients refuse
- the decoration family landed (`decoration-{color,thickness,style}` — three appended longhands, the GPUI walk riding the pinned `text_decoration_*` methods); GPUI renders a single solid line, so the `double`/`dotted`/`dashed` style keywords and the `auto`/`from-font`/rem/percent thickness forms keep their typed refusals (the CSS/interp arms carry the real output)
- the shadow tier's `shadow-inner` placeholder keeps its typed note on the plan bridge (the `currentColor` has no plan answer); GPUI composes it (the pinned `BoxShadow` carries the `inset` flag), and the plan presets/`shadow-none`/`shadow-{color}` map through the oracle's own pass order (a preset beats the recolour, the oracle's own observable semantics)
- per-backend gradient gaps (conic on Masonry, radial/conic on Iced, solid fills on egui) → capability matrix

## Crate map

**Compiler** (zero-dep core discipline; MSRV 1.88)

| Crate            | Role                                                                                                                   |
| ---------------- | ---------------------------------------------------------------------------------------------------------------------- |
| `aeolus-syntax`  | zero-copy lexer: borrowed candidates + spans (std build today; `no_std + alloc` capable per the architecture plan)      |
| `aeolus-data`    | the registry as data: 213 audited rows → generated roots index, docs, sort order                                       |
| `aeolus-ir`      | `Copy` longhand IR: `Prop`/`Val`/`Decl`/`Layer`/`Program` + `PropMask` bitset algebra                                  |
| `aeolus-theme`   | CSS-first `ThemeSource` → tiered `TokenTable`s; `@source` directives                                                   |
| `aeolus-compile` | the pipeline: `compile`/`cn`/`overlay`, 4 emit arms, `Customs`, `DynCache`, `sort`, `scan`/`build`, `emit_plan` bridge |

**Emit & runtime**

| Crate                 | Role                                                                                                                        |
| --------------------- | --------------------------------------------------------------------------------------------------------------------------- |
| `aeolus-codegen`      | backend-neutral `Emit` trait + `Capability`/`Conformance` report                                                            |
| `aeolus-codegen-gpui` | GPUI token emitter, native + polyfill capability tiers, `lower_class` third-party seam                                      |
| `aeolus-macro`        | `tw!` / `cn!` / `tw_cn!` / `variants!` (compound + slots) / `theme!` — syn-free; emits the frozen program, never the string |
| `aeolus-rt`           | host seam: `StaticProgram`, `Env`, `TokenTable`, `DynCache`, `ClassCell`, `MeasuredContainers`, `theme_from_css`            |

**Integration & tooling**

| Crate                               | Role                                                                                            |
| ----------------------------------- | ----------------------------------------------------------------------------------------------- |
| `aeolus-gpui` + five backend crates | see [Backends](#backends) — plan lane + V1 static-program lane each                             |
| `aeolus-shadcn`                     | shadcn `new-york-v4` class tables verbatim + per-backend wrappers + the closed-world program tables (`ButtonPrograms`, the F6 fix) |
| `aeolus-cli`                        | `resolve` `css` `diff` `cn` `check` `explain` `sort` `fmt` `expand` `build` `plan` — zero external crates    |
| `aeolus-lsp`                        | the P5 LSP row: headless engine (extraction seam → V1 compile walk → UTF-16 ranges, hover signatures, completion from the audited tables, colour swatches, `documentDiagnostic` pull) + stdio JSON-RPC state machine (`bin/aeolus-lsp`) |
| `xtask`                             | `gen-tables` / `verify-tables` drift gates for the committed artifacts (+ the §3.7 `gen-gpui-caps` / `verify-gpui-caps` pair)                          |
| `aeolus-core`                       | v0.x `Resolver` + `StylePlan`: the runtime lane and the differential oracle — **publish-ready** |

_`crates/muda-stub` is a dev-sandbox patch crate — never a dependency of a publishable crate._

## Examples

The fastest way in — every crate ships runnable, headless examples:

```sh
cargo run -p aeolus-compile --example compile_pipeline  # lex → registry → longhand Program + diagnostics
cargo run -p aeolus-compile --example theme_seam        # tiers + the /opacity identity
cargo run -p aeolus-compile --example project_build     # scan → dedupe → whole-project stylesheet
cargo run -p aeolus-rt --example class_cell             # the dynamic lane: hit/miss counters, honest degradation
cargo run -p aeolus-compile --example dyn_store         # the DynStore storage seam: a host's own bounded store
cargo run -p aeolus-core --example gamut                # the CSS Color 4 gamut mapping: clamp beside chroma reduction
cargo run -p aeolus-gpui --example ring                 # ring-* → v4 BoxShadow bands
cargo run -p aeolus-gpui --example shadow               # shadow-* → GPUI's own BoxShadow list (the native tier)
cargo run -p aeolus-gpui --example filters               # the filter chain's colour-matrix polyfill + drop-shadow bands
cargo run -p aeolus-gpui --example decoration            # the decoration tier: colour/thickness/style on GPUI's own methods
cargo run -p aeolus-compile --example filters           # the pipeline's filter layer: the v4 var-chain forms
cargo run -p aeolus-compile --example backdrop_filters  # the backdrop layer: both properties, -webkit- beside the standard
cargo run -p aeolus-compile --example grid_flow         # the grid-flow family: the auto-flow slot end to end
cargo run -p aeolus-compile --example hints             # the wider v4 hint table: angle/time/position/family/string end to end
cargo run -p aeolus-compile --example shadow_plan       # the plan bridge's shadow slot: presets + recolour, differential
cargo run -p aeolus-gpui --example containers           # named @container queries through the bounds seam
cargo run -p aeolus-gpui --example container_query      # the measuring wrapper: publish → walk → resize beat
cargo run -p aeolus-gpui --example struct               # first:/last:/odd: through the DOM-position seam
cargo run -p aeolus-gpui --example motion               # the animation lane: frozen shorthand → gpui::Animation
cargo run -p aeolus-gpui --example placement            # grid placement + flex-basis on GPUI's own slots
cargo run -p aeolus-shadcn --example program_tables     # the closed-world program tables (the F6 fix)
cargo run -p aeolus-codegen-gpui --example conformance  # the published score, from the emit gates themselves
cargo run -p aeolus-compile --example metamorphic       # compile ⇒ interpret ⇒ equals emit: the per-situation agreement report
cargo run -p aeolus-lsp --example lsp_hover             # the LSP engine headless: diagnostics + hover renderings
cargo run -p aeolus-lsp --example lsp_completion        # the completion answer: variants, keywords, tokens
cargo run -p aeolus-lsp --example lsp_colours           # the colour swatches: the table's resolved rgba
cargo run -p aeolus-lsp --example lsp_diagnostics       # the documentDiagnostic pull (the push channel's findings, pulled)
cargo run -p aeolus-lsp --example lsp_semantic_tokens   # the semantic tokens: the legend, the decoded highlights
cargo run -p aeolus-lsp --example lsp_colour_presentations  # the picker's spellings: the table walked backwards
cargo run -p aeolus-lsp --example lsp_inlay_hints       # the inline hints: the class's CSS at its end
cargo run -p aeolus-lsp --example lsp_code_actions      # the quick fixes + the click-to-find highlights
cargo run -p aeolus-lsp --example lsp_document_highlights  # the click-to-find highlights: every occurrence whose text equals the cursor's
cargo run -p aeolus-lsp --example lsp_format            # the canonical-order formatting: one edit per out-of-order string
cargo run -p aeolus-cli --example fmt                   # the fmt command's file face: the drift report + the rewrite
cargo run -p aeolus-compile --example descendants       # the descendant-probe seam: the has-* family answers
cargo run -p aeolus-cli --example expand_descendants    # expand --has: the subtree facts the has-* layers answer from
cargo run -p aeolus-compile --example supports          # the supports seam: the @supports family answers
cargo run -p aeolus-cli --example expand_supports       # expand --supports: the capability facts the supports layers answer from
cargo run -p aeolus-compile --example preferences       # the preference families: the media-feature tier answers
cargo run -p aeolus-compile --example capabilities      # the capability tier: pointer/orientation media-feature answers
cargo run -p aeolus-cli --example expand_capabilities   # expand --pointer-fine/--portrait: the facts the capability layers answer from
cargo run -p aeolus-compile --example direction         # the direction family: ltr/rtl selector-side answers
cargo run -p aeolus-cli --example expand_direction      # expand --rtl: the direction fact the rtl/ltr layers answer from
cargo run -p aeolus-compile --example validation        # the validation states: valid/invalid/user-* element-state answers
cargo run -p aeolus-cli --example expand_validation     # expand --invalid/--user-*: the facts the validation layers answer from
cargo run -p aeolus-compile --example element_states    # the element states: target/indeterminate/autofill/read-only answers
cargo run -p aeolus-compile --example descendants       # the descendant-probe seam: the has-* family answers
cargo run -p aeolus-compile --example supports          # the supports seam: the @supports family answers
cargo run -p aeolus-compile --example preferences       # the preference families: the media-feature tier answers
cargo run -p aeolus-compile --example capabilities      # the capability tier: pointer/orientation media-feature answers
cargo run -p aeolus-compile --example widget_states     # the widget states: open/inert/default element-state answers
cargo run -p aeolus-cli --example expand_element_states # expand --target/--indeterminate/--autofill/--read-write: the facts the element-state layers answer from
cargo run -p aeolus-compile --example widget_states     # the widget states: open/inert/default element-state answers
cargo run -p aeolus-cli --example expand_widget_states  # expand --open/--inert/--default: the facts the widget-state layers answer from
cargo run -p aeolus-cli --example sort                  # the sort face: canonical order beside the miss channel
cargo run -p aeolus-codegen-gpui --example gate  # the macro surface's capability gate
cargo run -p aeolus-codegen-gpui --example caps_table  # the generated GPUI capability table (xtask gen-gpui-caps)
cargo run -p aeolus-codegen-gpui --example lower_class  # the third-party (gpui-rsx) seam
cargo run -p aeolus-macro --example frozen_program  # what a tw! expansion actually emits
cargo run -p aeolus-macro --example variants        # variants!/tw_cn!/theme!: the cva tables + the baked theme
```

<details><summary><b>All examples, by crate</b></summary>

```sh
# core (runtime lane)
cargo run -p aeolus-core --example resolve
cargo run -p aeolus-core --example plan_algebra
cargo run -p aeolus-core --example cn_merge
cargo run -p aeolus-core --example gamut  # the CSS Color 4 chroma reduction beside the clamp oracle
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
cargo run -p aeolus-compile --example custom_css  # the CSS-file form: @apply composites + functional stars
cargo run -p aeolus-compile --example dyn_cache
cargo run -p aeolus-compile --example dyn_store  # a host's own bounded store on the DynStore seam
cargo run -p aeolus-compile --example arbitrary_values
cargo run -p aeolus-compile --example composites  # the composite grammar: calc(), track lists, url(), ratios
cargo run -p aeolus-compile --example attr_variants
cargo run -p aeolus-compile --example probes
cargo run -p aeolus-compile --example plan_bridge
cargo run -p aeolus-compile --example sort_order
cargo run -p aeolus-compile --example project_build
cargo run -p aeolus-compile --example named_containers
cargo run -p aeolus-compile --example container_bounds
cargo run -p aeolus-compile --example struct_probe  # the structural seam, facts in, layers out
cargo run -p aeolus-compile --example pseudo_targets
cargo run -p aeolus-compile --example motion
cargo run -p aeolus-compile --example shadow_plan  # the plan bridge's shadow slot, the differential walk
cargo run -p aeolus-compile --example metamorphic  # the metamorphic checker: per-situation agreement report
cargo run -p aeolus-compile --example filters           # the pipeline's filter layer: the v4 var-chain forms
cargo run -p aeolus-compile --example backdrop_filters  # the backdrop layer: both properties, -webkit- beside the standard
cargo run -p aeolus-compile --example grid_flow         # the grid-flow family: the auto-flow slot end to end
cargo run -p aeolus-compile --example hints             # the wider v4 hint table: angle/time/position/family/string end to end
cargo run -p aeolus-compile --example direction         # the direction family: ltr/rtl selector-side answers
cargo run -p aeolus-compile --example validation        # the validation states: valid/invalid/user-* element-state answers
# codegen & runtime
cargo run -p aeolus-codegen --example text_emit
cargo run -p aeolus-codegen-gpui --example conformance
cargo run -p aeolus-rt --example class_cell
cargo run -p aeolus-rt --example static_walk
cargo run -p aeolus-rt --example program_overlay
cargo run -p aeolus-macro --example frozen_program  # the tw!/cn! surface: the frozen program, printed
# GPUI
cargo run -p aeolus-gpui --example ring
cargo run -p aeolus-gpui --example grad
cargo run -p aeolus-gpui --example filters
cargo run -p aeolus-gpui --example variants
cargo run -p aeolus-gpui --example slots
cargo run -p aeolus-gpui --example attr_state
cargo run -p aeolus-gpui --example containers
cargo run -p aeolus-gpui --example container_query  # the measuring wrapper: publish → walk → resize beat
cargo run -p aeolus-gpui --example decoration       # the decoration tier: colour/thickness/style on GPUI's own methods
cargo run -p aeolus-gpui --example struct           # first:/last:/odd: through the DOM-position seam
cargo run -p aeolus-gpui --example motion
cargo run -p aeolus-gpui --example placement  # grid placement + flex-basis, GPUI's own slots
cargo run -p aeolus-gpui --example shadow     # shadow-* on GPUI's own BoxShadow list
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
cargo run -p aeolus-cli --example plan            # the plan bridge: snapshot + notes + the two lanes' answer
cargo run -p aeolus-cli --example check            # the CI lint face: candidates, typed findings, the fail-on gate
cargo run -p aeolus-cli --example fmt              # the fmt file face: the drift report beside the rewrite
cargo run -p aeolus-cli --example expand_descendants  # expand --has: the has-* layers answer from the subtree facts
cargo run -p aeolus-cli --example expand_supports  # expand --supports: the @supports layers answer from the capability facts
cargo run -p aeolus-cli --example expand_capabilities  # expand --pointer-fine/--portrait: the capability layers answer from the frame facts
cargo run -p aeolus-cli --example expand_direction  # expand --rtl: the rtl/ltr layers answer from the one direction fact
cargo run -p aeolus-cli --example expand_validation  # expand --invalid/--user-*: the validation layers answer from the facts
cargo run -p aeolus-cli --example expand_element_states  # expand --target/--indeterminate/--autofill/--read-write: the element-state layers answer from the facts
cargo run -p aeolus-cli --example expand_widget_states   # expand --open/--inert/--default: the widget-state layers answer from the facts
cargo run -p aeolus-cli --example sort              # the sort face: canonical order beside the miss channel
cargo run -p aeolus-cli --example cn                # the cn face: IR-level merge, the rightmost-wins answer
# the shadcn program tables
cargo run -p aeolus-shadcn --example program_tables  # the Button programs headless (the variants! output)
cargo run -p aeolus-shadcn --example program_components  # the non-button program tables: Badge/Alert/Base beside the input/card shapes
# the P5 LSP row
cargo run -p aeolus-lsp --example lsp_hover
cargo run -p aeolus-lsp --example lsp_completion  # completion at the typing edge
cargo run -p aeolus-lsp --example lsp_colours     # the documentColor swatches
cargo run -p aeolus-lsp --example lsp_diagnostics # the documentDiagnostic pull (the push channel's findings, pulled)
cargo run -p aeolus-lsp --example lsp_semantic_tokens  # the semanticTokens legend + decoded highlights
cargo run -p aeolus-lsp --example lsp_colour_presentations  # the picker's spellings, the table walked backwards
cargo run -p aeolus-lsp --example lsp_inlay_hints  # the inlayHint answers: the CSS at the class's end
cargo run -p aeolus-lsp --example lsp_code_actions  # the codeAction quick fixes + the documentHighlight occurrences
cargo run -p aeolus-lsp --example lsp_document_highlights  # the click-to-find highlights: every occurrence whose text equals the cursor's
cargo run -p aeolus-lsp --example lsp_format  # the formatting answer: the canonical-order edits, applied
cargo run -p aeolus-lsp --bin aeolus-lsp  # the stdio server (an editor spawns this)
```

</details>

## Learn more

**Design & specs**

- [V1 architecture](docs/AEOLUS_V1_ARCHITECTURE.md) — the zero-runtime plan of record: pipeline, IR, theme tiers, performance contract
- [docs/REGISTRY.md](docs/REGISTRY.md) — the utility registry audit page (generated from the rows)
- [docs/CONFORMANCE.md](docs/CONFORMANCE.md) — the GPUI conformance page, from the emitter's own gates
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
