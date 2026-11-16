# 23 — The supports-variant seam

Slice: the interp arm's FIFTH seam member — the `supports-[…]` family
(the `@supports` at-rule condition) answers through a host question,
the at-rule tier's second member beside the media forms. Evidence file
per the workspace convention: the RED spec written first, the GREEN
run that satisfies it, the benches and the consumer examples. Two
milestones, one red→green cycle each:

* **M1** — the seam: `Cond::Supports` (the rendered condition text as
  the interned payload, the tier-4 at-rule rank), the fold grammar
  (`variant::supports_condition` — the tailwindcss v4.1.17 SOURCE's own
  normalization, tightened to the honest refusal set), `CssAt::Supports`
  (the at-rule head renders the SAME payload), the `SupportsProbe`
  seam + `SupportsState` built-in + the dedicated
  `InterpNote::SupportsUnqueried` note, the `*_supports` entry-point
  matrix (32 free functions + 32 `Compiled` methods, the full composed
  matrix included), the static-program token emitter's payload arm, the
  GPUI/Floem runtime boundaries, the LSP hover label, and the plan
  bridge's `PlanNoteKind::SupportsCondition` note with the backends'
  report translates;
* **M2** — the CLI face: `expand --supports <condition>` (the fold's
  own grammar validating the facts, the five-seam dispatch matrix, the
  exit-2 refusal gate, the usage text).

## The differential evidence

The corpus carries no `supports-` class, so the pins key on the
tailwindcss v4.1.17 SOURCE itself
(`packages/tailwindcss/src/variants.ts`, the
`functional('supports', …)` arm), fetched and pinned at handoff time:

* function-prefixed (`/^[\w-]*\s*\(/`): the condition passes through
  verbatim, `and`/`or`/`not` connectives spaced (the source's own
  Chrome `(a)or(b)` normalization), whitespace collapsed;
* a colon-bearing declaration wraps: `display:grid` → `(display:grid)`;
* a bare feature is the declaration probe: `display` →
  `(display: var(--tw))` (the source's own shorthand words).

The honesty tightening (aeolus's own, documented in place): the empty
inner, braces, `;`, backslash escapes and unbalanced parens refuse.
The v4 arbitrary-value convention rides the fold: `_` spells the space
a class string cannot carry.

## M1 — the seam

**RED (spec written first, run against the post-0.19.0 tree):**

```text
$ cargo test -p aeolus-compile --test supports
error: could not compile `aeolus-compile` (test "supports") due to 12 previous errors
error[E0432]: unresolved imports `aeolus_compile::SupportsProbe`, … (33 names)
error[E0599]: no variant named `Supports` found for enum `Cond`
error[E0599]: no variant named `SupportsUnqueried` found for enum `InterpNote`
error[E0599]: no variant named `SupportsCondition` found for enum `PlanNoteKind`
```

(the pre-seam behavior itself, pinned by the baseline probe:
`aeolus expand 'supports-[display:grid]:flex'` →
`error: variant "supports-[display:grid]" is outside the typed Cond
surface … [aeolus::compile::unsupported_variant]`)

**GREEN:**

```text
$ cargo test -p aeolus-compile --test supports
test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured

$ cargo test --workspace --all-features
TOTAL passed: 1671 failed: 0
```

**Bench (divan, release, shared sandbox — medians of 100 samples):**

```text
$ cargo bench -p aeolus-compile --bench supports
interp_supports_button     median 114.8 ns   (the capability-table hit)
interp_sourceless_button   median 43.4 ns    (the note path, for the record)

$ cargo bench -p aeolus-cli --bench expand expand_supports
expand_supports_facts      median 229.9 ns   (the --supports walk)
```

**Example:** `cargo run -p aeolus-compile --example supports` — the
fire under facts, the answered miss (quiet), the sourceless note.

## M2 — the CLI face

**RED:** the `--supports` battery in `tests/cli.rs` fails against the
pre-seam CLI (no flag: unknown-argument refusal, exit 2 — the four
pins name the facts' shape the flag then carries).

**GREEN:**

```text
$ cargo test -p aeolus-cli --test cli
test result: ok. 83 passed; 0 failed (the four --supports pins beside)
```

## The boundaries kept

* the v0.x oracle (`aeolus-core`) stays untouched — the family is
  outside its grammar (its variant split is the first-colon split),
  the differential source is the v4.1.17 source itself;
* the compound `not-*` forms stay the diagnosed boundary (the has-
  compound precedent);
* the plan bridge keeps the honest note — the v0.x surface has no
  at-rule slot; the style does not land;
* the sourceless walks keep the dedicated note — the render target's
  capability set is not frame state, the `Env` carries no fallback.
