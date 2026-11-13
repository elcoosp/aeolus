//! The project-build seam (architecture plan §2's approach C —
//! "whole-project scan + one generated file: scan sources, emit one
//! global artifact, dedupe globally" — and the P6 row's `cargo aeolus
//! build`): many source files in, ONE stylesheet out.
//!
//! ```text
//! &[SourceFile] ─▶ scan(...)          the ClassSource extraction seam
//!                  │                  (built-in: the RustStrings
//!                  │                  string-literal walk)
//!                  ▼
//!               Scanned               every candidate hit, with its
//!                  │                  file/line/col/offset location
//!                  ▼
//!               build(&scanned, ...)  dedupe → per-unique-candidate
//!                  │                  emit → registry-ordered rules
//!                  │                  → theme vars → locations on
//!                  ▼                  every diagnostic
//!               BuildArtifact         `Display` renders the stylesheet
//! ```
//!
//! The three stages mirror the approach C row exactly:
//!
//! * **SCAN** — the [`ClassSource`] seam answers "what class strings
//!   does this file name?" for one file; the built-in [`RustStrings`]
//!   walks the quoted/raw string literals (the SAME walk the CLI's
//!   `check` linter rides, lifted here so one implementation answers
//!   both — the keyword-alignment precedent: the seam is the single
//!   source, the consumers stop hand-rolling). Extraction is
//!   deliberately loose (a class-shaped pre-filter only); the compile
//!   walk's own typed diagnostics are the precise filter — a prose
//!   literal that sneaks past returns as an honest
//!   [`BuildDiagnostic`], never as a wrong rule.
//! * **DEDUPE** — one candidate string compiles ONCE no matter how
//!   many files (or how many lines of one file) say it; identical
//!   rules from different candidates collapse by `(selector, media,
//!   body)` identity. The result is one artifact for the whole
//!   project, not one per file — the approach C row's "dedupe
//!   globally".
//! * **EMIT** — each unique candidate renders through the SAME CSS
//!   arm the single-string path uses ([`crate::emit_css`]/
//!   [`crate::emit_css_theme`]; the custom-authoring seam rides
//!   [`build_with`]) — the artifact is differential with the
//!   per-string sheets, byte-equal rules, only the global walk added.
//!   The rule ORDER is the registry's canonical progression (the
//!   [`crate::sort`] seam's own walk over the candidates' join — the
//!   §5.5 tooling-parity line applied globally), because a stylesheet's
//!   cascade reads rule order: `p-4` before `px-8` is the semantics,
//!   not a formatting choice.
//!
//! The theme half: with a [`TokenTable`] supplied the artifact carries
//! a self-contained vars block — `:root` holds every token's light
//! value, `.dark` the dark overrides (the shadcn/web redefinition
//! convention, so the rules' `var(--color-…)` references resolve
//! against the artifact ITSELF, the same file a web Tailwind v4 project
//! ships). Without a table the colour tokens stay typed boundaries per
//! location, the honest default [`crate::compile`] has.
//!
//! The diagnostics are per-LOCATION: one candidate's typed
//! [`Diagnostic`](crate::Diagnostic) reports at EVERY file/line that
//! names it (the dedupe must not hide where a miss lives — the editor
//! squiggle needs the site, not the string).

use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;

use aeolus_syntax::parse_class_string;
use aeolus_theme::TokenTable;

use crate::css::CssAt;
use crate::custom::Customs;
use crate::lower::rgba_text;
use crate::sort::{SortKey, sort_with};
use crate::{Config, CssRule, Diagnostic, emit_css_theme_with, emit_css_with};

// ---------------------------------------------------------------------------
// The extraction seam
// ---------------------------------------------------------------------------

/// One source file the scan walks: a NAME the diagnostics attribute to
/// (the path as the operator passed it) plus the text. The library
/// stays I/O-free — the CLI (and a host embedder) reads the disk and
/// hands the bytes over.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceFile<'a> {
    /// The file's name as the diagnostics render it (`src/view.rs`).
    pub name: String,
    /// The file's full text.
    pub text: &'a str,
}

impl<'a> SourceFile<'a> {
    /// Pair a file name with its text.
    pub fn new(name: impl Into<String>, text: &'a str) -> Self {
        Self {
            name: name.into(),
            text,
        }
    }
}

/// One extracted candidate: the literal's text plus the byte offset of
/// its CONTENT inside the file (first byte after the opening quote).
/// The [`scan`] walk turns offsets into the line/col pair the
/// diagnostics render.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScanHit {
    /// The literal's text, trimmed.
    pub text: String,
    /// The content's byte offset inside the file's text.
    pub offset: usize,
}

/// The extraction strategy seam (approach C's "scan sources"): where
/// class strings come from in one file. The built-in [`RustStrings`]
/// walks the string literals; a host plugs its own shapes (a
/// template dialect, a config format) by implementing the one method.
pub trait ClassSource {
    /// Every candidate class string one file names, in source order.
    fn scan(&self, file: &SourceFile<'_>) -> Vec<ScanHit>;
}

/// The built-in extractor: the quoted-string walk the CLI's `check`
/// linter has ridden since v0.7, lifted into the seam so one
/// implementation answers both consumers. Double-quoted and
/// single-quoted single-line literals plus the Rust raw-string forms
/// (`r"…"`, `r#"…"#` — the forms a `tw!` call site or an HTML
/// attribute use); everything else is prose and rides out.
///
/// The pre-filter is the documented "looks like Tailwind" heuristic:
/// a literal passes when at least one whitespace-separated token
/// carries a `-` or a `:`. It is deliberately LOOSE — prose with a
/// dash gets through, and the compile walk's typed diagnostics report
/// it per location ([`build`]'s contract) instead of a guessed
/// second filter here.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RustStrings;

impl ClassSource for RustStrings {
    fn scan(&self, file: &SourceFile<'_>) -> Vec<ScanHit> {
        scan_rust_strings(file.text)
    }
}

/// The byte walk: every quoted literal with one dash/colon token, in
/// source order, with content offsets. Public because it is a SEAM
/// with two consumers — [`RustStrings`]' trait impl and the CLI's
/// `check` linter (the documented extractor, one implementation now).
pub fn scan_rust_strings(source: &str) -> Vec<ScanHit> {
    let mut out = Vec::new();
    let bytes = source.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'"' | b'\'' => {
                let quote = bytes[i];
                let start = i + 1;
                let mut j = start;
                while j < bytes.len() && bytes[j] != quote && bytes[j] != b'\n' {
                    j += 1;
                }
                if j < bytes.len() && bytes[j] == quote {
                    push_hit(&mut out, source, start, j);
                    i = j + 1;
                    continue;
                }
                i = start.max(j);
            }
            b'r' if i + 1 < bytes.len() && bytes[i + 1] == b'"' => {
                // Rust raw string: r"…" or r#"…"#.
                let mut hashes = 0usize;
                let mut k = i + 1;
                while k < bytes.len() && bytes[k] == b'#' {
                    hashes += 1;
                    k += 1;
                }
                if k >= bytes.len() || bytes[k] != b'"' {
                    i += 1;
                    continue;
                }
                let body_start = k + 1;
                let mut needle = String::from("\"");
                for _ in 0..hashes {
                    needle.push('#');
                }
                if let Some(end) = source[body_start..].find(&needle) {
                    push_hit(&mut out, source, body_start, body_start + end);
                    i = body_start + end + needle.len();
                    continue;
                }
                i = body_start;
            }
            _ => i += 1,
        }
    }
    out
}

/// The shared post-walk filter: trim, drop empties, keep the literals
/// whose tokens look like Tailwind (one `-` or `:` somewhere). The
/// offset pins the CONTENT start.
fn push_hit(out: &mut Vec<ScanHit>, source: &str, start: usize, end: usize) {
    let raw = &source[start..end];
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return;
    }
    // Cheap pre-filter: a class string essentially always carries at
    // least one token with a `-` (utility-value) or `:` (variant).
    // Prose rarely passes; the compile walk's diagnostics do the
    // precise work.
    if !trimmed
        .split_ascii_whitespace()
        .any(|t| t.contains('-') || t.contains(':'))
    {
        return;
    }
    // The content offset points at the trimmed text (the location the
    // diagnostics render is the class text, not the quote).
    let lead = raw.len() - raw.trim_start().len();
    out.push(ScanHit {
        text: trimmed.to_string(),
        offset: start + lead,
    });
}

// ---------------------------------------------------------------------------
// The scan result
// ---------------------------------------------------------------------------

/// One candidate hit's resolved location: the file it sits in, the
/// one-based line and column of the literal's content, and the byte
/// offset inside the file's own text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Located {
    /// The file's name (the [`SourceFile::name`] the operator passed).
    pub file: String,
    /// The literal's text (the candidate class string).
    pub text: String,
    /// The content's byte offset inside the file's own text.
    pub offset: usize,
    /// The one-based line of the content.
    pub line: usize,
    /// The one-based column (byte) of the content.
    pub col: usize,
}

/// The scan result: every candidate hit across every file, in scan
/// order (file order, then in-file offset order). Deterministic — the
/// artifact's first-seen ties ride this order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Scanned {
    /// Every hit, scan order.
    pub hits: Vec<Located>,
}

impl Scanned {
    /// How many candidate hits the scan produced.
    pub fn len(&self) -> usize {
        self.hits.len()
    }

    /// True when no file named any candidate.
    pub fn is_empty(&self) -> bool {
        self.hits.is_empty()
    }
}

/// Scan every file through one extraction strategy: candidates in, with
/// their file/line/col/offset locations resolved. The walk is total on
/// any text; a file that names nothing scans to nothing.
pub fn scan(files: &[SourceFile<'_>], extractor: &impl ClassSource) -> Scanned {
    #[cfg(feature = "tracing")]
    tracing::debug!(files = files.len(), "scan: the extraction walk");
    let mut hits = Vec::new();
    for file in files {
        #[cfg(feature = "tracing")]
        let _span = tracing::debug_span!("aeolus_scan_file", file = file.name.as_str()).entered();
        for hit in extractor.scan(file) {
            let (line, col) = line_col(file.text, hit.offset);
            hits.push(Located {
                file: file.name.clone(),
                text: hit.text,
                offset: hit.offset,
                line,
                col,
            });
        }
    }
    #[cfg(feature = "tracing")]
    tracing::debug!(hits = hits.len(), "scan: extraction done");
    Scanned { hits }
}

/// The one-based `(line, col)` of a byte offset inside a text (the
/// column counts BYTES — the lexer's own span convention).
fn line_col(text: &str, offset: usize) -> (usize, usize) {
    let before = &text[..offset.min(text.len())];
    let line = 1 + before.bytes().filter(|b| *b == b'\n').count();
    let line_start = before.rfind('\n').map_or(0, |pos| pos + 1);
    (line, offset - line_start + 1)
}

// ---------------------------------------------------------------------------
// The artifact
// ---------------------------------------------------------------------------

/// One theme token's rendered var half: the `--color-{name}` entry the
/// rules' `var(--color-…)` references resolve against. `dark` carries
/// the `.dark` redefinition when the source defines one (the
/// shadcn/web convention: the SAME var name, redefined per mode).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ThemeVar {
    /// The token name (`primary`, `red-500`) — the var renders as
    /// `--color-{name}`.
    pub name: String,
    /// The light value text (the emitter's own rgb form).
    pub light: String,
    /// The dark value text, when the source's dark variant overrides
    /// the token.
    pub dark: Option<String>,
}

/// One candidate's typed compile diagnostic, at one of the locations
/// that names it (the dedupe compiles per unique string; the report
/// rides every site).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildDiagnostic {
    /// Where the offending candidate sits.
    pub location: Located,
    /// The typed report — the same spanned, coded, helped
    /// [`Diagnostic`](crate::Diagnostic) the per-string walks produce
    /// (its span indexes the candidate's own text).
    pub diagnostic: Diagnostic,
}

impl fmt::Display for BuildDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}:{}: {} [{}]",
            self.location.file,
            self.location.line,
            self.location.col,
            self.diagnostic,
            self.diagnostic.kind.code()
        )
    }
}

/// The global artifact (approach C's "one generated file"): the
/// self-contained theme vars block plus the globally-deduped,
/// registry-ordered rules and the honest per-location diagnostics.
/// `Display` renders the stylesheet — the vars blocks first (`:root`
/// then `.dark`), then the rules (consecutive same-at-rule rules
/// group into one block, the CSS arm's own contract).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BuildArtifact {
    /// The theme vars (`:root` light values, `.dark` overrides) —
    /// empty when no [`TokenTable`] rides.
    pub vars: Vec<ThemeVar>,
    /// One rule per lowered class, GLOBALLY deduped by
    /// `(selector, media, body)` identity, ordered by the registry's
    /// canonical progression.
    pub rules: Vec<CssRule>,
    /// One entry per (location, offending class), scan order — the
    /// no-silent-failures rule applied to the whole project.
    pub diagnostics: Vec<BuildDiagnostic>,
}

impl fmt::Display for BuildArtifact {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.vars.is_empty() {
            writeln!(f, ":root {{")?;
            for var in &self.vars {
                writeln!(f, "  --color-{}: {};", var.name, var.light)?;
            }
            writeln!(f, "}}")?;
            let dark: Vec<&ThemeVar> = self.vars.iter().filter(|v| v.dark.is_some()).collect();
            if !dark.is_empty() {
                writeln!(f, ".dark {{")?;
                for var in dark {
                    writeln!(
                        f,
                        "  --color-{}: {};",
                        var.name,
                        var.dark.as_deref().unwrap_or_default()
                    )?;
                }
                writeln!(f, "}}")?;
            }
        }
        let mut i = 0;
        while i < self.rules.len() {
            match self.rules[i].media {
                None => {
                    write_rule(f, &self.rules[i], "")?;
                    i += 1;
                }
                Some(at) => {
                    writeln!(f, "{} {{", at.text())?;
                    let media = self.rules[i].media;
                    while i < self.rules.len() && self.rules[i].media == media {
                        write_rule(f, &self.rules[i], "  ")?;
                        i += 1;
                    }
                    writeln!(f, "}}")?;
                }
            }
        }
        Ok(())
    }
}

fn write_rule(f: &mut fmt::Formatter<'_>, rule: &CssRule, indent: &str) -> fmt::Result {
    writeln!(f, "{indent}{} {{", rule.selector)?;
    for line in rule.body.lines() {
        writeln!(f, "{indent}  {line}")?;
    }
    writeln!(f, "{indent}}}")
}

/// Build the whole-project artifact: dedupe the scan's candidates,
/// compile each unique string once through the CSS arm, order the
/// rules by the registry's canonical progression, and render the
/// theme's vars block when a table rides. Total on any scan — a
/// project with no candidates builds to the empty stylesheet.
///
/// ```no_run
/// use aeolus_compile::{Config, RustStrings, SourceFile, build, scan};
///
/// let files = [SourceFile::new("src/view.rs", "tw!(\"p-4 flex\")")];
/// let scanned = scan(&files, &RustStrings);
/// let artifact = build(&scanned, None, &Config::default());
/// print!("{artifact}"); // the stylesheet
/// for report in &artifact.diagnostics {
///     println!("{report}");
/// }
/// ```
pub fn build(scanned: &Scanned, tokens: Option<&TokenTable>, config: &Config) -> BuildArtifact {
    build_with(scanned, tokens, &Customs::new(), config)
}

/// [`build`] with the custom-authoring seam supplied: registered
/// custom utilities render their rules through the same per-candidate
/// emit and ride the sort seam's registration-order tail — the
/// registry wins a name collision (the [`crate::Customs`] contract).
pub fn build_with(
    scanned: &Scanned,
    tokens: Option<&TokenTable>,
    customs: &Customs,
    config: &Config,
) -> BuildArtifact {
    #[cfg(feature = "tracing")]
    tracing::debug!(
        hits = scanned.hits.len(),
        themed = tokens.is_some(),
        "build: the global walk"
    );

    // Stage 1 — the global dedupe, first-seen order (the scan order's
    // own determinism decides the ties).
    let mut index: HashSet<&str> = HashSet::new();
    let mut candidates: Vec<&Located> = Vec::new();
    for located in &scanned.hits {
        if index.insert(located.text.as_str()) {
            candidates.push(located);
        }
    }

    // Stage 2 — the rule order: ONE sort walk over the candidates'
    // join gives every class its registry-progression position (the
    // §5.5 canonical order, applied globally); a candidate's key is
    // its head — the minimum `(bucket, row)` among its own classes
    // (the lexer's own split: bracket groups survive). The sort's own
    // diagnostics are the per-candidate emit walks' business — the
    // ordering only consumes the keys.
    let joined = candidates
        .iter()
        .map(|located| located.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    let mut pos_of_class: HashMap<&str, (u8, usize)> = HashMap::new();
    // The sorted walk lives to the end of the function: the position
    // map's keys borrow the classes' texts.
    let sorted = sort_with(&joined, customs).ok();
    if let Some(sorted) = sorted.as_ref()
        && !candidates.is_empty()
    {
        for class in &sorted.classes {
            pos_of_class.entry(class.raw.as_str()).or_insert_with(|| {
                if class.known {
                    (class.key.bucket, class.key.row)
                } else {
                    (SortKey::UNSORTABLE.bucket, SortKey::UNSORTABLE.row)
                }
            });
        }
    }
    // A class the join's sort did not see cannot happen (the join IS
    // the candidates' own classes); a missing entry stays the tail.
    let mut keys: Vec<((u8, usize), usize, &Located)> = Vec::with_capacity(candidates.len());
    for (seen, located) in candidates.iter().enumerate() {
        let key = parse_class_string(&located.text)
            .0
            .iter()
            .filter_map(|class| pos_of_class.get(class.raw).copied())
            .min()
            .unwrap_or((SortKey::UNSORTABLE.bucket, SortKey::UNSORTABLE.row));
        keys.push((key, seen, *located));
    }
    keys.sort_by_key(|(key, seen, _)| (*key, *seen));

    // Stage 3 — the per-unique emit (the CSS arm's own walk), the
    // global rule dedupe by (selector, media, body) identity, and the
    // diagnostics at every location that names the candidate.
    let mut artifact = BuildArtifact::default();
    let mut seen_rules: HashSet<(String, Option<String>, String)> = HashSet::new();
    for (_, _, first) in &keys {
        let locations: Vec<Located> = scanned
            .hits
            .iter()
            .filter(|located| located.text == first.text)
            .cloned()
            .collect();
        let sheet = match tokens {
            Some(tokens) => emit_css_theme_with(first.text.as_str(), tokens, customs, config),
            None => emit_css_with(first.text.as_str(), customs, config),
        };
        let Ok(sheet) = sheet else {
            continue; // only the empty input hard-fails; the scan's
            // pre-filter already rode empties out
        };
        for rule in sheet.rules {
            let identity = (
                rule.selector.clone(),
                rule.media.map(CssAt::text),
                rule.body.clone(),
            );
            if seen_rules.insert(identity) {
                artifact.rules.push(rule);
            }
        }
        for diagnostic in sheet.diagnostics {
            for location in &locations {
                artifact.diagnostics.push(BuildDiagnostic {
                    location: location.clone(),
                    diagnostic: diagnostic.clone(),
                });
            }
        }
    }

    // Stage 4 — the theme vars block: the table's own half, the light
    // values under `:root`, the dark overrides under `.dark` (the
    // emitter's rgb text forms, the emitter's `--color-` prefix).
    if let Some(tokens) = tokens {
        artifact.vars = theme_vars(tokens);
    }
    #[cfg(feature = "tracing")]
    tracing::debug!(
        rules = artifact.rules.len(),
        diagnostics = artifact.diagnostics.len(),
        vars = artifact.vars.len(),
        "build: artifact assembled"
    );
    artifact
}

/// The vars half: every token the table carries, in id order (the
/// sorted-name order the `Val::Tok` ids ride), light and dark values
/// through the emitter's own rgba text form. The u16 cap is the IR's
/// own token-id budget (`Val::Tok { id: u16 }`) — the surface the ids
/// can name, nothing beyond it exists to render.
fn theme_vars(tokens: &TokenTable) -> Vec<ThemeVar> {
    let cap = usize::from(u16::MAX) + 1;
    let mut out = Vec::with_capacity(tokens.len().min(cap));
    for (id, name) in tokens.names().enumerate().take(cap) {
        let id = id as u16;
        let Some(rgba) = tokens.light_rgba(id) else {
            continue;
        };
        let dark = tokens
            .has_dark(id)
            .then(|| tokens.dark_rgba(id))
            .flatten()
            .map(rgba_text);
        out.push(ThemeVar {
            name: name.to_string(),
            light: rgba_text(rgba),
            dark,
        });
    }
    out
}
