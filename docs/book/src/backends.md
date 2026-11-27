# Backends

Six backends consume the pipeline. One is native (GPUI); five are
interpreter lanes that walk the frozen `StaticProgram` under one probe
`Env` into their toolkit's own payload. Both lanes share the interp
arm's verdicts verbatim, carry the `AttrProbe`/`TokenSource` seams on
their `*_with` entry points, and map every longhand or report it onto
their `LoweringReport` — never a silent skip.

## The GPUI native lane

`aeolus-gpui` maps the frozen program onto GPUI's own style fields:
`TwExt::tw` applies the base plus environment-matched layers for any
`Styled` element, `TwStateExt` wires the interaction states, and the
specialized lanes compose what GPUI lacks natively — `RingState` (the
v4 ring as spread-only box-shadow bands), `ShadowState` (the multi-layer
presets onto GPUI's own `Vec<BoxShadow>`), `GradState` (the two-stop
linear gradients), `FilterState` (the colour-matrix filter primitives
recolouring the element's own slots). Motion rides the stateful
`TwAnimExt` lane (`with_animation` needs an element id and an animator
— the two inputs a frozen program cannot name), and `AeolusRoot` +
the prelude are the app-facing front door.

The per-longhand capability table (`caps_generated.rs`) is generated
from the emit gates' own verdicts — the published score is the
emitter's, never a parallel claim — and the hand-maintained rows carry
the per-declaration refinements.

## The five interpreter lanes

Xilem, Floem, Iced, egui and Slint each walk the frozen program into
their native payload: `lower_program`/`lower_program_with` (and the
probe/container/descendants/supports entry matrix), with the plan-lane
backends consuming the plan bridge's `StylePlan` where their toolkit's
API shape prefers it. Each lane carries its `REM_PX` bake, its
probe-to-`Env` conversion, and a differential pin against the plan
path. egui ships the gradient mesh painter its solid-`Color32` frame
lacks (`GradientPaint` + `gradient_mesh`).

## The capability report

Every backend implements the `Capability` seam — one tiered verdict
(native / polyfill / refused) per IR longhand, with a typed reason on
the refusals. Six conformance pages publish the scores (GPUI plus the
five interpreter lanes), regenerated through the same drift gate as
the registry page, and the CLI's `aeolus audit` tiers a real project's
classes against the same tables. [The conformance
chapter](conformance.md) includes the GPUI page; the per-backend pages
sit beside it in `docs/`.
