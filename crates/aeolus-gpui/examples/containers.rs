//! Consumer demonstration: the named-container bounds seam (plan
//! §3.5's `@container` row, the `ContainerProbe` half). A named
//! container query (`@md/card:`) tests a NAMED ancestor container's
//! width, not the viewport — the host passes the named containers'
//! widths it measured in its layout, and the layers answer per the
//! same AT/strictly-below boundary rules the viewport breakpoints
//! obey. The demo resolves into a headless `StyleRefinement` the same
//! way a running app's `div()` would consume the program.
//!
//! ```sh
//! cargo run -p aeolus-gpui --example containers
//! ```

use aeolus_gpui::prelude::*;
use aeolus_rt::interp_containers;

fn main() {
    // The dashboard-card surface: the base card, the `@md/card` min
    // form (roomier padding once the card container is at least `md`)
    // and the `@max-lg/card` max form (tighter padding while it is
    // still below `lg`).
    let program = tw!("flex flex-col gap-2 rounded-md border p-4 \
         @md/card:p-6 \
         @max-lg/card:p-2");

    // The named containers' widths — what a real host reads off its
    // layout each frame (here: one `card` container, measured wide).
    let bounds = [ContainerBound {
        name: "card",
        rem: 48.0,
    }];
    let card = ContainerState::new(&bounds);

    // 1. The sourceless walk (plain `.tw`): the named layers stay
    //    quiet — the CSS selector-miss skip the interp reports as
    //    `ContainerUnqueried` notes. The base card applies.
    let base = gpui::StyleRefinement::default().tw(program);
    println!("── sourceless walk (no bounds) ──");
    println!("padding:      {:?}", base.padding.top);

    // 2. The bounds walk (`.tw_containers`): the `card` container is
    //    at `md` (48rem) — the min form fires AT the boundary AND the
    //    max form fires (48 is still below `lg`), so the CASCADE
    //    decides: both longhand the same prop, the later layer wins,
    //    the tight padding applies.
    let at_md = gpui::StyleRefinement::default().tw_containers(program, &card);
    println!("── bounds walk (card = 48rem: at md, below lg) ──");
    println!("padding:      {:?}", at_md.padding.top);

    // 3. A wider card: above `lg` the max form misses and the min form
    //    (at md and above) keeps the roomy padding.
    let wide = [ContainerBound {
        name: "card",
        rem: 96.0,
    }];
    let big = ContainerState::new(&wide);
    let above = gpui::StyleRefinement::default().tw_containers(program, &big);
    println!("── bounds walk (card = 96rem: above md, above lg) ──");
    println!("padding:      {:?}", above.padding.top);

    // 4. A narrower card: below `md` the min form misses, the max form
    //    fires strictly below `lg` — the tight padding wins.
    let narrow = [ContainerBound {
        name: "card",
        rem: 32.0,
    }];
    let small = ContainerState::new(&narrow);
    let below = gpui::StyleRefinement::default().tw_containers(program, &small);
    println!("── bounds walk (card = 32rem: below md, below lg) ──");
    println!("padding:      {:?}", below.padding.top);

    // 5. A different panel in the same layout: the bounds are per
    //    container NAME, the program is shared. An unknown name (the
    //    host measured no such container) is the CSS-quiet miss.
    let unknown = [ContainerBound {
        name: "aside",
        rem: 96.0,
    }];
    let other = ContainerState::new(&unknown);
    let quiet = gpui::StyleRefinement::default().tw_containers(program, &other);
    println!("── bounds walk (no `card` container known) ──");
    println!("padding:      {:?}", quiet.padding.top);

    // The differential half: the interp arm answers the same question
    // from the same program + env + bounds — one semantics, three
    // walks (the interp's `interp_containers` entry point).
    let interped = interp_containers(&program.program(), &Env::default(), &card)
        .expect("the interp answers the same question");
    println!("── interp arm (same bounds) ──");
    println!(
        "resolved decls: {} (every fired layer's winners, container layers included)",
        interped.decls.len()
    );
}
