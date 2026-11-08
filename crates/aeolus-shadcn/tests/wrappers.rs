//! Smoke tests for the new Iced + egui wrappers over the Shadcn v4 class
//! tables: the books lower, the interactive button cascade changes on
//! hover, and the style-only wrappers carry their component colors.

use aeolus_core::{Resolver, shadcn_default};
use aeolus_shadcn::classes::{BadgeVariant, ButtonVariant};
use aeolus_shadcn::iced as shadcn_iced;

fn resolver() -> Resolver<'static> {
    let theme = shadcn_default().unwrap();
    Resolver::new(Box::leak(Box::new(theme)))
}

fn solid(bg: &iced_widget::core::Background) -> iced_widget::core::Color {
    match *bg {
        iced_widget::core::Background::Color(c) => c,
        iced_widget::core::Background::Gradient(g) => {
            panic!("expected a solid color, got gradient {g:?}")
        }
    }
}

#[test]
fn iced_button_book_is_interactive() {
    let resolver = resolver();
    let book = shadcn_iced::button_book(
        &resolver,
        ButtonVariant::Default,
        aeolus_shadcn::classes::ButtonSize::Default,
        "",
    );
    let light = iced_widget::core::Theme::Light;
    let idle = book.button_style(iced_widget::button::Status::Active, &light);
    let hovered = book.button_style(iced_widget::button::Status::Hovered, &light);
    let a = solid(idle.background.as_ref().unwrap());
    let b = solid(hovered.background.as_ref().unwrap());
    assert!(
        (a.a - b.a).abs() > 1e-3 || (a.r - b.r).abs() > 1e-3,
        "hover:bg-primary/90 must restyle on hover"
    );
}

#[test]
fn iced_badge_book_carries_the_variant_colors() {
    let resolver = resolver();
    let book = shadcn_iced::badge_book(&resolver, BadgeVariant::Secondary, "");
    let style = book.container_style(&iced_widget::core::Theme::Light);
    assert!(style.background.is_some());
    assert!(style.text_color.is_some(), "badge text token lowers");
}

#[test]
fn egui_button_frame_lowers_the_variant_fill() {
    let resolver = resolver();
    // The interactive `shadcn_egui::button` wrapper needs a live `Ui`
    // (immediate mode), so pin its style source instead: the Default
    // variant table lowers to the primary fill (#171717).
    let classes = aeolus_core::cn(&[
        &aeolus_shadcn::classes::button_classes(
            ButtonVariant::Default,
            aeolus_shadcn::classes::ButtonSize::Default,
        ),
        "",
    ]);
    let book = aeolus_egui::StyleBook::tw(&resolver, &classes);
    let [r, g, b, _a] = book.base.frame.fill.to_array();
    assert_eq!([r, g, b], [23, 23, 23], "bg-primary = #171717");
    assert_eq!(
        book.base.text.weight,
        Some(500),
        "font-medium carries into the RichText hints"
    );
}
