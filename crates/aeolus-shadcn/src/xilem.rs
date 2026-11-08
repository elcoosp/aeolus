//! Thin Xilem wrappers over the Shadcn v1 class tables.
//!
//! Phase-3 Track A: the two INTERACTIVE components are real.
//!
//! - [`button`] — a real [`StyledButton`](aeolus_xilem::button::StyledButton)
//!   widget (pointer + keyboard + accesskit activation, disabled state)
//!   styled by the Shadcn Button class table, taking an `on_press` callback.
//! - [`input`] — Masonry's real `TextInput` styled by the Shadcn Input class
//!   table via the [`Tw`](aeolus_xilem::view::Tw) wrapper, with the standard
//!   controlled-value contract (`value` + `on_changed`).
//!
//! The remaining wrappers ([`badge`], [`label`], card family, [`alert`]) are
//! style-only boxes around labels — they are *not* interactive, and say so.
//!
//! The user `class` is merged LAST via [`cn`], so it wins
//! over the component defaults (mission brief §3 API contract). Layout
//! utilities (`inline-flex items-center justify-center`) map to the centered
//! box / the button's own centering.

use aeolus_core::StylePlan;
use aeolus_xilem::button::{StyledButtonView, styled_button};
use aeolus_xilem::view::TwViewExt;
use xilem_masonry::WidgetView;
use xilem_masonry::view::label as xilem_label;

use crate::classes::{
    AlertVariant, BadgeVariant, ButtonSize, ButtonVariant, CARD, CARD_DESCRIPTION, CARD_FOOTER,
    CARD_HEADER, CARD_TITLE, INPUT_BASE, LABEL_BASE, alert_classes, badge_classes, button_classes,
};
use aeolus_core::{Resolver, cn};

/// Resolve `classes`, degrading to an empty plan on parse failure (the same
/// honest contract as `TwViewExt::tw`: panic in debug, warn + no-op in
/// release).
fn resolve_plan(resolver: &Resolver<'_>, classes: &str) -> StylePlan {
    match resolver.resolve(classes) {
        Ok(resolution) => resolution.plan,
        Err(err) => {
            assert!(
                !cfg!(debug_assertions),
                "aeolus: failed to parse class string {classes:?}: {err:?}"
            );
            tracing::warn!("aeolus: dropping unparseable class string {classes:?}: {err:?}");
            StylePlan::default()
        }
    }
}

/// A REAL Shadcn Button: an interactive widget (pointer release, Space /
/// Enter while focused, accesskit Click), styled by the v4 class table.
///
/// ```ignore
/// aeolus_shadcn::xilem::button(&resolver, "save", "Save",
///     ButtonVariant::Default, ButtonSize::Default, "shadow-lg",
///     |state: &mut AppState| state.save(),
/// )
/// ```
///
/// Activation follows Masonry `Button` semantics: primary-pointer release
/// over the pressed button, Space/Enter when focused, or an accesskit
/// `Click`. Middle/right clicks are ignored (see `styled_button`).
pub fn button<'a, State, Action>(
    resolver: &Resolver<'a>,
    _id: &str,
    text: &'static str,
    variant: ButtonVariant,
    size: ButtonSize,
    class: &str,
    on_press: impl Fn(&mut State) -> Action + Send + Sync + 'static,
) -> StyledButtonView<
    State,
    Action,
    impl Fn(
        &mut State,
        Option<masonry::core::PointerButton>,
    ) -> xilem_masonry::core::MessageResult<Action>
    + Send
    + Sync
    + 'static,
    xilem_masonry::view::Label,
>
where
    State: 'static,
    Action: 'static,
{
    let classes = cn(&[&button_classes(variant, size), class]);
    let plan = resolve_plan(resolver, &classes);
    styled_button(xilem_label(text), on_press).with_plan(plan)
}

/// A REAL Shadcn Input: Masonry's `TextInput` styled by the v4 class table,
/// with the controlled-value contract (`value` in, `on_changed` out).
///
/// ```ignore
/// aeolus_shadcn::xilem::input(&resolver, state.email.clone(),
///     |state: &mut AppState, v: String| { state.email = v; },
///     "border-red-500",
/// )
/// ```
pub fn input<State, Action>(
    resolver: &Resolver<'_>,
    value: String,
    on_changed: impl Fn(&mut State, String) -> Action + Send + Sync + 'static,
    class: &str,
) -> aeolus_xilem::view::Tw<xilem_masonry::view::TextInput<State, Action>, State, Action>
where
    State: 'static,
    Action: 'static,
{
    let classes = cn(&[INPUT_BASE, class]);
    xilem_masonry::view::text_input(value, on_changed).tw(resolver, &classes)
}

/// A Shadcn Badge (style-only; badges are not interactive).
pub fn badge(
    resolver: &Resolver<'_>,
    text: &'static str,
    variant: BadgeVariant,
    class: &str,
) -> impl WidgetView<()> {
    let classes = cn(&[&badge_classes(variant), class]);
    xilem_label(text).tw_centered(resolver, &classes)
}

/// A Shadcn Label (style-only).
pub fn label(resolver: &Resolver<'_>, text: &'static str, class: &str) -> impl WidgetView<()> {
    let classes = cn(&[LABEL_BASE, class]);
    xilem_label(text).tw(resolver, &classes)
}

/// A Shadcn Card title (style-only).
pub fn card_title(resolver: &Resolver<'_>, text: &'static str, class: &str) -> impl WidgetView<()> {
    let classes = cn(&[CARD_TITLE, class]);
    xilem_label(text).tw(resolver, &classes)
}

/// A Shadcn Card description (style-only).
pub fn card_description(
    resolver: &Resolver<'_>,
    text: &'static str,
    class: &str,
) -> impl WidgetView<()> {
    let classes = cn(&[CARD_DESCRIPTION, class]);
    xilem_label(text).tw(resolver, &classes)
}

/// A Shadcn Card container (wrap `card_title`/`card_description` views).
///
/// Note: the v4 Card is a flex column with `gap-6`; until the Masonry layout
/// slot lands, the wrapper resolves the Card's *paint* properties (background,
/// border, radius, shadow, text color) and stacks children via the caller
/// (e.g. a xilem `v_flex`), which is documented in the capability matrix.
pub fn card<V: WidgetView<()>>(
    resolver: &Resolver<'_>,
    child: V,
    class: &str,
) -> aeolus_xilem::view::Tw<V, (), ()> {
    let classes = cn(&[CARD, class]);
    child.tw(resolver, &classes)
}

/// Card header container (same layout caveat as [`card`]).
pub fn card_header<V: WidgetView<()>>(
    resolver: &Resolver<'_>,
    child: V,
    class: &str,
) -> aeolus_xilem::view::Tw<V, (), ()> {
    let classes = cn(&[CARD_HEADER, class]);
    child.tw(resolver, &classes)
}

/// Card footer container (same layout caveat as [`card`]).
pub fn card_footer<V: WidgetView<()>>(
    resolver: &Resolver<'_>,
    child: V,
    class: &str,
) -> aeolus_xilem::view::Tw<V, (), ()> {
    let classes = cn(&[CARD_FOOTER, class]);
    child.tw(resolver, &classes)
}

/// A Shadcn Alert (title + description as the child sequence; style-only).
pub fn alert<V: WidgetView<()>>(
    resolver: &Resolver<'_>,
    child: V,
    variant: AlertVariant,
    class: &str,
) -> aeolus_xilem::view::Tw<V, (), ()> {
    let classes = cn(&[&alert_classes(variant), class]);
    child.tw(resolver, &classes)
}
