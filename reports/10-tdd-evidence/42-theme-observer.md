# 42 — The theme-resolution observability face

The theme stage's own seam. The compile crate's two faces answered
"what did the cache decide" (`DynObserver` under the composed
`session_key`) and "what did the walk do" (`WalkObserver` over the
stage boundaries) — and the theme stage the lowering's colour path
consults per declaration answered nothing: the `tracing` feature's
spans are the stringly channel, and a host reading a themed session
had no typed per-consult telemetry. The landing adds the house's face
to the one V1 layer without one, with the rules kept exactly: every
trait method a no-op default (the seam adds a channel, never replaces
one), `NoThemeObserver` the every-constructor default (both table
constructors start observation-free), the `Arc` blanket the two faces
keep (one handle totals several tables — the original, a clone, a
second table), `ThemeStats`/`ThemeSnapshot` the built-in collector
(the two channels total separately), and the events the table's OWN
answers: `on_resolve(id, alpha, ResolveOutcome)` the tier arm that
answered (`TokCarried` / `Literal` / `ModePair`) or the typed refusal
verbatim (`Missed(ResolveError)` — the same error the call returns; a
miss IS an event), `on_resolve_rgba(id, alpha, dark, RgbaOutcome)`
the runtime path's mode answer (`ModeHalf` / `Literal`). The wire-up
rides the table itself — `TokenTable::with_observer` chains after any
constructor, the events fire after the answer is computed and before
the call returns, in caller order — so the compile walk's lowering
consults stream through a wired table with ZERO call-site changes
upstream: a table wired before `compile_theme` reports the theme
stage's per-declaration verdicts DURING the walk, the stage boundary
the walk face's events sit above. The seam is answer-NEUTRAL — the
observed table's resolutions are the plain table's, which the
differential pins per id per alpha. The `TokenTable` Debug prints the
wired-ness only (the observer is a host handle, not printable state —
the cache Debug's own rule; `Clone` keeps the wiring, the suite pins
the clone's stream).

* **M1** (the whole landing): the `observer` module (trait + two
  outcome enums + default + blanket + collector + snapshot), the
  `TokenTable` wire-up (`with_observer`, the observer slot defaulting
  `None` on every construction path, the manual `Debug`), the
  red-first suite, the cross-layer battery in the compile crate, the
  bench, the consumer example, the README/architecture-doc/changelog
  rows.

## The shape evidence

The events report decisions the table already made — the outcome is
decided AT the return site (each arm returns its own verdict beside
the answer; no `Val` re-matching, no invented arms), so an event can
never disagree with the call's `Result`. The misses stream TYPED and
verbatim (`UnknownId`, `AlphaOnPair`, `AlphaNotFinite` ride the
`Missed` arm; a refused resolve never counts into a value arm — the
counts answer the artifact's own partition). The rgba channel decides
`ModeHalf` vs `Literal` by the tier (the runtime table's active-half
load, the baked table's light-only literal), the `dark` flag riding
the event as the caller's request. The cross-layer ride is the point:
the compile crate's battery builds the wired table, runs the themed
compile, and pins BOTH the snapshot split (T2 carries / T1 pairs + the
alpha-on-pair refusal riding `Missed`) AND the byte-identity of the
compiled artifact against the plain table's compile — the face adds a
channel, never changes a decision, at the pipeline level.

## RED → GREEN

* RED: `crates/aeolus-theme/tests/observer.rs` (9 pins, written
  first — the "no `ThemeObserver` trait / no `with_observer` entry"
  compile failure IS the capture: the observation-free defaults, the
  collector's arm totals, the typed misses, the caller-order stream,
  the rgba channel's split, the answer-neutrality differential, the
  clone/second-table shared handle, the `Arc` blanket, the tier
  honesty) + `crates/aeolus-compile/tests/theme_observer.rs` (2 pins,
  the cross-layer ride, same RED).
* GREEN: the same pins pass against the landed face — 9/9 + 2/2; the
  full theme crate suite re-measured (18/18), the compile crate's
  existing faces untouched; the full workspace gate set re-measured
  post-landing (fmt, clippy `--workspace --all-targets --all-features
  -- -D warnings` zero warnings, `cargo test --workspace
  --all-features`, benches).

## Bench

`crates/aeolus-theme/benches/observer.rs` (divan, stable), medians:
`resolve_unobserved` ~7.6 ns, `resolve_counted` ~12.5 ns (the channel
costs one atomic add when wired), `resolve_recorded` ~217 ns (the
worst-case host's own bound), `resolve_baked_unobserved` ~7.6 ns,
`resolve_baked_counted` ~12.2 ns, `resolve_rgba_counted` ~80 ns (the
premultiply's own math dominating), `theme_stats_snapshot` ~2.6 ns
(nine relaxed loads, no locks). The unobserved arms ARE the plain
`resolve` path — the face costs nothing when it is not wired.

## Files

`crates/aeolus-theme/src/observer.rs` (the face), `src/lib.rs` (the
wire-up), `tests/observer.rs`, `crates/aeolus-compile/tests/
theme_observer.rs` (the cross-layer battery), `benches/observer.rs` +
the manifest row, `examples/theme_observer.rs`, the README (Themes
note + example index), the architecture doc §3.4 note, the changelog.
