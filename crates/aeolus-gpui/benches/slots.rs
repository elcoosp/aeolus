//! The `variants!` slot dispatch's microbenchmark (divan, stable — the
//! workspace bench idiom): the per-call cost of the generated
//! `style_slot()` match over the slot × combo cartesian, with group
//! strings, per-slot braces and compound entries folded into the arms
//! at expansion time (the dispatch is the only runtime cost the table
//! ever had).

use divan::Bencher;

use aeolus_gpui::prelude::*;

variants! { pub Benched {
    slots: {
        root:"flex flex-col gap-6 rounded-xl border py-6",
        header:"flex flex-col gap-1.5 px-6",
        title:"font-semibold leading-none",
    },
    variant: {
        default:"text-card-foreground",
        destructive:"bg-destructive text-white",
        outline: { root:"border-gray-200", title:"text-muted-foreground"},
    },
    size: { default:"text-sm", sm:"text-xs", lg:"text-lg"},
    compound: [
        { variant: outline, size: sm, class: { root:"ring-2 ring-red-500"} },
        { variant: destructive, class:""},
    ],
} }

#[divan::bench]
fn slots_dispatch(bencher: Bencher) {
    // The multi-part surface: the match over 3 slots × 3×3 combos, the
    // per-slot brace values and compound classes resolved at expansion
    // time — `style_slot()` is a pure match dispatch to a frozen
    // program, the `style()` dispatch's own shape.
    use BenchedSize as S;
    use BenchedSlot as P;
    use BenchedVariant as V;
    bencher.bench(|| Benched::style_slot(P::Title, V::Outline, S::Sm));
}

fn main() {
    divan::main();
}
