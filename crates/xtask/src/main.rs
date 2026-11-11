//! The `xtask` binary — the library's `run` wired to the process
//! (the aeolus-cli main's own shape: skip argv-0, two sinks, exit code
//! out; no logic lives here).

fn main() {
    let code = xtask::run(
        std::env::args().skip(1),
        &mut std::io::stdout(),
        &mut std::io::stderr(),
    );
    std::process::exit(code);
}
