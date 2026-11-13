//! The project-build seam end to end (architecture plan §2's approach
//! C / the P6 row's `cargo aeolus build`): a synthetic three-file
//! project scanned, deduped, compiled and rendered as ONE stylesheet —
//! the theme vars block first (`:root`/`.dark`), then the
//! globally-deduped rules in the registry's canonical order, with the
//! honest per-location findings on the side.
//!
//! Run it: `cargo run -p aeolus-compile --example project_build`
//!
//! The files are IN-MEMORY strings (the library seam is I/O-free —
//! the CLI's `aeolus build` command is the disk-walking form of the
//! same walk); the shapes mirror a real consumer project: a GPUI view
//! file, an HTML template naming the SAME Button surface (the dedupe
//! collapses it), and a notes file whose quoted-but-unregistered
//! string rides the typed boundary path.

use aeolus_compile::{Config, RustStrings, SourceFile, build, scan};
use aeolus_theme::{ThemeSource, TokenTable};

fn main() {
    // The project: three files. The Button surface appears TWICE (the
    // dedupe collapses it to one rule set); the prose file shows the
    // typed boundary riding through with its location.
    const VIEW_RS: &str = "\
use aeolus_gpui::prelude::*;

fn card() -> Div {
    div().tw(tw!(\"flex flex-col gap-2 rounded-md bg-primary text-primary-foreground\"))
}

fn badge() -> Div {
    div().tw(tw!(\"inline-flex items-center rounded-full bg-secondary px-2 py-0.5\"))
        .tw(tw!(\"md:p-6 hover:bg-primary/90\"))
}
";
    const TEMPLATE_HTML: &str = "\
<button class=\"flex flex-col gap-2 rounded-md bg-primary text-primary-foreground\">
  Save
</button>
";
    const NOTES_MD: &str = "\
# Notes

The design tokens live in themes/app.css. A quoted-but-unregistered
string scans like any other candidate and reports honestly:

> `tw!(\"brand-gradient\")` — no such registry root, a typed finding.
";

    let files = [
        SourceFile::new("src/view.rs", VIEW_RS),
        SourceFile::new("assets/template.html", TEMPLATE_HTML),
        SourceFile::new("NOTES.md", NOTES_MD),
    ];

    // Stage 1 — scan: every candidate with its file/line/col location.
    let scanned = scan(&files, &RustStrings);
    println!("== scan: {} candidate hit(s)", scanned.len());
    for hit in &scanned.hits {
        println!("  {}:{}:{}  {:?}", hit.file, hit.line, hit.col, hit.text);
    }

    // Stage 2 — build with the theme pass supplied (the same CSS a web
    // Tailwind v4 project uses, shadcn light+dark): the artifact is
    // self-contained — the vars block the `var(--color-…)` references
    // resolve against, then the deduped rules in registry order.
    let source = ThemeSource::shadcn_dark().expect("the embedded shadcn source parses");
    let tokens = TokenTable::runtime(&source).expect("the pair surface fits the u16 slot");
    let artifact = build(&scanned, Some(&tokens), &Config::default());

    println!(
        "\n== build: {} rule(s), {} var(s), {} finding(s)\n",
        artifact.rules.len(),
        artifact.vars.len(),
        artifact.diagnostics.len()
    );
    print!("{artifact}");

    // Stage 3 — the honest findings: the prose candidate's typed
    // reports, per location, with the stable codes a linter keys on.
    if artifact.diagnostics.is_empty() {
        println!("-- no findings: every candidate lowered");
    } else {
        println!("-- findings:");
        for report in &artifact.diagnostics {
            println!("  {report}");
        }
    }
}
