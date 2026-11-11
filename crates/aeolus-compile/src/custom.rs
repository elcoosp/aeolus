//! The custom-authoring seam (architecture plan §5.4: "CSS-first
//! compatibility: `@theme`, `@utility` (incl. functional `--value()`),
//! `@custom-variant`" — the remaining P2 slice): users REGISTER custom
//! utilities and custom variants, and the pipeline lowers and renders
//! them exactly like registry rows and built-in spellings.
//!
//! ```text
//! Customs::new()
//!   .utility(CustomUtility::functional("brand-p", CustomForm::Spacing, &[…paddings]))
//!   .utility(CustomUtility::fixed("brand-r", Val::Rem(0.375), &[…radii]))
//!   .variant(CustomVariant::new("hocus", Cond::Hover))
//!   ↓
//! compile_with("brand-p-4 hocus:brand-r", &customs, &config)   -> Compiled
//! compile_theme_with(…, &tokens, &customs, &config)            -> Compiled
//! emit_css_with(…, &customs, &config)                          -> CssSheet
//! emit_css_theme_with(…, &tokens, &customs, &config)           -> CssSheet
//! ```
//!
//! The design keeps the pipeline's load-bearing invariants intact:
//!
//! * **The registry is the oracle; customs are ADDITIVE.** A built-in
//!   root always wins a name collision (`p` stays the padding scale
//!   even when a custom `p` is registered), and the typed read side is
//!   byte-shared: a custom miss is the SAME [`aeolus_data::LookupError`]
//!   shape (unknown root vs rejected shape, the custom root's name and
//!   the form's label in the tried list), rendered through the same
//!   stable diagnostic codes.
//! * **The IR is the budget.** A custom utility targets the IR's
//!   longhand set ([`Prop`]) and carries values the IR can hold
//!   ([`Val`]): functional forms lower the suffix through the SAME
//!   numeric grammar and finite guards as the built-in value pass
//!   (`nan`/`inf` spellings refuse, the px step is one absolute pixel,
//!   the spacing scale honours the [`crate::Spacing`] config), and a
//!   fixed [`Val::Kw`] is a typed refusal — the keyword slot indexes a
//!   row's own table, and a custom utility carries no row table.
//! * **The flat `Cond` model is the variant surface.** A registered
//!   custom variant is an ALIAS: a project's own spelling for a typed
//!   condition (`hocus` → [`Cond::Hover`], `night` → [`Cond::Dark`]).
//!   Both walks fold and render it identically (the differential pins
//!   diagnostics EQUAL with customs in play); spellings that need a
//!   selector the IR has no slot for stay the typed
//!   unsupported-variant boundary (the arbitrary-variant form, the
//!   parent×attribute compositions) — the attribute-probe families
//!   themselves fold as built-ins since the 2026-11 slice, and an
//!   alias may target their conds.
//! * **Registration interns into the leak pool.** The root names and
//!   target lists leak at registration — the same documented storage
//!   the IR's dynamic form uses ("one interned `Box<[Decl]>` shared by
//!   reference"): a one-time, bounded cost per registered item, never a
//!   per-compile allocation.
//!
//! What this slice deliberately is NOT: the `/opacity` modifier on a
//! custom utility (the modifier rides only the audited theme-token
//! path — a fixed literal has no premultiply bake of its own), and
//! selector-bearing custom variants (no IR slot; diagnosed, never
//! dropped). The theme-token functional form (`--value(color)`) is IN
//! this slice: [`CustomForm::Token`] lowers the suffix through the
//! tiered tables, modifier and all.

use aeolus_data::LookupError;
use aeolus_ir::{Cond, Prop, Val};

use crate::Config;
use crate::lower::{self, Reject};

/// Why a fixed [`Val::Kw`] cannot lower: the keyword slot indexes the
/// ROW'S OWN table (the codegen crate owns the names, plan §3.3) — a
/// custom utility carries no row table, so keywords are the built-in
/// registry's surface. The typed refusal renders through
/// [`crate::DiagnosticKind::UnsupportedValue`].
pub(crate) const REASON_CUSTOM_KEYWORD: &str = "the fixed value is the IR's row-relative keyword slot: a custom \
     utility carries no row table, so keyword utilities are the \
     built-in registry's surface";

/// Why a [`Val::Tok`]/[`Val::ModePair`] value cannot render in the
/// theme-less CSS walk: the runtime token forms are theme-pass forms —
/// `emit_css_theme` carries the table they render from.
pub(crate) const REASON_CUSTOM_TOKEN_TABLE: &str = "the token value renders through the theme pass's TokenTable: \
     emit_css_theme carries the table a var() reference or a dark pair \
     resolves from";

/// The value grammar a functional custom utility accepts — the v4
/// `@utility name-*` body's `--value(type)` hints, scoped to what the
/// IR's numeric slots, the audited spacing scale and the theme pass's
/// token path hold. The labels are the tried-list entries the typed
/// shape rejection reports.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CustomForm {
    /// `--value(integer)`: an integral decimal suffix (`brand-cols-3` →
    /// `Val::Num(3.0)`). A fractional suffix is a shape rejection.
    Integer,
    /// `--value(number)`: any finite decimal suffix (`brand-z-2.5` →
    /// `Val::Num(2.5)`). The `nan`/`inf` spellings parse and refuse
    /// exactly like the built-in numeric grammar.
    Number,
    /// `--value(percentage)`: a `%`-suffixed decimal (`brand-w-40%` →
    /// `Val::Pct(40.0)`) — the built-in `Percent` grammar's own
    /// spelling.
    Percentage,
    /// The `--spacing(n)` multiple: `n × 0.25rem` under the default
    /// config, the px bake under [`crate::Spacing::Px`], the `px` step
    /// one absolute pixel — the built-in spacing scale, byte-shared.
    Spacing,
    /// `--value(color)`: the suffix IS a theme token name
    /// (`brand-ink-primary` → the token's tiered [`Val`] form) — the
    /// THEME-token functional form, lowered through the same audited
    /// path the built-in colour rows ride ([`crate::TokenTable`]'s
    /// resolve: the T0 literal, the T1 pair, the T2
    /// [`Val::Tok`] `{ id, alpha }`). The `/opacity` modifier RIDES
    /// this form (the plan §2 premultiplied identity — the one form
    /// where the modifier is the theme pass's own arithmetic); without
    /// a table the refusal is the typed token-table boundary. Requires
    /// a suffix (the built-in colour shape's own rule); an unknown
    /// token name is the typed not-a-token miss.
    Token,
}

impl CustomForm {
    /// The short label the typed shape rejection reports in its tried
    /// list (the same list position the registry's value-kind labels
    /// hold).
    pub const fn label(self) -> &'static str {
        match self {
            CustomForm::Integer => "integer",
            CustomForm::Number => "number",
            CustomForm::Percentage => "percentage",
            CustomForm::Spacing => "spacing",
            CustomForm::Token => "color",
        }
    }
}

/// The value a custom utility lowers to: a FIXED literal (baked at
/// registration) or a FUNCTIONAL form (the class suffix fills the
/// slot).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CustomValue {
    /// `@utility brand-r { border-radius: 0.375rem; }` — the value is
    /// fixed; the class is the bare root (`brand-r`, no suffix). The
    /// negated `-brand-r` form refuses (a fixed value has no sign to
    /// flip), and a [`Val::Kw`] fixed value refuses typed (the keyword
    /// slot is row-relative).
    Fixed(Val),
    /// `@utility brand-p-* { padding: --spacing(n); }` — the suffix
    /// lowers through the form's grammar.
    Functional(CustomForm),
}

/// One registered custom utility: the class root, the value shape and
/// the longhand fan-out — the same three-part contract a registry row
/// carries ([`aeolus_data::UtilityRow`]), authored at runtime instead
/// of compiled in.
///
/// The root and the target list intern into the leak pool at
/// construction (the crate docs carry the storage story): a one-time
/// bounded cost, never a per-compile allocation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CustomUtility {
    /// The class root (`"brand-p"` for `brand-p-4`). Built-in roots
    /// always win a name collision — the registry is the semantics
    /// oracle, registrations are additive surface.
    pub root: &'static str,
    /// The value shape (fixed literal or functional form).
    pub value: CustomValue,
    /// The longhand targets, in the row's own fan-out order (the layer
    /// re-sorts into cascade order). Never empty — an empty target list
    /// would be a row that lowers to nothing, the one silent failure
    /// the pipeline does not have.
    pub targets: &'static [Prop],
}

impl CustomUtility {
    /// Register a fixed-value utility: the class is the bare root and
    /// the value bakes as given (guard-checked at lower time — a
    /// non-finite numeric or a row-relative [`Val::Kw`] refuse typed).
    pub fn fixed(root: &str, val: Val, targets: &[Prop]) -> Self {
        Self {
            root: intern(root),
            value: CustomValue::Fixed(val),
            targets: intern_props(targets),
        }
    }

    /// Register a functional utility: the suffix after `root-` lowers
    /// through the form's grammar (the class root is the v4
    /// `@utility brand-p-*` spelling's star-less part).
    pub fn functional(root: &str, form: CustomForm, targets: &[Prop]) -> Self {
        Self {
            root: intern(root),
            value: CustomValue::Functional(form),
            targets: intern_props(targets),
        }
    }
}

/// One registered custom variant: a project's own spelling for a typed
/// condition — the v4 `@custom-variant night (&:where(.dark, .dark *))"
/// shape, scoped to the flat [`Cond`] surface the IR carries.
///
/// Selector spellings the IR has no slot for (arbitrary selectors,
/// `has-*`/`aria-*` chains) stay the typed unsupported-variant
/// boundary — diagnosed, never dropped.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CustomVariant {
    /// The variant spelling (`"hocus"` for `hocus:bg-primary`).
    pub name: &'static str,
    /// The typed condition the spelling folds to (the alias's whole
    /// meaning — the IR walk layers under it, the CSS walk renders the
    /// condition's own selector shape).
    pub maps_to: Cond,
}

impl CustomVariant {
    /// Register an alias: the spelling folds to `maps_to` in both
    /// walks. Aliasing to [`Cond::Base`] is accepted and folds to the
    /// base layer (the spelling behaves as if no variant rode along) —
    /// deterministic, documented, and the honest reading of the flat
    /// model.
    pub fn new(name: &str, maps_to: Cond) -> Self {
        Self {
            name: intern(name),
            maps_to,
        }
    }
}

/// The custom-variant classification seam: what a variant segment the
/// built-in table does not know folds to. [`Customs`] implements it
/// over its registered aliases; the pipeline's fold is generic over
/// this trait, so a host can supply its own classification table
/// without owning the registrations.
pub trait ClassifyVariant {
    /// The typed condition `variant` folds to, or `None` when the
    /// table does not know it (the caller keeps the typed
    /// unsupported-variant boundary).
    fn classify(&self, variant: &str) -> Option<Cond>;
}

impl ClassifyVariant for Customs {
    fn classify(&self, variant: &str) -> Option<Cond> {
        self.variants
            .iter()
            .find(|v| v.name == variant)
            .map(|v| v.maps_to)
    }
}

/// The registrations — the custom-authoring half of the pipeline's
/// inputs. `Copy`-free by design (it owns the tables); empty
/// registrations are byte-identical to the theme-less siblings
/// (the differential pins it), so existing call sites can adopt the
/// `*_with` entry points without behaviour drift.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Customs {
    utilities: Vec<CustomUtility>,
    variants: Vec<CustomVariant>,
}

impl Customs {
    /// Empty registrations.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a custom utility (builder style; first registration of
    /// a root wins ties among customs — the registry itself wins over
    /// ALL of them).
    pub fn utility(mut self, utility: CustomUtility) -> Self {
        self.utilities.push(utility);
        self
    }

    /// Register a custom variant (builder style).
    pub fn variant(mut self, variant: CustomVariant) -> Self {
        self.variants.push(variant);
        self
    }

    /// True when nothing is registered (the differential's subject).
    pub fn is_empty(&self) -> bool {
        self.utilities.is_empty() && self.variants.is_empty()
    }

    /// The registered utilities, in registration order (the audit
    /// view; first root wins ties among customs).
    pub fn utilities(&self) -> &[CustomUtility] {
        &self.utilities
    }

    /// The registered variants, in registration order (the audit view).
    pub fn variants(&self) -> &[CustomVariant] {
        &self.variants
    }

    /// The custom read side: the utilities whose root addresses `body`
    /// under the longest-root rule (the registry's own rule, mirrored),
    /// every one whose value shape accepts the suffix — a root can
    /// carry several arms exactly like a registry root does. The typed
    /// misses are the SAME [`LookupError`] shape the registry reports,
    /// with the custom root's name and the forms' labels.
    pub(crate) fn lookup(&self, body: &str) -> Result<Vec<CustomUtility>, LookupError> {
        #[cfg(feature = "tracing")]
        tracing::debug!(body = %body, utilities = self.utilities.len(), "custom lookup");
        let best = self
            .utilities
            .iter()
            .filter(|utility| body_matches_root(body, utility.root))
            .map(|utility| utility.root.len())
            .max();
        let Some(len) = best else {
            #[cfg(feature = "tracing")]
            tracing::debug!(body = %body, "no registered custom root prefixes the body");
            return Err(LookupError::UnknownRoot {
                body: body.to_string(),
            });
        };
        let matching: Vec<CustomUtility> = self
            .utilities
            .iter()
            .copied()
            .filter(|utility| utility.root.len() == len && body_matches_root(body, utility.root))
            .collect();
        let accepting: Vec<CustomUtility> = matching
            .iter()
            .copied()
            .filter(|utility| value_accepts(utility.value, body, utility.root))
            .collect();
        if accepting.is_empty() {
            let mut tried: Vec<&'static str> = Vec::new();
            for utility in &matching {
                let label = match utility.value {
                    CustomValue::Fixed(_) => "fixed",
                    CustomValue::Functional(form) => form.label(),
                };
                if !tried.contains(&label) {
                    tried.push(label);
                }
            }
            #[cfg(feature = "tracing")]
            tracing::debug!(
                root = matching.first().map(|u| u.root).unwrap_or_default(),
                body = %body,
                ?tried,
                "custom root matched but no value form accepted the suffix"
            );
            return Err(LookupError::ShapeRejected {
                body: body.to_string(),
                root: matching
                    .first()
                    .map(|utility| utility.root)
                    .unwrap_or_default(),
                tried,
            });
        }
        Ok(accepting)
    }
}

/// Does `body` address `root`? Either exact (`brand-r`) or
/// `root + '-' + suffix` (`brand-p-4` under `brand-p`) — the registry's
/// own rule (`aeolus_data::registry::body_matches_root` is private; the
/// four lines are mirrored here so the read sides cannot drift apart in
/// BEHAVIOUR, and the differential pins they do not).
fn body_matches_root(body: &str, root: &str) -> bool {
    body == root || (body.starts_with(root) && body.as_bytes().get(root.len()) == Some(&b'-'))
}

/// The suffix after the root (`"4"` for `brand-p-4`, `None` for the
/// bare root).
fn suffix_of<'a>(body: &'a str, root: &str) -> Option<&'a str> {
    if body == root {
        None
    } else {
        Some(&body[root.len() + 1..])
    }
}

/// The shape contract: does the utility's value form accept the body's
/// suffix? The built-in `ValueKind::accepts` grammar, mirrored per
/// form — including its honest edges (the numeric grammars accept the
/// `nan`/`inf` PARSES and the finite guard refuses them at lower time,
/// the same two-stage shape the built-in value pass pins).
fn value_accepts(value: CustomValue, body: &str, root: &str) -> bool {
    let suffix = suffix_of(body, root);
    match value {
        CustomValue::Fixed(_) => suffix.is_none(),
        CustomValue::Functional(form) => match form {
            CustomForm::Integer => suffix.is_some_and(integral),
            CustomForm::Number => suffix.is_some_and(|s| s.parse::<f32>().is_ok()),
            CustomForm::Percentage => {
                suffix.is_some_and(|s| s.ends_with('%') && s[..s.len() - 1].parse::<f32>().is_ok())
            }
            CustomForm::Spacing => {
                let s = suffix.unwrap_or("");
                s == "px" || s.strip_prefix('-').unwrap_or(s).parse::<f32>().is_ok()
            }
            // The token shape accepts ANY suffix (the built-in colour
            // arm's own rule): the name resolution is lower-time, where
            // the not-a-token miss is typed.
            CustomForm::Token => suffix.is_some(),
        },
    }
}

/// The integral grammar: a finite decimal with no fractional part
/// (`"3"` yes, `"2.5"` no — the `--value(integer)` hint's own shape).
fn integral(s: &str) -> bool {
    s.parse::<f32>()
        .is_ok_and(|n| n.is_finite() && n.fract() == 0.0)
}

/// Lower ONE registered utility for one candidate body: the fixed
/// value fan-out, or the functional form's parse. The same typed
/// boundary set as the built-in value pass (`Reject`), rendered through
/// the same diagnostic layer.
pub(crate) fn utility_to_decls(
    utility: CustomUtility,
    body: &str,
    negative: bool,
    alpha: Option<f32>,
    tokens: Option<&aeolus_theme::TokenTable>,
    config: Config,
) -> Result<Vec<aeolus_ir::Decl>, Reject> {
    let val = match utility.value {
        CustomValue::Fixed(val) => {
            // The modifier boundary on the fixed forms: the audited
            // premultiply bake lives in the theme pass's token path, and
            // a fixed literal has no bake of its own — a modifier never
            // half-applies, it refuses the whole row.
            if alpha.is_some() {
                return Err(Reject::Modifier);
            }
            if negative {
                return Err(Reject::Negation);
            }
            fixed_guard(val)?;
            val
        }
        // The theme-token form: the suffix IS the token name, the
        // table's tier resolves it, and the modifier RIDES (the plan §2
        // identity — the audited path, not a re-implementation).
        CustomValue::Functional(CustomForm::Token) => {
            if negative {
                return Err(Reject::Negation);
            }
            token_value(suffix_of(body, utility.root), alpha, tokens)?
        }
        CustomValue::Functional(form) => {
            // The modifier boundary on the numeric forms (see the fixed
            // arm): a modifier never half-applies.
            if alpha.is_some() {
                return Err(Reject::Modifier);
            }
            let suffix = suffix_of(body, utility.root);
            let val = functional_value(form, suffix, config)?;
            if negative { negate(val) } else { val }
        }
    };
    Ok(lower::fan(utility.targets, val))
}

/// The theme-token functional form's resolve — the built-in colour
/// path's own seam (`lower::token_decls`' resolve half, mirrored so the
/// custom arm carries the SAME tier semantics and refusals): the table
/// resolves the id per tier, the alpha rides the colour-form value, and
/// every miss is typed (the theme-less token-table boundary, the
/// not-a-token read miss, the alpha-on-pair modifier boundary).
fn token_value(
    suffix: Option<&str>,
    alpha: Option<f32>,
    tokens: Option<&aeolus_theme::TokenTable>,
) -> Result<Val, Reject> {
    let Some(table) = tokens else {
        return Err(Reject::Value {
            reason: lower::REASON_TOKEN_TABLE,
        });
    };
    let Some(name) = suffix else {
        // Unreachable through `lookup` (the token shape accepts a suffix
        // only); the refusal stays typed regardless.
        return Err(Reject::Value {
            reason: lower::REASON_SHAPE_MISMATCH,
        });
    };
    let Some(id) = table.id(name) else {
        return Err(Reject::Value {
            reason: lower::REASON_NOT_A_TOKEN,
        });
    };
    table.resolve(id, alpha).map_err(|err| match err {
        aeolus_theme::ResolveError::UnknownId(_) => Reject::Value {
            reason: lower::REASON_NOT_A_TOKEN,
        },
        aeolus_theme::ResolveError::AlphaOnPair | aeolus_theme::ResolveError::AlphaNotFinite(_) => {
            Reject::Modifier
        }
    })
}

/// Render ONE registered utility for the CSS walk: the same parse the
/// IR form runs, rendered through the value-text forms the emitter
/// owns (`1rem`, `rgb(r g b)`, the `var(--color-…)` references). The
/// modifier boundary mirrors the IR form: the audited premultiply bake
/// lives in the theme pass's token path, so a modifier refuses the
/// whole utility instead of half-applying.
pub(crate) fn utility_to_css(
    utility: CustomUtility,
    body: &str,
    negative: bool,
    alpha: Option<f32>,
    tokens: Option<&aeolus_theme::TokenTable>,
    config: Config,
) -> Result<Vec<lower::CssDecl>, Reject> {
    let val = match utility.value {
        CustomValue::Fixed(val) => {
            if alpha.is_some() {
                return Err(Reject::Modifier);
            }
            if negative {
                return Err(Reject::Negation);
            }
            fixed_guard(val)?;
            val
        }
        // The theme-token form renders through the same resolve (the
        // `var()`/`color-mix`/pair forms are `val_css`'s own arms).
        CustomValue::Functional(CustomForm::Token) => {
            if negative {
                return Err(Reject::Negation);
            }
            token_value(suffix_of(body, utility.root), alpha, tokens)?
        }
        CustomValue::Functional(form) => {
            if alpha.is_some() {
                return Err(Reject::Modifier);
            }
            let suffix = suffix_of(body, utility.root);
            let val = functional_value(form, suffix, config)?;
            if negative { negate(val) } else { val }
        }
    };
    let value = val_css(val, tokens)?;
    Ok(utility
        .targets
        .iter()
        .map(|&prop| lower::CssDecl {
            prop,
            prop_name: prop.name(),
            value: value.clone(),
        })
        .collect())
}

/// The fixed-value guard: a non-finite numeric (a caller-baked `NaN`)
/// or a row-relative keyword slot refuse typed — the IR never carries
/// a non-finite literal, and a keyword with no table has no name.
fn fixed_guard(val: Val) -> Result<(), Reject> {
    if let Some(n) = val.as_num() {
        lower::finite(n)?;
    }
    if let Val::Tok { alpha, .. } = val {
        lower::finite(alpha)?;
    }
    if matches!(val, Val::Kw(_)) {
        return Err(Reject::Value {
            reason: REASON_CUSTOM_KEYWORD,
        });
    }
    Ok(())
}

/// The functional forms' parse — the SAME numeric grammar and finite
/// guards the built-in value pass pins (parse_num refuses the
/// non-finite spellings; the spacing scale honours the config).
fn functional_value(form: CustomForm, suffix: Option<&str>, config: Config) -> Result<Val, Reject> {
    match form {
        CustomForm::Integer => {
            let s = suffix.unwrap_or_default();
            if !integral(s) {
                return Err(Reject::Value {
                    reason: lower::REASON_SHAPE_MISMATCH,
                });
            }
            // `integral` vetted the parse; the unwrap-or keeps the
            // reachable path panic-free regardless.
            let n = s.parse::<f32>().unwrap_or(0.0);
            Ok(Val::Num(n))
        }
        CustomForm::Number => {
            let n = lower::parse_num(suffix.unwrap_or_default())?;
            Ok(Val::Num(n))
        }
        CustomForm::Percentage => {
            let s = suffix.unwrap_or_default();
            let pct = lower::parse_num(s.strip_suffix('%').unwrap_or(s))?;
            Ok(Val::Pct(pct))
        }
        CustomForm::Spacing => {
            let s = suffix.unwrap_or_default();
            if s == "px" {
                return Ok(Val::Px(1.0));
            }
            let rem = lower::parse_num(s)? * aeolus_data::tables::SPACING_STEP_REM;
            let rem = lower::finite(rem)?;
            match config.spacing {
                crate::Spacing::Tailwind => Ok(Val::Rem(rem)),
                crate::Spacing::Px => {
                    let px = lower::finite(rem * config.rem_px)?;
                    Ok(Val::Px(px))
                }
            }
        }
        // The theme-token form's resolve needs the alpha and the table;
        // it never reaches this helper (the caller dispatches it to
        // `token_value` before the numeric grammar runs).
        CustomForm::Token => Err(Reject::Value {
            reason: lower::REASON_SHAPE_MISMATCH,
        }),
    }
}

/// The sign flip for a functional negation (every functional form is
/// numeric — the shape contract guarantees it).
fn negate(val: Val) -> Val {
    match val {
        Val::Px(n) => Val::Px(-n),
        Val::Rem(n) => Val::Rem(-n),
        Val::Pct(n) => Val::Pct(-n),
        Val::Num(n) => Val::Num(-n),
        // Unreachable through `functional_value`; the value rides
        // unchanged rather than panicking on a reachable path.
        other => other,
    }
}

/// The value-text forms the CSS walk renders a custom utility's value
/// through — the same primitives the built-in emitter owns
/// (`rgba_text`, `token_text`), with the theme-less boundaries typed.
/// The baked dark pair is handled BEFORE this helper (the pair renders
/// as the light base plus the `:where(.dark, .dark *)` companion rule).
fn val_css(val: Val, tokens: Option<&aeolus_theme::TokenTable>) -> Result<lower::CssValue, Reject> {
    if let Val::ModePair(slot) = val {
        let Some(table) = tokens else {
            return Err(Reject::Value {
                reason: REASON_CUSTOM_TOKEN_TABLE,
            });
        };
        let Some((light, dark)) = table.pair_at(slot) else {
            return Err(Reject::Value {
                reason: lower::REASON_NOT_A_TOKEN,
            });
        };
        return Ok(lower::CssValue {
            base: lower::rgba_text(light),
            dark: Some(lower::rgba_text(dark)),
        });
    }
    let base = match val {
        Val::Rem(n) => format!("{n}rem"),
        Val::Px(n) => format!("{n}px"),
        Val::Pct(n) => format!("{n}%"),
        Val::Num(n) => format!("{n}"),
        Val::Rgba(rgba) => lower::rgba_text(rgba),
        Val::Tok { id, alpha } => {
            let Some(table) = tokens else {
                return Err(Reject::Value {
                    reason: REASON_CUSTOM_TOKEN_TABLE,
                });
            };
            let Some(name) = table.name(id) else {
                return Err(Reject::Value {
                    reason: lower::REASON_NOT_A_TOKEN,
                });
            };
            lower::token_text(name, (alpha < 1.0).then_some(alpha))
        }
        // The pair form is handled above; this arm is the drift guard
        // the match totality wants — typed, never a panic.
        Val::ModePair(_) => {
            return Err(Reject::Value {
                reason: lower::REASON_SHAPE_MISMATCH,
            });
        }
        Val::Kw(_) => {
            return Err(Reject::Value {
                reason: REASON_CUSTOM_KEYWORD,
            });
        }
    };
    Ok(lower::CssValue { base, dark: None })
}

/// Intern a root/variant name into the leak pool (the crate docs carry
/// the storage story: a one-time bounded cost per registered item).
fn intern(s: &str) -> &'static str {
    Box::leak(s.to_string().into_boxed_str())
}

/// Intern a target list into the leak pool ([`intern`]'s slice form).
fn intern_props(props: &[Prop]) -> &'static [Prop] {
    Box::leak(props.to_vec().into_boxed_slice())
}
