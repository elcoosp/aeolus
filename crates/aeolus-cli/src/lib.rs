//! `aeolus` — the headless Aeolus CLI (v0.7, plan of record
//! `AEOLUS_V0.7.md` §3).
//!
//! The project's consumption paths were all GUI backends; the CLI is the
//! headless surface: resolve class strings into portable plan snapshots,
//! emit the verification CSS, diff two resolutions across the
//! cosmetic/geometric lanes, merge with `cn()`, and LINT class strings in
//! source files (unknown classes, unsupported utilities, conflicts).
//!
//! Design contract:
//! - **No external crates in any build** (hand-rolled arg parsing, std
//!   I/O only) — the same discipline as the core; the workspace-internal
//!   `aeolus-compile` join (the `explain` command, plan §5.5's
//!   tooling-parity line) rides the pipeline's own dependency-light
//!   graph. The `tracing` and `miette` cargo features are opt-in
//!   instrumentation seams (off by default, so the default dependency
//!   graph stays lean): `tracing` adds spans around the command
//!   dispatch and the per-file check loop; `miette` adds structured
//!   `Diagnostic` impls (code + help text) to the typed errors.
//! - **`run()` is the whole program**: arguments in, writers out, exit code
//!   back — fully testable without spawning a process (`tests/cli.rs`).
//! - Exit codes: `0` success (warnings are non-fatal, printed to stderr),
//!   `1` findings matched `--fail-on`, `2` usage/theme/IO errors.
//! - Every diagnostic is precise (the no-silent-failures rule): unknown
//!   classes carry suggestions, findings name the token, the string and
//!   the reason, and theme-spec failures are typed
//!   ([`ThemeSpecError`]), never bare strings.

use aeolus_compile::{
    ClassKeywords, Config, Env, RustSheet, compile, compile_theme, emit_rust, emit_rust_theme,
    interp_theme, keyword_texts, sort,
};
use aeolus_core::color::ParseError;
use aeolus_core::plan::diff;
use aeolus_core::{Breakpoint, Resolver, StylePlan, TailwindTheme, Warning, cn};
use aeolus_ir::{Cond, Decl, Prop, Val};
use aeolus_theme::{ThemeSource, TokenTable};
use std::collections::HashMap;
use std::fmt::Write as _;
use std::io::Write;

/// The CLI version, from the workspace manifest.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The structured classification of one [`Finding`] — the fail-on
/// modes key on the KIND, never on the prose (`--fail-on unknown`
/// matches the typed `Unknown` kind, not a `detail` substring). The
/// Display/render contract is unchanged; the kind exists so programmatic
/// consumers (wrappers, CI) can match without parsing messages.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FindingKind {
    /// No registered root under the spelling — the `--fail-on unknown`
    /// mode.
    Unknown,
    /// A known root the theme or the grammar refuses (unsupported
    /// utility, missing theme token, unparseable value) — the
    /// `--fail-on unsupported` mode.
    Unsupported,
    /// `cn()` would drop the token (a later class won its conflict
    /// group) — the `--fail-on conflict` mode.
    Conflict,
}

impl FindingKind {
    /// The stable machine name the `--json` output and the `--fail-on`
    /// values share.
    pub const fn name(self) -> &'static str {
        match self {
            FindingKind::Unknown => "unknown",
            FindingKind::Unsupported => "unsupported",
            FindingKind::Conflict => "conflict",
        }
    }
}

/// One lint finding of `aeolus check`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    /// The offending token.
    pub token: String,
    /// The class string it came from.
    pub from: String,
    /// The precise reason (never empty).
    pub detail: String,
    /// The structured classification ([`FindingKind`]) — the fail-on
    /// modes and the `--json` output key on this, never on the prose.
    pub kind: FindingKind,
}

impl Finding {
    /// Rendered one-line form (`token: detail`).
    pub fn render(&self) -> String {
        format!("{}: {}", self.token, self.detail)
    }
}

/// The typed failure of the `--theme <spec>` seam ([`load_theme`]).
///
/// Every variant renders the SAME text the historical `String` error
/// produced, so the `run()` stderr contract is unchanged; the type exists
/// so programmatic consumers (wrappers, tests) can match on the failure
/// mode instead of parsing prose. With the `miette` feature, the variants
/// additionally carry structured help text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThemeSpecError {
    /// `spec` named neither a built-in theme nor an existing CSS file.
    UnknownSpec {
        /// The offending spec, verbatim.
        spec: String,
    },
    /// A built-in palette failed to parse. Unreachable for the embedded
    /// data (compile-time constants); handled honestly anyway.
    BuiltIn {
        /// Which built-in spec (`shadcn` or `shadcn-dark`).
        spec: &'static str,
        /// The underlying theme-parse failure.
        source: ParseError,
    },
    /// The CSS path could not be read from disk.
    ReadFile {
        /// The offending path, verbatim.
        path: String,
        /// The OS-reported failure kind.
        kind: std::io::ErrorKind,
    },
    /// The CSS file was read but its `@theme`/`:root` declarations did
    /// not parse (e.g. a `calc()` referencing a var that is never
    /// defined).
    ParseCss {
        /// The offending path, verbatim.
        path: String,
        /// The underlying theme-parse failure.
        source: ParseError,
    },
}

impl std::fmt::Display for ThemeSpecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThemeSpecError::UnknownSpec { spec } => write!(
                f,
                "unknown theme spec {spec:?} — expected shadcn | shadcn-dark | default | <path-to-theme-css>"
            ),
            ThemeSpecError::BuiltIn { spec, source } => write!(f, "{spec} theme: {source}"),
            ThemeSpecError::ReadFile { path, kind } => {
                write!(f, "cannot read theme CSS {path:?}: {kind}")
            }
            ThemeSpecError::ParseCss { path, source } => {
                write!(f, "theme CSS {path:?}: {source}")
            }
        }
    }
}

impl std::error::Error for ThemeSpecError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ThemeSpecError::BuiltIn { source, .. } | ThemeSpecError::ParseCss { source, .. } => {
                Some(source)
            }
            ThemeSpecError::UnknownSpec { .. } | ThemeSpecError::ReadFile { .. } => None,
        }
    }
}

#[cfg(feature = "miette")]
impl miette::Diagnostic for ThemeSpecError {
    fn code(&self) -> Option<Box<dyn std::fmt::Display + '_>> {
        Some(Box::new(match self {
            ThemeSpecError::UnknownSpec { .. } => "aeolus::theme_spec::unknown",
            ThemeSpecError::BuiltIn { .. } => "aeolus::theme_spec::builtin",
            ThemeSpecError::ReadFile { .. } => "aeolus::theme_spec::read_file",
            ThemeSpecError::ParseCss { .. } => "aeolus::theme_spec::parse_css",
        }))
    }

    fn help(&self) -> Option<Box<dyn std::fmt::Display + '_>> {
        Some(Box::new(match self {
            ThemeSpecError::UnknownSpec { .. } => {
                "the theme spec accepts: shadcn | shadcn-dark | default | <path-to-theme-css>"
                    .to_string()
            }
            ThemeSpecError::BuiltIn { .. } => {
                "the embedded shadcn palettes are compile-time data; if this fires, report it as a bug"
                    .to_string()
            }
            ThemeSpecError::ReadFile { path, .. } => format!(
                "check that {path} exists and is readable, or pass a built-in spec (shadcn | shadcn-dark | default)"
            ),
            ThemeSpecError::ParseCss { .. } => {
                "the file was read but its @theme/:root declarations did not parse — every var() referenced inside calc() must be defined in the same file"
                    .to_string()
            }
        }))
    }
}

/// Load a theme by spec: `shadcn` (the default), `shadcn-dark`, `default`
/// (the verbatim Tailwind v4 default theme), or a path to theme CSS.
///
/// A spec is treated as a PATH when it contains a path separator or names
/// a `.css` extension (any letter case); any other bare word that is not a
/// built-in name is [`ThemeSpecError::UnknownSpec`] (the accepted forms are
/// in the message and in the miette help) instead of a misleading
/// file-not-found. Relative CSS files are still supported when written as a
/// path (`./custom.css`).
///
/// # Errors
/// A typed [`ThemeSpecError`] per failure mode: unknown spec, unreadable
/// file, unparseable CSS (and the never-expected built-in parse failure).
pub fn load_theme(spec: &str) -> Result<TailwindTheme, ThemeSpecError> {
    #[cfg(feature = "tracing")]
    tracing::debug!(spec = %spec, "loading theme by spec");
    match spec {
        "shadcn" => aeolus_core::shadcn_default().map_err(|e| ThemeSpecError::BuiltIn {
            spec: "shadcn",
            source: e,
        }),
        "shadcn-dark" => aeolus_core::shadcn_dark().map_err(|e| ThemeSpecError::BuiltIn {
            spec: "shadcn-dark",
            source: e,
        }),
        "default" => Ok(TailwindTheme::new()),
        spec if !(spec.contains('/')
            || spec.contains('\\')
            || std::path::Path::new(spec)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("css"))) =>
        {
            Err(ThemeSpecError::UnknownSpec {
                spec: spec.to_string(),
            })
        }
        path => {
            let css = std::fs::read_to_string(path).map_err(|e| ThemeSpecError::ReadFile {
                path: path.to_string(),
                kind: e.kind(),
            })?;
            TailwindTheme::from_theme_css(&css).map_err(|e| ThemeSpecError::ParseCss {
                path: path.to_string(),
                source: e,
            })
        }
    }
}

/// Heuristic class-string extractor (`aeolus check`).
///
/// Scans quoted strings (single, double, and Rust raw `r#"…"#`) and keeps a
/// string as a candidate class string when at least half of its
/// whitespace-separated tokens resolve with ZERO warnings and at least one
/// token resolves — the same "looks like Tailwind" shape the official
/// extractor uses. Prose, identifiers and non-class literals never reach
/// the linter; a candidate's remaining warnings ARE the findings.
pub fn extract_candidate_strings(source: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = source.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'"' | b'\'' => {
                let quote = bytes[i];
                let start = i + 1;
                let mut j = start;
                while j < bytes.len() && bytes[j] != quote && bytes[j] != b'\n' {
                    j += 1;
                }
                if j < bytes.len() && bytes[j] == quote {
                    push_candidate(&mut out, &source[start..j]);
                    i = j + 1;
                    continue;
                }
                i = start.max(j);
            }
            b'r' if i + 1 < bytes.len() && bytes[i + 1] == b'"' => {
                // Rust raw string: r"…" or r#"…"#.
                let mut hashes = 0usize;
                let mut k = i + 1;
                while k < bytes.len() && bytes[k] == b'#' {
                    hashes += 1;
                    k += 1;
                }
                if k >= bytes.len() || bytes[k] != b'"' {
                    i += 1;
                    continue;
                }
                let body_start = k + 1;
                let mut needle = String::from("\"");
                for _ in 0..hashes {
                    needle.push('#');
                }
                if let Some(end) = source[body_start..].find(&needle) {
                    push_candidate(&mut out, &source[body_start..body_start + end]);
                    i = body_start + end + needle.len();
                    continue;
                }
                i = body_start;
            }
            _ => i += 1,
        }
    }
    out
}

fn push_candidate(out: &mut Vec<String>, s: &str) {
    let s = s.trim();
    if s.is_empty() {
        return;
    }
    // Cheap pre-filter: a class string essentially always carries at least
    // one token with a `-` (utility-value) or `:` (variant). Prose rarely
    // passes; the resolver filter below does the precise work.
    if !s
        .split_ascii_whitespace()
        .any(|t| t.contains('-') || t.contains(':'))
    {
        return;
    }
    out.push(s.to_string());
}

/// Lint ONE candidate string: resolve it and turn every warning into a
/// [`Finding`]; additionally report `cn()` conflicts (tokens the merge
/// would drop).
pub fn check_string(classes: &str, theme: &TailwindTheme) -> Vec<Finding> {
    let resolver = Resolver::new(theme);
    let mut findings = Vec::new();
    let res = match resolver.resolve(classes) {
        Ok(res) => res,
        Err(errors) => {
            for e in errors {
                findings.push(Finding {
                    token: classes.to_string(),
                    from: classes.to_string(),
                    detail: format!("color parse error: {e}"),
                    kind: FindingKind::Unsupported,
                });
            }
            return findings;
        }
    };
    for w in &res.warnings {
        let token = warning_class(w).to_string();
        let detail = warning_detail(w).clone();
        let kind = match w {
            Warning::Unknown { .. } => FindingKind::Unknown,
            Warning::Unsupported { .. }
            | Warning::MissingThemeToken { .. }
            | Warning::Unparseable { .. } => FindingKind::Unsupported,
        };
        findings.push(Finding {
            token,
            from: classes.to_string(),
            detail,
            kind,
        });
    }
    // Conflict detection: cn() over the tokens; anything dropped is a
    // conflict with the later class that won its group.
    let tokens: Vec<&str> = classes.split_ascii_whitespace().collect();
    if tokens.len() > 1 {
        let merged = cn(&tokens);
        let survivors: Vec<&str> = merged.split_ascii_whitespace().collect();
        for t in &tokens {
            if !survivors.contains(t) {
                // Attribute the winner: the last token that absorbs t.
                let mut winner = None;
                for other in &tokens {
                    if other == t {
                        continue;
                    }
                    let pair = cn(&[*t, other]);
                    if pair.split_ascii_whitespace().count() == 1
                        && pair.split_ascii_whitespace().next() == Some(*other)
                    {
                        winner = Some(*other);
                    }
                }
                findings.push(Finding {
                    token: (*t).to_string(),
                    from: classes.to_string(),
                    detail: match winner {
                        Some(w) => {
                            format!("cn() conflict: dropped by {w:?} (later in the string wins)")
                        }
                        None => "cn() conflict: dropped by the merge".to_string(),
                    },
                    kind: FindingKind::Conflict,
                });
            }
        }
    }
    findings
}

fn warning_class(w: &Warning) -> &str {
    match w {
        Warning::Unsupported { class, .. } => class,
        Warning::Unknown { class, .. } => class,
        Warning::MissingThemeToken { class, .. } => class,
        Warning::Unparseable { class, .. } => class,
    }
}

fn warning_detail(w: &Warning) -> String {
    match w {
        Warning::Unsupported { reason, .. } => (*reason).to_string(),
        Warning::Unknown { suggestion, .. } => match suggestion {
            Some(s) => format!("unknown class — did you mean {s:?}?"),
            None => "unknown class".to_string(),
        },
        Warning::MissingThemeToken { token, .. } => {
            format!("theme token {token:?} is missing")
        }
        Warning::Unparseable { message, .. } => message.clone(),
    }
}

/// Run the CLI: arguments (WITHOUT the argv-0 program name), two sinks,
/// and the process exit code back.
pub fn run<I>(args: I, out: &mut dyn Write, err: &mut dyn Write) -> i32
where
    I: IntoIterator<Item = String>,
{
    #[cfg(feature = "tracing")]
    let _run_span = tracing::debug_span!("aeolus_run").entered();
    let argv: Vec<String> = args.into_iter().collect();
    if argv.iter().any(|a| a == "--help" || a == "-h") {
        let _ = write!(out, "{USAGE}");
        return 0;
    }
    if argv.iter().any(|a| a == "--version") {
        let _ = writeln!(out, "aeolus {VERSION}");
        return 0;
    }
    let Some(cmd) = argv.first() else {
        let _ = write!(err, "{USAGE}");
        return 2;
    };
    let rest = &argv[1..];
    match cmd.as_str() {
        "resolve" => cmd_resolve(rest, out, err),
        "css" => cmd_css(rest, out, err),
        "diff" => cmd_diff(rest, out, err),
        "cn" => cmd_cn(rest, out, err),
        "check" => cmd_check(rest, out, err),
        "explain" => cmd_explain(rest, out, err),
        "sort" => cmd_sort(rest, out, err),
        "expand" => cmd_expand(rest, out, err),
        other => {
            let _ = writeln!(err, "unknown command {other:?}\n{USAGE}");
            2
        }
    }
}

/// Parsed flag bag shared by the class-string commands.
struct Args {
    theme: String,
    /// Whether the caller actually passed `--theme` (the default spec is
    /// NOT a passed theme — `explain`'s themeless emit rides on this).
    theme_given: bool,
    summary: bool,
    json: bool,
    /// The `--dark` probe flag (`expand` only): lift the mode tier the
    /// interp resolves under.
    dark: bool,
    positional: Vec<String>,
}

fn parse_flags(rest: &[String], err: &mut dyn Write, allow: &[&str]) -> Option<Args> {
    let mut theme = "shadcn".to_string();
    let mut theme_given = false;
    let mut summary = false;
    let mut json = false;
    let mut dark = false;
    let mut positional = Vec::new();
    let mut i = 0usize;
    while i < rest.len() {
        let a = &rest[i];
        match a.as_str() {
            "--theme" if allow.contains(&"theme") => {
                i += 1;
                match rest.get(i) {
                    Some(v) => {
                        theme.clone_from(v);
                        theme_given = true;
                    }
                    None => {
                        let _ = writeln!(
                            err,
                            "--theme needs a value (shadcn|shadcn-dark|default|<path>)"
                        );
                        return None;
                    }
                }
            }
            "--summary" if allow.contains(&"summary") => summary = true,
            "--json" if allow.contains(&"json") => json = true,
            "--dark" if allow.contains(&"dark") => dark = true,
            other => {
                if other.starts_with("--") && other != "--" {
                    let _ = writeln!(err, "unknown flag {other:?} for this command");
                    return None;
                }
                positional.push(other.trim_matches('"').to_string());
            }
        }
        i += 1;
    }
    Some(Args {
        theme,
        theme_given,
        summary,
        json,
        dark,
        positional,
    })
}

/// Load the theme named by `args.theme`, reporting errors to `err`.
fn theme_or_report(args: &Args, err: &mut dyn Write) -> Option<TailwindTheme> {
    match load_theme(&args.theme) {
        Ok(t) => Some(t),
        Err(e) => {
            let _ = writeln!(err, "error: {e}");
            None
        }
    }
}

/// Report resolve warnings to stderr (non-fatal, exit stays 0).
fn report_warnings(warnings: &[Warning], err: &mut dyn Write) {
    for w in warnings {
        let _ = writeln!(err, "warning: {}: {}", warning_class(w), warning_detail(w));
    }
}

/// Load the V1 token table `explain` lowers colour tokens through
/// (plan §3.4's CSS-first pass): the two built-ins the pipeline embeds
/// plus a CSS file path — the same `--theme` spec grammar the v0.x
/// commands take, minus `default` (no V1 source exists for it, reported
/// as a usage error, never guessed). The source resolves into the
/// runtime tier (the T2 table the plan's dynamic lane uses — the same
/// construction the emit arms' own doctests build).
fn token_table_or_report(spec: &str, err: &mut dyn Write) -> Option<TokenTable> {
    let source = match spec {
        "shadcn" => ThemeSource::shadcn().expect("the built-in shadcn source parses"),
        "shadcn-dark" => {
            ThemeSource::shadcn_dark().expect("the built-in shadcn-dark source parses")
        }
        "default" => {
            let _ = writeln!(
                err,
                "error: explain's --theme accepts shadcn | shadcn-dark | <path> (default has no V1 source)"
            );
            return None;
        }
        path => match std::fs::read_to_string(path) {
            Ok(css) => match ThemeSource::from_css(&css, None) {
                Ok(source) => source,
                Err(e) => {
                    let _ = writeln!(err, "error: theme CSS: {e}");
                    return None;
                }
            },
            Err(e) => {
                let _ = writeln!(err, "error: cannot read theme CSS {path}: {e}");
                return None;
            }
        },
    };
    match TokenTable::runtime(&source) {
        Ok(table) => Some(table),
        Err(e) => {
            let _ = writeln!(err, "error: token table: {e}");
            None
        }
    }
}

/// `explain` (plan §5.5's tooling-parity line, "class → Rust it emits"):
/// render the per-class Rust fragments the Target::Rust arm emits —
/// one `Layer::new` expression per lowered class, keyword and token
/// slots annotated with the names only the pipeline can see — and every
/// class that does not lower as the SAME typed, coded diagnostics the
/// pipeline reports (exit 1 findings, never a silent drop). Without
/// `--theme` the emit stays themeless: colour tokens are the typed
/// boundary whose diagnostic names [`emit_rust_theme`] as the lowering
/// path — the honest default, not a gap.
fn cmd_explain(rest: &[String], out: &mut dyn Write, err: &mut dyn Write) -> i32 {
    #[cfg(feature = "tracing")]
    let _explain_span = tracing::debug_span!("aeolus_explain").entered();
    let Some(args) = parse_flags(rest, err, &["theme"]) else {
        return 2;
    };
    if args.positional.is_empty() {
        let _ = writeln!(err, "explain needs a class string\n{USAGE}");
        return 2;
    }
    let classes = args.positional.join(" ");
    let config = Config::default();
    let sheet = if args.theme_given {
        let Some(table) = token_table_or_report(&args.theme, err) else {
            return 2;
        };
        emit_rust_theme(&classes, &table, &config)
    } else {
        emit_rust(&classes, &config)
    };
    match sheet {
        Ok(sheet) => write_sheet(&sheet, out, err),
        // The pipeline's only hard error: empty input (per-class misses
        // ride the sheet as diagnostics, never this arm).
        Err(e) => {
            let _ = writeln!(err, "error: {e}");
            2
        }
    }
}

/// The sheet's two honest channels: fragments to stdout (one header per
/// lowered class, the embeddable expression indented beneath), the
/// misses to stderr as coded diagnostics — findings, exit 1.
fn write_sheet(sheet: &RustSheet, out: &mut dyn Write, err: &mut dyn Write) -> i32 {
    for fragment in &sheet.fragments {
        let _ = writeln!(out, "{}", fragment.class);
        for line in fragment.tokens.lines() {
            let _ = writeln!(out, "  {line}");
        }
        let _ = writeln!(out);
    }
    if sheet.diagnostics.is_empty() {
        return 0;
    }
    for diagnostic in &sheet.diagnostics {
        let _ = writeln!(err, "error: {diagnostic} [{}]", diagnostic.kind.code());
    }
    1
}

/// `sort` (plan §5.5's tooling-parity line, "class sorting (official
/// order from the registry)"): reorder a class string into the registry
/// rows' canonical utility progression, the variant buckets riding
/// after the base by the cascade specificity the IR itself sorts layers
/// by. Classes the seam cannot order ride the tail in source order and
/// report the SAME typed, coded diagnostics the pipeline reports (exit
/// 1 findings, never a silent drop). `--json` renders the same report
/// as one object: the sorted string plus the diagnostics array.
fn cmd_sort(rest: &[String], out: &mut dyn Write, err: &mut dyn Write) -> i32 {
    #[cfg(feature = "tracing")]
    let _sort_span = tracing::debug_span!("aeolus_sort").entered();
    let Some(args) = parse_flags(rest, err, &["json"]) else {
        return 2;
    };
    if args.positional.is_empty() {
        let _ = writeln!(err, "sort needs a class string\n{USAGE}");
        return 2;
    }
    let classes = args.positional.join(" ");
    match sort(&classes) {
        Ok(sorted) => {
            let text = sorted.to_class_string();
            if args.json {
                let items: Vec<String> = sorted
                    .diagnostics
                    .iter()
                    .map(|diagnostic| {
                        format!(
                            "{{\"raw\":\"{}\",\"code\":\"{}\",\"message\":\"{}\"}}",
                            json_escape(&diagnostic.raw),
                            diagnostic.kind.code(),
                            json_escape(&diagnostic.to_string())
                        )
                    })
                    .collect();
                let _ = writeln!(
                    out,
                    "{{\"sorted\":\"{}\",\"diagnostics\":[{}]}}",
                    json_escape(&text),
                    items.join(",")
                );
            } else {
                let _ = writeln!(out, "{text}");
                for diagnostic in &sorted.diagnostics {
                    let _ = writeln!(err, "error: {diagnostic} [{}]", diagnostic.kind.code());
                }
            }
            i32::from(!sorted.diagnostics.is_empty())
        }
        Err(e) => {
            let _ = writeln!(err, "error: {e}");
            2
        }
    }
}

/// The text form of one effective declaration's value (the `expand`
/// render): the keyword slots resolved to their ROW text (the annotation
/// the bare [`Val::Kw`] slot cannot recover — the winner's own class's
/// row text, see [`effective_keyword_texts`]), colours as hex, lengths
/// in their own units. The token forms never reach this render from the
/// interp (the source resolves them); they stay the honest fallback
/// text for the defensive paths.
fn value_text(val: Val, kw: Option<&str>) -> String {
    match val {
        Val::Px(v) => format!("{v}px"),
        Val::Rem(v) => format!("{v}rem"),
        Val::Pct(v) => format!("{v}%"),
        Val::Num(v) => format!("{v}"),
        Val::Rgba([r, g, b, 255]) => format!("#{r:02x}{g:02x}{b:02x}"),
        Val::Rgba([r, g, b, a]) => format!("#{r:02x}{g:02x}{b:02x}{a:02x}"),
        Val::Kw(_) => {
            _ = kw;
            kw.map_or_else(
                || "keyword slot (row text unavailable)".to_string(),
                str::to_string,
            )
        }
        Val::Tok { .. } | Val::ModePair(_) => {
            "token slot (the theme pass resolves the id)".to_string()
        }
    }
}

/// The last keyword text per `(cond, prop)`, in source order — the
/// annotation table the effective render reads. The interp attributes
/// each winning declaration to its condition ([`Interped::conds`], the
/// cascade's winner attribution); the winner's row text is this table's
/// entry for that exact `(cond, prop)` pair — the winner's own class's
/// row text, never a lucky sibling's (an unfired layer's text can no
/// longer shadow the winner's).
fn last_keyword_texts(annotations: &[ClassKeywords]) -> HashMap<(Cond, Prop), &'static str> {
    let mut last: HashMap<(Cond, Prop), &'static str> = HashMap::new();
    for class in annotations {
        for &(prop, text) in &class.keywords {
            if !text.is_empty() {
                last.insert((class.cond, prop), text);
            }
        }
    }
    last
}

/// `expand` (§3.2's CLI row, "expand"; the interp arm's human-readable
/// form): render a class string's EFFECTIVE declarations under one probe
/// `Env` — the per-longhand cascade the specificity sort encodes, the
/// keyword slots resolved to their row texts, the honest exclusions as
/// typed notes. Without `--theme`, colour tokens and modifiers stay the
/// typed boundaries whose diagnostics the compile walk reports (exit 1
/// findings, the render still carries what lowered). `--dark` lifts the
/// mode tier; the probe states rest. `--json` renders the same report
/// as one object: the declarations array plus the notes.
fn cmd_expand(rest: &[String], out: &mut dyn Write, err: &mut dyn Write) -> i32 {
    #[cfg(feature = "tracing")]
    let _expand_span = tracing::debug_span!("aeolus_expand").entered();
    let Some(args) = parse_flags(rest, err, &["theme", "json", "dark"]) else {
        return 2;
    };
    if args.positional.is_empty() {
        let _ = writeln!(err, "expand needs a class string\n{USAGE}");
        return 2;
    }
    let classes = args.positional.join(" ");
    let config = Config::default();
    // The table stays in scope: the themed walk resolves the token forms
    // through it ([`interp_theme`]), the themeless walk refuses them (the
    // typed boundary the exit-1 findings carry).
    let (compiled, table) = if args.theme_given {
        let Some(table) = token_table_or_report(&args.theme, err) else {
            return 2;
        };
        (compile_theme(&classes, &table, &config), Some(table))
    } else {
        (compile(&classes, &config), None)
    };
    let compiled = match compiled {
        Ok(compiled) => compiled,
        Err(e) => {
            let _ = writeln!(err, "error: {e}");
            return 2;
        }
    };
    let env = Env {
        dark: args.dark,
        ..Env::default()
    };
    let interped = match &table {
        Some(table) => interp_theme(&compiled.program, table, &env),
        None => compiled.interp(&env),
    };
    let interped = match interped {
        Ok(interped) => interped,
        Err(e) => {
            let _ = writeln!(err, "error: {e}");
            return 2;
        }
    };
    // The annotations ride the same walk the compile ran (the contract:
    // only the empty input errors it, and `compiled` proves non-empty).
    let last =
        last_keyword_texts(&keyword_texts(&classes, &config).expect("non-empty input annotates"));
    let kw_text = |index: usize, decl: &Decl| -> Option<&'static str> {
        let cond = *interped.conds.get(index)?;
        last.get(&(cond, decl.prop)).copied()
    };
    if args.json {
        let decls: Vec<String> = interped
            .decls
            .iter()
            .enumerate()
            .map(|(index, decl)| {
                format!(
                    "{{\"prop\":\"{}\",\"value\":\"{}\"}}",
                    json_escape(decl.prop.name()),
                    json_escape(&value_text(decl.val, kw_text(index, decl)))
                )
            })
            .collect();
        let notes: Vec<String> = interped
            .notes
            .iter()
            .map(|note| format!("\"{}\"", json_escape(&note.to_string())))
            .collect();
        let _ = writeln!(
            out,
            "{{\"declarations\":[{}],\"notes\":[{}]}}",
            decls.join(","),
            notes.join(",")
        );
    } else {
        for (index, decl) in interped.decls.iter().enumerate() {
            let _ = writeln!(
                out,
                "{} = {}",
                decl.prop.name(),
                value_text(decl.val, kw_text(index, decl))
            );
        }
        for note in &interped.notes {
            let _ = writeln!(out, "note: {note}");
        }
    }
    // The compile-level boundaries (themeless tokens, modifiers) are
    // findings: the render still carries what lowered, exit 1 reports.
    for diagnostic in &compiled.diagnostics {
        let _ = writeln!(err, "error: {diagnostic} [{}]", diagnostic.kind.code());
    }
    i32::from(!compiled.diagnostics.is_empty())
}

fn cmd_resolve(rest: &[String], out: &mut dyn Write, err: &mut dyn Write) -> i32 {
    let Some(args) = parse_flags(rest, err, &["theme", "summary", "json"]) else {
        return 2;
    };
    let Some(theme) = theme_or_report(&args, err) else {
        return 2;
    };
    let classes = args.positional.join(" ");
    let resolver = Resolver::new(&theme);
    let resolution = match resolver.resolve(&classes) {
        Ok(r) => r,
        Err(errors) => {
            for e in errors {
                let _ = writeln!(err, "error: color parse: {e}");
            }
            return 2;
        }
    };
    report_warnings(&resolution.warnings, err);
    if args.summary {
        write_summary(out, &resolution.plan);
    } else {
        // The default artifact IS the snapshot: a portable, canonical,
        // strictly parseable JSON document.
        let _ = writeln!(out, "{}", resolution.plan.to_snapshot_json());
    }
    0
}

fn write_summary(out: &mut dyn Write, plan: &StylePlan) {
    let slots: Vec<String> = plan.base.slots().iter().map(|s| format!("{s:?}")).collect();
    let _ = writeln!(
        out,
        "base: {}",
        if slots.is_empty() {
            "-".to_string()
        } else {
            slots.join(", ")
        }
    );
    if !plan.layers.is_empty() {
        let conds: Vec<String> = plan
            .layers
            .iter()
            .map(|l| format!("{:?}", l.condition))
            .collect();
        let _ = writeln!(out, "layers: {}", conds.join(", "));
    }
    if !plan.responsive.is_empty() {
        let bps: Vec<String> = plan
            .responsive
            .keys()
            .map(|b| match b {
                Breakpoint::Sm => "sm",
                Breakpoint::Md => "md",
                Breakpoint::Lg => "lg",
                Breakpoint::Xl => "xl",
                Breakpoint::Xxl => "2xl",
            })
            .map(String::from)
            .collect();
        let _ = writeln!(out, "responsive: {}", bps.join(", "));
    }
    if plan.dark.is_some() {
        let _ = writeln!(out, "dark: yes");
    }
    if !plan.rules.is_empty() {
        let _ = writeln!(out, "rules: {}", plan.rules.len());
    }
}

fn cmd_css(rest: &[String], out: &mut dyn Write, err: &mut dyn Write) -> i32 {
    let Some(args) = parse_flags(rest, err, &["theme"]) else {
        return 2;
    };
    let Some(theme) = theme_or_report(&args, err) else {
        return 2;
    };
    let classes = args.positional.join(" ");
    let resolver = Resolver::new(&theme);
    match resolver.to_css(&classes) {
        Ok(output) => {
            let _ = writeln!(out, "{}", aeolus_core::format_rules(&output));
            0
        }
        Err(errors) => {
            for e in errors {
                let _ = writeln!(err, "error: {e}");
            }
            2
        }
    }
}

fn cmd_diff(rest: &[String], out: &mut dyn Write, err: &mut dyn Write) -> i32 {
    let Some(args) = parse_flags(rest, err, &["theme", "json"]) else {
        return 2;
    };
    if args.positional.len() != 2 {
        let _ = writeln!(
            err,
            "diff takes exactly two class strings: aeolus diff <A> <B>"
        );
        return 2;
    }
    let Some(theme) = theme_or_report(&args, err) else {
        return 2;
    };
    let resolver = Resolver::new(&theme);
    let (a_str, b_str) = (&args.positional[0], &args.positional[1]);
    let a = match resolver.resolve(a_str) {
        Ok(r) => r,
        Err(errors) => {
            for e in errors {
                let _ = writeln!(err, "error: {a_str:?}: {e}");
            }
            return 2;
        }
    };
    let b = match resolver.resolve(b_str) {
        Ok(r) => r,
        Err(errors) => {
            for e in errors {
                let _ = writeln!(err, "error: {b_str:?}: {e}");
            }
            return 2;
        }
    };
    report_warnings(&a.warnings, err);
    report_warnings(&b.warnings, err);
    let d = diff(&a.plan.base, &b.plan.base);
    if args.json {
        let changed: Vec<String> = d.changed.iter().map(|s| format!("{s:?}")).collect();
        let _ = writeln!(
            out,
            "{{\"changed\":[{}],\"needs_layout\":{}}}",
            changed
                .iter()
                .map(|s| format!("\"{s}\""))
                .collect::<Vec<_>>()
                .join(","),
            d.needs_layout()
        );
    } else {
        if d.changed.is_empty() {
            let _ = writeln!(out, "no base-layer changes");
        }
        for s in &d.changed {
            let lane = match s.lane() {
                aeolus_core::Lane::Cosmetic => "cosmetic",
                aeolus_core::Lane::Geometric => "geometric",
            };
            let _ = writeln!(out, "{s:?} ({lane})");
        }
        if !d.changed.is_empty() {
            let _ = writeln!(
                out,
                "lane: {}",
                if d.needs_layout() {
                    "relayout"
                } else {
                    "repaint-only"
                }
            );
        }
    }
    0
}

fn cmd_cn(rest: &[String], out: &mut dyn Write, err: &mut dyn Write) -> i32 {
    let Some(args) = parse_flags(rest, err, &[]) else {
        return 2;
    };
    if args.positional.is_empty() {
        let _ = writeln!(err, "cn takes at least one class string group");
        return 2;
    }
    let refs: Vec<&str> = args.positional.iter().map(String::as_str).collect();
    let _ = writeln!(out, "{}", cn(&refs));
    0
}

fn cmd_check(rest: &[String], out: &mut dyn Write, err: &mut dyn Write) -> i32 {
    // check accepts --theme, --fail-on and --json.
    let mut fail_on = "unknown".to_string();
    let mut theme_spec = "shadcn".to_string();
    let mut json = false;
    let mut files: Vec<String> = Vec::new();
    let mut i = 0usize;
    while i < rest.len() {
        match rest[i].as_str() {
            "--theme" => {
                i += 1;
                match rest.get(i) {
                    Some(v) => theme_spec.clone_from(v),
                    None => {
                        let _ = writeln!(err, "--theme needs a value");
                        return 2;
                    }
                }
            }
            "--fail-on" => {
                i += 1;
                match rest.get(i) {
                    Some(v)
                        if matches!(
                            v.as_str(),
                            "unknown" | "unsupported" | "conflict" | "any" | "none"
                        ) =>
                    {
                        fail_on.clone_from(v);
                    }
                    _ => {
                        let _ =
                            writeln!(err, "--fail-on takes unknown|unsupported|conflict|any|none");
                        return 2;
                    }
                }
            }
            "--json" => json = true,
            other if other.starts_with("--") => {
                let _ = writeln!(err, "unknown flag {other:?}");
                return 2;
            }
            other => files.push(other.to_string()),
        }
        i += 1;
    }
    if files.is_empty() {
        let _ = writeln!(err, "check takes at least one file");
        return 2;
    }
    let Some(theme) = load_theme(&theme_spec)
        .map_err(|e| {
            let _ = writeln!(err, "error: {e}");
        })
        .ok()
    else {
        return 2;
    };
    let mut findings: Vec<(String, Finding)> = Vec::new();
    let mut io_errors = 0usize;
    for file in &files {
        #[cfg(feature = "tracing")]
        let _file_span = tracing::debug_span!("aeolus_check_file", file = file.as_str()).entered();
        let source = match std::fs::read_to_string(file) {
            Ok(s) => s,
            Err(e) => {
                let _ = writeln!(err, "error: cannot read {file:?}: {e}");
                io_errors += 1;
                continue;
            }
        };
        for candidate in extract_candidate_strings(&source) {
            for f in check_string(&candidate, &theme) {
                findings.push((file.clone(), f));
            }
        }
    }
    if io_errors > 0 {
        return 2;
    }
    if json {
        let items: Vec<String> = findings
            .iter()
            .map(|(file, f)| {
                format!(
                    "{{\"file\":\"{}\",\"token\":\"{}\",\"kind\":\"{}\",\"detail\":\"{}\"}}",
                    json_escape(file),
                    json_escape(&f.token),
                    f.kind.name(),
                    json_escape(&f.detail)
                )
            })
            .collect();
        let _ = writeln!(out, "[{}]", items.join(","));
    } else {
        for (file, f) in &findings {
            let _ = writeln!(out, "{file}: {}", f.render());
        }
        let _ = writeln!(out, "{} finding(s)", findings.len());
    }
    let fails = match fail_on.as_str() {
        "none" => 0,
        "any" => findings.len(),
        "unknown" => findings
            .iter()
            .filter(|(_, f)| f.kind == FindingKind::Unknown)
            .count(),
        "conflict" => findings
            .iter()
            .filter(|(_, f)| f.kind == FindingKind::Conflict)
            .count(),
        "unsupported" => findings
            .iter()
            .filter(|(_, f)| f.kind == FindingKind::Unsupported)
            .count(),
        _ => 0,
    };
    i32::from(fails > 0)
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out
}

const USAGE: &str = "\
aeolus — Tailwind v4 class compiler, headless surface

USAGE:
  aeolus resolve [--theme <spec>] [--summary] <classes...>   plan snapshot JSON (the artifact)
  aeolus css     [--theme <spec>] <classes...>               verification CSS (v4 shapes)
  aeolus diff    [--theme <spec>] [--json] <A> <B>           base-layer diff with lanes
  aeolus cn      <classes...>                                cn() conflict merge
  aeolus check   [--theme <spec>] [--fail-on <mode>] [--json] <files...>
                                                             lint class strings in sources
  aeolus explain [--theme <spec>] <classes...>               per-class Rust tokens (the emit (a) arm)
  aeolus sort    [--json] <classes...>                       registry-order class sort (the §5.5 parity line)
  aeolus expand  [--theme <spec>] [--dark] [--json] <classes...>
                                                             effective declarations (the interp (c) arm)

THEME: shadcn (default) | shadcn-dark | default | <path-to-css>
       (`explain` accepts shadcn | shadcn-dark | <path> — the V1 theme pass)
FAIL-ON: unknown (default) | unsupported | conflict | any | none
EXIT: 0 success · 1 findings · 2 usage/theme/IO error
";
