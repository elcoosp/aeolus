//! The closed-world CONST tables (architecture plan §3.6's surface (2),
//! §10's "shadcn components are `const` tables" headline — the M3
//! slice's promise made real): every (slot, combo) arm compiles at
//! EXPANSION time to a `&'static` program through
//! [`crate::variants_static!`] over the embedded shadcn bake — a call is
//! the match dispatch plus the [`aeolus_rt::StaticProgram`] Copy, never
//! a compile and never the first-call intern the
//! [`crate::program::ButtonPrograms`] RefCell handle ran (the F6 fix,
//! now the headline form).
//!
//! `allow_refusals: true` rides every table: the verbatim v4 strings
//! carry the documented arbitrary-variant/attribute boundary families
//! (`[&_svg]`, `has-[>svg]`, `data-[orientation]`, `[.border-b]`) — the
//! runtime-interned path reports them as TYPED diagnostics (the
//! handle's honesty channel), the static form freezes the SAME lowered
//! program and keeps the runtime path as the diagnostics source of
//! record beside it. The differential pins the bytes: every combo's
//! static program is byte-identical to the pre-migration
//! [`crate::program`] path's (the `Combo.program` bytes, keyword slices
//! included).
//!
//! The runtime-interned Combo path stays the FALLBACK for dynamic
//! strings — a host composing `SEPARATOR_BASE` with an orientation
//! rides the [`aeolus_rt::ClassCell`] lane (the §3.6 "host composition"
//! precedent); the two paths produce byte-identical programs for the
//! same class string (the differential pin).

crate::variants_static! {
    pub Button {
        allow_refusals: true,
        base: "inline-flex shrink-0 items-center justify-center gap-2 rounded-md text-sm font-medium whitespace-nowrap transition-all outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50 aria-invalid:border-destructive aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4",
        variant: {
            default: "bg-primary text-primary-foreground hover:bg-primary/90",
            destructive: "bg-destructive text-white hover:bg-destructive/90 focus-visible:ring-destructive/20 dark:bg-destructive/60 dark:focus-visible:ring-destructive/40",
            outline: "border bg-background shadow-xs hover:bg-accent hover:text-accent-foreground dark:border-input dark:bg-input/30 dark:hover:bg-input/50",
            secondary: "bg-secondary text-secondary-foreground hover:bg-secondary/80",
            ghost: "hover:bg-accent hover:text-accent-foreground dark:hover:bg-accent/50",
            link: "text-primary underline-offset-4 hover:underline",
        },
        size: {
            default: "h-9 px-4 py-2 has-[>svg]:px-3",
            xs: "h-6 gap-1 rounded-md px-2 text-xs has-[>svg]:px-1.5 [&_svg:not([class*='size-'])]:size-3",
            sm: "h-8 gap-1.5 rounded-md px-3 has-[>svg]:px-2.5",
            lg: "h-10 rounded-md px-6 has-[>svg]:px-4",
            icon: "size-9",
            icon_xs: "size-6 rounded-md [&_svg:not([class*='size-'])]:size-3",
            icon_sm: "size-8",
            icon_lg: "size-10",
        },
    }
}

crate::variants_static! {
    pub Badge {
        allow_refusals: true,
        base: "inline-flex w-fit shrink-0 items-center justify-center gap-1 overflow-hidden rounded-full border border-transparent px-2 py-0.5 text-xs font-medium whitespace-nowrap transition-[color,box-shadow] focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 aria-invalid:border-destructive aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 [&>svg]:pointer-events-none [&>svg]:size-3",
        variant: {
            default: "bg-primary text-primary-foreground [a&]:hover:bg-primary/90",
            secondary: "bg-secondary text-secondary-foreground [a&]:hover:bg-secondary/90",
            destructive: "bg-destructive text-white focus-visible:ring-destructive/20 dark:bg-destructive/60 dark:focus-visible:ring-destructive/40 [a&]:hover:bg-destructive/90",
            outline: "border-border text-foreground [a&]:hover:bg-accent [a&]:hover:text-accent-foreground",
            ghost: "[a&]:hover:bg-accent [a&]:hover:text-accent-foreground",
            link: "text-primary underline-offset-4 [a&]:hover:underline",
        },
    }
}

crate::variants_static! {
    pub Alert {
        allow_refusals: true,
        base: "relative grid w-full grid-cols-[0_1fr] items-start gap-y-0.5 rounded-lg border px-4 py-3 text-sm has-[>svg]:grid-cols-[calc(var(--spacing)*4)_1fr] has-[>svg]:gap-x-3 [&>svg]:size-4 [&>svg]:translate-y-0.5 [&>svg]:text-current",
        variant: {
            default: "bg-card text-card-foreground",
            destructive: "bg-card text-destructive *:data-[slot=alert-description]:text-destructive/90 [&>svg]:text-current",
        },
    }
}

crate::variants_static! {
    pub Base {
        allow_refusals: true,
        slots: {
            input: "h-9 w-full min-w-0 rounded-md border border-input bg-transparent px-3 py-1 text-base shadow-xs transition-[color,box-shadow] outline-none selection:bg-primary selection:text-primary-foreground file:inline-flex file:h-7 file:border-0 file:bg-transparent file:text-sm file:font-medium file:text-foreground placeholder:text-muted-foreground disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50 md:text-sm dark:bg-input/30",
            label: "flex items-center gap-2 text-sm leading-none font-medium select-none group-data-[disabled=true]:pointer-events-none group-data-[disabled=true]:opacity-50 peer-disabled:cursor-not-allowed peer-disabled:opacity-50",
            card: "flex flex-col gap-6 rounded-xl border bg-card py-6 text-card-foreground shadow-sm",
            card_header: "@container/card-header grid auto-rows-min grid-rows-[auto_auto] items-start gap-2 px-6 has-data-[slot=card-action]:grid-cols-[1fr_auto] [.border-b]:pb-6",
            card_title: "leading-none font-semibold",
            card_description: "text-sm text-muted-foreground",
            card_content: "px-6",
            card_footer: "flex items-center px-6 [.border-t]:pt-6",
            separator: "shrink-0 bg-border",
            separator_horizontal: "h-px w-full",
            separator_vertical: "h-full w-px",
            skeleton: "animate-pulse rounded-md bg-accent",
            alert_title: "col-start-2 line-clamp-1 min-h-4 font-medium tracking-tight",
            alert_description: "col-start-2 grid justify-items-start gap-1 text-sm text-muted-foreground [&_p]:leading-relaxed",
        },
    }
}
