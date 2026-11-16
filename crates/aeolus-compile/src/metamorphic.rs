//! The metamorphic seam (architecture plan §5.3's differential line —
//! the `compile ⇒ interpret ⇒ equals emit` target the README's
//! correctness section promised): the three pipeline walks answer one
//! class string, and the report states — per probe situation — whether
//! they AGREE, with every documented divergence a typed note rather
//! than a silent mismatch.
//!
//! ```text
//! compile(input)  ──▶ Compiled { program, diagnostics }   ─┐
//! emit_css(input) ──▶ CssSheet  { rules,     diagnostics } ├─▶ MetamorphicReport
//! interp(program, Env) ──▶ Interped { decls, conds, ... }  ─┘    (agree | mismatches)
//! ```
//!
//! The invariant, in the shape the plan's differential triangle means
//! it: interpreting the compiled program under a probe [`Env`] yields
//! the same effective declarations the CSS arm renders for the
//! situations that env fires — cascade resolved on BOTH sides (the IR
//! folds same-condition candidates last-wins per longhand; the CSS
//! rules apply in source order with the later rule winning), theme
//! dynamism included (a baked pair answers its active half; a runtime
//! token resolves to concrete bytes while the CSS keeps the `var()`
//! reference — the typed [`MetamorphicNote::TokenDynamic`] note, never
//! a text lie).
//!
//! The walk is built from the pipeline's OWN parts, so it cannot drift
//! from them:
//!
//! * the per-candidate situations come from the same lexer walk and the
//!   same variant classifier the two emit walks run ([`classify`]);
//! * the per-cond verdicts under the probe env come from the
//!   interpreter's own `decide` — one `decide`, three consumers, so
//!   the checker cannot disagree with the cascade on when a layer
//!   fires;
//! * the colour texts render through the CSS arm's own [`rgba_text`]
//!   form, and the keyword slots resolve through the annotation seam
//!   ([`keyword_texts`]) — the same row texts the emit arm renders;
//! * the diagnostics channels are compared verbatim (the two walks
//!   share the typed lexer channel, the variant refusal set, the
//!   registry read side and the value pass's boundaries — a drift is a
//!   bug in one of the three, exactly the plan §5.2 triangle's rule).
//!
//! The honest boundaries, all typed ([`MetamorphicNote`]):
//!
//! * [`MetamorphicNote::StackedVariantFold`] — the CSS arm renders
//!   stacked variants conjunctively (`.hover\:md\:p-8:hover` under one
//!   `@media`), the IR folds to the dominant condition; the checker
//!   walks the env EVERY conjunct holds, where both sides answer, and
//!   the note records the fold;
//! * [`MetamorphicNote::PseudoTarget`] — `::selection`/`::placeholder`
//!   layers style a DIFFERENT render target: out of the element cascade
//!   on both sides (the interp's own [`InterpNote::PseudoElement`]
//!   semantics, translated);
//! * [`MetamorphicNote::TokenDynamic`] — a runtime-tier token renders
//!   `var(--color-…)`/`color-mix(...)` in the CSS and concrete bytes in
//!   the interp; the reference IS the theme's dynamism, so the pair is
//!   counted, not compared;
//! * [`MetamorphicNote::ContainerUnqueried`]/`AttributeUnqueried`/
//!   `StructuralUnqueried` — the sourceless probe layers never fire on
//!   either side (a CSS selector that does not match), the interp's own
//!   notes translated;
//! * the IR's unit-less `Num` slot renders bare (`500`) where the CSS
//!   text carries the row's unit (`500ms`) — the comparison is
//!   numeric-normalized for those pairs, textually exact everywhere
//!   else;
//! * the one row whose CSS property name overrides the IR longhand's
//!   (`stroke-2` → `stroke-width`, the `lower.rs` `NUMBER_FORMS` row)
//!   is mapped back so the two sides key identically.
//!
//! The checker is deterministic and pure over its inputs (the plan
//! §3.1 totality rule): the only whole-string failure remains the empty
//! input ([`CompileError::Empty`]); everything else degrades per class
//! into the report. The fuzz target `fuzz/fuzz_targets/compile_interp_emit.rs`
//! runs the same walk over arbitrary bytes (panic freedom + the
//! diagnostics equality always; agreement whenever a string lowers);
//! the deterministic battery in `tests/metamorphic.rs` pins the
//! agreement contracts; the bench and the example ride the same entry
//! points.

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fmt;

use aeolus_ir::{Cond, Prop, PropMask, Val};
use aeolus_syntax::parse_class_string;
use aeolus_theme::TokenTable;

use crate::custom::Customs;
use crate::interp::{self, Fire, InterpNote};
use crate::lower::rgba_text;
use crate::variant::classify;
use crate::{
    CompileError, Compiled, Config, CssRule, CssSheet, Env, compile_theme_with, compile_with,
    emit_css_theme_with, emit_css_with, interp_theme, keyword_texts,
};

// ---------------------------------------------------------------------------
// The report types
// ---------------------------------------------------------------------------

/// One disagreement the checker found: the property, and the two
/// sides' answers (`None` — the side has no answer for that property;
/// the CSS text carries the row's rendering, the interp text the
/// cascade's winner).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MetamorphicMismatch {
    /// The IR longhand's name (the comparison key).
    pub prop: String,
    /// The interpreter's rendered value, when it answered.
    pub interp: Option<String>,
    /// The CSS arm's rendered value, when it rendered.
    pub css: Option<String>,
}

/// One probe situation the checker walked: the conjunctive [`Env`] the
/// candidate's condition stack names, and whether the interp's
/// effective declarations agree with the CSS rules the env fires.
#[derive(Clone, Debug, PartialEq)]
pub struct SituationReport {
    /// The situation's label (`base`, `hover`, `md`, `hover+md`,
    /// `base (dark)`) — the condition stack, conjunctive.
    pub label: String,
    /// The probe situation the walk ran under (every conjunct of the
    /// candidate's condition stack holds; the dark-mode flip rides the
    /// battery so baked pairs answer both halves).
    pub env: Env,
    /// Whether the two sides agree on every longhand they both answer.
    pub agree: bool,
    /// The interp's effective declaration count under the env.
    pub interp_decls: usize,
    /// The CSS declaration count the env's fired rules cascade to.
    pub css_decls: usize,
    /// The per-property disagreements, in `Prop` order — empty exactly
    /// when [`SituationReport::agree`] holds.
    pub mismatches: Vec<MetamorphicMismatch>,
    /// The interp walk's own refusal, when the resolution failed (a
    /// themed interp refuses per longhand with the typed
    /// [`InterpError`](crate::InterpError); the report carries the
    /// text, the situation does not agree).
    pub interp_error: Option<String>,
}

/// A documented divergence or exclusion the walk recorded — the
/// metamorphic contract's no-silent-failures surface: the report never
/// papers over a difference, it NAMES it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MetamorphicNote {
    /// The candidate stacks variants (`hover:md:p-8`): the CSS arm
    /// renders the stack conjunctively, the IR folds to the dominant
    /// condition. The checker's situation walks the env every conjunct
    /// holds — the note records the fold so a wider env's divergence is
    /// never a surprise.
    StackedVariantFold {
        /// The stacked candidate's raw text.
        raw: String,
        /// How many variant segments stacked.
        segments: usize,
    },
    /// The candidate targets a pseudo-element (`selection:`/
    /// `placeholder:`): a DIFFERENT render target than the element's
    /// own cascade — excluded from the element walk on both sides (the
    /// interp's [`InterpNote::PseudoElement`] semantics, translated).
    PseudoTarget {
        /// The pseudo candidate's raw text.
        raw: String,
        /// The target the layer styles (`::selection`/`::placeholder`).
        target: &'static str,
    },
    /// The declaration is a runtime-tier token: the CSS keeps the
    /// `var(--color-…)`/`color-mix(...)` reference (the theme's own
    /// dynamism), the interp resolves concrete bytes for the probe. The
    /// pair is COUNTED here, never compared as text.
    TokenDynamic {
        /// The longhand the reference sits on (the IR name, the
        /// comparison key's own spelling).
        prop: String,
    },
    /// A named-container layer the sourceless walk cannot answer (the
    /// interp's [`InterpNote::ContainerUnqueried`], translated): the
    /// layer never fires on either side (a CSS selector that does not
    /// match).
    ContainerUnqueried,
    /// An attribute-probe layer the sourceless walk cannot answer (the
    /// interp's [`InterpNote::AttributeUnqueried`], translated).
    AttributeUnqueried,
    /// A structural layer the sourceless walk cannot answer (the
    /// interp's [`InterpNote::StructuralUnqueried`], translated).
    StructuralUnqueried,
    /// A descendant layer the sourceless walk cannot answer (the
    /// interp's [`InterpNote::DescendantUnqueried`], translated).
    DescendantUnqueried,
    /// A supports layer the sourceless walk cannot answer (the
    /// interp's [`InterpNote::SupportsUnqueried`], translated).
    SupportsUnqueried,
    /// The CSS arm's composed-filter chain line (`filter: var(--tw-blur,)
    /// …` — the filter family's v4 emission form): the re-declaration
    /// the primitive rules carry beside their var lines. The IR keeps
    /// the PRIMITIVES (one longhand each); the chain is the CSS
    /// cascade's own composition, so the pair is COUNTED here, never
    /// compared as text.
    FilterCompose {
        /// The longhand the chain re-declares (the composed property's
        /// IR name — `filter`).
        prop: String,
    },
}

impl fmt::Display for MetamorphicNote {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MetamorphicNote::StackedVariantFold { raw, segments } => write!(
                f,
                "{raw:?} stacks {segments} variants: the CSS renders them \
                 conjunctively, the IR folds to the dominant condition"
            ),
            MetamorphicNote::PseudoTarget { raw, target } => write!(
                f,
                "{raw:?} styles {target}: a separate render target, \
                 excluded from the element's own cascade"
            ),
            MetamorphicNote::TokenDynamic { prop } => write!(
                f,
                "the {prop} value is a runtime token: the CSS keeps the \
                 var() reference, the interp resolves the probe's bytes"
            ),
            MetamorphicNote::FilterCompose { prop } => write!(
                f,
                "the {prop} chain line is the CSS cascade's own \
                 composition (the filter family's var-chain form); the \
                 IR keeps the primitives the vars name"
            ),
            MetamorphicNote::ContainerUnqueried => write!(
                f,
                "the named-container layer never matched: no bounds source \
                 rides, and a CSS selector that does not match is quiet"
            ),
            MetamorphicNote::AttributeUnqueried => write!(
                f,
                "the attribute-probe layer never matched: no AttrProbe \
                 source rides, and a CSS selector that does not match is \
                 quiet"
            ),
            MetamorphicNote::StructuralUnqueried => write!(
                f,
                "the structural layer never matched: no StructProbe source \
                 rides, and a CSS selector that does not match is quiet"
            ),
            MetamorphicNote::DescendantUnqueried => write!(
                f,
                "the descendant layer never matched: no DescendantProbe \
                 source rides, and a CSS selector that does not match is \
                 quiet"
            ),
            MetamorphicNote::SupportsUnqueried => write!(
                f,
                "the supports layer never matched: no SupportsProbe \
                 source rides, and an @supports block the target fails \
                 is quiet"
            ),
        }
    }
}

/// The metamorphic walk's artifact: the three pipeline answers over one
/// class string, their agreement, and every typed note the walk
/// recorded.
#[derive(Clone, Debug, PartialEq)]
pub struct MetamorphicReport {
    /// The checked input, verbatim.
    pub input: String,
    /// The compile walk's diagnostics and the CSS walk's diagnostics are
    /// byte-equal (the two channels share the typed refusal surfaces —
    /// a drift is a bug in one of the three, plan §5.2's triangle rule).
    pub diagnostics_equal: bool,
    /// The whole-string sheet is the composition of the per-candidate
    /// sheets (the arm is per-candidate; the parts add up to the whole).
    pub compositional: bool,
    /// The probe situations the battery walked, first-seen order: the
    /// base situation always, one per distinct condition stack among the
    /// lowered candidates, each in both mode halves.
    pub situations: Vec<SituationReport>,
    /// The typed notes: every documented divergence or exclusion the
    /// walk recorded, deduped, in walk order.
    pub notes: Vec<MetamorphicNote>,
}

impl MetamorphicReport {
    /// Whether the three walks agree end to end: the diagnostics
    /// channels equal, the sheet compositional, every walked situation
    /// agreeing. The fuzz target asserts this for every string that
    /// lowers; the battery pins it over the corpus.
    pub fn agree(&self) -> bool {
        self.diagnostics_equal
            && self.compositional
            && self
                .situations
                .iter()
                .all(|s| s.agree && s.interp_error.is_none())
    }
}

impl fmt::Display for MetamorphicReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "metamorphic {} ({:?})",
            if self.agree() { "agree" } else { "DIVERGED" },
            self.input
        )?;
        writeln!(
            f,
            "  diagnostics {}equal, sheet {}compositional",
            if self.diagnostics_equal { "" } else { "NOT " },
            if self.compositional { "" } else { "NOT " },
        )?;
        for situation in &self.situations {
            writeln!(
                f,
                "  {} [{}]: interp {} decls, css {} decls{}",
                situation.label,
                if situation.agree { "agree" } else { "DIVERGED" },
                situation.interp_decls,
                situation.css_decls,
                situation
                    .interp_error
                    .as_ref()
                    .map_or(String::new(), |err| format!(" (interp: {err})")),
            )?;
            for mismatch in &situation.mismatches {
                writeln!(
                    f,
                    "    {}: interp {:?} vs css {:?}",
                    mismatch.prop,
                    mismatch.interp.as_deref().unwrap_or("(none)"),
                    mismatch.css.as_deref().unwrap_or("(none)"),
                )?;
            }
        }
        for note in &self.notes {
            writeln!(f, "  note: {note}")?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// The entry points
// ---------------------------------------------------------------------------

/// The metamorphic walk, theme-less: colour tokens and opacity
/// modifiers stay the typed boundaries both channels report identically
/// (the honest default [`compile_with`] has).
///
/// ```no_run
/// use aeolus_compile::{Config, metamorphic_check};
///
/// let report = metamorphic_check("p-4 hover:px-8", &Config::default())?;
/// assert!(report.agree());
/// for situation in &report.situations {
///     println!("{}: {}", situation.label, situation.interp_decls);
/// }
/// # Ok::<(), aeolus_compile::CompileError>(())
/// ```
///
/// # Errors
/// The same [`CompileError`] as [`compile_with`] (only the empty
/// input). Per-class refusals ride the report, never an error.
pub fn metamorphic_check(input: &str, config: &Config) -> Result<MetamorphicReport, CompileError> {
    run(input, None, &Customs::new(), *config)
}

/// [`metamorphic_check`] with the custom-authoring seam supplied (the
/// [`compile_with`] precedent): registered utilities and variant
/// aliases ride the same walk — the registry wins a name collision,
/// empty registrations are byte-identical to the plain entry point.
///
/// # Errors
/// The same [`CompileError`] as [`compile_with`] (only the empty
/// input).
pub fn metamorphic_check_with(
    input: &str,
    customs: &Customs,
    config: &Config,
) -> Result<MetamorphicReport, CompileError> {
    run(input, None, customs, *config)
}

/// The metamorphic walk, themed: colour tokens and `/opacity`
/// modifiers lower through the [`TokenTable`](aeolus_theme::TokenTable)
/// (the tiered tables the theme pass resolves), the interp resolves the
/// probe's bytes, and the runtime-tier `var()` references ride the
/// typed [`MetamorphicNote::TokenDynamic`] notes.
///
/// ```no_run
/// use aeolus_compile::{Config, metamorphic_check_theme};
/// use aeolus_theme::{ThemeSource, TokenTable};
///
/// let source = ThemeSource::shadcn_dark()?;
/// let tokens = TokenTable::baked(&source)?;
/// let report = metamorphic_check_theme(
///     "bg-primary dark:bg-primary/90",
///     &tokens,
///     &Config::default(),
/// )?;
/// assert!(report.agree());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
/// The same [`CompileError`] as [`compile_theme_with`] (only the empty
/// input).
pub fn metamorphic_check_theme(
    input: &str,
    tokens: &TokenTable,
    config: &Config,
) -> Result<MetamorphicReport, CompileError> {
    run(input, Some(tokens), &Customs::new(), *config)
}

/// [`metamorphic_check_theme`] with the custom-authoring seam supplied
/// (the [`compile_theme_with`] precedent).
///
/// # Errors
/// The same [`CompileError`] as [`compile_theme_with`] (only the empty
/// input).
pub fn metamorphic_check_theme_with(
    input: &str,
    tokens: &TokenTable,
    customs: &Customs,
    config: &Config,
) -> Result<MetamorphicReport, CompileError> {
    run(input, Some(tokens), customs, *config)
}

// ---------------------------------------------------------------------------
// The walk
// ---------------------------------------------------------------------------

/// One lowered candidate: the raw text, its folded condition stack, and
/// the sheet the CSS arm renders for the candidate ALONE (the
/// compositional identity's part).
struct Lowered {
    raw: String,
    conds: Vec<Cond>,
    sheet: CssSheet,
    /// The candidate carries a pseudo-element variant — a separate
    /// render target, out of the element cascade both sides.
    pseudo: Option<&'static str>,
}

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(level = "debug", skip_all, fields(input_len = input.len()))
)]
fn run(
    input: &str,
    tokens: Option<&TokenTable>,
    customs: &Customs,
    config: Config,
) -> Result<MetamorphicReport, CompileError> {
    #[cfg(feature = "tracing")]
    tracing::debug!(
        input_len = input.len(),
        "metamorphic: compile -> emit -> interp"
    );

    if input.trim().is_empty() {
        return Err(CompileError::Empty);
    }

    // Stage 1: the two pipeline answers, through the SAME entry points
    // a host uses — the checker derives nothing the walks do not.
    let compiled: Compiled = match tokens {
        Some(table) => compile_theme_with(input, table, customs, &config)?,
        None => compile_with(input, customs, &config)?,
    };
    let sheet: CssSheet = match tokens {
        Some(table) => emit_css_theme_with(input, table, customs, &config)?,
        None => emit_css_with(input, customs, &config)?,
    };
    let diagnostics_equal = compiled.diagnostics == sheet.diagnostics;

    // Stage 2: the per-candidate lowering — the same lexer walk, the
    // same variant classifier, the candidate's OWN sheet through the
    // emit arm (the parts the whole must compose to).
    let (candidates, _) = parse_class_string(input);
    let kw_map = keyword_map(input, config)?;
    let mut lowered: Vec<Lowered> = Vec::new();
    for candidate in &candidates {
        if candidate.important {
            continue; // never lowers (the diagnostic rides both channels)
        }
        let mut conds = Vec::with_capacity(candidate.variants.len());
        let mut pseudo = None;
        let mut classified = true;
        for segment in &candidate.variants {
            match classify(segment, customs) {
                Some(cond) => {
                    if pseudo.is_none() {
                        pseudo = pseudo_target(cond);
                    }
                    conds.push(cond);
                }
                None => {
                    classified = false; // refuses both channels — no rule
                    break;
                }
            }
        }
        if !classified {
            continue;
        }
        let sheet_c = match tokens {
            Some(table) => emit_css_theme_with(candidate.raw, table, customs, &config)?,
            None => emit_css_with(candidate.raw, customs, &config)?,
        };
        if sheet_c.rules.is_empty() {
            continue; // the boundary shapes: reported, never a rule
        }
        lowered.push(Lowered {
            raw: candidate.raw.to_string(),
            conds,
            sheet: sheet_c,
            pseudo,
        });
    }

    // The compositional identity: the whole sheet IS the per-candidate
    // parts, in candidate order.
    let mut parts: Vec<CssRule> = Vec::new();
    for candidate in &lowered {
        parts.extend(candidate.sheet.rules.iter().cloned());
    }
    let compositional = parts == sheet.rules;

    // Stage 3: the situation battery — one probe env per distinct
    // condition stack (first-seen order), each in both mode halves. The
    // dark flip walks only where the stack does not already carry the
    // mode (a `dark:` stack IS its own dark walk).
    let mut situations: Vec<SituationReport> = Vec::new();
    let mut notes: Vec<MetamorphicNote> = Vec::new();
    let mut seen: Vec<String> = Vec::new();
    for candidate in &lowered {
        if candidate.conds.len() > 1 {
            notes.push(MetamorphicNote::StackedVariantFold {
                raw: candidate.raw.clone(),
                segments: candidate.conds.len(),
            });
        }
        if let Some(target) = candidate.pseudo {
            notes.push(MetamorphicNote::PseudoTarget {
                raw: candidate.raw.clone(),
                target,
            });
        }
        let env = conjunctive_env(&candidate.conds);
        let label = situation_label(&candidate.conds);
        let dark_label = format!("{label} (dark)");
        let mut walks: Vec<(Env, String)> = vec![(env, label)];
        if !candidate.conds.contains(&Cond::Dark) {
            walks.push((Env { dark: true, ..env }, dark_label));
        }
        for (env, label) in walks {
            if seen.contains(&label) {
                continue;
            }
            seen.push(label.clone());
            let (report, walk_notes) =
                situation_walk(&compiled, &lowered, tokens, &kw_map, &env, &label);
            notes.extend(walk_notes);
            situations.push(report);
        }
    }

    Ok(MetamorphicReport {
        input: input.to_string(),
        diagnostics_equal,
        compositional,
        situations,
        notes: dedup_notes(notes),
    })
}

/// Walk ONE probe situation: the CSS rules the env fires, cascade
/// resolved (source order, later wins per longhand), against the
/// interp's effective declarations under the same env.
fn situation_walk(
    compiled: &Compiled,
    lowered: &[Lowered],
    tokens: Option<&TokenTable>,
    kw_map: &HashMap<(Cond, Prop), &'static str>,
    env: &Env,
    label: &str,
) -> (SituationReport, Vec<MetamorphicNote>) {
    // The CSS expectation: every fired candidate's rules, source order
    // (the later rule wins per longhand — the cascade the browser runs;
    // the dark companion sits AFTER its base rule, so under dark the
    // pair's dark half wins).
    let mut css_map: BTreeMap<String, String> = BTreeMap::new();
    let mut compose_notes: Vec<MetamorphicNote> = Vec::new();
    for candidate in lowered {
        if !fires(env, &candidate.conds) {
            continue;
        }
        for rule in &candidate.sheet.rules {
            // The dark companion carries the `:where(.dark, .dark *)`
            // scope prefix (the arm's own render form); it fires only
            // under the dark half.
            let dark_scoped = rule.selector.starts_with(DARK_SCOPE_PREFIX);
            if dark_scoped && !env.dark {
                continue;
            }
            for line in rule.body.lines() {
                if let Some((name, value)) = line.split_once(": ") {
                    let value = value.trim_end_matches(';');
                    // The filter family's chain lines: the composition
                    // re-declarations the primitive rules carry beside
                    // their var lines (the filter chain; the backdrop
                    // chain on BOTH properties) — the typed note,
                    // never a text comparison (the IR keeps the
                    // primitives).
                    if value == aeolus_data::rows::FILTER_CHAIN
                        || value == aeolus_data::rows::BACKDROP_FILTER_CHAIN
                    {
                        compose_notes.push(MetamorphicNote::FilterCompose {
                            prop: css_prop_to_ir(name).to_string(),
                        });
                        continue;
                    }
                    css_map.insert(css_prop_to_ir(name).to_string(), value.to_string());
                }
            }
        }
    }

    // The interp answer: the effective declarations under the SAME env.
    let interped = match tokens {
        Some(table) => interp_theme(&compiled.program, table, env),
        None => interp::interp(&compiled.program, env),
    };
    let mut interp_map: BTreeMap<String, String> = BTreeMap::new();
    let mut interp_error = None;
    let mut interp_decls = 0usize;
    if let Ok(resolved) = &interped {
        interp_decls = resolved.decls.len();
        for (index, decl) in resolved.decls.iter().enumerate() {
            let cond = resolved.conds.get(index).copied().unwrap_or(Cond::Base);
            let kw = kw_map.get(&(cond, decl.prop)).copied();
            if let Some(text) = value_text(decl.val, kw) {
                interp_map.insert(decl.prop.name().to_string(), text);
            }
            // A `Val` the text form cannot carry (a host source's
            // non-colour escape) is an honest mismatch, not a lie.
        }
    } else if let Err(err) = &interped {
        interp_error = Some(err.to_string());
    }

    // The runtime-token pairs: the CSS keeps the var() reference (the
    // theme's own dynamism), the interp answers the probe's bytes —
    // counted as the typed note, never compared as text.
    let mut mismatches: Vec<MetamorphicMismatch> = Vec::new();
    let mut token_notes: Vec<MetamorphicNote> = Vec::new();
    for (prop, css_text) in &css_map {
        if is_token_reference(css_text) {
            token_notes.push(MetamorphicNote::TokenDynamic { prop: prop.clone() });
            continue;
        }
        let Some(interp_text) = interp_map.get(prop) else {
            mismatches.push(MetamorphicMismatch {
                prop: prop.clone(),
                interp: None,
                css: Some(css_text.clone()),
            });
            continue;
        };
        if !value_agrees(interp_text, css_text) {
            mismatches.push(MetamorphicMismatch {
                prop: prop.clone(),
                interp: Some(interp_text.clone()),
                css: Some(css_text.clone()),
            });
        }
    }
    for (prop, interp_text) in &interp_map {
        let covered = css_map
            .get(prop)
            .is_some_and(|text| is_token_reference(text));
        if !css_map.contains_key(prop) && !covered {
            mismatches.push(MetamorphicMismatch {
                prop: prop.clone(),
                interp: Some(interp_text.clone()),
                css: None,
            });
        }
    }
    // The interp's own exclusion notes, translated (the sourceless
    // probe layers: quiet on both sides, reported here).
    let mut notes: Vec<MetamorphicNote> = token_notes;
    notes.extend(compose_notes);
    if let Ok(resolved) = &interped {
        for note in &resolved.notes {
            match note {
                InterpNote::PseudoElement { .. } => {} // the PseudoTarget note rides per candidate
                InterpNote::ContainerUnqueried { .. } => {
                    notes.push(MetamorphicNote::ContainerUnqueried);
                }
                InterpNote::AttributeUnqueried { .. } => {
                    notes.push(MetamorphicNote::AttributeUnqueried);
                }
                InterpNote::StructuralUnqueried { .. } => {
                    notes.push(MetamorphicNote::StructuralUnqueried);
                }
                InterpNote::DescendantUnqueried { .. } => {
                    notes.push(MetamorphicNote::DescendantUnqueried);
                }
                InterpNote::SupportsUnqueried { .. } => {
                    notes.push(MetamorphicNote::SupportsUnqueried);
                }
            }
        }
    }

    let agree = interp_error.is_none() && mismatches.is_empty();
    (
        SituationReport {
            label: label.to_string(),
            env: *env,
            agree,
            interp_decls,
            css_decls: css_map.len(),
            mismatches,
            interp_error,
        },
        notes,
    )
}

// ---------------------------------------------------------------------------
// The situation helpers
// ---------------------------------------------------------------------------

/// The dark companion rule's scope prefix — the arm's own render form
/// (the CSS emitter's `dark_companion` half).
const DARK_SCOPE_PREFIX: &str = ":where(.dark, .dark *) ";

/// The interp's probe verdict for one condition — the interpreter's own
/// `decide` (one source of truth: the checker cannot disagree with the
/// cascade on when a layer fires).
fn fires(env: &Env, conds: &[Cond]) -> bool {
    conds
        .iter()
        .all(|&cond| matches!(decide_shim(cond, env), Fire::Yes))
}

/// The `decide` call with the sourceless probe set (no attribute,
/// container or structural source): the probe layers refuse to fire and
/// the notes carry them, exactly the plain interp forms' semantics.
fn decide_shim(cond: Cond, env: &Env) -> Fire {
    crate::interp::decide(cond, env, None, None, None, None, None, PropMask::EMPTY)
}

/// The conjunctive probe env a condition stack names: every state bit
/// the stack carries, the viewport at the tightest breakpoint (min
/// forms fire AT their rem; max forms strictly below — the walk sits
/// half a rem under), the unnamed container at its boundary. The
/// sourceless probe shapes (named containers, attribute, structural,
/// descendant, pseudo-element) set nothing — their layers never fire on
/// either side, the notes carry them.
fn conjunctive_env(conds: &[Cond]) -> Env {
    let mut env = Env::default();
    for cond in conds {
        match cond {
            Cond::Dark => env.dark = true,
            Cond::Hover => env.hover = true,
            Cond::Focus => env.focus = true,
            Cond::FocusVisible => env.focus_visible = true,
            Cond::FocusWithin => env.focus_within = true,
            Cond::Active => env.active = true,
            Cond::Disabled => env.disabled = true,
            Cond::Visited => env.visited = true,
            Cond::Checked => env.checked = true,
            Cond::PlaceholderShown => env.placeholder_shown = true,
            Cond::GroupHover => env.group_hover = true,
            Cond::GroupFocus => env.group_focus = true,
            Cond::PeerHover => env.peer_hover = true,
            Cond::PeerFocus => env.peer_focus = true,
            Cond::Bp(bp) | Cond::Min(bp) => env.viewport_rem = env.viewport_rem.max(bp.rem()),
            Cond::Max(bp) => {
                let below = (bp.rem() - 0.5).max(0.0);
                env.viewport_rem = if env.viewport_rem <= 0.0 {
                    below
                } else {
                    env.viewport_rem.min(below)
                };
            }
            Cond::Container { bp, name: None } => {
                env.container_rem = Some(
                    env.container_rem
                        .map_or(bp.rem(), |current| current.max(bp.rem())),
                );
            }
            Cond::MaxContainer { bp, name: None } => {
                let below = (bp.rem() - 0.5).max(0.0);
                env.container_rem = Some(match env.container_rem {
                    Some(current) if current > 0.0 => current.min(below),
                    _ => below,
                });
            }
            // The media-feature preferences are frame state: the
            // conjunctive env carries the fact, the layer fires on
            // both sides when the query holds.
            Cond::Media(pref) => match pref {
                aeolus_ir::Pref::MotionSafe => env.reduced_motion = false,
                aeolus_ir::Pref::MotionReduce => env.reduced_motion = true,
                aeolus_ir::Pref::ContrastMore => env.more_contrast = true,
                aeolus_ir::Pref::ContrastLess => env.less_contrast = true,
                aeolus_ir::Pref::ForcedColors => env.forced_colors = true,
                aeolus_ir::Pref::Print => env.print_media = true,
            },
            // The sourceless shapes: the layer never fires on either
            // side (a CSS selector that does not match), the note rides.
            Cond::Base
            | Cond::Container { name: Some(_), .. }
            | Cond::MaxContainer { name: Some(_), .. }
            | Cond::Data { .. }
            | Cond::Aria { .. }
            | Cond::Has(_)
            | Cond::GroupHas(_)
            | Cond::PeerHas(_)
            | Cond::Structural(_)
            | Cond::Supports(_)
            | Cond::Selection
            | Cond::Placeholder => {}
        }
    }
    env
}

/// The situation's label — the condition stack, conjunctive (`base`,
/// `hover`, `md`, `hover+md`). Deterministic per stack.
fn situation_label(conds: &[Cond]) -> String {
    if conds.is_empty() {
        return "base".to_string();
    }
    conds.iter().map(cond_label).collect::<Vec<_>>().join("+")
}

/// One condition's label fragment.
fn cond_label(cond: &Cond) -> String {
    match cond {
        Cond::Base => "base".to_string(),
        Cond::Dark => "dark".to_string(),
        Cond::Hover => "hover".to_string(),
        Cond::Focus => "focus".to_string(),
        Cond::FocusVisible => "focus-visible".to_string(),
        Cond::FocusWithin => "focus-within".to_string(),
        Cond::Active => "active".to_string(),
        Cond::Disabled => "disabled".to_string(),
        Cond::Visited => "visited".to_string(),
        Cond::Checked => "checked".to_string(),
        Cond::PlaceholderShown => "placeholder-shown".to_string(),
        Cond::Selection => "selection".to_string(),
        Cond::Placeholder => "placeholder".to_string(),
        Cond::GroupHover => "group-hover".to_string(),
        Cond::GroupFocus => "group-focus".to_string(),
        Cond::PeerHover => "peer-hover".to_string(),
        Cond::PeerFocus => "peer-focus".to_string(),
        Cond::Bp(bp) | Cond::Min(bp) => bp.name().to_string(),
        Cond::Max(bp) => format!("max-{}", bp.name()),
        Cond::Container {
            bp,
            name: Some(name),
        } => format!("@{}/{}", bp.name(), name),
        Cond::Container { bp, name: None } => format!("@{}", bp.name()),
        Cond::MaxContainer {
            bp,
            name: Some(name),
        } => format!("@max-{}/{}", bp.name(), name),
        Cond::MaxContainer { bp, name: None } => format!("@max-{}", bp.name()),
        Cond::Data { name, value } => match value {
            Some(value) => format!("data-{name}={value}"),
            None => format!("data-{name}"),
        },
        Cond::Aria { name, value } => match value {
            Some(value) => format!("aria-{name}={value}"),
            None => format!("aria-{name}"),
        },
        Cond::Has(_) => "has".to_string(),
        Cond::Supports(_) => "supports".to_string(),
        Cond::Media(pref) => pref.source().to_string(),
        Cond::GroupHas(_) => "group-has".to_string(),
        Cond::PeerHas(_) => "peer-has".to_string(),
        Cond::Structural(_) => "structural".to_string(),
    }
}

/// The pseudo-element render target a condition folds to, when it is
/// one (`::selection`/`::placeholder` — a different render target than
/// the element's own cascade).
fn pseudo_target(cond: Cond) -> Option<&'static str> {
    match cond {
        Cond::Selection => Some("::selection"),
        Cond::Placeholder => Some("::placeholder"),
        _ => None,
    }
}

/// The one CSS property name that overrides the IR longhand's name —
/// `stroke-2` renders `stroke-width` (the `lower.rs` `NUMBER_FORMS`
/// row's own override: the Prop slot stays the IR's, the CSS text takes
/// the real property name). Everything else keys identically.
fn css_prop_to_ir(name: &str) -> &str {
    if name == "stroke-width" {
        "stroke"
    } else {
        name
    }
}

/// Whether the CSS text is a runtime-token reference (`var(--color-…)`
/// or the `color-mix(...)` alpha form) — the theme's own dynamism, the
/// typed [`MetamorphicNote::TokenDynamic`] note's input.
fn is_token_reference(text: &str) -> bool {
    text.starts_with("var(--color-") || text.starts_with("color-mix(")
}

/// Dedup the notes, walk order kept (the report's contract: every
/// documented divergence ONCE, in the order the walk recorded it).
fn dedup_notes(notes: Vec<MetamorphicNote>) -> Vec<MetamorphicNote> {
    let mut out: Vec<MetamorphicNote> = Vec::new();
    for note in notes {
        if !out.contains(&note) {
            out.push(note);
        }
    }
    out
}

// ---------------------------------------------------------------------------
// The value text helpers
// ---------------------------------------------------------------------------

/// The interp declaration's canonical text — the CSS arm's own render
/// forms: the keyword slots through the row texts (the annotation seam),
/// the literals through the CSS arm's own colour and unit forms.
/// `None` — the value the text form cannot carry (a host source's
/// non-colour escape), an honest mismatch rather than a lie.
fn value_text(val: Val, kw: Option<&'static str>) -> Option<String> {
    match val {
        Val::Px(v) => Some(format!("{v}px")),
        Val::Rem(v) => Some(format!("{v}rem")),
        Val::Pct(v) => Some(format!("{v}%")),
        Val::Num(v) => Some(format!("{v}")),
        Val::Rgba(rgba) => Some(rgba_text(rgba)),
        Val::Kw(_) => kw.map(str::to_string),
        // The composite grammar's verbatim slot: the text IS the value —
        // both sides render it (the CSS arm emits the decoded text as
        // typed), so the agreement is textual.
        Val::Verbatim(text) => Some(text.to_string()),
        // The theme forms: the built-in tables resolve them BEFORE the
        // interp answers (concrete bytes); a host source returning one
        // is the honest mismatch.
        Val::Tok { .. } | Val::ModePair(_) => None,
    }
}

/// Whether the two renderings agree: textually exact first (both sides
/// render from the same row tables and the same f32 math), then
/// numeric-normalized for the IR's unit-less `Num` slot (the interp
/// renders `500`, the CSS text carries the row's unit — `500ms`), then
/// colour-normalized for the arbitrary literal's VERBATIM form (the CSS
/// arm passes `bg-[#0f172a]`'s hex through as typed, the interp renders
/// the baked bytes in the space-separated `rgb(...)` form — one colour,
/// two renderings).
fn value_agrees(interp: &str, css: &str) -> bool {
    if interp == css {
        return true;
    }
    if let (Some(a), Some(b)) = (parse_f32(interp), parse_f32(css)) {
        return a == b;
    }
    // The filter family's fn-wrapped numbers (`brightness(110%)` vs the
    // interp's bare `110`, `blur(2px)` vs `2px`): the comparison key is
    // the PROP, which fixes the function, so the number carries the
    // agreement.
    if let (Some(a), Some(b)) = (parse_f32(interp), parse_f32_fn_wrap(css)) {
        return a == b;
    }
    if let (Some(a), Some(b)) = (parse_f32_fn_wrap(interp), parse_f32(css)) {
        return a == b;
    }
    match (parse_colour(interp), parse_colour(css)) {
        (Some(a), Some(b)) => a == b,
        _ => false,
    }
}

/// The filter functions the fn-wrap parse accepts (the family's own
/// `fn(number)` forms — everything else stays textual).
const FN_WRAP_FNS: &[&str] = &[
    "blur",
    "brightness",
    "contrast",
    "grayscale",
    "hue-rotate",
    "invert",
    "saturate",
    "sepia",
];

/// Parse a fn-wrapped numeric text — `brightness(110%)`,
/// `hue-rotate(30deg)`, `blur(2px)`: the name must be one of the
/// filter functions, the inner text a bare/unit-carrying number.
/// Multi-value texts (the composed drop-shadow lists) parse to
/// nothing — the comparison stays textual there.
fn parse_f32_fn_wrap(text: &str) -> Option<f32> {
    let (name, inner) = text.strip_suffix(')')?.split_once('(')?;
    if !FN_WRAP_FNS.contains(&name.trim()) {
        return None;
    }
    let inner = inner.trim();
    // The wrapped text must be ENTIRELY one single-value number: a
    // composed list (the multi-band drop-shadow presets) carries
    // separators or a second function — the comparison stays textual
    // there.
    if inner.contains([' ', ',', '(']) {
        return None;
    }
    parse_f32(inner)
}

/// Parse a rendered colour text into its bytes: the hex spellings (the
/// arbitrary literal's verbatim passthrough) and the space-separated
/// `rgb(r g b)`/`rgb(r g b / N%)` forms ([`rgba_text`]'s own output).
/// Everything else parses to nothing (keywords, lengths — the textual
/// comparison stays in charge there).
fn parse_colour(text: &str) -> Option<[u8; 4]> {
    let text = text.trim();
    if let Some(hex) = text.strip_prefix('#') {
        return match hex.len() {
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some([r, g, b, 255])
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some([r, g, b, a])
            }
            _ => None,
        };
    }
    let rest = text
        .strip_prefix("rgb(")
        .and_then(|rest| rest.strip_suffix(')'))?;
    let mut parts = rest.split('/');
    let channels = parts.next()?.split_whitespace().collect::<Vec<_>>();
    if channels.len() != 3 {
        return None;
    }
    let mut rgba = [0u8, 0, 0, 255];
    for (slot, channel) in rgba.iter_mut().zip(&channels) {
        *slot = channel.parse::<u8>().ok()?;
    }
    if let Some(alpha) = parts.next() {
        let pct = alpha.trim().strip_suffix('%')?.trim().parse::<f32>().ok()?;
        rgba[3] = (pct / 100.0 * 255.0).round() as u8;
    }
    Some(rgba)
}

/// Parse a rendered numeric text — bare (`500`) or unit-carrying
/// (`500ms`, `1rem`, `50%`): the trailing unit strips, the leading
/// number parses. `rgb(...)`/keywords parse to nothing (the comparison
/// stays textual there).
fn parse_f32(text: &str) -> Option<f32> {
    let text = text.trim();
    let cut = text
        .find(|c: char| !(c.is_ascii_digit() || c == '-' || c == '.' || c == '+'))
        .unwrap_or(text.len());
    if cut == 0 {
        return None;
    }
    let (head, unit) = text.split_at(cut);
    let value = head.parse::<f32>().ok()?;
    match unit {
        "" | "px" | "rem" | "%" | "deg" | "ms" | "s" | "fr" | "vh" | "vw" => Some(value),
        _ => None,
    }
}

/// The `(condition, longhand) → row text` map the interp's keyword
/// slots resolve through — the annotation seam's own walk
/// ([`keyword_texts`]), the LAST class winning per key (the cascade's
/// own rule; empty texts skipped, exactly the CLI's fold).
fn keyword_map(
    input: &str,
    config: Config,
) -> Result<HashMap<(Cond, Prop), &'static str>, CompileError> {
    let annotations = keyword_texts(input, &config)?;
    let mut map = HashMap::new();
    for class in annotations {
        for (prop, text) in class.keywords {
            if !text.is_empty() {
                map.insert((class.cond, prop), text);
            }
        }
    }
    Ok(map)
}
