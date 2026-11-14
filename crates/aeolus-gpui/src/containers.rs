//! The element-bounds CONTAINER WRAPPER (architecture plan §3.5's
//! `@container` row, the half the docs keep assigning to the host: "the
//! element-bounds WRAPPER — who MEASURES the widths"): a stateless GPUI
//! element that wraps one child, publishes the width of the box IT
//! assigns that child at the layout phase (its own prepaint, where
//! bounds exist), and lets the landed walk half —
//! [`crate::TwExt::tw_containers`] — answer the named container queries
//! through the [`MeasuredContainers`] hand-off.
//!
//! ```ignore
//! use aeolus_gpui::prelude::*;
//!
//! // one hand-off per view; the wrapper publishes into it, the styled
//! // descendants read through it
//! let containers = MeasuredContainers::new();
//!
//! div()
//!     .tw_container("card", &containers)
//!     .child(
//!         div()
//!             .tw_containers(tw!("@md/card:p-4 @max-lg/card:hidden"), &containers),
//!     )
//! ```
//!
//! The division of labour is the seam's own: the wrapper NEVER styles
//! (it carries no refinement of its own — it is a measuring relay), the
//! walk NEVER measures (it consults the [`ContainerProbe`] answers).
//! The handshake rides the measurement delta: the wrapper publishes at
//! prepaint and asks for a re-render only when the width CHANGED
//! ([`MeasuredContainers::publish`]'s answer) — a stable width never
//! schedules a frame, a resize asks for exactly one.
//!
//! The frame beat is GPUI's own immediate-mode beat, the same one the
//! animation lane runs: the styles a frame BUILDS read the widths
//! measured at the previous frame's layout phase (a freshly mounted
//! wrapper publishes after its first layout, and the named layers
//! apply on the re-render the delta handshake asks for; until then the
//! layers ride the CSS-quiet miss the walk keeps for an unpublished
//! name). Nothing is lost — the measurement exists the moment the
//! wrapper's box exists; it reaches the styles one build later.

use gpui::{
    AnyElement, App, Bounds, Element, ElementId, GlobalElementId, InspectorElementId, IntoElement,
    LayoutId, ParentElement, Pixels, SharedString, Window,
};

use aeolus_rt::MeasuredContainers;

/// The width the wrapper publishes, in rem: the element bounds' width
/// divided by the window's rem size — the same arithmetic
/// [`crate::begin_frame`] runs for the viewport. A degenerate
/// (non-positive) rem size is NO measurement ([`None`]): the quiet
/// miss, never a half verdict from a division the window never
/// defined.
///
/// Exposed for tests (the [`crate::text_align_value`] precedent).
pub fn container_rem(width: Pixels, rem_size: Pixels) -> Option<f32> {
    let rem = rem_size.as_f32();
    (rem > 0.0).then(|| width.as_f32() / rem)
}

/// Wrap one element in a measuring container: the wrapper publishes
/// the width of the box it assigns that element under `name` — the
/// element-bounds half of the named-container seam (plan §3.5's
/// `@container` row), paired with [`crate::TwExt::tw_containers`] reading the
/// same [`MeasuredContainers`] hand-off.
///
/// The wrapper is a STATELESS element: no per-element state, no
/// refinement of its own — the name doubles as the element id, the
/// shared handle is the only carrier, and the child receives every
/// phase (layout, prepaint, paint) unchanged.
pub trait TwContainerExt: IntoElement + Sized {
    /// Wrap `self` in a measuring container published under `name`.
    /// Give the wrapper's children the program through
    /// [`crate::TwExt::tw_containers`] with the SAME handle.
    fn tw_container(
        self,
        name: impl Into<SharedString>,
        containers: &MeasuredContainers,
    ) -> ContainerQuery<Self> {
        ContainerQuery::new(name, containers, self)
    }
}

impl<E: IntoElement> TwContainerExt for E {}

/// The measuring wrapper element (see [`TwContainerExt`]).
pub struct ContainerQuery<E> {
    name: SharedString,
    containers: MeasuredContainers,
    element: Option<E>,
}

impl<E> ContainerQuery<E> {
    /// Wrap one element as the measured container `name`.
    pub fn new(name: impl Into<SharedString>, containers: &MeasuredContainers, element: E) -> Self {
        Self {
            name: name.into(),
            containers: containers.clone(),
            element: Some(element),
        }
    }
}

impl<E: ParentElement> ParentElement for ContainerQuery<E> {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        if let Some(element) = &mut self.element {
            element.extend(elements);
        }
    }
}

impl<E: IntoElement + 'static> IntoElement for ContainerQuery<E> {
    type Element = Self;

    fn into_element(self) -> Self {
        self
    }
}

impl<E: IntoElement + 'static> Element for ContainerQuery<E> {
    type RequestLayoutState = AnyElement;
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        Some(ElementId::Name(self.name.clone()))
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut element = self
            .element
            .take()
            .expect("the wrapped element was set at construction")
            .into_any_element();
        let layout_id = element.request_layout(window, cx);
        (layout_id, element)
    }

    fn prepaint(
        &mut self,
        _global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        element: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        // THE MEASUREMENT: this element's assigned box is the named
        // container's width (the wrapper carries no styling of its own,
        // so the box it gives its child IS the box it owns). Publish in
        // rem — the seam's currency — and ask for a re-render only when
        // the measurement changed (the handshake that converges: a
        // stable width never schedules a frame).
        if let Some(rem) = container_rem(bounds.size.width, window.rem_size()) {
            let changed = self.containers.publish(&self.name, rem);
            if changed {
                window.refresh();
            }
        }
        element.prepaint(window, cx);
    }

    fn paint(
        &mut self,
        _global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        element: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        element.paint(window, cx);
    }
}
