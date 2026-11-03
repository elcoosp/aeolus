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
    pub background: bool,
    pub content_color: bool,
    pub padding: bool,
    pub border_color: bool,
    pub border_width: bool,
    pub corner_radius: bool,
    pub dimensions: bool,
    pub min_dimensions: bool,
    pub max_dimensions: bool,
    pub gap: bool,
    pub shadow: bool,
    pub margin: bool,
    pub layout: bool,
    pub opacity: bool,
    pub ring: bool,
    pub outline: bool,
    pub cursor: bool,
    pub pointer_events: bool,
    pub text: bool,
}

impl PropertySupport {
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
            Slot::Shadow => self.shadow,
            Slot::Margin => self.margin,
            Slot::Layout => self.layout,
            Slot::Opacity => self.opacity,
            Slot::Ring(_) => self.ring,
            Slot::Outline => self.outline,
            Slot::Cursor => self.cursor,
            Slot::PointerEvents => self.pointer_events,
            Slot::Text(_) => self.text,
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
    pub properties: PropertySupport,
    pub cascade: CascadeModel,
    pub responsive: bool,
    pub container_queries: bool,
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
