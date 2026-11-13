//! The GPUI gradient lane (architecture plan §3.7's `bg-linear-*` row:
//! "native for 2 stops; ≥3 stops/radial/conic → diagnostic"). GPUI's
//! `linear_gradient(angle, from, to)` carries exactly the v4 two-stop
//! linear form, so the gradient longhands the registry lowers into —
//! the direction row's `BackgroundImage` text plus the
//! `--tw-gradient-from`/`-to` colours and their `-position` patches —
//! accumulate into a [`GradState`] and compose the linear gradient
//! instead of the silent no-op the un-mapped `_ => style` arm used to
//! be.
//!
//! The honest boundaries, all documented:
//!
//! * **Three stops** (`via-*`) paint NOTHING rather than a wrong
//!   (flattened) gradient — the static emit gate
//!   (`aeolus_codegen_gpui::unsupported_reason`) refuses the program
//!   typed, and the walk degrades to no gradient, never to a
//!   two-stop misreading of a three-stop intent.
//! * **Radial/conic** and the bare `bg-linear` placeholder texts have
//!   no `Fill` slot — the walk composes nothing for them (the static
//!   gate refuses them with the same reason the interp/plan surfaces
//!   carry).
//! * GPUI's background is ONE slot: where v4 layers the gradient image
//!   OVER a solid `background-color`, the composed gradient replaces
//!   the solid the walk applied (the more specific v4 intent wins; the
//!   coexistence is the plan path's `background` + `background_gradient`
//!   pair).

use gpui::{Background, Hsla, hsla};

use aeolus_compile::linear_gradient_angle;
use aeolus_ir::{Prop, Val};

/// v4's `@property` initial for a missing stop colour
/// (`--tw-gradient-from: oklch(0 0 0 / 0)`): the transparent edge the
/// gradient fades from/to.
const TRANSPARENT: Hsla = hsla(0.0, 0.0, 0.0, 0.0);

/// The direction row's shape the walk can paint: the composed
/// `linear-gradient(to …)` angle, or the `bg-none` explicit clear.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Shape {
    /// `bg-linear-to-*` — the CSS angle (0° = to top, clockwise).
    Linear(f32),
    /// `bg-none` — the explicit clear (no gradient paints; a solid
    /// `bg-*` the walk applied stays, the CSS layering).
    Clear,
}

/// The `--tw-gradient-*` machinery one walk accumulates — the direction
/// row's shape, the three stop colours and their position patches.
///
/// Every field is last-write-wins: the walk feeds the accumulator in
/// cascade order (base layers first, then the fired variant layers), so
/// a later `from-blue-500` replaces an earlier `from-red-500` exactly
/// like every other longhand family.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GradState {
    shape: Option<Shape>,
    from: Option<Hsla>,
    from_pos: Option<f32>,
    via: Option<Hsla>,
    to: Option<Hsla>,
    to_pos: Option<f32>,
}

impl GradState {
    /// Record one longhand the walk saw, with its frozen row text (the
    /// annotation a bare [`Val::Kw`] slot cannot recover — the
    /// direction's composed `linear-gradient(to …)` text). Gradient
    /// longhands mutate the machinery; every other prop is a no-op
    /// here (the caller applies those through the per-decl mapping as
    /// before).
    pub fn apply(&mut self, prop: Prop, val: Val, keyword: &str) {
        match prop {
            Prop::BackgroundImage => {
                // The composed row text is the verdict's input: the
                // direction forms parse to the angle, `bg-none` clears,
                // and the radial/conic/placeholder texts have no `Fill`
                // slot — the documented no-paint boundary.
                if keyword == "none" {
                    self.shape = Some(Shape::Clear);
                } else if let Some(angle_deg) = linear_gradient_angle(keyword) {
                    self.shape = Some(Shape::Linear(angle_deg));
                }
            }
            Prop::GradientFrom => self.from = hsla_of(val),
            Prop::GradientFromPosition => self.from_pos = fraction_of(val),
            Prop::GradientVia => self.via = hsla_of(val),
            Prop::GradientTo => self.to = hsla_of(val),
            Prop::GradientToPosition => self.to_pos = fraction_of(val),
            _ => {}
        }
    }

    /// The composed linear gradient the state paints — `None` when
    /// nothing would (no direction, the `bg-none` clear, or a via stop
    /// present: GPUI's gradient carries TWO stops, and a flattened
    /// three-stop reading would paint a WRONG gradient, so the via
    /// case degrades to no gradient — the static gate refuses it
    /// typed).
    ///
    /// The missing edges compose v4's `@property` initials: the
    /// transparent from/to stop at the 0%/100% defaults, so a lone
    /// `from-*` (or `to-*`) still paints the fade.
    pub fn background(&self) -> Option<Background> {
        let Shape::Linear(angle_deg) = self.shape? else {
            return None;
        };
        if self.via.is_some() {
            return None;
        }
        if self.from.is_none() && self.to.is_none() {
            return None;
        }
        Some(gpui::linear_gradient(
            angle_deg,
            gpui::linear_color_stop(
                self.from.unwrap_or(TRANSPARENT),
                self.from_pos.unwrap_or(0.0),
            ),
            gpui::linear_color_stop(self.to.unwrap_or(TRANSPARENT), self.to_pos.unwrap_or(1.0)),
        ))
    }

    /// True when the state paints nothing.
    pub fn is_empty(&self) -> bool {
        self.background().is_none()
    }
}

/// The [`Hsla`] a stop colour carries — the baked `Rgba` the theme
/// pass's T0/T1 forms produce, through the same conversion the crate's
/// other colour slots run.
fn hsla_of(val: Val) -> Option<Hsla> {
    match val {
        Val::Rgba([r, g, b, a]) => Some(gpui::rgba(u32::from_be_bytes([r, g, b, a])).into()),
        _ => None,
    }
}

/// The stop position a `Val::Pct` patch carries, as GPUI's 0..1
/// fraction (`from-20%` → 0.2).
fn fraction_of(val: Val) -> Option<f32> {
    match val {
        Val::Pct(pct) => Some(pct / 100.0),
        _ => None,
    }
}
