//! The measured-containers hand-off (architecture plan §3.5's
//! `@container` row, the "element-bounds WRAPPER — who MEASURES the
//! widths" half): the shared, interior-mutable [`ContainerProbe`] the
//! measuring half publishes real layout widths into and every walk
//! reads through the seam's ONE question.
//!
//! The shape is the [`crate::ClassCell`]'s own: a `Rc<RefCell<…>>`
//! handle — `!Sync` by construction (the UI-thread note in
//! type-system form), cloned per participant, one shared state. The
//! measuring wrapper clones the handle and publishes at its prepaint
//! phase (GPUI's layout phase, where bounds exist); the styled
//! descendants' clones read the same answers through
//! [`ContainerProbe::container`] — the [`crate::StaticProgram`] walks
//! (`tw_containers` and friends) consume the handle unchanged, because
//! it IS the probe.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use aeolus_compile::ContainerProbe;

/// The named containers' measured widths, as a shared
/// [`ContainerProbe`]: the layout-phase hand-off between the measuring
/// wrapper (who computes the width) and the program walks (who consume
/// the answers).
///
/// The handle is cheap to clone and the clones share ONE state — the
/// wrapper publishes through its clone at the layout phase, the
/// descendants' clones read through the seam. An unpublished name is
/// the CSS-quiet miss ([`ContainerProbe::container`] answers `None`),
/// never an exclusion; a re-publish under the same name overwrites
/// (the resize story), and [`MeasuredContainers::publish`] answers
/// whether the width CHANGED — the handshake the wrapper's
/// re-render request rides (the same width published again is not a
/// change, so the loop converges).
///
/// Entries persist until overwritten or cleared — the host owns the
/// handle's lifetime (one per view/session, the [`crate::ClassCell`]
/// precedent) and clears it when the containers unmount
/// ([`MeasuredContainers::clear`]).
///
/// `!Sync` by construction ([`Rc`]): publish and read from the UI
/// thread, exactly like the [`crate::ClassCell`].
#[derive(Clone, Debug, Default)]
pub struct MeasuredContainers {
    widths: Rc<RefCell<HashMap<String, f32>>>,
}

impl MeasuredContainers {
    /// An empty hand-off: every name a quiet miss until measured.
    pub fn new() -> Self {
        Self::default()
    }

    /// Publish a named container's inner width in rem. Answers whether
    /// the measurement CHANGED (a first publish counts, the same width
    /// again does not) — the measuring wrapper's re-render request
    /// rides this handshake, so a stable width never schedules a
    /// frame and a resize asks for exactly one.
    pub fn publish(&self, name: &str, rem: f32) -> bool {
        let mut widths = self.widths.borrow_mut();
        match widths.get(name) {
            Some(&previous) if previous == rem => false,
            _ => {
                widths.insert(name.to_owned(), rem);
                true
            }
        }
    }

    /// The width published for `name`, if any.
    pub fn measured(&self, name: &str) -> Option<f32> {
        self.widths.borrow().get(name).copied()
    }

    /// Drop every measurement — the host's reset when the containers
    /// unmount (the next frame answers the quiet miss until a wrapper
    /// publishes again).
    pub fn clear(&self) {
        self.widths.borrow_mut().clear();
    }
}

impl ContainerProbe for MeasuredContainers {
    fn container(&self, name: &str) -> Option<f32> {
        self.measured(name)
    }
}
