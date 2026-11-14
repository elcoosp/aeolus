//! The custom-authoring seam's CSS-FILE form (architecture plan §5.4:
//! "CSS-first compatibility: `@theme`, `@utility` (incl. functional
//! `--value()`), `@custom-variant` — same file as web"): the
//! `@custom-variant name (selector);` statements and the
//! `@utility name { @apply …; }` composite bodies a project's theme CSS
//! carries lower onto the SAME typed [`Customs`] surface the API form
//! registers — no parallel semantics, no legacy read.
//!
//! ```css
//! @custom-variant night (&:where(.dark, .dark *));  /* typed alias  */
//! @utility brand-badge { @apply rounded-md px-2; }  /* applied body */
//! ```
//!
//! The design keeps the pipeline's load-bearing invariants intact:
//!
//! * **The parse is typed end to end** (the no-silent-failures
//!   contract): the statement walk mirrors the `@source` seam's shape
//!   (comments stripped first, statements run to their `;`/`}`), and
//!   every malformed form is a typed [`CustomCssError`] naming the
//!   statement — never a guessed registration, never a quiet skip.
//! * **The selector table is the audited set**: a `@custom-variant`
//!   alias folds through the spellings the flat [`Cond`] surface can
//!   name — the `.dark` family (the audited reader's own containment
//!   rule) and the built-in state selectors — and every other spelling
//!   refuses typed. The IR has no slot for arbitrary selectors; the
//!   refusal is the honest boundary, not a drop.
//! * **The applied body compiles at registration**: the `@apply`
//!   classes run through the SAME pipeline (`compile_with`, the
//!   registration-order customs resolving), the body's variant-carrying
//!   classes refuse (the flat `Cond` model composes the CANDIDATE's own
//!   condition, never the body's), the body's diagnostics refuse (the
//!   honest-failure contract reaches registration), and the compiled
//!   declarations + their keyword texts intern — the multi-decl shape
//!   the single-`Val` forms cannot hold.
//! * **The functional star form rides the SAME grammar**: the
//!   `@utility brand-p-* { padding: --spacing(n); }` bodies register
//!   through the SAME [`CustomForm`] the typed API takes — one suffix
//!   fills ONE form, the CSS property names map onto the IR longhand
//!   targets (the audited shorthands fan like the registry rows do),
//!   and every shape outside the grammar (mixed forms, literal values,
//!   unknown properties) refuses typed.
//! * **The registration order resolves**: a later `@utility` may apply
//!   an earlier custom root (the same longest-root rule the read side
//!   walks), and a duplicate root registration refuses typed — the
//!   builder form's "first wins" is a silent drop here, and this seam
//!   has no silent channel.

use std::fmt;

use crate::Config;
use crate::custom::{CustomUtility, CustomVariant, Customs};

/// Why a CSS statement did not register: the typed refusal set, one
/// variant per statement family. The `Display` renders the statement's
/// own name and payload — the caller's error path reads the same text
/// the diagnostics channel would carry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CustomCssError {
    /// A `@custom-variant`/`@utility` statement is malformed: a missing
    /// selector, an unbalanced block, a duplicate root — the parse
    /// never guesses.
    Statement {
        /// The statement's own name (`card`, `brand-x`), when the
        /// header carries one.
        name: String,
        /// The malformed shape's reason.
        message: String,
    },
    /// A `@custom-variant` selector outside the audited table — the
    /// arbitrary-selector boundary (the flat `Cond` surface has no
    /// slot for it).
    Selector {
        /// The alias's spelling.
        name: String,
        /// The refused selector, verbatim.
        selector: String,
    },
    /// An `@utility` body that is not a pure `@apply` list: a raw CSS
    /// declaration body, or `@apply` mixed with raw declarations — the
    /// CSS declaration grammar has no IR lowering in this slice.
    Body {
        /// The utility's root name.
        name: String,
    },
    /// An `@utility` body whose `@apply` classes did not compile to a
    /// clean, base-condition-only program: the first diagnostic's own
    /// text rides (unknown class, variant-carrying class, the empty
    /// body) — the honest-failure contract reaches registration.
    ApplyBody {
        /// The utility's root name.
        name: String,
        /// The first refusal's text (the compile walk's own message).
        message: String,
    },
    /// The functional star form's body refusals (`@utility brand-w-*`):
    /// mixed `--value` forms in one body, literal CSS values, unknown
    /// property names, unknown value forms, the empty value, the
    /// star-less header carrying a functional body — one suffix fills
    /// ONE form, and every refusal names its reason.
    Functional {
        /// The utility's root name (star-less).
        name: String,
        /// The refused shape's reason.
        message: String,
    },
}

impl fmt::Display for CustomCssError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CustomCssError::Statement { name, message } => {
                if name.is_empty() {
                    write!(f, "malformed custom statement: {message}")
                } else {
                    write!(f, "malformed custom statement `{name}`: {message}")
                }
            }
            CustomCssError::Selector { name, selector } => write!(
                f,
                "custom variant `{name}`: the selector `{selector}` is outside the typed \
                 condition surface — register an alias whose selector names a built-in \
                 condition (the `.dark` family, the state spellings)"
            ),
            CustomCssError::Body { name } => write!(
                f,
                "custom utility `{name}`: the body is not a pure `@apply` list — raw CSS \
                 declarations have no IR lowering; spell the utility through `@apply` or \
                 the typed `Customs` API"
            ),
            CustomCssError::ApplyBody { name, message } => write!(
                f,
                "custom utility `{name}`: the `@apply` body did not compile cleanly — \
                 {message}"
            ),
            CustomCssError::Functional { name, message } => write!(
                f,
                "custom utility `{name}`: the functional body did not register — {message}"
            ),
        }
    }
}

impl std::error::Error for CustomCssError {}

/// The audited selector table: the spellings a `@custom-variant`
/// statement may alias, keyed by the exact selector text. The `.dark`
/// family folds by the audited reader's own containment rule (the
/// legacy `extract_custom_rules` walk — the differential source); the
/// state spellings are the built-in conditions' own CSS selector
/// forms. Everything outside the table refuses typed.
fn selector_to_cond(name: &str, selector: &str) -> Result<crate::Cond, CustomCssError> {
    use crate::Cond;
    // The `.dark` family: the audited containment rule (the selector
    // names the dark class somewhere in its chain).
    if selector.contains(".dark") {
        return Ok(Cond::Dark);
    }
    let cond = match selector {
        "&:hover" => Cond::Hover,
        "&:focus" => Cond::Focus,
        "&:focus-visible" => Cond::FocusVisible,
        "&:focus-within" => Cond::FocusWithin,
        "&:active" => Cond::Active,
        "&:disabled" => Cond::Disabled,
        "&:visited" => Cond::Visited,
        "&:checked" => Cond::Checked,
        "&::placeholder" => Cond::PlaceholderShown,
        _ => {
            return Err(CustomCssError::Selector {
                name: name.to_string(),
                selector: selector.to_string(),
            });
        }
    };
    Ok(cond)
}

/// Collect the custom-authoring statements a theme CSS carries and
/// register them in source order — the file form of
/// [`Customs`](crate::Customs). Only the `@custom-variant` and
/// `@utility` statements count; the `@theme`/`:root`/`.dark`/`@source`
/// bodies are the theme pass's own (never registrations here).
///
/// The `@utility` bodies compile AT REGISTRATION — the apply classes
/// run through [`compile_with`](crate::compile_with) with the customs
/// registered so far, so a later utility may apply an earlier custom
/// root. A body that does not compile cleanly (unknown class,
/// variant-carrying class, the empty body) refuses typed with the
/// compile walk's own first message.
///
/// # Errors
/// [`CustomCssError`] per malformed statement, refused selector or
/// refused body — never a guessed registration.
pub fn customs_from_css(css: &str, config: &Config) -> Result<Customs, CustomCssError> {
    let owned = strip_comments(css);
    let mut customs = Customs::new();
    let mut rest: &str = &owned;
    while let Some(i) = rest.find("@custom-variant") {
        let after = &rest[i + "@custom-variant".len()..];
        let (name, selector, consumed) = parse_variant_statement(after)?;
        let cond = selector_to_cond(&name, &selector)?;
        customs = customs.variant(CustomVariant::new(&name, cond));
        rest = &after[consumed..];
    }
    rest = &owned;
    while let Some(i) = rest.find("@utility") {
        let after = &rest[i + "@utility".len()..];
        let (name, body, consumed) = parse_utility_statement(after)?;
        // The star form is the functional spelling: the body's
        // `--value(type)`/`--spacing(n)` declarations register through
        // the SAME typed `CustomForm` grammar the API form uses (the
        // plan §5.4 row's "(incl. functional `--value()`)" half).
        let functional_root = name.strip_suffix("-*").map(str::to_string);
        if let Some(root) = functional_root {
            if customs.utilities().iter().any(|u| u.root == root)
                || customs.utilities().iter().any(|u| u.root == name)
            {
                return Err(CustomCssError::Statement {
                    name: root.clone(),
                    message: "duplicate root registration — the first statement wins, the \
                              builder form's tie is a silent drop this seam refuses instead"
                        .to_string(),
                });
            }
            let utility = functional_utility(&root, &body)?;
            customs = customs.utility(utility);
            rest = &after[consumed..];
            continue;
        }
        if customs.utilities().iter().any(|u| u.root == name) {
            return Err(CustomCssError::Statement {
                name: name.clone(),
                message: "duplicate root registration — the first statement wins, the \
                          builder form's tie is a silent drop this seam refuses instead"
                    .to_string(),
            });
        }
        let utility = applied_utility(&name, &body, &customs, *config)?;
        customs = customs.utility(utility);
        rest = &after[consumed..];
    }
    Ok(customs)
}

/// Parse one `@custom-variant` statement: `name (selector);`. The
/// block form (`name { … }`) and the bare form (`name;`) are the typed
/// refusals — the block's selector body has no flat `Cond` slot and a
/// selector-less alias names nothing. Returns the name, the selector
/// text and the consumed byte count.
fn parse_variant_statement(after: &str) -> Result<(String, String, usize), CustomCssError> {
    let brace = after.find('{');
    let open = after.find('(');
    let semi = after.find(';');
    // The block form: a `{` before any `(` — the selector body has no
    // flat slot, refused with its own payload named.
    if let Some(b) = brace
        && open.is_none_or(|o| b < o)
    {
        let name = after[..b].trim().to_string();
        let close = matching_brace(after.as_bytes(), b);
        let payload = after
            .get(b + 1..close.min(after.len()))
            .unwrap_or("")
            .trim();
        return Err(CustomCssError::Selector {
            name,
            selector: payload.to_string(),
        });
    }
    match (open, semi) {
        // `name (selector);` — the alias form.
        (Some(o), Some(s)) if o < s => {
            let name = after[..o].trim().to_string();
            let Some(close_rel) = after[o..].find(')') else {
                return Err(CustomCssError::Statement {
                    name,
                    message: "unterminated selector — the `(` never closes".to_string(),
                });
            };
            let close = o + close_rel;
            if close > s {
                return Err(CustomCssError::Statement {
                    name,
                    message: "the selector runs past its `;` terminator".to_string(),
                });
            }
            let selector = after[o + 1..close].trim().to_string();
            if name.is_empty() || selector.is_empty() {
                return Err(CustomCssError::Statement {
                    name,
                    message: "the alias form carries both a name and a `(selector)`".to_string(),
                });
            }
            Ok((name, selector, s + 1))
        }
        // The bare form: names nothing.
        (None, Some(s)) => {
            let name = after[..s].trim().to_string();
            Err(CustomCssError::Statement {
                name,
                message: "the bare `@custom-variant name;` names nothing — the alias \
                          form carries a `(selector)`"
                    .to_string(),
            })
        }
        _ => Err(CustomCssError::Statement {
            name: String::new(),
            message: "unterminated `@custom-variant` statement — no `;` terminator".to_string(),
        }),
    }
}

/// Parse one `@utility` statement: `name { body }`. The body runs to
/// its matching `}` (an unbalanced block refuses — the malformed tail
/// is never sliced out of bounds, the audited walk's own guard).
/// Returns the root name, the body text and the consumed byte count.
fn parse_utility_statement(after: &str) -> Result<(String, String, usize), CustomCssError> {
    let Some(b) = after.find('{') else {
        return Err(CustomCssError::Statement {
            name: after.trim().to_string(),
            message: "the `@utility` statement carries a `{ body }` block".to_string(),
        });
    };
    let name = after[..b].trim().to_string();
    if name.is_empty() {
        return Err(CustomCssError::Statement {
            name,
            message: "the `@utility` header carries a root name".to_string(),
        });
    }
    let close = matching_brace(after.as_bytes(), b);
    if close >= after.len() {
        return Err(CustomCssError::Statement {
            name,
            message: "unbalanced `@utility` block — the `}` never closes".to_string(),
        });
    }
    let body = after.get(b + 1..close).unwrap_or("").trim().to_string();
    Ok((name, body, close + 1))
}

/// Lower one functional star body (`@utility brand-w-* { … }`) onto its
/// [`CustomUtility::functional`] registration. Every declaration in the
/// body carries a `--value(type)`/`--spacing(n)` value; ONE suffix
/// fills ONE form, so all the declarations must share the form (the
/// mixed body refuses typed), and the CSS property names map onto the
/// IR longhand targets — the exact longhand names, plus the audited
/// shorthands fanning like the registry rows do (padding/margin TRBL,
/// border-radius corners, inset edges).
fn functional_utility(root: &str, body: &str) -> Result<CustomUtility, CustomCssError> {
    let functional = |message: String| CustomCssError::Functional {
        name: root.to_string(),
        message,
    };
    let mut form: Option<crate::CustomForm> = None;
    let mut targets: Vec<crate::Prop> = Vec::new();
    let mut saw_decl = false;
    for stmt in split_statements(body) {
        let stmt = stmt.trim();
        if stmt.is_empty() {
            continue;
        }
        saw_decl = true;
        let Some((prop_name, value)) = stmt.split_once(':') else {
            return Err(functional(
                "every declaration carries `property: value`".to_string(),
            ));
        };
        let prop_name = prop_name.trim();
        let value = value.trim();
        let decl_form = value_form(value).ok_or_else(|| {
            functional(format!(
                "the value `{value}` is outside the typed \
                 forms — the `--value(integer|number|percentage|color)` and `--spacing(n)` \
                 spellings are the grammar"
            ))
        })?;
        if let Some(seen) = form {
            if seen != decl_form {
                return Err(functional(
                    "one suffix fills ONE form — the body's declarations carry mixed forms"
                        .to_string(),
                ));
            }
        } else {
            form = Some(decl_form);
        }
        targets.extend(longhand_targets(prop_name).ok_or_else(|| {
            functional(format!(
                "the property `{prop_name}` is outside the IR's longhand set and the \
                 audited shorthands"
            ))
        })?);
    }
    if !saw_decl {
        return Err(functional("the body carries no declarations".to_string()));
    }
    let Some(form) = form else {
        return Err(functional("the body carries no value form".to_string()));
    };
    Ok(CustomUtility::functional(root, form, &targets))
}

/// The value forms the functional body grammar carries — the typed
/// [`crate::CustomForm`] labels: the `--value(...)` hints and the
/// `--spacing(n)` shorthand (the built-in spacing scale's own spelling).
fn value_form(value: &str) -> Option<crate::CustomForm> {
    use crate::CustomForm;
    let inner = value
        .strip_prefix("--value(")
        .and_then(|rest| rest.strip_suffix(')'));
    if let Some(hint) = inner {
        return match hint.trim() {
            "integer" => Some(CustomForm::Integer),
            "number" => Some(CustomForm::Number),
            "percentage" => Some(CustomForm::Percentage),
            "color" => Some(CustomForm::Token),
            _ => None,
        };
    }
    // The `--spacing(n)` shorthand and its `--value(spacing)` spelling.
    if value == "--spacing(n)" {
        return Some(CustomForm::Spacing);
    }
    if value
        .strip_prefix("--value(")
        .and_then(|r| r.strip_suffix(')'))
        == Some("spacing")
    {
        return Some(CustomForm::Spacing);
    }
    None
}

/// The IR longhand targets a CSS property name fans onto: the exact
/// longhand names map 1:1 (the IR's own `name()` spellings), and the
/// audited shorthands fan in the registry rows' own order — padding and
/// margin TRBL, border-radius corners, inset edges.
fn longhand_targets(prop_name: &str) -> Option<Vec<crate::Prop>> {
    use crate::Prop;
    let fan: &[(&str, &[Prop])] = &[
        (
            "padding",
            &[
                Prop::PaddingTop,
                Prop::PaddingRight,
                Prop::PaddingBottom,
                Prop::PaddingLeft,
            ],
        ),
        (
            "margin",
            &[
                Prop::MarginTop,
                Prop::MarginRight,
                Prop::MarginBottom,
                Prop::MarginLeft,
            ],
        ),
        (
            "border-radius",
            &[
                Prop::BorderTopLeftRadius,
                Prop::BorderTopRightRadius,
                Prop::BorderBottomRightRadius,
                Prop::BorderBottomLeftRadius,
            ],
        ),
        ("inset", &[Prop::Top, Prop::Right, Prop::Bottom, Prop::Left]),
    ];
    for (name, targets) in fan {
        if *name == prop_name {
            return Some(targets.to_vec());
        }
    }
    Prop::ALL
        .iter()
        .find(|prop| prop.name() == prop_name)
        .map(|prop| vec![*prop])
}

/// Lower one `@utility` body onto its [`CustomUtility::applied`]
/// registration: the body must be a pure `@apply` list, the classes
/// compile through the registration-order customs, and the compiled
/// base layer's declarations + their keyword texts intern (the leak
/// pool — the registration's one-time storage story). A body with raw
/// CSS declarations, with no `@apply` at all, with variant-carrying
/// classes or with any compile diagnostic refuses typed.
fn applied_utility(
    name: &str,
    body: &str,
    customs: &Customs,
    config: Config,
) -> Result<CustomUtility, CustomCssError> {
    let classes = apply_classes(name, body)?;
    let compiled = crate::compile_with(&classes, customs, &config).map_err(|err| {
        CustomCssError::ApplyBody {
            name: name.to_string(),
            message: err.to_string(),
        }
    })?;
    if let Some(diagnostic) = compiled.diagnostics.first() {
        return Err(CustomCssError::ApplyBody {
            name: name.to_string(),
            message: diagnostic.to_string(),
        });
    }
    // The flat `Cond` model composes the CANDIDATE's own condition — a
    // body whose classes carry variants would need a composed condition
    // the IR has no slot for; the registration refuses instead of
    // folding the body's variants away.
    let base = compiled
        .program
        .layers
        .iter()
        .find(|layer| layer.cond == crate::Cond::Base);
    let Some(base) = base else {
        return Err(CustomCssError::ApplyBody {
            name: name.to_string(),
            message: "the `@apply` body lowered nothing — every class refused".to_string(),
        });
    };
    if compiled
        .program
        .layers
        .iter()
        .any(|layer| layer.cond != crate::Cond::Base)
    {
        return Err(CustomCssError::ApplyBody {
            name: name.to_string(),
            message: "the `@apply` body carries variant-prefixed classes — the flat \
                      condition model composes the candidate's own condition, never the \
                      body's; spell the body's states at the call site"
                .to_string(),
        });
    }
    // The keyword texts: the body's own classes are built-ins (the
    // compile-with-customs walk splices customs as declarations), so
    // the annotation seam names every row-relative slot the body
    // carries — aligned to the compiled program, filtered to the base
    // layer, interned beside the decls.
    let annotations =
        crate::keyword_texts(&classes, &config).map_err(|err| CustomCssError::ApplyBody {
            name: name.to_string(),
            message: err.to_string(),
        })?;
    let aligned = crate::keyword_alignment(&compiled, &annotations);
    let mut flat = aligned.into_iter();
    let mut keywords: Vec<(crate::Prop, &'static str)> = Vec::new();
    for layer in compiled.program.layers {
        for decl in layer.decls {
            let text = flat.next().unwrap_or_default();
            if layer.cond == crate::Cond::Base && !text.is_empty() {
                keywords.push((decl.prop, text));
            }
        }
    }
    let decls: &'static [crate::Decl] = Box::leak(base.decls.to_vec().into_boxed_slice());
    let keywords: &'static [(crate::Prop, &'static str)] = Box::leak(keywords.into_boxed_slice());
    Ok(CustomUtility::applied(name, decls, keywords))
}

/// The `@apply` classes a utility body carries. The body must be a
/// PURE `@apply` list — one or more `@apply …;` statements and nothing
/// else (a raw CSS declaration beside an `@apply` is the [`CustomCss
/// Error::Body`] boundary). Returns the class string (multiple
/// statements concatenate, the v4 form's own reading).
fn apply_classes(name: &str, body: &str) -> Result<String, CustomCssError> {
    let mut classes = String::new();
    let mut saw_apply = false;
    for stmt in split_statements(body) {
        let stmt = stmt.trim();
        if stmt.is_empty() {
            continue;
        }
        match stmt.strip_prefix("@apply") {
            Some(rest) => {
                saw_apply = true;
                if !classes.is_empty() {
                    classes.push(' ');
                }
                classes.push_str(rest.trim());
            }
            None => {
                return Err(CustomCssError::Body {
                    name: name.to_string(),
                });
            }
        }
    }
    if !saw_apply {
        return Err(CustomCssError::Body {
            name: name.to_string(),
        });
    }
    Ok(classes)
}

/// Split a body into top-level statements — `;` at paren-depth zero
/// (the audited walk's own split; a `;` inside a selector payload is
/// not a terminator).
fn split_statements(body: &str) -> impl Iterator<Item = &str> {
    let mut items = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    for (i, b) in body.bytes().enumerate() {
        match b {
            b'(' => depth += 1,
            b')' => depth = depth.saturating_sub(1),
            b';' if depth == 0 => {
                items.push(&body[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    if start < body.len() {
        items.push(&body[start..]);
    }
    items.into_iter()
}

/// Remove `/* ... */` comments so a statement mentioned inside one is
/// not one — the `@source` seam's own walk (the audited reader's
/// simplified form, mirrored so the two statement walkers cannot drift
/// apart in behaviour).
fn strip_comments(css: &str) -> String {
    if !css.contains("/*") {
        return css.to_string();
    }
    let mut out = String::with_capacity(css.len());
    let mut rest = css;
    while let Some(i) = rest.find("/*") {
        out.push_str(&rest[..i]);
        let Some(end) = rest[i + 2..].find("*/") else {
            return out;
        };
        rest = &rest[i + 2 + end + 2..];
    }
    out.push_str(rest);
    out
}

/// The matching `}` for the `{` at `open` — the audited walk's own
/// guard (an unbalanced block runs to the end and the caller refuses).
fn matching_brace(bytes: &[u8], open: usize) -> usize {
    let mut depth = 0usize;
    for (i, &b) in bytes.iter().enumerate().skip(open) {
        match b {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return i;
                }
            }
            _ => {}
        }
    }
    bytes.len()
}
