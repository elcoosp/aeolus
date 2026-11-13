//! The project-build seam's microbenchmarks (divan, stable — the
//! workspace bench idiom): the extraction walk, the global
//! dedupe+emit artifact over a synthetic multi-file project (the
//! shadcn Button surface repeated across files), and the dedupe's
//! saving against the naive per-occurrence emit.
//!
//! The bench bodies `expect` on the walk: a scan of real class strings
//! always produces an artifact (the scan is total on any text).
//!
//! Numbers land in `reports/08-benchmarks/BENCHMARKS.md` when the gate
//! environment runs them.

use aeolus_compile::{Config, RustStrings, SourceFile, build, scan};
use aeolus_theme::{ThemeSource, TokenTable};
use divan::Bencher;

fn main() {
    divan::main();
}

/// The shadcn v4 Button table verbatim (base + default variant + default
/// size) — the same 33-class string the compile benches key on.
const BUTTON_CLASSES: &str = "inline-flex shrink-0 items-center justify-center gap-2 rounded-md text-sm font-medium whitespace-nowrap transition-all outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50 aria-invalid:border-destructive aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4 bg-primary text-primary-foreground hover:bg-primary/90 h-9 px-4 py-2 has-[>svg]:px-3";

/// A synthetic project: 8 "files", each naming the Button surface plus
/// a per-file utility — the repeated-string shape the global dedupe
/// exists for.
fn project_files() -> Vec<SourceFile<'static>> {
    (0..8)
        .map(|i| {
            let text = Box::leak(
                format!("fn card{i}() {{ div().tw(tw!(\"{BUTTON_CLASSES} p-{i}\")) }}\n")
                    .into_boxed_str(),
            );
            SourceFile::new(Box::leak(format!("src/card{i}.rs").into_boxed_str()), text)
        })
        .collect()
}

/// The runtime table over the shadcn light+dark palette (T2) — the
/// themed build's vars-block path.
fn runtime_tokens() -> TokenTable {
    let source = ThemeSource::shadcn_dark().expect("the embedded shadcn source parses");
    TokenTable::runtime(&source).expect("the pair surface fits the u16 slot")
}

/// The extraction walk alone: 8 files, ~9 literals each — the pre-
/// filter and the location resolution.
#[divan::bench]
fn scan_project_cold(bencher: Bencher) {
    let files = project_files();
    bencher.bench(|| scan(&files, &RustStrings));
}

/// The themed build end to end: scan → dedupe (the 8 Button strings
/// collapse to ONE candidate each + 8 per-file utilities) → per-unique
/// emit → registry order → vars block.
#[divan::bench]
fn build_project_themed_cold(bencher: Bencher) {
    let files = project_files();
    let tokens = runtime_tokens();
    bencher.bench(|| {
        let scanned = scan(&files, &RustStrings);
        build(&scanned, Some(&tokens), &Config::default())
    });
}

/// The themeless floor of the same project (no vars block, the colour
/// tokens as typed boundaries per location).
#[divan::bench]
fn build_project_themeless_cold(bencher: Bencher) {
    let files = project_files();
    bencher.bench(|| {
        let scanned = scan(&files, &RustStrings);
        build(&scanned, None, &Config::default())
    });
}
