//! Integration proof for the `TwViewExt` view sugar (deliverable 4):
//!
//! `.tw(&resolver, classes)` must produce a view whose **build** output is a
//! `StyledWidget` pod carrying the resolved plan, with the child view (here a
//! `Label`) nested inside — and the resulting widget tree must actually style
//! itself in a real masonry run, including the hover flip.
//!
//! The view is built **headlessly** (`xilem_masonry::ViewCtx::new` is public
//! and needs only a proxy + tokio runtime handle), then the returned pod is
//! attached to a real masonry `TestHarness`. This exercises the whole chain:
//!
//! ```text
//! classes → Resolver → StylePlan → Tw::build → Pod(StyledWidget{plan})
//!         → masonry tree → StyledWidget::update/pre_paint → pixels
//! ```

use std::sync::Arc;

use aeolus_core::{Resolver, shadcn_default};
use aeolus_xilem::view::{Tw, TwViewExt};
use aeolus_xilem::widget::StyledWidget;
use masonry::core::{DefaultProperties, NewWidget, WidgetTag};
use masonry::dpi::PhysicalSize;
use masonry_testing::{ModularWidget, TestHarness, TestHarnessParams};
use xilem_masonry::core::{ProxyError, RawProxy, SendMessage, View as _, ViewId};
use xilem_masonry::view::label;
use xilem_masonry::ViewCtx;

/// A `RawProxy` that goes nowhere: the headless build never sends messages.
struct NoopProxy;

impl std::fmt::Debug for NoopProxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("NoopProxy")
    }
}

impl RawProxy for NoopProxy {
    fn send_message(
        &self,
        _path: Arc<[ViewId]>,
        _message: SendMessage,
    ) -> Result<(), ProxyError> {
        Ok(())
    }

    fn dyn_debug(&self) -> &dyn std::fmt::Debug {
        self
    }
}

#[test]
fn tw_view_sugar_builds_a_styled_widget_tree() {
    let theme = shadcn_default().unwrap();
    let resolver = Resolver::new(&theme);

    // --- view build (xilem side, headless) ----------------------------------
    let mut ctx = ViewCtx::new(
        Arc::new(NoopProxy),
        Arc::new(tokio::runtime::Runtime::new().unwrap()),
    );
    let view: Tw<_, (), ()> =
        label("OK").tw(&resolver, "p-4 bg-primary hover:bg-primary/90 rounded-md");
    let (element, _view_state) = view.build(&mut ctx, &mut ());

    // The element is the styled wrapper; pull the `NewWidget` out of the pod
    // and attach it to a real masonry tree.
    let masonry_pod = element.new_widget;

    // --- masonry side --------------------------------------------------------
    let tag: WidgetTag<StyledWidget> = WidgetTag::unique();
    let root = ModularWidget::new_parent(masonry_pod.with_tag(tag));
    let mut harness = TestHarness::create_with(
        DefaultProperties::default(),
        NewWidget::new(root),
        TestHarnessParams::size_and_padding(PhysicalSize::new(200, 200), 0),
    );

    // The plan traveled through the wrapper…
    let carried = harness.get_widget(tag).plan().clone();
    assert!(
        carried.base.background.is_some(),
        "plan carries the base background"
    );
    assert_eq!(carried.layers.len(), 1, "hover layer is in the carried plan");
    let p = carried.base.padding.expect("p-4 padding");
    assert_eq!((p.left, p.top), (16.0, 16.0), "view path lowers padding 16px");

    // …and the widget styles itself: hover over the label flips the pixels.
    let before = harness.render();
    harness.mouse_move(masonry::kurbo::Point::new(24., 24.));
    let after = harness.render();
    assert_ne!(before, after, "hover via .tw() must change the render");
}
