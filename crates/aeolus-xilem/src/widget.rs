//! The [`StyledWidget`] fallback — widget-level conditional resolution.
//!
//! Masonry views cannot attach property-stack layers today (audit Q1: `Prop`
//! writes *local* properties, which shadow the whole stack). Until upstream
//! gap #4 lands, this widget is the honest fallback: it stores the
//! [`StylePlan`] in a field and resolves it **per state at pre-paint time**,
//! so a hover flip costs one repaint and nothing else (no view rebuild, no
//! relayout for cosmetic changes — proven by `hover_restyle_is_paint_only`).
//!
//! The plan is mutated through `WidgetMut::set_plan`, which routes via
//! Masonry's normal mutation channel and requests the right invalidation.

use masonry::accesskit::{Node, Role};
use masonry::core::{
    ChildrenIds, MeasureCtx, NewWidget, PaintCtx, PointerEvent, RegisterCtx, TextEvent, Update,
    UpdateCtx, Widget, WidgetMut, WidgetPod,
};
use masonry::imaging::Painter;
use masonry::kurbo::{Axis, Point, Size};
use masonry::layout::{LayoutSize, LenReq, Length, SizeDef};
use masonry::properties::{Background, BorderColor, BorderWidth, BoxShadow, CornerRadius};
use tracing::{Span, trace_span};

use aeolus_core::plan::{ConditionalLayer, ResolvedStyle, StylePlan, VariantCondition};

use crate::to_masonry_color;

/// A passthrough box that styles itself from a [`StylePlan`].
///
/// It forwards measure/layout to its child (inset by the resolved padding),
/// and overrides `pre_paint` to paint shadow/background/border from the
/// plan — the same three-phase painting Masonry's default `pre_paint` does
/// from properties (`widget_paint.rs:48-62`), but with per-state resolution.
pub struct StyledWidget {
    plan: StylePlan,
    /// Center the child within the padded box (shadcn Button semantics:
    /// `items-center justify-center`).
    center: bool,
    child: WidgetPod<dyn Widget>,
}

impl StyledWidget {
    pub fn new(child: NewWidget<impl Widget + ?Sized>) -> Self {
        Self {
            plan: StylePlan::default(),
            center: false,
            child: child.erased().to_pod(),
        }
    }

    /// Center the child in the padded box (both axes).
    pub fn centered(mut self, center: bool) -> Self {
        self.center = center;
        self
    }

    /// Set the plan this widget resolves (builder form).
    pub fn with_plan(mut self, plan: StylePlan) -> Self {
        self.plan = plan;
        self
    }

    /// The plan this widget resolves (read accessor for tests/diagnostics).
    pub fn plan(&self) -> &StylePlan {
        &self.plan
    }

    /// Replace the plan at mutation time (`WidgetMut` channel).
    ///
    /// Requests layout unconditionally: a plan swap may change geometry
    /// (padding/dimensions); Masonry coalesces redundant layout passes.
    pub fn set_plan(this: &mut WidgetMut<'_, Self>, plan: StylePlan) {
        if this.widget.plan != plan {
            this.widget.plan = plan;
            this.ctx.request_layout();
        }
    }

    /// Mutable access to the wrapped child (used by the `Tw` view).
    pub fn child_mut<'t>(this: &'t mut WidgetMut<'_, Self>) -> WidgetMut<'t, dyn Widget> {
        this.ctx.get_mut(&mut this.widget.child)
    }
}

/// Apply every matching conditional layer over the base style.
///
/// Conditions are applied in ascending [`VariantCondition::rank`] so that
/// e.g. `active` wins over `hover` when both flags are set — the same order
/// the stack path uses. `hover`/`active` use the *has\_* accessors at the
/// call site, matching CSS `:hover` containment semantics for containers
/// (pointer over a descendant keeps the parent hovered).
pub fn resolved_for_state(
    plan: &StylePlan,
    hovered: bool,
    active: bool,
    disabled: bool,
    focused: bool,
) -> ResolvedStyle {
    let mut out = plan.base.clone();
    let mut layers: Vec<&ConditionalLayer> = plan.layers.iter().collect();
    layers.sort_by_key(|l| l.condition.rank());
    for layer in layers {
        let applies = match layer.condition {
            VariantCondition::Hover => hovered,
            VariantCondition::Active => active,
            VariantCondition::Disabled => disabled,
            VariantCondition::Focus | VariantCondition::FocusVisible => focused,
            // The fallback widget has no signal for these; the stack path
            // reports them as dropped instead (see `lower`).
            VariantCondition::FocusWithin | VariantCondition::Visited => false,
        };
        if applies {
            merge_into(&mut out, &layer.style);
        }
    }
    out
}

/// Per-slot `Some`-replacing merge (`over` wins where it is set).
fn merge_into(base: &mut ResolvedStyle, over: &ResolvedStyle) {
    macro_rules! m {
        ($($field:ident),* $(,)?) => {
            $(if over.$field.is_some() {
                base.$field = over.$field.clone();
            })*
        };
    }
    m!(
        background,
        content_color,
        padding,
        border_color,
        border_width,
        corner_radius,
        dimensions,
        min_dimensions,
        max_dimensions,
        gap,
        shadow,
        margin,
        layout,
        opacity,
        ring,
        outline,
        cursor,
        pointer_events,
    );
    for (prop, value) in &over.text {
        base.text.retain(|(p, _)| p != prop);
        base.text.push((*prop, *value));
    }
}

impl Widget for StyledWidget {
    type Action = ();

    fn on_pointer_event(
        &mut self,
        _ctx: &mut masonry::core::EventCtx<'_>,
        _props: &mut masonry::core::PropertiesMut<'_>,
        _event: &PointerEvent,
    ) {
    }

    fn on_text_event(
        &mut self,
        _ctx: &mut masonry::core::EventCtx<'_>,
        _props: &mut masonry::core::PropertiesMut<'_>,
        _event: &TextEvent,
    ) {
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx<'_>,
        _props: &mut masonry::core::PropertiesMut<'_>,
        event: &Update,
    ) {
        // Two-lane invalidation: recompute the plan for the state transition
        // this event represents and request the *cheapest* invalidation that
        // covers the changed slots (`Slot::lane` mirrors Masonry's
        // `core_property_changed` lanes).
        //
        // The event carries the NEW flag value; every other flag is at its
        // current value (read from ctx), so reconstructing the previous state
        // means flipping exactly this one flag back.
        let (mut h, mut a, mut d, mut f) = (
            ctx.has_hovered(),
            ctx.has_active(),
            ctx.is_disabled(),
            ctx.has_focus_target(),
        );
        match event {
            Update::HoveredChanged(v) => h = !*v,
            Update::ActiveChanged(v) => a = !*v,
            Update::DisabledChanged(v) => d = !*v,
            Update::FocusChanged(v) => f = !*v,
            _ => return,
        }
        let prev = resolved_for_state(&self.plan, h, a, d, f);
        // ctx already reflects the new state.
        let next = resolved_for_state(
            &self.plan,
            ctx.has_hovered(),
            ctx.has_active(),
            ctx.is_disabled(),
            ctx.has_focus_target(),
        );
        let diff = aeolus_core::diff(&prev, &next);
        if diff.is_empty() {
            return;
        }
        if diff.needs_layout() {
            ctx.request_layout();
        } else {
            // Cosmetic lane: box painting (shadow/background/border) lives in
            // `pre_paint`, so request the pre-paint lane — the same channel
            // masonry's own `core_property_changed` uses for `Background`
            // (`masonry_core/src/properties/mod.rs:29-42`).
            ctx.request_pre_paint();
        }
    }

    fn register_children(&mut self, ctx: &mut RegisterCtx<'_>) {
        ctx.register_child(&mut self.child);
    }

    fn measure(
        &mut self,
        ctx: &mut MeasureCtx<'_>,
        _props: &masonry::core::PropertiesRef<'_>,
        axis: Axis,
        len_req: LenReq,
        cross_length: Option<Length>,
    ) -> Length {
        let pad = self.plan.base.padding.unwrap_or_default();
        let pad_total = match axis {
            Axis::Horizontal => pad.left + pad.right,
            Axis::Vertical => pad.top + pad.bottom,
        };
        let auto_length = len_req.into();
        let context_size = LayoutSize::maybe(axis.cross(), cross_length);
        let child_length =
            ctx.compute_length(&mut self.child, auto_length, context_size, axis, cross_length);
        child_length.saturating_add(Length::px(pad_total as f64))
    }

    fn layout(
        &mut self,
        ctx: &mut masonry::core::LayoutCtx<'_>,
        _props: &masonry::core::PropertiesRef<'_>,
        size: Size,
    ) {
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
        let origin = if self.center {
            Point::new(
                pl + ((inner.width - child_size.width) * 0.5).max(0.),
                pt + ((inner.height - child_size.height) * 0.5).max(0.),
            )
        } else {
            Point::new(pl, pt)
        };
        ctx.place_child(&mut self.child, origin);
        ctx.derive_baselines(&self.child);
    }

    fn pre_paint(
        &mut self,
        ctx: &mut PaintCtx<'_>,
        _props: &masonry::core::PropertiesRef<'_>,
        painter: &mut Painter<'_>,
    ) {
        let style = resolved_for_state(
            &self.plan,
            ctx.has_hovered(),
            ctx.has_active(),
            ctx.is_disabled(),
            ctx.has_focus_target(),
        );
        let bbox = ctx.border_box();

        // Convert resolved slots into Masonry's property values and reuse the
        // engine's own box-painting routines — identical rendering to the
        // default property-driven path (DRY with widget_paint.rs).
        let corner_radius = {
            let r = style
                .corner_radius
                .map(|c| c.top_left.max(c.top_right).max(c.bottom_right).max(c.bottom_left))
                .unwrap_or(0.0);
            CornerRadius::all(Length::px(r as f64))
        };
        let border_width = {
            let w = style.border_width.map(|b| b.top).unwrap_or(0.0);
            BorderWidth::all(Length::px(w as f64))
        };
        let background = Background::Color(to_masonry_color(
            style
                .background
                .map(|b| b.0)
                .unwrap_or(aeolus_core::Color::TRANSPARENT),
        ));
        let border_color = BorderColor::new(to_masonry_color(
            style
                .border_color
                .map(|b| b.0)
                .unwrap_or(aeolus_core::Color::TRANSPARENT),
        ));
        let box_shadow = style.shadow.map(|s| BoxShadow {
            color: to_masonry_color(s.color),
            offset: masonry::kurbo::Point::new(s.offset_x as f64, s.offset_y as f64),
            blur_radius: Length::px(s.blur as f64),
        });

        masonry::core::paint_box_shadow(
            painter,
            bbox,
            &box_shadow.unwrap_or(BoxShadow::default()),
            &corner_radius,
        );
        masonry::core::paint_background(painter, bbox, &background, &border_width, &corner_radius);
        masonry::core::paint_border(painter, bbox, &border_color, &border_width, &corner_radius);
    }

    fn paint(
        &mut self,
        _ctx: &mut PaintCtx<'_>,
        _props: &masonry::core::PropertiesRef<'_>,
        _painter: &mut Painter<'_>,
    ) {
    }

    fn accessibility_role(&self) -> Role {
        Role::Group
    }

    fn accessibility(
        &mut self,
        _ctx: &mut masonry::core::AccessCtx<'_>,
        _props: &masonry::core::PropertiesRef<'_>,
        _node: &mut Node,
    ) {
    }

    fn children_ids(&self) -> ChildrenIds {
        ChildrenIds::from_slice(&[self.child.id()])
    }

    fn make_trace_span(&self, id: masonry::core::WidgetId) -> Span {
        trace_span!("StyledWidget", id = id.trace())
    }
}
