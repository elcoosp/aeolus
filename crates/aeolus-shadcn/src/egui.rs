//! Thin egui wrappers over the Shadcn v1 class tables.
//!
//! egui resolves per frame, so these wrappers lower ONCE into a
//! `StyleBook` and resolve per state at call
//! time:
//!
//! - [`button`] is REAL and interactive: the book's native cascade
//!   (base → `inactive`, hover → `hovered`, active → `active`) is lowered
//!   onto `ui.visuals_mut().widgets` inside a `ui.scope`, and a plain
//!   `egui::widgets::Button` picks its appearance from those slots —
//!   hover/press restyling is egui's own state machinery.
//! - [`badge`], [`label`], the card family and [`alert`] are style-only
//!   frames around labels — they are *not* interactive and do not pretend
//!   to be.
//!
//! The user `class` is merged LAST via [`cn`], so it wins over the
//! component defaults (mission brief §3 API contract). Whatever the
//! backend cannot express is reported through the book's
//! `LoweringReport` and logged — never
//! silently ignored.

use aeolus_core::{Resolver, cn};
use aeolus_egui::{StyleBook, TwUiExt, apply_widget_visuals_cascade, style_text};

use crate::classes::{
    AlertVariant, BadgeVariant, ButtonSize, ButtonVariant, CARD, CARD_DESCRIPTION, CARD_FOOTER,
    CARD_HEADER, CARD_TITLE, LABEL_BASE, alert_classes, badge_classes, button_classes,
};

/// Resolve + lower the merged classes into a book (the honest contract:
/// panic in debug, warn + empty style in release).
fn book(resolver: &Resolver<'_>, classes: &str) -> StyleBook {
    StyleBook::tw(resolver, classes)
}

/// A REAL Shadcn Button on egui: an interactive `egui::widgets::Button`
/// styled by the v4 class table through the widget-visual cascade.
///
/// The text carries the book's typography hints (`font-medium`,
/// `text-sm`, `text-primary-foreground`); disabled/`focus-visible:`
/// conditions resolve per frame via [`StyleBook::resolve_at`] (egui 0.37
/// has no disabled/focused WidgetVisuals slots).
pub fn button(
    ui: &mut egui::Ui,
    resolver: &Resolver<'_>,
    label_text: &'static str,
    variant: ButtonVariant,
    size: ButtonSize,
    class: &str,
) -> egui::Response {
    let classes = cn(&[&button_classes(variant, size), class]);
    let book = book(resolver, &classes);
    ui.scope(|ui| {
        apply_widget_visuals_cascade(ui, &book);
        let mut text = egui::RichText::new(label_text);
        text = style_text(text, &book.base.text);
        ui.add(egui::widgets::Button::new(text))
    })
    .inner
}

/// A Shadcn Badge: a styled frame around a label.
pub fn badge(
    ui: &mut egui::Ui,
    resolver: &Resolver<'_>,
    label_text: &'static str,
    variant: BadgeVariant,
    class: &str,
) -> egui::Response {
    let classes = cn(&[&badge_classes(variant), class]);
    let book = book(resolver, &classes);
    framed_label(ui, &book, label_text)
}

/// A Shadcn Label: a styled frame around a label.
pub fn label(
    ui: &mut egui::Ui,
    resolver: &Resolver<'_>,
    label_text: &'static str,
    class: &str,
) -> egui::Response {
    let classes = cn(&[LABEL_BASE, class]);
    let book = book(resolver, &classes);
    framed_label(ui, &book, label_text)
}

/// A Shadcn Card body: base frame + content closure.
pub fn card<R>(
    ui: &mut egui::Ui,
    resolver: &Resolver<'_>,
    class: &str,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    let classes = cn(&[CARD, class]);
    let book = book(resolver, &classes);
    frame_show(ui, &book, add_contents)
}

/// Card header container.
pub fn card_header<R>(
    ui: &mut egui::Ui,
    resolver: &Resolver<'_>,
    class: &str,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    let classes = cn(&[CARD_HEADER, class]);
    let book = book(resolver, &classes);
    frame_show(ui, &book, add_contents)
}

/// Card footer container.
pub fn card_footer<R>(
    ui: &mut egui::Ui,
    resolver: &Resolver<'_>,
    class: &str,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    let classes = cn(&[CARD_FOOTER, class]);
    let book = book(resolver, &classes);
    frame_show(ui, &book, add_contents)
}

/// Card title: a styled frame around a label.
pub fn card_title(
    ui: &mut egui::Ui,
    resolver: &Resolver<'_>,
    label_text: &'static str,
    class: &str,
) -> egui::Response {
    let classes = cn(&[CARD_TITLE, class]);
    let book = book(resolver, &classes);
    framed_label(ui, &book, label_text)
}

/// Card description: a styled frame around a label.
pub fn card_description(
    ui: &mut egui::Ui,
    resolver: &Resolver<'_>,
    label_text: &'static str,
    class: &str,
) -> egui::Response {
    let classes = cn(&[CARD_DESCRIPTION, class]);
    let book = book(resolver, &classes);
    framed_label(ui, &book, label_text)
}

/// A Shadcn Alert: base frame + content closure.
pub fn alert<R>(
    ui: &mut egui::Ui,
    resolver: &Resolver<'_>,
    variant: AlertVariant,
    class: &str,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    let classes = cn(&[&alert_classes(variant), class]);
    let book = book(resolver, &classes);
    frame_show(ui, &book, add_contents)
}

/// Paint the book's base frame around a label carrying its typography.
fn framed_label(ui: &mut egui::Ui, book: &StyleBook, label_text: &'static str) -> egui::Response {
    frame_show(ui, book, |ui| {
        let mut text = egui::RichText::new(label_text);
        text = style_text(text, &book.base.text);
        ui.add(egui::Label::new(text))
    })
    .inner
}

/// Resolve for the context and show the frame, applying the hints the
/// scoped `Ui` can carry (wrap mode, selectability, spacing, sizes).
fn frame_show<R>(
    ui: &mut egui::Ui,
    book: &StyleBook,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    let style = book.resolve_for_ctx(ui.ctx());
    style
        .frame
        .show(ui, |ui| ui.tw_scope(&style, add_contents).inner)
}
