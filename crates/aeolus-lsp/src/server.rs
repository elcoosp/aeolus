//! The wire half: the JSON-RPC state machine and its stdio loop.
//!
//! The STATE MACHINE ([`handle_message`] over a [`Session`]) is the
//! crate's own surface — deterministic, testable without a pipe (the
//! tests drive it message by message). The stdio loop
//! ([`run_stdio`]) is the transport's thin shell: one thread per side
//! (the `lsp-server` pairing, no async runtime on the editor-adjacent
//! path), the engine behind it answering from the same pipeline the
//! CLI runs.

use lsp_server::{Connection, Message, Notification, Request, RequestId, Response};
use lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, CodeActionParams,
    CodeActionProviderCapability, Color, ColorInformation, ColorPresentation,
    ColorPresentationParams, ColorProviderCapability, CompletionItem, CompletionItemKind,
    CompletionList, CompletionOptions, CompletionParams, DiagnosticOptions,
    DiagnosticServerCapabilities, DidChangeTextDocumentParams, DidChangeWatchedFilesParams,
    DidCloseTextDocumentParams, DidOpenTextDocumentParams, DocumentColorParams,
    DocumentDiagnosticParams, DocumentDiagnosticReport, DocumentDiagnosticReportResult,
    DocumentFormattingParams, DocumentHighlight, DocumentHighlightParams,
    DocumentRangeFormattingParams, DocumentSymbolParams, ExecuteCommandOptions,
    ExecuteCommandParams, FoldingRange, FoldingRangeParams, FoldingRangeProviderCapability,
    FullDocumentDiagnosticReport, GotoDefinitionParams, Hover, HoverContents,
    HoverProviderCapability, InitializeResult, InlayHint, InlayHintKind, InlayHintLabel,
    InlayHintOptions, InlayHintParams, InlayHintServerCapabilities, InlayHintTooltip,
    LinkedEditingRangeParams, LinkedEditingRangeServerCapabilities, LinkedEditingRanges, Location,
    MarkupContent, MarkupKind, NumberOrString, OneOf, PrepareRenameResponse,
    PublishDiagnosticsParams, ReferenceParams, RelatedFullDocumentDiagnosticReport, RenameParams,
    SelectionRange, SelectionRangeParams, SelectionRangeProviderCapability, SemanticToken,
    SemanticTokenType, SemanticTokens, SemanticTokensFullOptions, SemanticTokensLegend,
    SemanticTokensOptions, SemanticTokensParams, SemanticTokensRangeParams,
    SemanticTokensServerCapabilities, ServerCapabilities, ServerInfo, SymbolKind,
    TextDocumentPositionParams, TextDocumentSyncCapability, TextDocumentSyncKind,
    TextDocumentSyncOptions, TextEdit, Uri, WorkDoneProgressOptions, WorkspaceEdit,
    WorkspaceSymbolParams,
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use crate::RethemeOutcome;
use crate::engine::{AeolusEngine, EngineConfig};
use crate::transport_observer::{NotificationOutcome, RequestOutcome, TransportObserver};

// ---------------------------------------------------------------------------
// The session state
// ---------------------------------------------------------------------------

/// One server session: the engine behind it and the lifecycle state the
/// loop needs. The editor's view of the session is the wire contract;
/// the engine's view is the document map.
pub struct Session {
    /// The engine (the pipeline answers).
    engine: AeolusEngine,
    /// Whether the client completed the initialize handshake (requests
    /// before it ride the protocol's own quiet-drop contract).
    initialized: bool,
    /// Whether the client sent the shutdown request (the loop's exit
    /// is the exit NOTIFICATION, per the protocol).
    shutdown: bool,
    /// The wired transport observer, `None` the every-constructor
    /// default — the observability face the wire half rides
    /// ([`Session::with_observer`]). The events are the state
    /// machine's own answers (the outgoing verdicts), never opinions;
    /// a session wired before its first
    /// [`handle_message`](crate::handle_message) streams the whole
    /// loop's traffic.
    observer: Option<Arc<dyn TransportObserver>>,
    /// The wired theme loader, `None` the every-constructor default —
    /// the theme lifecycle's face ([`Session::with_theme_loader`]): the
    /// watched-files notification that names the loader's URI re-themes
    /// the engine (a session without one rides the notification
    /// silently, the session's answers staying the construction's).
    theme: Option<Arc<dyn crate::ThemeLoader>>,
}

impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session")
            .field("engine", &self.engine)
            .field("initialized", &self.initialized)
            .field("shutdown", &self.shutdown)
            // The wired observer is a host handle, not printable state —
            // the table Debug's own rule: the wired-ness only.
            .field("observer", &self.observer.is_some())
            .field("theme", &self.theme.as_ref().map(crate::ThemeLoader::uri))
            .finish()
    }
}

impl Session {
    /// A session over one engine configuration.
    pub fn new(config: EngineConfig) -> Self {
        Self {
            engine: AeolusEngine::new(config),
            initialized: false,
            shutdown: false,
            observer: None,
            theme: None,
        }
    }

    /// Wire the transport observability face after construction (the
    /// table's own shape): every later
    /// [`handle_message`](crate::handle_message) reports the machine's
    /// verdict to the observer — the requests' answers and quiet
    /// drops, the notifications' publish/exit/silence verdicts — while
    /// the outgoing messages stay the plain session's, byte for byte
    /// (the differential pins it). Sharing rides the `Arc` blanket:
    /// one handle totals several sessions.
    pub fn with_observer(mut self, observer: impl TransportObserver + 'static) -> Self {
        self.observer = Some(Arc::new(observer));
        self
    }

    /// Wire the theme lifecycle's loader (the reload seam's wire half,
    /// chained after `new` like the observer faces): the loader owns
    /// ONE watched URI, and the `workspace/didChangeWatchedFiles`
    /// notification naming it re-themes the session — `load` runs, the
    /// typed [`ThemeError`](crate::ThemeError) failure keeps the
    /// riding table (nothing republishes — nothing changed), an equal-
    /// content success is the byte-neutral no-op
    /// ([`RethemeOutcome::Unchanged`]), and a real swap
    /// ([`RethemeOutcome::Swapped`]) republishes EVERY open document's
    /// diagnostics (URI order — the push channel's own determinism).
    /// A watched-files event for a URI the loader does not own rides
    /// silently. Without a loader the notification stays the quiet
    /// drop every unhandled notification is.
    pub fn with_theme_loader(mut self, loader: impl crate::ThemeLoader + 'static) -> Self {
        // The loader's URI is the goto-definition face's origin too: the
        // file the session's tables come from is the file the token
        // definitions jump into. The face stays dark until the FIRST
        // load lands (the built-in bake is no file); the watched-file
        // handler re-notes on every load (idempotent — one loader, one
        // file).
        self.engine.note_theme_origin(loader.uri());
        self.theme = Some(Arc::new(loader));
        self
    }
}

/// One handled message: the outgoing messages the handler produced
/// (responses and publish notifications, wire order kept) and whether
/// the session ENDS after this message (the exit notification's own
/// contract).
#[derive(Debug, Default)]
pub struct Handled {
    /// The messages to send, in order.
    pub outgoing: Vec<Message>,
    /// Whether the session ends here.
    pub exit: bool,
}

// ---------------------------------------------------------------------------
// The state machine
// ---------------------------------------------------------------------------

/// Handle one wire message against the session — the server's whole
/// behaviour, testable message by message. Unhandled methods ride
/// silently (the protocol's own quiet-drop for capabilities the server
/// does not advertise); the exit notification reports through
/// [`Handled::exit`].
///
/// A session wired with an observer
/// ([`Session::with_observer`]) streams the machine's own verdict
/// per message — [`RequestOutcome`] for requests,
/// [`NotificationOutcome`] for notifications — read off the SAME
/// [`Handled`] the caller receives (outgoing messages rode, or the
/// protocol's own quiet-drop stayed silent). The unwired session
/// answers byte-identically: the channel adds a verdict, never a
/// decision.
pub fn handle_message(session: &mut Session, message: Message) -> Handled {
    match message {
        Message::Request(request) => {
            // The method rides the event (the wire's own correlation);
            // the clone is the observed path's only cost — an unwired
            // session keeps its zero-allocation loop.
            let observed = session.observer.clone();
            let method = observed.is_some().then(|| request.method.clone());
            let handled = handle_request(session, request);
            if let (Some(observer), Some(method)) = (observed.as_deref(), method.as_deref()) {
                observer.on_request(method, request_outcome(&handled));
            }
            handled
        }
        Message::Notification(notification) => {
            let observed = session.observer.clone();
            let method = observed.is_some().then(|| notification.method.clone());
            let handled = handle_notification(session, notification);
            if let (Some(observer), Some(method)) = (observed.as_deref(), method.as_deref()) {
                observer.on_notification(method, notification_outcome(&handled));
            }
            handled
        }
        Message::Response(_) => Handled::default(), // the server never drives a client request
    }
}

/// The request verdict read off the machine's own [`Handled`]: a
/// response rode (`Answered`) or the wire stayed silent
/// (`QuietDropped` — the pre-handshake gate, the post-shutdown gate,
/// the unadvertised method).
fn request_outcome(handled: &Handled) -> RequestOutcome {
    if handled.outgoing.is_empty() {
        RequestOutcome::QuietDropped
    } else {
        RequestOutcome::Answered
    }
}

/// The notification verdict read off the machine's own [`Handled`]:
/// the exit contract (`Exit`), the diagnostics push (`Published`) or
/// the quiet-drop silence (`QuietDropped`).
fn notification_outcome(handled: &Handled) -> NotificationOutcome {
    if handled.exit {
        NotificationOutcome::Exit
    } else if handled.outgoing.is_empty() {
        NotificationOutcome::QuietDropped
    } else {
        NotificationOutcome::Published
    }
}

/// One request: initialize, hover, shutdown. An unknown method answers
/// nothing (the capabilities never advertised it), and a session the
/// client has shut down rides the same quiet-drop — the protocol's own
/// contract (the one exception is a repeated `shutdown`, which still
/// answers: a client waiting on it must not hang).
fn handle_request(session: &mut Session, request: Request) -> Handled {
    if session.shutdown && request.method != "shutdown" {
        return Handled::default();
    }
    match request.method.as_str() {
        "initialize" => {
            session.initialized = true;
            let result = InitializeResult {
                capabilities: server_capabilities(),
                server_info: Some(ServerInfo {
                    name: "aeolus".to_string(),
                    version: Some(env!("CARGO_PKG_VERSION").to_string()),
                }),
            };
            let result = serde_json::to_value(result).unwrap_or(Value::Null);
            respond(request.id, Some(result))
        }
        "shutdown" => {
            session.shutdown = true;
            respond(request.id, None)
        }
        "textDocument/hover" => {
            if !session.initialized {
                return Handled::default();
            }
            let answer = serde_json::from_value::<TextDocumentPositionParams>(request.params)
                .ok()
                .and_then(|params| {
                    let line = params.position.line;
                    let character = params.position.character;
                    session
                        .engine
                        .hover(params.text_document.uri.as_str(), line, character)
                });
            let result = answer.map(|hover| {
                serde_json::to_value(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: hover.contents,
                    }),
                    range: hover.range.map(wire_range),
                })
                .unwrap_or(Value::Null)
            });
            respond(request.id, result)
        }
        "textDocument/definition" => {
            if !session.initialized {
                return Handled::default();
            }
            // The token goto-definition face: the cursor's token class
            // answers the theme file's own declaration site — the
            // engine's provenance path (or the wired store's), byte-true
            // in the UTF-16 pairs. A URI the `Uri` parser rejects
            // answers the protocol's own null (the publish half's own
            // rule: the wire gave it, no editor can have named it
            // either).
            let answer = serde_json::from_value::<GotoDefinitionParams>(request.params)
                .ok()
                .and_then(|params| {
                    let position = params.text_document_position_params.position;
                    session.engine.definition(
                        params
                            .text_document_position_params
                            .text_document
                            .uri
                            .as_str(),
                        position.line,
                        position.character,
                    )
                });
            let result = answer.map(|definition| {
                // A URI the `Uri` parser rejects answers the protocol's
                // own null (the publish half's own rule: the wire gave
                // it, no editor can have named it either).
                let Ok(uri) = lsp_types::Uri::from_str(&definition.uri) else {
                    return Value::Null;
                };
                serde_json::to_value(Location {
                    uri,
                    range: wire_range(definition.range),
                })
                .unwrap_or(Value::Null)
            });
            respond(request.id, result)
        }
        "textDocument/documentColor" => {
            if !session.initialized {
                return Handled::default();
            }
            let answer = serde_json::from_value::<DocumentColorParams>(request.params)
                .ok()
                .map(|params| {
                    session
                        .engine
                        .document_colours(params.text_document.uri.as_str())
                        .iter()
                        .map(|swatch| ColorInformation {
                            range: wire_range(swatch.range),
                            // The protocol's colour model is rgba
                            // itself (0..1 fractions) — the resolved
                            // bytes map channel for channel.
                            color: Color {
                                red: f32::from(swatch.rgba[0]) / 255.0,
                                green: f32::from(swatch.rgba[1]) / 255.0,
                                blue: f32::from(swatch.rgba[2]) / 255.0,
                                alpha: f32::from(swatch.rgba[3]) / 255.0,
                            },
                        })
                        .collect::<Vec<_>>()
                });
            let result = answer.map_or(
                Some(Value::Null), // malformed params: the protocol's own empty answer
                |swatches| Some(serde_json::to_value(swatches).unwrap_or(Value::Null)),
            );
            respond(request.id, result)
        }
        "textDocument/documentDiagnostic" => {
            if !session.initialized {
                return Handled::default();
            }
            // The pull model (LSP 3.17's fifth request): the engine's
            // analyze walk IS the report — the same findings the push
            // channel publishes, recomputed on demand (no result ID:
            // the engine never caches, so the answer is always a full
            // report).
            let answer = serde_json::from_value::<DocumentDiagnosticParams>(request.params)
                .ok()
                .map(|params| {
                    let items = session
                        .engine
                        .analyze(params.text_document.uri.as_str())
                        .iter()
                        .map(wire_diagnostic)
                        .collect::<Vec<_>>();
                    DocumentDiagnosticReportResult::Report(DocumentDiagnosticReport::Full(
                        RelatedFullDocumentDiagnosticReport {
                            related_documents: None,
                            full_document_diagnostic_report: FullDocumentDiagnosticReport {
                                result_id: None,
                                items,
                            },
                        },
                    ))
                });
            let result = answer.map_or(
                Some(Value::Null), // malformed params: the protocol's own empty answer
                |report| Some(serde_json::to_value(report).unwrap_or(Value::Null)),
            );
            respond(request.id, result)
        }
        "textDocument/completion" => {
            if !session.initialized {
                return Handled::default();
            }
            let answer = serde_json::from_value::<CompletionParams>(request.params)
                .ok()
                .map(|params| {
                    let position = params.text_document_position.position;
                    session.engine.complete(
                        params.text_document_position.text_document.uri.as_str(),
                        position.line,
                        position.character,
                    )
                });
            let result = answer.map_or(
                Some(Value::Null), // malformed params: the protocol's own empty answer
                |items| {
                    let ranked = items
                        .iter()
                        .map(|item| CompletionItem {
                            label: item.label.clone(),
                            kind: Some(match item.kind {
                                crate::EngineCompletionKind::Utility => {
                                    CompletionItemKind::PROPERTY
                                }
                                crate::EngineCompletionKind::Token => CompletionItemKind::VALUE,
                                crate::EngineCompletionKind::Variant => CompletionItemKind::KEYWORD,
                            }),
                            detail: Some(item.detail.clone()),
                            // The registry's canonical order as the sort
                            // key: the editor renders the answer's own
                            // order, never its label-alphabetical one.
                            sort_text: Some(format!("{:04}", item.rank)),
                            ..CompletionItem::default()
                        })
                        .collect();
                    Some(
                        serde_json::to_value(CompletionList {
                            is_incomplete: false,
                            items: ranked,
                        })
                        .unwrap_or(Value::Null),
                    )
                },
            );
            respond(request.id, result)
        }
        "textDocument/semanticTokens/full" => {
            if !session.initialized {
                return Handled::default();
            }
            // The highlighter feed (LSP 3.16's semantic tokens): the
            // engine's absolute tokens, in document order, onto the
            // protocol's delta encoding (one legend, the wire's own).
            let answer = serde_json::from_value::<SemanticTokensParams>(request.params)
                .ok()
                .map(|params| {
                    wire_semantic_tokens(
                        &session
                            .engine
                            .semantic_tokens(params.text_document.uri.as_str()),
                    )
                });
            let result = answer.map_or(
                Some(Value::Null), // malformed params: the protocol's own empty answer
                |tokens| Some(serde_json::to_value(tokens).unwrap_or(Value::Null)),
            );
            respond(request.id, result)
        }
        "textDocument/semanticTokens/range" => {
            if !session.initialized {
                return Handled::default();
            }
            // The highlighter feed's SELECTION-SCOPED form (LSP 3.16's
            // semanticTokens/range): the full-token walk restricted to
            // the candidates the requested range touches — a candidate
            // whose span intersects the range rides its token whole (a
            // partial selection still names the token), a candidate
            // outside stays out and never compiles (the walk's win over
            // the full legend). The answer rides the SAME delta encoder
            // the full arm answers through — each response encodes from
            // its own start (row 0), so the restricted array's first
            // token carries its absolute line as the delta (the
            // encoding's contract, the re-baseline the battery pins); a
            // reversed range rides the engine's honest empty.
            let answer = serde_json::from_value::<SemanticTokensRangeParams>(request.params)
                .ok()
                .map(|params| {
                    wire_semantic_tokens(&session.engine.semantic_tokens_range(
                        params.text_document.uri.as_str(),
                        params.range.start.line,
                        params.range.start.character,
                        params.range.end.line,
                        params.range.end.character,
                    ))
                });
            let result = answer.map_or(
                Some(Value::Null), // malformed params: the protocol's own empty answer
                |tokens| Some(serde_json::to_value(tokens).unwrap_or(Value::Null)),
            );
            respond(request.id, result)
        }
        "textDocument/colorPresentation" => {
            if !session.initialized {
                return Handled::default();
            }
            // The colour provider's paired half (the picker's spellings):
            // the protocol's 0..1 fractions map back onto the byte rgba
            // the swatches came from (the same channel-for-channel
            // mapping, inverted), the table walk answers the spellings,
            // each riding as an edit over the range the picker names.
            let answer = serde_json::from_value::<ColorPresentationParams>(request.params)
                .ok()
                .map(|params| {
                    let byte = |fraction: f32| -> u8 {
                        (fraction * 255.0).round().clamp(0.0, 255.0) as u8
                    };
                    let rgba = [
                        byte(params.color.red),
                        byte(params.color.green),
                        byte(params.color.blue),
                        byte(params.color.alpha),
                    ];
                    session
                        .engine
                        .colour_presentations(rgba)
                        .iter()
                        .map(|presentation| ColorPresentation {
                            label: presentation.label.clone(),
                            // The swap writes the class over the
                            // requested range (the colour's own).
                            text_edit: Some(TextEdit {
                                range: params.range,
                                new_text: presentation.label.clone(),
                            }),
                            ..ColorPresentation::default()
                        })
                        .collect::<Vec<_>>()
                });
            let result = answer.map_or(
                Some(Value::Null), // malformed params: the protocol's own empty answer
                |items| Some(serde_json::to_value(items).unwrap_or(Value::Null)),
            );
            respond(request.id, result)
        }
        "textDocument/inlayHint" => {
            if !session.initialized {
                return Handled::default();
            }
            // The inline answers (LSP 3.17's inlayHint): the engine's
            // hints are the whole document's — the params' visible range
            // is accepted and answered document-wide (no incremental
            // model, the pull diagnostics' own contract: the engine
            // recomputes, the answer is always full).
            let answer = serde_json::from_value::<InlayHintParams>(request.params)
                .ok()
                .map(|params| {
                    session
                        .engine
                        .inlay_hints(params.text_document.uri.as_str())
                        .iter()
                        .map(|hint| InlayHint {
                            position: lsp_types::Position {
                                line: hint.position.line,
                                character: hint.position.character,
                            },
                            label: InlayHintLabel::String(hint.label.clone()),
                            // The type-annotation kind: the hint shows
                            // what the class IS — the CSS it lowers to.
                            kind: Some(InlayHintKind::TYPE),
                            tooltip: Some(InlayHintTooltip::MarkupContent(MarkupContent {
                                kind: MarkupKind::Markdown,
                                value: hint.tooltip.clone(),
                            })),
                            text_edits: None,
                            padding_left: None,
                            padding_right: None,
                            data: None,
                        })
                        .collect::<Vec<_>>()
                });
            let result = answer.map_or(
                Some(Value::Null), // malformed params: the protocol's own empty answer
                |hints| Some(serde_json::to_value(hints).unwrap_or(Value::Null)),
            );
            respond(request.id, result)
        }
        "textDocument/codeAction" => {
            if !session.initialized {
                return Handled::default();
            }
            // The quick fixes (LSP 3.16's codeAction): the engine's
            // answer for the requested range's start (the cursor the
            // editor invokes the action on) — each finding riding the
            // diagnostic it repairs, the correction as the edit over
            // the finding's own range.
            let answer = serde_json::from_value::<CodeActionParams>(request.params)
                .ok()
                .map(|params| {
                    let position = params.range.start;
                    session
                        .engine
                        .code_actions(
                            params.text_document.uri.as_str(),
                            position.line,
                            position.character,
                        )
                        .iter()
                        .map(|action| {
                            // The edit over the finding's own range (the
                            // wire's TextEdit pair, the mapping the push
                            // channel's ranges ride).
                            let edit_range = wire_range(action.diagnostic.range);
                            CodeActionOrCommand::CodeAction(CodeAction {
                                title: action.title.clone(),
                                // The kind the editors filter on: the
                                // correction is a quick fix, never a
                                // refactor.
                                kind: Some(CodeActionKind::QUICKFIX),
                                // The finding the action repairs (the
                                // editor's filter key — the same
                                // mapping the push channel publishes).
                                diagnostics: Some(vec![wire_diagnostic(&action.diagnostic)]),
                                edit: Some(WorkspaceEdit {
                                    changes: Some(HashMap::from([(
                                        params.text_document.uri.clone(),
                                        vec![TextEdit {
                                            range: edit_range,
                                            new_text: action.replacement.clone(),
                                        }],
                                    )])),
                                    ..WorkspaceEdit::default()
                                }),
                                ..CodeAction::default()
                            })
                        })
                        .collect::<Vec<_>>()
                });
            let result = answer.map_or(
                Some(Value::Null), // malformed params: the protocol's own empty answer
                |actions| Some(serde_json::to_value(actions).unwrap_or(Value::Null)),
            );
            respond(request.id, result)
        }
        "textDocument/documentHighlight" => {
            if !session.initialized {
                return Handled::default();
            }
            // The click-to-find answer (LSP 3.16's documentHighlight):
            // the engine's occurrences of the cursor's candidate, each
            // riding its byte-true range (no kind: a plain highlight,
            // the honest shape).
            let answer = serde_json::from_value::<DocumentHighlightParams>(request.params)
                .ok()
                .map(|params| {
                    let position = params.text_document_position_params.position;
                    session
                        .engine
                        .document_highlights(
                            params
                                .text_document_position_params
                                .text_document
                                .uri
                                .as_str(),
                            position.line,
                            position.character,
                        )
                        .iter()
                        .map(|highlight| DocumentHighlight {
                            range: wire_range(highlight.range),
                            kind: None,
                        })
                        .collect::<Vec<_>>()
                });
            let result = answer.map_or(
                Some(Value::Null), // malformed params: the protocol's own empty answer
                |highlights| Some(serde_json::to_value(highlights).unwrap_or(Value::Null)),
            );
            respond(request.id, result)
        }
        "textDocument/linkedEditingRange" => {
            if !session.initialized {
                return Handled::default();
            }
            // The simultaneous-edit answer (LSP 3.16's
            // linkedEditingRange): the engine's same-file occurrences
            // of the cursor's candidate, each riding its byte-true
            // range — the client edits them together. No
            // `word_pattern` rides: the ranges ARE the contract (the
            // engine's textual identity needs no regex re-statement).
            let answer = serde_json::from_value::<LinkedEditingRangeParams>(request.params)
                .ok()
                .map(|params| {
                    let position = params.text_document_position_params.position;
                    LinkedEditingRanges {
                        ranges: session
                            .engine
                            .linked_ranges(
                                params
                                    .text_document_position_params
                                    .text_document
                                    .uri
                                    .as_str(),
                                position.line,
                                position.character,
                            )
                            .iter()
                            .map(|range| wire_range(*range))
                            .collect(),
                        word_pattern: None,
                    }
                });
            let result = answer.map_or(
                Some(Value::Null), // malformed params: the protocol's own empty answer
                |ranges| Some(serde_json::to_value(ranges).unwrap_or(Value::Null)),
            );
            respond(request.id, result)
        }
        "textDocument/references" => {
            if !session.initialized {
                return Handled::default();
            }
            // The workspace's occurrences (LSP 3.16's references): the
            // engine's first cross-document answer — the highlights'
            // textual equality over every open document, each riding its
            // `Location` (the URI the wire gave, the range it jumps to),
            // URI order across the map. The `includeDeclaration` context
            // rides unanswered: a class spelling has no declaration site
            // among the open documents (the theme CSS the config bakes
            // is not one), so every occurrence IS a reference — the
            // boundary the engine's rustdoc carries. A URI the `Uri`
            // parser rejects skips its quad (the publish half's own
            // rule: the wire gave it, no editor can have named it
            // either).
            let answer = serde_json::from_value::<ReferenceParams>(request.params)
                .ok()
                .map(|params| {
                    let position = params.text_document_position.position;
                    session
                        .engine
                        .references(
                            params.text_document_position.text_document.uri.as_str(),
                            position.line,
                            position.character,
                        )
                        .into_iter()
                        .filter_map(|reference| {
                            let uri = Uri::from_str(&reference.uri).ok()?;
                            Some(Location {
                                uri,
                                range: wire_range(reference.range),
                            })
                        })
                        .collect::<Vec<_>>()
                });
            let result = answer.map_or(
                Some(Value::Null), // malformed params: the protocol's own empty answer
                |locations| Some(serde_json::to_value(locations).unwrap_or(Value::Null)),
            );
            respond(request.id, result)
        }
        "textDocument/formatting" => {
            if !session.initialized {
                return Handled::default();
            }
            // The class-sort seam's editor face (the tooling-parity
            // line's "class sorting" half at the editor): the engine's
            // canonical-order edits — one TextEdit per out-of-order
            // class string, the string content's byte-true range plus
            // the sort seam's canonical join. The params' formatting
            // options (tab size, spaces) are accepted and left to the
            // engine's shape-level walk: the canonical order is a
            // registry fact, not an indentation choice.
            let answer = serde_json::from_value::<DocumentFormattingParams>(request.params)
                .ok()
                .map(|params| {
                    session
                        .engine
                        .format(params.text_document.uri.as_str())
                        .iter()
                        .map(|edit| TextEdit {
                            range: wire_range(edit.range),
                            new_text: edit.new_text.clone(),
                        })
                        .collect::<Vec<_>>()
                });
            let result = answer.map_or(
                Some(Value::Null), // malformed params: the protocol's own empty answer
                |edits| Some(serde_json::to_value(edits).unwrap_or(Value::Null)),
            );
            respond(request.id, result)
        }
        "textDocument/rangeFormatting" => {
            if !session.initialized {
                return Handled::default();
            }
            // The sort seam's SELECTION-SCOPED form (LSP 3.12's
            // rangeFormatting): the formatting request's own walk
            // restricted to the class strings the requested range
            // touches — a string whose span intersects the range rides
            // its WHOLE-string edit (a partial selection still names
            // the string, the sort never emits a fragment), a string
            // outside the range stays out, an already-canonical string
            // answers nothing wherever the range sits. The params'
            // formatting options are accepted and left to the engine's
            // shape-level walk (the canonical order is a registry fact,
            // not an indentation choice — the formatting arm's own
            // rule); a reversed range rides the engine's honest empty.
            let answer = serde_json::from_value::<DocumentRangeFormattingParams>(request.params)
                .ok()
                .map(|params| {
                    let range = params.range;
                    session
                        .engine
                        .format_range(
                            params.text_document.uri.as_str(),
                            range.start.line,
                            range.start.character,
                            range.end.line,
                            range.end.character,
                        )
                        .iter()
                        .map(|edit| TextEdit {
                            range: wire_range(edit.range),
                            new_text: edit.new_text.clone(),
                        })
                        .collect::<Vec<_>>()
                });
            let result = answer.map_or(
                Some(Value::Null), // malformed params: the protocol's own empty answer
                |edits| Some(serde_json::to_value(edits).unwrap_or(Value::Null)),
            );
            respond(request.id, result)
        }
        "workspace/executeCommand" => {
            if !session.initialized {
                return Handled::default();
            }
            // The server's command surface (LSP 3.16's
            // workspace/executeCommand): the CLOSED command table the
            // capability advertises — the sort face invoked as a
            // command, and the token-rename fold beside it. An unknown
            // command and a malformed argument answer the protocol's
            // own null — the state machine's quiet vocabulary (no
            // invented command runs, the honest boundary the table's
            // closedness keeps); the known ones ride the engine's own
            // answers byte for byte.
            let Ok(params) = serde_json::from_value::<ExecuteCommandParams>(request.params) else {
                return respond(request.id, Some(Value::Null));
            };
            match params.command.as_str() {
                // The class-sort seam's editor face as a command: the
                // first argument names the URI, the edits the
                // formatting request answers ride back.
                "aeolus.sortDocument" => {
                    let Some(argument) = params.arguments.first().and_then(|value| value.as_str())
                    else {
                        return respond(request.id, Some(Value::Null));
                    };
                    let edits = session
                        .engine
                        .format(argument)
                        .iter()
                        .map(|edit| TextEdit {
                            range: wire_range(edit.range),
                            new_text: edit.new_text.clone(),
                        })
                        .collect::<Vec<_>>();
                    respond(
                        request.id,
                        Some(serde_json::to_value(edits).unwrap_or(Value::Null)),
                    )
                }
                // The token-rename fold as a command: the arguments
                // ride [uri, line, character, newName] — a URI string,
                // the two position numbers, the new token name. A
                // missing or mistyped argument answers the protocol's
                // own null (the quiet vocabulary); a well-formed
                // invocation answers the rename face's own changes map
                // — the engine's token fold, byte for byte.
                "aeolus.renameToken" => {
                    let Some((uri, line, character, new_name)) =
                        rename_token_arguments(&params.arguments)
                    else {
                        return respond(request.id, Some(Value::Null));
                    };
                    let edits = session
                        .engine
                        .rename_token(&uri, line, character, &new_name);
                    respond(request.id, Some(changes_map(&edits)))
                }
                _ => respond(request.id, Some(Value::Null)),
            }
        }
        "textDocument/prepareRename" => {
            if !session.initialized {
                return Handled::default();
            }
            // The rename face's first half: the renameable span under
            // the cursor — the candidate's own byte-true range, the
            // full spelling the highlights' raw-text contract names. A
            // cursor outside any candidate answers null (the protocol's
            // cannot-rename), the same shape a malformed-params request
            // rides.
            let answer = serde_json::from_value::<TextDocumentPositionParams>(request.params)
                .ok()
                .and_then(|params| {
                    let position = params.position;
                    session.engine.prepare_rename(
                        params.text_document.uri.as_str(),
                        position.line,
                        position.character,
                    )
                })
                .map(|range| PrepareRenameResponse::Range(wire_range(range)));
            let result = answer.map_or(
                Some(Value::Null), // malformed params and the no-candidate cursor: the protocol's own null
                |span| Some(serde_json::to_value(span).unwrap_or(Value::Null)),
            );
            respond(request.id, result)
        }
        "textDocument/rename" => {
            if !session.initialized {
                return Handled::default();
            }
            // The rename face's second half (the workspace upgrade): one
            // edit per textual occurrence of the cursor's candidate over
            // EVERY open document (the references' own fold — the
            // highlights' walk generalized), the new spelling riding
            // every edit, the engine's own empty-name guard keeping the
            // deletion walk out. The FULL changes map answers: one key
            // per carrying document (the edit lists ride URI order, the
            // engine's own grouping), a URI the `Uri` parser rejects
            // skipping its key (the references half's own rule) — never
            // the requesting URI's edits alone, the workspace's edit.
            let answer = serde_json::from_value::<RenameParams>(request.params)
                .ok()
                .map(|params| {
                    let position = params.text_document_position.position;
                    let uri = params.text_document_position.text_document.uri;
                    let edits = session.engine.rename(
                        uri.as_str(),
                        position.line,
                        position.character,
                        &params.new_name,
                    );
                    // The shared fold (the rename command's own map — one
                    // fold, two faces, no parallel edit shape; the keys
                    // ride the engine's own URI order, the bytes
                    // deterministic).
                    changes_map(&edits)
                });
            // Malformed params answer the wire's own NULL RESULT (the
            // protocol's empty answer — the battery's pinned shape), a
            // well-formed one the fold's map.
            let result = answer.map_or(Some(Value::Null), Some);
            respond(request.id, result)
        }
        "textDocument/documentSymbol" => {
            if !session.initialized {
                return Handled::default();
            }
            // The outline face (LSP 3.16's documentSymbol): the engine's
            // whole-document answer — one symbol per candidate class,
            // the semantic legend's verdict mapped onto the protocol's
            // kinds (a lowered class is the protocol's own `Class`; a
            // refused one stays `String` — the raw spelling, not a
            // compiled class, and no custom SymbolKind exists to name
            // the boundary). The shape rides the protocol's own JSON
            // names directly: the typed `DocumentSymbol` carries a
            // deprecated field the literal construction would trip (the
            // workspace's zero-allow bar), and the wire quads pin the
            // exact keys beside. The whole spelling is the name, so the
            // selection range rides the candidate's own span.
            let answer = serde_json::from_value::<DocumentSymbolParams>(request.params)
                .ok()
                .map(|params| {
                    session
                        .engine
                        .document_symbols(params.text_document.uri.as_str())
                        .iter()
                        .map(|symbol| {
                            let kind = match symbol.kind {
                                crate::EngineSemanticKind::Lowered => SymbolKind::CLASS,
                                crate::EngineSemanticKind::Refused => SymbolKind::STRING,
                            };
                            let range = wire_range(symbol.range);
                            json!({
                                "name": symbol.name,
                                "detail": symbol.detail,
                                "kind": kind,
                                "range": range,
                                "selectionRange": range,
                            })
                        })
                        .collect::<Vec<_>>()
                });
            let result = answer.map_or(
                Some(Value::Null), // malformed params: the protocol's own empty answer
                |symbols| Some(Value::Array(symbols)),
            );
            respond(request.id, result)
        }
        "workspace/symbol" => {
            if !session.initialized {
                return Handled::default();
            }
            // The workspace's outline (LSP 3.16's workspace/symbol):
            // the document symbols' contract over EVERY open document —
            // the query the filter (case-insensitive substring on the
            // spelling, the empty query riding every open candidate),
            // each match a `SymbolInformation` quad over its
            // `Location`, URI order across the map. The quad rides the
            // protocol's own JSON names directly (the documentSymbol
            // arm's own shape — the typed struct's deprecated field
            // stays out of the literal), the verdict mapping the
            // outline's own (a lowered class `Class`, a refused one
            // `String`). A URI the `Uri` parser rejects skips its quad
            // (the publish half's own rule); the flat symbols answer —
            // no hierarchical `WorkspaceSymbol[]`, the outline's own
            // document-order shape.
            let answer = serde_json::from_value::<WorkspaceSymbolParams>(request.params)
                .ok()
                .map(|params| {
                    session
                        .engine
                        .workspace_symbols(&params.query)
                        .into_iter()
                        .filter_map(|workspace_symbol| {
                            let uri = Uri::from_str(&workspace_symbol.uri).ok()?;
                            let kind = match workspace_symbol.symbol.kind {
                                crate::EngineSemanticKind::Lowered => SymbolKind::CLASS,
                                crate::EngineSemanticKind::Refused => SymbolKind::STRING,
                            };
                            Some(json!({
                                "name": workspace_symbol.symbol.name,
                                "kind": kind,
                                "location": {
                                    "uri": uri,
                                    "range": wire_range(workspace_symbol.symbol.range),
                                },
                            }))
                        })
                        .collect::<Vec<_>>()
                });
            let result = answer.map_or(
                Some(Value::Null), // malformed params: the protocol's own empty answer
                |symbols| Some(Value::Array(symbols)),
            );
            respond(request.id, result)
        }
        "textDocument/selectionRange" => {
            if !session.initialized {
                return Handled::default();
            }
            // The expand-selection walk (LSP 3.16's selectionRange): the
            // engine's parent chains, one per requested position, request
            // order kept — the protocol's own 1:1 contract the wire keeps
            // (the answer's length IS the request's). The chain rides the
            // protocol's own `parent` key (the wire_selection mapping's
            // recursion — the content span the chain's head, `None`).
            let answer = serde_json::from_value::<SelectionRangeParams>(request.params)
                .ok()
                .map(|params| {
                    let positions = params
                        .positions
                        .iter()
                        .map(|position| crate::EnginePosition {
                            line: position.line,
                            character: position.character,
                        })
                        .collect::<Vec<_>>();
                    session
                        .engine
                        .selection_ranges(params.text_document.uri.as_str(), &positions)
                        .iter()
                        .map(wire_selection)
                        .collect::<Vec<_>>()
                });
            let result = answer.map_or(
                Some(Value::Null), // malformed params: the protocol's own empty answer
                |ranges| Some(serde_json::to_value(ranges).unwrap_or(Value::Null)),
            );
            respond(request.id, result)
        }
        "textDocument/foldingRange" => {
            if !session.initialized {
                return Handled::default();
            }
            // The outline folding face (LSP 3.16's foldingRange): one
            // fold per extracted class string, document order — the
            // engine's whole-document walk. The columns ride (the
            // client folds the exact span) and no `kind` rides (a
            // plain region fold, the client's own default).
            let answer = serde_json::from_value::<FoldingRangeParams>(request.params)
                .ok()
                .map(|params| {
                    session
                        .engine
                        .folding_ranges(params.text_document.uri.as_str())
                        .iter()
                        .map(|fold| FoldingRange {
                            start_line: fold.start.line,
                            start_character: Some(fold.start.character),
                            end_line: fold.end.line,
                            end_character: Some(fold.end.character),
                            kind: None,
                            collapsed_text: None,
                        })
                        .collect::<Vec<_>>()
                });
            let result = answer.map_or(
                Some(Value::Null), // malformed params: the protocol's own empty answer
                |folds| Some(serde_json::to_value(folds).unwrap_or(Value::Null)),
            );
            respond(request.id, result)
        }
        "workspace/diagnostic" => {
            if !session.initialized {
                return Handled::default();
            }
            // The workspace-wide pull (LSP 3.17's workspace/diagnostic):
            // every open document's report beside its result identity —
            // the engine's answer, wire-shaped. lsp-types 0.97 carries
            // no typed shape for the workspace report, so the items
            // ride the json! form (the workspace-symbol answer's own
            // wire precedent): `full` re-lists the findings, `unchanged`
            // echoes the identity the client sent and carries none.
            let answer = previous_result_ids(&request.params).map(|previous| {
                // The engine's answer rides borrowed string pairs (the
                // wire-free signature); the parsed ids lend theirs.
                let previous = previous
                    .iter()
                    .map(|(uri, id)| (uri.as_str(), id.as_str()))
                    .collect::<Vec<_>>();
                let items = session
                    .engine
                    .workspace_diagnostics(&previous)
                    .iter()
                    .map(|report| match &report.outcome {
                        crate::EngineWorkspaceOutcome::Full { diagnostics } => {
                            let findings =
                                diagnostics.iter().map(wire_diagnostic).collect::<Vec<_>>();
                            json!({
                                "uri": report.uri,
                                "kind": "full",
                                "resultId": report.result_id,
                                "items": findings,
                            })
                        }
                        crate::EngineWorkspaceOutcome::Unchanged => {
                            json!({
                                "uri": report.uri,
                                "kind": "unchanged",
                                "resultId": report.result_id,
                            })
                        }
                    })
                    .collect::<Vec<_>>();
                json!({ "items": items })
            });
            let result = answer.map_or(
                Some(Value::Null), // malformed params: the protocol's own empty answer
                Some,
            );
            respond(request.id, result)
        }
        _ => Handled::default(),
    }
}

/// One notification: the document lifecycle (didOpen/didChange/didClose
/// publish diagnostics), the theme lifecycle (the watched-files event
/// naming the wired loader's URI re-themes the session) and the exit.
/// Everything else rides silently.
fn handle_notification(session: &mut Session, notification: Notification) -> Handled {
    match notification.method.as_str() {
        "textDocument/didOpen" => {
            let Ok(params) =
                serde_json::from_value::<DidOpenTextDocumentParams>(notification.params)
            else {
                return Handled::default();
            };
            let uri = params.text_document.uri;
            let version = params.text_document.version;
            session
                .engine
                .open(uri.as_str(), &params.text_document.text, version);
            publish(&mut session.engine, uri.as_str())
        }
        "textDocument/didChange" => {
            let Ok(params) =
                serde_json::from_value::<DidChangeTextDocumentParams>(notification.params)
            else {
                return Handled::default();
            };
            // The incremental sync (the capability's own upgrade): EVERY
            // change applies sequentially — a ranged change splices the
            // replacement text over the range the CURRENT text names
            // (each change riding the text the previous change left, the
            // protocol's own application order), a range-less change the
            // full-text replace (the backward-compat shape a full-sync
            // client sends, still legal under the incremental
            // capability). The engine's own boundary keeps the honest
            // no-ops (a line past the document, a reversed range — no
            // invented rewrite); the version tag lands after the patch.
            let changes = params
                .content_changes
                .iter()
                .map(|change| crate::EngineTextChange {
                    range: change.range.map(|range| {
                        (
                            crate::EnginePosition {
                                line: range.start.line,
                                character: range.start.character,
                            },
                            crate::EnginePosition {
                                line: range.end.line,
                                character: range.end.character,
                            },
                        )
                    }),
                    text: change.text.clone(),
                })
                .collect::<Vec<_>>();
            let uri = params.text_document.uri;
            let version = params.text_document.version;
            session
                .engine
                .change_incremental(uri.as_str(), &changes, version);
            publish(&mut session.engine, uri.as_str())
        }
        "textDocument/didClose" => {
            let Ok(params) =
                serde_json::from_value::<DidCloseTextDocumentParams>(notification.params)
            else {
                return Handled::default();
            };
            let uri = params.text_document.uri;
            session.engine.close(uri.as_str());
            // The protocol's own contract: a closed document publishes
            // an EMPTY diagnostic set so the editor clears its squiggles.
            publish(&mut session.engine, uri.as_str())
        }
        "workspace/didChangeWatchedFiles" => {
            let Ok(params) =
                serde_json::from_value::<DidChangeWatchedFilesParams>(notification.params)
            else {
                return Handled::default();
            };
            // The theme lifecycle's face: the session's loader owns ONE
            // watched uri. No loader (the every-constructor default) or
            // an event for a file it does not own rides silently — the
            // quiet vocabulary every unhandled notification keeps.
            let Some(loader) = session.theme.clone() else {
                return Handled::default();
            };
            if !params
                .changes
                .iter()
                .any(|event| event.uri.as_str() == loader.uri())
            {
                return Handled::default();
            }
            // A failed load (the parse refusal, the vanished file) keeps
            // the riding table: nothing changed, nothing republishes —
            // the honest degradation the last good program rides.
            let Ok(tokens) = loader.load() else {
                return Handled::default();
            };
            // The origin rides the loader's URI (the definitions jump
            // into the file the tables come from — idempotent).
            session.engine.note_theme_origin(loader.uri());
            // The engine's own contract: equal content is the
            // byte-neutral no-op (the pool keeps serving, nothing
            // republishes), a real swap retires the pool it invalidates.
            if session.engine.retheme(Some(tokens)) == RethemeOutcome::Unchanged {
                return Handled::default();
            }
            // The swap: every open document's findings ride the push
            // channel again (uri order — the cross-document answers'
            // own determinism; an unparseable uri skips its publish,
            // the publish half's own rule).
            let mut outgoing = Vec::new();
            for uri in session.engine.documents() {
                outgoing.extend(publish(&mut session.engine, &uri).outgoing);
            }
            Handled {
                outgoing,
                exit: false,
            }
        }
        "exit" => Handled {
            outgoing: Vec::new(),
            exit: true,
        },
        _ => Handled::default(),
    }
}

// ---------------------------------------------------------------------------
// The publish + respond helpers
// ---------------------------------------------------------------------------

/// The engine's byte-true range onto the protocol's own — the ONE
/// mapping every range-shaped answer rides (the hover's, the colours',
/// the quick fixes', the highlights', the edits' and the findings'
/// ranges; one conversion, owned once here).
fn wire_range(range: crate::EngineRange) -> lsp_types::Range {
    lsp_types::Range {
        start: lsp_types::Position {
            line: range.start.line,
            character: range.start.character,
        },
        end: lsp_types::Position {
            line: range.end.line,
            character: range.end.character,
        },
    }
}
/// The engine's rename edits folded onto the wire's full `changes` map —
/// one key per carrying document, the keys riding the ENGINE'S OWN URI
/// order (the fold's determinism carried to the BYTES: the engine hands
/// back URI-ordered edits and the map inserts in that order, so the
/// answer serializes the same whatever the dependency graph's
/// serde_json map semantics — a `HashMap`-keyed `WorkspaceEdit` rides
/// the hasher's whim, and the byte-identity contract every
/// cross-document face keeps does not), a URI the `Uri` parser rejects
/// skipping its key (the references half's own rule). The rename
/// REQUEST and the rename COMMAND answer the same fold — no parallel
/// edit shape at the wire.
fn changes_map(edits: &[crate::EngineRenameEdit]) -> Value {
    let mut changes = serde_json::Map::new();
    for edit in edits {
        // The wire names the keys with parsed `Uri`s: a URI the parser
        // rejects was never a document the editor could have named
        // either.
        if Uri::from_str(&edit.uri).is_err() {
            continue;
        }
        let list = changes
            .entry(edit.uri.clone())
            .or_insert_with(|| Value::Array(Vec::new()));
        if let Some(list) = list.as_array_mut() {
            list.push(json!({
                "range": serde_json::to_value(wire_range(edit.range)).unwrap_or(Value::Null),
                "newText": edit.new_text,
            }));
        }
    }
    json!({ "changes": changes })
}
/// The `aeolus.renameToken` command's argument shape: `[uri, line,
/// character, newName]` — a URI string, the two position numbers, the
/// new token name. Any missing or mistyped slot answers None (the
/// protocol's own null at the dispatch site).
fn rename_token_arguments(arguments: &[Value]) -> Option<(String, u32, u32, String)> {
    let uri = arguments.first()?.as_str()?.to_string();
    let line = u32::try_from(arguments.get(1)?.as_u64()?).ok()?;
    let character = u32::try_from(arguments.get(2)?.as_u64()?).ok()?;
    let new_name = arguments.get(3)?.as_str()?.to_string();
    Some((uri, line, character, new_name))
}

/// One engine selection chain onto the protocol's `SelectionRange` —
/// the parent chain riding the protocol's own `parent` key (the
/// recursion bottoms at the chain's head, `None`).
fn wire_selection(selection: &crate::EngineSelectionRange) -> SelectionRange {
    SelectionRange {
        range: wire_range(selection.range),
        parent: selection
            .parent
            .as_ref()
            .map(|parent| Box::new(wire_selection(parent))),
    }
}

/// One engine finding onto the protocol's Diagnostic — the mapping BOTH
/// channels share (the push channel's publish and the pull channel's
/// report; one walk, two transports, one render). The findings are the
/// non-fatal WARNING tier: every miss degrades, nothing aborts (the
/// CLI's own honesty contract at the editor).
fn wire_diagnostic(finding: &crate::EngineDiagnostic) -> lsp_types::Diagnostic {
    lsp_types::Diagnostic {
        range: wire_range(finding.range),
        severity: Some(lsp_types::DiagnosticSeverity::WARNING),
        code: Some(NumberOrString::String(finding.code.clone())),
        message: finding.message.clone(),
        source: Some("aeolus".to_string()),
        ..lsp_types::Diagnostic::default()
    }
}

/// The engine's absolute tokens onto the protocol's delta encoding: the
/// first token's deltas are absolute, each later one rides relative to
/// the previous token's START (same line: the character delta; a new
/// line: the absolute column). The engine answers in document order —
/// the encoding's own precondition, the engine battery pins it — and a
/// candidate never spans a line (the lexer splits on the whitespace,
/// newlines included), so the length is the column arithmetic. The
/// legend indexes mirror [`server_capabilities`]' own legend, the wire
/// contract the tests pin beside the deltas.
fn wire_semantic_tokens(tokens: &[crate::EngineSemanticToken]) -> SemanticTokens {
    let mut data = Vec::with_capacity(tokens.len());
    let mut prev_line = 0u32;
    let mut prev_character = 0u32;
    for token in tokens {
        let delta_line = token.range.start.line - prev_line;
        let delta_start = if delta_line == 0 {
            token.range.start.character - prev_character
        } else {
            token.range.start.character
        };
        let length = token.range.end.character - token.range.start.character;
        let token_type = match token.kind {
            crate::EngineSemanticKind::Lowered => 0, // the legend's "class"
            crate::EngineSemanticKind::Refused => 1, // the legend's "refused"
        };
        data.push(SemanticToken {
            delta_line,
            delta_start,
            length,
            token_type,
            token_modifiers_bitset: 0,
        });
        prev_line = token.range.start.line;
        prev_character = token.range.start.character;
    }
    SemanticTokens {
        result_id: None, // the engine never caches: every pull recomputes
        data,
    }
}

/// Publish the engine's findings for one URI (the didOpen/didChange/
/// didClose half). A URI the `Url` parser rejects skips the publish —
/// the wire gave it, and no editor can have named it either.
fn publish(engine: &mut AeolusEngine, uri: &str) -> Handled {
    let Ok(parsed) = Uri::from_str(uri) else {
        return Handled::default();
    };
    let version = engine.version(uri);
    let diagnostics = engine.analyze(uri).iter().map(wire_diagnostic).collect();
    let params = serde_json::to_value(PublishDiagnosticsParams {
        uri: parsed,
        diagnostics,
        version,
    })
    .unwrap_or(Value::Null);
    Handled {
        outgoing: vec![Message::Notification(Notification {
            method: "textDocument/publishDiagnostics".to_string(),
            params,
        })],
        exit: false,
    }
}

/// One response message (the id verbatim, the result serialized).
fn respond(id: RequestId, result: Option<Value>) -> Handled {
    Handled {
        outgoing: vec![Message::Response(Response {
            id,
            result,
            error: None,
        })],
        exit: false,
    }
}

/// The workspace-pull params' `previousResultIds` — the client's cache
/// keys, walked off the raw JSON (lsp-types 0.97 carries no typed shape
/// for the workspace report). The spec's own shape: the array each of
/// whose entries carries the document `uri` and the `value` string the
/// client cached under. `None` — the params are malformed (the key
/// missing, an entry shapeless): the protocol's own empty answer rides.
fn previous_result_ids(params: &Value) -> Option<Vec<(String, String)>> {
    let ids = params.get("previousResultIds")?.as_array()?;
    let mut out = Vec::with_capacity(ids.len());
    for id in ids {
        let uri = id.get("uri")?.as_str()?.to_string();
        let value = id.get("value")?.as_str()?.to_string();
        out.push((uri, value));
    }
    Some(out)
}

/// The advertised capabilities: hover, completion, colour swatches, the
/// semantic tokens (full and range), the inlay hints, the quick fixes,
/// the document highlights, the rename face, the canonical-order
/// formatting, the document-symbol outline, the folding ranges and the
/// incremental sync. The
/// engine never forks the pipeline, so the capability surface is the
/// pipeline's own honesty contract at the editor.
fn server_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        // The token goto-definition face: the cursor's token class
        // answers the theme file's own declaration site (the theme
        // pass's provenance scan, or the wired host store's). The plain
        // `true` form: the answer is the engine's own walk, no options
        // object.
        definition_provider: Some(OneOf::Left(true)),
        color_provider: Some(ColorProviderCapability::Simple(true)),
        // The quick fixes: every answer is the pipeline's own
        // correction (the audited tables' spellings) with the edit over
        // the finding's range — no resolve round-trip, the action is
        // final.
        code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
        // The click-to-find highlights: the textual occurrences of the
        // cursor's candidate, document order, no compile verdict
        // riding.
        document_highlight_provider: Some(OneOf::Left(true)),
        // The simultaneous-edit face (LSP 3.16's linkedEditingRange):
        // the highlights' textual equality as the client's
        // edit-together ranges — same-file only (the cross-document
        // fold stays the rename face's). The plain `true` form: the
        // answer is the engine's own walk, no options object.
        linked_editing_range_provider: Some(LinkedEditingRangeServerCapabilities::Simple(true)),
        // The workspace's occurrences: the highlights' textual equality
        // over EVERY open document (the engine's first cross-document
        // answer), each riding its `Location`, URI order across the
        // map. The plain `true` form: the answer is the engine's own
        // walk, no options object.
        references_provider: Some(OneOf::Left(true)),
        // The workspace's outline: the document symbols' contract over
        // EVERY open document, the query the filter (case-insensitive
        // substring, the empty query riding every open candidate), URI
        // order across the map. The plain `true` form: the answer is
        // the engine's own filtered walk, no options object (no
        // resolve round-trip — the symbols are final, the outline's
        // own contract).
        workspace_symbol_provider: Some(OneOf::Left(true)),
        // The rename face: the candidate span under the cursor
        // (`prepareRename` — the hover locator's own walk) beside the
        // per-occurrence edits (`rename` — the references' fold over
        // the rename edits: the highlights' walk generalized over the
        // WHOLE document map, one edit per occurrence, each riding its
        // document's URI). The plain `true` form: the answers are the
        // engine's own textual walks, no prepare gate options object.
        rename_provider: Some(OneOf::Left(true)),
        // The expand-selection walk (LSP 3.16's selectionRange): one
        // parent chain per requested position — the candidate span the
        // shared locator keys, the string's content span its parent,
        // the caret's zero-width range the chain-less answer. The
        // plain `true` form: the answer is the engine's own walk, no
        // options object.
        selection_range_provider: Some(SelectionRangeProviderCapability::Simple(true)),
        // The class-sort seam's editor faces: the whole-document walk
        // (`formatting` — one edit per out-of-order class string, the
        // canonical order as the new text, an already-canonical string
        // answering nothing: the format-on-save no-op) beside the
        // SELECTION-SCOPED form (`rangeFormatting` — the same walk
        // restricted to the strings the requested range touches, a
        // partial selection still naming the string). Both answer the
        // engine's own walks, no options object.
        document_formatting_provider: Some(OneOf::Left(true)),
        // The sort seam's range form (LSP 3.12's rangeFormatting): the
        // selection-scoped walk the selection or the cursor's block
        // rides — the strings the range INTERSECTS reorder, the whole
        // string riding its edit, never a fragment. The plain `true`
        // form: the answer is the engine's own walk, no options object.
        document_range_formatting_provider: Some(OneOf::Left(true)),
        // The server's command surface: `aeolus.sortDocument` (the
        // class-sort seam's editor face invoked as a command) beside
        // `aeolus.renameToken` (the theme-token rename fold — the
        // engine's cross-document token edits as the client's drive).
        // The table is CLOSED and advertised: two commands, both
        // answering the engine's own walks, no resolve round-trip.
        execute_command_provider: Some(ExecuteCommandOptions {
            commands: vec![
                "aeolus.sortDocument".to_string(),
                "aeolus.renameToken".to_string(),
            ],
            work_done_progress_options: WorkDoneProgressOptions::default(),
        }),
        // The outline face: one symbol per candidate class, document
        // order (the engine's whole-document walk — no hierarchical
        // tree, no options object; the plain `true` form).
        document_symbol_provider: Some(OneOf::Left(true)),
        // The outline folding face (LSP 3.16's foldingRange): one fold
        // per extracted class string — the shape-level walk, no compile
        // verdict riding (the highlights' textual family). The plain
        // `true` form: the answer is the engine's own walk, no options
        // object.
        folding_range_provider: Some(FoldingRangeProviderCapability::Simple(true)),
        // The inline answers (LSP 3.17): the engine's hints are final —
        // no resolve round-trip (the completion's own contract), the
        // label and tooltip ride the hint itself.
        inlay_hint_provider: Some(lsp_types::OneOf::Right(
            InlayHintServerCapabilities::Options(InlayHintOptions {
                work_done_progress_options: WorkDoneProgressOptions::default(),
                resolve_provider: Some(false),
            }),
        )),
        // The highlighter feed: the legend the delta encoding indexes —
        // a lowered candidate is the protocol's own `class`, a refused
        // one the server's custom `refused` type (an editor that does
        // not know it renders neutral, the honest degradation). The
        // ORDER is the wire contract the encoder's indexes mirror.
        semantic_tokens_provider: Some(SemanticTokensServerCapabilities::SemanticTokensOptions(
            SemanticTokensOptions {
                work_done_progress_options: WorkDoneProgressOptions::default(),
                legend: SemanticTokensLegend {
                    token_types: vec![SemanticTokenType::CLASS, SemanticTokenType::new("refused")],
                    token_modifiers: Vec::new(),
                },
                range: Some(true),
                full: Some(SemanticTokensFullOptions::Bool(true)),
            },
        )),
        // The pull model rides beside the push channel: the client can
        // ask any time (the refresh path, a newly focused file) and the
        // engine recomputes — per document, no cross-file model. The
        // workspace-wide pull rides beside it (LSP 3.17's
        // workspace/diagnostic): every open document's report beside
        // its result identity, the unchanged verdict the client's
        // cache answers from.
        diagnostic_provider: Some(DiagnosticServerCapabilities::Options(DiagnosticOptions {
            identifier: None,
            inter_file_dependencies: false,
            workspace_diagnostics: true,
            ..DiagnosticOptions::default()
        })),
        completion_provider: Some(CompletionOptions {
            // The class grammar's own separators: `-` joins root and
            // suffix, `:` ends a variant segment — the two keystrokes
            // the completion answer re-fires on.
            trigger_characters: Some(vec!["-".to_string(), ":".to_string()]),
            resolve_provider: Some(false),
            ..CompletionOptions::default()
        }),
        text_document_sync: Some(TextDocumentSyncCapability::Options(
            TextDocumentSyncOptions {
                open_close: Some(true),
                // The incremental sync: the didChange payload carries
                // RANGED content changes — each spliced sequentially
                // against the text the previous change left (the
                // engine's `change_incremental` walk), a range-less
                // change the full-text replace the full-sync clients
                // keep sending (the backward-compat shape the
                // capability keeps legal).
                change: Some(TextDocumentSyncKind::INCREMENTAL),
                ..TextDocumentSyncOptions::default()
            },
        )),
        ..ServerCapabilities::default()
    }
}

// ---------------------------------------------------------------------------
// The stdio loop
// ---------------------------------------------------------------------------

/// Run the server over stdio (the editor spawns the binary): the
/// transport's thin shell — one thread per side, the state machine
/// behind it. Blocks until the client exits.
///
/// # Errors
/// The transport's own IO/thread failures; the state machine itself
/// cannot fail (every answer is deterministic over the document map).
pub fn run_stdio() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    serve_io(None)
}

/// The observed loop: the SAME stdio shell with the transport
/// observability face wired — every request/notification verdict the
/// machine answers streams to the observer for the whole session's
/// lifetime (the [`Session::with_observer`] contract, the loop the
/// binary drives). The answers are the plain loop's, byte for byte:
/// the channel reports the traffic, never shapes it.
///
/// # Errors
/// [`run_stdio`]'s own contract, unchanged.
pub fn run_stdio_observed(
    observer: impl TransportObserver + 'static,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    serve_io(Some(Arc::new(observer)))
}

/// The themed loop (the reload seam's binary face): the session's
/// table bakes from the loader's OWN first read — one construction
/// path, the loader seam's — and the SAME loader rides the session, so
/// every `workspace/didChangeWatchedFiles` event naming the loader's
/// URI re-themes the running session (the [`Session::with_theme_loader`]
/// contract). The startup bake failure is the loop's startup error: a
/// bad theme file names its [`ThemeError`](crate::ThemeError) before
/// the transport starts, the CLI's own honest-failure shape — the
/// session never starts half-themed.
///
/// # Errors
/// The loader's startup bake (the typed [`ThemeError`](crate::ThemeError)
/// through the boxed boundary) beside [`run_stdio`]'s own IO contract.
pub fn run_stdio_with_theme(
    loader: impl crate::ThemeLoader + 'static,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let tokens = loader.load()?;
    let mut session = Session::new(EngineConfig::new(Some(tokens))).with_theme_loader(loader);
    let (connection, io_threads) = Connection::stdio();
    for message in &connection.receiver {
        let handled = handle_message(&mut session, message);
        for out in handled.outgoing {
            connection.sender.send(out)?;
        }
        if handled.exit {
            break;
        }
    }
    io_threads.join()?;
    Ok(())
}

/// The stdio shell both entries drive (the loop is the transport's
/// thin shell — the observer rides the session, `None` the plain
/// loop's every-constructor default).
fn serve_io(
    observer: Option<Arc<dyn TransportObserver>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (connection, io_threads) = Connection::stdio();
    let mut session = Session::new(EngineConfig::default());
    if let Some(observer) = observer {
        session = session.with_observer(observer);
    }
    for message in &connection.receiver {
        let handled = handle_message(&mut session, message);
        for out in handled.outgoing {
            connection.sender.send(out)?;
        }
        if handled.exit {
            break;
        }
    }
    io_threads.join()?;
    Ok(())
}
