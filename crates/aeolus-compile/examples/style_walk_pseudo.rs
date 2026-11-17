//! Consumer demonstration: the program lane's pseudo-target face (the
//! `interp_pseudo` contract grown onto the frozen static program). A
//! frozen program — the two slices the `tw!` macro emits — resolves a
//! [`PseudoTarget`]'s OWN cascade into the v0.x [`ResolvedStyle`]:
//! only the layers carrying the target's condition participate (the
//! flat fold collapses stacked variants onto the pseudo condition),
//! the element's own layers and the sibling pseudo targets stay
//! SILENT on this side — the element walk's
//! [`InterpNote::PseudoElement`] note is the other half of the same
//! render-target contract, reported on ITS side. The themeless form
//! refuses the runtime-tier theme forms per longhand into
//! [`StyleWalk::refusals`]; the themed form resolves them through one
//! [`TokenSource`].
//!
//! ```sh
//! cargo run -p aeolus-compile --example style_walk_pseudo
//! ```

use aeolus_compile::{
    Config, Env, InterpNote, PseudoTarget, keyword_alignment, keyword_texts, program_style,
    program_style_pseudo, program_style_pseudo_theme,
};
use aeolus_theme::{ThemeSource, TokenTable};

/// The baked shadcn light table (T0's shape) — the freeze walk's own
/// construction.
fn baked_table() -> TokenTable {
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    TokenTable::baked(&source).expect("the baked shadcn table builds")
}

/// The runtime-tier table (T2): the colour tokens stay `Tok` slots the
/// themed walk resolves through the source — and the themeless walk
/// refuses into its report.
fn runtime_table() -> TokenTable {
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    TokenTable::runtime(&source).expect("the runtime table builds")
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
    // One program: the element's own cascade beside THREE render
    // targets' layers (the selection recolour, the placeholder text,
    // the marker).
    let (layers, keywords) = frozen(
        "bg-primary p-4 rounded-md hover:bg-primary/90 \
         selection:bg-primary/90 placeholder:text-red-500 marker:text-primary-foreground",
    );

    println!("── the element walk: the pseudo layers are the EXCLUDED targets ──");
    let (element, walk) = program_style(layers, keywords, &Env::default());
    println!(
        "  element background: {:?} (the base rule)",
        element.background.is_some()
    );
    for note in &walk.notes {
        println!("  reported: {note}");
    }
    assert_eq!(
        walk.notes
            .iter()
            .filter(|n| matches!(n, InterpNote::PseudoElement { .. }))
            .count(),
        3,
        "the three targets' layers report on the element walk's side"
    );

    println!("── the selection target's own cascade ──");
    let (style, walk) = program_style_pseudo(PseudoTarget::Selection, layers, keywords);
    println!("  ::selection background: {:?}", style.background);
    println!(
        "  report: notes={} boundaries={} refusals={}",
        walk.notes.len(),
        walk.boundaries.len(),
        walk.refusals.len()
    );
    assert!(
        walk.is_empty(),
        "a clean target walk reports nothing: {walk:?}"
    );
    assert!(
        style.background.is_some() && element.background != style.background,
        "the target's recolour is its OWN answer, not the element's"
    );

    println!("── the placeholder target: the sibling layers stay silent ──");
    let (style, walk) = program_style_pseudo(PseudoTarget::Placeholder, layers, keywords);
    println!(
        "  ::placeholder text colour: {:?}",
        style.content_color.is_some()
    );
    println!(
        "  background: {:?} (the selection layers are another target)",
        style.background.is_none()
    );
    assert!(walk.is_empty(), "silence, not notes: {walk:?}");

    println!("── an unnamed target: the quiet answer ──");
    let (style, walk) = program_style_pseudo(PseudoTarget::FirstLetter, layers, keywords);
    println!(
        "  ::first-letter: {} (the corpus names no first-letter layer)",
        style == Default::default()
    );
    assert!(
        walk.is_empty() && style == Default::default(),
        "the quiet path"
    );

    println!("── the runtime tier: the themeless refusal beside the themed form ──");
    let runtime = runtime_table();
    let config = Config::default();
    let compiled = aeolus_compile::compile_theme("selection:bg-primary", &runtime, &config)
        .expect("the string lowers");
    let annotations =
        keyword_texts("selection:bg-primary", &config).expect("non-empty input annotates");
    let keywords: &'static [&'static str] =
        Box::leak(keyword_alignment(&compiled, &annotations).into_boxed_slice());
    let (style, walk) =
        program_style_pseudo(PseudoTarget::Selection, compiled.program.layers, keywords);
    println!("  themeless: background={:?}", style.background.is_some());
    for (prop, reason) in &walk.refusals {
        println!("  refused {prop:?}: {reason}");
    }
    assert!(
        style.background.is_none() && !walk.refusals.is_empty(),
        "the slow lane's boundary"
    );
    let (themed, walk) = program_style_pseudo_theme(
        PseudoTarget::Selection,
        compiled.program.layers,
        keywords,
        &runtime,
        &Env::default(),
    );
    println!(
        "  themed: background={:?} (T2's one load + one multiply)",
        themed.background.is_some()
    );
    assert!(
        themed.background.is_some() && walk.is_empty(),
        "the source answers"
    );

    println!("\nall pseudo-target walks answered headless");
}
