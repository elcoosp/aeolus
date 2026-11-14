//! Consumer demonstration: the GPUI motion half (plan §3.7's
//! transition/animate row, the `with_animation` boundary). The frozen
//! static program cannot carry an element id or an animator closure —
//! the two inputs GPUI's `with_animation` needs — so the HOST names
//! the element and supplies the visual mapping, while the lane builds
//! the `gpui::Animation` from the frozen v4 shorthand (`spin 1s linear
//! infinite`). The demo resolves into headless refinements the same
//! way a running app's `div()` would consume the program.
//!
//! ```sh
//! cargo run -p aeolus-gpui --example motion
//! ```

use aeolus_compile::Config;
use aeolus_gpui::prelude::*;
use aeolus_gpui::{animation_for, parse_animation, pulse_animator};
use aeolus_rt::ClassCell;

fn main() {
    // The frozen shorthands the pipeline lowers — the same texts the
    // CSS arm renders and the interp arm resolves. The lane parses
    // them into the `gpui::Animation` fields (duration, easing curve,
    // iteration).
    println!("── the frozen shorthands, parsed ──");
    for text in [
        "spin 1s linear infinite",
        "ping 1s cubic-bezier(0, 0, 0.2, 1) infinite",
        "pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite",
        "bounce 1s infinite",
    ] {
        let spec = parse_animation(text).expect("the preset parses");
        println!(
            "  {text}\n    name={} duration={}ms easing={:?} infinite={}",
            spec.name,
            spec.duration.as_millis(),
            spec.easing,
            spec.infinite
        );
    }

    // The built animation: the parse plus one `Rc` easing allocation —
    // what `tw_anim` hands to GPUI's `with_animation`.
    let spin = animation_for("spin 1s linear infinite").expect("the preset builds");
    println!("── the built animation ──");
    println!(
        "  spin: duration={}ms looping={}",
        spin.duration.as_millis(),
        !spin.oneshot
    );

    // The stateful lane, dynamic entrance: the cell compiles the
    // motion string at runtime (the tw! macro's gate keeps refusing
    // motion classes — the static emit's honest refusal), the host
    // names the element and supplies the animator. A spinner's rotate
    // is element-specific (no `Styled` transform slot), so the demo's
    // animator maps the eased delta onto opacity — `pulse_animator` is
    // the built-in preset for exactly that shape.
    let cell = ClassCell::new();
    let program = cell
        .get("animate-pulse", &Config::default())
        .expect("the cell compiles the motion string");
    let element =
        gpui::div().tw_anim_env(program, Env::default(), "save-indicator", pulse_animator);
    let _ = element;
    println!("── the lane ──");
    println!("  div().tw_anim_env(pulse program, \"save-indicator\", pulse_animator) built");

    // `animate-none` (or any animation-free program) renders the
    // element statically — the zero-animation wrap, the host's
    // animator never called.
    let still = cell
        .get("animate-none p-4", &Config::default())
        .expect("the cell compiles");
    let element = gpui::div().tw_anim_env(still, Env::default(), "card", pulse_animator);
    let _ = element;
    println!("  animate-none wraps the element statically (no scheduling)");

    // The built-in animator itself: opacity 1 → 0.5 over the eased
    // delta, the v4 pulse keyframes' own range.
    let mid = pulse_animator(gpui::StyleRefinement::default(), 0.5);
    println!("  pulse at eased 0.5: opacity={:?}", mid.opacity);
}
