//! Consumer demonstration: the `@source` registration seam (plan §5.4's
//! CSS-first compatibility line, item 4) — the SAME theme CSS a web
//! Tailwind v4 project uses can register the content the class scan
//! walks, and the parse is typed end to end.
//!
//! ```sh
//! cargo run -p aeolus-theme --example theme_sources
//! ```
//!
//! The example walks one small theme file through the seam: the three
//! forms in source order, the carried form on the theme source itself,
//! and the honest typed failure for a malformed statement.

use aeolus_theme::{SourceDirective, ThemeSource, source_directives};

fn main() {
    // The theme file: the web v4 shape — @theme tokens plus the three
    // @source registration forms.
    let css = "\
@theme {
  --color-primary: #0ea5e9;
  --color-primary-foreground: #ffffff;
}

/* the scanner registration (paths resolve beside this file) */
@source inline(\"p-4 bg-primary text-primary-foreground hover:bg-primary/90\");
@source \"src/views.rs\";
@source not \"src/generated.rs\";
";

    // The standalone scan: the directive list in source order.
    let directives = source_directives(css).expect("well-formed directives parse");
    for directive in &directives {
        match directive {
            SourceDirective::Inline(content) => {
                println!("inline registration: {content:?}");
            }
            SourceDirective::Path { path, negated } => {
                println!(
                    "path registration: {path:?} ({})",
                    if *negated { "pruned" } else { "walked" }
                );
            }
        }
    }

    // The carried form: `ThemeSource::from_css` parses the tokens AND
    // the registrations in one walk — a host reads the registrations
    // beside the tiered token tables.
    let source = ThemeSource::from_css(css, None).expect("the theme parses");
    println!(
        "carried on the theme source: {} directive(s)",
        source.sources().len()
    );

    // The typed failure: a malformed statement is an error, never a
    // guessed registration.
    let malformed = "@source \"unterminated.rs;\n";
    match source_directives(malformed) {
        Ok(_) => unreachable!("the malformed form errors"),
        Err(err) => println!("typed failure: {err}"),
    }
}
