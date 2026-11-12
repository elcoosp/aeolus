//! The `@source` directive seam (architecture plan §5.4's CSS-first
//! compatibility line, item 4 — the `@theme`/`@utility`/
//! `@custom-variant` trio landed with the theme pass and the
//! custom-authoring seam; the scanner registration is this module).
//!
//! The SAME theme CSS a web Tailwind v4 project uses can register the
//! content the class scan walks — the v4 scanner surface, rendered as
//! data:
//!
//! ```css
//! @source inline("p-4 bg-primary");   /* literal content           */
//! @source "src/views.rs";             /* a file the scan walks     */
//! @source not "src/generated.rs";     /* a file the scan prunes    */
//! ```
//!
//! The parse is typed end to end (the no-silent-failures contract):
//! the three forms land as [`SourceDirective`] entries in source
//! order, and a malformed statement (an unterminated string, an
//! unrecognized payload, the bare `@source;`) is a typed
//! [`ThemeError::Source`] — never a guessed entry, never a quiet skip.
//! Comments are stripped first, so a directive mentioned inside one is
//! not one. The scanner reads STATEMENTS (`@source …;`), the same
//! shape `extract_custom_rules` walks for the custom-authoring seam.

use std::fmt;

use crate::ThemeError;

/// One `@source` directive the theme CSS carries — the v4 scanner's
/// registration surface, as data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SourceDirective {
    /// `@source inline("p-4 bg-primary")` — literal content registered
    /// as if scanned from a file (the docs/demo inline form).
    Inline(String),
    /// `@source "path"` — a file the scan walks; `@source not "path"`
    /// (`negated`) — a file the scan prunes. Paths are relative to the
    /// theme CSS's own directory (the web scanner's semantics).
    Path {
        /// The path, verbatim.
        path: String,
        /// True for the `not` form.
        negated: bool,
    },
}

/// Collect the `@source` directives a theme CSS carries, in source
/// order. Only the `@source` statements count — comments are stripped,
/// and the `@theme`/`:root`/`.dark` bodies carry custom properties,
/// never registrations.
///
/// # Errors
/// [`ThemeError::Source`] per malformed statement: an unterminated
/// string, a missing statement terminator, an unrecognized payload or
/// the bare `@source;` form — the scan never guesses.
pub fn source_directives(css: &str) -> Result<Vec<SourceDirective>, ThemeError> {
    let owned = strip_comments(css);
    let mut out = Vec::new();
    let mut rest: &str = &owned;
    while let Some(i) = rest.find("@source") {
        let after = &rest[i + "@source".len()..];
        let (directive, consumed) = parse_statement(after)?;
        if let Some(directive) = directive {
            out.push(directive);
        }
        rest = &after[consumed..];
    }
    Ok(out)
}

/// Parse one `@source` statement body (everything after the keyword).
/// Returns the directive (if any) and how many bytes were consumed —
/// the statement runs to its `;`, the same statement shape
/// `extract_custom_rules` walks (payloads are paths and class lists;
/// a `;` inside one is the same pathological case the audited reader
/// simplifies on).
fn parse_statement(after: &str) -> Result<(Option<SourceDirective>, usize), ThemeError> {
    let fail = |message: &str| {
        Err(ThemeError::Source {
            message: message.to_string(),
        })
    };
    let Some(semi) = after.find(';') else {
        return fail("unterminated @source: the statement never ends");
    };
    let consumed = semi + 1;
    let stmt = after[..semi].trim();
    if stmt.is_empty() {
        return fail("empty @source: the statement registers nothing");
    }
    // The negated form: `not "path"`.
    if let Some(rest) = stmt.strip_prefix("not") {
        let Some((path, _)) = read_quoted(rest.trim_start()) else {
            return fail("unterminated @source: the not-form needs a quoted path");
        };
        return Ok((
            Some(SourceDirective::Path {
                path,
                negated: true,
            }),
            consumed,
        ));
    }
    // The inline form: `inline("…")`.
    if let Some(rest) = stmt.strip_prefix("inline") {
        let rest = rest.trim_start();
        let Some(open_rel) = rest.find('(') else {
            return fail("unrecognized @source: the inline form needs a parenthesized payload");
        };
        let Some((content, _)) = read_quoted(rest[open_rel + 1..].trim_start()) else {
            return fail("unterminated @source: the inline payload needs a quoted string");
        };
        if !rest[open_rel + 1..].trim_start()[content.len() + 2..].starts_with(')') {
            return fail("unterminated @source: the inline payload is missing its ')'");
        }
        return Ok((Some(SourceDirective::Inline(content)), consumed));
    }
    // The path form: `"path"` is the only remaining shape — anything
    // else is unrecognized, and a quoted shape that never closes is
    // the unterminated form (the message names the difference).
    if stmt.starts_with('"') {
        let Some((path, _)) = read_quoted(stmt) else {
            return fail("unterminated @source: the quoted path never closes");
        };
        return Ok((
            Some(SourceDirective::Path {
                path,
                negated: false,
            }),
            consumed,
        ));
    }
    fail("unrecognized @source: expected a quoted path, the not-form or inline(...)")
}

/// Read a double-quoted string; returns the content and the consumed
/// length INCLUDING both quotes. A string that never closes is `None`
/// (the typed error names it).
fn read_quoted(body: &str) -> Option<(String, usize)> {
    let body = body.strip_prefix('"')?;
    let close = body.find('"')?;
    Some((body[..close].to_string(), close + 2))
}

/// Strip `/* … */` comments (non-nested, the CSS form) so a directive
/// mentioned inside one is not one. QUOTED STRINGS ARE PROTECTED — a
/// `/**/*`-shaped glob inside `@source "**/*.rs";` is payload, not a
/// comment opener (the comment rule is a string-aware walk: inside a
/// `"…"` span neither comments nor quotes nest; a backslash escapes
/// the next byte, the CSS string form).
fn strip_comments(css: &str) -> String {
    let mut out = String::with_capacity(css.len());
    let mut rest = css;
    let mut in_string = false;
    'walk: while !rest.is_empty() {
        // One char per step (multi-byte safe): the comment/quote tests
        // are ASCII, the general fallthrough advances by the char's
        // own length.
        let ch = rest.chars().next().expect("non-empty");
        if in_string {
            match ch {
                '\\' => {
                    // The escape consumes the next char verbatim.
                    out.push(ch);
                    let after = &rest[1..];
                    if let Some(next) = after.chars().next() {
                        out.push(next);
                        rest = &after[next.len_utf8()..];
                    } else {
                        rest = after;
                    }
                    continue 'walk;
                }
                '"' => in_string = false,
                _ => {}
            }
            out.push(ch);
            rest = &rest[ch.len_utf8()..];
            continue 'walk;
        }
        if ch == '"' {
            in_string = true;
            out.push('"');
            rest = &rest[1..];
            continue 'walk;
        }
        if ch == '/' && rest[1..].starts_with('*') {
            match rest[2..].find("*/") {
                Some(close) => {
                    rest = &rest[2 + close + 2..];
                    continue 'walk;
                }
                // An unterminated comment swallows the tail (CSS recovery).
                None => return out,
            }
        }
        out.push(ch);
        rest = &rest[ch.len_utf8()..];
    }
    out
}

impl fmt::Display for SourceDirective {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SourceDirective::Inline(content) => write!(f, "@source inline({content:?})"),
            SourceDirective::Path { path, negated } => {
                if *negated {
                    write!(f, "@source not {path:?}")
                } else {
                    write!(f, "@source {path:?}")
                }
            }
        }
    }
}
