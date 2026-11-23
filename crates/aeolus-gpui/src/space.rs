//! The between-child polyfill's GPUI half (the parity wave 1's
//! space/divide row): no per-element slot carries the between-child
//! margins/borders, so the runtime composes them the way the CSS arm's
//! `:where(& > :not(:last-child))` rule does — a WRAPPER seam, the
//! [`crate::TwContainerExt::tw_container`] measuring-wrapper precedent.
//!
//! The shape is the audit gate's POLYFILL tier made concrete: the walk
//! routes the ten between-child longhands into [`SpaceDivState`] (the
//! `RingState`/`GradState` accumulator pattern), and
//! [`TwSpaceExt::tw_space`] consumes the state by styling the wrapper's
//! CHILDREN — every non-last child carries the end-side margin/border
//! the v4 machinery composes (`margin-inline-end` /
//! `border-inline-end-width`, the reverse flips flipping the side).
//! The plain [`crate::TwExt::tw`] lane keeps the walk total (the
//! between-child longhands route into the state like every polyfill
//! family) but drops it — a per-element surface has no children to
//! style; the wrapper is the consumer.
//!
//! The reach boundary, documented like every polyfill's: the patch
//! restyles children it can see (`AnyElement::downcast_mut` to GPUI's
//! own `Div` — the div-based children the examples and the suites
//! build) and leaves any other element kind untouched; children the
//! patch cannot reach stay unstyled, visible, never mis-styled. The
//! state is public ([`SpaceDiv::state`]) so a host can see exactly
//! what the program's between-child longhands composed.

use gpui::{
    AnyElement, BorderStyle, DefiniteLength, Div, Element, ElementId, IntoElement, ParentElement,
    Styled,
};

use aeolus_ir::{Prop, Val};

use crate::{
    Env, FilterState, GradState, GridState, RingState, ShadowState, StaticProgram, absolute,
    apply_matching_polyfills, color, definite,
};

/// The between-child parts one program composes (the accumulator the
/// walk fills and the wrapper consumes) — the `RingState` precedent's
/// shape: one slot per longhand, the cascade's last write wins. The
/// fields are the polyfill's PUBLIC data surface (the host's and the
/// suites' visibility — the no-silent-failures contract: what the
/// between-child classes composed is readable, never hidden).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SpaceDivState {
    /// `--tw-space-x` (`space-x-{n}`): the column margin value and its
    /// row text (`Val::Kw` slots carry the row's px text, the spacing
    /// rows the rem value).
    pub space_x: Option<(Val, &'static str)>,
    /// `--tw-space-x-reverse`: the flip flag (the value moves to the
    /// start side).
    pub space_x_reverse: bool,
    /// `--tw-space-y` (`space-y-{n}`): the row margin value.
    pub space_y: Option<(Val, &'static str)>,
    /// `--tw-space-y-reverse`: the flip flag.
    pub space_y_reverse: bool,
    /// `--tw-divide-x` (`divide-x`/`divide-x-{n}`): the column border
    /// width.
    pub divide_x: Option<(Val, &'static str)>,
    /// `--tw-divide-x-reverse`: the flip flag.
    pub divide_x_reverse: bool,
    /// `--tw-divide-y` (`divide-y`/`divide-y-{n}`): the row border
    /// width.
    pub divide_y: Option<(Val, &'static str)>,
    /// `--tw-divide-y-reverse`: the flip flag.
    pub divide_y_reverse: bool,
    /// `--tw-divide-color` (`divide-{color}`): the child border colour.
    pub divide_color: Option<Val>,
    /// `--tw-divide-style` (`divide-{style}`): the child border style
    /// keyword (the pinned GPUI surface carries solid/dashed; the other
    /// keywords stay the documented no-op — the scene's own closed set).
    pub divide_style: Option<&'static str>,
}

/// The between-child axes the state's reads take (the margin/width
/// slots' vocabulary).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    /// The column axis (`space-x`/`divide-x`).
    X,
    /// The row axis (`space-y`/`divide-y`).
    Y,
}

impl SpaceDivState {
    /// One between-child decl lands (the walk's per-decl route; the
    /// cascade's last write wins).
    pub(crate) fn apply(&mut self, prop: Prop, val: Val, keyword: &'static str) {
        match prop {
            Prop::SpaceX => self.space_x = Some((val, keyword)),
            Prop::SpaceXReverse => self.space_x_reverse = true,
            Prop::SpaceY => self.space_y = Some((val, keyword)),
            Prop::SpaceYReverse => self.space_y_reverse = true,
            Prop::DivideX => self.divide_x = Some((val, keyword)),
            Prop::DivideXReverse => self.divide_x_reverse = true,
            Prop::DivideY => self.divide_y = Some((val, keyword)),
            Prop::DivideYReverse => self.divide_y_reverse = true,
            Prop::DivideColor => self.divide_color = Some(val),
            Prop::DivideStyle => self.divide_style = Some(keyword),
            _ => {}
        }
    }

    /// No between-child parts composed (the wrapper's zero-cost test).
    pub fn is_empty(&self) -> bool {
        self.space_x.is_none()
            && !self.space_x_reverse
            && self.space_y.is_none()
            && !self.space_y_reverse
            && self.divide_x.is_none()
            && !self.divide_x_reverse
            && self.divide_y.is_none()
            && !self.divide_y_reverse
            && self.divide_color.is_none()
            && self.divide_style.is_none()
    }

    /// The composed margin length for one axis — the value shape the
    /// patch applies (`None` when the axis's longhand never fired or
    /// the value has no GPUI length form).
    pub fn margin(&self, axis: Axis) -> Option<gpui::Length> {
        let (val, keyword) = match axis {
            Axis::X => self.space_x.as_ref()?,
            Axis::Y => self.space_y.as_ref()?,
        };
        let length: DefiniteLength = definite(*val, keyword)?;
        Some(length.into())
    }

    /// The composed border width for one axis — the value shape the
    /// patch applies (the absolute lengths only, the divide scale's own
    /// px shape; `None` when the axis never fired or the value is
    /// relative).
    pub fn border(&self, axis: Axis) -> Option<gpui::AbsoluteLength> {
        let (val, keyword) = match axis {
            Axis::X => self.divide_x.as_ref()?,
            Axis::Y => self.divide_y.as_ref()?,
        };
        absolute(*val, keyword)
    }

    /// Patch ONE non-last child with the composed margins/borders — the
    /// between-child rule's own per-child reading (`:not(:last-child)`
    /// excludes the last child; the reverse flips move the value to the
    /// start side). The reach boundary: a `Div` child patches, any
    /// other element kind stays untouched.
    pub(crate) fn patch_child(&self, child: &mut AnyElement) {
        let Some(div) = child.downcast_mut::<Div>() else {
            return;
        };
        let style = div.style();
        // space-x: the END side carries the value (`margin-inline-end`
        // under the 0 default the reverse var reads); the flip moves it
        // to the start side.
        if let Some((val, keyword)) = &self.space_x
            && let Some(length) = definite(*val, keyword)
        {
            let length: gpui::Length = length.into();
            if self.space_x_reverse {
                style.margin.left = Some(length);
            } else {
                style.margin.right = Some(length);
            }
        }
        // space-y: the row axis twin (`margin-block-end` / start).
        if let Some((val, keyword)) = &self.space_y
            && let Some(length) = definite(*val, keyword)
        {
            let length: gpui::Length = length.into();
            if self.space_y_reverse {
                style.margin.top = Some(length);
            } else {
                style.margin.bottom = Some(length);
            }
        }
        // divide-x: the end border carries the width (the absolute
        // lengths only — the divide scale's own px shape, the border
        // widths' own slot).
        if let Some((val, keyword)) = &self.divide_x
            && let Some(width) = absolute(*val, keyword)
        {
            if self.divide_x_reverse {
                style.border_widths.left = Some(width);
            } else {
                style.border_widths.right = Some(width);
            }
        }
        // divide-y: the row axis twin (top under the flip, bottom
        // default — the v4 rule's own start/end reading).
        if let Some((val, keyword)) = &self.divide_y
            && let Some(width) = absolute(*val, keyword)
        {
            if self.divide_y_reverse {
                style.border_widths.top = Some(width);
            } else {
                style.border_widths.bottom = Some(width);
            }
        }
        if let Some(colour) = self.divide_color.and_then(color) {
            style.border_color = Some(colour);
        }
        if let Some(keyword) = self.divide_style {
            style.border_style = Some(match keyword {
                "dashed" => BorderStyle::Dashed,
                _ => BorderStyle::Solid,
            });
        }
    }
}

/// The between-child wrapper (the [`crate::ContainerQuery`] precedent:
/// a custom element carrying no styling of its own, forwarding the
/// layout/paint pipeline to the inner element, restyling the CHILDREN
/// on the way through). `tw_space` applies the program's base styling
/// to the element itself AND extracts the between-child parts for the
/// children — one call, one walk.
pub struct SpaceDiv<E> {
    element: Option<E>,
    state: SpaceDivState,
}

impl<E: Styled + 'static> SpaceDiv<E> {
    /// Walk the program once: the base styles onto the element AND the
    /// between-child parts out of the SAME walk (the accumulator the
    /// plain lane drops, the wrapper lane consumes).
    pub fn new(element: E, program: StaticProgram, env: Env) -> Self {
        let mut state = SpaceDivState::default();
        let element = apply_matching_polyfills(
            element,
            &program,
            &env,
            None,
            None,
            None,
            &mut RingState::default(),
            &mut GradState::default(),
            &mut GridState::default(),
            &mut ShadowState::default(),
            &mut FilterState::default(),
            &mut state,
        );
        Self {
            element: Some(element),
            state,
        }
    }

    /// The composed between-child parts (the host's visibility into the
    /// polyfill — the no-silent-failures surface).
    pub fn state(&self) -> &SpaceDivState {
        &self.state
    }
}

impl<E: ParentElement> ParentElement for SpaceDiv<E> {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        // The `:not(:last-child)` shape: every child except the LAST
        // carries the composed margin/border.
        let children: Vec<AnyElement> = elements.into_iter().collect();
        let last = children.len().saturating_sub(1);
        for (index, mut child) in children.into_iter().enumerate() {
            if index < last {
                self.state.patch_child(&mut child);
            }
            if let Some(element) = &mut self.element {
                element.extend([child]);
            }
        }
    }
}

impl<E: IntoElement + 'static> IntoElement for SpaceDiv<E> {
    type Element = Self;

    fn into_element(self) -> Self {
        self
    }
}

impl<E: IntoElement + 'static> Element for SpaceDiv<E> {
    type RequestLayoutState = AnyElement;
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _global_id: Option<&gpui::GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) -> (gpui::LayoutId, Self::RequestLayoutState) {
        let mut element = self
            .element
            .take()
            .expect("the inner element rides the tree the wrapper extends")
            .into_any_element();
        let layout_id = element.request_layout(window, cx);
        (layout_id, element)
    }

    fn prepaint(
        &mut self,
        _global_id: Option<&gpui::GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        _bounds: gpui::Bounds<gpui::Pixels>,
        element: &mut Self::RequestLayoutState,
        window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) -> Self::PrepaintState {
        element.prepaint(window, cx);
    }

    fn paint(
        &mut self,
        _global_id: Option<&gpui::GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        _bounds: gpui::Bounds<gpui::Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) {
        request_layout.paint(window, cx);
    }
}

/// The between-child polyfill's extension trait (the
/// [`crate::TwContainerExt`] precedent: blanket over the element
/// bounds, the prelude carries it).
pub trait TwSpaceExt: Styled + IntoElement + Sized + 'static {
    /// Apply the program under the current `env()`: the base styles
    /// onto the element, the between-child margins/borders onto the
    /// wrapper's children (every non-last child, the v4 rule's own
    /// reading).
    fn tw_space(self, program: StaticProgram) -> SpaceDiv<Self> {
        self.tw_space_env(program, crate::env())
    }

    /// [`TwSpaceExt::tw_space`] under an explicit probe environment —
    /// the `tw_env` pairing.
    fn tw_space_env(self, program: StaticProgram, env: Env) -> SpaceDiv<Self> {
        SpaceDiv::new(self, program, env)
    }
}

impl<E: Styled + IntoElement + 'static> TwSpaceExt for E {}

/// The walk's own route check (the `is_ring_prop` precedent): the ten
/// between-child longhands the state owns — the plain walk's total
/// route (the state the wrapper consumes) and the wrapper's own read.
pub(crate) fn is_space_div_prop(prop: Prop) -> bool {
    matches!(
        prop,
        Prop::SpaceX
            | Prop::SpaceXReverse
            | Prop::SpaceY
            | Prop::SpaceYReverse
            | Prop::DivideX
            | Prop::DivideXReverse
            | Prop::DivideY
            | Prop::DivideYReverse
            | Prop::DivideColor
            | Prop::DivideStyle
    )
}

#[cfg(test)]
mod tests {
    use super::{Axis, SpaceDivState};
    use crate::Env;
    use aeolus_ir::Val;
    use gpui::{IntoElement, Styled, rems};

    /// The walk fills the state from a `space-x-4` program: the value
    /// is the rem the spacing scale bakes, no reverse, no border parts.
    #[test]
    fn the_walk_fills_the_column_margin() {
        use crate::prelude::*;
        let wrapper = gpui::div().tw_space(tw!("space-x-4"));
        let state = wrapper.state();
        assert_eq!(state.space_x, Some((Val::Rem(1.0), "")));
        assert!(!state.space_x_reverse);
        assert!(state.space_y.is_none());
        assert!(state.divide_x.is_none());
        let margin = state.margin(Axis::X).expect("the rem lowers");
        assert_eq!(margin, gpui::Length::Definite(rems(1.0).into()));
    }

    /// The reverse flag composes (the `space-x-reverse` class's own
    /// var) and the divide family lands beside it.
    #[test]
    fn the_walk_fills_reverse_and_divide() {
        use crate::prelude::*;
        let wrapper = gpui::div().tw_space(tw!("space-x-2 space-x-reverse divide-y divide-dashed"));
        let state = wrapper.state();
        assert!(state.space_x_reverse);
        assert_eq!(state.divide_y, Some((Val::Kw(0), "1px")));
        assert_eq!(state.divide_style, Some("dashed"));
        let width = state.border(Axis::Y).expect("the px row lowers");
        assert_eq!(width, gpui::AbsoluteLength::Pixels(gpui::px(1.0)));
    }

    /// The patch carries the end-side margin onto a `Div` child — the
    /// per-child call the extend loop makes for every NON-last child
    /// (the `:not(:last-child)` exclusion is the loop's index test, the
    /// patch itself is per-child).
    #[test]
    fn the_patch_styles_the_child_it_is_given() {
        let state = SpaceDivState {
            space_x: Some((Val::Rem(1.0), "")),
            ..SpaceDivState::default()
        };
        let mut patched = gpui::div().into_any_element();
        let mut untouched = gpui::div().into_any_element();
        state.patch_child(&mut patched);
        let patched = patched.downcast_mut::<gpui::Div>().expect("div child");
        assert_eq!(
            patched.style().margin.right,
            Some(gpui::Length::Definite(rems(1.0).into()))
        );
        assert_eq!(patched.style().margin.left, None);
        let untouched = untouched.downcast_mut::<gpui::Div>().expect("div child");
        assert_eq!(untouched.style().margin.right, None);
    }

    /// The state the plain `tw()` lane composes is empty for a
    /// between-child-free program (the zero-cost shape).
    #[test]
    fn the_empty_state_is_empty() {
        assert!(SpaceDivState::default().is_empty());
        assert!(SpaceDivState::default().margin(Axis::X).is_none());
        let env = Env::default();
        let _ = env;
    }
}
