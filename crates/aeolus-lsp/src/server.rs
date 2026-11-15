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
    Color, ColorInformation, ColorPresentation, ColorPresentationParams, ColorProviderCapability,
    CompletionItem, CompletionItemKind, CompletionList, CompletionOptions, CompletionParams,
    DiagnosticOptions, DiagnosticServerCapabilities, DidChangeTextDocumentParams,
    DidCloseTextDocumentParams, DidOpenTextDocumentParams, DocumentColorParams,
    DocumentDiagnosticParams, DocumentDiagnosticReport, DocumentDiagnosticReportResult,
    FullDocumentDiagnosticReport, Hover, HoverContents, HoverProviderCapability, InitializeResult,
    MarkupContent, MarkupKind, NumberOrString, PublishDiagnosticsParams,
    RelatedFullDocumentDiagnosticReport, SemanticToken, SemanticTokenType, SemanticTokens,
    SemanticTokensFullOptions, SemanticTokensLegend, SemanticTokensOptions, SemanticTokensParams,
    SemanticTokensServerCapabilities, ServerCapabilities, ServerInfo, TextDocumentPositionParams,
    TextDocumentSyncCapability, TextDocumentSyncKind, TextDocumentSyncOptions, TextEdit, Uri,
    WorkDoneProgressOptions,
};
use serde_json::Value;
use std::str::FromStr;

use crate::engine::{AeolusEngine, EngineConfig};

// ---------------------------------------------------------------------------
// The session state
// ---------------------------------------------------------------------------

/// One server session: the engine behind it and the lifecycle state the
/// loop needs. The editor's view of the session is the wire contract;
/// the engine's view is the document map.
#[derive(Debug)]
pub struct Session {
    /// The engine (the pipeline answers).
    engine: AeolusEngine,
    /// Whether the client completed the initialize handshake (requests
    /// before it ride the protocol's own quiet-drop contract).
    initialized: bool,
    /// Whether the client sent the shutdown request (the loop's exit
    /// is the exit NOTIFICATION, per the protocol).
    shutdown: bool,
}

impl Session {
    /// A session over one engine configuration.
    pub fn new(config: EngineConfig) -> Self {
        Self {
            engine: AeolusEngine::new(config),
            initialized: false,
            shutdown: false,
        }
    }

    /// The session's engine (a host embedding the state machine reads
    /// the pipeline answers directly).
    pub fn engine(&self) -> &AeolusEngine {
        &self.engine
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
pub fn handle_message(session: &mut Session, message: Message) -> Handled {
    match message {
        Message::Request(request) => handle_request(session, request),
        Message::Notification(notification) => handle_notification(session, notification),
        Message::Response(_) => Handled::default(), // the server never drives a client request
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
                    range: hover.range.map(|range| lsp_types::Range {
                        start: lsp_types::Position {
                            line: range.start.line,
                            character: range.start.character,
                        },
                        end: lsp_types::Position {
                            line: range.end.line,
                            character: range.end.character,
                        },
                    }),
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
                            range: lsp_types::Range {
                                start: lsp_types::Position {
                                    line: swatch.range.start.line,
                                    character: swatch.range.start.character,
                                },
                                end: lsp_types::Position {
                                    line: swatch.range.end.line,
                                    character: swatch.range.end.character,
                                },
                            },
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
        _ => Handled::default(),
    }
}

/// One notification: the document lifecycle (didOpen/didChange/didClose
/// publish diagnostics) and the exit. Everything else rides silently.
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
            // Full-text sync: the LAST change carries the whole text.
            let Some(change) = params.content_changes.last() else {
                return Handled::default();
            };
            let uri = params.text_document.uri;
            let version = params.text_document.version;
            session.engine.change(uri.as_str(), &change.text, version);
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

/// One engine finding onto the protocol's Diagnostic — the mapping BOTH
/// channels share (the push channel's publish and the pull channel's
/// report; one walk, two transports, one render). The findings are the
/// non-fatal WARNING tier: every miss degrades, nothing aborts (the
/// CLI's own honesty contract at the editor).
fn wire_diagnostic(finding: &crate::EngineDiagnostic) -> lsp_types::Diagnostic {
    lsp_types::Diagnostic {
        range: lsp_types::Range {
            start: lsp_types::Position {
                line: finding.range.start.line,
                character: finding.range.start.character,
            },
            end: lsp_types::Position {
                line: finding.range.end.line,
                character: finding.range.end.character,
            },
        },
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

/// The advertised capabilities: hover, completion, colour swatches, the
/// semantic tokens and full-text sync. The engine never forks the
/// pipeline, so the capability surface is the pipeline's own honesty
/// contract at the editor.
fn server_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        color_provider: Some(ColorProviderCapability::Simple(true)),
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
                range: None,
                full: Some(SemanticTokensFullOptions::Bool(true)),
            },
        )),
        // The pull model rides beside the push channel: the client can
        // ask any time (the refresh path, a newly focused file) and the
        // engine recomputes — per document, no cross-file model, no
        // workspace-wide pull (the document map is the answer's scope).
        diagnostic_provider: Some(DiagnosticServerCapabilities::Options(DiagnosticOptions {
            identifier: None,
            inter_file_dependencies: false,
            workspace_diagnostics: false,
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
                change: Some(TextDocumentSyncKind::FULL),
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
    let (connection, io_threads) = Connection::stdio();
    let mut session = Session::new(EngineConfig::default());
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
