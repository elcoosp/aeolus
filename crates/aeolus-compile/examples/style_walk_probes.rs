//! Consumer demonstration: the program lane's full probe seam (the
//! §7 re-target's style fold grown to the interp arm's whole probe
//! surface). A frozen static program — the two slices the `tw!` macro
//! emits — resolves into the v0.x [`ResolvedStyle`] under one probe
//! [`Env`]; the widest entry point
//! (`program_style_probes_containers_struct_descendants_supports`)
//! threads the named-container, structural, descendant and supports
//! seams beside the attribute and token seams, so a host whose layout
//! KNOWS the answers (a measured container, a tracked subtree, a
//! capability query) gets those layers to FIRE instead of the
//! unqueried reports the sourceless entries honestly carry.
//!
//! ```sh
//! cargo run -p aeolus-compile --example style_walk_probes
//! ```

use aeolus_compile::{
    AttrBit, AttrKind, AttrState, Config, ContainerBound, ContainerState, DescendantFact,
    DescendantKind, DescendantState, Env, InterpNote, StructFacts, SupportsState,
    keyword_alignment, keyword_texts, program_style,
    program_style_probes_containers_struct_descendants_supports, program_style_with,
};
use aeolus_theme::{ThemeSource, TokenTable};

/// The baked shadcn light table (T0's shape) — the freeze walk's own
/// construction.
fn baked_table() -> TokenTable {
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    TokenTable::baked(&source).expect("the baked shadcn table builds")
}

/// Compile the demo corpus the way `tw!` does and freeze the two
/// slices the macro emits.
fn frozen(classes: &str) -> (&'static [aeolus_ir::Layer], &'static [&'static str]) {
    let config = Config::default();
    let compiled =
        aeolus_compile::compile_theme(classes, &baked_table(), &config).expect("the class lowers");
    let annotations = keyword_texts(classes, &config).expect("non-empty input annotates");
    let keywords = keyword_alignment(&compiled, &annotations);
    (
        compiled.program.layers,
        Box::leak(keywords.into_boxed_slice()),
    )
}

fn main() {
    // One program, every seam family on it: the named container, the
    // structural head, the descendant shape, the supports layer and
    // the element-own attribute probe.
    let (layers, keywords) = frozen(
        "bg-primary @md/card:w-64 first:w-8 has-focus:opacity-40 \
         supports-[display:grid]:hidden data-[state=open]:ring-2",
    );

    println!("── the sourceless walk: the reported boundary ──");
    let (_, walk) = program_style(layers, keywords, &Env::default());
    for note in &walk.notes {
        println!("  reported: {note}");
    }
    assert!(
        walk.notes
            .iter()
            .any(|n| matches!(n, InterpNote::ContainerUnqueried { .. }))
            && walk
                .notes
                .iter()
                .any(|n| matches!(n, InterpNote::DescendantUnqueried { .. }))
            && walk
                .notes
                .iter()
                .any(|n| matches!(n, InterpNote::SupportsUnqueried { .. }))
            && walk
                .notes
                .iter()
                .any(|n| matches!(n, InterpNote::StructuralUnqueried { .. })),
        "every sourceless probe family reports honestly"
    );

    println!("── the full-probe walk: the seams answer ──");
    let attrs = AttrState::new(&[AttrBit {
        kind: AttrKind::Data,
        name: "state",
        value: "open",
    }]);
    let card = ContainerState::new(&[ContainerBound {
        name: "card",
        rem: 64.0,
    }]);
    let facts = StructFacts::new(0, 2);
    let focused = DescendantState::new(&[DescendantFact {
        kind: DescendantKind::Element,
        fragment: "*:focus",
    }]);
    let grid = SupportsState::new(&["(display:grid)"]);
    let (style, walk) = program_style_probes_containers_struct_descendants_supports(
        layers,
        keywords,
        &Env::default(),
        Some(&attrs),
        Some(&card),
        Some(&facts),
        Some(&focused),
        Some(&grid),
        Some(&baked_table()),
    );
    assert!(walk.notes.is_empty(), "every layer answered: {walk:?}");
    println!(
        "  @md/card:w-64 fires  → width {:?}",
        style.dimensions.map(|d| d.width)
    );
    println!(
        "  has-focus:opacity-40 fires → opacity {:?}",
        style.opacity.map(|o| o.0)
    );
    println!(
        "  bg-primary through the token seam → {:?}",
        style.background.map(|b| b.0.to_srgba())
    );
    println!(
        "  data-[state=open]:ring-2 fires → ring {:?}",
        style.ring.map(|r| r.width)
    );
    assert!(
        style.ring.is_some(),
        "the attribute-probe layer fires through the seam"
    );
    assert_eq!(
        style.dimensions.expect("the container layer fires").width,
        Some(aeolus_core::SizedValue::Px(256.0)),
        "w-64 = 16rem bakes through the env's rem"
    );

    println!("── the answered miss: the quiet skip ──");
    let narrow = ContainerState::new(&[ContainerBound {
        name: "card",
        rem: 32.0,
    }]);
    let (_, walk) = program_style_probes_containers_struct_descendants_supports(
        layers,
        keywords,
        &Env::default(),
        None,
        Some(&narrow),
        None,
        None,
        None,
        None,
    );
    println!("  card at 32rem: the md layer misses, NO note — CSS quiet");
    assert!(
        !walk
            .notes
            .iter()
            .any(|n| matches!(n, InterpNote::ContainerUnqueried { .. })),
        "the ANSWERED miss is quiet: {walk:?}"
    );
    assert!(
        walk.notes
            .iter()
            .any(|n| matches!(n, InterpNote::DescendantUnqueried { .. })),
        "the families the demo wired no source for keep reporting: {walk:?}"
    );

    println!("── the narrow entries agree with the widest form ──");
    let (narrow_style, narrow_walk) =
        program_style_with(layers, keywords, &Env::default(), Some(&attrs), None);
    let (wide_style, wide_walk) = program_style_probes_containers_struct_descendants_supports(
        layers,
        keywords,
        &Env::default(),
        Some(&attrs),
        None,
        None,
        None,
        None,
        None,
    );
    assert_eq!(narrow_style, wide_style, "one decide, two walks");
    assert_eq!(narrow_walk, wide_walk, "the report agrees too");
    println!("  program_style_with ≡ the widest form (seams riding None)");
    println!("ok");
}
