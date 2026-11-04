#!/bin/bash
#
# commit-per-file.sh — commit every changed file individually with a
# Conventional Commit message. Bash 3.2 compatible (macOS default):
# no associative arrays, no mapfile, no ${var^^}, no &>>.
#
# Refuses to run with a dirty index; if you have staged changes, run
#   git reset -q HEAD
# first.
#
set -u
set -o pipefail

# ---- safety checks --------------------------------------------------------

if [ ! -d .git ]; then
    echo "error: not inside a git repository" >&2
    exit 1
fi

if ! git rev-parse --verify HEAD >/dev/null 2>&1; then
    echo "error: no commits yet (unborn HEAD); make an initial commit first" >&2
    exit 1
fi

if ! git diff --cached --quiet; then
    echo "error: staging area is not empty; run 'git reset -q HEAD' first" >&2
    exit 1
fi

# ---- gather changes (rename detection needs a temporary stage) -----------

echo "==> staging everything (temporarily) to detect renames"
git add -A

declare -a ENTRIES=()
while IFS=$'\t' read -r status p1 p2; do
    [ -z "$status" ] && continue
    case "$status" in
        R*|C*) ENTRIES+=("R|$p1|$p2") ;;
        *)     ENTRIES+=("$status|$p1") ;;
    esac
done < <(git diff --cached --name-status -M HEAD)

git reset -q HEAD

if [ ${#ENTRIES[@]} -eq 0 ]; then
    echo "==> working tree is clean; nothing to do" >&2
    exit 0
fi

echo "==> ${#ENTRIES[@]} file operation(s) to commit"

# ---- conventional-commit message table -----------------------------------
#
# The lookup key is the NEW path for renames, the sole path otherwise.
# Every arm echoes a subject + body; the fallback guarantees a message
# even if this table drifts.

commit_msg_for() {
    local f="$1"
    case "$f" in
        .github/workflows/ci.yml) cat <<'EOF'
ci: add full CI pipeline

GitHub Actions workflow covering fmt --check, clippy -D warnings,
test matrix (Linux/macOS/Windows), rustdoc -D warnings plus
aeolus-core doctests, cargo package gate for the publishable set,
cargo deny licenses, and a 60s-per-target libFuzzer smoke on main
pushes plus a weekly cron.
EOF
        ;;

        CHANGELOG.md) cat <<'EOF'
docs: add v0.1.0 changelog

Keep a Changelog format. Documents what ships in 0.1.0 (core and
macro publishable, backends gated) and explicitly lists what is
deferred to 0.2 (backend publishing, group/peer layers, container
queries, transitions, structured variants).
EOF
        ;;

        CONTRIBUTING.md) cat <<'EOF'
docs: add contributor guide

Restate the project non-negotiables: TDD-first (failing test -> RED
-> fix -> GREEN), no silent failures (LoweringReport / Warning
contract), no shortcuts, zero-dependency aeolus-core, verbatim
Shadcn class tables, and conventional commits.
EOF
        ;;

        Cargo.lock) cat <<'EOF'
build(deps): refresh lockfile for v0.1.0

Pick up proptest (core dev-dep), taffy (floem dev-dep for the
Length::Pt pin), masonry_testing (shadcn dev-dep), and the pinned
git revisions for xilem/masonry @ 9f504ae and floem @ 1351ffb.
EOF
        ;;

        Cargo.toml) cat <<'EOF'
build: pin backend deps to git revs, exclude fuzz workspace

- Backend manifests now reference xilem @ 9f504ae and floem @
  1351ffb directly (workspace builds with repos/ deleted).
- exclude = ["fuzz"] so cargo-fuzz's own workspace doesn't join the
  root workspace.
- Point the dev-only muda stub patch at its new crates/muda-stub
  home and document why it must never ship.
EOF
        ;;

        LICENSE-APACHE) cat <<'EOF'
docs(license): add Apache-2.0 license text

Required by the MIT OR Apache-2.0 dual-license declared in every
manifest; previously referenced but absent from the repo root.
EOF
        ;;

        LICENSE-MIT) cat <<'EOF'
docs(license): add MIT license text

Required by the MIT OR Apache-2.0 dual-license declared in every
manifest; previously referenced but absent from the repo root.
EOF
        ;;

        README.md) cat <<'EOF'
docs: rewrite README to match v0.1.0 reality

Replace the deliverables table with the crate publish status,
honest shipped-vs-deferred lists, exact build/package/fuzz
commands, and a one-paragraph architecture summary. Backend crates
are clearly marked publish=false with the upstream blockers named.
EOF
        ;;

        SECURITY.md) cat <<'EOF'
docs: add security policy

Scope: panics, unsafety, resource exhaustion, or incorrect styling
semantics reachable through the published crates' public APIs.
Out of scope: the publish=false backend crates and the dev-only
muda stub. Disclosure via GitHub advisory or security@aeolus.rs;
7-day acknowledgment target.
EOF
        ;;

        crates/aeolus-core/Cargo.toml) cat <<'EOF'
build(core): add proptest as dev-dependency

Property tests are dev-only; aeolus-core itself stays zero-
dependency per the project guardrail.
EOF
        ;;

        crates/aeolus-core/benches/hot.rs) cat <<'EOF'
style(core): reformat benches per rustfmt

Import ordering and closure formatting only; no benchmark behavior
change.
EOF
        ;;

        crates/aeolus-core/src/ast.rs) cat <<'EOF'
docs(core): add rustdoc for AST types

Enables #![deny(missing_docs)] on aeolus-core. Documents Variant,
NamedVariant, ParentKind, and Breakpoint with their Tailwind v4
semantics.
EOF
        ;;

        crates/aeolus-core/src/cache.rs) cat <<'EOF'
feat(core): bound caches, poison-tolerant locks

- Interner and PlanCache use two-generation rotation
  (DEFAULT_CACHE_CAP = 4096 per generation, <= 2x cap total).
  Re-interning an evicted string yields a fresh id (documented
  tradeoff).
- All lock access goes through read_map/write_map, which recover
  the guard from a poisoned lock via unwrap_or_else(|e|
  e.into_inner()). The justification is documented in rustdoc: the
  guarded maps hold plain resolved values, no cross-lock
  invariants.
- Tests: poisoned-lock survival, interner eviction, plan-cache
  eviction, warm previous-generation Arc identity.
EOF
        ;;

        crates/aeolus-core/src/caps.rs) cat <<'EOF'
docs(core): add rustdoc for capability types

Documents PropertySupport fields, CascadeModel, BackendCapabilities,
and the Backend trait's lowering contract. Part of the
missing_docs-enable pass.
EOF
        ;;

        crates/aeolus-core/src/color.rs) cat <<'EOF'
fix(core): approximate Color equality, clamp Oklab conversions

- impl PartialEq for Color with per-component tolerance 1e-4 (below
  one 8-bit sRGB step). Exact f32 equality split colors that
  differed only by round-trip noise, causing spurious plan diffs and
  false cache misses.
- parse_hex: reject non-ASCII input (byte-length checks sliced
  mid-char; fuzz finding "#abᐁ").
- split_alpha: clamp parsed alpha to [0,1] (fuzz finding 1e95 ->
  inf).
- oklch(): clamp hue to +/-360 deg (fuzz finding 1.9e83 -> NaN in
  cos).
- to_srgba(): clamp Oklab components before the matrix (fuzz
  finding 3.3e40 -> inf cubes), then clamp linear components to
  0..=1 before the transfer function (property-test finding: NaN via
  powf on negative linear components).
- from_srgb(): defense-in-depth input clamp.
- Unit tests for every fix above.
EOF
        ;;

        crates/aeolus-core/src/conflict.rs) cat <<'EOF'
fix(core): classify missing conflict families in cn()

Add collision_group() arms for the tw_merge families that were
previously unclassified: justify-items/self, align-content
(content-*), place-content/items/self, flex-wrap (split from
flex-direction), flex-basis, order, position, inset-x/y/inset,
top/right/bottom/left, z-index, overflow-x/y/overflow,
overscroll-behavior (+ parent->child knockouts), aspect, columns,
visibility, isolation, resize, object-fit/object-position,
font-style, text-transform, text-decoration, vertical-align, and
text-align (split from text-color).

Also fix cn()'s fast path: pure-whitespace and single-word inputs
now normalize to a word sequence (fuzz findings: cn("\n") echoed
"\n"; cn("\n\u{1c}") broke idempotence). Unparseable input is still
kept verbatim, but always whitespace-normalized.

This makes justify-items-start + justify-items-center merge while
keeping independent groups independent (justify-items vs
justify-content).
EOF
        ;;

        crates/aeolus-core/src/lib.rs) cat <<'EOF'
docs(core): enforce missing_docs, add runnable crate example

- Add #![deny(missing_docs)] at the crate root.
- Re-export sorting and small rustdoc additions.
- Change the crate-level example from `no_run` to runnable so
  `cargo test --doc -p aeolus-core` actually exercises it.
- Document the shadcn_css module constants.
EOF
        ;;

        crates/aeolus-core/src/parser.rs) cat <<'EOF'
refactor(core): simplify parser code, add rustdoc

- Use if-let chains for bracket tracking.
- Add rustdoc for parse_named_variant.
- Reformat for rustfmt.

No behavior change; covered by the parser_roundtrip property test.
EOF
        ;;

        crates/aeolus-core/src/plan.rs) cat <<'EOF'
docs(core): document StylePlan IR types

Add rustdoc to every public field of ConditionalLayer,
VariantCondition, ResolvedStyle, Slot, Lane, RingPart, Padding,
BorderWidth, CornerRadius, Dimensions, SizedValue, BoxShadow,
Layout, Display, FlexDirection, AlignItems, JustifyContent, Ring,
Outline, Cursor, PointerEvents, TextProperty, TextValue, StyleDiff.
Required by #![deny(missing_docs)].
EOF
        ;;

        crates/aeolus-core/src/resolve.rs) cat <<'EOF'
refactor(core): reformat resolver, add rustdoc

- Reformat long struct literals and closures for rustfmt.
- Use if-let chains and is_some_and where it simplifies.
- Add rustdoc for Warning variants, Resolution, Resolver.theme, and
  the Applied enum's private fields.

No behavior change; suite stays green.
EOF
        ;;

        crates/aeolus-core/src/theme.rs) cat <<'EOF'
fix(core): shunting-yard calc precedence, fuzz-hardened CSS parsing

- Replace the left-to-right calc() evaluator with a real
  shunting-yard one that supports standard operator precedence,
  unary +/-, nested parentheses, and substituted var()s.
  calc(4px + var(--r) * 2) now correctly evaluates as 4 + r*2 (was
  (4+r)*2).
- extract_declarations(): early-return on an unbalanced block
  instead of slicing out of bounds (fuzz finding: CSS ending in
  "{").
- Document BlockKind variants.
- Tests: calc_has_operator_precedence (precedence, parens, nesting,
  unary minus on a parenthesized group);
  unbalanced_blocks_do_not_panic.
EOF
        ;;

        crates/aeolus-core/tests/properties.proptest-regressions) cat <<'EOF'
test(core): add proptest regression seed

Persist the shrunk failure case for the oklab_srgb_roundtrip
property (l=0, a=0, b=0.274) so the previously-NaN-producing input
re-runs on every future test invocation.
EOF
        ;;

        crates/aeolus-core/tests/properties.rs) cat <<'EOF'
test(core): add property suite (4 x 512 cases)

Properties:
1. cn is idempotent on arbitrary class soup.
2. parse -> serialize -> parse roundtrips (test-local serializer;
   the crate deliberately has none).
3. OKLab <-> sRGB is tight (1e-3) for in-gamut colors and finite
   everywhere (out-of-gamut is clamped by design). This property
   immediately caught the powf-on-negative-linear NaN bug in
   Color::to_srgba.
4. Opacity modifiers preserve hue and scale alpha as v4 defines.

Dev-only: proptest stays a dev-dependency; aeolus-core remains
zero-dependency.
EOF
        ;;

        crates/aeolus-core/tests/shadcn_components.rs) cat <<'EOF'
style(core): reformat shadcn integration test per rustfmt

Import ordering, struct-literal formatting, assertion wrapping
only. All 13 component tests still pass unchanged.
EOF
        ;;

        crates/aeolus-floem/Cargo.toml) cat <<'EOF'
build(floem): switch to git-pinned floem, mark publish=false

- floem = { git = "https://github.com/lapce/floem", rev =
  "1351ffb..." } so the workspace builds without a sibling repos/
  checkout.
- publish = false: crates.io rejects git dependencies, and the Linux
  GTK/muda hard-dependency is documented in
  reports/09-publishing/PUBLISHING.md.
- taffy 0.9.2 (grid feature) as a dev-dependency so the Length::Pt
  -> taffy pass-through is testable end-to-end.
EOF
        ;;

        crates/aeolus-floem/src/lib.rs) cat <<'EOF'
docs(floem): document Pt-is-pixel semantics, reformat

- Add a rustdoc block on pt() explaining that floem's Length::Pt is
  a logical layout pixel, not a CSS point (verified against floem @
  1351ffb src/style/unit.rs: to_taffy passes the value through
  unscaled). Pinned by pt_is_logical_pixel_not_css_point.
- Reformat long expressions for rustfmt.

No behavior change.
EOF
        ;;

        crates/aeolus-floem/tests/lowering.rs) cat <<'EOF'
test(floem): pin Length::Pt to logical pixels via taffy

New test pt_is_logical_pixel_not_css_point:
(a) lowered p-4 padding -> LengthPercentage::length(16.0) at taffy
    level;
(b) a real taffy layout resolves 16px padding and a 32px content
    box in a 64px box at scale factor 1.

Mutation-verified: giving pt() CSS-point semantics (x72/96) makes
the test fail immediately, so a future floem upgrade that rescales
lengths cannot slip through silently.
EOF
        ;;

        crates/aeolus-macro/Cargo.toml) cat <<'EOF'
build(macro): add repository and rust-version metadata

Aligns aeolus-macro with the workspace manifests so cargo package
records the same repository URL and MSRV (1.88) as aeolus-core.
EOF
        ;;

        crates/aeolus-macro/src/lib.rs) cat <<'EOF'
style(macro): reformat error message

Wrap the tw! unexpected-token error to one line per rustfmt. No
behavior change.
EOF
        ;;

        crates/aeolus-shadcn/Cargo.toml) cat <<'EOF'
build(shadcn): switch to git-pinned deps, mark publish=false

- masonry and xilem_masonry point at xilem @ 9f504ae; floem points
  at floem @ 1351ffb. No sibling repos/ needed.
- publish = false: depends on the two publish=false backend crates
  (see reports/09-publishing/PUBLISHING.md).
- masonry_testing added as a dev-dependency for the acceptance
  tests.
EOF
        ;;

        crates/aeolus-shadcn/src/classes.rs) cat <<'EOF'
style(shadcn): reformat class tables per rustfmt

Long string literals wrapped onto continuation lines only. The
class content is still verbatim from the upstream registry (commit
pinned in reports/06-shadcn-v1/SHADCN_V1.md); no "fixes" applied.
EOF
        ;;

        crates/aeolus-shadcn/src/floem.rs) cat <<'EOF'
style(shadcn): reformat floem wrapper per rustfmt

Import grouping and one-line signature collapse only. No behavior
change.
EOF
        ;;

        crates/aeolus-shadcn/src/lib.rs) cat <<'EOF'
docs(shadcn): document three consumption layers

- classes: backend-agnostic class tables.
- xilem interactive: button and input are real widgets.
- xilem/floem style-only: badge, label, card, alert.

The docs explicitly say badge/label/card/alert are NOT interactive,
so the API surface is honest about what each wrapper does.
EOF
        ;;

        crates/aeolus-shadcn/src/xilem.rs) cat <<'EOF'
feat(shadcn): make button and input real interactive widgets

- button(resolver, id, text, variant, size, class, on_press) builds
  a real StyledButton (pointer, Space/Enter, accesskit Click,
  disabled) — the old signature produced a dead label and is
  intentionally replaced.
- input(resolver, value, on_changed, class) composes Masonry's
  TextInput with the Shadcn Input class table and the standard
  controlled-value contract.
- resolve_plan() helper: panic in debug, warn + empty plan in
  release, matching TwViewExt::tw.
- badge/label/card/alert remain style-only and their docs say so.
EOF
        ;;

        crates/aeolus-shadcn/tests/classes.rs) cat <<'EOF'
style(shadcn): reformat class tests per rustfmt

Line wrapping and import ordering only. All class-table integrity
assertions unchanged.
EOF
        ;;

        crates/aeolus-shadcn/tests/xilem_widgets.rs) cat <<'EOF'
test(shadcn): acceptance tests for interactive button/input

- shadcn_button_view_builds_an_interactive_button: builds a real
  StyledButton, clicks it headlessly, asserts exactly one
  StyledButtonPress, and checks the carried plan has bg-primary and
  a hover layer.
- shadcn_input_typing_changes_the_value: focuses the inner TextArea,
  types "hello", asserts the value.
- shadcn_input_view_builds_a_styled_wrapper: asserts the wrapper
  carries border/padding/shadow from INPUT_BASE.

These tests are what let the crate claim button/input are real
widgets rather than styled labels.
EOF
        ;;

        crates/aeolus-xilem/Cargo.toml) cat <<'EOF'
build(xilem): switch to git-pinned masonry/xilem, mark publish=false

- masonry and xilem_masonry point at xilem @ 9f504ae.
- publish = false: crates.io rejects git deps and the pinned
  commits are pre-release upstream state (see PUBLISHING.md).
- masonry_testing (dev) points at the same rev.
EOF
        ;;

        crates/aeolus-xilem/src/button.rs) cat <<'EOF'
feat(xilem): add StyledButton — real interactive widget

A real widget mirroring masonry::widgets::Button's semantics:
- pointer: Down -> request_focus + capture; Up -> submit when
  active && hovered (drag-off cancels);
- keyboard: Space/Enter on key-up;
- access: Role::Button + accesskit Click;
- disabled: no submit (Masonry routing + in-widget is_disabled
  guard);
- painting: shared paint_plan_box extracted from StyledWidget
  (DRY), so per-state hover/active/disabled/focus layers render
  identically;
- measurement: fixed dims from the plan act as minimums (h-9).

styled_button() view wraps the widget with a callback (on_press),
following xilem_masonry::view::button's action routing. The action
type StyledButtonPress { button: Option<PointerButton> } mirrors
Masonry's ButtonPress so hosts can branch on the pointer button.
EOF
        ;;

        crates/aeolus-xilem/src/lib.rs) cat <<'EOF'
feat(xilem): expose button module, report per-side border flattening

- Add `pub mod button`.
- lower_resolved(): when the four BorderWidth edges differ, emit a
  LoweringReport entry "border-width:per-side" with the reason
  "Masonry BorderWidth is uniform; using top edge". Previously this
  was silently flattened to the top edge (honesty-rule violation).
- Reformat to rustfmt.
EOF
        ;;

        crates/aeolus-xilem/src/view.rs) cat <<'EOF'
style(xilem): collapse multiline signatures per rustfmt

One-line tw() / tw_centered() signatures. No behavior change.
EOF
        ;;

        crates/aeolus-xilem/src/widget.rs) cat <<'EOF'
refactor(xilem): extract paint_plan_box, fix disabled-state repaint

- Extract the box-painting block from StyledWidget::pre_paint into
  pub(crate) fn paint_plan_box, shared with the new StyledButton
  (identical rendering, DRY with masonry's default pre_paint).
- Fix a real repaint bug in update(): the disabled lane flips
  state.is_disabled AFTER update() returns (masonry
  update_disabled_for_widget), so ctx.is_disabled() was stale — the
  disabled pixel diff never scheduled a repaint. Both StyledWidget
  and StyledButton now take the new flag value from the Update
  event itself (comment documents the masonry call order).
EOF
        ;;

        crates/aeolus-xilem/tests/button.rs) cat <<'EOF'
test(xilem): StyledButton acceptance tests

Covers the mission's Track A criteria:
- click fires the callback exactly once (and press-alone does not);
- focus + Space/Enter fires it (other keys do not);
- disabled + click and disabled + Space do NOT fire;
- hover/active/disabled render different pixels (pixel diff via
  Masonry TestHarness render()); moving the pointer away restores
  base pixels; disabling then re-enabling does too;
- the action carries the pointer button (None for harness clicks).
EOF
        ;;

        crates/aeolus-xilem/tests/cn_wins_rightmost.rs) cat <<'EOF'
style(xilem): reorder imports per rustfmt

Pure import ordering change; the cn/rightmost-wins proof test is
unchanged.
EOF
        ;;

        crates/aeolus-xilem/tests/hover.rs) cat <<'EOF'
style(xilem): reformat hover test

Line wrapping and one trailing blank-line removal. The
hover_restyle_is_paint_only proof is unchanged.
EOF
        ;;

        crates/aeolus-xilem/tests/lowering.rs) cat <<'EOF'
test(xilem): pin per-side border-width report entry

New test per_side_border_width_is_reported_not_silently_dropped:
- border-t border-b now composes (top=1, bottom=1, left=0, right=0)
  instead of resetting the top edge to 0;
- lower() emits "border-width:per-side" in LoweringReport.dropped
  when the edges differ.

Pins the honesty-rule behavior for per-side border flattening.
EOF
        ;;

        crates/aeolus-xilem/tests/view_sugar.rs) cat <<'EOF'
style(xilem): reformat view_sugar test per rustfmt

Line wrapping and import ordering only. The tw view-sugar
assertions are unchanged.
EOF
        ;;

        crates/muda-stub/Cargo.toml) cat <<'EOF'
refactor: move muda-stub to crates/ top level

Move the dev-only muda stub manifest out of crates/aeolus-floem/ so
it isn't nested under the floem backend. The stub remains a
non-member, publish=false, and only reachable via the workspace
[patch.crates-io] block. See reports/09-publishing/PUBLISHING.md.
EOF
        ;;

        crates/muda-stub/src/lib.rs) cat <<'EOF'
refactor: move muda-stub source with its manifest

Companion move for the muda stub library. Content is bit-identical
to the previous location; only the path changed.
EOF
        ;;

        deny.toml) cat <<'EOF'
build: add cargo-deny config with enumerated licenses

The allow list is enumerated from Cargo.lock's dependency graph
(MIT, Apache-2.0 +/- LLVM-exception, BSD-2/3, ISC, Unicode-3.0,
Zlib, BSL-1.0, Unlicense, CC0-1.0, 0BSD, MPL-2.0, BlueOak-1.0.0).
LGPL only appears inside OR-duals and is intentionally not
accepted.

[bans]: multiple-versions = warn, wildcards = deny.
[sources]: only the two pinned framework git repos are allowed.
[advisories]: yanked = warn; advisories are a per-release manual
review (documented in SECURITY.md), not part of the license CI gate.
EOF
        ;;

        docs/logo.png) cat <<'EOF'
chore: remove unused logo asset

The README no longer references it and nothing else uses it.
EOF
        ;;

        fuzz/.gitignore) cat <<'EOF'
build(fuzz): ignore fuzz build artifacts

Ignore target/, corpus/, artifacts/, and coverage/ so the fuzz
workspace stays clean (cargo-fuzz generates these locally).
EOF
        ;;

        fuzz/Cargo.lock) cat <<'EOF'
build(fuzz): add fuzz workspace lockfile

Pins libfuzzer-sys and the aeolus-core path dependency for
reproducible fuzz runs. Separate from the root lockfile because
fuzz/ is its own workspace.
EOF
        ;;

        fuzz/Cargo.toml) cat <<'EOF'
build(fuzz): add fuzz crate with four targets

cargo-fuzz workspace declaring four libFuzzer binaries:
parse_class_string, color_parse, cn, theme_from_css. Excluded from
the root workspace via the root Cargo.toml's exclude = ["fuzz"].
EOF
        ;;

        fuzz/fuzz_targets/cn.rs) cat <<'EOF'
test(fuzz): add cn idempotence fuzz target

Asserts that cn() never panics and is idempotent on every input,
including adversarial whitespace and unparseable tokens (the class
of bug the Phase-4 unit test caught).
EOF
        ;;

        fuzz/fuzz_targets/color_parse.rs) cat <<'EOF'
test(fuzz): add color parse finiteness fuzz target

Any string Color::parse accepts must convert to finite sRGBA.
Catches NaN/inf in the Oklab->sRGB math (the class of bug the
property test caught: powf on a negative linear component).
EOF
        ;;

        fuzz/fuzz_targets/parse_class_string.rs) cat <<'EOF'
test(fuzz): add class parser coverage fuzz target

Asserts parse_class_string accounts for every whitespace-separated
word: each is either parsed or produces an error entry. This is
the honest-degradation contract; no word may be silently lost, and
the parser must never panic.
EOF
        ;;

        fuzz/fuzz_targets/theme_from_css.rs) cat <<'EOF'
test(fuzz): add CSS theme parser fuzz target

Asserts TailwindTheme::from_theme_css never panics on arbitrary
CSS (malformed/unbalanced blocks are tolerated). A successfully
parsed theme must answer all lookup methods and a subsequent
resolve() without panicking.
EOF
        ;;

        reports/08-benchmarks/BENCHMARKS.md) cat <<'EOF'
docs(reports): add v0.1.0 benchmark re-run

Same sandbox, release profile: hot paths unchanged
(resolve_cached_hit 405 ns, resolve_button_cold 4.6 us,
diff_unchanged 505 ns ~4% faster via approximate Color equality).
Theme parse +15% (231 us, one-time cost) from calc precedence and
fuzz hardening — accepted. Verdict: the production-hardening work
did not regress the hot paths the brief pins.
EOF
        ;;

        reports/09-publishing/PUBLISHING.md) cat <<'EOF'
docs(reports): document publishing decision (option c)

Records the Phase-1 decision (a/b impractical, c chosen), what
ships (aeolus-core, aeolus-macro), what is gated (the three
backend crates with their upstream blockers), and the dev-sandbox
artifacts that must never be published (crates/muda-stub). Includes
the cargo-package publish-order caveat (core first, then macro).
EOF
        ;;

        reports/09-release/RELEASE_CHECKLIST.md) cat <<'EOF'
docs(reports): add release checklist with all verification steps

Every step actually run for v0.1.0 with its observed result: test
(87 passed), clippy, fmt, rustdoc, doctest, package, repos/-
independent build, cargo-deny licenses, fuzz (4x60s), and the
property suite. Also records Track A decision and pre-tag audit.
EOF
        ;;

        reports/10-tdd-evidence/01-publishability.md) cat <<'EOF'
docs(reports): add phase 1 TDD evidence

Verbatim RED/GREEN for the publishability acceptance commands:
workspace-load failure without repos/, macro-alone packaging
failure, 224 missing-doc errors after enabling deny(missing_docs),
and the final green packaging output. Documents the option-(c)
choice with source citations.
EOF
        ;;

        reports/10-tdd-evidence/02-correctness.md) cat <<'EOF'
docs(reports): add phase 2 TDD evidence

Per-item RED/GREEN for the nine correctness bugs fixed in this
phase, each with a failing test captured first: Length::Pt unit
semantics, Color f32 equality, per-side border flattening, calc()
precedence, justify-items classification, poisoned RwLocks,
unbounded Interner/PlanCache. Includes the mutation-verified Pt
pin and the two bugs the per-side test surfaced (report entry +
border_t composition).
EOF
        ;;

        reports/10-tdd-evidence/03-real-widgets.md) cat <<'EOF'
docs(reports): add phase 3 TDD evidence

Track A rationale (TestHarness can drive real widgets headlessly,
Masonry's Button is the interactivity reference), the compile-first
RED (unresolved imports for the new API), the three bugs the
acceptance tests caught (stale is_disabled in update(), root button
can't un-hover, keyboard_type_chars sends IME commits), and the
verbatim GREEN test output. Maps each test to a mission acceptance
criterion.
EOF
        ;;

        reports/10-tdd-evidence/04-hardening.md) cat <<'EOF'
docs(reports): add phase 4 TDD evidence

Property suite (4 x 512) plus the powf-NaN bug it caught on the
first run; the four fuzz targets with their 60s clean run counts
(1.19M / 7.26M / 540K / 6.3K execs); the six fuzz findings, each
with a unit-test RED first and the fix that followed (non-ASCII hex
slice, cn whitespace, unbalanced-block OOB, alpha inf, hue NaN,
chroma inf cubes); the CI job list; and the enumerated deny.toml
license allow list.
EOF
        ;;

        reports/10-tdd-evidence/05-release.md) cat <<'EOF'
docs(reports): add phase 5 TDD evidence

Records the version bump, the five new release documents
(CHANGELOG, SECURITY, CONTRIBUTING, README, RELEASE_CHECKLIST),
and the final green run of the release battery (tests, clippy,
fmt, doc, package, deny, tag). Explicitly notes that the 0.2
deferrals are documented so v0.1.0 overpromises nothing.
EOF
        ;;

        *)
            printf 'chore: update %s\n' "$f"
            ;;
    esac
}

# ---- commit one entry ----------------------------------------------------

commit_one() {
    local status="$1"
    shift
    local paths=("$@")
    local n=${#paths[@]}
    if [ "$n" -eq 0 ]; then return 0; fi

    local primary="${paths[$((n - 1))]}"   # new path for renames

    local p
    for p in "${paths[@]}"; do
        git add -A -- "$p" || {
            echo "error: git add failed for $p" >&2
            return 1
        }
    done

    if git diff --cached --quiet; then
        echo "skip (no staged changes): $primary" >&2
        return 0
    fi

    echo "commit: $primary" >&2
    if ! commit_msg_for "$primary" | git commit -F -; then
        echo "error: commit failed for $primary" >&2
        return 1
    fi
}

# ---- main loop ------------------------------------------------------------

for entry in "${ENTRIES[@]}"; do
    status="${entry%%|*}"
    rest="${entry#*|}"
    case "$status" in
        R|C)
            old="${rest%%|*}"
            new="${rest#*|}"
            commit_one "$status" "$old" "$new" || exit 1
            ;;
        *)
            commit_one "$status" "$rest" || exit 1
            ;;
    esac
done

echo
echo "==> done; recent commits:" >&2
git log --oneline -20
