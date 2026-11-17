//! Layer conditions (architecture plan §3.3 `Cond`, §3.5 variant costs).
//!
//! A [`Cond`] is the ONE condition a [`crate::Layer`] fires under. The
//! model is deliberately flat: stacked variants (`hover:focus:…`) are
//! folded by the P2 compile stage into the single dominant condition plus
//! the layer ordering — the plan's "layer 0 = base, rest sorted by
//! specificity" contract only needs an ordering, not a tree. Deeply
//! nested group/peer chains (`group-hover/menu:`) ride the compile-stage
//! fold; what the IR carries is where the cost is paid (plan §3.5): a
//! probe query, a media query, or a container query.

/// The responsive breakpoints, in ascending order (`--breakpoint-*`
/// defaults; the theme tier can re-map the rem values, not the order).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Bp {
    /// `sm` (40rem).
    Sm,
    /// `md` (48rem).
    Md,
    /// `lg` (64rem).
    Lg,
    /// `xl` (80rem).
    Xl,
    /// `2xl` (96rem).
    Xl2,
}

impl Bp {
    /// Every breakpoint, ascending — the sort order for `Bp` layers.
    pub const ALL: &'static [Bp] = &[Bp::Sm, Bp::Md, Bp::Lg, Bp::Xl, Bp::Xl2];

    /// The canonical v4 name (`sm`, `2xl`).
    pub const fn name(self) -> &'static str {
        match self {
            Bp::Sm => "sm",
            Bp::Md => "md",
            Bp::Lg => "lg",
            Bp::Xl => "xl",
            Bp::Xl2 => "2xl",
        }
    }

    /// The `--breakpoint-*` default min-width, in rem — the media-query
    /// half of the emit story (the CSS target renders `@media (min-width:
    /// Nrem)` from it). The theme tier may re-map the rem VALUES (plan
    /// §3.4), never the order; these are the v4 defaults the docs pin.
    pub const fn rem(self) -> f32 {
        match self {
            Bp::Sm => 40.0,
            Bp::Md => 48.0,
            Bp::Lg => 64.0,
            Bp::Xl => 80.0,
            Bp::Xl2 => 96.0,
        }
    }
}

/// The element's own DOM-position tests — the structural variants
/// (`first:`, `last:`, `odd:`, `even:`, `only:`, `empty:`, `nth-N:`,
/// `nth-last-N:`). `Copy`, `const`-constructible, no heap: the payload
/// is the ordinal itself, so the static form stays a `static` item end
/// to end.
///
/// The parity and ordinal semantics are CSS's own and 1-BASED: the
/// first child (index 0) is `:nth-child(odd)` / `:nth-child(1)`, the
/// second even. `Nth(-1)` is a legal fold (the v0.x oracle's
/// `parse_nth` accepts every `i32`) that renders a selector CSS never
/// matches — the honest spelling pass, never a refusal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Structural {
    /// `first:` (`:first-child`) — the parent's first element child.
    First,
    /// `last:` (`:last-child`) — the parent's final element child.
    Last,
    /// `odd:` (`:nth-child(odd)`) — the odd-numbered children.
    Odd,
    /// `even:` (`:nth-child(even)`) — the even-numbered children.
    Even,
    /// `only:` (`:only-child`) — the parent's single element child.
    Only,
    /// `empty:` (`:empty`) — the element carries no children.
    Empty,
    /// `nth-N:` (`:nth-child(N)`) — the Nth child, N 1-based.
    Nth(i32),
    /// `nth-last-N:` (`:nth-last-child(N)`) — the Nth child counting
    /// from the end, N 1-based.
    NthLast(i32),
}

/// The user-agent preference/media-type facts the media-feature
/// variants ask about — the closed spelling set the variant fold
/// accepts (`motion-safe:`, `motion-reduce:`, `contrast-more:`,
/// `contrast-less:`, `forced-colors:`, `print:`, the input-capability
/// tier `pointer-fine:`..`any-pointer-none:`, and the orientation forms
/// `portrait:`/`landscape:` — every media-feature static variant
/// tailwindcss v4.1.17 carries). `Copy`,
/// `const`-constructible, no heap: the static form stays a `static`
/// item end to end.
///
/// The bits are FRAME STATE (the user agent's own media-query answers,
/// the tier `dark:` rides): the `Env` carries one fact per
/// question and the interp answers from it — `motion-safe:` is the
/// COMPLEMENT of `motion-reduce:` and `landscape:` of `portrait:`
/// (each feature's two v4 values share one fact, the orientation query
/// always holding an answer the way the reduced-motion one does), the
/// rest fire on their own bit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Pref {
    /// `motion-safe:` — the user has NO reduced-motion preference
    /// (`@media (prefers-reduced-motion: no-preference)`).
    MotionSafe,
    /// `motion-reduce:` — the user prefers reduced motion
    /// (`@media (prefers-reduced-motion: reduce)`).
    MotionReduce,
    /// `contrast-more:` — the user requests more contrast
    /// (`@media (prefers-contrast: more)`).
    ContrastMore,
    /// `contrast-less:` — the user requests less contrast
    /// (`@media (prefers-contrast: less)`).
    ContrastLess,
    /// `forced-colors:` — the user agent forces a color palette
    /// (`@media (forced-colors: active)`).
    ForcedColors,
    /// `print:` — the print media type (`@media print`).
    Print,
    /// `pointer-fine:` — the PRIMARY pointer is precise
    /// (`@media (pointer: fine)`).
    PointerFine,
    /// `pointer-coarse:` — the primary pointer is coarse
    /// (`@media (pointer: coarse)`).
    PointerCoarse,
    /// `pointer-none:` — there is no primary pointer
    /// (`@media (pointer: none)`).
    PointerNone,
    /// `any-pointer-fine:` — ANY pointer is precise
    /// (`@media (any-pointer: fine)`).
    AnyPointerFine,
    /// `any-pointer-coarse:` — ANY pointer is coarse
    /// (`@media (any-pointer: coarse)`).
    AnyPointerCoarse,
    /// `any-pointer-none:` — NO pointer exists
    /// (`@media (any-pointer: none)`).
    AnyPointerNone,
    /// `portrait:` — the viewport is taller than wide
    /// (`@media (orientation: portrait)`).
    Portrait,
    /// `landscape:` — the viewport is at least as wide as tall — the
    /// COMPLEMENT of [`Pref::Portrait`] (the feature's two v4 values
    /// share one fact, the `motion-safe:` precedent).
    Landscape,
    /// `inverted-colors:` — the user agent inverts colours (the
    /// media-features block's
    /// `staticVariant('inverted-colors', ['@media (inverted-colors:
    /// inverted)'])`, the v4.1 addition; the accessibility answer the
    /// `forced-colors:` bit rides beside).
    InvertedColors,
    /// `noscript:` — scripting is unavailable (the same block's
    /// `staticVariant('noscript', ['@media (scripting: none)'])`; the
    /// scripting feature's own static spelling — `enable`/`initial-only`
    /// have no v4.1.17 registration, the spelling set stays closed).
    NoScript,
}

impl Pref {
    /// The canonical v4 spelling (the variant segment the fold
    /// accepted, the diagnostics' and the metamorphic labels' text).
    pub const fn source(self) -> &'static str {
        match self {
            Pref::MotionSafe => "motion-safe",
            Pref::MotionReduce => "motion-reduce",
            Pref::ContrastMore => "contrast-more",
            Pref::ContrastLess => "contrast-less",
            Pref::ForcedColors => "forced-colors",
            Pref::Print => "print",
            Pref::PointerFine => "pointer-fine",
            Pref::PointerCoarse => "pointer-coarse",
            Pref::PointerNone => "pointer-none",
            Pref::AnyPointerFine => "any-pointer-fine",
            Pref::AnyPointerCoarse => "any-pointer-coarse",
            Pref::AnyPointerNone => "any-pointer-none",
            Pref::Portrait => "portrait",
            Pref::Landscape => "landscape",
            Pref::InvertedColors => "inverted-colors",
            Pref::NoScript => "noscript",
        }
    }
}

/// The text-direction facts the direction variants ask about — the
/// closed spelling set the variant fold accepts (`ltr:`, `rtl:`; every
/// direction static variant tailwindcss v4.1.17 carries,
/// `packages/tailwindcss/src/variants.ts`'s `staticVariant('ltr', …)` /
/// `staticVariant('rtl', …)` calls). `Copy`, `const`-constructible, no
/// heap: the static form stays a `static` item end to end.
///
/// The fact is FRAME STATE (the element's resolved `direction`, the
/// tier the preference and capability bits ride): the `Env` carries ONE
/// fact and the two spellings answer from it — `rtl:` on the bit,
/// `ltr:` its COMPLEMENT (the feature's two v4 values share one fact,
/// the `portrait:`/`landscape:` precedent; CSS's own initial
/// `direction` value is `ltr`, so the query always holds an answer the
/// way the orientation one does).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Direction {
    /// `ltr:` — the resolved direction is left-to-right
    /// (`&:where(:dir(ltr), [dir="ltr"], [dir="ltr"] *)` — the
    /// complement of [`Direction::Rtl`]).
    Ltr,
    /// `rtl:` — the resolved direction is right-to-left
    /// (`&:where(:dir(rtl), [dir="rtl"], [dir="rtl"] *)`).
    Rtl,
}

impl Direction {
    /// The canonical v4 spelling (the variant segment the fold
    /// accepted, the diagnostics' and the metamorphic labels' text).
    pub const fn source(self) -> &'static str {
        match self {
            Direction::Ltr => "ltr",
            Direction::Rtl => "rtl",
        }
    }
}

/// The condition a layer fires under. `Copy`, `const`-constructible, no
/// heap: the static program is a `static` item end to end.
///
/// Specificity (the [`Cond::specificity`] rank) mirrors the v4 cascade:
/// base < mode < probe states < parent-relative < media < container.
///
/// The payload-carrying variants ([`Cond::Data`], [`Cond::Aria`],
/// [`Cond::Has`], [`Cond::Supports`]) carry their payload as
/// `&'static str`: a spelling the
/// compile stage folds interns the payload into the same leak pool the
/// program's slices use (the IR's documented dynamic-form storage —
/// "one interned `Box<[Decl]>` shared by reference"), and a literal
/// spelling is `&'static` outright, so the static form stays a `static`
/// item end to end. Equality and hashing compare the PAYLOAD TEXT, not
/// the pointer: two spellings that name the same probe (`data-open:` and
/// `data-[open]:`, `supports-[display:grid]` and
/// `supports-[(display:grid)]`) are one condition and one layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Cond {
    /// No condition — the base rule (layer 0).
    Base,
    /// `dark:` — the mode tier (class/OS dark mode resolved by the theme).
    Dark,
    /// `hover:`.
    Hover,
    /// `focus:`.
    Focus,
    /// `focus-visible:`.
    FocusVisible,
    /// `focus-within:`.
    FocusWithin,
    /// `active:`.
    Active,
    /// `disabled:`.
    Disabled,
    /// `visited:`.
    Visited,
    /// `checked:`.
    Checked,
    /// `placeholder-shown:`.
    PlaceholderShown,
    /// `valid:` — the element's constraints hold (the tailwindcss
    /// v4.1.17 forms block's static variant,
    /// `packages/tailwindcss/src/variants.ts`
    /// `staticVariant('valid', ['&:valid'])`). The COMPLEMENT of
    /// [`Cond::Invalid`] — the pair's two v4 values share one fact,
    /// the `ltr:`/`rtl:` precedent (CSS's own reading for a pristine
    /// field is `:valid`, so the resting env answers the `valid:`
    /// query).
    Valid,
    /// `invalid:` — the element's constraints fail (the same forms
    /// block's `staticVariant('invalid', ['&:invalid'])`).
    Invalid,
    /// `user-valid:` — the element's constraints hold AND the user has
    /// interacted with it (the forms block's
    /// `staticVariant('user-valid', ['&:user-valid'])`; the HTML
    /// spec's `:user-*` pseudo-classes match only after user
    /// interaction, so the spelling gates on BOTH facts).
    UserValid,
    /// `user-invalid:` — the element's constraints fail AND the user
    /// has interacted with it (the forms block's
    /// `staticVariant('user-invalid', ['&:user-invalid'])`; the same
    /// interaction gate).
    UserInvalid,
    /// `target:` — the element is the document's fragment target (the
    /// state block's `staticVariant('target', ['&:target'])`). The
    /// fragment-target state is ELEMENT STATE the host answers — the
    /// resting env's quiet miss a `:target` selector that does not
    /// match is.
    Target,
    /// `indeterminate:` — the element's own indeterminate state (the
    /// forms block's `staticVariant('indeterminate', ['&:indeterminate'])`;
    /// the checkbox/radio answer the host sets beside `checked:`, one
    /// fact per question — an element either is or is not
    /// indeterminate, a non-control is NOT, the resting env's quiet
    /// miss).
    Indeterminate,
    /// `autofill:` — the element was autofilled (the forms block's
    /// `staticVariant('autofill', ['&:autofill'])`; the browser's
    /// filled-form answer, one element-own fact — the resting env's
    /// quiet miss).
    Autofill,
    /// `read-only:` — the element is NOT user-editable (the forms
    /// block's `staticVariant('read-only', ['&:read-only'])`). The
    /// COMPLEMENT of the editability fact the Env carries — the pair's
    /// two CSS values share one fact, the `valid:`/`invalid:`
    /// precedent (CSS's own reading for a non-editable element IS
    /// `:read-only`, so the resting env answers the `read-only:`
    /// query; `read-write:` has no v4.1.17 registration, the spelling
    /// stays out — evidence 27's own boundary).
    ReadOnly,
    /// `open:` — the element is open (the state block's
    /// `staticVariant('open', ['&:is([open], :popover-open, :open)'])`;
    /// the disclosure/popover answer — a `dialog[open]`, a
    /// `details[open]`, an open popover — the host widget tracks). One
    /// AFFIRMATIVE element-own fact: a non-open element matches
    /// neither arm the way a non-target element misses `:target`, so
    /// the resting env's quiet miss answers.
    Open,
    /// `inert:` — the element is inside (or itself carries) an
    /// `[inert]` frozen subtree (the state block's
    /// `staticVariant('inert', ['&:is([inert], [inert] *)'])`; the
    /// compound head's two arms share ONE per-element fact the host
    /// computes — when a widget freezes, the whole subtree's elements
    /// know they are inert). The resting env's quiet miss a selector
    /// that does not match is.
    Inert,
    /// `default:` — the element is its form group's default control
    /// (the forms block's `staticVariant('default', ['&:default'])`;
    /// the `:default` answer the host widget tracks beside
    /// [`Cond::Checked`] — the checkbox with the `checked` attribute,
    /// the form's first submit button). One AFFIRMATIVE element-own
    /// fact: a non-form element matches neither the way a non-control
    /// misses `:indeterminate`, so the resting env's quiet miss
    /// answers.
    Default,
    /// `enabled:` — the element is a form-associated control that is
    /// not disabled (the forms block's
    /// `staticVariant('enabled', ['&:enabled'])`). The matches-neither
    /// pair evidence 27 documented: a non-form element matches NEITHER
    /// `:enabled` NOR the `:disabled` arms, so the condition answers
    /// from TWO facts — the form-membership fact the Env carries
    /// beside the disabled bit (the two-fact shape the form-state
    /// family closes the boundary with).
    Enabled,
    /// `required:` — the element is a form-associated control that
    /// carries the `required` attribute (the forms block's
    /// `staticVariant('required', ['&:required'])`). The
    /// matches-neither pair the membership fact answers: the attribute
    /// bit alone never fires — a non-form element matches NEITHER
    /// `:required` NOR `:optional`, so BOTH halves gate on the
    /// form-membership fact (the [`Cond::Enabled`] shape, evidence
    /// 27's own boundary).
    Required,
    /// `optional:` — the element is a form-associated control that
    /// carries NO `required` attribute (the forms block's
    /// `staticVariant('optional', ['&:optional'])`). The required
    /// attribute's COMPLEMENT within the form universe (the
    /// `validation_invalid` precedent — within the membership the
    /// pair shares one state fact), gated on the same membership fact:
    /// the complement is NOT a resting answer, a non-form element
    /// matches neither half.
    Optional,
    /// `in-range:` — the element is a bounded field whose value sits
    /// within the declared min/max bounds (the forms block's
    /// `staticVariant('in-range', ['&:in-range'])`). The matches-neither
    /// pair the range-membership fact answers: a non-range element (or
    /// a field with no position answer yet — the pristine empty
    /// reading) matches NEITHER `:in-range` NOR `:out-of-range`, so
    /// both halves gate on the bounded fact the Env carries beside the
    /// value's position bit (the [`Cond::Enabled`] shape).
    InRange,
    /// `out-of-range:` — the element is a bounded field whose value
    /// sits OUTSIDE the declared min/max bounds (the forms block's
    /// `staticVariant('out-of-range', ['&:out-of-range'])`). The value
    /// position's COMPLEMENT within the bounded universe (the
    /// `validation_invalid` precedent — within the membership the
    /// pair shares one position fact), gated on the same membership
    /// fact: the complement is NOT a resting answer, a non-range
    /// element matches neither half.
    OutOfRange,
    /// `selection:` — the element's own `::selection` rendering (v0.9 §1).
    Selection,
    /// `placeholder:` — the `::placeholder` rendering.
    Placeholder,
    /// `before:` — the `::before` rendering (the tailwindcss v4.1.17
    /// source's functional body, `packages/tailwindcss/src/variants.ts`
    /// pinned verbatim: `styleRule('&::before', [decl('content',
    /// 'var(--tw-content)'), …nodes])` — the variant WRAPS the
    /// utility's declarations in the pseudo-element rule with the
    /// content line riding first). Like [`Cond::Selection`], a
    /// pseudo-element styles a DIFFERENT render target than the
    /// element's own cascade, so the element walks exclude the layer
    /// and the pseudo cascades resolve per target.
    Before,
    /// `after:` — the `::after` rendering (the same functional body
    /// [`Cond::Before`] pins, `'&::after'` the selector).
    After,
    /// `marker:` — the list-marker rendering (the state block's
    /// `staticVariant('marker', ['& *::marker', '&::marker',
    /// '& *::-webkit-details-marker', '&::-webkit-details-marker'])` —
    /// the FOUR-selector disjunction the CSS arm renders on one rule).
    Marker,
    /// `first-letter:` — the `::first-letter` rendering (the
    /// pseudo-elements block's
    /// `staticVariant('first-letter', ['&::first-letter'])`).
    FirstLetter,
    /// `first-line:` — the `::first-line` rendering (the same block's
    /// `staticVariant('first-line', ['&::first-line'])`).
    FirstLine,
    /// `file:` — the `::file-selector-button` rendering (the state
    /// block's `staticVariant('file',
    /// ['&::file-selector-button'])`; the v0.x surface ALREADY carries
    /// the part — `PseudoElement::FileSelectorButton` — so the plan
    /// bridge lands the layer where the selection:/placeholder: pair
    /// lands, the other pseudo spellings reporting their boundary).
    FileSelector,
    /// `backdrop:` — the `::backdrop` rendering (the state block's
    /// `staticVariant('backdrop', ['&::backdrop'])` — the
    /// dialog/popover backdrop part).
    Backdrop,
    /// `details-content:` — the `::details-content` rendering (the
    /// state block's `staticVariant('details-content',
    /// ['&::details-content'])`, the v4.1 addition).
    DetailsContent,
    /// `group-hover:` — a `.group` ancestor is hovered (the named
    /// `group-hover/x` label form folds to the same probe with a name the
    /// compile stage threads through).
    GroupHover,
    /// `group-focus:`.
    GroupFocus,
    /// `peer-hover:` — a `.peer` sibling is hovered.
    PeerHover,
    /// `peer-focus:`.
    PeerFocus,
    /// `data-*:` — the element's own `data-` attribute probe (plan §3.5's
    /// "typed state bits, not string probes" row; the probe IS the typed
    /// bit the plan names, the payload its name). `data-open:` folds to
    /// `Data { name: "open", value: None }` — the presence test the v4
    /// corpus renders as `[data-open]`; `data-[state=open]:` folds to
    /// `Data { name: "state", value: Some("open") }` — the value test the
    /// corpus renders as `[data-state="open"]`. The payload is the
    /// ATTRIBUTE text (name, optional value verbatim), not the rendered
    /// selector: a renderer derives its own form, a host probe asks the
    /// pair.
    Data {
        /// The attribute name without the `data-` prefix (`"state"`).
        name: &'static str,
        /// The attribute value, verbatim (`Some("open")`), `None` for the
        /// presence form.
        value: Option<&'static str>,
    },
    /// `aria-*:` — the element's own `aria-` attribute probe. Bare
    /// `aria-busy:` names the "true" value (the v4 corpus renders
    /// `[aria-busy="true"]`), so the fold normalizes it to
    /// `Aria { name: "busy", value: Some("true") }`; the bracketed
    /// `aria-[sort=asc]:` form carries its value verbatim. Bare and
    /// bracketed true are ONE condition — they render identically.
    Aria {
        /// The attribute name without the `aria-` prefix (`"busy"`).
        name: &'static str,
        /// The attribute value, verbatim; bare spellings normalize to
        /// `Some("true")` (the corpus's own reading).
        value: Option<&'static str>,
    },
    /// `has-*:` — a descendant of the element matches the probe
    /// (`has-focus:`, `has-data-[slot=card-action]:`). The payload is the
    /// RENDERED `:has(...)` inner fragment — `*:focus`,
    /// `*[data-slot="card-action"]` — because the has- inner is selector
    /// text by grammar (the arbitrary form's inner passes into `:has()`
    /// as-is, star-prefixed per the v4 corpus's own output: non-tag
    /// inners read `*`, tags stay bare). There is no row table for a
    /// has- inner to be relative to — the row-text boundary that keeps
    /// `Val::Kw` row-relative does not apply here.
    Has(&'static str),
    /// `group-has-*:` — a `.group` ancestor HAS a descendant matching
    /// the probe (the parent-relative composition of [`Cond::Has`]; the
    /// named `group-has-x/label` form folds to the same probe, the
    /// label riding the CSS head only, the group-hover precedent). The
    /// payload is the same rendered `:has()` fragment [`Cond::Has`]
    /// carries.
    GroupHas(&'static str),
    /// `peer-has-*:` — a `.peer` sibling HAS a descendant matching the
    /// probe. Same payload contract as [`Cond::GroupHas`].
    PeerHas(&'static str),
    /// `first:` / `last:` / `odd:` / `even:` / `only:` / `empty:` /
    /// `nth-N:` / `nth-last-N:` — the element's own DOM-position probe
    /// ([`Structural`]; plan §3.5's "typed state bits, not string
    /// probes" row, the position half). The payload is the TEST itself
    /// (the ordinal rides the variant), not selector text: a renderer
    /// derives its own positional selector, a host probe answers from
    /// the element's DOM-position facts (the interp arm documents the
    /// facts → verdict semantics).
    Structural(Structural),
    /// `sm:` / `md:` / ... — min-width media query.
    Bp(Bp),
    /// `min-md:` — explicit min-width form.
    Min(Bp),
    /// `max-md:` — max-width media query.
    Max(Bp),
    /// `@md:` — container query (min-width on the nearest eligible
    /// container). The named form `@md/card:` carries the container's
    /// NAME — the payload that keeps two different containers two
    /// different conditions (the v0.9 oracle's `@md/name` fold; a bare
    /// `@md:` folds with `name: None`). The payload interns into the
    /// same leak pool the attribute probes use, so the static form
    /// stays a `static` item end to end.
    Container {
        /// The container-query breakpoint (`@md:` → [`Bp::Md`]).
        bp: Bp,
        /// The named container (`@md/card:` → `Some("card")`); `None`
        /// for the bare `@md:` form (the nearest eligible container).
        name: Option<&'static str>,
    },
    /// `@max-md:` — upper-bounded container query. Same payload
    /// contract as [`Cond::Container`].
    MaxContainer {
        /// The container-query breakpoint (`@max-md:` → [`Bp::Md`]).
        bp: Bp,
        /// The named container (`@max-lg/sidebar:` → `Some("sidebar")`).
        name: Option<&'static str>,
    },
    /// `supports-[<condition>]:` — the `@supports` at-rule condition
    /// (tailwindcss v4.1.17's arbitrary `supports-[…]` family; no named
    /// spelling exists in v4, so the bracketed form is the family's
    /// whole grammar). The payload is the RENDERED condition text —
    /// everything after the `@supports ` head (`(display:grid)`,
    /// `selector(:hover)`, `(display: var(--tw))`) — because the fold's
    /// normalization IS the render (the tailwindcss v4.1.17 source's own
    /// rule, pinned as the differential evidence: function-prefixed
    /// inners pass through verbatim with `and`/`or`/`not` connectives
    /// spaced, colon-bearing declarations wrap in parens, bare features
    /// expand to the `(feature: var(--tw))` declaration probe). There is
    /// no row table for a supports condition to be relative to — the
    /// row-text boundary that keeps `Val::Kw` row-relative does not
    /// apply here, the [`Cond::Has`] payload precedent.
    Supports(&'static str),
    /// `ltr:` / `rtl:` — the element's resolved text-direction probe
    /// ([`Direction`]; the v4.1.17 static variants' selector-side
    /// family — the direction fact is FRAME STATE, the tier the
    /// preference and capability bits ride, so the `Env` answers and a
    /// renderer derives its own `:where(...)` form from the bit).
    Dir(Direction),
    /// `motion-safe:` / `motion-reduce:` / `contrast-more:` /
    /// `contrast-less:` / `forced-colors:` / `print:` — the
    /// media-feature tier (tailwindcss v4.1.17's static variants; the
    /// at-rule texts the corpus renders are the [`Pref`] enum's own
    /// doc pins). The payload is the TYPED bit (plan §3.5's "typed
    /// state bits, not string probes" row): the user agent's
    /// preference facts are frame state, the `Env` carries
    /// one fact per question, and a renderer derives its own
    /// media-query form from the bit.
    Media(Pref),
}

impl Cond {
    /// The specificity rank used to sort [`crate::Program`] layers. Higher
    /// fires later; equal ranks keep the source order (stable sort). The
    /// exact ranks mirror the v4 cascade contract the v0.x resolver's
    /// `sort_key` already implements (probe 2 < parent 3 < media 4 <
    /// container 5, dark as the mode tier above base).
    pub const fn specificity(self) -> u8 {
        match self {
            Cond::Base => 0,
            Cond::Dark => 1,
            Cond::Hover
            | Cond::Focus
            | Cond::FocusVisible
            | Cond::FocusWithin
            | Cond::Active
            | Cond::Disabled
            | Cond::Visited
            | Cond::Checked
            | Cond::PlaceholderShown
            // The validation states ride the element-own probe tier
            // beside them: the constraint test is a selector conjunct
            // (`:invalid`, `:user-valid`), not an at-rule.
            | Cond::Valid
            | Cond::Invalid
            | Cond::UserValid
            | Cond::UserInvalid
            // The element-state family rides the element-own probe tier
            // beside the validation states: the fragment-target,
            // indeterminate, autofill and read-only tests are selector
            // conjuncts (`:target`, `:read-only`), not at-rules.
            | Cond::Target
            | Cond::Indeterminate
            | Cond::Autofill
            | Cond::ReadOnly
            // The widget-state family rides the same element-own probe
            // tier: the open, inert and form-default tests are selector
            // conjuncts (`:is([open], …)`, `:default`), not at-rules.
            | Cond::Open
            | Cond::Inert
            | Cond::Default
            // The form-state family rides the same element-own probe
            // tier: the enabled, required, optional and range tests are
            // selector conjuncts (`:enabled`, `:in-range`), not
            // at-rules.
            | Cond::Enabled
            | Cond::Required
            | Cond::Optional
            | Cond::InRange
            | Cond::OutOfRange
            | Cond::Selection
            | Cond::Placeholder
            // The pseudo-element tier rides the same element-own probe
            // tier beside the selection/placeholder pair: a
            // pseudo-element head is a selector conjunct on the `&`
            // (`::before`, `::file-selector-button`), not an at-rule.
            | Cond::Before
            | Cond::After
            | Cond::Marker
            | Cond::FirstLetter
            | Cond::FirstLine
            | Cond::FileSelector
            | Cond::Backdrop
            | Cond::DetailsContent
            // The attribute probes compound on the element itself, the
            // same tier the pseudo-class states occupy (the corpus's
            // output nests them on the `&` alongside `:hover`). The
            // direction probe rides the same tier: the direction test
            // is a selector conjunct (`:where(:dir(rtl), …)`), not an
            // at-rule.
            | Cond::Data { .. }
            | Cond::Aria { .. }
            | Cond::Has(_)
            | Cond::Dir(_)
            | Cond::Structural(_) => 2,
            Cond::GroupHas(_) | Cond::PeerHas(_) => 3,
            Cond::GroupHover | Cond::GroupFocus | Cond::PeerHover | Cond::PeerFocus => 3,
            // The at-rule tier: the supports condition sorts BESIDE the
            // media forms — tailwindcss v4.1.17's own conditional-rule
            // order (`@media`, `@supports`, `@container`) maps onto the
            // flat tier model as 4 < 5.
            // The at-rule tier: the supports condition and the
            // media-feature preferences sort BESIDE the media forms —
            // tailwindcss v4.1.17's own conditional-rule order
            // (`@media`, `@supports`, `@container`) maps onto the flat
            // tier model as 4 < 5.
            Cond::Bp(_)
            | Cond::Min(_)
            | Cond::Max(_)
            | Cond::Supports(_)
            | Cond::Media(_) => 4,
            Cond::Container { .. } | Cond::MaxContainer { .. } => 5,
        }
    }

    /// The canonical variant text this condition came from (diagnostics
    /// and the P2 differential harness). `None` for [`Cond::Base`].
    pub const fn source(self) -> Option<&'static str> {
        match self {
            Cond::Base => None,
            Cond::Dark => Some("dark"),
            // The media-feature spellings are CLOSED (the Pref enum's
            // own table) — the condition reconstructs its text.
            Cond::Media(pref) => Some(pref.source()),
            Cond::Hover => Some("hover"),
            Cond::Focus => Some("focus"),
            Cond::FocusVisible => Some("focus-visible"),
            Cond::FocusWithin => Some("focus-within"),
            Cond::Active => Some("active"),
            Cond::Disabled => Some("disabled"),
            Cond::Visited => Some("visited"),
            Cond::Checked => Some("checked"),
            Cond::PlaceholderShown => Some("placeholder-shown"),
            // The validation spellings are bare variants: the condition
            // names its own closed text (the hover arm's shape).
            Cond::Valid => Some("valid"),
            Cond::Invalid => Some("invalid"),
            Cond::UserValid => Some("user-valid"),
            Cond::UserInvalid => Some("user-invalid"),
            // The element-state spellings are bare variants: the
            // condition names its own closed text (the validation
            // arm's shape).
            Cond::Target => Some("target"),
            Cond::Indeterminate => Some("indeterminate"),
            Cond::Autofill => Some("autofill"),
            Cond::ReadOnly => Some("read-only"),
            // The widget-state spellings are bare variants: the
            // condition names its own closed text (the element-state
            // arm's shape).
            Cond::Open => Some("open"),
            Cond::Inert => Some("inert"),
            Cond::Default => Some("default"),
            // The form-state spellings are bare variants: the condition
            // names its own closed text (the widget-state arm's shape).
            Cond::Enabled => Some("enabled"),
            Cond::Required => Some("required"),
            Cond::Optional => Some("optional"),
            Cond::InRange => Some("in-range"),
            Cond::OutOfRange => Some("out-of-range"),
            Cond::Selection => Some("selection"),
            Cond::Placeholder => Some("placeholder"),
            // The pseudo-element tier's spellings are bare variants:
            // the condition names its own closed text (the
            // selection/placeholder arm's shape).
            Cond::Before => Some("before"),
            Cond::After => Some("after"),
            Cond::Marker => Some("marker"),
            Cond::FirstLetter => Some("first-letter"),
            Cond::FirstLine => Some("first-line"),
            Cond::FileSelector => Some("file"),
            Cond::Backdrop => Some("backdrop"),
            Cond::DetailsContent => Some("details-content"),
            // The direction spellings are CLOSED (the Direction enum's
            // own table) — the condition reconstructs its text.
            Cond::Dir(dir) => Some(dir.source()),
            Cond::GroupHover => Some("group-hover"),
            Cond::GroupFocus => Some("group-focus"),
            Cond::PeerHover => Some("peer-hover"),
            Cond::PeerFocus => Some("peer-focus"),
            // The attribute probes carry their spelling in the payload —
            // the segment text itself (`data-[state=open]`, `has-focus`),
            // which no const table reconstructs. `source_text` returns
            // None for them: the diagnostic surfaces carry the payload
            // when a reader needs the probe text. The supports condition
            // rides the same precedent (its payload IS the condition
            // text; the spelling it came from is the segment's own).
            Cond::Data { .. }
            | Cond::Aria { .. }
            | Cond::Has(_)
            | Cond::GroupHas(_)
            | Cond::PeerHas(_)
            | Cond::Supports(_)
            | Cond::Structural(_) => None,
            // Breakpoint names need the u8 formatting — not const. The
            // non-const wrapper covers them (see `source_text`), and a
            // NAMED container form has no static text either way (the
            // name is the payload, the attribute-probe precedent).
            Cond::Bp(_)
            | Cond::Min(_)
            | Cond::Max(_)
            | Cond::Container { .. }
            | Cond::MaxContainer { .. } => None,
        }
    }

    /// The canonical variant text including breakpoint forms
    /// (`"md"`, `"@max-sm"`, ...). The non-const sibling of
    /// [`Cond::source`]; table lookups instead of `format!` keep every
    /// name a `&'static str`.
    pub fn source_text(self) -> Option<&'static str> {
        const BARE: [&str; 5] = ["sm", "md", "lg", "xl", "2xl"];
        const MIN: [&str; 5] = ["min-sm", "min-md", "min-lg", "min-xl", "min-2xl"];
        const MAX: [&str; 5] = ["max-sm", "max-md", "max-lg", "max-xl", "max-2xl"];
        const CONTAINER: [&str; 5] = ["@sm", "@md", "@lg", "@xl", "@2xl"];
        const MAX_CONTAINER: [&str; 5] = ["@max-sm", "@max-md", "@max-lg", "@max-xl", "@max-2xl"];
        let (table, i) = match self {
            Cond::Bp(b) => (&BARE, b as usize),
            Cond::Min(b) => (&MIN, b as usize),
            Cond::Max(b) => (&MAX, b as usize),
            // The unnamed forms keep the table spellings; a NAMED form
            // has no static text (the name is the payload, the
            // attribute-probe precedent) — `None`.
            Cond::Container { name: None, bp } => (&CONTAINER, bp as usize),
            Cond::MaxContainer { name: None, bp } => (&MAX_CONTAINER, bp as usize),
            Cond::Container { name: Some(_), .. } | Cond::MaxContainer { name: Some(_), .. } => {
                return None;
            }
            other => return other.source(),
        };
        Some(table[i])
    }
}
