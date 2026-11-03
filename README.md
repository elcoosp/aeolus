# Aeolus — deliverables

Backend-agnostic Tailwind CSS v4 lowering compiler for native Rust UI
frameworks. Built per the AEOLUS master briefing: every framework claim is
sourced from the audited repos (commit hashes recorded), all locked
decisions honored, Q1–Q10 answered from source.

**Verification status**: `cargo test --workspace` → **62 passed / 0 failed**
(aeolus-core 36, aeolus-xilem 12, aeolus-floem 7, aeolus-shadcn 6, macro
+ doc tests), zero compiler warnings. `cargo search aeolus` → name
**available** (0 crates named aeolus on crates.io, checked 2026-11-03).

## The eight deliverables

| # | deliverable | path |
|---|---|---|
| 1 | Source audit + Q1–Q10 answers | `reports/01-source-audit/` (SOURCE_AUDIT.md, QA.md; per-repo detail in `reports/sections/`) |
| 2 | Backend capability matrix | `reports/02-capability-matrix/CAPABILITY_MATRIX.md` |
| 3 | aeolus-core design | `reports/03-aeolus-core-design/DESIGN.md` |
| 4 | Xilem backend prototype + proofs | `crates/aeolus-xilem/` + `reports/sections/xilem-backend-notes.md` |
| 5 | Floem backend prototype | `crates/aeolus-floem/` + `reports/sections/floem-backend-notes.md` |
| 6 | Shadcn v1 (8 components, v4 verbatim) | `crates/aeolus-shadcn/` + `reports/06-shadcn-v1/SHADCN_V1.md` |
| 7 | Upstream patch proposals | `reports/07-upstream-patches/PATCHES.md` |
| 8 | Benchmark report | `reports/08-benchmarks/BENCHMARKS.md` |

Workspace: `crates/` (`aeolus-core`, `aeolus-macro`, `aeolus-xilem`,
`aeolus-floem`, `aeolus-shadcn`); `Cargo.lock` included. `aeolus-core` and
`aeolus-macro` are framework-free and build anywhere. The two backend crates
reference the audited framework checkouts by **sibling-relative path** — to
build them, clone the audited commits into a `repos/` folder next to this
one (no manifest edits needed):

```sh
mkdir repos && cd repos
git clone --depth 1 https://github.com/linebender/xilem
git -C xilem checkout 9f504ae71aa2db1f64c2ecb7ee10cc877d9fc405
git clone --depth 1 https://github.com/lapce/floem
git -C floem checkout 1351ffb162faeb8be983f1301d718a0d5fb59c27
```

## Headline findings (details in the reports)

1. **The briefing's premise was false — and better.** Xilem views cannot
   register `PropertyStack` layers (`Prop` sets *local* props, which shadow
   the stack); stacks come from theme/host/widget code only. Masonry
   nonetheless already has a real cascade with `Selector` pseudo-flags and a
   relevance-gated cache — the Xilem backend lowers to it, with a small
   honest fallback widget (`StyledWidget`) until upstream gap #4 lands.
2. **Floem is the best v1 target**: fully programmatic rule API (10
   selectors + responsive + dark), transitions, zero-view-work lowering —
   proven by tests that drive its public `apply_selectors` evaluator.
3. **`has_hovered` already exists** in Masonry (Q2) — group-hover needs a
   ~150-LOC patch, not a new mechanism.
4. **Iced's `Catalog` is mechanical** to generate from a `StylePlan` (map
   per family), but carries per-instance cost and construction-property
   leakage (padding/font/size live outside `Style`).
5. **Shadcn v4 class tables** ship verbatim from the official registry
   (`4aaea19`); unknown variants (`has-[…]`, `data-[…]`) now warn-and-skip so
   real Shadcn strings keep styling.
6. **Numbers**: warm resolve ~405 ns, cold ~4.6 µs (3.7 Mitem/s), full-plan
   diff ~520 ns (report 08).
