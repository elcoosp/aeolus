# Publishing — what ships in v0.1.0 and what is gated

Status: **aeolus-core and aeolus-macro are publishable today** (`cargo
package` succeeds, no local path dependencies leak into their manifests).
The three backend crates are `publish = false` in v0.1.0, with the reason
recorded here.

## Phase-1 decision record (options a / b / c)

The mission offered three routes to remove the `[patch.crates-io] muda`
blocker. Findings from the audited sources:

- **(a) — impractical.** `lapce/floem @ 1351ffb` declares, for Linux,
  `muda = { workspace = true, default-features = false, features = ["gtk"] }`
  in a `[target.'cfg(target_os = "linux")'.dependencies]` section with **no
  feature gate of its own** (`floem/Cargo.toml`, target-specific section).
  There is no no-GTK feature path: `default-features = false` only removes
  `editor`/`vger`/`serde`/image formats — the muda/gtk Linux dependency is
  unconditional. (The backend notes in `reports/sections/floem-backend-notes.md`
  already recorded the related fact that Floem also hard-requires its default
  renderer features.)
- **(b) — impractical.** A "minimal muda fork with the gtk feature removed"
  cannot be applied per-target: `[patch.crates-io]` is not target-gateable,
  so a stub fork would replace real muda on *every* target (silently
  disabling real menus on Windows/macOS), and a *real* fork means vendoring
  and maintaining muda's GTK-free Linux backend — out of scope for v0.1.0.
- **(c) — chosen.** Publish only `aeolus-core` + `aeolus-macro` in v0.1.0;
  `publish = false` on the three backend manifests; keep the muda-stub patch
  as a **documented dev-sandbox artifact** (it lives in `crates/muda-stub`,
  is not a workspace member, and no publishable crate depends on it).

## What is publishable now

| crate | version | `cargo package` | notes |
|---|---|---|---|
| `aeolus-core` | 0.1.0 | ✅ | zero dependencies; `#![deny(missing_docs)]`; `MIT OR Apache-2.0` license files present at repo root |
| `aeolus-macro` | 0.1.0 | ✅ | depends only on `aeolus-core` (version + path) |

Commands (as verified on the release branch):

```sh
cargo package -p aeolus-core -p aeolus-macro --allow-dirty --no-verify
```

Publishing-order caveat: `cargo package -p aeolus-macro` **alone** resolves
`aeolus-core` from the crates.io index and therefore fails until
`aeolus-core` is actually published (`error: no matching package named
aeolus-core found`). This is inherent crates.io behavior, not a defect:
publish `aeolus-core` first, then `aeolus-macro`. The two-crate invocation
above is the verification used for v0.1.0.

## What is gated to v0.2 (and why)

| crate | blocker | unblocking condition |
|---|---|---|
| `aeolus-xilem` | depends on git-pinned Masonry/Xilem commits (crates.io rejects git deps); the pinned commits are pre-release upstream state | upstream cuts a crates.io release of `masonry`/`xilem_masonry` compatible with the pinned behavior |
| `aeolus-floem` | same (git-pinned `lapce/floem @ 1351ffb`) **plus** the muda/gtk hard-dependency documented above | upstream makes muda optional on Linux, or drops it; then also remove the dev-only `muda-stub` patch |
| `aeolus-shadcn` | depends on both backend crates | whichever of the two unblocks last |

## Repositories / reproducibility

All three backend manifests reference upstream frameworks by **pinned git
rev** (not local paths), so the workspace builds with `repos/` deleted —
verified: with `repos/` renamed away, `cargo test -p aeolus-core
-p aeolus-macro` is green. The `repos/` directory in the original research
setup is no longer referenced by any manifest.

## Dev-sandbox artifacts (must never be published)

- `crates/muda-stub` — no-op, API-compatible stand-in for `muda` 0.17 on
  Linux, where floem hard-requires the `gtk` feature (GTK3 dev headers are
  unavailable in the build sandbox). Wired via `[patch.crates-io]` at the
  workspace root. Not a workspace member; covered by the rationale block in
  `crates/muda-stub/src/lib.rs`.
- The three backend crates carry `publish = false` as a hard guard, so even
  an accidental `cargo publish` cannot ship them (or transitively the patch)
  to crates.io.
