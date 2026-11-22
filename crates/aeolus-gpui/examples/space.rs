//! Consumer demonstration: the GPUI between-child polyfill (the parity
//! wave 1's space/divide row — the `TwSpaceExt::tw_space` wrapper, the
//! audit gate's POLYFILL tier made concrete). The v4 between-child
//! machinery styles the container's OTHER elements — no per-element
//! slot carries that intent — so the runtime composes it the way the
//! CSS arm's `:where(& > :not(:last-child))` rule does: the wrapper's
//! ONE walk takes BOTH outputs (the base styles onto the element, the
//! between-child margins/borders into the state), and `extend` patches
//! every non-last child the v4 rule's own reading. The demo resolves
//! the composed state headless the same way a running app's
//! `div().children(...)` would consume the wrapper.
//!
//! ```sh
//! cargo run -p aeolus-gpui --example space
//! ```

use aeolus_gpui::prelude::*;

fn main() {
    println!("── GPUI between-child polyfill from the V1 program ──");

    // The toolbar row a settings screen would wear: base layout and
    // colour on the element, the between-child spacing/borders on its
    // children (every non-last child, the v4 rule's own reading).
    let toolbar =
        tw!("flex items-center gap-2 px-4 py-2 space-x-4 divide-y divide-primary divide-dashed");
    let wrapper = gpui::div().tw_space(toolbar);
    let state = wrapper.state();

    println!("program layers:      {}", toolbar.layers.len());
    println!(
        "space-x:             {:?}",
        state.space_x.map(|(v, _)| format!("{v:?}"))
    );
    println!(
        "space-y:             {:?}",
        state.space_y.map(|(v, _)| format!("{v:?}"))
    );
    println!(
        "divide-y:            {:?}",
        state.divide_y.map(|(v, _)| format!("{v:?}"))
    );
    println!("divide-style:        {:?}", state.divide_style);
    println!(
        "divide-colour:       {}",
        state
            .divide_color
            .map(|c| format!("{c:?}"))
            .unwrap_or_else(|| "none".into())
    );
    println!(
        "margin (x):          {:?}",
        state.margin(Axis::X).map(|length| format!("{length:?}"))
    );
    println!(
        "border  (y):         {:?}",
        state.border(Axis::Y).map(|width| format!("{width:?}"))
    );

    // The reverse flip: the same margin value, the START side.
    let flipped = tw!("space-x-2 space-x-reverse");
    let wrapper = gpui::div().tw_space(flipped);
    println!("reverse flag:        {}", wrapper.state().space_x_reverse);

    // The plain `tw()` lane composes the state and drops it (the
    // documented boundary: a per-element surface has no children) —
    // the wrapper is the polyfill's consumer, and the audit gate
    // classifies the longhands POLYFILL, so `tw!("space-x-4")` still
    // compiles.
    let plain = tw!("space-x-4");
    let _style = gpui::StyleRefinement::default().tw(plain);
    println!(
        "plain lane:          compiles, between-child parts dropped (the documented boundary)"
    );
    println!("wrapper lane:        the consumer — `.children(...)` patches every non-last child");
}
