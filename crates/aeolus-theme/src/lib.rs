//! The Aeolus V1 theme pass (architecture plan §3.4, phase P2's second
//! slice): CSS-first theme sources in, tiered [`TokenTable`]s out — the
//! table the compile seam's colour path lowers through.
//!
//! ```text
//! ThemeSource::from_css(light, dark) -> Result<ThemeSource, ThemeError>
//! TokenTable::baked(&ThemeSource)    -> Result<TokenTable, ThemeError>
//! TokenTable::runtime(&ThemeSource)  -> Result<TokenTable, ThemeError>
//! table.resolve(id, alpha)           -> Result<Val, ResolveError>
//! ```
//!
//! This slice resolves the compile seam's discovered IR budget (P2
//! slice 1, `reports/10-tdd-evidence/13-compile-seam.md`): the colour
//! keyword table is **270 entries** and the IR's `Val::Kw` slot is a u8 —
//! colour tokens lower here, through the IR's u16 `Val::Tok` slot, never
//! through truncation. Three tiers, exactly the plan's §3.4 table:
//!
//! | Tier                   | Value form                          | Runtime cost                     |
//! | ---------------------- | ----------------------------------- | -------------------------------- |
//! | T0 static (`Baked`, no dark value) | [`Val::Rgba`] literal   | 0                                |
//! | T1 light/dark (`Baked`, dark value) | [`Val::ModePair`]       | 1 index by the mode              |
//! | T2 runtime (`Runtime`) | [`Val::Tok`] `{ id, alpha }`        | 1 `TokenTable` load + 1 multiply |
//!
//! The `/opacity` modifier is the plan §2 identity, pinned against the
//! audited oracle: mixing with `transparent` in a premultiplied space
//! scales alpha (and oklab chroma) — one multiply, which
//! [`Color::mix_with_transparent`] already implements and the golden
//! corpus pins. [`TokenTable::resolve`] applies it for the `Baked` tier
//! (a literal per alpha) and carries it for the `Runtime` tier (the host
//! multiplies at theme-resolution time); an alpha on a T1 pair slot is a
//! typed [`ResolveError::AlphaOnPair`] — the pair form has no alpha arm,
//! and baking per-alpha pair tables is the codegen's business.
//!
//! The v0.x parser is the interim oracle, on purpose: [`ThemeSource`]
//! wraps `TailwindTheme`'s `@theme`/`:root`/`.dark` reader (fuzz-pinned
//! by `color_parse`/`theme_from_css`) and `Color`'s oklab machinery.
//! The xtask build-time bake (plan §3.4's tracked-path story) replaces
//! the runtime parse in a later slice — the TIER model this crate pins
//! is what stays. Every theme is default-seeded with the full v4
//! palette + theme (`TailwindTheme::default()`), so a built table always
//! carries the 270-entry colour surface plus the source's own tokens;
//! token ids are assigned in SORTED-name order so a rebuilt table gives
//! the same ids (the ids ride compiled `Program`s — stability is a
//! contract, test-pinned).
//!
//! Instrumentation is the workspace's opt-in pattern (CLI v0.9.1, the
//! registry P1, the compile seam P2): `tracing` (spans on the build and
//! the resolve) and `miette` (code + help on [`ThemeError`] and
//! [`ResolveError`]) are OFF by default; the always-on `Display` strings
//! carry the same honesty.

use std::fmt;

use aeolus_core::{Color, ParseError, TailwindTheme, shadcn_css};
use aeolus_ir::Val;

/// Which value form a [`TokenTable`] resolves tokens to (plan §3.4's
/// "pay only for the dynamism you use").
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tier {
    /// T0/T1: colours bake to [`Val::Rgba`] literals, or to
    /// [`Val::ModePair`] when the token carries a dark value (the pair
    /// lives in [`TokenTable::pair_at`]).
    Baked,
    /// T2: colours carry as [`Val::Tok`] `{ id, alpha }` — one
    /// `TokenTable` load and one multiply at theme-resolution time.
    Runtime,
}

/// A CSS-first theme source: the light theme plus the optional dark
/// variant, parsed through the audited v0.x reader.
///
/// The dark side re-parses the light source with the `.dark` overrides
/// appended (`TailwindTheme::dark_variant`), so `@theme inline` var
/// chains re-resolve against the dark palette — the propagation CSS
/// gives us for free.
#[derive(Debug)]
pub struct ThemeSource {
    light: TailwindTheme,
    dark: Option<TailwindTheme>,
}

impl ThemeSource {
    /// Parse the light CSS (the same `@theme`/`:root`/`.dark` surface a
    /// web Tailwind v4 project uses, unchanged — plan §3.4's headline)
    /// and, when given, the `.dark` overrides that form the dark
    /// variant.
    ///
    /// # Errors
    /// [`ThemeError::Parse`] when either CSS block does not parse (the
    /// audited `ParseError`, carried verbatim).
    pub fn from_css(light_css: &str, dark_css: Option<&str>) -> Result<Self, ThemeError> {
        #[cfg(feature = "tracing")]
        tracing::debug!(light_len = light_css.len(), "theme source: parse light css");
        let light = TailwindTheme::from_theme_css(light_css)
            .map_err(|source| ThemeError::Parse { source })?;
        let dark = match dark_css {
            None => None,
            Some(css) => {
                #[cfg(feature = "tracing")]
                tracing::debug!(dark_len = css.len(), "theme source: parse dark variant");
                Some(
                    light
                        .dark_variant(css)
                        .map_err(|source| ThemeError::Parse { source })?,
                )
            }
        };
        Ok(Self { light, dark })
    }

    /// The shadcn neutral LIGHT palette only (T0's shape: no dark
    /// values, every token bakes to a literal).
    ///
    /// # Errors
    /// [`ThemeError::Parse`] when the embedded palette does not parse
    /// (unreachable for the compile-time data; handled honestly anyway).
    pub fn shadcn() -> Result<Self, ThemeError> {
        Self::from_css(shadcn_css::LIGHT, None)
    }

    /// The shadcn neutral palette, light AND dark (T1's shape: the
    /// paired tokens form const pairs).
    ///
    /// # Errors
    /// [`ThemeError::Parse`] when the embedded palettes do not parse
    /// (unreachable for the compile-time data; handled honestly anyway).
    pub fn shadcn_dark() -> Result<Self, ThemeError> {
        Self::from_css(shadcn_css::LIGHT, Some(shadcn_css::DARK))
    }

    /// The parsed light theme.
    pub fn light(&self) -> &TailwindTheme {
        &self.light
    }

    /// The parsed dark variant, when the source carries one.
    pub fn dark(&self) -> Option<&TailwindTheme> {
        self.dark.as_ref()
    }
}

/// Why a theme source or a token table failed to build. An empty colour
/// surface cannot occur (every theme is default-seeded with the full v4
/// palette), so the only failure mode is a parse failure — carried
/// verbatim, never stringly re-wrapped.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ThemeError {
    /// The light CSS or the dark overrides did not parse (the audited
    /// `ParseError`, carried verbatim).
    Parse {
        /// The underlying theme-parse failure.
        source: ParseError,
    },
}

impl fmt::Display for ThemeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ThemeError::Parse { source } => write!(f, "theme CSS: {source}"),
        }
    }
}

impl std::error::Error for ThemeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ThemeError::Parse { source } => Some(source),
        }
    }
}

#[cfg(feature = "miette")]
impl miette::Diagnostic for ThemeError {
    fn code(&self) -> Option<Box<dyn fmt::Display + '_>> {
        Some(Box::new("aeolus::theme::parse"))
    }

    fn help(&self) -> Option<Box<dyn fmt::Display + '_>> {
        Some(Box::new(
            "the theme source must be @theme/:root/.dark CSS with custom-property \
             declarations; every var() referenced inside calc() must be defined in \
             the same file (the audited v4 reader is the oracle)",
        ))
    }
}

/// Why a token id did not resolve to a [`Val`]. The compile seam turns
/// these into its own typed diagnostics; direct consumers match on them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ResolveError {
    /// The id is outside the table (ids come from
    /// [`TokenTable::id`]; an out-of-range id is a caller bug).
    UnknownId(u16),
    /// An opacity modifier rode a T1 pair slot (`Baked` tier, dark
    /// token): the pair form has no alpha arm — use the `Runtime` tier
    /// (the alpha rides `Val::Tok`) or bake a per-alpha program.
    AlphaOnPair,
    /// The alpha was not finite (`NaN`/`inf`): the audited clamp
    /// semantics cannot be honoured, so the resolve refuses instead of
    /// baking a NaN literal into the IR.
    AlphaNotFinite(f32),
}

impl fmt::Display for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResolveError::UnknownId(id) => write!(f, "token id {id} is outside the table"),
            ResolveError::AlphaOnPair => f.write_str(
                "an opacity modifier cannot ride the baked (light, dark) pair slot",
            ),
            ResolveError::AlphaNotFinite(alpha) => {
                write!(f, "token alpha {alpha} is not finite")
            }
        }
    }
}

impl std::error::Error for ResolveError {}

#[cfg(feature = "miette")]
impl miette::Diagnostic for ResolveError {
    fn code(&self) -> Option<Box<dyn fmt::Display + '_>> {
        Some(Box::new(match self {
            ResolveError::UnknownId(_) => "aeolus::theme::unknown_id",
            ResolveError::AlphaOnPair => "aeolus::theme::alpha_on_pair",
            ResolveError::AlphaNotFinite(_) => "aeolus::theme::alpha_not_finite",
        }))
    }

    fn help(&self) -> Option<Box<dyn fmt::Display + '_>> {
        Some(Box::new(match self {
            ResolveError::UnknownId(_) => {
                "token ids come from TokenTable::id(); an out-of-range id is a \
                 caller bug, not a styling result"
            }
            ResolveError::AlphaOnPair => {
                "use the Runtime tier (the alpha rides Val::Tok) or bake a \
                 per-alpha program; the pair form indexes a const (light, dark) \
                 table and has no alpha arm"
            }
            ResolveError::AlphaNotFinite(_) => {
                "the modifier grammar accepts finite decimals only (the audited \
                 clamp is 0..=1); NaN/inf modifiers are typed refusals upstream"
            }
        }))
    }
}

/// The tiered token table the compile seam's colour path lowers through:
/// one stable id per colour token (sorted-name order), the light colour
/// always, the dark colour when the source carries one.
///
/// The table is built ONCE per theme (the build parses nothing — the
/// source did — and snapshots `Color`s into a flat store); ids ride
/// compiled `Program`s, so they are a contract: sorted-name order,
/// test-pinned stable across rebuilds.
#[derive(Clone, Debug)]
pub struct TokenTable {
    tier: Tier,
    /// Sorted, deduplicated token names — index IS the id.
    names: Vec<String>,
    /// The light colour per token (same index as `names`).
    light: Vec<Color>,
    /// The dark colour per token, when the source carries a dark variant
    /// that defines one (same index as `names`).
    dark: Vec<Option<Color>>,
    /// The (light, dark) const pairs, indexed by `Val::ModePair`'s slot.
    pairs: Vec<(Color, Color)>,
    /// Per token: the pair slot it forms, when a dark value exists.
    pair_of: Vec<Option<u16>>,
}

impl TokenTable {
    /// Build the T0/T1 table: tokens without a dark value bake to
    /// [`Val::Rgba`] literals, tokens with one form the [`Val::ModePair`]
    /// const pairs.
    ///
    /// # Errors
    /// [`ThemeError`] is impossible here (the source is already parsed);
    /// the `Result` keeps the build seam symmetric for the xtask bake
    /// that replaces the runtime parse.
    pub fn baked(source: &ThemeSource) -> Result<Self, ThemeError> {
        Self::build(source, Tier::Baked)
    }

    /// Build the T2 table: every resolve carries as
    /// [`Val::Tok`] `{ id, alpha }`.
    ///
    /// # Errors
    /// Symmetric with [`TokenTable::baked`] (see its note).
    pub fn runtime(source: &ThemeSource) -> Result<Self, ThemeError> {
        Self::build(source, Tier::Runtime)
    }

    fn build(source: &ThemeSource, tier: Tier) -> Result<Self, ThemeError> {
        #[cfg(feature = "tracing")]
        tracing::debug!(?tier, "token table: build");
        let mut names: Vec<String> = source
            .light
            .color_names()
            .into_iter()
            .map(str::to_string)
            .collect();
        names.sort();
        names.dedup();
        // Snapshot the colours: the light theme defines every name (they
        // came from its own colour surface), the dark theme defines the
        // subset it overrides. A name its own source cannot resolve is
        // read-side drift — skipped, never a silent wrong value.
        let mut light: Vec<Color> = Vec::with_capacity(names.len());
        let mut dark: Vec<Option<Color>> = Vec::with_capacity(names.len());
        let mut kept: Vec<String> = Vec::with_capacity(names.len());
        for name in &names {
            let Some(color) = source.light.color(name) else {
                #[cfg(feature = "tracing")]
                tracing::warn!(name = %name, "token table: light colour missing, skipped");
                continue;
            };
            let dark_color = source.dark.as_ref().and_then(|theme| theme.color(name));
            light.push(color);
            dark.push(dark_color);
            kept.push(name.clone());
        }
        let mut pairs: Vec<(Color, Color)> = Vec::new();
        let mut pair_of: Vec<Option<u16>> = Vec::with_capacity(light.len());
        for (idx, dark_color) in dark.iter().enumerate() {
            match dark_color {
                Some(dark) => {
                    pairs.push((light[idx], *dark));
                    // The T1 budget: `Val::ModePair`'s slot is a u16. The
                    // same shape of constraint as the compile seam's u8
                    // keyword budget — refused honestly, never truncated.
                    let slot = u16::try_from(pairs.len() - 1).map_err(|_| ThemeError::Parse {
                        source: ParseError::new(
                            "the dark pair surface exceeds the IR's u16 ModePair slot",
                        ),
                    })?;
                    pair_of.push(Some(slot));
                }
                None => pair_of.push(None),
            }
        }
        #[cfg(feature = "tracing")]
        tracing::debug!(
            tokens = kept.len(),
            pairs = pairs.len(),
            "token table: built"
        );
        Ok(Self { tier, names: kept, light, dark, pairs, pair_of })
    }

    /// The tier this table resolves to (set at build time).
    pub fn tier(&self) -> Tier {
        self.tier
    }

    /// How many colour tokens the table carries.
    pub fn len(&self) -> usize {
        self.names.len()
    }

    /// True when no colour token resolved (impossible for a
    /// default-seeded theme; the guard the tests pin).
    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }

    /// Every token name, in id order (sorted-name order — the id
    /// contract).
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.names.iter().map(String::as_str)
    }

    /// The stable id of a colour token name (`"primary"`, `"red-500"`) —
    /// sorted-name order, stable across rebuilds of the same source.
    pub fn id(&self, name: &str) -> Option<u16> {
        self.names
            .binary_search_by(|probe| probe.as_str().cmp(name))
            .ok()
            .map(|idx| idx as u16)
    }

    /// The token name an id resolves to (the `id`/`name` inverse).
    pub fn name(&self, id: u16) -> Option<&str> {
        self.names.get(usize::from(id)).map(String::as_str)
    }

    /// The token's light colour as sRGB u8 channels (the baked literal's
    /// bytes).
    pub fn light_rgba(&self, id: u16) -> Option<[u8; 4]> {
        self.light.get(usize::from(id)).map(|color| color.to_rgba8())
    }

    /// True when the token carries a dark value (the T1 pair form).
    pub fn has_dark(&self, id: u16) -> bool {
        self.pair_of.get(usize::from(id)).is_some_and(Option::is_some)
    }

    /// The token's dark colour, when the source's dark variant defines
    /// one (`None` for a light-only token or a light-only source).
    pub fn dark_rgba(&self, id: u16) -> Option<[u8; 4]> {
        self.dark
            .get(usize::from(id))
            .and_then(Option::as_ref)
            .map(|color| color.to_rgba8())
    }

    /// The `(light, dark)` const pair at a [`Val::ModePair`] slot — the
    /// data the codegen indexes per mode (plan §3.4 T1: 1 index, 0
    /// multiplies).
    pub fn pair_at(&self, slot: u16) -> Option<([u8; 4], [u8; 4])> {
        self.pairs.get(usize::from(slot)).map(|(l, d)| (l.to_rgba8(), d.to_rgba8()))
    }

    /// How many const pairs the table carries (the T1 surface).
    pub fn pairs(&self) -> usize {
        self.pairs.len()
    }

    /// Resolve a token id into the IR value the tier promises:
    ///
    /// * `Runtime` — [`Val::Tok`] `{ id, alpha }`; `alpha` is `1.0` when
    ///   no modifier rode along, and the HOST applies the multiply at
    ///   theme-resolution time;
    /// * `Baked`, no dark value — [`Val::Rgba`], the premultiplied bake
    ///   when an alpha rides ([`Color::mix_with_transparent`], the
    ///   audited `/N` identity);
    /// * `Baked`, dark value, no alpha — [`Val::ModePair`], the slot into
    ///   [`TokenTable::pair_at`];
    /// * `Baked`, dark value, WITH alpha — [`ResolveError::AlphaOnPair`]
    ///   (typed boundary: the pair form has no alpha arm).
    ///
    /// The alpha is clamped to `0..=1` (the audited clamp); a non-finite
    /// alpha is [`ResolveError::AlphaNotFinite`], never a NaN literal in
    /// the IR.
    ///
    /// # Errors
    /// [`ResolveError`] per the three cases above.
    pub fn resolve(&self, id: u16, alpha: Option<f32>) -> Result<Val, ResolveError> {
        #[cfg(feature = "tracing")]
        tracing::trace!(id, ?alpha, tier = ?self.tier, "token resolve");
        let idx = usize::from(id);
        if idx >= self.names.len() {
            return Err(ResolveError::UnknownId(id));
        }
        match self.tier {
            Tier::Runtime => Ok(Val::Tok { id, alpha: alpha.unwrap_or(1.0) }),
            Tier::Baked => match alpha {
                None => match self.pair_of[idx] {
                    Some(slot) => Ok(Val::ModePair(slot)),
                    None => Ok(Val::Rgba(self.light[idx].to_rgba8())),
                },
                Some(a) => {
                    if !a.is_finite() {
                        return Err(ResolveError::AlphaNotFinite(a));
                    }
                    if self.pair_of[idx].is_some() {
                        return Err(ResolveError::AlphaOnPair);
                    }
                    let a = a.clamp(0.0, 1.0);
                    Ok(Val::Rgba(self.light[idx].mix_with_transparent(a * 100.0).to_rgba8()))
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The id contract: sorted-name order, first id zero.
    #[test]
    fn ids_are_sorted_name_order() {
        let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
        let table = TokenTable::runtime(&source).expect("default-seeded source builds");
        let names: Vec<&str> = table.names().collect();
        let mut sorted = names.clone();
        sorted.sort_unstable();
        assert_eq!(names, sorted, "ids are assigned in sorted-name order");
        assert_eq!(table.len(), names.len());
        assert!(!table.is_empty());
        for (idx, name) in names.iter().enumerate() {
            assert_eq!(table.id(name), Some(idx as u16), "id(name) is the index");
            assert_eq!(table.name(idx as u16), Some(*name), "name(id) inverts");
        }
    }
}
