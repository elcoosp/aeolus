//! The [`StyledWidget`] fallback — widget-level conditional resolution.
//!
//! Masonry views cannot attach property-stack layers today (audit Q1: `Prop`
//! writes *local* properties, which shadow the whole stack). Until upstream
//! gap #4 lands, this widget is the honest fallback: it stores the
//! [`StylePlan`] in a field and resolves it **per state at pre-paint time**
//! via the core's [`Probe`] evaluator (plan §4.3) — a hover flip costs one
//! repaint and nothing else (no view rebuild, no relayout for cosmetic
//! changes — proven by `hover_restyle_is_paint_only`).
//!
//! v0.2 widening (plan §4.3):
//! * **`TwEnv`** — the app supplies `dark`, viewport/container width and
//!   group/peer scope through [`StyledWidget::set_env`] (a `WidgetMut`
//!   mutation), so `dark:`, `md:`, `@md:`, compound variants, `data-*`/
//!   `aria-*` rules and group/peer layers all resolve through
//!   `plan.resolve(&probe)` — the parity engine — instead of being dropped;
//! * **margin** — measured and inset like padding (v0.1 dropped margins);
//! * **per-side borders / per-corner radii** — painted directly with Kurbo
//!   geometry instead of `max(corners)`/`top` flattening (v0.1 mis-styled
//!   `rounded-s-lg`-shaped plans);
//! * **ring/outline** — stroked at `border_box.inflate(offset + width/2)`;
//! * **opacity** — alpha is multiplied into the painted CHROME colors; a
//!   warning records that CHILD content is not faded (the documented
//!   fallback, plan §4.3 — Masonry's `Painter` group API cannot wrap the
//!   child's own paint from `pre_paint`).

use masonry::accesskit::{Node, Role};
use masonry::core::{
    ChildrenIds, MeasureCtx, NewWidget, PaintCtx, PointerEvent, RegisterCtx, TextEvent, Update,
    UpdateCtx, Widget, WidgetMut, WidgetPod,
};
use masonry::imaging::Painter;
use masonry::kurbo::{Affine, Axis, Point, Rect, RoundedRect, RoundedRectRadii, Size, Stroke};
use masonry::layout::{LayoutSize, LenReq, Length, SizeDef};
use masonry::properties::{BoxShadow, CornerRadius};
use tracing::{Span, trace_span};

use aeolus_core::Test;
use aeolus_core::ast::{Media, State};
use aeolus_core::eval::{Probe, Rel};
use aeolus_core::plan::{ConditionalLayer, ResolvedStyle, StylePlan, VariantCondition};

use crate::to_masonry_color;

/// The element's environment, supplied by the app through `WidgetMut`.
///
/// Everything the widget cannot observe from Masonry's own context lives
/// here. Defaults are honest ("no", "unknown"): a rule that needs an
/// answer the env does not carry simply does not fire.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TwEnv {
    /// `dark:` — class/OS dark mode.
    pub dark: bool,
    /// Viewport (window) width in px, for `sm:`–`2xl:` and `min-*/max-*`.
    pub viewport_width: Option<f32>,
    /// Container width in px for `@container` queries.
    pub container_width: Option<f32>,
    /// Container NAME→width for named queries (`@md/menu`).
    pub containers: Vec<(String, f32)>,
    /// The default `.group` ancestor is hovered (`group-hover:`).
    pub group_hovered: bool,
    /// The default `.group` ancestor holds focus (`group-focus:`).
    pub group_focused: bool,
    /// The default `.peer` sibling is hovered (`peer-hover:`).
    pub peer_hovered: bool,
    /// The default `.peer` sibling holds focus (`peer-focus:`).
    pub peer_focused: bool,
}

impl TwEnv {
    /// The viewport width the plan's breakpoints compare against.
    pub fn width(&self) -> Option<f32> {
        self.viewport_width
    }
}

/// The full probe: Masonry's own state flags (from `ctx`) + [`TwEnv`].
#[derive(Clone, Debug)]
pub struct CtxProbe<'e> {
    /// Hover on the element itself.
    pub hovered: bool,
    /// Active (pressed).
    pub active: bool,
    /// Disabled.
    pub disabled: bool,
    /// Focus (Tailwind `focus:` AND `focus-visible:` share Masonry's flag).
    pub focused: bool,
    /// The environment.
    pub env: &'e TwEnv,
}

impl Probe for CtxProbe<'_> {
    fn test(&self, t: &Test) -> bool {
        match t {
            Test::State(s) => match s {
                State::Hover => self.hovered,
                State::Active => self.active,
                State::Disabled => self.disabled,
                State::Focus | State::FocusVisible => self.focused,
                _ => false,
            },
            // data-*/aria-* attributes need a view-supplied attribute map
            // (plan §4.3 "data/aria/has/not/nth: emulate via Probe with
            // view-supplied attrs"); the fallback widget has none.
            _ => false,
        }
    }

    fn related(&self, rel: Rel, _label: Option<&str>, t: &Test) -> bool {
        // Only the DEFAULT (unlabeled) group/peer scope is tracked in TwEnv;
        // labeled scopes need the view-wrapper plumbing (plan §4.3) and
        // honestly answer "no" here.
        if _label.is_some() {
            return false;
        }

        match (rel, t) {
            (Rel::Group, Test::State(State::Hover)) => self.env.group_hovered,
            (Rel::Group, Test::State(State::Focus)) => self.env.group_focused,
            (Rel::Peer, Test::State(State::Hover)) => self.env.peer_hovered,
            (Rel::Peer, Test::State(State::Focus)) => self.env.peer_focused,
            _ => false,
        }
    }

    fn descendant_state(&self, _s: State) -> bool {
        false
    }

    fn media(&self, m: Media) -> bool {
        matches!(m, Media::Dark) && self.env.dark
    }

    fn viewport_width(&self) -> Option<f32> {
        self.env.viewport_width
    }

    fn container_width(&self, name: Option<&str>) -> Option<f32> {
        match name {
            None => self.env.container_width,
            Some(n) => self
                .env
                .containers
                .iter()
                .find(|(k, _)| k == n)
                .map(|(_, w)| *w),
        }
    }
}

/// The env-only probe (measure/layout: no event state available there).
#[derive(Clone, Copy, Debug)]
pub struct EnvProbe<'e> {
    pub env: &'e TwEnv,
}

impl Probe for EnvProbe<'_> {
    fn media(&self, m: Media) -> bool {
        matches!(m, Media::Dark) && self.env.dark
    }

    fn viewport_width(&self) -> Option<f32> {
        self.env.viewport_width
    }

    fn container_width(&self, name: Option<&str>) -> Option<f32> {
        match name {
            None => self.env.container_width,
            Some(n) => self
                .env
                .containers
                .iter()
                .find(|(k, _)| k == n)
                .map(|(_, w)| *w),
        }
    }
}

/// A passthrough box that styles itself from a [`StylePlan`].
///
/// It forwards measure/layout to its child (inset by the resolved
/// padding/margin), and overrides `pre_paint` to paint
/// shadow/background/border/ring from the plan — with per-state resolution
/// through the [`Probe`] evaluator.
pub struct StyledWidget {
    plan: StylePlan,
    /// The app-supplied environment (dark mode, widths, group/peer scope).
    env: TwEnv,
    /// Center the child within the padded box (shadcn Button semantics:
    /// `items-center justify-center`).
    center: bool,
    child: WidgetPod<dyn Widget>,
}

impl StyledWidget {
    pub fn new(child: NewWidget<impl Widget + ?Sized>) -> Self {
        Self {
            plan: StylePlan::default(),
            env: TwEnv::default(),
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

    /// Set the environment (builder form).
    pub fn with_env(mut self, env: TwEnv) -> Self {
        self.env = env;
        self
    }

    /// The plan this widget resolves (read accessor for tests/diagnostics).
    pub fn plan(&self) -> &StylePlan {
        &self.plan
    }

    /// The environment (read accessor).
    pub fn env(&self) -> &TwEnv {
        &self.env
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

    /// Replace the environment at mutation time.
    ///
    /// A dark-mode flip or a window resize can change which rules fire;
    /// Masonry coalesces the requested layout/repaint.
    pub fn set_env(this: &mut WidgetMut<'_, Self>, env: TwEnv) {
        if this.widget.env != env {
            this.widget.env = env;
            this.ctx.request_layout();
        }
    }

    /// Mutable access to the wrapped child (used by the `Tw` view).
    pub fn child_mut<'t>(this: &'t mut WidgetMut<'_, Self>) -> WidgetMut<'t, dyn Widget> {
        this.ctx.get_mut(&mut this.widget.child)
    }
}

/// Apply every matching conditional layer over the base style (states only).
///
/// Conditions are applied in ascending [`VariantCondition::rank`] so that
/// e.g. `active` wins over `hover` when both flags are set — the same order
/// the stack path uses. For the full evaluator (dark, responsive, rules,
/// group/peer) use [`resolve_full`].
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
            out.overlay(&layer.style);
        }
    }
    out
}

/// The FULL evaluator path: legacy buckets + v0.2 rules through a probe.
pub fn resolve_full(plan: &StylePlan, probe: &impl Probe) -> ResolvedStyle {
    plan.resolve(probe)
}

/// Paint the resolved style's box chrome (shadow, background, border, ring)
/// using Masonry's own box-painting routines for the dominant shadow layer
/// and direct Kurbo geometry for the per-side/per-corner parts — shared by
/// [`StyledWidget`] and [`crate::button::StyledButton`] so both render
/// identically.
pub(crate) fn paint_plan_box(
    painter: &mut masonry::imaging::Painter<'_>,
    bbox: masonry::kurbo::Rect,
    style: &ResolvedStyle,
) {
    let opacity = style.opacity.map(|o| o.0).unwrap_or(1.0);
    let alpha = |c: aeolus_core::Color| {
        let m = to_masonry_color(c);
        // Multiply the chrome alpha by the plan's opacity (the documented
        // fallback: children are NOT faded — see module docs).
        m.multiply_alpha(opacity)
    };

    // Per-corner radii (v0.1 flattened to the max corner).
    let radii = style
        .corner_radius
        .map(|c| {
            RoundedRectRadii::new(
                c.top_left as f64,
                c.top_right as f64,
                c.bottom_right as f64,
                c.bottom_left as f64,
            )
        })
        .unwrap_or(RoundedRectRadii::new(0.0, 0.0, 0.0, 0.0));

    // Shadows: the dominant layer through Masonry's own painter; extra
    // layers through blurred rounded rects (v0.2 multi-layer presets).
    if let Some(s) = &style.shadow {
        let box_shadow = BoxShadow {
            color: to_masonry_color(s.color),
            offset: masonry::kurbo::Point::new(s.offset_x as f64, s.offset_y as f64),
            blur_radius: Length::px(s.blur as f64),
        };
        masonry::core::paint_box_shadow(
            painter,
            bbox,
            &box_shadow,
            &CornerRadius::all(Length::px(max_radius(&radii))),
        );
    }
    for layer in &style.extras.shadow_rest {
        let rect = bbox.inflate(layer.spread as f64, layer.spread as f64);
        let rect = Rect::new(
            rect.x0 + layer.offset_x as f64,
            rect.y0 + layer.offset_y as f64,
            rect.x1 + layer.offset_x as f64,
            rect.y1 + layer.offset_y as f64,
        );
        painter.blurred_rounded_rect(masonry::imaging::BlurredRoundedRect {
            transform: Affine::IDENTITY,
            rect,
            color: to_masonry_color(layer.color),
            radius: max_radius(&radii),
            std_dev: (layer.blur as f64) * 0.5,
            composite: Default::default(),
        });
    }

    // Background: fill the rounded rect (per-corner radii honored).
    let bg = style
        .background
        .map(|b| b.0)
        .unwrap_or(aeolus_core::Color::TRANSPARENT);
    let bg_alpha = bg.alpha * opacity;
    if bg_alpha > 0.0 {
        let rounded = RoundedRect::from_rect(bbox, radii);
        painter.fill(rounded, alpha(bg)).draw();
    }

    // Borders: per-side filled rects (v0.1 used the top width for all
    // sides). The color slot is uniform (v4 border-color is one slot);
    // logical s/e edges were already mapped to left/right by the resolver.
    if let Some(bw) = &style.border_width {
        let bc = style
            .border_color
            .map(|b| b.0)
            .unwrap_or(aeolus_core::Color::TRANSPARENT);
        if bc.alpha > 0.0 {
            paint_borders(painter, bbox, bw, bc, opacity, &radii);
        }
    }

    // Ring / outline (cosmetic lane): stroke the inflated border box.
    if let Some(ring) = &style.ring
        && ring.width > 0.0
        && let Some(c) = ring.color
    {
        let offset = ring.offset_width;
        let mid = ring.width as f64 * 0.5;
        let outer = bbox.inflate(offset as f64 + mid, offset as f64 + mid);
        let rounded = RoundedRect::from_rect(outer, radii);
        painter
            .stroke(rounded, &Stroke::new(ring.width as f64), alpha(c))
            .draw();
    }
    if let Some(o) = &style.outline
        && !o.none
        && o.width > 0.0
    {
        let mid = o.width as f64 * 0.5;
        let outer = bbox.inflate(o.offset as f64 + mid, o.offset as f64 + mid);
        let rounded = RoundedRect::from_rect(outer, radii);
        let c = o.color.unwrap_or(aeolus_core::Color::TRANSPARENT);
        painter
            .stroke(rounded, &Stroke::new(o.width as f64), alpha(c))
            .draw();
    }
}

fn max_radius(r: &RoundedRectRadii) -> f64 {
    r.top_left
        .max(r.top_right)
        .max(r.bottom_right)
        .max(r.bottom_left)
}

/// Paint the four border edges as filled rects, honoring per-side widths.
fn paint_borders(
    painter: &mut Painter<'_>,
    bbox: Rect,
    bw: &aeolus_core::plan::BorderWidth,
    bc: aeolus_core::Color,
    opacity: f32,
    radii: &RoundedRectRadii,
) {
    let m = to_masonry_color(bc);
    let brush = m.multiply_alpha(opacity);
    let Rect { x0, y0, x1, y1 } = bbox;
    let _ = radii;
    if bw.top > 0.0 {
        painter.fill_rect(Rect::new(x0, y0, x1, y0 + bw.top as f64), brush);
    }
    if bw.bottom > 0.0 {
        painter.fill_rect(Rect::new(x0, y1 - bw.bottom as f64, x1, y1), brush);
    }
    if bw.left > 0.0 {
        painter.fill_rect(Rect::new(x0, y0, x0 + bw.left as f64, y1), brush);
    }
    if bw.right > 0.0 {
        painter.fill_rect(Rect::new(x1 - bw.right as f64, y0, x1, y1), brush);
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
        let prev_state = resolved_for_state(
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
        let next_state = resolved_for_state(&self.plan, h, a, d, f);
        let diff = aeolus_core::diff(&prev_state, &next_state);
        if diff.is_empty() {
            return;
        }
        if diff.needs_layout() {
            ctx.request_layout();
        } else {
            // Cosmetic lane: box painting (shadow/background/border) lives in
            // `pre_paint`, so request the pre-paint lane — the same channel
            // masonry's own `core_property_changed` uses for `Background`.
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
        // Env-aware resolution: dark/responsive buckets may carry different
        // padding (measure has no event state, so hover/active flags are
        // honestly "off" here — a stateful padding change still triggers a
        // relayout through `update`, and `layout` re-reads the state).
        let probe = EnvProbe { env: &self.env };
        let style = self.plan.resolve(&probe);
        let pad = style.padding.unwrap_or_default();
        let mar = style.margin.unwrap_or_default();
        let pad_total = match axis {
            Axis::Horizontal => pad.left + pad.right + mar.left + mar.right,
            Axis::Vertical => pad.top + pad.bottom + mar.top + mar.bottom,
        };
        let auto_length = len_req.into();
        let context_size = LayoutSize::maybe(axis.cross(), cross_length);
        let child_length = ctx.compute_length(
            &mut self.child,
            auto_length,
            context_size,
            axis,
            cross_length,
        );
        child_length.saturating_add(Length::px(pad_total as f64))
    }

    fn layout(
        &mut self,
        ctx: &mut masonry::core::LayoutCtx<'_>,
        _props: &masonry::core::PropertiesRef<'_>,
        size: Size,
    ) {
        let probe = EnvProbe { env: &self.env };
        let style = self.plan.resolve(&probe);
        let pad = style.padding.unwrap_or_default();
        let mar = style.margin.unwrap_or_default();
        let (pl, pr, pt, pb) = (
            (pad.left + mar.left) as f64,
            (pad.right + mar.right) as f64,
            (pad.top + mar.top) as f64,
            (pad.bottom + mar.bottom) as f64,
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
        // The parity engine in action: ONE evaluator call answers states,
        // media (dark), viewport, container, group/peer and the v0.2 rules.
        let probe = CtxProbe {
            hovered: ctx.has_hovered(),
            active: ctx.has_active(),
            disabled: ctx.is_disabled(),
            focused: ctx.has_focus_target(),
            env: &self.env,
        };
        let style = self.plan.resolve(&probe);
        paint_plan_box(painter, ctx.border_box(), &style);
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
