# TDD evidence — v0.4 parity proof (deliverable 10.6)

Plan of record: `AEOLUS_V0.4.md`. Workflow per `CONTRIBUTING.md`: failing
test FIRST → RED captured → implement → GREEN captured → the whole suite
stays green. Every command below was run in the working tree at the
recorded commits.

## 1. The RED state (commit `bec9c03`)

The corpus generator (`tools/gen_golden.mjs`) ran the REAL tailwindcss
4.1.17 over 157 classes across two themes and wrote the verbatim rules to
`crates/aeolus-core/tests/fixtures/golden.corpus`. The harness
(`tests/parity_golden.rs`) was written against the intended API — which
did not exist:

```
$ cargo test -p aeolus-core --test parity_golden
error[E0599]: no method named `to_css` found for struct `Resolver<'t>` in the current scope
  --> crates/aeolus-core/tests/parity_golden.rs:654:34
error: could not compile `aeolus-core` (test "parity_golden") due to 1 previous error
```

A single, pure RED: the harness compiles except for the missing
`Resolver::to_css` — the emitter is the whole delta.

## 2. GREEN iterations (commits `7451746` → `10ce952` → `e19be50`)

The harness failure output IS the TDD loop: each run printed per-entry
diffs (golden vs ours, per declaration). The significant cycles:

1. **First run: ~30 failures.** Biggest classes: the `--tw-*`
   shadow/ring machinery needed composition into final `box-shadow`
   values (harness), element-relative line-height needed the
   same-rule font-size, and golden rules needed cascade-order flattening
   (nested `@supports` overrides the plain fallback).
2. **Corpus-caught resolver bugs (fixed in the RESOLVER, not the
   emitter — the honesty contract):**
   - `-top-2` / `-translate-y-1` lost the sign (`inset_value` /
     `transform_size` numeric arms ignored `t.negative`);
   - `rounded-t-full` resolved 6px (`--radius-full` is not a v4 token;
     it is `calc(infinity * 1px)`) — now the 9999px sentinel, emitted
     v4-shaped; `rounded-{edge}-none` resets to 0.
   Each fix is pinned by its corpus entry.
3. **Emitter shape cycles:** per-edge stems derived from the token
   (`px-2` → `padding-inline`, `border-t` → `border-top-width` — the
   elements are dash-split so the head alone is ambiguous), scale axes
   compose to the `--tw-scale-*` initial `1`, `!important` suffix for
   both v4 spellings, inert ring variants (`ring-{color}` alone,
   `ring-inset` alone) emit nothing — matching v4's isolated rules.
4. **GREEN:**

```
$ cargo test -p aeolus-core --test parity_golden
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; ... finished in 0.03s
```

157/157 entries at parity; zero warnings asserted per entry (the corpus
only covers the supported surface, per plan §2.3 — element-relative
`tracking-*`/bare `leading-*` stay out and are documented in the harness
header).

## 3. No-regression battery

```
$ cargo test --workspace          # 26 test binaries, all ok (167 → 181 tests with the emitter units)
$ cargo clippy --workspace --all-targets -- -D warnings   # 0 errors
$ cargo fmt --check               # clean
$ RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps  # clean
$ cargo package -p aeolus-core -p aeolus-macro --allow-dirty --no-verify  # packaged
$ cargo deny check licenses       # licenses ok
```

## 4. Fuzz

The v0.3 targets are unchanged and cover the parser/color/cn/theme
surfaces; the emitter consumes only already-validated resolutions (no
new parsing), so no new target was added. `selector.rs`'s panic-free
contract (v0.3 §3.3) is untouched.
