//! The class-sorting seam (architecture plan §5.5's tooling-parity
//! line, "class sorting (official order from the registry)" — the same
//! line the CLI's `sort` command serves; §3.2's CLI row names the
//! surface).
//!
//! One pure read-side walk over the pipeline's own sources of truth:
//!
//! * **the registry's row order** decides each class's position
//!   ([`aeolus_data::registry::sort_index`] — the FIRST matching row's
//!   index; the rows are committed in the audited tailwindcss v4
//!   utilities progression the rows module documents);
//! * **the IR's cascade specificity** buckets the variants after the
//!   base ([`aeolus_ir::Cond::specificity`]: base < mode < probe states
//!   < parent-relative < media < container) — the same rank the
//!   pipeline sorts layers by, so a sorted string reads in the order
//!   the CSS emit arm renders rules;
//! * **registered customs** ride after every registry row, in
//!   registration order ([`crate::Customs`] via `sort_with`) — the
//!   registry wins a name collision by construction (its read side
//!   answers first);
//! * **everything the seam cannot order** — a lex failure, a variant
//!   outside the typed `Cond` surface, a registry/custom miss — rides
//!   the tail in source order and reports the SAME typed, coded
//!   diagnostic the compile walk reports ([`Diagnostic`]): the honest
//!   tail, never a guessed position, never a silent drop.
//!
//! What the walk deliberately does NOT consult: the value pass and the
//! theme pass. The order is a shape-level fact (the row a body
//! addresses); whether the value lowers, whether a token resolves, and
//! whether a modifier rides are the compile walk's concerns
//! ([`crate::compile`]/[`crate::compile_theme`]) and stay there — a
//! themeless token boundary sorts exactly like its lowered sibling.

use aeolus_data::LookupError;
use aeolus_data::registry::lookup_detailed;
use aeolus_data::registry::row_count;
use aeolus_data::registry::sort_index;
use aeolus_syntax::parse_class_string;

use crate::custom::Customs;
use crate::variant;
use crate::{CompileError, Diagnostic, DiagnosticKind, lookup_diagnostic, variant_diagnostic};

/// The order key one class sorts by: the variant bucket (the folded
/// [`Cond`](aeolus_ir::Cond)'s cascade specificity, `0` = base) and the
/// registry row position (`row_count()` + the registration slot for a
/// custom utility).
///
/// [`SortKey::UNSORTABLE`] marks the honest tail — a class the seam
/// cannot order. Tail classes never reorder among themselves (the sort
/// is stable over equal keys).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SortKey {
    /// The variant bucket: the folded condition's cascade specificity
    /// (base 0 < dark 1 < probe states 2 < parent-relative 3 < media 4
    /// < container 5 — the IR's own rank).
    pub bucket: u8,
    /// The registry row position (the FIRST matching row's index);
    /// `row_count() + slot` for a registered custom utility.
    pub row: usize,
}

impl SortKey {
    /// The unsortable marker: the key rides last (the sort is stable,
    /// so the tail keeps its source order).
    pub const UNSORTABLE: Self = Self {
        bucket: u8::MAX,
        row: usize::MAX,
    };
}

/// One class of the input, with its order key. `known` marks a class
/// the registry (or a registration) could order; the tail classes
/// (`known == false`, [`SortKey::UNSORTABLE`]) carry the same span and
/// raw text they had in the input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SortedClass {
    /// The class text, verbatim.
    pub raw: String,
    /// The class's byte range inside the input string.
    pub span: (usize, usize),
    /// The order key the sort applied ([`SortKey::UNSORTABLE`] for the
    /// honest tail).
    pub key: SortKey,
    /// Whether a registry row (or a registration) could order this
    /// class. `false` members are exactly the ones
    /// [`Sorted::diagnostics`] reports.
    pub known: bool,
}

/// The sort result: the classes in SORTED order plus the honest tail's
/// typed reports.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sorted {
    /// Every class of the input, in sorted order: known classes first
    /// (registry progression, variant buckets by cascade specificity,
    /// stable on equal keys), the unsortable tail after (source order).
    pub classes: Vec<SortedClass>,
    /// One diagnostic per class that could not be ORDERED — lex
    /// failures, unsupported variants, registry/custom misses — in
    /// source order. The same [`DiagnosticKind`] set the compile walk
    /// reports, with the same spans and codes.
    pub diagnostics: Vec<Diagnostic>,
}

impl Sorted {
    /// The sorted class string: the classes joined by single spaces
    /// (the input's whitespace run does not survive the join — the
    /// canonical form a formatter emits).
    pub fn to_class_string(&self) -> String {
        let mut out =
            String::with_capacity(self.classes.iter().map(|c| c.raw.len() + 1).sum::<usize>());
        for (index, class) in self.classes.iter().enumerate() {
            if index > 0 {
                out.push(' ');
            }
            out.push_str(&class.raw);
        }
        out
    }
}

/// Sort a class string into the registry's official order (plan §5.5):
/// base classes first by the rows' canonical progression, then the
/// variant buckets by the IR's cascade specificity, the unsortable
/// classes riding the tail with their typed diagnostics.
///
/// ```no_run
/// use aeolus_compile::sort;
///
/// let sorted = sort("hover:block block p-4")?;
/// assert_eq!(sorted.to_class_string(), "p-4 block hover:block");
/// assert!(sorted.diagnostics.is_empty());
/// # Ok::<(), aeolus_compile::CompileError>(())
/// ```
///
/// # Errors
/// [`CompileError::Empty`] for an empty class string (the same contract
/// as [`crate::compile`]). Per-class misses ride
/// [`Sorted::diagnostics`] — never a failure, never a silent drop.
pub fn sort(input: &str) -> Result<Sorted, CompileError> {
    sort_with(input, &Customs::new())
}

/// [`sort`] with the custom-authoring seam supplied: registered custom
/// utilities order after every registry row, in registration order
/// (the registry wins a name collision — its read side answers first).
///
/// With EMPTY registrations every contract is byte-identical to
/// [`sort`]: the seam is additive, never a behaviour switch.
///
/// ```no_run
/// use aeolus_compile::{CustomForm, CustomUtility, Customs, sort_with};
/// use aeolus_ir::Prop;
///
/// let customs = Customs::new().utility(CustomUtility::functional(
///     "brand-p",
///     CustomForm::Spacing,
///     &[Prop::PaddingTop, Prop::PaddingRight, Prop::PaddingBottom, Prop::PaddingLeft],
/// ));
/// let sorted = sort_with("brand-p-2 p-4", &customs)?;
/// assert_eq!(sorted.to_class_string(), "p-4 brand-p-2");
/// # Ok::<(), aeolus_compile::CompileError>(())
/// ```
///
/// # Errors
/// The same [`CompileError`] as [`sort`].
pub fn sort_with(input: &str, customs: &Customs) -> Result<Sorted, CompileError> {
    #[cfg(feature = "tracing")]
    tracing::debug!(
        utilities = customs.utilities().len(),
        variants = customs.variants().len(),
        "sort_with: the custom seam is supplied"
    );
    if input.trim().is_empty() {
        return Err(CompileError::Empty);
    }

    // Stage 1: the zero-copy lexer, byte-shared with the compile walk.
    // A token the lexer refuses cannot even be READ — it joins the
    // tail with its own span and the typed Lex report.
    let (candidates, degraded) = parse_class_string(input);
    let mut classes: Vec<SortedClass> = Vec::with_capacity(candidates.len() + degraded.len());
    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    for failure in &degraded {
        diagnostics.push(Diagnostic {
            span: failure.span,
            raw: failure.raw.to_string(),
            kind: DiagnosticKind::Lex { kind: failure.kind },
        });
        classes.push(SortedClass {
            raw: failure.raw.to_string(),
            span: failure.span,
            key: SortKey::UNSORTABLE,
            known: false,
        });
    }

    for candidate in &candidates {
        // Stage 2: the variant fold — the bucket source. A segment
        // outside the typed Cond surface cannot be BUCKETED (the tail).
        let cond = match variant::fold(&candidate.variants, customs) {
            Ok(cond) => cond,
            Err(miss) => {
                diagnostics.push(variant_diagnostic(candidate, &miss));
                classes.push(SortedClass {
                    raw: candidate.raw.to_string(),
                    span: candidate.span,
                    key: SortKey::UNSORTABLE,
                    known: false,
                });
                continue;
            }
        };
        // Stage 3: the order position — the registry's read side first,
        // the registrations on the miss (the compile walk's own
        // precedence; the registry wins a name collision by
        // construction). The custom shape rejection names the custom
        // root — the more specific report — while a double miss keeps
        // the REGISTRY's own typed error (the unknown-root vs
        // shape-rejected mode lookup_detailed reports).
        let resolved: Result<SortKey, LookupError> = match sort_index(candidate.body) {
            Some(row) => Ok(SortKey {
                bucket: cond.specificity(),
                row,
            }),
            None => {
                // The registry's typed miss (sort_index already said
                // no accepted row, so the detailed read reports the
                // mode; the unwrap_or is the defensive-totality
                // spelling for a probe that cannot succeed).
                let builtin =
                    lookup_detailed(candidate.body)
                        .err()
                        .unwrap_or(LookupError::UnknownRoot {
                            body: candidate.body.to_string(),
                        });
                match customs.lookup(candidate.body) {
                    Err(custom_err) => Err(match custom_err {
                        LookupError::ShapeRejected { .. } => custom_err,
                        LookupError::UnknownRoot { .. } => builtin,
                    }),
                    Ok(utilities) => {
                        // The FIRST matched registration's order position —
                        // lookup collects only utilities from the
                        // registration slice, so the position exists; a
                        // defensive miss falls back to the honest tail.
                        let slot = utilities
                            .iter()
                            .filter_map(|utility| {
                                customs
                                    .utilities()
                                    .iter()
                                    .position(|registered| registered == utility)
                            })
                            .min();
                        match slot {
                            Some(slot) => Ok(SortKey {
                                bucket: cond.specificity(),
                                row: row_count() + slot,
                            }),
                            None => Err(builtin),
                        }
                    }
                }
            }
        };
        match resolved {
            Ok(key) => classes.push(SortedClass {
                raw: candidate.raw.to_string(),
                span: candidate.span,
                key,
                known: true,
            }),
            Err(err) => {
                diagnostics.push(lookup_diagnostic(candidate, err));
                classes.push(SortedClass {
                    raw: candidate.raw.to_string(),
                    span: candidate.span,
                    key: SortKey::UNSORTABLE,
                    known: false,
                });
            }
        }
    }

    // The assembly: a STABLE key sort — equal keys keep source order
    // (the merge is cn()'s business; the sort never reorders equals),
    // the unsortable tail rides after every known key in source order.
    classes.sort_by_key(|class| class.key);
    #[cfg(feature = "tracing")]
    tracing::debug!(
        classes = classes.len(),
        diagnostics = diagnostics.len(),
        "sorted"
    );
    Ok(Sorted {
        classes,
        diagnostics,
    })
}
