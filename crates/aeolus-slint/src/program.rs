//! The V1 static-program lane (architecture plan §7's slint re-target —
//! the sixth backend after GPUI, Floem, Xilem, Iced and egui): a
//! `tw!`-compiled [`StaticProgram`] resolves
//! into the crate's own [`SlintStyle`] under the
//! interp's [`Env`] probe semantics, so a frozen program feeds the SAME
//! [`lower_resolved`] mapping the plan path's
//! `StyleBook` has driven since v0.1.
//!
//! The WALK itself lives IN the pipeline
//! ([`aeolus_rt::program_style`], the compile crate's `style_walk`
//! seam): the interp's own `decide` verdicts and the plan bridge's own
//! slot writer, so the string-side plan and the program-side style
//! cannot drift apart. THIS module is the backend's honest surface:
//!
//! * the [`SlintProbe`]-to-[`Env`] conversion the
//!   element-state flags need ([`env_for_probe`], the lane's new seam —
//!   the probe the parity path already keeps answers the walk's two rem
//!   slots and the state flags),
//! * the rem bake through [`REM_PX`] (§9's per-backend `rem_px`
//!   constant, default 16),
//! * the composition with [`lower_resolved`] —
//!   the walk returns the v0.x `ResolvedStyle`, the crate's own
//!   mapping folds it into the brush/property payload, so a capability
//!   bump re-audits ONE backend mapping, and
//! * the translation of the walk's typed [`StyleWalk`] report into the
//!   crate's [`LoweringReport`] — every excluded
//!   layer, unmapped longhand and theme refusal lands there beside the
//!   lowering's own drops, never a silent drop.

use aeolus_rt::{
    AttrProbe, Env, InterpNote, PlanNoteKind, StaticProgram, StyleWalk, TokenSource,
    program_style_with,
};

use crate::{LoweringReport, SlintProbe, SlintStyle, lower_resolved};

/// The backend's rem size, in px (plan §9's "per-backend `rem_px`
/// constant, default 16"): every [`SlintProbe`] width bakes into the
/// probe [`Env`]'s rem slots through this one constant — the same 16px
/// bake the v0.x resolver's spacing scale applies (1 unit = 0.25rem =
/// 4px, so `p-4` = 1rem = 16px on both lanes — the differential pins
/// agree).
pub const REM_PX: f32 = 16.0;

/// Resolve one compiled static program into a
/// [`SlintStyle`] under the element's probe [`Env`] —
/// the §7 re-target's lane, the program form the `tw!` macro freezes
/// and [`ClassCell`](aeolus_rt::ClassCell) pools.
///
/// Theme-token longhands (`Val::Tok`/`Val::ModePair`) refuse into the
/// report here: pass a source through [`lower_program_with`] when the
/// program carries them (the runtime tier's explicit slow lane).
/// Attribute-probe layers stay unqueried and reported for the same
/// reason.
pub fn lower_program(program: &StaticProgram, env: &Env) -> (SlintStyle, LoweringReport) {
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
) -> (SlintStyle, LoweringReport) {
    let (style, walk) = program_style_with(program.layers, program.keywords, env, probes, tokens);
    let mut report = translate(&walk);
    let lowered = lower_resolved(&style, &mut report);
    (lowered, report)
}

/// The element-state probe conversion: the
/// [`SlintProbe`] the app keeps for the parity path —
/// the same struct the `TouchArea`/`FocusScope` flags feed
/// (`has-hover`, `pressed`, `enabled`, `has-focus`) — into the walk's
/// probe [`Env`]. Widths bake through [`REM_PX`]
/// (`width`/`container_width` are px), the focus flag feeds BOTH
/// `focus` and `focus_visible` (the plan path's documented
/// normalization: Slint's `has-focus` carries no focus-visible
/// distinction and [`SlintProbe`] maps `State::Focus`
/// and `State::FocusVisible` onto the same bool), the checked flag
/// rides (the view state `SlintProbe` already answers), and the
/// conditions [`SlintProbe`] cannot answer
/// (focus-within, visited, placeholder, group/peer) honestly stay off —
/// the probe's own `related` answers "no", the same semantics CSS has
/// when a selector does not match.
pub fn env_for_probe(probe: &SlintProbe) -> Env {
    Env {
        dark: probe.dark,
        hover: probe.has_hover,
        focus: probe.has_focus,
        focus_visible: probe.has_focus,
        focus_within: false,
        active: probe.pressed,
        disabled: !probe.enabled,
        visited: false,
        checked: probe.checked,
        placeholder_shown: false,
        group_hover: false,
        group_focus: false,
        peer_hover: false,
        peer_focus: false,
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
            | InterpNote::StructuralUnqueried { cond, .. } => *cond,
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
