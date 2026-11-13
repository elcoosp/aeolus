//! The program lane's style form (architecture plan §7's re-target
//! half the interp arm named "interpreter otherwise"): a FROZEN static
//! program — the two slices the `tw!` macro emits — resolves into the
//! v0.x [`ResolvedStyle`] under one probe [`Env`], the typed style the
//! legacy backends resolve per state.
//!
//! ```text
//! program_style(layers, keywords, &Env)         -> (ResolvedStyle, StyleWalk)
//! program_style_with(…, probes, tokens)         -> (ResolvedStyle, StyleWalk)
//!
//! StaticProgram (layers + the ROW TEXTS the Kw slots index)
//!   └─ front-to-back walk: a layer FIRES when its Cond matches the Env
//!      ├─ fired   → its declarations write the slots IN ORDER (the
//!      │            specificity sort makes later-beats-earlier the
//!      │            cascade; per-edge families accumulate field-wise)
//!      ├─ quiet   → skipped (the probe says no — CSS semantics)
//!      ├─ excluded→ reported as the interp's typed [`InterpNote`]s
//!      └─ unmapped→ reported as the plan bridge's typed
//!                   [`PlanNoteKind`] boundaries (the SAME note set
//!                   `emit_plan` carries)
//! ```
//!
//! # Why the walk runs IN the pipeline
//!
//! The slot WRITE is the plan bridge's own [`PlanWriter`] — one arm per
//! registry longhand family, the row-table keyword texts resolving the
//! `Val::Kw` slots the bare IR cannot. Re-running it here means the
//! string-side plan (`emit_plan`) and the program-side style fold
//! through ONE writer: the differential batteries pin the two paths
//! slot for slot, and a registry semantics bump re-audits one
//! implementation. The layer VERDICTS are the interp arm's own
//! `decide` (shared in-crate), so the style walk and the effective-
//! declaration walk cannot diverge on when a layer fires.
//!
//! # The theme forms ride the TokenSource seam
//!
//! [`Val::Tok`]/[`Val::ModePair`] are the theme pass's forms — they
//! resolve through the [`TokenSource`] seam (the built-in
//! [`TokenTable`] answering the T2 tier's one-load-one-multiply), like
//! every `*_with` entry point. Without a source the themed longhand
//! REFUSES into the walk's report — never a silent transparent
//! colour.
//!
//! # What this seam deliberately is NOT
//!
//! The per-backend mappings stay per-backend (the plan bridge's
//! `Target` surface): this seam returns the v0.x [`ResolvedStyle`] the
//! legacy backends already resolve — it is the migration seam's
//! program-side half, not a new emit arm. The `outline-none` SPELLING
//! (the accepted-suffix flag the bridge reads from the source body)
//! has no frozen form on the program slices; the walk bakes it as the
//! width-0 outline it visually is, the same reading the GPUI/Floem
//! walks take.

use std::fmt;

use aeolus_core::plan::ResolvedStyle;
use aeolus_ir::{Layer, Prop, Val};

use crate::Config;
use crate::interp::{AttrProbe, Env, Fire, InterpError, InterpNote, TokenSource, decide};
use crate::plan::{PlanNoteKind, PlanWriter};

// ---------------------------------------------------------------------------
// The artifact: StyleWalk
// ---------------------------------------------------------------------------

/// The walk's honest report: every layer that did not fold into the
/// style and every longhand the v0.x slots cannot carry, TYPED — the
/// no-silent-failures contract the emit arms carry, on the program
/// lane. Three channels, each the typed note set an existing walk
/// already defines:
///
/// * [`StyleWalk::notes`] — the excluded layers (the interp arm's
///   [`InterpNote`] set: pseudo-elements, unqueried containers,
///   attribute probes, the descendant `has-*` shapes);
/// * [`StyleWalk::boundaries`] — the unmapped longhands (the plan
///   bridge's [`PlanNoteKind`] set: slot-less longhands, row texts the
///   v0.x vocabulary lacks, the shadow-preset and gradient-row
///   families, uncarryable `has-*` fragments);
/// * [`StyleWalk::refusals`] — the theme forms ([`Val::Tok`]/
///   [`Val::ModePair`]) that refused per longhand because no
///   [`TokenSource`] rode along or the source did not know the slot,
///   as `(prop, reason)` pairs.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StyleWalk {
    /// The excluded layers, in program order (see [`InterpNote`]).
    pub notes: Vec<InterpNote>,
    /// The unmapped longhands, in walk order (see [`PlanNoteKind`]).
    pub boundaries: Vec<PlanNoteKind>,
    /// The per-longhand theme refusals, in walk order — `(prop,
    /// reason)` pairs, the reason the source's own typed failure text.
    pub refusals: Vec<(Prop, String)>,
}

impl StyleWalk {
    /// True when nothing was reported (the program fully mapped under
    /// this env).
    pub fn is_empty(&self) -> bool {
        self.notes.is_empty() && self.boundaries.is_empty() && self.refusals.is_empty()
    }
}

// ---------------------------------------------------------------------------
// The entry points (the `*_with` convention)
// ---------------------------------------------------------------------------

/// Resolve a frozen static program into the v0.x [`ResolvedStyle`]
/// under one probe [`Env`] — the theme-LESS form: token values
/// ([`Val::Tok`]/[`Val::ModePair`]) refuse per longhand into the
/// walk's report (they are the theme pass's forms;
/// [`program_style_with`] carries the source they resolve from).
///
/// `layers` and `keywords` are the two slices the macro freezes — the
/// `StaticProgram` shape `aeolus-rt` re-exports
/// (`layers` the specificity-sorted IR layers, `keywords` the row
/// texts parallel to the flattened declarations). This crate cannot
/// name that type (it lives one crate up, re-exporting this one), so
/// the seam takes the two slices the type carries.
///
/// The walk is pure: same program + env, same answer.
pub fn program_style(
    layers: &'static [Layer],
    keywords: &'static [&'static str],
    env: &Env,
) -> (ResolvedStyle, StyleWalk) {
    run(layers, keywords, env, None, None)
}

/// [`program_style`] with the interp's two seams wired: an
/// [`AttrProbe`] source for the element-own `data-*`/`aria-*` layers
/// (the `tw_probes` convention) and a [`TokenSource`] for the theme
/// forms (the `interp_theme` convention). `None` keeps the CSS-quiet
/// semantics — an unanswered probe layer never fires and is reported,
/// an unresolvable token refuses into the walk's report.
pub fn program_style_with(
    layers: &'static [Layer],
    keywords: &'static [&'static str],
    env: &Env,
    probes: Option<&dyn AttrProbe>,
    tokens: Option<&dyn TokenSource>,
) -> (ResolvedStyle, StyleWalk) {
    run(layers, keywords, env, probes, tokens)
}

// ---------------------------------------------------------------------------
// The walk (the interp's verdicts, the bridge's writer)
// ---------------------------------------------------------------------------

fn run(
    layers: &'static [Layer],
    keywords: &'static [&'static str],
    env: &Env,
    probes: Option<&dyn AttrProbe>,
    tokens: Option<&dyn TokenSource>,
) -> (ResolvedStyle, StyleWalk) {
    #[cfg(feature = "tracing")]
    let _span = tracing::debug_span!("program_style", layers = layers.len()).entered();

    let mut style = ResolvedStyle::default();
    let mut writer = PlanWriter {
        config: Config::default(),
        notes: Vec::new(),
        font_size_px: None,
        gap_row: None,
        gap_col: None,
        gradient_shape: None,
        gradient_from: None,
        gradient_via: None,
        gradient_to: None,
        gradient_from_pos: None,
        gradient_via_pos: None,
        gradient_to_pos: None,
    };
    let mut notes: Vec<InterpNote> = Vec::new();
    let mut refusals: Vec<(Prop, String)> = Vec::new();

    // The keyword annotations are flattened in layer-major order (the
    // same accumulation `StaticProgram::keyword` reads back); `index`
    // is the flattened offset of the current layer's first declaration.
    let mut index = 0usize;
    for layer in layers {
        // The verdicts are the INTERP's own `decide` — one
        // implementation, two walks (the effective-declaration arm and
        // this style fold), so the two cannot disagree on when a layer
        // fires. The container-bounds seam rides the interp arm's
        // `*_containers` forms; this fold keeps the sourceless shape
        // (the style lane's named-container layers report, exactly as
        // they did before the seam landed).
        match decide(layer.cond, env, probes, None, layer.mask) {
            Fire::Yes => {
                for (di, decl) in layer.decls.iter().enumerate() {
                    let text = keywords
                        .get(index + di)
                        .copied()
                        .filter(|text| !text.is_empty());
                    let val = match concrete(decl.val, env, tokens) {
                        Ok(val) => val,
                        Err(reason) => {
                            #[cfg(feature = "tracing")]
                            tracing::debug!(prop = ?decl.prop, %reason, "theme form refused");
                            refusals.push((decl.prop, reason.to_string()));
                            continue;
                        }
                    };
                    writer.write(&mut style, decl.prop, val, text, false);
                }
                // The gap pair's slot split is PER LAYER (the plan
                // bridge's per-candidate `finish`): `gap-2` fills the
                // `Gap` slot, a later fired `gap-x-4` fills the axes
                // slot — the two coexist exactly as the bridge's
                // buckets do.
                writer.finish(&mut style);
                #[cfg(feature = "tracing")]
                tracing::debug!(cond = ?layer.cond, fired = layer.decls.len(), "layer fired");
            }
            Fire::No => {}
            Fire::Note(note) => {
                #[cfg(feature = "tracing")]
                tracing::debug!(cond = ?layer.cond, note = %note, "layer excluded");
                notes.push(note);
            }
        }
        index += layer.decls.len();
    }

    (
        style,
        StyleWalk {
            notes,
            boundaries: writer.notes,
            refusals,
        },
    )
}

/// The concrete value a declaration carries: theme forms
/// ([`Val::Tok`]/[`Val::ModePair`]) resolve through the
/// [`TokenSource`] seam (the interp's own two questions, the mode
/// riding every ask); everything else rides verbatim. A refusal — no
/// source wired, an unknown slot — carries the typed
/// [`ConcreteRefusal`], the machine-checkable form of the text the
/// walk's report renders.
fn concrete(val: Val, env: &Env, tokens: Option<&dyn TokenSource>) -> Result<Val, ConcreteRefusal> {
    match val {
        Val::Tok { id, alpha } => match tokens {
            Some(source) => source
                .resolve(id, alpha, env.dark)
                .map_err(ConcreteRefusal::Source),
            None => Err(ConcreteRefusal::NoTokenSource),
        },
        Val::ModePair(slot) => match tokens {
            Some(source) => source
                .resolve_pair(slot, env.dark)
                .map_err(ConcreteRefusal::Source),
            None => Err(ConcreteRefusal::NoTokenSourcePair),
        },
        concrete => Ok(concrete),
    }
}

/// The theme-form refusal the style fold reports per longhand — the
/// TYPED form of the refusal text [`StyleWalk::refusals`] renders. The
/// helper's two hand-written prose arms become machine-checkable
/// variants (a report can distinguish "no source wired" from "the
/// source refused" without string matching), while the public report
/// keeps its `(prop, reason)` text shape (the crates' report
/// convention).
#[derive(Clone, Debug, PartialEq)]
enum ConcreteRefusal {
    /// No [`TokenSource`] rides and the value is a runtime token slot
    /// ([`Val::Tok`]): the runtime tier's explicit slow lane.
    NoTokenSource,
    /// No [`TokenSource`] rides and the value is a T1 pair slot
    /// ([`Val::ModePair`]): the static dark pair rides the baked table.
    NoTokenSourcePair,
    /// The source's own typed failure (see [`InterpError`]).
    Source(InterpError),
}

impl fmt::Display for ConcreteRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoTokenSource => write!(
                f,
                "no TokenSource — the runtime tier's explicit slow lane \
                 (pass one via program_style_with)"
            ),
            Self::NoTokenSourcePair => write!(
                f,
                "no TokenSource — the static dark pair rides the baked \
                 table (pass one via program_style_with)"
            ),
            Self::Source(err) => err.fmt(f),
        }
    }
}
