//! The Rust-tokens emit arm (architecture plan §3.1 emit (a) — the LAST
//! arm of the [`Target`](crate::Target) surface): class strings in, Rust
//! token fragments out — one
//! [`Layer::new`](aeolus_ir::Layer::new) expression per lowered class,
//! embeddable as-is and assembling into the IR's own program shape.
//!
//! ```text
//! emit_rust(input, &Config)                    -> Result<RustSheet,  CompileError>
//! emit_rust_theme(input, &TokenTable, &Config) -> Result<RustSheet,  CompileError>
//!
//! RustSheet ─ Display → one compilable `static PROGRAM: &[aeolus_ir::Layer]`
//!             item, one layer per fragment, specificity-sorted
//! RustSheet.fragments ─ one RustFragment per lowered class, source order
//! ```
//!
//! The emit runs IN the pipeline, not over the [`Compiled`](crate::Compiled)
//! artifact — the load-bearing design choice, and the same one the CSS
//! arm documented: the IR's `Val::Kw` slot indexes the ROW'S OWN keyword
//! table (the codegen crate owns the names, plan §3.3), so a bare
//! `Program` cannot recover the text `flex-col` lowered to. The
//! static-data arm carries the slot alone (honest, but slot-only); the
//! interpreter passes keywords through (the codegen's business). THIS
//! arm renders where the rows still talk:
//!
//! * keyword/bare/composite slots render their ROW text as a trailing
//!   comment (`Val::Kw(0) }, // column`) — the names the future
//!   `tw!`-style macro and the P3 codegen crates key on, read from the
//!   row tables at the lowering site (the `lower` module's annotation
//!   pass, arm-selection identical to the value pass);
//! * runtime token slots render their TABLE name when a `TokenTable`
//!   rides (`Val::Tok { id: 7, alpha: 0.5 } }, // --color-primary`) —
//!   the id→name answer only the table gives; the baked dark pair has
//!   no name table and rides verbatim, honestly unannotated;
//! * every other value form renders the static arm's constructor text
//!   (one float contract, two consumers — the emitters cannot drift);
//! * conditions render the flat-IR fold's constructor
//!   (`aeolus_ir::Cond::Hover`, `aeolus_ir::Cond::Bp(Sm)`) — the SAME
//!   fold [`compile`](crate::compile) applies, not the CSS arm's
//!   conjunctive selector render, because the artifact is the IR's own
//!   token shape.
//!
//! Everything a class does NOT render to comes back in
//! [`RustSheet::diagnostics`] — the same spanned, coded, helped reports
//! [`compile`](crate::compile) produces (the walks share the typed
//! lexer channel, the variant refusal set, the registry read side and
//! the value pass's boundary set; `tests/rust_tokens.rs` pins the
//! diagnostics EQUAL over a battery).
//!
//! The fragment unit is per-CANDIDATE (one class, one fragment, source
//! order) — the `tw!`-style macro's consumption shape — while the
//! `Display` assembles the whole sheet into the static item the
//! static-data arm names, layers specificity-sorted (stable, source
//! order breaking ties), so the rendered text reconstructs a sorted
//! program whose masks [`Layer::new`](aeolus_ir::Layer::new) derives.
//!
//! Observability follows the crate's opt-in instrumentation seam: the
//! `tracing` feature spans the walk and logs the fragment count (no
//! subscriber is installed here — the host app owns one).
//!
//! # The emitted shape, compiled
//!
//! The Display output IS the static form — this doc test compiles and
//! RUNS it, asserting the item reconstructs a sorted program whose
//! masks the [`Layer::new`](aeolus_ir::Layer::new) derivation filled
//! in:
//!
//! ```rust
//! use aeolus_ir::{Layer, Program};
//!
//! // `emit_rust("p-4", &Config::default())`'s Display output, verbatim
//! // (the shape tests/rust_tokens.rs pins byte for byte):
//! static PROGRAM: &[aeolus_ir::Layer] = &[
//!     // p-4
//!     aeolus_ir::Layer::new(
//!         aeolus_ir::Cond::Base,
//!         &[
//!             aeolus_ir::Decl { prop: aeolus_ir::Prop::PaddingTop, val: aeolus_ir::Val::Rem(1.0) },
//!             aeolus_ir::Decl { prop: aeolus_ir::Prop::PaddingRight, val: aeolus_ir::Val::Rem(1.0) },
//!             aeolus_ir::Decl { prop: aeolus_ir::Prop::PaddingBottom, val: aeolus_ir::Val::Rem(1.0) },
//!             aeolus_ir::Decl { prop: aeolus_ir::Prop::PaddingLeft, val: aeolus_ir::Val::Rem(1.0) },
//!         ],
//!     ),
//! ];
//!
//! let program = Program::new(PROGRAM);
//! assert!(program.is_sorted(), "the Display is specificity-sorted");
//! assert_eq!(program.layers[0].mask.len(), 4, "Layer::new derived the mask");
//! ```
//!
//! What this arm deliberately is NOT: the BACKEND method calls (the
//! plan §3.1 emit (a) end form — `FlexDirection::Column`, builder
//! chains — are the P3 codegen crate's surface, per-backend) and
//! the xtask bake that commits this output as source. The
//! has-/data-/aria- variant folding landed 2026-11: the probe conds
//! render through this arm's own `Debug`-driven cond text (the
//! statics arm's name-table contract).

use std::fmt;

use aeolus_data::LookupError;
use aeolus_data::registry::lookup_detailed;
use aeolus_ir::{Cond, Decl, Prop, Val};
use aeolus_syntax::parse_class_string;
use aeolus_theme::TokenTable;

use crate::custom;
use crate::variant;
use crate::{
    CompileError, Config, Customs, Diagnostic, DiagnosticKind, lookup_diagnostic,
    normalize_modifier, reject_diagnostic, variant_diagnostic,
};

// ---------------------------------------------------------------------------
// The artifact: RustFragment, RustSheet
// ---------------------------------------------------------------------------

/// One lowered class's Rust token fragment — the emit (a) form's unit:
/// the class text, the folded condition, and the
/// [`Layer::new`](aeolus_ir::Layer::new) expression the class lowers to,
/// keyword and token slots annotated with the names only the pipeline
/// can see.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RustFragment {
    /// The candidate's class text, verbatim (the fragment's identity in
    /// the sheet — one lowered class, one fragment).
    pub class: String,
    /// The folded condition (the flat-IR model): the typed handle the
    /// Display sorts the layers by and the consumer probes.
    pub cond: Cond,
    /// The Rust token text: one complete, embeddable
    /// `aeolus_ir::Layer::new(...)` expression — the declarations in
    /// cascade order, the fully-qualified `aeolus_ir::` paths, the
    /// keyword/token annotations as trailing comments.
    pub tokens: String,
}

/// The emit artifact (the [`Target::Rust`](crate::Target::Rust) form of
/// one class string): one fragment per lowered candidate in source
/// order, plus the SAME honest per-class diagnostics the IR form
/// reports. `Display` renders the compilable `static` item — one layer
/// per fragment, specificity-sorted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RustSheet {
    /// One fragment per lowered candidate, source order.
    pub fragments: Vec<RustFragment>,
    /// One entry per class that did NOT render, in source order — the
    /// same reports [`compile`](crate::compile) produces.
    pub diagnostics: Vec<Diagnostic>,
}

impl fmt::Display for RustSheet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("static PROGRAM: &[aeolus_ir::Layer] = &[\n")?;
        // The static item renders the layers SPECIFICITY-SORTED (stable,
        // source order breaking ties) — the same assembly order the IR
        // walk applies, so the Display reconstructs a sorted program.
        let mut order: Vec<&RustFragment> = self.fragments.iter().collect();
        order.sort_by_key(|fragment| fragment.cond.specificity());
        for fragment in order {
            let _ = writeln!(f, "    // {}", fragment.class);
            let lines: Vec<&str> = fragment.tokens.lines().collect();
            let last = lines.len().saturating_sub(1);
            for (i, line) in lines.iter().enumerate() {
                if i == last {
                    // The expression's closing paren gains the item's
                    // element comma (the fragment itself stays a pure,
                    // pasteable expression).
                    let _ = writeln!(f, "    {line},");
                } else {
                    let _ = writeln!(f, "    {line}");
                }
            }
        }
        f.write_str("];\n")
    }
}

// ---------------------------------------------------------------------------
// The entry points (the emit_css family's shape, emit (a))
// ---------------------------------------------------------------------------

/// Render the [`Target::Rust`](crate::Target::Rust) form of a class
/// string WITHOUT the theme pass: colour tokens and opacity modifiers
/// stay typed boundaries (their diagnostics name [`emit_rust_theme`] as
/// the lowering path) — the same honest default [`compile`](crate::compile)
/// has.
///
/// ```no_run
/// use aeolus_compile::{emit_rust, Config};
///
/// let sheet = emit_rust("p-4 flex-col hover:bg-primary/90", &Config::default())?;
/// // One fragment per lowered class — the token text embeds as-is:
/// for fragment in &sheet.fragments {
///     println!("// {}\n{}", fragment.class, fragment.tokens);
/// }
/// print!("{sheet}"); // the assembled static item
/// // Every class the emitter does not render comes back with a span,
/// // a stable code and help text — never a silent drop.
/// for diagnostic in &sheet.diagnostics {
///     println!("{diagnostic} [{}]", diagnostic.kind.code());
/// }
/// # Ok::<(), aeolus_compile::CompileError>(())
/// ```
///
/// # Errors
/// The same [`CompileError`] as [`compile`](crate::compile) (only the
/// empty input). Per-class reports never fail an emit.
pub fn emit_rust(input: &str, config: &Config) -> Result<RustSheet, CompileError> {
    run_rust(input, *config, None, &Customs::new())
}

/// [`emit_rust`] with the custom-authoring seam supplied (plan §5.4's
/// `@utility`/`@custom-variant` surface): registered custom utilities
/// render their fragments through the same value pass and registered
/// variant aliases fold like built-in spellings — the same diagnostics,
/// the same spans, the same fragment ordering. With EMPTY registrations
/// every contract is byte-identical to [`emit_rust`] (the differential
/// pins it): the seam is additive, never a behaviour switch.
///
/// ```no_run
/// use aeolus_compile::{emit_rust_with, Config, CustomForm, CustomUtility, Customs};
/// use aeolus_ir::Prop;
///
/// let customs = Customs::new().utility(CustomUtility::functional(
///     "brand-p",
///     CustomForm::Spacing,
///     &[Prop::PaddingTop, Prop::PaddingRight, Prop::PaddingBottom, Prop::PaddingLeft],
/// ));
/// let sheet = emit_rust_with("brand-p-4", &customs, &Config::default())?;
/// assert!(sheet.diagnostics.is_empty()); // the registration rendered
/// # Ok::<(), aeolus_compile::CompileError>(())
/// ```
///
/// # Errors
/// The same [`CompileError`] as [`emit_rust`] (only the empty input).
pub fn emit_rust_with(
    input: &str,
    customs: &Customs,
    config: &Config,
) -> Result<RustSheet, CompileError> {
    #[cfg(feature = "tracing")]
    tracing::debug!(
        utilities = customs.utilities().len(),
        variants = customs.variants().len(),
        "emit_rust_with: the custom seam is supplied"
    );
    run_rust(input, *config, None, customs)
}

/// Render the [`Target::Rust`](crate::Target::Rust) form of a class
/// string with the theme pass supplied: colour tokens lower through the
/// tiered `TokenTable` and the token slots render their TABLE names as
/// annotations (the id→name answer only the table gives).
///
/// ```no_run
/// use aeolus_compile::{emit_rust_theme, Config};
/// use aeolus_theme::{ThemeSource, TokenTable};
///
/// let source = ThemeSource::shadcn_dark()?;
/// let tokens = TokenTable::runtime(&source)?;
/// let sheet = emit_rust_theme("bg-primary/90 hover:bg-primary", &tokens, &Config::default())?;
/// assert!(sheet.diagnostics.is_empty()); // the tokens rendered
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
/// The same [`CompileError`] as [`compile`](crate::compile) (only the
/// empty input).
pub fn emit_rust_theme(
    input: &str,
    tokens: &TokenTable,
    config: &Config,
) -> Result<RustSheet, CompileError> {
    #[cfg(feature = "tracing")]
    tracing::debug!(
        tokens = tokens.len(),
        tier = ?tokens.tier(),
        "emit_rust_theme: the theme pass is supplied"
    );
    run_rust(input, *config, Some(tokens), &Customs::new())
}

/// [`emit_rust_theme`] with the custom-authoring seam supplied: colour
/// tokens render through the tiered `TokenTable` AND registered custom
/// utilities/variants render through the same pipeline (see
/// [`emit_rust_with`] for the seam's contract).
///
/// ```no_run
/// use aeolus_compile::{emit_rust_theme_with, Config, CustomVariant, Customs};
/// use aeolus_ir::Cond;
/// use aeolus_theme::{ThemeSource, TokenTable};
///
/// let source = ThemeSource::shadcn_dark()?;
/// let tokens = TokenTable::runtime(&source)?;
/// let customs = Customs::new().variant(CustomVariant::new("night", Cond::Dark));
/// let sheet = emit_rust_theme_with("night:bg-primary", &tokens, &customs, &Config::default())?;
/// assert!(sheet.diagnostics.is_empty()); // the alias folded, the token rendered
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
/// The same [`CompileError`] as [`emit_rust`] (only the empty input).
pub fn emit_rust_theme_with(
    input: &str,
    tokens: &TokenTable,
    customs: &Customs,
    config: &Config,
) -> Result<RustSheet, CompileError> {
    #[cfg(feature = "tracing")]
    tracing::debug!(
        tokens = tokens.len(),
        tier = ?tokens.tier(),
        utilities = customs.utilities().len(),
        variants = customs.variants().len(),
        "emit_rust_theme_with: the theme pass and the custom seam are supplied"
    );
    run_rust(input, *config, Some(tokens), customs)
}

// ---------------------------------------------------------------------------
// The walk
// ---------------------------------------------------------------------------

/// The whole emit walk, theme-less or themed. `tokens` selects whether
/// the colour/modifier surfaces render (the theme pass) or stay typed
/// boundaries (the honest default); `customs` is the registered
/// custom-authoring surface (empty registrations change nothing — the
/// differential pins it). Mirrors the IR walk's stages and shares its
/// refusal set; the rust-tokens delta is the ANNOTATION pass (the row
/// tables' texts and the token table's names, read at the lowering
/// site) and the per-candidate fragment render.
pub(crate) fn run_rust(
    input: &str,
    config: Config,
    tokens: Option<&TokenTable>,
    customs: &Customs,
) -> Result<RustSheet, CompileError> {
    #[cfg(feature = "tracing")]
    let _span = tracing::debug_span!("run_rust", input_len = input.len()).entered();
    #[cfg(feature = "tracing")]
    tracing::debug!(
        input_len = input.len(),
        "emit_rust: lex -> lookup -> render"
    );

    if input.trim().is_empty() {
        return Err(CompileError::Empty);
    }

    // Stage 1: the zero-copy lexer (the typed degraded channel — spans
    // ride the failure, no re-derivation). The SAME channel the IR and
    // CSS walks share; the diagnostics-equal battery pins the agreement.
    let (candidates, degraded) = parse_class_string(input);
    #[cfg(feature = "tracing")]
    tracing::debug!(
        tokens = candidates.len(),
        degraded = degraded.len(),
        "lexer done"
    );

    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    for failure in &degraded {
        diagnostics.push(Diagnostic {
            span: failure.span,
            raw: failure.raw.to_string(),
            kind: DiagnosticKind::Lex { kind: failure.kind },
        });
    }

    let mut fragments: Vec<RustFragment> = Vec::new();
    for candidate in &candidates {
        #[cfg(feature = "tracing")]
        tracing::trace!(raw = %candidate.raw, "candidate");

        if candidate.important {
            diagnostics.push(Diagnostic::new(
                candidate.span,
                candidate.raw,
                DiagnosticKind::UnsupportedImportant,
            ));
            continue;
        }
        // The SAME typed fold the IR walk applies (the artifact is the
        // IR's own token shape — not the CSS arm's conjunctive render).
        let cond = match variant::fold(&candidate.variants, customs) {
            Ok(cond) => cond,
            Err(miss) => {
                diagnostics.push(variant_diagnostic(candidate, &miss));
                continue;
            }
        };
        // The fraction re-attachment, byte-identical to the IR walk's
        // (the audited oracle's F1: the raw `/modifier` is the
        // DENOMINATOR for fraction-shaped rows).
        let joined = candidate
            .modifier
            .map(|modifier| format!("{}/{}", candidate.body, modifier));
        let (rows, effective_body, fraction_ride) = match &joined {
            Some(text) => match lookup_detailed(text) {
                Ok(retried) => (Ok(retried), text.as_str(), true),
                Err(_) => (lookup_detailed(candidate.body), candidate.body, false),
            },
            None => (lookup_detailed(candidate.body), candidate.body, false),
        };
        let alpha = if fraction_ride {
            None
        } else {
            match candidate.modifier {
                None => None,
                Some(modifier) => match normalize_modifier(modifier).filter(|_| tokens.is_some()) {
                    Some(alpha) => Some(alpha),
                    None => {
                        diagnostics.push(Diagnostic::new(
                            candidate.span,
                            candidate.raw,
                            DiagnosticKind::UnsupportedModifier {
                                modifier: modifier.to_string(),
                            },
                        ));
                        continue;
                    }
                },
            }
        };
        let mut decls: Vec<Decl> = Vec::new();
        // The keyword annotations: first row wins per longhand (the
        // sibling rows of one root share the slot space only within one
        // row's own fan-out — collisions cannot arise for one suffix).
        let mut annotations: Vec<(Prop, &'static str)> = Vec::new();
        // The arbitrary candidates' deferred boundary set (the IR walk's
        // own tracking — the diagnostics stay EQUAL over the battery):
        // silent sibling-row skips, one deduped report per candidate,
        // the catch-all when no row took the class.
        let mut deferred: Vec<&'static str> = Vec::new();
        let mut reported = false;
        match rows {
            Ok(rows) => {
                for row in rows {
                    match crate::lower::row_to_decls(
                        *row,
                        effective_body,
                        candidate.negative,
                        alpha,
                        tokens,
                        config,
                        candidate.arbitrary,
                    ) {
                        Ok(row_decls) => {
                            // The annotation pass: the row tables' texts,
                            // only for the non-arbitrary lowering (the
                            // arbitrary arm lowers concrete values — no
                            // row-table slots to name).
                            if candidate.arbitrary.is_none() {
                                for (prop, text) in
                                    crate::lower::kw_annotations(*row, effective_body)
                                {
                                    if !annotations.iter().any(|&(seen, _)| seen == prop) {
                                        annotations.push((prop, text));
                                    }
                                }
                            }
                            decls.extend(row_decls);
                        }
                        Err(reject) => {
                            reported = true;
                            if let (true, crate::lower::Reject::Value { reason }) =
                                (candidate.arbitrary.is_some(), &reject)
                                && crate::arbitrary::is_boundary_reason(reason)
                            {
                                if !deferred.contains(reason) {
                                    deferred.push(reason);
                                }
                                continue;
                            }
                            diagnostics.push(reject_diagnostic(
                                candidate,
                                effective_body,
                                reject,
                                candidate.modifier,
                            ));
                        }
                    }
                }
                // The catch-all: an arbitrary candidate whose NO row
                // took (every sibling skipped silently) and that no
                // other report answered still reports — the
                // no-silent-failures rule, applied to the class gate's
                // quiet half. Deferred boundaries render here, deduped
                // (one class, one boundary).
                if candidate.arbitrary.is_some() && decls.is_empty() {
                    if !deferred.is_empty() {
                        for reason in deferred {
                            diagnostics.push(Diagnostic::new(
                                candidate.span,
                                candidate.raw,
                                DiagnosticKind::UnsupportedValue {
                                    body: effective_body.to_string(),
                                    reason,
                                },
                            ));
                        }
                    } else if !reported {
                        diagnostics.push(Diagnostic::new(
                            candidate.span,
                            candidate.raw,
                            DiagnosticKind::UnsupportedValue {
                                body: effective_body.to_string(),
                                reason: crate::arbitrary::REASON_ARBITRARY_UNIT,
                            },
                        ));
                    }
                }
            }
            // The registry is the oracle; the customs are additive
            // surface consulted on the registry's unknown-root miss
            // only — the SAME read side the IR walk consults, the SAME
            // boundary set and the SAME more-specific-miss rule (the
            // differential pins the two walks' diagnostics EQUAL with
            // customs in play). Custom utilities carry their own `Val`s
            // verbatim — no row-table annotations exist for them.
            Err(builtin_err) => match customs.lookup(candidate.body) {
                Ok(utilities) => {
                    for utility in utilities {
                        match custom::utility_to_decls(
                            utility,
                            candidate.body,
                            candidate.negative,
                            alpha,
                            tokens,
                            config,
                        ) {
                            Ok(row_decls) => decls.extend(row_decls),
                            Err(reject) => {
                                diagnostics.push(reject_diagnostic(
                                    candidate,
                                    effective_body,
                                    reject,
                                    candidate.modifier,
                                ));
                            }
                        }
                    }
                }
                Err(custom_err) => {
                    let err = match custom_err {
                        LookupError::ShapeRejected { .. } => custom_err,
                        LookupError::UnknownRoot { .. } => builtin_err,
                    };
                    diagnostics.push(lookup_diagnostic(candidate, err));
                    continue;
                }
            },
        }
        if decls.is_empty() {
            // Every row refused: the reports ride, no fragment renders.
            continue;
        }
        // The cascade order (the same sort the IR walk applies — Prop's
        // discriminant IS the sort order; the stable sort keeps source
        // order within one longhand). The annotations ride the sort by
        // their prop key.
        decls.sort_by_key(|decl: &Decl| decl.prop);
        let token_text = layer_tokens(cond, &decls, &annotations, tokens);
        fragments.push(RustFragment {
            class: candidate.raw.to_string(),
            cond,
            tokens: token_text,
        });
    }

    #[cfg(feature = "tracing")]
    tracing::debug!(
        fragments = fragments.len(),
        diagnostics = diagnostics.len(),
        "rust tokens emitted"
    );
    Ok(RustSheet {
        fragments,
        diagnostics,
    })
}

/// One fragment's token text: the `Layer::new` expression over the
/// cascade-sorted declarations, keyword and token slots annotated with
/// the names only the pipeline can see (the row tables' texts, the
/// token table's names). Total over every `Val` variant — the value
/// text rides the static arm's own `val_literal` (one float contract,
/// two consumers — the emitters cannot drift).
fn layer_tokens(
    cond: Cond,
    decls: &[Decl],
    annotations: &[(Prop, &'static str)],
    tokens: Option<&TokenTable>,
) -> String {
    use std::fmt::Write as _;
    let mut out = String::from("aeolus_ir::Layer::new(\n");
    let _ = writeln!(out, "    aeolus_ir::Cond::{cond:?},");
    out.push_str("    &[\n");
    for decl in decls {
        let _ = write!(
            out,
            "        aeolus_ir::Decl {{ prop: aeolus_ir::Prop::{:?}, val: {} }},",
            decl.prop,
            crate::statics::val_literal(decl.val)
        );
        // The in-pipeline annotations — the two slots whose meaning a
        // bare `Program` cannot recover:
        if let Val::Kw(_) = decl.val
            && let Some(text) = annotation_text(annotations, decl.prop)
        {
            // The `Val::Kw` slot indexes the ROW'S OWN table: the text
            // the suffix lowered to (the static arm carries the slot
            // alone; this arm renders where the rows still talk).
            let _ = write!(out, " // {text}");
        } else if let Val::Tok { id, .. } = decl.val
            && let Some(table) = tokens
            && let Some(name) = table.name(id)
        {
            // The runtime token's id answers through the table (the
            // baked dark pair has no name table and rides verbatim).
            let _ = write!(out, " // {name}");
        }
        out.push('\n');
    }
    out.push_str("    ],\n");
    out.push(')');
    out
}

/// The annotation text for one keyword declaration (first row wins per
/// longhand — the candidate's fan-outs never collide on one prop, but
/// the lookup stays honest anyway).
fn annotation_text(annotations: &[(Prop, &'static str)], prop: Prop) -> Option<&'static str> {
    annotations
        .iter()
        .find(|&&(seen, _)| seen == prop)
        .map(|&(_, text)| text)
}
