//! The wire half's observability face (the transport seam): the
//! state machine's own answers — one request/notification verdict per
//! handled message — streamed to a default-no-op observer the session
//! wires, the built-in totals the host asserts on. Written FIRST (the
//! RED spec): the missing-trait compile failure WAS the capture.
//!
//! The suite pins the face's house rules beside the answers:
//!
//! * answer-NEUTRALITY — the observed session's outgoing messages are
//!   the plain session's, byte for byte (the seam adds a channel,
//!   never changes a decision);
//! * the built-in [`TransportStats`] totals a scripted session exactly
//!   (the answered/quiet split, the publish/exit split);
//! * the quiet-drops ARE events (the protocol's own silence is the
//!   machine's answer, typed) and the publish/exit verdicts name
//!   themselves;
//! * ONE handle totals several sessions (the `Arc` blanket);
//! * [`NoTransportObserver`] is the constructible every-default, and
//!   the session's `Debug` prints the wired-ness only (the observer is
//!   a host handle, not printable state — the cache Debug's own rule).

use std::sync::{Arc, Mutex};

use aeolus_lsp::{
    EngineConfig, NoTransportObserver, NotificationOutcome, RequestOutcome, Session,
    TransportObserver, TransportStats, handle_message,
};
use lsp_server::{Message, RequestId};
use serde_json::json;

const DOC: &str = "fn card() {\n    let s = \"p-4 not-a-root\";\n}\n";

fn uri() -> String {
    "file:///card.rs".to_string()
}

fn request(id: i32, method: &str, params: serde_json::Value) -> Message {
    Message::Request(lsp_server::Request {
        id: RequestId::from(id),
        method: method.to_string(),
        params,
    })
}

fn notification(method: &str, params: serde_json::Value) -> Message {
    Message::Notification(lsp_server::Notification {
        method: method.to_string(),
        params,
    })
}

fn did_open(version: i32) -> Message {
    notification(
        "textDocument/didOpen",
        json!({
            "textDocument": {
                "uri": uri(),
                "languageId": "rust",
                "version": version,
                "text": DOC,
            }
        }),
    )
}

fn did_change(version: i32) -> Message {
    notification(
        "textDocument/didChange",
        json!({
            "textDocument": {"uri": uri(), "version": version},
            "contentChanges": [{"text": DOC}]
        }),
    )
}

fn initialize(session: &mut Session) {
    let handled = handle_message(
        session,
        request(0, "initialize", json!({"capabilities": {}})),
    );
    assert!(!handled.outgoing.is_empty(), "initialize answers");
}

fn hover_request(id: i32) -> Message {
    // The cursor inside `p-4` (the same position arithmetic the
    // server suite's hover pin rides).
    let start = DOC.find("\"p-4 not-a-root\"").expect("the fixture holds") + 3;
    let line = DOC[..start].matches('\n').count();
    let line_start = DOC[..start].rfind('\n').map_or(0, |i| i + 1);
    let character = DOC[line_start..start]
        .chars()
        .map(char::len_utf16)
        .sum::<usize>();
    request(
        id,
        "textDocument/hover",
        json!({
            "textDocument": {"uri": uri()},
            "position": {"line": line, "character": character}
        }),
    )
}

/// The scripted session's traffic: a pre-initialize hover (the quiet
/// drop), the handshake, a didOpen, a hover (answered), an unknown
/// request and an unknown notification (the quiet drops), a didChange
/// (republished), the shutdown and the exit.
fn script() -> Vec<Message> {
    vec![
        hover_request(1),
        request(0, "initialize", json!({"capabilities": {}})),
        did_open(1),
        hover_request(2),
        request(3, "textDocument/definitely-not-mine", json!({})),
        notification("$/setTrace", json!({"value": "off"})),
        did_change(2),
        request(4, "shutdown", json!(null)),
        notification("exit", json!(null)),
    ]
}

/// The recorder: the per-message verdicts, in order, as the
/// outcome-typed strings the assertions read.
#[derive(Default)]
struct Recorder {
    events: Mutex<Vec<String>>,
}

impl TransportObserver for Recorder {
    fn on_request(&self, method: &str, outcome: RequestOutcome) {
        self.events
            .lock()
            .expect("the recorder owns its lock")
            .push(format!("request {method} {outcome:?}"));
    }

    fn on_notification(&self, method: &str, outcome: NotificationOutcome) {
        self.events
            .lock()
            .expect("the recorder owns its lock")
            .push(format!("notification {method} {outcome:?}"));
    }
}

/// The outgoing messages as the wire's own JSON, one string per
/// message — the byte-neutrality differential's read side.
fn outgoing_json(session: &mut Session, script: &[Message]) -> Vec<String> {
    script
        .iter()
        .flat_map(|message| handle_message(session, message.clone()).outgoing)
        .map(|message| serde_json::to_string(&message).expect("the wire serializes"))
        .collect()
}

#[test]
fn observed_session_answers_byte_identical() {
    // The differential: the SAME script over the plain session and the
    // observed session — the answers are the plain answers, byte for
    // byte (the seam adds a channel, never changes a decision).
    let plain = outgoing_json(&mut Session::new(EngineConfig::default()), &script());
    let mut observed_session = Session::new(EngineConfig::default())
        .with_observer(TransportStats::new())
        .with_observer(NoTransportObserver);
    let observed = outgoing_json(&mut observed_session, &script());
    assert_eq!(plain, observed, "the observed answers are the plain ones");
}

#[test]
fn stats_total_the_scripted_session() {
    let stats = Arc::new(TransportStats::new());
    let mut session = Session::new(EngineConfig::default()).with_observer(Arc::clone(&stats));
    for message in script() {
        let _ = handle_message(&mut session, message);
    }
    let snap = stats.snapshot();
    // Requests: hover (quiet), initialize, hover (answered), the
    // unknown method (quiet), shutdown.
    assert_eq!(snap.requests, 5);
    assert_eq!(snap.answered, 3);
    assert_eq!(snap.quiet_requests, 2);
    // Notifications: didOpen, the unknown $/ method, didChange, exit.
    assert_eq!(snap.notifications, 4);
    assert_eq!(snap.published, 2);
    assert_eq!(snap.quiet_notifications, 1);
    assert_eq!(snap.exits, 1);
}

#[test]
fn quiet_drops_report_their_outcome() {
    let recorder = Arc::new(Recorder::default());
    let mut session = Session::new(EngineConfig::default()).with_observer(Arc::clone(&recorder));
    // Before the handshake: the hover request rides the protocol's own
    // quiet-drop — and the silence IS the event.
    let _ = handle_message(&mut session, hover_request(1));
    // The unknown method: the capabilities never advertised it.
    let _ = handle_message(
        &mut session,
        request(3, "textDocument/definitely-not-mine", json!({})),
    );
    let events = recorder.events.lock().expect("the recorder owns its lock");
    assert_eq!(
        *events,
        vec![
            "request textDocument/hover QuietDropped".to_string(),
            "request textDocument/definitely-not-mine QuietDropped".to_string(),
        ]
    );
}

#[test]
fn publish_and_exit_report_their_outcomes() {
    let recorder = Arc::new(Recorder::default());
    let mut session = Session::new(EngineConfig::default()).with_observer(Arc::clone(&recorder));
    initialize(&mut session);
    let _ = handle_message(&mut session, did_open(1));
    let _ = handle_message(
        &mut session,
        notification("$/setTrace", json!({"value": "off"})),
    );
    let handled = handle_message(&mut session, notification("exit", json!(null)));
    assert!(handled.exit, "the exit notification ends the session");
    let events = recorder.events.lock().expect("the recorder owns its lock");
    assert_eq!(
        *events,
        vec![
            "request initialize Answered".to_string(),
            "notification textDocument/didOpen Published".to_string(),
            "notification $/setTrace QuietDropped".to_string(),
            "notification exit Exit".to_string(),
        ]
    );
}

#[test]
fn one_handle_totals_two_sessions() {
    // The Arc blanket's rule: ONE handle totals several sessions and
    // stays alive beside them.
    let stats = Arc::new(TransportStats::new());
    for _ in 0..2 {
        let mut session = Session::new(EngineConfig::default()).with_observer(Arc::clone(&stats));
        for message in script() {
            let _ = handle_message(&mut session, message);
        }
    }
    let snap = stats.snapshot();
    assert_eq!((snap.requests, snap.answered, snap.exits), (10, 6, 2));
}

#[test]
fn no_transport_observer_is_the_default_wiring() {
    // The every-default handle: constructible, all channels no-op.
    let observer = NoTransportObserver;
    observer.on_request("textDocument/hover", RequestOutcome::Answered);
    observer.on_notification("exit", NotificationOutcome::Exit);
}

#[test]
fn session_debug_prints_the_wired_ness() {
    let session = Session::new(EngineConfig::default());
    let rendered = format!("{session:?}");
    assert!(rendered.contains("observer: false"), "unwired: {rendered}");
    let session = session.with_observer(TransportStats::new());
    let rendered = format!("{session:?}");
    assert!(rendered.contains("observer: true"), "wired: {rendered}");
}
