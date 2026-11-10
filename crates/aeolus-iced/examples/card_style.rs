//! Consumer demonstration: the Iced backend's lowering — a
//! `StylePlan` into the native `container::Style` / `button::Style`
//! payloads an Iced Catalog closure reads, per theme and per widget
//! status. Headless by construction: the lowering is pure, the theme
//! is `iced_core::Theme`'s own default.
//!
//! ```sh
//! cargo run -p aeolus-iced --example card_style
//! ```

use iced_core::Theme;
use iced_widget::button;

use aeolus_core::{Resolver, shadcn_default};
use aeolus_iced::StyleBook;

fn main() {
    let theme = shadcn_default().expect("the embedded shadcn default theme parses");
    let resolver = Resolver::new(Box::leak(Box::new(theme)));

    // The card surface a container would wear, and the button that
    // sits inside it — the base + dark companion pattern.
    let book = StyleBook::tw(
        &resolver,
        "p-4 bg-card text-card-foreground rounded-md border",
    );
    let button_book = StyleBook::tw(
        &resolver,
        "px-4 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90",
    );

    let iced_theme = Theme::Light;
    println!("── iced container::Style (theme: Light) ──");
    let container = book.container_style(&iced_theme);
    println!("  background: {:?}", container.background);
    println!("  border:     {:?}", container.border);

    println!("── iced button::Style per status ──");
    for (name, status) in [
        ("idle", button::Status::Active),
        ("hovered", button::Status::Hovered),
        ("pressed", button::Status::Pressed),
    ] {
        let s = button_book.button_style(status, &iced_theme);
        println!(
            "  {name}: background={:?} text_color={:?}",
            s.background, s.text_color
        );
    }
}
