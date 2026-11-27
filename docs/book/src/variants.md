# Variants & state

Every variant the pipeline accepts folds onto a typed `Cond`, and every
question a `Cond` can ask has a typed answer channel. The model is:
frame facts live in the `Copy` `Env` (one field per question), host
state lives behind probe traits, and an unanswered question is a typed
note — never a silent skip.

## The tiers

| Family | Examples | Answered from |
| ------ | -------- | ------------- |
| Interaction states | `hover:` `focus:` `active:` `disabled:` | `Env` state bits (GPUI wires its own interaction closures) |
| Mode & breakpoints | `dark:` `sm:`…`2xl:` `max-*:` | the `Env` mode bit and viewport rem |
| Media features | `motion-*:` `contrast-*:` `forced-colors:` `pointer-*:` `portrait:` `inverted-colors:` `noscript:` | `Env` frame facts — unset bits are the quiet miss a media query that does not hold is |
| Direction & validation | `rtl:` `ltr:` `valid:` `invalid:` `user-valid:` … | `Env` facts with complement readings |
| Element/widget/form states | `target:` `open:` `inert:` `enabled:` `required:` … | `Env` facts (the matches-neither pairs carry membership facts beside the state bit) |
| Attributes | `data-*:` `aria-*:` `group-*:` | the `AttrProbe` host seam (`tw_probes`) |
| Containers | `@md:` `@md/card:` | the `Env`'s nearest-container slot, or the `ContainerProbe` seam for named containers |
| Descendants | `has-*:` `group-has-*:` | the `DescendantProbe` seam (`DescendantState`) |
| Feature support | `supports-*:` | the `SupportsProbe` seam (`SupportsState`) |
| Structure | `first:` `nth-*:` `of-type` tiers | the `StructFacts` seam |
| Pseudo-elements | `selection:` `before:` `marker:` … | the `PseudoTarget` render targets — their own scoped cascade, never element state |

The GPUI lane carries the wire-ups beside the walks: `TwExt::tw_probes`
for the attribute bits, `TwExt::tw_containers` for named container
queries, and `TwContainerExt::tw_container` — the measuring element
that publishes its own box's width into the shared `MeasuredContainers`
hand-off at the layout phase, asking for exactly one re-render on a
real change.

## The honest boundaries

Arbitrary variants (`*:…`) stay diagnosed with a reason and a suggested
alternative; the arbitrary bracketed `has-` inner IS supported (the
selector-text grammar the real v4.1.17 renderings pin). Where a
backend cannot carry a layer, the walk reports it onto its
`LoweringReport` — the no-silent-failures rule reaches the last mile.
