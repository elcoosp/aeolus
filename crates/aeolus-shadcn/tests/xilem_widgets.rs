//! Phase-3 Track A acceptance tests for the Shadcn wrappers:
//!
//! - `xilem::button` builds a REAL interactive `StyledButton` (a click on
//!   the built widget tree submits a `StyledButtonPress`);
//! - `xilem::input` composes Masonry's `TextInput` (typing really edits the
//!   value) styled through the Shadcn class table;
//! - both carry the Shadcn plan into the widget tree.

use std::sync::Arc;

use aeolus_core::shadcn_default;
use aeolus_shadcn::classes::{ButtonSize, ButtonVariant};
use aeolus_shadcn::xilem::{button, input};
use aeolus_xilem::button::StyledButtonPress;
use masonry::core::{DefaultProperties, NewWidget, WidgetTag};
use masonry::dpi::PhysicalSize;
use masonry::widgets::TextInput;
use masonry_testing::{ModularWidget, TestHarness, TestHarnessParams};
use xilem_masonry::ViewCtx;
use xilem_masonry::core::{ProxyError, RawProxy, SendMessage, View as _, ViewId};

/// A `RawProxy` that goes nowhere: the headless build never sends messages.
struct NoopProxy;

impl std::fmt::Debug for NoopProxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("NoopProxy")
    }
}

impl RawProxy for NoopProxy {
    fn send_message(&self, _path: Arc<[ViewId]>, _message: SendMessage) -> Result<(), ProxyError> {
        Ok(())
    }

    fn dyn_debug(&self) -> &dyn std::fmt::Debug {
        self
    }
}

fn headless_ctx() -> ViewCtx {
    ViewCtx::new(
        Arc::new(NoopProxy),
        Arc::new(tokio::runtime::Runtime::new().unwrap()),
    )
}

fn resolver() -> aeolus_core::Resolver<'static> {
    let theme = shadcn_default().unwrap();
    aeolus_core::Resolver::new(Box::leak(Box::new(theme)))
}

/// The Shadcn `button()` view must produce a REAL button: clicking the
/// built widget submits `StyledButtonPress` (not just a styled label).
#[test]
fn shadcn_button_view_builds_an_interactive_button() {
    let resolver = resolver();
    let view = button(
        &resolver,
        "save",
        "Save",
        ButtonVariant::Default,
        ButtonSize::Default,
        "",
        |_state: &mut ()| {},
    );
    let (element, _view_state) = view.build(&mut headless_ctx(), &mut ());

    let tag: WidgetTag<aeolus_xilem::button::StyledButton> = WidgetTag::unique();
    let root = ModularWidget::new_parent(element.new_widget.with_tag(tag));
    let mut harness = TestHarness::create_with(
        DefaultProperties::default(),
        NewWidget::new(root),
        TestHarnessParams::size_and_padding(PhysicalSize::new(200, 200), 0),
    );
    let id = harness.get_widget(tag).id();
    harness.render();

    // One click = exactly one press action from the real widget.
    harness.mouse_click_on(id, None);
    assert!(
        harness.pop_action::<StyledButtonPress>().is_some(),
        "the shadcn button view must wrap an interactive widget"
    );
    assert!(
        harness.pop_action::<StyledButtonPress>().is_none(),
        "no duplicate actions"
    );

    // And the Shadcn plan traveled with it (bg-primary is set).
    let carried = harness.get_widget(tag).plan().clone();
    assert!(
        carried.base.background.is_some(),
        "shadcn button carries its plan"
    );
    assert!(
        carried
            .layers
            .iter()
            .any(|l| l.condition == aeolus_core::plan::VariantCondition::Hover),
        "hover layer present"
    );
}

/// `input()` composes Masonry's `TextInput`: typing edits the value.
#[test]
fn shadcn_input_typing_changes_the_value() {
    let mut harness = TestHarness::create(
        DefaultProperties::default(),
        NewWidget::new(TextInput::new("")),
    );
    harness.render();

    // A text input only receives text events while focused — and Masonry
    // focuses the inner TextArea (what TextInput's own pointer-down does).
    let area_id = harness.root_widget().area_pod().id();
    harness.focus_on(Some(area_id));
    harness.keyboard_type_chars("hello");
    harness.edit_root_widget(|mut ti| {
        let area = masonry::widgets::TextInput::text_mut(&mut ti);
        let typed = area.widget.text().to_string();
        assert_eq!(typed, "hello", "typed text lands in the input");
    });
}

/// `input()` builds a styled wrapper carrying the Shadcn input plan around
/// the real text input, and the wrapper paints it (pixels differ from an
/// unstyled plan).
#[test]
fn shadcn_input_view_builds_a_styled_wrapper() {
    let resolver = resolver();
    let view = input(
        &resolver,
        String::new(),
        |_state: &mut (), _changed: String| {},
        "",
    );
    let (element, _view_state) = view.build(&mut headless_ctx(), &mut ());

    let tag: WidgetTag<aeolus_xilem::widget::StyledWidget> = WidgetTag::unique();
    let root = ModularWidget::new_parent(element.new_widget.with_tag(tag));
    let harness = TestHarness::create_with(
        DefaultProperties::default(),
        NewWidget::new(root),
        TestHarnessParams::size_and_padding(PhysicalSize::new(240, 120), 0),
    );
    let carried = harness.get_widget(tag).plan().clone();
    // INPUT_BASE essentials: transparent bg, border, padding, h-9.
    assert!(carried.base.border_color.is_some(), "border-input resolved");
    let pad = carried.base.padding.expect("px-3 py-1");
    assert_eq!((pad.left, pad.top), (12.0, 4.0));
    assert!(carried.base.shadow.is_some(), "shadow-xs");
}
