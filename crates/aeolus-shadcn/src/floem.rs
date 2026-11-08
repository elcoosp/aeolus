//! Thin Floem wrappers over the Shadcn v1 class tables.
//!
//! Floem's cascade is a superset of the Xilem one (real flex layout,
//! transitions), so these wrappers additionally consume the layout utilities
//! via the plan (`inline-flex items-center justify-center gap-*`); what is
//! still dropped is reported by the lowering, never silent.

use aeolus_floem::FloemTwExt;
use floem::{IntoView, views::Label};

use crate::classes::{
    AlertVariant, BadgeVariant, ButtonSize, ButtonVariant, CARD, CARD_DESCRIPTION, CARD_FOOTER,
    CARD_HEADER, CARD_TITLE, LABEL_BASE, alert_classes, badge_classes, button_classes,
};
use aeolus_core::{Resolver, cn};

/// A Shadcn Button (centered text via floem's own flex utilities).
pub fn button<'a>(
    resolver: &Resolver<'a>,
    _id: &str,
    label_text: &'static str,
    variant: ButtonVariant,
    size: ButtonSize,
    class: &str,
) -> impl IntoView {
    let classes = cn(&[&button_classes(variant, size), class]);
    let plan = match resolver.resolve(&classes) {
        Ok(r) => r.plan,
        Err(err) => {
            tracing::warn!("aeolus-shadcn: button classes unparseable: {err:?}");
            aeolus_core::StylePlan::default()
        }
    };
    Label::new(label_text).tw(&plan)
}

/// A Shadcn Badge.
pub fn badge<'a>(
    resolver: &Resolver<'a>,
    label_text: &'static str,
    variant: BadgeVariant,
    class: &str,
) -> impl IntoView {
    let classes = cn(&[&badge_classes(variant), class]);
    let plan = match resolver.resolve(&classes) {
        Ok(r) => r.plan,
        Err(err) => {
            tracing::warn!("aeolus-shadcn: badge classes unparseable: {err:?}");
            aeolus_core::StylePlan::default()
        }
    };
    Label::new(label_text).tw(&plan)
}

/// A Shadcn Label.
pub fn label<'a>(resolver: &Resolver<'a>, label_text: &'static str, class: &str) -> impl IntoView {
    let classes = cn(&[LABEL_BASE, class]);
    let plan = match resolver.resolve(&classes) {
        Ok(r) => r.plan,
        Err(err) => {
            tracing::warn!("aeolus-shadcn: label classes unparseable: {err:?}");
            aeolus_core::StylePlan::default()
        }
    };
    Label::new(label_text).tw(&plan)
}

/// Style any view sequence as a Shadcn Card (paint + flex column + gap).
pub fn card<'a, V: floem::views::Decorators>(
    resolver: &Resolver<'a>,
    child: V,
    class: &str,
) -> V::Intermediate {
    let classes = cn(&[CARD, class]);
    let plan = match resolver.resolve(&classes) {
        Ok(r) => r.plan,
        Err(err) => {
            tracing::warn!("aeolus-shadcn: card classes unparseable: {err:?}");
            aeolus_core::StylePlan::default()
        }
    };
    child.tw(&plan)
}

/// Card header container.
pub fn card_header<'a, V: floem::views::Decorators>(
    resolver: &Resolver<'a>,
    child: V,
    class: &str,
) -> V::Intermediate {
    let classes = cn(&[CARD_HEADER, class]);
    let plan = match resolver.resolve(&classes) {
        Ok(r) => r.plan,
        Err(err) => {
            tracing::warn!("aeolus-shadcn: card header classes unparseable: {err:?}");
            aeolus_core::StylePlan::default()
        }
    };
    child.tw(&plan)
}

/// Card footer container.
pub fn card_footer<'a, V: floem::views::Decorators>(
    resolver: &Resolver<'a>,
    child: V,
    class: &str,
) -> V::Intermediate {
    let classes = cn(&[CARD_FOOTER, class]);
    let plan = match resolver.resolve(&classes) {
        Ok(r) => r.plan,
        Err(err) => {
            tracing::warn!("aeolus-shadcn: card footer classes unparseable: {err:?}");
            aeolus_core::StylePlan::default()
        }
    };
    child.tw(&plan)
}

/// Style any view as a Shadcn Card title.
pub fn card_title<'a>(
    resolver: &Resolver<'a>,
    label_text: &'static str,
    class: &str,
) -> impl IntoView {
    let classes = cn(&[CARD_TITLE, class]);
    let plan = match resolver.resolve(&classes) {
        Ok(r) => r.plan,
        Err(err) => {
            tracing::warn!("aeolus-shadcn: card title classes unparseable: {err:?}");
            aeolus_core::StylePlan::default()
        }
    };
    Label::new(label_text).tw(&plan)
}

/// Style any view as a Shadcn Card description.
pub fn card_description<'a>(
    resolver: &Resolver<'a>,
    label_text: &'static str,
    class: &str,
) -> impl IntoView {
    let classes = cn(&[CARD_DESCRIPTION, class]);
    let plan = match resolver.resolve(&classes) {
        Ok(r) => r.plan,
        Err(err) => {
            tracing::warn!("aeolus-shadcn: card description classes unparseable: {err:?}");
            aeolus_core::StylePlan::default()
        }
    };
    Label::new(label_text).tw(&plan)
}

/// Style any view sequence as a Shadcn Alert.
pub fn alert<'a, V: floem::views::Decorators>(
    resolver: &Resolver<'a>,
    child: V,
    variant: AlertVariant,
    class: &str,
) -> V::Intermediate {
    let classes = cn(&[&alert_classes(variant), class]);
    let plan = match resolver.resolve(&classes) {
        Ok(r) => r.plan,
        Err(err) => {
            tracing::warn!("aeolus-shadcn: alert classes unparseable: {err:?}");
            aeolus_core::StylePlan::default()
        }
    };
    child.tw(&plan)
}
