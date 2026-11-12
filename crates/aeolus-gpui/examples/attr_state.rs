//! Consumer demonstration: the attribute-bits host state seam (plan
//! §3.5's "typed state bits passed WITH the element", P4's last named
//! piece). A GPUI element's `data-*`/`aria-*` state is not a string
//! probe — the host passes its typed attribute bits and the
//! `data-*:`/`aria-*:` layers answer per CSS selector semantics. The
//! demo resolves into a headless `StyleRefinement` the same way a
//! running app's `div()` would consume the program.
//!
//! ```sh
//! cargo run -p aeolus-gpui --example attr_state
//! ```

use aeolus_gpui::prelude::*;

fn main() {
    // The form-row surface a settings dialog would wear: the base card,
    // the `aria-busy` save-in-progress layer and the `data-[state=open]`
    // separation layer.
    let program = tw!("flex items-center gap-2 rounded-md p-4 \
         aria-busy:opacity-50 aria-busy:pointer-events-none \
         data-[state=open]:mt-2");

    // The element's attribute bits — what a real host reads off the
    // element each frame (here: the form is saving, the row is open).
    let rows = [
        AttrBit {
            kind: AttrKind::Aria,
            name: "busy",
            value: "true",
        },
        AttrBit {
            kind: AttrKind::Data,
            name: "state",
            value: "open",
        },
    ];
    let attrs = AttrState::new(&rows);

    // 1. The sourceless walk (plain `.tw`): the attribute layers stay
    //    quiet — the CSS selector-miss skip the interp reports as
    //    `AttributeUnqueried` notes. The base card applies.
    let base = gpui::StyleRefinement::default().tw(program);
    println!("── sourceless walk (no bits) ──");
    println!("  padding:      {:?}", base.padding.top);
    println!("  opacity:      {:?}", base.opacity);

    // 2. The probe walk (`.tw_probes`): the bits answer, the
    //    attribute layers fire — one probe lookup per probed layer, no
    //    allocation, the plan §3.5 "1 mask test" cost.
    let busy_open = gpui::StyleRefinement::default().tw_probes(program, &attrs);
    println!("── probe walk (aria-busy=true, data-state=open) ──");
    println!("  padding:      {:?}", busy_open.padding.top);
    println!("  opacity:      {:?}", busy_open.opacity);

    // 3. A different element in the same list: the bits are per
    //    element, the program is shared. The closed row (no
    //    `data-state` bit) keeps the disclosure layer quiet.
    let closed = [AttrBit {
        kind: AttrKind::Aria,
        name: "busy",
        value: "true",
    }];
    let closed_attrs = AttrState::new(&closed);
    let closed_style = gpui::StyleRefinement::default().tw_probes(program, &closed_attrs);
    println!("── probe walk (aria-busy=true, no data-state) ──");
    println!("  opacity:      {:?}", closed_style.opacity);
}
