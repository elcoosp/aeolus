//! CSS-first theme registry.
//!
//! `TailwindTheme::from_theme_css` parses Tailwind v4 `@theme` blocks
//! (including `@theme inline` / `@theme static`, which behave identically for
//! token *resolution* in v1 — documented limitation) plus the light/dark
//! `:root`/`.dark` variable blocks that Shadcn uses.
//!
//! v0.2 (plan §3.3): every theme ships the **full v4 default theme** —
//! `--container-*`, `--shadow-*` (parsed as multi-layer comma-separated
//! lists), `--text-*` + paired line heights, `--font-weight-*`,
//! `--tracking-*`, `--leading-*`, `--radius-*`, `--blur-*`,
//! `--perspective-*`, `--ease-*`, `--animate-*` and the default transition
//! duration — diffed against upstream tailwindcss v4.1.17. `@custom-variant`
//! registrations (e.g. the class-based `dark` strategy) and `@utility`
//! expansions (`@utility name { @apply … }`) are extracted alongside tokens.
//!
//! Runtime mutation is explicit: swap the token map, bump the version (`TailwindTheme::version`),
//! and re-resolve. Aeolus does not pretend to have CSS custom-property
//! propagation.

use crate::ast::Breakpoint;
use crate::color::{Color, ParseError};
use crate::plan::Ease;
use std::collections::HashMap;
use std::fmt::Write as _;
use std::sync::Arc;

/// Tailwind CSS v4 default color palette (`@theme default`, colors only),
/// verbatim from tailwindcss v4.1.17 `packages/tailwindcss/theme.css`.
/// Every theme is seeded with it; user declarations merge on top
/// (same key → later wins), matching v4's `@theme` layering.
pub const TAILWIND_DEFAULT_PALETTE: &str = include_str!("../data/tailwind-default-palette.css");

/// Tailwind CSS v4 default theme — the NON-color tokens (`--container-*`,
/// `--shadow-*`, `--text-*`, `--font-*`, `--ease-*`, `--animate-*`, ...),
/// verbatim from tailwindcss v4.1.17 (plan §3.3). Seeded into every theme
/// under the palette.
pub const TAILWIND_DEFAULT_THEME: &str = include_str!("../data/tailwind-default-theme.css");

/// A typed theme token.
#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    /// A color token (Oklab + alpha).
    Color(Color),
    /// Length in px (rem resolved at 16px, the v4 root assumption).
    Length(f32),
    /// `--text-sm` font size + paired `--text-sm--line-height`.
    FontSize {
        /// Font size in px.
        size: f32,
        /// Paired line height in px, when declared.
        line_height: Option<f32>,
    },
    /// `--shadow-*` presets — v4 values are multi-layer comma-separated
    /// lists; every layer is kept (v0.2, plan §3.3).
    Shadow(Vec<BoxShadowSpec>),
    /// A value that is not a length or color (font stacks, `cubic-bezier()`
    /// easings, `--animate-*` shorthand, `16 / 9` aspect ratios, ...).
    /// Stored verbatim so the default theme parses cleanly and consumers
    /// (`ease`, `animation`) can interpret their own keys.
    Raw(String),
}

/// One shadow layer.
#[derive(Clone, Debug, PartialEq)]
pub struct BoxShadowSpec {
    /// Horizontal offset in px.
    pub offset_x: f32,
    /// Vertical offset in px.
    pub offset_y: f32,
    /// Blur radius in px.
    pub blur: f32,
    /// Spread in px.
    pub spread: f32,
    /// Shadow color.
    pub color: Color,
    /// `inset` keyword (`--inset-shadow-*`, `shadow-inner`).
    pub inset: bool,
}

/// What a `@custom-variant` registration means to the resolver.
#[derive(Clone, Debug, PartialEq)]
pub enum CustomVariant {
    /// `@custom-variant dark (&:where(.dark, .dark *))` — this name means
    /// the `dark` media/class strategy.
    Dark,
    /// Registered but not mappable to a typed condition; the resolver
    /// reports it with the selector text.
    Unsupported(String),
}

/// A parsed `@utility name { @apply … }` expansion (depth-limited).
#[derive(Clone, Debug, PartialEq)]
pub struct CustomUtility {
    /// The utility name (`tab-corpher` for `@utility tab-corpher`).
    pub name: String,
    /// The `@apply` class list (verbatim from the body).
    pub classes: String,
}

/// A parsed, immutable-by-convention theme snapshot.
#[derive(Clone, Debug)]
pub struct TailwindTheme {
    tokens: HashMap<String, Token>,
    breakpoints: Vec<(Breakpoint, f32)>,
    /// `calc(var(--radius) - 2px)` values deferred until their deps exist.
    raw_calcs: Vec<(String, String)>,
    /// `--text-sm--line-height` ratios pending their `--text-sm` parent.
    pending_lh: Vec<(String, f32)>,
    /// Bare `var(--x)` values for COLOR keys (`@theme inline
    /// { --color-primary: var(--primary); }`) whose target token had not
    /// been parsed yet — declaration order is irrelevant to custom-property
    /// resolution, so these re-resolve in a fixpoint pass.
    pending_vars: Vec<(String, String)>,
    /// Raw declaration values (for `var()` opacity modifiers like
    /// `bg-red-500/(--o)`).
    raw_vars: HashMap<String, String>,
    /// `@custom-variant` registrations.
    custom_variants: Vec<(String, CustomVariant)>,
    /// `@utility name { @apply … }` expansions.
    custom_utilities: Vec<CustomUtility>,
    /// Source CSS, re-parsed when deriving the dark variant so
    /// `@theme inline` var chains re-resolve against the dark palette.
    source_css: String,
    /// Monotonic version — bump on swap so caches can key on it.
    version: u64,
    /// True when the snapshot came from the dark palette.
    dark_mode: bool,
}

impl Default for TailwindTheme {
    fn default() -> Self {
        // Seed every theme with the full v4 defaults (palette + theme),
        // parsed once and cloned (plan §3.3). The version base is unique
        // per instance so two distinct themes never share a cache key.
        static TEMPLATE: std::sync::OnceLock<TailwindTheme> = std::sync::OnceLock::new();
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let tpl = TEMPLATE.get_or_init(|| {
            let mut t = TailwindTheme::empty();
            let _ = t.absorb_css(TAILWIND_DEFAULT_PALETTE);
            let _ = t.absorb_css(TAILWIND_DEFAULT_THEME);
            t
        });
        let mut t = tpl.clone();
        t.version = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        t
    }
}

/// Outcome of one pending bare-var color retry.
enum PendingVar {
    /// Deps still missing — keep pending.
    Unresolved,
    /// The value became parseable but is not a valid color.
    Invalid(ParseError),
}

impl TailwindTheme {
    fn empty() -> Self {
        Self {
            tokens: HashMap::new(),
            breakpoints: Vec::new(),
            raw_calcs: Vec::new(),
            pending_lh: Vec::new(),
            pending_vars: Vec::new(),
            raw_vars: HashMap::new(),
            custom_variants: Vec::new(),
            custom_utilities: Vec::new(),
            source_css: String::new(),
            version: 0,
            dark_mode: false,
        }
    }

    /// A theme with no user tokens (seeded with the full v4 defaults).
    pub fn new() -> Self {
        Self::default()
    }

    /// Monotonic theme version, used as a cache key component.
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Was this snapshot derived from the `.dark` palette?
    pub fn dark_mode(&self) -> bool {
        self.dark_mode
    }

    /// Build the `--color-{name}` / `--{name}` lookup keys in a 96-byte
    /// stack buffer — no `format!` on the hot path (plan §3.4.2).
    fn prefixed<'b>(buf: &'b mut [u8; 96], prefix: &str, name: &str) -> Option<&'b str> {
        if prefix.len() + name.len() > buf.len() {
            return None;
        }
        buf[..prefix.len()].copy_from_slice(prefix.as_bytes());
        buf[prefix.len()..prefix.len() + name.len()].copy_from_slice(name.as_bytes());
        std::str::from_utf8(&buf[..prefix.len() + name.len()]).ok()
    }

    /// Look up a color token. `theme.color("primary")` matches
    /// `--color-primary` (v4 registry naming) or `--primary` (shadcn app
    /// naming, mapped via `@theme inline`).
    pub fn color(&self, name: &str) -> Option<Color> {
        let mut buf = [0u8; 96];
        if let Some(k) = Self::prefixed(&mut buf, "--color-", name)
            && let Some(Token::Color(c)) = self.tokens.get(k)
        {
            return Some(*c);
        }
        let mut buf2 = [0u8; 96];
        if let Some(k) = Self::prefixed(&mut buf2, "--", name)
            && let Some(Token::Color(c)) = self.tokens.get(k)
        {
            return Some(*c);
        }
        None
    }

    /// Look up a length token (`--spacing`, `--radius-md`, ...) in px.
    pub fn length(&self, name: &str) -> Option<f32> {
        let mut buf = [0u8; 96];
        let k = Self::prefixed(&mut buf, "--", name)?;
        match self.tokens.get(k) {
            Some(Token::Length(l)) => Some(*l),
            _ => None,
        }
    }

    /// The spacing base (`--spacing`, v4 default `0.25rem` = 4px).
    pub fn spacing(&self) -> f32 {
        self.length("spacing").unwrap_or(4.0)
    }

    /// Look up a `--text-{name}` font-size token (+ paired line height).
    pub fn font_size(&self, name: &str) -> Option<(f32, Option<f32>)> {
        let mut buf = [0u8; 96];
        let k = Self::prefixed(&mut buf, "--text-", name)?;
        match self.tokens.get(k) {
            Some(Token::FontSize { size, line_height }) => Some((*size, *line_height)),
            _ => None,
        }
    }

    /// Look up a `--font-weight-{name}` token (v0.2 default scale).
    pub fn font_weight(&self, name: &str) -> Option<u16> {
        match name {
            "thin" => Some(100),
            "extralight" => Some(200),
            "light" => Some(300),
            "normal" => Some(400),
            "medium" => Some(500),
            "semibold" => Some(600),
            "bold" => Some(700),
            "extrabold" => Some(800),
            "black" => Some(900),
            other => self.raw_number(&format!("--font-weight-{other}")),
        }
    }

    /// The `--default-transition-duration` in ms (v4 default 150).
    pub fn default_transition_ms(&self) -> f32 {
        match self.tokens.get("--default-transition-duration") {
            Some(Token::Raw(v)) => parse_duration(v).unwrap_or(150.0),
            _ => 150.0,
        }
    }

    /// A `--leading-{name}` unitless ratio (`--leading-tight: 1.25`).
    /// Leading tokens are unitless so they live in `Raw`.
    pub fn leading(&self, name: &str) -> Option<f32> {
        let mut buf = [0u8; 96];
        let k = Self::prefixed(&mut buf, "--leading-", name)?;
        match self.tokens.get(k) {
            Some(Token::Length(l)) => Some(*l),
            Some(Token::Raw(s)) => s.trim().parse().ok(),
            _ => None,
        }
    }

    /// The raw declaration value of a var (`--aspect-video` → `"16 / 9"`).
    pub fn raw_var(&self, name: &str) -> Option<&str> {
        self.raw_vars.get(name).map(String::as_str)
    }

    fn raw_number(&self, key: &str) -> Option<u16> {
        match self.tokens.get(key) {
            Some(Token::Raw(s)) => s.trim().parse().ok(),
            _ => None,
        }
    }

    /// The FIRST (dominant) layer of `--shadow-{name}` — v0.1 compat.
    /// Use [`TailwindTheme::shadows`] for the full multi-layer list.
    pub fn shadow(&self, name: &str) -> Option<BoxShadowSpec> {
        self.shadows(name).into_iter().next()
    }

    /// All layers of `--shadow-{name}` (v0.2, plan §3.3). Bare `shadow`
    /// (`name == ""`) maps to the deprecated `--shadow` token, falling back
    /// to `--shadow-sm` (the v4 default box shadow).
    pub fn shadows(&self, name: &str) -> Vec<BoxShadowSpec> {
        if name.is_empty() {
            if let Some(Token::Shadow(v)) = self.tokens.get("--shadow") {
                return v.clone();
            }
            if let Some(Token::Shadow(v)) = self.tokens.get("--shadow-sm") {
                return v.clone();
            }
            return Vec::new();
        }
        let mut buf = [0u8; 96];
        let Some(k) = Self::prefixed(&mut buf, "--shadow-", name) else {
            return Vec::new();
        };
        match self.tokens.get(k) {
            Some(Token::Shadow(v)) => v.clone(),
            _ => Vec::new(),
        }
    }

    /// All layers of `--inset-shadow-{name}` (v0.2).
    pub fn inset_shadows(&self, name: &str) -> Vec<BoxShadowSpec> {
        let mut buf = [0u8; 96];
        let Some(k) = Self::prefixed(&mut buf, "--inset-shadow-", name) else {
            return Vec::new();
        };
        match self.tokens.get(k) {
            Some(Token::Shadow(v)) => v.clone(),
            _ => Vec::new(),
        }
    }

    /// Look up a `--radius-{name}` token in px.
    pub fn radius(&self, name: &str) -> Option<f32> {
        let mut buf = [0u8; 96];
        let k = Self::prefixed(&mut buf, "--radius-", name)?;
        match self.tokens.get(k) {
            Some(Token::Length(l)) => Some(*l),
            _ => None,
        }
    }

    /// A `--container-{name}` token in px (v0.2 `@container` scale).
    pub fn container_px(&self, name: &str) -> Option<f32> {
        let mut buf = [0u8; 96];
        let k = Self::prefixed(&mut buf, "--container-", name)?;
        match self.tokens.get(k) {
            Some(Token::Length(l)) => Some(*l),
            _ => None,
        }
    }

    /// The container-query px for a `@container` breakpoint
    /// (`@sm` = `--container-sm` = 24rem = 384px — NOT the `sm` viewport
    /// breakpoint; plan §3.3 fixes the v0.1 conflation).
    pub fn container_breakpoint_px(&self, b: Breakpoint) -> f32 {
        self.container_px(b.name()).unwrap_or(match b {
            Breakpoint::Sm => 384.0,
            Breakpoint::Md => 448.0,
            Breakpoint::Lg => 512.0,
            Breakpoint::Xl => 576.0,
            Breakpoint::Xxl => 672.0,
        })
    }

    /// The effective breakpoint px for `b` (user override or v4 default).
    pub fn breakpoint_px(&self, b: Breakpoint) -> f32 {
        self.breakpoints
            .iter()
            .find(|(n, _)| *n == b)
            .map_or_else(|| b.default_px(), |(_, px)| *px)
    }

    /// The full theme-aware breakpoint px table (`Breakpoint::index` order).
    pub fn breakpoints_px(&self) -> [f32; 5] {
        [
            self.breakpoint_px(Breakpoint::Sm),
            self.breakpoint_px(Breakpoint::Md),
            self.breakpoint_px(Breakpoint::Lg),
            self.breakpoint_px(Breakpoint::Xl),
            self.breakpoint_px(Breakpoint::Xxl),
        ]
    }

    /// The `--ease-{name}` timing function (`linear`, `in`, `out`,
    /// `in-out`, or a theme-defined `cubic-bezier(...)`; v0.2).
    pub fn ease(&self, name: &str) -> Option<Ease> {
        let mut buf = [0u8; 96];
        if let Some(k) = Self::prefixed(&mut buf, "--ease-", name)
            && let Some(Token::Raw(v)) = self.tokens.get(k)
        {
            return Some(ease_from_value(v));
        }
        match name {
            "linear" => Some(Ease::Linear),
            "in" => Some(Ease::In),
            "out" => Some(Ease::Out),
            "in-out" => Some(Ease::InOut),
            _ => None,
        }
    }

    /// An `--animate-{name}` shorthand (`spin 1s linear infinite`),
    /// parsed into `(name, duration_ms, ease, infinite)`.
    pub fn animation(&self, name: &str) -> Option<AnimationSpec> {
        let mut buf = [0u8; 96];
        let k = Self::prefixed(&mut buf, "--animate-", name)?;
        match self.tokens.get(k) {
            Some(Token::Raw(v)) => parse_animation(v),
            _ => None,
        }
    }

    /// The registered custom variant, when `name` was declared via
    /// `@custom-variant`.
    pub fn custom_variant(&self, name: &str) -> Option<&CustomVariant> {
        self.custom_variants
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, m)| m)
    }

    /// The `@utility` expansion for `name`, when declared.
    pub fn custom_utility(&self, name: &str) -> Option<&CustomUtility> {
        self.custom_utilities.iter().find(|u| u.name == name)
    }

    /// True when the theme declares any `@utility` — the resolver's
    /// expansion pre-scan fast path (zero cost for stock themes).
    pub fn has_custom_utilities(&self) -> bool {
        !self.custom_utilities.is_empty()
    }

    /// Resolve a raw variable value to an opacity PERCENT for
    /// `bg-{color}/(--var)` modifiers (F7): `"50%"` → 50, `"0.5"` → 50,
    /// `"50"` → 50.
    pub fn var_percent(&self, name: &str) -> Option<f32> {
        let key = if name.starts_with("--") {
            name.to_string()
        } else {
            format!("--{name}")
        };
        let raw = self.raw_vars.get(key.as_str())?;
        let v = raw.trim();
        if let Some(p) = v.strip_suffix('%') {
            return p.trim().parse().ok();
        }
        let n: f32 = v.parse().ok()?;
        Some(if (0.0..=1.0).contains(&n) {
            n * 100.0
        } else {
            n
        })
    }

    /// All color token names (for `tw_audit`), without prefixes.
    pub fn color_names(&self) -> Vec<&str> {
        self.tokens
            .keys()
            .filter_map(|k| {
                k.strip_prefix("--color-")
                    // shadcn app naming: --primary etc. are colors too
                    .or_else(|| is_color_only_key(k).then(|| k.trim_start_matches("--")))
            })
            .collect()
    }

    /// Parse `@theme`/`:root`/`.dark` CSS into a fresh theme.
    pub fn from_theme_css(css: &str) -> Result<Self, ParseError> {
        let mut theme = Self::default();
        // v4 semantics: the default palette + theme are the baseline; user
        // `@theme` declarations are merged over them (same key → later wins).
        theme.absorb_css(css)?;
        Ok(theme)
    }

    /// Parse CSS into an existing theme (merge semantics). Bumps `version`.
    pub fn absorb_css(&mut self, css: &str) -> Result<(), ParseError> {
        self.source_css.push('\n');
        self.source_css.push_str(css);
        let (variants, utilities) = extract_custom_rules(css);
        self.custom_variants.extend(variants);
        self.custom_utilities.extend(utilities);
        self.absorb_declarations(&extract_declarations(css))
    }

    fn absorb_declarations(
        &mut self,
        decls: &[(BlockKind, String, String)],
    ) -> Result<(), ParseError> {
        for (_kind, key, value) in decls {
            self.insert_var(key, value)?;
        }
        // Deferred calc()/var() pass, iterated to a FIXPOINT: entries may
        // depend on other deferred entries (`--b: calc(var(--a) + 2px)` with
        // `--a: calc(1px + 1px)`), and CSS declaration order is irrelevant to
        // custom-property resolution (`@theme inline` may precede its
        // `:root` source). Only entries whose deps NEVER appear error.
        let mut pending_calcs = std::mem::take(&mut self.raw_calcs);
        let mut pending_vars = std::mem::take(&mut self.pending_vars);
        while !pending_calcs.is_empty() || !pending_vars.is_empty() {
            let mut progress = false;
            let mut still_calcs = Vec::new();
            for (key, expr) in pending_calcs {
                match eval_calc(&expr, &self.tokens) {
                    Ok((raw, is_px)) => {
                        progress = true;
                        if let Some(parent) = key.strip_suffix("--line-height")
                            && parent.starts_with("--text-")
                        {
                            // `calc(1.25 / 0.875)` is a unitless ratio of font-size.
                            self.pending_lh.push((parent.to_string(), raw));
                            continue;
                        }
                        self.store_length(&key, if is_px { raw } else { raw * 16.0 });
                    }
                    // Deps not parsed yet — retry after the next round.
                    Err(_) => still_calcs.push((key, expr)),
                }
            }
            pending_calcs = still_calcs;
            let mut still_vars = Vec::new();
            for (key, value) in pending_vars {
                match self.try_pending_color_var(&key, &value) {
                    Ok(()) => progress = true,
                    Err(PendingVar::Unresolved) => still_vars.push((key, value)),
                    Err(PendingVar::Invalid(e)) => return Err(e),
                }
            }
            pending_vars = still_vars;
            if !progress {
                break;
            }
        }
        // Truly-unresolvable references keep the old precise-error contract.
        if let Some((key, expr)) = pending_calcs.first() {
            let err = eval_calc(expr, &self.tokens).expect_err("unresolved calc must still fail");
            return Err(ParseError::new(format!("{key}: {}", err.message)));
        }
        if let Some((key, _)) = pending_vars.first() {
            return Err(ParseError::new(format!(
                "{key}: var() reference never resolves to a color token"
            )));
        }
        // Pair any pending line-height ratios whose parents arrived earlier.
        let pending_lh = std::mem::take(&mut self.pending_lh);
        for (parent, ratio) in pending_lh {
            if let Some(Token::FontSize { size, line_height }) = self.tokens.get_mut(&parent) {
                let sz = *size;
                *line_height = Some(sz * ratio);
            } else {
                // Parent arrives later; keep pending for the insert path.
                self.pending_lh.push((parent, ratio));
            }
        }
        self.version += 1;
        Ok(())
    }

    /// Build the dark variant by re-parsing the source CSS with the
    /// `.dark` overrides appended, so `@theme inline` var chains
    /// (`--color-primary: var(--primary)`) re-resolve against the dark
    /// palette — the propagation CSS gives us for free and Aeolus does
    /// explicitly (mission brief §3.5).
    pub fn dark_variant(&self, dark_css: &str) -> Result<Self, ParseError> {
        let mut dark = TailwindTheme {
            dark_mode: true,
            ..TailwindTheme::default()
        };
        // 1. light palette (`:root` etc.)
        let all = extract_declarations(&self.source_css);
        let roots: Vec<_> = all
            .iter()
            .filter(|(k, _, _)| *k != BlockKind::Theme)
            .cloned()
            .collect();
        let themes: Vec<_> = all
            .iter()
            .filter(|(k, _, _)| *k == BlockKind::Theme)
            .cloned()
            .collect();
        dark.absorb_declarations(&roots)?;
        // 2. dark palette overrides
        dark.absorb_declarations(&extract_declarations(dark_css))?;
        // 3. re-resolve `@theme inline` var chains against the final palette
        dark.absorb_declarations(&themes)?;
        // Custom rules from the source CSS carry over.
        let (variants, utilities) = extract_custom_rules(&self.source_css);
        dark.custom_variants.extend(variants);
        dark.custom_utilities.extend(utilities);
        Ok(dark)
    }

    /// Mark the snapshot immutable and share it.
    pub fn seal(mut self) -> Arc<Self> {
        self.version += 1;
        Arc::new(self)
    }

    /// Store a resolved length, patching a paired `--text-*--line-height`.
    fn store_length(&mut self, key: &str, v: f32) {
        if let Some(parent) = key.strip_suffix("--line-height")
            && parent.starts_with("--text-")
            && let Some(Token::FontSize { line_height, .. }) = self.tokens.get_mut(parent)
        {
            *line_height = Some(v);
            return;
        }
        self.tokens.insert(key.to_string(), Token::Length(v));
    }

    /// Substitute a single `var(--x)` when the token is a known length.
    fn try_var_length(&self, value: &str) -> Option<f32> {
        let inner = value.trim().strip_prefix("var(")?.strip_suffix(')')?;
        match self.tokens.get(inner.trim()) {
            Some(Token::Length(l)) => Some(*l),
            _ => None,
        }
    }

    /// Retry one deferred bare-`var()` COLOR value (`--color-x: var(--x)`).
    /// `Ok(())` = stored; `Unresolved` = the target token is still unknown;
    /// `Invalid` = resolvable but not a color (precise error, as before).
    fn try_pending_color_var(&mut self, key: &str, value: &str) -> Result<(), PendingVar> {
        let inner = value
            .trim()
            .strip_prefix("var(")
            .and_then(|r| r.strip_suffix(')'))
            .map_or("", str::trim);
        if !matches!(self.tokens.get(inner), Some(Token::Color(_))) {
            return Err(PendingVar::Unresolved);
        }
        let resolved = resolve_vars(value, &self.tokens).map_err(PendingVar::Invalid)?;
        match Color::parse(&resolved) {
            Ok(c) => {
                self.tokens.insert(key.to_string(), Token::Color(c));
                Ok(())
            }
            Err(e) => Err(PendingVar::Invalid(e)),
        }
    }

    fn insert_var(&mut self, key: &str, value: &str) -> Result<(), ParseError> {
        // Keep the raw declaration for var() opacity modifiers.
        self.raw_vars.insert(key.to_string(), value.to_string());
        // Defer calc() values until all their var() deps are parsed.
        if value.trim_start().starts_with("calc(") {
            self.raw_calcs.push((key.to_string(), value.to_string()));
            return Ok(());
        }
        // A bare `var(--x)` for a length key: substitute now if known,
        // otherwise defer like a calc.
        if value.trim_start().starts_with("var(")
            && !key.starts_with("--color-")
            && !is_color_only_key(key)
        {
            if let Some(v) = self.try_var_length(value) {
                self.store_length(key, v);
                return Ok(());
            }
            self.raw_calcs.push((key.to_string(), value.to_string()));
            return Ok(());
        }
        // A bare `var(--x)` for a COLOR key (`@theme inline` chains):
        // substitute now when the target is a known color, otherwise defer —
        // CSS declaration order must not matter (the `@theme inline` block
        // may precede the `:root` block that defines the variable).
        if value.trim_start().starts_with("var(")
            && (key.starts_with("--color-") || is_color_only_key(key))
        {
            match self.try_pending_color_var(key, value) {
                Ok(()) => return Ok(()),
                Err(PendingVar::Unresolved) => {
                    self.pending_vars.push((key.to_string(), value.to_string()));
                    return Ok(());
                }
                Err(PendingVar::Invalid(e)) => return Err(e),
            }
        }
        // --text-sm--line-height patches its parent --text-sm token.
        if let Some(parent) = key.strip_suffix("--line-height")
            && parent.starts_with("--text-")
        {
            // v4.1 ships unitless ratios for large sizes (`--text-5xl--line-height: 1`)
            // alongside px/calc forms — a bare number is a RATIO of the size.
            if let Ok(ratio) = value.trim().parse::<f32>() {
                self.pending_lh.push((parent.to_string(), ratio));
                return Ok(());
            }
            let lh = parse_length(value)?;
            if let Some(Token::FontSize { line_height, .. }) = self.tokens.get_mut(parent) {
                *line_height = Some(lh);
                return Ok(());
            }
            // Parent not seen yet: store under the pair key, patched later.
            self.tokens.insert(key.to_string(), Token::Length(lh));
            return Ok(());
        }
        let token = match key {
            k if k.starts_with("--text-") => {
                let size = parse_length(value)?;
                let lh = match self.tokens.get(&format!("{k}--line-height")) {
                    Some(Token::Length(l)) => Some(*l),
                    _ => None,
                };
                Token::FontSize {
                    size,
                    line_height: lh,
                }
            }
            k if k.starts_with("--shadow-") => {
                Token::Shadow(parse_shadow_layers(&resolve_vars(value, &self.tokens)?)?)
            }
            k if k.starts_with("--inset-shadow-") => Token::Shadow(
                parse_shadow_layers(&resolve_vars(value, &self.tokens)?)?
                    .into_iter()
                    .map(|mut s| {
                        s.inset = true;
                        s
                    })
                    .collect(),
            ),
            k if k.starts_with("--animate-")
                || k.starts_with("--ease-")
                || k.starts_with("--font-sans")
                || k.starts_with("--font-serif")
                || k.starts_with("--font-mono")
                || k.starts_with("--default-")
                || k.starts_with("--aspect-")
                || k.starts_with("--perspective-") =>
            {
                Token::Raw(value.to_string())
            }
            // Value-based sniffing: any oklch()/rgb()/hsl()/hex/color-mix value is a
            // color token regardless of its name (shadcn uses --chart-1..5,
            // --sidebar-*, arbitrary app tokens). Name prefixes stay as fallbacks
            // for var() indirection (e.g. `--color-x: var(--x)`).
            _ if value_is_color(value, &self.tokens) => {
                Token::Color(Color::parse(&resolve_vars(value, &self.tokens)?)?)
            }
            k if k.starts_with("--color-") => {
                Token::Color(Color::parse(&resolve_vars(value, &self.tokens)?)?)
            }
            k if is_color_only_key(k) => {
                Token::Color(Color::parse(&resolve_vars(value, &self.tokens)?)?)
            }
            k if k.starts_with("--breakpoint-") => {
                let px = parse_length(value)?;
                match Breakpoint::from_name(k.trim_start_matches("--breakpoint-")) {
                    Some(b) => {
                        self.breakpoints.retain(|(n, _)| *n != b);
                        self.breakpoints.push((b, px));
                    }
                    // Custom breakpoint names (`--breakpoint-3xl`) are kept
                    // as lengths so `Custom` variants can resolve them.
                    None => {
                        self.tokens.insert(key.to_string(), Token::Length(px));
                    }
                }
                return Ok(());
            }
            _ => match parse_length(value) {
                Ok(l) => Token::Length(l),
                // Non-length, non-color values (raw numbers like
                // `--font-weight-thin: 100`, prose, ...) are kept verbatim
                // so the default theme always parses (v0.2).
                Err(_) => Token::Raw(value.to_string()),
            },
        };
        self.tokens.insert(key.to_string(), token);
        Ok(())
    }
}

/// The parsed form of an `--animate-*` shorthand.
#[derive(Clone, Debug, PartialEq)]
pub struct AnimationSpec {
    /// The animation name (`spin`).
    pub name: String,
    /// Duration in ms (`1s` → 1000).
    pub duration_ms: f32,
    /// Timing function keyword/cubic-bezier, when present.
    pub ease: Option<Ease>,
    /// `infinite` keyword present.
    pub infinite: bool,
}

fn parse_animation(v: &str) -> Option<AnimationSpec> {
    let mut parts = v.split_ascii_whitespace();
    let name = parts.next()?.trim().to_string();
    let mut spec = AnimationSpec {
        name,
        duration_ms: 1000.0,
        ease: None,
        infinite: true,
    };
    for p in parts {
        if let Some(d) = parse_duration(p) {
            spec.duration_ms = d;
        } else if p == "infinite" {
            spec.infinite = true;
        } else if p == "linear" {
            spec.ease = Some(Ease::Linear);
        } else if p.starts_with("cubic-bezier(") {
            spec.ease = Some(ease_from_value(p));
        }
        // (iteration counts, alternate, ... are carried in the name token by
        // backends that support them; Aeolus models the v4 presets.)
    }
    Some(spec)
}

/// `150ms` → 150, `1s` → 1000.
fn parse_duration(s: &str) -> Option<f32> {
    if let Some(ms) = s.strip_suffix("ms") {
        return ms.trim().parse().ok();
    }
    if let Some(sec) = s.strip_suffix('s') {
        return sec.trim().parse::<f32>().ok().map(|v| v * 1000.0);
    }
    None
}

/// Map a `cubic-bezier(...)` value (or `linear`) to the typed [`Ease`].
fn ease_from_value(v: &str) -> Ease {
    let v = v.trim();
    if v == "linear" {
        return Ease::Linear;
    }
    if let Some(inner) = v
        .strip_prefix("cubic-bezier(")
        .and_then(|r| r.strip_suffix(')'))
    {
        let nums: Vec<f32> = inner
            .split(',')
            .filter_map(|n| n.trim().parse().ok())
            .collect();
        if nums.len() == 4 {
            // Match against the v4 default easings.
            let close = |a: f32, b: f32| (a - b).abs() < 1e-4;
            if close(nums[0], 0.4)
                && close(nums[1], 0.0)
                && close(nums[2], 1.0)
                && close(nums[3], 1.0)
            {
                return Ease::In;
            }
            if close(nums[0], 0.0)
                && close(nums[1], 0.0)
                && close(nums[2], 0.2)
                && close(nums[3], 1.0)
            {
                return Ease::Out;
            }
            if close(nums[0], 0.4)
                && close(nums[1], 0.0)
                && close(nums[2], 0.2)
                && close(nums[3], 1.0)
            {
                return Ease::InOut;
            }
        }
    }
    Ease::InOut
}

fn value_is_color(value: &str, tokens: &HashMap<String, Token>) -> bool {
    let v = value.trim();
    if v.starts_with("var(") {
        return false;
    }
    let resolved = resolve_vars(v, tokens).unwrap_or_else(|_| v.to_string());
    Color::parse(&resolved).is_ok()
}

fn is_color_only_key(k: &str) -> bool {
    // shadcn app themes use `--primary`, `--border`, ... (no `--color-` prefix).
    matches!(
        k,
        "--background"
            | "--foreground"
            | "--card"
            | "--card-foreground"
            | "--popover"
            | "--popover-foreground"
            | "--primary"
            | "--primary-foreground"
            | "--secondary"
            | "--secondary-foreground"
            | "--muted"
            | "--muted-foreground"
            | "--accent"
            | "--accent-foreground"
            | "--destructive"
            | "--destructive-foreground"
            | "--border"
            | "--input"
            | "--ring"
            | "--sidebar"
            | "--sidebar-foreground"
            | "--sidebar-primary"
            | "--sidebar-primary-foreground"
            | "--sidebar-accent"
            | "--sidebar-accent-foreground"
            | "--sidebar-border"
            | "--sidebar-ring"
            | "--surface"
            | "--surface-foreground"
            | "--selection"
            | "--selection-foreground"
    )
}

/// Substitute already-parsed `var(--x)` references with their CSS text so a
/// color defined as `var(--color-blue-300)` can still be parsed.
fn resolve_vars(value: &str, tokens: &HashMap<String, Token>) -> Result<String, ParseError> {
    if !value.contains("var(") {
        return Ok(value.to_string());
    }
    let mut out = String::with_capacity(value.len());
    let mut rest = value;
    while let Some(i) = rest.find("var(") {
        out.push_str(&rest[..i]);
        let after = &rest[i + 4..];
        let Some(end) = after.find(')') else {
            return Err(ParseError::new("unbalanced var() in theme value"));
        };
        let name = after[..end].trim();
        let substituted = match tokens.get(name) {
            Some(Token::Color(c)) => c.to_oklch_string(),
            Some(Token::Length(l)) => format!("{l}px"),
            _ => {
                // Unknown: keep var text; parse will fail with a clear error.
                format!("var({name})")
            }
        };
        out.push_str(&substituted);
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    Ok(out)
}

/// Parse a length: `0.25rem`, `4px`. Bare numbers are rejected (v4 theme
/// lengths always carry units).
pub fn parse_length(s: &str) -> Result<f32, ParseError> {
    let s = s.trim();
    if let Some(rem) = s.strip_suffix("rem") {
        return rem
            .trim()
            .parse::<f32>()
            .map(|v| v * 16.0)
            .map_err(|_| bad_length(s));
    }
    if let Some(px) = s.strip_suffix("px") {
        return px.trim().parse::<f32>().map_err(|_| bad_length(s));
    }
    Err(bad_length(s))
}

fn bad_length(s: &str) -> ParseError {
    ParseError::new(format!("theme length requires px/rem unit: {s:?}"))
}

/// Parse a multi-layer v4 shadow value (`0 1px 3px 0 rgb(0 0 0 / 0.1), 0 1px
/// 2px -1px rgb(0 0 0 / 0.1)`) into every layer — v0.2 stops truncating at
/// the first comma (plan §1.3/§3.3).
fn parse_shadow_layers(value: &str) -> Result<Vec<BoxShadowSpec>, ParseError> {
    let mut layers = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    for (i, ch) in value.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                layers.push(parse_shadow_layer(&value[start..i])?);
                start = i + 1;
            }
            _ => {}
        }
    }
    layers.push(parse_shadow_layer(&value[start..])?);
    Ok(layers)
}

fn parse_shadow_layer(value: &str) -> Result<BoxShadowSpec, ParseError> {
    let mut value = value.trim();
    let mut inset = false;
    if let Some(rest) = value.strip_prefix("inset") {
        inset = true;
        value = rest.trim();
    }
    // v4 shape: `0px 1px 2px 0px rgb(0 0 0 / 5%)` / oklch(...) — the color
    // may itself contain spaces, so scan from the last numeric token.
    let mut nums: Vec<f32> = Vec::with_capacity(4);
    let mut color_start = value.len();
    for part in value.split_whitespace() {
        if nums.len() >= 4 || !(part.ends_with("px") || part.parse::<f32>().is_ok()) {
            color_start = value.find(part).unwrap_or(value.len());
            break;
        }
        if let Ok(v) = part.trim_end_matches("px").parse::<f32>() {
            nums.push(v);
        }
    }
    let color_src = value[color_start..].trim();
    let color = if color_src.is_empty() {
        Color::TRANSPARENT
    } else {
        Color::parse(color_src)?
    };
    Ok(BoxShadowSpec {
        offset_x: *nums.first().unwrap_or(&0.0),
        offset_y: *nums.get(1).unwrap_or(&0.0),
        blur: *nums.get(2).unwrap_or(&0.0),
        spread: *nums.get(3).unwrap_or(&0.0),
        color,
        inset,
    })
}

/// Standard operator precedence for the shunting-yard calc evaluator.
fn prec(op: char) -> u8 {
    match op {
        '*' | '/' => 2,
        _ => 1,
    }
}

/// Evaluate a `calc()` expression with `var()` refs, `+ - * /`, parentheses
/// and **standard operator precedence** (shunting-yard, ~50 LOC, zero deps).
///
/// The Shadcn v4 strings only need `var(--radius) - 2px` shapes, but a
/// left-to-right evaluator silently mis-computes `4px + var(--r) * 2` —
/// and precedence bugs are silent visual bugs, so the evaluator implements
/// the real CSS calc() grammar subset: unary minus, `+ - * /`, nested
/// parentheses, substituted `var()`s.
///
/// Unit rule: if the expression (after substitution) contains any px-suffixed
/// token or any substituted var, the result is px; otherwise the expression
/// was dimensionless (e.g. `calc(1.25 / 0.875)`) and the result is rem (×16),
/// matching v4's `--text-sm--line-height` idiom.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Tok {
    Num(f32),
    Op(char),
    LParen,
    RParen,
}

fn eval_calc(expr: &str, tokens: &HashMap<String, Token>) -> Result<(f32, bool), ParseError> {
    let inner = expr
        .trim()
        .strip_prefix("calc(")
        .and_then(|s| s.strip_suffix(')'))
        .ok_or_else(|| ParseError::new(format!("not a calc(): {expr:?}")))?;

    let mut resolved = String::new();
    let mut rest = inner;
    let mut had_var = false;
    while let Some(i) = rest.find("var(") {
        resolved.push_str(&rest[..i]);
        let after = &rest[i + 4..];
        let end = after
            .find(')')
            .ok_or_else(|| ParseError::new("unbalanced var()"))?;
        let name = after[..end].trim();
        let v = match tokens.get(name) {
            Some(Token::Length(l)) => *l,
            _ => {
                return Err(ParseError::new(format!(
                    "calc() references unknown token {name}"
                )));
            }
        };
        had_var = true;
        let _ = write!(resolved, "{v} ");
        rest = &after[end + 1..];
    }
    resolved.push_str(rest);

    let is_px = had_var || resolved.contains("px");

    // --- tokenize: numbers ("4px" -> 4.0), operators, parens ----------------
    // Unary +/- in operator position (start, after an operator or '(') is
    // folded into the following number, or, when it guards a parenthesized
    // group, rewritten as the binary `0 <op> (group)`.
    let mut toks: Vec<Tok> = Vec::new();
    let mut sign: f32 = 1.0; // pending unary sign for the next number
    let mut wants_operand = true; // true = a unary +/- may appear here
    let mut it = resolved.split_whitespace().peekable();
    while let Some(raw) = it.next() {
        let mut piece = raw;
        // Leading parens may be glued to the rest of the token: `(4px`.
        while let Some(rest) = piece.strip_prefix('(') {
            toks.push(Tok::LParen);
            piece = rest;
        }
        if piece.is_empty() {
            continue;
        }
        // Split a trailing `)` off: `2)`, `4px)`.
        let mut value_part = piece;
        let mut trailing_parens: usize = 0;
        while let Some(rest) = value_part.strip_suffix(')') {
            trailing_parens += 1;
            value_part = rest;
        }
        if !value_part.is_empty() {
            if value_part.len() == 1 && "+-*/".contains(value_part) {
                // A single-byte operator token: the byte IS the char, so
                // `chars().next()` is Some by construction; spelled as a
                // let-else returning a parse error so this reachable
                // public path (untrusted theme CSS) carries no panic
                // (CONTRIBUTING §2).
                let Some(op) = value_part.chars().next() else {
                    return Err(ParseError::new(format!("bad calc token {raw:?}")));
                };
                if wants_operand {
                    if op == '-' || op == '+' {
                        // Unary sign: fold into the next number, or convert
                        // a guarded paren group into `0 <op> (group)`.
                        let next_paren = it.peek().is_some_and(|n| n.starts_with('('));
                        if next_paren {
                            toks.push(Tok::Num(0.0));
                            toks.push(Tok::Op(op));
                            // wants_operand stays true: still waiting for the
                            // operand, which is the paren group.
                        } else if op == '-' {
                            sign = -sign;
                        }
                    } else {
                        return Err(ParseError::new(format!(
                            "operator {op:?} needs a left operand"
                        )));
                    }
                } else {
                    toks.push(Tok::Op(op));
                    wants_operand = true;
                }
            } else {
                let n: f32 = if let Some(stripped) = value_part.strip_suffix("px") {
                    stripped.parse()
                } else {
                    value_part.parse()
                }
                .map_err(|_| ParseError::new(format!("bad calc token {raw:?}")))?;
                toks.push(Tok::Num(n * sign));
                sign = 1.0;
                wants_operand = false;
            }
        }
        for _ in 0..trailing_parens {
            toks.push(Tok::RParen);
            wants_operand = false;
        }
    }
    if wants_operand && !toks.is_empty() {
        return Err(ParseError::new("calc() ends with an operator"));
    }

    // --- shunting-yard -> RPN (standard precedence: * / over + -) ----------
    let mut rpn: Vec<Tok> = Vec::with_capacity(toks.len());
    let mut ops: Vec<char> = Vec::new();
    for t in toks {
        match t {
            Tok::Num(v) => rpn.push(Tok::Num(v)),
            Tok::Op(op) => {
                // Shunting-yard: pop while the top binds at least as
                // tightly. The pop rides the same guard that scoped the
                // loop, so the operator stack carries no panic path
                // (CONTRIBUTING §2: no unwraps on reachable paths).
                while let Some(&top) = ops.last() {
                    if top == '(' || prec(top) < prec(op) {
                        break;
                    }
                    ops.pop();
                    rpn.push(Tok::Op(top));
                }
                ops.push(op);
            }
            Tok::LParen => ops.push('('),
            Tok::RParen => loop {
                match ops.pop() {
                    Some('(') | None => break,
                    Some(o) => rpn.push(Tok::Op(o)),
                }
            },
        }
    }
    while let Some(o) = ops.pop() {
        if o == '(' {
            return Err(ParseError::new("unbalanced '(' in calc()"));
        }
        rpn.push(Tok::Op(o));
    }

    // --- evaluate the RPN ----------------------------------------------------
    let mut stack: Vec<f32> = Vec::new();
    for t in rpn {
        match t {
            Tok::Num(v) => stack.push(v),
            Tok::Op(op) => {
                let (Some(b), Some(a)) = (stack.pop(), stack.pop()) else {
                    return Err(ParseError::new("calc() operand missing"));
                };
                stack.push(match op {
                    '+' => a + b,
                    '-' => a - b,
                    '*' => a * b,
                    '/' => a / b,
                    _ => return Err(ParseError::new("unknown calc() operator")),
                });
            }
            Tok::LParen | Tok::RParen => {
                return Err(ParseError::new("unbalanced parens in calc()"));
            }
        }
    }
    if stack.len() != 1 {
        return Err(ParseError::new("calc() left surplus operands"));
    }
    let v = stack[0];
    Ok((v, is_px))
}

/// Which block a declaration came from (drives dark-variant re-resolution).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockKind {
    /// An `@theme { ... }` block (v4 theme layer).
    Theme,
    /// A `:root` / `.dark` block (app-variable layer).
    Palette,
}

/// Extract `--key: value;` declarations from `@theme`, `:root`, `.dark`
/// blocks. Other blocks (component rules) are ignored.
pub fn extract_declarations(css: &str) -> Vec<(BlockKind, String, String)> {
    let mut out = Vec::new();
    let owned = strip_comments(css);
    let mut rest: &str = &owned;
    while let Some(open) = rest.find('{') {
        let selector = rest[..open].trim();
        let close = matching_brace(rest.as_bytes(), open);
        if close >= rest.len() {
            // Unbalanced block (no closing brace): the malformed tail is
            // ignored, never sliced out of bounds (fuzz finding).
            return out;
        }
        let body = &rest[open + 1..close];
        let kind = if selector.contains("@theme") {
            BlockKind::Theme
        } else if selector == ":root" || selector.contains(".dark") {
            BlockKind::Palette
        } else {
            rest = &rest[close + 1..];
            continue;
        };
        for decl in split_decls(body) {
            if let Some((k, v)) = decl.split_once(':') {
                let k = k.trim();
                if k.starts_with("--") && !k.starts_with("--tw-") {
                    out.push((
                        kind,
                        k.to_string(),
                        v.trim().trim_end_matches(';').to_string(),
                    ));
                }
            }
        }
        rest = &rest[close + 1..];
    }
    out
}

/// Extract `@custom-variant name (selector);` registrations and
/// `@utility name { … }` blocks (with `@apply …;` bodies) from CSS.
pub fn extract_custom_rules(css: &str) -> (Vec<(String, CustomVariant)>, Vec<CustomUtility>) {
    let owned = strip_comments(css);
    let mut variants = Vec::new();
    let mut utilities = Vec::new();
    let mut rest: &str = &owned;
    while let Some(i) = rest.find("@custom-variant") {
        let after = &rest[i + "@custom-variant".len()..];
        // Form A: `@custom-variant name (selector);`
        // Form B: `@custom-variant name { … }` (block form — registered as
        // unsupported with its body text).
        let semi = after.find(';');
        let brace = after.find('{');
        match (brace, semi) {
            (Some(b), Some(s)) if b < s => {
                // Block form.
                let header = after[..b].trim();
                let close = matching_brace(after.as_bytes(), b);
                let body = after.get(b + 1..close).unwrap_or("");
                let name = header.split_whitespace().next().unwrap_or("").to_string();
                if !name.is_empty() {
                    variants.push((name, CustomVariant::Unsupported(body.trim().to_string())));
                }
                if close >= after.len() {
                    // Unbalanced block: the malformed tail is ignored, never
                    // sliced out of bounds (same contract as
                    // `extract_declarations`; fuzz-reachable panic).
                    break;
                }
                rest = &after[close + 1..];
            }
            (Some(b), None) => {
                let header = after[..b].trim();
                let close = matching_brace(after.as_bytes(), b);
                let body = after.get(b + 1..close).unwrap_or("");
                let name = header.split_whitespace().next().unwrap_or("").to_string();
                if !name.is_empty() {
                    variants.push((name, CustomVariant::Unsupported(body.trim().to_string())));
                }
                if close >= after.len() {
                    break;
                }
                rest = &after[close + 1..];
            }
            (_, Some(s)) => {
                let decl = after[..s].trim();
                let (name, selector) = match decl.split_once(char::is_whitespace) {
                    Some((n, sel)) => (n.trim(), sel.trim()),
                    None => (decl, ""),
                };
                if !name.is_empty() {
                    let meaning = if selector.contains(".dark") {
                        CustomVariant::Dark
                    } else {
                        CustomVariant::Unsupported(selector.to_string())
                    };
                    variants.push((name.to_string(), meaning));
                }
                rest = &after[s + 1..];
            }
            (None, None) => break,
        }
    }
    rest = &owned;
    while let Some(i) = rest.find("@utility") {
        let after = &rest[i + "@utility".len()..];
        let Some(b) = after.find('{') else { break };
        let header = after[..b].trim();
        let close = matching_brace(after.as_bytes(), b);
        let body = after.get(b + 1..close).unwrap_or("");
        let name = header.split_whitespace().next().unwrap_or("").to_string();
        if !name.is_empty() {
            // Expansion source: the `@apply …;` line (v4 custom utilities
            // may also carry raw CSS — those properties are outside the
            // class-string model and stay reported by the resolver).
            let classes = body
                .split(';')
                .find_map(|stmt| {
                    let stmt = stmt.trim();
                    stmt.strip_prefix("@apply").map(|c| c.trim().to_string())
                })
                .unwrap_or_default();
            utilities.push(CustomUtility { name, classes });
        }
        if close >= after.len() {
            // Unbalanced `@utility` block: ignore the malformed tail
            // (fuzz-reachable panic — `rest[close + 1..]` was out of bounds).
            break;
        }
        rest = &after[close + 1..];
    }
    (variants, utilities)
}

/// Remove `/* ... */` comments so selector classification cannot be fooled
/// by documentation text inside them.
fn strip_comments(css: &str) -> String {
    if !css.contains("/*") {
        return css.to_string();
    }
    let mut out = String::with_capacity(css.len());
    let mut rest = css;
    while let Some(i) = rest.find("/*") {
        out.push_str(&rest[..i]);
        let Some(end) = rest[i + 2..].find("*/") else {
            return out;
        };
        rest = &rest[i + 2 + end + 2..];
    }
    out.push_str(rest);
    out
}

fn matching_brace(bytes: &[u8], open: usize) -> usize {
    let mut depth = 0usize;
    for (i, &b) in bytes.iter().enumerate().skip(open) {
        match b {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return i;
                }
            }
            _ => {}
        }
    }
    bytes.len()
}

fn split_decls(body: &str) -> impl Iterator<Item = &str> {
    let mut items = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    for (i, b) in body.bytes().enumerate() {
        match b {
            b'(' => depth += 1,
            b')' => depth = depth.saturating_sub(1),
            b';' if depth == 0 => {
                items.push(&body[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    items.push(&body[start..]);
    items.into_iter()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The calc() operator-edge errors are typed parse failures, never
    /// panics: the operator/operand invariants ride the parse path, and
    /// the shunting-yard pops its operator stack under the same guard
    /// that scoped the loop (the no-panic contract, CONTRIBUTING §2).
    #[test]
    fn calc_operator_edges_error_not_panic() {
        for expr in [
            "calc(2px + )",      // ends with an operator
            "calc(* 4px)",       // operator needs a left operand
            "calc(2px + * 3px)", // second operator in operand position
        ] {
            let css = format!(":root {{ --x: {expr}; }}");
            let err = TailwindTheme::from_theme_css(&css)
                .expect_err("a malformed calc() must be a typed parse error");
            assert!(
                !err.message.is_empty(),
                "the error names what failed: {expr}"
            );
        }
        // The sane spellings still resolve end to end.
        let t = TailwindTheme::from_theme_css(":root { --x: calc(2px + 3px) }").unwrap();
        assert_eq!(t.length("x"), Some(5.0));
        let t = TailwindTheme::from_theme_css(":root { --x: calc((2px + 3px) * 2) }").unwrap();
        assert_eq!(t.length("x"), Some(10.0));
    }

    /// Fuzz finding: a CSS string ending in an UNBALANCED block must not
    /// panic (matching_brace returns the string length; `rest[close + 1..]`
    /// was out of bounds).
    #[test]
    fn unbalanced_blocks_do_not_panic() {
        assert!(TailwindTheme::from_theme_css(".dark { --x: oklch(0 0 0)").is_ok());
        assert!(TailwindTheme::from_theme_css("{a").is_ok());
        assert!(TailwindTheme::from_theme_css(":root { --primary: oklch(0.2 0 0) } {").is_ok());
        // Same contract for the custom-rule scanner (panicked before the
        // guard: `@utility foo {` with no closing brace).
        assert!(TailwindTheme::from_theme_css("@utility foo { @apply px-2;").is_ok());
        assert!(TailwindTheme::from_theme_css("@custom-variant sidebar { &::before").is_ok());
        // Well-formed input still parses its declarations.
        let t = TailwindTheme::from_theme_css(":root { --radius: 10px }").unwrap();
        assert_eq!(t.length("radius"), Some(10.0));
    }

    /// CSS custom properties resolve at USE time, so declaration ORDER must
    /// not matter: an `@theme inline` var chain may precede its `:root`
    /// source (regression: it used to fail with `unsupported color function
    /// "var"`), and a calc() may reference a var whose own value is a later
    /// calc() (regression: `calc() references unknown token`).
    #[test]
    fn deferred_vars_and_calcs_are_order_independent() {
        let t = TailwindTheme::from_theme_css(
            "@theme inline { --color-primary: var(--primary); }\n:root { --primary: oklch(0.205 0 0); }",
        )
        .unwrap();
        let p = t.color("primary").expect("out-of-order var chain");
        assert!((p.l - 0.205).abs() < 1e-4);

        let t = TailwindTheme::from_theme_css(
            "@theme { --radius-md: calc(var(--radius) - 2px); }\n:root { --radius: 10px; }",
        )
        .unwrap();
        assert_eq!(t.radius("md"), Some(8.0));

        // calc() → calc() dependency chains (multi-level deferral).
        let t = TailwindTheme::from_theme_css(
            "@theme { --b: calc(var(--a) + 2px); --a: calc(1px + 1px); }",
        )
        .unwrap();
        assert_eq!(t.length("b"), Some(4.0));

        // A truly-unresolvable reference keeps the precise-error contract.
        assert!(
            TailwindTheme::from_theme_css("@theme inline { --color-x: var(--never-defined); }")
                .is_err()
        );
    }

    /// v4 `calc()` uses standard operator precedence. The Shadcn strings only
    /// need `var +- px`, but a left-to-right evaluator silently mis-computes
    /// `4px + var(--r) * 2` (28 instead of 24) — a silent visual bug class.
    #[test]
    fn calc_has_operator_precedence() {
        let t = TailwindTheme::from_theme_css(
            ":root { --radius: 10px; --radius-md: calc(4px + var(--radius) * 2); --radius-lg: calc((4px + var(--radius)) * 2); --radius-sm: calc(var(--radius) / 2 - 1px); --radius-xl: calc(2 * var(--radius) - 4px); }",
        )
        .unwrap();
        assert_eq!(t.radius("md"), Some(24.0), "4 + 10*2 = 24, not (4+10)*2");
        assert_eq!(t.radius("lg"), Some(28.0), "(4 + 10) * 2 = 28");
        assert_eq!(t.radius("sm"), Some(4.0), "10 / 2 - 1 = 4");
        assert_eq!(t.radius("xl"), Some(16.0), "2 * 10 - 4 = 16");
    }

    const SHADCN_LIGHT: &str = r"
    :root {
      --radius: 0.625rem;
      --background: oklch(1 0 0);
      --foreground: oklch(0.145 0 0);
      --primary: oklch(0.205 0 0);
      --primary-foreground: oklch(0.985 0 0);
      --border: oklch(0.922 0 0);
      --ring: oklch(0.708 0 0);
      --destructive: oklch(0.577 0.245 27.325);
    }
    @theme inline {
      --radius-sm: calc(var(--radius) - 4px);
      --radius-md: calc(var(--radius) - 2px);
      --radius-lg: var(--radius);
      --color-primary: var(--primary);
      --color-background: var(--background);
    }
    ";

    #[test]
    fn parses_shadcn_tokens() {
        let t = TailwindTheme::from_theme_css(SHADCN_LIGHT).unwrap();
        assert_eq!(t.radius("sm"), Some(6.0));
        assert_eq!(t.radius("md"), Some(8.0));
        assert_eq!(t.radius("lg"), Some(10.0));
        let p = t.color("primary").unwrap();
        assert!((p.l - 0.205).abs() < 1e-4, "primary L = {}", p.l);
        assert!(t.color("background").is_some());
        assert!(t.color_names().contains(&"primary"));
    }

    #[test]
    fn dark_variant_overrides() {
        let light = TailwindTheme::from_theme_css(SHADCN_LIGHT).unwrap();
        let dark = light
            .dark_variant(".dark { --background: oklch(0.145 0 0); --border: oklch(1 0 0 / 10%); }")
            .unwrap();
        assert!(dark.dark_mode());
        let bg = dark.color("background").unwrap();
        assert!((bg.l - 0.145).abs() < 1e-4);
        let border = dark.color("border").unwrap();
        assert!(border.alpha < 0.11 && border.alpha > 0.09);
        // light unchanged
        assert!((light.color("background").unwrap().l - 1.0).abs() < 1e-4);
    }

    #[test]
    fn text_size_pairs_line_height() {
        let t = TailwindTheme::from_theme_css(
            "@theme { --text-sm: 0.875rem; --text-sm--line-height: calc(1.25 / 0.875); }",
        )
        .unwrap();
        let (size, lh) = t.font_size("sm").unwrap();
        assert_eq!(size, 14.0);
        let lh = lh.expect("line height resolved");
        assert!((lh - 20.0).abs() < 0.1, "lh={lh}");
    }

    #[test]
    fn breakpoint_tokens() {
        let t = TailwindTheme::from_theme_css("@theme { --breakpoint-3xl: 120rem; }").unwrap();
        assert_eq!(t.breakpoint_px(Breakpoint::Sm), 640.0);
    }

    #[test]
    fn spacing_default() {
        let t = TailwindTheme::new();
        assert_eq!(t.spacing(), 4.0);
        let t = TailwindTheme::from_theme_css("@theme { --spacing: 0.25rem; }").unwrap();
        assert_eq!(t.spacing(), 4.0);
    }

    /// v0.2: every theme ships the full v4 default theme (plan §3.3).
    #[test]
    fn full_v4_defaults_are_seeded() {
        let t = TailwindTheme::new();
        // Container scale.
        assert_eq!(t.container_px("sm"), Some(384.0));
        assert_eq!(t.container_px("7xl"), Some(1280.0));
        assert_eq!(t.container_breakpoint_px(Breakpoint::Sm), 384.0);
        // Multi-layer shadows (v0.1 truncated to the first layer).
        let sm = t.shadows("sm");
        assert_eq!(sm.len(), 2, "shadow-sm has two layers");
        assert_eq!(sm[0].offset_y, 1.0);
        assert_eq!(sm[1].spread, -1.0);
        let md = t.shadows("md");
        assert_eq!(md.len(), 2);
        assert!((md[0].color.alpha - 0.1).abs() < 1e-3);
        // Easings.
        assert_eq!(t.ease("in-out"), Some(Ease::InOut));
        assert_eq!(t.ease("out"), Some(Ease::Out));
        // Animations.
        let spin = t.animation("spin").unwrap();
        assert_eq!(spin.name, "spin");
        assert_eq!(spin.duration_ms, 1000.0);
        assert!(spin.infinite);
        let pulse = t.animation("pulse").unwrap();
        assert_eq!(pulse.duration_ms, 2000.0);
        // Text sizes carry paired line heights (v4.1 ratios).
        let (size, lh) = t.font_size("sm").unwrap();
        assert_eq!(size, 14.0);
        assert!((lh.unwrap() - 20.0).abs() < 0.1, "text-sm lh");
        // Radius scale.
        assert_eq!(t.radius("xs"), Some(2.0));
        assert_eq!(t.radius("4xl"), Some(32.0));
        // Font weights.
        assert_eq!(t.font_weight("semibold"), Some(600));
        // Default transition duration.
        assert_eq!(t.length("default-transition-duration"), None);
        assert!(matches!(
            t.tokens.get("--default-transition-duration"),
            Some(Token::Raw(_))
        ));
    }

    /// @custom-variant + @utility extraction (plan §3.3).
    #[test]
    fn custom_rules_extract() {
        let t = TailwindTheme::from_theme_css(
            "@custom-variant dark (&:where(.dark, .dark *));\n@utility tab-corpher { @apply tabular-nums px-2; }",
        )
        .unwrap();
        assert_eq!(t.custom_variant("dark"), Some(&CustomVariant::Dark));
        let u = t.custom_utility("tab-corpher").unwrap();
        assert_eq!(u.classes, "tabular-nums px-2");
    }

    /// var() opacity modifiers resolve through the raw var table (F7).
    #[test]
    fn var_opacity_resolves() {
        let t = TailwindTheme::from_theme_css(":root { --o: 37%; --half: 0.5; }").unwrap();
        assert_eq!(t.var_percent("--o"), Some(37.0));
        assert_eq!(t.var_percent("half"), Some(50.0));
        assert_eq!(t.var_percent("--missing"), None);
    }

    /// Multi-layer shadow parsing keeps every layer and the inset keyword.
    #[test]
    fn multi_layer_shadows_parse() {
        let t = TailwindTheme::from_theme_css(
            "@theme { --shadow-x: 0 1px 2px 0 rgb(0 0 0 / 0.05), 0 2px 4px -1px oklch(0 0 0 / 0.2); --inset-shadow-y: inset 0 1px 1px rgb(0 0 0 / 0.05); }",
        )
        .unwrap();
        let v = t.shadows("x");
        assert_eq!(v.len(), 2);
        assert_eq!(v[1].offset_y, 2.0);
        assert!((v[1].color.alpha - 0.2).abs() < 1e-3);
        let i = t.inset_shadows("y");
        assert_eq!(i.len(), 1);
        assert!(i[0].inset);
    }
}
