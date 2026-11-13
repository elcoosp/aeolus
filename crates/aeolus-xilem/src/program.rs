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
            | InterpNote::AttributeUnqueried { cond, .. } => *cond,
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
        };
        report.drop(what, reason);
    }
    for (prop, reason) in &walk.refusals {
        report.drop(prop.name(), reason.clone());
    }
    report
}
