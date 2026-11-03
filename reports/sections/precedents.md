# AEOLUS Section Audit — Precedent Crates (14 clones)

- **Method:** read-only audit. For each clone: `git rev-parse HEAD`, LICENSE file + Cargo.toml `license`, crate name/version. No code modified; no new clones. **MANIFEST.txt does not exist** under `/home/z/my-project/aeolus/` (checked `reports/`, root) — nothing to fix; if a MANIFEST is generated later, the HEADs below are the hashes to record.
- **Standing question per repo:** REUSE (vendor) / BORROW AN IDEA / PARITY-ORACLE ONLY, and the license constraint. Rule of thumb: MIT/Apache/ISC → vendoring OK with attribution; **MPL-2.0 → file-level copyleft on any vendored file**.

## 0. Clone inventory (HEAD · license · crate)

| dir | HEAD (date) | license (file + Cargo.toml) | crate(s) |
|---|---|---|---|
| tailwind-rs | `511a1904…` (2025-08-26) | **MPL-2.0** (`License.md`; `license = "MPL-2.0"` in every member) | oovm/tailwind-rs workspace |
| tailwind-fuse | `5bed8225…` (2025-01-19) | MIT (`fuse/Cargo.toml:8`) | tailwind_fuse 0.3.2, tailwind_fuse_macro 0.3.0 |
| tw_merge | `4f5a516e…` (2026-10-30) | MIT (`tw_merge/LICENSE`; `crates/tw_merge/tw_merge/Cargo.toml`) | tw_merge 0.1.22 (monorepo root crate: rust-ui-dioxus 0.2.0) |
| twill | `e529d3e1…` (2026-10-27) | MIT (`LICENSE`; `Cargo.toml:17`) | twill 0.3.0 (+ twill-core, -egui, -iced, -slint) |
| encre-css | `ae2870ac…` (2026-11-01) | MIT (`LICENSE`) | encre-css 0.21.1 |
| crepuscularity | `eb60f0f9…` (2026-10-31) | **ISC** (`LICENSE`; core `license = "ISC"`) | crepuscularity-core 0.5.0 (23-crate workspace) |
| icss | `b3d21d6f…` (2026-08-24) | MIT (`LICENSE`; `Cargo.toml:11`) | icss 0.1.1 (rustc ≥ 1.88) |
| ironframe | `b7c151a1…` (2026-02-16) | **MIT OR Apache-2.0** (`LICENSE`; `Cargo.toml:6`) | ironframe 0.4.0 |
| gpui-rsx | `51837497…` (2026-10-31) | MIT (`LICENSE`; `Cargo.toml:10`) | gpui-rsx 0.10.2 (rustc ≥ 1.88) |
| floem-tailwind | `3b165092…` (2025-12-28) | MIT (`Cargo.toml:7`; no LICENSE file at top) | floem-tailwind 0.1.0 |
| floem-shadcn | `ae2f61ba…` (2026-01-28) | MIT (`Cargo.toml:7`; no LICENSE file at top) | floem-shadcn 0.1.0 |
| shadcn-ui | `21a6cf5d…` (2026-02-02) | MIT (`LICENSE.md`) | Rust shadcn/ui port (RustForWeb), workspace 0.0.1 — **README states "This project is unmaintained." (README.md:2)** |
| shadcn-official | `4aaea19d…` (2026-11-02) | MIT (`LICENSE.md`) | shadcn/ui JS monorepo (apps/v4, packages/*) |
| leptos-shadcn | `c2fbe3ee…` (2026-01-10) | MIT (`LICENSE`; `Cargo.toml:99,101`) | leptos-shadcn-ui workspace 0.9.0 (CloudShuttle) |

All 14 clones are present and non-empty; all are git repos with the HEADs above (verified via `git -C <dir> rev-parse HEAD`).

**Naming trap:** `shadcn-ui/` is *not* the ui.shadcn.com monorepo — it is the RustForWeb port. The official monorepo (with `apps/v4`) lives in `shadcn-official/`.

---

## 1. tailwind-rs (oovm) — `tailwind-ast` grammar, MPL-2.0

**Workspace** (`Cargo.toml:3` members `projects/*`, default members = tailwind-to-css/cli/rs):

| member | crate name | version | role |
|---|---|---|---|
| `projects/tailwind-ast` | tailwind-ast | 0.4.1 | nom grammar for class strings (`Cargo.toml:2-9`) |
| `projects/tailwind-to-css` | tailwind-css | 0.12.7 | v3-era utility resolver (243 source files) |
| `projects/tailwind-cli` | tailwind | 0.1.0 | CLI |
| `projects/tailwind-error` | tailwind-error | 1.3.3 | error types |
| `projects/tailwind-macro` | macrowind | 0.1.1 | `tailwindscss!`-style macro |
| `projects/tailwind-rs` | tailwind-rs | 0.2.0 | framework glue |
| `projects/tailwind-show` | tailwind-show-case | 1.0.0 | demo |

**all seven are `license = "MPL-2.0"`** and the root `License.md` is the full MPL text.

**What tailwind-ast parses** — a nom-based, zero-copy (`&str` slices, `Copy`-friendly AST) *shape* grammar, no semantics:
- Entry: `parse_tailwind(input) -> Result<Vec<AstStyle>, Err<Error<&str>>>` (`projects/tailwind-ast/src/ast/mod.rs:23-35`); expands `(grouped)` styles recursively.
- `AstStyle { important, negative, variants, elements, arbitrary }` (`src/ast/mod.rs:59-70`); groups carry a trailing `!` (`src/ast/parse.rs:8-12`).
- Variants `(not-)?name(-name)*(:|::)` (`src/ast/parse.rs:100-121`): names are `alphanumeric1` split on `-` — **arbitrary variants `data-[state=open]:`, `[&>*]:` and `/named` group variants are NOT supported**; pseudo-detection via hardcoded list (`parse.rs:123-137`).
- Arbitrary values `-[...]` (`parse.rs:140-148`) with escape-aware `take_until_unbalanced` (`parse.rs:159-212`).
- **No `/opacity` modifier in the grammar at all** (`rg "'/'" src/` → 0 matches).
- Errors: raw `nom::Err<Error<&str>>` — position-only, no spans/diagnostics.

**tailwind-css (0.12.7)** is the v3-era resolver: `TailwindInstruction::get_instance` matches element slices to `TailwindInstance` impls across 243 files (`projects/tailwind-to-css/src/systems/instruction/resolver.rs:9-60+`). Coverage is broad but v3-shaped (opacity module at `src/systems/effects/opacity/`), not v4 `@theme`/`color-mix` semantics.

**Verdict: do NOT reuse; do not even vendor.** MPL-2.0 makes any copied file copyleft-contaminated. Parser coverage is *narrower* than ours (no arbitrary/data variants, no opacity modifier, no v4 prefixes like `@`), so it is not worth the license cost. BORROW the idea of expanding `(a b)` groups at parse time (we already do) and nothing else.

## 2. tailwind-fuse (gaucho-labs) — merge engine, MIT

- `tailwind_fuse` 0.3.2 (edition 2021, nom 7), MIT (`fuse/Cargo.toml:8`); derive crates `variant-macro` 0.3.0 (MIT): `#[derive(TwClass)]` / `#[derive(TwVariant)]` with `#[tw(class = "...")]` attributes (`variant-macro/src/lib.rs:7-13`).
- API: traits `TailwindFuse`, `IntoTailwindClass`, structs `TailwindMerge`/`TailwindJoin` (`fuse/src/lib.rs:276-291,390-399`); `tw_merge!(...)` macro + `pub fn tw_merge(class: impl AsRef<str>) -> String` (`fuse/src/core/merge/mod.rs:27-35`); slice forms `tw_merge_slice(&[&str])` and `tw_merge_options(class, MergeOptions)`.
- `MergeOptions { prefix: &str, separator: &str }` (default `""`/`":"`), settable globally once via `set_merge_options` + `OnceLock` (`fuse/src/core/merge/config.rs:7-50`).
- **It is a Rust port of tailwind-merge**: `get_collision_id.rs` (984 lines) maps element slices → collision group with validator fns (`is_t_shirt_size`, `parse_fraction`, `is_arbitrary_len` — `fuse/src/core/merge/validators.rs:9-40`); `get_collisions.rs` holds the parent→children conflict table; `MergeOptions { prefix, separator }` (`config.rs:7-22`).
- History note: `tw_merge` (§3) is the same lineage forked and extended to v4 (identical file names, +858 lines in `get_collision_id.rs`).

**Verdict: superseded by tw_merge; ORACLE ONLY for regression diffs** (its tests in `fuse/tests/` are the v3-flavored subset of tw_merge's).

## 3. tw_merge (rust-ui/ui) — `cn()` parity oracle, MIT ★

Crate: `tw_merge` 0.1.22, edition 2024, MIT, nom 7, no `rust-version` declared (`crates/tw_merge/tw_merge/Cargo.toml:2-13`; edition 2024 ⇒ needs rustc ≥ 1.85 in practice).

**Parser** (`src/ast/parser.rs`): zero-copy `AstStyle { source, important, negative, variants, elements, arbitrary }` (`src/ast/mod.rs:15-27`). v3 *and* v4 important: optional `!` before elements **and** after them (`parser.rs:27-37`). Variants are `Normal` (incl. `*`, `@lg`, `group-hover/dropdown`), `DataAttribute` (`data-[size=large]`, `supports-[...]`), `ArbitraryAttribute` (`[&>*]`) (`parser.rs:85-126`). Failure mode: any class that does not fully parse is kept verbatim as `Err(&str)` and passed through (`parser.rs:16-26`, `tw_merge_override.rs:22-26`).

**Merge algorithm — per-group scan, NOT a trie** (`src/core/merge/tw_merge_override.rs:9-87`):
1. Parse each whitespace-separated class (`parse_tailwind`, `parser.rs:16-26`).
2. Iterate **in reverse**; compute `collision_id` = `get_collision_id(elements, arbitrary)` (`get_collision_id.rs:5`, a ~1842-line match with validators); on error keep class as-is.
3. Insert `Collision { important, variants, collision_id }` into a `HashSet` (`:53-60`); a later (right-most) class wins by the reverse iteration; earlier same-collision classes are dropped.
4. Also insert every parent group's children from `get_collisions(collision_id)` (`get_collisions.rs:1-102`, 35 mappings; e.g. `"inset" → [inset-x, inset-y, start, end, top, right, bottom, left]`) so `inset-1` removes `hover:left-1`'s axis siblings only where the table says so (`tests/group_conflicts.rs:141`).
5. Output = kept classes joined with a **single space** (`tw_merge_override.rs:79-87`); `tw_join!` trims parts, skips empties and `None` (`src/core/tw_join.rs:3-24`). There is no `classListToString`; the Rust equivalent is the final `.join(" ")`. Separator config (`MergeOptions.separator`, default `":"` — `merge/config.rs`) applies to *parsing variants*, not to output joining.

**Behaviors to match exactly (test-cited):**
- Variant-aware: only same-variant classes merge — `hover:overflow-x-hidden` survives `overflow-x-scroll` (`tests/group_conflicts.rs:84-90`); variants are compared as an ordered `Vec<&str>` inside the `Collision` key (`tw_merge_override.rs:50-60,89-94`), so `dark:hover:` ≠ `hover:dark:` and `hover:[&>*]:` ≠ `[&>*]:hover:` (`tests/merge.rs:296-299`).
- `!important` is part of the collision key: `!font-medium !font-bold` → `!font-bold`, and `!font-bold` never evicts plain `font-thin` (`tests/merge.rs:251-255`).
- Negative: `negative` is **not** in the `Collision` key (`tw_merge_override.rs:89-94`) — `-inset-x-px` and `!right-2` merge as peers (`tests/merge.rs:254`); effectively negatives compete in the same group.
- Arbitrary values: `bg-[#f5f5f5]` conflicts with `bg-orange-300` (`tests/merge.rs:23`); `hover:m-[2px]` vs `hover:m-[length:var(--c)]` → typed arbitrary label (`length:`) wins as a *different* group (`tests/merge.rs:100-106`); `[color:blue]`'s label before `:` is used as collision id (`tw_merge_override.rs:96-104`).
- Unparseable/unknown classes are preserved verbatim (pass-through guarantee above).

**Public API:** `merge_classes(class: impl AsRef<str>) -> String` (`src/core/merge/mod.rs:32-34`), `tw_merge_slice(&[&str])`, `tw_merge_options(class, MergeOptions)`, `tw_merge_slice_options` (`mod.rs:41-84`), extensible via `CollisionIdFn`/`GetCollisionsFn` args of `tw_merge_override` (`tw_merge_override.rs:9-14`). `AsTailwindClass` is implemented for `&str/String/Cow/Rc/Arc/Box/Option<T>` (`src/core/mod.rs:16-92`).

**Test inventory (oracle fixtures):** `tests/merge.rs` 425 lines / 25 tests (conflicts, modifiers, arbitrary typed values, important, data-attrs, arbitrary variants), `tests/group_conflicts.rs` 231 lines / 21 tests (axis merging: inset/padding/margin/border/rounded families), `tests/override.rs` 48 lines / 2 tests (custom `collision_id_fn` + custom prefix `tw-` with `|` separator, `:41-43`), `tests/variant_macro.rs` 98 lines; inline: `src/ast/parser_test.rs` 31 tests, `src/core/merge/get_collision_id.rs` 44 tests (group membership + validators incl. `is_valid_length("--my-0")` double-negative rejection, `:1111`).

### Conflict-group table (complete — 272 unique ids in `get_collision_id.rs`, 359 `Ok(...)` arms; parent→child table in `get_collisions.rs`)

Representative prefixes per id (source of truth: `get_collision_id.rs`; `x{a,b}` = both `x-a` and `x-b` exist):

| family | group ids |
|---|---|
| layout/position | `position`(static/fixed/absolute/relative/sticky), `inset`, `inset-x`, `inset-y`, `top/right/bottom/left`, `z-index`, `order`, `columns`, `container`, `aspect`, `box-sizing`, `break-after/before/inside`, `object-fit`, `object-position`, `overflow`, `overflow-wrap`, `overscroll-behavior`, `isolation`, `clear`, `visibility`, `pointer-events`, `resize`, `user-select`, `color-scheme`, `forced-color-adjust`, `screen-readers`(sr-only), `custom-bem`, `not-prose`, `prose`, `group`, `peer`, `@container` — note: **no `float` group** (`float-left` etc. are unknown ids and pass through unmerged) |
| flex/grid | `flex`, `flex-basis`, `flex-direction`, `flex-grow`, `flex-shrink`, `flex-wrap`, `gap`, `gap-x`, `gap-y`, `grid-template-columns`, `grid-template-rows`, `grid-auto-flow`, `auto-cols`, `auto-rows`, `col-start`, `col-end`, `col-start-end`, `row-start`, `row-end`, `row-start-end`, `justify-content`, `justify-items`, `justify-self`, `align-content`, `align-items`, `align-self`, `place-content`, `place-items`, `place-self`, `content` |
| spacing/sizing | `padding`, `padding-{x,y,s,e,t,r,b,l}`, `margin`, `margin-{x,y,s,e,t,r,b,l}`, `width`, `height`, `size`, `min-width`, `min-height`, `max-width`, `max-height` |
| typography | `font-family`, `font-size`, `font-weight`, `font-style`, `font-smoothing`, `font-stretch`, `letter-spacing`, `line-height`, `text-align`, `text-color`(any other `text-*`), `text-decoration`, `text-decoration-color`, `text-decoration-style`, `text-decoration-thickness`, `text-transform`, `text-overflow`, `text-wrap`, `text-indent`, `text-underline-offset`, `text-shadow`, `text-shadow-color`, `whitespace`, `hyphens`, `vertical-align`, `list-style-image`, `list-style-position`, `list-style-type` |
| color/background | `background-color`(all other `bg-*`), `background-image`, `background-clip`, `background-origin`→(part of `background-position` family: `background-position`, `background-repeat`, `background-size`, `background-attachment`, `background-blend-mode`), `from`, `via`, `to` (gradient stops) |
| borders/rings | `border-w`, `border-w-{x,y,s,e,t,r,b,l}`, `border-color`, `border-color-{x,y,s,e,t,r,b,l}`, `border-collapse`, `divide-x`, `divide-x-reverse`, `divide-y`, `divide-y-reverse`, `divide-style`, `divide-color`, `ring-width`, `ring-color`, `ring-offset-width`, `ring-offset-color`, `inset-ring`, `inset-ring-color`, `outline-width`, `outline-color`, `outline-offset`, `rounded`, `rounded-{t,r,b,l,s,e,ss,se,ee,es}` |
| effects/shadows | `box-shadow`, `box-shadow-color`, `inset-shadow`, `inset-shadow-color`, `opacity`, `mix-blend-mode`, `blur`, `brightness`, `contrast`, `grayscale`, `hue-rotate`, `invert`, `saturate`, `sepia`, `drop-shadow`, `drop-shadow-color`, `backdrop-blur/brightness/contrast/grayscale/hue-rotate/invert/opacity/saturate/sepia`, `fill`, `stroke-width`, `caret-color`, `accent-color` |
| transforms/animation | `transform`, `transform-3d`, `transform-origin`, `translate`, `translate-x`, `translate-y`, `translate-z`, `scale`, `scale-x`, `scale-y`, `scale-z`, `rotate`, `rotate-x`, `rotate-y`, `rotate-z`, `skew`, `skew-x`, `skew-y`, `perspective`, `perspective-origin`, `transition-property`, `transition-behavior`, `transition-duration`, `transition-delay`, `transition-timing-function`, `animate`, `animate-in-out`, `animate-opacity`, `animate-rotate`, `animate-scale`, `animate-translate-x`, `animate-translate-y` |
| v4 masks/tables/misc | `mask-image`, `mask-linear-{t,r,b,l}`, `mask-radial`, `mask-conic`, `mask-clip`, `mask-composite`, `mask-origin`, `mask-position`, `mask-repeat`, `mask-size`, `mask-type`, `table-layout`, `border-spacing`, `border-spacing-x`, `border-spacing-y`, `scroll-margin`, `scroll-padding`, `scroll-snap-align`, `scroll-snap-stop`, `scroll-behavior`, `touch`, `touch-x`, `touch-y`, `touch-pz`, `line-clamp`, `field-sizing`, `appearance`, `will-change`, `cursor`, `fvn-normal`, `fvn-ordinal`, `fvn-slashed-zero`, `fvn-figure`, `fvn-spacing`, `fvn-fraction` |

Cross-cutting removals live in `get_collisions.rs:1-102` (35 entries): `overflow→[overflow-x,overflow-y]`, `inset→[inset-x,inset-y,start,end,top,right,bottom,left]`, `inset-x→[right,left]`, `inset-y→[top,bottom]`, `flex→[basis,grow,shrink]`, `gap→[gap-x,gap-y]`, `padding→[padding-x,padding-y,padding-start,padding-end,padding-top,padding-right,padding-bottom,padding-left]`, `padding-x→[padding-right,padding-left]`, `padding-y→[padding-top,padding-bottom]`, same shape for `margin*` and `scroll-m*`/`scroll-p*`, `size→[width,height]`, `font-size→[line-height]`, `line-clamp→[display,overflow]`, `rounded→[14 corner axes]`, `border-w→[border-w-{s,e,t,r,b,l}]`, `border-color→[4 sides]`, `border-spacing→[x,y]`, `touch→[touch-x,touch-y,touch-pz]`, `fvn-*` mutuals.

**Verdict: PARITY-ORACLE ONLY for `cn()`** — vendoring is tempting (MIT) but the algorithm is a linear reverse scan + giant match; our conflict-trie supersedes it. Take: (a) its **test corpus** (`tests/merge.rs`, `tests/group_conflicts.rs`, `tests/override.rs`, `tests/variant_macro.rs`) as fixture data for our merge tests; (b) the 272-id table above as a checklist against our conflict-trie; (c) its parser edge semantics (v4 `!` suffix, `/` in variants, `@` container variants, pass-through on parse failure) as oracle behaviors.

## 4. twill (FerrisMind) — typed nested state/responsive IR, MIT

`twill-core` 0.3.0, MIT, rustc ≥ 1.93 (`Cargo.toml:11-17`), `#![forbid(unsafe_code)]` (`twill-core/src/lib.rs:1`). Backend-agnostic core + backends for egui/iced/slint.

**IR shape** — method-only, no class strings anywhere:
- `Style` is a flat struct of ~60 `Option<T>` typed fields (`src/style/style.rs:40-121`): `display`, `position`, `padding: Option<Padding>`, `text_color: Option<TextColor>`, … plus two nested containers: `states: Option<Box<StateStyles>>` (`style.rs:119`) and `responsive: Option<BTreeMap<Breakpoint, Style>>` (`style.rs:120`).
- `StateStyles { hover, focus, focus_visible, active, disabled, selected, checked, open, … }` each `Option<Style>` (`src/style/state.rs:168-183`); typed `DataState`/`DataAttr` enums (`state.rs:5-19,37-86`) with `state=`/`name=value` selector rendering.
- `Breakpoint { Sm..S2xl }` with rem values (`src/tokens/spacing.rs:337-343`; `Sm → 40.0` rem — v4-consistent).
- Nested builder: `Style::card().padding(..).hover(|s| s.opacity(0.9)).data_attr(DataState::Open, ..).at_md(|s| s.padding(..))` (`src/style/layers.rs:18-30+`, lib.rs:36-47).
- Merge = per-field `override.or(base)` struct-wise (`src/style/merge.rs:9-24+`) — a hand-written conflict resolution per property group, the typed analogue of our StylePlan merge.
- Tokens are typed (`Spacing`, `Color`, `BorderRadius`, …) with an OKLCH converter delegated to the `palette` crate (`src/tokens/oklch.rs:1-27`).

**What NOT to take:** no class-string parsing exists (it never parses `hover:` — the `hover()` builder is a closure, `layers.rs:18-27`); breakpoints are a closed enum (no arbitrary `min-[…]` values); no utilities beyond its typed token set (e.g. no `backdrop-*`, no mask utilities). Its `style/merge.rs` merge is *hand-written per property group* (`merge_flex_container` `:9-24`, `merge_padding`, …) — that maintenance burden is exactly what our conflict-trie + StylePlan avoids.

**Verdict: BORROW IDEAS.** Its `BTreeMap<Breakpoint, Style>` + `StateStyles` nesting mirrors our StylePlan layers; the per-field `or()` merge semantics is a clean reference for our Plan merge. Typed `DataState` enum list (open/closed/checked/…, `state.rs:5-19`) is a good checklist for our `data-state` variant support. Nothing to vendor; API is method-only and its breakpoint set is fixed (no arbitrary breakpoints).

## 5. encre-css — plugin `CssType` hints + v4-style color-mix opacity, MIT

`encre-css` 0.21.1, MIT, edition 2024 (`crates/encre-css/Cargo.toml`). A scanner→generator Tailwind-compatible compiler with a **plugin registry**:
- **Static/Dynamic plugin split**: `StaticPlugin` uses `&'static str`/`phf::Map`/`&'static [CssType]` (const, zero-alloc) vs `DynamicPlugin` with `String`/`HashMap` (`src/plugins/mod.rs:60-73`, `CustomPlugin` enum `:85-89`). This is the "fixed vs dynamic" hint idea.
- **`CssType` hint enum** (`src/selector/mod.rs:119-150`): `Shadow`, `AbsoluteSize`, `RelativeSize`, `Url`, `LineWidth`, `LineStyle`, `Color`, … — a plugin declares which CSS *types* its arbitrary values accept (`hint: Option<CssType>` at `selector/mod.rs:218-221`); predicates live in `src/utils/value_matchers.rs`: `is_matching_shadow:223`, `is_matching_absolute_size:235`, `is_matching_relative_size:247`, `is_matching_url:259`, `is_matching_line_width:271`, `is_matching_line_style:283`, `is_matching_color:296`, `is_matching_length:312`, `is_matching_number:326`, `is_matching_percentage:338`, `is_matching_time:351`, `is_matching_position:365`, `is_matching_angle:377`, `is_matching_image:389`, `is_matching_font_family_name:410` (462-line file). The color matcher **misses `oklch(`/`oklab(`/`color-mix(` prefixes**, only `rgb/rgba/hsl/hsla/hwb/lch/lab` + `#`-hex + named (`:296-302`).
- **Opacity via v4 `color-mix`**: `bg-red-500/25` → `color-mix(in oklab, <color> 25%, transparent)` and `/[--var]` → `color-mix(in oklab, <color> var(--my-opacity), transparent)` (`src/utils/color.rs:38-78`, format at `:70-75`; tests `:94-126`). This matches Tailwind v4 semantics (no `--tw-*` opacity vars).
- **Theme**: config-based (`config.theme.colors.add("primary", "#d3198c")`, `dark_mode` class/media — `src/config.rs:12-22,1301-1315`); **no `@theme` CSS directive support** (`rg -F '"@theme"' src` → 0). Builtin palette is a `phf` map (`BUILTIN_COLORS`).
- Custom theme colors are emitted verbatim (oklch strings pass through unmodified, tests `utils/color.rs:94-126`).

**Verdict: BORROW.** (a) the `CssType` type-hint idea for classifying our arbitrary values; (b) the Static/Dynamic plugin duality informs our table-vs-codegen split; (c) its `color-mix(in oklab, …)` emission is a *behavioral oracle* for our opacity lowering (including the `[--var]` escape). Its own color matcher is weaker than ours (no oklch), so nothing to take there.

## 6. crepuscularity (crepuscularity-core) — typed length/color tokens, ISC

`crepuscularity-core` 0.5.0, **ISC** (`crates/crepuscularity-core/Cargo.toml:2-5`); 23-crate GPUI-first monorepo. Shared v4 utility parsing used by all its backends (`src/tailwind/mod.rs:1-16`).

- **`LengthToken`** (`crates/crepuscularity-core/src/tailwind/length.rs:10-19`): `Auto | Fraction(f32) | Px(f32) | Rems(f32)` — resolved-but-not-lowered token; `parse_length_token` handles keywords (`full|screen|auto|px`), fractions `1/2`, numeric scale → `Rems(n * 0.25)`, arbitrary `[12px]|[1.5rem]|[50%]|[12]` (`length.rs:23-59`). Deliberately rejects non-finite/overflow values (`length.rs:70-91`). Note: **`em`/`vh`/etc. are rejected** (`length.rs:132-139`).
- **Colors**: `resolve_css_color` (`tailwind/parse.rs:225-247`) — named palette / hex / `family-shade/opacity` encoded as `#RRGGBBAA`; `parse_css_hex_color` accepts 3/4/6/8-digit hex with `#`/`0x` (`parse.rs:249-270`). Full public fn set (`tailwind/mod.rs:8-16`): `parse_color_rgb`, `parse_css_hex_color`, `parse_font_size_named`, `parse_fraction`, `parse_opacity_percentage`, `parse_radius_px`, `parse_size_pt`, `parse_size_width_height`, `parse_spacing_pt`, `parse_spacing_px`, `parse_text_size_px`, `resolve_arbitrary_css_color`, `resolve_css_color`, `SizeToken`, `SIZE_FILL`, `SIZE_FIT` — i.e. one *token-level* fn per Tailwind axis, each backend lowers independently.
- **OKLCH→sRGB**: *there is no runtime conversion anywhere*. `crepuscularity-colors/src/palette.rs:1-6` documents the trick: "**Full Tailwind CSS v4 default palette (OKLCH → sRGB hex). Generated from tailwindcss v4.3.0 `theme.css` via `scripts/gen-tailwind-colors.mjs`**" — the conversion is baked at generation time into a `lookup_named_color(name) -> &'static str` match table (`palette.rs:11+`). Compare with our `color.rs` runtime conversion: theirs can't handle arbitrary `oklch(...)` input (e.g. `bg-[oklch(...)]`); ours can. Their approach *is* interesting for our shipped-theme path (precompute hex tables for speed) but as an oracle it pins them to one palette version.

**ISC implications:** ISC is MIT-equivalent permissiveness (one extra non-endorsement clause); vendoring with attribution is fine. **Verdict: BORROW** the `LengthToken` intermediate-representation idea (token resolved from class, lowered per backend) and their test lists (non-finite rejection cases, `length.rs:70-91`); do NOT copy palette.rs (huge generated table, and we want runtime conversion for arbitrary values).

## 7. icss (andaja) — CSS-subset parser + per-widget resolver, MIT

`icss` 0.1.1, MIT, rustc ≥ 1.88 (`Cargo.toml:9-12`). An iced design-system crate, not a Tailwind compiler:
- **CSS engine** (`crates/icss/src/theme/css/`, 1746 lines, no iced dependency — `css/mod.rs:1-5`): tokenizer → parser → AST: `Stylesheet { custom_properties: HashMap<String, CssValue>, rules: Vec<Rule> }` (`css/ast.rs:10-15`), `Rule { selector, declarations, source_order }` (`:18-24`), `Selector { classes, pseudo }` with `specificity() -> (pseudo_count, class_count)` (`:30-42`) and `matches(element_classes, active_pseudo)` subset-matching (`:44-65`), typed `CssValue` with `as_color/as_length/as_number/as_shadow` accessors (`:109-176`), `var()` custom-property substitution in `parser.rs`.
- **Resolve layer** (`theme/resolve/`): one file per widget (button, checkbox, slider, … — 23 files incl. computed.rs, rule.rs, theme.rs) turning matched rules + specificity + source order into iced styles — i.e., a programmatic "rule → widget style" mapping with cascade.
- Extras: `theme/color.rs` has `parse_hex_color` (`:4`), `ThemeColor { from_hex, darken, lighten, with_alpha, to_css_string }` (`:39-111`); `engine/tailwind.rs` + a `gen_tailwind` bin (`crates/icss/Cargo.toml:15`) emit the JS preset; `src/widgets/` holds 12 hand-built widgets (menu, data_table, tab_bar, …) that consume the resolved theme.
- **Tailwind relation**: `engine/tailwind.rs:1-23` only *generates a JS preset file* (`tailwind.preset.js`) so a web app can mirror the desktop theme — no class parsing at all.

**Verdict: BORROW.** The `Selector::specificity()/matches()` pair and `source_order` tie-break (`ast.rs:39-65`) are a compact, tested model for our shadcn class-table cascade if we ever need selector-style precedence; the `resolve/<widget>.rs` per-widget mapping pattern is a good structure for `aeolus-shadcn`. Not reusable as-is (iced-coupled value types; CSS subset only).

## 8. ironframe — CI parity oracle, MIT OR Apache-2.0 ★

`ironframe` 0.4.0, edition 2024, **MIT OR Apache-2.0** (`Cargo.toml:2-6`) — self-described "Production-ready Tailwind CSS v4.1-compatible Rust engine" (`Cargo.toml:10`). Bin/lib split; vendored data files `include_str!`ed at `src/lib.rs:18-22`:
- `src/default_theme.css` (453 lines, v4 default `@theme` values), `src/preflight.css`, `src/properties_definitions.css` (`@property --tw-*` declarations, `:1-12+`), `src/properties_fallback.css`.
- **`core::parse()` is a STUB** — `src/core.rs:33-39`: `pub fn parse(_input: &str) -> Result<AstNode, ParseError> { Err(ParseError { message: "parse not implemented", … }) }` — **always errors**. `rg 'todo!|unimplemented!' src tests` → 0 matches, so the stub is the *only* dead surface (a generic tokenizer/AST skeleton around it is inert). `scanner::scan` (`src/scanner.rs:46-59`) and `generator::generate` (`src/generator.rs:32-34`, dispatch into 22k lines of v4 rules incl. variant tables `generator.rs:50+`) are real.
- Its Build command accepts `compare_css: Option<String>` (`src/lib.rs:28-41`) — i.e. it diffs its own output against a reference CSS.
- Generator internals: `GeneratorConfig { minify, colors: BTreeMap<String, BTreeMap<String,String>> }` (`generator.rs:7-10`), `GenerationResult { css, class_count }` (`:12-16`); per class: `parse_variants(class)` (`:10531`), then `generate_rule` (`:1876`), then a deterministic `RuleSortKey { variant_bucket, variant_rank, property_rank, … }` (`:18-27`, builder `:230`) for v4-compatible output ordering; `build_variant_tables` (`:127`) centralizes responsive/container/dark selectors with `VariantOverrides` hooks (`:50-65`).
- **Supported-class corpus:** `tests/fixtures/complex_parity/` — `complex.html` (2.1 KB class corpus covering ~200 v4 utilities incl. `text-lg/[1.75]`, `text-(length:--my-var)`, `max-h-[calc(100dvh-(--spacing(6)))]`, `rotate-[3.142rad]`), `tailwind.css` (25.6 KB expected output, header `/*! tailwindcss v4.1.17 | MIT License */` — real Tailwind v4.1.17 output), `app.css`.

**CI-oracle recipe for Aeolus:** (1) extract the class list from `complex_parity/complex.html` (it is dual-licensed, vendoring OK); (2) run our parser→StylePlan→CSS lowering; (3) compare against the `@layer utilities` section of `complex_parity/tailwind.css` (the ground truth is genuine tailwindcss v4.1.17 output — stronger than ironframe's own generator); (4) additionally diff our `@layer properties` against `src/properties_definitions.css`. Do **not** depend on `ironframe::core::parse` — it is a stub.

## 9. gpui-rsx — compile-time static expansion, MIT

`gpui-rsx` 0.10.2, MIT, rustc ≥ 1.88 (`Cargo.toml:8-21`). A proc-macro JSX for GPUI that expands `<div class="flex gap-4">` **at compile time into GPUI builder calls** — "Static RSX Has No Runtime Parser" (`README.md:12-22`). Class handling (`crates/gpui-rsx-core/src/codegen/class.rs:1-15`): every class goes through `analyze_class` → *supported* → method-call chain; *unknown identifier* → passthrough to same-named GPUI method + `#[deprecated]`-based compile-time warning with spelling suggestions; *unexpressible* (`dark:`, `md:`) → warning (permissive) or error (strict); *invalid values* (`w-[abc]`) → error. Backed by a 6733-line generated table (`codegen/tables_generated.rs`) mapping 242 colors + utilities; `rsx.toml` selects spacing scale `px` vs Tailwind `ln` (`theme/parse.rs`). **Idea for Aeolus (one paragraph):** a `styl!{ ... }`/derive front-end that runs our parser + StylePlan at macro time and emits native builder calls with zero runtime cost, using diagnostics (spanned deprecation shims) to surface unknown classes at compile time — the same UX gpui-rsx gives GPUI, but with full v4 coverage via aeolus-core instead of a hardcoded table. **Verdict: BORROW IDEA** (diag style, `analyze_class` triage, compile-time expansion), no code.

## 10. floem-tailwind + floem-shadcn — ANTI-EXAMPLES, MIT

- **floem-tailwind** 0.1.0 (`Cargo.toml:2-7`): a `TailwindExt` trait extending floem's `Style` with method sugar — `w_64().h_32().p_4().gap_2().rounded_lg()`, `bg_blue_500()` (`src/lib.rs:1-40`, trait at `:553+`, ~928 generated methods via `width_methods!` macros, `lib.rs:554-561`). Spacing scale hardcoded 4px/unit (`lib.rs:19-27`). **No class-string parsing anywhere** (`rg 'parse|split' src/lib.rs` → 0 relevant hits); **no v4 anything** (no oklch, no color-mix, no `@theme`).
- **floem-shadcn** 0.1.0: builder-based shadcn clone for Floem — 45+ components (`src/components/`: accordion…tooltip). Variants are Rust enums: `ButtonVariant { Default, Destructive, Outline, Secondary, Ghost, Link }`, `ButtonSize { Sm, Default, Lg, Icon }` (`src/components/button.rs:26-38`), chosen by builder methods `Button::new("x").sm().ghost()` (`button.rs:1-18`); styling via `floem_tailwind::TailwindExt` methods and a `ShadcnTheme` struct — **zero class strings** in the styling path.

**What Aeolus keeps as optional sugar:** nothing from these APIs — they demonstrate the failure mode Aeolus exists to fix (no class ergonomics, per-widget codegen, no theme interop). Their *component coverage list* is still a useful completeness checklist for `aeolus-shadcn` (45+ components beyond our 13 mandatory ones).

## 11. shadcn-official (shadcn/ui monorepo) — token + registry oracle ★

The real JS monorepo. **v4 theme CSS with OKLCH tokens:**
- Current dev theme: `apps/v4/app/globals.css` `:root` block at `:100-136` and `.dark` at `:137-165` — **redesigned palette** (`--foreground: oklch(0% 0 0)`, extra `--surface/--code/--selection/--chart-*` tokens).
- Classic/legacy v4 default ("neutral") theme: `apps/v4/app/legacy-themes.css:498` `.theme-neutral { … }` with nested `@variant dark { … }` — this matches our shipped files (see diff below). Also mirrored in `apps/v4/registry/themes.ts` and `apps/v4/registry.json`.
- **Diff of our `/home/z/my-project/aeolus/crates/aeolus-core/data/shadcn-light.css` + `shadcn-dark.css` vs upstream `.theme-neutral` (legacy-themes.css:498+, dark via nested `@variant dark`), token by token (scripted diff, `--` = not defined upstream / not shipped by us):**

| token | ours light | upstream light | ours dark | upstream dark |
|---|---|---|---|---|
| --background | oklch(1 0 0) | = | oklch(0.145 0 0) | = |
| --foreground | oklch(0.145 0 0) | = | oklch(0.985 0 0) | = |
| --card / --card-foreground | 1 / 0.145 | = / = | 0.205 / 0.985 | = / = |
| --popover / --popover-foreground | 1 / 0.145 | = / = | 0.205 / 0.985 | = / = |
| --primary / --primary-foreground | 0.205 / 0.985 | = / = | 0.922 / 0.205 | = / = |
| --secondary / --secondary-foreground | 0.97 / 0.205 | = / = | 0.269 / 0.985 | = / = |
| --muted / --muted-foreground | 0.97 / 0.556 | = / = | 0.269 / 0.708 | = / = |
| --accent / --accent-foreground | 0.97 / 0.205 | = / = | **0.371** / 0.985 | **0.269** / 0.985 |
| --destructive | 0.577 0.245 27.325 | = | 0.704 0.191 22.216 | = |
| --destructive-foreground | 0.985 0 0 | **—** (dropped) | 0.985 0 0 | **—** (dropped) |
| --border | 0.922 0 0 | = | 1 0 0 / 10% | = |
| --input | 0.922 0 0 | = | 1 0 0 / 15% | = |
| --ring | 0.708 0 0 | = | 0.556 0 0 | = |
| --radius | 0.625rem | = | (shared) | = |
| --chart-1..5 | **missing** | 0.87/0.556/0.439/0.371/0.269 | **missing** | same as light |
| --sidebar + --sidebar-{foreground,primary,primary-foreground,accent,accent-foreground,border,ring} | **missing** | 0.985/0.145/0.205/0.985/0.97/0.205/0.922/0.708 | **missing** | 0.205/0.985/0.488 0.243 264.376/0.985/0.269/0.985/1 0 0 / 10%/0.556 |

  Verdict: **all 18 overlapping core tokens match exactly** (light and dark); three deltas:
  - **Discrepancy 1 — `--accent` in dark:** ours `oklch(0.371 0 0)` vs upstream classic `oklch(0.269 0 0)`. 0.371 is the *current redesigned* `.dark` value (globals.css:151) — our dark file mixed sources. Recommend: switch to `oklch(0.269 0 0)` (classic) or re-pin deliberately.
  - **Discrepancy 2 — missing tokens:** upstream defines `--chart-1..5` and the 8 `--sidebar-*` tokens (light+dark) that our files omit (fine if intentional, but note the `sidebar` component and chart examples reference them).
  - **Discrepancy 3 — `--destructive-foreground`:** we ship `oklch(0.985 0 0)`; the current upstream theme *dropped* this token (components hardcode `text-white` on destructive — `ui/button.tsx:13`). Keep if our shadcn tables reference it, else document.
  - **Header rot:** both our files cite "Source: apps/v4/app/globals.css (HEAD 2026-11)" — at current HEAD globals.css no longer contains the classic values (its `:root` has `--foreground: oklch(0% 0 0)`, globals.css:102); correct source is `apps/v4/app/legacy-themes.css` `.theme-neutral` (also mirrored in `apps/v4/registry/themes.ts` and `apps/v4/registry.json`).
- **Registry component set:** `apps/v4/registry/new-york-v4/ui/` — 60+ components. All 13 required present: `button.tsx` ✓, `input.tsx` ✓, `label.tsx` ✓, `card.tsx` ✓, `badge.tsx` ✓, `separator.tsx` ✓, `skeleton.tsx` ✓, `alert.tsx` ✓, `checkbox.tsx` ✓, `switch.tsx` ✓, `tabs.tsx` ✓, `dialog.tsx` ✓, `select.tsx` ✓ (plus alert-dialog, avatar, breadcrumb, calendar, chart, command, drawer, field, form, sidebar, table, etc.).
- Reference class table (upstream `ui/button.tsx:11-31`): base `"inline-flex shrink-0 items-center justify-center gap-2 rounded-md text-sm font-medium whitespace-nowrap transition-all outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50 aria-invalid:border-destructive …"`; variants `default: "bg-primary text-primary-foreground hover:bg-primary/90"`, `destructive: "bg-destructive text-white hover:bg-destructive/90 …"`, `outline: "border bg-background shadow-xs hover:bg-accent hover:text-accent-foreground dark:border-input dark:bg-input/30 …"`, `secondary`, `ghost`, `link`; sizes `default/xs/sm/lg/icon/icon-xs/icon-sm/icon-lg` (`:22-31`).

**Verdict: PARITY-ORACLE ONLY** (MIT; copying component JSX is allowed but it's TS not Rust — our tables should *match its class strings*).

## 12. shadcn-ui (RustForWeb port) — stale class tables, MIT, unmaintained

Clone of `RustForWeb/shadcn-ui`: Leptos + Yew ports, workspace 0.0.1, edition 2024, MIT (`Cargo.toml`, README banner "This project is unmaintained." at README.md:2). Workspace: `packages/leptos/*`, `packages/yew/*`, `packages/shadcn` (CLI/registry), `packages/registry`. Per-component crates (e.g. `packages/leptos/button`) ship `default` + `new_york` styles via `#[tw(class = "…")]` attribute macros: base `"inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 …"` (`packages/leptos/button/src/default.rs:9`) — **v3-era classes** (`ring-offset-*`, `ring-2`, no `ring-[3px]`/`outline-none` v4 style) and pre-redesign variants (`new_york.rs:9,20`: default variant `"bg-primary text-primary-foreground shadow hover:bg-primary/90"`). **Verdict: ignore as oracle** (stale vs shadcn-official); only use if we need historical v3 class comparisons. No license risk (MIT).

## 13. leptos-shadcn (CloudShuttle) — class-string reference, MIT

Workspace 0.9.0, MIT, edition 2024 (`Cargo.toml:95-101`); 20+ per-component packages under `packages/leptos/` (button, input, label, checkbox, switch, radio-group, select, textarea, card, separator, tabs, accordion, dialog, popover, tooltip, alert, badge, skeleton, progress, toast, table, slider, toggle, menubar, hover-card, aspect-ratio, collapsible, scroll-area, sheet, drawer, alert-dialog, avatar, resizable — per `Cargo.toml:38-78`) plus infra packages (registry, cli, contract-testing, tailwind-rs-core). Classes are `pub const` strings + variant matches — directly comparable with our `aeolus-shadcn` tables. Button (`packages/leptos/button/src/default.rs`):
- `BUTTON_CLASS` (`default.rs:16`): `"inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50"` (note: **no `gap-2`**, vs upstream) + `BUTTON_TOUCH_CLASS = "min-h-[44px] min-w-[44px]"` (`:21`).
- Variants (`default.rs:127-131`): Default → `"bg-primary text-primary-foreground hover:bg-primary/90"`; Destructive → `"bg-destructive text-destructive-foreground hover:bg-destructive/90"`; Outline → `"border border-input bg-background hover:bg-accent hover:text-accent-foreground"`; Secondary → `"bg-secondary text-secondary-foreground hover:bg-secondary/80"` (+ Ghost/Link further down).
- **Verdict: PARITY-ORACLE (secondary).** It tracks the *old* shadcn design (v3 classes, `ring-2`, `ring-offset-background`) — use as a diff-checklist, not as ground truth; shadcn-official `apps/v4/registry/new-york-v4/ui/*.tsx` remains authoritative.

---

## 14. Reuse-decision table

| repo | license | version | verdict | what exactly to take |
|---|---|---|---|---|
| tailwind-rs | **MPL-2.0** | tailwind-ast 0.4.1 | anti-reuse (license + weaker grammar) | nothing vendored; only the idea of group expansion at parse time |
| tailwind-fuse | MIT | 0.3.2 | oracle (superseded) | v3-subset regression fixtures only |
| tw_merge | MIT | 0.1.22 | **oracle** for `cn()` | test corpus (`tests/*.rs`), 272-id group table as checklist, parser edge semantics (v4 `!`, `/`+`@` variants, pass-through) |
| twill | MIT | 0.3.0 | borrow | `BTreeMap<Breakpoint, Style>` nesting, `StateStyles` shape, per-field `or()` merge, `DataState` enum checklist |
| encre-css | MIT | 0.21.1 | borrow | `CssType` hint enum idea; Static/Dynamic plugin duality; `color-mix(in oklab, …)` opacity emission as oracle |
| crepuscularity | ISC | core 0.5.0 | borrow | `LengthToken` resolved-token IR + non-finite test cases; palette.rs as build-time-bake idea only |
| icss | MIT | 0.1.1 | borrow | `Selector::specificity/matches` + `source_order` cascade model; per-widget resolve/ file pattern |
| ironframe | MIT OR Apache-2.0 | 0.4.0 | **oracle** for class→CSS CI | `tests/fixtures/complex_parity/{complex.html,tailwind.css}` corpus + `properties_definitions.css`; ignore `core::parse` (stub) |
| gpui-rsx | MIT | 0.10.2 | borrow idea | compile-time class→builder expansion, `analyze_class` triage, `#[deprecated]` diagnostics |
| floem-tailwind | MIT | 0.1.0 | anti-example | none (component coverage list only) |
| floem-shadcn | MIT | 0.1.0 | anti-example | none (45-component completeness checklist) |
| shadcn-ui (RustForWeb) | MIT (unmaintained) | 0.0.1 | ignore | nothing current (stale v3 classes) |
| shadcn-official | MIT | (JS monorepo) | **oracle** for tokens/components | `apps/v4/app/legacy-themes.css` `.theme-neutral` token values; `registry/new-york-v4/ui/*.tsx` class strings |
| leptos-shadcn | MIT | 0.9.0 | secondary oracle | per-component const class strings for diff-checklist |

## 15. License red flags & plan impact

1. **MPL-2.0:** entire oovm/tailwind-rs workspace (`License.md`; `license = "MPL-2.0"` in all 7 member Cargo.tomls). Any vendored file from it becomes MPL — forbid `tailwind-ast`/`tailwind-css` code in Aeolus; also avoid copying its test *files* (test code is MPL too; retype fixtures from behavior instead).
2. **ISC** (crepuscularity): permissive; vendor with attribution if ever needed.
3. Everything else MIT or MIT-OR-Apache-2.0 → vendoring permitted with attribution; prefer *oracle over vendor* everywhere since aeolus-core already implements parser/merge/color.
4. Plan impact: (a) adopt tw_merge tests + group table as `cn()` parity suite; (b) adopt ironframe `complex_parity` corpus as class→CSS golden test; (c) fix our dark `--accent` and theme-file source headers (§11); (d) optionally add `--chart-*`/`--sidebar-*` tokens to shipped theme files; (e) do not couple to `ironframe::core::parse` or the unmaintained RustForWeb port.
