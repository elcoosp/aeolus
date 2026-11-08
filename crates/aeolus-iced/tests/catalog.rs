//! Catalog integration: the style closures type-check against Iced's own
//! `Catalog`/`StyleFn` contracts and the dark fallback behaves.

use std::sync::Arc;

use iced_widget::button::{Status, Style as ButtonStyle};
use iced_widget::container::{Style as ContainerStyle, StyleFn};

use aeolus_core::{Resolver, shadcn_default};
use aeolus_iced::StyleBook;

fn resolver() -> Resolver<'static> {
    let theme = shadcn_default().unwrap();
    Resolver::new(Box::leak(Box::new(theme)))
}

#[test]
fn container_style_fn_satisfies_iced_stylefn() {
    let resolver = resolver();
    let book: Arc<StyleBook> = Arc::new(StyleBook::tw(
        &resolver,
        "p-4 bg-primary text-primary-foreground rounded-md hover:bg-primary/90",
    ));
    // `StyleFn<'a, Theme>` = `Box<dyn Fn(&Theme) -> Style + 'a>` — the
    // closure produced by `container_style_fn` must slot straight in.
    let style_fn: StyleFn<'_, iced_core::Theme> = Box::new(book.container_style_fn());
    let s: ContainerStyle = style_fn(&iced_core::Theme::Light);
    assert!(s.background.is_some());
    assert!(s.text_color.is_some(), "text-primary-foreground lowered");
}

#[test]
fn button_style_fn_satisfies_iced_button_catalog() {
    let resolver = resolver();
    let book: Arc<StyleBook> = Arc::new(StyleBook::tw(
        &resolver,
        "bg-primary text-primary-foreground hover:bg-primary/90 disabled:opacity-50",
    ));
    let style_fn = book.button_style_fn();
    let light = iced_core::Theme::Light;
    let s: ButtonStyle = style_fn(&light, Status::Hovered);
    assert!(s.background.is_some());
    // `button::Style.text_color` is NOT optional: no class set one? The
    // shadcn string does, but the palette fallback is what the type needs.
    assert_eq!(s.text_color.a, 1.0);

    // The disabled layer's opacity-50 bakes into every alpha.
    let disabled = style_fn(&light, Status::Disabled);
    let bg = match disabled.background.unwrap() {
        iced_core::Background::Color(c) => c,
        other => panic!("{other:?}"),
    };
    assert!((bg.a - 0.5).abs() < 1e-3, "opacity-50 bakes: {}", bg.a);
}

#[test]
fn shadcn_button_string_is_interactive_end_to_end() {
    // The shadcn Default-variant button class table, verbatim (the
    // aeolus-shadcn wrappers are exercised there; this pins the lowering).
    const DEFAULT_BUTTON: &str = "inline-flex items-center justify-center gap-2 rounded-md text-sm font-medium \
whitespace-nowrap bg-primary text-primary-foreground hover:bg-primary/90";
    let resolver = resolver();
    let book = Arc::new(StyleBook::tw(&resolver, DEFAULT_BUTTON));
    let light = iced_core::Theme::Light;
    let idle = book.button_style(Status::Active, &light);
    let hovered = book.button_style(Status::Hovered, &light);
    let idle_bg = match idle.background.unwrap() {
        iced_core::Background::Color(c) => c,
        other => panic!("{other:?}"),
    };
    let hovered_bg = match hovered.background.unwrap() {
        iced_core::Background::Color(c) => c,
        other => panic!("{other:?}"),
    };
    // hover:bg-primary/90 is the SAME hue at 90% opacity (v4 color-mix
    // with transparent), so the full RGBA must differ — alpha included.
    assert_ne!(
        [
            (idle_bg.r * 255.0) as u8,
            (idle_bg.g * 255.0) as u8,
            (idle_bg.b * 255.0) as u8,
            (idle_bg.a * 255.0) as u8,
        ],
        [
            (hovered_bg.r * 255.0) as u8,
            (hovered_bg.g * 255.0) as u8,
            (hovered_bg.b * 255.0) as u8,
            (hovered_bg.a * 255.0) as u8,
        ],
        "hover:bg-primary/90 must restyle on hover (alpha 1.0 → 0.9)"
    );
    assert!(
        (idle_bg.a - 1.0).abs() < 1e-6 && (hovered_bg.a - 0.9).abs() < 1e-3,
        "the hover layer is primary at 90% opacity: {} → {}",
        idle_bg.a,
        hovered_bg.a
    );
}
