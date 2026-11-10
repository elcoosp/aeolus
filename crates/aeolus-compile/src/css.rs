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

use aeolus_data::registry::lookup_detailed;
use aeolus_ir::Bp;
use aeolus_syntax::parse_class_string;
use aeolus_theme::TokenTable;

use crate::lower;
use crate::variant::{self, VariantMiss};
use crate::{
    lookup_diagnostic, normalize_modifier, reject_diagnostic, CompileError, Config, Diagnostic,
    DiagnosticKind,
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
    run_css(input, config, None)
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
    run_css(input, config, Some(tokens))
}

/// The whole emit walk, theme-less or themed. `tokens` selects whether
/// the colour/modifier surfaces render (the theme pass) or stay typed
/// boundaries (the honest default). Mirrors the IR walk's stages; the
/// two walks share the lexer channel,
/// the variant refusal set, the registry read side and the value pass's
/// boundary set, and `tests/css.rs` pins their diagnostics EQUAL over a
/// battery of strings.
pub(crate) fn run_css(
    input: &str,
    config: &Config,
    tokens: Option<&TokenTable>,
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
    tracing::debug!(tokens = candidates.len(), degraded = degraded.len(), "lexer done");

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
        let alpha = match candidate.modifier {
            None => None,
            Some(modifier) => match normalize_modifier(modifier).filter(|_| tokens.is_some()) {
                Some(alpha) => Some(alpha),
                None => {
                    diagnostics.push(Diagnostic::new(
                        candidate.span,
                        candidate.raw,
                        DiagnosticKind::UnsupportedModifier { modifier: modifier.to_string() },
                    ));
                    continue;
                }
            },
        };
        let set = match classify_segments(&candidate.variants) {
            Ok(set) => set,
            Err(missed) => {
                diagnostics.push(variant_diagnostic(candidate, &VariantMiss::new(&missed)));
                continue;
            }
        };
        let rows = match lookup_detailed(candidate.body) {
            Ok(rows) => rows,
            Err(err) => {
                diagnostics.push(lookup_diagnostic(candidate, err));
                continue;
            }
        };
        let mut decls: Vec<lower::CssDecl> = Vec::new();
        for row in rows {
            match lower::row_to_css(*row, candidate.body, candidate.negative, alpha, tokens, config)
            {
                Ok(mut rendered) => decls.append(&mut rendered),
                Err(reject) => {
                    diagnostics.push(reject_diagnostic(candidate, reject, candidate.modifier));
                }
            }
        }
        if decls.is_empty() {
            // Every row refused: the reports ride, no rule renders.
            continue;
        }
        decls.sort_by_key(|decl| decl.prop);
        push_rules(&mut rules, candidate.raw, &set, &decls);
    }

    #[cfg(feature = "tracing")]
    tracing::debug!(rules = rules.len(), diagnostics = diagnostics.len(), "css emitted");
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
    /// The `dark:` tier (the `:where(.dark, .dark *)` suffix).
    dark: bool,
    /// The parent/peer head, unescaped (`.group/menu:hover`) and whether
    /// it combines as a SIBLING (`.peer:… ~`) or an ancestor (`.group:… `).
    parent: Option<String>,
    peer: bool,
    /// The one @-rule the rule sits under.
    at: Option<CssAt>,
}

/// Classify the variant segments into the selector shape. The FIRST
/// segment outside the typed surface refuses (the same typed miss the
/// IR fold reports, with the segment text for the spanned diagnostic).
fn classify_segments(variants: &[&str]) -> Result<CondSet, String> {
    let mut set = CondSet::default();
    for &seg in variants {
        // The refusal agreement: the IR fold's typed surface decides
        // what a segment IS; the css render only chooses HOW it shows.
        if variant::classify(seg).is_none() {
            return Err(seg.to_string());
        }
        let (name, label) = seg.split_once('/').unwrap_or((seg, None));
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
                    Some(l) => format!(".group/{l}:{probe}"),
                    None => format!(".group:{probe}"),
                });
                set.peer = false;
            }
            "peer-hover" | "peer-focus" => {
                let probe = name.strip_prefix("peer-").unwrap_or(name);
                set.parent = Some(match label {
                    Some(l) => format!(".peer/{l}:{probe}"),
                    None => format!(".peer:{probe}"),
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
/// combinator, the escaped class, the probe suffixes, the dark suffix.
/// `dark_companion` renders the baked-pair rule's `:where(.dark, .dark *)`
/// even when the candidate itself carried no `dark:` segment. The head's
/// own leading class dot stays unescaped (it IS the selector dot); the
/// rest of the head escapes like any class text.
fn selector_for(raw: &str, set: &CondSet, dark_companion: bool) -> String {
    let mut out = String::new();
    if let Some(head) = &set.parent {
        out.push('.');
        out.push_str(&escape_class(&head[1..]));
        out.push_str(if set.peer { " ~ " } else { " " });
    }
    out.push('.');
    out.push_str(&escape_class(raw));
    for pseudo in &set.pseudos {
        out.push_str(pseudo);
    }
    if set.dark || dark_companion {
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
                format!("{}: {}", decl.prop_name, value)
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
