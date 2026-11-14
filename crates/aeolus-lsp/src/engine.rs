//! The headless engine (the LSP crate's core): the document map, the
//! extraction seam through the V1 compile walk, and the two answers —
//! per-location diagnostics with UTF-16 ranges, and the hover
//! resolution keyed by the candidate under the cursor.
//!
//! The engine carries NONE of the wire types: [`EngineDiagnostic`] and
//! [`EngineHover`] are the crate's own shapes, and the server module
//! maps them onto the protocol's JSON. The engine is pure over its
//! document map — the same input answers the same output, gate over
//! gate.

use std::fmt::Write as _;

use aeolus_compile::{
    Config, Diagnostic, ScanHit, compile, compile_theme, emit_css, emit_css_theme,
    scan_rust_strings,
};
use aeolus_ir::Cond;
use aeolus_syntax::parse_class_string;
use aeolus_theme::TokenTable;

use crate::{DocumentMap, EngineDiagnostic, EngineHover, EnginePosition, EngineRange};

// ---------------------------------------------------------------------------
// The engine config
// ---------------------------------------------------------------------------

/// The engine's pipeline configuration — the same knobs the CLI's
/// `--theme` flag and the compile seam's [`Config`] carry. The default
/// rides the built-in shadcn bake (the CLI's own default theme spec),
/// so an editor session answers `bg-primary` the way the CLI does.
#[derive(Clone, Debug)]
pub struct EngineConfig {
    /// The theme pass's table: `Some` — the themed walk
    /// ([`compile_theme`], colour tokens and opacity modifiers lower
    /// through the tiered tables); `None` — the theme-less boundary
    /// (the tokens stay typed refusals, the honest default
    /// [`compile`] has).
    pub tokens: Option<TokenTable>,
    /// The compile seam's own knobs (the spacing strategy, the rem
    /// size) — the engine never forks pipeline semantics.
    pub compile: Config,
}

impl EngineConfig {
    /// A config over an explicit table (the host CSS shape: the same
    /// [`ThemeSource`](aeolus_theme::ThemeSource) construction the CLI's
    /// `--theme <path>` accepts).
    pub fn new(tokens: Option<TokenTable>) -> Self {
        Self {
            tokens,
            compile: Config::default(),
        }
    }
}

impl Default for EngineConfig {
    /// The CLI-parity default: the built-in shadcn bake. The embedded
    /// palette is an internal invariant (the macro's freeze walk rides
    /// the same construction), so the build panics only on drift the
    /// suite pins.
    fn default() -> Self {
        let source =
            aeolus_theme::ThemeSource::shadcn().expect("the built-in shadcn source parses");
        let table = TokenTable::baked(&source).expect("the baked shadcn table builds");
        Self {
            tokens: Some(table),
            compile: Config::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// The engine
// ---------------------------------------------------------------------------

/// The headless LSP engine: the open documents, the pipeline answers,
/// the position conversions. One engine per server session (the wire
/// half owns it); a host embedding the engine directly rides the same
/// two questions without the transport.
#[derive(Debug)]
pub struct AeolusEngine {
    /// The engine's pipeline configuration.
    config: EngineConfig,
    /// The open documents, keyed by their URI as the wire names them.
    documents: DocumentMap,
}

impl AeolusEngine {
    /// An engine over one configuration.
    pub fn new(config: EngineConfig) -> Self {
        Self {
            config,
            documents: DocumentMap::new(),
        }
    }

    /// The engine's configuration (the server's capabilities render
    /// from the same knobs).
    pub fn config(&self) -> &EngineConfig {
        &self.config
    }

    /// Open a document (the wire's didOpen): the text replaces any
    /// previous state for the URI.
    pub fn open(&mut self, uri: &str, text: &str, version: i32) {
        self.documents.insert(
            uri.to_string(),
            crate::Document {
                text: text.to_string(),
                version,
            },
        );
    }

    /// Replace a document's text (the wire's didChange, full-text
    /// sync). An unknown URI opens implicitly — the wire contract
    /// allows a didChange without a matching didOpen from some clients.
    pub fn change(&mut self, uri: &str, text: &str, version: i32) {
        self.documents.insert(
            uri.to_string(),
            crate::Document {
                text: text.to_string(),
                version,
            },
        );
    }

    /// Close a document (the wire's didClose): the state drops, the
    /// answers go empty.
    pub fn close(&mut self, uri: &str) {
        self.documents.remove(uri);
    }

    /// The document's version tag, when it is open.
    pub fn version(&self, uri: &str) -> Option<i32> {
        self.documents.get(uri).map(|doc| doc.version)
    }

    // -----------------------------------------------------------------------
    // The diagnostics answer
    // -----------------------------------------------------------------------

    /// The document's findings: the extraction seam's strings through
    /// the compile walk, every typed diagnostic mapped onto its
    /// byte-true UTF-16 range with its stable code. Deterministic and
    /// pure over the document map; an unknown URI answers empty.
    pub fn analyze(&self, uri: &str) -> Vec<EngineDiagnostic> {
        let Some(doc) = self.documents.get(uri) else {
            return Vec::new();
        };
        let text = doc.text.as_str();
        let mut out = Vec::new();
        for hit in scan_rust_strings(text) {
            // The pipeline answer: the themed walk when the config
            // carries a table, the theme-less boundary otherwise — the
            // same entry points the CLI runs, never a parallel matcher.
            let compiled = match &self.config.tokens {
                Some(table) => compile_theme(hit.text.as_str(), table, &self.config.compile),
                None => compile(hit.text.as_str(), &self.config.compile),
            };
            let Ok(compiled) = compiled else {
                // Only the empty input errors; a trimmed-empty hit is
                // the extractor's own artifact — no findings either way.
                continue;
            };
            for diagnostic in &compiled.diagnostics {
                out.push(engine_diagnostic(text, &hit, diagnostic));
            }
        }
        out
    }

    // -----------------------------------------------------------------------
    // The hover answer
    // -----------------------------------------------------------------------

    /// The hover resolution: the candidate under the cursor, rendered
    /// as markdown. A lowered class renders its folded condition and
    /// the CSS arm's own declaration texts; a refused class renders the
    /// typed refusal (code + help); a cursor outside any candidate
    /// string answers None.
    pub fn hover(&self, uri: &str, line: u32, character: u32) -> Option<EngineHover> {
        let doc = self.documents.get(uri)?;
        let text = doc.text.as_str();
        let offset = offset_of(text, line, character)?;
        let hit = scan_rust_strings(text)
            .into_iter()
            .find(|hit| offset >= hit.offset && offset < hit.offset + hit.text.len())?;
        // The candidate under the cursor: the same lexer walk the
        // compile runs, the candidate's span keyed on the hit text.
        let local = offset.saturating_sub(hit.offset);
        let (candidates, _) = parse_class_string(&hit.text);
        let candidate = candidates
            .iter()
            .find(|candidate| local >= candidate.span.0 && local < candidate.span.1)?;
        let raw = candidate.raw;

        // The single-candidate pipeline answer: the themed walk when the
        // config carries a table, the theme-less boundary otherwise.
        let compiled = match &self.config.tokens {
            Some(table) => compile_theme(raw, table, &self.config.compile),
            None => compile(raw, &self.config.compile),
        };
        let Ok(compiled) = compiled else {
            return None;
        };

        let (line, start_character) = position_of(text, hit.offset + candidate.span.0);
        let (_, end_character) = position_of(text, hit.offset + candidate.span.1);
        let range = EngineRange {
            start: EnginePosition {
                line,
                character: start_character,
            },
            end: EnginePosition {
                line,
                character: end_character,
            },
        };

        if !compiled.diagnostics.is_empty() {
            // The refusal hover: the class, every typed report with its
            // code, every help line — the editor sees WHY.
            let mut contents = String::new();
            let _ = writeln!(contents, "`{raw}`");
            for diagnostic in &compiled.diagnostics {
                let _ = writeln!(contents, "\n- {diagnostic} [`{}`]", diagnostic.kind.code());
                let _ = writeln!(contents, "\n  help: {}", diagnostic.kind.help());
            }
            return Some(EngineHover {
                contents,
                range: Some(range),
            });
        }

        // The lowered hover: the folded condition (the program's own
        // cascade tail — the class's most specific layer) and the CSS
        // arm's own declaration texts — the same rendering the
        // verification stylesheet carries.
        let cond = compiled
            .program
            .layers
            .last()
            .map_or(Cond::Base, |layer| layer.cond);
        let sheet = match &self.config.tokens {
            Some(table) => emit_css_theme(raw, table, &self.config.compile),
            None => emit_css(raw, &self.config.compile),
        };
        let Ok(sheet) = sheet else {
            return None;
        };
        let mut contents = String::new();
        let _ = writeln!(contents, "`{raw}` ({})", cond_label(cond));
        if let Some(rule) = sheet.rules.first() {
            let _ = writeln!(contents, "\n```css");
            for body_line in rule.body.lines() {
                let _ = writeln!(contents, "{body_line}");
            }
            let _ = writeln!(contents, "```");
        }
        for diagnostic in &sheet.diagnostics {
            let _ = writeln!(contents, "\n- {diagnostic}");
        }
        Some(EngineHover {
            contents,
            range: Some(range),
        })
    }
}

// ---------------------------------------------------------------------------
// The position conversions (the LSP's UTF-16 model, owned ONCE here)
// ---------------------------------------------------------------------------

/// Byte offset → `(line, character-UTF16)` for the offset's own
/// position (any partial character clamps to its boundary).
fn position_of(text: &str, offset: usize) -> (u32, u32) {
    let offset = offset.min(text.len());
    // Clamp to a character boundary: the last index at or before the
    // offset that starts a character.
    let safe = text[..=offset]
        .char_indices()
        .map(|(i, _)| i)
        .next_back()
        .unwrap_or(offset);
    let head = &text[..safe];
    let line = head.matches('\n').count() as u32;
    let line_start = head.rfind('\n').map_or(0, |i| i + 1);
    let character = head[line_start..]
        .chars()
        .map(|ch| ch.len_utf16() as u32)
        .sum::<u32>();
    (line, character)
}

/// Byte range → the LSP's half-open range.
fn range_of(text: &str, start: usize, end: usize) -> EngineRange {
    let (line, character) = position_of(text, start);
    let (end_line, end_character) = position_of(text, end);
    EngineRange {
        start: EnginePosition { line, character },
        end: EnginePosition {
            line: end_line,
            character: end_character,
        },
    }
}

/// The protocol's position → byte offset, UTF-16 columns counted per
/// the character model (a column inside a multi-byte character clamps
/// to that character's start; a column past the line's end clamps to
/// the line end; a line past the document answers None).
fn offset_of(text: &str, line: u32, character: u32) -> Option<usize> {
    let mut current_line = 0u32;
    let mut line_start = 0usize;
    while current_line < line {
        let relative = text[line_start..].find('\n')?;
        line_start += relative + 1;
        current_line += 1;
    }
    let line_end = text[line_start..]
        .find('\n')
        .map_or(text.len(), |relative| line_start + relative);
    let mut units = 0u32;
    for (index, ch) in text[line_start..line_end].char_indices() {
        if units >= character {
            return Some(line_start + index);
        }
        units += ch.len_utf16() as u32;
    }
    Some(line_end)
}

// ---------------------------------------------------------------------------
// The diagnostic and hover renderings
// ---------------------------------------------------------------------------

/// Map one pipeline diagnostic onto the wire-free shape: the absolute
/// range (the hit's content offset plus the diagnostic's span), the
/// message with its help line, the stable code.
fn engine_diagnostic(text: &str, hit: &ScanHit, diagnostic: &Diagnostic) -> EngineDiagnostic {
    let (start, end) = diagnostic.span;
    let range = range_of(text, hit.offset + start, hit.offset + end);
    let mut message = String::new();
    let _ = write!(message, "{diagnostic}");
    let _ = write!(message, "\nhelp: {}", diagnostic.kind.help());
    EngineDiagnostic {
        range,
        message,
        code: diagnostic.kind.code().to_string(),
    }
}

/// The condition's label for the hover line (the folded `Cond`'s
/// canonical spelling — the same labels the CLI renders).
fn cond_label(cond: Cond) -> &'static str {
    match cond {
        Cond::Base => "base",
        Cond::Dark => "dark",
        Cond::Hover => "hover",
        Cond::Focus => "focus",
        Cond::FocusVisible => "focus-visible",
        Cond::FocusWithin => "focus-within",
        Cond::Active => "active",
        Cond::Disabled => "disabled",
        Cond::Visited => "visited",
        Cond::Checked => "checked",
        Cond::PlaceholderShown => "placeholder-shown",
        Cond::Selection => "selection",
        Cond::Placeholder => "placeholder",
        Cond::GroupHover => "group-hover",
        Cond::GroupFocus => "group-focus",
        Cond::PeerHover => "peer-hover",
        Cond::PeerFocus => "peer-focus",
        Cond::Bp(_) | Cond::Min(_) => "breakpoint",
        Cond::Max(_) => "max-breakpoint",
        Cond::Container { .. } | Cond::MaxContainer { .. } => "container",
        Cond::Data { .. } => "data probe",
        Cond::Aria { .. } => "aria probe",
        Cond::Has(_) | Cond::GroupHas(_) | Cond::PeerHas(_) => "descendant probe",
        Cond::Structural(_) => "structural",
    }
}
