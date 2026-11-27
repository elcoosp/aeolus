//! Consumer demonstration: the emit (a) END FORM — the builder-chain
//! emitter (architecture plan §3.2 status: "The backend METHOD calls
//! the emit (a) end form implies (`FlexDirection::Column`, builder
//! chains) remain the P3 codegen crate's surface"). The macro face
//! (`tw_builder!` + `TwExt::tw_chain`) applies the compile-time
//! method calls with NO walk; this example pins the parity headlessly
//! — the chain and the walk land the SAME `StyleRefinement`, field by
//! field — and prints the per-frame lanes the `benches/builder.rs`
//! harness measures.
//!
//! ```sh
//! cargo run -p aeolus-gpui --example builder_chain
//! ```

use aeolus_gpui::prelude::*;

/// The shadcn Button's own class string (the perf_contract
/// harness's representative shape — every declaration
/// chain-expressible under the baked shadcn table).
const BUTTON_CLASSES: &str = "inline-flex items-center justify-center gap-2 p-4 rounded-md \
     bg-primary text-primary-foreground";

fn main() {
    // The frame-Env escape hatch (plan §3.5's debug-assert keys on it).
    set_env(Default::default());

    let walked = gpui::StyleRefinement::default().tw(tw!("inline-flex items-center justify-center gap-2 p-4 rounded-md bg-primary text-primary-foreground"));
    let chained = gpui::StyleRefinement::default().tw_chain(tw_builder!("inline-flex items-center justify-center gap-2 p-4 rounded-md bg-primary text-primary-foreground"));

    // The parity pin, field by field (the perf_contract harness's
    // idiom — the comparison is only fair when both sides do the same
    // work).
    assert_eq!(chained.display, walked.display, "display");
    assert_eq!(chained.align_items, walked.align_items, "align_items");
    assert_eq!(chained.justify_content, walked.justify_content, "justify");
    assert_eq!(chained.gap, walked.gap, "gap");
    assert_eq!(chained.padding, walked.padding, "padding");
    assert_eq!(chained.corner_radii, walked.corner_radii, "radii");
    let (Some(a), Some(b)) = (&chained.background, &walked.background) else {
        panic!("both lanes set a background");
    };
    // The pinned `Fill` carries the one `Color` variant (the walk's own
    // tag — the perf_contract harness's reading), so the destructuring
    // is irrefutable.
    let (gpui::Fill::Color(a), gpui::Fill::Color(b)) = (a, b);
    let (Some(a), Some(b)) = (a.as_solid(), b.as_solid()) else {
        panic!("both lanes carry solid colours");
    };
    assert!(
        u32::from(gpui::Rgba::from(a)) == u32::from(gpui::Rgba::from(b)),
        "the baked colour bytes match"
    );

    println!("chain == walk on every field the string sets ({BUTTON_CLASSES})");
    println!("  display:      {:?}", chained.display);
    println!("  align_items:  {:?}", chained.align_items);
    println!("  justify:      {:?}", chained.justify_content);
    println!("  padding:      {:?}", chained.padding);
    println!("  radii:        {:?}", chained.corner_radii);
    println!("  gap:          {:?}", chained.gap);
    println!("  background:   {:?}", chained.background);
    println!("  text colour:  {:?}", chained.text.color);
    println!();
    println!("the emitted chain is the hand-written builder chain's shape:");
    println!("  tw!       -> frozen StaticProgram + the per-frame walk");
    println!("  tw_builder! -> the compile-time method calls, applied via tw_chain");
}
