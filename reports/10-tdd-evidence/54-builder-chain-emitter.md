# 54 — The builder-chain emitter (the emit (a) END FORM)

The V1 architecture plan's standing deferral, three notes deep in the
tree at HEAD (`docs/AEOLUS_V1_ARCHITECTURE.md` §3.2 status: "The
backend METHOD calls the emit (a) end form implies
(`FlexDirection::Column`, builder chains) remain the P3 codegen
crate's surface"; the §8 P2 row: "the backend method calls staying the
P3 codegen's surface"; `reports/10-tdd-evidence/
19-rust-tokens-arm.md`'s boundary list) — and the goal statement's own
shape (§1: "a Tailwind v4 compiler whose output is the code you would
have written by hand"), the §2 Approach A verdict ("Chosen for call
sites"), the §4 target row ("= hand-written builder chain") and §10's
first done-definition bullet ("compiles to code indistinguishable in
cost from hand-written builders"). Where the static emitter renders
the frozen `aeolus_rt::StaticProgram` the walk applies per frame, the
builder-chain emitter renders the program AS the GPUI builder-call
closure — the `Emit` trait's own documented shape ("renders the
target's builder calls") — and the macro face applies it with NO walk.

## The shape evidence

The seam is `aeolus-codegen-gpui`'s second `Emit` impl beside
[`GpuiEmit`]: `GpuiBuilderEmit` (+ `emit_builder`, the free face;
`lower_class_builder`, the `lower_class` twin the §5 integration
bullet's `resolve_style()` equivalent names; `audit_builder`, the
macro gate's twin) renders one compiled program as ONE closure literal
— `|s| s.flex().flex_col().pt(::aeolus_gpui::chain::rems(1.0))
.hover(|s| s.bg(..rgba(0x..)))`. The contract is complete for its
stated scope, every input answered, none silent:

* the base cascade folds in cascade order (last call wins per
  longhand, the walk's own front-to-back reading; the `!important`
  origin's layers sort after their normal siblings, so order carries
  the win here too);
* the four native state methods §3.5's GPUI mapping column names
  (`hover`/`active`/`focus`/`focus_visible`) are the conditional face;
  every other conditional family refuses typed — the frozen-program
  walk's probe seams answer them;
* the value forms mirror the walk's helpers one for one: `Val::Rgba`
  as the walk's own `rgba(u32::from_be_bytes(..))` conversion, the
  numeric/length slots as `px`/`rems`/`relative` literals (the
  shortest-round-trip f32 form), the row-text keyword slots through
  the walk's row-length resolution (the CALC texts stay the no-op the
  walk answers — never a guessed value);
* the static gate's verdicts stand (the chain runs
  `decl_unsupported_reason` first), then the chain gate refuses the
  theme-token slots (the runtime `TokenTable` resolves them — the
  T0/T1 bakes freeze literals) and the state-accumulator longhands the
  plain `Styled` methods do not carry (`ShadowState`/`RingState`/
  `GradState`/`FilterState`/`TwSpaceExt`, the text-style mutations);
* all-or-nothing: one refused declaration refuses the whole emit —
  never a partial chain.

The constructor paths ride `aeolus_gpui::chain` (the crate users
depend on), so the emitted tokens compile with no `gpui` naming of the
consumer's own; the trait methods resolve through `Styled`, which
joins the prelude beside `TwExt`. The macro face is `tw_builder!`
(the macro row's "thin shell over aeolus-codegen*" shape), applied
through the new `TwExt::tw_chain` entry.

## RED → GREEN

* RED: `crates/aeolus-codegen-gpui/tests/builder.rs` (12 pins,
  written first — the missing seam WAS the capture: the compile
  failure before `GpuiBuilderEmit` existed). The plan's acceptance
  criterion rides the module docs and the battery header: the §3.2
  status note's METHOD-calls sentence, quoted. The pins: the button
  string's method calls, the keyword families' walk methods, the
  numeric/length value forms, the four state closures, the `Emit`
  trait impl, the `lower_class_builder` seam shape, the theme-token
  refusal (the runtime table), the state-accumulator refusals, the
  non-native conditional families' refusal, all-or-nothing,
  determinism, the same-cond cascade fold's render order.
* RED: `crates/aeolus-gpui/tests/builder_parity.rs` (5 pins, written
  first — the wire-up was the compile failure). The differential
  target the contract claims: the emitted chain lands the SAME
  `StyleRefinement` the walk lands, FIELD BY FIELD (the perf_contract
  harness's parity-pin idiom) — display, align, justify, gap, radii,
  padding, border widths, the baked colour BYTES, text colour, size,
  weight, underline, text-align, white-space, opacity, grid cols,
  cursor, clamp count, the same-cond fold's last-wins on BOTH lanes.
* GREEN: `crates/aeolus-codegen-gpui/src/builder.rs` (the emitter),
  `crates/aeolus-gpui/src/lib.rs` (the `chain` module, `TwExt::
  tw_chain`, the prelude's `Styled`), `crates/aeolus-macro/src/
  lib.rs` (`tw_builder!`, the audit-gated expansion).

## Bench

* `crates/aeolus-codegen-gpui/benches/builder.rs`: the per-expansion
  cost the macro face adds (the gate walk, the render match, the
  closure assembly) — builder_emit 9.89 µs, builder_gate 11.11 µs
  medians (divan, release, 2026-11-27).
* `crates/aeolus-gpui/benches/builder.rs`: the §4 target row's first
  real measurement — the emitted chain against the walk against the
  hand-written calls on ONE harness: hand_button 47.5 ns, CHAIN
  61.3 ns, walk 829.8 ns medians (divan, release, 2026-11-27). The
  chain's per-frame cost is the hand-written chain's shape — the §4
  target's own reading, at the wall-clock level the stable harness
  measures.

## Example

`crates/aeolus-gpui/examples/builder_chain.rs` — the parity read-out
headless (chain == walk, field by field, the baked colour bytes) and
the two lanes' story. `crates/aeolus-codegen-gpui/examples/builder.rs`
— the seam face: literals in, closures out, the typed refusals with
their lane names.

## Boundaries kept

* the chain carries compile-time values only: a `Val::Tok`/
  `Val::ModePair` slot (the runtime table's T2 forms) refuses typed —
  the frozen-program lane's `tw_env_theme` walk or the T0/T1 bakes
  carry them;
* the conditional face is the four native state methods §3.5 names;
  dark/breakpoint/group/data/aria/has/supports/container and the
  pseudo-element families ride the walk's probe seams and refuse here
  (the reason names the lanes);
* the state accumulators (shadow bands, ring bands, gradient stops,
  filter matrices, the space/divide composite) and the decoration
  colour/thickness text-style mutations are the walk's — the chain
  refuses with the accumulator names in the reason;
* the `theme:` prefix does not ride `tw_builder!` yet — the themed
  chain bake is the follow-up slice, the frozen-program macros carry
  it today;
* the mapping duplication (the emitter's keyword/value arms beside
  the walk's helpers) is pinned by the parity battery, not assumed —
  the arm comments name the walk arm each mirrors, and a walk-arm edit
  that forgets the emitter fails the field pins.
