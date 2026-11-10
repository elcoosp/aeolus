//! Variant → `Cond` classification: the compile stage's flat fold
//! (architecture plan §3.3 / `aeolus_ir::cond`'s own docs).
//!
//! The IR model is deliberately flat — one condition per layer — so a
//! variant STACK folds to its single dominant condition: the
//! highest-specificity segment, source order breaking ties. What folds
//! today is the typed `Cond` surface (`aeolus_ir::Cond`'s every
//! constructor: probe states, dark, breakpoints, containers, the
//! group/peer probes). What does NOT fold (has-/data-/aria-/arbitrary
//! variants) comes back as a typed unsupported-variant diagnostic —
//! the next P2 slice's job, never a silent drop.

use aeolus_ir::{Bp, Cond};

/// Fold the variant stack into the one condition the layer fires under.
/// An EMPTY stack is the bare utility: [`Cond::Base`].
///
/// # Errors
/// The first variant outside the typed surface, verbatim — the caller
/// renders it as an unsupported-variant diagnostic with the segment's
/// own span.
pub fn fold(variants: &[&str]) -> Result<Cond, String> {
    let mut dominant = Cond::Base;
    for &variant in variants {
        let cond = classify(variant).ok_or_else(|| variant.to_string())?;
        if cond.specificity() >= dominant.specificity() {
            dominant = cond;
        }
    }
    Ok(dominant)
}

/// The typed classification for ONE variant segment. The named
/// group/peer label (`group-hover/menu`) folds to its probe: the NAME
/// has no IR slot yet (cond.rs documents the threading as a
/// compile-stage job this slice leaves to the codegen).
fn classify(variant: &str) -> Option<Cond> {
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

/// The `--breakpoint-*` defaults, by name (`aeolus_ir::Bp::name`'s
/// inverse). The theme tier may re-map the rem values, not the names.
fn breakpoint(name: &str) -> Option<Bp> {
    match name {
        "sm" => Some(Bp::Sm),
        "md" => Some(Bp::Md),
        "lg" => Some(Bp::Lg),
        "xl" => Some(Bp::Xl),
        "2xl" => Some(Bp::Xl2),
        _ => None,
    }
}
