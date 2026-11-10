//! Consumer demonstration: the Floem backend's lowering — a
//! `StylePlan` into Floem's own `Style` (peniko brushes, taffy
//! lengths), the values a Floem widget's `Style::apply_selectors`
//! cascade reads. Headless by construction: the lowering is pure.
//!
//! ```sh
//! cargo run -p aeolus-floem --example lower_style
//! ```

use floem::style::{Background, PaddingLeft, TextColor};

use aeolus_core::{Resolver, shadcn_default};
use aeolus_floem::lower;

fn main() {
    let theme = shadcn_default().expect("the embedded shadcn default theme parses");
    let resolver = Resolver::new(Box::leak(Box::new(theme)));

    // The card surface a Floem container would wear. The plan's hover
    // layer rides `StyleSelector` state in the app's own cascade.
    let resolution = resolver
        .resolve("p-4 bg-card text-card-foreground rounded-md border")
        .expect("parseable class string");
    let (style, report) = lower(&resolution.plan);

    println!("── floem Style (base layer) ──");
    println!("  padding-left (p-4):  {:?}", style.get(PaddingLeft));
    println!("  text-color:          {:?}", style.get(TextColor));
    println!("  background (brush):  {:?}", style.get(Background));
    println!(
        "  report: {} drops (every slot egui-style Floem cannot express lands here, never a silent no-op)",
        report.dropped.len()
    );
}
