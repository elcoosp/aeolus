//! Consumer demonstration: the `build` command — the whole-project
//! stylesheet (scan → dedupe → registry-ordered rules + the
//! self-contained `:root`/`.dark` vars block). The walk reads the
//! source files' class strings, compiles every occurrence through the
//! same pipeline the other commands run, and renders ONE v4 sheet;
//! diagnostics ride at every occurrence (the honest-failure contract),
//! `--out` writes the file.
//!
//! ```sh
//! cargo run -p aeolus-cli --example build
//! ```
//!
//! This is the headless form of:
//!
//! ```sh
//! aeolus build src/ --out app.css
//! ```

use aeolus_cli::run;

fn main() {
    // The project stand-in: two source files' class strings, one
    // carrying a token the built-in theme resolves (`bg-primary`), one
    // plain. The scan walks the directory the operator names.
    let dir = std::env::temp_dir().join("aeolus-cli-example-build");
    std::fs::create_dir_all(&dir).expect("the temp dir builds");
    std::fs::write(
        dir.join("button.rs"),
        r#"fn button() {
    let classes = "inline-flex items-center rounded-md bg-primary px-4 py-2 hover:bg-primary/90";
}
"#,
    )
    .expect("the source writes");
    std::fs::write(
        dir.join("card.rs"),
        r#"fn card() {
    let classes = "rounded-lg border p-4 shadow-sm md:p-6";
}
"#,
    )
    .expect("the source writes");

    // The whole-project build: scan → dedupe → the sheet, written to
    // --out (exit 0 — every class lowers on the built-in theme).
    let mut out = Vec::new();
    let mut err = Vec::new();
    let sheet = dir.join("app.css");
    let code = run(
        [
            "build".to_string(),
            dir.display().to_string(),
            "--out".to_string(),
            sheet.display().to_string(),
        ],
        &mut out,
        &mut err,
    );
    println!("$ aeolus build <dir> --out app.css (exit {code})");
    print!("{}", String::from_utf8(out).unwrap_or_default());
    print!("{}", String::from_utf8(err).unwrap_or_default());
    if let Ok(sheet) = std::fs::read_to_string(&sheet) {
        println!("--- app.css ---");
        println!("{sheet}");
    }
}
