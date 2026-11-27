# Migrating from `gpui-rsx` to Aeolus

This guide moves a gpui-rsx app — its `rsx.toml` theme file, its class
strings and its dynamic classes — onto Aeolus's V1 pipeline. It is the
architecture plan's migration deliverable
([`docs/AEOLUS_V1_ARCHITECTURE.md`](AEOLUS_V1_ARCHITECTURE.md), §5
"Integrating with `gpui-rsx`": the *"migration table for behavioural
differences"* line, and the P5 phase's *"migration guide"* exit
criterion). Every claim below is pinned by a test in the tree; where
the two compilers genuinely differ, the difference is stated, never
papered over — the same honesty contract the compiler applies to class
strings applies to this document.

The one-sentence version: **hand your `rsx.toml` to
[`import_rsx`](../crates/aeolus-theme/src/rsx.rs), keep your class
vocabulary, and gain the v4 variant surface, the v4 CSS-first theme
and one compiler for static and dynamic strings** — after reading the
scale row below, which is the one difference that changes pixels.

## The behavioural-differences table

| Behaviour                       | gpui-rsx                                                                                          | Aeolus                                                                                                                                 | What to do on migration                                                                                                                                                                                                                     |
| ------------------------------- | ------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **The spacing scale**           | Numeric classes are **px by default** (`gap-4` = 4 px); `spacing-scale = "tailwind"` switches to 4 px per unit | The v4 scale always: 1 unit = 0.25 rem, emitted as `rem` and honouring the window's rem size (16 px root ⇒ `gap-4` = 16 px)           | With `spacing-scale = "tailwind"` the two agree (4 px/unit at a 16 px root). From the default px scale the classes change size 4× — either accept the v4 scale (the recommended path: your sheet's `[spacing]` lengths and the migration's `[text-size]` px values are unaffected) or rewrite the numeric classes. There is no 1-px-unit mode in Aeolus; the compile crate's `Spacing::Px` is a DIFFERENT knob (the v4 rem scale baked to absolute px for px-only worlds), and the importer's `RsxSpacingScale` report names which scale a sheet asked for |
| **`dark:` / `md:` … breakpoints** | Unsupported — a warning/error at expansion                                                                       | Live: `dark:` indexes the theme's T1 light/dark pairs (`Env.dark`, branch-free); `sm:`…`2xl:` and the `max-*:` complements are one `Env.viewport_rem` compare | Add an `app.css` with a `.dark` block (the same file a web v4 project uses) and the classes start working — no code change                                                                                                                           |
| **The variant surface**         | `hover:` `focus:` `focus-visible:` `active:` `group-*`                                                            | The full typed family: state refinements, named `group-*/peer-*` (incl. `*-has-*`), `data-*`/`aria-*`, `has-*`, `supports-*`, the media/preference families, structural selectors, pseudo-element render targets — each a typed `Cond`, each with a CLI `expand --…` probe face | Existing variant spellings keep working; new ones compile when the target can express them or produce a typed refusal with a reason — nothing is dropped silently                                                                                     |
| **Where the theme lives**       | `rsx.toml` sections (`colors`, `radius`, `spacing`, `text-size`, `line-height`, `opacity`, `shadow`, `options`)   | `@theme`/`:root`/`.dark` CSS — the same `app.css` a web Tailwind v4 project uses, read by the audited v4 reader                         | `aeolus_theme::import_rsx` walks the file onto that surface (below); keep the emitted sheet in version control as `themes/app.css` and iterate there afterwards                                                                                       |
| **Theme token values**          | Static hex/length strings or Rust expressions (`{ expr = "…" }`) resolved by the GPUI runtime                     | Static CSS values in three tiers (T0 baked literals, T1 dark pairs, T2 runtime `Tok` slots) — expressions have no CSS form              | Static strings import 1:1; expression-valued tokens ride the importer's typed note channel — write the value as CSS, or keep the runtime resolution through the `aeolus-gpui-component` bridge (the `gpui-component` preset's own seam)             |
| **Named opacities**             | `[opacity] muted = "60%"` (a named token)                                                                        | No opacity theme namespace — v4 and Aeolus both carry opacity on the per-class `/N` modifier                                             | The import notes each named entry; use the value as the modifier (`text-muted/60`)                                                                                                                                                                   |
| **Shadows**                     | `[shadow] card = { expr = "vec![…BoxShadow…]" }` — Rust only, by design                                            | `--shadow-*` CSS layer lists (v4 multi-layer form), lowered to GPUI's own `BoxShadow` bands                                             | Rewrite the expression as the CSS list (`--shadow-card: 0 1px 2px rgb(0 0 0 / 20%);`) — the GPUI walk applies it natively                                                                                                                             |
| **Dynamic class strings**       | A smaller runtime matcher handles `class={expr}` — a capability split from the static path                        | ONE compiler for both: the dynamic lane is the same pipeline behind `DynCache` + `ClassCell` (bounded, content-keyed, `!Sync` UI-thread cells) | No code change; dynamic strings gain the full v4 surface instead of the smaller matcher's                                                                                                                                                            |
| **Unsupported classes**         | Warning/error at expansion                                                                                        | A spanned, coded, helped diagnostic — in the macro, the CLI (`aeolus check`), the LSP and every backend walk                             | Nothing to do; the refusal is the contract. `aeolus check src/` is the CI face, `aeolus-lsp` the editor face                                                                                                                                          |
| **Colour-class vocabulary**     | Token names resolve directly (`bg-demo-accent` if the token exists)                                               | The colour suffixes are the registry's closed table (the v4 palette + the shadcn semantic tokens): a sheet OVERRIDES those names' values; a brand-new NAME lowers through the bare-var arbitrary form (`bg-(--color-demo-accent)`) | Override the closed names where the spelling already exists; otherwise reach the imported token through `--color-{name}` in the bare-var form — both paths are pinned in the import's wire-up battery (`crates/aeolus-compile/tests/rsx_import.rs`)   |

## Migrating the theme: `rsx.toml` → `app.css`

The import seam is one function over the upstream grammar, mirrored
line for line (the section set, the quoted-string/inline-expression
value forms, the quote-aware `#` comment strip, per-section value
validation):

```rust,ignore
use aeolus_theme::{TokenTable, import_rsx};

let sheet = std::fs::read_to_string("rsx.toml")?;
let import = import_rsx(&sheet)?;

// 1. The emitted `@theme` sheet — check it in as themes/app.css.
std::fs::write("themes/app.css", import.css())?;

// 2. The config bridge's spacing half: which scale the sheet asked for.
assert_eq!(import.spacing_scale().is_px(), false);

// 3. The honest channel: what could not map, and why.
for note in import.notes() {
    eprintln!("{note}"); // rsx.toml:{line}: `{name}` — {reason}
}

// 4. The round-trip: the imported sheet bakes and resolves.
let source = import.theme_source()?;
let table = TokenTable::baked(&source)?;   // or TokenTable::runtime for theme swaps
```

What maps onto the sheet, one namespace per section:

| `rsx.toml` section | Emitted namespace                      | Notes |
| ------------------ | -------------------------------------- | ----- |
| `[colors]`         | `--color-{name}`                       | `#rgb`/`#rrggbb`/`#rrggbbaa`, verbatim; the reader's oklab machinery gamut-maps on the bake |
| `[radius]`         | `--radius-{name}`                      | `Npx`/`Nrem` lengths, verbatim |
| `[spacing]`        | `--spacing-{name}`                     | Rides the sheet (the v4 functional namespace) AND the note channel — Aeolus's spacing classes consume the numeric scale |
| `[text-size]`      | `--text-{name}`                        | |
| `[line-height]`    | `--text-{name}--line-height` when the name matches a `[text-size]` entry (the reader's own pairing form), else `--leading-{name}` | The gpui-rsx demo sheet's `body` pair lands as one paired token |
| `[opacity]`        | — (note channel)                       | No v4 namespace; the value is the `/N` modifier to use |
| `[shadow]`         | — (note channel)                       | Rust expressions only upstream; rewrite as a `--shadow-{name}` layer list |
| `[options]`        | — (report + note channel)              | `spacing-scale` rides `RsxImport::spacing_scale()`; `preset`/`gpui-path`/`context`/`dynamic-runtime` ride notes |

Errors are typed and line-numbered, one variant per upstream failure
mode (`unknown section`, `invalid key`, `duplicate key`, `bad color`,
`bad length`, `bad opacity`, `shadow needs expression`, …), Displayed
in the upstream messages' own `rsx.toml:{line}: …` shape — a sheet
gpui-rsx refuses for one reason refuses here for the same reason.

Run the whole story headless:

```sh
cargo run -p aeolus-theme --example rsx_import
```

## Migrating the class strings

1. **Keep the vocabulary.** Utility spellings are the v4 ones
   (`p-4`, `rounded-md`, `bg-primary`); gpui-rsx's Tailwind-inspired
   subset maps onto the same registry rows. The scale row above is the
   one place spellings stay but pixels change.
2. **Hand the theme to every surface the backend reads.** The frozen
   program path bakes at expansion (`tw_static!`/`theme:`), the GPUI
   walk reads the runtime tier through the frame `Env`, and the CLI
   tools accept the same sheet (`aeolus check src/ --theme app.css`,
   `aeolus build src/ --theme app.css`).
3. **Turn the diagnostics on in CI.** `aeolus check src/ --fail-on
   unknown` is the lint gate; `aeolus fmt --check` gates the canonical
   class order; `aeolus audit --target gpui` tiers every scanned class
   against the backend's own capability table.
4. **gpui-component hosts keep their palette.** The
   `gpui-component` preset's runtime colour expressions are exactly
   what the `aeolus-gpui-component` bridge projects: one
   `AeolusTheme` handle carries the Tailwind table AND the
   `ActiveTheme` palette, and `absorb_css` is the hot-swap.

## What has no automatic answer

- **The 1-px unit scale.** Deliberately absent — a silent 4× bake
  would be the kind of quiet surprise the project refuses. The
  importer's report names the scale; the rewrite decision is yours.
- **Rust-expression tokens.** A CSS sheet cannot carry a Rust
  expression; the notes tell you exactly which entries need a static
  value (or the bridge's runtime resolution).
- **`preset`-seeded palettes.** The preset's colours resolve through
  Kit's component theme at render time — a live dependency, imported
  only as the bridge's projection. Bake static hex values into the
  sheet if you want them in the compile-time tiers.
