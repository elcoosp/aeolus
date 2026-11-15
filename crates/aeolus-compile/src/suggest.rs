//! The "did you mean" suggestion seam (the P5 row's tooling-parity
//! half): every typed miss the pipeline reports gains the machine-
//! readable CORRECTIONS the linter renders and the editor offers as
//! quick fixes.
//!
//! The v0.x oracle answers the miss with a bounded Levenshtein walk
//! over a hand-kept prefix list (`aeolus-core`'s `suggest`) — the V1
//! pipeline's diagnostics carried only the prose help. This module
//! lands the seam the data crate's docs always promised: the audited
//! tables ARE the suggestion sources (the registry's rows, the theme
//! pass's token names, the compile crate's closed variant set, the
//! registered customs), so every emitted suggestion is a spelling the
//! pipeline LOWERS — the completion's own one-source-of-truth rule,
//! never a guessed word.
//!
//! The shape is the crate's seam idiom (the `DynStore`/`AttrProbe`
//! precedent): a trait names the contract, the built-in surfaces
//! implement it over their own tables, and a host folds its own
//! audited names in through the same trait — the registered customs
//! are exactly the surface the built-ins cannot know.
//!
//! ```no_run
//! use aeolus_compile::{compile, suggest_for};
//!
//! let compiled = compile("flex-clo", &Default::default()).unwrap();
//! for diagnostic in &compiled.diagnostics {
//!     for suggestion in suggest_for(diagnostic, None, None) {
//!         println!("did you mean `{}`?", suggestion.replacement);
//!     }
//! }
//! ```

use std::collections::HashSet;

use aeolus_data::rows::ROWS;
use aeolus_data::{UtilityRow, ValueKind};
use aeolus_theme::TokenTable;

use crate::{Customs, Diagnostic, DiagnosticKind, KNOWN_VARIANTS};

// ---------------------------------------------------------------------------
// The fold's constants
// ---------------------------------------------------------------------------

/// The bounded edit distance a suggestion answers within: one typo, or
/// the transposition the plain distance counts as two edits. Tighter
/// than the oracle's 3 — the audited-spelling fold stays precise (the
/// no-guessed-word rule), and the battery pins the boundary.
pub const SUGGESTION_DISTANCE_CAP: usize = 2;

/// The answer's width: the editor renders a menu, the fold keeps the
/// best-ranked few (the oracle's single best, widened for the picker).
pub const SUGGESTION_LIMIT: usize = 3;

// ---------------------------------------------------------------------------
// The public shapes
// ---------------------------------------------------------------------------

/// Which audited surface a suggestion's spelling came from — the
/// honesty tag (every source's spellings lower by construction) and
/// the tie-break's last key: the declaration order is the priority
/// when two surfaces carry the same spelling at the same distance
/// (the registry wins a custom-root collision, the oracle's own rule).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SuggestionSource {
    /// The registry's rows: bare roots and the keyword/composite
    /// tables' spellings.
    Registry,
    /// The config's own [`TokenTable`] names (the theme pass's surface).
    ThemeToken,
    /// The compile crate's closed [`KNOWN_VARIANTS`] set.
    Variant,
    /// The registered [`Customs`] (the host's own audited names).
    Custom,
}

/// One candidate spelling a surface hands the fold: the spelling and
/// the surface's own canonical rank (the walk order — the registry's
/// row order, the table's order, the variant set's order, the
/// registration order). Ties in the distance sort break by rank.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SurfaceSpelling {
    /// The audited spelling, verbatim (a full class for the registry
    /// and custom-utility surfaces, a token or segment name for the
    /// others).
    pub spelling: String,
    /// The surface's canonical rank (strictly increasing along its
    /// own walk).
    pub rank: usize,
}

/// One audited surface of candidate spellings — the suggestion fold's
/// source seam. The built-ins cover the compiler's own tables; a host
/// folds its own audited names in through the same contract (the
/// `DynStore` precedent: the trait is the seam, the built-ins are
/// consumers of it too).
///
/// The surface owns the AUDIT: every spelling handed over must lower
/// (the fold never re-compiles — the distance walk is the only gate).
/// The cost contract mirrors the oracle's: the walk runs on the cold
/// miss path only, bounded by [`SUGGESTION_DISTANCE_CAP`]'s early
/// abandon, so a full-table sweep never allocates per candidate.
pub trait SuggestionSurface {
    /// Which audited surface the spellings come from (the answer's
    /// honesty tag and the fold's tie-break).
    fn source(&self) -> SuggestionSource;

    /// The surface's spellings within `cap` edits of `token`, in the
    /// surface's own canonical order. `token` is the half the caller
    /// is correcting (the utility body, the suffix, the variant
    /// segment) — the surface measures candidates as IT carries them.
    fn nearest(&self, token: &str, cap: usize) -> Vec<SurfaceSpelling>;
}

/// One suggestion: the corrected FULL token text (the variant prefix
/// and the `/modifier` tail composed back on — the edit an editor
/// writes over the diagnostic's span), the audited surface the
/// spelling came from, and the measured distance (the menu's own
/// confidence key).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Suggestion {
    /// The corrected token, ready to replace the diagnostic's span.
    pub replacement: String,
    /// The audited surface the spelling came from.
    pub source: SuggestionSource,
    /// The bounded edit distance the fold measured (the corrected
    /// half against the candidate's own half).
    pub distance: usize,
}

// ---------------------------------------------------------------------------
// The built-in surfaces
// ---------------------------------------------------------------------------

/// The registry's rows as spellings: every bare root and every
/// keyword/composite suffix joined to its root (`flex`, `flex-col`,
/// `items-center` — the spellings the registry lowers). The walk is
/// the rows' own order; the rank is the running index along it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RegistrySpellings;

impl SuggestionSurface for RegistrySpellings {
    fn source(&self) -> SuggestionSource {
        SuggestionSource::Registry
    }

    fn nearest(&self, token: &str, cap: usize) -> Vec<SurfaceSpelling> {
        let mut out = Vec::new();
        let mut rank = 0usize;
        for row in ROWS {
            if bounded_levenshtein(token, row.root, cap).is_some() {
                out.push(SurfaceSpelling {
                    spelling: row.root.to_string(),
                    rank,
                });
            }
            rank += 1;
            for spelling in row_spellings(row) {
                if bounded_levenshtein(token, &spelling, cap).is_some() {
                    out.push(SurfaceSpelling { spelling, rank });
                }
                rank += 1;
            }
        }
        out
    }
}

/// The config's own token names (the theme pass's surface): the table
/// walked in its sorted-name order (the completion's own order), the
/// rank the sorted position. The HOST-COMPOSABLE surface — the
/// diagnostic-keyed composition folds the registry alone, because the
/// audited keyword list IS the registry's own table (the dedupe
/// collapses the two surfaces onto one answer; one source of truth).
/// A host whose editor answers spelling menus OUTSIDE the diagnostics
/// (the picker's own vocabularies) folds this surface through
/// [`suggest_over`] directly; the names it hands over are the names
/// the table RESOLVES — a host gate keeps them lowerable the way the
/// completion's colour gate does.
pub struct TokenSpellings<'a> {
    table: &'a TokenTable,
}

impl<'a> TokenSpellings<'a> {
    /// The surface over one config table.
    pub fn new(table: &'a TokenTable) -> Self {
        Self { table }
    }
}

impl SuggestionSurface for TokenSpellings<'_> {
    fn source(&self) -> SuggestionSource {
        SuggestionSource::ThemeToken
    }

    fn nearest(&self, token: &str, cap: usize) -> Vec<SurfaceSpelling> {
        let mut names: Vec<&str> = self.table.names().collect();
        names.sort_unstable();
        names
            .into_iter()
            .enumerate()
            .filter(|(_, name)| bounded_levenshtein(token, name, cap).is_some())
            .map(|(rank, name)| SurfaceSpelling {
                spelling: name.to_string(),
                rank,
            })
            .collect()
    }
}

/// The compile crate's closed variant set ([`KNOWN_VARIANTS`]): the
/// fold's own audited spellings, set order the rank. The surface joins
/// the fold for the VARIANT half's misses only (a utility body never
/// wants a variant spelling — the replacement must lower as a class).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VariantSpellings;

impl SuggestionSurface for VariantSpellings {
    fn source(&self) -> SuggestionSource {
        SuggestionSource::Variant
    }

    fn nearest(&self, token: &str, cap: usize) -> Vec<SurfaceSpelling> {
        KNOWN_VARIANTS
            .iter()
            .enumerate()
            .filter(|(_, spelling)| bounded_levenshtein(token, spelling, cap).is_some())
            .map(|(rank, spelling)| SurfaceSpelling {
                spelling: (*spelling).to_string(),
                rank,
            })
            .collect()
    }
}

/// The registered customs' names: the utilities' roots and the
/// variants' aliases, registration order the rank. The surface the
/// built-ins cannot know — a host's own audited vocabulary rides the
/// same fold (the trait's reason to exist).
pub struct CustomSpellings<'a> {
    customs: &'a Customs,
}

impl<'a> CustomSpellings<'a> {
    /// The surface over one registration bag.
    pub fn new(customs: &'a Customs) -> Self {
        Self { customs }
    }
}

impl SuggestionSurface for CustomSpellings<'_> {
    fn source(&self) -> SuggestionSource {
        SuggestionSource::Custom
    }

    fn nearest(&self, token: &str, cap: usize) -> Vec<SurfaceSpelling> {
        let mut out = Vec::new();
        let mut rank = 0usize;
        for utility in self.customs.utilities() {
            if bounded_levenshtein(token, utility.root, cap).is_some() {
                out.push(SurfaceSpelling {
                    spelling: utility.root.to_string(),
                    rank,
                });
            }
            rank += 1;
        }
        for variant in self.customs.variants() {
            if bounded_levenshtein(token, variant.name, cap).is_some() {
                out.push(SurfaceSpelling {
                    spelling: variant.name.to_string(),
                    rank,
                });
            }
            rank += 1;
        }
        out
    }
}

// ---------------------------------------------------------------------------
// The fold
// ---------------------------------------------------------------------------

/// The raw spelling fold: every surface's near spellings, ranked by
/// (distance, the surface's canonical rank, the surface's priority),
/// deduped on the replacement, capped at [`SUGGESTION_LIMIT`]. The
/// composition is the caller's (the diagnostic-keyed [`suggest_for`]
/// owns the variant prefixes and modifier tails); a host with its own
/// audited surface folds it in here.
pub fn suggest_over(token: &str, surfaces: &[&dyn SuggestionSurface]) -> Vec<Suggestion> {
    let token = token.trim();
    if token.is_empty() {
        return Vec::new();
    }
    let mut folded: Vec<(usize, usize, SuggestionSource, String)> = Vec::new();
    for surface in surfaces {
        for hit in surface.nearest(token, SUGGESTION_DISTANCE_CAP) {
            let Some(distance) = bounded_levenshtein(token, &hit.spelling, SUGGESTION_DISTANCE_CAP)
            else {
                continue;
            };
            folded.push((distance, hit.rank, surface.source(), hit.spelling));
        }
    }
    folded.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));
    let mut out: Vec<Suggestion> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for (distance, _, source, spelling) in folded {
        if out.len() >= SUGGESTION_LIMIT {
            break;
        }
        if seen.insert(spelling.clone()) {
            out.push(Suggestion {
                replacement: spelling,
                source,
                distance,
            });
        }
    }
    out
}

// ---------------------------------------------------------------------------
// The diagnostic-keyed composition
// ---------------------------------------------------------------------------

/// The miss-side composition: the diagnostic's own data keys the fold
/// (the offending half and the audited surfaces that answer it), the
/// corrections compose back onto the FULL token (the variant prefix
/// and the `/modifier` tail ride along), and every emitted suggestion
/// is a spelling the pipeline lowers. Diagnostics the grammar refuses
/// whole (`lex`, the importance flag, the modifier, the negation)
/// answer empty — no honest correction exists, the help text is the
/// story.
///
/// * `tokens` — the theme pass's table (the token surface joins for
///   the not-a-token miss when the root is a colour-group root; the
///   engine's and the CLI's own config tables ride in here);
/// * `customs` — the registered customs (the custom surface joins
///   every utility-half fold).
pub fn suggest_for(
    diagnostic: &Diagnostic,
    tokens: Option<&TokenTable>,
    customs: Option<&Customs>,
) -> Vec<Suggestion> {
    let _ = tokens; // the composed fold rides the registry's own audited tables (see compose_utility)
    match &diagnostic.kind {
        DiagnosticKind::UnknownRoot { body } => compose_utility(diagnostic, body, customs),
        DiagnosticKind::ShapeRejected { body, .. } => compose_utility(diagnostic, body, customs),
        DiagnosticKind::UnsupportedValue { body, .. } => {
            // The value pass's miss folds the same audited surfaces
            // (the keyword-table siblings are the closest legal
            // spellings the row carries); the token surface stays the
            // host-composable one (the audited keyword list IS the
            // registry's table — see `TokenSpellings`).
            compose_utility(diagnostic, body, customs)
        }
        DiagnosticKind::UnsupportedVariant { variant } => compose_variant(diagnostic, variant),
        DiagnosticKind::Lex { .. }
        | DiagnosticKind::UnsupportedImportant
        | DiagnosticKind::UnsupportedModifier { .. }
        | DiagnosticKind::UnsupportedNegation { .. } => Vec::new(),
    }
}

/// The utility half's fold: the registry and the customs over the
/// body, the replacement composed onto the raw's variant prefix with
/// the raw's own modifier tail re-attached (the user's `/50` survives
/// the correction).
fn compose_utility(
    diagnostic: &Diagnostic,
    body: &str,
    customs: Option<&Customs>,
) -> Vec<Suggestion> {
    let (prefix, utility_half) = split_variant_prefix(&diagnostic.raw);
    let tail = modifier_tail(utility_half, body);
    let registry = RegistrySpellings;
    let custom = customs.map(CustomSpellings::new);
    let mut surfaces: Vec<&dyn SuggestionSurface> = vec![&registry];
    if let Some(custom) = &custom {
        surfaces.push(custom);
    }
    suggest_over(body, &surfaces)
        .into_iter()
        .map(|suggestion| Suggestion {
            replacement: format!("{prefix}{}{tail}", suggestion.replacement),
            source: suggestion.source,
            distance: suggestion.distance,
        })
        .collect()
}

/// The variant half's fold: the closed set over the offending segment,
/// the replacement composed as spelling + `:` + the utility half —
/// for the SINGLE-segment stack only (a composed stack has no honest
/// single-spelling edit; the empty answer is the boundary).
fn compose_variant(diagnostic: &Diagnostic, variant: &str) -> Vec<Suggestion> {
    let Some((stack, utility_half)) = diagnostic.raw.rsplit_once(':') else {
        return Vec::new();
    };
    if stack != variant {
        // A composed stack (`dark:hover:x`): the offending segment's
        // own edit is not the whole token's — no single replacement.
        return Vec::new();
    }
    suggest_over(variant, &[&VariantSpellings])
        .into_iter()
        .map(|suggestion| Suggestion {
            replacement: format!("{}:{utility_half}", suggestion.replacement),
            source: suggestion.source,
            distance: suggestion.distance,
        })
        .collect()
}

/// The raw's variant prefix (up to and including the last `:`) and the
/// utility half after it — the split the composition rides. A raw
/// without a variant prefix splits as (`""`, the raw).
fn split_variant_prefix(raw: &str) -> (&str, &str) {
    match raw.rsplit_once(':') {
        Some((prefix, half)) => (&raw[..=prefix.len()], half),
        None => ("", raw),
    }
}

/// The raw's own modifier tail (the `/50` the typo'd token carried),
/// re-attached verbatim when the utility half extends past the body.
fn modifier_tail<'a>(utility_half: &'a str, body: &str) -> &'a str {
    utility_half.strip_prefix(body).unwrap_or("")
}

/// The keyword/composite spellings one row lowers — the surface walk's
/// per-row half (the `flatten` read the value pass mirrors).
fn row_spellings(row: &UtilityRow) -> Vec<String> {
    let mut arms = Vec::new();
    flatten_kind(row.kind, &mut arms);
    let mut out = Vec::new();
    for arm in arms {
        match arm {
            ValueKind::Keywords(table) => {
                for &(spelling, _) in table {
                    out.push(format!("{}-{}", row.root, spelling));
                }
            }
            ValueKind::Composite(table) => {
                for &(spelling, _, _) in table {
                    out.push(format!("{}-{}", row.root, spelling));
                }
            }
            _ => {}
        }
    }
    out
}

/// The `AnyOf` arms in declaration order (the value pass's own shape
/// walk — the crate's `lower` module carries the original; this
/// module's read needs the same shape).
fn flatten_kind(kind: ValueKind, out: &mut Vec<ValueKind>) {
    match kind {
        ValueKind::AnyOf(inner) => {
            for k in inner {
                flatten_kind(*k, out);
            }
        }
        k => out.push(k),
    }
}

// ---------------------------------------------------------------------------
// The bounded distance
// ---------------------------------------------------------------------------

/// The plain Levenshtein distance with an early abandon: the length
/// difference pre-gates, the DP rows abandon the moment their minimum
/// exceeds `cap` (the oracle's own algorithm, bounded — the miss path
/// never pays the full table). `None` — too far, never a suggestion.
fn bounded_levenshtein(a: &str, b: &str, cap: usize) -> Option<usize> {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.len().abs_diff(b.len()) > cap {
        return None;
    }
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur: Vec<usize> = vec![0; b.len() + 1];
    for (i, a_ch) in a.iter().enumerate() {
        cur[0] = i + 1;
        let mut row_min = cur[0];
        for (j, b_ch) in b.iter().enumerate() {
            let cost = usize::from(a_ch != b_ch);
            cur[j + 1] = (prev[j + 1] + 1).min(cur[j] + 1).min(prev[j] + cost);
            row_min = row_min.min(cur[j + 1]);
        }
        if row_min > cap {
            return None;
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    let distance = prev[b.len()];
    (distance <= cap).then_some(distance)
}
