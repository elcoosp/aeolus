//! Resolve parsed class tokens into a [`StylePlan`] against a [`TailwindTheme`].
//!
//! v0.2 routing (plan §2.1): a token whose variant stack is exactly one
//! classic variant (`hover`, `md`, `dark`, `group-hover`, …) keeps going to
//! the v0.1 bucket, which backends lower natively. **Everything else goes to
//! `rules`**, evaluated by the [`crate::eval::Probe`] parity engine —
//! nothing is dropped, and nothing is collapsed into the wrong bucket
//! (fixes F3/F4: `dark:hover:*` and `md:hover:*` no longer lose the `dark`/
//! breakpoint half).
//!
//! The utility table covers the Shadcn v4 utility set plus the v0.2
//! expansion (plan §3.2): sizing/spacing (`mx-auto`, `gap-x/y`, container
//! scale, viewport sizes), layout (`basis`, `order`, `grid-*`, `aspect-*`),
//! position (`absolute`, `inset-*`, `z-*`, `overflow-*`), typography
//! (`italic`, `uppercase`, `truncate`, `line-clamp`, `font-sans`), effects
//! (`transition-*`, `animate-*`, `scale/rotate/translate/skew`, shadow
//! colors, inset shadows, border styles), interaction (`cursor-*` full set,
//! `select-*`). Utilities outside it produce typed **warnings**, never
//! silent mis-styling.

use crate::ast::{
    ArbForm, Arbitrary, AttrKind, Breakpoint, ClassToken, Media, NamedVariant, ParentKind, State,
    Variant,
};
use crate::color::Color;
use crate::cond::{Atom, Cond, Rule};
use crate::parser::parse_class_string;
use crate::plan::*;
use crate::theme::TailwindTheme;
use std::collections::BTreeMap;

/// A non-fatal resolution outcome for one class.
#[derive(Clone, Debug, PartialEq)]
pub enum Warning {
    /// Recognized but not expressible natively (pseudo-elements, filters, ...).
    Unsupported {
        /// The raw class that triggered the warning.
        class: String,
        /// Why it cannot be lowered.
        reason: &'static str,
    },
    /// No utility table entry matched.
    Unknown {
        /// The raw class that triggered the warning.
        class: String,
        /// A close known utility, when one is within edit distance.
        suggestion: Option<String>,
    },
    /// Parsed but the theme token is missing (`bg-notacolor`).
    MissingThemeToken {
        /// The raw class that triggered the warning.
        class: String,
        /// The token name that was not found.
        token: String,
    },
    /// The class failed the grammar (malformed value). The rest of the
    /// string is unaffected; the class is skipped.
    Unparseable {
        /// The raw class that failed to parse.
        class: String,
        /// The parser's message.
        message: String,
    },
}

/// Result of resolving a class string.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Resolution {
    /// The resolved plan.
    pub plan: StylePlan,
    /// Non-fatal diagnostics (unsupported/unknown/unparseable classes).
    pub warnings: Vec<Warning>,
}

/// The resolver: one per theme snapshot.
pub struct Resolver<'t> {
    /// The theme tokens utilities resolve against.
    pub theme: &'t TailwindTheme,
}

/// Where a token's utility lands (plan §2.1 `Route`).
#[derive(Clone, Debug)]
enum Route {
    /// No variants — the base style.
    Base,
    /// Exactly one classic state variant — the v0.1 layer bucket.
    Layer(VariantCondition),
    /// Exactly one default breakpoint variant — the v0.1 responsive bucket.
    Responsive(Breakpoint),
    /// Exactly `dark:` — the v0.1 dark bucket.
    Dark,
    /// Exactly one `group-<state>` — the v0.1 group bucket (legacy label
    /// `{label}{state}`, e.g. `hover`, `menufocus`).
    Group(String),
    /// Exactly one `peer-<state>` — the v0.1 peer bucket.
    Peer(String),
    /// Anything else — the v0.2 rules with the full atom conjunction.
    Rules(Cond),
    /// Not expressible at all — a precise `Unsupported` warning.
    Skip(Warning),
}

/// A single utility's effect on a bucket.
#[derive(Clone, Debug, PartialEq)]
enum Applied {
    Replace(Slot, PartialValue),
    /// Per-edge families accumulate (`p-4` then `px-2` keeps top/bottom).
    Padding {
        top: Option<f32>,
        right: Option<f32>,
        bottom: Option<f32>,
        left: Option<f32>,
    },
    Margin {
        top: Option<f32>,
        right: Option<f32>,
        bottom: Option<f32>,
        left: Option<f32>,
    },
    BorderWidth {
        top: Option<f32>,
        right: Option<f32>,
        bottom: Option<f32>,
        left: Option<f32>,
    },
    Radius {
        tl: Option<f32>,
        tr: Option<f32>,
        br: Option<f32>,
        bl: Option<f32>,
    },
    Text(TextProperty, TextValue),
    /// font-size + paired line-height (`text-sm` sets both)
    Texts(Vec<(TextProperty, TextValue)>),
    // ---- v0.2 extras -------------------------------------------------------
    Position(Position),
    Inset {
        top: Option<SizedValue>,
        right: Option<SizedValue>,
        bottom: Option<SizedValue>,
        left: Option<SizedValue>,
    },
    ZIndex(i32),
    Overflow(Overflow),
    Visibility(Visibility),
    AspectRatio(f32),
    Grid(Grid),
    Transform {
        scale_x: Option<f32>,
        scale_y: Option<f32>,
        rotate_deg: Option<f32>,
        translate_x: Option<SizedValue>,
        translate_y: Option<SizedValue>,
        skew_x: Option<f32>,
        skew_y: Option<f32>,
    },
    Transition {
        set: Option<TransitionSet>,
        duration_ms: Option<f32>,
        ease: Option<Ease>,
        delay_ms: Option<f32>,
    },
    Animation(Animation),
    BorderStyle(BorderStyle),
    FlexBasis(SizedValue),
    Order(i16),
    AlignContent(ContentAlign),
    MarginAuto(AutoEdges),
    GapAxes(GapAxes),
    /// `shadow-{color}` — recolors the shadow preset in the same bucket.
    ShadowColor(Color),
    /// A multi-layer `shadow-*` preset: the dominant first layer plus the
    /// remaining layers (v0.2, plan §3.3).
    ShadowPreset {
        first: BoxShadow,
        rest: Vec<BoxShadow>,
    },
    InsetShadows(Vec<BoxShadow>),
}

#[derive(Clone, Debug, PartialEq)]
enum PartialValue {
    Background(Background),
    ContentColor(ContentColor),
    BorderColor(BorderColor),
    Dimensions(Dimensions),
    MinDimensions(Dimensions),
    MaxDimensions(Dimensions),
    Gap(Gap),
    Shadow(BoxShadow),
    Layout(Layout),
    Opacity(Opacity),
    Ring(Ring),
    Outline(Outline),
    Cursor(Cursor),
    PointerEvents(PointerEvents),
}

/// Per-condition accumulator: raw applied list finalized by overlay order.
#[derive(Default)]
struct Bucket {
    applied: Vec<Applied>,
}

impl<'t> Resolver<'t> {
    /// A resolver bound to one theme snapshot.
    pub fn new(theme: &'t TailwindTheme) -> Self {
        Self { theme }
    }

    /// Parse + resolve a class string into a plan.
    ///
    /// `@utility name { @apply … }` expansions are inlined first
    /// (depth-limited to 4; plan §3.3).
    pub fn resolve(&self, classes: &str) -> Result<Resolution, Vec<crate::color::ParseError>> {
        let expanded = self.expand_utilities(classes, 0);
        let (tokens, unparseable) = parse_class_string(&expanded);
        let mut resolution = self.resolve_tokens(tokens);
        // Unknown SYNTAX is a warning, not a failure: shadcn v4 strings
        // legitimately contain variants we do not model (has-*, data-*,
        // aria-*), and the rest of the string must still style.
        if !unparseable.is_empty() {
            resolution.warnings.extend(
                unparseable
                    .into_iter()
                    .map(|(class, message)| Warning::Unparseable { class, message }),
            );
        }
        Ok(resolution)
    }

    /// Inline `@utility` expansions at the class-string level (plan §3.3):
    /// each token whose utility body matches a registered `@utility` is
    /// replaced by `variants:@apply-classes`, depth-limited to 4.
    fn expand_utilities(&self, classes: &str, depth: u8) -> String {
        if depth > 4 {
            return classes.to_string();
        }
        // Fast path: stock themes declare no @utility rules — return the
        // input untouched (no per-word re-parse on the hot path).
        if !self.theme.has_custom_utilities() {
            return classes.to_string();
        }
        let mut out = String::with_capacity(classes.len());
        let mut changed = false;
        for (i, word) in classes.split_ascii_whitespace().enumerate() {
            if i > 0 {
                out.push(' ');
            }
            let replaced = parse_class_string(word).0.into_iter().next().and_then(|t| {
                self.theme
                    .custom_utility(t.body)
                    .filter(|u| !u.classes.is_empty())
                    .map(|u| {
                        let prefix = t.variant_prefix();
                        if prefix.is_empty() {
                            u.classes.clone()
                        } else {
                            format!("{prefix}:{}", u.classes)
                        }
                    })
            });
            match replaced {
                Some(exp) => {
                    changed = true;
                    out.push_str(&self.expand_utilities(&exp, depth + 1));
                }
                None => out.push_str(word),
            }
        }
        if changed {
            self.expand_utilities(&out, depth + 1)
        } else {
            out
        }
    }

    /// Resolve pre-parsed tokens (used by the cache and by tests).
    pub fn resolve_tokens<'a>(&self, tokens: Vec<ClassToken<'a>>) -> Resolution {
        let mut warnings = Vec::new();
        let mut base = Bucket::default();
        let mut layers: Vec<(VariantCondition, Bucket)> = Vec::new();
        let mut responsive: BTreeMap<Breakpoint, Bucket> = BTreeMap::new();
        let mut container: BTreeMap<Breakpoint, Bucket> = BTreeMap::new();
        let mut dark = Bucket::default();
        let mut group: Vec<(String, Bucket)> = Vec::new();
        let mut peer: Vec<(String, Bucket)> = Vec::new();
        let mut rules: Vec<Rule> = Vec::new();

        for token in tokens {
            let route = self.route(&token);
            match route {
                Route::Skip(w) => warnings.push(w),
                Route::Rules(cond) => match self.apply_utility(&token) {
                    Ok(applied) => {
                        let mut b = Bucket::default();
                        b.applied.push(applied);
                        rules.push(Rule {
                            when: cond,
                            style: self.finalize(&b),
                        });
                    }
                    Err(w) => warnings.push(w),
                },
                Route::Base => {
                    if let Some(applied) = self.applied_of(&token, &mut warnings) {
                        base.applied.push(applied);
                    }
                }
                Route::Layer(cond) => {
                    let idx = layers.iter().position(|(k, _)| *k == cond);
                    let idx = idx.unwrap_or_else(|| {
                        layers.push((cond, Bucket::default()));
                        layers.len() - 1
                    });
                    if let Some(applied) = self.applied_of(&token, &mut warnings) {
                        layers[idx].1.applied.push(applied);
                    }
                }
                Route::Responsive(bp) => {
                    if let Some(applied) = self.applied_of(&token, &mut warnings) {
                        responsive.entry(bp).or_default().applied.push(applied);
                    }
                }
                Route::Dark => {
                    if let Some(applied) = self.applied_of(&token, &mut warnings) {
                        dark.applied.push(applied);
                    }
                }
                Route::Group(label) => {
                    let idx = group.iter().position(|(l, _)| *l == label);
                    let idx = idx.unwrap_or_else(|| {
                        group.push((label, Bucket::default()));
                        group.len() - 1
                    });
                    if let Some(applied) = self.applied_of(&token, &mut warnings) {
                        group[idx].1.applied.push(applied);
                    }
                }
                Route::Peer(label) => {
                    let idx = peer.iter().position(|(l, _)| *l == label);
                    let idx = idx.unwrap_or_else(|| {
                        peer.push((label, Bucket::default()));
                        peer.len() - 1
                    });
                    if let Some(applied) = self.applied_of(&token, &mut warnings) {
                        peer[idx].1.applied.push(applied);
                    }
                }
            }
        }

        // Legacy @container bucket (v0.1 compat): a single @md variant used
        // to land here; v0.2 routes container queries into `rules`. The
        // bucket stays for backends compiled against v0.1.
        let _ = &mut container;

        let plan = StylePlan {
            base: self.finalize(&base),
            layers: layers
                .into_iter()
                .map(|(condition, b)| ConditionalLayer {
                    condition,
                    style: self.finalize(&b),
                })
                .collect(),
            responsive: responsive
                .into_iter()
                .map(|(b, v)| (b, self.finalize(&v)))
                .collect(),
            container: container
                .into_iter()
                .map(|(b, v)| (b, self.finalize(&v)))
                .collect(),
            dark: (!dark.applied.is_empty()).then(|| self.finalize(&dark)),
            group: group
                .into_iter()
                .filter(|(_, b)| !b.applied.is_empty())
                .map(|(l, v)| (l.into_boxed_str(), self.finalize(&v)))
                .collect(),
            peer: peer
                .into_iter()
                .filter(|(_, b)| !b.applied.is_empty())
                .map(|(l, v)| (l.into_boxed_str(), self.finalize(&v)))
                .collect(),
            rules,
            breakpoints: self.theme.breakpoints_px(),
        };

        Resolution { plan, warnings }
    }

    /// Apply one token, reporting any warning; `None` = the class warned.
    fn applied_of(&self, token: &ClassToken<'_>, warnings: &mut Vec<Warning>) -> Option<Applied> {
        match self.apply_utility(token) {
            Ok(applied) => Some(applied),
            Err(w) => {
                warnings.push(w);
                None
            }
        }
    }

    // ------------------------------------------------------------------
    // routing: the WHOLE variant stack decides where the utility lands
    // ------------------------------------------------------------------
    fn route(&self, t: &ClassToken<'_>) -> Route {
        let class = t.raw.to_string();
        let mut atoms = Vec::new();
        for v in t.variants.iter() {
            match self.atoms_of(v) {
                Ok(mut more) => atoms.append(&mut more),
                Err(reason) => {
                    return Route::Skip(Warning::Unsupported { class, reason });
                }
            }
        }
        match atoms.as_slice() {
            [] => Route::Base,
            [Atom::On(crate::cond::Test::State(s))] => match s.classic() {
                Some(c) => Route::Layer(c),
                None => Route::Rules(Cond(atoms)),
            },
            [Atom::Group(crate::cond::Test::State(s), l)] if s.classic().is_some() => Route::Group(
                format!("{}{}", l.as_deref().unwrap_or(""), state_suffix(*s)),
            ),
            [Atom::Peer(crate::cond::Test::State(s), l)] if s.classic().is_some() => Route::Peer(
                format!("{}{}", l.as_deref().unwrap_or(""), state_suffix(*s)),
            ),
            [Atom::Media(Media::Dark)] => Route::Dark,
            [
                Atom::Viewport {
                    min: Some(px),
                    max: None,
                },
            ] => match self.default_bp(*px) {
                Some(bp) => Route::Responsive(bp),
                None => Route::Rules(Cond(atoms)),
            },
            _ => Route::Rules(Cond(atoms)),
        }
    }

    /// The default breakpoint whose theme px equals `px`, if any.
    fn default_bp(&self, px: f32) -> Option<Breakpoint> {
        const ALL: [Breakpoint; 5] = [
            Breakpoint::Sm,
            Breakpoint::Md,
            Breakpoint::Lg,
            Breakpoint::Xl,
            Breakpoint::Xxl,
        ];
        ALL.into_iter().find(|b| self.theme.breakpoint_px(*b) == px)
    }

    /// Map one syntactic variant to its semantic [`Atom`]s (theme-aware:
    /// `md` → 768px from `--breakpoint-*`, `@md` → 384px from
    /// `--container-*`, `dark` → `Media::Dark` incl. `@custom-variant`
    /// registrations; plan §2.1). Arbitrary selector variants classify
    /// into a CONJUNCTION (`[&:hover:focus]` → two atoms, v0.3 §1.2), so
    /// this returns a list.
    fn atoms_of(&self, v: &Variant<'_>) -> Result<Vec<Atom>, &'static str> {
        match v {
            Variant::Arbitrary(raw) => crate::selector::classify(raw),
            v => Ok(vec![self.atom_of(v)?]),
        }
    }

    /// Map one syntactic variant to ONE semantic [`Atom`] (the
    /// non-arbitrary path of [`Self::atoms_of`]).
    fn atom_of(&self, v: &Variant<'_>) -> Result<Atom, &'static str> {
        Ok(match v {
            Variant::Named(n) => self.named_atom(n)?,
            Variant::Grouped {
                parent,
                inner,
                label,
            } => {
                let test = self.variant_test(inner)?;
                let label = label.map(|l| l.into());
                match parent {
                    ParentKind::Group => Atom::Group(test, label),
                    ParentKind::Peer => Atom::Peer(test, label),
                }
            }
            Variant::Not(inner) => Atom::Not(self.variant_test(inner)?),
            Variant::Has(inner) => return self.has_atom(inner),
            Variant::Attr { kind, name, value } => {
                let boxed = |s: &str| s.into();
                match kind {
                    AttrKind::Data => {
                        Atom::On(crate::cond::Test::Data(boxed(name), value.map(boxed)))
                    }
                    AttrKind::Aria => {
                        Atom::On(crate::cond::Test::Aria(boxed(name), value.map(boxed)))
                    }
                }
            }
            Variant::Structural(s) => Atom::On(crate::cond::Test::Structural(*s)),
            Variant::PseudoElement(p) => {
                return Err(match *p {
                    "::before" => {
                        "pseudo-element ::before has no native equivalent in Rust UI toolkits — reported, never silent (plan §7)"
                    }
                    "::after" => {
                        "pseudo-element ::after has no native equivalent in Rust UI toolkits — reported, never silent (plan §7)"
                    }
                    _ => {
                        "pseudo-elements have no native equivalent in Rust UI toolkits — reported, never silent (plan §7)"
                    }
                });
            }
            Variant::Arbitrary(_) => {
                // Routed through `atoms_of` → `selector::classify` (v0.3
                // §1.2); this arm stays for direct callers and keeps the
                // honest report instead of a panic.
                return Err(
                    "arbitrary/structural selectors ([&…], *, supports-…) have no native equivalent — reported, never silent (plan §7)",
                );
            }
        })
    }

    fn named_atom(&self, n: &NamedVariant<'_>) -> Result<Atom, &'static str> {
        Ok(match n {
            NamedVariant::Custom(name) => return self.custom_atom(name),
            _ if n.state().is_some() => Atom::On(crate::cond::Test::State(n.state().unwrap())),
            _ if n.media().is_some() => Atom::Media(n.media().unwrap()),
            NamedVariant::Breakpoint(b) => Atom::Viewport {
                min: Some(self.theme.breakpoint_px(*b)),
                max: None,
            },
            NamedVariant::Min(b) => Atom::Viewport {
                min: Some(self.theme.breakpoint_px(*b)),
                max: None,
            },
            NamedVariant::Max(b) => Atom::Viewport {
                min: None,
                max: Some(self.theme.breakpoint_px(*b)),
            },
            NamedVariant::Container(b) => Atom::Container {
                min: Some(self.theme.container_breakpoint_px(*b)),
                max: None,
                name: None,
            },
            NamedVariant::MaxContainer(b) => Atom::Container {
                min: None,
                max: Some(self.theme.container_breakpoint_px(*b)),
                name: None,
            },
            _ => unreachable!("named variant without state/media/breakpoint meaning"),
        })
    }

    /// Resolve `Custom` variant names against the theme: `@custom-variant`
    /// registrations first, then custom `--breakpoint-*` / `--container-*`
    /// sizes and the `@min-[400px]` / `@md/name` forms.
    fn custom_atom(&self, name: &str) -> Result<Atom, &'static str> {
        if let Some(m) = self.theme.custom_variant(name) {
            return match m {
                crate::theme::CustomVariant::Dark => Ok(Atom::Media(Media::Dark)),
                crate::theme::CustomVariant::Unsupported(sel) => Err(
                    "custom variant resolves to a CSS selector with no typed equivalent — reported, never silent",
                )
                .map_err(|_| {
                    let _ = sel;
                    "custom variant resolves to a CSS selector with no typed equivalent — reported, never silent"
                }),
            };
        }
        if let Some(rest) = name.strip_prefix('@') {
            // @min-[400px] / @max-[600px] / @4xl / @max-4xl / @md/name
            let (dir, rest) = if let Some(r) = rest.strip_prefix("min-") {
                ("min", r)
            } else if let Some(r) = rest.strip_prefix("max-") {
                ("max", r)
            } else {
                ("min", rest)
            };
            // Named container: @md/name
            let (size, named) = match rest.split_once('/') {
                Some((s, n)) => (s, Some(n)),
                None => (rest, None),
            };
            let px = if let Some(inner) = size.strip_prefix('[').and_then(|r| r.strip_suffix(']')) {
                crate::theme::parse_length(inner)
                    .ok()
                    .or_else(|| self.resolve_var_px(inner))
            } else {
                self.theme.container_px(size)
            };
            let named = named.map(|n| n.into());
            return match (px, dir) {
                (Some(px), "min") => Ok(Atom::Container {
                    min: Some(px),
                    max: None,
                    name: named,
                }),
                (Some(px), "max") => Ok(Atom::Container {
                    min: None,
                    max: Some(px),
                    name: named,
                }),
                _ => Err("unknown @container variant (no --container-* token)"),
            };
        }
        // Custom viewport breakpoints (`--breakpoint-3xl: 120rem`).
        if let Some(px) = self.theme.length(&format!("breakpoint-{name}")) {
            return Ok(Atom::Viewport {
                min: Some(px),
                max: None,
            });
        }
        Err(
            "unknown variant name (not a v4 variant, --breakpoint-*, --container-* or @custom-variant)",
        )
    }

    /// `has-checked` → descendant state; `has-[:focus]` (v0.3 §1.2) →
    /// the same typed descendant state; other selector forms are reported.
    fn has_atom(&self, inner: &str) -> Result<Atom, &'static str> {
        let sel = inner
            .strip_prefix('[')
            .and_then(|s| s.strip_suffix(']'))
            .unwrap_or(inner);
        if let Some(st) = state_name(sel.trim_start_matches(':')) {
            return Ok(Atom::Has(st));
        }
        if inner.starts_with('[') || inner.starts_with(':') || inner.starts_with('(') {
            return Err(
                "has-[selector] has no typed equivalent in the Probe model — reported, never silent (v0.3 §1.3)",
            );
        }
        Err("unknown has-<state> variant")
    }

    /// The probe-testable `Test` of a simple variant (for `not-*` and
    /// `group-*`/`peer-*` inners). Arbitrary selectors classify to a
    /// single self test (v0.3 §1.2).
    fn variant_test(&self, v: &Variant<'_>) -> Result<crate::cond::Test, &'static str> {
        Ok(match v {
            Variant::Named(n) => {
                if let Some(s) = n.state() {
                    crate::cond::Test::State(s)
                } else {
                    return Err(
                        "not-*/group-* over media/breakpoint variants have no typed equivalent",
                    );
                }
            }
            Variant::Arbitrary(raw) => return crate::selector::single_test(raw),
            Variant::Attr { kind, name, value } => {
                let boxed = |s: &str| s.into();
                match kind {
                    AttrKind::Data => crate::cond::Test::Data(boxed(name), value.map(boxed)),
                    AttrKind::Aria => crate::cond::Test::Aria(boxed(name), value.map(boxed)),
                }
            }
            Variant::Structural(s) => crate::cond::Test::Structural(*s),
            _ => {
                return Err("nested selector variants have no typed equivalent");
            }
        })
    }

    /// Resolve `var(--x)` / `(--x)` to a px length via the theme.
    fn resolve_var_px(&self, name: &str) -> Option<f32> {
        let clean = name
            .strip_prefix("var(")
            .and_then(|r| r.strip_suffix(')'))?;
        self.theme.length(clean.trim())
    }
}

/// The canonical state suffix of a legacy group/peer label (`hover`,
/// `menufocus` → "focus").
fn state_suffix(s: State) -> &'static str {
    match s {
        State::Hover => "hover",
        State::Focus => "focus",
        State::FocusVisible => "focus-visible",
        State::FocusWithin => "focus-within",
        State::Active => "active",
        State::Disabled => "disabled",
        State::Visited => "visited",
        State::Target => "target",
        State::Enabled => "enabled",
        State::Checked => "checked",
        State::Indeterminate => "indeterminate",
        State::Required => "required",
        State::Optional => "optional",
        State::Valid => "valid",
        State::Invalid => "invalid",
        State::UserValid => "user-valid",
        State::UserInvalid => "user-invalid",
        State::PlaceholderShown => "placeholder-shown",
        State::Autofill => "autofill",
        State::ReadOnly => "read-only",
        State::InRange => "in-range",
        State::OutOfRange => "out-of-range",
    }
}

/// A bare state name → `State` (for `has-*` inners and the arbitrary
/// selector classifier).
pub(crate) fn state_name(name: &str) -> Option<State> {
    Some(match name {
        "hover" => State::Hover,
        "focus" => State::Focus,
        "focus-visible" => State::FocusVisible,
        "focus-within" => State::FocusWithin,
        "active" => State::Active,
        "disabled" => State::Disabled,
        "visited" => State::Visited,
        "target" => State::Target,
        "enabled" => State::Enabled,
        "checked" => State::Checked,
        "indeterminate" => State::Indeterminate,
        "required" => State::Required,
        "optional" => State::Optional,
        "valid" => State::Valid,
        "invalid" => State::Invalid,
        "user-valid" => State::UserValid,
        "user-invalid" => State::UserInvalid,
        "placeholder-shown" => State::PlaceholderShown,
        "autofill" => State::Autofill,
        "read-only" => State::ReadOnly,
        "in-range" => State::InRange,
        "out-of-range" => State::OutOfRange,
        _ => return None,
    })
}

impl<'t> Resolver<'t> {
    // ------------------------------------------------------------------
    // finalize: applied list -> ResolvedStyle (later entries win per slot)
    // ------------------------------------------------------------------
    fn finalize(&self, bucket: &Bucket) -> ResolvedStyle {
        let mut style = ResolvedStyle::default();
        // Edge families accumulate first so `p-4 px-2` merges correctly.
        let mut pad = Edges::NONE;
        let mut mar = Edges::NONE;
        let mut bw = Edges::NONE;
        let mut rad = Corners::NONE;

        for a in &bucket.applied {
            match a {
                Applied::Padding {
                    top,
                    right,
                    bottom,
                    left,
                } => {
                    pad = Edges {
                        top: top.or(pad.top),
                        right: right.or(pad.right),
                        bottom: bottom.or(pad.bottom),
                        left: left.or(pad.left),
                    };
                }
                Applied::Margin {
                    top,
                    right,
                    bottom,
                    left,
                } => {
                    mar = Edges {
                        top: top.or(mar.top),
                        right: right.or(mar.right),
                        bottom: bottom.or(mar.bottom),
                        left: left.or(mar.left),
                    };
                }
                Applied::BorderWidth {
                    top,
                    right,
                    bottom,
                    left,
                } => {
                    bw = Edges {
                        top: top.or(bw.top),
                        right: right.or(bw.right),
                        bottom: bottom.or(bw.bottom),
                        left: left.or(bw.left),
                    };
                }
                Applied::Radius { tl, tr, br, bl } => {
                    rad = Corners {
                        tl: tl.or(rad.tl),
                        tr: tr.or(rad.tr),
                        br: br.or(rad.br),
                        bl: bl.or(rad.bl),
                    };
                }
                _ => {}
            }
        }
        if pad != Edges::NONE {
            style.padding = Some(Padding {
                top: pad.top.unwrap_or(0.0),
                right: pad.right.unwrap_or(0.0),
                bottom: pad.bottom.unwrap_or(0.0),
                left: pad.left.unwrap_or(0.0),
            });
        }
        if mar != Edges::NONE {
            style.margin = Some(Margin {
                top: mar.top.unwrap_or(0.0),
                right: mar.right.unwrap_or(0.0),
                bottom: mar.bottom.unwrap_or(0.0),
                left: mar.left.unwrap_or(0.0),
            });
        }
        if bw != Edges::NONE {
            style.border_width = Some(BorderWidth {
                top: bw.top.unwrap_or(0.0),
                right: bw.right.unwrap_or(0.0),
                bottom: bw.bottom.unwrap_or(0.0),
                left: bw.left.unwrap_or(0.0),
            });
        }
        if rad != Corners::NONE {
            style.corner_radius = Some(CornerRadius {
                top_left: rad.tl.unwrap_or(0.0),
                top_right: rad.tr.unwrap_or(0.0),
                bottom_right: rad.br.unwrap_or(0.0),
                bottom_left: rad.bl.unwrap_or(0.0),
            });
        }

        // Layout and Dimensions accumulate per-field (`h-9 w-full`,
        // `inline-flex items-center justify-center`).
        let mut dims = Dimensions::default();
        let mut min_dims = Dimensions::default();
        let mut max_dims = Dimensions::default();
        let mut layout = Layout::default();
        let mut dims_set = false;
        let mut min_set = false;
        let mut max_set = false;
        let mut layout_set = false;
        for a in &bucket.applied {
            match a {
                Applied::Replace(Slot::Dimensions, PartialValue::Dimensions(d)) => {
                    if d.width.is_some() {
                        dims.width = d.width;
                    }
                    if d.height.is_some() {
                        dims.height = d.height;
                    }
                    dims_set = true;
                }
                Applied::Replace(Slot::MinDimensions, PartialValue::MinDimensions(d)) => {
                    if d.width.is_some() {
                        min_dims.width = d.width;
                    }
                    if d.height.is_some() {
                        min_dims.height = d.height;
                    }
                    min_set = true;
                }
                Applied::Replace(Slot::MaxDimensions, PartialValue::MaxDimensions(d)) => {
                    if d.width.is_some() {
                        max_dims.width = d.width;
                    }
                    if d.height.is_some() {
                        max_dims.height = d.height;
                    }
                    max_set = true;
                }
                Applied::Replace(Slot::Layout, PartialValue::Layout(l)) => {
                    merge_layout(&mut layout, l);
                    layout_set = true;
                }
                _ => {}
            }
        }
        if dims_set {
            style.dimensions = Some(dims);
        }
        if min_set {
            style.min_dimensions = Some(min_dims);
        }
        if max_set {
            style.max_dimensions = Some(max_dims);
        }
        if layout_set {
            style.layout = Some(layout);
        }

        // Transition accumulates its four sub-fields (transition-* +
        // duration-* + ease-* + delay-* land in one slot).
        let mut trans_set: Option<TransitionSet> = None;
        let mut trans_dur: Option<f32> = None;
        let mut trans_ease: Option<Ease> = None;
        let mut trans_delay: Option<f32> = None;
        let mut transform = Transform::default();
        let mut transform_set = false;
        let mut grid = Grid::default();
        let mut grid_set = false;
        let mut inset_acc = Inset::default();
        let mut inset_set = false;
        let mut gap_axes = GapAxes::default();
        let mut gap_axes_set = false;

        for a in &bucket.applied {
            match a {
                Applied::Transition {
                    set,
                    duration_ms,
                    ease,
                    delay_ms,
                } => {
                    trans_set = set.or(trans_set);
                    trans_dur = duration_ms.or(trans_dur);
                    trans_ease = ease.or(trans_ease);
                    trans_delay = delay_ms.or(trans_delay);
                }
                Applied::Transform {
                    scale_x,
                    scale_y,
                    rotate_deg,
                    translate_x,
                    translate_y,
                    skew_x,
                    skew_y,
                } => {
                    if scale_x.is_some() {
                        transform.scale_x = *scale_x;
                    }
                    if scale_y.is_some() {
                        transform.scale_y = *scale_y;
                    }
                    if rotate_deg.is_some() {
                        transform.rotate_deg = *rotate_deg;
                    }
                    if translate_x.is_some() {
                        transform.translate_x = *translate_x;
                    }
                    if translate_y.is_some() {
                        transform.translate_y = *translate_y;
                    }
                    if skew_x.is_some() {
                        transform.skew_x = *skew_x;
                    }
                    if skew_y.is_some() {
                        transform.skew_y = *skew_y;
                    }
                    transform_set = true;
                }
                Applied::Grid(g) => {
                    if g.cols.is_some() {
                        grid.cols = g.cols;
                    }
                    if g.rows.is_some() {
                        grid.rows = g.rows;
                    }
                    if g.col_span.is_some() {
                        grid.col_span = g.col_span;
                    }
                    if g.row_span.is_some() {
                        grid.row_span = g.row_span;
                    }
                    if g.col_start.is_some() {
                        grid.col_start = g.col_start;
                    }
                    if g.row_start.is_some() {
                        grid.row_start = g.row_start;
                    }
                    grid_set = true;
                }
                Applied::Inset {
                    top,
                    right,
                    bottom,
                    left,
                } => {
                    if top.is_some() {
                        inset_acc.top = *top;
                    }
                    if right.is_some() {
                        inset_acc.right = *right;
                    }
                    if bottom.is_some() {
                        inset_acc.bottom = *bottom;
                    }
                    if left.is_some() {
                        inset_acc.left = *left;
                    }
                    inset_set = true;
                }
                Applied::GapAxes(g) => {
                    if g.col.is_some() {
                        gap_axes.col = g.col;
                    }
                    if g.row.is_some() {
                        gap_axes.row = g.row;
                    }
                    gap_axes_set = true;
                }
                _ => {}
            }
        }
        if trans_set.is_some()
            || trans_dur.is_some()
            || trans_ease.is_some()
            || trans_delay.is_some()
        {
            style.extras.transition = Some(Transition {
                set: trans_set.unwrap_or(TransitionSet::Default),
                duration_ms: trans_dur.unwrap_or_else(|| self.theme.default_transition_ms()),
                ease: trans_ease,
                delay_ms: trans_delay.unwrap_or(0.0),
            });
        }
        if transform_set {
            style.extras.transform = Some(transform);
        }
        if grid_set {
            style.extras.grid = Some(grid);
        }
        if inset_set {
            style.extras.inset = Some(inset_acc);
        }
        if gap_axes_set {
            style.extras.gap_axes = Some(gap_axes);
        }

        for a in &bucket.applied {
            if let Applied::Replace(slot, val) = a {
                // Layout/Dimensions/Min/Max were already accumulated above.
                if matches!(
                    slot,
                    Slot::Layout | Slot::Dimensions | Slot::MinDimensions | Slot::MaxDimensions
                ) {
                    continue;
                }
                match (slot, val) {
                    (Slot::Background, PartialValue::Background(v)) => style.background = Some(*v),
                    (Slot::ContentColor, PartialValue::ContentColor(v)) => {
                        style.content_color = Some(*v)
                    }
                    (Slot::BorderColor, PartialValue::BorderColor(v)) => {
                        style.border_color = Some(*v)
                    }
                    (Slot::Gap, PartialValue::Gap(v)) => style.gap = Some(*v),
                    (Slot::Shadow, PartialValue::Shadow(v)) => style.shadow = Some(v.clone()),
                    (Slot::Opacity, PartialValue::Opacity(v)) => style.opacity = Some(*v),
                    (Slot::Ring(_), PartialValue::Ring(v)) => merge_ring(&mut style, v),
                    (Slot::Outline, PartialValue::Outline(v)) => merge_outline(&mut style, v),
                    (Slot::Cursor, PartialValue::Cursor(v)) => style.cursor = Some(*v),
                    (Slot::PointerEvents, PartialValue::PointerEvents(v)) => {
                        style.pointer_events = Some(*v)
                    }
                    _ => {}
                }
            } else if let Applied::Text(p, v) = a {
                if let Some(slot) = style.text.iter_mut().find(|(q, _)| q == p) {
                    slot.1 = *v;
                } else {
                    style.text.push((*p, *v));
                }
            } else if let Applied::Texts(items) = a {
                for (p, v) in items {
                    if let Some(slot) = style.text.iter_mut().find(|(q, _)| q == p) {
                        slot.1 = *v;
                    } else {
                        style.text.push((*p, *v));
                    }
                }
            }
        }

        // Shadow color patches the preset's layers (shadow-{color}).
        for a in &bucket.applied {
            if let Applied::ShadowColor(c) = a {
                if let Some(sh) = style.shadow.as_mut() {
                    sh.color = *c;
                }
                for layer in style.extras.shadow_rest.iter_mut() {
                    layer.color = *c;
                }
            }
        }

        // v0.2 scalar extras (last wins).
        for a in &bucket.applied {
            match a {
                Applied::Position(p) => style.extras.position = Some(*p),
                Applied::ZIndex(z) => style.extras.z_index = Some(*z),
                Applied::Overflow(o) => style.extras.overflow = Some(*o),
                Applied::Visibility(v) => style.extras.visibility = Some(*v),
                Applied::AspectRatio(r) => style.extras.aspect_ratio = Some(*r),
                Applied::BorderStyle(b) => style.extras.border_style = Some(*b),
                Applied::FlexBasis(b) => style.extras.flex_basis = Some(*b),
                Applied::Order(o) => style.extras.order = Some(*o),
                Applied::AlignContent(c) => style.extras.align_content = Some(*c),
                Applied::MarginAuto(m) => style.extras.margin_auto = Some(*m),
                Applied::Animation(anim) => style.extras.animation = Some(anim.clone()),
                Applied::ShadowPreset { first, rest } => {
                    style.shadow = Some(first.clone());
                    style.extras.shadow_rest = rest.clone();
                }
                Applied::InsetShadows(sh) => {
                    style.extras.inset_shadows = sh.clone();
                }
                _ => {}
            }
        }
        style
    }
}

impl<'t> Resolver<'t> {
    // ------------------------------------------------------------------
    // utility table
    // ------------------------------------------------------------------
    fn apply_utility(&self, t: &ClassToken<'_>) -> Result<Applied, Warning> {
        let class = t.raw.to_string();
        let unknown = |suggestion: Option<String>| Warning::Unknown {
            class: class.clone(),
            suggestion,
        };
        let unsupported = |reason: &'static str| Warning::Unsupported {
            class: class.clone(),
            reason,
        };
        let missing = |token: String| Warning::MissingThemeToken {
            class: class.clone(),
            token,
        };

        let els: &[&str] = &t.elements;
        // Two-word utility heads — detected by slicing `t.body`, never by
        // `join("-")` (hot path, plan §3.4.1).
        let two_word = els.len() >= 2
            && match els[0] {
                "min" | "max" => els[1] == "w" || els[1] == "h",
                "ring" => els[1] == "offset",
                "underline" => els[1] == "offset",
                "pointer" => els[1] == "events",
                "overflow" => els[1] == "x" || els[1] == "y",
                "no" => els[1] == "underline",
                "text" => els[1] == "wrap" || els[1] == "ellipsis" || els[1] == "clip",
                "inset" => matches!(els[1], "x" | "y" | "shadow"),
                "auto" => els[1] == "rows",
                "scale" | "translate" | "skew" => els[1] == "x" || els[1] == "y",
                "line" => els[1] == "clamp" || els[1] == "through",
                "not" => els[1] == "italic",
                "normal" => els[1] == "case",
                "tabular" => els[1] == "nums",
                "drop" => els[1] == "shadow",
                _ => false,
            };
        let head_n = if two_word { 2 } else { 1 };
        let head: &str = if two_word {
            &t.body[..els[0].len() + 1 + els[1].len()]
        } else {
            els[0]
        };
        let value_elems: &[&str] = if els.len() > head_n {
            &els[head_n..]
        } else {
            &[]
        };
        // The value as a raw body slice (no allocation): `primary` for
        // `bg-primary`, `1.5` for `gap-1.5`.
        let value: &str = if value_elems.is_empty() {
            ""
        } else {
            &t.body[head.len() + 1..]
        };
        let spacing = self.theme.spacing();

        // Numeric scale value (n -> n * 4px), honoring the negative sign.
        // (Ring/outline/underline-offset widths are plain px in v4, so only
        // the spacing families use the scale.)
        let scale = |n: f32| n * spacing * if t.negative { -1.0 } else { 1.0 };

        // F1: the modifier is the denominator for sizing utilities
        // (`w-1/2` → 0.5; `translate-x-1/2`, `basis-1/3`, `top-1/2` reuse).
        let fraction = || -> Option<f32> {
            let num: f32 = value_elems.last()?.parse().ok()?;
            let den: f32 = t.modifier?.parse().ok()?;
            (den != 0.0).then(|| num / den)
        };

        // Color spec resolver: arbitrary `[...]`/`(--var)` wins, else theme.
        let color_of = |name: &str| -> Result<Option<Color>, Warning> {
            if let Some(arb) = t.arbitrary {
                return color_from_arbitrary(arb, self.theme)
                    .map(Some)
                    .ok_or_else(|| {
                        Warning::Unsupported {
                            class: class.clone(),
                            reason: "arbitrary value is not a parsable color (gradients/images not modeled)",
                        }
                    });
            }
            Ok(self.theme.color(name))
        };
        let opacity = || -> Result<Option<f32>, Warning> {
            match t.opacity_pct(self.theme) {
                // No `/modifier` at all.
                None => Ok(None),
                // Modifier present but unresolvable (bad number / missing var).
                Some(None) => Err(unsupported(
                    "opacity modifier does not resolve (bad number or missing theme var)",
                )),
                Some(Some(v)) => Ok(Some(v)),
            }
        };
        let with_opacity = |c: Color, pct: Option<f32>| match pct {
            Some(p) => c.mix_with_transparent(p),
            None => c,
        };
        let pct = opacity()?;

        // F2: arbitrary lengths are absolute — NOT spacing multiples.
        let arb_size = |a: Arbitrary<'_>| -> Result<SizedValue, Warning> {
            sized_from_arbitrary(a, self.theme, &class).ok_or_else(|| unknown(None))
        };

        match head {
            // ---------------- colors ----------------
            "bg" => {
                if let Some(arb) = t.arbitrary {
                    let c = color_from_arbitrary(arb, self.theme).ok_or_else(|| {
                        unsupported("bg-[...] value is not a color (gradients/images not modeled)")
                    })?;
                    return Ok(Applied::Replace(
                        Slot::Background,
                        PartialValue::Background(Background(with_opacity(c, pct))),
                    ));
                }
                match color_of(value)? {
                    Some(c) => Ok(Applied::Replace(
                        Slot::Background,
                        PartialValue::Background(Background(with_opacity(c, pct))),
                    )),
                    None if value == "transparent" => Ok(Applied::Replace(
                        Slot::Background,
                        PartialValue::Background(Background(Color::TRANSPARENT)),
                    )),
                    None => Err(unknown(suggest(value, self.theme))),
                }
            }
            "text" if t.arbitrary.is_none() && is_size_name(value_elems) => {
                let name = value;
                match self.theme.font_size(name) {
                    Some((size, lh)) => {
                        let mut items = vec![(TextProperty::FontSize, TextValue::Px(size))];
                        // F6: `text-sm/6` — the modifier is a line-height
                        // scale multiple; `text-sm/tight` a leading token.
                        if let Some(m) = t.modifier {
                            items.push((
                                TextProperty::LineHeight,
                                TextValue::Px(
                                    leading_from_modifier(m, spacing, self.theme)
                                        .ok_or_else(|| unknown(None))?,
                                ),
                            ));
                        } else if let Some(lh) = lh {
                            items.push((TextProperty::LineHeight, TextValue::Px(lh)));
                        }
                        Ok(Applied::Texts(items))
                    }
                    None => Err(unsupported("unknown text size token")),
                }
            }
            "text"
                if matches!(
                    value,
                    "left" | "center" | "right" | "justify" | "start" | "end"
                ) && t.arbitrary.is_none() =>
            {
                let kw = match value {
                    "left" | "start" => "left",
                    "center" => "center",
                    "right" | "end" => "right",
                    "justify" => "justify",
                    _ => unreachable!(),
                };
                Ok(Applied::Text(
                    TextProperty::TextAlign,
                    TextValue::Keyword(kw),
                ))
            }
            "text" => {
                // color (or arbitrary size/color disambiguation)
                if let Some(arb) = t.arbitrary {
                    if let Some(c) = color_from_arbitrary(arb, self.theme) {
                        return Ok(Applied::Replace(
                            Slot::ContentColor,
                            PartialValue::ContentColor(ContentColor(with_opacity(c, pct))),
                        ));
                    }
                    if let Some(v) = sized_from_arbitrary(arb, self.theme, &class)
                        && let SizedValue::Px(px) = v
                    {
                        return Ok(Applied::Text(TextProperty::FontSize, TextValue::Px(px)));
                    }
                    return Err(unsupported("text-[...] value is neither color nor length"));
                }
                match color_of(value)? {
                    Some(c) => Ok(Applied::Replace(
                        Slot::ContentColor,
                        PartialValue::ContentColor(ContentColor(with_opacity(c, pct))),
                    )),
                    None => Err(unknown(suggest(value, self.theme))),
                }
            }
            "border" => {
                // `border` | `border-2` | `border-t` | `border-{color}` |
                // `border-{style}` | logical sides `border-s/e` | arbitrary
                if let Some(arb) = t.arbitrary {
                    if let Some(c) = color_from_arbitrary(arb, self.theme) {
                        return Ok(Applied::Replace(
                            Slot::BorderColor,
                            PartialValue::BorderColor(BorderColor(with_opacity(c, pct))),
                        ));
                    }
                    if let Some(v) = sized_from_arbitrary(arb, self.theme, &class)
                        && let SizedValue::Px(px) = v
                    {
                        return Ok(Applied::BorderWidth {
                            top: Some(px),
                            right: Some(px),
                            bottom: Some(px),
                            left: Some(px),
                        });
                    }
                    return Err(unsupported("border-[...] value is neither color nor width"));
                }
                if value_elems.is_empty() {
                    return Ok(Applied::BorderWidth {
                        top: Some(1.0),
                        right: Some(1.0),
                        bottom: Some(1.0),
                        left: Some(1.0),
                    });
                }
                // v0.2: `border-{solid,dashed,dotted,double,none,hidden}`.
                if let Some(style) = border_style_of(value) {
                    return Ok(Applied::BorderStyle(style));
                }
                // width edges first: border-t / border-x / border-s / border-2
                if let Some(applied) = border_width_family(value, t.negative) {
                    return Ok(applied);
                }
                match color_of(value)? {
                    Some(c) => Ok(Applied::Replace(
                        Slot::BorderColor,
                        PartialValue::BorderColor(BorderColor(with_opacity(c, pct))),
                    )),
                    None if value == "transparent" => Ok(Applied::Replace(
                        Slot::BorderColor,
                        PartialValue::BorderColor(BorderColor(Color::TRANSPARENT)),
                    )),
                    None => Err(unknown(suggest(value, self.theme))),
                }
            }
            "ring" => {
                let mut ring = Ring::default();
                if let Some(arb) = t.arbitrary {
                    let w = ring_width_from_arbitrary(arb)
                        .ok_or_else(|| unsupported("ring-[...] expects a width"))?;
                    ring.width = w;
                    return Ok(Applied::Replace(
                        Slot::Ring(RingPart::Width),
                        PartialValue::Ring(ring),
                    ));
                }
                if value_elems.is_empty() {
                    // v4.1.17: bare `ring` → `--default-ring-width` fallback
                    // '1px' (verified against the tailwindcss source — NOT a
                    // spacing multiple).
                    ring.width = 1.0;
                    return Ok(Applied::Replace(
                        Slot::Ring(RingPart::Width),
                        PartialValue::Ring(ring),
                    ));
                }
                if value == "inset" {
                    // v0.2: ring-inset draws the ring inside the border box.
                    ring.inset = true;
                    return Ok(Applied::Replace(
                        Slot::Ring(RingPart::Width),
                        PartialValue::Ring(ring),
                    ));
                }
                if let Ok(px) = value.parse::<f32>() {
                    // v4.1.17: `ring-<number>` → `<number>px` (plain px, not
                    // `--spacing` multiples).
                    ring.width = px;
                    return Ok(Applied::Replace(
                        Slot::Ring(RingPart::Width),
                        PartialValue::Ring(ring),
                    ));
                }
                match color_of(value)? {
                    Some(c) => {
                        ring.color = Some(with_opacity(c, pct));
                        Ok(Applied::Replace(
                            Slot::Ring(RingPart::Color),
                            PartialValue::Ring(ring),
                        ))
                    }
                    None => Err(unknown(suggest(value, self.theme))),
                }
            }
            "ring-offset" => {
                let mut ring = Ring::default();
                if value_elems.is_empty() {
                    // Legacy default offset (v3: `ring-offset` = 2px); v4
                    // requires an explicit number.
                    ring.offset_width = 2.0;
                    return Ok(Applied::Replace(
                        Slot::Ring(RingPart::OffsetWidth),
                        PartialValue::Ring(ring),
                    ));
                }
                if let Ok(px) = value.parse::<f32>() {
                    // v4.1.17: `ring-offset-<number>` → `<number>px`.
                    ring.offset_width = px;
                    return Ok(Applied::Replace(
                        Slot::Ring(RingPart::OffsetWidth),
                        PartialValue::Ring(ring),
                    ));
                }
                match color_of(value)? {
                    Some(c) => {
                        ring.offset_color = Some(c);
                        Ok(Applied::Replace(
                            Slot::Ring(RingPart::OffsetColor),
                            PartialValue::Ring(ring),
                        ))
                    }
                    None => Err(unknown(suggest(value, self.theme))),
                }
            }
            "outline" => {
                if value_elems.is_empty() {
                    // v4.1.17: bare `outline` → `--default-outline-width`
                    // fallback '1px' (verified against the tailwindcss
                    // source; v3's 2px default does not apply to v4).
                    return Ok(Applied::Replace(
                        Slot::Outline,
                        PartialValue::Outline(Outline {
                            width: 1.0,
                            color: None,
                            none: false,
                            offset: 0.0,
                        }),
                    ));
                }
                if let Some(off) = value.strip_prefix("offset-") {
                    // v0.2: outline-offset-* — v4.1.17 semantics:
                    // `outline-offset-<number>` → `<number>px` (plain px,
                    // NOT spacing multiples), negatives supported.
                    let px = off
                        .parse::<f32>()
                        .map(|n| n * if t.negative { -1.0 } else { 1.0 })
                        .ok()
                        .or_else(|| t.arbitrary.and_then(ring_width_from_arbitrary))
                        .ok_or_else(|| unknown(None))?;
                    return Ok(Applied::Replace(
                        Slot::Outline,
                        PartialValue::Outline(Outline {
                            width: 0.0,
                            color: None,
                            none: false,
                            offset: px,
                        }),
                    ));
                }
                match value {
                    "none" | "hidden" => Ok(Applied::Replace(
                        Slot::Outline,
                        PartialValue::Outline(Outline {
                            width: 0.0,
                            color: None,
                            none: true,
                            offset: 0.0,
                        }),
                    )),
                    other => {
                        if let Ok(px) = other.parse::<f32>() {
                            Ok(Applied::Replace(
                                Slot::Outline,
                                PartialValue::Outline(Outline {
                                    width: px,
                                    color: None,
                                    none: false,
                                    offset: 0.0,
                                }),
                            ))
                        } else {
                            match color_of(other)? {
                                Some(c) => Ok(Applied::Replace(
                                    Slot::Outline,
                                    PartialValue::Outline(Outline {
                                        width: 0.0,
                                        color: Some(c),
                                        none: false,
                                        offset: 0.0,
                                    }),
                                )),
                                None => Err(unknown(suggest(other, self.theme))),
                            }
                        }
                    }
                }
            }

            // ---------------- spacing ----------------
            "p" | "px" | "py" | "pt" | "pr" | "pb" | "pl" | "ps" | "pe" => {
                let v = spacing_value(t, value, value_elems, spacing, scale, &unknown, self.theme)?;
                Ok(padding_applied(head, v))
            }
            "m" | "mx" | "my" | "mt" | "mr" | "mb" | "ml" | "ms" | "me" => {
                // v0.2: mx-auto / m-auto / ml-auto ... (plan §3.2).
                if value == "auto" {
                    let mut e = AutoEdges::default();
                    match head {
                        "m" => {
                            e = AutoEdges {
                                left: true,
                                right: true,
                                top: true,
                                bottom: true,
                            }
                        }
                        "mx" => {
                            e.left = true;
                            e.right = true;
                        }
                        "my" => {
                            e.top = true;
                            e.bottom = true;
                        }
                        "mt" => e.top = true,
                        "mr" | "me" => e.right = true,
                        "mb" => e.bottom = true,
                        "ml" | "ms" => e.left = true,
                        _ => {}
                    }
                    return Ok(Applied::MarginAuto(e));
                }
                let v = spacing_value(t, value, value_elems, spacing, scale, &unknown, self.theme)?;
                Ok(margin_applied(head, v))
            }
            "gap" => {
                // v0.2: per-axis gap (gap-x-*/gap-y-*), plan §3.2.
                if let Some(axis) = value_elems.first().copied()
                    && (axis == "x" || axis == "y")
                {
                    let rest: &[&str] = &value_elems[1..];
                    let v = spacing_value(
                        t,
                        value
                            .strip_prefix(axis)
                            .and_then(|r| r.strip_prefix('-'))
                            .unwrap_or(value),
                        rest,
                        spacing,
                        scale,
                        &unknown,
                        self.theme,
                    )?;
                    let mut g = GapAxes::default();
                    if axis == "x" {
                        g.col = Some(v);
                    } else {
                        g.row = Some(v);
                    }
                    return Ok(Applied::GapAxes(g));
                }
                let v = spacing_value(t, value, value_elems, spacing, scale, &unknown, self.theme)?;
                Ok(Applied::Replace(Slot::Gap, PartialValue::Gap(Gap(v))))
            }

            // ---------------- sizing ----------------
            "w" | "h" | "size" | "min-w" | "min-h" | "max-w" | "max-h" => {
                let dim = size_value(
                    t,
                    value,
                    value_elems,
                    fraction(),
                    &arb_size,
                    spacing,
                    self.theme,
                    &unknown,
                )?;
                let mut d = Dimensions::default();
                match head {
                    "w" | "min-w" | "max-w" => d.width = Some(dim),
                    "h" | "min-h" | "max-h" => d.height = Some(dim),
                    "size" => {
                        d.width = Some(dim);
                        d.height = Some(dim);
                    }
                    _ => unreachable!(),
                }
                let slot = match head {
                    "min-w" | "min-h" => Slot::MinDimensions,
                    "max-w" | "max-h" => Slot::MaxDimensions,
                    _ => Slot::Dimensions,
                };
                Ok(Applied::Replace(
                    slot,
                    match slot {
                        Slot::MinDimensions => PartialValue::MinDimensions(d),
                        Slot::MaxDimensions => PartialValue::MaxDimensions(d),
                        _ => PartialValue::Dimensions(d),
                    },
                ))
            }

            // ---------------- radius ----------------
            "rounded" => {
                let full = t
                    .arbitrary
                    .and_then(|a| sized_from_arbitrary(a, self.theme, &class))
                    .and_then(|v| match v {
                        SizedValue::Px(v) => Some(v),
                        _ => None,
                    });
                // F5: logical start/end edges (`rounded-s-lg` = left side in
                // LTR; flip under RTL at lowering time, plan §3.2).
                let (edges, token) = radius_family(value);
                let px = match full {
                    Some(v) => v,
                    _ => match token {
                        Some(tok) => self
                            .theme
                            .radius(tok)
                            .unwrap_or_else(|| default_radius(tok, spacing)),
                        None => self.theme.radius("DEFAULT").unwrap_or(4.0),
                    },
                };
                if value == "full" {
                    return Ok(Applied::Radius {
                        tl: Some(9999.0),
                        tr: Some(9999.0),
                        br: Some(9999.0),
                        bl: Some(9999.0),
                    });
                }
                Ok(match edges {
                    RadiusEdges::All => Applied::Radius {
                        tl: Some(px),
                        tr: Some(px),
                        br: Some(px),
                        bl: Some(px),
                    },
                    RadiusEdges::Top => Applied::Radius {
                        tl: Some(px),
                        tr: Some(px),
                        br: None,
                        bl: None,
                    },
                    RadiusEdges::Bottom => Applied::Radius {
                        tl: None,
                        tr: None,
                        br: Some(px),
                        bl: Some(px),
                    },
                    RadiusEdges::Left | RadiusEdges::S => Applied::Radius {
                        tl: Some(px),
                        tr: None,
                        br: None,
                        bl: Some(px),
                    },
                    RadiusEdges::Right | RadiusEdges::E => Applied::Radius {
                        tl: None,
                        tr: Some(px),
                        br: Some(px),
                        bl: None,
                    },
                    RadiusEdges::Ss => Applied::Radius {
                        tl: Some(px),
                        tr: None,
                        br: None,
                        bl: None,
                    },
                    RadiusEdges::Se => Applied::Radius {
                        tl: None,
                        tr: Some(px),
                        br: None,
                        bl: None,
                    },
                    RadiusEdges::Ee => Applied::Radius {
                        tl: None,
                        tr: None,
                        br: Some(px),
                        bl: None,
                    },
                    RadiusEdges::Es => Applied::Radius {
                        tl: None,
                        tr: None,
                        br: None,
                        bl: Some(px),
                    },
                    RadiusEdges::Tl => Applied::Radius {
                        tl: Some(px),
                        tr: None,
                        br: None,
                        bl: None,
                    },
                    RadiusEdges::Tr => Applied::Radius {
                        tl: None,
                        tr: Some(px),
                        br: None,
                        bl: None,
                    },
                    RadiusEdges::Br => Applied::Radius {
                        tl: None,
                        tr: None,
                        br: Some(px),
                        bl: None,
                    },
                    RadiusEdges::Bl => Applied::Radius {
                        tl: None,
                        tr: None,
                        br: None,
                        bl: Some(px),
                    },
                })
            }

            // ---------------- shadows ----------------
            "shadow" => {
                if value_elems.is_empty() {
                    let mut layers = self.theme.shadows("");
                    if layers.is_empty() {
                        layers = vec![v4_default_shadow("")];
                    }
                    let first = layers.remove(0);
                    let rest = layers;
                    return Ok(shadow_applied(first, rest));
                }
                if value == "none" {
                    return Ok(Applied::Replace(
                        Slot::Shadow,
                        PartialValue::Shadow(BoxShadow {
                            offset_x: 0.0,
                            offset_y: 0.0,
                            blur: 0.0,
                            spread: 0.0,
                            color: Color::TRANSPARENT,
                            inset: false,
                        }),
                    ));
                }
                // v0.2: multi-layer presets — the first layer lands in
                // `shadow`, the rest in `extras.shadow_rest` (plan §3.3).
                let layers = self.theme.shadows(value);
                if !layers.is_empty() {
                    let mut layers = layers;
                    let first = layers.remove(0);
                    return Ok(shadow_applied(first, layers));
                }
                // `shadow-{color}` recolors the preset in the same bucket.
                if let Some(c) = self.theme.color(value) {
                    let c = with_opacity(c, pct);
                    return Ok(Applied::ShadowColor(c));
                }
                if let Some(arb) = t.arbitrary
                    && let Some(c) = color_from_arbitrary(arb, self.theme)
                {
                    return Ok(Applied::ShadowColor(c));
                }
                Err(missing(format!("--shadow-{value}")))
            }
            "inset-shadow" => {
                // v0.2: inset-shadow-* presets.
                let layers = if value_elems.is_empty() {
                    self.theme.inset_shadows("")
                } else {
                    self.theme.inset_shadows(value)
                };
                if layers.is_empty() {
                    return Err(missing(format!("--inset-shadow-{value}")));
                }
                Ok(Applied::InsetShadows(
                    layers.iter().map(to_shadow).collect(),
                ))
            }

            // ---------------- layout ----------------
            "flex" => {
                let mut layout = Layout {
                    display: Some(Display::Flex),
                    ..Default::default()
                };
                match value_elems {
                    [] => Ok(Applied::Replace(Slot::Layout, PartialValue::Layout(layout))),
                    ["col"] => {
                        layout.flex_direction = Some(FlexDirection::Column);
                        Ok(Applied::Replace(Slot::Layout, PartialValue::Layout(layout)))
                    }
                    ["row"] => {
                        layout.flex_direction = Some(FlexDirection::Row);
                        Ok(Applied::Replace(Slot::Layout, PartialValue::Layout(layout)))
                    }
                    ["col-reverse"] => {
                        layout.flex_direction = Some(FlexDirection::ColumnReverse);
                        Ok(Applied::Replace(Slot::Layout, PartialValue::Layout(layout)))
                    }
                    ["row-reverse"] => {
                        layout.flex_direction = Some(FlexDirection::RowReverse);
                        Ok(Applied::Replace(Slot::Layout, PartialValue::Layout(layout)))
                    }
                    ["wrap"] => {
                        layout.flex_wrap = Some(true);
                        Ok(Applied::Replace(Slot::Layout, PartialValue::Layout(layout)))
                    }
                    ["nowrap"] => {
                        layout.flex_wrap = Some(false);
                        Ok(Applied::Replace(Slot::Layout, PartialValue::Layout(layout)))
                    }
                    ["1"] => {
                        layout.flex_grow = Some(1.0);
                        layout.flex_shrink = Some(1.0);
                        Ok(Applied::Replace(Slot::Layout, PartialValue::Layout(layout)))
                    }
                    ["auto"] | ["initial"] | ["none"] => Err(unsupported(
                        "flex-auto/flex-initial/flex-none shorthand flex items are outside the v0.2 table",
                    )),
                    _ => Err(unknown(suggest(head, self.theme))),
                }
            }
            "inline" if value_elems == ["flex"] => Ok(Applied::Replace(
                Slot::Layout,
                PartialValue::Layout(Layout {
                    display: Some(Display::InlineFlex),
                    ..Default::default()
                }),
            )),
            "grid" if value_elems.is_empty() => Ok(Applied::Replace(
                Slot::Layout,
                PartialValue::Layout(Layout {
                    display: Some(Display::Grid),
                    ..Default::default()
                }),
            )),
            "block" if value_elems.is_empty() => Ok(Applied::Replace(
                Slot::Layout,
                PartialValue::Layout(Layout {
                    display: Some(Display::Block),
                    ..Default::default()
                }),
            )),
            "hidden" if value_elems.is_empty() => Ok(Applied::Replace(
                Slot::Layout,
                PartialValue::Layout(Layout {
                    display: Some(Display::None),
                    ..Default::default()
                }),
            )),
            "items" => {
                let v = match value_elems {
                    ["center"] => AlignItems::Center,
                    ["start"] => AlignItems::Start,
                    ["end"] => AlignItems::End,
                    ["stretch"] => AlignItems::Stretch,
                    ["baseline"] => AlignItems::Baseline,
                    _ => return Err(unknown(suggest(head, self.theme))),
                };
                Ok(Applied::Replace(
                    Slot::Layout,
                    PartialValue::Layout(Layout {
                        align_items: Some(v),
                        ..Default::default()
                    }),
                ))
            }
            "justify" => {
                let v = match value_elems {
                    ["center"] => JustifyContent::Center,
                    ["start"] => JustifyContent::Start,
                    ["end"] => JustifyContent::End,
                    ["between"] => JustifyContent::SpaceBetween,
                    ["around"] => JustifyContent::SpaceAround,
                    ["evenly"] => JustifyContent::SpaceEvenly,
                    _ => return Err(unknown(suggest(head, self.theme))),
                };
                Ok(Applied::Replace(
                    Slot::Layout,
                    PartialValue::Layout(Layout {
                        justify_content: Some(v),
                        ..Default::default()
                    }),
                ))
            }
            "self" => {
                let v = match value_elems {
                    ["center"] => AlignItems::Center,
                    ["start"] => AlignItems::Start,
                    ["end"] => AlignItems::End,
                    ["stretch"] => AlignItems::Stretch,
                    _ => return Err(unknown(suggest(head, self.theme))),
                };
                Ok(Applied::Replace(
                    Slot::Layout,
                    PartialValue::Layout(Layout {
                        align_self: Some(v),
                        ..Default::default()
                    }),
                ))
            }
            "grow" | "shrink" => {
                let n = if value_elems.is_empty() {
                    1.0
                } else {
                    value.parse::<f32>().map_err(|_| unknown(None))?
                };
                let mut layout = Layout::default();
                if head == "grow" {
                    layout.flex_grow = Some(n);
                } else {
                    layout.flex_shrink = Some(n);
                }
                Ok(Applied::Replace(Slot::Layout, PartialValue::Layout(layout)))
            }

            // ---- v0.2 layout additions (plan §3.2) ----
            "basis" => {
                let v = size_value(
                    t,
                    value,
                    value_elems,
                    fraction(),
                    &arb_size,
                    spacing,
                    self.theme,
                    &unknown,
                )?;
                Ok(Applied::FlexBasis(v))
            }
            "order" => {
                let n: i16 = match value {
                    "first" => -9999,
                    "last" => 9999,
                    "none" => 0,
                    other => other.parse::<i16>().map_err(|_| unknown(None))?,
                };
                Ok(Applied::Order(n * if t.negative { -1 } else { 1 }))
            }
            "content" => {
                let v = match value_elems {
                    ["start"] => ContentAlign::Start,
                    ["center"] => ContentAlign::Center,
                    ["end"] => ContentAlign::End,
                    ["between"] => ContentAlign::SpaceBetween,
                    ["around"] => ContentAlign::SpaceAround,
                    ["evenly"] => ContentAlign::SpaceEvenly,
                    ["baseline"] => ContentAlign::Baseline,
                    ["stretch"] => ContentAlign::Stretch,
                    ["normal"] => ContentAlign::Normal,
                    _ => return Err(unknown(suggest(head, self.theme))),
                };
                Ok(Applied::AlignContent(v))
            }
            "grid" => {
                // grid-cols-N / grid-rows-N (arbitrary templates reported).
                if value == "cols" || value.starts_with("cols-") {
                    let rest = value.strip_prefix("cols-").unwrap_or("");
                    let n: u16 = rest.parse().map_err(|_| {
                        if t.arbitrary.is_some() {
                            unsupported("arbitrary grid templates (grid-cols-[…]) are outside the typed IR — reported, never silent")
                        } else {
                            unknown(None)
                        }
                    })?;
                    return Ok(Applied::Grid(Grid {
                        cols: Some(n),
                        ..Default::default()
                    }));
                }
                if value == "rows" || value.starts_with("rows-") {
                    let rest = value.strip_prefix("rows-").unwrap_or("");
                    let n: u16 = rest.parse().map_err(|_| {
                        if t.arbitrary.is_some() {
                            unsupported("arbitrary grid templates (grid-rows-[…]) are outside the typed IR — reported, never silent")
                        } else {
                            unknown(None)
                        }
                    })?;
                    return Ok(Applied::Grid(Grid {
                        rows: Some(n),
                        ..Default::default()
                    }));
                }
                Err(unknown(suggest(head, self.theme)))
            }
            "col" => {
                if let Some(rest) = value.strip_prefix("span-") {
                    let n: u16 = rest
                        .parse()
                        .map_err(|_| unsupported("col-span-full/all are outside the typed IR"))?;
                    return Ok(Applied::Grid(Grid {
                        col_span: Some(n),
                        ..Default::default()
                    }));
                }
                if let Some(rest) = value.strip_prefix("start-") {
                    let n: i16 = rest
                        .parse()
                        .map_err(|_| unsupported("col-start-N/arbitrary outside the typed IR"))?;
                    return Ok(Applied::Grid(Grid {
                        col_start: Some(n),
                        ..Default::default()
                    }));
                }
                Err(unknown(suggest(head, self.theme)))
            }
            "row" => {
                if let Some(rest) = value.strip_prefix("span-") {
                    let n: u16 = rest
                        .parse()
                        .map_err(|_| unsupported("row-span-full/all are outside the typed IR"))?;
                    return Ok(Applied::Grid(Grid {
                        row_span: Some(n),
                        ..Default::default()
                    }));
                }
                if let Some(rest) = value.strip_prefix("start-") {
                    let n: i16 = rest
                        .parse()
                        .map_err(|_| unsupported("row-start-N/arbitrary outside the typed IR"))?;
                    return Ok(Applied::Grid(Grid {
                        row_start: Some(n),
                        ..Default::default()
                    }));
                }
                Err(unknown(suggest(head, self.theme)))
            }
            "auto-rows" if els.len() >= 2 && els[0] == "auto" && els[1] == "rows" => {
                Err(unsupported(
                    "auto-rows-min/max/fr tracks are outside the typed IR — reported, never silent",
                ))
            }
            "aspect" => {
                let v = match value {
                    "square" => 1.0,
                    "video" => 16.0 / 9.0,
                    "auto" => {
                        return Err(unsupported(
                            "aspect-auto resets to the intrinsic ratio — nothing to store",
                        ));
                    }
                    _ => {
                        if let Some(f) = fraction() {
                            f
                        } else if let Some(arb) = t.arbitrary {
                            // aspect-[4/3] or aspect-(--ratio)
                            arb_aspect_ratio(arb, self.theme)
                                .ok_or_else(|| unsupported("aspect-[...] is not a ratio"))?
                        } else {
                            return Err(unknown(suggest(head, self.theme)));
                        }
                    }
                };
                Ok(Applied::AspectRatio(v))
            }

            // ---------------- position (v0.2) ----------------
            "absolute" if value_elems.is_empty() => Ok(Applied::Position(Position::Absolute)),
            "fixed" if value_elems.is_empty() => Ok(Applied::Position(Position::Fixed)),
            "sticky" if value_elems.is_empty() => Ok(Applied::Position(Position::Sticky)),
            "relative" if value_elems.is_empty() => Ok(Applied::Position(Position::Relative)),
            "static" if value_elems.is_empty() => Ok(Applied::Position(Position::Static)),
            "inset" | "inset-x" | "inset-y" | "top" | "bottom" | "left" | "right" | "start"
            | "end" => {
                let v = inset_value(
                    t,
                    value,
                    fraction(),
                    &arb_size,
                    spacing,
                    self.theme,
                    &unknown,
                )?;
                Ok(match head {
                    "inset" => Applied::Inset {
                        top: Some(v),
                        right: Some(v),
                        bottom: Some(v),
                        left: Some(v),
                    },
                    "inset-x" => Applied::Inset {
                        top: None,
                        right: Some(v),
                        bottom: None,
                        left: Some(v),
                    },
                    "inset-y" => Applied::Inset {
                        top: Some(v),
                        right: None,
                        bottom: Some(v),
                        left: None,
                    },
                    "top" => Applied::Inset {
                        top: Some(v),
                        right: None,
                        bottom: None,
                        left: None,
                    },
                    "bottom" => Applied::Inset {
                        top: None,
                        right: None,
                        bottom: Some(v),
                        left: None,
                    },
                    "left" | "start" => Applied::Inset {
                        top: None,
                        right: None,
                        bottom: None,
                        left: Some(v),
                    },
                    "right" | "end" => Applied::Inset {
                        top: None,
                        right: Some(v),
                        bottom: None,
                        left: None,
                    },
                    _ => unreachable!(),
                })
            }
            "z" => {
                let n: i32 = match value {
                    "auto" => 0,
                    other => other.parse::<i32>().map_err(|_| unknown(None))?,
                };
                Ok(Applied::ZIndex(n * if t.negative { -1 } else { 1 }))
            }
            "overflow" => {
                let kind = |kw: &str| -> Option<OverflowKind> {
                    match kw {
                        "hidden" => Some(OverflowKind::Hidden),
                        "visible" => Some(OverflowKind::Visible),
                        "clip" => Some(OverflowKind::Clip),
                        "scroll" => Some(OverflowKind::Scroll),
                        "auto" => Some(OverflowKind::Auto),
                        _ => None,
                    }
                };
                let Some(k) = kind(value) else {
                    return Err(unknown(suggest(head, self.theme)));
                };
                Ok(Applied::Overflow(Overflow {
                    x: Some(k),
                    y: Some(k),
                }))
            }
            "overflow-x" | "overflow-y" => {
                let kind = match value {
                    "hidden" => OverflowKind::Hidden,
                    "visible" => OverflowKind::Visible,
                    "clip" => OverflowKind::Clip,
                    "scroll" => OverflowKind::Scroll,
                    "auto" => OverflowKind::Auto,
                    _ => return Err(unknown(suggest(head, self.theme))),
                };
                let mut o = Overflow::default();
                if head == "overflow-x" {
                    o.x = Some(kind);
                } else {
                    o.y = Some(kind);
                }
                Ok(Applied::Overflow(o))
            }
            "visible" if value_elems.is_empty() => Ok(Applied::Visibility(Visibility::Visible)),
            "invisible" if value_elems.is_empty() => Ok(Applied::Visibility(Visibility::Hidden)),

            // ---------------- typography ----------------
            "font" => {
                // font-sans/serif/mono (v0.2) vs font-{weight}.
                if let Some(fam) = match value {
                    "sans" => Some("sans"),
                    "serif" => Some("serif"),
                    "mono" => Some("mono"),
                    _ => None,
                } {
                    return Ok(Applied::Text(
                        TextProperty::FontFamily,
                        TextValue::Keyword(fam),
                    ));
                }
                let w = self.theme.font_weight(value).ok_or_else(|| {
                    unsupported("font-family / font-stretch classes are outside the typed table")
                })?;
                Ok(Applied::Text(
                    TextProperty::FontWeight,
                    TextValue::Weight(w),
                ))
            }
            "leading" => {
                if let Some(v) = leading_from_modifier(value, spacing, self.theme) {
                    return Ok(Applied::Text(TextProperty::LineHeight, TextValue::Px(v)));
                }
                if let Some(arb) = t.arbitrary
                    && let Some(v) = sized_from_arbitrary(arb, self.theme, &class)
                    && let SizedValue::Px(px) = v
                {
                    return Ok(Applied::Text(TextProperty::LineHeight, TextValue::Px(px)));
                }
                Err(missing(format!("--leading-{value}")))
            }
            "tracking" => {
                if let Some(arb) = t.arbitrary
                    && let Some(v) = sized_from_arbitrary(arb, self.theme, &class)
                    && let SizedValue::Px(px) = v
                {
                    return Ok(Applied::Text(
                        TextProperty::LetterSpacing,
                        TextValue::Spacing(px * if t.negative { -1.0 } else { 1.0 }),
                    ));
                }
                let em = match value_elems {
                    ["tighter"] => -0.05,
                    ["tight"] => -0.025,
                    ["normal"] => 0.0,
                    ["wide"] => 0.025,
                    ["wider"] => 0.05,
                    ["widest"] => 0.1,
                    _ => return Err(missing(format!("--tracking-{value}"))),
                };
                Ok(Applied::Text(
                    TextProperty::LetterSpacing,
                    TextValue::Spacing(em * 16.0),
                ))
            }
            "underline-offset" => {
                // v4.1.17: `underline-offset-<number>` → `text-underline-offset:
                // <number>px` (plain px, NOT spacing multiples; negatives
                // supported). Arbitrary lengths pass through.
                let n = if let Some(arb) = t.arbitrary {
                    ring_width_from_arbitrary(arb).ok_or_else(|| unknown(None))?
                } else {
                    let n: f32 = value_elems
                        .last()
                        .and_then(|e| e.parse().ok())
                        .ok_or_else(|| unknown(None))?;
                    n * if t.negative { -1.0 } else { 1.0 }
                };
                Ok(Applied::Text(
                    TextProperty::UnderlineOffset,
                    TextValue::Px(n),
                ))
            }
            "whitespace" => {
                let kw = match value {
                    "nowrap" => "nowrap",
                    "pre" => "pre",
                    "pre-line" => "pre-line",
                    "pre-wrap" => "pre-wrap",
                    "normal" => "normal",
                    _ => return Err(unknown(suggest(head, self.theme))),
                };
                Ok(Applied::Text(
                    TextProperty::Whitespace,
                    TextValue::Keyword(kw),
                ))
            }
            "select" => {
                let kw = match value {
                    "none" => "select-none",
                    "text" => "select-text",
                    "all" => "select-all",
                    "auto" => "select-auto",
                    _ => return Err(unknown(suggest(head, self.theme))),
                };
                Ok(Applied::Text(TextProperty::Select, TextValue::Keyword(kw)))
            }
            // v0.2 text transforms / decorations / overflow (plan §3.2).
            "italic" if value_elems.is_empty() => Ok(Applied::Text(
                TextProperty::FontStyle,
                TextValue::Keyword("italic"),
            )),
            "not-italic" if value_elems.is_empty() => Ok(Applied::Text(
                TextProperty::FontStyle,
                TextValue::Keyword("normal"),
            )),
            "uppercase" if value_elems.is_empty() => Ok(Applied::Text(
                TextProperty::TextTransform,
                TextValue::Keyword("uppercase"),
            )),
            "lowercase" if value_elems.is_empty() => Ok(Applied::Text(
                TextProperty::TextTransform,
                TextValue::Keyword("lowercase"),
            )),
            "capitalize" if value_elems.is_empty() => Ok(Applied::Text(
                TextProperty::TextTransform,
                TextValue::Keyword("capitalize"),
            )),
            "normal-case" if value_elems.is_empty() => Ok(Applied::Text(
                TextProperty::TextTransform,
                TextValue::Keyword("none"),
            )),
            "underline" if value_elems.is_empty() => Ok(Applied::Text(
                TextProperty::TextDecoration,
                TextValue::Keyword("underline"),
            )),
            "overline" if value_elems.is_empty() => Ok(Applied::Text(
                TextProperty::TextDecoration,
                TextValue::Keyword("overline"),
            )),
            "line-through" if value_elems.is_empty() => Ok(Applied::Text(
                TextProperty::TextDecoration,
                TextValue::Keyword("line-through"),
            )),
            "no-underline" if value_elems.is_empty() => Ok(Applied::Text(
                TextProperty::TextDecoration,
                TextValue::Keyword("none"),
            )),
            "text-ellipsis" if value_elems.is_empty() => Ok(Applied::Text(
                TextProperty::TextOverflow,
                TextValue::Keyword("ellipsis"),
            )),
            "text-clip" if value_elems.is_empty() => Ok(Applied::Text(
                TextProperty::TextOverflow,
                TextValue::Keyword("clip"),
            )),
            "truncate" if value_elems.is_empty() => Ok(Applied::Texts(vec![
                (TextProperty::TextOverflow, TextValue::Keyword("ellipsis")),
                (TextProperty::Whitespace, TextValue::Keyword("nowrap")),
            ])),
            "line-clamp" => {
                let n: u16 = value
                    .parse()
                    .map_err(|_| unsupported("line-clamp-[…] arbitrary values are reported"))?;
                Ok(Applied::Texts(vec![
                    (TextProperty::LineClamp, TextValue::Count(n)),
                    (TextProperty::TextOverflow, TextValue::Keyword("ellipsis")),
                ]))
            }
            "tabular-nums" if value_elems.is_empty() => Ok(Applied::Text(
                TextProperty::FontVariantNumeric,
                TextValue::Keyword("tabular-nums"),
            )),
            "ordinal" if value_elems.is_empty() => Ok(Applied::Text(
                TextProperty::FontVariantNumeric,
                TextValue::Keyword("ordinal"),
            )),
            "text-wrap" => Err(unsupported(
                "text-wrap-balance/pretty are CSS text-layout features with no Rust-toolkit equivalent — reported, never silent",
            )),

            // ---------------- effects (v0.2) ----------------
            "transition" => {
                let set = match value {
                    "" => TransitionSet::Default,
                    "all" => TransitionSet::All,
                    "colors" => TransitionSet::Colors,
                    "opacity" => TransitionSet::Opacity,
                    "shadow" => TransitionSet::Shadow,
                    "transform" => TransitionSet::Transform,
                    "none" => TransitionSet::None,
                    _ => return Err(unknown(suggest(head, self.theme))),
                };
                Ok(Applied::Transition {
                    set: Some(set),
                    duration_ms: None,
                    ease: None,
                    delay_ms: None,
                })
            }
            "duration" => {
                let ms = if let Some(arb) = t.arbitrary {
                    arb_duration(arb).ok_or_else(|| unknown(None))?
                } else {
                    value.parse::<f32>().map_err(|_| unknown(None))?
                };
                Ok(Applied::Transition {
                    set: None,
                    duration_ms: Some(ms),
                    ease: None,
                    delay_ms: None,
                })
            }
            "delay" => {
                let ms = if let Some(arb) = t.arbitrary {
                    arb_duration(arb).ok_or_else(|| unknown(None))?
                } else {
                    value.parse::<f32>().map_err(|_| unknown(None))?
                };
                Ok(Applied::Transition {
                    set: None,
                    duration_ms: None,
                    ease: None,
                    delay_ms: Some(ms),
                })
            }
            "ease" => {
                let e = if t.arbitrary.is_some() {
                    return Err(unsupported(
                        "ease-[cubic-bezier(…)] arbitrary curves are reported",
                    ));
                } else {
                    self.theme
                        .ease(value)
                        .ok_or_else(|| unknown(suggest(head, self.theme)))?
                };
                Ok(Applied::Transition {
                    set: None,
                    duration_ms: None,
                    ease: Some(e),
                    delay_ms: None,
                })
            }
            "animate" => {
                if value == "none" {
                    return Ok(Applied::Animation(Animation {
                        name: "none".into(),
                        duration_ms: None,
                        ease: None,
                        infinite: false,
                    }));
                }
                let spec = self
                    .theme
                    .animation(value)
                    .ok_or_else(|| missing(format!("--animate-{value}")))?;
                Ok(Applied::Animation(Animation {
                    name: spec.name.into(),
                    duration_ms: Some(spec.duration_ms),
                    ease: spec.ease,
                    infinite: spec.infinite,
                }))
            }
            "scale" | "scale-x" | "scale-y" => {
                let v = transform_number(t, value, &unknown)?;
                let pct01 = v / 100.0;
                Ok(Applied::Transform {
                    scale_x: (head != "scale-y").then(|| if t.negative { -pct01 } else { pct01 }),
                    scale_y: (head != "scale-x").then(|| if t.negative { -pct01 } else { pct01 }),
                    rotate_deg: None,
                    translate_x: None,
                    translate_y: None,
                    skew_x: None,
                    skew_y: None,
                })
            }
            "rotate" => {
                let deg = transform_number(t, value, &unknown)?;
                Ok(Applied::Transform {
                    scale_x: None,
                    scale_y: None,
                    rotate_deg: Some(deg * if t.negative { -1.0 } else { 1.0 }),
                    translate_x: None,
                    translate_y: None,
                    skew_x: None,
                    skew_y: None,
                })
            }
            "translate-x" | "translate-y" => {
                let v = transform_size(
                    t,
                    value,
                    fraction(),
                    &arb_size,
                    spacing,
                    self.theme,
                    &unknown,
                )?;
                let (x, y) = if head == "translate-x" {
                    (Some(v), None)
                } else {
                    (None, Some(v))
                };
                Ok(Applied::Transform {
                    scale_x: None,
                    scale_y: None,
                    rotate_deg: None,
                    translate_x: x,
                    translate_y: y,
                    skew_x: None,
                    skew_y: None,
                })
            }
            "skew-x" | "skew-y" => {
                let deg = transform_number(t, value, &unknown)?;
                Ok(Applied::Transform {
                    scale_x: None,
                    scale_y: None,
                    rotate_deg: None,
                    translate_x: None,
                    translate_y: None,
                    skew_x: (head == "skew-x").then(|| deg * if t.negative { -1.0 } else { 1.0 }),
                    skew_y: (head == "skew-y").then(|| deg * if t.negative { -1.0 } else { 1.0 }),
                })
            }

            // ---------------- misc ----------------
            "opacity" => {
                let n = if let Some(arb) = t.arbitrary {
                    arb_duration(arb).ok_or_else(|| unknown(None))?
                } else {
                    value.parse::<f32>().map_err(|_| unknown(None))?
                };
                Ok(Applied::Replace(
                    Slot::Opacity,
                    PartialValue::Opacity(Opacity(n / 100.0)),
                ))
            }
            "cursor" => {
                let v = cursor_of(value).ok_or_else(|| unknown(suggest(value, self.theme)))?;
                Ok(Applied::Replace(Slot::Cursor, PartialValue::Cursor(v)))
            }
            "pointer-events" => {
                let v = match value_elems {
                    ["none"] => PointerEvents::None,
                    ["auto"] => PointerEvents::Auto,
                    _ => return Err(unknown(suggest(head, self.theme))),
                };
                Ok(Applied::Replace(
                    Slot::PointerEvents,
                    PartialValue::PointerEvents(v),
                ))
            }

            // Recognized-but-dropped families (no filter/scroll engines).
            "blur" | "brightness" | "contrast" | "grayscale" | "hue-rotate" | "invert"
            | "saturate" | "sepia" | "drop-shadow" | "backdrop" => Err(unsupported(
                "CSS filters/backdrop-filters have no native equivalent — reported, never silent (plan §7)",
            )),
            "space" => Err(unsupported(
                "space-x/y between-child margins have no typed slot — use gap-* on the flex/grid container (the supported equivalent, plan §3.2)",
            )),
            "divide" => Err(unsupported(
                "divide-* between-child borders have no container-child border model — reported, never silent",
            )),
            "caret" | "object" | "overscroll" | "scroll" | "place" | "float" | "clear"
            | "columns" | "break" | "box" | "container" | "origin" | "perspective"
            | "transform" | "snap" | "forced" | "print" | "field-sizing" | "mask" | "filter"
            | "sr" | "not-sr" | "accent" | "appearance" | "resize" | "touch" | "will-change"
            | "fill" | "stroke" | "decoration" | "align" | "vertical" | "list" | "hyphens"
            | "antialiased" | "subpixel" | "ordinal-fake" | "isolate" | "outline-hidden"
            | "has" | "group" | "peer" | "not" | "supports" | "min" | "max" | "clamp" | "fit" => {
                let _ = head;
                Err(unsupported(
                    "utility family is outside the typed v0.2 table",
                ))
            }

            _ => Err(unknown(suggest(head, self.theme))),
        }
    }
}

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

/// Spacing-family value resolution (plan §3.2 F2 fix): arbitrary values are
/// ABSOLUTE px (never spacing multiples); numeric names use the 4px scale.
fn spacing_value<'a>(
    t: &ClassToken<'a>,
    value: &str,
    elems: &[&'a str],
    _spacing: f32,
    scale: impl Fn(f32) -> f32,
    unknown: &impl Fn(Option<String>) -> Warning,
    theme: &TailwindTheme,
) -> Result<f32, Warning> {
    if let Some(arb) = t.arbitrary {
        return match sized_from_arbitrary(arb, theme, t.raw) {
            Some(SizedValue::Px(px)) => Ok(px * if t.negative { -1.0 } else { 1.0 }),
            _ => Err(unknown(None)),
        };
    }
    if let Some(n) = elems.last().and_then(|e| e.parse::<f32>().ok()) {
        return Ok(scale(n));
    }
    match value {
        "px" => Ok(1.0 * if t.negative { -1.0 } else { 1.0 }),
        _ => Err(unknown(None)),
    }
}

/// Sizing-family value resolution (`w-*`, `basis-*`, ...). Fractions come
/// from the raw modifier (F1), viewport sizes are typed (v0.2), and the
/// container scale resolves `max-w-xl` & friends from `--container-*`.
#[allow(clippy::too_many_arguments)]
fn size_value<'a>(
    t: &ClassToken<'a>,
    value: &str,
    _elems: &[&'a str],
    frac: Option<f32>,
    arb_size: &impl Fn(Arbitrary<'a>) -> Result<SizedValue, Warning>,
    spacing: f32,
    theme: &TailwindTheme,
    unknown: &impl Fn(Option<String>) -> Warning,
) -> Result<SizedValue, Warning> {
    if let Some(a) = t.arbitrary {
        return arb_size(a);
    }
    if let Some(f) = frac {
        return Ok(SizedValue::Fraction(
            f * if t.negative { -1.0 } else { 1.0 },
        ));
    }
    Ok(match value {
        "full" => SizedValue::Full,
        "fit" => SizedValue::FitContent,
        "auto" => SizedValue::Auto,
        "px" => SizedValue::Px(1.0),
        // v0.2: viewport sizes are typed, resolved by the backend.
        "screen" | "dvh" | "svh" | "lvh" | "dvw" | "svw" | "lvw" | "min" | "max" => {
            SizedValue::Viewport
        }
        other => {
            if let Ok(n) = other.parse::<f32>() {
                SizedValue::Px(n * spacing)
            } else if let Some(c) = theme.container_px(other) {
                SizedValue::Px(c)
            } else {
                return Err(unknown(None));
            }
        }
    })
}

/// Inset/offset value resolution (`top-*`, `inset-*`, ...).
fn inset_value<'a>(
    t: &ClassToken<'a>,
    value: &str,
    frac: Option<f32>,
    arb_size: &impl Fn(Arbitrary<'a>) -> Result<SizedValue, Warning>,
    spacing: f32,
    _theme: &TailwindTheme,
    unknown: &impl Fn(Option<String>) -> Warning,
) -> Result<SizedValue, Warning> {
    if let Some(a) = t.arbitrary {
        return arb_size(a);
    }
    if let Some(f) = frac {
        return Ok(SizedValue::Fraction(f));
    }
    Ok(match value {
        "auto" => SizedValue::Auto,
        "full" => SizedValue::Full,
        "px" => SizedValue::Px(1.0),
        other => {
            if let Ok(n) = other.parse::<f32>() {
                SizedValue::Px(n * spacing)
            } else {
                return Err(unknown(None));
            }
        }
    })
}

/// `rotate-45` / `scale-105` numbers; arbitrary `deg` values included.
fn transform_number(
    t: &ClassToken<'_>,
    value: &str,
    unknown: &impl Fn(Option<String>) -> Warning,
) -> Result<f32, Warning> {
    if let Some(arb) = t.arbitrary {
        let v = arb
            .value
            .trim()
            .trim_end_matches("deg")
            .parse::<f32>()
            .map_err(|_| unknown(None))?;
        return Ok(v);
    }
    value.parse::<f32>().map_err(|_| unknown(None))
}

/// `translate-x-4` → 16px; `translate-x-1/2` → Fraction(0.5) (F1 reuse);
/// `translate-x-full` → Fraction(1).
fn transform_size<'a>(
    t: &ClassToken<'a>,
    value: &str,
    frac: Option<f32>,
    arb_size: &impl Fn(Arbitrary<'a>) -> Result<SizedValue, Warning>,
    spacing: f32,
    theme: &TailwindTheme,
    unknown: &impl Fn(Option<String>) -> Warning,
) -> Result<SizedValue, Warning> {
    if let Some(a) = t.arbitrary {
        return arb_size(a);
    }
    if let Some(f) = frac {
        return Ok(SizedValue::Fraction(
            f * if t.negative { -1.0 } else { 1.0 },
        ));
    }
    Ok(match value {
        "full" => SizedValue::Fraction(1.0),
        "px" => SizedValue::Px(1.0),
        "auto" => SizedValue::Auto,
        other => {
            if let Ok(n) = other.parse::<f32>() {
                SizedValue::Px(n * spacing)
            } else if let Some(c) = theme.container_px(other) {
                SizedValue::Px(c)
            } else {
                return Err(unknown(None));
            }
        }
    })
}

/// F6 helper: `text-sm/6` → 24px (6 × spacing); `text-sm/tight` → the
/// `--leading-tight` ratio × 16px base.
fn leading_from_modifier(m: &str, spacing: f32, theme: &TailwindTheme) -> Option<f32> {
    if let Ok(n) = m.parse::<f32>() {
        return Some(n * spacing);
    }
    if let Some(ratio) = theme.leading(m) {
        return Some(ratio * 16.0);
    }
    None
}

fn border_style_of(value: &str) -> Option<BorderStyle> {
    Some(match value {
        "solid" => BorderStyle::Solid,
        "dashed" => BorderStyle::Dashed,
        "dotted" => BorderStyle::Dotted,
        "double" => BorderStyle::Double,
        "none" => BorderStyle::None,
        "hidden" => BorderStyle::Hidden,
        _ => return None,
    })
}

fn shadow_applied(
    first: crate::theme::BoxShadowSpec,
    rest: Vec<crate::theme::BoxShadowSpec>,
) -> Applied {
    Applied::ShadowPreset {
        first: to_shadow(&first),
        rest: rest.iter().map(to_shadow).collect(),
    }
}

/// The full v4 cursor keyword set.
fn cursor_of(value: &str) -> Option<Cursor> {
    Some(match value {
        "auto" => Cursor::Auto,
        "default" => Cursor::Default,
        "pointer" => Cursor::Pointer,
        "wait" => Cursor::Wait,
        "text" => Cursor::Text,
        "move" => Cursor::Move,
        "help" => Cursor::Help,
        "not-allowed" => Cursor::NotAllowed,
        "none" => Cursor::None,
        "context-menu" => Cursor::ContextMenu,
        "progress" => Cursor::Progress,
        "cell" => Cursor::Cell,
        "crosshair" => Cursor::Crosshair,
        "vertical-text" => Cursor::VerticalText,
        "alias" => Cursor::Alias,
        "copy" => Cursor::Copy,
        "no-drop" => Cursor::NoDrop,
        "grab" => Cursor::Grab,
        "grabbing" => Cursor::Grabbing,
        "all-scroll" => Cursor::AllScroll,
        "col-resize" => Cursor::ColResize,
        "row-resize" => Cursor::RowResize,
        "n-resize" => Cursor::NResize,
        "e-resize" => Cursor::EResize,
        "s-resize" => Cursor::SResize,
        "w-resize" => Cursor::WResize,
        "ne-resize" => Cursor::NeResize,
        "nw-resize" => Cursor::NwResize,
        "se-resize" => Cursor::SeResize,
        "sw-resize" => Cursor::SwResize,
        "ew-resize" => Cursor::EwResize,
        "ns-resize" => Cursor::NsResize,
        "nesw-resize" => Cursor::NeswResize,
        "nwse-resize" => Cursor::NwseResize,
        "zoom-in" => Cursor::ZoomIn,
        "zoom-out" => Cursor::ZoomOut,
        _ => return None,
    })
}

/// Arbitrary value → [`Color`] (`[#fff]`, `[oklch(...)]`, `[var(--x)]`,
/// `(--x)`; F7's `bg-primary/(--o)` opacity path is separate).
fn color_from_arbitrary(a: Arbitrary<'_>, theme: &TailwindTheme) -> Option<Color> {
    if a.is_property {
        return None;
    }
    if matches!(a.hint, Some(h) if h != "color") {
        return None;
    }
    let css = a.css();
    // var() forms resolve through the theme color registry.
    if a.form == ArbForm::Paren || css.starts_with("var(") {
        let name = css
            .strip_prefix("var(")
            .and_then(|r| r.strip_suffix(')'))
            .map(str::trim)?
            .trim_start_matches("--");
        return theme.color(name);
    }
    Color::parse(&css).ok()
}

/// Arbitrary value → [`SizedValue`] (`[42px]`, `[50%]`, `[length:3rem]`,
/// `[var(--w)]`, `(--w)`; F2: absolute px, never spacing multiples).
fn sized_from_arbitrary(
    a: Arbitrary<'_>,
    theme: &TailwindTheme,
    class: &str,
) -> Option<SizedValue> {
    let _ = class;
    if a.is_property {
        return None;
    }
    if matches!(a.hint, Some(h) if h != "length" && h != "percentage") {
        return None;
    }
    let css = a.css();
    // var() forms resolve through the theme length registry.
    if a.form == ArbForm::Paren || css.starts_with("var(") {
        let name = css
            .strip_prefix("var(")
            .and_then(|r| r.strip_suffix(')'))
            .map(str::trim)?
            .trim_start_matches("--");
        return theme.length(name).map(SizedValue::Px);
    }
    let v = css.trim();
    if let Some(p) = v.strip_suffix("px") {
        return p.trim().parse::<f32>().ok().map(SizedValue::Px);
    }
    if let Some(r) = v.strip_suffix("rem") {
        return r
            .trim()
            .parse::<f32>()
            .ok()
            .map(|n| SizedValue::Px(n * 16.0));
    }
    if let Some(p) = v.strip_suffix('%') {
        return p
            .trim()
            .parse::<f32>()
            .ok()
            .map(|n| SizedValue::Fraction(n / 100.0));
    }
    None
}

/// `ring-[3px]` → 3.0.
fn ring_width_from_arbitrary(a: Arbitrary<'_>) -> Option<f32> {
    if a.is_property {
        return None;
    }
    let v = a.value.trim();
    v.strip_suffix("px").unwrap_or(v).trim().parse::<f32>().ok()
}

/// `duration-[2s]` → 2000; `duration-300` handled by the caller.
fn arb_duration(a: Arbitrary<'_>) -> Option<f32> {
    if a.is_property {
        return None;
    }
    let v = a.value.trim();
    if let Some(ms) = v.strip_suffix("ms") {
        return ms.trim().parse().ok();
    }
    if let Some(s) = v.strip_suffix('s') {
        return s.trim().parse::<f32>().ok().map(|n| n * 1000.0);
    }
    v.parse().ok()
}

/// `aspect-[4/3]` → 1.333; `aspect-(--ratio)` resolves via `--aspect-*`.
fn arb_aspect_ratio(a: Arbitrary<'_>, theme: &TailwindTheme) -> Option<f32> {
    if a.is_property {
        return None;
    }
    if a.form == ArbForm::Paren || a.value.starts_with("var(") {
        let name = a
            .value
            .trim()
            .strip_prefix("var(")
            .and_then(|r| r.strip_suffix(')'))
            .map(str::trim)?;
        // --aspect-* tokens are Raw ("16 / 9").
        let raw = theme.raw_var(name)?;
        let mut it = raw.split('/');
        let w: f32 = it.next()?.trim().parse().ok()?;
        let h: f32 = it.next()?.trim().parse().ok()?;
        return (h != 0.0).then(|| w / h);
    }
    let v = a.value.trim();
    if let Some((w, h)) = v.split_once('/') {
        let w: f32 = w.trim().parse().ok()?;
        let h: f32 = h.trim().parse().ok()?;
        return (h != 0.0).then(|| w / h);
    }
    v.parse().ok()
}

fn padding_applied(head: &str, v: f32) -> Applied {
    use Applied::Padding as P;
    match head {
        "p" => P {
            top: Some(v),
            right: Some(v),
            bottom: Some(v),
            left: Some(v),
        },
        "px" => P {
            top: None,
            right: Some(v),
            bottom: None,
            left: Some(v),
        },
        "py" => P {
            top: Some(v),
            right: None,
            bottom: Some(v),
            left: None,
        },
        "pt" => P {
            top: Some(v),
            right: None,
            bottom: None,
            left: None,
        },
        "pr" | "pe" => P {
            top: None,
            right: Some(v),
            bottom: None,
            left: None,
        },
        "pb" => P {
            top: None,
            right: None,
            bottom: Some(v),
            left: None,
        },
        "pl" | "ps" => P {
            top: None,
            right: None,
            bottom: None,
            left: Some(v),
        },
        _ => unreachable!(),
    }
}

fn margin_applied(head: &str, v: f32) -> Applied {
    use Applied::Margin as M;
    match head {
        "m" => M {
            top: Some(v),
            right: Some(v),
            bottom: Some(v),
            left: Some(v),
        },
        "mx" => M {
            top: None,
            right: Some(v),
            bottom: None,
            left: Some(v),
        },
        "my" => M {
            top: Some(v),
            right: None,
            bottom: Some(v),
            left: None,
        },
        "mt" => M {
            top: Some(v),
            right: None,
            bottom: None,
            left: None,
        },
        "mr" | "me" => M {
            top: None,
            right: Some(v),
            bottom: None,
            left: None,
        },
        "mb" => M {
            top: None,
            right: None,
            bottom: Some(v),
            left: None,
        },
        "ml" | "ms" => M {
            top: None,
            right: None,
            bottom: None,
            left: Some(v),
        },
        _ => unreachable!(),
    }
}

fn border_width_family(name: &str, negative: bool) -> Option<Applied> {
    // v4 border widths are px, not spacing-scale multiples. Non-target
    // edges are `None` (not `Some(0.0)`) so per-side classes COMPOSE like
    // the padding/margin families do. v0.2 adds the logical `s`/`e` sides
    // (LTR: left/right; flipped under RTL at lowering time).
    let sign = if negative { -1.0 } else { 1.0 };
    let w = |n: f32| Some(n * sign);
    use Applied::BorderWidth as B;
    let (edge, width) = match name.split_once('-') {
        Some((e, n)) => (Some(e), n.parse::<f32>().ok()),
        None => {
            // Bare edge letters (`border-t`, `border-x`) mean width 1.
            if matches!(name, "t" | "r" | "b" | "l" | "x" | "y" | "s" | "e") {
                (Some(name), None)
            } else {
                (None, name.parse::<f32>().ok())
            }
        }
    };
    Some(match (edge, width) {
        (None, Some(n)) => B {
            top: w(n),
            right: w(n),
            bottom: w(n),
            left: w(n),
        },
        (Some("t"), n) => B {
            top: w(n.unwrap_or(1.0)),
            right: None,
            bottom: None,
            left: None,
        },
        (Some("r"), n) => B {
            top: None,
            right: w(n.unwrap_or(1.0)),
            bottom: None,
            left: None,
        },
        (Some("b"), n) => B {
            top: None,
            right: None,
            bottom: w(n.unwrap_or(1.0)),
            left: None,
        },
        (Some("l"), n) => B {
            top: None,
            right: None,
            bottom: None,
            left: w(n.unwrap_or(1.0)),
        },
        (Some("s"), n) => B {
            top: None,
            right: None,
            bottom: None,
            left: w(n.unwrap_or(1.0)),
        },
        (Some("e"), n) => B {
            top: None,
            right: w(n.unwrap_or(1.0)),
            bottom: None,
            left: None,
        },
        (Some("x"), n) => B {
            top: None,
            right: w(n.unwrap_or(1.0)),
            bottom: None,
            left: w(n.unwrap_or(1.0)),
        },
        (Some("y"), n) => B {
            top: w(n.unwrap_or(1.0)),
            right: None,
            bottom: w(n.unwrap_or(1.0)),
            left: None,
        },
        _ => return None,
    })
}

#[derive(Clone, Copy, PartialEq)]
struct Edges {
    top: Option<f32>,
    right: Option<f32>,
    bottom: Option<f32>,
    left: Option<f32>,
}

impl Edges {
    const NONE: Edges = Edges {
        top: None,
        right: None,
        bottom: None,
        left: None,
    };
}

#[derive(Clone, Copy, PartialEq)]
struct Corners {
    tl: Option<f32>,
    tr: Option<f32>,
    br: Option<f32>,
    bl: Option<f32>,
}

impl Corners {
    const NONE: Corners = Corners {
        tl: None,
        tr: None,
        br: None,
        bl: None,
    };
}

fn merge_layout(acc: &mut Layout, next: &Layout) {
    if next.display.is_some() {
        acc.display = next.display;
    }
    if next.flex_direction.is_some() {
        acc.flex_direction = next.flex_direction;
    }
    if next.align_items.is_some() {
        acc.align_items = next.align_items;
    }
    if next.justify_content.is_some() {
        acc.justify_content = next.justify_content;
    }
    if next.flex_wrap.is_some() {
        acc.flex_wrap = next.flex_wrap;
    }
    if next.flex_grow.is_some() {
        acc.flex_grow = next.flex_grow;
    }
    if next.flex_shrink.is_some() {
        acc.flex_shrink = next.flex_shrink;
    }
    if next.align_self.is_some() {
        acc.align_self = next.align_self;
    }
}

fn merge_ring(style: &mut ResolvedStyle, ring: &Ring) {
    let r = style.ring.get_or_insert_with(Ring::default);
    if ring.width != 0.0 {
        r.width = ring.width;
    }
    if ring.color.is_some() {
        r.color = ring.color;
    }
    if ring.offset_width != 0.0 {
        r.offset_width = ring.offset_width;
    }
    if ring.offset_color.is_some() {
        r.offset_color = ring.offset_color;
    }
    if ring.inset {
        r.inset = true;
    }
}

fn merge_outline(style: &mut ResolvedStyle, o: &Outline) {
    let cur = style.outline.get_or_insert_with(Outline::default);
    if o.none {
        cur.none = true;
        cur.width = 0.0;
    }
    if o.width != 0.0 {
        cur.width = o.width;
    }
    if o.color.is_some() {
        cur.color = o.color;
    }
    if o.offset != 0.0 {
        cur.offset = o.offset;
    }
}

fn to_shadow(spec: &crate::theme::BoxShadowSpec) -> BoxShadow {
    BoxShadow {
        offset_x: spec.offset_x,
        offset_y: spec.offset_y,
        blur: spec.blur,
        spread: spec.spread,
        color: spec.color,
        inset: spec.inset,
    }
}

/// v4 default shadow presets (used when the theme has no --shadow-* tokens).
fn v4_default_shadow(name: &str) -> crate::theme::BoxShadowSpec {
    use crate::color::Color;
    let black = Color::from_srgb(0.0, 0.0, 0.0, 0.05);
    match name {
        "sm" => crate::theme::BoxShadowSpec {
            offset_x: 0.0,
            offset_y: 1.0,
            blur: 3.0,
            spread: 0.0,
            color: black,
            inset: false,
        },
        "md" => crate::theme::BoxShadowSpec {
            offset_x: 0.0,
            offset_y: 2.0,
            blur: 6.0,
            spread: -1.0,
            color: black,
            inset: false,
        },
        "lg" => crate::theme::BoxShadowSpec {
            offset_x: 0.0,
            offset_y: 4.0,
            blur: 10.0,
            spread: -2.0,
            color: black,
            inset: false,
        },
        "xl" => crate::theme::BoxShadowSpec {
            offset_x: 0.0,
            offset_y: 8.0,
            blur: 16.0,
            spread: -4.0,
            color: black,
            inset: false,
        },
        // "xs" (v4 default for empty) — 0 1px 2px 0 rgb(0 0 0 / 0.05)
        _ => crate::theme::BoxShadowSpec {
            offset_x: 0.0,
            offset_y: 1.0,
            blur: 2.0,
            spread: 0.0,
            color: black,
            inset: false,
        },
    }
}

fn is_size_name(elems: &[&str]) -> bool {
    matches!(
        elems,
        ["xs"]
            | ["sm"]
            | ["base"]
            | ["lg"]
            | ["xl"]
            | ["2xl"]
            | ["3xl"]
            | ["4xl"]
            | ["5xl"]
            | ["6xl"]
            | ["7xl"]
            | ["8xl"]
            | ["9xl"]
    )
}

#[derive(Clone, Copy, PartialEq)]
enum RadiusEdges {
    All,
    Top,
    Bottom,
    Left,
    Right,
    /// `rounded-s-*` — the start side (left in LTR; F5).
    S,
    /// `rounded-e-*` — the end side (right in LTR; F5).
    E,
    /// `rounded-ss-*` — start-start corner (F5).
    Ss,
    /// `rounded-se-*` — start-end corner (F5).
    Se,
    /// `rounded-ee-*` — end-end corner (F5).
    Ee,
    /// `rounded-es-*` — end-start corner (F5).
    Es,
    Tl,
    Tr,
    Br,
    Bl,
}

/// Split `rounded-t-md` into edges + token name, including the logical
/// start/end edges (F5, plan §3.2).
fn radius_family(name: &str) -> (RadiusEdges, Option<&str>) {
    match name {
        "" => (RadiusEdges::All, Some("DEFAULT")),
        "full" | "none" => (RadiusEdges::All, None),
        other => {
            let (edge, token) = match other.split_once('-') {
                Some((e, t)) => (e, Some(t)),
                None => (other, None),
            };
            let edges = match edge {
                "t" => RadiusEdges::Top,
                "b" => RadiusEdges::Bottom,
                "l" => RadiusEdges::Left,
                "r" => RadiusEdges::Right,
                "s" => RadiusEdges::S,
                "e" => RadiusEdges::E,
                "ss" => RadiusEdges::Ss,
                "se" => RadiusEdges::Se,
                "ee" => RadiusEdges::Ee,
                "es" => RadiusEdges::Es,
                "tl" => RadiusEdges::Tl,
                "tr" => RadiusEdges::Tr,
                "br" => RadiusEdges::Br,
                "bl" => RadiusEdges::Bl,
                _ => RadiusEdges::All,
            };
            if edges == RadiusEdges::All {
                (RadiusEdges::All, Some(other))
            } else {
                (edges, token)
            }
        }
    }
}

fn default_radius(token: &str, spacing: f32) -> f32 {
    match token {
        "DEFAULT" | "sm" => spacing,
        "md" => spacing * 1.5,
        "lg" => spacing * 2.0,
        "xl" => spacing * 3.0,
        "2xl" => spacing * 4.0,
        "3xl" => spacing * 6.0,
        _ => spacing * 1.5,
    }
}

/// "did you mean" suggestion via bounded Levenshtein over known utility
/// prefixes and color tokens.
fn suggest(name: &str, theme: &TailwindTheme) -> Option<String> {
    const PREFIXES: &[&str] = &[
        "bg",
        "text",
        "border",
        "rounded",
        "p",
        "px",
        "py",
        "pt",
        "pr",
        "pb",
        "pl",
        "m",
        "mx",
        "my",
        "mt",
        "mr",
        "mb",
        "ml",
        "w",
        "h",
        "size",
        "min-w",
        "min-h",
        "max-w",
        "max-h",
        "gap",
        "flex",
        "grid",
        "items",
        "justify",
        "shadow",
        "ring",
        "ring-offset",
        "outline",
        "font",
        "leading",
        "tracking",
        "opacity",
        "cursor",
        "pointer-events",
        "transition",
        "animate",
        "select",
        "whitespace",
        "underline-offset",
        "grow",
        "shrink",
        "basis",
        "order",
        "content",
        "aspect",
        "inset",
        "top",
        "bottom",
        "left",
        "right",
        "z",
        "overflow",
        "visible",
        "invisible",
        "absolute",
        "relative",
        "fixed",
        "sticky",
        "static",
        "truncate",
        "uppercase",
        "lowercase",
        "capitalize",
        "italic",
        "underline",
        "line-clamp",
        "scale",
        "rotate",
        "translate",
        "skew",
        "duration",
        "delay",
        "ease",
    ];
    let mut best: Option<(usize, String)> = None;
    let mut consider = |candidate: String| {
        let d = levenshtein(name, &candidate);
        if d <= 3 && best.as_ref().is_none_or(|(bd, _)| d < *bd) {
            best = Some((d, candidate));
        }
    };
    for p in PREFIXES {
        consider((*p).to_string());
    }
    // Full scan: suggestions run only on the cold error path, and the v4
    // default palette alone adds ~244 color names — truncating the list
    // made suggestions depend on hash order.
    for c in theme.color_names() {
        consider(c.to_string());
    }
    best.map(|(_, c)| c)
}

fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0usize; b.len() + 1];
    for i in 1..=a.len() {
        cur[0] = i;
        for j in 1..=b.len() {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            cur[j] = (prev[j] + 1).min(cur[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b.len()]
}
