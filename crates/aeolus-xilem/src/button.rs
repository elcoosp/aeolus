//! [`StyledButton`] — a REAL interactive button styled from a [`StylePlan`].
//!
//! Phase-3 Track A. Interactivity mirrors Masonry's own `Button` widget
//! (`masonry/src/widgets/button.rs`): pointer capture, `active`+`hovered`
//! release semantics, Space/Enter keyboard activation, accesskit `Click`,
//! and central disabled handling (Masonry routes no pointer/text events to
//! disabled widgets; the widget additionally guards on `is_disabled`).
//! Painting comes from the plan's per-state resolution — the same
//! [`resolved_for_state`] + box-painting path the [`StyledWidget`](crate::widget::StyledWidget) fallback
//! uses, so hover/active/disabled/focus layers render like CSS.
//!
//! The [`styled_button`] view wraps the widget with a callback, following
//! `xilem_masonry::view::button`'s action-routing pattern
//! (`with_action_widget` / `teardown_action_source` / `take_message`).

use masonry::accesskit::{Node, Role};
use masonry::core::keyboard::{Key, NamedKey};
use masonry::core::{
    AccessCtx, AccessEvent, ChildrenIds, EventCtx, LayoutCtx, MeasureCtx, NewWidget, PaintCtx,
    PointerButton, PointerEvent, PropertiesMut, PropertiesRef, RegisterCtx, TextEvent, Update,
    UpdateCtx, Widget, WidgetId, WidgetMut, WidgetPod,
};
use masonry::imaging::Painter;
use masonry::kurbo::{Axis, Size};
use masonry::layout::{LayoutSize, LenReq, Length, SizeDef};
use tracing::{Span, trace_span};

use xilem_masonry::core::{
    MessageCtx, MessageResult, Mut, View, ViewId, ViewMarker, ViewPathTracker,
};
use xilem_masonry::{Pod, ViewCtx, WidgetView};

use aeolus_core::plan::{SizedValue, StylePlan};

use crate::widget::{paint_plan_box, resolved_for_state};

/// A button was activated (pointer release, keyboard, or accesskit click).
#[derive(Clone, Debug, PartialEq)]
pub struct StyledButtonPress {
    /// The pointer button that activated it; `None` for keyboard/accesskit.
    pub button: Option<PointerButton>,
}

/// A real, plan-styled button widget (see module docs).
pub struct StyledButton {
    plan: StylePlan,
    child: WidgetPod<dyn Widget>,
}

// --- MARK: BUILDERS
impl StyledButton {
    /// A button with `child` as its content (usually a non-interactive label).
    pub fn new(child: NewWidget<impl Widget + ?Sized>) -> Self {
        Self {
            plan: StylePlan::default(),
            child: child.erased().to_pod(),
        }
    }

    /// Style the button from a resolved plan (builder form).
    pub fn with_plan(mut self, plan: StylePlan) -> Self {
        self.plan = plan;
        self
    }

    /// The plan this button resolves (read accessor for tests/diagnostics).
    pub fn plan(&self) -> &StylePlan {
        &self.plan
    }

    /// Replace the plan at mutation time (`WidgetMut` channel).
    pub fn set_plan(this: &mut WidgetMut<'_, Self>, plan: StylePlan) {
        if this.widget.plan != plan {
            this.widget.plan = plan;
            this.ctx.request_layout();
        }
    }

    /// Mutable access to the wrapped child (used by the view).
    pub fn child_mut<'t>(this: &'t mut WidgetMut<'_, Self>) -> WidgetMut<'t, dyn Widget> {
        this.ctx.get_mut(&mut this.widget.child)
    }

    fn submit_press(ctx: &mut EventCtx<'_>, button: Option<PointerButton>) {
        // Belt-and-braces: Masonry already routes no pointer/text events to
        // disabled widgets (passes/event.rs `skip_if_disabled`), but a
        // disabled button must never submit, whatever the routing.
        if ctx.is_disabled() {
            trace_span!("StyledButton suppressed press (disabled)");
            return;
        }
        ctx.submit_action::<StyledButtonPress>(StyledButtonPress { button });
    }
}

// --- MARK: IMPL WIDGET
impl Widget for StyledButton {
    type Action = StyledButtonPress;

    fn on_pointer_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        event: &PointerEvent,
    ) {
        match event {
            PointerEvent::Down(..) => {
                ctx.request_focus();
                ctx.capture_pointer();
                // Capture/appearance change only — no accessibility change.
                ctx.request_paint_only();
            }
            PointerEvent::Up(pointer) => {
                // Masonry Button semantics: fire only when the release lands
                // on a button that is still held (active) and under the
                // pointer (hovered) — i.e. press + release on the same
                // widget. Dragging off cancels.
                if ctx.is_active() && ctx.is_hovered() {
                    Self::submit_press(ctx, pointer.button);
                }
                ctx.request_paint_only();
            }
            _ => (),
        }
    }

    fn on_text_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        event: &TextEvent,
    ) {
        match event {
            TextEvent::Keyboard(event)
                if event.state.is_up()
                    && (matches!(&event.key, Key::Character(c) if c == " ")
                        || event.key == Key::Named(NamedKey::Enter)) =>
            {
                Self::submit_press(ctx, None);
            }
            _ => (),
        }
    }

    fn on_access_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        event: &AccessEvent,
    ) {
        if event.action == masonry::accesskit::Action::Click {
            Self::submit_press(ctx, None);
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx<'_>, _props: &mut PropertiesMut<'_>, event: &Update) {
        // Same two-lane invalidation as the StyledWidget fallback: recompute
        // the plan for the state flip and request the cheapest invalidation
        // covering the changed slots.
        // The event carries the NEW flag value. For most lanes `ctx` already
        // reflects it, but the disabled lane flips `state.is_disabled` only
        // AFTER `update()` returns (masonry update_disabled_for_widget), so
        // `ctx.is_disabled()` is stale here — always use the event's value.
        let (h, a, d, f) = (
            ctx.has_hovered(),
            ctx.has_active(),
            ctx.is_disabled(),
            ctx.has_focus_target(),
        );
        let (h, a, d, f) = match event {
            Update::HoveredChanged(v) => (*v, a, d, f),
            Update::ActiveChanged(v) => (h, *v, d, f),
            Update::DisabledChanged(v) => (h, a, *v, f),
            Update::FocusChanged(v) => (h, a, d, *v),
            _ => return,
        };
        // Reconstruct the PREVIOUS state by flipping exactly this one flag.
        let prev = resolved_for_state(
            &self.plan,
            match event {
                Update::HoveredChanged(v) => !*v,
                _ => h,
            },
            match event {
                Update::ActiveChanged(v) => !*v,
                _ => a,
            },
            match event {
                Update::DisabledChanged(v) => !*v,
                _ => d,
            },
            match event {
                Update::FocusChanged(v) => !*v,
                _ => f,
            },
        );
        let next = resolved_for_state(&self.plan, h, a, d, f);
        let diff = aeolus_core::diff(&prev, &next);
        if diff.is_empty() {
            return;
        }
        if diff.needs_layout() {
            ctx.request_layout();
        } else {
            ctx.request_pre_paint();
        }
    }

    fn register_children(&mut self, ctx: &mut RegisterCtx<'_>) {
        ctx.register_child(&mut self.child);
    }

    fn measure(
        &mut self,
        ctx: &mut MeasureCtx<'_>,
        _props: &PropertiesRef<'_>,
        axis: Axis,
        len_req: LenReq,
        cross_length: Option<Length>,
    ) -> Length {
        let pad = self.plan.base.padding.unwrap_or_default();
        let pad_total = match axis {
            Axis::Horizontal => pad.left + pad.right,
            Axis::Vertical => pad.top + pad.bottom,
        };
        // Fixed dimensions from the plan act as MIN sizes (`h-9` buttons
        // measure at least 36px; the exact `Dimensions` property path is the
        // stack lowering's job — see the capability matrix).
        let fixed_min = match axis {
            Axis::Horizontal => self.plan.base.dimensions.and_then(|d| d.width),
            Axis::Vertical => self.plan.base.dimensions.and_then(|d| d.height),
        }
        .map_or(0.0, |v| match v {
            SizedValue::Px(px) => px,
            SizedValue::Scale(n) => n * 4.0,
            _ => 0.0,
        }) as f64;

        let auto_length = len_req.into();
        let context_size = LayoutSize::maybe(axis.cross(), cross_length);
        let child_length = ctx.compute_length(
            &mut self.child,
            auto_length,
            context_size,
            axis,
            cross_length,
        );
        child_length
            .saturating_add(Length::px(pad_total as f64))
            .max(Length::px(fixed_min))
    }

    fn layout(&mut self, ctx: &mut LayoutCtx<'_>, _props: &PropertiesRef<'_>, size: Size) {
        let pad = self.plan.base.padding.unwrap_or_default();
        let (pl, pr, pt, pb) = (
            pad.left as f64,
            pad.right as f64,
            pad.top as f64,
            pad.bottom as f64,
        );
        let inner = Size::new(
            (size.width - pl - pr).max(0.),
            (size.height - pt - pb).max(0.),
        );
        let child_size = ctx.compute_size(&mut self.child, SizeDef::fit(inner), inner.into());
        ctx.run_layout(&mut self.child, child_size);
        // Buttons center their content (shadcn: items-center justify-center).
        let origin = masonry::kurbo::Point::new(
            pl + ((inner.width - child_size.width) * 0.5).max(0.),
            pt + ((inner.height - child_size.height) * 0.5).max(0.),
        );
        ctx.place_child(&mut self.child, origin);
        ctx.derive_baselines(&self.child);
    }

    fn pre_paint(
        &mut self,
        ctx: &mut PaintCtx<'_>,
        _props: &PropertiesRef<'_>,
        painter: &mut Painter<'_>,
    ) {
        let style = resolved_for_state(
            &self.plan,
            ctx.has_hovered(),
            ctx.has_active(),
            ctx.is_disabled(),
            ctx.has_focus_target(),
        );
        paint_plan_box(painter, ctx.border_box(), &style);
    }

    fn paint(
        &mut self,
        _ctx: &mut PaintCtx<'_>,
        _props: &PropertiesRef<'_>,
        _painter: &mut Painter<'_>,
    ) {
    }

    fn accessibility_role(&self) -> Role {
        Role::Button
    }

    fn accessibility(
        &mut self,
        _ctx: &mut AccessCtx<'_>,
        _props: &PropertiesRef<'_>,
        node: &mut Node,
    ) {
        node.add_action(masonry::accesskit::Action::Click);
    }

    fn children_ids(&self) -> ChildrenIds {
        ChildrenIds::from_slice(&[self.child.id()])
    }

    fn propagates_pointer_interaction(&self) -> bool {
        // The button consumes pointer interaction (hover/active stop here).
        false
    }

    fn accepts_focus(&self) -> bool {
        true
    }

    fn accepts_text_input(&self) -> bool {
        false
    }

    fn make_trace_span(&self, id: WidgetId) -> Span {
        trace_span!("StyledButton", id = id.trace())
    }
}

// --- MARK: VIEW
/// Use a distinctive view id for the child slot (see
/// `xilem_masonry/src/view/button.rs` for why).
const BUTTON_CONTENT_VIEW_ID: ViewId = ViewId::new(0xae01_0001);

/// The [`View`] created by [`styled_button`].
#[must_use = "View values do nothing unless provided to Xilem."]
pub struct StyledButtonView<State, Action, F, V> {
    child: V,
    callback: F,
    plan: StylePlan,
    disabled: bool,
    phantom: std::marker::PhantomData<fn(State) -> Action>,
}

/// A real, plan-styled button view that calls `on_press` when activated.
///
/// Activation = primary-pointer release over the pressed button (Masonry
/// `active`+`hovered` semantics) OR Space/Enter while focused OR an
/// accesskit `Click`. Middle/right clicks are reported as `Nop`, matching
/// `xilem_masonry::view::button`.
pub fn styled_button<State, Action, V, F>(
    child: V,
    on_press: F,
) -> StyledButtonView<
    State,
    Action,
    impl Fn(&mut State, Option<PointerButton>) -> MessageResult<Action> + Send + 'static,
    V,
>
where
    State: 'static,
    Action: 'static,
    V: WidgetView<State, Action>,
    F: Fn(&mut State) -> Action + Send + 'static,
{
    StyledButtonView {
        callback: move |state: &mut State, button| match button {
            None | Some(PointerButton::Primary) => MessageResult::Action(on_press(state)),
            _ => MessageResult::Nop,
        },
        child,
        plan: StylePlan::default(),
        disabled: false,
        phantom: std::marker::PhantomData,
    }
}

impl<State, Action, F, V> StyledButtonView<State, Action, F, V> {
    /// Style the button from a resolved plan (shadcn component defaults).
    pub fn with_plan(mut self, plan: StylePlan) -> Self {
        self.plan = plan;
        self
    }

    /// Set the disabled state of the widget.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl<State, Action, F, V> ViewMarker for StyledButtonView<State, Action, F, V> {}

impl<State, Action, F, V> View<State, Action, ViewCtx> for StyledButtonView<State, Action, F, V>
where
    State: 'static,
    Action: 'static,
    V: WidgetView<State, Action>,
    F: Fn(&mut State, Option<PointerButton>) -> MessageResult<Action> + Send + Sync + 'static,
{
    type Element = Pod<StyledButton>;
    type ViewState = V::ViewState;

    fn build(&self, ctx: &mut ViewCtx, app_state: &mut State) -> (Self::Element, Self::ViewState) {
        let (child, child_state) = ctx.with_id(BUTTON_CONTENT_VIEW_ID, |ctx| {
            View::<State, Action, _>::build(&self.child, ctx, app_state)
        });
        (
            ctx.with_action_widget(|ctx| {
                let mut pod = ctx
                    .create_pod(StyledButton::new(child.new_widget).with_plan(self.plan.clone()));
                pod.new_widget.options.disabled = self.disabled;
                pod
            }),
            child_state,
        )
    }

    fn rebuild(
        &self,
        prev: &Self,
        state: &mut Self::ViewState,
        ctx: &mut ViewCtx,
        mut element: Mut<'_, Self::Element>,
        app_state: &mut State,
    ) {
        if prev.disabled != self.disabled {
            element.ctx.set_disabled(self.disabled);
        }
        if self.plan != prev.plan {
            StyledButton::set_plan(&mut element, self.plan.clone());
        }
        ctx.with_id(BUTTON_CONTENT_VIEW_ID, |ctx| {
            View::<State, Action, _>::rebuild(
                &self.child,
                &prev.child,
                state,
                ctx,
                StyledButton::child_mut(&mut element).downcast(),
                app_state,
            );
        });
    }

    fn teardown(
        &self,
        view_state: &mut Self::ViewState,
        ctx: &mut ViewCtx,
        mut element: Mut<'_, Self::Element>,
    ) {
        ctx.with_id(BUTTON_CONTENT_VIEW_ID, |ctx| {
            View::<State, Action, _>::teardown(
                &self.child,
                view_state,
                ctx,
                StyledButton::child_mut(&mut element).downcast(),
            );
        });
        ctx.teardown_action_source(element);
    }

    fn message(
        &self,
        view_state: &mut Self::ViewState,
        message: &mut MessageCtx,
        mut element: Mut<'_, Self::Element>,
        app_state: &mut State,
    ) -> MessageResult<Action> {
        match message.take_first() {
            Some(BUTTON_CONTENT_VIEW_ID) => self.child.message(
                view_state,
                message,
                StyledButton::child_mut(&mut element).downcast(),
                app_state,
            ),
            None => match message.take_message::<StyledButtonPress>() {
                Some(press) => (self.callback)(app_state, press.button),
                None => {
                    tracing::error!("Wrong message type in StyledButtonView::message: {message:?}");
                    MessageResult::Stale
                }
            },
            _ => {
                tracing::warn!("Got unexpected id path in StyledButtonView::message");
                MessageResult::Stale
            }
        }
    }
}
