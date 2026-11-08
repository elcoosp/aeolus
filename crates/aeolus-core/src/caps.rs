//! Backend capability negotiation — the central public contract.
//!
//! Backends answer two questions:
//! 1. which `ResolvedStyle` slots map to native properties (`properties`);
//! 2. whether the backend has a native cascade and what its selector model is
//!    (`cascade`).
//!
//! Unsupported utilities are **dropped with debug-build warnings** — never
//! silently mis-styled.

use crate::plan::{Slot, StylePlan};

/// Which `ResolvedStyle` slots a backend can express natively.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct PropertySupport {
    /// `bg-*`.
    pub background: bool,
    /// `text-{color}`.
    pub content_color: bool,
    /// `p-*`.
    pub padding: bool,
    /// `border-{color}`.
    pub border_color: bool,
    /// `border-*` widths.
    pub border_width: bool,
    /// `rounded-*`.
    pub corner_radius: bool,
    /// `w-*` / `h-*` / `size-*`.
    pub dimensions: bool,
    /// `min-w-*` / `min-h-*`.
    pub min_dimensions: bool,
    /// `max-w-*` / `max-h-*`.
    pub max_dimensions: bool,
    /// `gap-*`.
    pub gap: bool,
    /// `shadow-*`.
    pub shadow: bool,
    /// `m-*`.
    pub margin: bool,
    /// flex/grid display + axis settings.
    pub layout: bool,
    /// `opacity-*`.
    pub opacity: bool,
    /// `ring-*`.
    pub ring: bool,
    /// `outline-*`.
    pub outline: bool,
    /// `cursor-*`.
    pub cursor: bool,
    /// `pointer-events-*`.
    pub pointer_events: bool,
    /// typography properties (font size/weight/...).
    pub text: bool,
}

impl PropertySupport {
    /// Does this backend natively express `slot`?
    pub fn supports(&self, slot: Slot) -> bool {
        match slot {
            Slot::Background => self.background,
            Slot::ContentColor => self.content_color,
            Slot::Padding => self.padding,
            Slot::BorderColor => self.border_color,
            Slot::BorderWidth => self.border_width,
            Slot::CornerRadius => self.corner_radius,
            Slot::Dimensions => self.dimensions,
            Slot::MinDimensions => self.min_dimensions,
            Slot::MaxDimensions => self.max_dimensions,
            Slot::Gap => self.gap,
            Slot::Shadow | Slot::ShadowRest | Slot::InsetShadows => self.shadow,
            Slot::Margin | Slot::MarginAuto => self.margin,
            Slot::Layout | Slot::FlexBasis | Slot::Order | Slot::AlignContent | Slot::GapAxes => {
                self.layout
            }
            Slot::Opacity => self.opacity,
            Slot::Ring(_) => self.ring,
            Slot::Outline | Slot::BorderStyle => self.outline,
            Slot::Cursor => self.cursor,
            Slot::PointerEvents => self.pointer_events,
            Slot::Text(_) => self.text,
            // v0.2 extras have no per-slot capability flags yet; they are
            // negotiated by the backend lowering (native vs emulated vs
            // reported — the parity contract, plan §4.1).
            Slot::Position
            | Slot::Inset
            | Slot::ZIndex
            | Slot::Overflow
            | Slot::Visibility
            | Slot::AspectRatio
            | Slot::Grid
            | Slot::Transform
            | Slot::Transition
            | Slot::Animation
            // v0.6 gradient: negotiated by the backend lowering (masonry
            // native, floem reported — the parity contract, plan §4.1).
            // v0.7 container containment: same (no backend has a native
            // container-query surface; the reactive probe path emulates).
            | Slot::Gradient(_)
            | Slot::ContainerType => false,
        }
    }
}

/// The selector/cascade model of a backend.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CascadeModel {
    /// Native selector/cascade: hover/focus/disabled re-resolution happens
    /// inside the framework, zero view work (Masonry `PropertyStack`,
    /// Floem `StyleSelector`, Vizia CSS).
    Native,
    /// Per-state style closures only (Iced `Catalog`): Aeolus emits
    /// state-keyed style maps; re-resolution is per-frame by the framework.
    StateClosures,
    /// Neither: Aeolus emits a view-level conditional resolver (fallback).
    ViewLevel,
    /// No conditional styling at all.
    None,
}

/// Capability matrix entry for one backend.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BackendCapabilities {
    /// Per-slot native support.
    pub properties: PropertySupport,
    /// The selector/cascade model.
    pub cascade: CascadeModel,
    /// Breakpoint variants (`md:`) handled natively.
    pub responsive: bool,
    /// Container queries (`@md:`) handled natively.
    pub container_queries: bool,
    /// `dark:` handled natively.
    pub dark_mode: bool,
    /// Typography via normal properties (false = side-channel needed).
    pub typography: bool,
    /// Theme can be swapped at runtime without an app rebuild.
    pub theme_swap: ThemeSwap,
}

/// How a runtime theme swap propagates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeSwap {
    /// Free — styles are re-resolved every frame (Iced, egui).
    Free,
    /// Works with framework cache invalidation (Masonry
    /// `set_default_properties`, Floem reactive styles).
    WithInvalidation,
    /// Requires an explicit app/view rebuild.
    RequiresRebuild,
}

/// The lowering contract (locked §3.2).
pub trait Backend {
    /// The backend's lowered-style type (a `PropertyStack`, a Floem `Style`, ...).
    type View;
    /// Lower a `StylePlan` into the backend's native styling construct.
    fn lower(plan: &StylePlan, caps: &BackendCapabilities) -> Self::View;
}

/// Report an unsupported slot the way every backend must: drop + warn in
/// debug builds. `class` is the offending class string, for diagnostics.
pub fn report_unsupported(slot: Slot, class: &str) {
    if cfg!(debug_assertions) {
        tracing_or_stderr(format!(
            "[aeolus] dropping unsupported slot {slot:?} for class {class:?} on this backend"
        ));
    }
}

fn tracing_or_stderr(msg: String) {
    // v1 keeps the core dependency-free; backends with `tracing` should call
    // `tracing::warn!` directly. This hook stays for the pure-core path.
    eprintln!("{msg}");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The `supports()` routing table: v0.1/v0.2 slots negotiate through the
    /// per-slot flags; the extras family (position/…/grid/transform/…),
    /// gradients and the v0.7 container containment have no flags — they are
    /// negotiated by the backend lowering. Pin both halves.
    #[test]
    fn supports_routes_slots_to_their_flags() {
        let full = PropertySupport {
            background: true,
            content_color: true,
            padding: true,
            border_color: true,
            border_width: true,
            corner_radius: true,
            dimensions: true,
            min_dimensions: true,
            max_dimensions: true,
            gap: true,
            shadow: true,
            margin: true,
            layout: true,
            opacity: true,
            ring: true,
            outline: true,
            cursor: true,
            pointer_events: true,
            text: true,
        };
        // The flag-backed slots answer their flag.
        assert!(full.supports(Slot::Background));
        assert!(full.supports(Slot::Text(crate::plan::TextProperty::FontSize)));
        // Shadow-family slots share one flag; margin covers the auto form.
        assert!(full.supports(Slot::ShadowRest));
        assert!(full.supports(Slot::MarginAuto));
        // Layout covers the flex/grid axis family.
        assert!(full.supports(Slot::GapAxes));
        // The extras family is NEVER flag-backed — all default to false
        // even on an all-flags backend (negotiated by the lowering).
        assert!(!full.supports(Slot::Position));
        assert!(!full.supports(Slot::Grid));
        assert!(!full.supports(Slot::Transform));
        assert!(!full.supports(Slot::Gradient(crate::plan::GradientPart::From)));
        assert!(!full.supports(Slot::ContainerType));
        // The default (no flags) supports nothing flag-backed.
        let none = PropertySupport::default();
        assert!(!none.supports(Slot::Background));
        assert!(!none.supports(Slot::Padding));
    }

    /// `report_unsupported` is total (never panics) for any slot/class —
    /// the drop-with-warning contract runs in debug builds.
    #[test]
    fn report_unsupported_is_total() {
        report_unsupported(Slot::ContainerType, "@container/card-header");
        report_unsupported(Slot::Background, "bg-red-500");
    }
}
