//! The V1 static-program lane (architecture plan §7's Xilem re-target —
//! the third backend after GPUI and Floem): a `tw!`-compiled
//! [`StaticProgram`] resolves into the plan
//! path's own [`ResolvedStyle`] under the interp's [`Env`] probe
//! semantics, so a frozen program feeds the SAME
//! [`StyledWidget`](crate::widget::StyledWidget) resolution the plan
//! path has driven since v0.1.
//!
//! The walk itself lives IN the pipeline
//! ([`aeolus_rt::program_style`], re-exported from the compile crate's
//! `style_walk` seam): the interp's own `decide` verdicts and the plan
//! bridge's own slot writer, so the string-side plan and the
//! program-side style cannot drift apart. THIS module is the backend's
//! honest surface: the [`TwEnv`]-to-[`Env`] conversion the widget's
//! per-state resolution needs, the rem bake through [`REM_PX`] (§9's
//! per-backend `rem_px` constant, default 16), and the translation of
//! the walk's typed report into the crate's
//! [`LoweringReport`] — every excluded layer,
//! unmapped longhand and theme refusal lands there, never a silent
//! drop.

use aeolus_core::plan::ResolvedStyle;
use aeolus_rt::{
    AttrProbe, Env, InterpNote, PlanNoteKind, StaticProgram, StyleWalk, TokenSource,
    program_style_with,
};

use crate::LoweringReport;
use crate::widget::TwEnv;

/// The backend's rem size, in px (plan §9's "per-backend `rem_px`
/// constant, default 16"): every [`TwEnv`] width bakes into the probe
/// [`Env`]'s rem slots through this one constant — the same 16px bake
/// the v0.x resolver's spacing scale applies (1 unit = 0.25rem = 4px,
/// so `p-4` = 1rem = 16px on both lanes — the differential pins
/// agree).
pub const REM_PX: f32 = 16.0;

/// Resolve one compiled static program into a [`ResolvedStyle`] under
/// the frame's probe [`Env`] — the §7 re-target's lane, the program
/// form the `tw!` macro freezes and
/// [`ClassCell`](aeolus_rt::ClassCell) pools.
///
/// Theme-token longhands (`Val::Tok`/
/// `Val::ModePair`) refuse into the report
/// here: pass a source through [`lower_program_with`] when the program
/// carries them (the runtime tier's explicit slow lane).
/// Attribute-probe layers stay unqueried and reported for the same
/// reason.
pub fn lower_program(program: &StaticProgram, env: &Env) -> (ResolvedStyle, LoweringReport) {
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
) -> (ResolvedStyle, LoweringReport) {
    let (style, walk) = program_style_with(program.layers, program.keywords, env, probes, tokens);
    (style, translate(&walk))
}

/// The widget's probe conversion: the [`TwEnv`] the app supplies
/// through `StyledWidget::set_env` plus the four Masonry state flags
/// the widget reads from its context — one probe [`Env`], the shape
/// the pipeline's walks ask about. Widths bake through [`REM_PX`]
/// (`viewport_width`/`container_width` are px), the focus flag feeds
/// BOTH `focus` and `focus_visible` (the plan path's documented
/// normalization: Masonry's `Selector` carries a single focus flag),
/// and the conditions [`TwEnv`] cannot answer (`focus-within`,
/// `visited`, `checked`, placeholder) honestly stay off.
pub fn env_for_state(
    env: &TwEnv,
    hovered: bool,
    active: bool,
    disabled: bool,
    focused: bool,
) -> Env {
    Env {
        dark: env.dark,
        hover: hovered,
        focus: focused,
        focus_visible: focused,
        focus_within: false,
        active,
        disabled,
        visited: false,
        checked: false,
        placeholder_shown: false,
        group_hover: env.group_hovered,
        group_focus: env.group_focused,
        peer_hover: env.peer_hovered,
        peer_focus: env.peer_focused,
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
        viewport_rem: env.viewport_width.unwrap_or(0.0) / REM_PX,
        container_rem: env.container_width.map(|width| width / REM_PX),
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
            PlanNoteKind::HasFragment { inner } => (format!("has:{inner}"), boundary.help()),
            PlanNoteKind::SupportsCondition { cond } => {
                (format!("supports:{cond}"), boundary.help())
            }
            PlanNoteKind::MediaFeature { pref } => {
                (format!("media:{}", pref.source()), boundary.help())
            }
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
