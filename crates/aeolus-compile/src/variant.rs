//! Variant → `Cond` classification: the compile stage's flat fold
//! (architecture plan §3.3 / `aeolus_ir::cond`'s own docs).
//!
//! The IR model is deliberately flat — one condition per layer — so a
//! variant STACK folds to its single dominant condition: the
//! highest-specificity segment, source order breaking ties. What folds
//! today is the typed `Cond` surface (`aeolus_ir::Cond`'s every
//! constructor: probe states, dark, breakpoints, containers, the
//! group/peer probes, the attribute probes) PLUS the registered
//! custom-variant aliases (the `custom` module's [`crate::Customs`] — the
//! pipeline's fold is generic over the [`crate::ClassifyVariant`] seam,
//! so a host can supply its own table). The ATTRIBUTE-PROBE families
//! (`data-*`, `aria-*`, `has-*`) fold here too — the differential oracle
//! is the v0.x resolver's own `has_atom`/`parse_attr`/`attr_test`, pinned
//! against the real tailwindcss v4.1.17 output in the golden corpus:
//!
//! * `data-<name>:` / `data-[<name>=<value>]:` — the element's own
//!   attribute probe; the bare spelling is the presence test, the
//!   bracketed spelling carries the value verbatim;
//! * `aria-<name>:` / `aria-[<name>=<value>]:` — the same probe shape,
//!   with the bare spelling normalized to the `"true"` value the corpus
//!   renders (`[aria-busy="true"]`);
//! * `has-<state>:` / `has-[:<state>]:` / `has-data-…` / `has-aria-…` —
//!   the descendant probe, its payload the star-prefixed `:has()`
//!   fragment the corpus renders (`:has(*:focus)`,
//!   `:has(*[data-slot="card-action"])`).
//!
//! What does NOT fold (arbitrary variants, unregistered spellings,
//! compound `has-` inners, structural inners, parent-relative compounds
//! like `group-data-*`) comes back as a typed unsupported-variant
//! diagnostic — never a silent drop.

use aeolus_ir::{Bp, Cond};

use crate::custom::ClassifyVariant;

/// Why one variant segment did not fold: the segment is outside the
/// typed [`Cond`] surface. A struct, not a bare `String` — the miss is
/// DATA (one meaning: this segment refused) with the segment text as
/// its payload, the same typed shape every other seam in the pipeline
/// reports through. The diagnostics layer renders
/// [`VariantMiss::variant`] verbatim as an unsupported-variant
/// diagnostic.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct VariantMiss {
    /// The variant segment that refused, verbatim (`"has-data-[x]"`).
    pub(crate) variant: String,
}

impl VariantMiss {
    /// The miss for one refusing segment.
    pub(crate) fn new(variant: &str) -> Self {
        Self {
            variant: variant.to_string(),
        }
    }
}

/// Fold the variant stack into the one condition the layer fires under.
/// An EMPTY stack is the bare utility: [`Cond::Base`]. The classifier
/// extends the built-in table with the registered custom-variant
/// aliases (built-ins win name collisions — the registry is the
/// oracle, the registrations are additive surface).
///
/// # Errors
/// The first variant outside the typed surface (built-in or
/// registered), as a typed [`VariantMiss`] — the caller renders it as
/// an unsupported-variant diagnostic with the segment's own span.
pub fn fold(variants: &[&str], classifier: &impl ClassifyVariant) -> Result<Cond, VariantMiss> {
    let mut dominant = Cond::Base;
    for &variant in variants {
        let cond = classify(variant, classifier).ok_or_else(|| VariantMiss::new(variant))?;
        if cond.specificity() >= dominant.specificity() {
            dominant = cond;
        }
    }
    Ok(dominant)
}

/// The typed classification for ONE variant segment: the built-in
/// table first, the registered aliases second. The named group/peer
/// label (`group-hover/menu`) folds to its probe: the NAME has no IR
/// slot yet (cond.rs documents the threading as a compile-stage job
/// this slice leaves to the codegen). The CSS emit arm reuses this as
/// the REFUSAL oracle (what a segment IS) while rendering the selector
/// shape itself (HOW it shows) — the two must agree, and `tests/css.rs`
/// pins it.
pub(crate) fn classify(variant: &str, classifier: &impl ClassifyVariant) -> Option<Cond> {
    classify_builtin(variant).or_else(|| classifier.classify(variant))
}

/// The built-in table's classification for ONE variant segment — the
/// closed spelling set the typed `Cond` surface covers. The custom
/// aliases ride [`classify`]'s second half, never this table.
///
/// The attribute-probe families check the RAW segment (their payloads
/// may contain `/`, which the group/peer label split would truncate).
/// A folding payload interns into the leak pool the IR documents as
/// the dynamic form's storage — one bounded string per folded
/// segment, the same cost the program's own slices pay.
pub(crate) fn classify_builtin(variant: &str) -> Option<Cond> {
    if let Some(rest) = variant.strip_prefix("data-") {
        let (name, value) = attr_fields(rest)?;
        return Some(Cond::Data { name, value });
    }
    if let Some(rest) = variant.strip_prefix("aria-") {
        let (name, value) = attr_fields(rest)?;
        // Bare `aria-*:` names the "true" value — the corpus renders
        // `[aria-busy="true"]` for the bare spelling, so both spellings
        // fold to ONE condition.
        let value = value.or(Some(intern("true")));
        return Some(Cond::Aria { name, value });
    }
    if let Some(rest) = variant.strip_prefix("has-") {
        let fragment = has_inner_fragment(rest)?;
        return Some(Cond::Has(fragment));
    }
    let (name, label) = match variant.split_once('/') {
        Some((name, label)) => (name, Some(label)),
        None => (variant, None),
    };
    if let Some(bp) = breakpoint(name) {
        return Some(Cond::Bp(bp));
    }
    // The parent-relative has-probes: the label split happened above
    // (the label rides the CSS head only, the group-hover precedent),
    // and the inner reads through the SAME fragment grammar the
    // element-own `has-` arm uses — one grammar, three consumers, the
    // refusal sets cannot drift.
    if let Some(rest) = name.strip_prefix("group-has-") {
        return has_inner_fragment(rest).map(Cond::GroupHas);
    }
    if let Some(rest) = name.strip_prefix("peer-has-") {
        return has_inner_fragment(rest).map(Cond::PeerHas);
    }
    if let Some(rest) = name.strip_prefix("min-") {
        return breakpoint(rest).map(Cond::Min);
    }
    if let Some(rest) = name.strip_prefix("max-") {
        return breakpoint(rest).map(Cond::Max);
    }
    // The container forms are the ONE family whose label is SEMANTIC:
    // the name names which container the query tests (the v0.9
    // oracle's `@md/name` fold), so it rides the condition instead of
    // staying a CSS-head decoration. An empty name (`@md/:`) is the
    // unknown-variant refusal, never a silent unnamed fold.
    if let Some(rest) = name.strip_prefix("@max-") {
        let bp = breakpoint(rest)?;
        let named = match label {
            Some("") => return None,
            Some(label) => Some(intern(label)),
            None => None,
        };
        return Some(Cond::MaxContainer { bp, name: named });
    }
    if let Some(rest) = name.strip_prefix('@') {
        let bp = breakpoint(rest)?;
        let named = match label {
            Some("") => return None,
            Some(label) => Some(intern(label)),
            None => None,
        };
        return Some(Cond::Container { bp, name: named });
    }
    match name {
        "dark" => Some(Cond::Dark),
        "hover" => Some(Cond::Hover),
        "focus" => Some(Cond::Focus),
        "focus-visible" => Some(Cond::FocusVisible),
        "focus-within" => Some(Cond::FocusWithin),
        "active" => Some(Cond::Active),
        "disabled" => Some(Cond::Disabled),
        "visited" => Some(Cond::Visited),
        "checked" => Some(Cond::Checked),
        "placeholder-shown" => Some(Cond::PlaceholderShown),
        "selection" => Some(Cond::Selection),
        "placeholder" => Some(Cond::Placeholder),
        "group-hover" => Some(Cond::GroupHover),
        "group-focus" => Some(Cond::GroupFocus),
        "peer-hover" => Some(Cond::PeerHover),
        "peer-focus" => Some(Cond::PeerFocus),
        _ => None,
    }
}

/// The attribute-probe payload: `(name, value)` from the `data-`/`aria-`
/// remainder — the v0.x oracle's `parse_attr`, tightened to the honest
/// grammar. The bracketed form (`[state=open]`) splits at the FIRST `=`
/// (the oracle's own split; a second `=` rides the value); the bare form
/// is the presence test and takes no `=` (the bracketed spelling is the
/// value form). Names are attribute-name text, values selector-safe
/// text — anything else refuses, so no renderer can emit a broken
/// selector from a folded probe.
fn attr_fields(rest: &str) -> Option<(&'static str, Option<&'static str>)> {
    let (name, value) = if let Some(inner) = rest.strip_prefix('[')
        && let Some(stripped) = inner.strip_suffix(']')
    {
        match stripped.split_once('=') {
            Some((n, v)) => (n, Some(v)),
            None => (stripped, None),
        }
    } else {
        // Bare spelling: the presence test. An `=` here is the
        // bracketed form's job (`data-state=open` is not a v4 spelling).
        if rest.contains('=') || rest.contains('[') || rest.contains(']') {
            return None;
        }
        (rest, None)
    };
    if name.is_empty() || !is_attr_name(name) {
        return None;
    }
    if let Some(v) = value
        && (v.is_empty() || v.contains('"') || v.contains('\\'))
    {
        return None;
    }
    Some((intern(name), value.map(intern)))
}

/// The attribute-name charset: ASCII identifier text (`a-z A-Z 0-9 _ -
/// :` is CSS's own namespace split; a name keeps to the identifier
/// half). Anything else refuses — the diagnostic is the honest report.
fn is_attr_name(name: &str) -> bool {
    name.bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

/// The `has-` inner → the rendered `:has(...)` fragment — the v0.x
/// oracle's `has_atom`, pinned by the corpus's star-prefixed forms.
/// One SIMPLE test per inner (the oracle's own boundary):
///
/// * a probe state, named (`has-focus`) or colon-wrapped
///   (`has-[:focus]`) → `*:focus`;
/// * a data-/aria- attribute compound (`has-data-[slot=x]`,
///   `has-aria-invalid`) → `*[data-slot="x"]`, `*[aria-invalid="true"]`;
/// * anything else (compound selectors, relative selectors, structural
///   inners — the compile stage has no `first:` surface yet) refuses.
///
/// `pub(crate)`: the CSS arm's parent-relative heads
/// (`group-has-*`/`peer-has-*`) render the SAME fragment from the SAME
/// grammar — the render cannot drift the fold.
pub(crate) fn has_inner_fragment(rest: &str) -> Option<&'static str> {
    // The bracketed form strips to the same inner the named form reads
    // (the oracle: `sel = inner without surrounding brackets`).
    let inner = rest
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'))
        .unwrap_or(rest);
    let state = inner.trim_start_matches(':');
    if let Some(pseudo) = state_pseudo(state) {
        return Some(intern(&format!("*{pseudo}")));
    }
    if let Some(data) = inner.strip_prefix("data-") {
        let (name, value) = attr_fields(if data.starts_with('[') {
            data
        } else {
            // The bare compound takes the name only (`has-data-slot`);
            // a value rides the bracketed spelling.
            if data.contains('=') {
                return None;
            }
            data
        })?;
        return Some(intern(&format!("*{}", attr_selector("data", name, value))));
    }
    if let Some(aria) = inner.strip_prefix("aria-") {
        let (name, value) = attr_fields(if aria.starts_with('[') {
            aria
        } else {
            if aria.contains('=') {
                return None;
            }
            aria
        })?;
        // Bare aria inside has- means "true" too (the corpus's
        // `has-aria-invalid` → `*[aria-invalid="true"]`).
        let value = value.or(Some(intern("true")));
        return Some(intern(&format!("*{}", attr_selector("aria", name, value))));
    }
    None
}

/// The pseudo-class text for one probe-state name — the same spellings
/// the CSS arm's built-in table renders (`focus` → `:focus`), minus the
/// pseudo-ELEMENT render targets (`selection`/`placeholder` style a
/// different box; `has-*` descendants are state probes).
fn state_pseudo(name: &str) -> Option<&'static str> {
    match name {
        "hover" => Some(":hover"),
        "focus" => Some(":focus"),
        "focus-visible" => Some(":focus-visible"),
        "focus-within" => Some(":focus-within"),
        "active" => Some(":active"),
        "disabled" => Some(":disabled"),
        "visited" => Some(":visited"),
        "checked" => Some(":checked"),
        "placeholder-shown" => Some(":placeholder-shown"),
        _ => None,
    }
}

/// The attribute-test selector text for a probe payload — the corpus's
/// `attr_test` form: the value quotes, the presence test stays bare,
/// and the bare aria value is the caller-normalized `"true"`. The CSS
/// arm derives the SAME text from the [`Cond`] payload (one grammar,
/// two consumers — the render cannot drift the fold).
pub(crate) fn attr_selector(kind: &str, name: &str, value: Option<&str>) -> String {
    match value {
        Some(v) => format!("[{kind}-{name}=\"{v}\"]"),
        None => format!("[{kind}-{name}]"),
    }
}

/// Intern a payload into the leak pool — the IR's documented dynamic-
/// form storage ("one interned `Box<[Decl]>` shared by reference"): a
/// one-time cost per folded segment, never a per-render allocation.
fn intern(text: &str) -> &'static str {
    Box::leak(text.to_string().into_boxed_str())
}

/// The named breakpoint (`"sm"` → `Bp::Sm`) — a reverse walk over
/// [`Bp::ALL`]/[`Bp::name`], so the IR's own table is the single source
/// and a new breakpoint cannot drift the compile stage's spelling table
/// (this used to be a parallel `match` wall; it is the IR's data now).
pub(crate) fn breakpoint(name: &str) -> Option<Bp> {
    Bp::ALL.iter().copied().find(|bp| bp.name() == name)
}
