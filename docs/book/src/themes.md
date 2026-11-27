# Themes

The theme system is CSS-first: the same `app.css` a web Tailwind v4
project uses themes a native app unchanged. `ThemeSource::from_css`
parses the `@theme`/`:root`/`.dark` surface over the audited v0.x
reader, and `TokenTable` resolves it into three tiers — pay only for
the dynamism you use:

| Tier | When | Value form | Runtime cost |
| ---- | ---- | ---------- | ------------ |
| T0 static | theme baked at expansion | `Val::Rgba` literals | 0 |
| T1 light/dark baked | same, with a `.dark` block | `Val::ModePair` const pair | 1 index by the mode |
| T2 runtime | user-editable themes, live swaps | `Val::Tok { id, alpha }` | 1 table load + 1 multiply |

The `/opacity` modifier is the premultiplied identity: mixing with
`transparent` in a premultiplied space leaves the colour channels
unchanged and scales alpha — one multiply, pinned against the audited
oracle and the golden corpus. Out-of-gamut oklch colours are
gamut-mapped once at bake time (the hue-true chroma reduction), so
every frozen `Val::Rgba` is gamut-legal.

## Where each tier is produced

The macros take an optional `theme: "path",` prefix argument: the
sheet is read at expansion relative to the calling crate's
`CARGO_MANIFEST_DIR`, baked through `ThemeSource::from_css` +
`TokenTable::baked`, and the expansion carries an `include_str!` rebuild
anchor so Cargo re-runs rustc when the CSS changes. The no-arg form
keeps the embedded shadcn bake (and the macro-baked program is
byte-equal to `compile_theme` under the same CSS — differentially
pinned). `theme!` stays the T2 runtime handle, and the
`aeolus-gpui-component` bridge's `AeolusTheme` carries the hot-swap
(`absorb_css`) for hosts that re-theme live.

Registration metadata rides the same file: `@source` statements (the
inline/walked/pruned triple the CLI's `check` consults) and the
custom-authoring statements (`@custom-variant`, `@utility`) walk onto
the typed `Customs` surface through `customs_from_css`.

## Coming from gpui-rsx

`import_rsx` walks an upstream `rsx.toml` onto the CSS surface — the
section set, the value forms and the failure modes mirrored line for
line — and reports which spacing scale the sheet asked for. The
[migration chapter](migration.md) is the whole story.
