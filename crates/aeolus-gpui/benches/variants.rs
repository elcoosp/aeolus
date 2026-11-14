//! The `variants!` compound-variant dispatch's microbenchmark (divan,
//! stable — the workspace bench idiom): the per-call cost of the
//! generated `style()` match over the cartesian combos, with compound
//! entries folded into the arms at expansion time (the dispatch is the
//! only runtime cost the table ever had).

use divan::Bencher;

use aeolus_gpui::prelude::*;

variants! { pub Benched {
    base:"inline-flex items-center gap-2 rounded-md font-medium",
    variant: {
        default:"bg-primary text-primary-foreground",
        destructive:"bg-destructive text-white",
        outline:"border border-gray-200 text-foreground",
    },
    size: { default:"h-9 px-4 py-2", sm:"h-8 px-3", lg:"h-10 px-6"},
    compound: [
        { variant: outline, size: sm, class:"ring-2 ring-red-500"},
        { variant: destructive, class:""},
    ],
} }

#[divan::bench]
fn variants_dispatch(bencher: Bencher) {
    // The shadcn-shaped surface: the match over 3×3 combos, two of them
    // carrying compound classes — `style()` is a pure match dispatch to
    // a frozen program.
    use BenchedSize as S;
    use BenchedVariant as V;
    bencher.bench(|| Benched::style(V::Outline, S::Sm));
}

fn main() {
    divan::main();
}
