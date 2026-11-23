//! The frame `Env`'s debug-assert (the §3.5 Env note's own ask, and
//! §9's mitigation row's first half):
//!
//! > Multi-window is safe because windows render sequentially per
//! > thread; add a debug-assert that `begin_frame` ran.
//!
//! > Thread-local `Env` correctness (multi-window, tests, non-UI
//! > threads) — `begin_frame` debug-assert; explicit `.tw_in(env, …)`
//! > escape hatch
//!
//! The escape-hatch half (`tw_env*` / `set_env`) landed with the P4
//! frame; the ASSERT half is this battery's capture: a read of the
//! thread-local Env on a thread where neither [`begin_frame`] nor
//! [`set_env`] ever ran must refuse IN DEBUG BUILDS (the resting
//! default's dark/viewport answers would ride silently). Release
//! builds keep the zero-cost read — the assert compiles out, the
//! release bench pins the shape.
//!
//! [`begin_frame`]: aeolus_gpui::begin_frame
//! [`set_env`]: aeolus_gpui::set_env
use aeolus_gpui::env;
use aeolus_gpui::prelude::{Env, set_env};

/// Each `#[test]` runs on its own fresh thread — the TLS frame marker
/// starts unset — so the unframed read refuses: the plan's
/// debug-assert, pinned from the consumer's side.
#[test]
#[should_panic(expected = "begin_frame")]
fn an_unframed_read_refuses_in_debug_builds() {
    let _ = env();
}

/// The escape hatch's contract: an EXPLICIT env is a deliberate read —
/// `set_env` marks the thread and the read answers (the §9 row's
/// second half, the tests/non-UI-threads lane).
#[test]
fn the_explicit_env_is_the_escape_hatch() {
    set_env(Env::default());
    let resting = env();
    assert!(
        !resting.dark,
        "the resting env is the default the walk reads"
    );
}

/// The marker is per-THREAD: the marked thread answers, a fresh
/// thread (spawned inside the test) still refuses — the multi-window
/// correctness the note guarantees rides the thread-local discipline.
#[test]
fn the_frame_marker_is_per_thread() {
    set_env(Env::default());
    let _ = env(); // the marked thread answers

    let unmarked = std::thread::spawn(|| {
        // A fresh thread's first read refuses (debug builds); the
        // panic text names the escape hatches.
        let result = std::panic::catch_unwind(|| {
            let _ = env();
        });
        result.is_err()
    })
    .join()
    .expect("the spawned thread does not abort");
    assert!(unmarked, "the fresh thread's unframed read refuses");
}
