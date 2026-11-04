# Phase 3 — real widgets (Track A) — TDD evidence

**Track decision: A (full components).** Rationale: Masonry's `TestHarness`
drives real widgets headlessly (mouse/keyboard/focus/disabled/render-diff —
verified against `masonry_testing/src/harness.rs` before deciding), and
Masonry's own `Button` widget provides the interactivity reference, so both
acceptance widgets were implementable within budget. Both tracks were valid;
A is the more honest v0.1.0.

RED for new code = the acceptance tests were written FIRST and did not
compile against the old API. Verbatim RED:

```text
error[E0432]: unresolved import `aeolus_xilem::button`
```
```text
error[E0432]: unresolved import `aeolus_shadcn::xilem::input`
error[E0061]: this function takes 6 arguments but 7 arguments were supplied
```

## Implementation

- `aeolus-xilem/src/button.rs` (new):
  - `StyledButton` — real widget. Pointer: `Down` → request_focus +
    capture; `Up` → submit when `active && hovered` (Masonry Button
    semantics; drag-off cancels). Keyboard: Space/Enter on key-up.
    Access: `Role::Button` + accesskit `Click`. Disabled: Masonry routes
    no pointer/text events to disabled widgets, plus an in-widget
    `is_disabled` guard. Painting: shared `paint_plan_box` extracted from
    `StyledWidget::pre_paint` (refactor step — both widgets now DRY on the
    same box-painting path). Fixed dims (`h-9`) act as measure minimums.
  - `styled_button(child, on_press)` view — action routing follows
    `xilem_masonry::view::button` (`with_action_widget`,
    `teardown_action_source`, `take_message::<StyledButtonPress>`),
    with `.with_plan(plan)` and `.disabled(bool)` builders.
- `aeolus-shadcn/src/xilem.rs`: `button(..., on_press)` builds the real
  button view from the Shadcn class table; `input(resolver, value,
  on_changed, class)` composes Masonry `TextInput` (controlled value) with
  the styled wrapper. badge/label/card/alert stay style-only and say so.

## Bugs the tests caught (fixed, tests kept)

1. **Stale `is_disabled` in `update()`** — Masonry's
   `update_disabled_for_widget` calls `widget.update(DisabledChanged)`
   and only flips `state.is_disabled` AFTER the call, so
   `ctx.is_disabled()` inside `update()` was stale; the disabled pixel
   diff never scheduled a repaint. Both widgets now take the new flag
   value from the event itself (comment cites the pass code).
   RED verbatim (pixel test, first ordering attempt):

```text
thread 'hover_active_disabled_states_render_different_pixels' panicked:
assertion `left != right` failed: disabled state must paint differently
```

2. **Test-side hover semantics** — a root button fills the 400×400 window
   and can never un-hover; the pixel test now roots the button in a flex
   row and un-hovers by moving the pointer off (asserting base pixels
   return).
3. **Keyboard activation** — `keyboard_type_chars` sends `Ime::Commit`
   events (buttons ignore them); the test sends real
   `TextEvent::key_down/key_up` pairs. Focusing an input means focusing
   the inner `TextArea` (what `TextInput`'s pointer-down does).

## GREEN — verbatim

```text
$ cargo test -p aeolus-xilem --test button -- --test-threads=1
test action_carries_the_pointer_button ... ok
test click_fires_callback_exactly_once ... ok
test disabled_click_does_not_fire ... ok
test focus_and_space_fires_callback ... ok
test hover_active_disabled_states_render_different_pixels ... ok
test result: ok. 5 passed; 0 failed

$ cargo test -p aeolus-shadcn --test xilem_widgets -- --test-threads=1
test shadcn_button_view_builds_an_interactive_button ... ok
test shadcn_input_typing_changes_the_value ... ok
test shadcn_input_view_builds_a_styled_wrapper ... ok
test result: ok. 3 passed; 0 failed
```

Workspace after Phase 3: **79 passed / 0 failed** (was 71).

Acceptance-criteria mapping (mission §4 Phase 3, Track A):
- click fires exactly once → `click_fires_callback_exactly_once` ✅
- focus + Space fires → `focus_and_space_fires_callback` ✅ (Enter too)
- disabled + click does NOT fire → `disabled_click_does_not_fire` ✅
- hover/active/disabled render different pixels →
  `hover_active_disabled_states_render_different_pixels` ✅
- Input: type → value changes → `shadcn_input_typing_changes_the_value` ✅
- Input: styling applied → `shadcn_input_view_builds_a_styled_wrapper` ✅

Design one-liners: action type `StyledButtonPress { button }` mirrors
Masonry's `ButtonPress` so hosts can branch on the pointer button; the
shadcn `button()` signature gained the `on_press` callback (an intentional
API change — the old signature could only produce a dead label).
