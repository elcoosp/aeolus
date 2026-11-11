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

/// The condition a layer fires under. `Copy`, `const`-constructible, no
/// heap: the static program is a `static` item end to end.
///
/// Specificity (the [`Cond::specificity`] rank) mirrors the v4 cascade:
/// base < mode < probe states < parent-relative < media < container.
///
/// The three attribute-probe variants ([`Cond::Data`], [`Cond::Aria`],
/// [`Cond::Has`]) carry their payload as `&'static str`: a spelling the
/// compile stage folds interns the payload into the same leak pool the
/// program's slices use (the IR's documented dynamic-form storage —
/// "one interned `Box<[Decl]>` shared by reference"), and a literal
/// spelling is `&'static` outright, so the static form stays a `static`
/// item end to end. Equality and hashing compare the PAYLOAD TEXT, not
/// the pointer: two spellings that name the same probe (`data-open:` and
/// `data-[open]:`) are one condition and one layer.
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
    /// `selection:` — the element's own `::selection` rendering (v0.9 §1).
    Selection,
    /// `placeholder:` — the `::placeholder` rendering.
    Placeholder,
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
    /// `sm:` / `md:` / ... — min-width media query.
    Bp(Bp),
    /// `min-md:` — explicit min-width form.
    Min(Bp),
    /// `max-md:` — max-width media query.
    Max(Bp),
    /// `@md:` — container query (min-width on the named container).
    Container(Bp),
    /// `@max-md:` — upper-bounded container query.
    MaxContainer(Bp),
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
            | Cond::Selection
            | Cond::Placeholder
            // The attribute probes compound on the element itself, the
            // same tier the pseudo-class states occupy (the corpus's
            // output nests them on the `&` alongside `:hover`).
            | Cond::Data { .. }
            | Cond::Aria { .. }
            | Cond::Has(_) => 2,
            Cond::GroupHas(_) | Cond::PeerHas(_) => 3,
            Cond::GroupHover | Cond::GroupFocus | Cond::PeerHover | Cond::PeerFocus => 3,
            Cond::Bp(_) | Cond::Min(_) | Cond::Max(_) => 4,
            Cond::Container(_) | Cond::MaxContainer(_) => 5,
        }
    }

    /// The canonical variant text this condition came from (diagnostics
    /// and the P2 differential harness). `None` for [`Cond::Base`].
    pub const fn source(self) -> Option<&'static str> {
        match self {
            Cond::Base => None,
            Cond::Dark => Some("dark"),
            Cond::Hover => Some("hover"),
            Cond::Focus => Some("focus"),
            Cond::FocusVisible => Some("focus-visible"),
            Cond::FocusWithin => Some("focus-within"),
            Cond::Active => Some("active"),
            Cond::Disabled => Some("disabled"),
            Cond::Visited => Some("visited"),
            Cond::Checked => Some("checked"),
            Cond::PlaceholderShown => Some("placeholder-shown"),
            Cond::Selection => Some("selection"),
            Cond::Placeholder => Some("placeholder"),
            Cond::GroupHover => Some("group-hover"),
            Cond::GroupFocus => Some("group-focus"),
            Cond::PeerHover => Some("peer-hover"),
            Cond::PeerFocus => Some("peer-focus"),
            // The attribute probes carry their spelling in the payload —
            // the segment text itself (`data-[state=open]`, `has-focus`),
            // which no const table reconstructs. `source_text` returns
            // None for them: the diagnostic surfaces carry the payload
            // when a reader needs the probe text.
            Cond::Data { .. }
            | Cond::Aria { .. }
            | Cond::Has(_)
            | Cond::GroupHas(_)
            | Cond::PeerHas(_) => None,
            // Breakpoint names need the u8 formatting — not const. The
            // non-const wrapper covers them (see `source_text`).
            Cond::Bp(_)
            | Cond::Min(_)
            | Cond::Max(_)
            | Cond::Container(_)
            | Cond::MaxContainer(_) => None,
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
            Cond::Container(b) => (&CONTAINER, b as usize),
            Cond::MaxContainer(b) => (&MAX_CONTAINER, b as usize),
            other => return other.source(),
        };
        Some(table[i])
    }
}
