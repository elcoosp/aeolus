# feat: p2-theme-pass — the theme pass, tiered tokens

## Milestone — P2's second slice: the theme half of the compile signature

The V1 pipeline's first seam (`aeolus-compile`) landed with one honest
boundary test-pinned: the registry's colour keyword table is 270
entries and the IR's `Val::Kw` slot is a u8, so colour tokens refused
typed instead of truncating. This change lands the slice every seam
said was next (architecture plan §3.2/§3.4): **`aeolus-theme`**, the
CSS-first theme pass —

```rust
ThemeSource::from_css(light, dark)?        // the web project's own file
let tokens = TokenTable::runtime(&source)? // T2: Tok { id, alpha }
let tokens = TokenTable::baked(&source)?;  // T0 literals / T1 pairs
compile_theme(input, &tokens, &Config)?;   // the signature's theme half
```

— and with it `bg-red-500` lowers, `/90` becomes one premultiplied
multiply (the audited `mix_with_transparent` identity, corpus-pinned),
and `dark:bg-primary` forms the T1 const pair. The tiers are exactly
the plan's §3.4 table: T0 literals, T1 light/dark pairs, T2 runtime
slots — pay only for the dynamism you use. The theme-less `compile`
keeps the v0.10.0 contract byte-for-byte (pinned), and nothing in the
v0.x surface changes: `aeolus-theme` wraps the audited v0.x reader as
the interim oracle, exactly the differential discipline P1 used.

### What changed

- **`aeolus-theme`** (git-only, `publish = false`, default graph
  aeolus-core + aeolus-ir only): `ThemeSource::from_css`/`shadcn`/
  `shadcn_dark` parse through `TailwindTheme` (fuzz-pinned by
  `color_parse`/`theme_from_css`; `dark_variant` propagates
  `@theme inline` chains into the dark variant); `TokenTable::baked`/
  `runtime` snapshot the colours with sorted-name STABLE ids (ids ride
  compiled `Program`s — a contract, test-pinned); `resolve(id, alpha)`
  emits the tier's `Val` and refuses typed on unknown ids, non-finite
  alphas and an alpha on a T1 pair slot. The u16 `Tok` slot is the
  budget fix the u8 `Kw` discovery forced; the u16 `ModePair` pair
  table is guarded at build the same way. Two OFF-by-default
  instrumentation features land with the crate, the workspace's
  pattern: `tracing` (build/resolve spans) and `miette` (code + help on
  `ThemeError`/`ResolveError`).
- **The compile seam gains its theme half**: `compile_theme(input,
  &TokenTable, &Config)` (plan §3.2's second argument; the `Target`
  half stays future). The value pass routes the oversized colour table
  through the supplied table (tier-aware); the `/opacity` modifier
  normalizes once per candidate (`"90"` → 0.9, `"[0.37]"` → 0.37) and
  rides COLOUR-form decls only — a modifier on a non-colour lowering
  (`text-sm/6`) or on a baked dark pair stays the typed
  `UnsupportedModifier` boundary with the raw text, never a
  half-applied alpha. Two new reasons join the diagnostic set
  (`not-a-token`, `non-finite`); the theme-less path is pinned
  byte-identical to v0.10.0, and a registry-level miss
  (`bg-not-a-token`) stays the lookup's `ShapeRejected` — the theme
  pass does not shadow the registry read side.
- **Micro-debt cleanup (CONTRIBUTING §2/§3), both verified by
  reading, no toolchain needed:**
  - `crates/aeolus-compile/benches/compile.rs` could not BUILD: four
    `unwrap_or_default()` calls on `Result<Compiled, CompileError>`
    while `Compiled` has no `Default` — `cargo test --benches` /
    `cargo bench -p aeolus-compile` failed to compile, so the
    "gates green pre-handoff" claim broke at the bench gate. The bench
    bodies `expect` like the examples do (a non-empty string always
    produces a `Compiled`; only the empty input hard-fails), and the
    bench doc records why no `Default` is deliberate.
  - The f32 grammar's non-finite spellings lowered SILENTLY into the
    IR: `p-nan`, `z-inf`, `w-1e40` are legal `f32::from_str` parses
    (`nan`/`inf`/`infinity`, any case, and overflow-to-inf), the
    spacing/number shape contracts accepted them, and `Val::Rem(NaN)` /
    `Val::Num(inf)` reached the program — a no-silent-failures
    violation. The value pass now refuses them typed
    (`REASON_NON_FINITE`) on every numeric arm (spacing, the negated
    forms, fractions, percents, numbers, and the px bake's overflow
    against a large `rem_px`), with the rest of the string still
    compiling; pinned end to end.
- **Docs drift, everywhere it was found:**
  - The README's hot-path bullet said CI fuzzes "four of them" while
    CI has swept all six targets since v0.10.0 (`3794b82`); it now says
    six. CONTRIBUTING's local fuzz smoke listed 4 of the 6 targets;
    it lists all six.
  - The CHANGELOG carried TWO `## [0.10.0]` headers (a git-cliff block
    dated 2026-11-10 at the top AND a prose section dated 2026-11-09)
    plus an `[Unreleased]` still describing the already-released
    compile seam — consolidated into one `[0.10.0]`; the new
    `[Unreleased]` is this change.
  - The architecture plan's P2 status block and §8 row record the
    theme pass's landed reality (and mark the colour-table boundary
    RESOLVED by the u16 `Tok` slot); the README's crate table, the
    consumer-examples block (nine → eleven runnable examples), the
    status paragraph and the test counts follow the tree (535 tests;
    542 with the instrumentation features — the authored +26 ungated
    and +3 gated, added to v0.10.0's 509/513).
  - The stale `golden.json` fixture (the v0.4-era 157-entry artifact;
    the harness reads `golden.corpus`, 193 `@entry` lines) is left as
    the generator's own output with this note: regenerating it is the
    tool's job (`tools/gen_golden.mjs`), not a hand edit.
- **TDD order preserved; gates capture at handoff.** The suites were
  authored FIRST as the RED specs (15 theme-side tests incl. the
  crate's inline id-order pin + 12 compile-side, 3 of them gated), the
  implementations second, the bench and the two consumer examples
  (`examples/theme_tiers.rs`, `examples/theme_seam.rs`) last. This
  handoff's sandbox has no Rust toolchain, so the RED→GREEN capture
  and the gate outputs below run with the first cargo invocation in
  the gate environment; the `Cargo.lock` entries (`aeolus-theme` as a
  member, `aeolus-compile`'s new dep edge) are already in place so the
  run is `--locked`-clean.

### Gates (static verification in the no-toolchain sandbox; executable capture at handoff)

- `cargo fmt --all --check` — **not run here** (no cargo; the task
  forbids toolchain execution in this sandbox). Authored to rustfmt
  style; capture at handoff.
- `cargo clippy --workspace --all-targets --all-features -- -D
  warnings -W pedantic` — **not run here**. Statically: pedantic is
  enabled by the workspace `[lints.clippy]` table (`pedantic = warn`,
  priority −1) and every exception lives in that manifest table with
  its rationale — a per-lint exception always outranks its group in
  rustc's lint resolution regardless of flag order, so the manifest's
  exceptions hold after the CLI's `-W pedantic` (and the group levels
  agree: warn either way). The zero-allowance contract is grep-verified
  below; the empirical override check (and, if a future clippy ever
  flips the order, moving `-W pedantic` ahead of `--`) stays part of
  the gate capture.
- Allowance grep (RUN here, it is not a toolchain execution):
  `grep -rnE '#!?\[(allow|expect)\(|cfg_attr\([^)]*_allow\(' src/
  crates/ tests/ benches/ examples/ build.rs xtask/` → **empty** (the
  new crate and suites included; zero allowances in code, all
  exceptions in the manifest tables with rationale).
- `cargo test --workspace --all-features` — **not run here**. The net
  new is 26 ungated tests (15 theme-side incl. the inline id pin + 11
  compile-side) plus 3 gated (2 theme, 1 compile): the suite count
  moves 509 → 535 (542 with the instrumentation features). Every
  test's registry rows, tables and token names were cross-checked
  against the committed data by hand (the 270-entry colour table
  carries the shadcn app names — `bg-primary` is shape-legal; the
  TRBL orders; the sorted-id contract).
- `cargo test --benches --all-features` — **not run here**. All
  benches in the workspace remain divan and stable; the new bench adds
  no nightly gating. **No nightly-only benches exist** — the only
  nightly surface is the `cargo-fuzz` job (six targets), which is
  CI-gated and not a bench.
- Feature matrix: **no mutually-exclusive features exist anywhere in
  the workspace** (cli/data/compile/theme carry the additive
  `tracing`/`miette` optional-dep switches; macro's
  `verbose-diagnostics` is additive) — so `--all-features` is safe and
  there is no `ci-bundle` to assemble. The one deliberately
  non-portable wiring remains `[patch.crates-io] muda = { path =
  "crates/muda-stub" }` — the dev-sandbox stub that must never ship
  (PUBLISHING.md), unchanged here.
- Cargo.lock: the `aeolus-theme` member entry (version 0.10.0, deps +
  optionals + dev-dep, alphabetical order preserved) and
  `aeolus-compile`'s new dep edge are in place so `cargo test
  --locked` resolves before the first update.

### Per-file history lives on the branch

Merging is `--rebase` with **one commit per logical change** landed as
Conventional-Commit commits on `feat/p2-theme-pass` (the crate feat
carrying its workspace wiring and lock entry, the test pins, the
compile-seam feat, the bench fix, the examples feat, the token bench,
the evidence record, the docs alignment, the pr-body chore) — the
grouping is declared in `ship.toml` at the repo root, so the per-file
history survives the rebase onto `origin/main`; the review shows the
same tree this PR body describes. **No commits have been made on the
base branch for this work** — the whole change ships as the branch's
commits authored by ship from the dirty tree it was handed.

### Verify locally

```sh
cargo run -p aeolus-theme  --example theme_tiers
cargo run -p aeolus-compile --example theme_seam
cargo test --workspace --all-features
cargo test --benches --all-features
cargo bench -p aeolus-theme
```
