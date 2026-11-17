//! Consumer demonstration: the host surface's OWN style face — the
//! [`StaticProgram`] type carries the walk entries as methods, so a
//! host threads no slices and no imports beyond the `aeolus_rt`
//! surface it already consumes. The element walk (`style`/`style_with`)
//! resolves the frozen program into the v0.x [`ResolvedStyle`] under
//! one probe [`Env`]; the pseudo-target face (`style_pseudo`/
//! `style_pseudo_theme`) resolves a render target's own cascade — the
//! `interp_pseudo` contract on the program lane. The methods are the
//! free walk entries' delegates (the differential suite pins the
//! answers equal); the demo reads both faces headless.
//!
//! ```sh
//! cargo run -p aeolus-rt --example style_face
//! ```

use aeolus_compile::Config;
use aeolus_rt::{ClassCell, Env, PseudoTarget, ResolvedStyle, StaticProgram};
use aeolus_theme::{ThemeSource, TokenTable};

/// The baked shadcn light table (T0's shape) — the freeze walk's own
/// construction.
fn baked_table() -> TokenTable {
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    TokenTable::baked(&source).expect("the baked shadcn table builds")
}

fn main() {
    // The dynamic lane is the runtime path's constructor: the cell
    // compiles THEMED (colour utilities lower through a theme table
    // in the V1 pipeline) and hands back the frozen program.
    let cell = ClassCell::new();
    let table = baked_table();
    let program: StaticProgram = cell
        .get_theme(
            "bg-primary p-4 rounded-md hover:bg-primary/90 selection:bg-primary/90",
            &table,
            &Config::default(),
        )
        .expect("the cell compiles what lowers");

    println!("── the element walk: the host surface's own face ──");
    let (style, walk) = program.style(&Env::default());
    println!("  background: {:?}", style.background.is_some());
    println!(
        "  padding top: {} px",
        style.padding.as_ref().map(|p| p.top).unwrap_or(0.0)
    );
    println!(
        "  walk: notes={} boundaries={} refusals={}",
        walk.notes.len(),
        walk.boundaries.len(),
        walk.refusals.len()
    );
    let _: ResolvedStyle = style;
    assert_eq!(
        walk.notes.len(),
        1,
        "the selection layer reports on the element walk's side (the render-target contract's other half)"
    );

    println!("── the probe face: the attribute seam answers ──");
    let (style, walk) = program.style_with(&Env::default(), None, Some(&table));
    println!(
        "  themed form over the baked table: background={:?} (baked tier, the source rides unused)",
        style.background.is_some()
    );
    assert_eq!(
        walk.notes.len(),
        1,
        "the selection layer reports here too — the element walk's side"
    );

    println!("── the pseudo-target face: the selection cascade ──");
    let (style, walk) = program.style_pseudo(PseudoTarget::Selection);
    println!("  ::selection background: {:?}", style.background.is_some());
    println!(
        "  walk: notes={} boundaries={} refusals={}",
        walk.notes.len(),
        walk.boundaries.len(),
        walk.refusals.len()
    );
    assert!(style.background.is_some(), "the target's own recolour");
    assert!(walk.is_empty(), "silence, not notes: {walk:?}");

    println!("── the unnamed target: the quiet answer ──");
    let (style, walk) = program.style_pseudo(PseudoTarget::FirstLetter);
    println!("  ::first-letter: {}", style == ResolvedStyle::default());
    assert!(
        walk.is_empty() && style == ResolvedStyle::default(),
        "the quiet path"
    );

    println!("── the delegation parity: the methods ARE the free walks ──");
    let (method, _) = program.style_pseudo(PseudoTarget::Selection);
    let (free, _) =
        aeolus_rt::program_style_pseudo(PseudoTarget::Selection, program.layers, program.keywords);
    println!("  method == free: {}", method == free);
    assert_eq!(method, free, "one walk, two spellings");

    println!("\nthe host surface answered both faces headless");
}
