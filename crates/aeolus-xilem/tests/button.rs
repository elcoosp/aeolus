//! Phase-3 Track A acceptance tests: `StyledButton` is a REAL widget.
//!
//! Each test below is one of the mission's Track A acceptance criteria:
//! - click fires the action exactly once;
//! - focus + Space fires it (keyboard parity);
//! - disabled + click does NOT fire it;
//! - hover / active / disabled states render different pixels.

use masonry::core::keyboard::{Key, NamedKey};
use masonry::core::{NewWidget, TextEvent, Widget as _};
use masonry_testing::TestHarness;

use aeolus_core::StylePlan;
use aeolus_xilem::button::{StyledButton, StyledButtonPress};

/// A plan with distinct per-state backgrounds so every state flip is
/// visible in the rendered pixels (shadcn's own `disabled:opacity-50`
/// cannot pixel-diff on Masonry — opacity is a reported drop there).
fn plan() -> StylePlan {
    let theme = aeolus_core::shadcn_default().unwrap();
    let resolver = aeolus_core::Resolver::new(&theme);
    resolver
        .resolve("p-4 bg-primary hover:bg-red-500 active:bg-black disabled:bg-gray-700")
        .unwrap()
        .plan
}

fn button_with_plan(plan: StylePlan) -> StyledButton {
    StyledButton::new(masonry::widgets::Label::new("Save").prepare()).with_plan(plan)
}

/// Root the button itself (no recorder wrapper): every pointer/keyboard
/// event lands directly on `StyledButton`'s Widget impl.
fn harness(button: StyledButton) -> (TestHarness<StyledButton>, masonry::core::WidgetId) {
    let mut harness = TestHarness::create(Default::default(), NewWidget::new(button));
    let id = harness.root_widget().id();
    harness.render(); // settle initial layout/paint
    (harness, id)
}

/// Count the StyledButtonPress actions the harness collected.
fn drain_actions(harness: &mut TestHarness<StyledButton>) -> usize {
    let mut n = 0;
    while harness.pop_action::<StyledButtonPress>().is_some() {
        n += 1;
    }
    n
}

#[test]
fn click_fires_callback_exactly_once() {
    let (mut harness, id) = harness(button_with_plan(plan()));

    harness.mouse_click_on(id, None);
    assert_eq!(
        drain_actions(&mut harness),
        1,
        "one click = exactly one button action"
    );

    // A second click fires again (state resets after release).
    harness.mouse_click_on(id, None);
    assert_eq!(
        drain_actions(&mut harness),
        1,
        "second click = one more action"
    );

    // Pointer-down alone must NOT fire; release on the widget fires once.
    harness.mouse_move_to(id);
    harness.mouse_button_press(None);
    assert_eq!(drain_actions(&mut harness), 0, "press alone fires nothing");
    harness.mouse_button_release(None);
    assert_eq!(
        drain_actions(&mut harness),
        1,
        "press+release on the widget fires once"
    );
}

#[test]
fn focus_and_space_fires_callback() {
    let (mut harness, id) = harness(button_with_plan(plan()));

    harness.focus_on(Some(id));
    // A real key press: down + up (keyboard_type_chars sends IME commits,
    // which buttons ignore). Space activates the focused button on release.
    harness.process_text_event(TextEvent::key_down(Key::Character("x".into())));
    harness.process_text_event(TextEvent::key_up(Key::Character("x".into())));
    assert_eq!(drain_actions(&mut harness), 0, "other keys do nothing");

    harness.process_text_event(TextEvent::key_down(Key::Character(" ".into())));
    harness.process_text_event(TextEvent::key_up(Key::Character(" ".into())));
    assert_eq!(drain_actions(&mut harness), 1, "Space fires once");

    harness.process_text_event(TextEvent::key_down(Key::Named(NamedKey::Enter)));
    harness.process_text_event(TextEvent::key_up(Key::Named(NamedKey::Enter)));
    assert_eq!(drain_actions(&mut harness), 1, "Enter fires once");
}

#[test]
fn disabled_click_does_not_fire() {
    let (mut harness, id) = harness(button_with_plan(plan()));

    // Disable through the same channel the view uses (WidgetMut ctx).
    harness.edit_root_widget(|mut btn| {
        btn.ctx.set_disabled(true);
    });

    // Even a full click sequence must produce no action.
    harness.mouse_move_to(id);
    harness.mouse_button_press(None);
    harness.mouse_button_release(None);
    assert_eq!(
        drain_actions(&mut harness),
        0,
        "disabled buttons never submit actions"
    );

    // Keyboard parity: a disabled widget refuses focus (Masonry's own rule),
    // and even a synthesized Space press is ignored while unfocused.
    harness.process_text_event(TextEvent::key_down(Key::Character(" ".into())));
    harness.process_text_event(TextEvent::key_up(Key::Character(" ".into())));
    assert_eq!(
        drain_actions(&mut harness),
        0,
        "disabled + Space fires nothing"
    );
}

#[test]
fn hover_active_disabled_states_render_different_pixels() {
    // The button lives inside a flex row so the pointer can LEAVE it
    // (a root button fills the whole window and can never un-hover).
    let tag = masonry::core::WidgetTag::named("pixel-button");
    let button = NewWidget::new(button_with_plan(plan())).with_tag(tag);
    let root = masonry::widgets::Flex::row().with(button, 0.).prepare();
    let mut harness = TestHarness::create(Default::default(), root);
    let id = harness.get_widget(tag).id();
    harness.render(); // settle

    let base = harness.render();

    // Hover: pointer over the button.
    harness.mouse_move_to(id);
    let hovered = harness.render();
    assert_ne!(base, hovered, "hover state must paint differently");

    // Active: press and hold.
    harness.mouse_button_press(None);
    let active = harness.render();
    assert_ne!(hovered, active, "active state must paint differently");
    harness.mouse_button_release(None);

    // Move the pointer away: back to the base state (release alone leaves
    // the button hovered; the flex row leaves the rest of the window empty).
    harness.mouse_move(masonry::kurbo::Point::new(390., 390.));
    let base2 = harness.render();
    assert_eq!(
        base, base2,
        "moving the pointer away restores the base pixels"
    );

    // Disabled: flip through WidgetMut (same channel as the view uses).
    harness.edit_widget(tag, |mut btn| {
        btn.ctx.set_disabled(true);
    });
    let disabled = harness.render();
    harness.edit_widget(tag, |mut btn| {
        btn.ctx.set_disabled(false);
    });
    let re_enabled = harness.render();
    assert_ne!(base, disabled, "disabled state must paint differently");
    assert_eq!(base, re_enabled, "re-enabling restores the base pixels");
}

#[test]
fn action_carries_the_pointer_button() {
    let (mut harness, id) = harness(button_with_plan(plan()));

    harness.mouse_click_on(id, None);
    let (action, _from) = harness
        .pop_action::<StyledButtonPress>()
        .expect("click produced an action");
    assert_eq!(
        action.button, None,
        "harness clicks carry the primary button as None"
    );
}
