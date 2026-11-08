//! # Aeolus core — the Tailwind v4 lowering compiler's semantic layer
#![deny(missing_docs)]
//! `aeolus-core` parses Tailwind CSS v4 class strings, resolves theme tokens,
//! variants, opacity modifiers and conflict groups, and lowers the result
//! into the `StylePlan` IR — **without any framework dependency**.
//!
//! ```
//! use aeolus_core::{TailwindTheme, Resolver};
//!
//! let theme = TailwindTheme::from_theme_css(concat!(
//!     ":root { --primary: oklch(0.205 0 0); --radius: 0.625rem; }",
//!     "@theme inline { --color-primary: var(--primary); --radius-md: calc(var(--radius) - 2px); }",
//! )).unwrap();
//! let resolver = Resolver::new(&theme);
//! let resolution = resolver.resolve("p-4 bg-primary hover:bg-primary/90 rounded-md").unwrap();
//! assert!(resolution.plan.base.background.is_some());
//! assert_eq!(resolution.plan.layers.len(), 1); // the hover layer
//! ```
//!
//! Layering (mission brief §0):
//!
//! | Layer | Crate | Contents |
//! |---|---|---|
//! | Semantics | `aeolus-core` | parser, theme registry, `StylePlan`, conflict trie, caps |
//! | Lowering | `aeolus-xilem`, `aeolus-floem`, ... | `StylePlan` → native styling |
//! | Components | `aeolus-shadcn` | class tables shared by every backend |

pub mod ast;
pub mod cache;
pub mod caps;
pub mod color;
pub mod cond;
pub mod conflict;
pub mod css;
pub mod eval;
pub mod parser;
pub mod plan;
pub mod resolve;
pub(crate) mod selector;
pub mod snapshot;
pub mod theme;

pub use ast::{
    ArbForm, Arbitrary, AttrKind, Breakpoint, ClassToken, INLINE_ELEMENTS, INLINE_VARIANTS, Inline,
    Media, NamedVariant, ParentKind, PseudoElement, Segments, State, Structural, Variant,
};
pub use cache::{CachedPlan, Interner, PlanCache, hash_classes};
pub use caps::{Backend, BackendCapabilities, CascadeModel, PropertySupport, ThemeSwap};
pub use color::{Color, ParseError};
pub use cond::{Atom, Cond, Rule, Test};
pub use conflict::{cn, cn_opt};
pub use css::{CssOutput, CssRule, format_rules};
pub use eval::{Probe, Rel, in_window, split_group_label};
pub use plan::{
    AlignItems, Animation, AutoEdges, Background, BorderColor, BorderStyle, BorderWidth, BoxShadow,
    ConditionalLayer, ContainerType, ContentAlign, ContentColor, CornerRadius, CornerSet, Cursor,
    Dimensions, Display, Ease, EdgeSet, Extras, FlexDirection, Gap, GapAxes, Gradient,
    GradientPart, GradientStop, GradientStops, Grid, GridTrack, Inset, JustifyContent, Lane,
    Layout, Margin, Opacity, Outline, Overflow, OverflowKind, Padding, PointerEvents, Position,
    ResolvedStyle, Ring, RingPart, SizedValue, Slot, StyleDiff, StylePlan, TextProperty, TextValue,
    TrackComponent, TrackSizing, Transform, Transition, TransitionSet, VariantCondition,
    Visibility, diff,
};
pub use resolve::{Resolution, Resolver, Warning};
pub use snapshot::{PLAN_SNAPSHOT_FORMAT, PLAN_SNAPSHOT_VERSION, SnapshotError};
pub use theme::{
    AnimationSpec, BoxShadowSpec, CustomUtility, CustomVariant, TAILWIND_DEFAULT_PALETTE,
    TAILWIND_DEFAULT_THEME, TailwindTheme, Token,
};

/// The Shadcn default theme (light + dark) as `@theme`-compatible CSS.
///
/// Values are verbatim from shadcn-ui v4 (`apps/v4/app/globals.css`,
/// shadcn-ui/ui @ HEAD 2026-11); `shadcn_default()` ships both palettes.
pub mod shadcn_css {
    /// The Shadcn neutral light palette (`:root` block, verbatim).
    pub const LIGHT: &str = include_str!("../data/shadcn-light.css");
    /// The Shadcn neutral dark palette (`.dark` block, verbatim).
    pub const DARK: &str = include_str!("../data/shadcn-dark.css");
}

/// Build the standard Shadcn light theme.
pub fn shadcn_default() -> Result<TailwindTheme, ParseError> {
    TailwindTheme::from_theme_css(shadcn_css::LIGHT)
}

/// Build the standard Shadcn dark theme.
pub fn shadcn_dark() -> Result<TailwindTheme, ParseError> {
    let light = shadcn_default()?;
    light.dark_variant(shadcn_css::DARK)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shadcn_theme_loads() {
        let t = shadcn_default().unwrap();
        assert!(t.color("primary").is_some());
        assert!(t.color("primary-foreground").is_some());
        assert_eq!(t.radius("md"), Some(8.0));
        let d = shadcn_dark().unwrap();
        assert!(d.dark_mode());
        assert!((d.color("background").unwrap().l - 0.145).abs() < 1e-3);
    }

    #[test]
    fn plan_shape_mission_brief() {
        let theme = shadcn_default().unwrap();
        let r = Resolver::new(&theme)
            .resolve("p-4 bg-primary hover:bg-primary/90")
            .unwrap();
        assert_eq!(r.warnings, Vec::new());
        assert!(r.plan.base.background.is_some());
        assert_eq!(r.plan.layers.len(), 1);
        assert_eq!(r.plan.layers[0].condition, VariantCondition::Hover);
        let bg = r.plan.layers[0].style.background.unwrap();
        let base_bg = r.plan.base.background.unwrap();
        // hover = 90% opacity of the same hue (color-mix in oklab)
        assert_eq!(base_bg.0.l, bg.0.l);
        assert!((bg.0.alpha - 0.9).abs() < 1e-4);
    }
}
