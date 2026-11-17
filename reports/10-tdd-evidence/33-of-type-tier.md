# 33 — The of-type structural tier

The structural family's type-relative slice: the five v4.1.17 spellings
`first-of-type:`/`last-of-type:`/`only-of-type:` and the functional
`nth-of-type-N:`/`nth-last-of-type-N:` pair (the upstream
`packages/tailwindcss/src/variants.ts` `staticVariant('first-of-type',
['&:first-of-type'])` trio plus the `functional('nth-of-type', …)` /
`functional('nth-last-of-type', …)` registrations — verified verbatim
against the pinned upstream source). The child-index family's own
semantics read in the TYPE-relative dimension: the `Structural` enum
grows five arms, the `StructFacts` seam value grows
`type_index`/`type_sibling_count` beside the child-index fields (what
"same type" means is the HOST's own reading — the seam carries the
answer, never a tag string), the fold routes the of-type prefixes FIRST
(longest prefix wins, the `nth-last-`/`nth-` order's own discipline),
the CSS arm renders `:first-of-type` through `:nth-last-of-type(N)`, the
plan bridge lands them on the oracle's own grown `ast::Structural` table
(the differential battery pins layer-for-layer), and the CLI's
`expand --child-of-type i/n` facts flag answers them (the `--child`
flag's sibling, one facts value per dimension).

* **M1** (the whole landing): the of-type tier end to end — IR arms,
  fold spellings, seam facts + `with_type`, verdict arms, CSS texts,
  static emit, plan-bridge twins (core `ast::Structural`, the parser's
  spellings, the snapshot writer/reader, the oracle CSS render), CLI
  flag, bench, example, docs.

## The differential evidence

```
staticVariant('first-of-type', ['&:first-of-type']),
staticVariant('last-of-type', ['&:last-of-type']),
staticVariant('only-of-type', ['&:only-of-type']),
variants.functional('nth-of-type', …),   // isPositiveInteger upstream;
variants.functional('nth-last-of-type', …)  // the fold keeps the repo's
                                            // own every-i32 `nth-` grammar
```

Upstream's remainder this landing deliberately does NOT carry beyond
the tier: the `nth-of-type-*` bracketed selector forms (`nth-[2n+1]`
stay outside, the child-index family's own refusal) and the
`has-*`-inner composition (structural inners refuse, the
`has-first:` boundary the fold documents — of-type rides the same
line). `nth-last-of-type-2:` keeps the oracle's last-flag divergence
pinned (the `nth-last-2:` precedent's sibling).

## RED → GREEN

* RED: `crates/aeolus-compile/tests/of_type.rs` (11 pins, written
  first) + `crates/aeolus-cli/tests/cli.rs`'s of-type battery (2) —
  compile failures against the un-landed `Structural` arms (`E0599`)
  and the fold's unknown-variant refusals.
* GREEN: the same 13 pins pass against the landed tier; the workspace
  battery re-measures at the gates (`tests/structural.rs`'s 17,
  `tests/supports.rs`'s 25 and `tests/cli.rs`'s battery unchanged).

## Bench

`cargo bench -p aeolus-compile --bench struct` (divan, release):
`interp_struct_of_type_hit` **161.1 ns** median (the fire path, two
fact asks), `interp_struct_of_type_unknown_miss` **114.9 ns** (the
resting path) — the child-index family's own per-layer cost, the type
dimension riding the same integer-compare verdicts.

## Consumer examples

`examples/struct_probe.rs` grows the of-type demos (sections 7-8: the
middle-child/first-of-type facts walk and the CSS selectors) —
`cargo run -p aeolus-compile --example struct_probe`.
