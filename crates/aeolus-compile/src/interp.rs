//! The interpreter arm (architecture plan §3.1 emit (c) — the `Target`
//! arm the emit diagram names "interp" and the crate docs carried as
//! named-but-unarmed): a compiled [`Program`] resolves to its EFFECTIVE
//! declarations under one probe situation — the runtime half the §3.2
//! table's "interpreter otherwise" migration line needs.
//!
//! ```text
//! interp(program, &Env)                    -> Result<Interped, InterpError>
//! interp_theme(program, &TokenSource, &Env) -> Result<Interped, InterpError>
//!
//! Program (layers, specificity-sorted)
//!   └─ front-to-back read: a layer FIRES when its Cond matches the Env
//!      ├─ fired   → its declarations win per longhand (later beats
//!      │            earlier, the cascade the IR's sort encodes)
//!      ├─ quiet   → skipped (the probe says no — CSS semantics)
//!      └─ excluded→ reported as a typed NOTE (never silent):
//!                   ::selection/::placeholder are separate render
//!                   targets; container layers without a container
//!                   context never match
//! ```
//!
//! The theme dynamism resolves through the [`TokenSource`] SEAM — the
//! same shape as the custom seam's [`ClassifyVariant`](crate::ClassifyVariant)
//! and the cache's [`DynStore`](crate::DynStore): the walk is generic
//! over the source, [`TokenTable`] is the built-in (the T2 tier's
//! "one load from TokenTable + one multiply", mode-aware, through the
//! audited [`TokenTable::resolve_rgba`] primitive), and a host wires its
//! own (a live theme bridge, a test double) through the same two
//! methods. The theme-less [`interp`] is the honest boundary: token
//! values ([`Val::Tok`]/[`Val::ModePair`]) are the theme pass's forms —
//! they refuse per longhand with a typed error, exactly like the
//! theme-less compile walk, never a silent passthrough.
//!
//! The ATTRIBUTE-PROBE layers (the `data-*`/`aria-*` families the
//! variant fold lands on the typed `Cond` surface) resolve through the
//! plan §3.5 answer the [`InterpNote::AttributeUnqueried`] note named:
//! typed state bits passed WITH the element — the [`AttrProbe`] seam
//! (one question: what value does the element carry for this
//! attribute?), the built-in [`AttrState`] static-bit table, and the
//! [`interp_probes`]/[`interp_theme_probes`] entry points the
//! `*_with` convention carries (the [`compile_with`](crate::compile_with)
//! precedent). A probe layer FIRES when the element carries the
//! attribute (the presence spelling) or carries it with the folded
//! value (the bracketed spelling); a miss is a QUIET skip — a CSS
//! selector that does not match, not an exclusion. The descendant
//! probes (`has-`/`group-has-`/`peer-has-`) stay the plan §3.5 row's
//! diagnosed boundary — reported, honest failure kept — because the
//! descendant shape is not an attribute bit.
//!
//! The NAMED-CONTAINER layers (the `@md/card:` spellings the fold lands
//! on [`Cond::Container`]/[`Cond::MaxContainer`] with a name) resolve
//! through the plan §3.5 answer the same note named: the container's
//! OWN width — the [`ContainerProbe`] seam (one question: what width
//! does this container name carry?), the built-in [`ContainerState`]
//! static table, and the `*_containers` entry points the `*_probes`
//! convention carries. A named layer asks the seam for ITS container:
//! a hit fires by the same AT/strictly-below boundary semantics the
//! viewport breakpoints obey, a container the source does not know is
//! a QUIET miss (a CSS selector that does not match), and the
//! sourceless forms keep the reported exclusion. The UNNAMED forms
//! keep answering from [`Env::container_rem`] — the seam takes over
//! the per-name question only.
//!
//! The PSEUDO-ELEMENT layers (`selection:`/`placeholder:`, the plan
//! §3.5 render-target row) resolve through the [`PseudoTarget`]
//! selector and the [`interp_pseudo`]/[`interp_pseudo_theme`] entry
//! points: a pseudo-element styles a DIFFERENT render target, so its
//! cascade resolves in its OWN walk — only the rules targeting THAT
//! pseudo-element participate (the element's own layers never do), the
//! answer is scoped and silent about the rest, and the element walk's
//! [`InterpNote::PseudoElement`] note stays the element-side half of
//! the same contract.
//!
//! Observability follows the crate's opt-in instrumentation seam: the
//! `tracing` feature spans the walk and logs every fired layer and
//! reported note (no subscriber is installed here — the host app owns
//! one); the [`Interped`] surface exposes the resolved declarations and
//! the notes directly. Errors carry the stable code + help contract
//! (`InterpError::code`/`InterpError::help` always on, the `miette`
//! feature wiring the same strings into the structured impl).
//!
//! What this slice deliberately is NOT (each documented where it
//! belongs): the keyword names ([`Val::Kw`] is row-relative by design —
//! the codegen crate owns the names, so the interp resolves THEME
//! dynamism and passes keywords through; the rust-tokens arm renders
//! them where the rows still talk) and the named group/peer label
//! (no IR slot yet — the Env answers the probe, not the label). The
//! static-data and rust-tokens emit arms are the pipeline's other
//! program forms (`emit_static`, `emit_rust`).

use std::fmt;

use aeolus_ir::{Cond, Decl, Program, Prop, PropMask, Val};
use aeolus_theme::{ResolveError, TokenTable};

// ---------------------------------------------------------------------------
// The probe: Env
// ---------------------------------------------------------------------------

/// The probe situation a program resolves under — the answers the
/// [`Cond`] surface asks for, and nothing else (plan §3.5's `Env`: the
/// questions a backend can answer about the element's current situation).
///
/// Every field defaults to the CSS "does not match" answer: states off,
/// the viewport at zero rem (no breakpoint matches), no container
/// context (container layers never match — reported under the
/// sourceless forms, answered per name through the
/// [`ContainerProbe`] seam under the `*_containers` forms). A backend
/// implements only what it knows; an unanswered condition simply does
/// not fire, the same semantics CSS has when a selector does not match.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Env {
    /// The mode tier: `.dark` active? ([`Cond::Dark`]; also the half
    /// [`Val::ModePair`] slots and the T2 multiply select).
    pub dark: bool,
    /// The element is hovered ([`Cond::Hover`]).
    pub hover: bool,
    /// The element has keyboard focus ([`Cond::Focus`]).
    pub focus: bool,
    /// The element matches `:focus-visible` ([`Cond::FocusVisible`]).
    pub focus_visible: bool,
    /// The element contains the focused element ([`Cond::FocusWithin`]).
    pub focus_within: bool,
    /// The element is pressed ([`Cond::Active`]).
    pub active: bool,
    /// The element is disabled ([`Cond::Disabled`]).
    pub disabled: bool,
    /// The element was visited ([`Cond::Visited`]).
    pub visited: bool,
    /// The element is checked ([`Cond::Checked`]).
    pub checked: bool,
    /// The element shows its placeholder ([`Cond::PlaceholderShown`]).
    pub placeholder_shown: bool,
    /// A `.group` ancestor is hovered ([`Cond::GroupHover`]).
    pub group_hover: bool,
    /// A `.group` ancestor has focus ([`Cond::GroupFocus`]).
    pub group_focus: bool,
    /// A `.peer` sibling is hovered ([`Cond::PeerHover`]).
    pub peer_hover: bool,
    /// A `.peer` sibling has focus ([`Cond::PeerFocus`]).
    pub peer_focus: bool,
    /// The viewport width in rem ([`Cond::Bp`]/[`Cond::Min`] fire AT and
    /// above their breakpoint's rem; [`Cond::Max`] strictly below it).
    pub viewport_rem: f32,
    /// The nearest eligible container's width in rem (the UNNAMED
    /// [`Cond::Container`]/[`Cond::MaxContainer`] forms). `None` — no
    /// container context — means those layers never match, reported.
    /// A NAMED container query (`@md/card:`) needs THAT container's
    /// width, which one nearest-container slot cannot answer — those
    /// layers ask the [`ContainerProbe`] seam through the
    /// `*_containers` entry points (a source-answered miss is CSS
    /// -quiet), and the sourceless forms report
    /// [`InterpNote::ContainerUnqueried`] whatever this field carries.
    pub container_rem: Option<f32>,
}

// ---------------------------------------------------------------------------
// The attribute-probe seam: AttrProbe
// ---------------------------------------------------------------------------

/// The element-own attribute family a probe asks about — the families
/// the variant fold lands on the typed `Cond` surface
/// ([`Cond::Data`]/[`Cond::Aria`]). The descendant probes
/// (`has-`/`group-has-`/`peer-has-`) are NOT here: the plan §3.5 row
/// keeps them the diagnosed boundary (the descendant shape is not an
/// attribute bit).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttrKind {
    /// The `data-*` family ([`Cond::Data`] — `data-[state=open]:`).
    Data,
    /// The `aria-*` family ([`Cond::Aria`] — `aria-busy:`/`aria-[sort=asc]:`).
    Aria,
}

/// One typed attribute bit: the element carries this attribute with
/// this value — the plan §3.5 "typed state bits passed WITH the
/// element", one row per bit. The bit's value is what a value probe
/// compares against; a presence probe only asks whether the bit exists.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AttrBit<'t> {
    /// The family the attribute belongs to.
    pub kind: AttrKind,
    /// The attribute name WITHOUT the family prefix (`"state"` for
    /// `data-state`) — the folded [`Cond`]'s own payload shape.
    pub name: &'t str,
    /// The attribute's value (`"open"`), verbatim.
    pub value: &'t str,
}

/// The attribute-probe SEAM the interpreter walks through for the
/// element-own `data-*`/`aria-*` layers: where a probe's answer comes
/// from. The same shape as the token seam's [`TokenSource`], the
/// custom seam's [`ClassifyVariant`](crate::ClassifyVariant) and the
/// cache's [`DynStore`](crate::DynStore): the walk is generic over the
/// source, [`AttrState`] is the built-in (a static bit table), and a
/// host implements the ONE question itself (a DOM-backed bridge, a
/// component-state map, a test double recording what it was asked).
pub trait AttrProbe {
    /// The value the element carries for this attribute, or `None`
    /// when it carries none. The answer drives the CSS semantics the
    /// probes encode: a presence probe fires iff the answer exists, a
    /// value probe fires iff the answer EQUALS the folded value.
    fn probe(&self, kind: AttrKind, name: &str) -> Option<&str>;
}

/// The built-in [`AttrProbe`]: the element's attribute bits as a
/// borrowed, static table — the zero-allocation form of the plan §3.5
/// "typed state bits passed with the element". A per-frame host
/// borrows its bit rows into one table (the same borrow shape the
/// `Env`'s own fields take); an empty table is the no-bits element.
#[derive(Clone, Copy, Debug)]
pub struct AttrState<'t> {
    bits: &'t [AttrBit<'t>],
}

impl<'t> AttrState<'t> {
    /// The element's attribute bits, in table order (the first
    /// matching bit answers; a host that never repeats a
    /// (kind, name) pair needs no order — and the table is small by
    /// construction, one bit per probed attribute).
    pub const fn new(bits: &'t [AttrBit<'t>]) -> Self {
        Self { bits }
    }
}

impl AttrProbe for AttrState<'_> {
    fn probe(&self, kind: AttrKind, name: &str) -> Option<&str> {
        self.bits
            .iter()
            .find(|bit| bit.kind == kind && bit.name == name)
            .map(|bit| bit.value)
    }
}

// ---------------------------------------------------------------------------
// The container-bounds seam: ContainerProbe
// ---------------------------------------------------------------------------

/// One named container's inner width in rem — one row of the built-in
/// [`ContainerState`] table. The NAME is the `@md/card:` spelling's
/// `card` (the container context a component declares in the host's
/// layout); the WIDTH answers in the same unit the unnamed
/// [`Env::container_rem`] slot answers in (CSS `cqw`'s basis, one rem
/// per 100 — the walk compares against the breakpoints' rem values
/// directly).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ContainerBound<'t> {
    /// The container's name, verbatim (`"card"` for `@md/card:`).
    pub name: &'t str,
    /// The container's inner width in rem.
    pub rem: f32,
}

/// The container-bounds SEAM the interpreter walks through for the
/// NAMED container queries ([`Cond::Container`]/[`Cond::MaxContainer`]
/// with a name): where a named container's width comes from. The same
/// shape as the token seam's [`TokenSource`], the attribute seam's
/// [`AttrProbe`] and the cache's [`DynStore`](crate::DynStore): the
/// walk is generic over the source, [`ContainerState`] is the built-in
/// (a borrowed static table), and a host implements the ONE question
/// itself (a layout-phase bridge measuring its `@container` elements,
/// a component-tree walk, a test double recording what it was asked).
///
/// The seam's answer IS the CSS semantics: `Some(width)` compares
/// against the layer's breakpoint by the min/max boundary rules (AT and
/// above for the min forms, strictly below for the max forms — the
/// viewport breakpoints' own pinned semantics); `None` — the host knows
/// no such container — is a QUIET miss (a CSS selector that does not
/// match), never an exclusion note. The SOURCELESS forms (no
/// `*_containers` entry point) keep the honest exclusion instead: a
/// named layer they cannot answer reports
/// [`InterpNote::ContainerUnqueried`].
pub trait ContainerProbe {
    /// The inner width in rem of the nearest container with this NAME,
    /// or `None` when the host knows no such container ancestor. The
    /// name is the folded [`Cond`]'s own payload (the `@md/card:`
    /// spelling's `card`), verbatim.
    fn container(&self, name: &str) -> Option<f32>;
}

/// The built-in [`ContainerProbe`]: the named containers' widths as a
/// borrowed, static table — the zero-allocation form of the seam (the
/// [`AttrState`] precedent). A host borrows its container rows into one
/// table (the same borrow shape the `Env`'s own fields take); an empty
/// table answers `None` for every name — every named layer a QUIET
/// miss, every unnamed layer still the [`Env`]'s own question.
#[derive(Clone, Copy, Debug)]
pub struct ContainerState<'t> {
    bounds: &'t [ContainerBound<'t>],
}

impl<'t> ContainerState<'t> {
    /// The named containers' widths, in table order (the first
    /// matching name answers; a host that never repeats a name needs
    /// no order — and the table is small by construction, one row per
    /// container the layout declares).
    pub const fn new(bounds: &'t [ContainerBound<'t>]) -> Self {
        Self { bounds }
    }
}

impl ContainerProbe for ContainerState<'_> {
    fn container(&self, name: &str) -> Option<f32> {
        self.bounds
            .iter()
            .find(|bound| bound.name == name)
            .map(|bound| bound.rem)
    }
}

// ---------------------------------------------------------------------------
// The structural seam: StructProbe
// ---------------------------------------------------------------------------

/// The element's DOM-position facts — the ONE answer the structural
/// seam carries. Three fields cover every positional test the
/// [`Structural`](aeolus_ir::Structural) family encodes:
///
/// * `index` — the element's 0-based position among its parent's
///   element children (`first` is 0; CSS's 1-based ordinals subtract
///   the one);
/// * `sibling_count` — the parent's element-child count, THIS element
///   included (`last` is `index + 1 == count`; `only` is
///   `count == 1`);
/// * `empty` — the element carries no children (`:empty`; a fact about
///   the element's OWN children, independent of its position).
///
/// CSS parity: the first child (index 0) is `:nth-child(odd)` and
/// `:nth-child(1)`, the second (index 1) even.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct StructFacts {
    /// The element's 0-based child index, or `None` when the host does
    /// not know (an unknown index quietly misses every index-based
    /// test — a positional selector that cannot be evaluated never
    /// matches).
    pub index: Option<usize>,
    /// The parent's element-child count (this element included), or
    /// `None` when the host does not know.
    pub sibling_count: Option<usize>,
    /// The element carries no children (the `:empty` test).
    pub empty: bool,
}

impl StructFacts {
    /// The unknown-facts value: every index-based test quietly misses,
    /// `empty` false — the honest "the host answered but knows
    /// nothing" answer (a QUIET miss, never an exclusion note; the
    /// sourceless forms carry the note instead).
    pub const UNKNOWN: Self = Self {
        index: None,
        sibling_count: None,
        empty: false,
    };

    /// The facts for one positioned element: `index` 0-based,
    /// `sibling_count` the parent's element-child count (this element
    /// included), not empty.
    pub const fn new(index: usize, sibling_count: usize) -> Self {
        Self {
            index: Some(index),
            sibling_count: Some(sibling_count),
            empty: false,
        }
    }
}

/// The structural SEAM the interpreter walks through for the
/// [`Cond::Structural`](aeolus_ir::Cond::Structural) layers: where the
/// element's DOM-position facts come from. The same shape as the token
/// seam's [`TokenSource`], the attribute seam's [`AttrProbe`], the
/// container seam's [`ContainerProbe`] and the cache's
/// [`DynStore`](crate::DynStore): the walk is generic over the source,
/// [`StructFacts`] is the built-in (one Copy value), and a host
/// implements the ONE question itself (a layout bridge that knows the
/// element's position in its parent, a component-tree walk, a test
/// double recording what it was asked).
///
/// The seam's answer IS the CSS semantics: the facts decide the
/// positional tests (`first:` is index 0, `last:` the final child,
/// `odd:`/`even:` the 1-based parity, `only:` the single child,
/// `empty:` the childless element, `nth-N:`/`nth-last-N:` the 1-based
/// ordinals), an unknown field QUIETLY misses its tests (a positional
/// selector that cannot be evaluated never matches), and the
/// SOURCELESS forms (no `*_struct` entry point) keep the honest
/// exclusion instead: a structural layer they cannot answer reports
/// [`InterpNote::StructuralUnqueried`].
pub trait StructProbe {
    /// The element's DOM-position facts. Asked once per structural
    /// layer the walk meets (the answers are not cached — the trait
    /// object is a borrow, the facts are Copy).
    fn structure(&self) -> StructFacts;
}

impl StructProbe for StructFacts {
    fn structure(&self) -> StructFacts {
        *self
    }
}

/// The verdict one [`Structural`](aeolus_ir::Structural) test takes
/// under one element's facts — the CSS semantics the seam documents,
/// in one place so the interp walk and any host-side re-derivation
/// cannot drift (the GPUI walk consumes this through the crate
/// surface — the `one decide, two walks` discipline extended to the
/// structural seam's third walk).
pub fn structural_fires(test: aeolus_ir::Structural, facts: StructFacts) -> bool {
    use aeolus_ir::Structural;
    match test {
        Structural::First => facts.index == Some(0),
        Structural::Last => match (facts.index, facts.sibling_count) {
            (Some(index), Some(count)) => index + 1 == count,
            _ => false,
        },
        // 1-based CSS parity: index 0 is the first child, `odd`.
        Structural::Odd => facts.index.is_some_and(|index| index % 2 == 0),
        Structural::Even => facts.index.is_some_and(|index| index % 2 == 1),
        Structural::Only => facts.index == Some(0) && facts.sibling_count == Some(1),
        Structural::Empty => facts.empty,
        // The 1-based ordinal: `nth-3` is index 2. A non-positive
        // ordinal (the oracle's `parse_nth` accepts every `i32`) never
        // matches — the selector CSS never matches either. The compare
        // stays in the usize domain via `try_from` (no wrap casts).
        Structural::Nth(n) => usize::try_from(n)
            .ok()
            .and_then(|ordinal| ordinal.checked_sub(1))
            .is_some_and(|ordinal| facts.index == Some(ordinal)),
        Structural::NthLast(n) => usize::try_from(n).is_ok_and(|ordinal| {
            facts
                .index
                .zip(facts.sibling_count)
                .is_some_and(|(index, count)| count - index == ordinal)
        }),
    }
}

// ---------------------------------------------------------------------------
// The token seam: TokenSource
// ---------------------------------------------------------------------------

/// The token-resolution SEAM the interpreter walks through: where a
/// [`Val::Tok`] or [`Val::ModePair`] gets its concrete colour.
///
/// The same shape as the custom seam's
/// [`ClassifyVariant`](crate::ClassifyVariant) and the cache's
/// [`DynStore`](crate::DynStore): the walk is generic over the source,
/// [`TokenTable`] is the built-in, and a host implements the two
/// questions itself (a live theme bridge answering per frame, a test
/// double recording what it was asked). The mode rides every question —
/// a source that resolves half-selected colours needs it.
pub trait TokenSource {
    /// Resolve a runtime token slot ([`Val::Tok`] `{ id, alpha }`) to
    /// its value for the `dark` mode. The built-in table answers with
    /// the concrete [`Val::Rgba`] (the active half, premultiplied by the
    /// clamped alpha — one load + one multiply, plan §3.4).
    ///
    /// # Errors
    /// [`InterpError::UnknownToken`] for an id the source does not know;
    /// [`InterpError::AlphaOnPair`]/[`InterpError::AlphaNotFinite`] per
    /// the built-in table's audited refusals.
    fn resolve(&self, id: u16, alpha: f32, dark: bool) -> Result<Val, InterpError>;

    /// Resolve a T1 pair slot ([`Val::ModePair`]) to its `dark`-mode
    /// half. The built-in table answers with the concrete [`Val::Rgba`]
    /// from [`TokenTable::pair_at`] — no alpha arm (the pair form has
    /// none; alphas ride [`Val::Tok`]).
    ///
    /// # Errors
    /// [`InterpError::UnknownPair`] for a slot the source does not know
    /// (including a runtime-tier table, which carries no pairs).
    fn resolve_pair(&self, slot: u16, dark: bool) -> Result<Val, InterpError>;
}

impl TokenSource for TokenTable {
    fn resolve(&self, id: u16, alpha: f32, dark: bool) -> Result<Val, InterpError> {
        self.resolve_rgba(id, alpha, dark)
            .map(Val::Rgba)
            .map_err(|err| match err {
                ResolveError::UnknownId(id) => InterpError::UnknownToken { id },
                ResolveError::AlphaOnPair => InterpError::AlphaOnPair { id },
                ResolveError::AlphaNotFinite(alpha) => InterpError::AlphaNotFinite { alpha },
            })
    }

    fn resolve_pair(&self, slot: u16, dark: bool) -> Result<Val, InterpError> {
        self.pair_at(slot)
            .map(|(light, dark_rgba)| if dark { dark_rgba } else { light })
            .map(Val::Rgba)
            .ok_or(InterpError::UnknownPair { slot })
    }
}

// ---------------------------------------------------------------------------
// The typed failure set: InterpError
// ---------------------------------------------------------------------------

/// Why one declaration did not resolve. These are ERRORS, not reports:
/// a token value the source cannot answer is a caller/host bug (the
/// pipeline never emits one), so the walk stops and says so — the
/// no-silent-failures rule applied to the runtime half.
#[derive(Clone, Debug, PartialEq)]
pub enum InterpError {
    /// The theme-less [`interp`] met a token value ([`Val::Tok`] or
    /// [`Val::ModePair`]): the theme forms resolve through a
    /// [`TokenSource`], which [`interp_theme`] carries. The longhand
    /// that carried the value names the boundary.
    ThemeRequired {
        /// The longhand whose value is a theme form.
        prop: Prop,
    },
    /// The source does not know the token id (the built-in table maps
    /// [`ResolveError::UnknownId`]; ids come from
    /// [`TokenTable::id`]).
    UnknownToken {
        /// The out-of-range id.
        id: u16,
    },
    /// The source does not know the pair slot (the built-in table maps
    /// an out-of-range [`Val::ModePair`] through
    /// [`TokenTable::pair_at`]; a runtime-tier table carries no pairs).
    UnknownPair {
        /// The out-of-range slot.
        slot: u16,
    },
    /// An alpha rode a T1 pair token: the pair form has no alpha arm
    /// (the built-in table maps [`ResolveError::AlphaOnPair`]; the
    /// alpha rides [`Val::Tok`] on the `Runtime` tier).
    AlphaOnPair {
        /// The pair-carrying token id.
        id: u16,
    },
    /// The alpha was not finite (`NaN`/`inf`): the audited clamp cannot
    /// be honoured, so the resolve refuses instead of baking a NaN
    /// literal (the built-in table maps
    /// [`ResolveError::AlphaNotFinite`]).
    AlphaNotFinite {
        /// The non-finite alpha.
        alpha: f32,
    },
}

impl InterpError {
    /// The stable machine code (`aeolus::compile::interp::*`) a linter
    /// or editor keys on — the same strings the `miette` feature exposes.
    pub fn code(&self) -> &'static str {
        match self {
            InterpError::ThemeRequired { .. } => "aeolus::compile::interp_theme_required",
            InterpError::UnknownToken { .. } => "aeolus::compile::interp_unknown_token",
            InterpError::UnknownPair { .. } => "aeolus::compile::interp_unknown_pair",
            InterpError::AlphaOnPair { .. } => "aeolus::compile::interp_alpha_on_pair",
            InterpError::AlphaNotFinite { .. } => "aeolus::compile::interp_alpha_not_finite",
        }
    }

    /// The actionable help text, stable per variant.
    pub fn help(&self) -> &'static str {
        match self {
            InterpError::ThemeRequired { .. } => {
                "the token value renders through the theme pass: interp_theme() \
                 carries the TokenSource a var() reference or a dark pair \
                 resolves from"
            }
            InterpError::UnknownToken { .. } => {
                "token ids come from TokenTable::id(); an id the source does \
                 not know is a caller bug or a host-source gap, not a \
                 styling result"
            }
            InterpError::UnknownPair { .. } => {
                "pair slots come from the baked table's pair_at(); a slot the \
                 source does not know is a caller bug or a host-source gap \
                 (a runtime-tier table carries no pairs)"
            }
            InterpError::AlphaOnPair { .. } => {
                "use the Runtime tier (the alpha rides Val::Tok) or bake a \
                 per-alpha program; the pair form indexes a const (light, \
                 dark) table and has no alpha arm"
            }
            InterpError::AlphaNotFinite { .. } => {
                "the modifier grammar accepts finite decimals only (the \
                 audited clamp is 0..=1); NaN/inf modifiers are typed \
                 refusals upstream"
            }
        }
    }
}

impl fmt::Display for InterpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InterpError::ThemeRequired { prop } => write!(
                f,
                "the token value on {prop:?} resolves through the theme pass: \
                 interp_theme carries the TokenSource"
            ),
            InterpError::UnknownToken { id } => write!(f, "token id {id} is outside the source"),
            InterpError::UnknownPair { slot } => {
                write!(f, "pair slot {slot} is outside the source")
            }
            InterpError::AlphaOnPair { id } => write!(
                f,
                "an opacity modifier cannot ride the baked (light, dark) pair \
                 slot for token id {id}"
            ),
            InterpError::AlphaNotFinite { alpha } => {
                write!(f, "token alpha {alpha} is not finite")
            }
        }
    }
}

impl std::error::Error for InterpError {}

#[cfg(feature = "miette")]
impl miette::Diagnostic for InterpError {
    fn code(&self) -> Option<Box<dyn fmt::Display + '_>> {
        Some(Box::new(self.code()))
    }

    fn help(&self) -> Option<Box<dyn fmt::Display + '_>> {
        Some(Box::new(self.help()))
    }
}

// ---------------------------------------------------------------------------
// The pseudo-element render targets: PseudoTarget
// ---------------------------------------------------------------------------

/// The pseudo-element RENDER TARGET a [`Program`]'s pseudo layers
/// resolve to — the `selection:`/`placeholder:` spellings the fold
/// lands on [`Cond::Selection`]/[`Cond::Placeholder`]. A pseudo-element
/// styles a DIFFERENT render target than the element's own cascade (the
/// CSS arm renders it as a `::selection`/`::placeholder` rule), so its
/// effective declarations resolve in their OWN walk:
/// [`interp_pseudo`]/[`interp_pseudo_theme`] fire ONLY the layers
/// carrying the target's condition — the element's own layers (base,
/// states, breakpoints, the sibling pseudo) never participate, and the
/// answer is silent about them rather than noting them. The element
/// walk's [`InterpNote::PseudoElement`] note is the other half of the
/// same contract: the element cascade excludes the pseudo layers and
/// reports them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PseudoTarget {
    /// The `::selection` render target (the `selection:` variant).
    Selection,
    /// The `::placeholder` render target (the `placeholder:` variant).
    Placeholder,
}

impl PseudoTarget {
    /// The [`Cond`] the target's layers carry.
    const fn cond(self) -> Cond {
        match self {
            PseudoTarget::Selection => Cond::Selection,
            PseudoTarget::Placeholder => Cond::Placeholder,
        }
    }
}

// ---------------------------------------------------------------------------
// The honest exclusions: InterpNote
// ---------------------------------------------------------------------------

/// A layer the walk did NOT fold into the element's cascade, with the
/// reason — the interpreter's no-silent-failures surface. Notes are
/// REPORTS, not errors: the resolution is still a complete answer for
/// the element's own declarations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InterpNote {
    /// A `::selection`/`::placeholder` layer: pseudo-elements style a
    /// DIFFERENT render target, never the element's own cascade (the
    /// CSS arm renders them as `::selection`/`::placeholder` rules; the
    /// pseudo cascades themselves resolve through
    /// [`interp_pseudo`]/[`interp_pseudo_theme`], the [`PseudoTarget`]
    /// arm).
    PseudoElement {
        /// The excluded layer's condition.
        cond: Cond,
        /// The longhands the excluded layer sets.
        props: PropMask,
    },
    /// A container-conditioned layer the walk could not answer: either
    /// the UNNAMED form with no container context in the [`Env`], or a
    /// NAMED form under a sourceless entry point (the plain
    /// [`interp`]/[`interp_theme`]/`*_probes` forms — the
    /// `*_containers` forms carry the [`ContainerProbe`] seam, and a
    /// source-answered miss is a CSS-quiet skip instead). CSS
    /// semantics — no eligible container never matches — so the layer
    /// did not fire, reported so the exclusion is never silent.
    ContainerUnqueried {
        /// The unqueried layer's condition.
        cond: Cond,
        /// The longhands the unqueried layer sets.
        props: PropMask,
    },
    /// An attribute-probe layer evaluated under conditions the probe
    /// surface cannot answer — one of the two UNQUERIED cases:
    ///
    /// * the entry point carries no [`AttrProbe`] source (the plain
    ///   [`interp`]/[`interp_theme`] forms — the [`interp_probes`]/
    ///   [`interp_theme_probes`] forms carry the plan §3.5 typed
    ///   state bits, and their `data-*`/`aria-*` layers fire), or
    /// * the probe is a DESCENDANT shape ([`Cond::Has`]/
    ///   [`Cond::GroupHas`]/[`Cond::PeerHas`]) — the descendant shape
    ///   is not an attribute bit, and the plan §3.5 row keeps those
    ///   the diagnosed boundary (honest failure, kept).
    ///
    /// Either way the layer never fires, reported so the exclusion is
    /// never silent. A source-ANSWERED miss (the element does not
    /// carry the attribute, or carries another value) is NOT this
    /// note — a CSS selector that does not match is CSS-quiet.
    AttributeUnqueried {
        /// The unqueried layer's condition.
        cond: Cond,
        /// The longhands the unqueried layer sets.
        props: PropMask,
    },
    /// A structural layer ([`Cond::Structural`] — `first:`, `last:`,
    /// `odd:`, `even:`, `only:`, `empty:`, `nth-N:`, `nth-last-N:`) the
    /// walk could not answer: the entry point carries no
    /// [`StructProbe`] source (the plain [`interp`]/[`interp_theme`]/
    /// `*_probes`/`*_containers` forms — the `*_struct` forms carry the
    /// seam, and a source-ANSWERED miss is a CSS-quiet skip instead).
    /// The element's DOM-position facts are not frame state — the
    /// [`Env`] cannot answer them (they describe ONE element's position
    /// among its siblings, not the window) — so there is no sourceless
    /// fallback value. CSS semantics — a positional selector that
    /// cannot be evaluated never matches — so the layer did not fire,
    /// reported so the exclusion is never silent.
    StructuralUnqueried {
        /// The unqueried layer's condition.
        cond: Cond,
        /// The longhands the unqueried layer sets.
        props: PropMask,
    },
}

impl fmt::Display for InterpNote {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The longhand NAMES, not the mask's raw bits — the report is
        // for readers (the mask itself rides the struct for programs).
        let names: Vec<&str> = match self {
            InterpNote::PseudoElement { props, .. }
            | InterpNote::ContainerUnqueried { props, .. }
            | InterpNote::AttributeUnqueried { props, .. }
            | InterpNote::StructuralUnqueried { props, .. } => {
                props.to_vec().iter().map(|p| p.name()).collect()
            }
        };
        let props = names.join(", ");
        match self {
            InterpNote::PseudoElement { cond, .. } => write!(
                f,
                "the {cond:?} layer is a separate render target ({props} \
                 excluded from the element's own cascade)"
            ),
            InterpNote::ContainerUnqueried { cond, .. } => write!(
                f,
                "the {cond:?} layer never matched: the Env carries no \
                 container context ({props} unset)"
            ),
            InterpNote::AttributeUnqueried { cond, .. } => write!(
                f,
                "the {cond:?} layer never matched: the Env answers element \
                 states, not attribute/DOM probes ({props} unset)"
            ),
            InterpNote::StructuralUnqueried { cond, .. } => write!(
                f,
                "the {cond:?} layer never matched: no StructProbe source \
                 rides — the element's DOM-position facts are not frame \
                 state ({props} unset)"
            ),
        }
    }
}

// ---------------------------------------------------------------------------
// The result: Interped
// ---------------------------------------------------------------------------

/// The effective declarations one program resolves to under one probe
/// situation: the winning value per longhand (later fired layers beat
/// earlier, within a layer the later declaration wins — the cascade the
/// IR's specificity sort encodes), in `Prop` cascade order, plus the
/// honest exclusion notes.
#[derive(Clone, Debug, PartialEq)]
pub struct Interped {
    /// The resolved declarations, `Prop`-sorted, one per longhand. The
    /// theme forms are CONCRETE here ([`Val::Rgba`] under the built-in
    /// source); [`Val::Kw`] rides through (the codegen crate owns the
    /// keyword names, plan §3.3).
    pub decls: Vec<Decl>,
    /// The winning declaration's condition, PARALLEL to [`Interped::decls`]
    /// (same length, same order): the layer the cascade's winner came
    /// from — the row-relative [`Val::Kw`] annotation's key, since the
    /// keyword texts ride the `(cond, prop)` annotation stream. A host
    /// reading `decls[i]` alongside `conds[i]` knows WHICH condition
    /// won the longhand without re-walking the layers.
    pub conds: Vec<Cond>,
    /// The excluded layers, in program order (see [`InterpNote`]).
    pub notes: Vec<InterpNote>,
}

impl Interped {
    /// The longhands the resolution touches — the "what would this
    /// element style now" mask (the repaint-lane question the backends'
    /// diff story starts from).
    pub fn touched(&self) -> PropMask {
        let mut mask = PropMask::EMPTY;
        for decl in &self.decls {
            mask = mask.insert(decl.prop);
        }
        mask
    }
}

// ---------------------------------------------------------------------------
// The walk
// ---------------------------------------------------------------------------

/// Whether a layer fires under the env, stays quiet, or is excluded
/// with a note. Crate-visible so the program lane's style fold
/// (`style_walk`) shares the VERDICTS — one `decide`, two walks, so
/// the two cannot disagree on when a layer fires.
pub(crate) enum Fire {
    Yes,
    No,
    Note(InterpNote),
}

/// One condition's answer under the env. Min-width forms fire AT and
/// above their breakpoint's rem (`min-width: Nrem`); max forms strictly
/// below it (`width < Nrem`) — the CSS boundary semantics the tests
/// pin at the exact rem values. The element-own attribute probes ask
/// the [`AttrProbe`] seam when a source rides ([`interp_probes`]/
/// [`interp_theme_probes`]) — a hit FIRES, a miss is a quiet skip (a
/// CSS selector that does not match); with no source the layer is
/// excluded and REPORTED (the note the follow-up surface was named by).
/// The NAMED container queries ask the [`ContainerProbe`] seam the same
/// way ([`interp_containers`] and friends): a hit fires by the same
/// boundary rules against THAT container's width, a container the
/// source does not know is a quiet skip, and with no source the layer
/// is excluded and reported. The UNNAMED container forms keep asking
/// the [`Env::container_rem`] slot whatever the seam carries.
/// Crate-visible: the program lane's style fold shares the verdicts.
pub(crate) fn decide(
    cond: Cond,
    env: &Env,
    probes: Option<&dyn AttrProbe>,
    containers: Option<&dyn ContainerProbe>,
    structs: Option<&dyn StructProbe>,
    props: PropMask,
) -> Fire {
    let fires = |yes: bool| if yes { Fire::Yes } else { Fire::No };
    match cond {
        Cond::Base => Fire::Yes,
        Cond::Dark => fires(env.dark),
        Cond::Hover => fires(env.hover),
        Cond::Focus => fires(env.focus),
        Cond::FocusVisible => fires(env.focus_visible),
        Cond::FocusWithin => fires(env.focus_within),
        Cond::Active => fires(env.active),
        Cond::Disabled => fires(env.disabled),
        Cond::Visited => fires(env.visited),
        Cond::Checked => fires(env.checked),
        Cond::PlaceholderShown => fires(env.placeholder_shown),
        Cond::GroupHover => fires(env.group_hover),
        Cond::GroupFocus => fires(env.group_focus),
        Cond::PeerHover => fires(env.peer_hover),
        Cond::PeerFocus => fires(env.peer_focus),
        Cond::Bp(bp) | Cond::Min(bp) => fires(env.viewport_rem >= bp.rem()),
        Cond::Max(bp) => fires(env.viewport_rem < bp.rem()),
        // The UNNAMED container question answers from the nearest
        // eligible container the Env carries. The NAMED question needs
        // THAT container's width — the seam answers it (a hit fires by
        // the same boundary rules, an unknown container is a quiet
        // miss), and the sourceless forms report (the honest boundary;
        // the note says which container went unqueried).
        Cond::Container {
            bp,
            name: Some(name),
        } => match containers {
            Some(source) => fires(
                source
                    .container(name)
                    .is_some_and(|width| width >= bp.rem()),
            ),
            None => Fire::Note(InterpNote::ContainerUnqueried { cond, props }),
        },
        Cond::Container { bp, name: None } => match env.container_rem {
            Some(width) => fires(width >= bp.rem()),
            None => Fire::Note(InterpNote::ContainerUnqueried { cond, props }),
        },
        Cond::MaxContainer {
            bp,
            name: Some(name),
        } => match containers {
            Some(source) => fires(source.container(name).is_some_and(|width| width < bp.rem())),
            None => Fire::Note(InterpNote::ContainerUnqueried { cond, props }),
        },
        Cond::MaxContainer { bp, name: None } => match env.container_rem {
            Some(width) => fires(width < bp.rem()),
            None => Fire::Note(InterpNote::ContainerUnqueried { cond, props }),
        },
        // The element-own attribute probes answer through the seam
        // (plan §3.5's typed state bits): presence fires on any value,
        // the value spelling fires on equality, a miss is CSS-quiet.
        // The descendant shapes (`has-`/`group-has-`/`peer-has-`) are
        // NOT attribute bits — the plan §3.5 row keeps them diagnosed
        // (reported, honest failure kept), with or without a source.
        Cond::Data { name, value } => match probes {
            Some(source) => fires(match source.probe(AttrKind::Data, name) {
                Some(answer) => value.is_none_or(|folded| answer == folded),
                None => false,
            }),
            None => Fire::Note(InterpNote::AttributeUnqueried { cond, props }),
        },
        Cond::Aria { name, value } => match probes {
            Some(source) => fires(match source.probe(AttrKind::Aria, name) {
                Some(answer) => value.is_none_or(|folded| answer == folded),
                None => false,
            }),
            None => Fire::Note(InterpNote::AttributeUnqueried { cond, props }),
        },
        Cond::Has(_) | Cond::GroupHas(_) | Cond::PeerHas(_) => {
            Fire::Note(InterpNote::AttributeUnqueried { cond, props })
        }
        // The structural probes ask the DOM-position seam (the
        // position half of the plan §3.5 typed-state-bits row): the
        // facts decide — an unknown field quietly misses its tests (a
        // positional selector that cannot be evaluated never matches)
        // — and the sourceless forms report (the element's position is
        // not frame state; the Env cannot answer it).
        Cond::Structural(test) => match structs {
            Some(source) => fires(structural_fires(test, source.structure())),
            None => Fire::Note(InterpNote::StructuralUnqueried { cond, props }),
        },
        // The pseudo-elements are render TARGETS, not element state: the
        // interp resolves the element's own cascade, so these layers are
        // excluded and reported — never folded in, never an error.
        Cond::Selection | Cond::Placeholder => {
            Fire::Note(InterpNote::PseudoElement { cond, props })
        }
    }
}

/// The shared front-to-back walk: fire the layers the env answers yes
/// to, resolve their values through the caller's resolver, keep the
/// last value per longhand in `Prop` order. The attribute-probe seam
/// rides along as the (optional) probe source — `None` keeps the
/// sourceless exclusion notes, `Some` answers the element-own probes —
/// and the container-bounds seam rides beside it (the NAMED container
/// queries ask it; `None` keeps their sourceless exclusion notes).
fn walk(
    program: &Program,
    env: &Env,
    probes: Option<&dyn AttrProbe>,
    containers: Option<&dyn ContainerProbe>,
    structs: Option<&dyn StructProbe>,
    resolve: impl Fn(Prop, Val) -> Result<Val, InterpError>,
) -> Result<Interped, InterpError> {
    #[cfg(feature = "tracing")]
    let _span = tracing::debug_span!("interp", layers = program.layers.len()).entered();
    let mut decls: Vec<Decl> = Vec::new();
    let mut conds: Vec<Cond> = Vec::new();
    let mut notes = Vec::new();
    for layer in program.layers {
        match decide(layer.cond, env, probes, containers, structs, layer.mask) {
            Fire::Yes => {
                for decl in layer.decls {
                    let val = resolve(decl.prop, decl.val)?;
                    // Keep the winners Prop-sorted: a fired layer's
                    // declaration overwrites the longhand's earlier
                    // value in place, or inserts at the sorted slot.
                    // The winner's condition rides the parallel conds
                    // (the winner attribution the keyword annotations
                    // key on).
                    match decls.binary_search_by_key(&decl.prop, |d| d.prop) {
                        Ok(index) => {
                            decls[index].val = val;
                            conds[index] = layer.cond;
                        }
                        Err(index) => {
                            decls.insert(
                                index,
                                Decl {
                                    prop: decl.prop,
                                    val,
                                },
                            );
                            conds.insert(index, layer.cond);
                        }
                    }
                }
                #[cfg(feature = "tracing")]
                tracing::debug!(cond = ?layer.cond, fired = layer.decls.len(), "layer fired");
            }
            Fire::No => {}
            Fire::Note(note) => {
                #[cfg(feature = "tracing")]
                tracing::debug!(cond = ?layer.cond, note = %note, "layer excluded");
                notes.push(note);
            }
        }
    }
    Ok(Interped {
        decls,
        conds,
        notes,
    })
}

/// The pseudo-element walk: the TARGET's own cascade. Only the layers
/// carrying the target's [`Cond`] participate — a pseudo-element's
/// rules style a different render target, so the element's own layers
/// (base, states, breakpoints, the sibling pseudo) are simply not part
/// of the answer, silent rather than noted (the element walk's
/// [`InterpNote::PseudoElement`] note reports them on ITS side). Same
/// cascade shape as [`walk`]: within the target's layers, later fired
/// declarations win per longhand, the winners come out `Prop`-sorted,
/// and the winner condition is the pseudo-element's own.
fn walk_pseudo(
    program: &Program,
    target: PseudoTarget,
    resolve: impl Fn(Prop, Val) -> Result<Val, InterpError>,
) -> Result<Interped, InterpError> {
    #[cfg(feature = "tracing")]
    let _span = tracing::debug_span!("interp_pseudo", layers = program.layers.len()).entered();
    let wanted = target.cond();
    let mut decls: Vec<Decl> = Vec::new();
    let mut conds: Vec<Cond> = Vec::new();
    for layer in program.layers {
        if layer.cond != wanted {
            continue;
        }
        for decl in layer.decls {
            let val = resolve(decl.prop, decl.val)?;
            match decls.binary_search_by_key(&decl.prop, |d| d.prop) {
                Ok(index) => {
                    decls[index].val = val;
                }
                Err(index) => {
                    decls.insert(
                        index,
                        Decl {
                            prop: decl.prop,
                            val,
                        },
                    );
                    conds.insert(index, wanted);
                }
            }
        }
        #[cfg(feature = "tracing")]
        tracing::debug!(cond = ?wanted, fired = layer.decls.len(), "pseudo layer fired");
    }
    Ok(Interped {
        decls,
        conds,
        notes: Vec::new(),
    })
}

/// Resolve a compiled [`Program`] to its effective declarations under
/// one probe situation — the theme-LESS form: token values
/// ([`Val::Tok`]/[`Val::ModePair`]) refuse with the typed
/// [`InterpError::ThemeRequired`] boundary (they are the theme pass's
/// forms; [`interp_theme`] carries the source they resolve from).
///
/// The walk is pure: same program + env, same answer; the program is
/// borrowed, never mutated (the [`crate::DynCache`] loop compiles once
/// and interps per frame).
///
/// # Errors
/// [`InterpError::ThemeRequired`] when a declaration's value is a theme
/// form and no source rides along.
pub fn interp(program: &Program, env: &Env) -> Result<Interped, InterpError> {
    #[cfg(feature = "tracing")]
    let _span = tracing::debug_span!("interp_themeless", layers = program.layers.len()).entered();
    walk(program, env, None, None, None, |prop, val| match val {
        Val::Tok { .. } | Val::ModePair(_) => Err(InterpError::ThemeRequired { prop }),
        concrete => Ok(concrete),
    })
}

/// Resolve a compiled [`Program`] to its effective declarations under
/// one probe situation and one [`TokenSource`] — the themed form: every
/// [`Val::Tok`]/[`Val::ModePair`] asks the source (the built-in
/// [`TokenTable`] answers with concrete [`Val::Rgba`]s; a host source
/// answers for itself).
///
/// # Errors
/// The source's own typed failures ([`InterpError::UnknownToken`],
/// [`InterpError::UnknownPair`], [`InterpError::AlphaOnPair`],
/// [`InterpError::AlphaNotFinite`] under the built-in table) — the walk
/// stops and reports, never second-guesses.
pub fn interp_theme(
    program: &Program,
    tokens: &impl TokenSource,
    env: &Env,
) -> Result<Interped, InterpError> {
    #[cfg(feature = "tracing")]
    let _span = tracing::debug_span!("interp_theme", layers = program.layers.len()).entered();
    walk(program, env, None, None, None, |_, val| match val {
        Val::Tok { id, alpha } => tokens.resolve(id, alpha, env.dark),
        Val::ModePair(slot) => tokens.resolve_pair(slot, env.dark),
        concrete => Ok(concrete),
    })
}

/// Resolve a compiled [`Program`] to its effective declarations under
/// one probe situation and one [`AttrProbe`] — the attribute-probe
/// half of the seam family (the `compile_with` convention's `*_with`
/// shape): the element-own `data-*`/`aria-*` layers ask the source
/// (the built-in [`AttrState`] carries the element's attribute bits; a
/// host wires its own). A hit FIRES the layer; a miss is a QUIET skip
/// — a CSS selector that does not match, never an exclusion — and the
/// descendant probes (`has-`/`group-has-`/`peer-has-`) stay the plan
/// §3.5 diagnosed boundary: reported as
/// [`InterpNote::AttributeUnqueried`], honest failure kept.
///
/// The walk stays pure: same program + env + probe source, same
/// answer.
///
/// # Errors
/// [`InterpError::ThemeRequired`] when a declaration's value is a
/// theme form and no token source rides along (use
/// [`interp_theme_probes`] for the themed form).
pub fn interp_probes(
    program: &Program,
    env: &Env,
    probes: &impl AttrProbe,
) -> Result<Interped, InterpError> {
    #[cfg(feature = "tracing")]
    let _span = tracing::debug_span!("interp_probes", layers = program.layers.len()).entered();
    walk(
        program,
        env,
        Some(probes as &dyn AttrProbe),
        None,
        None,
        |prop, val| match val {
            Val::Tok { .. } | Val::ModePair(_) => Err(InterpError::ThemeRequired { prop }),
            concrete => Ok(concrete),
        },
    )
}

/// The themed form of [`interp_probes`]: one [`TokenSource`] resolves
/// the theme forms, one [`AttrProbe`] answers the element-own
/// attribute probes — both seams ride one walk (the DynCache loop
/// compiles once and interps per frame with both).
///
/// # Errors
/// The token source's own typed failures ([`InterpError::UnknownToken`],
/// [`InterpError::UnknownPair`], [`InterpError::AlphaOnPair`],
/// [`InterpError::AlphaNotFinite`] under the built-in table) — the walk
/// stops and reports, never second-guesses.
pub fn interp_theme_probes(
    program: &Program,
    tokens: &impl TokenSource,
    env: &Env,
    probes: &impl AttrProbe,
) -> Result<Interped, InterpError> {
    #[cfg(feature = "tracing")]
    let _span =
        tracing::debug_span!("interp_theme_probes", layers = program.layers.len()).entered();
    walk(
        program,
        env,
        Some(probes as &dyn AttrProbe),
        None,
        None,
        |_, val| match val {
            Val::Tok { id, alpha } => tokens.resolve(id, alpha, env.dark),
            Val::ModePair(slot) => tokens.resolve_pair(slot, env.dark),
            concrete => Ok(concrete),
        },
    )
}

/// Resolve a compiled [`Program`] to its effective declarations under
/// one probe situation and one [`ContainerProbe`] — the named-
/// container half of the seam family (the `compile_with` convention's
/// `*_with` shape): the NAMED container queries ask the source (the
/// built-in [`ContainerState`] carries the named containers' widths; a
/// host wires its own — a layout-phase bridge, a test double). A hit
/// fires by the same AT/strictly-below boundary rules the viewport
/// breakpoints obey; a container the source does not know is a QUIET
/// skip (a CSS selector that does not match, never an exclusion). The
/// UNNAMED container forms keep answering from [`Env::container_rem`].
///
/// The walk stays pure: same program + env + source, same answer.
///
/// # Errors
/// [`InterpError::ThemeRequired`] when a declaration's value is a theme
/// form and no token source rides along (use
/// [`interp_theme_containers`] for the themed form).
pub fn interp_containers(
    program: &Program,
    env: &Env,
    containers: &impl ContainerProbe,
) -> Result<Interped, InterpError> {
    #[cfg(feature = "tracing")]
    let _span = tracing::debug_span!("interp_containers", layers = program.layers.len()).entered();
    walk(
        program,
        env,
        None,
        Some(containers as &dyn ContainerProbe),
        None,
        |prop, val| match val {
            Val::Tok { .. } | Val::ModePair(_) => Err(InterpError::ThemeRequired { prop }),
            concrete => Ok(concrete),
        },
    )
}

/// The themed form of [`interp_containers`]: one [`TokenSource`]
/// resolves the theme forms, one [`ContainerProbe`] answers the named
/// container queries — both seams ride one walk (the DynCache loop
/// compiles once and interps per frame with both).
///
/// # Errors
/// The token source's own typed failures ([`InterpError::UnknownToken`],
/// [`InterpError::UnknownPair`], [`InterpError::AlphaOnPair`],
/// [`InterpError::AlphaNotFinite`] under the built-in table) — the walk
/// stops and reports, never second-guesses.
pub fn interp_theme_containers(
    program: &Program,
    tokens: &impl TokenSource,
    env: &Env,
    containers: &impl ContainerProbe,
) -> Result<Interped, InterpError> {
    #[cfg(feature = "tracing")]
    let _span =
        tracing::debug_span!("interp_theme_containers", layers = program.layers.len()).entered();
    walk(
        program,
        env,
        None,
        Some(containers as &dyn ContainerProbe),
        None,
        |_, val| match val {
            Val::Tok { id, alpha } => tokens.resolve(id, alpha, env.dark),
            Val::ModePair(slot) => tokens.resolve_pair(slot, env.dark),
            concrete => Ok(concrete),
        },
    )
}

/// The probe-composed form of [`interp_containers`]: one [`AttrProbe`]
/// answers the element-own attribute probes, one [`ContainerProbe`]
/// answers the named container queries — the full themeless walk.
///
/// # Errors
/// [`InterpError::ThemeRequired`] when a declaration's value is a theme
/// form and no token source rides along (use
/// [`interp_theme_probes_containers`] for the themed form).
pub fn interp_probes_containers(
    program: &Program,
    env: &Env,
    probes: &impl AttrProbe,
    containers: &impl ContainerProbe,
) -> Result<Interped, InterpError> {
    #[cfg(feature = "tracing")]
    let _span =
        tracing::debug_span!("interp_probes_containers", layers = program.layers.len()).entered();
    walk(
        program,
        env,
        Some(probes as &dyn AttrProbe),
        Some(containers as &dyn ContainerProbe),
        None,
        |prop, val| match val {
            Val::Tok { .. } | Val::ModePair(_) => Err(InterpError::ThemeRequired { prop }),
            concrete => Ok(concrete),
        },
    )
}

/// The FULL form: one [`TokenSource`] for the theme forms, one
/// [`AttrProbe`] for the element-own attribute probes, one
/// [`ContainerProbe`] for the named container queries — every seam the
/// walk consults rides one call (the DynCache loop's own shape).
///
/// # Errors
/// The token source's own typed failures (see [`interp_theme_containers`])
/// — the walk stops and reports, never second-guesses.
pub fn interp_theme_probes_containers(
    program: &Program,
    tokens: &impl TokenSource,
    env: &Env,
    probes: &impl AttrProbe,
    containers: &impl ContainerProbe,
) -> Result<Interped, InterpError> {
    #[cfg(feature = "tracing")]
    let _span = tracing::debug_span!(
        "interp_theme_probes_containers",
        layers = program.layers.len()
    )
    .entered();
    walk(
        program,
        env,
        Some(probes as &dyn AttrProbe),
        Some(containers as &dyn ContainerProbe),
        None,
        |_, val| match val {
            Val::Tok { id, alpha } => tokens.resolve(id, alpha, env.dark),
            Val::ModePair(slot) => tokens.resolve_pair(slot, env.dark),
            concrete => Ok(concrete),
        },
    )
}

/// Resolve a compiled [`Program`] to its effective declarations under
/// one probe situation and one [`StructProbe`] — the structural half
/// of the seam family (the `compile_with` convention's `*_with` shape):
/// the [`Cond::Structural`](aeolus_ir::Cond::Structural) layers ask the
/// source (the built-in [`StructFacts`] carries the element's
/// DOM-position facts; a host wires its own — a layout bridge, a test
/// double). The facts DECIDE the positional tests (`first:` is index 0,
/// `last:` the final child, `odd:`/`even:` the 1-based parity, `only:`
/// the single child, `empty:` the childless element, `nth-N:`/
/// `nth-last-N:` the 1-based ordinals); an unknown field quietly
/// misses its tests (a positional selector that cannot be evaluated
/// never matches). The sourceless forms keep the honest exclusion
/// ([`InterpNote::StructuralUnqueried`]).
///
/// The walk stays pure: same program + env + source, same answer.
///
/// # Errors
/// [`InterpError::ThemeRequired`] when a declaration's value is a theme
/// form and no token source rides along (use [`interp_theme_struct`]
/// for the themed form).
pub fn interp_struct(
    program: &Program,
    env: &Env,
    structs: &impl StructProbe,
) -> Result<Interped, InterpError> {
    #[cfg(feature = "tracing")]
    let _span = tracing::debug_span!("interp_struct", layers = program.layers.len()).entered();
    walk(
        program,
        env,
        None,
        None,
        Some(structs as &dyn StructProbe),
        |prop, val| match val {
            Val::Tok { .. } | Val::ModePair(_) => Err(InterpError::ThemeRequired { prop }),
            concrete => Ok(concrete),
        },
    )
}

/// The themed form of [`interp_struct`]: one [`TokenSource`] resolves
/// the theme forms, one [`StructProbe`] answers the structural layers —
/// both seams ride one walk (the DynCache loop compiles once and
/// interps per frame with both).
///
/// # Errors
/// The token source's own typed failures ([`InterpError::UnknownToken`],
/// [`InterpError::UnknownPair`], [`InterpError::AlphaOnPair`],
/// [`InterpError::AlphaNotFinite`] under the built-in table) — the walk
/// stops and reports, never second-guesses.
pub fn interp_theme_struct(
    program: &Program,
    tokens: &impl TokenSource,
    env: &Env,
    structs: &impl StructProbe,
) -> Result<Interped, InterpError> {
    #[cfg(feature = "tracing")]
    let _span =
        tracing::debug_span!("interp_theme_struct", layers = program.layers.len()).entered();
    walk(
        program,
        env,
        None,
        None,
        Some(structs as &dyn StructProbe),
        |_, val| match val {
            Val::Tok { id, alpha } => tokens.resolve(id, alpha, env.dark),
            Val::ModePair(slot) => tokens.resolve_pair(slot, env.dark),
            concrete => Ok(concrete),
        },
    )
}

/// The probe-composed form of [`interp_struct`]: one [`AttrProbe`]
/// answers the element-own attribute probes, one [`StructProbe`]
/// answers the structural layers — both element-own seams ride one
/// walk.
///
/// # Errors
/// [`InterpError::ThemeRequired`] when a declaration's value is a theme
/// form and no token source rides along (use
/// [`interp_theme_probes_struct`] for the themed form).
pub fn interp_probes_struct(
    program: &Program,
    env: &Env,
    probes: &impl AttrProbe,
    structs: &impl StructProbe,
) -> Result<Interped, InterpError> {
    #[cfg(feature = "tracing")]
    let _span =
        tracing::debug_span!("interp_probes_struct", layers = program.layers.len()).entered();
    walk(
        program,
        env,
        Some(probes as &dyn AttrProbe),
        None,
        Some(structs as &dyn StructProbe),
        |prop, val| match val {
            Val::Tok { .. } | Val::ModePair(_) => Err(InterpError::ThemeRequired { prop }),
            concrete => Ok(concrete),
        },
    )
}

/// The themed form of [`interp_probes_struct`]: one [`TokenSource`],
/// one [`AttrProbe`], one [`StructProbe`] — the element-own seams and
/// the token forms ride one walk.
///
/// # Errors
/// The token source's own typed failures (see [`interp_theme_probes`])
/// — the walk stops and reports, never second-guesses.
pub fn interp_theme_probes_struct(
    program: &Program,
    tokens: &impl TokenSource,
    env: &Env,
    probes: &impl AttrProbe,
    structs: &impl StructProbe,
) -> Result<Interped, InterpError> {
    #[cfg(feature = "tracing")]
    let _span =
        tracing::debug_span!("interp_theme_probes_struct", layers = program.layers.len()).entered();
    walk(
        program,
        env,
        Some(probes as &dyn AttrProbe),
        None,
        Some(structs as &dyn StructProbe),
        |_, val| match val {
            Val::Tok { id, alpha } => tokens.resolve(id, alpha, env.dark),
            Val::ModePair(slot) => tokens.resolve_pair(slot, env.dark),
            concrete => Ok(concrete),
        },
    )
}

/// The container-composed form of [`interp_struct`]: one
/// [`ContainerProbe`] answers the named container queries, one
/// [`StructProbe`] answers the structural layers.
///
/// # Errors
/// [`InterpError::ThemeRequired`] when a declaration's value is a theme
/// form and no token source rides along (use
/// [`interp_theme_containers_struct`] for the themed form).
pub fn interp_containers_struct(
    program: &Program,
    env: &Env,
    containers: &impl ContainerProbe,
    structs: &impl StructProbe,
) -> Result<Interped, InterpError> {
    #[cfg(feature = "tracing")]
    let _span =
        tracing::debug_span!("interp_containers_struct", layers = program.layers.len()).entered();
    walk(
        program,
        env,
        None,
        Some(containers as &dyn ContainerProbe),
        Some(structs as &dyn StructProbe),
        |prop, val| match val {
            Val::Tok { .. } | Val::ModePair(_) => Err(InterpError::ThemeRequired { prop }),
            concrete => Ok(concrete),
        },
    )
}

/// The themed form of [`interp_containers_struct`]: one [`TokenSource`],
/// one [`ContainerProbe`], one [`StructProbe`].
///
/// # Errors
/// The token source's own typed failures (see [`interp_theme_containers`])
/// — the walk stops and reports, never second-guesses.
pub fn interp_theme_containers_struct(
    program: &Program,
    tokens: &impl TokenSource,
    env: &Env,
    containers: &impl ContainerProbe,
    structs: &impl StructProbe,
) -> Result<Interped, InterpError> {
    #[cfg(feature = "tracing")]
    let _span = tracing::debug_span!(
        "interp_theme_containers_struct",
        layers = program.layers.len()
    )
    .entered();
    walk(
        program,
        env,
        None,
        Some(containers as &dyn ContainerProbe),
        Some(structs as &dyn StructProbe),
        |_, val| match val {
            Val::Tok { id, alpha } => tokens.resolve(id, alpha, env.dark),
            Val::ModePair(slot) => tokens.resolve_pair(slot, env.dark),
            concrete => Ok(concrete),
        },
    )
}

/// The probe-and-container-composed form of [`interp_struct`]: one
/// [`AttrProbe`], one [`ContainerProbe`], one [`StructProbe`] — the
/// themeless walk's every element-own seam rides one call.
///
/// # Errors
/// [`InterpError::ThemeRequired`] when a declaration's value is a theme
/// form and no token source rides along (use
/// [`interp_theme_probes_containers_struct`] for the themed form).
pub fn interp_probes_containers_struct(
    program: &Program,
    env: &Env,
    probes: &impl AttrProbe,
    containers: &impl ContainerProbe,
    structs: &impl StructProbe,
) -> Result<Interped, InterpError> {
    #[cfg(feature = "tracing")]
    let _span = tracing::debug_span!(
        "interp_probes_containers_struct",
        layers = program.layers.len()
    )
    .entered();
    walk(
        program,
        env,
        Some(probes as &dyn AttrProbe),
        Some(containers as &dyn ContainerProbe),
        Some(structs as &dyn StructProbe),
        |prop, val| match val {
            Val::Tok { .. } | Val::ModePair(_) => Err(InterpError::ThemeRequired { prop }),
            concrete => Ok(concrete),
        },
    )
}

/// The FULL walk: one [`TokenSource`] for the theme forms, one
/// [`AttrProbe`] for the element-own attribute probes, one
/// [`ContainerProbe`] for the named container queries, one
/// [`StructProbe`] for the structural layers — every seam the walk
/// consults rides one call (the DynCache loop's own shape).
///
/// # Errors
/// The token source's own typed failures (see [`interp_theme_containers`])
/// — the walk stops and reports, never second-guesses.
pub fn interp_theme_probes_containers_struct(
    program: &Program,
    tokens: &impl TokenSource,
    env: &Env,
    probes: &impl AttrProbe,
    containers: &impl ContainerProbe,
    structs: &impl StructProbe,
) -> Result<Interped, InterpError> {
    #[cfg(feature = "tracing")]
    let _span = tracing::debug_span!(
        "interp_theme_probes_containers_struct",
        layers = program.layers.len()
    )
    .entered();
    walk(
        program,
        env,
        Some(probes as &dyn AttrProbe),
        Some(containers as &dyn ContainerProbe),
        Some(structs as &dyn StructProbe),
        |_, val| match val {
            Val::Tok { id, alpha } => tokens.resolve(id, alpha, env.dark),
            Val::ModePair(slot) => tokens.resolve_pair(slot, env.dark),
            concrete => Ok(concrete),
        },
    )
}

/// Resolve a compiled [`Program`]'s PSEUDO-ELEMENT cascade for one
/// [`PseudoTarget`] — the theme-LESS form: the `::selection`/
/// `::placeholder` render target's own effective declarations (only the
/// layers carrying the target's condition participate; the element's
/// own layers are a different target, silent here, noted by the
/// element walk). Token values ([`Val::Tok`]/[`Val::ModePair`]) refuse
/// with the typed [`InterpError::ThemeRequired`] boundary — they are
/// the theme pass's forms; [`interp_pseudo_theme`] carries the source.
/// The walk takes no [`Env`]: a pseudo layer fires by its condition
/// alone (the flat fold collapses stacked variants onto the pseudo
/// condition), and the themeless resolution consults nothing else —
/// the themed form's `env` rides the pair selection, not the firing.
///
/// The walk stays pure: same program + target, same answer.
///
/// # Errors
/// [`InterpError::ThemeRequired`] when a declaration's value is a theme
/// form and no source rides along.
pub fn interp_pseudo(program: &Program, target: PseudoTarget) -> Result<Interped, InterpError> {
    walk_pseudo(program, target, |prop, val| match val {
        Val::Tok { .. } | Val::ModePair(_) => Err(InterpError::ThemeRequired { prop }),
        concrete => Ok(concrete),
    })
}

/// The themed form of [`interp_pseudo`]: the [`PseudoTarget`]'s own
/// cascade with one [`TokenSource`] resolving the theme forms — the
/// same two questions the element walk asks, on the pseudo render
/// target (the mode still selects the pair halves; the `env` rides the
/// resolution, not the firing — a pseudo layer fires by its condition
/// alone).
///
/// # Errors
/// The token source's own typed failures ([`InterpError::UnknownToken`],
/// [`InterpError::UnknownPair`], [`InterpError::AlphaOnPair`],
/// [`InterpError::AlphaNotFinite`] under the built-in table) — the walk
/// stops and reports, never second-guesses.
pub fn interp_pseudo_theme(
    program: &Program,
    tokens: &impl TokenSource,
    env: &Env,
    target: PseudoTarget,
) -> Result<Interped, InterpError> {
    walk_pseudo(program, target, |_, val| match val {
        Val::Tok { id, alpha } => tokens.resolve(id, alpha, env.dark),
        Val::ModePair(slot) => tokens.resolve_pair(slot, env.dark),
        concrete => Ok(concrete),
    })
}
