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

use crate::ast::{ClassToken, NamedVariant};
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
    let size_name = |e: &str| matches!(e, "xs" | "sm" | "base" | "lg" | "xl" | "2xl" | "3xl" | "4xl" | "5xl" | "6xl" | "7xl" | "8xl" | "9xl");
    Some(match els {
        ["bg", ..] => "bg",
        ["p"] | ["p", ..] => "padding",
        ["px", ..] => "padding-x",
        ["py", ..] => "padding-y",
        ["pt", ..] => "padding-top",
        ["pr", ..] | ["pe", ..] => "padding-right",
        ["pb", ..] => "padding-bottom",
        ["pl", ..] | ["ps", ..] => "padding-left",
        ["m"] | ["m", ..] => "margin",
        ["mx", ..] => "margin-x",
        ["my", ..] => "margin-y",
        ["mt", ..] => "margin-top",
        ["mr", ..] | ["me", ..] => "margin-right",
        ["mb", ..] => "margin-bottom",
        ["ml", ..] | ["ms", ..] => "margin-left",
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
        ["shadow", ..] => "shadow",
        // Ring splits like border: width (numbers / px-rem arbitrary) vs
        // color, offset width vs offset color — four independent groups
        // (tw_merge 'ring-w'/'ring-color'/'ring-offset-w'/'ring-offset-color').
        ["ring", "offset", only] | ["ring-offset", only] if is_width_value(only, t.arbitrary) => "ring-offset-w",
        ["ring", "offset", ..] | ["ring-offset", ..] => "ring-offset-color",
        ["ring"] => "ring-w",
        ["ring", only] if is_width_value(only, t.arbitrary) => "ring-w",
        ["ring", ..] if is_width_value("", t.arbitrary) => "ring-w",
        ["ring", ..] => "ring-color",
        ["outline", ..] => "outline",
        ["opacity", ..] => "opacity",
        ["text", only] if size_name(only) => "font-size",
        ["text", ..] if els.len() == 2 && els[1] == "2xl" => "font-size",
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
        ["flex", e, ..] if matches!(*e, "col" | "row" | "wrap" | "1" | "auto" | "initial" | "none") => "flex-direction",
        ["items", ..] => "align-items",
        ["justify", e, ..] if !matches!(*e, "items" | "self" | "center") => "justify-content",
        ["justify", "center"] => "justify-content",
        ["self", ..] => "align-self",
        ["grow", ..] => "flex-grow",
        ["shrink", ..] => "flex-shrink",
        ["cursor", ..] => "cursor",
        ["pointer-events", ..] | ["pointer", "events", ..] => "pointer-events",
        ["select", ..] => "user-select",
        ["hidden"] | ["block"] | ["inline-flex"] | ["inline"] | ["grid"] => "display",
        ["display"] => "display",
        _ => return None,
    })
}

fn is_border_width(els: &[&str]) -> bool {
    match els {
        ["border"] => true,
        ["border", e] => matches!(*e, "t" | "r" | "b" | "l" | "x" | "y" | "s" | "e") || e.parse::<f32>().is_ok(),
        ["border", e, ..] => matches!(*e, "x" | "y"),
        _ => false,
    }
}

/// Is this ring element an explicit width? `2` (ring-2), `3px` (ring-[3px]),
/// bare `ring` (default width). Colors never parse as numbers and arbitrary
/// colors carry `#`/`oklch(`/names.
fn is_width_value(e: &str, arbitrary: Option<&str>) -> bool {
    let v = arbitrary.unwrap_or(e);
    v.parse::<f32>().is_ok() || v.ends_with("px") || v.ends_with("rem") || v.ends_with("em")
}

/// Parent group → groups it knocks out (tw_merge `get_collisions`, complete).
fn collisions(group: &str) -> &'static [&'static str] {
    match group {
        "overflow" => &["overflow-x", "overflow-y"],
        "inset" => &["inset-x", "inset-y", "start", "end", "top", "right", "bottom", "left"],
        "inset-x" => &["right", "left"],
        "inset-y" => &["top", "bottom"],
        "flex" => &["flex-basis", "flex-grow", "flex-shrink"],
        "gap" => &["gap-x", "gap-y"],
        "padding" => &["padding-x", "padding-y", "padding-start", "padding-end", "padding-top", "padding-right", "padding-bottom", "padding-left"],
        "padding-x" => &["padding-right", "padding-left"],
        "padding-y" => &["padding-top", "padding-bottom"],
        "margin" => &["margin-x", "margin-y", "margin-start", "margin-end", "margin-top", "margin-right", "margin-bottom", "margin-left"],
        "margin-x" => &["margin-right", "margin-left"],
        "margin-y" => &["margin-top", "margin-bottom"],
        "size" => &["width", "height"],
        "font-size" => &["line-height"],
        "rounded" => &["rounded-s", "rounded-e", "rounded-t", "rounded-r", "rounded-b", "rounded-l", "rounded-ss", "rounded-se", "rounded-ee", "rounded-es", "rounded-tl", "rounded-tr", "rounded-br", "rounded-bl"],
        "rounded-s" => &["rounded-ss", "rounded-es"],
        "rounded-e" => &["rounded-se", "rounded-ee"],
        "rounded-t" => &["rounded-tl", "rounded-tr"],
        "rounded-r" => &["rounded-tr", "rounded-br"],
        "rounded-b" => &["rounded-br", "rounded-bl"],
        "rounded-l" => &["rounded-tl", "rounded-bl"],
        "border-w" => &["border-w-s", "border-w-e", "border-w-t", "border-w-r", "border-w-b", "border-w-l"],
        "border-w-x" => &["border-w-r", "border-w-l"],
        "border-w-y" => &["border-w-t", "border-w-b"],
        "border-color" => &["border-color-t", "border-color-r", "border-color-b", "border-color-l"],
        "border-color-x" => &["border-color-r", "border-color-l"],
        "border-color-y" => &["border-color-t", "border-color-b"],
        _ => &[],
    }
}

/// The merge key: same variants + importance + group ⇒ later wins.
fn key_of(t: &ClassToken<'_>) -> (String, bool, &'static str) {
    let variants = t
        .variants
        .iter()
        .map(|v| match v {
            crate::ast::Variant::Named(n) => name_of(n),
            crate::ast::Variant::Grouped { name, .. } => name_of(name),
            crate::ast::Variant::Arbitrary(a) => a,
        })
        .collect::<Vec<_>>()
        .join(":");
    (variants, t.important, collision_group(t).unwrap_or("unclassified"))
}

fn name_of(n: &NamedVariant) -> &'static str {
    match n {
        NamedVariant::Hover => "hover",
        NamedVariant::Focus => "focus",
        NamedVariant::FocusVisible => "focus-visible",
        NamedVariant::FocusWithin => "focus-within",
        NamedVariant::Active => "active",
        NamedVariant::Disabled => "disabled",
        NamedVariant::Visited => "visited",
        NamedVariant::Dark => "dark",
        NamedVariant::Breakpoint(b) | NamedVariant::Container(b) => b.name(),
    }
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
    // Fast path: single token, nothing to merge.
    if joined.split_ascii_whitespace().count() <= 1 {
        return joined;
    }
    let parsed = {
        let (parsed, errs) = parse_class_string(&joined);
        if !errs.is_empty() {
            // Token/raw pairing would desync if some classes were skipped;
            // keep malformed input verbatim (cn is best-effort).
            return joined;
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
        assert_eq!(cn(&["rounded-md", "rounded-t-lg"]), "rounded-md rounded-t-lg");
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
        assert_eq!(cn(&["bg-primary", "hover:bg-primary/90"]), "bg-primary hover:bg-primary/90");
        assert_eq!(cn(&["hover:bg-primary/90", "hover:bg-accent"]), "hover:bg-accent");
    }

    #[test]
    fn unknown_kept_verbatim() {
        assert_eq!(cn(&["custom-plugin-thing", "px-2", "px-4"]), "custom-plugin-thing px-4");
    }

    #[test]
    fn display_conflicts() {
        assert_eq!(cn(&["block", "hidden"]), "hidden");
        assert_eq!(cn(&["flex", "items-center", "hidden"]), "items-center hidden");
    }

    #[test]
    fn opt_form() {
        assert_eq!(cn_opt([Some("px-2"), None, Some("px-4")]), "px-4");
    }

    #[test]
    fn ring_offset_is_its_own_group() {
        // `ring-offset-*` must not be shadowed by the `ring-*` catch-all:
        // offset width and ring color collide with neither.
        assert_eq!(cn(&["ring-ring/50", "ring-offset-2"]), "ring-ring/50 ring-offset-2");
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
