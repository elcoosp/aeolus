//! Consumer demonstration: the component-variant macros (`variants!`,
//! `tw_cn!`, `theme!`) — the closed-world cva/tailwind-variants surface
//! beside the `tw!`/`cn!` pair the `frozen_program` example shows. Every
//! arm compiles at EXPANSION time through the same V1 pipeline (each
//! generated program is the frozen static form, never the input string;
//! an un-expressible class is a compile error with the typed refusal).
//!
//! The three shapes the `variants!` grammar covers, each printing the
//! program its call evaluates to:
//!
//! 1. the group form (`variant:`/`size:` — one enum per group, a
//!    `style(...)` per combo),
//! 2. the slot form (`slots:` — one base per PART, a `style_slot(...)`
//!    dispatch, compounds riding per-slot braces),
//! 3. `tw_cn!` (the compile-time `cn()` merge — later classes win their
//!    conflict group at EXPANSION time), and
//! 4. `theme!` (the host CSS baked into the binary through the same
//!    theme pass the CLI's `--theme` rides).
//!
//! ```sh
//! cargo run -p aeolus-macro --example variants
//! ```

use aeolus_macro::{theme, tw_cn, variants};

// 1. The group form: two groups, one enum each, a `style` per combo.
//    Every class compiles at expansion — a typo here is a compile
//    error, not a runtime surprise.
variants! { pub Button {
    base: "inline-flex items-center justify-center rounded-md text-sm font-medium",
    variant: {
        default:    "bg-primary text-primary-foreground hover:bg-primary/90",
        destructive: "bg-destructive text-white hover:bg-destructive/90",
        outline:    "border border-gray-200 bg-white hover:bg-gray-50",
    },
    size: {
        default: "h-9 px-4 py-2",
        sm:      "h-8 px-3 text-xs",
        lg:      "h-10 px-6",
    },
} }

// 2. The slot form: one base per PART (the card's root/header/title),
//    a variant whose brace names the slots it rides, one compound.
variants! { pub Card {
    base: "text-card-foreground",
    slots: {
        root:   "flex flex-col gap-6 rounded-xl border py-6",
        header: "flex flex-col gap-1.5 px-6",
        title:  "font-semibold leading-none",
    },
    variant: {
        default: "text-sm",
        outline: { root: "border-gray-200 shadow-sm", title: "text-muted-foreground" },
    },
    compound: [ { variant: outline, class: { root: "bg-white" } } ],
} }

fn main() {
    // The group form: the merged program per (variant, size) combo —
    // the DEFAULT keys ride the enums' `#[default]` variants.
    println!("── Button::style (the group form) ──");
    for variant in [ButtonVariant::Default, ButtonVariant::Destructive] {
        let program = Button::style(variant, ButtonSize::Sm);
        println!("{variant:?} × Sm: {} layer(s)", program.layers.len());
        for decl in program.layers[0].decls {
            println!("    {:?}", decl.prop);
        }
    }

    // The slot form: one program PER PART — the dispatch names the slot,
    // the variant's per-slot braces land where they named.
    println!("\n── Card::style_slot (the slot form) ──");
    for slot in [CardSlot::Root, CardSlot::Header, CardSlot::Title] {
        let program = Card::style_slot(slot, CardVariant::Outline);
        println!("{slot:?} × Outline: {} layer(s)", program.layers.len());
    }

    // 3. The compile-time merge: `tw_cn!` folds same-condition layers
    //    by `Cond` equality and keeps the LAST occurrence per longhand —
    //    `px-8` replaces `p-4`'s horizontal halves at expansion time.
    println!("\n── tw_cn! (the compile-time merge) ──");
    let merged = tw_cn!("p-4 px-8 hover:px-6", "px-12");
    println!("layers: {}", merged.layers.len());
    for decl in merged.layers[0].decls {
        let text = merged.keyword(0, 0).map_or_else(
            || format!("{:?}", decl.val),
            |text| format!("{text:?} (row text)"),
        );
        println!("    {:?} = {text}", decl.prop);
    }

    // 4. The host theme: the CSS file bakes into the binary through the
    //    theme pass (the same construction the macro's `tw!` rides for
    //    its token classes; the CLI's `--theme <path>` accepts the same
    //    file).
    println!("\n── theme! (the host CSS baked in) ──");
    let source = theme!("theme.css").expect("the fixture theme parses");
    let tokens = aeolus_rt::TokenTable::baked(&source).expect("the baked table builds");
    println!(
        "tokens baked: {} (the fixture's colours beside the seeded palette)",
        tokens.len()
    );
    for name in ["primary", "destructive"] {
        let Some(id) = tokens.id(name) else {
            panic!("the fixture's {name} token bakes");
        };
        let [r, g, b, a] = tokens.light_rgba(id).expect("the light tier resolves");
        println!("    {name} -> id {id} rgba({r}, {g}, {b}, {a})");
    }
}
