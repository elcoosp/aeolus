//! Consumer demonstration: the compile seam's wave-1 emission shapes
//! (the CSS arm's child-selector rules, the clamp composite and the
//! sr-only fan-out, the plan bridge's oracle-faithful clamp slots).
//! Headless — the same rendered sheets a server-side stylesheet or the
//! docs page would carry.
//!
//! ```sh
//! cargo run -p aeolus-compile --example wave1_css
//! ```

use aeolus_compile::{Config, emit_css, emit_css_theme, emit_plan};
use aeolus_core::TAILWIND_DEFAULT_THEME;
use aeolus_theme::{ThemeSource, TokenTable};

fn main() {
    println!("── parity wave 1: the CSS arm's five new families ──\n");

    // The default-palette baked table: the divide COLOUR arm rides the
    // token path (the theme pass's own bake), everything else compiles
    // either way.
    let source = ThemeSource::from_css(TAILWIND_DEFAULT_THEME, None).expect("default theme parses");
    let tokens = TokenTable::baked(&source).expect("the baked default table builds");

    // The between-child rule: the selector wraps the class compound in
    // v4's own `:where(& > :not(:last-child))` form (the real
    // tailwindcss v4.1.17 emission the corpus pins), the machinery
    // lines the reverse var composes.
    for class in ["space-x-4", "divide-y-2 divide-dashed", "divide-red-500"] {
        let sheet =
            emit_css_theme(class, &tokens, &Config::default()).expect("the wave-1 classes compile");
        println!("· {class}:");
        print!("{}", sheet);
        println!();
    }

    // The clamp composite: the overflow/-webkit-box pair beside the
    // clamp value (the oracle's own text), and its plan-bridge slots
    // (the v0.x oracle resolves the family — Count AND the ellipsis
    // overflow leg).
    let sheet = emit_css("line-clamp-2", &Config::default()).expect("compiles");
    println!("· line-clamp-2:");
    print!("{}", sheet);
    println!();
    let plan = emit_plan("line-clamp-2", &Config::default()).expect("the bridge lowers");
    println!("· line-clamp-2 plan slots: {:?}", plan.plan.base.text);
    println!();

    // The sr-only composite: the fifteen-row fan-out over the EXISTING
    // longhands (the truncate precedent's bigger sibling) + the clip
    // leg the real emission carries.
    let sheet = emit_css("sr-only", &Config::default()).expect("compiles");
    println!(
        "· sr-only ({} declarations):",
        sheet.rules[0].body.lines().count()
    );
    print!("{}", sheet);
}
