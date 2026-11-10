//! Declarations, layers and programs (architecture plan §3.3).
//!
//! ```text
//! Decl    = one longhand + one value
//! Layer   = one condition + the longhands it sets (PropMask) + its decls
//! Program = layers sorted by condition specificity (layer 0 = base)
//! ```
//!
//! The static form is `&'static [Layer]` over `&'static [Decl]` slices —
//! a `static` item end to end, `const`-constructible. The dynamic form
//! (runtime theme tiers) internes one `Box<[Decl]>` and shares it; no
//! per-frame allocation, no locks on the UI path.

use crate::cond::Cond;
use crate::prop::{Prop, PropMask};
use crate::val::Val;

/// One longhand declaration (`padding-top: 1rem`).
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Decl {
    /// The longhand.
    pub prop: Prop,
    /// The value.
    pub val: Val,
}

/// One cascade layer: the declarations that fire under one condition.
///
/// `mask` mirrors `decls` (one bit per declaration's longhand) so layer
/// interaction is a bit operation BEFORE any declaration is read: two
/// layers with disjoint masks cannot conflict, and a mask that covers
/// another's bits tells the backend exactly which earlier declarations a
/// later layer evicts. The mask is data, not a cache — an inconsistent
/// `Layer` is a bug, so [`Layer::new`] recomputes it and the const path
/// asserts it (tests pin both).
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Layer {
    /// The condition this layer fires under ([`Cond::Base`] for layer 0).
    pub cond: Cond,
    /// The declarations, already in [`Prop`] cascade order.
    pub decls: &'static [Decl],
    /// One bit per declaration longhand — always `PropMask::of(decls')`
    /// masks, kept in sync by construction.
    pub mask: PropMask,
}

impl Layer {
    /// Build a layer, deriving the mask from the declarations (the
    /// dynamic-form constructor; the static form writes the literals and
    /// the tests pin the invariant).
    pub const fn new(cond: Cond, decls: &'static [Decl]) -> Self {
        // Const `for` isn't available; index loop over the slice.
        let mut mask = PropMask::EMPTY;
        let mut i = 0;
        while i < decls.len() {
            mask = mask.insert(decls[i].prop);
            i += 1;
        }
        Self { cond, decls, mask }
    }

    /// The longhands this layer sets (the mask's longhand list).
    pub fn props(&self) -> Vec<Prop> {
        self.mask.to_vec()
    }
}

/// A compiled program: layers sorted by condition specificity, layer 0
/// the base rule. Backends interpret front to back; dominance between two
/// layers is `mask.intersect(other) != EMPTY` plus the sort order.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Program {
    /// The layers, specificity-sorted (stable — source order breaks ties).
    pub layers: &'static [Layer],
}

impl Program {
    /// Wrap a slice as a program. The slice is expected pre-sorted (the
    /// compile stage emits it sorted); [`Program::is_sorted`] verifies and
    /// [`Program::sorted`] repairs the dynamic form.
    pub const fn new(layers: &'static [Layer]) -> Self {
        Self { layers }
    }

    /// True when the layers are in non-decreasing specificity order
    /// (stable ties included). A `const` walk over the slice.
    pub const fn is_sorted(&self) -> bool {
        let mut i = 1;
        while i < self.layers.len() {
            if self.layers[i].cond.specificity() < self.layers[i - 1].cond.specificity() {
                return false;
            }
            i += 1;
        }
        true
    }

    /// The layers re-sorted by specificity (stable), as an owned Vec —
    /// the dynamic form's repair path. The static form never calls this:
    /// its slices are `const`-built in order.
    pub fn sorted(&self) -> Vec<Layer> {
        let mut out = self.layers.to_vec();
        out.sort_by_key(|l| l.cond.specificity());
        out
    }

    /// Every longhand the program sets at any condition (the union of the
    /// layer masks) — the "what would this class string touch" answer the
    /// explain/linter surfaces start from.
    pub const fn touched(&self) -> PropMask {
        let mut m = PropMask::EMPTY;
        let mut i = 0;
        while i < self.layers.len() {
            m = m.union(self.layers[i].mask);
            i += 1;
        }
        m
    }

    /// The longhands the BASE rule re-sets on a front-to-back read —
    /// the conflicts `cn()` merges away, as bits. Two shapes count:
    /// a longhand declared TWICE within the base layer (the raw cascade
    /// keeps both fan-outs — `p-4 px-8` sorts into one base rule whose
    /// later duplicates win the read), and a longhand a later
    /// **same-condition** layer re-sets over the base. Conditional
    /// layers (`hover:`, `md:`, ...) are deliberately excluded: they are
    /// additive cascade tiers that win only where they fire, so they
    /// never evict a base declaration (their overlap with base lives in
    /// [`Program::touched`] instead).
    pub const fn base_conflicts(&self) -> PropMask {
        let mut m = PropMask::EMPTY;
        // The base layer's own duplicates: a stable cascade-ordered read
        // keeps the LAST declaration per longhand, so every re-declared
        // longhand is a re-set.
        match self.layers.first() {
            Some(base) => {
                let mut seen = PropMask::EMPTY;
                let mut i = 0;
                while i < base.decls.len() {
                    let prop = base.decls[i].prop;
                    if seen.contains(prop) {
                        m = m.insert(prop);
                    }
                    seen = seen.insert(prop);
                    i += 1;
                }
            }
            // No base layer: nothing to re-set.
            None => return PropMask::EMPTY,
        }
        let mut i = 1;
        while i < self.layers.len() {
            if matches!(self.layers[i].cond, Cond::Base) {
                m = m.union(self.layers[i].mask);
            }
            i += 1;
        }
        // Only bits the base also sets are conflicts.
        match self.layers.first() {
            Some(base) => base.mask.intersect(m),
            None => PropMask::EMPTY,
        }
    }
}
