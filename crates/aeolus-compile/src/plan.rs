//! The plan-lowering emit arm (`Target::Plan` — architecture plan §7's
//! migration seam): the V1 pipeline's compiled artifact lowered into
//! the v0.x [`StylePlan`](aeolus_core::StylePlan) the five backend
//! crates already consume.
//!
//! ```text
//! emit_plan(input, &Config)                    -> Result<PlanSheet, CompileError>
//! emit_plan_theme(input, &TokenTable, &Config) -> Result<PlanSheet, CompileError>
//!
//! PlanSheet.plan ─ the v0.x StylePlan (base + layers + dark +
//!                  responsive + group/peer + rules), ready for
//!                  aeolus-xilem / -floem / -iced / -egui / -slint
//! PlanSheet.diagnostics ─ the SAME spanned, coded, helped reports the
//!                  IR walk produces (the walks share the lexer
//!                  channel, the variant fold, the read side, the value
//!                  pass and the refusal set)
//! PlanSheet.notes ─ the TYPED boundary set: the longhands that lower
//!                  into the IR fine but have no v0.x plan slot, one
//!                  note per boundary, never a silent drop
//! ```
//!
//! # Why the arm runs IN the pipeline
//!
//! The load-bearing design choice, the same one the CSS and Rust-tokens
//! arms documented: the IR's [`Val::Kw`] slot indexes the ROW'S OWN
//! keyword table (`flex-col` → `Kw(0)` → the row's `FLEX_DIRECTIONS`
//! entry), so a bare `Program` cannot recover the text — or the v0.x
//! enum variant — the class lowered to. THIS arm renders where the rows
//! still talk: per candidate it walks the value pass's typed [`Decl`]s
//! plus the row-table keyword texts, and writes the v0.x slots from
//! BOTH.
//!
//! # The two generations' colour story
//!
//! The v0.x plan carries Oklab [`Color`]s; the pipeline's theme pass
//! bakes `[u8; 4]` sRGB. The bridge converts the baked table through
//! the audited `aeolus_core::Color` parser — both sides mix opacity in
//! Oklab (the same premultiplied identity), so a baked table's colour
//! and the v0.x resolver's colour agree inside the `/255` quantization
//! step. `tests/plan_bridge.rs` pins that agreement differentially:
//! for every battery class, `emit_plan_theme`'s plan must agree slot
//! for slot with the v0.x `Resolver` fed the SAME theme CSS — the P1
//! exit criterion's plan-level form ("new compiler == old resolver").
//!
//! # The boundary set (the notes)
//!
//! The v0.x plan is a TYPED slot set, and some row families lower into
//! the IR fine but have no honest slot to land in:
//!
//! * [`Prop::BoxShadow`] — the row carries the preset CSS text (the
//!   `--tw-shadow` machinery); the v0.x slot wants a parsed
//!   [`BoxShadow`](aeolus_core::BoxShadow) layer. Parsing the preset
//!   text is its own slice; the note names it.
//! * [`Prop::BackgroundImage`] — the gradient rows carry row CSS text;
//!   the v0.x [`Gradient`](aeolus_core::Gradient) slot wants composed
//!   stops. The CSS arm renders the real output today.
//! * the slot-less longhands (`PlaceContent`, `OutlineStyle`,
//!   `OverscrollX`/`Y`, `ObjectFit`, `Columns`, ... — everything the
//!   v0.x `ResolvedStyle`/`Extras` surface does not carry) — the
//!   [`PlanNoteKind::SlotUnsupported`] note names the prop.
//! * the `has-*` inners beyond the state/data/aria forms — the
//!   [`PlanNoteKind::HasFragment`] note names the fragment.
//!
//! Everything else in the registry's Prop surface lands: the per-edge
//! families ride the v0.x `EdgeSet` masks, the per-corner families the
//! `CornerSet`, the composite `text-sm` resolves through its row table
//! to the px pair, the runtime tokens resolve through the theme pass's
//! table, and the fold's conditions route into the v0.x buckets — the
//! seven classic states into `layers`, `dark:` into `dark`, `sm:`/`min-*:`
//! into `responsive`, `group-*:`/`peer-*:` into the label vecs, and the
//! attribute probes, `max-*:`, container queries and `has-*` into
//! `rules` over the v0.x `Atom` conjunction.
//!
//! What this slice deliberately is NOT: the backend METHOD calls (the
//! P3 codegen crates' surface, per-backend) and the theme-remapped
//! radius scale (the registry's radius row carries the v4 defaults; a
//! theme that re-maps `--radius-*` — shadcn's `calc(var(--radius) -
//! 2px)` — diverges by design until the theme pass carries radii; the
//! differential battery pins radii under the v4-default theme, where
//! the two generations agree exactly).

use std::fmt;

use aeolus_core::ast::{PseudoElement, State};
use aeolus_core::cond::{Atom, Cond as V0Cond, Rule, Test};
use aeolus_core::plan::{
    AlignItems, AutoEdges, BorderStyle as V0BorderStyle, BorderWidth, ContentAlign,
    Cursor as V0Cursor, Display as V0Display, FlexDirection as V0FlexDirection,
    JustifyContent as V0JustifyContent, Overflow, OverflowKind, Padding, Position as V0Position,
    ResolvedStyle, SizedValue, StylePlan, VariantCondition, Visibility as V0Visibility,
};
use aeolus_core::{Breakpoint, Color, PointerEvents, TextProperty, TextValue};
use aeolus_data::registry::lookup_detailed;
use aeolus_ir::{Bp, Cond, Decl, Prop, Val};
use aeolus_syntax::parse_class_string;
use aeolus_theme::TokenTable;

use crate::custom;
use crate::lower::{self, Reject};
use crate::variant;
use crate::{
    CompileError, Config, Customs, Diagnostic, DiagnosticKind, LookupError, lookup_diagnostic,
    normalize_modifier, reject_diagnostic, variant_diagnostic,
};

// ---------------------------------------------------------------------------
// The artifact: PlanSheet, PlanNote
// ---------------------------------------------------------------------------

/// The emit artifact (the `Target::Plan` form of one class string): the
/// v0.x plan the backends consume, plus the SAME honest per-class
/// diagnostics the IR form reports and the typed boundary notes this
/// arm adds. The whole sheet is `PartialEq` — equal inputs produce
/// equal artifacts, like every emit form.
#[derive(Clone, Debug, PartialEq)]
pub struct PlanSheet {
    /// The lowered plan: base plus the conditional buckets, ready for
    /// the backend lowering each crate already performs.
    pub plan: StylePlan,
    /// One entry per class that did NOT lower, in source order — the
    /// same reports [`compile`](crate::compile) produces.
    pub diagnostics: Vec<Diagnostic>,
    /// One entry per class that lowered into the IR but has no honest
    /// v0.x slot, in source order — the arm's typed boundary set.
    pub notes: Vec<PlanNote>,
}

/// One boundary report: the class lowered, but the longhand it produced
/// has no v0.x plan slot. These are NOTES, not diagnostics: the plan is
/// still the string's result, the note says what the plan does NOT
/// carry (the interp arm's [`InterpNote`](crate::InterpNote) precedent).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlanNote {
    /// The offending class's byte range inside the INPUT string.
    pub span: (usize, usize),
    /// The class text, verbatim.
    pub raw: String,
    /// What the plan could not carry, and why.
    pub kind: PlanNoteKind,
}

impl fmt::Display for PlanNote {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {} [{}]",
            self.raw,
            self.kind.help(),
            self.kind.code()
        )
    }
}

/// Why the plan does not carry one longhand — the typed boundary set,
/// one variant per family, with the stable machine code and help text a
/// linter renders (the same contract the diagnostics carry).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlanNoteKind {
    /// The longhand has no slot in the v0.x `ResolvedStyle`/`Extras`
    /// surface (`PlaceContent`, `OutlineStyle`, `OverscrollX`, ...).
    /// The lowering is correct in the IR; the v0.x shape is the limit.
    SlotUnsupported {
        /// The longhand the plan cannot carry.
        prop: Prop,
    },
    /// The row keyword's text is outside the v0.x vocabulary for its
    /// slot (`items-normal` on the five-variant enum). The typed IR
    /// carries it; the enum arm does not exist yet.
    RowTextUnmappable {
        /// The longhand the text belongs to.
        prop: Prop,
        /// The row's own text, verbatim.
        text: &'static str,
    },
    /// The gradient rows lower into `BackgroundImage` as row CSS text;
    /// the v0.x gradient slot wants composed stops. The CSS arm renders
    /// this class's real output today.
    GradientRow,
    /// The shadow preset rows lower into `BoxShadow` as the preset CSS
    /// text; the v0.x slot wants a parsed `BoxShadow` layer.
    ShadowPreset,
    /// The `has-*` inner fragment is selector text whose shape the v0.x
    /// `Test` surface cannot carry (the state/data/aria inners map; the
    /// rest — structural selectors, tags — report here).
    HasFragment {
        /// The rendered `:has(...)` inner text, verbatim.
        inner: &'static str,
    },
}

impl PlanNoteKind {
    /// The stable machine code (`aeolus::compile::plan::*`) a linter or
    /// editor keys on.
    pub fn code(&self) -> &'static str {
        match self {
            PlanNoteKind::SlotUnsupported { .. } => "aeolus::compile::plan::slot_unsupported",
            PlanNoteKind::RowTextUnmappable { .. } => "aeolus::compile::plan::row_text_unmappable",
            PlanNoteKind::GradientRow => "aeolus::compile::plan::gradient_row",
            PlanNoteKind::ShadowPreset => "aeolus::compile::plan::shadow_preset",
            PlanNoteKind::HasFragment { .. } => "aeolus::compile::plan::has_fragment",
        }
    }

    /// The actionable help text, stable per variant.
    pub fn help(&self) -> &'static str {
        match self {
            PlanNoteKind::SlotUnsupported { .. } => {
                "the longhand lowers into the IR correctly; the v0.x plan shape has no \
                 slot for it — the class stays reported, the rest of the plan is usable"
            }
            PlanNoteKind::RowTextUnmappable { .. } => {
                "the row's keyword text is outside the v0.x enum vocabulary for this \
                 slot; the IR carries the typed value, the plan carries the rest"
            }
            PlanNoteKind::GradientRow => {
                "the gradient lowers into BackgroundImage as row CSS text; the v0.x \
                 Gradient slot wants composed stops — emit_css renders the real output"
            }
            PlanNoteKind::ShadowPreset => {
                "the shadow preset lowers into BoxShadow as the preset CSS text; the \
                 v0.x slot wants a parsed layer — the CSS arm renders the real output"
            }
            PlanNoteKind::HasFragment { .. } => {
                "the has-* inner is selector text outside the v0.x Test surface's \
                 state/data/aria forms; the rule reports instead of misfiring"
            }
        }
    }
}

// ---------------------------------------------------------------------------
// The entry points (the emit_css family's shape)
// ---------------------------------------------------------------------------

/// Render the `Target::Plan` form of a class string WITHOUT the theme
/// pass: colour tokens and opacity modifiers stay typed boundaries (the
/// same honest default [`compile`](crate::compile) has) — the baked
/// palette colours and every non-token utility lower.
///
/// ```no_run
/// use aeolus_compile::{emit_plan, Config};
///
/// let sheet = emit_plan("p-4 hover:px-8", &Config::default())?;
/// // The plan shape the backends already consume:
/// println!("{:?}", sheet.plan.base.padding);
/// // Every class that lowered but has no v0.x slot comes back as a
/// // typed note — never a silent drop:
/// for note in &sheet.notes {
///     println!("{note}");
/// }
/// # Ok::<(), aeolus_compile::CompileError>(())
/// ```
///
/// # Errors
/// The same [`CompileError`] as [`compile`](crate::compile) (only the
/// empty input). Per-class reports never fail an emit.
pub fn emit_plan(input: &str, config: &Config) -> Result<PlanSheet, CompileError> {
    run_plan(input, *config, None, &Customs::new())
}

/// [`emit_plan`] with the custom-authoring seam supplied (plan §5.4's
/// `@utility`/`@custom-variant` surface): registered custom utilities
/// lower into the plan through the same value pass the registry rows
/// take. With EMPTY registrations every contract is byte-identical to
/// [`emit_plan`] (the differential pins it).
///
/// ```no_run
/// use aeolus_compile::{emit_plan_with, Config, CustomUtility, Customs};
/// use aeolus_ir::{Prop, Val};
///
/// let customs = Customs::new().utility(CustomUtility::fixed(
///     "brand-box",
///     Val::Rem(1.5),
///     &[Prop::PaddingTop, Prop::PaddingBottom],
/// ));
/// let sheet = emit_plan_with("brand-box", &customs, &Config::default())?;
/// assert!(sheet.diagnostics.is_empty());
/// # Ok::<(), aeolus_compile::CompileError>(())
/// ```
///
/// # Errors
/// The same [`CompileError`] as [`emit_plan`] (only the empty input).
pub fn emit_plan_with(
    input: &str,
    customs: &Customs,
    config: &Config,
) -> Result<PlanSheet, CompileError> {
    run_plan(input, *config, None, customs)
}

/// Render the `Target::Plan` form of a class string WITH the theme
/// pass: colour tokens and `/opacity` modifiers lower through the
/// tiered [`TokenTable`], and the runtime token slots resolve through
/// its table — the differential battery's themed half.
///
/// ```no_run
/// use aeolus_compile::{emit_plan_theme, Config};
/// use aeolus_theme::{ThemeSource, TokenTable};
///
/// let source = ThemeSource::shadcn()?;
/// let tokens = TokenTable::baked(&source)?;
/// let sheet = emit_plan_theme(
///     "p-4 bg-primary text-primary-foreground hover:bg-primary/90",
///     &tokens,
///     &Config::default(),
/// )?;
/// assert!(sheet.diagnostics.is_empty());
/// assert!(sheet.plan.base.background.is_some());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
/// The same [`CompileError`] as [`emit_plan_theme`] (only the empty
/// input).
pub fn emit_plan_theme(
    input: &str,
    tokens: &TokenTable,
    config: &Config,
) -> Result<PlanSheet, CompileError> {
    run_plan(input, *config, Some(tokens), &Customs::new())
}

/// [`emit_plan_theme`] with the custom-authoring seam supplied: the
/// theme pass AND the custom registrations lower through the same
/// pipeline (see [`compile_theme_with`](crate::compile_theme_with)).
///
/// # Errors
/// The same [`CompileError`] as [`emit_plan_theme`] (only the empty
/// input).
pub fn emit_plan_theme_with(
    input: &str,
    tokens: &TokenTable,
    customs: &Customs,
    config: &Config,
) -> Result<PlanSheet, CompileError> {
    run_plan(input, *config, Some(tokens), customs)
}

// ---------------------------------------------------------------------------
// The walk (the run shape, one sink swap)
// ---------------------------------------------------------------------------

fn run_plan(
    input: &str,
    config: Config,
    tokens: Option<&TokenTable>,
    customs: &Customs,
) -> Result<PlanSheet, CompileError> {
    #[cfg(feature = "tracing")]
    tracing::debug!(input_len = input.len(), "emit_plan: lex -> lookup -> plan");

    if input.trim().is_empty() {
        return Err(CompileError::Empty);
    }

    // Stage 1: the zero-copy lexer (the typed degraded channel — the
    // SAME spans and kinds the other arms report).
    let (candidates, degraded) = parse_class_string(input);
    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    for failure in &degraded {
        diagnostics.push(Diagnostic {
            span: failure.span,
            raw: failure.raw.to_string(),
            kind: DiagnosticKind::Lex { kind: failure.kind },
        });
    }

    let mut sheet = PlanSheet {
        plan: StylePlan::default(),
        diagnostics,
        notes: Vec::new(),
    };

    for candidate in &candidates {
        if candidate.important {
            sheet.diagnostics.push(Diagnostic::new(
                candidate.span,
                candidate.raw,
                DiagnosticKind::UnsupportedImportant,
            ));
            continue;
        }
        // The variant fold: the SAME typed surface and refusal set the
        // IR walk applies (the customs' aliases ride the classifier).
        let cond = match variant::fold(&candidate.variants, customs) {
            Ok(cond) => cond,
            Err(miss) => {
                sheet.diagnostics.push(variant_diagnostic(candidate, &miss));
                continue;
            }
        };

        // The registry read side (byte-shared with the other arms): the
        // fraction re-attachment first, the modifier gate after — the
        // audited oracle's F1 precedence, byte-identical to the walks.
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
                        sheet.diagnostics.push(Diagnostic::new(
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

        // The value pass: the IR walk's own typed decls, plus — per row
        // — the keyword texts the `Val::Kw` slots resolve through and
        // the accepted suffix the per-prop spellings read (`outline`).
        // One entry per accepted declaration: the typed value, the row
        // table's keyword text (the `Val::Kw` annotation), and whether
        // the accepted suffix is the `none` spelling (the per-prop form
        // the row text cannot distinguish — `outline-none`).
        let mut written: Vec<(Decl, Option<&'static str>, bool)> = Vec::new();
        let mut deferred: Vec<&'static str> = Vec::new();
        let mut reported = false;
        match rows {
            Ok(rows) => {
                for row in rows {
                    let annotations = lower::kw_annotations(*row, effective_body);
                    let suffix = lower::suffix_of(effective_body, row.root).ok().flatten();
                    match lower::row_to_decls(
                        *row,
                        effective_body,
                        candidate.negative,
                        alpha,
                        tokens,
                        config,
                        candidate.arbitrary,
                    ) {
                        Ok(produced) => {
                            for decl in produced {
                                let text = annotations
                                    .iter()
                                    .find(|(prop, _)| *prop == decl.prop)
                                    .map(|&(_, text)| text);
                                written.push((decl, text, suffix == Some("none")));
                            }
                        }
                        Err(reject) => {
                            reported = true;
                            if let (true, Reject::Value { reason }) =
                                (candidate.arbitrary.is_some(), &reject)
                                && crate::arbitrary::is_boundary_reason(reason)
                            {
                                if !deferred.contains(reason) {
                                    deferred.push(reason);
                                }
                                continue;
                            }
                            sheet.diagnostics.push(reject_diagnostic(
                                candidate,
                                effective_body,
                                reject,
                                candidate.modifier,
                            ));
                        }
                    }
                }
                if candidate.arbitrary.is_some() && written.is_empty() {
                    if !deferred.is_empty() {
                        for reason in deferred {
                            sheet.diagnostics.push(Diagnostic::new(
                                candidate.span,
                                candidate.raw,
                                DiagnosticKind::UnsupportedValue {
                                    body: effective_body.to_string(),
                                    reason,
                                },
                            ));
                        }
                    } else if !reported {
                        sheet.diagnostics.push(Diagnostic::new(
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
            // surface consulted on the unknown-root miss only — the
            // SAME read-side rule the other arms apply.
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
                            Ok(produced) => {
                                for decl in produced {
                                    written.push((decl, None, false));
                                }
                            }
                            Err(reject) => {
                                sheet.diagnostics.push(reject_diagnostic(
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
                    sheet.diagnostics.push(lookup_diagnostic(candidate, err));
                    continue;
                }
            },
        }
        if written.is_empty() {
            // Every row refused: the reports ride, no style lands.
            continue;
        }

        // The slot write: DIRECTLY into the candidate's bucket — the
        // v0.x resolver applies every class into one plan (per-edge
        // masks and partial slots accumulate field-wise; `ring-2
        // ring-red-500` is ONE ring carrying both), so the bridge
        // writes into the same bucket the fold names. Unmappable
        // `has-*` fragments land in the notes.
        let mut writer = PlanWriter {
            config,
            notes: Vec::new(),
            font_size_px: None,
            gap_row: None,
            gap_col: None,
        };
        {
            let notes = &mut sheet.notes;
            bucket(
                &mut sheet.plan,
                cond,
                candidate.span,
                candidate.raw,
                notes,
                |style| {
                    for (decl, text, none_spelling) in &written {
                        writer.write(style, decl.prop, decl.val, *text, *none_spelling);
                    }
                    writer.finish(style);
                },
            );
        }
        for kind in writer.notes {
            sheet.notes.push(PlanNote {
                span: candidate.span,
                raw: candidate.raw.to_string(),
                kind,
            });
        }
    }

    Ok(sheet)
}

// ---------------------------------------------------------------------------
// The slot writer: Prop + Val (+ row text) -> v0.x slots
// ---------------------------------------------------------------------------

/// The per-candidate slot writer: one arm per registry longhand family,
/// the row table's keyword text resolving the `Val::Kw` slots the bare
/// IR cannot. Boundary notes accumulate un-stamped — the walk stamps
/// the candidate's span and class text on them.
struct PlanWriter {
    config: Config,
    notes: Vec<PlanNoteKind>,
    /// The candidate's font size in px, when the composite's first
    /// target lowered (`text-sm`'s 14px) — the line-height ratio's
    /// multiplier (the v0.x `text-sm` → 20px semantics).
    font_size_px: Option<f32>,
    /// The candidate's gap pair (the v0.x oracle's split — both axes
    /// equal → the `Gap` slot, a single axis → `GapAxes` — needs the
    /// pair, so the gap writes defer to `finish`).
    gap_row: Option<f32>,
    gap_col: Option<f32>,
}

impl PlanWriter {
    /// One declaration into the v0.x style. `text` is the row table's
    /// keyword text for `Val::Kw` slots (the annotation pass),
    /// `none_spelling` the accepted-suffix flag for the per-prop forms
    /// the row text cannot distinguish (`outline-none` rides the width
    /// row's `0px`).
    fn write(
        &mut self,
        style: &mut ResolvedStyle,
        prop: Prop,
        val: Val,
        text: Option<&'static str>,
        none_spelling: bool,
    ) {
        // The keyword slot's effective text: the row table's annotation,
        // present exactly for the Kw forms.
        let kw = match val {
            Val::Kw(_) => text,
            _ => None,
        };
        match prop {
            // ---- the per-edge families (the EdgeSet masks) ----------
            Prop::PaddingTop | Prop::PaddingRight | Prop::PaddingBottom | Prop::PaddingLeft => {
                if let Some(px) = self.length_px(val, kw) {
                    let slot = style.padding.get_or_insert_with(Padding::default);
                    set_edge(slot, edge_of(prop), px);
                }
            }
            Prop::MarginTop | Prop::MarginRight | Prop::MarginBottom | Prop::MarginLeft => {
                // `mx-auto`'s keyword lands in the auto-edge slot (the
                // v0.x oracle's own split: the margin slot stays unset).
                if matches!(val, Val::Kw(_)) {
                    if kw == Some("auto") {
                        let slot = style
                            .extras
                            .margin_auto
                            .get_or_insert_with(AutoEdges::default);
                        set_auto_edge(slot, edge_of(prop));
                    }
                    return;
                }
                if let Some(px) = self.length_px(val, kw) {
                    let slot = style.margin.get_or_insert_with(Padding::default);
                    set_edge(slot, edge_of(prop), px);
                }
            }
            Prop::BorderTopWidth
            | Prop::BorderRightWidth
            | Prop::BorderBottomWidth
            | Prop::BorderLeftWidth => {
                // The numeric scale arrives as `Num` (`border-2`), the
                // keyword forms as row px texts (`border`-family arms).
                if let Some(px) = self.number(val).or_else(|| self.length_px(val, kw)) {
                    let slot = style.border_width.get_or_insert_with(BorderWidth::default);
                    set_edge(slot, edge_of(prop), px);
                }
            }
            Prop::BorderTopLeftRadius
            | Prop::BorderTopRightRadius
            | Prop::BorderBottomRightRadius
            | Prop::BorderBottomLeftRadius => {
                if let Some(px) = self.length_px(val, kw) {
                    let slot = style
                        .corner_radius
                        .get_or_insert_with(aeolus_core::CornerRadius::default);
                    let (value, corner) = (px, corner_of(prop));
                    set_corner(slot, corner, value);
                    set_corner_set(&mut slot.set, corner, true);
                }
            }
            // ---- colours --------------------------------------------
            Prop::Background => {
                if let Val::Rgba(rgba) = val {
                    style.background = Some(aeolus_core::Background(oklab(rgba)));
                }
            }
            Prop::Color => {
                if let Val::Rgba(rgba) = val {
                    style.content_color = Some(aeolus_core::ContentColor(oklab(rgba)));
                }
            }
            Prop::BorderColor => {
                if let Val::Rgba(rgba) = val {
                    style.border_color = Some(aeolus_core::BorderColor(oklab(rgba)));
                }
            }
            Prop::RingColor => {
                if let Val::Rgba(rgba) = val {
                    style
                        .ring
                        .get_or_insert_with(aeolus_core::Ring::default)
                        .color = Some(oklab(rgba));
                }
            }
            Prop::RingOffsetColor => {
                if let Val::Rgba(rgba) = val {
                    style
                        .ring
                        .get_or_insert_with(aeolus_core::Ring::default)
                        .offset_color = Some(oklab(rgba));
                }
            }
            Prop::OutlineColor => {
                if let Val::Rgba(rgba) = val {
                    style
                        .outline
                        .get_or_insert_with(aeolus_core::Outline::default)
                        .color = Some(oklab(rgba));
                }
            }
            // ---- ring / outline widths ------------------------------
            Prop::RingWidth => {
                if let Some(px) = self.number(val) {
                    style
                        .ring
                        .get_or_insert_with(aeolus_core::Ring::default)
                        .width = px;
                }
            }
            Prop::RingOffsetWidth => {
                if let Some(px) = self.number(val) {
                    style
                        .ring
                        .get_or_insert_with(aeolus_core::Ring::default)
                        .offset_width = px;
                }
            }
            Prop::OutlineWidth => {
                // `outline-none` rides the same width row (`0px`); the
                // accepted suffix is what the v0.x oracle distinguishes
                // on (`outline-0` → width, `outline-none` → the flag).
                if none_spelling {
                    style
                        .outline
                        .get_or_insert_with(aeolus_core::Outline::default)
                        .none = true;
                } else if let Some(px) = self.number(val) {
                    style
                        .outline
                        .get_or_insert_with(aeolus_core::Outline::default)
                        .width = px;
                }
            }
            Prop::OutlineOffset => {
                if let Some(px) = self.number(val) {
                    style
                        .outline
                        .get_or_insert_with(aeolus_core::Outline::default)
                        .offset = px;
                }
            }
            Prop::OutlineStyle => {
                // The v0.x Outline slot has no style field.
                self.notes.push(PlanNoteKind::SlotUnsupported { prop });
            }
            // ---- dimensions -----------------------------------------
            Prop::JustifySelf | Prop::ShadowColor => {
                // `justify-self` (the v0.x Layout carries no self-justify
                // field) and the shadow colour (the parsed-layer
                // boundary the BoxShadow note owns) — the slot-less
                // longhands' honest set.
                self.notes.push(PlanNoteKind::SlotUnsupported { prop });
            }
            Prop::Width
            | Prop::Height
            | Prop::MinWidth
            | Prop::MinHeight
            | Prop::MaxWidth
            | Prop::MaxHeight => {
                if let Some(sized) = self.sized(val, kw) {
                    let slot = match prop {
                        Prop::Width | Prop::Height => &mut style.dimensions,
                        Prop::MinWidth | Prop::MinHeight => &mut style.min_dimensions,
                        _ => &mut style.max_dimensions,
                    };
                    let slot = slot.get_or_insert_with(aeolus_core::Dimensions::default);
                    match prop {
                        Prop::Width | Prop::MinWidth | Prop::MaxWidth => slot.width = Some(sized),
                        _ => slot.height = Some(sized),
                    }
                }
            }
            // ---- gap (deferred to `finish`) --------------------------
            Prop::GapRow => {
                if let Some(px) = self.length_px(val, kw) {
                    self.gap_row = Some(px);
                }
            }
            Prop::GapCol => {
                if let Some(px) = self.length_px(val, kw) {
                    self.gap_col = Some(px);
                }
            }
            // ---- layout ----------------------------------------------
            Prop::Display => {
                if let Some(text) = kw {
                    match display_kw(text) {
                        Some(display) => {
                            style
                                .layout
                                .get_or_insert_with(aeolus_core::Layout::default)
                                .display = Some(display);
                        }
                        None => self
                            .notes
                            .push(PlanNoteKind::RowTextUnmappable { prop, text }),
                    }
                }
            }
            Prop::FlexDirection => {
                if let Some(text) = kw {
                    match flex_direction_kw(text) {
                        Some(dir) => {
                            let slot = style
                                .layout
                                .get_or_insert_with(aeolus_core::Layout::default);
                            slot.flex_direction = Some(dir);
                            // The v0.x oracle's own reading: a
                            // flex-direction implies the flex display
                            // (`flex-col` carries both) — the
                            // differential pins it.
                            slot.display = Some(V0Display::Flex);
                        }
                        None => self
                            .notes
                            .push(PlanNoteKind::RowTextUnmappable { prop, text }),
                    }
                }
            }
            Prop::FlexWrap => {
                if let Some(text) = kw {
                    let slot = style
                        .layout
                        .get_or_insert_with(aeolus_core::Layout::default);
                    match text {
                        "wrap" => slot.flex_wrap = Some(true),
                        "nowrap" => slot.flex_wrap = Some(false),
                        // `wrap-reverse` has no bool slot in the v0.x
                        // layout (the enum-less field).
                        _ => self
                            .notes
                            .push(PlanNoteKind::RowTextUnmappable { prop, text }),
                    }
                }
            }
            Prop::FlexGrow => {
                // The bare forms are keyword slots too (`grow` → Bare("1")
                // → Kw(0)); the row text parses to the factor.
                if let Some(n) = self
                    .number(val)
                    .or_else(|| kw.and_then(|text| text.parse::<f32>().ok()))
                {
                    style
                        .layout
                        .get_or_insert_with(aeolus_core::Layout::default)
                        .flex_grow = Some(n);
                }
            }
            Prop::FlexShrink => {
                if let Some(n) = self
                    .number(val)
                    .or_else(|| kw.and_then(|text| text.parse::<f32>().ok()))
                {
                    style
                        .layout
                        .get_or_insert_with(aeolus_core::Layout::default)
                        .flex_shrink = Some(n);
                }
            }
            Prop::FlexBasis => {
                if let Some(sized) = self.sized(val, kw) {
                    style.extras.flex_basis = Some(sized);
                }
            }
            Prop::JustifyContent => {
                if let Some(text) = kw {
                    match justify_kw(text) {
                        Some(j) => {
                            style
                                .layout
                                .get_or_insert_with(aeolus_core::Layout::default)
                                .justify_content = Some(j);
                        }
                        None => self
                            .notes
                            .push(PlanNoteKind::RowTextUnmappable { prop, text }),
                    }
                }
            }
            Prop::AlignItems | Prop::JustifyItems => {
                if let Some(text) = kw {
                    match items_kw(text) {
                        Some(a) => {
                            let slot = style
                                .layout
                                .get_or_insert_with(aeolus_core::Layout::default);
                            if prop == Prop::AlignItems {
                                slot.align_items = Some(a);
                            } else {
                                slot.justify_items = Some(a);
                            }
                        }
                        None => self
                            .notes
                            .push(PlanNoteKind::RowTextUnmappable { prop, text }),
                    }
                }
            }
            Prop::AlignSelf => {
                if let Some(text) = kw {
                    match items_kw(text) {
                        Some(a) => {
                            style
                                .layout
                                .get_or_insert_with(aeolus_core::Layout::default)
                                .align_self = Some(a);
                        }
                        None => self
                            .notes
                            .push(PlanNoteKind::RowTextUnmappable { prop, text }),
                    }
                }
            }
            Prop::AlignContent => {
                if let Some(text) = kw {
                    match content_kw(text) {
                        Some(a) => style.extras.align_content = Some(a),
                        None => self
                            .notes
                            .push(PlanNoteKind::RowTextUnmappable { prop, text }),
                    }
                }
            }
            // The place-* families: the v0.x typed table has no slot
            // (the oracle warns on them too — the diagnostics-equal
            // surface).
            Prop::PlaceContent | Prop::PlaceItems | Prop::PlaceSelf => {
                self.notes.push(PlanNoteKind::SlotUnsupported { prop });
            }
            // ---- position & extras ----------------------------------
            Prop::Position => {
                if let Some(text) = kw {
                    match position_kw(text) {
                        Some(p) => style.extras.position = Some(p),
                        None => self
                            .notes
                            .push(PlanNoteKind::RowTextUnmappable { prop, text }),
                    }
                }
            }
            Prop::Top | Prop::Right | Prop::Bottom | Prop::Left => {
                let sized = match val {
                    Val::Kw(_) => Some(SizedValue::Auto),
                    _ => self.sized(val, kw),
                };
                if let Some(sized) = sized {
                    let slot = style
                        .extras
                        .inset
                        .get_or_insert_with(aeolus_core::Inset::default);
                    match prop {
                        Prop::Top => slot.top = Some(sized),
                        Prop::Right => slot.right = Some(sized),
                        Prop::Bottom => slot.bottom = Some(sized),
                        _ => slot.left = Some(sized),
                    }
                }
            }
            // The logical inset longhands: no v0.x slot (the v0.x
            // carries the physical edges only).
            Prop::InsetInlineStart | Prop::InsetInlineEnd => {
                self.notes.push(PlanNoteKind::SlotUnsupported { prop });
            }
            Prop::ZIndex => match val {
                Val::Num(n) => style.extras.z_index = Some(n as i32),
                Val::Kw(_) => {
                    let text = kw.unwrap_or("auto");
                    self.notes
                        .push(PlanNoteKind::RowTextUnmappable { prop, text });
                }
                _ => {}
            },
            Prop::Order => {
                if let Val::Num(n) = val {
                    style.extras.order = Some(n as i16);
                }
            }
            Prop::OverflowX | Prop::OverflowY => {
                if let Some(text) = kw {
                    match overflow_kw(text) {
                        Some(kind) => {
                            let slot = style.extras.overflow.get_or_insert_with(Overflow::default);
                            if prop == Prop::OverflowX {
                                slot.x = Some(kind);
                            } else {
                                slot.y = Some(kind);
                            }
                        }
                        None => self
                            .notes
                            .push(PlanNoteKind::RowTextUnmappable { prop, text }),
                    }
                }
            }
            // The slot-less longhands: the v0.x surface carries no
            // native slot — the note names each (the honest boundary).
            Prop::OverscrollX
            | Prop::OverscrollY
            | Prop::Columns
            | Prop::ObjectFit
            | Prop::ObjectPosition
            | Prop::Stroke
            | Prop::Fill
            | Prop::Isolation
            | Prop::WordBreak
            | Prop::VerticalAlign
            | Prop::Resize
            | Prop::ContainerType
            | Prop::TextOverflow => {
                self.notes.push(PlanNoteKind::SlotUnsupported { prop });
            }
            Prop::Visibility => {
                if let Some(text) = kw {
                    match text {
                        "hidden" => style.extras.visibility = Some(V0Visibility::Hidden),
                        "visible" => style.extras.visibility = Some(V0Visibility::Visible),
                        _ => self
                            .notes
                            .push(PlanNoteKind::RowTextUnmappable { prop, text }),
                    }
                }
            }
            Prop::BorderStyle => {
                if let Some(text) = kw {
                    match text {
                        "solid" => style.extras.border_style = Some(V0BorderStyle::Solid),
                        "dashed" => style.extras.border_style = Some(V0BorderStyle::Dashed),
                        "dotted" => style.extras.border_style = Some(V0BorderStyle::Dotted),
                        "double" => style.extras.border_style = Some(V0BorderStyle::Double),
                        "none" => style.extras.border_style = Some(V0BorderStyle::None),
                        _ => self
                            .notes
                            .push(PlanNoteKind::RowTextUnmappable { prop, text }),
                    }
                }
            }
            Prop::AspectRatio => match val {
                Val::Kw(_) => {
                    let text = kw.unwrap_or("auto");
                    if text == "auto" {
                        // The auto form clears the slot (the oracle's
                        // `aspect-auto` reading).
                        style.extras.aspect_ratio = None;
                    } else if let Some(r) = ratio_text(text) {
                        style.extras.aspect_ratio = Some(r);
                    }
                }
                Val::Num(n) => style.extras.aspect_ratio = Some(n),
                _ => {}
            },
            Prop::Cursor => {
                if let Some(text) = kw {
                    match cursor_kw(text) {
                        Some(c) => style.cursor = Some(c),
                        None => self
                            .notes
                            .push(PlanNoteKind::RowTextUnmappable { prop, text }),
                    }
                }
            }
            Prop::PointerEvents => {
                if let Some(text) = kw {
                    match text {
                        "none" => style.pointer_events = Some(PointerEvents::None),
                        "auto" => style.pointer_events = Some(PointerEvents::Auto),
                        _ => self
                            .notes
                            .push(PlanNoteKind::RowTextUnmappable { prop, text }),
                    }
                }
            }
            Prop::Opacity => match val {
                Val::Num(n) => {
                    // The row's percent scale (`50` → 0.5); an arbitrary
                    // unitless value at or below 1 is already the factor
                    // (`opacity-[0.6]`) — the differential pins both.
                    let factor = if n > 1.0 { n / 100.0 } else { n };
                    style.opacity = Some(aeolus_core::Opacity(factor));
                }
                Val::Pct(p) => style.opacity = Some(aeolus_core::Opacity(p / 100.0)),
                _ => {}
            },
            // ---- the transform family --------------------------------
            Prop::ScaleX => {
                if let Val::Num(n) = val {
                    style
                        .extras
                        .transform
                        .get_or_insert_with(aeolus_core::Transform::default)
                        .scale_x = Some(n / 100.0);
                }
            }
            Prop::ScaleY => {
                if let Val::Num(n) = val {
                    style
                        .extras
                        .transform
                        .get_or_insert_with(aeolus_core::Transform::default)
                        .scale_y = Some(n / 100.0);
                }
            }
            Prop::Rotate => {
                if let Val::Num(n) = val {
                    style
                        .extras
                        .transform
                        .get_or_insert_with(aeolus_core::Transform::default)
                        .rotate_deg = Some(n);
                }
            }
            Prop::SkewX => {
                if let Val::Num(n) = val {
                    style
                        .extras
                        .transform
                        .get_or_insert_with(aeolus_core::Transform::default)
                        .skew_x = Some(n);
                }
            }
            Prop::SkewY => {
                if let Val::Num(n) = val {
                    style
                        .extras
                        .transform
                        .get_or_insert_with(aeolus_core::Transform::default)
                        .skew_y = Some(n);
                }
            }
            Prop::TranslateX | Prop::TranslateY => {
                let sized = match val {
                    Val::Kw(_) => Some(SizedValue::Auto),
                    _ => self.sized(val, kw),
                };
                if let Some(sized) = sized {
                    let slot = style
                        .extras
                        .transform
                        .get_or_insert_with(aeolus_core::Transform::default);
                    if prop == Prop::TranslateX {
                        slot.translate_x = Some(sized);
                    } else {
                        slot.translate_y = Some(sized);
                    }
                }
            }
            // ---- grid -------------------------------------------------
            Prop::GridTemplateColumns => {
                if let Val::Num(n) = val {
                    style
                        .extras
                        .grid
                        .get_or_insert_with(aeolus_core::Grid::default)
                        .cols = Some(n as u16);
                } else {
                    // The keyword templates (`none`, `subgrid`) and the
                    // arbitrary track lists: the v0.x template parse is
                    // its own surface.
                    self.notes.push(PlanNoteKind::SlotUnsupported { prop });
                }
            }
            Prop::GridTemplateRows => {
                if let Val::Num(n) = val {
                    style
                        .extras
                        .grid
                        .get_or_insert_with(aeolus_core::Grid::default)
                        .rows = Some(n as u16);
                } else {
                    self.notes.push(PlanNoteKind::SlotUnsupported { prop });
                }
            }
            Prop::GridRowStart | Prop::GridColumnStart => {
                if let Val::Num(n) = val {
                    let slot = style
                        .extras
                        .grid
                        .get_or_insert_with(aeolus_core::Grid::default);
                    if prop == Prop::GridRowStart {
                        slot.row_start = Some(n as i16);
                    } else {
                        slot.col_start = Some(n as i16);
                    }
                } else {
                    self.notes.push(PlanNoteKind::SlotUnsupported { prop });
                }
            }
            Prop::GridRowEnd | Prop::GridColumnEnd | Prop::GridAutoRows | Prop::GridAutoColumns => {
                self.notes.push(PlanNoteKind::SlotUnsupported { prop });
            }
            // ---- the shadow & gradient rows (the boundary families) ---
            Prop::BoxShadow => {
                self.notes.push(PlanNoteKind::ShadowPreset);
            }
            Prop::BackgroundImage => {
                self.notes.push(PlanNoteKind::GradientRow);
            }
            // ---- typography (the text property vec) -------------------
            Prop::FontSize => {
                if let Some(px) = self.length_px(val, kw) {
                    self.font_size_px = Some(px);
                    set_text(style, TextProperty::FontSize, TextValue::Px(px));
                } else if let Some(text) = kw {
                    self.notes
                        .push(PlanNoteKind::RowTextUnmappable { prop, text });
                }
            }
            Prop::LineHeight => {
                // The composite's secondary text is the RATIO the v0.x
                // theme multiplies out (`calc(1.25 / 0.875)` → 20px on
                // `text-sm`); the leading rows' texts are the unitless
                // ratios (`leading-tight` → 1.25, element-relative).
                // A length value (the spacing form `leading-6`) bakes
                // to px.
                if let Some(text) = kw {
                    if let Some(ratio) = ratio_text(text) {
                        match self.font_size_px {
                            Some(size) => set_text(
                                style,
                                TextProperty::LineHeight,
                                TextValue::Px(ratio * size),
                            ),
                            None => set_text(
                                style,
                                TextProperty::LineHeight,
                                TextValue::LineHeightRatio(ratio),
                            ),
                        }
                    } else if let Some(px) = self.length_px(val, kw) {
                        set_text(style, TextProperty::LineHeight, TextValue::Px(px));
                    } else {
                        self.notes
                            .push(PlanNoteKind::RowTextUnmappable { prop, text });
                    }
                } else if let Some(px) = self.length_px(val, kw) {
                    set_text(style, TextProperty::LineHeight, TextValue::Px(px));
                }
            }
            Prop::FontWeight => {
                if let Some(text) = kw {
                    match text.parse::<u16>() {
                        Ok(w) => set_text(style, TextProperty::FontWeight, TextValue::Weight(w)),
                        Err(_) => self
                            .notes
                            .push(PlanNoteKind::RowTextUnmappable { prop, text }),
                    }
                } else if let Val::Num(n) = val {
                    set_text(style, TextProperty::FontWeight, TextValue::Weight(n as u16));
                }
            }
            Prop::LetterSpacing => {
                if let Some(text) = kw {
                    match em_text(text) {
                        Some(em) => set_text(
                            style,
                            TextProperty::LetterSpacing,
                            TextValue::TrackingEm(em),
                        ),
                        None => self
                            .notes
                            .push(PlanNoteKind::RowTextUnmappable { prop, text }),
                    }
                } else if let Some(px) = self.length_px(val, kw) {
                    set_text(style, TextProperty::LetterSpacing, TextValue::Spacing(px));
                }
            }
            Prop::TextAlign => {
                if let Some(text) = kw {
                    set_text(style, TextProperty::TextAlign, TextValue::Keyword(text));
                }
            }
            Prop::FontStyle => {
                if let Some(text) = kw {
                    set_text(style, TextProperty::FontStyle, TextValue::Keyword(text));
                }
            }
            Prop::TextTransform => {
                if let Some(text) = kw {
                    set_text(style, TextProperty::TextTransform, TextValue::Keyword(text));
                }
            }
            Prop::TextDecorationLine => {
                if let Some(text) = kw {
                    set_text(
                        style,
                        TextProperty::TextDecoration,
                        TextValue::Keyword(text),
                    );
                }
            }
            Prop::WhiteSpace => {
                if let Some(text) = kw {
                    set_text(style, TextProperty::Whitespace, TextValue::Keyword(text));
                }
            }
            Prop::UserSelect => {
                if let Some(text) = kw {
                    // The v0.x keyword is the full class spelling
                    // (`select-none`), the row text the bare value — the
                    // per-prop mapping, oracle-faithful.
                    match select_kw(text) {
                        Some(full) => {
                            set_text(style, TextProperty::Select, TextValue::Keyword(full));
                        }
                        None => self
                            .notes
                            .push(PlanNoteKind::RowTextUnmappable { prop, text }),
                    }
                }
            }
            Prop::TextUnderlineOffset => {
                if let Some(px) = self.number(val) {
                    set_text(style, TextProperty::UnderlineOffset, TextValue::Px(px));
                }
            }
            Prop::FontFamily => {
                if let Some(text) = kw {
                    // The row text is the `var(--font-*)` reference; the
                    // v0.x keyword is the family NAME — the reverse read
                    // of the row's own table.
                    match font_family_kw(text) {
                        Some(name) => {
                            set_text(style, TextProperty::FontFamily, TextValue::Keyword(name));
                        }
                        None => self
                            .notes
                            .push(PlanNoteKind::RowTextUnmappable { prop, text }),
                    }
                }
            }
        }
    }

    /// The candidate-finish pass: the gap pair's slot split (the v0.x
    /// oracle's own shape — `gap-2` fills the `Gap` slot, `gap-x-4` the
    /// axes slot, and the two COEXIST in one plan).
    fn finish(&mut self, style: &mut ResolvedStyle) {
        match (self.gap_row, self.gap_col) {
            (Some(row), Some(col)) if row == col => style.gap = Some(aeolus_core::Gap(row)),
            (row, col) => {
                if row.is_some() || col.is_some() {
                    style.extras.gap_axes = Some(aeolus_core::GapAxes { row, col });
                }
            }
        }
    }

    // ---- the value-form readers ------------------------------------------

    /// A length value to px: the `Rem`/`Px` forms bake directly, the
    /// keyword forms parse their row text (`2px`, `0.375rem`) and bake
    /// the rem forms through [`Config::rem_px`].
    fn length_px(&self, val: Val, kw: Option<&'static str>) -> Option<f32> {
        match val {
            Val::Px(v) => Some(v),
            Val::Rem(v) => Some(v * self.config.rem_px),
            Val::Kw(_) => kw.and_then(len_text).map(|(v, unit)| match unit {
                LenUnit::Px => v,
                LenUnit::Rem => v * self.config.rem_px,
            }),
            _ => None,
        }
    }

    /// A bare number (`z-10`, `border-2`, `rotate-45`) — px where the
    /// slot is a length, the raw number where it is a count/angle.
    fn number(&self, val: Val) -> Option<f32> {
        let _ = self;
        match val {
            Val::Num(v) => Some(v),
            Val::Px(v) => Some(v),
            _ => None,
        }
    }

    /// The `SizedValue` forms the dimension slots carry: lengths bake
    /// to px, the row's keyword texts map to the symbolic forms (the
    /// v0.x oracle's own readings, quirk-faithful).
    fn sized(&self, val: Val, kw: Option<&'static str>) -> Option<SizedValue> {
        match val {
            Val::Px(v) => Some(SizedValue::Px(v)),
            Val::Rem(v) => Some(SizedValue::Px(v * self.config.rem_px)),
            Val::Pct(v) => Some(SizedValue::Fraction(v / 100.0)),
            Val::Kw(_) => {
                let text = kw?;
                match text {
                    "auto" => Some(SizedValue::Auto),
                    "100%" => Some(SizedValue::Full),
                    "fit-content" => Some(SizedValue::FitContent),
                    "100vw" => Some(SizedValue::Viewport),
                    // The v0.x oracle maps the content keywords onto the
                    // viewport slot too (`w-min` → Viewport) — quirk kept
                    // for parity, the differential pins it.
                    "min-content" | "max-content" => Some(SizedValue::Viewport),
                    other => len_text(other).map(|(v, unit)| match unit {
                        LenUnit::Px => SizedValue::Px(v),
                        LenUnit::Rem => SizedValue::Px(v * self.config.rem_px),
                    }),
                }
            }
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// The keyword-text tables (the v0.x vocabulary, one arm per row text)
// ---------------------------------------------------------------------------

fn display_kw(text: &str) -> Option<V0Display> {
    match text {
        "flex" => Some(V0Display::Flex),
        "inline-flex" => Some(V0Display::InlineFlex),
        "grid" => Some(V0Display::Grid),
        "block" => Some(V0Display::Block),
        "inline" => Some(V0Display::Inline),
        "hidden" => Some(V0Display::None),
        _ => None,
    }
}

fn flex_direction_kw(text: &str) -> Option<V0FlexDirection> {
    match text {
        "row" => Some(V0FlexDirection::Row),
        "column" => Some(V0FlexDirection::Column),
        "row-reverse" => Some(V0FlexDirection::RowReverse),
        "column-reverse" => Some(V0FlexDirection::ColumnReverse),
        _ => None,
    }
}

fn justify_kw(text: &str) -> Option<V0JustifyContent> {
    match text {
        "flex-start" => Some(V0JustifyContent::Start),
        "flex-end" => Some(V0JustifyContent::End),
        "center" => Some(V0JustifyContent::Center),
        "space-between" => Some(V0JustifyContent::SpaceBetween),
        "space-around" => Some(V0JustifyContent::SpaceAround),
        "space-evenly" => Some(V0JustifyContent::SpaceEvenly),
        _ => None,
    }
}

fn items_kw(text: &str) -> Option<AlignItems> {
    match text {
        "flex-start" => Some(AlignItems::Start),
        "flex-end" => Some(AlignItems::End),
        "center" => Some(AlignItems::Center),
        "baseline" => Some(AlignItems::Baseline),
        "stretch" => Some(AlignItems::Stretch),
        _ => None,
    }
}

fn content_kw(text: &str) -> Option<ContentAlign> {
    match text {
        "flex-start" => Some(ContentAlign::Start),
        "flex-end" => Some(ContentAlign::End),
        "center" => Some(ContentAlign::Center),
        "space-between" => Some(ContentAlign::SpaceBetween),
        "space-around" => Some(ContentAlign::SpaceAround),
        "space-evenly" => Some(ContentAlign::SpaceEvenly),
        "baseline" => Some(ContentAlign::Baseline),
        _ => None,
    }
}

fn position_kw(text: &str) -> Option<V0Position> {
    match text {
        "static" => Some(V0Position::Static),
        "fixed" => Some(V0Position::Fixed),
        "absolute" => Some(V0Position::Absolute),
        "relative" => Some(V0Position::Relative),
        "sticky" => Some(V0Position::Sticky),
        _ => None,
    }
}

fn overflow_kw(text: &str) -> Option<OverflowKind> {
    match text {
        "visible" => Some(OverflowKind::Visible),
        "hidden" => Some(OverflowKind::Hidden),
        "clip" => Some(OverflowKind::Clip),
        "scroll" => Some(OverflowKind::Scroll),
        "auto" => Some(OverflowKind::Auto),
        _ => None,
    }
}

fn cursor_kw(text: &str) -> Option<V0Cursor> {
    match text {
        "auto" => Some(V0Cursor::Auto),
        "default" => Some(V0Cursor::Default),
        "pointer" => Some(V0Cursor::Pointer),
        "wait" => Some(V0Cursor::Wait),
        "text" => Some(V0Cursor::Text),
        "move" => Some(V0Cursor::Move),
        "help" => Some(V0Cursor::Help),
        "not-allowed" => Some(V0Cursor::NotAllowed),
        "none" => Some(V0Cursor::None),
        "progress" => Some(V0Cursor::Progress),
        "grab" => Some(V0Cursor::Grab),
        "grabbing" => Some(V0Cursor::Grabbing),
        "crosshair" => Some(V0Cursor::Crosshair),
        "zoom-in" => Some(V0Cursor::ZoomIn),
        "zoom-out" => Some(V0Cursor::ZoomOut),
        _ => None,
    }
}

/// The `select-*` spellings: the v0.x keyword is the full class form
/// (`select-none`), the row text the bare value (`none`).
const SELECT_FORMS: &[(&str, &str)] = &[
    ("none", "select-none"),
    ("text", "select-text"),
    ("all", "select-all"),
    ("auto", "select-auto"),
];

fn select_kw(text: &str) -> Option<&'static str> {
    SELECT_FORMS
        .iter()
        .find(|&&(value, _)| value == text)
        .map(|&(_, full)| full)
}

/// The font families: the row text is the `var(--font-*)` reference,
/// the v0.x keyword the family name — the reverse read of the row's
/// own table.
const FONT_FAMILY_FORMS: &[(&str, &str)] = &[
    ("var(--font-sans)", "sans"),
    ("var(--font-serif)", "serif"),
    ("var(--font-mono)", "mono"),
];

fn font_family_kw(text: &str) -> Option<&'static str> {
    FONT_FAMILY_FORMS
        .iter()
        .find(|&&(value, _)| value == text)
        .map(|&(_, name)| name)
}

// ---------------------------------------------------------------------------
// The length/ratio text readers (the row tables' own spellings)
// ---------------------------------------------------------------------------

/// `2px` / `0.375rem` / `9999px` / `24rem` — the row tables' length
/// spellings, value + unit (the rem forms bake through the caller's
/// [`Config::rem_px`], not here).
fn len_text(text: &str) -> Option<(f32, LenUnit)> {
    let text = text.trim();
    if let Some(rest) = text.strip_suffix("px") {
        return rest.parse::<f32>().ok().map(|v| (v, LenUnit::Px));
    }
    if let Some(rest) = text.strip_suffix("rem") {
        return rest.parse::<f32>().ok().map(|v| (v, LenUnit::Rem));
    }
    text.parse::<f32>().ok().map(|v| (v, LenUnit::Px))
}

/// The row tables' length spellings' units.
enum LenUnit {
    Px,
    Rem,
}

/// The unitless/`calc(a / b)` ratios (`1.25`, `calc(1.25 / 0.875)` —
/// the TEXT_SIZES secondary column's only form) and the `a / b`
/// aspect texts (`16 / 9`).
fn ratio_text(text: &str) -> Option<f32> {
    let text = text.trim();
    if let Some(inner) = text
        .strip_prefix("calc(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        let (a, b) = inner.split_once('/')?;
        let a: f32 = a.trim().parse().ok()?;
        let b: f32 = b.trim().parse().ok()?;
        return Some(a / b);
    }
    if let Some((a, b)) = text.split_once('/') {
        let a: f32 = a.trim().parse().ok()?;
        let b: f32 = b.trim().parse().ok()?;
        return Some(a / b);
    }
    text.parse::<f32>().ok()
}

/// The em-valued letter-spacing texts (`-0.025em`).
fn em_text(text: &str) -> Option<f32> {
    let rest = text.strip_suffix("em")?;
    rest.parse::<f32>().ok()
}

/// The baked-table → Oklab conversion (the audited parser; the
/// differential tolerance covers the `/255` quantization step).
fn oklab(rgba: [u8; 4]) -> Color {
    let hex = format!(
        "#{:02x}{:02x}{:02x}{:02x}",
        rgba[0], rgba[1], rgba[2], rgba[3]
    );
    Color::parse(&hex).unwrap_or(Color::TRANSPARENT)
}

// ---------------------------------------------------------------------------
// The per-edge/per-corner slot helpers
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Edge {
    Top,
    Right,
    Bottom,
    Left,
}

fn edge_of(prop: Prop) -> Edge {
    match prop {
        Prop::PaddingTop | Prop::MarginTop | Prop::BorderTopWidth => Edge::Top,
        Prop::PaddingRight | Prop::MarginRight | Prop::BorderRightWidth => Edge::Right,
        Prop::PaddingBottom | Prop::MarginBottom | Prop::BorderBottomWidth => Edge::Bottom,
        _ => Edge::Left,
    }
}

/// The edge-masked slots (`Padding`/`Margin`/`BorderWidth` share the
/// same four-float + mask shape): value write + mask set, one helper.
fn set_edge<S>(slot: &mut S, edge: Edge, px: f32)
where
    S: EdgeMasked,
{
    *slot.edge_f32(edge) = px;
    *slot.edge_flag(edge) = true;
}

/// The shared shape of the per-edge slots (the value/mask pairs).
trait EdgeMasked {
    fn edge_f32(&mut self, edge: Edge) -> &mut f32;
    fn edge_flag(&mut self, edge: Edge) -> &mut bool;
}

impl EdgeMasked for Padding {
    fn edge_f32(&mut self, edge: Edge) -> &mut f32 {
        match edge {
            Edge::Top => &mut self.top,
            Edge::Right => &mut self.right,
            Edge::Bottom => &mut self.bottom,
            Edge::Left => &mut self.left,
        }
    }
    fn edge_flag(&mut self, edge: Edge) -> &mut bool {
        match edge {
            Edge::Top => &mut self.set.top,
            Edge::Right => &mut self.set.right,
            Edge::Bottom => &mut self.set.bottom,
            Edge::Left => &mut self.set.left,
        }
    }
}

impl EdgeMasked for BorderWidth {
    fn edge_f32(&mut self, edge: Edge) -> &mut f32 {
        match edge {
            Edge::Top => &mut self.top,
            Edge::Right => &mut self.right,
            Edge::Bottom => &mut self.bottom,
            Edge::Left => &mut self.left,
        }
    }
    fn edge_flag(&mut self, edge: Edge) -> &mut bool {
        match edge {
            Edge::Top => &mut self.set.top,
            Edge::Right => &mut self.set.right,
            Edge::Bottom => &mut self.set.bottom,
            Edge::Left => &mut self.set.left,
        }
    }
}

/// The auto-edge slot (`mx-auto`): bool fields, no values.
fn set_auto_edge(slot: &mut AutoEdges, edge: Edge) {
    match edge {
        Edge::Top => slot.top = true,
        Edge::Right => slot.right = true,
        Edge::Bottom => slot.bottom = true,
        Edge::Left => slot.left = true,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Corner {
    TopLeft,
    TopRight,
    BottomRight,
    BottomLeft,
}

fn corner_of(prop: Prop) -> Corner {
    match prop {
        Prop::BorderTopLeftRadius => Corner::TopLeft,
        Prop::BorderTopRightRadius => Corner::TopRight,
        Prop::BorderBottomRightRadius => Corner::BottomRight,
        _ => Corner::BottomLeft,
    }
}

fn set_corner(slot: &mut aeolus_core::CornerRadius, corner: Corner, px: f32) {
    match corner {
        Corner::TopLeft => slot.top_left = px,
        Corner::TopRight => slot.top_right = px,
        Corner::BottomRight => slot.bottom_right = px,
        Corner::BottomLeft => slot.bottom_left = px,
    }
}

fn set_corner_set(slot: &mut aeolus_core::CornerSet, corner: Corner, set: bool) {
    match corner {
        Corner::TopLeft => slot.top_left = set,
        Corner::TopRight => slot.top_right = set,
        Corner::BottomRight => slot.bottom_right = set,
        Corner::BottomLeft => slot.bottom_left = set,
    }
}

fn set_text(style: &mut ResolvedStyle, prop: TextProperty, value: TextValue) {
    if let Some(slot) = style.text.iter_mut().find(|(p, _)| *p == prop) {
        slot.1 = value;
    } else {
        style.text.push((prop, value));
    }
}

// ---------------------------------------------------------------------------
// The bucket routing: Cond -> the v0.x plan's buckets
// ---------------------------------------------------------------------------

/// Resolve the plan bucket the fold's condition names and run the
/// writer against it — the seven classic states into `layers`, `dark:`
/// into `dark`, `sm:`/`min-*:` into `responsive`, `group-*:`/`peer-*:`
/// into the label vecs, and everything the v0.1 buckets cannot express
/// into `rules` (the attribute probes, `max-*:`, container queries,
/// `has-*`). Same-condition candidates land in the SAME slot (found by
/// condition/label/atom-equality), the write accumulating field-wise
/// exactly like the v0.x resolver's own application. Unmappable
/// `has-*` fragments land in `notes` (stamped with the candidate's
/// span and class) and their style does not land.
fn bucket(
    plan: &mut StylePlan,
    cond: Cond,
    span: (usize, usize),
    raw: &str,
    notes: &mut Vec<PlanNote>,
    write: impl FnOnce(&mut ResolvedStyle),
) {
    match cond {
        Cond::Base => write(&mut plan.base),
        Cond::Dark => write(plan.dark.get_or_insert_with(Default::default)),
        Cond::Hover => bucket_layer(plan, VariantCondition::Hover, write),
        Cond::Focus => bucket_layer(plan, VariantCondition::Focus, write),
        Cond::FocusVisible => bucket_layer(plan, VariantCondition::FocusVisible, write),
        Cond::FocusWithin => bucket_layer(plan, VariantCondition::FocusWithin, write),
        Cond::Active => bucket_layer(plan, VariantCondition::Active, write),
        Cond::Disabled => bucket_layer(plan, VariantCondition::Disabled, write),
        Cond::Visited => bucket_layer(plan, VariantCondition::Visited, write),
        Cond::GroupHover | Cond::GroupFocus => {
            // The named labels fold away on the IR tier (the cond's
            // documented payload set) — the default group carries the
            // style; the label rides the CSS arm's selector head only.
            bucket_labelled(&mut plan.group, String::new().into_boxed_str(), write);
        }
        Cond::PeerHover | Cond::PeerFocus => {
            bucket_labelled(&mut plan.peer, String::new().into_boxed_str(), write);
        }
        Cond::Bp(bp) | Cond::Min(bp) => {
            // The v0.x oracle routes the explicit min-* form into the
            // same responsive bucket (the differential pins it).
            write(plan.responsive.entry(breakpoint_of(bp)).or_default());
        }
        Cond::Max(bp) => {
            let when = V0Cond(vec![Atom::Viewport {
                min: None,
                max: Some(bp.rem() * 16.0),
            }]);
            bucket_rule(plan, when, write);
        }
        Cond::Container(bp) => {
            let when = V0Cond(vec![Atom::Container {
                min: Some(bp.rem() * 16.0),
                max: None,
                name: None,
            }]);
            bucket_rule(plan, when, write);
        }
        Cond::MaxContainer(bp) => {
            let when = V0Cond(vec![Atom::Container {
                min: None,
                max: Some(bp.rem() * 16.0),
                name: None,
            }]);
            bucket_rule(plan, when, write);
        }
        Cond::Data { name, value } => {
            let when = V0Cond(vec![Atom::On(Test::Data(
                name.into(),
                value.map(Into::into),
            ))]);
            bucket_rule(plan, when, write);
        }
        Cond::Aria { name, value } => {
            let when = V0Cond(vec![Atom::On(Test::Aria(
                name.into(),
                value.map(Into::into),
            ))]);
            bucket_rule(plan, when, write);
        }
        Cond::Has(inner) => match has_test(inner) {
            Ok(test) => {
                let when = V0Cond(vec![Atom::Has(test)]);
                bucket_rule(plan, when, write);
            }
            Err(()) => notes.push(PlanNote {
                span,
                raw: raw.to_string(),
                kind: PlanNoteKind::HasFragment { inner },
            }),
        },
        Cond::GroupHas(inner) | Cond::PeerHas(inner) => match has_test(inner) {
            Ok(test) => {
                let when = V0Cond(vec![Atom::Group(
                    test,
                    Some(String::new().into_boxed_str()),
                )]);
                bucket_rule(plan, when, write);
            }
            Err(()) => notes.push(PlanNote {
                span,
                raw: raw.to_string(),
                kind: PlanNoteKind::HasFragment { inner },
            }),
        },
        Cond::Checked => {
            let when = V0Cond(vec![Atom::On(Test::State(State::Checked))]);
            bucket_rule(plan, when, write);
        }
        Cond::PlaceholderShown => {
            let when = V0Cond(vec![Atom::On(Test::State(State::PlaceholderShown))]);
            bucket_rule(plan, when, write);
        }
        Cond::Selection => {
            let when = V0Cond(vec![Atom::On(Test::Pseudo(PseudoElement::Selection))]);
            bucket_rule(plan, when, write);
        }
        Cond::Placeholder => {
            let when = V0Cond(vec![Atom::On(Test::Pseudo(PseudoElement::Placeholder))]);
            bucket_rule(plan, when, write);
        }
    }
}

fn bucket_layer(
    plan: &mut StylePlan,
    condition: VariantCondition,
    write: impl FnOnce(&mut ResolvedStyle),
) {
    if let Some(slot) = plan
        .layers
        .iter_mut()
        .find(|layer| layer.condition == condition)
    {
        write(&mut slot.style);
    } else {
        let mut style = ResolvedStyle::default();
        write(&mut style);
        plan.layers
            .push(aeolus_core::ConditionalLayer { condition, style });
    }
}

fn bucket_labelled(
    plan: &mut Vec<(Box<str>, ResolvedStyle)>,
    label: Box<str>,
    write: impl FnOnce(&mut ResolvedStyle),
) {
    if let Some(slot) = plan.iter_mut().find(|(l, _)| *l == label) {
        write(&mut slot.1);
    } else {
        let mut style = ResolvedStyle::default();
        write(&mut style);
        plan.push((label, style));
    }
}

fn bucket_rule(plan: &mut StylePlan, when: V0Cond, write: impl FnOnce(&mut ResolvedStyle)) {
    if let Some(rule) = plan.rules.iter_mut().find(|rule| rule.when == when) {
        write(&mut rule.style);
    } else {
        let mut style = ResolvedStyle::default();
        write(&mut style);
        plan.rules.push(Rule { when, style });
    }
}

/// The `:has(...)` inner fragment → the v0.x `Test` (the state, data
/// and aria inners the fold's own grammar produces; the rest —
/// structural selectors, tags — refuse into the note).
fn has_test(inner: &'static str) -> Result<Test, ()> {
    let inner = inner.strip_prefix('*').ok_or(())?;
    if let Some(state) = inner.strip_prefix(':') {
        let state = match state {
            "focus" => State::Focus,
            "hover" => State::Hover,
            "active" => State::Active,
            "visited" => State::Visited,
            "checked" => State::Checked,
            "disabled" => State::Disabled,
            "focus-visible" => State::FocusVisible,
            "focus-within" => State::FocusWithin,
            _ => return Err(()),
        };
        return Ok(Test::State(state));
    }
    let inner = inner.strip_prefix('[').ok_or(())?;
    let inner = inner.strip_suffix(']').ok_or(())?;
    if let Some(rest) = inner
        .strip_prefix("data-")
        .or_else(|| inner.strip_prefix("aria-"))
    {
        let test = match rest.split_once('=') {
            Some((name, value)) => Test::Data(name.into(), Some(value.trim_matches('"').into())),
            None => Test::Data(rest.into(), None),
        };
        return Ok(test);
    }
    Err(())
}

fn breakpoint_of(bp: Bp) -> Breakpoint {
    match bp {
        Bp::Sm => Breakpoint::Sm,
        Bp::Md => Breakpoint::Md,
        Bp::Lg => Breakpoint::Lg,
        Bp::Xl => Breakpoint::Xl,
        Bp::Xl2 => Breakpoint::Xxl,
    }
}
