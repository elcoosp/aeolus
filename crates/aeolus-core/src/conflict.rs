//! Conflict groups and `cn()` — last class wins within a conflict group.
//!
//! Algorithm and tables are lifted from `tw_merge` 0.1.22 (MIT,
//! rust-ui/ui @ 4f5a516e, audited 2026-10-30), which is itself the v4 fork of
//! `tailwind-fuse`. The merge is: iterate classes **right-to-left**, keep the
//! first class seen per `(variants, important, conflict-group)` key, and drop
//! earlier classes whose group was knocked out by a kept class's parent group.
//!
//! Runtime here operates on parsed tokens over `&'static str`-backed
//! tables; the compile-time trie (`tw!` pre-interned IDs) is the planned
//! optimization and produces identical results.

use crate::ast::ClassToken;
use crate::parser::parse_class_string;
use std::collections::HashSet;

/// Group classification for a parsed token. `None` = keep verbatim.
///
/// Scope: the Shadcn v1 utility set. Disambiguation follows tw_merge:
/// `text-*` splits into `font-size` (t-shirt names) vs `text-color`;
/// `font-*` splits into `font-weight` vs `font-family`; `border-*` splits
/// into `border-w*` vs `border-color*`.
pub fn collision_group(t: &ClassToken<'_>) -> Option<&'static str> {
    let els: &[&str] = &t.elements;
    let size_name = |e: &str| {
        matches!(
            e,
            "xs" | "sm"
                | "base"
                | "lg"
                | "xl"
                | "2xl"
                | "3xl"
                | "4xl"
                | "5xl"
                | "6xl"
                | "7xl"
                | "8xl"
                | "9xl"
        )
    };
    Some(match els {
        // v0.6: v4 keeps `background-color` and `background-image` as
        // separate declarations that COEXIST (`bg-red-500 bg-linear-to-r`
        // layers the gradient over the solid) — tw_merge models them as
        // the separate `bg-color` / `bg-image` groups. The image group is
        // the gradient shapes (`bg-linear-*`, `bg-radial`, `bg-conic*`,
        // `bg-none`) plus the deprecated `bg-gradient-to-*` alias.
        ["bg"] => "bg-color",
        ["bg", "linear" | "radial" | "conic" | "gradient", ..] | ["bg", "none"] => "bg-image",
        ["bg", ..] => "bg-color",
        // Gradient stop edges: color and position are different custom
        // properties in v4 (`--tw-gradient-from` vs
        // `--tw-gradient-from-position`) and COMPOSE, so they are separate
        // groups (`from-red-500 from-20%` keeps both; two colors conflict).
        ["from", v, ..] if v.ends_with('%') => "from-pos",
        ["from", ..] => "from-color",
        ["via", v, ..] if v.ends_with('%') => "via-pos",
        ["via", ..] => "via-color",
        ["to", v, ..] if v.ends_with('%') => "to-pos",
        ["to", ..] => "to-color",
        ["p"] | ["p", ..] => "padding",
        ["px", ..] => "padding-x",
        ["py", ..] => "padding-y",
        ["pt", ..] => "padding-top",
        ["pr" | "pe", ..] => "padding-right",
        ["pb", ..] => "padding-bottom",
        ["pl" | "ps", ..] => "padding-left",
        ["m"] | ["m", ..] => "margin",
        ["mx", ..] => "margin-x",
        ["my", ..] => "margin-y",
        ["mt", ..] => "margin-top",
        ["mr" | "me", ..] => "margin-right",
        ["mb", ..] => "margin-bottom",
        ["ml" | "ms", ..] => "margin-left",
        ["size", ..] => "size",
        ["w", ..] => "width",
        ["min-w", ..] | ["min", "w", ..] => "min-width",
        ["max-w", ..] | ["max", "w", ..] => "max-width",
        ["h", ..] => "height",
        ["min-h", ..] | ["min", "h", ..] => "min-height",
        ["max-h", ..] | ["max", "h", ..] => "max-height",
        ["gap", ..] => "gap",
        ["rounded"] | ["rounded", "DEFAULT"] => "rounded",
        ["rounded", e, ..] => match *e {
            "t" => "rounded-t",
            "b" => "rounded-b",
            "l" | "s" => "rounded-l",
            "r" | "e" => "rounded-r",
            "tl" | "ss" => "rounded-tl",
            "tr" | "se" => "rounded-tr",
            "br" | "ee" => "rounded-br",
            "bl" | "es" => "rounded-bl",
            _ => "rounded",
        },
        ["border"] => "border-w",
        ["border", e, ..] if is_border_width(els) => match *e {
            "t" => "border-w-t",
            "r" | "e" => "border-w-r",
            "b" => "border-w-b",
            "l" | "s" => "border-w-l",
            "x" => "border-w-x",
            "y" => "border-w-y",
            _ => "border-w",
        },
        ["border", ..] => "border-color",
        // Shadow splits like border: preset widths vs shadow color
        // (tw_merge 'shadow' vs 'shadow-color' — `shadow-md` and
        // `shadow-red-500` do NOT conflict).
        ["shadow"] => "shadow",
        ["shadow", e, ..] if SHADOW_PRESETS.contains(e) => "shadow",
        ["shadow", ..] => "shadow-color",
        // Ring splits like border: width (numbers / px-rem arbitrary) vs
        // color, offset width vs offset color — four independent groups
        // (tw_merge 'ring-w'/'ring-color'/'ring-offset-w'/'ring-offset-color').
        ["ring", "offset", only] | ["ring-offset", only]
            if is_width_value(only, t.arbitrary.as_ref()) =>
        {
            "ring-offset-w"
        }
        ["ring", "offset", ..] | ["ring-offset", ..] => "ring-offset-color",
        ["ring"] => "ring-w",
        ["ring", only] if is_width_value(only, t.arbitrary.as_ref()) => "ring-w",
        ["ring", ..] if is_width_value("", t.arbitrary.as_ref()) => "ring-w",
        ["ring", ..] => "ring-color",
        // Outline splits like ring: width / style / offset / color are four
        // independent groups (tw_merge 'outline-w'/'outline-style'/
        // 'outline-offset'/'outline-color' — `outline-2` and `outline-red-500`
        // do NOT conflict).
        ["outline", "offset", ..] | ["outline-offset", ..] => "outline-offset",
        ["outline", "solid" | "dashed" | "dotted" | "double"] => "outline-style",
        ["outline"] | ["outline", "none" | "hidden"] => "outline-w",
        ["outline", only] if is_width_value(only, t.arbitrary.as_ref()) => "outline-w",
        ["outline", ..] if is_width_value("", t.arbitrary.as_ref()) => "outline-w",
        ["outline", ..] => "outline-color",
        ["opacity", ..] => "opacity",
        ["text", only] if size_name(only) => "font-size",
        ["text", ..] if els.len() == 2 && els[1] == "2xl" => "font-size",
        [
            "text",
            "left" | "center" | "right" | "justify" | "start" | "end",
            ..,
        ] => "text-align",
        ["text", ..] => "text-color",
        ["font", e, ..] => match *e {
            "sans" | "serif" | "mono" => "font-family",
            _ => "font-weight",
        },
        ["leading", ..] => "line-height",
        ["tracking", ..] => "letter-spacing",
        ["underline-offset", ..] | ["underline", "offset", ..] => "underline-offset",
        ["whitespace", ..] => "whitespace",
        ["flex"] | ["flex", "inline"] => "display",
        ["flex", "wrap" | "nowrap" | "wrap-reverse", ..] => "flex-wrap",
        ["flex", e, ..] if matches!(*e, "col" | "row" | "1" | "auto" | "initial" | "none") => {
            "flex-direction"
        }
        ["basis", ..] => "flex-basis",
        ["grow", ..] => "flex-grow",
        ["shrink", ..] => "flex-shrink",
        ["order", ..] => "order",
        ["items", ..] => "align-items",
        ["content", ..] => "align-content",
        ["justify", "items", ..] => "justify-items",
        ["justify", "self", ..] => "justify-self",
        ["justify", e, ..] if !matches!(*e, "items" | "self") => "justify-content",
        ["self", ..] => "align-self",
        ["place", "content", ..] => "place-content",
        ["place", "items", ..] => "place-items",
        ["place", "self", ..] => "place-self",
        ["place", ..] => "place-content",
        ["cursor", ..] => "cursor",
        ["pointer-events", ..] | ["pointer", "events", ..] => "pointer-events",
        ["select", ..] => "user-select",
        ["hidden" | "block" | "inline-flex" | "inline" | "grid"] => "display",
        ["display"] => "display",
        // Positioning families (tw_merge: position/inset/top/right/bottom/left/
        // z-index/overflow/overscroll-behavior).
        ["static" | "fixed" | "absolute" | "relative" | "sticky"] => "position",
        ["inset", "x", ..] => "inset-x",
        ["inset", "y", ..] => "inset-y",
        ["inset", ..] => "inset",
        // Logical inset edges (`start-*`/`end-*`): the parent `inset` group
        // already knocks them out; they are independent of top/right/...
        ["start", ..] => "start",
        ["end", ..] => "end",
        ["top", ..] => "top",
        ["right", ..] => "right",
        ["bottom", ..] => "bottom",
        ["left", ..] => "left",
        ["z", ..] => "z-index",
        ["overflow", "x", ..] => "overflow-x",
        ["overflow", "y", ..] => "overflow-y",
        ["overflow", ..] => "overflow",
        ["overscroll", "x", ..] => "overscroll-x",
        ["overscroll", "y", ..] => "overscroll-y",
        ["overscroll", ..] => "overscroll-behavior",
        ["aspect", ..] => "aspect",
        ["columns", ..] => "columns",
        // Grid line families (tw_merge: six independent groups —
        // `col-span-2` and `col-start-1` do not conflict).
        ["col", "span", ..] => "col-span",
        ["col", "start", ..] => "col-start",
        ["col", "end", ..] => "col-end",
        ["row", "span", ..] => "row-span",
        ["row", "start", ..] => "row-start",
        ["row", "end", ..] => "row-end",
        // v0.9 §4: the grid TEMPLATE families are conflict groups — the
        // numeric and arbitrary forms set the same CSS property
        // (`grid-template-columns`), so the later one evicts the earlier
        // across the numeric/arbitrary boundary. The parser strips the
        // arbitrary value, so `grid-cols-[0_1fr]` and `grid-cols-3` share
        // the `["grid", "cols"]` elements.
        ["grid", "cols", ..] => "grid-template-columns",
        ["grid", "rows", ..] => "grid-template-rows",
        ["auto", "rows", ..] => "grid-auto-rows",
        ["auto", "cols", ..] => "grid-auto-columns",
        // v0.9 §3: `@container[/name]` — every form establishes the same
        // `container-type` declaration; the later one wins.
        ["@container", ..] => "container-type",
        ["visible" | "invisible"] => "visibility",
        ["isolate" | "isolation-auto"] => "isolation",
        ["resize", ..] => "resize",
        [
            "object",
            "contain" | "cover" | "fill" | "none" | "scale-down",
        ] => "object-fit",
        ["object", ..] => "object-position",
        // Typography (tw_merge: font-style / text-transform / text-decoration /
        // vertical-align).
        ["italic" | "not-italic"] => "font-style",
        ["uppercase" | "lowercase" | "capitalize" | "normal-case"] => "text-transform",
        ["underline" | "overline" | "line-through" | "no-underline"] => "text-decoration",
        ["align", ..] => "vertical-align",
        _ => return None,
    })
}

fn is_border_width(els: &[&str]) -> bool {
    match els {
        ["border"] => true,
        ["border", e] => {
            matches!(*e, "t" | "r" | "b" | "l" | "x" | "y" | "s" | "e") || e.parse::<f32>().is_ok()
        }
        ["border", e, ..] => matches!(*e, "x" | "y"),
        _ => false,
    }
}

/// The v4 shadow preset names (`shadow-{2xs,xs,sm,md,lg,xl,2xl,inner,none}`)
/// — any other value is a shadow COLOR.
const SHADOW_PRESETS: &[&str] = &["2xs", "xs", "sm", "md", "lg", "xl", "2xl", "inner", "none"];

/// Is this ring element an explicit width? `2` (ring-2), `3px` (ring-[3px]),
/// bare `ring` (default width). Colors never parse as numbers and arbitrary
/// colors carry `#`/`oklch(`/names.
fn is_width_value(e: &str, arbitrary: Option<&crate::ast::Arbitrary<'_>>) -> bool {
    let v = arbitrary.map_or(e, |a| a.value);
    v.parse::<f32>().is_ok() || v.ends_with("px") || v.ends_with("rem") || v.ends_with("em")
}

/// Parent group → groups it knocks out (tw_merge `get_collisions`, complete).
fn collisions(group: &str) -> &'static [&'static str] {
    match group {
        "overflow" => &["overflow-x", "overflow-y"],
        "inset" => &[
            "inset-x", "inset-y", "start", "end", "top", "right", "bottom", "left",
        ],
        "inset-x" => &["right", "left"],
        "inset-y" => &["top", "bottom"],
        "flex" => &["flex-basis", "flex-grow", "flex-shrink"],
        "gap" => &["gap-x", "gap-y"],
        "overscroll-behavior" => &["overscroll-x", "overscroll-y"],
        "padding" => &[
            "padding-x",
            "padding-y",
            "padding-start",
            "padding-end",
            "padding-top",
            "padding-right",
            "padding-bottom",
            "padding-left",
        ],
        "padding-x" => &["padding-right", "padding-left"],
        "padding-y" => &["padding-top", "padding-bottom"],
        "margin" => &[
            "margin-x",
            "margin-y",
            "margin-start",
            "margin-end",
            "margin-top",
            "margin-right",
            "margin-bottom",
            "margin-left",
        ],
        "margin-x" => &["margin-right", "margin-left"],
        "margin-y" => &["margin-top", "margin-bottom"],
        "size" => &["width", "height"],
        "font-size" => &["line-height"],
        "rounded" => &[
            "rounded-s",
            "rounded-e",
            "rounded-t",
            "rounded-r",
            "rounded-b",
            "rounded-l",
            "rounded-ss",
            "rounded-se",
            "rounded-ee",
            "rounded-es",
            "rounded-tl",
            "rounded-tr",
            "rounded-br",
            "rounded-bl",
        ],
        "rounded-s" => &["rounded-ss", "rounded-es"],
        "rounded-e" => &["rounded-se", "rounded-ee"],
        "rounded-t" => &["rounded-tl", "rounded-tr"],
        "rounded-r" => &["rounded-tr", "rounded-br"],
        "rounded-b" => &["rounded-br", "rounded-bl"],
        "rounded-l" => &["rounded-tl", "rounded-bl"],
        "border-w" => &[
            "border-w-s",
            "border-w-e",
            "border-w-t",
            "border-w-r",
            "border-w-b",
            "border-w-l",
        ],
        "border-w-x" => &["border-w-r", "border-w-l"],
        "border-w-y" => &["border-w-t", "border-w-b"],
        "border-color" => &[
            "border-color-t",
            "border-color-r",
            "border-color-b",
            "border-color-l",
        ],
        "border-color-x" => &["border-color-r", "border-color-l"],
        "border-color-y" => &["border-color-t", "border-color-b"],
        _ => &[],
    }
}

/// The merge key: same variants + importance + group ⇒ later wins.
///
/// v0.2 fix (plan §1.3): the variant key is the EXACT canonical variant
/// prefix (`token.variant_prefix()`), so `hover:` and `group-hover:` no
/// longer share a key, and `md:` no longer collides with `@md:`.
fn key_of(t: &ClassToken<'_>) -> (String, bool, &'static str) {
    (
        t.variant_prefix(),
        t.important,
        collision_group(t).unwrap_or("unclassified"),
    )
}

/// Merge conflicting Tailwind classes; the last conflicting class wins.
///
/// Classes Aeolus cannot classify (arbitrary properties, plugin utilities)
/// are kept verbatim, matching `tw_merge` behavior.
pub fn cn(classes: &[&str]) -> String {
    if classes.is_empty() {
        return String::new();
    }
    let joined = classes.join(" ");
    // Fast path: 0 or 1 words. cn's output is always a normalized
    // whitespace-separated word sequence, so zero words normalize to ""
    // and a single word is returned WITHOUT any surrounding whitespace
    // (fuzz findings: cn("\n") echoed "\n", cn("\n\u{1c}") echoed it
    // with a stray "\n" — both broke idempotence).
    let mut words = joined.split_ascii_whitespace();
    match (words.next(), words.next()) {
        (None, _) => return String::new(),
        (Some(w), None) => return w.to_string(),
        (Some(_), Some(_)) => {}
    }
    let parsed = {
        let (parsed, errs) = parse_class_string(&joined);
        if !errs.is_empty() {
            // Token/raw pairing would desync if some classes were skipped;
            // keep every WORD verbatim (cn is best-effort) — but still
            // normalize the output to a whitespace-separated word sequence.
            // Echoing `joined` itself leaked the caller's internal
            // whitespace and broke idempotence (fuzz finding).
            return joined
                .split_ascii_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
        }
        parsed
    };

    // Pair tokens with their original raw text for output.
    let raws: Vec<&str> = joined.split_ascii_whitespace().collect();
    debug_assert_eq!(raws.len(), parsed.len());

    let mut taken: HashSet<(String, bool, &'static str)> = HashSet::new();
    let mut kept: Vec<Option<&str>> = vec![None; parsed.len()];

    for i in (0..parsed.len()).rev() {
        let token = &parsed[i];
        let k = key_of(token);
        let group = collision_group(token);
        if let Some(group) = group {
            if taken.contains(&k) {
                continue;
            }
            // This class wins; knock out all colliding earlier classes.
            taken.insert((k.0.clone(), k.1, group));
            for g in collisions(group) {
                taken.insert((k.0.clone(), k.1, g));
            }
        }
        kept[i] = Some(token.raw);
    }

    let out: Vec<&str> = kept.into_iter().flatten().collect();
    out.join(" ")
}

/// `cn` accepting an iterator of optional classes (the shadcn `cn(...)`
/// ergonomic form: `None` entries are skipped).
pub fn cn_opt<'a>(classes: impl IntoIterator<Item = Option<&'a str>>) -> String {
    let v: Vec<&str> = classes.into_iter().flatten().collect();
    cn(&v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn last_padding_wins() {
        assert_eq!(cn(&["px-2", "px-4"]), "px-4");
        assert_eq!(cn(&["p-4", "px-2"]), "p-4 px-2");
        // px is knocked out by a later p, but keeps its own x-value role:
        assert_eq!(cn(&["px-2", "p-4"]), "p-4");
    }

    #[test]
    fn sub_edges_survive_parent() {
        // pt is a different group than px — both survive.
        assert_eq!(cn(&["px-2", "pt-2"]), "px-2 pt-2");
        // later p knocks out earlier px AND pt
        assert_eq!(cn(&["px-2 pt-2", "p-1"]), "p-1");
    }

    #[test]
    fn mission_brief_case() {
        assert_eq!(cn(&["px-2", "px-4"]), "px-4");
    }

    #[test]
    fn rounded_hierarchy() {
        assert_eq!(
            cn(&["rounded-md", "rounded-t-lg"]),
            "rounded-md rounded-t-lg"
        );
        assert_eq!(cn(&["rounded-t-lg", "rounded-md"]), "rounded-md");
    }

    #[test]
    fn color_vs_size_text() {
        assert_eq!(cn(&["text-sm", "text-primary"]), "text-sm text-primary");
        assert_eq!(cn(&["text-red-500", "text-primary"]), "text-primary");
        assert_eq!(cn(&["text-sm", "text-base"]), "text-base");
    }

    #[test]
    fn variants_are_separate_stacks() {
        assert_eq!(
            cn(&["bg-primary", "hover:bg-primary/90"]),
            "bg-primary hover:bg-primary/90"
        );
        assert_eq!(
            cn(&["hover:bg-primary/90", "hover:bg-accent"]),
            "hover:bg-accent"
        );
    }

    #[test]
    fn unknown_kept_verbatim() {
        assert_eq!(
            cn(&["custom-plugin-thing", "px-2", "px-4"]),
            "custom-plugin-thing px-4"
        );
    }

    #[test]
    fn display_conflicts() {
        assert_eq!(cn(&["block", "hidden"]), "hidden");
        assert_eq!(
            cn(&["flex", "items-center", "hidden"]),
            "items-center hidden"
        );
    }

    /// Fuzz finding: pure-whitespace class strings must normalize to ""
    /// (cn's output is a word sequence; a zero-word input has no classes).
    #[test]
    fn whitespace_only_classes_normalize_to_empty() {
        assert_eq!(cn(&["\n"]), "");
        assert_eq!(cn(&["  ", "\t"]), "");
        // A single "word" containing embedded ascii-whitespace normalizes
        // to the word itself (fuzz finding: cn("\n\u{1c}") kept the "\n").
        assert_eq!(cn(&["\n\u{1c}"]), "\u{1c}");
        // Unparseable words are kept verbatim, but the OUTPUT is still a
        // normalized word sequence (fuzz finding: cn("\nJ\n[") echoed "\n").
        assert_eq!(cn(&["\nJ\n["]), "J [");
        // A single real word is still returned verbatim (fast path).
        assert_eq!(cn(&["p-4"]), "p-4");
    }

    #[test]
    fn opt_form() {
        assert_eq!(cn_opt([Some("px-2"), None, Some("px-4")]), "px-4");
    }

    /// tw_merge group table (precedents.md §3): the justify-*/align-*/place-*
    /// families are six independent groups, not a single catch-all. v0.1.0
    /// only classified items-* and justify-content.
    #[test]
    fn grid_alignment_families_merge_independently() {
        // THE bug: justify-items was unclassified (kept both).
        assert_eq!(
            cn(&["justify-items-start", "justify-items-center"]),
            "justify-items-center"
        );
        assert_eq!(
            cn(&["justify-self-auto", "justify-self-end"]),
            "justify-self-end"
        );
        assert_eq!(cn(&["content-start", "content-center"]), "content-center");
        assert_eq!(
            cn(&["place-content-start", "place-content-center"]),
            "place-content-center"
        );
        assert_eq!(
            cn(&["place-items-start", "place-items-center"]),
            "place-items-center"
        );
        assert_eq!(
            cn(&["place-self-start", "place-self-center"]),
            "place-self-center"
        );
        // ...and they must NOT collide with each other:
        assert_eq!(
            cn(&["justify-items-center", "justify-center"]),
            "justify-items-center justify-center"
        );
        // flex-wrap is its own group (was lumped with flex-direction).
        assert_eq!(cn(&["flex-wrap", "flex-nowrap"]), "flex-nowrap");
        assert_eq!(cn(&["flex-col", "flex-wrap"]), "flex-col flex-wrap");
        // overflow + inset families (collisions() knew them; classification didn't).
        assert_eq!(
            cn(&["overflow-hidden", "overflow-x-auto"]),
            "overflow-hidden overflow-x-auto"
        );
        assert_eq!(
            cn(&["overflow-x-hidden", "overflow-x-auto"]),
            "overflow-x-auto"
        );
        assert_eq!(cn(&["inset-x-2", "left-4"]), "inset-x-2 left-4");
        assert_eq!(cn(&["left-4", "inset-x-2"]), "inset-x-2");
        // text-align (was swallowed by text-color).
        assert_eq!(cn(&["text-left", "text-center"]), "text-center");
        assert_eq!(
            cn(&["text-center", "text-primary"]),
            "text-center text-primary"
        );
    }

    /// The v4 default shadow scale keeps a preset and its color independent
    /// (tw_merge 'shadow' vs 'shadow-color').
    #[test]
    fn shadow_preset_and_color_are_independent() {
        assert_eq!(
            cn(&["shadow-md", "shadow-destructive"]),
            "shadow-md shadow-destructive"
        );
        // Presets still merge among themselves, colors among themselves.
        assert_eq!(cn(&["shadow-sm", "shadow-lg"]), "shadow-lg");
        assert_eq!(cn(&["shadow-red-500", "shadow-primary"]), "shadow-primary");
        // `shadow-none` is a preset, not a color.
        assert_eq!(cn(&["shadow-md", "shadow-none"]), "shadow-none");
    }

    /// Outline width / style / offset / color are four independent groups
    /// (tw_merge); `outline-2` + `outline-primary` must compose.
    #[test]
    fn outline_families_merge_independently() {
        assert_eq!(
            cn(&["outline-2", "outline-primary"]),
            "outline-2 outline-primary"
        );
        assert_eq!(cn(&["outline-2", "outline-4"]), "outline-4");
        assert_eq!(cn(&["outline-dashed", "outline-solid"]), "outline-solid");
        assert_eq!(
            cn(&["outline-offset-2", "outline-offset-4"]),
            "outline-offset-4"
        );
        assert_eq!(
            cn(&["outline-2", "outline-primary", "outline-offset-2"]),
            "outline-2 outline-primary outline-offset-2"
        );
        // `outline-none` is a width-class value, not a color.
        assert_eq!(cn(&["outline-2", "outline-none"]), "outline-none");
    }

    /// Logical inset edges (`start-*`/`end-*`) are their own groups; the
    /// parent `inset` group knocks them out (tw_merge 'start'/'end').
    #[test]
    fn logical_inset_edges_merge() {
        assert_eq!(cn(&["start-0", "start-4"]), "start-4");
        assert_eq!(cn(&["end-0", "end-4"]), "end-4");
        assert_eq!(cn(&["start-2", "end-2"]), "start-2 end-2");
        // A later inset shorthand knocks out an earlier start edge; a later
        // start edge survives an earlier inset (last-wins per CSS cascade).
        assert_eq!(cn(&["start-4", "inset-0"]), "inset-0");
        assert_eq!(cn(&["inset-0", "start-4"]), "inset-0 start-4");
    }

    /// Grid line families are six independent groups (tw_merge): spans merge
    /// among themselves, starts among themselves, never across.
    #[test]
    fn grid_line_families_merge_independently() {
        assert_eq!(cn(&["col-span-2", "col-span-3"]), "col-span-3");
        assert_eq!(cn(&["col-start-1", "col-start-2"]), "col-start-2");
        assert_eq!(cn(&["col-span-2", "col-start-1"]), "col-span-2 col-start-1");
        assert_eq!(cn(&["row-span-2", "row-span-3"]), "row-span-3");
        assert_eq!(cn(&["row-span-2", "row-start-1"]), "row-span-2 row-start-1");
        // col and row families never collide.
        assert_eq!(cn(&["col-span-2", "row-span-2"]), "col-span-2 row-span-2");
    }

    /// v0.9 §4: the grid TEMPLATE families are conflict groups — the later
    /// template (numeric or arbitrary) evicts the earlier one, across the
    /// numeric/arbitrary boundary (both set the same CSS property).
    #[test]
    fn grid_template_families_conflict() {
        assert_eq!(cn(&["grid-cols-3", "grid-cols-2"]), "grid-cols-2");
        assert_eq!(
            cn(&["grid-cols-3", "grid-cols-[0_1fr]"]),
            "grid-cols-[0_1fr]"
        );
        assert_eq!(cn(&["grid-cols-[0_1fr]", "grid-cols-3"]), "grid-cols-3");
        assert_eq!(
            cn(&["grid-rows-2", "grid-rows-[auto_auto]"]),
            "grid-rows-[auto_auto]"
        );
        assert_eq!(cn(&["auto-rows-min", "auto-rows-fr"]), "auto-rows-fr");
        assert_eq!(cn(&["auto-cols-min", "auto-cols-max"]), "auto-cols-max");
        // Rows and cols never collide; templates and placement never collide.
        assert_eq!(
            cn(&["grid-cols-3", "grid-rows-2"]),
            "grid-cols-3 grid-rows-2"
        );
        assert_eq!(cn(&["grid-cols-3", "col-span-2"]), "grid-cols-3 col-span-2");
        // v0.9 §3: named container declarations evict each other.
        assert_eq!(cn(&["@container/a", "@container/b"]), "@container/b");
    }

    #[test]
    fn ring_offset_is_its_own_group() {
        // `ring-offset-*` must not be shadowed by the `ring-*` catch-all:
        // offset width and ring color collide with neither.
        assert_eq!(
            cn(&["ring-ring/50", "ring-offset-2"]),
            "ring-ring/50 ring-offset-2"
        );
        assert_eq!(cn(&["ring-offset-2", "ring-offset-4"]), "ring-offset-4");
        assert_eq!(cn(&["ring-2", "ring-4"]), "ring-4");
        // ring width vs ring color are different groups (tailwind-merge).
        assert_eq!(cn(&["ring-2", "ring-primary"]), "ring-2 ring-primary");
        // Shadcn Button's focus ring: width arbitrary + color survive cn().
        assert_eq!(
            cn(&["focus-visible:ring-[3px]", "focus-visible:ring-ring/50"]),
            "focus-visible:ring-[3px] focus-visible:ring-ring/50"
        );
        // Bare `ring` is width; an arbitrary color is color.
        assert_eq!(cn(&["ring", "ring-ring"]), "ring ring-ring");
    }
}
