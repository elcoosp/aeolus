//! The committed value tables — the v4 default scales the registry rows
//! reference. Plain `const` data: auditable against tailwindcss v4's
//! `theme.css`, one table per scale, nothing generated this round (the
//! P2 xtask grows the generator FROM these rows — see the crate docs).
//! Numeric scales are stored as their CSS spellings so the keyword-table
//! kinds can reference them verbatim.

use crate::ValueKind;

/// The default spacing step: one Tailwind unit = 0.25rem (`p-4` = `1rem`).
pub const SPACING_STEP_REM: f32 = 0.25;

/// `text-{size}` → (name, font-size CSS, paired line-height CSS) — the
/// v4 default type scale with the calculated `--text-*--line-height`
/// pairs verbatim from theme.css.
pub const TEXT_SIZES: &[(&str, &str, &str)] = &[
    ("xs", "0.75rem", "calc(1 / 0.75)"),
    ("sm", "0.875rem", "calc(1.25 / 0.875)"),
    ("base", "1rem", "calc(1.5 / 1)"),
    ("lg", "1.125rem", "calc(1.75 / 1.125)"),
    ("xl", "1.25rem", "calc(1.75 / 1.25)"),
    ("2xl", "1.5rem", "calc(2 / 1.5)"),
    ("3xl", "1.875rem", "calc(2.25 / 1.875)"),
    ("4xl", "2.25rem", "calc(2.5 / 2.25)"),
    ("5xl", "3rem", "1"),
    ("6xl", "3.75rem", "1"),
    ("7xl", "4.5rem", "1"),
    ("8xl", "6rem", "1"),
    ("9xl", "8rem", "1"),
];

/// `font-{weight}` → weight CSS.
pub const FONT_WEIGHTS: &[(&str, &str)] = &[
    ("thin", "100"),
    ("extralight", "200"),
    ("light", "300"),
    ("normal", "400"),
    ("medium", "500"),
    ("semibold", "600"),
    ("bold", "700"),
    ("extrabold", "800"),
    ("black", "900"),
];

/// `font-{family}` → family CSS (the three v4 named stacks).
pub const FONT_FAMILIES: &[(&str, &str)] = &[
    ("sans", "var(--font-sans)"),
    ("serif", "var(--font-serif)"),
    ("mono", "var(--font-mono)"),
];

/// `rounded-{scale}` → radius CSS (v4 defaults; `full` is the 9999px
/// sentinel the px pipeline already pins).
pub const RADII: &[(&str, &str)] = &[
    ("xs", "0.125rem"),
    ("sm", "0.25rem"),
    ("md", "0.375rem"),
    ("lg", "0.5rem"),
    ("xl", "0.75rem"),
    ("2xl", "1rem"),
    ("3xl", "1.5rem"),
    ("4xl", "2rem"),
    ("full", "9999px"),
    ("none", "0px"),
];

/// `tracking-{scale}` → letter-spacing CSS (v4 defaults, negative steps
/// included — the row carries `negative = false` because the SCALE is
/// signed, not the class).
pub const TRACKING: &[(&str, &str)] = &[
    ("tighter", "-0.05em"),
    ("tight", "-0.025em"),
    ("normal", "0em"),
    ("wide", "0.025em"),
    ("wider", "0.05em"),
    ("widest", "0.1em"),
];

/// Bare `leading-{scale}` → unitless line-height CSS (v4.1 default scale;
/// the numeric `leading-6` form rides the spacing scale instead).
pub const LEADING_RATIOS: &[(&str, &str)] = &[
    ("none", "1"),
    ("tight", "1.25"),
    ("snug", "1.375"),
    ("normal", "1.5"),
    ("relaxed", "1.625"),
    ("loose", "2"),
];

/// `max-w-{scale}` → CSS width (the v4.1 container scale; `min-w`/`max-h`
/// reuse the spacing scale + the shared size keywords).
pub const CONTAINER_MAX_WIDTHS: &[(&str, &str)] = &[
    ("3xs", "16rem"),
    ("2xs", "18rem"),
    ("xs", "20rem"),
    ("sm", "24rem"),
    ("md", "28rem"),
    ("lg", "32rem"),
    ("xl", "36rem"),
    ("2xl", "42rem"),
    ("3xl", "48rem"),
    ("4xl", "56rem"),
    ("5xl", "64rem"),
    ("6xl", "72rem"),
    ("7xl", "80rem"),
];

/// The shared size keywords for the `w`/`h`/`min-*`/`max-*`/`size`
/// families (`w-full`, `h-screen`, `w-fit`, ...).
pub const SIZE_KEYWORDS: &[(&str, &str)] = &[
    ("auto", "auto"),
    ("full", "100%"),
    ("screen", "100vw"),
    ("min", "min-content"),
    ("max", "max-content"),
    ("fit", "fit-content"),
];

/// The `auto` placement keyword (`mx-auto`, `top-auto`).
pub const AUTO_KEYWORD: &[(&str, &str)] = &[("auto", "auto")];

/// `z-{level}` → z-index CSS.
pub const Z_LEVELS: &[(&str, &str)] = &[
    ("0", "0"),
    ("10", "10"),
    ("20", "20"),
    ("30", "30"),
    ("40", "40"),
    ("50", "50"),
    ("auto", "auto"),
];

/// `opacity-{step}` → percent (÷100 at lowering).
pub const OPACITY_STEPS: &[u16] = &[
    0, 5, 10, 15, 20, 25, 30, 35, 40, 45, 50, 55, 60, 65, 70, 75, 80, 85, 90, 95, 100,
];

/// `border-{step}` → px width (the v4 numeric border scale; bare `border`
/// is the 1px default).
pub const BORDER_WIDTH_STEPS: &[(&str, &str)] =
    &[("0", "0px"), ("2", "2px"), ("4", "4px"), ("8", "8px")];

/// The border STYLE keywords. NOTE (tw_merge quirk, oracle-faithful):
/// `border-dashed` & co. group under `border-color` in the conflict
/// table — the row carries that group, not `border-style`.
pub const BORDER_STYLES: &[(&str, &str)] = &[
    ("solid", "solid"),
    ("dashed", "dashed"),
    ("dotted", "dotted"),
    ("double", "double"),
    ("none", "none"),
    ("hidden", "hidden"),
];

/// `outline-{step}` → px width.
pub const OUTLINE_WIDTHS: &[(&str, &str)] = &[
    ("0", "0px"),
    ("1", "1px"),
    ("2", "2px"),
    ("4", "4px"),
    ("8", "8px"),
];

/// `outline-{style}` keywords.
pub const OUTLINE_STYLES: &[(&str, &str)] = &[
    ("solid", "solid"),
    ("dashed", "dashed"),
    ("dotted", "dotted"),
    ("double", "double"),
];

/// `ring-offset-{step}` → px width.
pub const RING_OFFSET_WIDTHS: &[(&str, &str)] = &[
    ("0", "0px"),
    ("1", "1px"),
    ("2", "2px"),
    ("4", "4px"),
    ("8", "8px"),
];

/// The colour surface: the full v4 default palette (name →
/// `var(--color-<name>)` theme reference), the plain keywords, and the
/// shadcn token names the corpus consumes. One table — the row for
/// `bg`/`text`-color/`border`-color all point here, which is the
/// one-source-of-truth claim `generate_docs()` renders.
pub const COLOR_KEYWORDS: &[(&str, &str)] = &[
    // Plain CSS keywords.
    ("inherit", "inherit"),
    ("current", "currentColor"),
    ("transparent", "transparent"),
    ("black", "var(--color-black)"),
    ("white", "var(--color-white)"),
    // The v4 default palette (every shade), theme-referenced.
    ("red-50", "var(--color-red-50)"),
    ("red-100", "var(--color-red-100)"),
    ("red-200", "var(--color-red-200)"),
    ("red-300", "var(--color-red-300)"),
    ("red-400", "var(--color-red-400)"),
    ("red-500", "var(--color-red-500)"),
    ("red-600", "var(--color-red-600)"),
    ("red-700", "var(--color-red-700)"),
    ("red-800", "var(--color-red-800)"),
    ("red-900", "var(--color-red-900)"),
    ("red-950", "var(--color-red-950)"),
    ("orange-50", "var(--color-orange-50)"),
    ("orange-100", "var(--color-orange-100)"),
    ("orange-200", "var(--color-orange-200)"),
    ("orange-300", "var(--color-orange-300)"),
    ("orange-400", "var(--color-orange-400)"),
    ("orange-500", "var(--color-orange-500)"),
    ("orange-600", "var(--color-orange-600)"),
    ("orange-700", "var(--color-orange-700)"),
    ("orange-800", "var(--color-orange-800)"),
    ("orange-900", "var(--color-orange-900)"),
    ("orange-950", "var(--color-orange-950)"),
    ("amber-50", "var(--color-amber-50)"),
    ("amber-100", "var(--color-amber-100)"),
    ("amber-200", "var(--color-amber-200)"),
    ("amber-300", "var(--color-amber-300)"),
    ("amber-400", "var(--color-amber-400)"),
    ("amber-500", "var(--color-amber-500)"),
    ("amber-600", "var(--color-amber-600)"),
    ("amber-700", "var(--color-amber-700)"),
    ("amber-800", "var(--color-amber-800)"),
    ("amber-900", "var(--color-amber-900)"),
    ("amber-950", "var(--color-amber-950)"),
    ("yellow-50", "var(--color-yellow-50)"),
    ("yellow-100", "var(--color-yellow-100)"),
    ("yellow-200", "var(--color-yellow-200)"),
    ("yellow-300", "var(--color-yellow-300)"),
    ("yellow-400", "var(--color-yellow-400)"),
    ("yellow-500", "var(--color-yellow-500)"),
    ("yellow-600", "var(--color-yellow-600)"),
    ("yellow-700", "var(--color-yellow-700)"),
    ("yellow-800", "var(--color-yellow-800)"),
    ("yellow-900", "var(--color-yellow-900)"),
    ("yellow-950", "var(--color-yellow-950)"),
    ("lime-50", "var(--color-lime-50)"),
    ("lime-100", "var(--color-lime-100)"),
    ("lime-200", "var(--color-lime-200)"),
    ("lime-300", "var(--color-lime-300)"),
    ("lime-400", "var(--color-lime-400)"),
    ("lime-500", "var(--color-lime-500)"),
    ("lime-600", "var(--color-lime-600)"),
    ("lime-700", "var(--color-lime-700)"),
    ("lime-800", "var(--color-lime-800)"),
    ("lime-900", "var(--color-lime-900)"),
    ("lime-950", "var(--color-lime-950)"),
    ("green-50", "var(--color-green-50)"),
    ("green-100", "var(--color-green-100)"),
    ("green-200", "var(--color-green-200)"),
    ("green-300", "var(--color-green-300)"),
    ("green-400", "var(--color-green-400)"),
    ("green-500", "var(--color-green-500)"),
    ("green-600", "var(--color-green-600)"),
    ("green-700", "var(--color-green-700)"),
    ("green-800", "var(--color-green-800)"),
    ("green-900", "var(--color-green-900)"),
    ("green-950", "var(--color-green-950)"),
    ("emerald-50", "var(--color-emerald-50)"),
    ("emerald-100", "var(--color-emerald-100)"),
    ("emerald-200", "var(--color-emerald-200)"),
    ("emerald-300", "var(--color-emerald-300)"),
    ("emerald-400", "var(--color-emerald-400)"),
    ("emerald-500", "var(--color-emerald-500)"),
    ("emerald-600", "var(--color-emerald-600)"),
    ("emerald-700", "var(--color-emerald-700)"),
    ("emerald-800", "var(--color-emerald-800)"),
    ("emerald-900", "var(--color-emerald-900)"),
    ("emerald-950", "var(--color-emerald-950)"),
    ("teal-50", "var(--color-teal-50)"),
    ("teal-100", "var(--color-teal-100)"),
    ("teal-200", "var(--color-teal-200)"),
    ("teal-300", "var(--color-teal-300)"),
    ("teal-400", "var(--color-teal-400)"),
    ("teal-500", "var(--color-teal-500)"),
    ("teal-600", "var(--color-teal-600)"),
    ("teal-700", "var(--color-teal-700)"),
    ("teal-800", "var(--color-teal-800)"),
    ("teal-900", "var(--color-teal-900)"),
    ("teal-950", "var(--color-teal-950)"),
    ("cyan-50", "var(--color-cyan-50)"),
    ("cyan-100", "var(--color-cyan-100)"),
    ("cyan-200", "var(--color-cyan-200)"),
    ("cyan-300", "var(--color-cyan-300)"),
    ("cyan-400", "var(--color-cyan-400)"),
    ("cyan-500", "var(--color-cyan-500)"),
    ("cyan-600", "var(--color-cyan-600)"),
    ("cyan-700", "var(--color-cyan-700)"),
    ("cyan-800", "var(--color-cyan-800)"),
    ("cyan-900", "var(--color-cyan-900)"),
    ("cyan-950", "var(--color-cyan-950)"),
    ("sky-50", "var(--color-sky-50)"),
    ("sky-100", "var(--color-sky-100)"),
    ("sky-200", "var(--color-sky-200)"),
    ("sky-300", "var(--color-sky-300)"),
    ("sky-400", "var(--color-sky-400)"),
    ("sky-500", "var(--color-sky-500)"),
    ("sky-600", "var(--color-sky-600)"),
    ("sky-700", "var(--color-sky-700)"),
    ("sky-800", "var(--color-sky-800)"),
    ("sky-900", "var(--color-sky-900)"),
    ("sky-950", "var(--color-sky-950)"),
    ("blue-50", "var(--color-blue-50)"),
    ("blue-100", "var(--color-blue-100)"),
    ("blue-200", "var(--color-blue-200)"),
    ("blue-300", "var(--color-blue-300)"),
    ("blue-400", "var(--color-blue-400)"),
    ("blue-500", "var(--color-blue-500)"),
    ("blue-600", "var(--color-blue-600)"),
    ("blue-700", "var(--color-blue-700)"),
    ("blue-800", "var(--color-blue-800)"),
    ("blue-900", "var(--color-blue-900)"),
    ("blue-950", "var(--color-blue-950)"),
    ("indigo-50", "var(--color-indigo-50)"),
    ("indigo-100", "var(--color-indigo-100)"),
    ("indigo-200", "var(--color-indigo-200)"),
    ("indigo-300", "var(--color-indigo-300)"),
    ("indigo-400", "var(--color-indigo-400)"),
    ("indigo-500", "var(--color-indigo-500)"),
    ("indigo-600", "var(--color-indigo-600)"),
    ("indigo-700", "var(--color-indigo-700)"),
    ("indigo-800", "var(--color-indigo-800)"),
    ("indigo-900", "var(--color-indigo-900)"),
    ("indigo-950", "var(--color-indigo-950)"),
    ("violet-50", "var(--color-violet-50)"),
    ("violet-100", "var(--color-violet-100)"),
    ("violet-200", "var(--color-violet-200)"),
    ("violet-300", "var(--color-violet-300)"),
    ("violet-400", "var(--color-violet-400)"),
    ("violet-500", "var(--color-violet-500)"),
    ("violet-600", "var(--color-violet-600)"),
    ("violet-700", "var(--color-violet-700)"),
    ("violet-800", "var(--color-violet-800)"),
    ("violet-900", "var(--color-violet-900)"),
    ("violet-950", "var(--color-violet-950)"),
    ("purple-50", "var(--color-purple-50)"),
    ("purple-100", "var(--color-purple-100)"),
    ("purple-200", "var(--color-purple-200)"),
    ("purple-300", "var(--color-purple-300)"),
    ("purple-400", "var(--color-purple-400)"),
    ("purple-500", "var(--color-purple-500)"),
    ("purple-600", "var(--color-purple-600)"),
    ("purple-700", "var(--color-purple-700)"),
    ("purple-800", "var(--color-purple-800)"),
    ("purple-900", "var(--color-purple-900)"),
    ("purple-950", "var(--color-purple-950)"),
    ("fuchsia-50", "var(--color-fuchsia-50)"),
    ("fuchsia-100", "var(--color-fuchsia-100)"),
    ("fuchsia-200", "var(--color-fuchsia-200)"),
    ("fuchsia-300", "var(--color-fuchsia-300)"),
    ("fuchsia-400", "var(--color-fuchsia-400)"),
    ("fuchsia-500", "var(--color-fuchsia-500)"),
    ("fuchsia-600", "var(--color-fuchsia-600)"),
    ("fuchsia-700", "var(--color-fuchsia-700)"),
    ("fuchsia-800", "var(--color-fuchsia-800)"),
    ("fuchsia-900", "var(--color-fuchsia-900)"),
    ("fuchsia-950", "var(--color-fuchsia-950)"),
    ("pink-50", "var(--color-pink-50)"),
    ("pink-100", "var(--color-pink-100)"),
    ("pink-200", "var(--color-pink-200)"),
    ("pink-300", "var(--color-pink-300)"),
    ("pink-400", "var(--color-pink-400)"),
    ("pink-500", "var(--color-pink-500)"),
    ("pink-600", "var(--color-pink-600)"),
    ("pink-700", "var(--color-pink-700)"),
    ("pink-800", "var(--color-pink-800)"),
    ("pink-900", "var(--color-pink-900)"),
    ("pink-950", "var(--color-pink-950)"),
    ("rose-50", "var(--color-rose-50)"),
    ("rose-100", "var(--color-rose-100)"),
    ("rose-200", "var(--color-rose-200)"),
    ("rose-300", "var(--color-rose-300)"),
    ("rose-400", "var(--color-rose-400)"),
    ("rose-500", "var(--color-rose-500)"),
    ("rose-600", "var(--color-rose-600)"),
    ("rose-700", "var(--color-rose-700)"),
    ("rose-800", "var(--color-rose-800)"),
    ("rose-900", "var(--color-rose-900)"),
    ("rose-950", "var(--color-rose-950)"),
    ("slate-50", "var(--color-slate-50)"),
    ("slate-100", "var(--color-slate-100)"),
    ("slate-200", "var(--color-slate-200)"),
    ("slate-300", "var(--color-slate-300)"),
    ("slate-400", "var(--color-slate-400)"),
    ("slate-500", "var(--color-slate-500)"),
    ("slate-600", "var(--color-slate-600)"),
    ("slate-700", "var(--color-slate-700)"),
    ("slate-800", "var(--color-slate-800)"),
    ("slate-900", "var(--color-slate-900)"),
    ("slate-950", "var(--color-slate-950)"),
    ("gray-50", "var(--color-gray-50)"),
    ("gray-100", "var(--color-gray-100)"),
    ("gray-200", "var(--color-gray-200)"),
    ("gray-300", "var(--color-gray-300)"),
    ("gray-400", "var(--color-gray-400)"),
    ("gray-500", "var(--color-gray-500)"),
    ("gray-600", "var(--color-gray-600)"),
    ("gray-700", "var(--color-gray-700)"),
    ("gray-800", "var(--color-gray-800)"),
    ("gray-900", "var(--color-gray-900)"),
    ("gray-950", "var(--color-gray-950)"),
    ("zinc-50", "var(--color-zinc-50)"),
    ("zinc-100", "var(--color-zinc-100)"),
    ("zinc-200", "var(--color-zinc-200)"),
    ("zinc-300", "var(--color-zinc-300)"),
    ("zinc-400", "var(--color-zinc-400)"),
    ("zinc-500", "var(--color-zinc-500)"),
    ("zinc-600", "var(--color-zinc-600)"),
    ("zinc-700", "var(--color-zinc-700)"),
    ("zinc-800", "var(--color-zinc-800)"),
    ("zinc-900", "var(--color-zinc-900)"),
    ("zinc-950", "var(--color-zinc-950)"),
    ("neutral-50", "var(--color-neutral-50)"),
    ("neutral-100", "var(--color-neutral-100)"),
    ("neutral-200", "var(--color-neutral-200)"),
    ("neutral-300", "var(--color-neutral-300)"),
    ("neutral-400", "var(--color-neutral-400)"),
    ("neutral-500", "var(--color-neutral-500)"),
    ("neutral-600", "var(--color-neutral-600)"),
    ("neutral-700", "var(--color-neutral-700)"),
    ("neutral-800", "var(--color-neutral-800)"),
    ("neutral-900", "var(--color-neutral-900)"),
    ("neutral-950", "var(--color-neutral-950)"),
    ("stone-50", "var(--color-stone-50)"),
    ("stone-100", "var(--color-stone-100)"),
    ("stone-200", "var(--color-stone-200)"),
    ("stone-300", "var(--color-stone-300)"),
    ("stone-400", "var(--color-stone-400)"),
    ("stone-500", "var(--color-stone-500)"),
    ("stone-600", "var(--color-stone-600)"),
    ("stone-700", "var(--color-stone-700)"),
    ("stone-800", "var(--color-stone-800)"),
    ("stone-900", "var(--color-stone-900)"),
    ("stone-950", "var(--color-stone-950)"),
    // The shadcn semantic tokens (theme-driven, corpus-facing).
    ("primary", "var(--color-primary)"),
    ("primary-foreground", "var(--color-primary-foreground)"),
    ("secondary", "var(--color-secondary)"),
    ("secondary-foreground", "var(--color-secondary-foreground)"),
    ("muted", "var(--color-muted)"),
    ("muted-foreground", "var(--color-muted-foreground)"),
    ("accent", "var(--color-accent)"),
    ("accent-foreground", "var(--color-accent-foreground)"),
    ("destructive", "var(--color-destructive)"),
    (
        "destructive-foreground",
        "var(--color-destructive-foreground)",
    ),
    ("border", "var(--color-border)"),
    ("input", "var(--color-input)"),
    ("ring", "var(--color-ring)"),
    ("background", "var(--color-background)"),
    ("foreground", "var(--color-foreground)"),
    ("card", "var(--color-card)"),
    ("card-foreground", "var(--color-card-foreground)"),
    ("popover", "var(--color-popover)"),
    ("popover-foreground", "var(--color-popover-foreground)"),
    ("sidebar", "var(--color-sidebar)"),
    ("sidebar-foreground", "var(--color-sidebar-foreground)"),
    ("sidebar-primary", "var(--color-sidebar-primary)"),
    ("sidebar-accent", "var(--color-sidebar-accent)"),
    ("sidebar-border", "var(--color-sidebar-border)"),
];

/// The colour `ValueKind` — one shared table behind every colour-keeping
/// root (`bg`, `text`-color, `border`-color, `outline`, `ring`, `shadow`,
/// gradient stops, `fill`, `stroke`).
pub const COLOR_KEYWORD_TABLE: ValueKind = ValueKind::Keywords(COLOR_KEYWORDS);

/// `bg-{image}` gradient shapes (`bg-linear-to-r`, `bg-radial`, ... plus
/// the deprecated `bg-gradient-to-*` alias and `bg-none`).
pub const BG_IMAGES: &[(&str, &str)] = &[
    ("none", "none"),
    ("linear", "var(--tw-gradient-linear)"),
    (
        "linear-to-t",
        "linear-gradient(to top, var(--tw-gradient-stops))",
    ),
    (
        "linear-to-tr",
        "linear-gradient(to top right, var(--tw-gradient-stops))",
    ),
    (
        "linear-to-r",
        "linear-gradient(to right, var(--tw-gradient-stops))",
    ),
    (
        "linear-to-br",
        "linear-gradient(to bottom right, var(--tw-gradient-stops))",
    ),
    (
        "linear-to-b",
        "linear-gradient(to bottom, var(--tw-gradient-stops))",
    ),
    (
        "linear-to-bl",
        "linear-gradient(to bottom left, var(--tw-gradient-stops))",
    ),
    (
        "linear-to-l",
        "linear-gradient(to left, var(--tw-gradient-stops))",
    ),
    (
        "linear-to-tl",
        "linear-gradient(to top left, var(--tw-gradient-stops))",
    ),
    (
        "gradient-to-t",
        "linear-gradient(to top, var(--tw-gradient-stops))",
    ),
    (
        "gradient-to-tr",
        "linear-gradient(to top right, var(--tw-gradient-stops))",
    ),
    (
        "gradient-to-r",
        "linear-gradient(to right, var(--tw-gradient-stops))",
    ),
    (
        "gradient-to-br",
        "linear-gradient(to bottom right, var(--tw-gradient-stops))",
    ),
    (
        "gradient-to-b",
        "linear-gradient(to bottom, var(--tw-gradient-stops))",
    ),
    (
        "gradient-to-bl",
        "linear-gradient(to bottom left, var(--tw-gradient-stops))",
    ),
    (
        "gradient-to-l",
        "linear-gradient(to left, var(--tw-gradient-stops))",
    ),
    (
        "gradient-to-tl",
        "linear-gradient(to top left, var(--tw-gradient-stops))",
    ),
    ("radial", "var(--tw-gradient-radial)"),
    ("conic", "var(--tw-gradient-conic)"),
];

/// `flex-{direction}` suffixes → CSS values.
pub const FLEX_DIRECTIONS: &[(&str, &str)] = &[
    ("col", "column"),
    ("col-reverse", "column-reverse"),
    ("row", "row"),
    ("row-reverse", "row-reverse"),
];

/// `flex-{wrap}` suffixes → CSS values.
pub const FLEX_WRAPS: &[(&str, &str)] = &[
    ("wrap", "wrap"),
    ("nowrap", "nowrap"),
    ("wrap-reverse", "wrap-reverse"),
];

/// The `flex-{1,auto,initial,none}` shorthand triplets
/// (`flex-grow`/`flex-shrink`/`flex-basis` composed values).
pub const FLEX_TRIPLETS: &[(&str, &str)] = &[
    ("1", "1 1 0%"),
    ("auto", "1 1 auto"),
    ("initial", "0 1 auto"),
    ("none", "none"),
];

/// Shared alignment keywords (`items-*`/`justify-items-*`).
pub const ITEM_ALIGNMENTS: &[(&str, &str)] = &[
    ("start", "flex-start"),
    ("end", "flex-end"),
    ("center", "center"),
    ("baseline", "baseline"),
    ("stretch", "stretch"),
    ("normal", "normal"),
];

/// `content-*` (the distribute family `items-*` lacks).
pub const CONTENT_ALIGNMENTS: &[(&str, &str)] = &[
    ("start", "flex-start"),
    ("end", "flex-end"),
    ("center", "center"),
    ("between", "space-between"),
    ("around", "space-around"),
    ("evenly", "space-evenly"),
    ("baseline", "baseline"),
    ("stretch", "stretch"),
    ("normal", "normal"),
];

/// `justify-*` (the space family).
pub const JUSTIFY_ALIGNMENTS: &[(&str, &str)] = &[
    ("start", "flex-start"),
    ("end", "flex-end"),
    ("center", "center"),
    ("between", "space-between"),
    ("around", "space-around"),
    ("evenly", "space-evenly"),
    ("normal", "normal"),
];

/// `self-*` (auto placement included).
pub const SELF_ALIGNMENTS: &[(&str, &str)] = &[
    ("auto", "auto"),
    ("start", "flex-start"),
    ("end", "flex-end"),
    ("center", "center"),
    ("stretch", "stretch"),
    ("baseline", "baseline"),
    ("normal", "normal"),
];

/// `place-*` keyword pairs.
pub const PLACE_ALIGNMENTS: &[(&str, &str)] = &[
    ("start", "start"),
    ("end", "end"),
    ("center", "center"),
    ("between", "space-between"),
    ("around", "space-around"),
    ("evenly", "space-evenly"),
    ("stretch", "stretch"),
    ("baseline", "baseline"),
];

/// `display` bare keywords (bare utilities keep their own roots;
/// `hidden`/`block`/... are registry rows of their own).
pub const DISPLAYS: &[(&str, &str)] = &[
    ("block", "block"),
    ("inline", "inline"),
    ("inline-block", "inline-block"),
    ("flex", "flex"),
    ("inline-flex", "inline-flex"),
    ("grid", "grid"),
    ("inline-grid", "inline-grid"),
    ("flow-root", "flow-root"),
    ("contents", "contents"),
    ("table", "table"),
    ("hidden", "none"),
];

/// `position` bare keywords.
pub const POSITIONS: &[(&str, &str)] = &[
    ("static", "static"),
    ("fixed", "fixed"),
    ("absolute", "absolute"),
    ("relative", "relative"),
    ("sticky", "sticky"),
];

/// `overflow-*` / bare `overflow` keywords.
pub const OVERFLOWS: &[(&str, &str)] = &[
    ("auto", "auto"),
    ("hidden", "hidden"),
    ("clip", "clip"),
    ("visible", "visible"),
    ("scroll", "scroll"),
];

/// `overscroll-*` keywords.
pub const OVERSCROLLS: &[(&str, &str)] =
    &[("auto", "auto"), ("contain", "contain"), ("none", "none")];

/// `aspect-{name}` keyword ratios (numeric ratios ride the fraction
/// shape: `aspect-3/2`).
pub const ASPECTS: &[(&str, &str)] = &[("auto", "auto"), ("square", "1 / 1"), ("video", "16 / 9")];

/// `text-align` bare keywords.
pub const TEXT_ALIGNS: &[(&str, &str)] = &[
    ("left", "left"),
    ("center", "center"),
    ("right", "right"),
    ("justify", "justify"),
    ("start", "start"),
    ("end", "end"),
];

/// `text-transform` bare keywords.
pub const TEXT_TRANSFORMS: &[(&str, &str)] = &[
    ("uppercase", "uppercase"),
    ("lowercase", "lowercase"),
    ("capitalize", "capitalize"),
    ("normal-case", "none"),
];

/// `text-decoration` bare keywords.
pub const TEXT_DECORATIONS: &[(&str, &str)] = &[
    ("underline", "underline"),
    ("overline", "overline"),
    ("line-through", "line-through"),
    ("no-underline", "none"),
];

/// `whitespace-*` keywords.
pub const WHITESPACE: &[(&str, &str)] = &[
    ("normal", "normal"),
    ("nowrap", "nowrap"),
    ("pre", "pre"),
    ("pre-line", "pre-line"),
    ("pre-wrap", "pre-wrap"),
    ("break-spaces", "break-spaces"),
];

/// `break-*` (word-break) keywords.
pub const WORD_BREAKS: &[(&str, &str)] = &[
    ("normal", "normal"),
    ("words", "break-word"),
    ("all", "break-all"),
    ("keep", "keep-all"),
];

/// `vertical-align` keywords.
pub const VERTICAL_ALIGNS: &[(&str, &str)] = &[
    ("baseline", "baseline"),
    ("top", "top"),
    ("middle", "middle"),
    ("bottom", "bottom"),
    ("text-top", "text-top"),
    ("text-bottom", "text-bottom"),
    ("sub", "sub"),
    ("super", "super"),
];

/// `object-fit` keywords.
pub const OBJECT_FITS: &[(&str, &str)] = &[
    ("contain", "contain"),
    ("cover", "cover"),
    ("fill", "fill"),
    ("none", "none"),
    ("scale-down", "scale-down"),
];

/// `object-position` keywords.
pub const OBJECT_POSITIONS: &[(&str, &str)] = &[
    ("center", "center"),
    ("top", "top"),
    ("right-top", "right top"),
    ("right", "right"),
    ("right-bottom", "right bottom"),
    ("bottom", "bottom"),
    ("left-bottom", "left bottom"),
    ("left", "left"),
    ("left-top", "left top"),
];

/// `cursor` keywords (the common surface; arbitrary cursors ride the
/// arbitrary passthrough).
pub const CURSORS: &[(&str, &str)] = &[
    ("auto", "auto"),
    ("default", "default"),
    ("pointer", "pointer"),
    ("wait", "wait"),
    ("text", "text"),
    ("move", "move"),
    ("help", "help"),
    ("not-allowed", "not-allowed"),
    ("none", "none"),
    ("progress", "progress"),
    ("grab", "grab"),
    ("grabbing", "grabbing"),
    ("crosshair", "crosshair"),
    ("zoom-in", "zoom-in"),
    ("zoom-out", "zoom-out"),
];

/// `resize` keywords (bare `resize` is the row's `Bare("both")` arm).
pub const RESIZES: &[(&str, &str)] = &[("none", "none"), ("y", "vertical"), ("x", "horizontal")];

/// `grid-auto-rows`/`auto-cols` keywords.
pub const AUTO_TRACKS: &[(&str, &str)] = &[
    ("auto", "auto"),
    ("min", "min-content"),
    ("max", "max-content"),
    ("fr", "minmax(0, 1fr)"),
];

/// The shadow preset scale (`shadow-{name}` → the composed `--tw-shadow`
/// reference; `inner`/`none` are structural).
pub const SHADOW_PRESETS: &[(&str, &str)] = &[
    ("2xs", "var(--tw-shadow-2xs)"),
    ("xs", "var(--tw-shadow-xs)"),
    ("sm", "var(--tw-shadow-sm)"),
    ("md", "var(--tw-shadow-md)"),
    ("lg", "var(--tw-shadow-lg)"),
    ("xl", "var(--tw-shadow-xl)"),
    ("2xl", "var(--tw-shadow-2xl)"),
    ("inner", "inset 0 0 0 0 currentColor"),
    ("none", "0 0 #0000"),
];

/// `grid-cols-*`/`grid-rows-*` keyword templates (numeric counts ride the
/// number shape, arbitrary track lists the passthrough).
pub const GRID_TEMPLATES: &[(&str, &str)] = &[("none", "none"), ("subgrid", "subgrid")];

/// `columns` keyword values (numeric counts ride the number shape).
pub const COLUMN_KEYWORDS: &[(&str, &str)] = &[
    ("auto", "auto"),
    ("3xs", "16rem"),
    ("2xs", "18rem"),
    ("xs", "20rem"),
    ("sm", "24rem"),
    ("md", "28rem"),
    ("lg", "32rem"),
    ("xl", "36rem"),
    ("2xl", "42rem"),
    ("3xl", "48rem"),
    ("4xl", "56rem"),
    ("5xl", "64rem"),
    ("6xl", "72rem"),
    ("7xl", "80rem"),
];

/// `container-type` values for the `@container` utility.
pub const CONTAINER_TYPES: &[(&str, &str)] = &[
    ("normal", "normal"),
    ("size", "size"),
    ("inline-size", "inline-size"),
];
