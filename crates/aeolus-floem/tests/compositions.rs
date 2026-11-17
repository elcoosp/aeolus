//! The composition tier's floem half: the frame facts the walk answers
//! from the host's `Env` — `group-active:p-4` fires only while the
//! env carries a `.group` ancestor's active bit, the unset bit the
//! QUIET miss a frame the composition does not hold in is (the
//! starting tier's own reading, the interp verdicts shared verbatim —
//! never an exclusion).
//!
//! Written FIRST (the RED spec): the suite fails while the `Cond` set
//! carries no composition arms.

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
fn the_group_composition_fires_on_the_frame_fact() {
    let (style, report) = lower_program(
        &frozen("group-active:p-4"),
        &Env {
            group_active: true,
            ..Env::default()
        },
    );
    assert_eq!(
        style.get(PaddingTop),
        Length::Pt(16.0),
        "the group-active layer answers the frame fact"
    );
    assert!(
        report.is_empty(),
        "a fired layer rides no exclusion: {:?}",
        report.dropped
    );
}

#[test]
fn the_peer_composition_fires_on_the_frame_fact() {
    let (style, report) = lower_program(
        &frozen("peer-disabled:p-2"),
        &Env {
            peer_disabled: true,
            ..Env::default()
        },
    );
    assert_eq!(
        style.get(PaddingTop),
        Length::Pt(8.0),
        "the peer-disabled layer answers the frame fact"
    );
    assert!(
        report.is_empty(),
        "a fired layer rides no exclusion: {:?}",
        report.dropped
    );
}

#[test]
fn the_unset_facts_stay_the_quiet_miss() {
    let (style, report) = lower_program(
        &frozen("group-active:p-4 peer-disabled:p-2"),
        &Env::default(),
    );
    assert_eq!(
        style.get(PaddingTop),
        Length::Pt(0.0),
        "an unset composition fact is the quiet miss"
    );
    assert!(
        report.is_empty(),
        "a quiet miss carries no exclusion (the media tier's reading): {:?}",
        report.dropped
    );
}
