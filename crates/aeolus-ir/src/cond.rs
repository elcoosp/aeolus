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
}

/// The condition a layer fires under. `Copy`, `const`-constructible, no
/// heap: the static program is a `static` item end to end.
///
/// Specificity (the [`Cond::specificity`] rank) mirrors the v4 cascade:
/// base < mode < probe states < parent-relative < media < container.
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
            | Cond::Placeholder => 2,
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
