//! Consumer demonstration: the GPUI grid-placement + flex-basis tier
//! (plan §3.7's grid row, the native subset finished for the slots the
//! pinned GPUI surface carries). A dashboard-shell layout string —
//! grid template, placement lines, a span and the basis family —
//! compiles through the `tw!` gate (which ACCEPTS the placement
//! families now) and applies onto GPUI's own `GridLocation` ranges and
//! `flex_basis` slot. The demo resolves into a headless
//! `StyleRefinement` the same way a running app's `div()` would
//! consume the program.
//!
//! ```sh
//! cargo run -p aeolus-gpui --example placement
//! ```

use aeolus_gpui::prelude::*;

fn main() {
    // The dashboard-shell surface: a 12-column grid whose main pane
    // starts at column 2, ends at column 9 (a line placement pair),
    // spans two rows (the SPAN fingerprint — one class fanning onto
    // the equal (start, end) pair the walk composes into GPUI's
    // `Span` placement) and sizes its main axis with the basis family.
    let program = tw!("grid grid-cols-12 col-start-2 col-end-9 row-span-2 basis-16");
    let style = gpui::StyleRefinement::default().tw(program);

    let location = style
        .grid_location
        .clone()
        .expect("the shell places itself");
    println!("── the dashboard shell ──");
    println!(
        "column:     {:?} .. {:?} (col-start-2 col-end-9)",
        location.column.start, location.column.end
    );
    println!(
        "row:        {:?} .. {:?} (row-span-2 — the equal pair composed)",
        location.row.start, location.row.end
    );
    println!("flex_basis: {:?} (basis-16 = 4rem)", style.flex_basis);

    // The cascade: a later line placement overwrites the span pair's
    // longhand it carries — `col-span-3 col-start-2` reads start
    // Line(2) with the span's end, the per-longhand cascade.
    let overridden = gpui::StyleRefinement::default().tw(tw!("col-span-3 col-start-2"));
    let location = overridden.grid_location.clone().expect("a location");
    println!();
    println!("── the cascade ──");
    println!(
        "column:     {:?} .. {:?} (col-span-3 col-start-2 — the start is the later class's)",
        location.column.start, location.column.end
    );

    // The span family in its own right: GPUI's `Span` placement, not
    // a degenerate start==end line range.
    let spanned = gpui::StyleRefinement::default().tw(tw!("col-span-3"));
    let location = spanned.grid_location.clone().expect("a location");
    println!();
    println!("── the span fingerprint ──");
    println!(
        "column:     {:?} .. {:?} (col-span-3 — Tailwind's `span 3 / span 3`)",
        location.column.start, location.column.end
    );

    // The honest boundary rides along: the spellings the registry rows
    // do not take (`full`, the auto-track forms) still report — on
    // every backend alike, never a silent drop.
    println!();
    println!("── the boundary ──");
    match aeolus_compile::compile("col-span-full", &aeolus_compile::Config::default()) {
        Ok(compiled) if !compiled.diagnostics.is_empty() => println!(
            "col-span-full: {} diagnostic(s) — the reported boundary (the \
             row takes numbers; the full-bleed form is the line pair \
             `col-start-1` + `-col-end-1`)",
            compiled.diagnostics.len()
        ),
        Ok(_) => println!("col-span-full lowers"),
        Err(err) => println!("col-span-full: {err}"),
    }
}
