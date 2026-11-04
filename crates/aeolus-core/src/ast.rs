//! AST for a single Tailwind v4 class token.

/// One class token, e.g. `dark:group-hover:bg-primary/90!`.
#[derive(Clone, Debug, PartialEq)]
pub struct ClassToken<'a> {
    /// Variants in **source order** (left-to-right stacking, v4 semantics).
    /// The first variant is the outermost condition.
    pub variants: Vec<Variant<'a>>,
    /// Dot-separated utility elements, e.g. `["bg", "primary"]`, `["rounded", "t", "lg"]`.
    pub elements: Vec<&'a str>,
    /// `[...]` arbitrary value without the brackets, e.g. `"3px"` for `ring-[3px]`.
    pub arbitrary: Option<&'a str>,
    /// `/opacity` modifier as a percent (0..=100), e.g. 50 for `/50`.
    pub opacity: Option<f32>,
    /// Leading `-` negation (`-mt-4`).
    pub negative: bool,
    /// v4 trailing `!` or legacy leading `!`.
    pub important: bool,
    /// Raw source span (for diagnostics).
    pub raw: &'a str,
}

/// A single variant prefix. Arbitrary/functional variants are modeled
/// explicitly so backends can reject them with a precise warning.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Variant<'a> {
    /// `hover`, `focus`, `active`, `disabled`, `dark`, `sm`... — a named
    /// variant looked up in the variant table.
    Named(NamedVariant),
    /// `group-hover`, `group-focus/label`, `peer-focus` — parent-relative.
    Grouped {
        /// The parent family (`group` or `peer`).
        parent: ParentKind,
        /// The guarded state on the parent/sibling.
        name: NamedVariant,
        /// Optional named container (`group-hover/menu`).
        label: Option<&'a str>,
    },
    /// `data-[state=open]`, `aria-[busy=true]`, `[&_svg]:...` — unsupported in v1.
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

/// The variant table — the canonical v4 order matters for cascade stability
/// but each layer is keyed by its condition, so we only need identity here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NamedVariant {
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
    /// `dark` (class/OS dark mode).
    Dark,
    /// Responsive breakpoints (`sm`..`2xl`) resolved against `--breakpoint-*`.
    Breakpoint(Breakpoint),
    /// `@container`-family variants (`@sm`, `@md`, ...) — container queries.
    Container(Breakpoint),
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
}
