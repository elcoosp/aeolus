# The V1 pipeline

The pipeline is one pure function, no global state:

```text
compile(input, &Config)                    -> Result<Compiled, CompileError>
compile_theme(input, &TokenTable, &Config) -> Result<Compiled, CompileError>
```

Five stages run in order, each deterministic and total, each with its
typed failure channel:

1. **Lex** (`aeolus-syntax`) — the input string segments into borrowed
   candidates with byte spans. Per-token failures degrade as typed
   `LexFailure`s; they never fail the string. The lexer is
   zero-dependency and `no_std + alloc` capable (the CI's
   `no-std-syntax` job proves the face).
2. **Fold the variants** — the stacked prefixes fold onto the single
   dominant typed `Cond` (the flat-IR model). The CSS emit arm renders
   the segments' conjunctive selectors instead, which is why the
   condition text lives in the pipeline.
3. **Read the registry** (`aeolus-data`) — `lookup_detailed` answers a
   miss with a typed `LookupError` that separates an unknown root from
   a rejected value shape.
4. **Lower** — the row plus suffix fans out into longhand `Decl`s
   (the value pass: the spacing scale, the keyword tables, the
   arbitrary-value grammars, colours through the theme pass when a
   table is supplied). Without a theme table, colour tokens are typed
   boundaries, never truncations.
5. **Assemble** — a stable specificity sort produces the cascade-sorted
   `Program`: layer 0 is the base rule, the rest are sorted by
   condition specificity, with the `PropMask` bitsets derived at
   construction.

Everything a class does not lower to lands in
`Compiled::diagnostics` — a byte span, a stable code, a help text. The
two walks (plain and themed) share the lexer channel, the variant
refusal set, the registry read side and the value pass's boundary set;
`crates/aeolus-compile/tests/css.rs` pins their diagnostics equal over
a battery.

## The five emit forms

The same `Program` carries five emit faces, all in-tree:

| Arm                | Entry points                                     | Shape                                        |
| ------------------ | ------------------------------------------------ | -------------------------------------------- |
| (a) Rust tokens    | `emit_rust` / `emit_rust_theme`                  | one embeddable `Layer::new` expression per lowered class |
| (b) Static data    | `emit_static`                                    | one byte-stable `static` item over `Layer::new` constructors |
| (c) Interp         | `interp` / `interp_theme`                        | effective declarations under one probe `Env`, typed exclusion notes |
| (d) CSS            | `emit_css` / `emit_css_theme`                    | v4-shaped rules, at-rule grouping, the vars block |
| plan bridge        | `emit_plan` / `emit_plan_theme`                  | the v0.x `StylePlan` the interpreter backends consumed |

The static form is what the macros emit; the interp arm is what the
CLI's `expand` prints and the backend walks share their verdicts with;
the CSS arm is what `aeolus build` ships and the LSP hover renders.

## Composition, caching, observation

`cn(&[&Compiled])` merges class lists at the IR level: same-condition
layers fold by `Cond` equality and the last occurrence per longhand
wins — the differential contract is pinned against compiling the
concatenation, and `overlay` composes a frozen base with a runtime
override rightmost-wins. `DynCache` is the bounded, content-keyed,
thread-safe cache over the compile surface with its `DynStore`
storage seam; the observability faces (`DynObserver` over the cache,
`WalkObserver` over the compile walk, `ThemeObserver` over the theme
stage) stream every decision to host-supplied collectors with no-op
defaults, and the differential pins hold every contract byte-identical
with and without an observer.

Custom authoring registers through `Customs` (the `@utility` and
`@custom-variant` seam, from the API or straight from the theme CSS
through `customs_from_css`), the metamorphic checker
(`metamorphic_check*`) walks compile ⟷ CSS ⟷ interp per string, and
the project-wide form is `scan`/`build` — the whole-source-tree
stylesheet behind the CLI's `aeolus build`.
