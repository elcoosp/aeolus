//! The headless engine (the LSP crate's core): the document map, the
//! extraction seam through the V1 compile walk, and the editor's
//! answers — per-location diagnostics with UTF-16 ranges, the hover
//! resolution keyed by the candidate under the cursor, the completion
//! at the typing edge, the colour swatches, the semantic legend, the
//! colour presentations and the inline hints.
//!
//! The engine carries NONE of the wire types: [`EngineDiagnostic`],
//! [`EngineHover`] and the other answer shapes are the crate's own, and
//! the server module maps them onto the protocol's JSON. The engine is
//! pure over its document map — the same input answers the same output,
//! gate over gate.

use std::collections::HashSet;
use std::fmt::Write as _;

use aeolus_compile::{
    COLOUR_GROUPS, CompileError, Compiled, Config, CssSheet, Diagnostic, DynCache, DynObserver,
    KNOWN_VARIANTS, ScanHit, emit_css, emit_css_theme, scan_rust_strings, sort, suggest_for,
};
use aeolus_data::ValueKind;
use aeolus_data::rows::ROWS;
use aeolus_ir::{Cond, Val};
use aeolus_syntax::parse_class_string;
use aeolus_theme::TokenTable;

use crate::{
    DocumentMap, EngineCodeAction, EngineColour, EngineColourPresentation, EngineCompletion,
    EngineCompletionKind, EngineDiagnostic, EngineHighlight, EngineHover, EngineInlayHint,
    EnginePosition, EngineRange, EngineSemanticKind, EngineSemanticToken, EngineSymbol,
    EngineTextEdit,
};

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
    /// The bounded pool the compile answers ride (the compile crate's
    /// `DynCache`, the leak-pool dedupe): an editor session re-answers
    /// the same strings on every keystroke and re-opens the same
    /// documents across sessions — the cache makes the repeat the CHEAP
    /// path (a hit hands back the same interned slices the first
    /// compile pooled; the identity is the engine's own config, fixed
    /// at construction, so one identity slot spans the session). The
    /// observability face rides [`AeolusEngine::with_observer`]; the
    /// default is observation-free and the answers are byte-identical
    /// (the cache never changes a decision — the compile crate's own
    /// differential contract).
    cache: DynCache,
}

impl AeolusEngine {
    /// An engine over one configuration.
    pub fn new(config: EngineConfig) -> Self {
        Self {
            config,
            documents: DocumentMap::new(),
            cache: DynCache::new(),
        }
    }

    /// The engine with a host-supplied observer — the compile seam's
    /// observability face joined (the `DynCache::with_observer` shape,
    /// chained after `new`: the built-in
    /// [`CacheStats`](aeolus_compile::CacheStats) rides an `Arc` the
    /// host keeps a handle to). The observer sees every decision the
    /// session's pool makes — hit/miss/store under the composed key,
    /// the store event carrying the entry's diagnostic count, the
    /// eviction face's rotation drops beside — one handle spanning the
    /// whole editor session. The observer adds a channel; it never
    /// changes a decision.
    pub fn with_observer(mut self, observer: impl DynObserver + 'static) -> Self {
        self.cache = self.cache.with_observer(observer);
        self
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

    /// The pipeline answer over the session's pool: the themed walk when
    /// the config carries a table, the theme-less boundary otherwise —
    /// the same entry points the CLI runs, never a parallel matcher,
    /// now deduped by the bounded cache (the cache's own differential
    /// contract: a hit hands back the same interned slices and the SAME
    /// diagnostics the miss stored; only the empty input errors, and
    /// errors are never cached).
    fn compiled(&self, raw: &str) -> Result<Compiled, CompileError> {
        match &self.config.tokens {
            Some(table) => self.cache.compile_theme(raw, table, &self.config.compile),
            None => self.cache.compile(raw, &self.config.compile),
        }
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
            // The pipeline answer over the session's pool: the themed
            // walk when the config carries a table, the theme-less
            // boundary otherwise — the same entry points the CLI runs,
            // never a parallel matcher; a re-analyzed document's
            // unchanged strings ride the cache instead of re-lowering.
            let compiled = self.compiled(hit.text.as_str());
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
        // The candidate under the cursor: the same lexer walk the
        // compile runs, the candidate's span keyed on the hit text —
        // the ONE locator the cursor-keyed answers share (the hover,
        // the highlights).
        let (start, end) = candidate_span_at(text, offset)?;
        let raw = &text[start..end];
        let range = range_of(text, start, end);

        // The single-candidate pipeline answer: the themed walk when the
        // config carries a table, the theme-less boundary otherwise.
        let compiled = self.compiled(raw);
        let Ok(compiled) = compiled else {
            return None;
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
        Some(EngineHover {
            contents: lowered_markdown(raw, cond, &sheet),
            range: Some(range),
        })
    }

    // -----------------------------------------------------------------------
    // The completion answer
    // -----------------------------------------------------------------------

    /// The completion for the fragment under the cursor: the candidate
    /// string the extraction seam finds, the whitespace word the cursor
    /// sits in (the word's END counts — completion fires at the typing
    /// edge), the typed-so-far fragment completed from the compiler's
    /// own audited sources. Deterministic and pure over the document
    /// map; an unknown URI, a position outside any string and a cursor
    /// outside any word answer empty.
    ///
    /// Two boundaries the editor half keeps honestly:
    ///
    /// * the extraction seam's prose pre-filter applies (a string with
    ///   no `-`/`:` token is not a class string — the diagnostics' own
    ///   rule), so a completion inside such a string answers empty: the
    ///   popup fires where the pipeline looks, never wider;
    /// * the WORD is not lex-validated — a half-typed `flex-` is not a
    ///   legal candidate yet (the lexer's empty-element rule) and the
    ///   popup must answer while typing; the lexical gate belongs to
    ///   the compile walk, not the suggestion list.
    ///
    /// The fragment model: with no `:` typed, the variant stack AND the
    /// utility root are both in play (both halves answer); after a `:`,
    /// the utility half alone does. An empty utility half (the cursor
    /// right after `hover:`) answers the BARE roots — the complete
    /// classes that need no suffix — never the full thousand-item
    /// class list. A dynamic scale shape (the spacing multiples, the
    /// fractions — every number is a legal spelling, no closed audited
    /// table exists) stays unnamed: the engine suggests only what the
    /// registry's own tables can name exactly, the no-invented-data
    /// rule the pipeline itself keeps.
    pub fn complete(&self, uri: &str, line: u32, character: u32) -> Vec<EngineCompletion> {
        let Some(doc) = self.documents.get(uri) else {
            return Vec::new();
        };
        let text = doc.text.as_str();
        let Some(offset) = offset_of(text, line, character) else {
            return Vec::new();
        };
        // The same extraction seam the hover and the diagnostics walk —
        // the string under the cursor, the cursor's byte offset inside
        // it (the END is inclusive: completion fires at the edge).
        let Some(hit) = scan_rust_strings(text)
            .into_iter()
            .find(|hit| offset >= hit.offset && offset <= hit.offset + hit.text.len())
        else {
            return Vec::new();
        };
        let local = offset.saturating_sub(hit.offset);
        // The word under the cursor: the whitespace-delimited slice the
        // lexer's own segmentation splits on (parse_class_string's
        // whitespace walk) — without its lex-validation (see above).
        let words = hit.text.as_str();
        let start = words[..local.min(words.len())]
            .rfind(char::is_whitespace)
            .map_or(0, |index| index + 1);
        let end = words[local.min(words.len())..]
            .find(char::is_whitespace)
            .map_or(words.len(), |index| local + index);
        let fragment = if local <= end {
            &words[start..local.min(end)]
        } else {
            // The cursor rests in the whitespace AFTER the word: the
            // empty fragment (the next word's first keystroke).
            ""
        };
        completions_for(fragment, self.config.tokens.as_ref())
    }

    // -----------------------------------------------------------------------
    // The colour-swatch answer
    // -----------------------------------------------------------------------

    /// The document's colour swatches: every candidate class whose
    /// program carries a colour-valued declaration, its byte-true range
    /// and the table's resolved light-tier rgba. The walk is the shared
    /// per-candidate compile (`compiled_candidates`), and the
    /// swatch is the FIRST colour declaration in cascade order (the
    /// class's primary colour — a class that writes two colour
    /// longhands paints the one the base layer carries first). The
    /// theme-less config answers empty for every token class (the
    /// honest boundary [`compile`] itself keeps), and the keyword slots
    /// (`transparent`, `currentColor`) stay unnamed — a CSS keyword is
    /// not a swatch. Deterministic and pure over the document map; an
    /// unknown URI answers empty.
    pub fn document_colours(&self, uri: &str) -> Vec<EngineColour> {
        let Some(doc) = self.documents.get(uri) else {
            return Vec::new();
        };
        let text = doc.text.as_str();
        self.compiled_candidates(text, |_raw, span, compiled| {
            let rgba = compiled.program.layers.iter().find_map(|layer| {
                layer
                    .decls
                    .iter()
                    .find_map(|decl| colour_of(decl.val, self.config.tokens.as_ref()))
            })?;
            Some(EngineColour {
                range: range_of(text, span.0, span.1),
                rgba,
            })
        })
    }

    // -----------------------------------------------------------------------
    // The semantic-token answer
    // -----------------------------------------------------------------------

    /// The document's semantic tokens: every candidate class the
    /// extraction seam's strings carry, its byte-true range and its
    /// compile outcome — a clean lower is the legend's `Lowered` (the
    /// wire's `class` type), a typed refusal is `Refused` (the wire's
    /// custom `refused` type). The walk is the shared per-candidate
    /// compile (`compiled_candidates`), the classification the
    /// diagnostics list's emptiness — no parallel matcher, no guessed
    /// shape. The theme-less config answers the same (a refusal is a
    /// refusal either way: the token classes refuse honestly, the
    /// highlighter shows it). Deterministic and pure over the document
    /// map; an unknown URI answers empty, and the answer rides in
    /// document order (the delta encoder's own precondition).
    pub fn semantic_tokens(&self, uri: &str) -> Vec<EngineSemanticToken> {
        let Some(doc) = self.documents.get(uri) else {
            return Vec::new();
        };
        let text = doc.text.as_str();
        self.compiled_candidates(text, |_raw, span, compiled| {
            let kind = if compiled.diagnostics.is_empty() {
                EngineSemanticKind::Lowered
            } else {
                EngineSemanticKind::Refused
            };
            Some(EngineSemanticToken {
                range: range_of(text, span.0, span.1),
                kind,
            })
        })
    }

    // -----------------------------------------------------------------------
    // The colour-presentation answer
    // -----------------------------------------------------------------------

    /// The spellings one colour answers back: the config's own table
    /// walked backwards — every (colour-group root, token, opacity step)
    /// whose light-tier resolve is EXACTLY the requested bytes, the same
    /// [`TokenTable::resolve_rgba`] the swatches resolve through (one
    /// round trip, both directions share it). The surface is the
    /// completion source's own: the registry's colour-group rows, one
    /// spelling per root (the SAME gate that makes a suggested token
    /// class a lowered token class). The answer carries no document
    /// argument: the spellings ride the config's table, the open file
    /// does not change them — the one engine question the document map
    /// does not gate. The theme-less config answers empty (the honest
    /// boundary the completion keeps), a colour no token resolves
    /// answers empty (never a guessed spelling), and the answer is pure
    /// and deduped over its config.
    pub fn colour_presentations(&self, rgba: [u8; 4]) -> Vec<EngineColourPresentation> {
        let Some(table) = self.config.tokens.as_ref() else {
            return Vec::new();
        };
        // The resolves run ONCE per (token, step) — the row walk below
        // only spells (the labels carry the root; the colour the picker
        // picked does not depend on it).
        let mut names: Vec<&str> = table.names().collect();
        names.sort_unstable();
        let mut matches = Vec::new();
        for name in names {
            let Some(id) = table.id(name) else {
                continue;
            };
            for step in aeolus_data::tables::OPACITY_STEPS {
                let alpha = f32::from(*step) / 100.0;
                let Ok(resolved) = table.resolve_rgba(id, alpha, false) else {
                    continue;
                };
                if resolved == rgba {
                    matches.push((name, *step));
                }
            }
        }
        // The spellings ride the registry's colour-group rows (the
        // completion source's own surface, one label per root) in the
        // registry's own order.
        let mut out = Vec::new();
        let mut seen = HashSet::new();
        for row in ROWS {
            if !COLOUR_GROUPS.contains(&row.group) {
                continue;
            }
            for (name, step) in &matches {
                // The bare spelling IS the 100% step (the modifier rides
                // only below it — the `/opacity` identity the theme pass
                // lowers through).
                let label = if *step == 100 {
                    format!("{}-{name}", row.root)
                } else {
                    format!("{}-{name}/{}", row.root, step)
                };
                if seen.insert(label.clone()) {
                    out.push(EngineColourPresentation {
                        label,
                        token: (*name).to_string(),
                        alpha: *step,
                    });
                }
            }
        }
        out
    }

    // -----------------------------------------------------------------------
    // The inlay-hint answer
    // -----------------------------------------------------------------------

    /// The document's inline hints: every candidate class that lowers
    /// cleanly rides one hint at its END position — the CSS arm's own
    /// declarations compacted to the single-line label, the hover's
    /// markdown riding as the tooltip. The walk is the shared
    /// per-candidate compile (`compiled_candidates`), and the
    /// rendering is the HOVER's own — the shared `lowered_markdown`,
    /// one answer shape at two surfaces. The refused candidates never
    /// hint (their story is the diagnostics' and the semantic legend's —
    /// a refused class has no CSS to show, the honest boundary), and the
    /// theme-less config answers the same (the CSS arm renders the
    /// lowered class either way). Deterministic and pure over the
    /// document map; an unknown URI answers empty.
    pub fn inlay_hints(&self, uri: &str) -> Vec<EngineInlayHint> {
        let Some(doc) = self.documents.get(uri) else {
            return Vec::new();
        };
        let text = doc.text.as_str();
        self.compiled_candidates(text, |raw, span, compiled| {
            // The refusal boundary: a class the walk refused has no
            // CSS to show — the diagnostics publish it, the legend
            // highlights it, the hint stays quiet.
            if !compiled.diagnostics.is_empty() {
                return None;
            }
            let sheet = match &self.config.tokens {
                Some(table) => emit_css_theme(raw, table, &self.config.compile),
                None => emit_css(raw, &self.config.compile),
            };
            let Ok(sheet) = sheet else {
                return None;
            };
            let cond = compiled
                .program
                .layers
                .last()
                .map_or(Cond::Base, |layer| layer.cond);
            // The hint rides where the class's spelling stops: the
            // candidate's END, already in the LSP's UTF-16 pairs.
            let (line, character) = position_of(text, span.1);
            Some(EngineInlayHint {
                position: EnginePosition { line, character },
                label: compact_css(&sheet),
                tooltip: lowered_markdown(raw, cond, &sheet),
            })
        })
    }

    // -----------------------------------------------------------------------
    // The code-action answer
    // -----------------------------------------------------------------------

    /// The quick fixes the cursor's candidate carries: the candidate
    /// under the position (the shared locator's walk — the hover's
    /// own), its compile walk's diagnostics each folded through the
    /// pipeline's suggestion seam — the corrections the compile crate's
    /// audited tables answer (the registry's rows, the closed variant
    /// set, the registered customs), the variant prefixes and modifier
    /// tails composed back onto the replacement. The action rides the
    /// finding it repairs (the editor's filter key), the CLI-shaped
    /// title and the corrected token text over the candidate's range.
    /// Deterministic and pure over the document map; an unknown URI, a
    /// cursor outside any candidate and a finding with no honest
    /// correction answer empty.
    pub fn code_actions(&self, uri: &str, line: u32, character: u32) -> Vec<EngineCodeAction> {
        let Some(doc) = self.documents.get(uri) else {
            return Vec::new();
        };
        let text = doc.text.as_str();
        let Some(offset) = offset_of(text, line, character) else {
            return Vec::new();
        };
        let Some((start, end)) = candidate_span_at(text, offset) else {
            return Vec::new();
        };
        let raw = &text[start..end];
        let range = range_of(text, start, end);
        // The single-candidate pipeline answer: the themed walk when the
        // config carries a table, the theme-less boundary otherwise.
        let compiled = self.compiled(raw);
        let Ok(compiled) = compiled else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for diagnostic in &compiled.diagnostics {
            // The miss-path fold the CLI renders, the engine's own
            // config table riding in (the customs stay the host's
            // registration — the engine's config carries none).
            for suggestion in suggest_for(diagnostic, None) {
                // The finding the action repairs: the same rendering
                // the diagnostics publish (the message with its help
                // line, the stable code).
                let mut message = String::new();
                let _ = write!(message, "{diagnostic}");
                let _ = write!(message, "\nhelp: {}", diagnostic.kind.help());
                out.push(EngineCodeAction {
                    diagnostic: EngineDiagnostic {
                        range,
                        message,
                        code: diagnostic.kind.code().to_string(),
                    },
                    title: format!("did you mean `{}`?", suggestion.replacement),
                    replacement: suggestion.replacement,
                });
            }
        }
        out
    }

    // -----------------------------------------------------------------------
    // The document-highlight answer
    // -----------------------------------------------------------------------

    /// The document's occurrences of the candidate under the cursor:
    /// every class whose text equals the cursor's candidate, its
    /// byte-true range, document order (the extraction seam's scan
    /// order). The highlight is TEXTUAL — a refused class's occurrences
    /// answer exactly like a lowered one's (the editor's own
    /// click-to-find, no compile verdict riding) — and the cursor's own
    /// occurrence rides the answer (the wire contract the editor
    /// renders). Deterministic and pure over the document map; an
    /// unknown URI and a cursor outside any candidate answer empty.
    pub fn document_highlights(
        &self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Vec<EngineHighlight> {
        let Some(doc) = self.documents.get(uri) else {
            return Vec::new();
        };
        let text = doc.text.as_str();
        let Some(offset) = offset_of(text, line, character) else {
            return Vec::new();
        };
        // The candidate under the cursor: the shared locator's walk
        // (the hover's own), the raw text the equality keys on.
        let Some((start, end)) = candidate_span_at(text, offset) else {
            return Vec::new();
        };
        let raw = &text[start..end];
        let mut out = Vec::new();
        for hit in scan_rust_strings(text) {
            let (candidates, _) = parse_class_string(&hit.text);
            for candidate in candidates {
                if candidate.raw != raw {
                    continue;
                }
                let range = range_of(
                    text,
                    hit.offset + candidate.span.0,
                    hit.offset + candidate.span.1,
                );
                out.push(EngineHighlight { range });
            }
        }
        out
    }

    // -----------------------------------------------------------------------
    // The rename face
    // -----------------------------------------------------------------------

    /// The renameable span under the cursor (the wire's
    /// `textDocument/prepareRename`): the candidate's own byte-true
    /// range — the full spelling INCLUDING its variant prefixes (the
    /// highlights' raw-text contract: `hover:flex-col` is one token,
    /// renamed whole). `None` outside any candidate — the protocol's
    /// cannot-rename answer the editor renders. Deterministic and pure
    /// over the document map; an unknown URI answers none.
    pub fn prepare_rename(&self, uri: &str, line: u32, character: u32) -> Option<EngineRange> {
        let doc = self.documents.get(uri)?;
        let text = doc.text.as_str();
        let offset = offset_of(text, line, character)?;
        let (start, end) = candidate_span_at(text, offset)?;
        Some(range_of(text, start, end))
    }

    /// The rename edits (the wire's `textDocument/rename`): ONE edit
    /// per textual occurrence of the cursor's candidate — the
    /// highlights' own walk, the new spelling riding every edit. The
    /// face is TEXTUAL (the highlights' own contract, no compile
    /// verdict riding): the diagnostics channel tells the truth about
    /// the NEW spelling's compile the moment the didChange lands. An
    /// empty new name answers NO edits (never a deletion walk — the
    /// protocol's own validity clause, the quiet shape). Deterministic
    /// and pure over the document map; an unknown URI answers empty.
    pub fn rename(
        &self,
        uri: &str,
        line: u32,
        character: u32,
        new_name: &str,
    ) -> Vec<EngineTextEdit> {
        if new_name.is_empty() {
            return Vec::new();
        }
        self.document_highlights(uri, line, character)
            .into_iter()
            .map(|highlight| EngineTextEdit {
                range: highlight.range,
                new_text: new_name.to_string(),
            })
            .collect()
    }

    // -----------------------------------------------------------------------
    // The formatting answer
    // -----------------------------------------------------------------------

    /// The document's canonical-order edits (the class-sort seam's
    /// editor face — the wire's `textDocument/formatting`): every
    /// extracted class string whose order is not yet the registry's
    /// canonical one rides ONE edit over the string's content span, the
    /// sort seam's own canonical join as the new text.
    ///
    /// The walk is the pipeline's own sort seam
    /// ([`sort`](aeolus_compile::sort)) — the rows' canonical utility
    /// progression, the variant buckets riding after the base by the
    /// IR's cascade specificity, registered customs unavailable here
    /// (the engine's config carries no registrations, the honest
    /// default [`sort`] has). The THEME-LESS walk is deliberate: the
    /// order is a shape-level fact (the row a body addresses), the
    /// sort module documents the boundary — a themeless token boundary
    /// sorts exactly like its lowered sibling, so the formatter needs
    /// no table.
    ///
    /// Three boundaries the answer keeps honestly:
    ///
    /// * an already-canonical string answers NO edit — the no-op
    ///   contract a format-on-save rides (the edit list is exactly the
    ///   diff, never a rewrite);
    /// * the canonical join normalizes the string's whitespace run to
    ///   single spaces (the seam's own documented canonical form), so a
    ///   whitespace-only difference is an edit too;
    /// * the classes the seam cannot order ride the TAIL in source
    ///   order and never block the edit (the known prefix still
    ///   reorders) — but the misses themselves are NOT this answer's
    ///   channel: the diagnostics publish them (push and pull), the
    ///   formatter carries edits only.
    ///
    /// Deterministic and pure over the document map; an unknown URI
    /// answers empty. The edit's range is the string CONTENT's span
    /// (the extractor's own trimmed hit, byte-true in the LSP's UTF-16
    /// pairs) — the quotes around it stay the editor's.
    pub fn format(&self, uri: &str) -> Vec<EngineTextEdit> {
        let Some(doc) = self.documents.get(uri) else {
            return Vec::new();
        };
        let text = doc.text.as_str();
        let mut out = Vec::new();
        for hit in scan_rust_strings(text) {
            // The themeless sort walk: the order is a shape-level fact,
            // deliberately value/theme free (the sort module's own
            // boundary). Only the empty input errors — the extractor's
            // prose pre-filter keeps those out anyway.
            let Ok(sorted) = sort(&hit.text) else {
                continue;
            };
            let canonical = sorted.to_class_string();
            if canonical == hit.text {
                continue;
            }
            let range = range_of(text, hit.offset, hit.offset + hit.text.len());
            out.push(EngineTextEdit {
                range,
                new_text: canonical,
            });
        }
        out
    }

    // -----------------------------------------------------------------------
    // The document-symbol answer
    // -----------------------------------------------------------------------

    /// The document's outline: every candidate class the extraction
    /// seam's strings carry, one symbol each — the spelling the NAME,
    /// its byte-true range, the semantic legend's own verdict (the
    /// same classification the highlighter feeds, no parallel matcher)
    /// and the pipeline's own detail line (a lowered class renders the
    /// compact CSS the inlay hint's label carries; a refused one the
    /// stable code the diagnostics key on — the detail names the WHY
    /// without re-rendering the refusal prose). The walk is the shared
    /// per-candidate compile ([`AeolusEngine::compiled_candidates`]),
    /// document order (the extraction seam's scan order) — the wire's
    /// outline answer, deterministic and pure over the document map; an
    /// unknown URI answers empty.
    pub fn document_symbols(&self, uri: &str) -> Vec<EngineSymbol> {
        let Some(doc) = self.documents.get(uri) else {
            return Vec::new();
        };
        let text = doc.text.as_str();
        self.compiled_candidates(text, |raw, span, compiled| {
            let (kind, detail) = if compiled.diagnostics.is_empty() {
                // The lowered symbol: the CSS arm's own declarations
                // compacted to the single-line label (the hint label's
                // own rendering — one answer shape at a third surface).
                let sheet = match &self.config.tokens {
                    Some(table) => emit_css_theme(raw, table, &self.config.compile),
                    None => emit_css(raw, &self.config.compile),
                };
                let Ok(sheet) = sheet else {
                    return None;
                };
                (EngineSemanticKind::Lowered, compact_css(&sheet))
            } else {
                // The refused symbol: the first finding's stable code —
                // the outline names the boundary, the diagnostics
                // publish the prose.
                let code = compiled
                    .diagnostics
                    .first()
                    .map_or_else(String::new, |diagnostic| diagnostic.kind.code().to_string());
                (EngineSemanticKind::Refused, code)
            };
            Some(EngineSymbol {
                name: raw.to_string(),
                range: range_of(text, span.0, span.1),
                detail,
                kind,
            })
        })
    }

    // -----------------------------------------------------------------------
    // The shared per-candidate compile walk
    // -----------------------------------------------------------------------

    /// The document's compiled candidates, ONE walk (the whole-document
    /// answers' shared scaffold — the colour swatches, the semantic
    /// legend and the inlay hints — the shared-locator precedent: one
    /// walk, many answers, no parallel scaffolds to drift). The fold
    /// receives each candidate's raw spelling, its ABSOLUTE byte span
    /// in the document text (the hit's content offset plus the
    /// candidate's span) and the single-candidate pipeline answer (the
    /// themed walk when the config carries a table, the theme-less
    /// boundary otherwise — the same entry points the CLI and the
    /// macros run, one candidate per call). A `None` answer skips the
    /// candidate: each answer's boundary (the keyword slots, the
    /// refusals, the sheet misses) stays the answer's own, documented
    /// in its rustdoc — the walker carries none of them.
    fn compiled_candidates<T>(
        &self,
        text: &str,
        mut fold: impl FnMut(&str, (usize, usize), &Compiled) -> Option<T>,
    ) -> Vec<T> {
        let mut out = Vec::new();
        for hit in scan_rust_strings(text) {
            let (candidates, _) = parse_class_string(&hit.text);
            for candidate in candidates {
                let compiled = self.compiled(candidate.raw);
                let Ok(compiled) = compiled else {
                    continue;
                };
                let span = (hit.offset + candidate.span.0, hit.offset + candidate.span.1);
                if let Some(item) = fold(candidate.raw, span, &compiled) {
                    out.push(item);
                }
            }
        }
        out
    }
}

// ---------------------------------------------------------------------------
// The position conversions (the LSP's UTF-16 model, owned ONCE here)
// ---------------------------------------------------------------------------

/// The candidate under one byte offset — the cursor-keyed answers'
/// shared locator (the hover, the quick fixes, the highlights): the
/// extraction seam's strings, the hit whose content range contains the
/// offset, the lexer's own candidate spans. Returns the candidate's
/// ABSOLUTE byte span in the document text (the hit's content offset
/// plus the candidate's span); `None` — the cursor rests outside any
/// candidate.
fn candidate_span_at(text: &str, offset: usize) -> Option<(usize, usize)> {
    let hit = scan_rust_strings(text)
        .into_iter()
        .find(|hit| offset >= hit.offset && offset < hit.offset + hit.text.len())?;
    let local = offset.saturating_sub(hit.offset);
    let (candidates, _) = parse_class_string(&hit.text);
    let candidate = candidates
        .iter()
        .find(|candidate| local >= candidate.span.0 && local < candidate.span.1)?;
    Some((hit.offset + candidate.span.0, hit.offset + candidate.span.1))
}

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

/// The lowered class's markdown — the ONE rendering the hover and the
/// inlay hint share (the wire-diagnostic mapping's own single-source
/// rule): the class, its folded condition, the CSS arm's own
/// declaration texts in a fenced block, the sheet's own diagnostics
/// riding after (they never silence, the honesty contract).
fn lowered_markdown(raw: &str, cond: Cond, sheet: &CssSheet) -> String {
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
    contents
}

/// The compact single-line label the inlay hint rides: the sheet's
/// first rule body — the CSS arm's own declarations, trimmed, each
/// declaration's trailing `;` folded into the `"; "` join. A paraphrase
/// never enters the label: the text is the pipeline's own output, one
/// declaration per join on the wire.
fn compact_css(sheet: &CssSheet) -> String {
    sheet
        .rules
        .first()
        .map(|rule| {
            rule.body
                .lines()
                .map(|line| line.trim().trim_end_matches(';').trim_end())
                .collect::<Vec<_>>()
                .join("; ")
        })
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// The colour-swatch resolution (the IR's three table-resolvable slots)
// ---------------------------------------------------------------------------

/// The swatch one declaration value resolves to — the light tier (the
/// editor's default preview; the dark half rides the same table the
/// hover renders). The IR's colour-carrying slots, one arm each:
/// the baked literal ([`Val::Rgba`]), the runtime token ([`Val::Tok`],
/// the alpha modifier riding the same premultiplied bake the token path
/// lowers), the T1 pair slot ([`Val::ModePair`], the light half). The
/// keyword slot ([`Val::Kw`]) stays unnamed — its row texts are CSS
/// keywords (`transparent`, `currentColor`), not swatches; the numeric
/// and verbatim slots are not colours at all.
fn colour_of(val: Val, tokens: Option<&TokenTable>) -> Option<[u8; 4]> {
    match val {
        Val::Rgba(bytes) => Some(bytes),
        Val::Tok { id, alpha } => {
            let table = tokens?;
            table.resolve_rgba(id, alpha, false).ok()
        }
        Val::ModePair(slot) => {
            let table = tokens?;
            table.pair_at(slot).map(|(light, _)| light)
        }
        Val::Kw(_) | Val::Px(_) | Val::Rem(_) | Val::Pct(_) | Val::Num(_) | Val::Verbatim(_) => {
            None
        }
    }
}

// ---------------------------------------------------------------------------
// The completion sources (the compiler's own audited tables, one walk)
// ---------------------------------------------------------------------------

/// The completion for one typed fragment over the config's token table
/// (the theme half; `None` — the theme-less boundary — completes no
/// token). The fragment model is the method doc's: no `:` typed, the
/// variant stack AND the utility root both answer; after a `:`, the
/// utility half alone. The rank is assigned last — the sources emit in
/// their canonical order, the numbering just names it.
fn completions_for(fragment: &str, tokens: Option<&TokenTable>) -> Vec<EngineCompletion> {
    let mut out: Vec<EngineCompletion> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    match fragment.rfind(':') {
        // The stack is complete: the utility half answers alone.
        Some(colon) => {
            let util = &fragment[colon + 1..];
            class_completions(util, tokens, &mut out, &mut seen);
        }
        // No `:` yet: both halves are in play (the fragment could be
        // the stack's first segment or the root's first characters).
        None => {
            variant_completions(fragment, &mut out, &mut seen);
            class_completions(fragment, tokens, &mut out, &mut seen);
        }
    }
    for (rank, item) in out.iter_mut().enumerate() {
        item.rank = rank;
    }
    out
}

/// The variant half: the compile crate's audited closed set, prefix-
/// filtered. The label is the spelling alone (the editor appends the
/// `:` it types next); the detail names the source family.
fn variant_completions(
    fragment: &str,
    out: &mut Vec<EngineCompletion>,
    seen: &mut HashSet<String>,
) {
    for name in KNOWN_VARIANTS {
        if name.starts_with(fragment) && seen.insert((*name).to_string()) {
            out.push(EngineCompletion {
                label: (*name).to_string(),
                detail: "variant — the folded condition's spelling".to_string(),
                kind: EngineCompletionKind::Variant,
                rank: 0,
            });
        }
    }
}

/// The utility half: one walk over the registry rows in their canonical
/// (sort-order) sequence, each source emitting in its own audited
/// order. Sources, in walk order:
///
/// 1. the BARE roots (the root IS the class — prefix match);
/// 2. the colour roots' theme tokens (the SAME gate the compile walk
///    lowers colour values through — one source of truth; the source
///    leads the keyword tables, so a name both carry renders as the
///    theme-aware item);
/// 3. the keyword/composite tables (root + `-` + suffix, the fragment's
///    typed remainder filtering);
/// 4. the audited default-step scales (the `opacity` root's own table —
///    the one Number-scale shape the registry carries a closed table
///    for).
fn class_completions(
    fragment: &str,
    tokens: Option<&TokenTable>,
    out: &mut Vec<EngineCompletion>,
    seen: &mut HashSet<String>,
) {
    let token_names: Vec<&str> = tokens
        .map(|table| {
            let mut names: Vec<&str> = table.names().collect();
            names.sort_unstable();
            names
        })
        .unwrap_or_default();
    for row in ROWS {
        let mut arms = Vec::new();
        flatten_kind(row.kind, &mut arms);
        // 1. The bare roots: the fragment is the root's typed prefix.
        if arms.iter().any(|arm| matches!(arm, ValueKind::Bare(_)))
            && row.root.starts_with(fragment)
            && seen.insert(row.root.to_string())
        {
            out.push(utility_item(row.root, row.targets));
        }
        // 2./3./4. The suffix spellings: the fragment already carries
        // the root (`flex-`, `bg-pri`, `opacity-5`) — a shorter
        // fragment keeps completing the bare roots it prefixes.
        let Some(rest) = fragment.strip_prefix(row.root) else {
            continue;
        };
        let Some(suffix) = rest.strip_prefix('-') else {
            continue;
        };
        // 2. The theme tokens, through the compile walk's own colour
        // gate: a suggested token class is a lowered token class. The
        // source leads the row's keyword tables — a name both tables
        // carry (`primary` rides COLOR_KEYWORDS too) renders as the
        // theme-aware item.
        if COLOUR_GROUPS.contains(&row.group) {
            for name in &token_names {
                if !name.starts_with(suffix) {
                    continue;
                }
                let label = format!("{}-{}", row.root, name);
                // The tier detail: a T1 pair names its dark half (the
                // branch-free index the token path lowers to).
                let dark = tokens
                    .and_then(|table| table.id(name).map(|id| table.has_dark(id)))
                    .unwrap_or(false);
                if seen.insert(label.clone()) {
                    out.push(EngineCompletion {
                        label,
                        detail: if dark {
                            format!("theme token — --color-{name} · dark pair")
                        } else {
                            format!("theme token — --color-{name}")
                        },
                        kind: EngineCompletionKind::Token,
                        rank: 0,
                    });
                }
            }
        }
        // 3. The keyword/composite tables.
        for arm in &arms {
            match arm {
                ValueKind::Keywords(table) => {
                    for (spelling, _) in *table {
                        if !spelling.starts_with(suffix) {
                            continue;
                        }
                        let label = format!("{}-{}", row.root, spelling);
                        if seen.insert(label.clone()) {
                            out.push(utility_item(&label, row.targets));
                        }
                    }
                }
                ValueKind::Composite(table) => {
                    for (spelling, _, _) in *table {
                        if !spelling.starts_with(suffix) {
                            continue;
                        }
                        let label = format!("{}-{}", row.root, spelling);
                        if seen.insert(label.clone()) {
                            out.push(utility_item(&label, row.targets));
                        }
                    }
                }
                _ => {}
            }
        }
        // 4. The audited default-step scale: the opacity root's own
        // closed table (the Number scales are dynamic — every number a
        // legal spelling — but this one the registry ships a table for,
        // and the completion gives the audited surface its consumer).
        if row.root == "opacity" && arms.iter().any(|arm| matches!(arm, ValueKind::Number)) {
            for step in aeolus_data::tables::OPACITY_STEPS {
                let spelling = step.to_string();
                if !spelling.starts_with(suffix) {
                    continue;
                }
                let label = format!("opacity-{spelling}");
                if seen.insert(label.clone()) {
                    out.push(utility_item(&label, row.targets));
                }
            }
        }
    }
}

/// One registry-sourced item: the detail is the row's own longhand
/// names, the CSS surface the class lowers to.
fn utility_item(label: &str, targets: &'static [aeolus_ir::Prop]) -> EngineCompletion {
    let mut detail = String::from("utility — ");
    for (index, prop) in targets.iter().enumerate() {
        if index > 0 {
            detail.push_str(", ");
        }
        detail.push_str(prop.name());
    }
    EngineCompletion {
        label: label.to_string(),
        detail,
        kind: EngineCompletionKind::Utility,
        rank: 0,
    }
}

/// The `AnyOf` arms in declaration order — the value pass's own shape
/// walk, mirrored for the completion's bare/keyword/composite reads
/// (the compile crate's `flatten` is the original; this module's read
/// needs the same shape).
fn flatten_kind(kind: ValueKind, out: &mut Vec<ValueKind>) {
    match kind {
        ValueKind::AnyOf(inner) => {
            for k in inner {
                flatten_kind(*k, out);
            }
        }
        k => out.push(k),
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
        Cond::Valid => "valid",
        Cond::Invalid => "invalid",
        Cond::UserValid => "user-valid",
        Cond::UserInvalid => "user-invalid",
        // The element-state family: the bare spellings (the validation
        // arm's shape — the hover renders the element-state tier).
        Cond::Target => "target",
        Cond::Indeterminate => "indeterminate",
        Cond::Autofill => "autofill",
        Cond::ReadOnly => "read-only",
        // The widget-state family: the bare spellings (the
        // element-state arm's shape — the hover renders the tier).
        Cond::Open => "open",
        Cond::Inert => "inert",
        Cond::Default => "default",
        // The form-state family: the bare spellings (the widget-state
        // arm's shape — the hover renders the tier).
        Cond::Enabled => "enabled",
        Cond::Required => "required",
        Cond::Optional => "optional",
        Cond::InRange => "in-range",
        Cond::OutOfRange => "out-of-range",
        Cond::Selection => "selection",
        Cond::Placeholder => "placeholder",
        // The pseudo-element tier: the bare spellings (the
        // selection/placeholder arm's shape — the hover renders the
        // render-target tier).
        Cond::Before => "before",
        Cond::After => "after",
        Cond::Marker => "marker",
        Cond::FirstLetter => "first-letter",
        Cond::FirstLine => "first-line",
        Cond::FileSelector => "file",
        Cond::Backdrop => "backdrop",
        Cond::DetailsContent => "details-content",
        Cond::GroupHover => "group-hover",
        Cond::GroupFocus => "group-focus",
        // The composition tier: the bare spellings (the
        // widget-state arm's shape — the hover renders the tier).
        Cond::GroupFocusVisible => "group-focus-visible",
        Cond::GroupFocusWithin => "group-focus-within",
        Cond::GroupActive => "group-active",
        Cond::GroupDisabled => "group-disabled",
        Cond::GroupVisited => "group-visited",
        Cond::GroupChecked => "group-checked",
        Cond::PeerHover => "peer-hover",
        Cond::PeerFocus => "peer-focus",
        Cond::PeerFocusVisible => "peer-focus-visible",
        Cond::PeerFocusWithin => "peer-focus-within",
        Cond::PeerActive => "peer-active",
        Cond::PeerDisabled => "peer-disabled",
        Cond::PeerVisited => "peer-visited",
        Cond::PeerChecked => "peer-checked",
        Cond::Bp(_) | Cond::Min(_) => "breakpoint",
        Cond::Max(_) => "max-breakpoint",
        Cond::Container { .. } | Cond::MaxContainer { .. } => "container",
        Cond::Data { .. } => "data probe",
        Cond::Aria { .. } => "aria probe",
        Cond::Has(_) | Cond::GroupHas(_) | Cond::PeerHas(_) => "descendant probe",
        Cond::Supports(_) => "supports condition",
        Cond::Media(_) => "media preference",
        Cond::Dir(_) => "direction",
        Cond::Structural(_) => "structural",
        Cond::Starting => "starting-style",
    }
}
