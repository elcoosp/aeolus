//! Consumer demonstration: the wire half's transport face — the
//! state machine's per-message verdicts streamed to a host handle.
//! The pool face (`lsp_cache`) answers "what did the pool decide", the
//! walk face (`lsp_walk_observer`) answers "what did the walk lower";
//! this one answers "what did the LOOP serve": every request's answer
//! or quiet drop, every notification's publish, exit or silence — the
//! wire's own truth, read off the same [`Handled`] verdicts the
//! callers receive. A session wired before its first message sees the
//! whole traffic; the answers stay byte-identical to the unwired
//! loop's (the differential prints here).
//!
//! ```sh
//! cargo run -p aeolus-lsp --example lsp_transport_observer
//! ```

use std::sync::{Arc, Mutex};

use aeolus_lsp::{
    EngineConfig, NotificationOutcome, RequestOutcome, Session, TransportObserver, TransportStats,
    handle_message,
};
use lsp_server::{Message, Notification, Request, RequestId};
use serde_json::json;

/// The demo document (the server suite's own fixture): one lowered
/// class, one unknown root.
const DOC: &str = "fn card() {\n    let s = \"p-4 not-a-root\";\n}\n";

/// A host-implemented channel: the per-message verdicts, wire order —
/// the shape an editor's traffic log writes.
struct Recorder {
    events: Mutex<Vec<String>>,
}

impl TransportObserver for Recorder {
    fn on_request(&self, method: &str, outcome: RequestOutcome) {
        let line = match outcome {
            RequestOutcome::Answered => format!("{method}: answered"),
            RequestOutcome::QuietDropped => format!("{method}: quiet-dropped"),
        };
        self.events
            .lock()
            .expect("the demo never poisons")
            .push(line);
    }

    fn on_notification(&self, method: &str, outcome: NotificationOutcome) {
        let line = match outcome {
            NotificationOutcome::Published => format!("{method}: published"),
            NotificationOutcome::Exit => format!("{method}: the session ends"),
            NotificationOutcome::QuietDropped => format!("{method}: quiet-dropped"),
        };
        self.events
            .lock()
            .expect("the demo never poisons")
            .push(line);
    }
}

/// Host-side composition: the session holds ONE observer slot (the
/// builders' replace semantics, the walk face's own), so two channels
/// ride one forwarding observer (the trait's no-op defaults make the
/// composite two forwards — the `Arc` handles stay alive beside the
/// session).
struct Telemetry {
    stats: Arc<TransportStats>,
    log: Arc<Recorder>,
}

impl TransportObserver for Telemetry {
    fn on_request(&self, method: &str, outcome: RequestOutcome) {
        self.stats.on_request(method, outcome);
        self.log.on_request(method, outcome);
    }

    fn on_notification(&self, method: &str, outcome: NotificationOutcome) {
        self.stats.on_notification(method, outcome);
        self.log.on_notification(method, outcome);
    }
}

/// One wire message (the state-machine suite's own builders).
fn request(id: i32, method: &str, params: serde_json::Value) -> Message {
    Message::Request(Request {
        id: RequestId::from(id),
        method: method.to_string(),
        params,
    })
}

fn notification(method: &str, params: serde_json::Value) -> Message {
    Message::Notification(Notification {
        method: method.to_string(),
        params,
    })
}

fn did_open(version: i32) -> Message {
    notification(
        "textDocument/didOpen",
        json!({
            "textDocument": {
                "uri": "file:///card.rs",
                "languageId": "rust",
                "version": version,
                "text": DOC,
            }
        }),
    )
}

/// The scripted traffic: a pre-handshake hover (the quiet drop), the
/// initialize, a didOpen, an answered hover, an unknown request, an
/// unknown notification, the shutdown and the exit.
fn script() -> Vec<Message> {
    vec![
        request(
            1,
            "textDocument/hover",
            json!({"position": {"line": 0, "character": 0}}),
        ),
        request(0, "initialize", json!({"capabilities": {}})),
        did_open(1),
        request(
            2,
            "textDocument/hover",
            json!({
                "textDocument": {"uri": "file:///card.rs"},
                "position": {"line": 1, "character": 14}
            }),
        ),
        request(3, "textDocument/definitely-not-mine", json!({})),
        notification("$/setTrace", json!({"value": "off"})),
        request(4, "shutdown", json!(null)),
        notification("exit", json!(null)),
    ]
}

/// The outgoing messages as the wire's own JSON, one string per
/// message — the byte-neutrality differential's read side.
fn drive(session: &mut Session) -> Vec<String> {
    script()
        .into_iter()
        .flat_map(|message| handle_message(session, message).outgoing)
        .map(|message| serde_json::to_string(&message).expect("the wire serializes"))
        .collect()
}

fn main() {
    // The composite rides beside both handles: the session's ONE slot
    // forwards to the two channels.
    let recorder = Arc::new(Recorder {
        events: Mutex::new(Vec::new()),
    });
    let stats = Arc::new(TransportStats::new());
    let mut observed = Session::new(EngineConfig::default()).with_observer(Telemetry {
        stats: Arc::clone(&stats),
        log: Arc::clone(&recorder),
    });
    let observed_answers = drive(&mut observed);

    // The recorder's verdicts, wire order.
    println!("the loop's verdicts, in wire order:");
    for event in recorder
        .events
        .lock()
        .expect("the demo never poisons")
        .iter()
    {
        println!("  {event}");
    }

    // The built-in totals: the answered/quiet split, the
    // publish/exit/silence split.
    let snap = stats.snapshot();
    println!(
        "requests: {} total — {} answered, {} quiet-dropped",
        snap.requests, snap.answered, snap.quiet_requests
    );
    println!(
        "notifications: {} total — {} published, {} exits, {} quiet-dropped",
        snap.notifications, snap.published, snap.exits, snap.quiet_notifications
    );

    // Answer-neutrality: the SAME script over a FRESH unwired session
    // produces byte-identical outgoing messages (the seam adds a
    // channel, never changes a decision).
    let plain_answers = drive(&mut Session::new(EngineConfig::default()));
    println!(
        "answer-neutrality: plain {} messages, observed {} — identical: {}",
        plain_answers.len(),
        observed_answers.len(),
        plain_answers == observed_answers
    );
    // The unwired session's own Debug prints the wired-ness only (the
    // observer is a host handle, not printable state — the cache
    // Debug's own rule).
    println!(
        "unwired session debug: {:?}",
        Session::new(EngineConfig::default())
    );
}
