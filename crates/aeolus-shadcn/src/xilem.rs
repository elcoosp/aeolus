//! Thin Xilem wrappers over the Shadcn v1 class tables.
//!
//! Every wrapper is a Shadcn-styled box around a Masonry `Label` — the user
//! `class` is merged LAST via [`cn`](aeolus_core::cn), so it wins over the
//! component defaults (mission brief §3 API contract). Layout utilities
//! (`inline-flex items-center justify-center`) map to the centered box.

use aeolus_xilem::view::TwViewExt;
use xilem_masonry::view::label as xilem_label;
use xilem_masonry::WidgetView;

use crate::classes::{alert_classes, badge_classes, button_classes, AlertVariant, BadgeVariant, ButtonSize, ButtonVariant, CARD, CARD_DESCRIPTION, CARD_FOOTER, CARD_HEADER, CARD_TITLE, LABEL_BASE};
use aeolus_core::{cn, Resolver};

/// A Shadcn Button: styled, centered label.
///
/// ```ignore
/// aeolus_shadcn::xilem::button(&resolver, "save", "Save",
///     ButtonVariant::Default, ButtonSize::Default, "shadow-lg")
/// ```
pub fn button<'a>(
    resolver: &Resolver<'a>,
    _id: &str,
    text: &'static str,
    variant: ButtonVariant,
    size: ButtonSize,
    class: &str,
) -> impl WidgetView<()> {
    let classes = cn(&[&button_classes(variant, size), class]);
    xilem_label(text).tw_centered(resolver, &classes)
}

/// A Shadcn Badge.
pub fn badge<'a>(
    resolver: &Resolver<'a>,
    text: &'static str,
    variant: BadgeVariant,
    class: &str,
) -> impl WidgetView<()> {
    let classes = cn(&[&badge_classes(variant), class]);
    xilem_label(text).tw_centered(resolver, &classes)
}

/// A Shadcn Label.
pub fn label<'a>(resolver: &Resolver<'a>, text: &'static str, class: &str) -> impl WidgetView<()> {
    let classes = cn(&[LABEL_BASE, class]);
    xilem_label(text).tw(resolver, &classes)
}

/// A Shadcn Card title.
pub fn card_title<'a>(
    resolver: &Resolver<'a>,
    text: &'static str,
    class: &str,
) -> impl WidgetView<()> {
    let classes = cn(&[CARD_TITLE, class]);
    xilem_label(text).tw(resolver, &classes)
}

/// A Shadcn Card description.
pub fn card_description<'a>(
    resolver: &Resolver<'a>,
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
pub fn card<'a, V: WidgetView<()>>(
    resolver: &Resolver<'a>,
    child: V,
    class: &str,
) -> aeolus_xilem::view::Tw<V, (), ()> {
    let classes = cn(&[CARD, class]);
    child.tw(resolver, &classes)
}

/// Card header container (same layout caveat as [`card`]).
pub fn card_header<'a, V: WidgetView<()>>(
    resolver: &Resolver<'a>,
    child: V,
    class: &str,
) -> aeolus_xilem::view::Tw<V, (), ()> {
    let classes = cn(&[CARD_HEADER, class]);
    child.tw(resolver, &classes)
}

/// Card footer container (same layout caveat as [`card`]).
pub fn card_footer<'a, V: WidgetView<()>>(
    resolver: &Resolver<'a>,
    child: V,
    class: &str,
) -> aeolus_xilem::view::Tw<V, (), ()> {
    let classes = cn(&[CARD_FOOTER, class]);
    child.tw(resolver, &classes)
}

/// A Shadcn Alert (title + description as the child sequence).
pub fn alert<'a, V: WidgetView<()>>(
    resolver: &Resolver<'a>,
    child: V,
    variant: AlertVariant,
    class: &str,
) -> aeolus_xilem::view::Tw<V, (), ()> {
    let classes = cn(&[&alert_classes(variant), class]);
    child.tw(resolver, &classes)
}
