//! Verbatim Shadcn v4 class tables (`new-york-v4` registry).
//!
//! Source: `shadcn-ui/ui @ 4aaea19` (2026-11-02),
//! `apps/v4/registry/new-york-v4/ui/*.tsx`. Multi-space runs inside the
//! strings are preserved; `cn()` treats runs of whitespace as separators.

// ---------------------------------------------------------------------------
// Button
// ---------------------------------------------------------------------------

pub const BUTTON_BASE: &str = "inline-flex shrink-0 items-center justify-center gap-2 rounded-md text-sm font-medium whitespace-nowrap transition-all outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50 aria-invalid:border-destructive aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4";

/// `variant` axis of the v4 Button.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    #[default]
    Default,
    Destructive,
    Outline,
    Secondary,
    Ghost,
    Link,
}

impl ButtonVariant {
    pub fn classes(self) -> &'static str {
        match self {
            ButtonVariant::Default => "bg-primary text-primary-foreground hover:bg-primary/90",
            ButtonVariant::Destructive => "bg-destructive text-white hover:bg-destructive/90 focus-visible:ring-destructive/20 dark:bg-destructive/60 dark:focus-visible:ring-destructive/40",
            ButtonVariant::Outline => "border bg-background shadow-xs hover:bg-accent hover:text-accent-foreground dark:border-input dark:bg-input/30 dark:hover:bg-input/50",
            ButtonVariant::Secondary => "bg-secondary text-secondary-foreground hover:bg-secondary/80",
            ButtonVariant::Ghost => "hover:bg-accent hover:text-accent-foreground dark:hover:bg-accent/50",
            ButtonVariant::Link => "text-primary underline-offset-4 hover:underline",
        }
    }
}

/// `size` axis of the v4 Button.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ButtonSize {
    #[default]
    Default,
    Xs,
    Sm,
    Lg,
    Icon,
    IconXs,
    IconSm,
    IconLg,
}

impl ButtonSize {
    pub fn classes(self) -> &'static str {
        match self {
            ButtonSize::Default => "h-9 px-4 py-2 has-[>svg]:px-3",
            ButtonSize::Xs => "h-6 gap-1 rounded-md px-2 text-xs has-[>svg]:px-1.5 [&_svg:not([class*='size-'])]:size-3",
            ButtonSize::Sm => "h-8 gap-1.5 rounded-md px-3 has-[>svg]:px-2.5",
            ButtonSize::Lg => "h-10 rounded-md px-6 has-[>svg]:px-4",
            ButtonSize::Icon => "size-9",
            ButtonSize::IconXs => "size-6 rounded-md [&_svg:not([class*='size-'])]:size-3",
            ButtonSize::IconSm => "size-8",
            ButtonSize::IconLg => "size-10",
        }
    }
}

/// Full Button class string for a variant/size, before user classes.
pub fn button_classes(variant: ButtonVariant, size: ButtonSize) -> String {
    cn(&[BUTTON_BASE, variant.classes(), size.classes()])
}

// ---------------------------------------------------------------------------
// Input
// ---------------------------------------------------------------------------

pub const INPUT_BASE: &str = "h-9 w-full min-w-0 rounded-md border border-input bg-transparent px-3 py-1 text-base shadow-xs transition-[color,box-shadow] outline-none selection:bg-primary selection:text-primary-foreground file:inline-flex file:h-7 file:border-0 file:bg-transparent file:text-sm file:font-medium file:text-foreground placeholder:text-muted-foreground disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50 md:text-sm dark:bg-input/30";

// ---------------------------------------------------------------------------
// Label
// ---------------------------------------------------------------------------

pub const LABEL_BASE: &str = "flex items-center gap-2 text-sm leading-none font-medium select-none group-data-[disabled=true]:pointer-events-none group-data-[disabled=true]:opacity-50 peer-disabled:cursor-not-allowed peer-disabled:opacity-50";

// ---------------------------------------------------------------------------
// Card
// ---------------------------------------------------------------------------

pub const CARD: &str =
    "flex flex-col gap-6 rounded-xl border bg-card py-6 text-card-foreground shadow-sm";
pub const CARD_HEADER: &str = "@container/card-header grid auto-rows-min grid-rows-[auto_auto] items-start gap-2 px-6 has-data-[slot=card-action]:grid-cols-[1fr_auto] [.border-b]:pb-6";
pub const CARD_TITLE: &str = "leading-none font-semibold";
pub const CARD_DESCRIPTION: &str = "text-sm text-muted-foreground";
pub const CARD_CONTENT: &str = "px-6";
pub const CARD_FOOTER: &str = "flex items-center px-6 [.border-t]:pt-6";

// ---------------------------------------------------------------------------
// Badge
// ---------------------------------------------------------------------------

pub const BADGE_BASE: &str = "inline-flex w-fit shrink-0 items-center justify-center gap-1 overflow-hidden rounded-full border border-transparent px-2 py-0.5 text-xs font-medium whitespace-nowrap transition-[color,box-shadow] focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 aria-invalid:border-destructive aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 [&>svg]:pointer-events-none [&>svg]:size-3";

/// `variant` axis of the v4 Badge.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum BadgeVariant {
    #[default]
    Default,
    Secondary,
    Destructive,
    Outline,
    Ghost,
    Link,
}

impl BadgeVariant {
    pub fn classes(self) -> &'static str {
        match self {
            BadgeVariant::Default => "bg-primary text-primary-foreground [a&]:hover:bg-primary/90",
            BadgeVariant::Secondary => "bg-secondary text-secondary-foreground [a&]:hover:bg-secondary/90",
            BadgeVariant::Destructive => "bg-destructive text-white focus-visible:ring-destructive/20 dark:bg-destructive/60 dark:focus-visible:ring-destructive/40 [a&]:hover:bg-destructive/90",
            BadgeVariant::Outline => "border-border text-foreground [a&]:hover:bg-accent [a&]:hover:text-accent-foreground",
            BadgeVariant::Ghost => "[a&]:hover:bg-accent [a&]:hover:text-accent-foreground",
            BadgeVariant::Link => "text-primary underline-offset-4 [a&]:hover:underline",
        }
    }
}

pub fn badge_classes(variant: BadgeVariant) -> String {
    cn(&[BADGE_BASE, variant.classes()])
}

// ---------------------------------------------------------------------------
// Separator
// ---------------------------------------------------------------------------

/// v4 Separator uses `data-[orientation]` variants; native backends pick the
/// branch explicitly, so the two orientation strings are kept separate here.
pub const SEPARATOR_BASE: &str = "shrink-0 bg-border";
pub const SEPARATOR_HORIZONTAL: &str = "h-px w-full";
pub const SEPARATOR_VERTICAL: &str = "h-full w-px";

// ---------------------------------------------------------------------------
// Skeleton
// ---------------------------------------------------------------------------

pub const SKELETON_BASE: &str = "animate-pulse rounded-md bg-accent";

// ---------------------------------------------------------------------------
// Alert
// ---------------------------------------------------------------------------

pub const ALERT_BASE: &str = "relative grid w-full grid-cols-[0_1fr] items-start gap-y-0.5 rounded-lg border px-4 py-3 text-sm has-[>svg]:grid-cols-[calc(var(--spacing)*4)_1fr] has-[>svg]:gap-x-3 [&>svg]:size-4 [&>svg]:translate-y-0.5 [&>svg]:text-current";

/// `variant` axis of the v4 Alert.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum AlertVariant {
    #[default]
    Default,
    Destructive,
}

impl AlertVariant {
    pub fn classes(self) -> &'static str {
        match self {
            AlertVariant::Default => "bg-card text-card-foreground",
            AlertVariant::Destructive => "bg-card text-destructive *:data-[slot=alert-description]:text-destructive/90 [&>svg]:text-current",
        }
    }
}

pub const ALERT_TITLE: &str =
    "col-start-2 line-clamp-1 min-h-4 font-medium tracking-tight";
pub const ALERT_DESCRIPTION: &str =
    "col-start-2 grid justify-items-start gap-1 text-sm text-muted-foreground [&_p]:leading-relaxed";

pub fn alert_classes(variant: AlertVariant) -> String {
    cn(&[ALERT_BASE, variant.classes()])
}

use aeolus_core::cn;
