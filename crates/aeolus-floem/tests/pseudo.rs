//! The pseudo-element render-target tier's Floem half: the static-program
//! walk answers the ELEMENT cascade, and a pseudo-element layer is never
//! element state — every pseudo spelling rides the walk's typed
//! exclusion report (the interp's `PseudoElement` note on the Floem
//! report channel), never the style.
//!
//! Written FIRST (the RED spec): the spellings refuse at the fold while
//! the tier is un-landed.

use aeolus_compile::{Config, compile_theme, keyword_alignment, keyword_texts};
use aeolus_floem::lower_program;
use aeolus_rt::{Env, StaticProgram};
use aeolus_theme::{ThemeSource, TokenTable};

use floem::style::PaddingTop;
use floem::style::unit::Length;

/// The baked shadcn LIGHT table (the `tw!` macro's freeze shape).
fn baked_table() -> TokenTable {
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    TokenTable::baked(&source).expect("the baked shadcn table builds")
}

/// Compile + freeze a class string the way `tw!` does.
fn frozen(classes: &str) -> StaticProgram {
    let config = Config::default();
    let compiled =
        compile_theme(classes, &baked_table(), &config).expect("the class string lowers");
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

#[test]
fn pseudo_layers_stay_out_of_the_element_cascade() {
    let (style, report) = lower_program(&frozen("before:p-4"), &Env::default());
    assert_eq!(
        style.get(PaddingTop),
        Length::Pt(0.0),
        "a pseudo layer is never element state"
    );
    assert!(
        !report.is_empty(),
        "the exclusion rides the report, never silent: {:?}",
        report.dropped
    );
}

#[test]
fn the_base_cascade_answers_beside_an_excluded_pseudo_layer() {
    let (style, _report) = lower_program(&frozen("p-8 after:p-4"), &Env::default());
    assert_eq!(
        style.get(PaddingTop),
        Length::Pt(32.0),
        "the base layer answers beside the exclusion"
    );
}

#[test]
fn every_pseudo_spelling_stays_excluded() {
    for classes in [
        "marker:p-4",
        "first-letter:p-4",
        "first-line:p-4",
        "file:p-4",
        "backdrop:p-4",
        "details-content:p-4",
    ] {
        let (style, report) = lower_program(&frozen(classes), &Env::default());
        assert_eq!(
            style.get(PaddingTop),
            Length::Pt(0.0),
            "{classes}: never element state"
        );
        assert!(!report.is_empty(), "{classes}: the exclusion reports");
    }
}
