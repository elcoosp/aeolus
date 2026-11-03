# Floem backend prototype — implementation notes (deliverable 5)

Status: **compiles and all tests green** (`cargo test -p aeolus-floem`:
7 passed). Code: `crates/aeolus-floem/` (single lib module, tests in
`tests/lowering.rs`). Audited framework commit:
`lapce/floem @ 1351ffb162faeb8be983f1301d718a0d5fb59c27` (v0.2.0 line).

## What was built

| piece | purpose |
|---|---|
| `lower(plan) -> (Style, LoweringReport)` | full `StylePlan` → one Floem `Style` value, **zero view-side code** (audit Q3 verdict: YES) |
| `apply_resolved(style, resolved)` | slot→prop mapping; per-side padding/margin/border, per-corner radius, `LengthAuto` dims, full `BoxShadow` (spread + signed offsets), flex props, typography |
| `lower_with_transitions(plan, Some(ms))` | attaches per-prop `Transition::linear` for background/text/border colors (`transition-colors` equivalent) |
| `FloemTwExt::{tw, tw_classes}` | attaches the lowered style via `Decorators::style(\|s\| s.apply(style.clone()))`; later style slots win, so user classes appended last win |
| `LoweringReport` | group/peer (no selectors), container (none), `visited`, text props without a v1 mapping — reported, never silent |

## Mapping highlights

- Colors: our Oklab → `peniko::Color` (straight sRGB); opacity modifiers
  survive as alpha (`bg-primary/50` → `alpha 0.5` through `Brush::Solid`).
- Conditions map 1:1 onto Floem's 10 `StyleSelector` variants:
  `hover/focus/focus_visible/focus_within/active/disabled/dark_mode`
  (Tailwind's `focus:` and `focus-visible:` both have native Floem forms).
- Responsive `md:` etc. lower to `Style::min_window_width(Pt(bp_px))` with
  v4 breakpoint px from `Breakpoint::default_px()` (40/48/64/80/96 rem).
- State priority on conflict is Floem's own (`disabled > active > focus >
  hover`, style/mod.rs:704-843) — matches Tailwind's cascade intent.

## Proof results (all passing)

Tests drive Floem's **public cascade evaluator**
(`Style::apply_selectors(&[StyleSelector])`) and assert *effective* values —
what a running app computes:

1. `base_layer_lowers_exactly` — bg-primary = #171717, p-4 = 16px,
   rounded-md = 8px.
2. `hover_resolves_through_selectors` — idle: primary; `apply_selectors(&[Hover])`:
   red-500 (the v4 oklch token, converted through our CSS-Color-4 path).
3. `focus_within_and_disabled_map_to_selectors` — `disabled:bg-red-500` and
   `focus-within:bg-black` both resolve through their selector maps.
4. `dark_layer_resolves_in_dark_mode` — `dark:bg-black` + `dark:text-white`
   resolve under `DarkMode` (inherited `TextColor` prop flips).
5. `group_and_peer_are_reported_not_silently_dropped` — group/peer dropped
   with reasons; `md:p-8` lowers into a responsive rule (no drop).
6. `oklab_alpha_semantics_match_v4` — `bg-primary/50` → alpha 128.
7. `transitions_attach_when_requested` — transition builders run cleanly.

## Honest caveats

- The `.tw()` attach path itself is exercised only by compilation (running it
  requires a live Floem reactive scope/window); the *lowering* it performs is
  fully tested. The audit (sections/floem.md) pins the attach semantics
  (`decorator.rs:80-95`, later-wins).
- Feature check: Floem hard-requires its default features
  (`vger`, `editor`) — `default-features = false` does not compile
  (hardcoded imports); the prototype therefore builds with stock features.
- The peniko version Floem uses (0.6) has no `PartialEq<[u8;4]>` for
  `Rgba8`; tests compare raw `AlphaColor::components` instead (comment in
  tests/lowering.rs).
- One earlier test expectation was wrong, not the code: v4's red-500 is the
  oklch token itself (no #ef4444 constant anymore) — the test now compares
  against our CSS-Color-4 conversion result.
