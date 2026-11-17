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
//!   `:has(*[data-slot="card-action"])`);
//! * `supports-[<condition>]:` — the `@supports` at-rule condition (the
//!   family's only v4 spelling is the bracketed arbitrary form; the
//!   differential evidence is the tailwindcss v4.1.17 source's own
//!   normalization, `packages/tailwindcss/src/variants.ts`'s
//!   `functional('supports', …)` arm — function-prefixed inners pass
//!   through verbatim with `and`/`or`/`not` connectives spaced,
//!   colon-bearing declarations wrap in parens, bare features expand to
//!   the `(feature: var(--tw))` declaration probe — see
//!   [`supports_condition`], the one-grammar public face the CLI fact
//!   flag reuses).
//!
//! * `first:` / `last:` / `odd:` / `even:` / `only:` / `empty:` /
//!   `nth-N:` / `nth-last-N:` — the STRUCTURAL family, the element's
//!   own DOM-position probes (the differential oracle is the v0.x
//!   resolver's own `Structural` table, pinned against the real
//!   tailwindcss v4.1.17 output; the `nth-` grammar is the oracle's
//!   `parse_nth` — every `i32`, the selector forms `nth-[2n+1]:`
//!   staying outside).
//!
//! What does NOT fold (the OTHER arbitrary variants — the `*:` and
//! `[@media…]` spellings — unregistered spellings,
//! compound `has-` inners — including structural inners, `has-first:` —
//! parent-relative compounds like `group-data-*`, and the structural
//! parent-relative compositions `group-first:`/`peer-last:`) comes back
//! as a typed unsupported-variant diagnostic — never a silent drop.

use aeolus_ir::{Bp, Cond, Structural};

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
    // The supports family checks the RAW segment like the other
    // payload-carrying probes (a condition's text may carry `/` —
    // `supports-[aspect-ratio:16/9]` — which the group/peer label split
    // would truncate). The only v4 spelling is the bracketed arbitrary
    // form: a bare `supports-display` refuses, a stray suffix after the
    // closing bracket refuses.
    if let Some(rest) = variant.strip_prefix("supports-") {
        let inner = rest.strip_prefix('[').and_then(|s| s.strip_suffix(']'))?;
        let condition = supports_condition(inner)?;
        return Some(Cond::Supports(condition));
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
    // The structural family: the bare spellings first, then the `nth-`
    // ordinals. `nth-last-` is checked BEFORE `nth-` — the shared
    // prefix would otherwise route `nth-last-2` into the `nth-` arm and
    // parse-fail `-last-2`. The ordinal grammar is the v0.x oracle's
    // own `parse_nth` (every `i32`; the bracketed selector forms
    // `nth-[2n+1]` stay outside — the `nth-` arm's parse refuses them
    // and the diagnostic is the honest report).
    if let Some(rest) = name.strip_prefix("nth-last-") {
        return rest
            .parse::<i32>()
            .ok()
            .map(|n| Cond::Structural(Structural::NthLast(n)));
    }
    if let Some(rest) = name.strip_prefix("nth-") {
        return rest
            .parse::<i32>()
            .ok()
            .map(|n| Cond::Structural(Structural::Nth(n)));
    }
    let bare = match name {
        "first" => Some(Structural::First),
        "last" => Some(Structural::Last),
        "odd" => Some(Structural::Odd),
        "even" => Some(Structural::Even),
        "only" => Some(Structural::Only),
        "empty" => Some(Structural::Empty),
        _ => None,
    };
    if let Some(test) = bare {
        return Some(Cond::Structural(test));
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
    // The media-feature preferences: the CLOSED static-variant set
    // tailwindcss v4.1.17 carries (the at-rule texts are the
    // `aeolus_ir::Pref` docs' pins) — the preference families beside
    // the input-capability tier and the orientation forms.
    match name {
        "motion-safe" => Some(Cond::Media(aeolus_ir::Pref::MotionSafe)),
        "motion-reduce" => Some(Cond::Media(aeolus_ir::Pref::MotionReduce)),
        "contrast-more" => Some(Cond::Media(aeolus_ir::Pref::ContrastMore)),
        "contrast-less" => Some(Cond::Media(aeolus_ir::Pref::ContrastLess)),
        "forced-colors" => Some(Cond::Media(aeolus_ir::Pref::ForcedColors)),
        "print" => Some(Cond::Media(aeolus_ir::Pref::Print)),
        "pointer-fine" => Some(Cond::Media(aeolus_ir::Pref::PointerFine)),
        "pointer-coarse" => Some(Cond::Media(aeolus_ir::Pref::PointerCoarse)),
        "pointer-none" => Some(Cond::Media(aeolus_ir::Pref::PointerNone)),
        "any-pointer-fine" => Some(Cond::Media(aeolus_ir::Pref::AnyPointerFine)),
        "any-pointer-coarse" => Some(Cond::Media(aeolus_ir::Pref::AnyPointerCoarse)),
        "any-pointer-none" => Some(Cond::Media(aeolus_ir::Pref::AnyPointerNone)),
        "portrait" => Some(Cond::Media(aeolus_ir::Pref::Portrait)),
        "landscape" => Some(Cond::Media(aeolus_ir::Pref::Landscape)),
        // The media-feature remainder (the v4.1 additions the same
        // blocks' `staticVariant` calls carry — the at-rule texts the
        // `Pref` docs pin): the user agent's inverted-colours answer
        // and the scripting feature's `none` value.
        "inverted-colors" => Some(Cond::Media(aeolus_ir::Pref::InvertedColors)),
        "noscript" => Some(Cond::Media(aeolus_ir::Pref::NoScript)),
        // The direction family: the selector-side static variants the
        // v4.1.17 source carries beside the media-feature tier (the
        // upstream selectors are the CSS arm's own pins) — the
        // resolved direction is frame state, the `Direction` bit the
        // Env answers.
        "ltr" => Some(Cond::Dir(aeolus_ir::Direction::Ltr)),
        "rtl" => Some(Cond::Dir(aeolus_ir::Direction::Rtl)),
        "dark" => Some(Cond::Dark),
        "hover" => Some(Cond::Hover),
        "focus" => Some(Cond::Focus),
        "focus-visible" => Some(Cond::FocusVisible),
        "focus-within" => Some(Cond::FocusWithin),
        "active" => Some(Cond::Active),
        "disabled" => Some(Cond::Disabled),
        "visited" => Some(Cond::Visited),
        "checked" => Some(Cond::Checked),
        // The validation states (the v4.1.17 forms block's static
        // variants, the selector-side tier beside `checked:`): the
        // constraint pair answers from ONE Env fact (`valid:` its
        // complement — CSS's own pristine-field reading), the `user-`
        // pair additionally gates on the interaction fact the HTML
        // spec requires.
        "valid" => Some(Cond::Valid),
        "invalid" => Some(Cond::Invalid),
        "user-valid" => Some(Cond::UserValid),
        "user-invalid" => Some(Cond::UserInvalid),
        // The element-state family (the v4.1.17 state/forms blocks'
        // static variants, the selector-side tier beside the
        // validation states): the target/indeterminate/autofill facts
        // answer their ONE question each, `read-only:` the
        // editability fact's COMPLEMENT (the `valid:` precedent —
        // CSS's own reading for a non-editable element IS
        // `:read-only`). The matches-neither spellings ride their own
        // family below (the form-state family — the boundary evidence
        // 27 documented closes with the two-fact shape: a membership
        // fact per universe beside the pair's shared state bit).
        "target" => Some(Cond::Target),
        "indeterminate" => Some(Cond::Indeterminate),
        "autofill" => Some(Cond::Autofill),
        "read-only" => Some(Cond::ReadOnly),
        // The widget-state family (the v4.1.17 state/forms blocks'
        // static variants evidence 28 left at the v0.x-surface
        // boundary — the family closes the gap in the other direction:
        // the v0.x `State` surface grew the slots AND the fold grows
        // the arms). All three are AFFIRMATIVE one-fact questions (the
        // `target:` shape: a non-open element matches neither
        // `:open`'s arms, a non-form element misses `:default`, a
        // non-inert subtree misses `:is([inert], [inert] *)`), so the
        // resting env answers every one of them with the quiet miss a
        // selector that does not match is — no complement rides.
        "open" => Some(Cond::Open),
        "inert" => Some(Cond::Inert),
        "default" => Some(Cond::Default),
        // The form-state family (the v4.1.17 state/forms blocks' static
        // variants evidence 27 left at the matches-neither boundary —
        // the family closes it): `enabled:`/`required:`/`optional:`
        // gate on the form-membership fact (a non-form element matches
        // NEITHER half of each pair — the two-fact shape one Env bit
        // could not answer honestly), `in-range:`/`out-of-range:` gate
        // on the bounded-universe membership fact beside the value's
        // position bit (the same shape, the `required_field` bit's
        // complement reading within its universe).
        "enabled" => Some(Cond::Enabled),
        "required" => Some(Cond::Required),
        "optional" => Some(Cond::Optional),
        "in-range" => Some(Cond::InRange),
        "out-of-range" => Some(Cond::OutOfRange),
        "placeholder-shown" => Some(Cond::PlaceholderShown),
        "selection" => Some(Cond::Selection),
        "placeholder" => Some(Cond::Placeholder),
        // The pseudo-element tier (the tailwindcss v4.1.17
        // pseudo-elements + state blocks' remaining `staticVariant`
        // registrations, pinned verbatim in the IR's own docs): the
        // render-target spellings beside the selection/placeholder
        // pair — `before:`/`after:` the functional bodies (the CSS
        // arm carries the upstream content-injection contract), the
        // rest the single-selector parts.
        "before" => Some(Cond::Before),
        "after" => Some(Cond::After),
        "marker" => Some(Cond::Marker),
        "first-letter" => Some(Cond::FirstLetter),
        "first-line" => Some(Cond::FirstLine),
        "file" => Some(Cond::FileSelector),
        "backdrop" => Some(Cond::Backdrop),
        "details-content" => Some(Cond::DetailsContent),
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
        // The validation states: the element-own probe states beside
        // `checked:` (the same tier, the same `:pseudo` shape — the
        // has- inner grammar answers them like any state spelling).
        "valid" => Some(":valid"),
        "invalid" => Some(":invalid"),
        "user-valid" => Some(":user-valid"),
        "user-invalid" => Some(":user-invalid"),
        // The element-state family: the element-own probe states
        // beside the validation states (the same tier, the same
        // `:pseudo` shape — the has- inner grammar answers them like
        // any state spelling).
        "target" => Some(":target"),
        "indeterminate" => Some(":indeterminate"),
        "autofill" => Some(":autofill"),
        "read-only" => Some(":read-only"),
        // The widget-state family: the element-own probe states beside
        // the element-state family (the same tier, the same
        // `:pseudo`/`:is(...)` shape the upstream `staticVariant`
        // texts pin — the has- inner grammar answers them like any
        // state spelling, the v4 mechanism rendering the inner's `&`
        // as `*`).
        "open" => Some(":is([open], :popover-open, :open)"),
        "inert" => Some(":is([inert], [inert] *)"),
        "default" => Some(":default"),
        // The form-state family: the element-own probe states beside
        // the widget-state family (the same tier, the same `:pseudo`
        // shape the upstream `staticVariant` texts pin — the has- inner
        // grammar answers them like any state spelling).
        "enabled" => Some(":enabled"),
        "required" => Some(":required"),
        "optional" => Some(":optional"),
        "in-range" => Some(":in-range"),
        "out-of-range" => Some(":out-of-range"),
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

/// The `supports-[…]` inner → the rendered `@supports` condition — the
/// tailwindcss v4.1.17 source's own normalization
/// (`packages/tailwindcss/src/variants.ts`, the
/// `functional('supports', …)` arm), tightened to the honest grammar.
/// The caller passes the INNER text (the bracket strip is the fold's
/// job — a bare `supports-display` is not v4 grammar and refuses there).
///
/// One normalization per shape, pinned from the source:
///
/// * function-prefixed (`^[\w-]*\s*\(` — `(display:grid)`,
///   `selector(:hover)`, `not(display:grid)`) → the condition passes
///   through verbatim, `and`/`or`/`not` connectives spaced (Chrome's
///   `(a)or(b)` bug is the source's own normalization reason), runs of
///   whitespace collapsed to one (CSS whitespace is insignificant; the
///   payload stays deterministic);
/// * a colon-bearing declaration (`display:grid`) → wraps in parens
///   (`(display:grid)`);
/// * a bare feature (`display`) → the declaration probe
///   (`(display: var(--tw))`, the source's own shorthand words).
///
/// The v4 arbitrary-value convention rides the fold too: `_` spells
/// the space a class string cannot carry (`supports-[(width_>_400px)]`
/// → `(width > 400px)`), converted before the branches; a backslash
/// escape is outside the honest grammar and refuses.
///
/// The tightening (the honest-failure contract — a condition no
/// `@supports` grammar accepts never folds, so no renderer can emit a
/// broken head): the empty inner refuses, braces and the `;` rule
/// separator refuse, and unbalanced parentheses refuse (the source
/// passes them through; aeolus reports them as the typed
/// unsupported-variant diagnostic instead).
///
/// `pub`: the fold's own grammar is the CLI fact flag's validator too
/// (`expand --supports <condition>` — one grammar, two consumers, the
/// refusal sets cannot drift; the `has_inner_fragment` precedent).
pub fn supports_condition(inner: &str) -> Option<&'static str> {
    if inner.is_empty()
        || inner.contains('{')
        || inner.contains('}')
        || inner.contains(';')
        || inner.contains('\\')
    {
        return None;
    }
    // The v4 arbitrary-value convention: `_` spells the spaces a class
    // string cannot carry (`supports-[(width_>_400px)]` -> `(width >
    // 400px)`). The conversion runs BEFORE the branches, so every
    // shape normalizes the same way; a backslash escape is outside the
    // honest grammar and refuses above.
    let spaced = inner.replace('_', " ");
    let inner = spaced.as_str();
    if starts_function(inner) {
        if !parens_balanced(inner) {
            return None;
        }
        let spaced = space_connectives(inner);
        return Some(intern(spaced.trim()));
    }
    if !inner.contains(':') {
        // The bare-feature shorthand expands to the declaration probe —
        // the source's own words: `supports-[display]` ->
        // `@supports (display: var(--tw))`.
        let text = format!("({inner}: var(--tw))");
        return Some(intern(&text));
    }
    if !parens_balanced(inner) {
        return None;
    }
    // The wrap branch: the declaration form gains its parens (the
    // source's `(${value})`; a paren-balanced text cannot start with
    // `(` here — the function branch took it — so the wrap is
    // unconditional, the source's own check degenerating to this form).
    let text = format!("({inner})");
    Some(intern(&text))
}

/// The source's function-prefix test (`/^[\w-]*\s*\(/`): zero or more
/// identifier chars, optional whitespace, then an open paren.
fn starts_function(inner: &str) -> bool {
    let rest =
        inner.trim_start_matches(|c: char| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    rest.trim_start().starts_with('(')
}

/// The parentheses balance: every `(` closes, no `)` closes an open
/// paren that is not there. The honesty tightening over the source's
/// own pass-through.
fn parens_balanced(text: &str) -> bool {
    let mut depth = 0usize;
    for ch in text.chars() {
        match ch {
            '(' => depth += 1,
            ')' => {
                if depth == 0 {
                    return false;
                }
                depth -= 1;
            }
            _ => {}
        }
    }
    depth == 0
}

/// The connective spacing pass: `and`/`or`/`not` WORDS get one space on
/// each side (the source's `replace(/\b(and|or|not)\b/g, ' $1 ')` — a
/// maximal identifier run equal to the word is the `\b` boundary pair's
/// Rust spelling), runs of whitespace collapsed to one, the result
/// trimmed. Deterministic per inner.
fn space_connectives(inner: &str) -> String {
    let mut out = String::with_capacity(inner.len() + 8);
    let mut run = String::new();
    for ch in inner.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            run.push(ch);
            continue;
        }
        flush_word(&mut out, &mut run);
        if ch == ' ' {
            if !out.is_empty() && !out.ends_with(' ') {
                out.push(' ');
            }
        } else {
            out.push(ch);
        }
    }
    flush_word(&mut out, &mut run);
    out
}

/// Flush one maximal identifier run into the output, spacing it when the
/// run is a connective word.
fn flush_word(out: &mut String, run: &mut String) {
    if run.is_empty() {
        return;
    }
    let spaced = matches!(run.as_str(), "and" | "or" | "not");
    if spaced && !out.is_empty() && !out.ends_with(' ') {
        out.push(' ');
    }
    out.push_str(run);
    if spaced {
        out.push(' ');
    }
    run.clear();
}

/// The named breakpoint (`"sm"` → `Bp::Sm`) — a reverse walk over
/// [`Bp::ALL`]/[`Bp::name`], so the IR's own table is the single source
/// and a new breakpoint cannot drift the compile stage's spelling table
/// (this used to be a parallel `match` wall; it is the IR's data now).
pub(crate) fn breakpoint(name: &str) -> Option<Bp> {
    Bp::ALL.iter().copied().find(|bp| bp.name() == name)
}

// ---------------------------------------------------------------------------
// The completion surface (the editor half's audited spelling set)
// ---------------------------------------------------------------------------

/// The closed, CONCRETE variant spelling set the fold accepts as-is —
/// the editor-completion surface (the LSP's completion consults it, so
/// a suggested variant is always a spelling the fold's own
/// `classify_builtin` answers). The audit is two-sided and test-pinned below: every entry
/// folds (no stale suggestion), and every base spelling the match
/// walls accept is present (no silent omission). The OPEN families
/// (`data-<attr>`, `aria-<attr>`, `has-<fragment>`, `nth-<n>`, named
/// group/peer/container labels) are represented by their documented
/// demo spellings — the payload grammars are open-ended, not a closed
/// list.
pub const KNOWN_VARIANTS: &[&str] = &[
    // The probe states (the state match wall's own order).
    "hover",
    "focus",
    "focus-visible",
    "focus-within",
    "active",
    "disabled",
    "visited",
    "checked",
    // The validation states (the closed forms-block spellings).
    "valid",
    "invalid",
    "user-valid",
    "user-invalid",
    // The element-state family (the closed state/forms-block
    // spellings beside the validation quartet).
    "target",
    "indeterminate",
    "autofill",
    "read-only",
    // The widget-state family (the closed state/forms-block spellings
    // evidence 28 left at the v0.x boundary — the family that closed
    // the gap).
    "open",
    "inert",
    "default",
    // The form-state family (the closed state/forms-block spellings
    // evidence 27 left at the matches-neither boundary — the family
    // that closes it with the two-fact shape).
    "enabled",
    "required",
    "optional",
    "in-range",
    "out-of-range",
    "placeholder-shown",
    "selection",
    "placeholder",
    // The mode.
    "dark",
    // The parent probes: the bare spellings, then the named-label form
    // (the one `/label` demo spelling the fold documents).
    "group-hover",
    "group-focus",
    "peer-hover",
    "peer-focus",
    "group-hover/menu",
    // The structural family — the bare spellings only (the `nth-N`
    // ordinals are every i32, not a closed spelling set).
    "first",
    "last",
    "odd",
    "even",
    "only",
    "empty",
    // The breakpoints (Bp::ALL's names, ascending).
    "sm",
    "md",
    "lg",
    "xl",
    "2xl",
    // The min/max prefixes over one representative breakpoint each.
    "min-sm",
    "max-md",
    // The container forms: ascending, the max form, and the named
    // container demo (`@md/card`).
    "@sm",
    "@md",
    "@lg",
    "@xl",
    "@2xl",
    "@max-md",
    "@md/card",
    // The attribute-probe families: the demo spellings (a custom
    // payload rides the same prefix grammar).
    "data-open",
    "aria-busy",
    "has-focus",
    "group-has-focus",
    "peer-has-focus",
    // The supports family: the bracketed arbitrary form's demo spelling
    // (a custom condition rides the same prefix grammar).
    "supports-[display:grid]",
    // The media-feature preferences (the closed static-variant set).
    "motion-safe",
    "motion-reduce",
    "contrast-more",
    "contrast-less",
    "forced-colors",
    "print",
    // The input-capability and orientation tier (the same closed set's
    // remaining v4.1.17 spellings).
    "pointer-fine",
    "pointer-coarse",
    "pointer-none",
    "any-pointer-fine",
    "any-pointer-coarse",
    "any-pointer-none",
    "portrait",
    "landscape",
    // The direction family (the selector-side static variants).
    "ltr",
    "rtl",
];

// ---------------------------------------------------------------------------
// The completion surface's audit (the drift catch, test-pinned in place)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod completion_tests {
    use super::*;

    /// Every entry of [`KNOWN_VARIANTS`] must fold — the completion
    /// surface only ever suggests spellings the pipeline's own oracle
    /// accepts (the `COLOUR_GROUPS` audit pattern: a stale entry fails
    /// here, never a suggested class the compiler refuses).
    #[test]
    fn every_known_variant_folds() {
        for name in KNOWN_VARIANTS {
            assert!(
                classify_builtin(name).is_some(),
                "the completion surface names {name:?}, which does not fold"
            );
        }
    }

    /// The closed set carries NO duplicates (a duplicate would render
    /// twice in an editor's popup) and stays in its canonical,
    /// semantically-grouped order — the deterministic answer the tests
    /// and the editor popups pin.
    #[test]
    fn no_duplicates() {
        let mut sorted = KNOWN_VARIANTS.to_vec();
        sorted.sort_unstable();
        let len = sorted.len();
        sorted.dedup();
        assert_eq!(sorted.len(), len, "KNOWN_VARIANTS carries a duplicate");
    }

    /// The hand-audit: every base spelling the fold's own match walls
    /// accept is present — a name added to `classify_builtin` without
    /// joining the completion surface is a test failure here (the
    /// reverse direction of [`every_known_variant_folds`]). The
    /// breakpoint spellings ride `Bp::ALL`'s names, the parameterized
    /// families (`nth-`, `data-`, `aria-`, `has-` with custom payloads)
    /// are represented by their documented demo spellings.
    #[test]
    fn the_closed_set_covers_every_base_spelling() {
        let expected: &[&str] = &[
            // The probe states (the state match wall).
            "hover",
            "focus",
            "focus-visible",
            "focus-within",
            "active",
            "disabled",
            "visited",
            "checked",
            "placeholder-shown",
            // The validation states (the closed forms-block spellings).
            "valid",
            "invalid",
            "user-valid",
            "user-invalid",
            // The element-state family (the closed state/forms-block
            // spellings beside the validation quartet).
            "target",
            "indeterminate",
            "autofill",
            "read-only",
            // The widget-state family (the closed state/forms-block
            // spellings evidence 28 left at the v0.x boundary — the
            // family that closed the gap).
            "open",
            "inert",
            "default",
            // The form-state family (the closed state/forms-block
            // spellings evidence 27 left at the matches-neither
            // boundary — the family that closes it with the two-fact
            // shape).
            "enabled",
            "required",
            "optional",
            "in-range",
            "out-of-range",
            "selection",
            "placeholder",
            // The mode.
            "dark",
            // The parent probes (bare + the named-label demo spelling).
            "group-hover",
            "group-focus",
            "peer-hover",
            "peer-focus",
            "group-hover/menu",
            // The structural family (the bare spellings; `nth-N` is the
            // parameterized arm the list represents with none — the
            // ordinals are every i32, not a closed spelling set).
            "first",
            "last",
            "odd",
            "even",
            "only",
            "empty",
            // The breakpoints (Bp::ALL's names).
            "sm",
            "md",
            "lg",
            "xl",
            "2xl",
            // The min/max prefixes over a representative breakpoint.
            "min-sm",
            "max-md",
            // The container forms (bare + max + the named-label demo).
            "@sm",
            "@md",
            "@lg",
            "@xl",
            "@2xl",
            "@max-md",
            "@md/card",
            // The attribute-probe families (the demo spellings the fold
            // documents; custom payloads are open-ended, not closed).
            "data-open",
            "aria-busy",
            "has-focus",
            "group-has-focus",
            "peer-has-focus",
            // The supports family (the demo spelling; custom conditions
            // are open-ended, not closed).
            "supports-[display:grid]",
            // The media-feature preferences (the closed set).
            "motion-safe",
            "motion-reduce",
            "contrast-more",
            "contrast-less",
            "forced-colors",
            "print",
            // The input-capability and orientation tier (the same
            // closed set's remaining v4.1.17 spellings).
            "pointer-fine",
            "pointer-coarse",
            "pointer-none",
            "any-pointer-fine",
            "any-pointer-coarse",
            "any-pointer-none",
            "portrait",
            "landscape",
            // The direction family (the selector-side static variants).
            "ltr",
            "rtl",
        ];
        let mut actual: Vec<&str> = KNOWN_VARIANTS.to_vec();
        let mut expected: Vec<&str> = expected.to_vec();
        actual.sort_unstable();
        expected.sort_unstable();
        assert_eq!(actual, expected, "the closed set drifted from the fold");
    }
}
