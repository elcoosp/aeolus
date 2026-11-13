//! Consumer demonstration: the `variants!` compound variants (the §9
//! open question's decision — the cva shape). A `compound:` LIST applies
//! extra classes to every combo that matches ALL of an entry's
//! constraints, resolved at EXPANSION time: the entries fold into the
//! cartesian match arms the macro already renders, so the runtime cost
//! stays the match dispatch the table always had.
//!
//! ```sh
//! cargo run -p aeolus-gpui --example variants
//! ```

use aeolus_gpui::prelude::*;

variants! { pub Badge {
    base: "inline-flex items-center rounded-md px-2 py-1 text-xs font-medium",
    variant: {
        default: "bg-primary text-primary-foreground",
        outline: "border border-gray-200 text-foreground",
    },
    size: { default: "h-9 px-4", sm: "h-8 px-3" },
    compound: [
        { variant: outline, size: sm, class: "ring-2 ring-red-500" },
    ],
} }

fn main() {
    println!("── the compound entry rides only (outline, sm) ──");
    for (variant, size) in [
        (BadgeVariant::Outline, BadgeSize::Sm),
        (BadgeVariant::Outline, BadgeSize::Default),
        (BadgeVariant::Default, BadgeSize::Sm),
    ] {
        let program = Badge::style(variant, size);
        let ring = program
            .layers
            .iter()
            .flat_map(|layer| layer.decls.iter())
            .any(|decl| decl.prop == aeolus_ir::Prop::RingWidth);
        println!(
            "  {variant:?} × {size:?}: ring rides = {ring} ({} layers)",
            program.layers.len()
        );
    }
}
