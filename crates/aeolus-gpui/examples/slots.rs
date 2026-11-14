//! Consumer demonstration: the `variants!` slot form (the §9 open
//! question's second half — the tailwind-variants shape). A `slots:`
//! map declares one base per part (a card's root/header/title) and the
//! macro generates a `Slot` enum plus a `style_slot(slot, …)` dispatch:
//! a group value that is a STRING applies to every slot, a brace
//! `{ slot:"classes"}` applies to the slots it names, and the
//! compound `class:` takes both forms — all resolved at EXPANSION time,
//! so the runtime cost stays the match dispatch the table always had.
//!
//! ```sh
//! cargo run -p aeolus-gpui --example slots
//! ```

use aeolus_gpui::prelude::*;

variants! { pub Card {
    slots: {
        root:"flex flex-col gap-6 rounded-xl border py-6",
        header:"flex flex-col gap-1.5 px-6",
        title:"font-semibold leading-none",
    },
    variant: {
        default:"text-card-foreground",
        outline: { root:"border-gray-200", title:"text-muted-foreground"},
    },
    size: { default:"text-sm", lg:"text-lg"},
} }

fn main() {
    println!("── the slot dispatch: each part is its own program ──");
    for slot in [CardSlot::Root, CardSlot::Header, CardSlot::Title] {
        let program = Card::style_slot(slot, CardVariant::Outline, CardSize::Default);
        let decls: usize = program.layers.iter().map(|layer| layer.decls.len()).sum();
        println!(
            "{slot:?} (outline): {decls} declarations over {} layers",
            program.layers.len()
        );
    }

    println!("── the string-vs-brace group values ──");
    let has = |program: &aeolus_rt::StaticProgram, prop: aeolus_ir::Prop| {
        program
            .layers
            .iter()
            .flat_map(|layer| layer.decls.iter())
            .any(|decl| decl.prop == prop)
    };
    for (slot, variant) in [
        (CardSlot::Root, CardVariant::Default),
        (CardSlot::Root, CardVariant::Outline),
        (CardSlot::Header, CardVariant::Outline),
    ] {
        let program = Card::style_slot(slot, variant, CardSize::Default);
        println!(
            "{slot:?} × {variant:?}: colour = {}, border-colour = {}",
            has(&program, aeolus_ir::Prop::Color),
            has(&program, aeolus_ir::Prop::BorderColor),
        );
    }
}
