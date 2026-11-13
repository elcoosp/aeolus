//! The GPUI ring polyfill (architecture plan §3.7's `ring-*` row:
//! "none → **polyfill** as spread-only `BoxShadow` — v4 semantics: ring
//! = box-shadow spread"). GPUI has no ring primitive, but its
//! `Vec<BoxShadow>` carries exactly the layers v4's ring machinery
//! composes, so the four `--tw-ring-*` longhands the registry lowers
//! into ([`Prop::RingWidth`] / [`Prop::RingColor`] /
//! [`Prop::RingOffsetWidth`] / [`Prop::RingOffsetColor`]) accumulate
//! into a [`RingState`] and render as the v4 box-shadow list instead of
//! the silent no-op the un-mapped `_ => style` arm used to be.
//!
//! The composition is v4's own (tailwindcss v4 `utilities.ts`):
//!
//! ```text
//! box-shadow: …, 0 0 0 var(--tw-ring-offset-width)  var(--tw-ring-offset-color),
//!             …, 0 0 0 calc(var(--tw-ring-offset-width) + var(--tw-ring-width))
//!                var(--tw-ring-color);
//! ```
//!
//! The FIRST layer paints on top, so the offset band sits over the ring
//! band's inner edge — the list order is [`RingState::shadows`]'s
//! contract. `ring-0` (a 0 width with no offset) paints nothing, and a
//! bare `ring` falls back to v4's `--tw-ring-color: currentColor`
//! through the element's own text colour ([`Prop::Color`], the same
//! cascade the walk applies).

use gpui::{BoxShadow, Hsla, hsla, px};

use aeolus_ir::{Prop, Val};

/// v4's `--tw-ring-offset-color` default (`#fff`): the offset band
/// paints WHITE when the program sets an offset width without an
/// explicit colour — the disc the ring sits on.
const DEFAULT_OFFSET_COLOR: Hsla = hsla(0.0, 0.0, 1.0, 1.0);

/// The static stand-in for v4's `--tw-ring-color: currentColor` when
/// the program carries NEITHER a ring colour NOR a text colour: the
/// inherited colour is unknowable in a frozen refinement, so the
/// neutral black paints (documented, unlike a silent drop).
const CURRENT_COLOR_FALLBACK: Hsla = hsla(0.0, 0.0, 0.0, 1.0);

/// The `--tw-ring-*` machinery one walk accumulates — the four ring
/// longhands plus the text colour the `currentColor` default reads.
///
/// Every field is last-write-wins: the walk feeds the accumulator in
/// cascade order (base layers first, then the fired variant layers),
/// so a later `ring-4` replaces an earlier `ring-2` exactly like every
/// other longhand family.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RingState {
    width: Option<f32>,
    color: Option<Hsla>,
    offset_width: Option<f32>,
    offset_color: Option<Hsla>,
    text_color: Option<Hsla>,
}

impl RingState {
    /// Record one longhand the walk saw, with its frozen row text (the
    /// annotation a bare [`Val::Kw`] slot cannot recover — bare `ring`
    /// lowers to `Val::Kw(0)` whose row text is `"1px"`, v4's own
    /// default). Ring longhands mutate the machinery; [`Prop::Color`]
    /// records the `currentColor` source. Every other prop is a no-op
    /// here (the caller applies those through the per-decl mapping as
    /// before).
    pub fn apply(&mut self, prop: Prop, val: Val, keyword: &str) {
        match prop {
            Prop::RingWidth => self.width = px_of(val).or_else(|| px_text(keyword)),
            Prop::RingColor => self.color = hsla_of(val),
            Prop::RingOffsetWidth => {
                self.offset_width = px_of(val).or_else(|| px_text(keyword));
            }
            Prop::RingOffsetColor => self.offset_color = hsla_of(val),
            Prop::Color => {
                if let Some(color) = hsla_of(val) {
                    self.text_color = Some(color);
                }
            }
            _ => {}
        }
    }

    /// The v4 box-shadow list the state composes — empty when nothing
    /// would paint (`ring-0` with no offset, the default state). The
    /// caller sets the whole list in one `shadow()` call: GPUI's
    /// `Vec<BoxShadow>` is the composed surface, and the walk is the
    /// only writer on this path.
    pub fn shadows(&self) -> Vec<BoxShadow> {
        let mut out = Vec::new();
        let offset = self.offset_width.unwrap_or(0.0);
        let width = self.width.unwrap_or(0.0);
        // The offset band first (it paints on top): `0 0 0
        // var(--tw-ring-offset-width) var(--tw-ring-offset-color)`.
        if offset > 0.0 {
            out.push(
                BoxShadow::new(
                    px(0.0),
                    px(0.0),
                    self.offset_color.unwrap_or(DEFAULT_OFFSET_COLOR),
                )
                .spread_radius(px(offset)),
            );
        }
        // The ring band: `0 0 0 calc(var(--tw-ring-offset-width) +
        // var(--tw-ring-width)) var(--tw-ring-color)` — the explicit
        // colour wins, `currentColor` falls back to the element's own
        // text colour, and the inherited case paints the documented
        // neutral stand-in.
        if width > 0.0 {
            let color = self
                .color
                .or(self.text_color)
                .unwrap_or(CURRENT_COLOR_FALLBACK);
            out.push(BoxShadow::new(px(0.0), px(0.0), color).spread_radius(px(offset + width)));
        }
        out
    }

    /// True when the state paints nothing.
    pub fn is_empty(&self) -> bool {
        self.width.is_none_or(|width| width <= 0.0)
            && self.offset_width.is_none_or(|offset| offset <= 0.0)
    }
}

/// The px a ring width carries. The registry's ring rows lower widths
/// in the numeric form — a UNITLESS px number in the IR (`ring-2` →
/// `Val::Num(2.0)`, the row's `NumForm::Px` contract whose emitted text
/// renders `2px`). Keyword/other forms have no px here and leave the
/// field untouched (the honest skip).
fn px_of(val: Val) -> Option<f32> {
    match val {
        Val::Num(n) => Some(n),
        Val::Px(n) => Some(n),
        _ => None,
    }
}

/// The px a row text carries (`"1px"` → 1.0, `"2"` → 2.0) — the
/// `Val::Kw` slot's recovery, the same row-text read the interp and the
/// emit arms run.
fn px_text(keyword: &str) -> Option<f32> {
    keyword
        .strip_suffix("px")
        .unwrap_or(keyword)
        .parse::<f32>()
        .ok()
}

/// The [`Hsla`] a colour carries — the baked `Rgba` the theme pass's
/// T0/T1 forms produce, through the same conversion the crate's other
/// colour slots run.
fn hsla_of(val: Val) -> Option<Hsla> {
    match val {
        Val::Rgba([r, g, b, a]) => Some(gpui::rgba(u32::from_be_bytes([r, g, b, a])).into()),
        _ => None,
    }
}
