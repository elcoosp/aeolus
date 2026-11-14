//! Consumer demonstration: the structural (DOM-position) seam at the
//! GPUI walk (plan §3.5's positional-variant row, the `StructProbe`
//! half): `first:`, `last:`, `odd:`, `even:`, `only:`, `empty:`,
//! `nth-N:`, `nth-last-N:` layers answer through the element's typed
//! position facts — the GPUI half of the interp arm's `interp_struct`
//! entry point, the interp's `structural_fires` verdict shared
//! verbatim. The demo resolves into a headless `StyleRefinement` the
//! same way a running app's `div()` would consume the program.
//!
//! ```sh
//! cargo run -p aeolus-gpui --example struct
//! ```

use aeolus_gpui::prelude::*;

fn main() {
    // The list-row surface a settings list would wear: the base row,
    // the `first:`/`last:` rounding layers and the `odd:` stripe.
    let program = tw!("flex items-center gap-2 border-b px-4 py-2 \
         first:rounded-t-md \
         last:rounded-b-md \
         odd:bg-accent/50");

    // 1. The sourceless walk (plain `.tw`): the structural layers stay
    //    quiet — the CSS selector-miss skip the interp reports as
    //    `StructuralUnqueried` notes. The base row applies.
    let base = gpui::StyleRefinement::default().tw(program);
    println!("── sourceless walk (no facts) ──");
    println!("padding-top:  {:?}", base.padding.top);
    println!("background:   {:?}", base.background);

    // 2. The FIRST row of three: the `first:` rounding fires, the
    //    `odd:` stripe fires (row 0 is odd in CSS's 1-based count),
    //    the `last:` rounding stays quiet.
    let first = gpui::StyleRefinement::default().tw_struct(program, &StructFacts::new(0, 3));
    println!("── first of three (index 0, count 3) ──");
    println!("padding-top:  {:?}", first.padding.top);
    println!("background:   {:?}", first.background.is_some());

    // 3. The MIDDLE row: both rounding layers quiet, the stripe quiet
    //    (row 1 is even).
    let middle = gpui::StyleRefinement::default().tw_struct(program, &StructFacts::new(1, 3));
    println!("── middle of three (index 1, count 3) ──");
    println!("padding-top:  {:?}", middle.padding.top);

    // 4. The LAST row: the `last:` rounding fires (the border-b base
    //    and the rounding compose), the stripe fires again (row 2 is
    //    odd).
    let last = gpui::StyleRefinement::default().tw_struct(program, &StructFacts::new(2, 3));
    println!("── last of three (index 2, count 3) ──");
    println!("padding-top:  {:?}", last.padding.top);
    println!("background:   {:?}", last.background.is_some());

    // 5. The honest no-opinion fact (`StructFacts::UNKNOWN`): the host
    //    knows the element is in a list but not where — every
    //    structural layer misses its test (the answered-knows-nothing
    //    verdict the interp shares).
    let unknown = gpui::StyleRefinement::default().tw_struct(program, &StructFacts::UNKNOWN);
    println!("── UNKNOWN (the host has no position) ──");
    println!("padding-top:  {:?}", unknown.padding.top);
}
