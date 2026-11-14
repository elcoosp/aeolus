//! The element-bounds CONTAINER WRAPPER (architecture plan §3.5's
//! `@container` row, the measuring half): [`TwContainerExt::tw_container`]
//! publishes the width of the box it assigns its child into the shared
//! [`MeasuredContainers`] hand-off at the layout phase, and
//! [`TwExt::tw_containers`] reads the same hand-off through the ONE
//! [`ContainerProbe`] question — the wrapper never styles, the walk
//! never measures.
//!
//! This demo is the headless half of that story: it publishes the
//! widths a wrapper would measure (the layout phase's numbers), walks
//! the program through the hand-off, prints the applied refinement and
//! the resize beat (the publish/refresh handshake), and diffs the
//! answers against the interp arm's `interp_containers` — the same
//! verdicts through the same seam.
//!
//! ```sh
//! cargo run -p aeolus-gpui --example container_query
//! ```

use aeolus_compile::{Env, interp_containers};
use aeolus_gpui::prelude::*;
use gpui::px;

fn main() {
    // One hand-off per view: the wrapper's clone publishes into it at
    // the layout phase, the styled descendants' clones read from it.
    let containers = MeasuredContainers::new();

    // The card program: base + two named container-query layers (the
    // `card` container a panel layout would declare).
    let program = tw!("flex flex-col gap-2 rounded-md border p-4 @md/card:p-6 @max-lg/card:p-2");
    println!("program: {} layers", program.layers.len());

    // ── the wrapper's measurement beat ─────────────────────────────
    // What `tw_container("card", &containers)` publishes at its
    // prepaint: the width of the box it assigns its child, divided by
    // the window's rem size (here: 768px at the 16px default).
    let first = container_rem(px(768.0), px(16.0)).expect("the window's rem size is defined");
    let changed = containers.publish("card", first);
    println!(
        "\nwrapper prepaint: card = {first}rem (publish changed: {changed} — the re-render asks for the first styled frame)"
    );

    // The styled descendant reads the hand-off: the AT-boundary min
    // form fires.
    let applied = gpui::StyleRefinement::default().tw_containers(program, &containers);
    println!(
        "\n@48rem (AT the md boundary):\n  padding-top = {:?}\n  gap-width = {:?}",
        applied.padding.top.map(length_text),
        applied.gap.width.map(length_text),
    );

    // ── the resize beat ────────────────────────────────────────────
    // The window resized: the wrapper's next prepaint publishes a NEW
    // width — the handshake reports the change (the re-render the
    // resize asks for), and the same walk now answers differently.
    let resized = container_rem(px(512.0), px(16.0)).expect("the window's rem size is defined");
    let changed = containers.publish("card", resized);
    println!(
        "\nwrapper prepaint after resize: card = {resized}rem (publish changed: {changed} — exactly one re-render asked for)"
    );
    let applied = gpui::StyleRefinement::default().tw_containers(program, &containers);
    println!(
        "@{resized}rem (strictly below md):\n  padding-top = {:?}",
        applied.padding.top.map(length_text),
    );

    // ── the converged beat ─────────────────────────────────────────
    // The same width published again is not a change: no re-render,
    // the loop converges — a stable layout costs a lookup and a
    // compare per frame.
    let changed = containers.publish("card", resized);
    println!(
        "\nwrapper prepaint, stable: card = {resized}rem (publish changed: {changed} — no re-render)"
    );

    // ── the differential arm ───────────────────────────────────────
    // The measured answers drive the interp EXACTLY like the built-in
    // static table: same program, same probe semantics, one seam.
    let rows = [ContainerBound {
        name: "card",
        rem: resized,
    }];
    let built_in = ContainerState::new(&rows);
    let from_handle = interp_containers(&program.program(), &Env::default(), &containers);
    let from_table = interp_containers(&program.program(), &Env::default(), &built_in);
    println!(
        "\ninterp differential (measured handle vs static table): {} vs {} declarations — {}",
        from_handle.as_ref().map_or(0, |i| i.decls.len()),
        from_table.as_ref().map_or(0, |i| i.decls.len()),
        if from_handle == from_table {
            "SAME verdicts"
        } else {
            "DIVERGED"
        },
    );

    // The unpublished name: the CSS-quiet miss the walk keeps.
    println!(
        "\nunpublished name `sidebar`: {:?} — the selector that does not match, never an exclusion",
        containers.measured("sidebar"),
    );
}

fn length_text(length: gpui::DefiniteLength) -> String {
    match length {
        gpui::DefiniteLength::Absolute(gpui::AbsoluteLength::Rems(rem)) => {
            format!("{}rem", rem.0)
        }
        gpui::DefiniteLength::Absolute(gpui::AbsoluteLength::Pixels(p)) => {
            format!("{}px", p.as_f32())
        }
        gpui::DefiniteLength::Fraction(f) => format!("{}%", f * 100.0),
    }
}
