//! The editor entry point (the P5 row's LSP surface): the stdio
//! transport's thin shell over the headless engine — spawn this binary
//! from the editor's LSP client configuration. The state machine is
//! [`aeolus_lsp::handle_message`]'s; the pipeline behind it is the
//! same one the CLI and the macros run.

fn main() {
    if let Err(err) = aeolus_lsp::run_stdio() {
        eprintln!("aeolus-lsp: {err}");
        std::process::exit(1);
    }
}
