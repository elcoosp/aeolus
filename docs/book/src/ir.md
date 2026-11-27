# The IR

`aeolus-ir` is the heart of the revamp: the longhand IR every stage
speaks. Three moving parts, all `Copy`, all `const`-constructible, none
of them touching the heap in the static form.

## `Prop` and `PropMask`

`Prop` is the longhand list, `#[repr(u16)]`. The discriminant order IS
the cascade and sort order — "sort declarations" is an integer compare
and nothing else, and the append-only tail has grown the list to 160
longhands (decoration, motion, filters, the arbitrary-property arm).
`PropMask` is one `u128` bitset over that list; the per-edge set-mask
model v0.8 shipped for padding/margin/border/radius generalizes to the
whole surface, so dominance between `p-4` and `px-8` (and between a
layer and its parent condition) is bit algebra, not table walks.

## `Val`

The value type, 24 bytes, `Copy`, no variant owns memory:

| Variant         | Carries                                        | Example                    |
| --------------- | ---------------------------------------------- | -------------------------- |
| `Px(f32)`       | a baked pixel length                           | `p-4` under the `px` config |
| `Rem(f32)`      | a rem length (first-class on purpose)          | `p-4` = `1rem`             |
| `Pct(f32)`      | a percentage                                   | `w-1/2`                    |
| `Num(f32)`      | a unitless number on the utility's own scale   | `z-10`                     |
| `Rgba([u8; 4])` | a baked colour, gamut-mapped once at build     | `bg-primary` (T0/T1)       |
| `Tok`           | a runtime token id + the `/opacity` multiplier | `bg-primary/90` (T2)       |
| `ModePair(u16)` | an index into a `(light, dark)` const pair     | the `dark:` tier's T1 form |
| `Kw(u8)`        | a per-prop keyword index the backend maps      | `flex-col`, `uppercase`    |
| `Verbatim`      | the composite grammar's interned value text    | `calc(100%-2rem)`, `url(...)` |

The `Verbatim` payload is an interned `&'static str` — the same
discipline the attribute-probe conds ride — so the static form stays a
`static` item end to end.

## `Decl`, `Layer`, `Program`

A `Decl` is one longhand binding (32 bytes); a `Layer` is one
condition's declaration set plus the derived mask; a `Program` is the
layer 0 base rule plus the condition layers in specificity order.
`Layer::new` is `const`-constructible, which is what makes the frozen
`static` form possible: the macro's `tw!` output is a `const` block of
layers, applied at runtime under a `Copy` `Env` with no parsing.

## `Cond`

The condition surface is the typed variant model: `Base`, the state
family (`Hover`, `Focus`, `FocusVisible`, …), `Dark`, the media tier
(`Cond::Media` over the `Pref` bits), breakpoints (`Bp`), the
attribute fold (`Data`/`Aria`/`Has`), `Supports`, the direction tier
(`Direction`), the structural selectors (`Structural`) and the
pseudo-element render targets. Every variant the pipeline accepts is
one of these; everything else is a typed refusal with a reason. The
flat model's boundary — stacked variants fold to the dominant
condition — is documented and pinned, and the CSS arm renders the
conjunctive selector text the fold cannot carry.

## The contract a consumer can implement

The IR is the stable seam other frameworks implement: read a frozen
`StaticProgram`'s layers, walk its declarations per condition, and map
(or honestly report) each longhand onto native style — exactly what
the five interpreter backends do, and what [the conformance
scores](conformance.md) measure per backend.
