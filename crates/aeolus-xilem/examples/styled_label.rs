//! Consumer example: the Xilem/Masonry backend, headlessly — resolve a
//! class string, lower it into a native Masonry `PropertyStack`, and
//! build the styled view with the `.tw()` sugar. No window required: the
//! lowering is a pure function, so this runs anywhere.
//!
//! ```sh
//! cargo run -p aeolus-xilem --example styled_label
//! ```

use aeolus_core::{Resolver, shadcn_default};
use aeolus_xilem::lower;
use aeolus_xilem::view::TwViewExt;
use masonry::properties::{Background, CornerRadius, Padding};
use xilem_masonry::view::label;

fn main() {
    let theme = shadcn_default().expect("the embedded palette parses");
    let resolver = Resolver::new(&theme);
    let classes = "p-4 bg-primary hover:bg-primary/90 rounded-md";

    // The typed IR in, the native cascade out: `lower` returns the Masonry
    // PropertyStack a hand-written widget would carry, plus the honest
    // per-slot report (empty here — every class is expressible).
    let resolution = resolver.resolve(classes).expect("all classes supported");
    let (stack, report) = lower(&resolution.plan);
    assert!(
        report.dropped.is_empty(),
        "no silent drops allowed: {report:?}"
    );

    println!("classes : {classes}");
    println!("→ {} Masonry layer(s):", stack.get_layers().len());
    for (i, (_selector, props)) in stack.get_layers().iter().enumerate() {
        println!(
            "  layer {i}: background={} padding={} corner_radius={}",
            props.get::<Background>().is_some(),
            props.get::<Padding>().is_some(),
            props.get::<CornerRadius>().is_some()
        );
    }
    println!("report  : {} dropped slot(s)", report.dropped.len());

    // The same plan as a real view child — `.tw(&resolver, classes)` parses,
    // resolves and wraps in one step (this is what a Xilem app's view fn
    // calls per rebuild; the label renders styled inside your `div()`s).
    let _styled: aeolus_xilem::view::Tw<_, (), ()> = label("Save changes").tw(&resolver, classes);
    println!(
        "built `label(\"Save changes\").tw(&resolver, classes)` headlessly — drop it into your Xilem view."
    );
}
