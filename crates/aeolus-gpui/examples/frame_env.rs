//! The frame `Env`'s debug-assert, headless: the plan §3.5 note's
//! own ask ("add a debug-assert that `begin_frame` ran") and §9's
//! mitigation row, demonstrated from the consumer's side.
//!
//! Run: `cargo run -p aeolus-gpui --example frame_env`
//!
//! The contract has three faces:
//!
//! 1. an unframed read (neither `begin_frame` nor `set_env` has run
//!    on the thread) REFUSES in debug builds — the resting default's
//!    dark/viewport answers would ride silently otherwise;
//! 2. `set_env(env)` is the deliberate escape hatch — an explicit env
//!    marks the thread and the read answers (the tests/non-UI-threads
//!    lane; the `tw_env*` entries' explicit `Env` values are the other
//!    half of the same hatch);
//! 3. the frame beat: `begin_frame(window)` marks the thread once per
//!    render (`AeolusRoot` calls it for you — GPUI-side, not
//!    headless, so this example demonstrates the marking through the
//!    escape hatch).
//!
//! In a release build the assert compiles out — face 1 prints the
//! compiled-out answer instead of the refusal, honestly.
use std::panic;

use aeolus_gpui::env;
use aeolus_gpui::prelude::{Env, set_env};

fn main() {
    // Face 2: the escape hatch — an explicit env marks this thread.
    set_env(Env::default());
    let resting = env();
    println!(
        "the marked thread answers: dark = {}, viewport_rem = {:.3}",
        resting.dark, resting.viewport_rem
    );

    // Face 1: a fresh thread's unframed read. In debug builds the
    // read panics with the assert's own text (caught here so the
    // demo prints it); in release builds the assert compiles out and
    // the read answers the resting default — the build shape is the
    // demo's honest report.
    let previous_hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));
    let fresh = std::thread::spawn(|| {
        panic::catch_unwind(|| {
            let unframed = env();
            format!("answered: dark = {}", unframed.dark)
        })
    })
    .join()
    .expect("the demo thread does not abort");
    panic::set_hook(previous_hook);
    match fresh {
        Err(payload) => {
            let text = payload
                .downcast_ref::<&str>()
                .map(|s| (*s).to_string())
                .or_else(|| payload.downcast_ref::<String>().cloned())
                .unwrap_or_default();
            println!("the unframed read refused (debug): {text}");
        }
        Ok(answer) => {
            println!("the unframed read answered (release: the assert compiles out): {answer}");
        }
    }

    // Face 3: the frame beat — `begin_frame(&window)` marks the
    // thread per render and `AeolusRoot` calls it for you. The
    // marked thread here keeps answering; a real app marks the same
    // way from its render loop.
    set_env(Env {
        dark: true,
        ..Env::default()
    });
    let dark = env();
    println!("after the frame's refresh: dark = {}", dark.dark);
}
