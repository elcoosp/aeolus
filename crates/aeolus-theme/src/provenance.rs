//! The token-provenance face (the goto-definition half's source): WHERE
//! a theme token's declaration lives in its own CSS. The audited reader
//! answers *what* a token resolves to ([`TokenTable`]); this module
//! answers *where* — the line, the UTF-16 column and the key's own
//! width of the `--color-{name}` (or bare `--{name}`) declaration the
//! source's light sheet carries, computed over the SAME text
//! [`ThemeSource::from_css`](crate::ThemeSource::from_css) parsed.
//!
//! The scan's honesty contract, the no-silent-failures rule applied to
//! positions: a token the source does not declare (a seeded palette
//! name) answers no origin — never a guessed span; a key mentioned in a
//! comment, a quoted string or a `var()` reference is not a
//! declaration; the prefixed `--color-{name}` form wins over the bare
//! shadcn `--{name}` form (the reader's own [`lookup
//! order`](aeolus_core::TailwindTheme::color)); and the dark sheet's
//! overrides ride the light site (the re-parsed variant re-answers the
//! light names — the light declaration is the canonical site).
//!
//! The built-in palettes ([`ThemeSource::shadcn`](crate::ThemeSource::shadcn)
//! and its dark twin) carry NO origins: they are the compiler's own
//! data — no file to jump to, no origin answered.
//!
//! Positions ride the LSP's own column model (zero-based line,
//! zero-based UTF-16 code-unit column) so a wire consumer relays them
//! verbatim — the theme source is the authority for its own positions,
//! and no host re-derives the model.

use std::collections::BTreeMap;

/// One token's declaration site in its source CSS: the zero-based
/// line, the zero-based UTF-16 code-unit column the key starts at, and
/// the key's own width in UTF-16 code units (the editor's jump
/// selection). The key never spans lines, so the site's end position is
/// the same line at `character + length`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TokenOrigin {
    /// The zero-based line the declaration rides.
    pub line: u32,
    /// The zero-based UTF-16 code-unit column the key starts at.
    pub character: u32,
    /// The key's own width in UTF-16 code units (`--color-primary` is
    /// 15; the bare forms are their own length).
    pub length: u32,
}

/// The token origins one theme source's light sheet carries: the name →
/// site map the scan built, keyed by the reader's own token names (the
/// single source of truth — the scan invents no names). Empty for the
/// built-in palettes. `BTreeMap` the Debug/clone determinism the
/// equality pins ride.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TokenProvenance {
    map: BTreeMap<String, TokenOrigin>,
}

impl TokenProvenance {
    /// The origin one token name answers, `None` the honest dark (a
    /// token the sheet does not declare — the seeded palette names).
    pub fn origin(&self, name: &str) -> Option<&TokenOrigin> {
        self.map.get(name)
    }

    /// How many origins the scan recorded.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// True when the source carries no origins (the built-in palettes).
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Record one origin. A name the scan sees twice keeps the FIRST
    /// declaration (CSS custom properties resolve same-key conflicts by
    /// source order — the first binding is the site a reader lands on).
    fn record(&mut self, name: String, origin: TokenOrigin) {
        self.map.entry(name).or_insert(origin);
    }
}

/// The character ranges a key must NOT be read inside: the block
/// comments and the quoted strings the CSS carries (the
/// `@source` scan's strip walk's own exclusions, kept offset-true —
/// stripping would move every position).
fn excluded_ranges(css: &str) -> Vec<(usize, usize)> {
    let bytes = css.as_bytes();
    let mut out = Vec::new();
    let mut i = 0usize;
    let mut in_comment = false;
    let mut in_string: Option<u8> = None;
    let mut range_start = 0usize;
    while i < bytes.len() {
        if in_comment {
            if bytes[i] == b'*' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                out.push((range_start, i + 2));
                in_comment = false;
                i += 2;
            } else {
                i += 1;
            }
        } else if let Some(quote) = in_string {
            if bytes[i] == b'\\' {
                // The escape consumes the next char verbatim (the
                // strip walk's own rule) — a quoted `\"` is content.
                i = (i + 2).min(bytes.len());
            } else if bytes[i] == quote {
                out.push((range_start, i + 1));
                in_string = None;
                i += 1;
            } else {
                i += 1;
            }
        } else if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
            range_start = i;
            in_comment = true;
            i += 2;
        } else if bytes[i] == b'"' || bytes[i] == b'\'' {
            range_start = i;
            in_string = Some(bytes[i]);
            i += 1;
        } else {
            i += 1;
        }
    }
    // An unterminated comment or string excludes to EOF (the reader
    // refuses such a file anyway; the scan stays conservative either
    // way — never a position inside an unterminated form).
    if in_comment || in_string.is_some() {
        out.push((range_start, bytes.len()));
    }
    out
}

fn is_excluded(at: usize, excluded: &[(usize, usize)]) -> bool {
    excluded.iter().any(|&(start, end)| start <= at && at < end)
}

/// The declaration one key names: the first `--key:` occurrence the
/// boundary rules accept. The key must open a declaration (start of
/// text, whitespace, `;` or `{` before it) and carry its colon (the
/// declaration's own `:`, optional horizontal whitespace between) — a
/// longer key (`--color-primary` inside `--color-primary-foreground`),
/// a `var()` reference (`var(--color-primary)`) and any excluded range
/// ride on as non-declarations.
fn find_declaration(css: &str, key: &str, excluded: &[(usize, usize)]) -> Option<TokenOrigin> {
    let mut from = 0usize;
    while let Some(relative) = css[from..].find(key) {
        let at = from + relative;
        from = at + 1;
        if is_excluded(at, excluded) {
            continue;
        }
        let accepted_before = at == 0
            || css[..at]
                .chars()
                .next_back()
                .is_some_and(|c| c.is_whitespace() || c == ';' || c == '{');
        if !accepted_before {
            continue;
        }
        let after = css[at + key.len()..].trim_start_matches([' ', '\t']);
        if !after.starts_with(':') {
            continue;
        }
        return Some(origin_at(css, at, key.len()));
    }
    None
}

/// The site one byte offset rides: zero-based line, zero-based UTF-16
/// column, the key's own UTF-16 width.
fn origin_at(css: &str, at: usize, key_len: usize) -> TokenOrigin {
    let head = &css[..at];
    let line = head.matches('\n').count() as u32;
    let line_start = head.rfind('\n').map_or(0, |i| i + 1);
    let character = css[line_start..at]
        .chars()
        .map(|c| c.len_utf16() as u32)
        .sum();
    let length = css[at..at + key_len]
        .chars()
        .map(|c| c.len_utf16() as u32)
        .sum();
    TokenOrigin {
        line,
        character,
        length,
    }
}

/// The scan: every name the reader's own colour surface carries, looked
/// up through its two key forms (`--color-{name}` first — the reader's
/// own lookup order — then the bare `--{name}`), declared on the light
/// sheet only (the dark variant re-answers the light names; the light
/// declaration is the canonical site). A name the text does not declare
/// records nothing — the honest dark.
pub(crate) fn scan(css: &str, names: &[&str]) -> TokenProvenance {
    let excluded = excluded_ranges(css);
    let mut out = TokenProvenance::default();
    for name in names {
        for key in [format!("--color-{name}"), format!("--{name}")] {
            if let Some(origin) = find_declaration(css, &key, &excluded) {
                out.record(name.to_string(), origin);
                break;
            }
        }
    }
    out
}
