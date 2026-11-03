# Shadcn v1 (deliverable 6)

Code: `crates/aeolus-shadcn/` (class tables + Xilem/Floem thin wrappers,
integrity tests in `tests/classes.rs`). Tests: **all green** in the
workspace run.

## Component set (v1, per briefing)

Button, Input, Label, Card, Badge, Separator, Skeleton, Alert.

## Source of truth — verbatim v4

`shadcn-ui/ui @ 4aaea19d9cff3e0908e12a8c1d504b8c5ec34610` (2026-11-02),
`apps/v4/registry/new-york-v4/ui/{button,input,label,card,badge,separator,
skeleton,alert}.tsx`. The strings in `classes.rs` are byte-verbatim from
those files (spacing-normalized only). The earlier extraction from the
RustForWeb/leptos-shadcn ports was **v3-era** and was replaced: v4 markers
(`shadow-xs`, `focus-visible:ring-[3px]`, `dark:bg-input/30`,
`text-white` on destructive, data-orientation separator variants) are all
present.

Integrity guarantees (tests):
- every shipped string **parses** (unknown variants like `has-[>svg]:` now
  warn-and-skip instead of failing the string — parser contract change made
  during this work);
- Button default resolves bg/text/padding/h-9/radius + hover & disabled
  layers (`disabled:opacity-50` parsed into the disabled layer);
- Outline: `border` + `shadow-xs`, `dark:border-input` lands in the dark
  layer (light mode intentionally has no border-color — CSS currentColor);
- Destructive: `bg-destructive` + `dark:bg-destructive/60` layer, `text-white`;
- Skeleton = `bg-accent` (resolves to the theme accent token);
- user classes appended LAST win (`cn` + plan merge, `bg-red-500` beats
  `bg-primary`).

## Wrappers

- **Xilem** (`xilem.rs`): `button(resolver, id, text, variant, size, class)`,
  `badge`, `label`, `card`, `card_header`, `card_title`, `card_description`,
  `card_footer`, `alert` — each is a shadcn-styled box around a
  `xilem_masonry::view::label`; layout utilities
  (`items-center justify-center`) map to the centered `StyledWidget`.
- **Floem** (`floem.rs`): same surface; layout utilities flow through the
  plan into floem's real flex engine; attach via `Decorators::style`.
- API contract preserved: `Button::…-style` enums + a trailing `class: &str`
  that always wins.

## Honest capability table (what each backend consumes)

| utility family | aeolus-xilem | aeolus-floem |
|---|---|---|
| colors/padding/radius/shadow/border/sizes/gap/font/leading | ✅ | ✅ |
| hover/focus-visible/disabled/dark | ✅ (Selector flags) | ✅ (selector maps) |
| ring/outline/transition-all/cursor/select-none | ⚠️ dropped (reported) | ⚠️ (ring) dropped |
| `[&_svg]`, `[a&]`, `has-[…]`, `data-[…]`, `aria-invalid`, `[.border-b]` | ⚠️ dropped (reported) | ⚠️ dropped (reported) |
| flex layout utilities | ⚠️ wrapper-level (centered box) | ✅ real flex |

`aria-invalid:*` and `data-*` variants are skipped by the parser with
`Warning::Unparseable` — adding them is a parser extension (v1 scope
decision: the Shadcn set's interactive states are hover/focus/disabled/dark,
which are fully modeled).
