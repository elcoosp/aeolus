//! The `Tw` view — wrap any widget view in a [`StyledWidget`] with a plan.
//!
//! ```ignore
//! use aeolus_xilem::TwViewExt;
//!
//! fn ui(state: &State) -> impl WidgetView<State> {
//!     label("Save").tw(&state.resolver, "p-4 bg-primary hover:bg-primary/90 rounded-md")
//! }
//! ```
//!
//! Because views cannot attach property-stack layers on today's Masonry
//! (audit Q1), `tw` wraps the child in a [`StyledWidget`] and inserts the
//! plan as a property. After upstream gap #4 (view-side stack registration)
//! lands, this view should attach the lowered stack instead — the wrapper
//! disappears and the classes flow straight into the child's cascade.

use std::marker::PhantomData;

use xilem_masonry::core::{MessageCtx, MessageResult, Mut, View, ViewMarker};
use xilem_masonry::{Pod, ViewCtx, WidgetView};

use aeolus_core::StylePlan;

use crate::widget::StyledWidget;

/// The [`View`] created by [`TwViewExt::tw`] / [`tw`].
#[must_use = "View values do nothing unless provided to Xilem."]
pub struct Tw<V, State, Action = ()> {
    child: V,
    plan: StylePlan,
    center: bool,
    phantom: PhantomData<fn() -> (State, Action)>,
}

/// Wrap `child` in a [`StyledWidget`] styled by `plan`.
pub fn tw<State, Action, V>(plan: StylePlan, child: V) -> Tw<V, State, Action>
where
    State: 'static,
    V: WidgetView<State, Action>,
{
    Tw {
        child,
        plan,
        center: false,
        phantom: PhantomData,
    }
}

/// Wrap `child` in a [`StyledWidget`] styled by `plan`, centering the child
/// (shadcn Button/Badge semantics: `items-center justify-center`).
pub fn tw_centered<State, Action, V>(plan: StylePlan, child: V) -> Tw<V, State, Action>
where
    State: 'static,
    V: WidgetView<State, Action>,
{
    Tw {
        child,
        plan,
        center: true,
        phantom: PhantomData,
    }
}

impl<V, State, Action> ViewMarker for Tw<V, State, Action> {}

impl<V, State, Action> View<State, Action, ViewCtx> for Tw<V, State, Action>
where
    State: 'static,
    Action: 'static,
    V: WidgetView<State, Action>,
{
    type Element = Pod<StyledWidget>;
    type ViewState = V::ViewState;

    fn build(&self, ctx: &mut ViewCtx, app_state: &mut State) -> (Self::Element, Self::ViewState) {
        let (child, child_state) = self.child.build(ctx, app_state);
        let widget = StyledWidget::new(child.new_widget)
            .with_plan(self.plan.clone())
            .centered(self.center);
        (Pod::new(widget), child_state)
    }

    fn rebuild(
        &self,
        prev: &Self,
        view_state: &mut Self::ViewState,
        ctx: &mut ViewCtx,
        mut element: Mut<'_, Self::Element>,
        app_state: &mut State,
    ) {
        self.child.rebuild(
            &prev.child,
            view_state,
            ctx,
            StyledWidget::child_mut(&mut element).downcast(),
            app_state,
        );
        // Only mutate when the plan actually changed; set_plan goes through
        // WidgetMut and requests the right invalidation.
        if self.plan != prev.plan {
            StyledWidget::set_plan(&mut element, self.plan.clone());
        }
    }

    fn teardown(
        &self,
        view_state: &mut Self::ViewState,
        ctx: &mut ViewCtx,
        mut element: Mut<'_, Self::Element>,
    ) {
        self.child.teardown(
            view_state,
            ctx,
            StyledWidget::child_mut(&mut element).downcast(),
        );
    }

    fn message(
        &self,
        view_state: &mut Self::ViewState,
        message: &mut MessageCtx,
        mut element: Mut<'_, Self::Element>,
        app_state: &mut State,
    ) -> MessageResult<Action> {
        self.child.message(
            view_state,
            message,
            StyledWidget::child_mut(&mut element).downcast(),
            app_state,
        )
    }
}

/// Extension methods for styling any [`WidgetView`] with a Tailwind plan.
pub trait TwViewExt<State: 'static, Action>: WidgetView<State, Action> {
    /// Wrap this view in a styled box driven by a pre-resolved [`StylePlan`].
    fn plan(self, plan: StylePlan) -> Tw<Self, State, Action>
    where
        State: 'static,
        Action: 'static,
        Self: Sized,
    {
        tw(plan, self)
    }

    /// Parse `classes` with `resolver` and wrap this view in a centered box
    /// (for labels inside buttons/badges: `items-center justify-center`).
    fn tw_centered(self, resolver: &aeolus_core::Resolver, classes: &str) -> Tw<Self, State, Action>
    where
        State: 'static,
        Action: 'static,
        Self: Sized,
    {
        match resolver.resolve(classes) {
            Ok(resolution) => tw_centered(resolution.plan, self),
            Err(err) => {
                assert!(
                    !cfg!(debug_assertions),
                    "aeolus: failed to parse class string {classes:?}: {err:?}"
                );
                tracing::warn!("aeolus: dropping unparseable class string {classes:?}: {err:?}");
                tw_centered(StylePlan::default(), self)
            }
        }
    }

    /// Parse `classes` with `resolver` and wrap this view in a styled box.
    ///
    /// The parse happens once per rebuild of the wrapping view; static call
    /// sites should prefer `aeolus-macro`'s `tw!` (compile-time interning).
    fn tw(self, resolver: &aeolus_core::Resolver, classes: &str) -> Tw<Self, State, Action>
    where
        State: 'static,
        Action: 'static,
        Self: Sized,
    {
        match resolver.resolve(classes) {
            Ok(resolution) => self.plan(resolution.plan),
            Err(err) => {
                // Honest failure: unparseable classes are a programmer error.
                // Panic loudly in debug, degrade to no-op styling in release.
                assert!(
                    !cfg!(debug_assertions),
                    "aeolus: failed to parse class string {classes:?}: {err:?}"
                );
                tracing::warn!("aeolus: dropping unparseable class string {classes:?}: {err:?}");
                self.plan(StylePlan::default())
            }
        }
    }
}

impl<V, State, Action> TwViewExt<State, Action> for V
where
    V: WidgetView<State, Action>,
    State: 'static,
{
}
