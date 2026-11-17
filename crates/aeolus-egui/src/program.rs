//! The V1 static-program lane (architecture plan §7's egui re-target —
//! the fifth backend after GPUI, Floem, Xilem and Iced): a
//! `tw!`-compiled [`StaticProgram`] resolves
//! into the crate's own [`EguiStyle`] under the
//! interp's [`Env`] probe semantics, so a frozen program feeds the SAME
//! [`lower_resolved`] mapping the plan path's
//! [`StyleBook`](crate::StyleBook) has driven since v0.1.
//!
//! The WALK itself lives IN the pipeline
//! ([`aeolus_rt::program_style`], the compile crate's `style_walk`
//! seam): the interp's own `decide` verdicts and the plan bridge's own
//! slot writer, so the string-side plan and the program-side style
//! cannot drift apart. THIS module is the backend's honest surface:
//!
//! * the [`EguiProbe`]-to-[`Env`] conversion the
//!   immediate-mode frame needs ([`env_for_probe`], the lane's new
//!   seam — the probe the parity path already keeps answers the walk's
//!   two rem slots and the state flags),
//! * the rem bake through [`REM_PX`] (§9's per-backend `rem_px`
//!   constant, default 16),
//! * the composition with [`lower_resolved`] —
//!   the walk returns the v0.x `ResolvedStyle`, the crate's own
//!   mapping folds it into the `Frame`/hint payload, so a capability
//!   bump re-audits ONE backend mapping, and
//! * the translation of the walk's typed [`StyleWalk`] report into the
//!   crate's [`LoweringReport`] — every excluded
//!   layer, unmapped longhand and theme refusal lands there beside the
//!   lowering's own drops, never a silent drop.

use aeolus_rt::{
    AttrProbe, ContainerProbe, DescendantProbe, Env, InterpNote, PlanNoteKind, PseudoTarget,
    StaticProgram, StructProbe, StyleWalk, SupportsProbe, TokenSource,
    program_style_probes_containers_struct_descendants_supports, program_style_pseudo,
    program_style_pseudo_theme, program_style_with,
};

use crate::{EguiProbe, EguiStyle, LoweringReport, lower_resolved};

/// The backend's rem size, in px (plan §9's "per-backend `rem_px`
/// constant, default 16"): every [`EguiProbe`] width bakes into the
/// probe [`Env`]'s rem slots through this one constant — the same 16px
/// bake the v0.x resolver's spacing scale applies (1 unit = 0.25rem =
/// 4px, so `p-4` = 1rem = 16px on both lanes — the differential pins
/// agree).
pub const REM_PX: f32 = 16.0;

/// Resolve one compiled static program into an
/// [`EguiStyle`] under the frame's probe [`Env`] —
/// the §7 re-target's lane, the program form the `tw!` macro freezes
/// and [`ClassCell`](aeolus_rt::ClassCell) pools.
///
/// Theme-token longhands (`Val::Tok`/`Val::ModePair`) refuse into the
/// report here: pass a source through [`lower_program_with`] when the
/// program carries them (the runtime tier's explicit slow lane).
/// Attribute-probe layers stay unqueried and reported for the same
/// reason.
pub fn lower_program(program: &StaticProgram, env: &Env) -> (EguiStyle, LoweringReport) {
    lower_program_with(program, env, None, None)
}

/// [`lower_program`] with the interp's two seams wired: an
/// [`AttrProbe`] source for the element-own `data-*`/`aria-*` layers
/// and a [`TokenSource`] for the theme forms. `None` keeps the
/// CSS-quiet semantics — an unanswered probe never fires and is
/// reported, an unresolvable token refuses into the report.
pub fn lower_program_with(
    program: &StaticProgram,
    env: &Env,
    probes: Option<&dyn AttrProbe>,
    tokens: Option<&dyn TokenSource>,
) -> (EguiStyle, LoweringReport) {
    let (style, walk) = program_style_with(program.layers, program.keywords, env, probes, tokens);
    let mut report = translate(&walk);
    let lowered = lower_resolved(&style, &mut report);
    (lowered, report)
}

/// [`lower_program`] with the walk's FULL probe surface wired — the
/// widest program-lane entry (the shared walk's
/// `program_style_probes_containers_struct_descendants_supports`
/// naming, the backend's report translation unchanged): the five
/// probe seams the plan-lane program's layers can consult, all
/// optional.
///
/// * `probes` — the element-own `data-*`/`aria-*` layers
///   ([`AttrProbe`], [`lower_program_with`]'s seam);
/// * `containers` — the NAMED container layers ([`ContainerProbe`]):
///   the host's measured `@container` elements answer by the same
///   AT-and-above / strictly-below boundary rules the interp pins;
///   `None` keeps the reported boundary — the walk's pre-seam shape,
///   unchanged for callers that pass no source;
/// * `structs` — the structural layers ([`StructProbe`]);
/// * `descendants` — the `has-*` layers ([`DescendantProbe`]): a YES
///   fires, a NO is a quiet skip, `None` keeps the reported boundary;
/// * `supports` — the `supports-*` layers ([`SupportsProbe`]): the
///   render target's capability answer, the same shape;
/// * `tokens` — the theme forms ([`TokenSource`]).
///
/// The verdicts stay the walk's own `decide` (one implementation, two
/// walks), so the widest form cannot disagree with the narrow entries
/// or the interp arm on when a layer fires.
pub fn lower_program_probes_containers_struct_descendants_supports(
    program: &StaticProgram,
    env: &Env,
    probes: Option<&dyn AttrProbe>,
    containers: Option<&dyn ContainerProbe>,
    structs: Option<&dyn StructProbe>,
    descendants: Option<&dyn DescendantProbe>,
    supports: Option<&dyn SupportsProbe>,
    tokens: Option<&dyn TokenSource>,
) -> (EguiStyle, LoweringReport) {
    let (style, walk) = program_style_probes_containers_struct_descendants_supports(
        program.layers,
        program.keywords,
        env,
        probes,
        containers,
        structs,
        descendants,
        supports,
        tokens,
    );
    let mut report = translate(&walk);
    let lowered = lower_resolved(&style, &mut report);
    (lowered, report)
}

/// Lower one [`PseudoTarget`]'s own cascade from a compiled static
/// program into an [`EguiStyle`] — the render-target contract's backend
/// half (the shared walk's `program_style_pseudo` on the backend's
/// face): the `::selection`/`::placeholder`-family cascade a host that
/// PAINTS the target consumes (a paint callback's selection layer, a
/// custom placeholder), folded through the SAME `lower_resolved`
/// mapping the element walk rides.
///
/// The verdict is CONDITION EQUALITY (the interp arm's own pseudo
/// walk): only the layers carrying the target's condition participate,
/// later fired declarations win per longhand, no [`Env`] and no probe
/// seams consult (the flat fold collapses stacked variants onto the
/// pseudo condition). The element's own layers and the sibling pseudo
/// targets stay SILENT here — the element walk's
/// [`InterpNote::PseudoElement`] note reports them on ITS side.
///
/// Theme-token longhands (`Val::Tok`/`Val::ModePair`) refuse into the
/// report here: pass a source through [`lower_program_pseudo_theme`]
/// when the program carries them (the runtime tier's explicit slow
/// lane).
pub fn lower_program_pseudo(
    program: &StaticProgram,
    target: PseudoTarget,
) -> (EguiStyle, LoweringReport) {
    let (style, walk) = program_style_pseudo(target, program.layers, program.keywords);
    let mut report = translate(&walk);
    let lowered = lower_resolved(&style, &mut report);
    (lowered, report)
}

/// [`lower_program_pseudo`] with one [`TokenSource`] resolving the
/// theme forms (the shared walk's `program_style_pseudo_theme` on the
/// backend's face): the `env` rides the RESOLUTION (the mode selects
/// the pair halves), never the firing — a pseudo layer fires by its
/// condition alone.
pub fn lower_program_pseudo_theme(
    program: &StaticProgram,
    target: PseudoTarget,
    tokens: &dyn TokenSource,
    env: &Env,
) -> (EguiStyle, LoweringReport) {
    let (style, walk) =
        program_style_pseudo_theme(target, program.layers, program.keywords, tokens, env);
    let mut report = translate(&walk);
    let lowered = lower_resolved(&style, &mut report);
    (lowered, report)
}

/// The immediate-mode probe conversion: the
/// [`EguiProbe`] the app keeps for the parity path —
/// the same struct [`StyleBook::resolve_at`](crate::StyleBook::resolve_at)
/// consumes, built per frame from the `Response` or the context — into
/// the walk's probe [`Env`]. Widths bake through [`REM_PX`]
/// (`width`/`container_width` are px), the focus flag feeds BOTH
/// `focus` and `focus_visible` (the plan path's documented
/// normalization: egui's `Response::has_focus` carries no
/// focus-visible distinction and `EguiProbe::test`
/// maps `State::Focus` and `State::FocusVisible` onto the same bool),
/// and the conditions [`EguiProbe`] cannot answer (focus-within,
/// visited, checked, placeholder, group/peer) honestly stay off — the
/// probe's own `related` answers "no", the same semantics CSS has when
/// a selector does not match.
pub fn env_for_probe(probe: &EguiProbe) -> Env {
    Env {
        dark: probe.dark,
        hover: probe.hovered,
        focus: probe.focused,
        focus_visible: probe.focused,
        focus_within: false,
        active: probe.active,
        disabled: probe.disabled,
        visited: false,
        checked: false,
        placeholder_shown: false,
        group_hover: false,
        group_focus: false,
        peer_hover: false,
        peer_focus: false,
        // The composition facts ride no probe surface either (the
        // composed states come from the host's own ancestor/sibling
        // tracking, which the probe path does not publish): the
        // composition layers stay the quiet miss a selector that does
        // not match is.
        group_focus_visible: false,
        group_focus_within: false,
        group_active: false,
        group_disabled: false,
        group_visited: false,
        group_checked: false,
        peer_focus_visible: false,
        peer_focus_within: false,
        peer_active: false,
        peer_disabled: false,
        peer_visited: false,
        peer_checked: false,
        // The preference and capability facts ride no probe surface
        // (the media-feature queries answer from the user agent, not
        // the element): the probe path honestly keeps them off — the
        // layers stay the quiet miss a media query that does not hold
        // is.
        reduced_motion: false,
        more_contrast: false,
        less_contrast: false,
        forced_colors: false,
        print_media: false,
        pointer_fine: false,
        pointer_coarse: false,
        pointer_none: false,
        any_pointer_fine: false,
        any_pointer_coarse: false,
        any_pointer_none: false,
        portrait: false,
        // The direction fact rides no probe surface either (the
        // resolved direction answers from the host's DOM, not the
        // element state): the probe path honestly keeps it off — the
        // rtl/ltr layers stay the quiet miss a selector that does not
        // match is.
        direction_rtl: false,
        // The validation facts ride no probe surface either (the
        // constraint state and the user-interaction gate answer from
        // the host's field, not the probe path's element state): the
        // probe path honestly keeps them off — the validation layers
        // stay the quiet miss a selector that does not match is.
        validation_invalid: false,
        user_validated: false,
        // The element-state facts ride no probe surface either (the
        // fragment target, the indeterminate/autofill answers and the
        // editability fact come from the host's element state, not
        // the probe path's viewport/container facts): the probe path
        // honestly keeps them off — the element-state layers stay the
        // quiet miss a selector that does not match is.
        url_target: false,
        indeterminate: false,
        autofilled: false,
        read_write: false,
        // The widget-state facts ride no probe surface either (the
        // disclosure/popover answer, the frozen-subtree answer and the
        // form-default answer come from the host's widget state, not
        // the probe path's viewport/container facts): the probe path
        // honestly keeps them off — the widget-state layers stay the
        // quiet miss a selector that does not match is.
        opened: false,
        inert: false,
        form_default: false,
        // The form-state facts ride no probe surface either (the
        // form-membership answer, the required-attribute answer and
        // the range-bounds answers come from the host's form state,
        // not the probe path's viewport/container facts): the probe
        // path honestly keeps them off — the form-state layers stay
        // the quiet miss a selector that does not match is.
        form_associated: false,
        required_field: false,
        range_bounded: false,
        range_in: false,
        // The media-feature remainder is frame state: the probe paths keep
        // the facts off (the form-state fields' own reading — the probe
        // tiers carry no media answers; the walks report nothing, the
        // layers stay the quiet miss). ''no'' here is the honest
        // frame default, never a silently-fabricated answer.
        inverted_colors: false,
        scripting_none: false,
        starting_style: false,
        viewport_rem: probe.width.unwrap_or(0.0) / REM_PX,
        container_rem: probe.container_width.map(|width| width / REM_PX),
    }
}

/// The walk's typed report → the crate's `(what, reason)` pairs — the
/// no-silent-failures channel the plan path has used since v0.1. The
/// excluded layers name their condition, the unmapped longhands their
/// prop, the refusals the longhand that refused and why.
fn translate(walk: &StyleWalk) -> LoweringReport {
    let mut report = LoweringReport::default();
    for note in &walk.notes {
        let cond = match note {
            InterpNote::PseudoElement { cond, .. }
            | InterpNote::ContainerUnqueried { cond, .. }
            | InterpNote::AttributeUnqueried { cond, .. }
            | InterpNote::StructuralUnqueried { cond, .. }
            | InterpNote::DescendantUnqueried { cond, .. }
            | InterpNote::SupportsUnqueried { cond, .. } => *cond,
        };
        let what = cond
            .source()
            .map(std::string::ToString::to_string)
            .unwrap_or_else(|| format!("{cond:?}"));
        report.drop(what, note.to_string());
    }
    for boundary in &walk.boundaries {
        let (what, reason) = match boundary {
            PlanNoteKind::SlotUnsupported { prop } => (prop.name().to_string(), boundary.help()),
            PlanNoteKind::RowTextUnmappable { prop, text } => {
                (format!("{} {text}", prop.name()), boundary.help())
            }
            PlanNoteKind::GradientRow => ("background-image".to_string(), boundary.help()),
            PlanNoteKind::ShadowPreset => ("box-shadow".to_string(), boundary.help()),
            PlanNoteKind::HasFragment { inner, .. } => (format!("has:{inner}"), boundary.help()),
            PlanNoteKind::SupportsCondition { cond } => {
                (format!("supports:{cond}"), boundary.help())
            }
            PlanNoteKind::MediaFeature { pref } => {
                (format!("media:{}", pref.source()), boundary.help())
            }
            PlanNoteKind::StartingStyle => ("starting-style".to_string(), boundary.help()),
            PlanNoteKind::Direction { cond } => {
                (format!("direction:{}", cond.source()), boundary.help())
            }
            // The pseudo-element render targets: the v0.x surface carries
            // selection/placeholder/file only — the tier's report rides
            // the boundary channel (emit_css renders the real heads).
            PlanNoteKind::PseudoElement { cond } => (
                format!("pseudo:{}", cond.source().unwrap_or("pseudo-element")),
                boundary.help(),
            ),
            PlanNoteKind::CompositeValue { prop, text } => {
                (format!("{} {text}", prop.name()), boundary.help())
            }
        };
        report.drop(what, reason);
    }
    for (prop, reason) in &walk.refusals {
        report.drop(prop.name(), reason.clone());
    }
    report
}
