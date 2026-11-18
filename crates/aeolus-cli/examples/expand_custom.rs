//! Consumer demonstration: the custom-authoring seam's CLI face (the
//! `expand --custom <path>` flag). A CSS file's `@custom-variant`
//! statements fold onto the typed `Cond` surface (the walk answers
//! them from the same probe facts the built-ins ride) and the
//! `@utility` composite bodies join the lookup with their longhands
//! compiled at registration. A missing or malformed file is the usage
//! error (exit 2), never a half-registered vocabulary.
//!
//! ```sh
//! cargo run -p aeolus-cli --example expand_custom
//! ```
//!
//! This is the headless form of:
//!
//! ```sh
//! aeolus expand --custom theme.css --dark 'p-4 brand-badge night:p-8'
//! ```

use aeolus_cli::run;

/// The project's theme CSS: one variant statement folding onto the
/// mode tier and one applied utility — the two statements a v4
/// project's own file carries.
const THEME_CSS: &str = "@custom-variant night (&:where(.dark, .dark *));\n\
                         @utility brand-badge { @apply rounded-md px-2 py-0.5; }\n";

fn main() {
    // The custom vocabulary rides a real file (the seam is CSS-first):
    // the temp file stands in for the project's theme.css.
    let dir = std::env::temp_dir().join("aeolus-cli-example-expand-custom");
    std::fs::create_dir_all(&dir).expect("the temp dir builds");
    let path = dir.join("theme.css");
    std::fs::write(&path, THEME_CSS).expect("the theme css writes");
    let custom = path.display().to_string();

    // The vocabulary-bearing form: the utility's composite body lands
    // its longhands, the `night:` alias answers from the mode fact.
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = run(
        [
            "expand".to_string(),
            "--custom".to_string(),
            custom.clone(),
            "--dark".to_string(),
            "p-4 brand-badge night:p-8".to_string(),
        ],
        &mut out,
        &mut err,
    );
    println!("$ aeolus expand --custom theme.css --dark … (exit {code})");
    print!("{}", String::from_utf8(out).unwrap_or_default());

    // The resting form: the utility still lowers, the `night:` alias
    // waits for the mode fact (the quiet miss a dark-mode-off run is).
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = run(
        [
            "expand".to_string(),
            "--custom".to_string(),
            custom,
            "p-4 brand-badge night:p-8".to_string(),
        ],
        &mut out,
        &mut err,
    );
    println!("\n$ aeolus expand --custom theme.css … (resting, exit {code})");
    print!("{}", String::from_utf8(out).unwrap_or_default());
    print!("{}", String::from_utf8(err).unwrap_or_default());
}
