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
use std::sync::Arc;

use aeolus_compile::{
    COLOUR_GROUPS, CompileError, Compiled, Config, CssSheet, Diagnostic, DynCache, DynObserver,
    KNOWN_VARIANTS, ScanHit, WalkObserver, emit_css, emit_css_theme, scan_rust_strings, sort,
    suggest_for,
};
use aeolus_data::ValueKind;
use aeolus_data::rows::ROWS;
use aeolus_ir::{Cond, Val};
use aeolus_syntax::parse_class_string;
use aeolus_theme::{ThemeError, TokenTable};

use crate::token_origins::{TokenOrigins, origin_range};
use crate::{
    DocumentMap, EngineCodeAction, EngineColour, EngineColourPresentation, EngineCompletion,
    EngineCompletionKind, EngineDefinition, EngineDiagnostic, EngineHighlight, EngineHover,
    EngineInlayHint, EnginePosition, EngineRange, EngineReference, EngineRenameEdit,
    EngineSelectionRange, EngineSemanticKind, EngineSemanticToken, EngineSymbol, EngineTextEdit,
    EngineWorkspaceDiagnostic, EngineWorkspaceOutcome, EngineWorkspaceSymbol, RethemeOutcome,
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
    /// ([`aeolus_compile::compile_theme`], colour tokens and opacity
    /// modifiers lower through the tiered tables); `None` — the
    /// theme-less boundary (the tokens stay typed refusals, the honest
    /// default [`aeolus_compile::compile`] has).
    pub tokens: Option<TokenTable>,
    /// The compile seam's own knobs (the spacing strategy, the rem
    /// size) — the engine never forks pipeline semantics.
    pub compile: Config,
    /// The origin URI the goto-definition face names (the wire's
    /// `Location.uri`): the file the config's table was parsed from —
    /// the `--theme` file's URI a host passes at construction, the
    /// wired [`ThemeLoader`](crate::ThemeLoader)'s URI the wire half
    /// composes in. `None` keeps the face dark (the honest default:
    /// the built-in shadcn bake is no file), and the provenance scan
    /// answers only what that file itself declares — a seeded palette
    /// name rides no origin either way.
    pub theme_origin: Option<String>,
}

impl EngineConfig {
    /// A config over an explicit table (the host CSS shape: the same
    /// [`ThemeSource`](aeolus_theme::ThemeSource) construction the CLI's
    /// `--theme <path>` accepts).
    pub fn new(tokens: Option<TokenTable>) -> Self {
        Self {
            tokens,
            compile: Config::default(),
            theme_origin: None,
        }
    }

    /// The config with the goto-definition face's origin URI named (the
    /// host's own statement of where the table's tokens live): the file
    /// [`TokenTable::provenance`](aeolus_theme::TokenTable::provenance)
    /// scans answer through it, byte-true. A host with a table built
    /// from [`ThemeSource::from_css`](aeolus_theme::ThemeSource::from_css)
    /// names the URI that CSS lives at; a host whose tokens live
    /// somewhere else wires a [`TokenOrigins`] store on the ENGINE
    /// instead (the store rides first).
    pub fn with_theme_origin(mut self, uri: impl Into<String>) -> Self {
        self.theme_origin = Some(uri.into());
        self
    }

    /// The CLI-parity default, fallible: the built-in shadcn bake with
    /// the theme pass's OWN typed error. `Default` rides this
    /// constructor's documented-panic shape (the embedded palette is an
    /// internal invariant — the build panics only on drift the suite
    /// pins); a host that prefers the typed boundary calls THIS and
    /// names the `ThemeError` the bake can carry, the same two-lane
    /// contract the CLI's own theme loading keeps.
    pub fn try_default() -> Result<Self, ThemeError> {
        let source = aeolus_theme::ThemeSource::shadcn()?;
        let table = TokenTable::baked(&source)?;
        Ok(Self {
            tokens: Some(table),
            compile: Config::default(),
            // The built-in bake is no file: the goto-definition face
            // stays dark until the host names an origin (or the wire
            // half's loader composes one in).
            theme_origin: None,
        })
    }
}

impl Default for EngineConfig {
    /// The CLI-parity default: the built-in shadcn bake. The embedded
    /// palette is an internal invariant (the macro's freeze walk rides
    /// the same construction), so the build panics only on drift the
    /// suite pins — the fallible path is
    /// [`EngineConfig::try_default`].
    fn default() -> Self {
        Self::try_default().expect("the built-in shadcn bake parses and bakes")
    }
}

// ---------------------------------------------------------------------------
// The engine
// ---------------------------------------------------------------------------

/// The headless LSP engine: the open documents, the pipeline answers,
/// the position conversions. One engine per server session (the wire
/// half owns it); a host embedding the engine directly rides the same
/// two questions without the transport.
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
    /// The wired walk observer, `None` the every-constructor default
    /// (the compile seam's walk face over the pooled path — see
    /// [`AeolusEngine::with_walk_observer`]; a miss lowers through the
    /// observed walk, a hit streams nothing).
    walk: Option<Arc<dyn WalkObserver>>,
    /// The wired token-origins store, `None` the every-constructor
    /// default — the goto-definition half's host override
    /// ([`AeolusEngine::with_token_origins`]): the store rides FIRST,
    /// the config's own provenance path the built-in fallback.
    origins: Option<Arc<dyn TokenOrigins>>,
}

impl std::fmt::Debug for AeolusEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // The wired walk observer is a host handle, not printable state
        // — the honest Debug names the session view and whether one rides
        // (the compile crate's cache Debug's own rule).
        f.debug_struct("AeolusEngine")
            .field("config", &self.config)
            .field("documents", &self.documents.len())
            .field("cache", &self.cache)
            .field("walk", &self.walk.is_some())
            .field("origins", &self.origins.is_some())
            .finish_non_exhaustive()
    }
}

impl AeolusEngine {
    /// An engine over one configuration.
    pub fn new(config: EngineConfig) -> Self {
        Self {
            config,
            documents: DocumentMap::new(),
            cache: DynCache::new(),
            walk: None,
            origins: None,
        }
    }

    /// The engine with a host-supplied token-origins store — the
    /// goto-definition half's override seam (the
    /// [`ThemeLoader`](crate::ThemeLoader) precedent: the host's own
    /// statement of where a resource lives, the built-in a consumer of
    /// the seam, never its definition), chained after `new` like the
    /// observer faces. The store rides FIRST: a wired store's answer IS
    /// the definition (the engine's config path — the riding table's
    /// own provenance under [`EngineConfig::theme_origin`] — the
    /// fallback a store-less session keeps). The compile gate rides
    /// ahead of every source: a class the walk refused carries no
    /// token, whatever the store knows.
    pub fn with_token_origins(mut self, origins: impl TokenOrigins + 'static) -> Self {
        self.origins = Some(Arc::new(origins));
        self
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

    /// The engine with a host-supplied walk observer — the compile
    /// seam's WALK face joined over the pooled path (the
    /// [`DynCache::compile_observed`] shape, chained after `new`: the
    /// built-in [`WalkStats`](aeolus_compile::WalkStats) rides an `Arc`
    /// the host keeps a handle to). Every analyze/hover/complete
    /// answer the pool MISSES lowers through the observed walk — the
    /// lex totals, the per-candidate verdicts and the assembly count
    /// stream in lowering order — while a HIT answers from the pool
    /// and streams nothing (the channel reports lowerings, not
    /// answers). The cache face (`with_observer`) answers "what did
    /// the pool decide"; this face answers "what did the walk do when
    /// the pool missed" — one host can wire both and read one session
    /// from the two vocabularies. The observer adds a channel; it
    /// never changes a decision.
    pub fn with_walk_observer(mut self, observer: impl WalkObserver + 'static) -> Self {
        self.walk = Some(Arc::new(observer));
        self
    }

    /// The theme lifecycle's seam: swap the table the answers ride.
    /// The config's `tokens` slot replaces wholesale (`None` the
    /// theme-less boundary, `Some` the themed walk), and the swap
    /// retires the pool it invalidates — every pooled row was lowered
    /// under the OUTGOING table, so [`DynCache::clear`] (the compile
    /// crate's invalidation seam) empties it rather than letting dead
    /// rows linger until rotation. The equality fast-path keeps the
    /// no-op save cheap: a candidate table whose CONTENT equals the
    /// riding one (tier, names and resolved colours — a re-bake of the
    /// same css, a fresh object) answers [`RethemeOutcome::Unchanged`]
    /// and touches NOTHING (the pool keeps serving, the answers stay
    /// byte-identical — the differential pins). A real swap answers
    /// [`RethemeOutcome::Swapped`].
    ///
    /// The document map rides untouched: the answers re-derive lazily
    /// from the strings (the same purity every other answer keeps), so
    /// a swap is pure over the documents and the caller decides what
    /// republishes (the wire half republishes EVERY open document's
    /// diagnostics on a swap, nothing on the no-op). Deterministic and
    /// pure over the configuration; a failed source construction never
    /// reaches here — the loader's `Err` keeps the riding table (the
    /// wire's own contract).
    pub fn retheme(&mut self, tokens: Option<TokenTable>) -> RethemeOutcome {
        let changed = match (&self.config.tokens, &tokens) {
            (Some(riding), Some(candidate)) => !tokens_equal(riding, candidate),
            (None, None) => false,
            _ => true,
        };
        if !changed {
            return RethemeOutcome::Unchanged;
        }
        self.config.tokens = tokens;
        self.cache.clear();
        RethemeOutcome::Swapped
    }

    /// The wire half's origin note: the wired loader's URI names the
    /// file the session's tables come from — the goto-definition face's
    /// origin (the config's own default stays `None` until a loader
    /// rides: the built-in bake is no file). Idempotent — one loader,
    /// one file, the URI fixed for the loader's life.
    pub(crate) fn note_theme_origin(&mut self, uri: &str) {
        self.config.theme_origin = Some(uri.to_string());
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

    /// The open documents' URIs, sorted (the cross-document answers'
    /// own determinism contract — the answer is pure over the hash
    /// map). The theme lifecycle's republish walk rides it (the wire
    /// half's swap republishes every open document's diagnostics, URI
    /// order), and a host embedding the engine with its own push
    /// channel needs the same set.
    pub fn documents(&self) -> Vec<String> {
        let mut uris: Vec<String> = self.documents.keys().cloned().collect();
        uris.sort();
        uris
    }

    /// The pipeline answer over the session's pool: the themed walk when
    /// the config carries a table, the theme-less boundary otherwise —
    /// the same entry points the CLI runs, never a parallel matcher,
    /// now deduped by the bounded cache (the cache's own differential
    /// contract: a hit hands back the same interned slices and the SAME
    /// diagnostics the miss stored; only the empty input errors, and
    /// errors are never cached).
    fn compiled(&self, raw: &str) -> Result<Compiled, CompileError> {
        match (&self.config.tokens, &self.walk) {
            (Some(table), Some(walk)) => self.cache.compile_theme_observed(
                raw,
                table,
                &self.config.compile,
                Arc::clone(walk),
            ),
            (Some(table), None) => self.cache.compile_theme(raw, table, &self.config.compile),
            (None, Some(walk)) => {
                self.cache
                    .compile_observed(raw, &self.config.compile, Arc::clone(walk))
            }
            (None, None) => self.cache.compile(raw, &self.config.compile),
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
    // The goto-definition answer
    // -----------------------------------------------------------------------

    /// The definition site the cursor's token class answers: the theme
    /// file's own declaration, byte-true in the wire's UTF-16 pairs.
    /// The identity rides the compiler's own vocabulary — the candidate
    /// under the cursor (the shared locator, the hover's own walk) must
    /// lower cleanly, its utility half must decompose onto a
    /// colour-group root whose remainder the riding table carries (the
    /// colour presentations' own composition walked backwards), and the
    /// site is the theme pass's provenance scan under the config's
    /// origin URI — or the wired host store's answer, which rides
    /// FIRST. Every miss is honest: a refused class, a non-token class,
    /// a token the file does not declare (the seeded palette names), a
    /// theme-less config, an unnamed origin — all answer `None`, never
    /// a guessed span. Deterministic and pure over the document map; an
    /// unknown URI answers `None`.
    pub fn definition(&self, uri: &str, line: u32, character: u32) -> Option<EngineDefinition> {
        let doc = self.documents.get(uri)?;
        let text = doc.text.as_str();
        let offset = offset_of(text, line, character)?;
        let (start, end) = candidate_span_at(text, offset)?;
        let raw = &text[start..end];
        // The compile gate rides ahead of every origin source: a
        // refused class carries no token — the diagnostics publish it,
        // the definition stays dark.
        let compiled = self.compiled(raw).ok()?;
        if !compiled.diagnostics.is_empty() {
            return None;
        }
        let table = self.config.tokens.as_ref()?;
        let token = token_under(raw, table)?;
        // The host store rides first (the wired override); the config's
        // own provenance path — the table's scan under the named
        // origin — the fallback.
        if let Some(store) = &self.origins {
            return store.definition(token);
        }
        let origin = table.origin(token)?;
        let theme_uri = self.config.theme_origin.as_ref()?;
        Some(EngineDefinition {
            uri: theme_uri.clone(),
            range: origin_range(origin),
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
    /// honest boundary [`aeolus_compile::compile`] itself keeps), and the keyword slots
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
    // The linked-editing answer
    // -----------------------------------------------------------------------

    /// The simultaneous-edit face (the wire's
    /// `textDocument/linkedEditingRange`): every textual occurrence of
    /// the cursor's candidate IN THE REQUESTING document, as the plain
    /// ranges the client edits together (the cursor's own occurrence
    /// rides, document order kept — the extraction seam's scan order).
    /// The identity is the highlights' TEXTUAL equality (a refused
    /// candidate's occurrences answer exactly like a lowered one's —
    /// no compile verdict riding; a variant-prefixed sibling is a
    /// DIFFERENT spelling), and the scope stays same-file: the
    /// cross-document fold is the RENAME face's answer, one face one
    /// scope. Deterministic and pure over the document map; an unknown
    /// URI and a cursor outside any candidate answer empty.
    pub fn linked_ranges(&self, uri: &str, line: u32, character: u32) -> Vec<EngineRange> {
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
                out.push(range_of(
                    text,
                    hit.offset + candidate.span.0,
                    hit.offset + candidate.span.1,
                ));
            }
        }
        out
    }

    // -----------------------------------------------------------------------
    // The cross-document references answer
    // -----------------------------------------------------------------------

    /// The workspace's occurrences of the candidate under the cursor:
    /// the document highlights' textual equality over EVERY open
    /// document — the engine's first cross-document answer. Each
    /// occurrence rides its document's URI beside its byte-true range:
    /// URI order across the documents (sorted, whatever the open order
    /// was — the answer is pure over the hash map), document order
    /// within each (the extraction seam's scan order). The identity is
    /// TEXTUAL — a refused class's occurrences answer exactly like a
    /// lowered one's (no compile verdict riding) — and the cursor's own
    /// occurrence rides the answer: a class spelling has no declaration
    /// site among the open documents (the theme CSS the config bakes is
    /// not one), so every occurrence is a reference and the wire's
    /// `includeDeclaration` context rides unanswered (the honest
    /// boundary the server documents). Deterministic and pure over the
    /// document map; an unknown URI and a cursor outside any candidate
    /// answer empty.
    pub fn references(&self, uri: &str, line: u32, character: u32) -> Vec<EngineReference> {
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
        // URI order across the map (the determinism pin), document
        // order within each — one scan per document, no parallel
        // matcher (the highlights' own walk, generalized).
        let mut uris: Vec<&String> = self.documents.keys().collect();
        uris.sort();
        let mut out = Vec::new();
        for key in uris {
            let text = self.documents[key].text.as_str();
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
                    out.push(EngineReference {
                        uri: key.clone(),
                        range,
                    });
                }
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

    /// The rename edits (the wire's `textDocument/rename` — the rename
    /// face's workspace upgrade): ONE edit per textual occurrence of
    /// the cursor's candidate over EVERY open document — the
    /// references' own fold over the rename edits, the new spelling
    /// riding every edit beside the URI of the document that carries
    /// it ([`EngineRenameEdit`]: URI order across the documents
    /// (sorted, whatever the open order was — the answer is pure over
    /// the hash map), document order within each). The face is TEXTUAL
    /// (the highlights' own contract, no compile verdict riding): the
    /// diagnostics channel tells the truth about the NEW spelling's
    /// compile the moment the didChange lands. An empty new name
    /// answers NO edits anywhere (never a deletion walk — the
    /// protocol's own validity clause, the quiet shape). Deterministic
    /// and pure over the document map; an unknown cursor URI and a
    /// cursor outside any candidate answer empty.
    pub fn rename(
        &self,
        uri: &str,
        line: u32,
        character: u32,
        new_name: &str,
    ) -> Vec<EngineRenameEdit> {
        if new_name.is_empty() {
            return Vec::new();
        }
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
        // URI order across the map (the determinism pin), document
        // order within each — one scan per document, no parallel
        // matcher (the references' own walk, generalized onto the
        // rename edits).
        let mut uris: Vec<&String> = self.documents.keys().collect();
        uris.sort();
        let mut out = Vec::new();
        for key in uris {
            let text = self.documents[key].text.as_str();
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
                    out.push(EngineRenameEdit {
                        uri: key.clone(),
                        range,
                        new_text: new_name.to_string(),
                    });
                }
            }
        }
        out
    }

    // -----------------------------------------------------------------------
    // The selection-range answer
    // -----------------------------------------------------------------------

    /// The expand-selection walk (the wire's
    /// `textDocument/selectionRange`): ONE parent chain per requested
    /// position, request order kept — the protocol's own 1:1 contract.
    /// The chain answers from the SAME locator the hover, the
    /// highlights, the references and the rename walk: a cursor inside
    /// a candidate answers the candidate's byte-true span
    /// ([`EngineSelectionRange::range`]) with the class string's own
    /// content span riding as its parent (the extractor's trimmed hit
    /// — the quotes stay the editor's territory); a cursor inside the
    /// string but between classes answers the content span alone (no
    /// candidate level, no invented inner range); a cursor outside any
    /// string answers the protocol's own minimal range — the
    /// position's zero-width caret, no parent. The zero-width caret is
    /// also the unknown URI's answer (one per requested position: the
    /// contract holds wherever the cursor was). Deterministic and pure
    /// over the document map.
    pub fn selection_ranges(
        &self,
        uri: &str,
        positions: &[EnginePosition],
    ) -> Vec<EngineSelectionRange> {
        // The zero-width caret: the protocol's own minimal range, at
        // the REQUESTED position (the wire's own units — no byte
        // round-trip can widen it).
        let caret = |position: &EnginePosition| EngineSelectionRange {
            range: EngineRange {
                start: *position,
                end: *position,
            },
            parent: None,
        };
        let Some(doc) = self.documents.get(uri) else {
            return positions.iter().map(caret).collect();
        };
        let text = doc.text.as_str();
        positions
            .iter()
            .map(|position| {
                let Some(offset) = offset_of(text, position.line, position.character) else {
                    return caret(position);
                };
                // The chain's two levels, one scan (the shared
                // locator's own shape): the candidate containing the
                // cursor, and the string content span containing it.
                let Some(hit) = scan_rust_strings(text)
                    .into_iter()
                    .find(|hit| offset >= hit.offset && offset < hit.offset + hit.text.len())
                else {
                    return caret(position);
                };
                let content = range_of(text, hit.offset, hit.offset + hit.text.len());
                let local = offset.saturating_sub(hit.offset);
                let (candidates, _) = parse_class_string(&hit.text);
                let Some(span) = candidates
                    .iter()
                    .find(|candidate| local >= candidate.span.0 && local < candidate.span.1)
                    .map(|candidate| {
                        (hit.offset + candidate.span.0, hit.offset + candidate.span.1)
                    })
                else {
                    // Between classes: the content span is the chain's
                    // head — no candidate level, no invented inner
                    // range.
                    return EngineSelectionRange {
                        range: content,
                        parent: None,
                    };
                };
                EngineSelectionRange {
                    range: range_of(text, span.0, span.1),
                    parent: Some(Box::new(EngineSelectionRange {
                        range: content,
                        parent: None,
                    })),
                }
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
    /// per-candidate compile (the engine's own `compiled_candidates` walk),
    /// document order (the extraction seam's scan order) — the wire's
    /// outline answer, deterministic and pure over the document map; an
    /// unknown URI answers empty.
    pub fn document_symbols(&self, uri: &str) -> Vec<EngineSymbol> {
        let Some(doc) = self.documents.get(uri) else {
            return Vec::new();
        };
        let text = doc.text.as_str();
        self.compiled_candidates(text, |raw, span, compiled| {
            self.outline_symbol(raw, span, text, compiled)
        })
    }

    // -----------------------------------------------------------------------
    // The cross-document workspace-symbol answer
    // -----------------------------------------------------------------------

    /// The workspace's outline: every candidate class across EVERY open
    /// document whose spelling matches the query — the document
    /// symbols' outline contract generalized over the whole map (the
    /// references fold's second consumer, the engine's second
    /// cross-document answer). The QUERY is the filter: a
    /// case-insensitive substring on the candidate's raw spelling (the
    /// variant prefixes included — `bg-` matches
    /// `hover:bg-primary/90`), and the EMPTY query filters nothing —
    /// every open candidate rides (the protocol's own shape: the query
    /// is a filter, an empty filter is the identity). Each match rides
    /// the outline entry (the spelling the NAME, its byte-true range,
    /// the legend's own verdict and the pipeline's own detail line —
    /// the shared `document_symbols` walk, no parallel scaffold) beside
    /// its document's URI: URI order across the documents (sorted,
    /// whatever the open order was — the answer is pure over the hash
    /// map), document order within each (the extraction seam's scan
    /// order). Deterministic and pure over the document map; a miss
    /// answers empty.
    pub fn workspace_symbols(&self, query: &str) -> Vec<EngineWorkspaceSymbol> {
        let needle = query.to_lowercase();
        // URI order across the map (the determinism pin), document
        // order within each — the outline walk per document, the
        // filter riding the fold.
        let mut uris: Vec<&String> = self.documents.keys().collect();
        uris.sort();
        let mut out = Vec::new();
        for key in uris {
            let text = self.documents[key].text.as_str();
            out.extend(self.compiled_candidates(text, |raw, span, compiled| {
                if !needle.is_empty() && !raw.to_lowercase().contains(&needle) {
                    return None;
                }
                let symbol = self.outline_symbol(raw, span, text, compiled)?;
                Some(EngineWorkspaceSymbol {
                    uri: key.clone(),
                    symbol,
                })
            }));
        }
        out
    }

    // -----------------------------------------------------------------------
    // The folding-range answer
    // -----------------------------------------------------------------------

    /// The document's folds (the wire's `textDocument/foldingRange`):
    /// ONE fold per extracted class string — the string's own content
    /// span (the extractor's trimmed hit — the quotes stay the editor's
    /// territory), document order (the extraction seam's scan order),
    /// each fold spanning the whole content (every candidate inside it
    /// rides the one range). The fold is a SHAPE-level answer — the
    /// highlights' textual family: no compile verdict rides, a refused
    /// candidate's string folds exactly like a lowered one's — and the
    /// answer set is CLOSED by the extraction seam's own pre-filter (a
    /// literal with no `-`/`:` token is not a class string, so it never
    /// folds): the server reports the class strings it knows, the
    /// client renders which folds it wants (the protocol's own
    /// contract). The multi-line raw-string call sites are the folds
    /// the editor actually collapses (the quoted walk stops at a
    /// newline, the raw-string form the multi-line shape).
    /// Deterministic and pure over the document map; an unknown URI
    /// answers empty.
    pub fn folding_ranges(&self, uri: &str) -> Vec<EngineRange> {
        let Some(doc) = self.documents.get(uri) else {
            return Vec::new();
        };
        let text = doc.text.as_str();
        scan_rust_strings(text)
            .into_iter()
            .map(|hit| range_of(text, hit.offset, hit.offset + hit.text.len()))
            .collect()
    }

    // -----------------------------------------------------------------------
    // The workspace-pull diagnostic answer
    // -----------------------------------------------------------------------

    /// The workspace's pull report (the wire's `workspace/diagnostic` —
    /// LSP 3.17's workspace-wide pull): every OPEN document's findings
    /// beside its result identity — URI order across the map (the
    /// cross-document answers' own determinism contract), the
    /// per-document walk the document pull rides generalized. The
    /// RESULT IDENTITY is the honest fingerprint: a 64-bit FNV-1a over
    /// the document's exact text (the module's own `fnv1a` walk below)
    /// — the same text
    /// answers the same id whatever the version tag rides (the
    /// equal-content re-save the byte-neutral no-op the retheme
    /// fast-path keeps), a text change answers a new one. The findings
    /// are pure over the text, so the content is the identity: nothing
    /// else can move the report. A `previous` pair
    /// (uri, id) whose id EQUALS the current one answers
    /// [`EngineWorkspaceOutcome::Unchanged`] — the client's cache is
    /// the report, nothing recomputes (the walk the pool makes cheap
    /// skipped wholesale); a stale id, or a uri the previous list
    /// never named, answers the full report — the engine never caches,
    /// the pull channel's own contract. Deterministic and pure over
    /// the document map; the previous entries for unopened documents
    /// ride unanswered (the map is the answer's scope).
    pub fn workspace_diagnostics(
        &self,
        previous: &[(&str, &str)],
    ) -> Vec<EngineWorkspaceDiagnostic> {
        let mut uris: Vec<&String> = self.documents.keys().collect();
        uris.sort();
        let mut out = Vec::with_capacity(uris.len());
        for key in uris {
            let doc = &self.documents[key];
            let result_id = format!("{:016x}", fnv1a(doc.text.as_bytes()));
            let outcome = if previous
                .iter()
                .any(|(uri, id)| *uri == key.as_str() && *id == result_id)
            {
                EngineWorkspaceOutcome::Unchanged
            } else {
                EngineWorkspaceOutcome::Full {
                    diagnostics: self.analyze(key),
                }
            };
            out.push(EngineWorkspaceDiagnostic {
                uri: key.clone(),
                version: doc.version,
                result_id,
                outcome,
            });
        }
        out
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

    /// One candidate's outline entry — the document-symbol answer's
    /// per-candidate half, SHARED with the workspace-symbol answer (the
    /// two outlines are one shape at two scopes; the scaffold lives
    /// once). The lowered candidate renders the compact CSS (the hint
    /// label's own rendering); the refused one the first finding's
    /// stable code. A sheet miss skips the candidate (`None` — the
    /// walker carries none of the boundaries).
    fn outline_symbol(
        &self,
        raw: &str,
        span: (usize, usize),
        text: &str,
        compiled: &Compiled,
    ) -> Option<EngineSymbol> {
        let (kind, detail) = if compiled.diagnostics.is_empty() {
            let sheet = match &self.config.tokens {
                Some(table) => emit_css_theme(raw, table, &self.config.compile),
                None => emit_css(raw, &self.config.compile),
            };
            let Ok(sheet) = sheet else {
                return None;
            };
            (EngineSemanticKind::Lowered, compact_css(&sheet))
        } else {
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
    }
}

// ---------------------------------------------------------------------------
// The position conversions (the LSP's UTF-16 model, owned ONCE here)
// ---------------------------------------------------------------------------

/// The 64-bit FNV-1a the workspace-pull result identity rides — the
/// one hash this crate owns, dependency-free by the no-manifest-add
/// discipline (the pool's entry identity lives in the compile crate's
/// `DynCache`; this one fingerprints the DOCUMENT TEXT the pull report
/// caches against, a fingerprint never keys a decision — it only names
/// one for the client).
fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

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

/// The token one lowered candidate's utility half names: the variant
/// stack's tail (the last `:` segment), the `/opacity` modifier
/// presentation stripped, decomposed onto the registry's colour-group
/// roots — the colour presentations' own composition walked backwards,
/// the SAME rows and roots, never a parallel matcher. The remainder
/// must verify through the riding table (`TokenTable::id`) — a class
/// the table cannot name (a non-colour root, an arbitrary value's own
/// colon) answers None, the honest miss.
fn token_under<'t>(raw: &'t str, table: &'t TokenTable) -> Option<&'t str> {
    let utility = raw.rsplit(':').next()?;
    let utility = utility.split('/').next()?;
    for row in ROWS {
        if !COLOUR_GROUPS.contains(&row.group) {
            continue;
        }
        let Some(rest) = utility
            .strip_prefix(row.root)
            .and_then(|tail| tail.strip_prefix('-'))
        else {
            continue;
        };
        if rest.is_empty() {
            continue;
        }
        if table.id(rest).is_some() {
            return Some(rest);
        }
    }
    None
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

/// Two tables with the SAME answers — the retheme's equality
/// fast-path. The comparison rides the table's own public getters
/// (tier, names and the resolved rgba per id beside the pair surface):
/// two tables equal under it lower to byte-identical programs (the
/// tier arms choose the same `Val` shapes, the ids resolve the same
/// colours), which is the contract the no-op save's `Unchanged` needs.
/// The tier rides FIRST: a baked and a runtime table over the same css
/// carry the same colours but answer different `Val` forms (the
/// literal beside the token), so content equality is tier equality
/// too.
fn tokens_equal(riding: &TokenTable, candidate: &TokenTable) -> bool {
    if riding.tier() != candidate.tier() || riding.len() != candidate.len() {
        return false;
    }
    for (slot, riding_name) in riding.names().enumerate() {
        let Some(candidate_id) = candidate.id(riding_name) else {
            return false;
        };
        if slot != usize::from(candidate_id) {
            // Sorted-name order is the id contract: the same names in
            // the same order resolve the same ids, and the same ids
            // are what a pooled row's `Val::Tok`/`Val::ModePair`
            // carries.
            return false;
        }
        let id = candidate_id;
        if riding.light_rgba(id) != candidate.light_rgba(id) {
            return false;
        }
        let riding_dark = riding.has_dark(id);
        if riding_dark != candidate.has_dark(id) {
            return false;
        }
        if riding_dark && riding.dark_rgba(id) != candidate.dark_rgba(id) {
            return false;
        }
    }
    if riding.pairs() != candidate.pairs() {
        return false;
    }
    for slot in 0..riding.pairs() {
        let Ok(index) = u16::try_from(slot) else {
            // `Val::ModePair`'s slot is a u16 (the IR's own bound): a
            // pair table beyond it is outside the model the equality
            // serves.
            return false;
        };
        if riding.pair_at(index) != candidate.pair_at(index) {
            return false;
        }
    }
    true
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
