# Release checklist — v0.1.0

Every step below was RUN on the release branch before tagging. Date:
2026-11-04. Toolchain: stable 1.99.1 (MSRV 1.88 declared in every
manifest), nightly used only for the sanitizers.

## Verification steps (all green)

| # | step | result |
|---|---|---|
| 1 | `cargo test --workspace` | **87 passed / 0 failed** (was 62 at baseline) |
| 2 | `cargo test --workspace --all-features` | green (workspace declares no feature flags beyond macro `verbose-diagnostics`, which is additive) |
| 3 | `cargo clippy --workspace --all-targets -- -D warnings` | green, zero warnings |
| 4 | `cargo fmt --check` | clean |
| 5 | `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` | green |
| 6 | `cargo test --doc -p aeolus-core` | 1 doc-test, passes (crate example is runnable) |
| 7 | `cargo package -p aeolus-core -p aeolus-macro --allow-dirty --no-verify` | both packaged (core 19 files / 189.0KiB, macro 5 files / 6.9KiB) |
| 8 | `cargo test -p aeolus-core -p aeolus-macro` with `repos/` REMOVED | green (workspace needs no sibling checkouts) |
| 9 | `cargo deny check licenses` | licenses ok (allow list enumerated from the graph in `deny.toml`) |
| 10 | Fuzz: 4 targets × 60s (release+ASAN) | clean — `parse_class_string` 1.19M, `color_parse` 7.26M, `cn` 540K, `theme_from_css` 6.3K execs |
| 11 | Property suites (4 × 512 cases) | green |
| 12 | Baseline integrity: the original 62 tests all still pass unchanged | ✅ (no test was deleted or weakened; 25 were added) |

## Track decision (Phase 3)

**Track A** — real interactive components: `StyledButton` (pointer /
keyboard / accesskit / disabled, acceptance-tested through Masonry's
`TestHarness`) and `input` (Masonry `TextInput`, controlled-value).
Documented in `reports/10-tdd-evidence/03-real-widgets.md`.

## Pre-tag audit

- [x] Version 0.1.0 in the workspace and every crate manifest
- [x] `CHANGELOG.md` (Keep-a-Changelog, shipped vs deferred)
- [x] `SECURITY.md` (disclosure address + scope)
- [x] `CONTRIBUTING.md` (TDD rules restated)
- [x] `README.md` rewritten to match reality
- [x] `LICENSE-MIT` + `LICENSE-APACHE` present, referenced by manifests
- [x] `reports/09-publishing/PUBLISHING.md` — what publishes, what is
      gated and why (option (c) with source citations)
- [x] `reports/10-tdd-evidence/` — one file per phase (01..04, 05 this
      file), RED+GREEN verbatim
- [x] `.github/workflows/ci.yml` — the full mission gate list
- [x] `deny.toml` — licenses enumerated from the dependency graph
- [x] Tag `v0.1.0` on the final commit of `feat/v0.1.0-production`
- [x] Zip + git bundle delivered to the downloads folder, `unzip -t` clean

## Known limitations carried into 0.2 (by design, all reported at runtime)

See `CHANGELOG.md` → *Deferred to 0.2* and the capability matrix. Nothing
on that list is silently dropped: each limitation surfaces through
`LoweringReport` entries or `Warning` values.
