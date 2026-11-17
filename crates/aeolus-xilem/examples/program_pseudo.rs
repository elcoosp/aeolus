//! Consumer demonstration: the pseudo-target face on the backend (the
//! render-target contract's backend half). A `tw!`-compiled
//! [`StaticProgram`] carries `selection:`/`placeholder:` layers the
//! ELEMENT walk never applies — they style a different render target.
//! [`lower_program_pseudo`] resolves ONE target's own cascade into the
//! plan path's own [`ResolvedStyle`] — the answer a host that PAINTS
//! the target (a selection layer, a custom placeholder) consumes —
//! while the element's own layers and the sibling targets stay silent
//! on the target's side. Headless by construction: the walk is pure.
//!
//! ```sh
//! cargo run -p aeolus-xilem --example program_pseudo
//! ```

use aeolus_compile::{Config, PseudoTarget, compile_theme, keyword_alignment, keyword_texts};
use aeolus_rt::{Env, StaticProgram};
use aeolus_theme::{ThemeSource, TokenTable};
use aeolus_xilem::{lower_program, lower_program_pseudo, lower_program_pseudo_theme};

/// Compile a class string the way `tw!` does (the theme pass's baked
/// shadcn table) and freeze it into the static form the macro emits:
/// the interned layers plus the row texts the keyword slots index.
fn frozen(classes: &str) -> StaticProgram {
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    let table = TokenTable::baked(&source).expect("the baked shadcn table builds");
    let config = Config::default();
    let compiled = compile_theme(classes, &table, &config).expect("the class string lowers");
    assert!(
        compiled.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        compiled.diagnostics
    );
    let annotations = keyword_texts(classes, &config).expect("non-empty input annotates");
    let keywords = keyword_alignment(&compiled, &annotations);
    StaticProgram::new(
        compiled.program.layers,
        Box::leak(keywords.into_boxed_slice()),
    )
}

fn bg_of(style: &aeolus_core::plan::ResolvedStyle) -> Option<String> {
    style
        .background
        .as_ref()
        .map(|b| format!("{:?}", b.0.to_srgba()))
}

fn main() {
    // One call site: the element's own classes beside two render
    // targets' cascades, exactly the shape a `tw!` on a text widget
    // compiles.
    let classes = "p-4 bg-primary text-primary-foreground rounded-md \
                   selection:bg-red-500 selection:text-white \
                   placeholder:text-muted-foreground";
    let program = frozen(classes);

    println!("── the render-target contract, both halves ──");
    println!("  layers walked: {}", program.layers.len());

    // The element walk's half: the element's own cascade, the pseudo
    // layers REPORTED (the InterpNote channel) — never applied, never
    // silently dropped.
    let (element, _report) = lower_program(&program, &Env::default());
    println!("  element background:       {}", bg_of(&element).unwrap());
    println!("  element padding (p-4):    {:?}", element.padding);
    println!(
        "  selection bg on the element side: {}",
        bg_of(&element).unwrap()
    );

    // The target walk's half: the SELECTION cascade only — later fired
    // wins per longhand (text-white is the later `selection:` beat for
    // the text colour, bg-red-500 the only background beat). The
    // element's own p-4/bg-primary and the placeholder sibling are
    // SILENCE here, not notes.
    let (selection, selection_report) = lower_program_pseudo(&program, PseudoTarget::Selection);
    println!("── the ::selection target's own cascade ──");
    println!(
        "  background (bg-red-500):  {}",
        bg_of(&selection).unwrap_or_else(|| "(unset)".to_string())
    );
    println!(
        "  content colour:           {:?}",
        selection.content_color.as_ref().map(|c| c.0.to_srgba())
    );
    println!(
        "  element padding present:  {}",
        selection.padding.is_some()
    );
    println!(
        "  walk report:              {} drops (a clean target walk reports nothing)",
        selection_report.dropped.len()
    );

    // The sibling stays silent: the PLACEHOLDER fold answers only the
    // placeholder layer's cascade.
    let (placeholder, _) = lower_program_pseudo(&program, PseudoTarget::Placeholder);
    println!("── the ::placeholder target's own cascade ──");
    println!(
        "  background present:       {}",
        placeholder.background.is_some()
    );
    println!(
        "  content colour:           {:?}",
        placeholder.content_color.as_ref().map(|c| c.0.to_srgba())
    );

    // The theme forms: the themeless face refuses the runtime tier's
    // token slots per longhand into the report; the themed face
    // resolves them through one TokenSource (the env rides the
    // resolution, never the firing).
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    let runtime = TokenTable::runtime(&source).expect("the runtime table builds");
    let config = Config::default();
    let token_classes = "selection:bg-primary";
    let compiled =
        compile_theme(token_classes, &runtime, &config).expect("the token string lowers");
    let annotations = keyword_texts(token_classes, &config).expect("non-empty input annotates");
    let keywords = keyword_alignment(&compiled, &annotations);
    let token_program = StaticProgram::new(
        compiled.program.layers,
        Box::leak(keywords.into_boxed_slice()),
    );
    println!("── the theme forms on the target face ──");
    let (refused, refused_report) = lower_program_pseudo(&token_program, PseudoTarget::Selection);
    println!(
        "  themeless face:           bg present: {}, {} refusal(s) in the report \
         (the longhand named, never fabricated)",
        refused.background.is_some(),
        refused_report.dropped.len()
    );
    let (resolved, resolved_report) = lower_program_pseudo_theme(
        &token_program,
        PseudoTarget::Selection,
        &runtime,
        &Env::default(),
    );
    println!(
        "  themed face:              bg present: {}, {} drops \
         (the table resolves the token)",
        resolved.background.is_some(),
        resolved_report.dropped.len()
    );
}
