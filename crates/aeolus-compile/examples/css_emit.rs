//! Consumer demonstration: the `Target::Css` emit arm (architecture
//! plan §3.1's emit stage, option (d)) — a class string plus the SAME
//! `app.css` a web Tailwind v4 project uses render into a real
//! stylesheet, and every class the emitter does not lower comes back as
//! a spanned, coded, helped diagnostic (never a silent drop).
//!
//! ```sh
//! cargo run -p aeolus-compile --example css_emit
//! ```
//!
//! The demo feeds a realistic project theme file through the theme pass
//! (`from_css`, light + `.dark`), renders one Button surface with the
//! runtime token table (the `var()`/`color-mix` forms), one with the
//! baked table (the light/dark rule companions), and walks the honest
//! boundaries. The IR form of the same strings lives in
//! `compile_pipeline`; `theme_seam` shows the table tiers alone.

use aeolus_compile::{emit_css, emit_css_theme, Config};
use aeolus_theme::{ThemeSource, TokenTable};

/// A realistic project theme file — the `@theme` token block, the same
/// shape a web Tailwind v4 project checks in.
const APP_CSS: &str = r#"
@theme {
    --color-primary: oklch(0.208 0.042 265.755);
    --color-primary-foreground: oklch(0.984 0.003 256.848);
    --color-muted: oklch(0.968 0.007 247.896);
    --color-ring: oklch(0.551 0.027 264.364);
}
"#;

/// The `.dark` override block — the same file's dark half, passed
/// through `from_css`'s dark slot (the theme pass's dark variant).
const DARK_CSS: &str = r#"
.dark {
    --color-primary: oklch(0.9219 0.034 255.508);
    --color-primary-foreground: oklch(0.208 0.042 265.755);
    --color-muted: oklch(0.279 0.041 260.031);
    --color-ring: oklch(0.446 0.043 257.281);
}
"#;

fn main() {
    // 1. The theme pass: the CSS file resolves into a tiered token
    //    table — the SAME file a web project uses, unchanged.
    let source = ThemeSource::from_css(APP_CSS, Some(DARK_CSS))
        .expect("the theme file parses");
    let tokens = TokenTable::runtime(&source).expect("the table builds");
    println!(
        "theme file    : {} colour tokens, {} tier\n",
        tokens.len(),
        match tokens.tier() {
            aeolus_theme::Tier::Baked => "T0/T1 (baked)",
            aeolus_theme::Tier::Runtime => "T2 (runtime Tok)",
        }
    );

    // 2. The runtime table renders `var()` references — the theme swap
    //    is the host stylesheet's `.dark` block, exactly like the web.
    let sheet = emit_css_theme(
        "bg-primary text-primary-foreground rounded-md p-4 px-8 hover:bg-primary/90 \
         focus-visible:ring-ring/50 md:p-6",
        &tokens,
        &Config::default(),
    )
    .expect("non-empty input renders");
    println!("── runtime table (T2): var() + color-mix ───────────────\n");
    print!("{sheet}");

    // 3. The baked table renders literals; a token with a dark value
    //    becomes TWO rules (light under the rule's own selector, dark
    //    under `:where(.dark, .dark *)`).
    let baked = TokenTable::baked(&source).expect("the table builds");
    let sheet =
        emit_css_theme("bg-primary rounded-md", &baked, &Config::default())
            .expect("non-empty input renders");
    println!("\n── baked table (T0/T1): literals + the dark companion ──\n");
    print!("{sheet}");

    // 4. The theme-less default: colour tokens and modifiers stay typed
    //    boundaries — the honest refusal, never a dropped class.
    let sheet = emit_css("p-4 bg-primary/90 not-a-utility", &Config::default())
        .expect("non-empty input renders");
    println!("\n── theme-less: the typed boundaries ────────────────────\n");
    print!("{sheet}");
    println!();
    for diagnostic in &sheet.diagnostics {
        println!(
            "  {} [{}] → {}",
            diagnostic,
            diagnostic.kind.code(),
            diagnostic.kind.help()
        );
    }
}
