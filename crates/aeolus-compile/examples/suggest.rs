//! Consumer demonstration: the suggestion seam (`suggest_for` /
//! [`SuggestionSurface`]) — the "did you mean" fold the CLI's `check`
//! face and the LSP's code actions ride. A refused class carries a
//! typed [`Diagnostic`]; the seam folds the AUDITED surfaces (the
//! registry's rows, the theme's keyword tables, the variant set, the
//! registered customs) over the miss's own half with a bounded edit
//! distance, and hands back replacements ready to write over the
//! diagnostic's span — never a guess (every handed spelling lowers).
//!
//! ```sh
//! cargo run -p aeolus-compile --example suggest
//! ```

use aeolus_compile::{
    Config, CustomUtility, Customs, RegistrySpellings, Suggestion, SuggestionSource,
    SuggestionSurface, compile, compile_with, suggest_for, suggest_over,
};
use aeolus_ir::{Prop, Val};

fn main() {
    let config = Config::default();

    println!("── a miss with a near spelling: flex-clo → the registry answers ──");
    let compiled = compile("flex-clo", &config).expect("non-empty input compiles");
    let suggestions = suggest_for(&compiled.diagnostics[0], None);
    for s in &suggestions {
        println!(
            "  did you mean `{}`? ({} edits, {:?})",
            s.replacement, s.distance, s.source
        );
    }
    assert!(
        suggestions
            .iter()
            .any(|s| s.replacement == "flex-col" && s.source == SuggestionSource::Registry),
        "the registry surface carries the audited keyword-table sibling"
    );

    println!();
    println!("── the variant half: havor:p-4 → the variant set answers ──");
    let compiled = compile("havor:p-4", &config).expect("non-empty input compiles");
    let suggestions = suggest_for(&compiled.diagnostics[0], None);
    for s in &suggestions {
        println!(
            "  did you mean `{}`? ({} edits, {:?})",
            s.replacement, s.distance, s.source
        );
    }
    assert!(
        suggestions
            .iter()
            .any(|s| s.replacement == "hover:p-4" && s.source == SuggestionSource::Variant),
        "the closed variant set composes back onto the utility"
    );

    println!();
    println!("── the raw surface walk: fold the audited spellings yourself ──");
    // The seam's host half: any surface the host owns joins the walk
    // beside the built-ins ([`SuggestionSurface`] is the seam, the
    // built-in tables are consumers of it too — the DynStore
    // precedent).
    let nearest = RegistrySpellings.nearest("flex-clo", 2);
    for candidate in nearest.iter().take(3) {
        println!("  within 2 edits of `flex-clo`: {candidate:?}");
    }
    let answers: Vec<Suggestion> = suggest_over("flex-clo", &[&RegistrySpellings]);
    assert!(
        answers
            .iter()
            .any(|s| s.replacement == "flex-col" && s.distance <= 2),
        "the bounded walk finds the row spelling"
    );

    println!();
    println!("── the custom surface: a registered alias joins the fold ──");
    // `brnd-pad` — no registry root, no custom hit either; with the
    // registration folded in, the host's own root is the answer (the
    // host's vocabulary is invisible without the surface).
    let customs = Customs::new().utility(brand_pad_utility());
    let compiled = compile_with("brnd-pad", &customs, &config).expect("non-empty input compiles");
    let with = suggest_for(&compiled.diagnostics[0], Some(&customs));
    for s in &with {
        println!(
            "  did you mean `{}`? ({} edits, {:?})",
            s.replacement, s.distance, s.source
        );
    }
    assert!(
        with.iter()
            .any(|s| s.replacement == "brand-pad" && s.source == SuggestionSource::Custom),
        "the customs surface answers: {with:?}"
    );

    println!();
    println!("the seam answers only with audited spellings — never a guess.");
}

/// The registered custom utility (the `CustomUtility::fixed`
/// constructor — root, baked value, the longhands it fans out to).
fn brand_pad_utility() -> CustomUtility {
    CustomUtility::fixed(
        "brand-pad",
        Val::Px(16.0),
        &[Prop::PaddingTop, Prop::PaddingBottom],
    )
}
