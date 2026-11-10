//! Consumer demonstration: the Slint backend's code generation — a
//! `StylePlan` into the element property bindings and `states [ … ]`
//! blocks a `.slint` component declares. Headless by construction:
//! the snippets are plain text the app pastes into its component (the
//! tests compile them with Slint's own compiler).
//!
//! ```sh
//! cargo run -p aeolus-slint --example style_snippet
//! ```

use aeolus_core::{Resolver, shadcn_default};
use aeolus_slint::{SlintProbe, StyleBook};

fn main() {
    let theme = shadcn_default().expect("the embedded shadcn default theme parses");
    let resolver = Resolver::new(Box::leak(Box::new(theme)));

    // The button surface a .slint component would declare: base
    // bindings + the hover/disabled state blocks.
    let book = StyleBook::tw(
        &resolver,
        "px-4 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90 disabled:opacity-50",
    );

    println!("── .slint element bindings (base) ──");
    println!("{}", book.base_snippet());

    println!("── .slint states (the TouchArea's has-hover/pressed/enabled gates) ──");
    println!("{}", book.to_states_snippet("touch"));

    // The parity path: the situation the element is in right now (the
    // TouchArea's flags) resolves through the probe — the same flags
    // the states block gates on, evaluated per frame by the app.
    let probe = SlintProbe {
        has_hover: true,
        ..SlintProbe::default()
    };
    let resolved = book.resolve_at(&probe);
    println!("── resolved for hover (brush snippet) ──");
    if let Some(brush) = &resolved.background {
        println!("  {}", aeolus_slint::brush_snippet(brush));
    }
}
