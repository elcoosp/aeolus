//! Consumer demonstration: the `xtask` driver's LIBRARY face — the
//! whole program is the library `run(args, out, err) -> code` shape
//! (the aeolus-cli precedent, argument list WITHOUT the argv-0 program
//! name, two sinks, exit code back), so a host that wants the drift
//! gate inside its own tooling calls the function instead of shelling
//! out to `cargo xtask`.
//!
//! ```sh
//! cargo run -p xtask --example drift_walkthrough
//! ```
//!
//! The commands below are the READ-ONLY half of the surface (`list`
//! and `verify-tables` compare against the committed bytes, they never
//! write) — the example mutates nothing: a clean tree verifies green,
//! a drifted artifact reports the drift and the non-zero exit code,
//! which is exactly what a CI gate or a pre-commit hook consumes.

use std::process::ExitCode;

fn main() -> ExitCode {
    // `list`: every committed artifact (name + workspace path) — the
    // surface the generator story owns.
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = xtask::run(["list".to_owned()], &mut out, &mut err);
    println!("── xtask list (exit {code}) ──");
    print!("{}", String::from_utf8_lossy(&out));

    // `verify-tables`: the drift gate. Regenerate every artifact from
    // the rows, compare against the committed bytes, exit 1 on ANY
    // drift — the same check `aeolus-data/tests/codegen.rs` pins in
    // `cargo test`, from the driver side.
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = xtask::run(["verify-tables".to_owned()], &mut out, &mut err);
    println!("── xtask verify-tables (exit {code}) ──");
    print!("{}", String::from_utf8_lossy(&out));
    if !err.is_empty() {
        print!("{}", String::from_utf8_lossy(&err));
    }

    // The gate's verdict as the process code: `0` clean, `1` drift —
    // the shape a CI step or a host's own pre-commit hook returns.
    ExitCode::from(code as u8)
}
