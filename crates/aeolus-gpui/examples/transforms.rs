//! Consumer demonstration: the GPUI transform lane (architecture plan
//! §3.7's transform row, the motion lane's own shape). The pinned
//! `Styled` surface carries NO transform slot (verified at the
//! `gpui-pre` 0.3.12 pin — no `Styled` transform methods, no
//! `StyleRefinement` transform field, the only transformed paint in the
//! crate the svg element's `with_transformation`), so the seven 2D
//! longhands the registry lowers into accumulate into a
//! [`TransformState`] and compose into GPUI's OWN
//! [`gpui::TransformationMatrix`] — the host applies it on the
//! transformed surfaces its element stack carries (the svg's
//! `with_transformation`, a `with_animation` animator, a transformed
//! paint primitive).
//!
//! ```sh
//! cargo run -p aeolus-gpui --example transforms
//! ```

use aeolus_compile::Config;
use aeolus_gpui::prelude::*;
use aeolus_rt::ClassCell;

fn main() {
    aeolus_gpui::set_env(Default::default()); // the frame-Env escape hatch (plan §3.5's debug-assert keys on it)
    // The static lane keeps the typed refusal (no per-element slot to
    // freeze the transform into — the emit gate's verified reason), so
    // the lane's programs come from the dynamic lane's cell (the
    // motion lane's own shape: the same compiler, no capability gate).
    let cell = ClassCell::new();
    let program = cell
        .get("rotate-45 scale-75", &Config::default())
        .expect("compiles");
    let state = transform_state(&program, &aeolus_gpui::env());
    println!("── the transform lane's frozen read ──");
    println!("state: {state:?}");

    // The composed 2D matrix: scale (the skew folded into the 2×2),
    // then rotate, then translate outermost — the CSS
    // individual-transform order — in GPUI's own matrix type.
    let matrix = state.transformation(16.0).expect("a transform composes");
    println!("── the composed 2D matrix (rem 16) ──");
    println!("rotation_scale: {:?}", matrix.rotation_scale);
    println!("translation:    {:?}", matrix.translation);

    // The spacing translates resolve against the host's rem: the
    // translate-x-4 matrix at rem 16 moves 64px right.
    let shifted = cell
        .get("translate-x-4", &Config::default())
        .expect("compiles");
    let matrix = transform_state(&shifted, &aeolus_gpui::env())
        .transformation(16.0)
        .expect("a translate composes");
    println!("── translate-x-4 at rem 16 ──");
    println!("translation: {:?}", matrix.translation);

    // The honest boundary: an empty state composes nothing (never a
    // unit matrix pretending to be a transform).
    let empty = TransformState::default();
    println!("── the honest boundaries ──");
    println!(
        "default state composes nothing: {}",
        empty.transformation(16.0).is_none()
    );
}
