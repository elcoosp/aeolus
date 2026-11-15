//! The filter tier's STATIC emit half: the macro gate's closed-world
//! pass. The nine polyfillable filter longhands (the seven
//! colour-matrix matrices, the `drop-shadow-*` presets and the bare
//! `filter`/`filter-none` reset arm) emit through `tw!` where they used
//! to refuse — the walk composes them (see `filters.rs` for the
//! headless pins). The spatial (`blur-*`) and `backdrop-*` halves keep
//! the typed refusal: `tw!("blur-md")` still refuses at expansion.

use aeolus_gpui::prelude::*;

#[test]
fn the_matrix_chain_compiles_through_the_macro_gate() {
    // All seven colour-matrix primitives + the reset arm in one string:
    // the emit gate passes the closed-world forms, the frozen program
    // carries the longhands the walk recolours.
    let program = tw!(
        "bg-red-500 brightness-110 contrast-125 grayscale hue-rotate-90 \
         invert saturate-150 sepia-50 filter-none"
    );
    assert!(
        program.layers.iter().any(|layer| layer
            .decls
            .iter()
            .any(|decl| matches!(decl.prop, aeolus_ir::Prop::FilterGrayscale))),
        "the matrix longhands ride the frozen program"
    );
    let style = gpui::StyleRefinement::default().tw(program);
    // `filter-none` resets the chain: the bake paints unfiltered.
    let Some(gpui::Fill::Color(background)) = style.background else {
        panic!("the solid background reaches the refinement");
    };
    assert_eq!(
        background.as_solid(),
        Some(gpui::rgba(0xfb2c_36ff).into()),
        "the reset arm cleared the chain"
    );
}

#[test]
fn the_drop_shadow_presets_compile_through_the_macro_gate() {
    let program = tw!("shadow-lg drop-shadow-md");
    let style = gpui::StyleRefinement::default().tw(program);
    let shadows = style.box_shadow.expect("both tiers compose");
    assert_eq!(shadows.len(), 3, "two preset bands + the drop band");
}
