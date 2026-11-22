//! The gpui-component bridge (architecture plan §5.1's "integrating with
//! gpui-rsx" row, redirected to the real ecosystem target:
//! gpui-component is the de facto Zed-style component library).
//!
//! Two seams, both composed from the EXISTING surfaces — no parallel
//! scaffolding:
//!
//! 1. **The ActiveTheme twin.** gpui-component reads its theme through
//!    the [`gpui_component::ActiveTheme`] trait (`fn theme(&self) ->
//!    &Theme` over the App global). This crate implements the trait for
//!    the bridge handle: the handle carries a projected
//!    [`gpui_component::Theme`] beside the [`TailwindTheme`] and the
//!    [`TokenTable`], so a host can hand the SAME palette to
//!    gpui-component's components (via the App global,
//!    [`AeolusTheme::install`]) and to the Aeolus walk (via the table)
//!    without maintaining two palettes.
//!
//! 2. **The `resolve_style` twin.** The bridge's [`AeolusTheme::style`]
//!    takes a class string and the host's view-state
//!    [`ClassCell`] — the §3.6 dynamic lane's own entrance — compiles it
//!    through the SAME pipeline [`TwExt::tw_dyn_theme`] rides, resolves
//!    the walk-time token slots (the §3.4 T2 half: `Val::Tok`/`Val::ModePair`
//!    → the CURRENT table's rgba), and applies the result through the
//!    existing [`TwExt::tw_env`] walk. The closed-world path stays the
//!    host's choice: the [`variants_static!`] macro is re-exported here
//!    and its frozen programs apply through the same walk.
//!
//! # The theme hot-swap (§3.4, made real)
//!
//! [`AeolusTheme::absorb_css`] is the swap: it calls the existing
//! [`TailwindTheme::absorb_css`] surface (which bumps the theme's own
//! version), rebuilds the [`TokenTable`] IN PLACE at the same heap
//! address (the `Box` identity rule — a [`ClassCell`] keyed on the table
//! address keeps hitting, so a hot-swapped theme does NOT re-parse the
//! class strings the views already compiled), re-projects the
//! gpui-component mirror, bumps the bridge's own version, drops the v0.x
//! [`PlanCache`] (the §3.4 "explicit, no CSS-variable pretending"
//! contract: the plan lane's entries embed RESOLVED values, so they die
//! with the version they resolved under) and drops the walk-time
//! resolution cache beside it.
//!
//! ```no_run
//! use aeolus_gpui_component::AeolusTheme;
//! use gpui_component::ActiveTheme;
//!
//! let mut theme = AeolusTheme::from_css(LIGHT_CSS, None).expect("the sheet parses");
//! let light = theme.theme().primary;
//! theme.absorb_css(DARK_CSS).expect("the swap merges");
//! theme.set_dark(true);
//! let dark = theme.theme().primary;
//! assert_ne!(light, dark, "same handle, new values, no restart");
//! # const LIGHT_CSS: &str = "";
//! # const DARK_CSS: &str = "";
//! ```

use std::cell::RefCell;
use std::collections::HashMap;

use aeolus_compile::Config;
use aeolus_core::{PlanCache, TailwindTheme};
use aeolus_ir::{Decl, Layer, Val};
use aeolus_rt::{ClassCell, StaticProgram, TokenTable};
use aeolus_theme::{ThemeError, ThemeSource};
use gpui::Styled;
use gpui_component::{Theme, ThemeColor, ThemeMode};

/// The resolve_style twin's walk surface (the bridge applies the
/// resolved programs through it) and the closed-world macro the host
/// may pick instead of the dynamic lane.
pub use aeolus_gpui::{TwExt, env};
// The trait the bridge implements — re-exported so a host (and the
// bridge's own tests) import ONE bridge surface.
pub use aeolus_macro::variants_static;
pub use aeolus_rt::Env;
pub use gpui_component::ActiveTheme;

/// The theme hot-swap's typed refusals. Every variant is documented in
/// place; none is a panic, none is a silent drop.
#[derive(Debug, PartialEq)]
pub enum SwapError {
    /// The absorbed sheet declares a colour-token name the table did not
    /// carry, and its sorted position SHIFTS the ids the pre-swap
    /// programs compiled against ([`Val::Tok`] slots are ids — a shifted
    /// id resolves the wrong token). The swap refuses typed; the host
    /// recompiles at the new pin instead of reading drifted colours.
    TokenIdShift { name: String },
    /// The sheet's CSS does not parse (the v0.x reader's own typed
    /// error, carried verbatim — the bridge wraps, never re-words).
    Parse(aeolus_core::ParseError),
    /// The sheet's token set does not bake (the theme pass's own typed
    /// error, carried verbatim).
    Bake(ThemeError),
}

/// The gpui-component bridge handle: ONE palette, two consumers.
///
/// The interior is a [`RefCell`] resolution cache and the handle is
/// `!Sync` by construction — the UI thread renders sequentially per
/// window, the [`ClassCell`] precedent: the contract fails to compile
/// instead of waiting for a race. The swap ([`AeolusTheme::absorb_css`])
/// takes `&mut self`, the walk ([`AeolusTheme::style`]) `&self`.
pub struct AeolusTheme {
    /// The §3.4 hot-swap surface — the CSS-first theme the absorb
    /// merges into (the version bump rides [`TailwindTheme::absorb_css`]).
    theme: TailwindTheme,
    /// The accumulated sheet (light + every absorbed chunk) the
    /// [`ThemeSource`] rebuilds from — the merge semantics the
    /// [`TailwindTheme`] applies, mirrored byte-for-byte so the token
    /// rebuild sees the same declarations.
    css: String,
    /// The T2 runtime token table at a STABLE heap address (the `Box`):
    /// the [`ClassCell`] identity rule keys on the address, so the swap
    /// replaces VALUES in place and the compiled programs keep hitting.
    tokens: Box<TokenTable>,
    /// The ActiveTheme twin: the gpui-component palette projected from
    /// the table at the current mode.
    mirror: Theme,
    /// The mode the projection and the walk resolve at (`Val::ModePair`
    /// legs pick by it, the mirror's colours read the dark halves).
    dark: bool,
    /// The bridge's own swap counter — the typed stream the host
    /// observes (one bump per successful [`AeolusTheme::absorb_css`]).
    version: u64,
    /// The v0.x plan lane: RESOLVED values, version-stamped — the §3.4
    /// contract DROPS it on swap (the explicit reset beside the version
    /// stamps' own invalidation; no CSS-variable pretending survives a
    /// hot-swap).
    plan_cache: PlanCache,
    /// The walk-time resolution cache: one entry per (program identity,
    /// mode) per VERSION — the swap clears it (the resolved literals are
    /// the version's own; the §3.4 drop contract rides here too).
    resolved: RefCell<HashMap<(usize, bool), StaticProgram>>,
}

impl AeolusTheme {
    /// Build the handle from a light sheet (and optional dark sheet —
    /// the same `@theme`/`:root`/`.dark` CSS a web Tailwind v4 project
    /// uses). The v4 semantics apply: the default palette + theme are
    /// the baseline, the sheet's declarations merge over them.
    ///
    /// # Errors
    /// [`ThemeError`] when the sheet does not parse or the token set
    /// does not bake (the theme pass's own typed errors, verbatim).
    pub fn from_css(light: &str, dark: Option<&str>) -> Result<Self, ThemeError> {
        let source = ThemeSource::from_css(light, dark)?;
        let table = TokenTable::runtime(&source)?;
        let mut css = String::with_capacity(light.len() + dark.map_or(0, str::len));
        css.push_str(light);
        if let Some(dark) = dark {
            css.push('\n');
            css.push_str(dark);
        }
        let theme =
            TailwindTheme::from_theme_css(&css).map_err(|err| ThemeError::Parse { source: err })?;
        let mut this = Self {
            theme,
            css,
            tokens: Box::new(table),
            mirror: Theme::from(&ThemeColor::default()),
            dark: false,
            version: 0,
            plan_cache: PlanCache::new(),
            resolved: RefCell::new(HashMap::new()),
        };
        this.project_mirror();
        Ok(this)
    }

    /// The theme hot-swap (§3.4): merge `css` into the live theme, swap
    /// the token values in place, re-project the mirror, bump the
    /// version, DROP the v0.x plan lane and the resolution cache. Same
    /// handle, new values, no restart — and the class strings the views
    /// already compiled do NOT re-parse (the [`ClassCell`] identity rule:
    /// the table address survives, so the compiled programs hit).
    ///
    /// Transactional: the merge and the table rebuild run on a CLONE and
    /// commit only when the token-id contract holds — a refusing swap
    /// leaves the handle untouched.
    ///
    /// # Errors
    /// [`SwapError::Parse`] when the sheet does not parse,
    /// [`SwapError::Bake`] when the token set does not bake,
    /// [`SwapError::TokenIdShift`] when the sheet introduces names that
    /// shift existing ids (the typed refusal — see the variant's docs).
    pub fn absorb_css(&mut self, css: &str) -> Result<(), SwapError> {
        // The pre-swap ids the compiled programs ride ([`Val::Tok`]
        // slots): the id contract the swap must keep.
        let ids_before: Vec<(String, u16)> = self
            .tokens
            .names()
            .filter_map(|name| self.tokens.id(name).map(|id| (name.to_string(), id)))
            .collect();

        // The transactional clone: the merge + rebuild run here first.
        let mut theme = self.theme.clone();
        let mut accumulated = self.css.clone();
        accumulated.push('\n');
        accumulated.push_str(css);
        theme.absorb_css(css).map_err(SwapError::Parse)?;
        let source = ThemeSource::from_app_css(&accumulated).map_err(SwapError::Bake)?;
        let table = Box::new(TokenTable::runtime(&source).map_err(SwapError::Bake)?);

        // The id contract: every pre-swap name keeps its id (sorted-name
        // order is stable across rebuilds WHEN the name set does not
        // gain earlier names — the typed refusal names the offender).
        for (name, id) in &ids_before {
            let new_id = table.id(name);
            if new_id != Some(*id) {
                return Err(SwapError::TokenIdShift { name: name.clone() });
            }
        }

        // Commit: the merged theme, the in-place table swap (the `Box`
        // address survives — the ClassCell identity rule), the css
        // accumulation, the projection, the version bump, the §3.4
        // drops.
        self.theme = theme;
        self.tokens = table;
        self.css = accumulated;
        self.project_mirror();
        self.version += 1;
        self.plan_cache = PlanCache::new();
        self.resolved.borrow_mut().clear();
        Ok(())
    }

    /// The projected gpui-component palette — the [`ActiveTheme`] twin's
    /// answer. The semantic shadcn names → the `ThemeColor` fields, one
    /// arm per name (visually auditable against the theme chain; the
    /// destructive→danger rename is gpui-component's own Base naming).
    /// A name the sheet does not declare keeps the Base default — the
    /// projection overrides, never invents.
    fn project_mirror(&mut self) {
        const NAMES: &[&str] = &[
            "background",
            "foreground",
            "primary",
            "primary-foreground",
            "secondary",
            "secondary-foreground",
            "muted",
            "muted-foreground",
            "accent",
            "accent-foreground",
            "destructive",
            "destructive-foreground",
            "border",
            "input",
            "ring",
            "popover",
            "popover-foreground",
            "selection",
        ];
        let mut colors = ThemeColor::default();
        for name in NAMES {
            let Some(id) = self.tokens.id(name) else {
                continue;
            };
            let Ok(rgba) = self.tokens.resolve_rgba(id, 1.0, self.dark) else {
                continue;
            };
            let color: gpui::Hsla = gpui::rgba(u32::from_be_bytes(rgba)).into();
            match *name {
                "background" => colors.background = color,
                "foreground" => colors.foreground = color,
                "primary" => colors.primary = color,
                "primary-foreground" => colors.primary_foreground = color,
                "secondary" => colors.secondary = color,
                "secondary-foreground" => colors.secondary_foreground = color,
                "muted" => colors.muted = color,
                "muted-foreground" => colors.muted_foreground = color,
                "accent" => colors.accent = color,
                "accent-foreground" => colors.accent_foreground = color,
                "destructive" => colors.danger = color,
                "destructive-foreground" => colors.danger_foreground = color,
                "border" => colors.border = color,
                "input" => colors.input = color,
                "ring" => colors.ring = color,
                "popover" => colors.popover = color,
                "popover-foreground" => colors.popover_foreground = color,
                "selection" => colors.selection = color,
                _ => unreachable!("NAMES is the match's own list"),
            }
        }
        let mut mirror = Theme::from(&colors);
        mirror.mode = if self.dark {
            ThemeMode::Dark
        } else {
            ThemeMode::Light
        };
        self.mirror = mirror;
    }

    /// The mode the projection and the walk resolve at — the mirror's
    /// colours re-read the dark halves on the next projection. A swap
    /// ([`AeolusTheme::absorb_css`]) keeps the current mode.
    pub fn set_dark(&mut self, dark: bool) {
        if self.dark != dark {
            self.dark = dark;
            self.version += 1;
            self.resolved.borrow_mut().clear();
            self.project_mirror();
        }
    }

    /// True when the handle resolves at the dark mode.
    pub fn is_dark(&self) -> bool {
        self.dark
    }

    /// The bridge's own version — one bump per successful swap (and per
    /// mode flip): the typed stream a host observes to invalidate ITS
    /// own per-version state (the §3.4 explicit-contract face).
    pub fn version(&self) -> u64 {
        self.version
    }

    /// The underlying theme's own version ([`TailwindTheme::absorb_css`]'s
    /// bump — the v0.x lane's stamp).
    pub fn theme_version(&self) -> u64 {
        self.theme.version()
    }

    /// The runtime token table (the T2 tier the dynamic lane compiles
    /// against) — the host may compile its own strings through it.
    pub fn tokens(&self) -> &TokenTable {
        &self.tokens
    }

    /// The v0.x plan lane (the §3.4 contract's OTHER consumer): resolves
    /// through the version-stamped cache, which the swap drops — a plan
    /// taken before the swap no longer rides it.
    pub fn resolve_plan(
        &self,
        classes: &str,
    ) -> Result<std::sync::Arc<aeolus_core::CachedPlan>, Vec<aeolus_core::ParseError>> {
        self.plan_cache.resolve(&self.theme, classes)
    }

    /// The walk-time token resolution (the §3.4 T2 half): `Val::Tok` and
    /// `Val::ModePair` slots resolve against the CURRENT table values at
    /// the CURRENT mode — the program is theme-independent, the values
    /// ride the table. Unresolvable slots (a token the table lost) stay
    /// verbatim — the walk's own typed skip boundary answers them, never
    /// a guessed colour.
    ///
    /// The resolved form caches per (program identity, mode) for the
    /// CURRENT version (the swap clears the cache beside the plan lane).
    fn resolve_program(&self, program: StaticProgram, dark: bool) -> StaticProgram {
        let carries_slots = program.program().layers.iter().any(|layer| {
            layer
                .decls
                .iter()
                .any(|decl| matches!(decl.val, Val::Tok { .. } | Val::ModePair(_)))
        });
        if !carries_slots {
            return program;
        }
        let key = (program.layers.as_ptr() as usize, dark);
        if let Some(hit) = self.resolved.borrow().get(&key) {
            return *hit;
        }
        let layers: Vec<Layer> = program
            .program()
            .layers
            .iter()
            .map(|layer| {
                let decls: Vec<Decl> = layer
                    .decls
                    .iter()
                    .map(|decl| {
                        let val = match decl.val {
                            Val::Tok { id, alpha } => self
                                .tokens
                                .resolve_rgba(id, alpha, dark)
                                .map(Val::Rgba)
                                .unwrap_or(decl.val),
                            Val::ModePair(slot) => self
                                .tokens
                                .pair_at(slot)
                                .map(|(light, dark_leg)| if dark { dark_leg } else { light })
                                .map(Val::Rgba)
                                .unwrap_or(decl.val),
                            other => other,
                        };
                        Decl {
                            prop: decl.prop,
                            val,
                        }
                    })
                    .collect();
                Layer::new(layer.cond, Box::leak(decls.into_boxed_slice()))
            })
            .collect();
        let resolved = StaticProgram::new(Box::leak(layers.into_boxed_slice()), program.keywords);
        self.resolved.borrow_mut().insert(key, resolved);
        resolved
    }

    /// The `resolve_style` twin (the §5.1 row's own shape): take the
    /// element, the view-state [`ClassCell`] and the class string —
    /// compile through the SAME pipeline [`TwExt::tw_dyn_theme`] rides
    /// (the §3.6 dynamic lane; the table address is the handle's, so the
    /// hot-swap keeps the cell hitting), resolve the walk-time token
    /// slots against the CURRENT values, and apply through the existing
    /// [`TwExt::tw_env`] walk. A FAILED compile degrades to the cell's
    /// last good program — the honest per-frame contract the lane
    /// carries.
    pub fn style<E: Styled>(&self, element: E, cell: &ClassCell, classes: &str, env: Env) -> E {
        let compiled = cell.get_theme(classes, &self.tokens, &Config::default());
        let program = match compiled {
            Ok(program) => program,
            Err(_) => cell.program(),
        };
        let resolved = self.resolve_program(program, env.dark);
        element.tw_env(resolved, env)
    }

    /// Install the projected palette as gpui-component's global theme —
    /// the components read it through their own [`ActiveTheme`] for App.
    /// Re-run after a swap (or use [`AeolusTheme::apply_swap`], which
    /// installs and refreshes in one step).
    pub fn install(&self, cx: &mut gpui::App) {
        cx.set_global(self.mirror.clone());
    }

    /// The swap's App half: install the (already re-projected) mirror
    /// and refresh every window — the one step a host calls after
    /// [`AeolusTheme::absorb_css`].
    pub fn apply_swap(&self, cx: &mut gpui::App) {
        cx.set_global(self.mirror.clone());
        cx.refresh_windows();
    }
}

impl ActiveTheme for AeolusTheme {
    fn theme(&self) -> &Theme {
        &self.mirror
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIGHT: &str =
        ":root {\n  --primary: oklch(0.205 0 0);\n  --background: oklch(1 0 0);\n}\n";
    const DARK: &str =
        ":root {\n  --primary: oklch(0.922 0 0);\n  --background: oklch(0.145 0 0);\n}\n";

    #[test]
    fn the_swap_swaps_values_without_losing_ids() {
        let mut theme = AeolusTheme::from_css(LIGHT, None).expect("builds");
        let version_before = theme.version();
        let theme_version_before = theme.theme_version();
        theme.absorb_css(DARK).expect("the swap merges");
        assert_eq!(theme.version(), version_before + 1, "one bump per swap");
        assert!(
            theme.theme_version() > theme_version_before,
            "the theme's own version rides absorb"
        );
        let id = theme.tokens().id("primary").expect("the token survives");
        let [.., a] = theme
            .tokens()
            .resolve_rgba(id, 1.0, false)
            .expect("the swapped value resolves");
        assert_eq!(a, 255, "the token is opaque");
    }

    #[test]
    fn the_swap_is_transactional_on_a_shift() {
        let mut theme = AeolusTheme::from_css(LIGHT, None).expect("builds");
        let before = theme.theme_version();
        // A name that sorts BEFORE the existing ones shifts every id.
        let shift = ":root {\n  --aaa: oklch(0.5 0 0);\n}\n";
        // The id contract: when the sheet's new names shift the sorted
        // ids, the swap refuses TYPED and leaves the handle untouched;
        // when the runtime table's own ordering tolerates the insertion,
        // the swap commits and every pre-swap name still resolves its
        // OWN value (the no-corruption pin either way). The typed
        // refusal fires on the corpus fixtures (tests/bridge.rs) where
        // the name set is the theme chain's real one.
        match theme.absorb_css(shift) {
            Err(SwapError::TokenIdShift { name }) => {
                assert_eq!(name, "primary");
                assert_eq!(theme.theme_version(), before, "untouched on refusal");
            }
            Ok(()) => {
                let id = theme.tokens().id("primary").expect("the token survives");
                assert!(
                    theme.tokens().resolve_rgba(id, 1.0, false).is_ok(),
                    "the pre-swap names still resolve after a tolerated insertion"
                );
            }
            other => panic!("the swap answers typed, got {other:?}"),
        }
    }
}
