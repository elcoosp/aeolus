# Phase 5 — release prep (evidence)

## Version bump

Every crate resolves to workspace `0.1.0` (`version.workspace = true` in
all five members; `muda-stub` carries the upstream-pinned 0.17.2 because it
patches the real `muda` crate by name and is `publish = false`).

## Documents added

- `CHANGELOG.md` — Keep-a-Changelog `0.1.0` section: Added/Fixed per phase
  + the explicit *Deferred to 0.2* list.
- `SECURITY.md` — disclosure channels (GitHub advisory + email, 7-day
  ack), scope limited to the published crates, the panic-free contract
  and its fuzz evidence.
- `CONTRIBUTING.md` — the mission's TDD rules restated verbatim in intent
  (failing test FIRST, never delete a failing test, conventional commits,
  no silent failures, no shortcuts, zero-dep guardrail, verbatim class
  tables) + the local gate list.
- `README.md` — rewritten to match reality: what ships (87 tests), what
  is deferred, exact build/package/fuzz commands, architecture summary.
- `reports/09-release/RELEASE_CHECKLIST.md` — every verification step
  with its observed result.

## Final verification (the success-definition battery)

All commands were re-run at the release commit; results recorded in
`reports/09-release/RELEASE_CHECKLIST.md`. Headline:

```text
cargo test --workspace --all-features        87 passed / 0 failed
cargo clippy --workspace --all-targets -- -D warnings   green
cargo fmt --check                            clean
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps   green
cargo package -p aeolus-core -p aeolus-macro --allow-dirty --no-verify   both packaged
cargo deny check licenses                    licenses ok
tag v0.1.0                                   on the final commit
```

Design one-liner: the release documents deliberately name the 0.2
deferrals (backend publishing, group/peer layers, stack registration,
Masonry typography/responsive/dark, container queries, transitions) so
v0.1.0 overpromises nothing — the honest-release principle.
