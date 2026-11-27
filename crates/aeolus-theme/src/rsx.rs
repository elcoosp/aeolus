//! The `rsx.toml` import (architecture plan §5, "Integrating with
//! `gpui-rsx`", config-bridge bullet: *"an `rsx.toml → aeolus theme`
//! importer"*). One function walks a gpui-rsx theme file onto the SAME
//! CSS-first surface every other theme rides — no parallel theme path,
//! no second reader to drift.
//!
//! The grammar is the upstream one, mirrored line for line (the
//! section set, the quoted-string/inline-expression value forms, the
//! quote-aware `#` comment strip, the per-section value grammar) so a
//! sheet gpui-rsx accepts is a sheet this import accepts. What the two
//! systems CANNOT share rides the honest channels instead of being
//! invented:
//!
//! - the sheet: colour tokens bake the `--color-*` namespace, lengths
//!   bake `--radius-*`/`--spacing-*`/`--text-*`/`--leading-*` — the
//!   v4 namespace map, read by the audited reader [`ThemeSource`]
//!   wraps. A `[line-height]` entry that shares a `[text-size]`
//!   entry's name pairs onto it (`--text-{name}--line-height`, the
//!   reader's own pairing form); a lone one takes `--leading-{name}`.
//! - the notes: Rust-expression values (`{ expr = "…" }`), the
//!   runtime preset, the gpui-rsx runtime options and named opacities
//!   have no CSS form — each rides a typed [`RsxNote`] naming what
//!   left and why, the house `LoweringReport` shape at the import seam.
//! - the errors: every grammar refusal is a line-numbered
//!   [`RsxError`] variant carrying the upstream message's shape, never
//!   a stringly panic.
//!
//! The config bridge's spacing half rides the report too:
//! [`RsxImport::spacing_scale`] answers the `[options]`
//! `spacing-scale` value. `tailwind` is aeolus's own default scale
//! (identical semantics); `px` is gpui-rsx's 1-unit-1-px scale, which
//! aeolus deliberately has NO equivalent of — the migration guide's
//! scale row is that difference's answer, not a silent bake.
//!
//! ```no_run
//! use aeolus_theme::{TokenTable, import_rsx};
//!
//! let sheet = "[colors]\nprimary = \"#0ea5e9\"\n";
//! let import = import_rsx(sheet).expect("the sheet imports");
//! let source = import.theme_source().expect("the sheet bakes");
//! let table = TokenTable::runtime(&source).expect("the table builds");
//! assert!(table.id("primary").is_some(), "the imported token rides");
//! ```

use std::fmt;

use crate::{ThemeError, ThemeSource};

// ---------------------------------------------------------------------------
// The report's types
// ---------------------------------------------------------------------------

/// The spacing-scale option the `[options]` section carries (the
/// config bridge's spacing half). gpui-rsx's default is [`Self::Px`];
/// aeolus's default is [`Self::Tailwind`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RsxSpacingScale {
    /// `spacing-scale = "tailwind"` (or absent): `1 unit = 0.25rem` —
    /// identical semantics to aeolus's own default scale.
    Tailwind,
    /// `spacing-scale = "px"`: gpui-rsx's default scale, `1 unit =
    /// 1px`. Aeolus has no equivalent mode — mapping it onto the
    /// compile crate's absolute-px bake would change every length by
    /// 4× (that bake is the v4 rem scale at a fixed window size, not a
    /// 1-px unit scale). The migration guide's scale row is the
    /// answer; the report carries the value so a host can CHECK.
    Px,
}

impl RsxSpacingScale {
    /// True when the sheet asks for gpui-rsx's 1-px unit scale (the
    /// migration guide's scale row applies).
    #[must_use]
    pub fn is_px(self) -> bool {
        matches!(self, Self::Px)
    }
}

impl fmt::Display for RsxSpacingScale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tailwind => f.write_str("tailwind"),
            Self::Px => f.write_str("px"),
        }
    }
}

/// Why an imported entry rode a NOTE instead of the sheet. Every kind
/// is a documented boundary, never a silent drop — the house
/// no-silent-failures contract at the import seam.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RsxNoteKind {
    /// The value is a Rust expression (`{ expr = "…" }`): the GPUI
    /// runtime resolves it, so it has no CSS form. The aeolus answer
    /// is the static value written as CSS (a hex colour, a length, a
    /// `--shadow-*` layer list).
    RustExpression,
    /// The `preset = "gpui-component"` option: it seeds the sheet with
    /// RUNTIME colour expressions (`cx.theme().colors.…`). The aeolus
    /// side of that seam is the `aeolus-gpui-component` bridge, whose
    /// projection maps the same token names onto a live palette.
    RuntimePreset,
    /// A gpui-rsx runtime option (`gpui-path`, `context`,
    /// `dynamic-runtime`): it names GPUI codegen wiring, not theme
    /// content — nothing to import.
    RuntimeOption,
    /// A named opacity (`[opacity]`): Tailwind v4 has no opacity theme
    /// namespace — opacity rides the per-class `/N` modifier in v4 and
    /// aeolus both. The value is the modifier to use.
    NamedOpacity,
    /// A named spacing token (`[spacing]`): the sheet carries it (the
    /// v4 functional namespace), but aeolus's spacing classes consume
    /// the numeric scale — the token is inert until a v4-namespace
    /// consumer names it.
    NamedSpacing,
}

impl fmt::Display for RsxNoteKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RustExpression => f.write_str(
                "a Rust expression: the GPUI runtime resolves it, so the sheet \
                 takes the static value written as CSS instead",
            ),
            Self::RuntimePreset => f.write_str(
                "the gpui-component preset seeds runtime colour expressions; \
                 the aeolus-gpui-component bridge is the live-palette seam",
            ),
            Self::RuntimeOption => {
                f.write_str("a gpui-rsx runtime option: codegen wiring, not theme content")
            }
            Self::NamedOpacity => f.write_str(
                "named opacities have no v4 theme namespace; the value \
                             is the per-class /N modifier to use",
            ),
            Self::NamedSpacing => f.write_str(
                "the token rides the sheet, but aeolus's spacing classes consume \
                 the numeric scale",
            ),
        }
    }
}

/// One imported entry that did NOT become a sheet line: what left, its
/// source line, and the typed why.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RsxNote {
    /// The entry's 1-based source line.
    pub line: usize,
    /// The entry's key, verbatim.
    pub name: String,
    /// The boundary kind.
    pub kind: RsxNoteKind,
}

impl fmt::Display for RsxNote {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "rsx.toml:{}: `{}` — {}", self.line, self.name, self.kind)
    }
}

/// The import result: the `@theme` sheet, the config bridge's spacing
/// half, and the note channel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RsxImport {
    /// The emitted `@theme` sheet (the mappings, source order).
    css: String,
    /// The `[options]` `spacing-scale` value (the default: Tailwind).
    spacing_scale: RsxSpacingScale,
    /// The unmappable entries, in source order.
    notes: Vec<RsxNote>,
}

impl RsxImport {
    /// The emitted `@theme` sheet — hand it to [`ThemeSource`] (or
    /// bake straight through [`Self::theme_source`]).
    #[must_use]
    pub fn css(&self) -> &str {
        &self.css
    }

    /// The sheet's `spacing-scale` option (the config bridge's spacing
    /// half). [`RsxSpacingScale::Px`] names the scale difference the
    /// migration guide answers.
    #[must_use]
    pub fn spacing_scale(&self) -> RsxSpacingScale {
        self.spacing_scale
    }

    /// The unmappable entries, in source order — what left and why,
    /// never a silent drop.
    #[must_use]
    pub fn notes(&self) -> &[RsxNote] {
        &self.notes
    }

    /// The imported sheet as a [`ThemeSource`] — the SAME surface the
    /// web `app.css` path and the built-in palettes ride.
    ///
    /// # Errors
    /// [`ThemeError`] when the emitted sheet does not bake (the
    /// audited reader's own typed errors; unreachable for the
    /// importer's own grammar, carried honestly anyway).
    pub fn theme_source(&self) -> Result<ThemeSource, ThemeError> {
        ThemeSource::from_css(&self.css, None)
    }
}

// ---------------------------------------------------------------------------
// The typed errors (the upstream grammar's own failure modes)
// ---------------------------------------------------------------------------

/// The import's typed refusals. Every variant carries the offending
/// line and the upstream message's shape (`rsx.toml:{line}: …`), so a
/// sheet gpui-rsx refuses for one reason refuses here for the SAME
/// reason.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RsxError {
    /// A `[section]` outside the closed set.
    UnknownSection {
        /// The header's line.
        line: usize,
        /// The section name, verbatim.
        section: String,
    },
    /// A `key = value` line before any section header.
    KeyOutsideSection {
        /// The line.
        line: usize,
        /// The key, verbatim.
        key: String,
    },
    /// The line carries no `=`.
    MissingEquals {
        /// The line.
        line: usize,
    },
    /// The key is empty or carries characters outside `[A-Za-z0-9_-]`.
    InvalidKey {
        /// The line.
        line: usize,
        /// The key, verbatim.
        key: String,
    },
    /// The same key declared twice in one section.
    DuplicateKey {
        /// The line.
        line: usize,
        /// The section.
        section: String,
        /// The key.
        key: String,
    },
    /// The value is neither a quoted string nor an
    /// `{ expr = "…" }` inline table (or an inline table whose key is
    /// not `expr`).
    BadValue {
        /// The line.
        line: usize,
        /// The value text, verbatim.
        value: String,
    },
    /// An `[options]` key outside the closed set.
    UnknownOption {
        /// The line.
        line: usize,
        /// The option key.
        option: String,
    },
    /// `spacing-scale` outside `"px" | "tailwind"`.
    BadSpacingScale {
        /// The line.
        line: usize,
        /// The value.
        value: String,
    },
    /// `preset` outside the closed set (the upstream grammar knows
    /// one preset: `"gpui-component"`).
    BadPreset {
        /// The line.
        line: usize,
        /// The value.
        value: String,
    },
    /// A `[colors]` string outside `#rgb/#rrggbb/#rrggbbaa`.
    BadColor {
        /// The line.
        line: usize,
        /// The key.
        name: String,
        /// The value.
        value: String,
    },
    /// A length outside the upstream grammar (`Npx` / `Nrem`).
    BadLength {
        /// The line.
        line: usize,
        /// The key.
        name: String,
        /// The value.
        value: String,
    },
    /// An opacity outside 0..=1 (fraction or percent form).
    BadOpacity {
        /// The line.
        line: usize,
        /// The key.
        name: String,
        /// The value.
        value: String,
    },
    /// A `[shadow]` string value: the upstream grammar takes
    /// expressions ONLY here (a shadow is GPUI-side Rust, by design).
    ShadowNeedsExpression {
        /// The line.
        line: usize,
        /// The key.
        name: String,
    },
}

impl fmt::Display for RsxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownSection { line, section } => write!(
                f,
                "rsx.toml:{line}: unknown section `[{section}]` (expected options, \
                 colors, radius, spacing, text-size, line-height, opacity or shadow)"
            ),
            Self::KeyOutsideSection { line, key } => {
                write!(f, "rsx.toml:{line}: `{key}` must be inside a section")
            }
            Self::MissingEquals { line } => write!(f, "rsx.toml:{line}: expected `key = value`"),
            Self::InvalidKey { line, key } => write!(f, "rsx.toml:{line}: invalid key `{key}`"),
            Self::DuplicateKey { line, section, key } => {
                write!(f, "rsx.toml:{line}: duplicate key `{key}` in [{section}]")
            }
            Self::BadValue { line, value } => write!(
                f,
                "rsx.toml:{line}: expected a quoted string or an inline \
                 `{{ expr = \"...\" }}`, found `{value}`"
            ),
            Self::UnknownOption { line, option } => write!(
                f,
                "rsx.toml:{line}: unknown option `{option}` (expected spacing-scale, \
                 preset, context, gpui-path or dynamic-runtime)"
            ),
            Self::BadSpacingScale { line, value } => write!(
                f,
                "rsx.toml:{line}: spacing-scale must be \"px\" or \"tailwind\", \
                 found \"{value}\""
            ),
            Self::BadPreset { line, value } => write!(
                f,
                "rsx.toml:{line}: unknown preset \"{value}\" (supported: \
                 \"gpui-component\")"
            ),
            Self::BadColor { line, name, value } => {
                write!(
                    f,
                    "rsx.toml:{line}: `{name}` is not a #rgb/#rrggbb/#rrggbbaa color (found `{value}`)"
                )
            }
            Self::BadLength { line, name, value } => write!(
                f,
                "rsx.toml:{line}: `{name}` must be a length like \"12px\" or \
                 \"0.75rem\" (found `{value}`)"
            ),
            Self::BadOpacity { line, name, value } => write!(
                f,
                "rsx.toml:{line}: `{name}` must be an opacity between 0 and 1, \
                 such as \"0.6\" or \"60%\" (found `{value}`)"
            ),
            Self::ShadowNeedsExpression { line, name } => write!(
                f,
                "rsx.toml:{line}: `{name}` expects an inline Rust expression like \
                 {{ expr = \"...\" }}"
            ),
        }
    }
}

impl std::error::Error for RsxError {}

#[cfg(feature = "miette")]
impl miette::Diagnostic for RsxError {
    fn code(&self) -> Option<Box<dyn fmt::Display + '_>> {
        Some(Box::new(match self {
            Self::UnknownSection { .. } => "aeolus::theme::rsx::unknown_section",
            Self::KeyOutsideSection { .. } => "aeolus::theme::rsx::key_outside_section",
            Self::MissingEquals { .. } => "aeolus::theme::rsx::missing_equals",
            Self::InvalidKey { .. } => "aeolus::theme::rsx::invalid_key",
            Self::DuplicateKey { .. } => "aeolus::theme::rsx::duplicate_key",
            Self::BadValue { .. } => "aeolus::theme::rsx::bad_value",
            Self::UnknownOption { .. } => "aeolus::theme::rsx::unknown_option",
            Self::BadSpacingScale { .. } => "aeolus::theme::rsx::bad_spacing_scale",
            Self::BadPreset { .. } => "aeolus::theme::rsx::bad_preset",
            Self::BadColor { .. } => "aeolus::theme::rsx::bad_color",
            Self::BadLength { .. } => "aeolus::theme::rsx::bad_length",
            Self::BadOpacity { .. } => "aeolus::theme::rsx::bad_opacity",
            Self::ShadowNeedsExpression { .. } => "aeolus::theme::rsx::shadow_needs_expression",
        }))
    }

    fn help(&self) -> Option<Box<dyn fmt::Display + '_>> {
        Some(Box::new(match self {
            Self::UnknownSection { .. } => {
                "the sections are options, colors, radius, spacing, text-size, \
                 line-height, opacity and shadow — the upstream grammar's set"
            }
            Self::KeyOutsideSection { .. } => {
                "every key = value line sits inside one [section] header"
            }
            Self::MissingEquals { .. } => "a line is `key = value`, one entry per line",
            Self::InvalidKey { .. } => {
                "keys are ASCII letters, digits, `-` and `_` (the upstream \
                 grammar's own set)"
            }
            Self::DuplicateKey { .. } => "declare each key once per section",
            Self::BadValue { .. } => {
                "values are quoted strings (\"…\") or inline { expr = \"…\" } \
                 tables; unescape \\\" and \\\\ inside strings"
            }
            Self::UnknownOption { .. } => {
                "the options are spacing-scale, preset, context, gpui-path and \
                 dynamic-runtime"
            }
            Self::BadSpacingScale { .. } => "spacing-scale is \"px\" or \"tailwind\"",
            Self::BadPreset { .. } => "the one supported preset is \"gpui-component\"",
            Self::BadColor { .. } => {
                "write #rgb, #rrggbb or #rrggbbaa (the upstream grammar's hex \
                 forms)"
            }
            Self::BadLength { .. } => "write a px or rem length: \"12px\", \"0.75rem\"",
            Self::BadOpacity { .. } => {
                "write a fraction (\"0.6\") or a percent (\"60%\") between 0 and 1"
            }
            Self::ShadowNeedsExpression { .. } => {
                "shadows are GPUI-side Rust by design: { expr = \"vec![…]\" }; the \
                 aeolus sheet form is a --shadow-{name} CSS layer list"
            }
        }))
    }
}

// ---------------------------------------------------------------------------
// The parser (the upstream grammar's own line walk, mirrored)
// ---------------------------------------------------------------------------

/// A section outside the closed set cannot reach the mapper.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Section {
    Options,
    Colors,
    Radius,
    Spacing,
    TextSize,
    LineHeight,
    Opacity,
    Shadow,
}

impl Section {
    /// The header's section, if the name is in the closed set.
    fn from_header(name: &str) -> Option<Self> {
        match name {
            "options" => Some(Self::Options),
            "colors" => Some(Self::Colors),
            "radius" => Some(Self::Radius),
            "spacing" => Some(Self::Spacing),
            "text-size" => Some(Self::TextSize),
            "line-height" => Some(Self::LineHeight),
            "opacity" => Some(Self::Opacity),
            "shadow" => Some(Self::Shadow),
            _ => None,
        }
    }
}

/// The upstream value grammar's two forms.
#[derive(Clone, Debug, PartialEq, Eq)]
enum RawValue {
    /// `"…"` — the static form (the sheet's input).
    Str(String),
    /// `{ expr = "…" }` — the GPUI-runtime form (the note channel's).
    Expr(String),
}

impl fmt::Display for RawValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Str(value) => f.write_str(value),
            Self::Expr(expr) => write!(f, "{{ expr = \"{expr}\" }}"),
        }
    }
}

/// One collected entry, grammar-validated, awaiting the mapper.
struct Entry {
    line: usize,
    section: Section,
    key: String,
    value: RawValue,
}

/// The quote-aware `#` comment strip (the upstream walk, mirrored): a
/// `#` inside a quoted value is payload (`canvas = "#111a16"`), a `\"`
/// escape does not close the string.
fn strip_comment(line: &str) -> &str {
    let mut in_string = false;
    let mut escaped = false;
    for (index, c) in line.char_indices() {
        match c {
            _ if escaped => escaped = false,
            '\\' if in_string => escaped = true,
            '"' => in_string = !in_string,
            '#' if !in_string => return &line[..index],
            _ => {}
        }
    }
    line
}

/// The quoted string form, unescaped the way the upstream reader does.
fn parse_string(value: &str, line: usize) -> Result<String, RsxError> {
    let inner = value
        .strip_prefix('"')
        .and_then(|v| v.strip_suffix('"'))
        .ok_or_else(|| RsxError::BadValue {
            line,
            value: value.to_string(),
        })?;
    Ok(inner.replace("\\\"", "\"").replace("\\\\", "\\"))
}

/// The value form: a quoted string or an inline `{ expr = "…" }`.
fn parse_value(value: &str, line: usize) -> Result<RawValue, RsxError> {
    if let Some(inner) = value.strip_prefix('{').and_then(|v| v.strip_suffix('}')) {
        let (key, rest) = inner.split_once('=').ok_or_else(|| RsxError::BadValue {
            line,
            value: value.to_string(),
        })?;
        if key.trim() != "expr" {
            return Err(RsxError::BadValue {
                line,
                value: value.to_string(),
            });
        }
        return parse_string(rest.trim(), line).map(RawValue::Expr);
    }
    parse_string(value, line).map(RawValue::Str)
}

/// The upstream colour grammar: `#rgb`, `#rrggbb`, `#rrggbbaa`.
fn parse_color(color: &str) -> bool {
    let Some(hex) = color
        .strip_prefix('#')
        .filter(|hex| hex.bytes().all(|b| b.is_ascii_hexdigit()))
    else {
        return false;
    };
    matches!(hex.len(), 3 | 6 | 8)
}

/// The upstream length grammar: a finite f32 with a `px`/`rem` suffix.
fn parse_length(length: &str) -> bool {
    let number = |raw: &str| raw.parse::<f32>().ok().filter(|n| n.is_finite());
    let raw = length
        .strip_suffix("px")
        .or_else(|| length.strip_suffix("rem"));
    raw.and_then(number).is_some()
}

/// The upstream opacity grammar: a fraction or percent between 0 and 1.
fn parse_opacity(value: &str) -> bool {
    let number = |raw: &str| raw.parse::<f32>().ok().filter(|n| n.is_finite());
    let fraction = if let Some(pct) = value.strip_suffix('%') {
        number(pct).map(|n| n / 100.0)
    } else {
        number(value)
    };
    fraction.is_some_and(|n| (0.0..=1.0).contains(&n))
}

/// The `[options]` string values, validated per key (the upstream
/// grammar's own closed sets).
fn parse_option(
    key: &str,
    value: &RawValue,
    line: usize,
    spacing_scale: &mut RsxSpacingScale,
    notes: &mut Vec<RsxNote>,
) -> Result<(), RsxError> {
    let RawValue::Str(value) = value else {
        return Err(RsxError::BadValue {
            line,
            value: value.to_string(),
        });
    };
    match key {
        "spacing-scale" => {
            *spacing_scale = match value.as_str() {
                "px" => RsxSpacingScale::Px,
                "tailwind" => RsxSpacingScale::Tailwind,
                other => {
                    return Err(RsxError::BadSpacingScale {
                        line,
                        value: other.to_string(),
                    });
                }
            };
            Ok(())
        }
        "preset" => {
            if value != "gpui-component" {
                return Err(RsxError::BadPreset {
                    line,
                    value: value.clone(),
                });
            }
            notes.push(RsxNote {
                line,
                name: key.to_string(),
                kind: RsxNoteKind::RuntimePreset,
            });
            Ok(())
        }
        // The runtime wiring options: named, validated as strings, and
        // noted — they are codegen concerns, not theme content.
        "context" | "gpui-path" | "dynamic-runtime" => {
            notes.push(RsxNote {
                line,
                name: key.to_string(),
                kind: RsxNoteKind::RuntimeOption,
            });
            Ok(())
        }
        other => Err(RsxError::UnknownOption {
            line,
            option: other.to_string(),
        }),
    }
}

/// Collect one source line into an entry (or an option mutation), the
/// upstream grammar's own validation order: section header, `=`,
/// key shape, duplicate check, value form, per-section value grammar.
fn collect(
    source: &str,
    section: &mut Option<Section>,
    seen: &mut Vec<(String, String)>,
    spacing_scale: &mut RsxSpacingScale,
    notes: &mut Vec<RsxNote>,
) -> Result<Vec<Entry>, RsxError> {
    let mut entries = Vec::new();
    for (index, raw_line) in source.lines().enumerate() {
        let line_no = index + 1;
        let line = strip_comment(raw_line).trim();
        if line.is_empty() {
            continue;
        }
        if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            let name = name.trim();
            let Some(section_kind) = Section::from_header(name) else {
                return Err(RsxError::UnknownSection {
                    line: line_no,
                    section: name.to_string(),
                });
            };
            *section = Some(section_kind);
            continue;
        }
        let Some(section_kind) = *section else {
            let (key, _) = line
                .split_once('=')
                .ok_or(RsxError::MissingEquals { line: line_no })?;
            return Err(RsxError::KeyOutsideSection {
                line: line_no,
                key: key.trim().trim_matches('"').to_string(),
            });
        };
        let (key, value) = line
            .split_once('=')
            .ok_or(RsxError::MissingEquals { line: line_no })?;
        let key = key.trim().trim_matches('"').to_string();
        if key.is_empty()
            || !key
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        {
            return Err(RsxError::InvalidKey { line: line_no, key });
        }
        if seen.contains(&(section_name(section_kind), key.clone())) {
            return Err(RsxError::DuplicateKey {
                line: line_no,
                section: section_name(section_kind),
                key,
            });
        }
        seen.push((section_name(section_kind), key.clone()));
        let value = parse_value(value.trim(), line_no)?;
        if section_kind == Section::Options {
            parse_option(&key, &value, line_no, spacing_scale, notes)?;
            continue;
        }
        validate_entry(section_kind, &key, &value, line_no)?;
        entries.push(Entry {
            line: line_no,
            section: section_kind,
            key,
            value,
        });
    }
    Ok(entries)
}

/// The section's wire name (the error messages' and the seen-set's).
fn section_name(section: Section) -> String {
    match section {
        Section::Options => "options",
        Section::Colors => "colors",
        Section::Radius => "radius",
        Section::Spacing => "spacing",
        Section::TextSize => "text-size",
        Section::LineHeight => "line-height",
        Section::Opacity => "opacity",
        Section::Shadow => "shadow",
    }
    .to_string()
}

/// The per-section value grammar (the upstream validation, mirrored):
/// colours are hex, lengths are px/rem, opacities are 0..=1, shadows
/// are expressions ONLY; the other sections take both forms (an
/// expression rides the note channel, never the sheet).
fn validate_entry(
    section: Section,
    key: &str,
    value: &RawValue,
    line: usize,
) -> Result<(), RsxError> {
    match section {
        Section::Colors => match value {
            RawValue::Str(v) if parse_color(v) => Ok(()),
            RawValue::Str(v) => Err(RsxError::BadColor {
                line,
                name: key.to_string(),
                value: v.clone(),
            }),
            RawValue::Expr(_) => Ok(()),
        },
        Section::Radius | Section::Spacing | Section::TextSize | Section::LineHeight => match value
        {
            RawValue::Str(v) if parse_length(v) => Ok(()),
            RawValue::Str(v) => Err(RsxError::BadLength {
                line,
                name: key.to_string(),
                value: v.clone(),
            }),
            RawValue::Expr(_) => Ok(()),
        },
        Section::Opacity => match value {
            RawValue::Str(v) if parse_opacity(v) => Ok(()),
            RawValue::Str(v) => Err(RsxError::BadOpacity {
                line,
                name: key.to_string(),
                value: v.clone(),
            }),
            RawValue::Expr(_) => Ok(()),
        },
        Section::Shadow => match value {
            RawValue::Expr(_) => Ok(()),
            RawValue::Str(_) => Err(RsxError::ShadowNeedsExpression {
                line,
                name: key.to_string(),
            }),
        },
        Section::Options => Ok(()),
    }
}

// ---------------------------------------------------------------------------
// The mapper (the namespace map + the note channel)
// ---------------------------------------------------------------------------

/// Map the collected entries onto the sheet and the note channel, in
/// source order. The `[text-size]` names ride first so a
/// `[line-height]` entry that shares one pairs onto it.
fn map(entries: &[Entry], notes: &mut Vec<RsxNote>) -> String {
    let text_sizes: Vec<&str> = entries
        .iter()
        .filter(|entry| entry.section == Section::TextSize && entry.value.is_static())
        .map(|entry| entry.key.as_str())
        .collect();
    let mut css = String::from("@theme {\n");
    for entry in entries {
        let RawValue::Str(value) = &entry.value else {
            notes.push(RsxNote {
                line: entry.line,
                name: entry.key.clone(),
                kind: RsxNoteKind::RustExpression,
            });
            continue;
        };
        let line = match entry.section {
            Section::Colors => format!("  --color-{}: {value};\n", entry.key),
            Section::Radius => format!("  --radius-{}: {value};\n", entry.key),
            Section::Spacing => {
                notes.push(RsxNote {
                    line: entry.line,
                    name: entry.key.clone(),
                    kind: RsxNoteKind::NamedSpacing,
                });
                format!("  --spacing-{}: {value};\n", entry.key)
            }
            Section::TextSize => format!("  --text-{}: {value};\n", entry.key),
            Section::LineHeight => {
                if text_sizes.contains(&entry.key.as_str()) {
                    format!("  --text-{}--line-height: {value};\n", entry.key)
                } else {
                    format!("  --leading-{}: {value};\n", entry.key)
                }
            }
            Section::Opacity => {
                notes.push(RsxNote {
                    line: entry.line,
                    name: entry.key.clone(),
                    kind: RsxNoteKind::NamedOpacity,
                });
                continue;
            }
            // A shadow string refuses at collect time; an expression
            // already noted above — nothing reaches here.
            Section::Shadow | Section::Options => continue,
        };
        css.push_str(&line);
    }
    css.push_str("}\n");
    css
}

impl RawValue {
    /// True when the value is the static (string) form.
    fn is_static(&self) -> bool {
        matches!(self, Self::Str(_))
    }
}

/// Import a gpui-rsx `rsx.toml` onto the aeolus theme surface (plan §5's
/// config-bridge bullet). The grammar is the upstream one, mirrored; the
/// mappings ride the v4 namespace map; what cannot map rides the note
/// channel; every refusal is a typed, line-numbered [`RsxError`].
///
/// # Errors
/// The grammar's own failure modes — unknown section, malformed key or
/// value, duplicate key, unknown option, a value outside its section's
/// grammar — each typed and line-numbered.
pub fn import_rsx(source: &str) -> Result<RsxImport, RsxError> {
    let mut section = None;
    let mut seen = Vec::new();
    let mut spacing_scale = RsxSpacingScale::Tailwind;
    let mut notes = Vec::new();
    let entries = collect(
        source,
        &mut section,
        &mut seen,
        &mut spacing_scale,
        &mut notes,
    )?;
    let css = map(&entries, &mut notes);
    Ok(RsxImport {
        css,
        spacing_scale,
        notes,
    })
}
