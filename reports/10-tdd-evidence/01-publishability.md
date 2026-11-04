# Phase 1 — publishability (TDD evidence)

Phase 1 is mostly manifest/build work; its "tests" are the acceptance
commands themselves. Each item below records the RED state observed first,
then the GREEN state after the change.

## 1. `cargo package -p aeolus-core` / `-p aeolus-macro`

RED (baseline `main`, before any change — observed while reading the tree;
the workspace did not even load without `repos/` present):

```text
error: failed to load manifest for workspace member `.../crates/aeolus-shadcn`
referenced by workspace at `.../Cargo.toml`

Caused by:
  failed to load manifest for dependency `aeolus-floem`

Caused by:
  failed to load manifest for dependency `floem`

Caused by:
  failed to read `/home/z/my-project/work/repos/floem/Cargo.toml`

Caused by:
  No such file or directory (os error 2)
```

RED (first packaging attempt after the git-dep switch; macro alone against
an unpublished core):

```text
error: failed to prepare local package for uploading

Caused by:
  no matching package named `aeolus-core` found
  location searched: crates.io index
  required by package `aeolus-macro v0.1.0`
```

GREEN (both crates packaged together; core-first ordering documented in
PUBLISHING.md):

```text
   Packaging aeolus-core v0.1.0 (/home/z/my-project/work/aeolus/crates/aeolus-core)
    Updating crates.io index
    Packaged 19 files, 189.0KiB (49.4KiB compressed)
   Packaging aeolus-macro v0.1.0 (/home/z/my-project/work/aeolus/crates/aeolus-macro)
    Updating crates.io index
    Packaged 5 files, 6.9KiB (2.9KiB compressed)
```

## 2. `#![deny(missing_docs)]` fallout

RED (224 missing-doc errors after adding the deny to `aeolus-core`):

```text
error: missing documentation for a struct field
   --> crates/aeolus-core/src/plan.rs:86:5
...
error: could not compile `aeolus-core` (lib) due to 224 previous errors
```

GREEN (every public item documented):

```text
    Checking aeolus-core v0.1.0
    Finished `dev` profile [unoptimized] target(s) in 0.43s
```

Justification: a publishable crate's rustdoc is its first API contract;
missing docs on slot payloads were the largest doc gap.

## 3. `repos/` independence

RED (baseline: with `repos/` removed the workspace did not even load —
error quoted in §1 above).

GREEN (after switching the three backend manifests to pinned git deps;
`repos/` renamed away):

```text
test result: ok. 22 passed; 0 failed; ... (aeolus-core)
test result: ok. 14 passed; 0 failed; ... (aeolus-core shadcn integration)
test result: ok. 1 passed;  0 failed; ... (aeolus-macro)
```

## 4. Acceptance commands (final state of this phase)

- `cargo test -p aeolus-core -p aeolus-macro` — green with `repos/` empty ✅
- `cargo package -p aeolus-core --allow-dirty --no-verify` — succeeds ✅
- `cargo package -p aeolus-macro --allow-dirty --no-verify` — succeeds when
  packaged together with core (publish-order caveat in PUBLISHING.md) ✅
- `reports/09-publishing/PUBLISHING.md` exists, option (c) documented ✅

Design choice one-liners:
- git revs over local paths: reproducible builds + independence from a
  sibling checkout; revs are the exact commits the audit cites.
- Option (c) over (a)/(b): floem has no no-GTK feature path on Linux and
  `[patch]` cannot be target-gated — documented with file citations in
  PUBLISHING.md.
