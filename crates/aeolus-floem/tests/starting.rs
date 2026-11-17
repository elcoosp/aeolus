//! The `starting:` variant's floem half: the frame fact the walk
//! answers from the host's `Env` — `starting:p-4` fires only while the
//! env carries the element's starting-style bit (the transition's
//! first frame, the host's first-render gate), the unset bit the QUIET
//! miss a frame the element is not in is (the media-feature facts'
//! own reading, the interp verdicts shared verbatim — never an
//! exclusion).
//!
//! Written FIRST (the RED spec): the suite fails while the `Cond` set
//! carries no starting-style arm.

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
fn the_starting_layer_fires_on_the_frame_fact() {
    let (style, report) = lower_program(
        &frozen("starting:p-4"),
        &Env {
            starting_style: true,
            ..Env::default()
        },
    );
    assert_eq!(
        style.get(PaddingTop),
        Length::Pt(16.0),
        "the starting-style layer answers the frame fact"
    );
    assert!(
        report.is_empty(),
        "a fired layer rides no exclusion: {:?}",
        report.dropped
    );
}

#[test]
fn the_unset_fact_stays_the_quiet_miss() {
    let (style, report) = lower_program(&frozen("starting:p-4"), &Env::default());
    assert_eq!(
        style.get(PaddingTop),
        Length::Pt(0.0),
        "an unset frame fact is the quiet miss"
    );
    assert!(
        report.is_empty(),
        "a quiet miss carries no exclusion (the media tier's reading): {:?}",
        report.dropped
    );
}

#[test]
fn the_base_layer_answers_beside_a_quiet_starting_layer() {
    let (style, _report) = lower_program(&frozen("p-8 starting:p-4"), &Env::default());
    assert_eq!(
        style.get(PaddingTop),
        Length::Pt(32.0),
        "the base cascade answers beside the quiet miss"
    );
}
