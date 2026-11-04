# Phase 4 — hardening (TDD + fuzz evidence)

## 1. Property tests (`proptest`, dev-dependency only)

`crates/aeolus-core/tests/properties.rs` — 4 properties × 512 cases:

1. `cn_is_idempotent` — `cn(cn(x)) == cn(x)` for arbitrary class soup.
2. `parser_roundtrip` — `parse(serialize(parse(x))) == parse(x)` via a
   test-local serializer (aeolus-core deliberately has none).
3. `oklab_srgb_roundtrip` — tight (1e-3) in-gamut; finiteness everywhere.
4. `opacity_preserves_hue` — v4 color-mix hue preservation + alpha scaling.

**The property suite caught a real bug immediately** (RED verbatim):

```text
thread 'oklab_srgb_roundtrip' panicked:
Test failed: assertion failed: (c.l - c2.l).abs() < 1e-3 at crates/aeolus-core/tests/properties.rs:128.
minimal failing input: l = 0.0, a = 0.0, b = 0.2738206, alpha = 0.0
```

Root cause: `Color::to_srgba`'s doc PROMISED clamping but the code never
clamped — out-of-gamut Oklab produced negative linear components and
`powf(1/2.4)` turned them into NaN. Fix: clamp linear components to 0..=1
before the transfer function (+ `from_srgb` input clamp as defense). The
property was then refined to assert finiteness for ALL colors and the tight
roundtrip only for provably in-gamut ones (clamping is lossy by design).

GREEN: `test result: ok. 4 passed; 0 failed` (2.8s), and the whole suite
stayed green (31→33 core lib tests including the two new unit tests).

## 2. Fuzz targets (`fuzz/`, cargo-fuzz + libFuzzer)

Four targets, one per mission bullet:

| target | asserts |
|---|---|
| `parse_class_string` | every whitespace word is either parsed or reported (no panics, no lost words) |
| `color_parse` | any parsed color converts to FINITE sRGB (no NaN/inf) |
| `cn` | merge never panics AND is idempotent on every input |
| `theme_from_css` | arbitrary CSS never panics the theme parser; parsed themes answer all lookups |

Runs (release, `-max_total_time=60`, ASAN, nightly):

```text
parse_class_string  Done 1194854 runs in 61 second(s)
color_parse         Done 7264605 runs in 61 second(s)
cn                  Done  540657 runs in 61 second(s)
theme_from_css      Done    6309 runs in 61 second(s)
```

(all clean after the fixes below; initial runs crashed in 3 of 4 targets.)

### Fuzz findings → TDD fixes (each: unit test RED first, then fix)

1. **`parse_hex` byte-slice panic** (`color_parse`):
   `thread panicked at color.rs:79: end byte index 4 is not a char boundary`
   — hex strings of the right BYTE length but multibyte content
   (`#abᐁ` = 6 bytes) sliced mid-char. Fix: `is_ascii` gate.
2. **`cn` idempotence broken on whitespace-only classes** (`cn`):
   `cn must be idempotent on "\n"` — output must be a normalized word
   sequence: 0 words → `""`, 1 word → the word, unparseable words kept
   verbatim but whitespace-normalized. Fix in `cn`'s fast path + fallback.
3. **`extract_declarations` out-of-bounds slice** (`theme_from_css`):
   `start byte index 3 is out of bounds for string of length 2` — CSS
   ending in an unbalanced block (`matching_brace` returns the length).
   Fix: early return when `close >= rest.len()`; malformed tail ignored.
4. **Absurd alpha → f32 inf** (`color_parse`): `rgb(1 2 2/1e95)` — CSS
   clamps alpha to [0,1]; `split_alpha` clamps now, `to_srgba` defends.
5. **Huge hue → NaN** (`color_parse`): `oklch(3 3 1.9e83)` — `cos`
   argument reduction loses all bits at f32; hue clamped to ±360°.
6. **Chroma overflow → inf cubes** (`color_parse`, folded into the
   to_srgba Oklab-range clamp): `oklch(3 3.3e40 1)` overflowed the
   Oklab→linear matrix cubes; chroma axes clamped to ±0.5 (beyond any
   real gamut).

## 3. CI (`.github/workflows/ci.yml`)

Jobs: `fmt --check` · `clippy --workspace --all-targets -- -D warnings` ·
`cargo test --workspace` (ubuntu/macos/windows matrix) · `cargo doc
--workspace --no-deps` with `RUSTDOCFLAGS=-D warnings` + `cargo test --doc
-p aeolus-core` · `cargo package -p aeolus-core -p aeolus-macro
--allow-dirty --no-verify` · `cargo deny check licenses` · 60s-per-target
fuzz smoke (main pushes + weekly cron).

Local equivalents all green at this commit:

```text
cargo clippy --workspace --all-targets -- -D warnings   Finished
cargo fmt --check                                       clean
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps   Finished
cargo test --doc -p aeolus-core                         1 passed
cargo deny check licenses                               licenses ok
cargo test --workspace                                  87 passed
```

`deny.toml`: the license allow list is ENUMERATED from the dependency
graph via cargo-deny (not guessed): MIT, Apache-2.0 (+LLVM-exception
variant), BSD-2/3-Clause, ISC, Unicode-3.0, Zlib, BSL-1.0, Unlicense,
CC0-1.0, 0BSD, MPL-2.0, BlueOak-1.0.0. Git sources restricted to the two
pinned framework repos. LGPL appears only inside OR-duals and is
intentionally not accepted.

## 4. Doc examples

The `aeolus-core` crate-level example is now runnable (```no_run →
```), so `cargo test --doc -p aeolus-core` executes and passes it.
Remaining `ignore` examples live in UI-crate docs (macro compile-fail
demo, xilem view sugar needing an app runtime) — genuinely not runnable
in a library doc-test, left as documented.

## 5. Benchmarks

Re-run at this commit (see `reports/08-benchmarks/BENCHMARKS.md` for the
table): hot paths unchanged — warm cache hit 405 ns, cold resolve 4.6 µs
(3.7 Mitem/s), diff 505 ns (≈4% faster via approximate color equality);
theme parse +15% (231 µs, one-time cost) from calc precedence + fuzz
hardening — accepted.

Design one-liner: the fuzz corpus lives in `fuzz/` (not committed —
regenerable); targets are dependency-free wrappers over the public APIs
so a CI crash reproduces with plain unit tests.
