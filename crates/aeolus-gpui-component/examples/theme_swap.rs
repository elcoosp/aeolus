//! The §3.4 theme hot-swap, made real: load `themes/light.css`, render a
//! component, absorb `themes/dark.css`, re-render — the SAME class
//! string, the SAME view-state [`ClassCell`], different token values, no
//! restart. The bridge's `style()` twin resolves the walk-time token
//! slots against the CURRENT table (the §3.4 T2 half), so the swap only
//! moves values — the compiled program (and the cell's identity rule)
//! survives untouched, and the resolution cache + the v0.x plan lane die
//! with the version they resolved under.
//!
//! The ActiveTheme twin's answer is the same palette gpui-component's
//! components read through their own trait: the mirror re-projects from
//! the swapped table, one `apply_swap` away from the App global (the
//! headless demo prints the projection; an app calls
//! `theme.apply_swap(cx)` instead).
//!
//! ```sh
//! cargo run -p aeolus-gpui-component --example theme_swap
//! ```

use aeolus_gpui_component::{ActiveTheme, AeolusTheme};
use aeolus_rt::ClassCell;

const LIGHT: &str = include_str!("fixtures/light.css");
const DARK: &str = include_str!("fixtures/dark.css");

/// The shadcn Button's semantic surface — tokens, opacity, state layer.
const BUTTON: &str = "bg-primary text-primary-foreground hover:bg-primary/90 px-4 py-2 rounded-md text-sm font-medium";

fn main() {
    let mut theme = AeolusTheme::from_css(LIGHT, None).expect("the light sheet parses");
    let cell = ClassCell::new();
    let env = aeolus_gpui_component::Env::default();

    println!("── light ──");
    let light = theme.style(gpui::StyleRefinement::default(), &cell, BUTTON, env);
    println!("background: {:?}", light.background);
    println!("text:       {:?}", light.text);
    println!("primary:    {:?}", theme.theme().primary);
    println!("theme mode: {:?}", theme.theme().mode);
    println!("bridge ver: {}", theme.version());
    println!(
        "compiled:   {} layers (the cell's own program)",
        cell.program().layers.len()
    );

    // THE SWAP: same handle, same cell, same class string — the sheet
    // merges, the values move, nothing re-parses. An app would call
    // `theme.apply_swap(cx)` here to reach the components' global.
    theme.absorb_css(DARK).expect("the dark sheet merges");
    theme.set_dark(true);

    println!("── dark (after absorb_css + set_dark, same cell) ──");
    let dark = theme.style(gpui::StyleRefinement::default(), &cell, BUTTON, env);
    println!("background: {:?}", dark.background);
    println!("text:       {:?}", dark.text);
    println!("primary:    {:?}", theme.theme().primary);
    println!("theme mode: {:?}", theme.theme().mode);
    println!("bridge ver: {}", theme.version());
    println!(
        "compiled:   {} layers (IDENTICAL program — the swap did not re-parse)",
        cell.program().layers.len()
    );

    assert_ne!(
        light.background, dark.background,
        "the values moved under the same program"
    );
    println!("✓ same class string, new token values, no restart — the §3.4 hot-swap contract");
}
