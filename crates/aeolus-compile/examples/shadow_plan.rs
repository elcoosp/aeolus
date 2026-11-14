//! Consumer demonstration: the plan bridge's shadow slot (the §3.7
//! shadow row's plan half, the §7 migration seam). The `shadow-*`
//! families the registry lowers into `Prop::BoxShadow` /
//! `Prop::ShadowColor` resolve through the oracle's OWN theme reader
//! into the parsed `BoxShadow` layers the v0.x plan slots carry —
//! `base.shadow` (the dominant layer), `extras.shadow_rest` (the
//! multi-layer rest) — so the five plan-path backends style real
//! shadow bands from V1 output. The demo prints the composed slots and
//! runs the differential walk beside the oracle's own `Resolver` (the
//! premise `tests/plan_bridge.rs` pins: one string, two generations in
//! agreement).
//!
//! ```sh
//! cargo run -p aeolus-compile --example shadow_plan
//! ```

use aeolus_compile::{Config, emit_plan_theme};
use aeolus_core::{Resolver, TailwindTheme};
use aeolus_theme::{ThemeSource, TokenTable};

fn main() {
    // The audited v4 default theme — the band source both generations
    // read (the bridge resolves the preset references through the
    // oracle's own theme reader; the differential's premise).
    let source = ThemeSource::from_css(aeolus_core::TAILWIND_DEFAULT_THEME, None)
        .expect("the audited theme parses");
    let tokens = TokenTable::baked(&source).expect("the source builds a table");

    // The preset scale, the clear, the recolour interactions — the
    // oracle's own pass order (the preset's wholesale replace beats the
    // recolour's patch, wherever the source order put the classes).
    for input in [
        "shadow-lg",
        "shadow-2xl",
        "shadow-none shadow-red-500",
        "shadow-lg shadow-red-500",
        "shadow-red-500 shadow-lg",
    ] {
        let sheet = emit_plan_theme(input, &tokens, &Config::default())
            .expect("the class string emits (only the empty input hard-fails)");
        let shadow = sheet.plan.base.shadow.as_ref();
        println!("── {input} ──");
        println!(
            "notes:           {} (the inset placeholder alone reports)",
            sheet.notes.len()
        );
        match shadow {
            Some(band) => {
                println!(
                    "base.shadow:     dx {} dy {} blur {} spread {} (alpha {:.2})",
                    band.offset_x, band.offset_y, band.blur, band.spread, band.color.alpha
                );
                println!(
                    "shadow_rest:     {} layer(s)",
                    sheet.plan.base.extras.shadow_rest.len()
                );
            }
            None => println!("base.shadow:     (nothing composes)"),
        }
    }

    // The differential walk: the bridge's plan beside the oracle's own
    // resolve of the SAME string — the two generations agree slot for
    // slot (the shadow tier's rows ride the battery).
    let theme = Box::leak(Box::new(
        TailwindTheme::from_theme_css(aeolus_core::TAILWIND_DEFAULT_THEME)
            .expect("the audited theme parses"),
    ));
    let resolver = Resolver::new(theme);
    let v1 = emit_plan_theme("shadow-lg shadow-red-500", &tokens, &Config::default())
        .expect("the class string emits");
    let v0 = resolver
        .resolve("shadow-lg shadow-red-500")
        .expect("the oracle resolves the shadow cascade");
    let same = match (&v1.plan.base.shadow, &v0.plan.base.shadow) {
        (Some(x), Some(y)) => {
            (x.offset_x, x.offset_y, x.blur, x.spread) == (y.offset_x, y.offset_y, y.blur, y.spread)
                && x.inset == y.inset
        }
        (None, None) => true,
        _ => false,
    };
    println!("── differential ──");
    println!(
        "shadow-lg shadow-red-500: V1 and V0 {} slot for slot (the recolour does not \
         survive the preset — the oracle's own pass order, pinned in tests/shadow.rs)",
        if same { "agree" } else { "DISAGREE" }
    );
}
