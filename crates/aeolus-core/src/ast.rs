//! AST for a single Tailwind v4 class token — the v0.2 no-alloc token shape.
//!
//! v0.2 changes (plan §3.1):
//! * [`ClassToken::variants`] is a stack-inlined buffer ([`Inline`]) — no
//!   per-token heap allocation for the variant stack;
//! * [`ClassToken::elements`] is a stack-inlined segment list
//!   ([`Segments`]) that derefs to `&[&str]`, so slice patterns and
//!   `.iter()` keep working;
//! * [`ClassToken::modifier`] is the **raw** `/…` suffix (`"2"`,
//!   `"[0.37]"`, `"6"`, `"tight"`, `"(--o)"`) — the parser no longer
//!   eagerly decides "opacity" (fixes F1/F6/F7 at the source);
//! * [`ClassToken::arbitrary`] models `[value]`, `[hint:value]` and the
//!   v4.1 `(--var)` shorthand;
//! * the variant grammar covers the full v4 set: states, media,
//!   breakpoints, `min-*`/`max-*`, `@container` forms, `group-*`/`peer-*`
//!   (with labels), `has-*`, `not-*` (recursive), `data-*`, `aria-*`,
//!   structural (`first`/`odd`/`nth-*`/…), pseudo-elements, and arbitrary
//!   `[&…]` selectors. Unknown identifiers become
//!   [`NamedVariant::Custom`] and are resolved later against theme
//!   breakpoints and `@custom-variant` rules;
//! * [`ClassToken::body`] keeps the raw utility head+value slice so the
//!   resolver can slice instead of `join("-")` (hot path, plan §3.4.1).

/// The sentinel capacity of [`ClassToken::variants`]. More than 8 stacked
/// variants on one class is pathological; such a class parses as
/// `Unparseable` (a per-class warning, never a panic).
pub const INLINE_VARIANTS: usize = 8;

/// The sentinel capacity of [`ClassToken::elements`]. Real utility bodies
/// have ≤ 6 dash-separated elements (`underline-offset-4` = 3); overflow is
/// a per-class warning, never a panic.
pub const INLINE_ELEMENTS: usize = 10;

/// One class token, e.g. `dark:group-hover:bg-primary/90!`.
#[derive(Clone, Debug, PartialEq)]
pub struct ClassToken<'a> {
    /// Variants in **source order** (left-to-right stacking, v4 semantics).
    /// The first variant is the outermost condition. Stack-inlined: up to
    /// [`INLINE_VARIANTS`] entries without touching the heap.
    pub variants: Inline<Variant<'a>, INLINE_VARIANTS>,
    /// Dash-separated utility elements, e.g. `["bg", "primary"]`,
    /// `["rounded", "t", "lg"]`. Derefs to `&[&'a str]`. Stack-inlined.
    pub elements: Segments<'a, INLINE_ELEMENTS>,
    /// The raw utility slice (variants, `-` sign, modifier and arbitrary
    /// brackets stripped): `"bg-primary"` for `dark:bg-primary/90`. Lets
    /// the resolver slice head/value instead of joining elements.
    pub body: &'a str,
    /// The `[...]` / `(...)` arbitrary group, when present.
    pub arbitrary: Option<Arbitrary<'a>>,
    /// The **raw** `/modifier` suffix: `"2"` for `w-1/2`, `"[0.37]"` for
    /// `bg-primary/[0.37]`, `"6"` for `text-sm/6`, `"(--o)"` for
    /// `bg-red-500/(--o)`. Interpretation is utility-specific (fraction
    /// denominator, opacity percent, line-height scale...).
    pub modifier: Option<&'a str>,
    /// Leading `-` negation (`-mt-4`).
    pub negative: bool,
    /// v4 trailing `!` or legacy leading `!`.
    pub important: bool,
    /// Raw source span (for diagnostics).
    pub raw: &'a str,
}

impl<'a> ClassToken<'a> {
    /// The opacity percent for color utilities, per v4 semantics:
    /// * `bg-primary/50` → `50.0` (bare number = percent);
    /// * `bg-primary/[0.37]` → `37.0` (bracketed decimal = raw alpha);
    /// * `bg-primary/(--o)` → resolve `--o` in `theme` (F7).
    ///
    /// Only color-ish utilities call this; sizing utilities read
    /// [`ClassToken::modifier`] as a fraction denominator instead.
    pub fn opacity_pct(&self, theme: &crate::theme::TailwindTheme) -> Option<Option<f32>> {
        let m = self.modifier?;
        let inner = m
            .strip_prefix('[')
            .and_then(|r| r.strip_suffix(']'))
            .unwrap_or(m);
        // `(--var)` / `[var(--x)]` / bare `--x`: resolve via the theme.
        let var_name = if let Some(v) = inner.strip_prefix('(').and_then(|r| r.strip_suffix(')')) {
            Some(v)
        } else if let Some(v) = inner.strip_prefix("var(").and_then(|r| r.strip_suffix(')')) {
            Some(v)
        } else if inner.starts_with("--") {
            Some(inner)
        } else {
            None
        };
        if let Some(name) = var_name {
            let bare = name.trim_start_matches("--");
            return match theme.length(bare) {
                Some(v) => Some(Some(v)),
                None => theme.var_percent(bare).map(Some),
            };
        }
        // Bracketed decimal `0.37` is a raw alpha fraction (v4).
        if inner.contains('.') && !inner.contains('%') {
            return inner.parse::<f32>().ok().map(|a| Some(a * 100.0));
        }
        let pct: f32 = inner.trim_end_matches('%').parse().ok()?;
        Some(Some(pct))
    }

    /// The exact variant-stack merge key for `cn()`: canonical text of every
    /// variant, joined by `:`. Distinguishes `group-hover:` from `hover:`
    /// and `@md:` from `md:` (plan §1.3 cn() bug).
    pub fn variant_prefix(&self) -> String {
        let mut s = String::new();
        let mut first = true;
        for v in self.variants.iter() {
            if !first {
                s.push(':');
            }
            first = false;
            write_variant(&mut s, v);
        }
        s
    }
}

fn write_variant(s: &mut String, v: &Variant<'_>) {
    match v {
        Variant::Named(n) => s.push_str(&n.canonical()),
        Variant::Grouped {
            parent,
            inner,
            label,
        } => {
            s.push_str(match parent {
                ParentKind::Group => "group-",
                ParentKind::Peer => "peer-",
            });
            write_variant(s, inner);
            if let Some(l) = label {
                s.push('/');
                s.push_str(l);
            }
        }
        Variant::Not(inner) => {
            s.push_str("not-");
            write_variant(s, inner);
        }
        Variant::Has(inner) => {
            s.push_str("has-");
            s.push_str(inner);
        }
        Variant::Attr { kind, name, value } => {
            s.push_str(match kind {
                AttrKind::Data => "data-",
                AttrKind::Aria => "aria-",
            });
            s.push_str(name);
            if let Some(v) = value {
                s.push_str(&format!("={v}"));
            }
        }
        Variant::Structural(st) => s.push_str(st.canonical()),
        Variant::PseudoElement(p) => s.push_str(p),
        Variant::Arbitrary(a) => s.push_str(a),
    }
}

/// A stack-inlined, cloneable small vector (zero dependencies).
///
/// Backed by a fully-initialized array of `T` plus a length; entries beyond
/// `len` hold the sentinel value passed at construction (for the parser:
/// `Variant::Arbitrary("")` / `""`, which never affect a bounded slice).
/// Derefs to `&[T]`, so iteration, indexing and slice patterns all work.
pub struct Inline<T, const N: usize> {
    items: [T; N],
    len: u8,
}

impl<T: Clone, const N: usize> Inline<T, N> {
    /// Build from an explicit sentinel plus a fallible producer: `f(i)`
    /// yields the i-th item or `None` to stop. Entries beyond the produced
    /// length hold clones of `sentinel` (never observable — the slice view
    /// is bounded by `len`). Returns `None` when `f` yields more than `N`
    /// items (the caller reports a per-class warning; never a panic).
    pub fn build(sentinel: T, mut f: impl FnMut(usize) -> Option<T>) -> Option<Self> {
        let mut items: [T; N] = std::array::from_fn(|_| sentinel.clone());
        let mut len = 0usize;
        for (i, slot) in items.iter_mut().enumerate() {
            match f(i) {
                Some(v) => {
                    *slot = v;
                    len += 1;
                }
                None => break,
            }
        }
        if f(len).is_some() {
            return None; // overflow beyond the inline capacity
        }
        Some(Self {
            items,
            len: len as u8,
        })
    }

    /// Collect from an exact-size iterator of items (parser fast path).
    pub fn collect(sentinel: T, items: impl IntoIterator<Item = T>) -> Option<Self> {
        let mut it = items.into_iter();
        Self::build(sentinel, |_| it.next())
    }

    /// The number of produced items.
    pub fn len(&self) -> usize {
        self.len as usize
    }

    /// True when no items were produced.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// The bounded slice view over the produced items.
    pub fn as_slice(&self) -> &[T] {
        &self.items[..self.len as usize]
    }
}

impl<T: Clone, const N: usize> std::ops::Deref for Inline<T, N> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        &self.items[..self.len as usize]
    }
}

impl<T: Clone, const N: usize> Clone for Inline<T, N> {
    fn clone(&self) -> Self {
        Self {
            items: std::array::from_fn(|i| self.items[i].clone()),
            len: self.len,
        }
    }
}

impl<T: Clone + PartialEq, const N: usize> PartialEq for Inline<T, N> {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<T: Clone + std::fmt::Debug, const N: usize> std::fmt::Debug for Inline<T, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.as_slice().iter()).finish()
    }
}

/// Stack-inlined `&'a str` segments (`["rounded", "t", "lg"]`).
pub struct Segments<'a, const N: usize> {
    items: [&'a str; N],
    len: u8,
}

impl<'a, const N: usize> Segments<'a, N> {
    /// Build from a fallible producer; `None` on overflow beyond `N`.
    pub fn from_iter_fn(mut f: impl FnMut(usize) -> Option<&'a str>) -> Option<Self> {
        let mut items: [&'a str; N] = [""; N];
        let mut len = 0usize;
        for (i, slot) in items.iter_mut().enumerate() {
            match f(i) {
                Some(v) => {
                    *slot = v;
                    len += 1;
                }
                None => break,
            }
        }
        if f(len).is_some() {
            return None;
        }
        Some(Self {
            items,
            len: len as u8,
        })
    }

    /// The number of produced segments.
    pub fn len(&self) -> usize {
        self.len as usize
    }

    /// True when no segments were produced.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// The bounded slice view over the produced segments.
    pub fn as_slice(&self) -> &[&'a str] {
        &self.items[..self.len as usize]
    }
}

impl<'a, const N: usize> std::ops::Deref for Segments<'a, N> {
    type Target = [&'a str];
    fn deref(&self) -> &[&'a str] {
        &self.items[..self.len as usize]
    }
}

impl<'a, const N: usize> Clone for Segments<'a, N> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, const N: usize> Copy for Segments<'a, N> {}

impl<'a, const N: usize> PartialEq for Segments<'a, N> {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<'a, const N: usize> Eq for Segments<'a, N> {}

impl<'a, const N: usize> std::fmt::Debug for Segments<'a, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.as_slice().iter()).finish()
    }
}

/// An arbitrary value group: `[value]`, `[hint:value]` or the v4.1
/// `(--var)` shorthand for `[var(--var)]`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Arbitrary<'a> {
    /// Type hint (`length`, `color`, `url`, ...), when declared.
    pub hint: Option<&'a str>,
    /// The value text without brackets (hint stripped).
    pub value: &'a str,
    /// Bracket `[...]` or shorthand `(...)`.
    pub form: ArbForm,
    /// True for the arbitrary-*property* form `[mask-type:luminance]` —
    /// a bare bracket containing `:` that sets a CSS property directly.
    pub is_property: bool,
}

/// Which arbitrary syntax a value used.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArbForm {
    /// `[...]`.
    Bracket,
    /// `(...)` — the v4.1 var() shorthand.
    Paren,
}

impl Arbitrary<'_> {
    /// The CSS text this arbitrary resolves to: the value itself for
    /// `[...]`, `var(--x)` for `(--x)`.
    pub fn css(&self) -> String {
        match self.form {
            ArbForm::Bracket => self.value.to_string(),
            ArbForm::Paren => format!("var({})", self.value),
        }
    }
}

/// A single variant prefix. v0.2 models the full v4 grammar explicitly so
/// backends can reject what they cannot express with a precise warning.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Variant<'a> {
    /// A named variant: states, media, breakpoints, `@container`, or an
    /// unresolved [`NamedVariant::Custom`] identifier.
    Named(NamedVariant<'a>),
    /// `group-hover`, `group-hover/menu`, `group-data-[x]` — parent-relative.
    Grouped {
        /// The parent family (`group` or `peer`).
        parent: ParentKind,
        /// The guarded condition on the parent/sibling (recursive).
        inner: Box<Variant<'a>>,
        /// Optional named container (`group-hover/menu`).
        label: Option<&'a str>,
    },
    /// `not-hover`, `not-data-[x]` — negation (v4, recursive).
    Not(Box<Variant<'a>>),
    /// `has-checked`, `has-[:focus]`, `has-[>svg]` — raw has-target.
    Has(&'a str),
    /// `data-[state=open]`, `data-disabled`, `aria-[busy=true]`, `aria-busy`.
    Attr {
        /// `data-*` or `aria-*`.
        kind: AttrKind,
        /// Attribute name (`state`, `disabled`, `busy`).
        name: &'a str,
        /// Value, when present (`Some("open")` for `data-[state=open]`).
        value: Option<&'a str>,
    },
    /// `first`, `last`, `odd`, `even`, `only`, `empty`, `nth-3`, `nth-last-2`.
    Structural(Structural),
    /// `::before`, `::first-line`, ... — no native equivalent; reported.
    PseudoElement(&'a str),
    /// `[&_svg]`, `*`, `**`, `supports-…`, `[@…]` — reported with a precise
    /// reason, never silently dropped.
    Arbitrary(&'a str),
}

/// The parent-relative variant families (`group-*` / `peer-*`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParentKind {
    /// A `.group` ancestor.
    Group,
    /// A `.peer` sibling.
    Peer,
}

/// `data-*` vs `aria-*` attribute variants.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttrKind {
    /// `data-*`.
    Data,
    /// `aria-*`.
    Aria,
}

/// Structural (DOM-position) variants.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Structural {
    /// `first` (`:first-child`).
    First,
    /// `last` (`:last-child`).
    Last,
    /// `odd` (`:nth-child(odd)`).
    Odd,
    /// `even` (`:nth-child(even)`).
    Even,
    /// `only` (`:only-child`).
    Only,
    /// `empty` (`:empty`).
    Empty,
    /// `nth-3` (`:nth-child(3)`).
    Nth(i32),
    /// `nth-last-2` (`:nth-last-child(2)`).
    NthLast(i32),
}

/// The element's own pseudo-element renderings that v4 names as plain
/// variants (v0.7 §1, plan `AEOLUS_V0.7.md`): `selection:` → `&::selection`,
/// `placeholder:` → `&::placeholder`, `file:` → `&::file-selector-button`.
///
/// These are NOT runtime states (there is no "selected" flag on a widget the
/// way `hover` is): they style a SUB-RENDERING of the element. The typed
/// model keeps them distinguishable from [`State`]s — a backend answers them
/// through `Probe::test(&Test::Pseudo(…))`, and the default probe answer is
/// "no" (the rule never fires where the rendering does not exist — the same
/// semantics CSS has when a selector does not match).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PseudoElement {
    /// `selection:` — `::selection`, the selected-text rendering.
    Selection,
    /// `placeholder:` — `::placeholder`, the placeholder rendering.
    Placeholder,
    /// `file:` — `::file-selector-button` (the `<input type=file>` button
    /// part). No native GUI toolkit has a file-selector sub-widget; the
    /// typed condition exists so the class parses, merges and emits, and
    /// the rule simply never fires natively.
    FileSelectorButton,
}

impl PseudoElement {
    /// The canonical variant name (merge keys and diagnostics).
    pub fn name(self) -> &'static str {
        match self {
            PseudoElement::Selection => "selection",
            PseudoElement::Placeholder => "placeholder",
            PseudoElement::FileSelectorButton => "file",
        }
    }

    /// The CSS selector suffix (`::selection`, ...).
    pub fn css(self) -> &'static str {
        match self {
            PseudoElement::Selection => "::selection",
            PseudoElement::Placeholder => "::placeholder",
            PseudoElement::FileSelectorButton => "::file-selector-button",
        }
    }
}

impl Structural {
    /// The canonical variant text (for merge keys and diagnostics).
    pub fn canonical(self) -> &'static str {
        match self {
            Structural::First => "first",
            Structural::Last => "last",
            Structural::Odd => "odd",
            Structural::Even => "even",
            Structural::Only => "only",
            Structural::Empty => "empty",
            Structural::Nth(_) => "nth",
            Structural::NthLast(_) => "nth-last",
        }
    }
}

/// The variant table — the canonical v4 order matters for cascade stability
/// but each layer is keyed by its condition, so we only need identity here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NamedVariant<'a> {
    // ---- states (Probe::test) ------------------------------------------
    /// `hover`.
    Hover,
    /// `focus`.
    Focus,
    /// `focus-visible`.
    FocusVisible,
    /// `focus-within`.
    FocusWithin,
    /// `active`.
    Active,
    /// `disabled`.
    Disabled,
    /// `visited`.
    Visited,
    /// `target`.
    Target,
    /// `enabled`.
    Enabled,
    /// `checked`.
    Checked,
    /// `indeterminate`.
    Indeterminate,
    /// `required`.
    Required,
    /// `optional`.
    Optional,
    /// `valid`.
    Valid,
    /// `invalid`.
    Invalid,
    /// `user-valid`.
    UserValid,
    /// `user-invalid`.
    UserInvalid,
    /// `placeholder-shown`.
    PlaceholderShown,
    /// `selection:` — the element's own `::selection` rendering (v0.7 §1).
    Pseudo(PseudoElement),
    /// `autofill`.
    Autofill,
    /// `read-only`.
    ReadOnly,
    /// `in-range`.
    InRange,
    /// `out-of-range`.
    OutOfRange,

    // ---- media -----------------------------------------------------------
    /// `dark` (class/OS dark mode).
    Dark,
    /// `print`.
    Print,
    /// `motion-safe`.
    MotionSafe,
    /// `motion-reduce`.
    MotionReduce,
    /// `contrast-more`.
    ContrastMore,
    /// `contrast-less`.
    ContrastLess,
    /// `forced-colors`.
    ForcedColors,
    /// `portrait`.
    Portrait,
    /// `landscape`.
    Landscape,
    /// `rtl` direction.
    Rtl,
    /// `ltr` direction.
    Ltr,
    /// `pointer-fine`.
    PointerFine,
    /// `pointer-coarse`.
    PointerCoarse,
    /// `pointer-none`.
    PointerNone,
    /// `any-pointer-fine`.
    AnyPointerFine,
    /// `any-pointer-coarse`.
    AnyPointerCoarse,
    /// `any-pointer-none`.
    AnyPointerNone,

    // ---- breakpoints & containers ---------------------------------------
    /// Responsive breakpoints (`sm`..`2xl`) resolved against `--breakpoint-*`.
    Breakpoint(Breakpoint),
    /// `min-md` (`@media (width >= md)`).
    Min(Breakpoint),
    /// `max-md` (`@media (width < md)`).
    Max(Breakpoint),
    /// `@container`-family variants (`@sm`, `@md`, ...) — container queries
    /// sized from `--container-*` (plan §3.3).
    Container(Breakpoint),
    /// `@max-sm` — upper-bounded container query.
    MaxContainer(Breakpoint),
    /// An identifier the static tables don't know — resolved later against
    /// theme breakpoints (`--breakpoint-3xl`), container sizes (`@4xl`),
    /// `@min-[400px]` forms and `@custom-variant` registrations.
    Custom(&'a str),
}

impl NamedVariant<'_> {
    /// Canonical text (for merge keys and diagnostics).
    pub fn canonical(&self) -> String {
        match self {
            NamedVariant::Hover => "hover".into(),
            NamedVariant::Focus => "focus".into(),
            NamedVariant::FocusVisible => "focus-visible".into(),
            NamedVariant::FocusWithin => "focus-within".into(),
            NamedVariant::Active => "active".into(),
            NamedVariant::Disabled => "disabled".into(),
            NamedVariant::Visited => "visited".into(),
            NamedVariant::Target => "target".into(),
            NamedVariant::Enabled => "enabled".into(),
            NamedVariant::Checked => "checked".into(),
            NamedVariant::Indeterminate => "indeterminate".into(),
            NamedVariant::Required => "required".into(),
            NamedVariant::Optional => "optional".into(),
            NamedVariant::Valid => "valid".into(),
            NamedVariant::Invalid => "invalid".into(),
            NamedVariant::UserValid => "user-valid".into(),
            NamedVariant::UserInvalid => "user-invalid".into(),
            NamedVariant::PlaceholderShown => "placeholder-shown".into(),
            NamedVariant::Pseudo(p) => p.name().into(),
            NamedVariant::Autofill => "autofill".into(),
            NamedVariant::ReadOnly => "read-only".into(),
            NamedVariant::InRange => "in-range".into(),
            NamedVariant::OutOfRange => "out-of-range".into(),
            NamedVariant::Dark => "dark".into(),
            NamedVariant::Print => "print".into(),
            NamedVariant::MotionSafe => "motion-safe".into(),
            NamedVariant::MotionReduce => "motion-reduce".into(),
            NamedVariant::ContrastMore => "contrast-more".into(),
            NamedVariant::ContrastLess => "contrast-less".into(),
            NamedVariant::ForcedColors => "forced-colors".into(),
            NamedVariant::Portrait => "portrait".into(),
            NamedVariant::Landscape => "landscape".into(),
            NamedVariant::Rtl => "rtl".into(),
            NamedVariant::Ltr => "ltr".into(),
            NamedVariant::PointerFine => "pointer-fine".into(),
            NamedVariant::PointerCoarse => "pointer-coarse".into(),
            NamedVariant::PointerNone => "pointer-none".into(),
            NamedVariant::AnyPointerFine => "any-pointer-fine".into(),
            NamedVariant::AnyPointerCoarse => "any-pointer-coarse".into(),
            NamedVariant::AnyPointerNone => "any-pointer-none".into(),
            NamedVariant::Breakpoint(b) => b.name().into(),
            NamedVariant::Min(b) => format!("min-{}", b.name()),
            NamedVariant::Max(b) => format!("max-{}", b.name()),
            NamedVariant::Container(b) => format!("@{}", b.name()),
            NamedVariant::MaxContainer(b) => format!("@max-{}", b.name()),
            NamedVariant::Custom(s) => (*s).into(),
        }
    }

    /// The probe-testable state, when this variant is a state.
    pub fn state(self) -> Option<State> {
        Some(match self {
            NamedVariant::Hover => State::Hover,
            NamedVariant::Focus => State::Focus,
            NamedVariant::FocusVisible => State::FocusVisible,
            NamedVariant::FocusWithin => State::FocusWithin,
            NamedVariant::Active => State::Active,
            NamedVariant::Disabled => State::Disabled,
            NamedVariant::Visited => State::Visited,
            NamedVariant::Target => State::Target,
            NamedVariant::Enabled => State::Enabled,
            NamedVariant::Checked => State::Checked,
            NamedVariant::Indeterminate => State::Indeterminate,
            NamedVariant::Required => State::Required,
            NamedVariant::Optional => State::Optional,
            NamedVariant::Valid => State::Valid,
            NamedVariant::Invalid => State::Invalid,
            NamedVariant::UserValid => State::UserValid,
            NamedVariant::UserInvalid => State::UserInvalid,
            NamedVariant::PlaceholderShown => State::PlaceholderShown,
            NamedVariant::Autofill => State::Autofill,
            NamedVariant::ReadOnly => State::ReadOnly,
            NamedVariant::InRange => State::InRange,
            NamedVariant::OutOfRange => State::OutOfRange,
            _ => return None,
        })
    }

    /// The media query, when this variant is a media variant.
    pub fn media(self) -> Option<Media> {
        Some(match self {
            NamedVariant::Dark => Media::Dark,
            NamedVariant::Print => Media::Print,
            NamedVariant::MotionSafe => Media::MotionSafe,
            NamedVariant::MotionReduce => Media::MotionReduce,
            NamedVariant::ContrastMore => Media::ContrastMore,
            NamedVariant::ContrastLess => Media::ContrastLess,
            NamedVariant::ForcedColors => Media::ForcedColors,
            NamedVariant::Portrait => Media::Portrait,
            NamedVariant::Landscape => Media::Landscape,
            NamedVariant::Rtl => Media::Rtl,
            NamedVariant::Ltr => Media::Ltr,
            NamedVariant::PointerFine => Media::PointerFine,
            NamedVariant::PointerCoarse => Media::PointerCoarse,
            NamedVariant::PointerNone => Media::PointerNone,
            NamedVariant::AnyPointerFine => Media::AnyPointerFine,
            NamedVariant::AnyPointerCoarse => Media::AnyPointerCoarse,
            NamedVariant::AnyPointerNone => Media::AnyPointerNone,
            _ => return None,
        })
    }
}

/// A probe-testable pseudo-state of an element.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum State {
    /// `:hover`.
    Hover,
    /// `:focus`.
    Focus,
    /// `:focus-visible`.
    FocusVisible,
    /// `:focus-within`.
    FocusWithin,
    /// `:active`.
    Active,
    /// `:disabled`.
    Disabled,
    /// `:visited`.
    Visited,
    /// `:target`.
    Target,
    /// `:enabled`.
    Enabled,
    /// `:checked`.
    Checked,
    /// `:indeterminate`.
    Indeterminate,
    /// `:required`.
    Required,
    /// `:optional`.
    Optional,
    /// `:valid`.
    Valid,
    /// `:invalid`.
    Invalid,
    /// `:user-valid`.
    UserValid,
    /// `:user-invalid`.
    UserInvalid,
    /// `:placeholder-shown`.
    PlaceholderShown,
    /// `:autofill`.
    Autofill,
    /// `:read-only`.
    ReadOnly,
    /// `:in-range`.
    InRange,
    /// `:out-of-range`.
    OutOfRange,
}

impl State {
    /// The 7 classic conditions backends lower natively (v0.1 buckets).
    pub fn classic(self) -> Option<crate::plan::VariantCondition> {
        use crate::plan::VariantCondition as V;
        Some(match self {
            State::Hover => V::Hover,
            State::Active => V::Active,
            State::Disabled => V::Disabled,
            State::Focus => V::Focus,
            State::FocusVisible => V::FocusVisible,
            State::FocusWithin => V::FocusWithin,
            State::Visited => V::Visited,
            _ => return None,
        })
    }
}

/// Media-query atoms (`dark`, `print`, motion, pointer, direction, ...).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Media {
    /// `dark` (`prefers-color-scheme` or class strategy).
    Dark,
    /// `print`.
    Print,
    /// `motion-safe`.
    MotionSafe,
    /// `motion-reduce`.
    MotionReduce,
    /// `contrast-more`.
    ContrastMore,
    /// `contrast-less`.
    ContrastLess,
    /// `forced-colors`.
    ForcedColors,
    /// `portrait`.
    Portrait,
    /// `landscape`.
    Landscape,
    /// `rtl`.
    Rtl,
    /// `ltr`.
    Ltr,
    /// `pointer-fine`.
    PointerFine,
    /// `pointer-coarse`.
    PointerCoarse,
    /// `pointer-none`.
    PointerNone,
    /// `any-pointer-fine`.
    AnyPointerFine,
    /// `any-pointer-coarse`.
    AnyPointerCoarse,
    /// `any-pointer-none`.
    AnyPointerNone,
}

/// Tailwind v4 default breakpoints (`--breakpoint-*`, 4px spacing base).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Breakpoint {
    /// `sm` (640px default).
    Sm,
    /// `md` (768px default).
    Md,
    /// `lg` (1024px default).
    Lg,
    /// `xl` (1280px default).
    Xl,
    /// `2xl` (1536px default).
    Xxl,
}

impl Breakpoint {
    /// Default `--breakpoint-*` values in px (v4 defaults: 40/48/64/80/96 rem).
    pub fn default_px(self) -> f32 {
        match self {
            Breakpoint::Sm => 640.0,
            Breakpoint::Md => 768.0,
            Breakpoint::Lg => 1024.0,
            Breakpoint::Xl => 1280.0,
            Breakpoint::Xxl => 1536.0,
        }
    }

    /// Map a variant name (`"sm"`, `"2xl"`) back to the breakpoint.
    pub fn from_name(s: &str) -> Option<Self> {
        Some(match s {
            "sm" => Breakpoint::Sm,
            "md" => Breakpoint::Md,
            "lg" => Breakpoint::Lg,
            "xl" => Breakpoint::Xl,
            "2xl" => Breakpoint::Xxl,
            _ => return None,
        })
    }

    /// The canonical variant name (`"sm"`, `"xl"`, `"2xl"`).
    pub fn name(self) -> &'static str {
        match self {
            Breakpoint::Sm => "sm",
            Breakpoint::Md => "md",
            Breakpoint::Lg => "lg",
            Breakpoint::Xl => "xl",
            Breakpoint::Xxl => "2xl",
        }
    }

    /// Index into `StylePlan::breakpoints` / `TailwindTheme::breakpoints_px`.
    pub fn index(self) -> usize {
        match self {
            Breakpoint::Sm => 0,
            Breakpoint::Md => 1,
            Breakpoint::Lg => 2,
            Breakpoint::Xl => 3,
            Breakpoint::Xxl => 4,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The v0.7 pseudo-element surface: canonical variant names (merge keys,
    /// diagnostics, snapshot reader) and the CSS suffixes the emitter
    /// reproduces verbatim.
    #[test]
    fn pseudo_element_names_and_css_suffixes() {
        assert_eq!(PseudoElement::Selection.name(), "selection");
        assert_eq!(PseudoElement::Placeholder.name(), "placeholder");
        assert_eq!(PseudoElement::FileSelectorButton.name(), "file");
        assert_eq!(PseudoElement::Selection.css(), "::selection");
        assert_eq!(PseudoElement::Placeholder.css(), "::placeholder");
        assert_eq!(PseudoElement::FileSelectorButton.css(), "::file-selector-button");
    }

    /// Structural canonical names feed merge keys; the Nth forms canonicalize
    /// to the family name (the number is part of `Segments`, not the key).
    #[test]
    fn structural_canonical_names() {
        assert_eq!(Structural::First.canonical(), "first");
        assert_eq!(Structural::Last.canonical(), "last");
        assert_eq!(Structural::Odd.canonical(), "odd");
        assert_eq!(Structural::Even.canonical(), "even");
        assert_eq!(Structural::Only.canonical(), "only");
        assert_eq!(Structural::Empty.canonical(), "empty");
        assert_eq!(Structural::Nth(3).canonical(), "nth");
        assert_eq!(Structural::NthLast(2).canonical(), "nth-last");
    }

    /// Breakpoint names and indices stay in lockstep — `index()` indexes
    /// `StylePlan::breakpoints` (a 5-slot table), so any drift breaks the
    /// responsive lowering silently.
    #[test]
    fn breakpoint_indices_are_dense_and_ordered() {
        let all = [
            Breakpoint::Sm,
            Breakpoint::Md,
            Breakpoint::Lg,
            Breakpoint::Xl,
            Breakpoint::Xxl,
        ];
        for (i, bp) in all.iter().enumerate() {
            assert_eq!(bp.index(), i, "{bp:?} index drifted");
        }
        assert_eq!(Breakpoint::Sm.name(), "sm");
        assert_eq!(Breakpoint::Xxl.name(), "2xl");
    }
}
