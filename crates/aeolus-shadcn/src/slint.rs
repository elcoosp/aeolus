//! Thin Slint wrappers over the Shadcn v1 class tables.
//!
//! Slint has no cascade and no runtime rule injection: element look is a
//! compile-time widget style, state restyling is authored with
//! `states [ name when condition : { … } ]`, and the generated-code path is
//! the capability audit's prescribed integration for exactly this kind of
//! framework. These wrappers therefore hand out BOTH surfaces:
//!
//! - **Books** ([`button_book`] and friends) — the merged class table
//!   lowered once into an [`Arc<StyleBook>`]; [`StyleBook::base_snippet`]
//!   emits the property bindings and [`StyleBook::to_states_snippet`] the
//!   `states` block that makes a `TouchArea`-driven element restyle itself
//!   (a REAL interactive shadcn Button in `.slint` terms — hover/press/
//!   disabled/focus are the element's own state machinery).
//! - **Rust-side resolution** — the same books resolve per situation via
//!   [`StyleBook::resolve_for`] with a
//!   [`SlintProbe`](aeolus_slint::SlintProbe) fed from the app's own
//!   properties, for programs that push values instead of generating
//!   `.slint`.
//!
//! The user `class` is merged LAST via [`cn`], so it wins over the
//! component defaults (mission brief §3 API contract). Whatever the
//! backend cannot express is reported through the book's
//! `LoweringReport` and logged — never silently ignored.

use std::sync::Arc;

use aeolus_core::{Resolver, cn};
use aeolus_slint::StyleBook;

use crate::classes::{
    AlertVariant, BadgeVariant, ButtonSize, ButtonVariant, CARD, CARD_DESCRIPTION, CARD_FOOTER,
    CARD_HEADER, CARD_TITLE, LABEL_BASE, alert_classes, badge_classes, button_classes,
};

/// Resolve + lower the merged classes into an owned book (the honest
/// contract: panic in debug, warn + empty style in release).
fn book(resolver: &Resolver<'_>, classes: &str) -> Arc<StyleBook> {
    Arc::new(StyleBook::tw(resolver, classes))
}

/// A REAL Shadcn Button book: lower once, then either
///
/// ```ignore
/// let book = aeolus_shadcn::slint::button_book(&resolver,
///     ButtonVariant::Default, ButtonSize::Default, "shadow-lg");
/// // (a) paste the generated code into your component:
/// println!("{}", book.base_snippet());
/// println!("{}", book.to_states_snippet("ta"));
/// // (b) or resolve per situation in Rust:
/// let style = book.resolve_for(&probe);
/// ```
///
/// The `hover:`/`active:`/`disabled:`/`focus-visible:` layers become
/// `states` gated on the named `TouchArea`'s `has-hover`/`pressed`/
/// `enabled`/`has-focus` — the element restyles itself, no app code.
pub fn button_book(
    resolver: &Resolver<'_>,
    variant: ButtonVariant,
    size: ButtonSize,
    class: &str,
) -> Arc<StyleBook> {
    let classes = cn(&[&button_classes(variant, size), class]);
    book(resolver, &classes)
}

/// A Shadcn Badge book (style-only: paint + radius + typography).
pub fn badge_book(resolver: &Resolver<'_>, variant: BadgeVariant, class: &str) -> Arc<StyleBook> {
    let classes = cn(&[&badge_classes(variant), class]);
    book(resolver, &classes)
}

/// A Shadcn Label book (style-only).
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
