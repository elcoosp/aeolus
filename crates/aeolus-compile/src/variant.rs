//! Variant → `Cond` classification: the compile stage's flat fold
//! (architecture plan §3.3 / `aeolus_ir::cond`'s own docs).
//!
//! The IR model is deliberately flat — one condition per layer — so a
//! variant STACK folds to its single dominant condition: the
//! highest-specificity segment, source order breaking ties. What folds
//! today is the typed `Cond` surface (`aeolus_ir::Cond`'s every
//! constructor: probe states, dark, breakpoints, containers, the
//! group/peer probes) PLUS the registered custom-variant aliases (the
//! `custom` module's [`crate::Customs`] — the pipeline's fold is
//! generic over the [`crate::ClassifyVariant`] seam, so a host can
//! supply its own table). What does NOT fold (has-/data-/aria-/
//! arbitrary variants, unregistered spellings) comes back as a typed
//! unsupported-variant diagnostic — never a silent drop.

use aeolus_ir::{Bp, Cond};

use crate::custom::ClassifyVariant;

/// Why one variant segment did not fold: the segment is outside the
/// typed [`Cond`] surface. A struct, not a bare `String` — the miss is
/// DATA (one meaning: this segment refused) with the segment text as
/// its payload, the same typed shape every other seam in the pipeline
/// reports through. The diagnostics layer renders
/// [`VariantMiss::variant`] verbatim as an unsupported-variant
/// diagnostic.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct VariantMiss {
    /// The variant segment that refused, verbatim (`"has-data-[x]"`).
    pub(crate) variant: String,
}

impl VariantMiss {
    /// The miss for one refusing segment.
    pub(crate) fn new(variant: &str) -> Self {
        Self {
            variant: variant.to_string(),
        }
    }
}

/// Fold the variant stack into the one condition the layer fires under.
/// An EMPTY stack is the bare utility: [`Cond::Base`]. The classifier
/// extends the built-in table with the registered custom-variant
/// aliases (built-ins win name collisions — the registry is the
/// oracle, the registrations are additive surface).
///
/// # Errors
/// The first variant outside the typed surface (built-in or
/// registered), as a typed [`VariantMiss`] — the caller renders it as
/// an unsupported-variant diagnostic with the segment's own span.
pub fn fold(variants: &[&str], classifier: &impl ClassifyVariant) -> Result<Cond, VariantMiss> {
    let mut dominant = Cond::Base;
    for &variant in variants {
        let cond = classify(variant, classifier).ok_or_else(|| VariantMiss::new(variant))?;
        if cond.specificity() >= dominant.specificity() {
            dominant = cond;
        }
    }
    Ok(dominant)
}

/// The typed classification for ONE variant segment: the built-in
/// table first, the registered aliases second. The named group/peer
/// label (`group-hover/menu`) folds to its probe: the NAME has no IR
/// slot yet (cond.rs documents the threading as a compile-stage job
/// this slice leaves to the codegen). The CSS emit arm reuses this as
/// the REFUSAL oracle (what a segment IS) while rendering the selector
/// shape itself (HOW it shows) — the two must agree, and `tests/css.rs`
/// pins it.
pub(crate) fn classify(variant: &str, classifier: &impl ClassifyVariant) -> Option<Cond> {
    classify_builtin(variant).or_else(|| classifier.classify(variant))
}

/// The built-in table's classification for ONE variant segment — the
/// closed spelling set the typed `Cond` surface covers. The custom
/// aliases ride [`classify`]'s second half, never this table.
pub(crate) fn classify_builtin(variant: &str) -> Option<Cond> {
    let (name, _label) = match variant.split_once('/') {
        Some((name, label)) => (name, Some(label)),
        None => (variant, None),
    };
    if let Some(bp) = breakpoint(name) {
        return Some(Cond::Bp(bp));
    }
    if let Some(rest) = name.strip_prefix("min-") {
        return breakpoint(rest).map(Cond::Min);
    }
    if let Some(rest) = name.strip_prefix("max-") {
        return breakpoint(rest).map(Cond::Max);
    }
    if let Some(rest) = name.strip_prefix("@max-") {
        return breakpoint(rest).map(Cond::MaxContainer);
    }
    if let Some(rest) = name.strip_prefix('@') {
        return breakpoint(rest).map(Cond::Container);
    }
    match name {
        "dark" => Some(Cond::Dark),
        "hover" => Some(Cond::Hover),
        "focus" => Some(Cond::Focus),
        "focus-visible" => Some(Cond::FocusVisible),
        "focus-within" => Some(Cond::FocusWithin),
        "active" => Some(Cond::Active),
        "disabled" => Some(Cond::Disabled),
        "visited" => Some(Cond::Visited),
        "checked" => Some(Cond::Checked),
        "placeholder-shown" => Some(Cond::PlaceholderShown),
        "selection" => Some(Cond::Selection),
        "placeholder" => Some(Cond::Placeholder),
        "group-hover" => Some(Cond::GroupHover),
        "group-focus" => Some(Cond::GroupFocus),
        "peer-hover" => Some(Cond::PeerHover),
        "peer-focus" => Some(Cond::PeerFocus),
        _ => None,
    }
}

/// The named breakpoint (`"sm"` → `Bp::Sm`) — a reverse walk over
/// [`Bp::ALL`]/[`Bp::name`], so the IR's own table is the single source
/// and a new breakpoint cannot drift the compile stage's spelling table
/// (this used to be a parallel `match` wall; it is the IR's data now).
pub(crate) fn breakpoint(name: &str) -> Option<Bp> {
    Bp::ALL.iter().copied().find(|bp| bp.name() == name)
}
