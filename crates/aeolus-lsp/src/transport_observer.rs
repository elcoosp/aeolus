//! The wire half's observability seam: the state machine's own
//! answers as a default-no-op observer trait, over the SAME
//! `handle_message` pipeline every transport drives — the engine's
//! pool face ([`crate::DynObserver`]) and walk face
//! ([`crate::WalkObserver`]) pointed at the loop instead of the
//! pipeline.
//!
//! ```text
//! Session::with_observer(stats)
//!   └─ handle_message(session, message)   — the machine's verdict
//!        ├─ on_request       — RequestOutcome (answered, or the quiet drop)
//!        └─ on_notification  — NotificationOutcome (published, exit, quiet drop)
//! ```
//!
//! The house rules the pool/walk/theme faces set, kept here: the
//! trait's methods are all no-ops by default (the seam adds a channel,
//! never replaces one — the [`Handled`](crate::Handled) verdicts the
//! callers already read keep answering); the built-in
//! [`NoTransportObserver`] is the every-constructor default
//! ([`Session::new`](crate::Session::new) starts observation-free, and
//! [`Session::with_observer`](crate::Session::with_observer) wires one
//! after construction); an `Arc<T>` of any observer IS an observer
//! (one host handle totals several sessions); and the seam is
//! answer-NEUTRAL — the observed session's outgoing messages are the
//! plain session's, byte for byte, which the differential pins. The
//! events report decisions the machine already made (the wire's own
//! verdict per handled message), never opinions: a host's telemetry
//! reads the loop's own truth.
//!
//! The channel rides the session itself, so the events reach every
//! caller of [`handle_message`](crate::handle_message) — the stdio
//! loop included, via [`run_stdio_observed`](crate::run_stdio_observed)
//! — without a single handler change: the outcome is read off the
//! machine's own [`Handled`](crate::Handled) verdict (outgoing
//! messages rode, or the protocol's own quiet-drop contract stayed
//! silent). Events stream in caller order, after the verdict is
//! computed and before the call returns; a silence IS an event (the
//! quiet drop is a wire decision the host reads — the pre-handshake
//! request, the unknown method, the unadvertised notification).

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// One request's wire verdict — the state machine's own answer, typed
/// so a host never parses prose to count.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequestOutcome {
    /// A response rode out (the result payload, the `null` answer and
    /// the shutdown ack are all answers — the wire spoke).
    Answered,
    /// No response rode: the protocol's own quiet-drop contract (a
    /// request before the initialize handshake, after the shutdown, or
    /// naming a method the capabilities never advertised).
    QuietDropped,
}

/// One notification's wire verdict — the same machine, the push
/// channel's own answers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NotificationOutcome {
    /// The diagnostics push rode out (the didOpen/didChange/didClose
    /// lifecycle: the engine's findings re-published).
    Published,
    /// The exit notification: the session ends (the loop's own exit
    /// contract).
    Exit,
    /// No message rode: the quiet-drop contract (a malformed lifecycle
    /// payload, an unadvertised method).
    QuietDropped,
}

/// The transport observer: the wire questions the state machine
/// answers as hosts and the loop drive it. Every method is a no-op
/// default — implement the channels a host reads.
pub trait TransportObserver: Send + Sync + 'static {
    /// One request's verdict, in caller order: `_method` the wire's
    /// method name, `_outcome` the machine's answer (`Answered` — a
    /// response rode; `QuietDropped` — the protocol's own silence). A
    /// silence IS an event — the quiet drop is a wire decision the
    /// host reads.
    fn on_request(&self, _method: &str, _outcome: RequestOutcome) {}

    /// One notification's verdict, in caller order: `_method` the
    /// wire's method name, `_outcome` the machine's answer
    /// (`Published` — the diagnostics push rode; `Exit` — the session
    /// ends; `QuietDropped` — the silence).
    fn on_notification(&self, _method: &str, _outcome: NotificationOutcome) {}
}

/// The observer's every-constructor default: all channels no-op — the
/// explicit "observe nothing" handle (the answer-neutrality suite's
/// silent twin).
#[derive(Debug, Default)]
pub struct NoTransportObserver;

impl TransportObserver for NoTransportObserver {}

/// Shared-observer forwarding: an `Arc<T>` of any observer IS an
/// observer (the pool face's own blanket — one host handle totals
/// several sessions and stays alive beside them).
impl<O: TransportObserver + ?Sized> TransportObserver for Arc<O> {
    fn on_request(&self, method: &str, outcome: RequestOutcome) {
        (**self).on_request(method, outcome);
    }

    fn on_notification(&self, method: &str, outcome: NotificationOutcome) {
        (**self).on_notification(method, outcome);
    }
}

/// The built-in [`TransportObserver`]: the loop's wire totals in one
/// shareable handle — the counts a host's telemetry reads without
/// holding the session. Sharing rides the [`Arc`] blanket (the pool
/// stats' own shape: one handle totals several sessions, the handle
/// alive beside them), the snapshot the `Copy` value the host asserts
/// on.
#[derive(Debug, Default)]
pub struct TransportStats {
    requests: AtomicU64,
    answered: AtomicU64,
    quiet_requests: AtomicU64,
    notifications: AtomicU64,
    published: AtomicU64,
    quiet_notifications: AtomicU64,
    exits: AtomicU64,
}

impl TransportStats {
    /// The empty session.
    pub fn new() -> Self {
        Self::default()
    }

    /// The session's totals so far — the `Copy` value the host's own
    /// telemetry or test asserts on. The splits partition: `requests`
    /// is `answered + quiet_requests`, `notifications` is `published +
    /// exits + quiet_notifications`.
    pub fn snapshot(&self) -> TransportSnapshot {
        let load = |cell: &AtomicU64| cell.load(Ordering::Relaxed);
        TransportSnapshot {
            requests: load(&self.requests),
            answered: load(&self.answered),
            quiet_requests: load(&self.quiet_requests),
            notifications: load(&self.notifications),
            published: load(&self.published),
            quiet_notifications: load(&self.quiet_notifications),
            exits: load(&self.exits),
        }
    }
}

impl TransportObserver for TransportStats {
    fn on_request(&self, _method: &str, outcome: RequestOutcome) {
        self.requests.fetch_add(1, Ordering::Relaxed);
        match outcome {
            RequestOutcome::Answered => {
                self.answered.fetch_add(1, Ordering::Relaxed);
            }
            RequestOutcome::QuietDropped => {
                self.quiet_requests.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    fn on_notification(&self, _method: &str, outcome: NotificationOutcome) {
        self.notifications.fetch_add(1, Ordering::Relaxed);
        match outcome {
            NotificationOutcome::Published => {
                self.published.fetch_add(1, Ordering::Relaxed);
            }
            NotificationOutcome::Exit => {
                self.exits.fetch_add(1, Ordering::Relaxed);
            }
            NotificationOutcome::QuietDropped => {
                self.quiet_notifications.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}

/// One observer session's totals: a `Copy` value the host holds
/// without holding the observer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct TransportSnapshot {
    /// The requests the session observed (every wire request, verdicts
    /// split below).
    pub requests: u64,
    /// The requests a response rode out for.
    pub answered: u64,
    /// The requests the quiet-drop contract silenced.
    pub quiet_requests: u64,
    /// The notifications the session observed (verdicts split below).
    pub notifications: u64,
    /// The notifications whose diagnostics push rode out.
    pub published: u64,
    /// The notifications the quiet-drop contract silenced.
    pub quiet_notifications: u64,
    /// The exit notifications (the session endings the loop observed).
    pub exits: u64,
}
