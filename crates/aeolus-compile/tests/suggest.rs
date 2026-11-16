//! The "did you mean" seam's contracts (the P5 row's tooling-parity
//! half, the miss-side fold): every suggestion is a spelling the
//! pipeline LOWERS — the audited tables are the sources, never a
//! guessed word — the corrections compose onto the FULL token (the
//! variant prefix and the `/modifier` tail ride along), and the
//! diagnostics the grammar refuses whole answer empty.
//!
//! Written FIRST (the RED spec): the suite fails against the
//! not-yet-existing seam and passes only once the fold satisfies
//! every pin below.

use aeolus_compile::{
    CustomUtility, Customs, RegistrySpellings, SUGGESTION_DISTANCE_CAP, SUGGESTION_LIMIT,
    Suggestion, SuggestionSource, SuggestionSurface, SurfaceSpelling, TokenSpellings, compile,
    compile_theme, suggest_for, suggest_over,
};
use aeolus_theme::{ThemeSource, TokenTable};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// The built-in shadcn bake (the engine's and the CLI's default table):
/// the token-surface fixtures fold against the SAME names the CLI's
/// `--theme shadcn` resolves.
fn shadcn_table() -> aeolus_theme::TokenTable {
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    TokenTable::baked(&source).expect("the baked shadcn table builds")
}

/// The diagnostics one string's compile carries (the theme walk: the
/// same entry point the engine runs).
fn diagnostics_of(raw: &str) -> Vec<aeolus_compile::Diagnostic> {
    let compiled = compile_theme(raw, &shadcn_table(), &Default::default())
        .expect("only the empty input errors");
    compiled.diagnostics
}

/// The fold's answers for one string's FIRST diagnostic.
fn suggestions_for(raw: &str) -> Vec<Suggestion> {
    let diagnostics = diagnostics_of(raw);
    suggest_for(&diagnostics[0], None)
}

// ---------------------------------------------------------------------------
// The registry surface
// ---------------------------------------------------------------------------

#[test]
fn unknown_root_suggests_the_registry_spelling() {
    // `flex-clo` — the root `flex` matches, the keyword table misses
    // (the `col`/`clo` transposition the plain distance counts twice).
    let suggestions = suggestions_for("flex-clo");
    assert!(
        suggestions
            .iter()
            .any(|s| s.replacement == "flex-col" && s.source == SuggestionSource::Registry),
        "the keyword-table sibling suggests: {suggestions:?}"
    );
}

#[test]
fn the_suggestion_recompiles_clean() {
    // The honesty rule: a suggested class is a lowered class. Every
    // suggestion the battery's fixtures emit re-compiles with ZERO
    // diagnostics (the audited tables' own guarantee, pinned here).
    for raw in ["flex-clo", "justify-betweenn", "hover:bg-priamry"] {
        for suggestion in suggestions_for(raw) {
            let compiled = compile_theme(
                &suggestion.replacement,
                &shadcn_table(),
                &Default::default(),
            )
            .expect("only the empty input errors");
            assert!(
                compiled.diagnostics.is_empty(),
                "{raw:?} suggested {:?} but it refuses: {:?}",
                suggestion.replacement,
                compiled.diagnostics
            );
        }
    }
}

#[test]
fn shape_rejected_suggests_the_sibling_keyword() {
    // `justify-betweenn` — the root `justify` matched and the suffix
    // fit no shape; the keyword table's own spelling is the nearest
    // legal sibling.
    let suggestions = suggestions_for("justify-betweenn");
    assert!(
        suggestions
            .iter()
            .any(|s| s.replacement == "justify-between" && s.source == SuggestionSource::Registry),
        "the sibling keyword suggests: {suggestions:?}"
    );
}

#[test]
fn the_variant_prefix_composes_back_on() {
    // `hover:flex-clo` — the fold corrects the UTILITY half; the
    // replacement carries the variant prefix (the edit an editor
    // writes over the whole token).
    let suggestions = suggestions_for("hover:flex-clo");
    assert!(
        suggestions
            .iter()
            .any(|s| s.replacement == "hover:flex-col"),
        "the prefix rides the replacement: {suggestions:?}"
    );
}

#[test]
fn the_modifier_tail_survives_the_correction() {
    // `bg-priamry/50` — the theme token miss; the corrected spelling
    // re-attaches the raw's own `/50` (the user's opacity survives).
    // The spelling itself also rides the registry's keyword table
    // (the same-name tie the registry wins), so the composition — not
    // the surface tag — is the pin here; the THEME-TOKEN tag pins in
    // the custom-table battery below.
    let suggestions = suggestions_for("bg-priamry/50");
    assert!(
        suggestions.iter().any(|s| s.replacement == "bg-primary/50"),
        "the modifier tail rides the replacement: {suggestions:?}"
    );
}

// ---------------------------------------------------------------------------
// The token surface (the host-composable one)
// ---------------------------------------------------------------------------

/// A numbered-shadcn theme source: the audited keyword list carries
/// `primary`, the config's own table carries `primary-1` (the
/// numbered-token naming real brand themes ride) — the audit-gate
/// fixture.
fn numbered_table() -> TokenTable {
    let source = ThemeSource::from_css(
        "@theme {\n  --color-primary-1: oklch(0.55 0.2 250);\n  \
         --color-secondary-1: oklch(0.7 0.1 250);\n}\n",
        None,
    )
    .expect("the numbered source parses");
    TokenTable::baked(&source).expect("the baked table builds")
}

#[test]
fn the_token_surface_answers_its_own_names() {
    // The unit-level fold: the shadcn bake's own names, the sorted
    // order the rank — the host-composable surface's contract (the
    // composed path dedupes onto the registry, whose keyword table IS
    // the audited list; see `TokenSpellings`).
    let table = shadcn_table();
    let answers = suggest_over("priamry", &[&TokenSpellings::new(&table)]);
    assert!(
        answers
            .iter()
            .any(|s| s.replacement == "primary" && s.source == SuggestionSource::ThemeToken),
        "the token surface answers its own names: {answers:?}"
    );
}

#[test]
fn the_composed_path_answers_the_keyword_sibling() {
    // `bg-priamry` — the audited spelling `bg-primary` is the answer
    // (the registry's own keyword table carries it; the composed fold
    // rides the registry, the honesty tag is the Registry's).
    let suggestions = suggestions_for("bg-priamry");
    assert!(
        suggestions.iter().any(|s| s.replacement == "bg-primary"),
        "the audited sibling answers: {suggestions:?}"
    );
    for suggestion in &suggestions {
        let compiled = compile_theme(
            &suggestion.replacement,
            &shadcn_table(),
            &Default::default(),
        )
        .expect("only the empty input errors");
        assert!(
            compiled.diagnostics.is_empty(),
            "{:?} refuses: {:?}",
            suggestion.replacement,
            compiled.diagnostics
        );
    }
}

#[test]
fn an_unaudited_token_name_never_suggests() {
    // The audit gate's own pin: the numbered table RESOLVES
    // `primary-1`, but the spelling is no audited keyword — the class
    // would refuse at the shape gate, so the token surface stays
    // quiet (a suggestion that does not lower is not a suggestion).
    let table = numbered_table();
    let compiled = compile_theme("bg-primary", &table, &Default::default())
        .expect("only the empty input errors");
    let suggestions = suggest_for(&compiled.diagnostics[0], None);
    assert!(
        suggestions.iter().all(|s| s.replacement != "bg-primary-1"),
        "the unaudited name never suggests: {suggestions:?}"
    );
}

#[test]
fn the_themeless_boundary_suggests_no_token() {
    // The theme-less fold (the `compile` boundary): the token surface
    // never joins — the keyword siblings still answer.
    let compiled = compile("bg-priamry", &Default::default()).expect("only the empty input errors");
    let suggestions = suggest_for(&compiled.diagnostics[0], None);
    assert!(
        suggestions
            .iter()
            .all(|s| s.source != SuggestionSource::ThemeToken),
        "no token surface without a table: {suggestions:?}"
    );
}

#[test]
fn a_non_colour_root_suggests_no_token() {
    // `p-priamry` — the root `p` carries no colour value shape; the
    // token answer would not lower, so the fold keeps it quiet.
    let suggestions = suggestions_for("p-priamry");
    assert!(
        suggestions
            .iter()
            .all(|s| s.source != SuggestionSource::ThemeToken),
        "the colour-group gate holds: {suggestions:?}"
    );
}

// ---------------------------------------------------------------------------
// The variant surface
// ---------------------------------------------------------------------------

#[test]
fn the_variant_miss_suggests_the_known_spelling() {
    // `havor:p-4` — the segment is outside the typed `Cond` surface;
    // the closed set's own spelling composes back onto the utility.
    let suggestions = suggestions_for("havor:p-4");
    assert!(
        suggestions
            .iter()
            .any(|s| s.replacement == "hover:p-4" && s.source == SuggestionSource::Variant),
        "the closed set answers: {suggestions:?}"
    );
}

#[test]
fn a_composed_stack_suggests_no_variant() {
    // `dark:havor:p-4` — the offending segment's own edit is not the
    // whole token's; the single-spelling boundary keeps it quiet.
    let diagnostics = diagnostics_of("dark:havor:p-4");
    let variant_miss = diagnostics
        .iter()
        .find(|d| d.raw == "dark:havor:p-4")
        .expect("the composed stack refuses");
    assert!(
        suggest_for(variant_miss, None).is_empty(),
        "no single-spelling edit for a composed stack"
    );
}

// ---------------------------------------------------------------------------
// The boundaries: the whole-token refusals stay quiet
// ---------------------------------------------------------------------------

#[test]
fn the_whole_token_refusals_answer_empty() {
    // The kinds with no honest correction (the help text is the
    // story): the lexer's miss, the importance flag, the modifier,
    // the negation.
    for raw in ["p--4", "!p-4", "bg-primary/var(--x)", "-p-4"] {
        let diagnostics = diagnostics_of(raw);
        assert!(
            !diagnostics.is_empty(),
            "{raw:?} refuses (the fixture's own precondition)"
        );
        for diagnostic in &diagnostics {
            assert!(
                suggest_for(diagnostic, None).is_empty(),
                "{raw:?} ({:?}) carries no correction",
                diagnostic.kind
            );
        }
    }
}

#[test]
fn a_far_miss_suggests_nothing() {
    // Beyond the distance cap the fold stays silent — no guessed
    // word. The empty answer is the honest one.
    let suggestions = suggestions_for("qwertyuiop-missing-thing");
    assert!(suggestions.is_empty(), "beyond the cap: {suggestions:?}");
}

// ---------------------------------------------------------------------------
// The customs surface (the host's own audited vocabulary)
// ---------------------------------------------------------------------------

#[test]
fn the_registered_custom_suggests_itself() {
    // `brnd-pad` — no registry root, no custom hit either; with the
    // registration folded in, the host's own root is the answer.
    let customs = Customs::new().utility(CustomUtility::fixed(
        "brand-pad",
        aeolus_ir::Val::Px(16.0),
        &[aeolus_ir::Prop::PaddingTop, aeolus_ir::Prop::PaddingBottom],
    ));
    let compiled = compile("brnd-pad", &Default::default()).expect("only the empty input errors");
    let with = suggest_for(&compiled.diagnostics[0], Some(&customs));
    assert!(
        with.iter()
            .any(|s| s.replacement == "brand-pad" && s.source == SuggestionSource::Custom),
        "the customs surface answers: {with:?}"
    );
    // The built-ins alone keep the quiet answer (the surface's reason
    // to exist: the host's vocabulary is invisible without it).
    let without = suggest_for(&compiled.diagnostics[0], None);
    assert!(
        without.iter().all(|s| s.replacement != "brand-pad"),
        "no custom answer without the registration: {without:?}"
    );
}

// ---------------------------------------------------------------------------
// The fold's own contract: order, dedupe, cap, determinism
// ---------------------------------------------------------------------------

#[test]
fn the_answer_is_ranked_and_capped() {
    // Distances ascend, the width respects the fold's limit, and the
    // registry's own order breaks the distance ties.
    let suggestions = suggestions_for("justify-betwen");
    assert!(
        suggestions.len() <= SUGGESTION_LIMIT,
        "capped: {suggestions:?}"
    );
    for pair in suggestions.windows(2) {
        assert!(
            pair[0].distance <= pair[1].distance,
            "distances ascend: {suggestions:?}"
        );
    }
}

#[test]
fn the_fold_is_deterministic() {
    let first = suggestions_for("flex-clo");
    let second = suggestions_for("flex-clo");
    assert_eq!(first, second, "the same miss answers the same menu");
}

#[test]
fn the_distance_cap_is_the_constant() {
    // The boundary the battery pins: distance 2 answers (the
    // transposition), 3 does not.
    let surfaces: [&dyn SuggestionSurface; 1] = [&RegistrySpellings];
    assert!(
        suggest_over("flex-cl", &surfaces)
            .iter()
            .any(|s| s.replacement == "flex-col" && s.distance <= SUGGESTION_DISTANCE_CAP),
        "the transposition answers"
    );
    assert!(
        suggest_over("flex-cccccccc", &surfaces).is_empty(),
        "far beyond the cap, the fold stays silent"
    );
}

// ---------------------------------------------------------------------------
// The trait is the seam: a host surface folds in beside the built-ins
// ---------------------------------------------------------------------------

/// A host's own audited surface (the seam's reason to exist: the
/// built-ins cannot know the host's vocabulary).
struct HostSpellings;

impl SuggestionSurface for HostSpellings {
    fn source(&self) -> SuggestionSource {
        SuggestionSource::Custom
    }

    fn nearest(&self, token: &str, cap: usize) -> Vec<SurfaceSpelling> {
        ["brand-shadow"]
            .iter()
            .filter(|spelling| spelling.distance_le(token, cap))
            .map(|spelling| SurfaceSpelling {
                spelling: (*spelling).to_string(),
                rank: 0,
            })
            .collect()
    }
}

/// The battery's own bounded-distance helper (the test-side twin of
/// the fold's, one call site, no parallel drift to pin).
trait DistanceLe {
    fn distance_le(&self, other: &str, cap: usize) -> bool;
}

impl DistanceLe for str {
    fn distance_le(&self, other: &str, cap: usize) -> bool {
        let a: Vec<char> = self.chars().collect();
        let b: Vec<char> = other.chars().collect();
        if a.len().abs_diff(b.len()) > cap {
            return false;
        }
        let mut prev: Vec<usize> = (0..=b.len()).collect();
        let mut cur = vec![0usize; b.len() + 1];
        for i in 1..=a.len() {
            cur[0] = i;
            for j in 1..=b.len() {
                let cost = usize::from(a[i - 1] != b[j - 1]);
                cur[j] = (prev[j] + 1).min(cur[j - 1] + 1).min(prev[j - 1] + cost);
            }
            std::mem::swap(&mut prev, &mut cur);
        }
        prev[b.len()] <= cap
    }
}

#[test]
fn the_host_surface_folds_into_the_answer() {
    let surfaces: Vec<&dyn SuggestionSurface> = vec![&RegistrySpellings, &HostSpellings];
    let answers = suggest_over("brand-shadw", &surfaces);
    assert!(
        answers.iter().any(|s| s.replacement == "brand-shadow"),
        "the host surface answers beside the built-ins: {answers:?}"
    );
}

#[test]
fn the_registry_wins_the_collision() {
    // The same spelling at the same distance from the registry and a
    // host surface: the registry's tag wins (the declaration order is
    // the tie-break's last key).
    struct RegistryTwin;
    impl SuggestionSurface for RegistryTwin {
        fn source(&self) -> SuggestionSource {
            SuggestionSource::Custom
        }
        fn nearest(&self, token: &str, cap: usize) -> Vec<SurfaceSpelling> {
            RegistrySpellings.nearest(token, cap)
        }
    }
    let surfaces: Vec<&dyn SuggestionSurface> = vec![&RegistryTwin, &RegistrySpellings];
    let answers = suggest_over("flex-clo", &surfaces);
    let flex_col = answers
        .iter()
        .find(|s| s.replacement == "flex-col")
        .expect("the spelling answers");
    assert_eq!(
        flex_col.source,
        SuggestionSource::Registry,
        "the registry's tag wins the tie: {answers:?}"
    );
}
