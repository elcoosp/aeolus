//! Consumer demonstration: the named-group lane (architecture plan
//! §3.5's group/peer composition tier, the named label form's GPUI
//! half). The IR's named-group slot carries the label
//! (`group-hover/menu:` folds to `Cond::GroupHover { name: Some("menu") }`
//! — two different groups are two different conditions), and the state
//! walk wires GPUI's OWN group dispatch: the native
//! `.group_hover(label, …)` / `.group_active(label, …)` closures keyed
//! on the label the host's `.group(label)` ancestor carries — the
//! label dispatch the flat boolean fold could not express.
//!
//! The honest boundaries, verified at the `gpui-pre` 0.3.12 pin: the
//! pinned `Interactivity` carries ONE group-hover/group-active style
//! slot per element (the FIRST label in layer order wins it) and no
//! group-focus/checked/visited/disabled slots at all — those named
//! families keep the env walk's quiet-miss boundary (the CSS/interp
//! arms render the named head the real selector carries).
//!
//! ```sh
//! cargo run -p aeolus-gpui --example group_states
//! ```

use aeolus_gpui::prelude::*;

fn main() {
    aeolus_gpui::set_env(Default::default()); // the frame-Env escape hatch (plan §3.5's debug-assert keys on it)
    // The named condition IS which group the question tests — the IR
    // slot rides the label (the named-container fold's own rule).
    let named = tw!("group-hover/menu:bg-red-500");
    println!("── the named-group IR slot ──");
    println!("cond: {:?}", named.layers[0].cond);

    // The unnamed form keeps the boolean fold: the env's flat
    // `.group` fact answers it (byte-identical to the wave-1 walk).
    let bare = tw!("group-hover:bg-red-500");
    let style = gpui::StyleRefinement::default().tw_env(
        bare,
        Env {
            group_hover: true,
            ..Env::default()
        },
    );
    println!("── the bare form answers the env's flat fact ──");
    println!(
        "hover-applied background set: {}",
        style.background.is_some()
    );

    // The named form stays quiet in the env walk (the unnamed boolean
    // cannot answer a named question) — the native `.group_hover(label,
    // …)` closure is the seam, wired by `tw_state` on the element that
    // carries the named layers. In a running app:
    //
    // ```ignore
    // div().group("menu")                    // the ancestor's label
    //     .child(
    //         div().id("item").tw_state(tw!("group-hover/menu:bg-red-500"))
    //     )
    // ```
    // — gpui's own group dispatch fires the closure when the labelled
    // ancestor is hovered, exactly the CSS `.group\/menu:hover &`
    // reading the CSS arm renders.
    let named = tw!("group-hover/menu:bg-red-500");
    let style = gpui::StyleRefinement::default().tw_env(
        named,
        Env {
            group_hover: true,
            ..Env::default()
        },
    );
    println!("── the named form stays quiet in the env walk ──");
    println!(
        "hover-applied background set: {}",
        style.background.is_some()
    );

    // The audit gate passes the named programs (per-decl verdicts — the
    // condition changes nothing about the declarations).
    let active = tw!("group-active/nav:underline from-blue-500");
    println!("── the audit gate passes the named programs ──");
    println!("group-active/nav program layers: {}", active.layers.len());
}
