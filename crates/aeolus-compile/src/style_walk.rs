//! The program lane's style form (architecture plan §7's re-target
//! half the interp arm named "interpreter otherwise"): a FROZEN static
//! program — the two slices the `tw!` macro emits — resolves into the
//! v0.x [`ResolvedStyle`] under one probe [`Env`], the typed style the
//! legacy backends resolve per state.
//!
//! ```text
//! program_style(layers, keywords, &Env)         -> (ResolvedStyle, StyleWalk)
//! program_style_with(…, probes, tokens)         -> (ResolvedStyle, StyleWalk)
//! program_style_probes_containers_struct_descendants_supports(
//!     …, probes, containers, structs, descendants, supports, tokens)
//!                                               -> (ResolvedStyle, StyleWalk)
//! program_style_pseudo(target, layers, keywords)
//!                                               -> (ResolvedStyle, StyleWalk)
//! program_style_pseudo_theme(target, …, tokens, &Env)
//!                                               -> (ResolvedStyle, StyleWalk)
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
//!
//! program_style_pseudo(target, …)
//!   └─ the TARGET's own walk (the `interp_pseudo` contract): only the
//!      layers carrying the target's `Cond` participate — condition
//!      equality, no `Env` and no probe seams — later fired wins per
//!      longhand; the element's own layers and the sibling pseudo
//!      targets are SILENT on this side (the element walk's
//!      [`InterpNote::PseudoElement`] note is the other half of the
//!      render-target contract)
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
use crate::interp::{
    AttrProbe, ContainerProbe, DescendantProbe, Env, Fire, InterpError, InterpNote, PseudoTarget,
    StructProbe, SupportsProbe, TokenSource, decide,
};
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
///   attribute probes, the descendant `has-*` shapes). The
///   pseudo-target walks ([`program_style_pseudo`]) never fill this
///   channel: a render-target's own cascade is SILENT about the layers
///   that belong to other targets — the element walk's
///   [`InterpNote::PseudoElement`] note is the other half of the
///   contract, reported on ITS side;
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
    run(layers, keywords, env, None, None, None, None, None, None)
}

/// [`program_style`] with the structural seam wired: a
/// [`StructProbe`] source answers the [`aeolus_ir::Cond::Structural`] layers'
/// DOM-position questions (the `interp_struct` convention — the
/// built-in value is one Copy `StructFacts`,
/// [`StructFacts::UNKNOWN`](crate::interp::StructFacts::UNKNOWN) the
/// honest "the host knows nothing" answer). The probe and token
/// seams stay sourceless (`None`), their reports riding the walk.
pub fn program_style_struct(
    layers: &'static [Layer],
    keywords: &'static [&'static str],
    env: &Env,
    structs: Option<&dyn StructProbe>,
) -> (ResolvedStyle, StyleWalk) {
    run(layers, keywords, env, None, None, structs, None, None, None)
}

/// [`program_style`] with the NAMED-CONTAINER seam wired — the
/// `interp_containers` convention on the program lane, the narrow
/// form of
/// [`program_style_probes_containers_struct_descendants_supports`]
/// with the seams it does not name riding `None`: one
/// [`ContainerProbe`] answers the `@container` layers (a hit fires by
/// the same AT-and-above / strictly-below boundary rules the interp
/// pins, a container the source does not know is a QUIET miss, `None`
/// keeps the reported boundary
/// [`InterpNote::ContainerUnqueried`]), and the token tail (the
/// program lane's `*_with` convention) resolves the theme forms last —
/// `None` refuses them per longhand into the walk's report.
pub fn program_style_containers(
    layers: &'static [Layer],
    keywords: &'static [&'static str],
    env: &Env,
    containers: Option<&dyn ContainerProbe>,
    tokens: Option<&dyn TokenSource>,
) -> (ResolvedStyle, StyleWalk) {
    run(
        layers, keywords, env, None, containers, None, None, None, tokens,
    )
}

/// [`program_style`] with the `has-*` seam wired — the
/// `interp_descendants` convention on the program lane, the narrow
/// form of
/// [`program_style_probes_containers_struct_descendants_supports`]
/// with the seams it does not name riding `None`: one
/// [`DescendantProbe`] answers the descendant layers (a YES fires, a
/// NO is a quiet skip, `None` keeps the reported boundary
/// [`InterpNote::DescendantUnqueried`]), and the token tail resolves
/// the theme forms (the program lane's `*_with` convention).
pub fn program_style_descendants(
    layers: &'static [Layer],
    keywords: &'static [&'static str],
    env: &Env,
    descendants: Option<&dyn DescendantProbe>,
    tokens: Option<&dyn TokenSource>,
) -> (ResolvedStyle, StyleWalk) {
    run(
        layers,
        keywords,
        env,
        None,
        None,
        None,
        descendants,
        None,
        tokens,
    )
}

/// [`program_style`] with the `@supports` seam wired — the
/// `interp_supports` convention on the program lane, the narrow form
/// of
/// [`program_style_probes_containers_struct_descendants_supports`]
/// with the seams it does not name riding `None`: one
/// [`SupportsProbe`] answers the capability layers (a YES fires, a NO
/// is a quiet skip, `None` keeps the reported boundary
/// [`InterpNote::SupportsUnqueried`]), and the token tail resolves the
/// theme forms (the program lane's `*_with` convention).
pub fn program_style_supports(
    layers: &'static [Layer],
    keywords: &'static [&'static str],
    env: &Env,
    supports: Option<&dyn SupportsProbe>,
    tokens: Option<&dyn TokenSource>,
) -> (ResolvedStyle, StyleWalk) {
    run(
        layers, keywords, env, None, None, None, None, supports, tokens,
    )
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
    run(
        layers, keywords, env, probes, None, None, None, None, tokens,
    )
}

/// [`program_style`] with the interp's FULL probe surface wired — the
/// widest program-lane entry (the
/// [`interp_probes_containers_struct_descendants_supports`]'s own
/// naming, the program lane's `*_with` token tail kept): the five
/// probe seams the walk's `decide` verdicts already consult, all
/// optional.
///
/// * `probes` — the element-own `data-*`/`aria-*` layers
///   ([`AttrProbe`], [`program_style_with`]'s seam);
/// * `containers` — the NAMED container layers
///   ([`ContainerProbe`]): a hit fires by the same AT-and-above /
///   strictly-below boundary rules the interp pins, a container the
///   source does not know is a QUIET miss, and `None` keeps the
///   reported boundary ([`InterpNote::ContainerUnqueried`]) — the
///   walk's pre-seam shape, unchanged for callers that pass no source;
/// * `structs` — the structural layers ([`StructProbe`],
///   [`program_style_struct`]'s seam);
/// * `descendants` — the `has-*` layers ([`DescendantProbe`]): a YES
///   fires, a NO is a quiet skip, `None` keeps
///   [`InterpNote::DescendantUnqueried`];
/// * `supports` — the `supports-*` layers ([`SupportsProbe`]): a YES
///   fires, a NO is a quiet skip, `None` keeps
///   [`InterpNote::SupportsUnqueried`];
/// * `tokens` — the theme forms ([`TokenSource`]).
///
/// The verdicts stay the interp arm's own `decide` — one
/// implementation, two walks — so the widest form cannot disagree
/// with the effective-declaration walk on when a layer fires, and the
/// narrow entries are this one with the seams they do not name riding
/// `None`.
pub fn program_style_probes_containers_struct_descendants_supports(
    layers: &'static [Layer],
    keywords: &'static [&'static str],
    env: &Env,
    probes: Option<&dyn AttrProbe>,
    containers: Option<&dyn ContainerProbe>,
    structs: Option<&dyn StructProbe>,
    descendants: Option<&dyn DescendantProbe>,
    supports: Option<&dyn SupportsProbe>,
    tokens: Option<&dyn TokenSource>,
) -> (ResolvedStyle, StyleWalk) {
    run(
        layers,
        keywords,
        env,
        probes,
        containers,
        structs,
        descendants,
        supports,
        tokens,
    )
}

// ---------------------------------------------------------------------------
// The pseudo-target walks (the `interp_pseudo` contract, frozen form)
// ---------------------------------------------------------------------------

/// Resolve a frozen static program's PSEUDO-ELEMENT cascade for one
/// [`PseudoTarget`] — the [`interp_pseudo`] contract on the program
/// lane: the `::selection`/`::placeholder` render target's own
/// effective style (only the layers carrying the target's condition
/// participate; the element's own layers and the sibling pseudo
/// targets are a different target, SILENT here — the element walk's
/// [`InterpNote::PseudoElement`] note reports them on ITS side).
///
/// `layers` and `keywords` are the two slices the macro freezes (the
/// [`program_style`] shape). The verdict is CONDITION EQUALITY (the
/// interp arm's own pseudo walk): a pseudo layer fires by its
/// condition alone — no [`Env`] and no probe seams ride (the flat fold
/// collapses stacked variants onto the pseudo condition) — later fired
/// declarations win per longhand, and the row texts index into the
/// flattened annotations exactly as the element walk reads them.
///
/// The theme forms ([`Val::Tok`]/[`Val::ModePair`]) refuse PER
/// LONGHAND into [`StyleWalk::refusals`] — the style fold's report
/// shape, never a hard error and never a silent transparent colour;
/// [`program_style_pseudo_theme`] carries the source they resolve
/// from. The fold runs through the SAME [`PlanWriter`] the element
/// walk uses, so a longhand the v0.x slots cannot carry reports its
/// typed [`PlanNoteKind`] boundary instead of vanishing.
///
/// The walk is pure: same program + target, same answer.
///
/// [`interp_pseudo`]: crate::interp::interp_pseudo
pub fn program_style_pseudo(
    target: PseudoTarget,
    layers: &'static [Layer],
    keywords: &'static [&'static str],
) -> (ResolvedStyle, StyleWalk) {
    run_pseudo(target, layers, keywords, None, None)
}

/// [`program_style_pseudo`] with one [`TokenSource`] resolving the
/// theme forms — the [`interp_pseudo_theme`] contract on the program
/// lane (the same two questions the element walk asks, on the pseudo
/// render target). The `env` rides the RESOLUTION (the mode selects
/// the pair halves), never the firing — a pseudo layer fires by its
/// condition alone, the condition-equality verdict the themeless form
/// pins.
///
/// The walk is pure: same program + target + source, same answer.
///
/// [`interp_pseudo_theme`]: crate::interp::interp_pseudo_theme
pub fn program_style_pseudo_theme(
    target: PseudoTarget,
    layers: &'static [Layer],
    keywords: &'static [&'static str],
    tokens: &dyn TokenSource,
    env: &Env,
) -> (ResolvedStyle, StyleWalk) {
    run_pseudo(target, layers, keywords, Some(tokens), Some(env))
}

/// The pseudo-target fold: the interp arm's own-cascade verdict
/// (condition equality) over the SAME [`PlanWriter`] the element walk
/// writes through. The keyword annotations are flattened in
/// layer-major order across ALL layers (the same accumulation
/// [`StaticProgram::keyword`] reads back), so `index` advances over
/// every layer — the target's own declarations read the texts their
/// flattened offsets name whether the element layers ride before or
/// after them.
///
/// [`StaticProgram::keyword`]: the re-exporting host crate's own
/// reading of the flattened annotations.
fn run_pseudo(
    target: PseudoTarget,
    layers: &'static [Layer],
    keywords: &'static [&'static str],
    tokens: Option<&dyn TokenSource>,
    env: Option<&Env>,
) -> (ResolvedStyle, StyleWalk) {
    #[cfg(feature = "tracing")]
    let _span = tracing::debug_span!("program_style_pseudo", layers = layers.len()).entered();

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
        // The shadow accumulator is PER LAYER here (the element walk's
        // per-candidate shape): each fired layer's preset/clear/recolour
        // composes at the layer's own `finish`.
        shadow_preset: None,
        shadow_clear: None,
        shadow_color: None,
    };
    let mut refusals: Vec<(Prop, String)> = Vec::new();
    let wanted = target.cond();

    let mut index = 0usize;
    for layer in layers {
        // The verdict is CONDITION EQUALITY (the interp arm's own
        // pseudo walk): only the layers carrying the target's
        // condition participate — the element's own layers and the
        // sibling pseudo targets are silence, not notes (the element
        // walk's `InterpNote::PseudoElement` note is the other half of
        // the render-target contract). No `Env` and no probe seams
        // consult: the flat fold collapses stacked variants onto the
        // pseudo condition.
        if layer.cond == wanted {
            for (di, decl) in layer.decls.iter().enumerate() {
                let text = keywords
                    .get(index + di)
                    .copied()
                    .filter(|text| !text.is_empty());
                let val = match concrete(decl.val, env.unwrap_or(&Env::default()), tokens) {
                    Ok(val) => val,
                    Err(reason) => {
                        let reason = reason.on_pseudo_face();
                        #[cfg(feature = "tracing")]
                        tracing::debug!(prop = ?decl.prop, %reason, "theme form refused");
                        refusals.push((decl.prop, reason.to_string()));
                        continue;
                    }
                };
                writer.write(&mut style, decl.prop, val, text, false);
            }
            writer.finish(&mut style);
            #[cfg(feature = "tracing")]
            tracing::debug!(cond = ?wanted, fired = layer.decls.len(), "pseudo layer fired");
        }
        index += layer.decls.len();
    }

    (
        style,
        StyleWalk {
            // The target-side silence: the sibling layers belong to
            // other targets, reported by the element walk on ITS side.
            notes: Vec::new(),
            boundaries: writer.notes,
            refusals,
        },
    )
}

// ---------------------------------------------------------------------------
// The walk (the interp's verdicts, the bridge's writer)
// ---------------------------------------------------------------------------

fn run(
    layers: &'static [Layer],
    keywords: &'static [&'static str],
    env: &Env,
    probes: Option<&dyn AttrProbe>,
    containers: Option<&dyn ContainerProbe>,
    structs: Option<&dyn StructProbe>,
    descendants: Option<&dyn DescendantProbe>,
    supports: Option<&dyn SupportsProbe>,
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
        // The shadow accumulator is PER LAYER here (the plan bridge's
        // per-candidate shape): each fired layer's preset/clear/recolour
        // composes at the layer's own `finish`, the fired layers being
        // independent buckets the same way the bridge's are.
        shadow_preset: None,
        shadow_clear: None,
        shadow_color: None,
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
        // fires. The full probe surface rides the widest entry point
        // (`program_style_probes_containers_struct_descendants_supports`):
        // the named-container, structural, descendant and supports
        // seams answer from the host's sources when one rides, and the
        // sourceless entries keep the reported boundary the narrow
        // forms have always carried.
        match decide(
            layer.cond,
            env,
            probes,
            containers,
            structs,
            descendants,
            supports,
            layer.mask,
        ) {
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
    /// [`Self::NoTokenSource`] on the pseudo-target face: the reason
    /// names the lane that works there
    /// ([`program_style_pseudo_theme`]).
    NoTokenSourcePseudo,
    /// [`Self::NoTokenSourcePair`] on the pseudo-target face.
    NoTokenSourcePairPseudo,
    /// The source's own typed failure (see [`InterpError`]).
    Source(InterpError),
}

impl ConcreteRefusal {
    /// The refusal as the pseudo-target face reports it: the
    /// sourceless arms name the face's own themed entry (the reason
    /// points at the lane that works — the codegen gates' own rule),
    /// the source's typed failures ride unchanged.
    fn on_pseudo_face(self) -> Self {
        match self {
            Self::NoTokenSource => Self::NoTokenSourcePseudo,
            Self::NoTokenSourcePair => Self::NoTokenSourcePairPseudo,
            other => other,
        }
    }
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
            Self::NoTokenSourcePseudo => write!(
                f,
                "no TokenSource — the runtime tier's explicit slow lane \
                 (pass one via program_style_pseudo_theme)"
            ),
            Self::NoTokenSourcePairPseudo => write!(
                f,
                "no TokenSource — the static dark pair rides the baked \
                 table (pass one via program_style_pseudo_theme)"
            ),
            Self::Source(err) => err.fmt(f),
        }
    }
}
