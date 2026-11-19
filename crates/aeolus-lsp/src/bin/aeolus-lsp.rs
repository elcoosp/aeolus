//! The editor entry point (the P5 row's LSP surface): the stdio
//! transport's thin shell over the headless engine — spawn this binary
//! from the editor's LSP client configuration. The state machine is
//! [`aeolus_lsp::handle_message`]'s; the pipeline behind it is the
//! same one the CLI and the macros run.
//!
//! An optional `--theme <path>` bakes the session's table from a theme
//! css file the way the CLI's `--theme` flag does, and the SAME file
//! rides the theme lifecycle: every
//! `workspace/didChangeWatchedFiles` event naming it re-bakes and
//! re-themes the running session (the reload seam's headless consumer
//! demonstration is `examples/lsp_theme_reload.rs`).

use std::path::PathBuf;

use aeolus_lsp::FileThemeLoader;

fn main() {
    if let Err(err) = serve() {
        eprintln!("aeolus-lsp: {err}");
        std::process::exit(1);
    }
}

/// The argv walk (the shell's own two-argument form: `--theme` then
/// the path), then the loop: the default (the embedded shadcn bake)
/// or the themed one. A bad `--theme` file is a STARTUP error — the
/// typed [`ThemeError`](aeolus_lsp::ThemeError) names it before the
/// transport starts, the honest-failure shape the CLI's own `--theme`
/// keeps. The path canonicalizes BEFORE the URI composes: the
/// watched-files identity and the goto-definition origin ride
/// `file://` + the ABSOLUTE path, so a relative invocation (`--theme
/// app.css`) answers the same URI the editor's events name — the
/// `file://{relative}` form a `Uri` parser reads as a host, never a
/// path, and the lifecycle silently never fires on it.
fn serve() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [] => aeolus_lsp::run_stdio(),
        [flag, path] if flag == "--theme" => {
            let path = PathBuf::from(path);
            let path = path
                .canonicalize()
                .map_err(|error| format!("the --theme file {}: {error}", path.display()))?;
            let loader = FileThemeLoader::new(format!("file://{}", path.display()), &path);
            aeolus_lsp::run_stdio_with_theme(loader)
        }
        [flag, ..] => Err(format!(
            "unknown argument {flag:?} (the one form: aeolus-lsp [--theme <path>])"
        )
        .into()),
    }
}
