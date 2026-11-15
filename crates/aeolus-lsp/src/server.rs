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
    Color, ColorInformation, ColorProviderCapability, CompletionItem, CompletionItemKind,
    CompletionList, CompletionOptions, CompletionParams, DidChangeTextDocumentParams,
    DidCloseTextDocumentParams, DidOpenTextDocumentParams, DocumentColorParams, Hover,
    HoverContents, HoverProviderCapability, InitializeResult, MarkupContent, MarkupKind,
    NumberOrString, PublishDiagnosticsParams, ServerCapabilities, ServerInfo,
    TextDocumentPositionParams, TextDocumentSyncCapability, TextDocumentSyncKind,
    TextDocumentSyncOptions, Uri,
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
/// nothing (the capabilities never advertised it).
fn handle_request(session: &mut Session, request: Request) -> Handled {
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

/// Publish the engine's findings for one URI (the didOpen/didChange/
/// didClose half). A URI the `Url` parser rejects skips the publish —
/// the wire gave it, and no editor can have named it either.
fn publish(engine: &mut AeolusEngine, uri: &str) -> Handled {
    let Ok(parsed) = Uri::from_str(uri) else {
        return Handled::default();
    };
    let version = engine.version(uri);
    let diagnostics = engine
        .analyze(uri)
        .iter()
        .map(|finding| lsp_types::Diagnostic {
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
        })
        .collect();
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

/// The advertised capabilities: hover, completion, colour swatches and
/// full-text sync. The engine never forks the pipeline, so the
/// capability surface is the pipeline's own honesty contract at the
/// editor.
fn server_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        color_provider: Some(ColorProviderCapability::Simple(true)),
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
