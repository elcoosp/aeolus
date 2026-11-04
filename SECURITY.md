# Security Policy

## Supported versions

| version | supported |
|---|---|
| 0.1.x | ✅ |

Aeolus is a **style compiler**. It parses untrusted-ish text (Tailwind class
strings, `@theme` CSS) and emits styling values for native UI frameworks.
It does not execute the input, does not perform I/O on its behalf, and the
hot path is designed so that **no input can panic it**:

- the parser returns `(tokens, errors)` — unknown syntax is skipped and
  reported, never fatal;
- the theme parser tolerates arbitrary CSS (malformed tails ignored);
- `cn()` is best-effort and total (any input returns a string);
- fuzz targets pin these properties (`fuzz/`): `parse_class_string`,
  `color_parse`, `cn`, `theme_from_css` each ran 60s clean in
  release+ASAN at the 0.1.0 tag;
- floats are clamped at conversion boundaries, so absurd numeric input
  degrades to clamped colors, never NaN/inf propagation.

## Reporting a vulnerability

- **Preferred**: open a GitHub Security Advisory ("Report a vulnerability")
  on the repository — private by default.
- **Alternative**: email the maintainers at `security@aeolus.rs`
  (placeholder until the project has real infrastructure; see the
  repository page for the current address) with:

  - affected crate(s) and version,
  - a reproducing input (class string / CSS / code),
  - impact assessment from your perspective.

You will get an acknowledgment within 7 days and a status update at least
every 14 days until resolution. We credit reporters in the changelog
unless they prefer otherwise.

## What is (and is not) in scope

**In scope**: panics, memory unsafety, unbounded resource consumption, or
incorrect styling semantics reachable from any input to the public APIs of
the published crates (`aeolus-core`, `aeolus-macro`).

**Out of scope**: the three `publish = false` backend crates in 0.1.0
(`aeolus-xilem`, `aeolus-floem`, `aeolus-shadcn`) track git-pinned upstream
framework commits — report upstream framework issues there; issues in the
dev-sandbox `muda-stub` (it is a build-time artifact and never ships); and
"issues" that the `LoweringReport`/`Warning` machinery already reports by
design (dropped features are contractual behavior, not vulnerabilities).

## Disclosure policy

We coordinate disclosure: fix lands on `main` with a regression test, the
fuzz target corpus gets the reproducer, the advisory publishes on the next
patch release (or immediately for actively exploited issues).
