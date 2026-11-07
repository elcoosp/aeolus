//! Thin Iced wrappers over the Shadcn v1 class tables.
//!
//! Iced's cascade is a `Catalog` closure, so these wrappers lower ONCE
//! into a `StyleBook` (kept behind an
//! [`Arc`] in view state) and hand out the two native style functions:
//!
//! - [`button_book`] → `button(...).style(book.button_style_fn())` is a
//!   REAL interactive button: `Hovered`/`Pressed`/`Disabled` statuses map
//!   the `hover:`/`active:`/`disabled:` layers, `dark:` fires on dark
//!   themes.
//! - [`badge_book`], [`label_book`], the card family and [`alert_book`]
//!   are style-only books for `container(...).style(...)` — they are not
//!   interactive and do not pretend to be.
//!
//! The user `class` is merged LAST via [`cn`], so it wins over the
//! component defaults (mission brief §3 API contract). Whatever the
//! backend cannot express is reported through the book's
//! `LoweringReport` and logged — never
//! silently ignored. Focus/compound/group conditions resolve per frame
//! through [`StyleBook::resolve_at`](aeolus_iced::StyleBook::resolve_at)
//! with an `IcedProbe`.

use std::sync::Arc;

use aeolus_core::{Resolver, cn};
use aeolus_iced::StyleBook;

use crate::classes::{
    AlertVariant, BadgeVariant, ButtonSize, ButtonVariant, CARD, CARD_DESCRIPTION, CARD_FOOTER,
    CARD_HEADER, CARD_TITLE, LABEL_BASE, alert_classes, badge_classes, button_classes,
};

/// Resolve + lower the merged classes into an owned book (the honest
/// contract: panic in debug, warn + empty style in release).
fn book(resolver: &Resolver<'_>, classes: &str) -> Arc<StyleBook> {
    Arc::new(StyleBook::tw(resolver, classes))
}

/// A REAL Shadcn Button book: feed it to Iced's button catalog —
///
/// ```ignore
/// let save = aeolus_shadcn::iced::button_book(&resolver,
///     ButtonVariant::Default, ButtonSize::Default, "shadow-lg");
/// button(Text::new("Save changes")).style(save.button_style_fn())
/// ```
///
/// The v4 table's `focus-visible:*` conditions are reported by the book
/// (Iced 0.14's `button::Status` has no `Focused` variant); resolve them
/// per frame with `resolve_at` if the app tracks focus itself.
pub fn button_book(
    resolver: &Resolver<'_>,
    variant: ButtonVariant,
    size: ButtonSize,
    class: &str,
) -> Arc<StyleBook> {
    let classes = cn(&[&button_classes(variant, size), class]);
    book(resolver, &classes)
}

/// A Shadcn Badge book for `container(...).style(...)`.
pub fn badge_book(resolver: &Resolver<'_>, variant: BadgeVariant, class: &str) -> Arc<StyleBook> {
    let classes = cn(&[&badge_classes(variant), class]);
    book(resolver, &classes)
}

/// A Shadcn Label book for `container(...).style(...)`.
pub fn label_book(resolver: &Resolver<'_>, class: &str) -> Arc<StyleBook> {
    let classes = cn(&[LABEL_BASE, class]);
    book(resolver, &classes)
}

/// A Shadcn Card book (paint + rounded corners + shadow).
pub fn card_book(resolver: &Resolver<'_>, class: &str) -> Arc<StyleBook> {
    let classes = cn(&[CARD, class]);
    book(resolver, &classes)
}

/// Card header book.
pub fn card_header_book(resolver: &Resolver<'_>, class: &str) -> Arc<StyleBook> {
    let classes = cn(&[CARD_HEADER, class]);
    book(resolver, &classes)
}

/// Card footer book.
pub fn card_footer_book(resolver: &Resolver<'_>, class: &str) -> Arc<StyleBook> {
    let classes = cn(&[CARD_FOOTER, class]);
    book(resolver, &classes)
}

/// Card title book.
pub fn card_title_book(resolver: &Resolver<'_>, class: &str) -> Arc<StyleBook> {
    let classes = cn(&[CARD_TITLE, class]);
    book(resolver, &classes)
}

/// Card description book.
pub fn card_description_book(resolver: &Resolver<'_>, class: &str) -> Arc<StyleBook> {
    let classes = cn(&[CARD_DESCRIPTION, class]);
    book(resolver, &classes)
}

/// A Shadcn Alert book.
pub fn alert_book(resolver: &Resolver<'_>, variant: AlertVariant, class: &str) -> Arc<StyleBook> {
    let classes = cn(&[&alert_classes(variant), class]);
    book(resolver, &classes)
}
