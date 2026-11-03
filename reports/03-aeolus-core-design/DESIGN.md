# aeolus-core design (deliverable 3)

The code IS the reference (`crates/aeolus-core/`, zero dependencies,
edition 2024, MSRV 1.88, rust-version pinned in Cargo.toml). This document
explains the shape and the why, with pointers into the source.

## 1. Grammar (parser.rs, ast.rs)

Class token grammar (v4 subset, the Shadcn v4 component surface):

```text
class       := [-] variant-list utility [opacity] [arbitrary]
variant     := named | breakpoint | container | grouped | arbitrary-variant
named       := "hover" | "focus" | "focus-visible" | "focus-within"
             | "active" | "disabled" | "dark" | "visited"
grouped     := ("group"|"peer") ["-" label] "-" named
utility     := prefix ["-" name-list]        // bg, p, rounded, shadow, …
opacity     := "/" number                     // v4 oklab color-mix
arbitrary   := "[" value "]"                  // [#0ea5e9], [13px], …
```

- Variants stack left-to-right (`md:hover:bg-*`): the last variant is the
  outermost condition; parse is right-to-left over `:` separators
  (`parse_token`, parser.rs).
- Unknown variants (`has-[>svg]:`, `data-[state=open]:`, `aria-*:`) are
  **skipped with a warning**, never fatal (`parse_class_string` returns
  `(tokens, errors)`); the `tw!` proc macro (`aeolus-macro`) turns the same
  errors into compile errors for static strings — hard contract at compile
  time, soft at runtime.
- Opacity is v4-correct: `bg-primary/90` = color-mix(in oklab, primary 90%,
  transparent) = premultiplied scale of chroma AND alpha
  (`Color::mix_with_transparent`, color.rs) — tested.

## 2. StylePlan IR (plan.rs)

Exactly the §3.1 locked shape: `base: ResolvedStyle` + ordered
`layers: Vec<ConditionalLayer { condition, style }>` + `responsive/container:
BTreeMap<Breakpoint, ResolvedStyle>` + `dark: Option<…>` + `group/peer` maps.
`ResolvedStyle` is a struct of typed `Option` slots (no dynamic maps) so
backends get exhaustive matching; every slot carries `Slot::lane()` =
Cosmetic|Geometric, mirroring Masonry's `core_property_changed` lanes, so
backends can pick the cheapest invalidation via `diff()` + `StyleDiff::needs_layout()`.
`VariantCondition::rank()` fixes the v4 cascade order
(hover < focus < focus-visible < focus-within < active < disabled < visited).

## 3. Theme registry (theme.rs)

- `TailwindTheme::from_theme_css(css)` — CSS-first: parses `@theme`,
  `@theme inline|static`, `:root`/`.dark` variable blocks; `@property`
  declarations pass through. Every theme is **seeded with the verbatim v4
  default palette** (`data/tailwind-default-palette.css`, from
  tailwindcss v4.1.17 `packages/tailwindcss/theme.css`); user declarations
  merge on top (same key → later wins).
- Typed tokens: `Color` (oklab+alpha), `Length(px)`, `FontSize{size,line_height}`,
  `BoxShadowSpec`, breakpoints. Type inference is **value-first**
  (`value_is_color` sniffs oklch/rgb/hsl/hex) so app tokens like
  `--chart-1` work without name lists; `--color-*` / `--text-*` / `--shadow-*`
  prefixes remain as fallbacks for `var()` indirection.
- `calc(var(--radius) - 2px)` is deferred until deps exist, then evaluated
  left-to-right (documented limitation for exotic expressions).
- Runtime theme change is explicit: `absorb_css` (merge + bump `version`) —
  callers drop their `PlanCache`; no CSS-variable pretending (mission §3.5).
- `shadcn_default()` / `shadcn_dark()` ship the neutral light/dark tokens
  **verbatim** from `shadcn-ui/ui @ 4aaea19 apps/v4/app/legacy-themes.css`
  (`.theme-neutral` + its `@variant dark`), including chart/sidebar tokens;
  upstream dropped `--destructive-foreground` and so do we (v4 destructive
  variants use `text-white`).

## 4. Conflict resolution (conflict.rs)

`cn(&["px-2","px-4"]) -> "px-4"`: classes are parsed to tokens, keyed by
`(important, variant-set, conflict-group)`; scanning right-to-left, a later
class in the same key evicts the earlier one; sub-edge survivors keep parents
(`pl-4` survives `px-4` is FALSE — `px-4` evicts `px-*` children; `pl-4`
after `p-4` survives because it is narrower). Group ids follow the tw_merge
semantics (272 groups audited in sections/precedents.md — including the
`text-{size}` vs `text-{color}` disambiguation), with `!` in the key and
negatives normalized. Unparseable input is returned verbatim (best-effort).
Parity is pinned by `tests/shadcn_components.rs` (merge assertions on real
Shadcn strings).

## 5. Cache & interning (cache.rs)

- `Interner`: string → id (hit 387 ns).
- `PlanCache`: keyed by `(interned id | FxHash of string, theme.version)`;
  cold resolve ~4.6 µs, warm hit ~404 ns (report 08).
- `CachedPlan` carries the `Resolution` and the theme version it was built
  against — a theme `absorb_css` invalidates by version bump, not by
  scanning.

## 6. Capabilities (caps.rs)

`BackendCapabilities { properties: PropertySupport, cascade: CascadeModel,
responsive, container_queries, dark_mode }` with
`CascadeModel::{Native{selector_kind}, StateClosures, ViewLevel, None}` —
the contract `lower()` implementations negotiate against (values from the
capability matrix, report 02). The lowering rule from the brief holds:
unsupported utility → `tracing::warn!` in debug, never a panic, never a
silent zero.

## 7. Test plan (tests/)

1. **Unit**: color round-trips, opacity premultiplication, theme parsing
   (shadcn tokens incl. percentage-lightness oklch), breakpoint tokens,
   spacing default (21 lib tests).
2. **Component parity**: the 8 v1 Shadcn class tables must parse, land their
   essential slots, merge with user classes last-wins
   (`tests/shadcn_components.rs`, 14 tests + aeolus-shadcn's 6).
3. **cn() parity oracle**: tw_merge behavior is the oracle (group table
   documented in sections/precedents.md); core merge tests mirror its cases.
4. **Golden corpus (CI parity, follow-up)**: ironframe's
   `tests/fixtures/complex_parity/` corpus (real class soup vs genuine
   tailwindcss v4.1.17 output) — `ironframe::core::parse()` itself is a stub
   (audited), so the *fixture files* are the oracle, not the crate API.
   Wire as: for each fixture class string, assert our plan's CSS-equivalent
   (a debug CSS writer) matches the golden output on the supported subset.
