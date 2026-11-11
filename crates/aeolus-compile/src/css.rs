//! The CSS emit arm (architecture plan §3.1's emit stage, option (d);
//! the [`Target`](crate::Target) half's first slice) — class strings in,
//! a rendered stylesheet out.
//!
//! The emit runs IN the pipeline, not over the [`Compiled`](crate::Compiled)
//! artifact, and that is the load-bearing design choice: the IR's
//! `Val::Kw` slot indexes the ROW'S OWN keyword table (the codegen crate
//! owns the names, plan §3.3), so a bare `Program` cannot recover the
//! text `flex-col` lowered to (`column`). Rendering at the lowering site
//! keeps the row's tables, the candidate's variant SEGMENTS and the
//! theme's tier resolution in hand — every form renders from its own
//! source of truth:
//!
//! * keyword/bare/composite values render the row table's CSS strings
//!   verbatim (`flex-col` → `column`, `text-sm` → `0.875rem` +
//!   `1.25rem`);
//! * spacing/fraction/percent render the same math the IR form lowers
//!   (`p-4` → `1rem`, `w-1/2` → `50%`);
//! * `Number` rows render through the per-root [`lower::NUMBER_FORMS`]
//!   table (units, the `repeat()`/`span()` grid composites, the
//!   `stroke-width` override) — registry drift is a test failure
//!   (`tests/css.rs` walks the rows), never a guessed unit;
//! * colour tokens render per theme tier: baked literals as
//!   `rgb(r g b / a%)`, runtime tokens as `var(--color-…)` with the
//!   `/opacity` modifier as the plan §2 `color-mix` identity, baked
//!   dark pairs as TWO rules (light under the rule's own selector, dark
//!   under `:where(.dark, .dark *)`);
//! * conditions render from the candidate's segments — every probe
//!   state and the dark tier stack conjunctively into the selector
//!   (`.hover\:p-4:hover`, `.dark\:p-4:where(.dark, .dark *)`), one
//!   parent/peer-relative probe prefixes the combinator form
//!   (`.group/menu:hover .group-hover\/menu\:p-4`), and one `@media`/
//!   `@container` rule wraps the block. This is MORE faithful than the
//!   IR's flat `Cond` fold (which keeps the dominant condition only);
//!   the differential is the IR's documented model, not an emitter bug.
//!
//! Everything a class does NOT render to comes back in
//! [`CssSheet::diagnostics`] — the same spanned, coded, helped reports
//! [`compile`](crate::compile) produces (the two walks share the typed
//! lexer channel, the variant refusal set, the registry read side and
//! the value pass's typed rejection boundaries).

use std::fmt;
use std::fmt::Write as _;

use aeolus_data::LookupError;
use aeolus_data::registry::lookup_detailed;
use aeolus_ir::Bp;
use aeolus_syntax::parse_class_string;
use aeolus_theme::TokenTable;

use crate::custom::{self, ClassifyVariant};
use crate::lower;
use crate::variant::{self, VariantMiss};
use crate::{
    CompileError, Config, Customs, Diagnostic, DiagnosticKind, lookup_diagnostic,
    normalize_modifier, reject_diagnostic, variant_diagnostic,
};

/// One @-rule condition a rule can sit under. Slice 1 carries ONE per
/// rule (the last responsive/container segment wins — the flat model's
/// own answer); stacked @-conditions are the P3 codegen's nesting story.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CssAt {
    /// `@media (min-width: Nrem)` — `sm:`/`md:`/… and the explicit
    /// `min-md:` form.
    MediaMin(Bp),
    /// `@media (width < Nrem)` — the `max-md:` form (v4's range syntax).
    MediaMax(Bp),
    /// `@container (min-width: Nrem)` — `@md:`.
    ContainerMin(Bp),
    /// `@container (width < Nrem)` — `@max-md:`.
    ContainerMax(Bp),
}

impl CssAt {
    /// The at-rule prelude text (`@media (min-width: 48rem)`).
    pub fn text(self) -> String {
        match self {
            CssAt::MediaMin(bp) => format!("@media (min-width: {}rem)", bp.rem()),
            CssAt::MediaMax(bp) => format!("@media (width < {}rem)", bp.rem()),
            CssAt::ContainerMin(bp) => format!("@container (min-width: {}rem)", bp.rem()),
            CssAt::ContainerMax(bp) => format!("@container (width < {}rem)", bp.rem()),
        }
    }
}

/// One rendered rule: the selector (escaped class text + condition
/// suffixes, the parent/peer combinator prefix when a group/peer probe
/// rode along), the optional at-rule it sits under, and the declaration
/// block (cascade order, `prop: value;` lines, newline-joined).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CssRule {
    /// The rule's selector, escaped (`md\:p-6`).
    pub selector: String,
    /// The `@media`/`@container` prelude, when a responsive/container
    /// variant rode the candidate.
    pub media: Option<CssAt>,
    /// The declaration lines (`padding-top: 1rem`), cascade-sorted.
    pub body: String,
}

/// The emit artifact (the `Target::Css` form of one class string): one
/// rule per lowered candidate in source order, plus the SAME honest
/// per-class diagnostics the IR form reports. `Display` renders the
/// stylesheet — consecutive rules sharing one at-rule group into a
/// single `@media`/`@container` block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CssSheet {
    /// One rule per lowered candidate, source order.
    pub rules: Vec<CssRule>,
    /// One entry per class that did NOT render, in source order — the
    /// same reports [`compile`](crate::compile) produces.
    pub diagnostics: Vec<Diagnostic>,
}

impl fmt::Display for CssSheet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut i = 0;
        while i < self.rules.len() {
            match self.rules[i].media {
                None => {
                    write_rule(f, &self.rules[i], "")?;
                    i += 1;
                }
                Some(at) => {
                    writeln!(f, "{} {{", at.text())?;
                    let media = self.rules[i].media;
                    while i < self.rules.len() && self.rules[i].media == media {
                        write_rule(f, &self.rules[i], "  ")?;
                        i += 1;
                    }
                    writeln!(f, "}}")?;
                }
            }
        }
        Ok(())
    }
}

fn write_rule(f: &mut fmt::Formatter<'_>, rule: &CssRule, indent: &str) -> fmt::Result {
    writeln!(f, "{indent}{} {{", rule.selector)?;
    for line in rule.body.lines() {
        writeln!(f, "{indent}  {line}")?;
    }
    writeln!(f, "{indent}}}")
}

/// Render the `Target::Css` form of a class string WITHOUT the theme
/// pass: colour tokens and opacity modifiers stay typed boundaries
/// (their diagnostics name [`emit_css_theme`] as the lowering path) —
/// the same honest default [`compile`](crate::compile) has.
///
/// ```no_run
/// use aeolus_compile::{emit_css, Config};
///
/// let sheet = emit_css("p-4 hover:bg-primary/90 md:hidden", &Config::default())?;
/// print!("{sheet}"); // the rendered stylesheet
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
pub fn emit_css(input: &str, config: &Config) -> Result<CssSheet, CompileError> {
    run_css(input, *config, None, &Customs::new())
}

/// [`emit_css`] with the custom-authoring seam supplied (plan §5.4's
/// `@utility`/`@custom-variant` surface): registered custom utilities
/// render their rules through the same value-text forms and registered
/// variant aliases render their condition's selector shape — the same
/// diagnostics, the same spans, the same rule ordering.
///
/// ```no_run
/// use aeolus_compile::{emit_css_with, Config, CustomForm, CustomUtility, Customs};
/// use aeolus_ir::Prop;
///
/// let customs = Customs::new().utility(CustomUtility::functional(
///     "brand-p",
///     CustomForm::Spacing,
///     &[Prop::PaddingTop, Prop::PaddingRight, Prop::PaddingBottom, Prop::PaddingLeft],
/// ));
/// let sheet = emit_css_with("brand-p-4", &customs, &Config::default())?;
/// assert!(sheet.diagnostics.is_empty()); // the registration rendered
/// # Ok::<(), aeolus_compile::CompileError>(())
/// ```
///
/// # Errors
/// The same [`CompileError`] as [`emit_css`] (only the empty input).
pub fn emit_css_with(
    input: &str,
    customs: &Customs,
    config: &Config,
) -> Result<CssSheet, CompileError> {
    #[cfg(feature = "tracing")]
    tracing::debug!(
        utilities = customs.utilities().len(),
        variants = customs.variants().len(),
        "emit_css_with: the custom seam is supplied"
    );
    run_css(input, *config, None, customs)
}

/// Render the `Target::Css` form of a class string with the theme pass
/// supplied: colour tokens lower through the tiered `TokenTable` (the
/// baked literal, the `var(--color-…)` reference, or the plan §2
/// `color-mix` identity under an `/opacity` modifier), and a baked dark
/// pair renders as two rules — light under the rule's own selector, dark
/// under `:where(.dark, .dark *)`.
///
/// ```no_run
/// use aeolus_compile::{emit_css_theme, Config};
/// use aeolus_theme::{ThemeSource, TokenTable};
///
/// let source = ThemeSource::shadcn_dark()?;
/// let tokens = TokenTable::runtime(&source)?;
/// let sheet = emit_css_theme("bg-primary/90 hover:bg-primary", &tokens, &Config::default())?;
/// assert!(sheet.diagnostics.is_empty()); // the tokens rendered
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
/// The same [`CompileError`] as [`compile`](crate::compile) (only the
/// empty input).
pub fn emit_css_theme(
    input: &str,
    tokens: &TokenTable,
    config: &Config,
) -> Result<CssSheet, CompileError> {
    #[cfg(feature = "tracing")]
    tracing::debug!(
        tokens = tokens.len(),
        tier = ?tokens.tier(),
        "emit_css_theme: the theme pass is supplied"
    );
    run_css(input, *config, Some(tokens), &Customs::new())
}

/// [`emit_css_theme`] with the custom-authoring seam supplied: colour
/// tokens render through the tiered `TokenTable` AND registered custom
/// utilities/variants render through the same pipeline (see
/// [`emit_css_with`] for the seam's contract).
///
/// ```no_run
/// use aeolus_compile::{emit_css_theme_with, Config, CustomVariant, Customs};
/// use aeolus_ir::Cond;
/// use aeolus_theme::{ThemeSource, TokenTable};
///
/// let source = ThemeSource::shadcn_dark()?;
/// let tokens = TokenTable::runtime(&source)?;
/// let customs = Customs::new().variant(CustomVariant::new("night", Cond::Dark));
/// let sheet = emit_css_theme_with("night:bg-primary", &tokens, &customs, &Config::default())?;
/// assert!(sheet.diagnostics.is_empty()); // the alias and the token rendered
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
/// The same [`CompileError`] as [`emit_css`] (only the empty input).
pub fn emit_css_theme_with(
    input: &str,
    tokens: &TokenTable,
    customs: &Customs,
    config: &Config,
) -> Result<CssSheet, CompileError> {
    #[cfg(feature = "tracing")]
    tracing::debug!(
        tokens = tokens.len(),
        tier = ?tokens.tier(),
        utilities = customs.utilities().len(),
        variants = customs.variants().len(),
        "emit_css_theme_with: the theme pass and the custom seam are supplied"
    );
    run_css(input, *config, Some(tokens), customs)
}

/// The whole emit walk, theme-less or themed. `tokens` selects whether
/// the colour/modifier surfaces render (the theme pass) or stay typed
/// boundaries (the honest default); `customs` is the registered
/// custom-authoring surface (empty registrations change nothing — the
/// differential pins it). Mirrors the IR walk's stages; the two walks
/// share the lexer channel, the variant refusal set (built-in AND
/// registered), the registry read side, the custom read side and the
/// value pass's boundary set, and `tests/css.rs`/`tests/custom.rs` pin
/// their diagnostics EQUAL over batteries of strings.
pub(crate) fn run_css(
    input: &str,
    config: Config,
    tokens: Option<&TokenTable>,
    customs: &Customs,
) -> Result<CssSheet, CompileError> {
    #[cfg(feature = "tracing")]
    tracing::debug!(input_len = input.len(), "emit_css: lex -> lookup -> render");

    if input.trim().is_empty() {
        return Err(CompileError::Empty);
    }

    // Stage 1: the zero-copy lexer (the typed degraded channel — spans
    // ride the failure, no re-derivation).
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

    let mut rules: Vec<CssRule> = Vec::new();
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
        let set = match classify_segments(&candidate.variants, customs) {
            Ok(set) => set,
            Err(missed) => {
                diagnostics.push(variant_diagnostic(candidate, &VariantMiss::new(&missed)));
                continue;
            }
        };
        // The fraction re-attachment, byte-identical to the IR walk's
        // (the audited oracle's F1: the raw `/modifier` is the
        // DENOMINATOR for fraction-shaped rows). The two walks share
        // the read side, the gate order and the refusal set — the
        // diagnostics-equal batteries pin the agreement.
        let joined = candidate
            .modifier
            .map(|modifier| format!("{}/{}", candidate.body, modifier));
        let (rows, effective_body, fraction_ride) = match &joined {
            // The joined body reads FIRST (the sizing family's own
            // precedence — a bare `w-1` also reads as spacing, so the
            // denominator reading must win the arm choice when a raw
            // modifier rides); a joined miss falls back to the lexed
            // body and the modifier goes through the opacity gate.
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
        let mut decls: Vec<lower::CssDecl> = Vec::new();
        // The arbitrary candidates' deferred boundary set (the IR walk's
        // own tracking — the diagnostics stay EQUAL over the battery):
        // silent sibling-row skips, one deduped report per candidate,
        // the catch-all when no row took the class.
        let mut deferred: Vec<&'static str> = Vec::new();
        let mut reported = false;
        match rows {
            Ok(rows) => {
                for row in rows {
                    match lower::row_to_css(
                        *row,
                        effective_body,
                        candidate.negative,
                        alpha,
                        tokens,
                        config,
                        candidate.arbitrary,
                    ) {
                        Ok(mut rendered) => decls.append(&mut rendered),
                        Err(reject) => {
                            reported = true;
                            if let (true, lower::Reject::Value { reason }) =
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
            // customs in play).
            Err(builtin_err) => match customs.lookup(candidate.body) {
                Ok(utilities) => {
                    for utility in utilities {
                        match custom::utility_to_css(
                            utility,
                            candidate.body,
                            candidate.negative,
                            alpha,
                            tokens,
                            config,
                        ) {
                            Ok(mut rendered) => decls.append(&mut rendered),
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
            // Every row refused: the reports ride, no rule renders.
            continue;
        }
        decls.sort_by_key(|decl| decl.prop);
        push_rules(&mut rules, candidate.raw, &set, &decls);
    }

    #[cfg(feature = "tracing")]
    tracing::debug!(
        rules = rules.len(),
        diagnostics = diagnostics.len(),
        "css emitted"
    );
    Ok(CssSheet { rules, diagnostics })
}

/// The classified condition set of one candidate — slice 1's conjunctive
/// render: every probe state and the dark tier stack into the selector,
/// one parent/peer probe prefixes the combinator form, one @-rule wraps
/// the block. The refusal set is IDENTICAL to the IR fold's (anything
/// `variant::classify` cannot type refuses; the tests pin the
/// classification agreement).
#[derive(Default)]
struct CondSet {
    /// The probe-state suffixes, source order (`:hover`, `::selection`).
    pseudos: Vec<&'static str>,
    /// The attribute probes and `:has()` fragments, source order
    /// (`[data-state="open"]`, `:has(*:focus)`) — the attribute-probe
    /// families' selector text, derived from the folded [`Cond`]
    /// payloads (one grammar with the fold, two consumers).
    probes: Vec<String>,
    /// The `dark:` tier (the `:where(.dark, .dark *)` suffix).
    dark: bool,
    /// The parent/peer head, unescaped (`.group/menu:hover`) and whether
    /// it combines as a SIBLING (`.peer:… ~`) or an ancestor (`.group:… `).
    parent: Option<String>,
    /// Whether `parent` is PRE-RENDERED — the class half already
    /// escaped, the probe half raw. The has-probe heads
    /// (`.group:has(*:focus)`) carry colons inside `:has(...)` that the
    /// default head's `rsplit_once(':')` class/probe split cannot see
    /// past, so those heads render themselves.
    parent_prerendered: bool,
    peer: bool,
    /// The one @-rule the rule sits under.
    at: Option<CssAt>,
}

/// Classify the variant segments into the selector shape. The FIRST
/// segment outside the typed surface (built-in table AND registered
/// aliases) refuses — the same typed miss the IR fold reports, with
/// the segment text for the spanned diagnostic. A registered alias
/// renders its condition's own selector shape (`apply_cond_shape`).
fn classify_segments(variants: &[&str], customs: &Customs) -> Result<CondSet, String> {
    let mut set = CondSet::default();
    for &seg in variants {
        // The refusal agreement: the IR fold's typed surface decides
        // what a segment IS; the css render only chooses HOW it shows.
        let builtin = variant::classify_builtin(seg);
        if builtin.is_none() {
            match customs.classify(seg) {
                Some(cond) => {
                    apply_cond_shape(&mut set, cond);
                    continue;
                }
                None => return Err(seg.to_string()),
            }
        }
        // The attribute probes render their selector fragments straight
        // from the folded cond (the same payload text the fold's own
        // grammar produced — the render cannot drift the fold).
        if let Some(cond) = builtin
            && matches!(
                cond,
                aeolus_ir::Cond::Data { .. }
                    | aeolus_ir::Cond::Aria { .. }
                    | aeolus_ir::Cond::Has(_)
            )
        {
            apply_cond_shape(&mut set, cond);
            continue;
        }
        // The named-group/peer label (`group-hover/menu`): the empty
        // string is the unnamed form (`split_once` gives no arm for
        // the missing half; `is_empty` is the label test).
        let (name, label) = seg.split_once('/').unwrap_or((seg, ""));
        // The parent-relative has-probes keep the label-aware render:
        // the head is `.group[/label]:has(...)`, pre-rendered (the
        // fragment's colons ride inside `:has(...)`), and the inner
        // reads through the SAME grammar the fold used — the refusal
        // sets cannot drift.
        if let Some(rest) = name.strip_prefix("group-has-") {
            let frag = variant::has_inner_fragment(rest).ok_or_else(|| seg.to_string())?;
            set.parent = Some(match label {
                "" => ".group".to_string(),
                l => format!(".{}", escape_class(&format!("group/{l}"))),
            });
            if let Some(head) = &mut set.parent {
                let _ = write!(head, ":has({frag})");
            }
            set.parent_prerendered = true;
            set.peer = false;
            continue;
        }
        if let Some(rest) = name.strip_prefix("peer-has-") {
            let frag = variant::has_inner_fragment(rest).ok_or_else(|| seg.to_string())?;
            set.parent = Some(match label {
                "" => ".peer".to_string(),
                l => format!(".{}", escape_class(&format!("peer/{l}"))),
            });
            if let Some(head) = &mut set.parent {
                let _ = write!(head, ":has({frag})");
            }
            set.parent_prerendered = true;
            set.peer = true;
            continue;
        }
        match name {
            "dark" => set.dark = true,
            "hover" => set.pseudos.push(":hover"),
            "focus" => set.pseudos.push(":focus"),
            "focus-visible" => set.pseudos.push(":focus-visible"),
            "focus-within" => set.pseudos.push(":focus-within"),
            "active" => set.pseudos.push(":active"),
            "disabled" => set.pseudos.push(":disabled"),
            "visited" => set.pseudos.push(":visited"),
            "checked" => set.pseudos.push(":checked"),
            "placeholder-shown" => set.pseudos.push(":placeholder-shown"),
            "selection" => set.pseudos.push("::selection"),
            "placeholder" => set.pseudos.push("::placeholder"),
            "group-hover" | "group-focus" => {
                let probe = name.strip_prefix("group-").unwrap_or(name);
                set.parent = Some(match label {
                    "" => format!(".group:{probe}"),
                    l => format!(".group/{l}:{probe}"),
                });
                set.peer = false;
            }
            "peer-hover" | "peer-focus" => {
                let probe = name.strip_prefix("peer-").unwrap_or(name);
                set.parent = Some(match label {
                    "" => format!(".peer:{probe}"),
                    l => format!(".peer/{l}:{probe}"),
                });
                set.peer = true;
            }
            _ => {
                // `variant::classify` typed it; the @-rule shape derives
                // from the same spelling the fold consumed.
                set.at = Some(at_of(seg).ok_or_else(|| seg.to_string())?);
            }
        }
    }
    Ok(set)
}

/// The registered alias's condition rendered into the selector shape —
/// the same edits the built-in segment spellings make, derived from the
/// typed [`Cond`] itself (an alias carries no group/peer LABEL, so the
/// unnamed head forms render; a `Cond::Base` alias renders no
/// condition, the flat model's honest reading).
fn apply_cond_shape(set: &mut CondSet, cond: aeolus_ir::Cond) {
    use aeolus_ir::Cond;
    match cond {
        Cond::Base => {}
        Cond::Dark => set.dark = true,
        Cond::Hover => set.pseudos.push(":hover"),
        Cond::Focus => set.pseudos.push(":focus"),
        Cond::FocusVisible => set.pseudos.push(":focus-visible"),
        Cond::FocusWithin => set.pseudos.push(":focus-within"),
        Cond::Active => set.pseudos.push(":active"),
        Cond::Disabled => set.pseudos.push(":disabled"),
        Cond::Visited => set.pseudos.push(":visited"),
        Cond::Checked => set.pseudos.push(":checked"),
        Cond::PlaceholderShown => set.pseudos.push(":placeholder-shown"),
        Cond::Selection => set.pseudos.push("::selection"),
        Cond::Placeholder => set.pseudos.push("::placeholder"),
        Cond::GroupHover => {
            set.parent = Some(".group:hover".to_string());
            set.peer = false;
        }
        Cond::GroupFocus => {
            set.parent = Some(".group:focus".to_string());
            set.peer = false;
        }
        Cond::PeerHover => {
            set.parent = Some(".peer:hover".to_string());
            set.peer = true;
        }
        Cond::PeerFocus => {
            set.parent = Some(".peer:focus".to_string());
            set.peer = true;
        }
        // The attribute probes: the corpus's attribute-test form (the
        // value quotes, the presence test stays bare, the has- inner
        // rides inside `:has(...)`).
        Cond::Data { name, value } => {
            set.probes.push(variant::attr_selector("data", name, value));
        }
        Cond::Aria { name, value } => {
            set.probes.push(variant::attr_selector("aria", name, value));
        }
        Cond::Has(inner) => set.probes.push(format!(":has({inner})")),
        // The parent-relative has-probes: the same conjunctive head the
        // group/peer state probes render, pre-escaped (the `:has(...)`
        // fragment carries colons the default head split cannot see
        // past). The LABEL forms render in the segment walk (the label
        // is head decoration, not an IR slot).
        Cond::GroupHas(inner) => {
            set.parent = Some(format!(".group:has({inner})"));
            set.parent_prerendered = true;
            set.peer = false;
        }
        Cond::PeerHas(inner) => {
            set.parent = Some(format!(".peer:has({inner})"));
            set.parent_prerendered = true;
            set.peer = true;
        }
        Cond::Bp(bp) | Cond::Min(bp) => set.at = Some(CssAt::MediaMin(bp)),
        Cond::Max(bp) => set.at = Some(CssAt::MediaMax(bp)),
        Cond::Container(bp) => set.at = Some(CssAt::ContainerMin(bp)),
        Cond::MaxContainer(bp) => set.at = Some(CssAt::ContainerMax(bp)),
    }
}

/// The responsive/container spellings → the @-rule form (the IR's
/// `Cond::{Bp,Min,Max,Container,MaxContainer}` set, rendered).
fn at_of(seg: &str) -> Option<CssAt> {
    if let Some(rest) = seg.strip_prefix("@max-") {
        return variant::breakpoint(rest).map(CssAt::ContainerMax);
    }
    if let Some(rest) = seg.strip_prefix('@') {
        return variant::breakpoint(rest).map(CssAt::ContainerMin);
    }
    if let Some(rest) = seg.strip_prefix("max-") {
        return variant::breakpoint(rest).map(CssAt::MediaMax);
    }
    if let Some(rest) = seg.strip_prefix("min-") {
        return variant::breakpoint(rest).map(CssAt::MediaMin);
    }
    variant::breakpoint(seg).map(CssAt::MediaMin)
}

/// The CSS identifier escape: everything outside `a-z A-Z 0-9 _ -` gets
/// a backslash (the v4 class-name escaping — `md\:p-6`,
/// `hover\:bg-primary\/90`).
fn escape_class(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len() + 4);
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
            out.push(ch);
        } else {
            out.push('\\');
            out.push(ch);
        }
    }
    out
}

/// Build one rule's selector: the optional parent/peer head with its
/// combinator, the escaped class, the probe suffixes, the dark scope.
/// `dark_companion` renders the baked-pair rule's `:where(.dark, .dark *)`
/// as a PREFIXED descendant scope (the companion resolves the pair's
/// dark half under the theme's own `.dark` block — an ancestor scope,
/// not a compound of the class), while the candidate's OWN `dark:`
/// segment keeps the compound suffix form. The head is
/// `.<class>[:<probe>]`: the class half escapes like any class text,
/// the probe half renders raw (a pseudo-class is selector syntax, not
/// class text).
fn selector_for(raw: &str, set: &CondSet, dark_companion: bool) -> String {
    let mut out = String::new();
    if dark_companion {
        out.push_str(":where(.dark, .dark *) ");
    }
    if let Some(head) = &set.parent {
        if set.parent_prerendered {
            // The has-probe heads render themselves (the class half is
            // escaped at build time, the probe half is selector text).
            out.push_str(head);
        } else {
            match head.rsplit_once(':') {
                Some((class_half, probe)) => {
                    out.push('.');
                    out.push_str(&escape_class(&class_half[1..]));
                    // rsplit consumed the delimiter; the probe re-attaches
                    // with its own colon (selector syntax, raw).
                    out.push(':');
                    out.push_str(probe);
                }
                None => {
                    out.push('.');
                    out.push_str(&escape_class(&head[1..]));
                }
            }
        }
        out.push_str(if set.peer { " ~ " } else { " " });
    }
    out.push('.');
    out.push_str(&escape_class(raw));
    for pseudo in &set.pseudos {
        out.push_str(pseudo);
    }
    // The attribute probes compound after the pseudo suffixes (the flat
    // arm's documented order: the source-stacked same-tier conditions
    // render pseudo-first, probe-second, the dark scope last).
    for probe in &set.probes {
        out.push_str(probe);
    }
    if set.dark {
        out.push_str(":where(.dark, .dark *)");
    }
    out
}

/// Render one candidate's declarations into its rule(s): the base rule
/// always; the `:where(.dark, .dark *)` companion when a baked pair rode
/// along and the candidate was not already `dark:`-conditioned. `dark`
/// picks the pair's dark half where one exists (the light half never
/// renders under dark).
fn push_rules(rules: &mut Vec<CssRule>, raw: &str, set: &CondSet, decls: &[lower::CssDecl]) {
    let lines = |dark: bool| {
        decls
            .iter()
            .map(|decl| {
                let value = if dark {
                    decl.value.dark.as_ref().unwrap_or(&decl.value.base)
                } else {
                    &decl.value.base
                };
                format!("{}: {};", decl.prop_name, value)
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    if set.dark {
        // The rule already fires under dark: the pair's dark half IS the
        // value (the light half never renders).
        rules.push(CssRule {
            selector: selector_for(raw, set, false),
            media: set.at,
            body: lines(true),
        });
        return;
    }
    rules.push(CssRule {
        selector: selector_for(raw, set, false),
        media: set.at,
        body: lines(false),
    });
    if decls.iter().any(|decl| decl.value.dark.is_some()) {
        rules.push(CssRule {
            selector: selector_for(raw, set, true),
            media: set.at,
            body: lines(true),
        });
    }
}
