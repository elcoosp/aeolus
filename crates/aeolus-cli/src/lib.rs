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
//! - **Zero dependencies** beyond `aeolus-core` (hand-rolled arg parsing,
//!   std I/O only) — the same discipline as the core.
//! - **`run()` is the whole program**: arguments in, writers out, exit code
//!   back — fully testable without spawning a process (`tests/cli.rs`).
//! - Exit codes: `0` success (warnings are non-fatal, printed to stderr),
//!   `1` findings matched `--fail-on`, `2` usage/theme/IO errors.
//! - Every diagnostic is precise (the no-silent-failures rule): unknown
//!   classes carry suggestions, findings name the token, the string and
//!   the reason.

use aeolus_core::plan::diff;
use aeolus_core::{Breakpoint, Resolver, StylePlan, TailwindTheme, Warning, cn};
use std::fmt::Write as _;
use std::io::Write;

/// The CLI version, from the workspace manifest.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// One lint finding of `aeolus check`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    /// The offending token.
    pub token: String,
    /// The class string it came from.
    pub from: String,
    /// The precise reason (never empty).
    pub detail: String,
}

impl Finding {
    /// Rendered one-line form (`token: detail`).
    pub fn render(&self) -> String {
        format!("{}: {}", self.token, self.detail)
    }
}

/// Load a theme by spec: `shadcn` (the default), `shadcn-dark`, `default`
/// (the verbatim Tailwind v4 default theme), or a path to theme CSS.
pub fn load_theme(spec: &str) -> Result<TailwindTheme, String> {
    match spec {
        "shadcn" => aeolus_core::shadcn_default().map_err(|e| format!("shadcn theme: {e}")),
        "shadcn-dark" => aeolus_core::shadcn_dark().map_err(|e| format!("shadcn-dark theme: {e}")),
        "default" => Ok(TailwindTheme::new()),
        path => {
            let css = std::fs::read_to_string(path)
                .map_err(|e| format!("cannot read theme CSS {path:?}: {e}"))?;
            TailwindTheme::from_theme_css(&css).map_err(|e| format!("theme CSS {path:?}: {e}"))
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
                });
            }
            return findings;
        }
    };
    for w in &res.warnings {
        let token = warning_class(w).to_string();
        let detail = warning_detail(w).clone();
        findings.push(Finding {
            token,
            from: classes.to_string(),
            detail,
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
        other => {
            let _ = writeln!(err, "unknown command {other:?}\n{USAGE}");
            2
        }
    }
}

/// Parsed flag bag shared by the class-string commands.
struct Args {
    theme: String,
    summary: bool,
    json: bool,
    positional: Vec<String>,
}

fn parse_flags(rest: &[String], err: &mut dyn Write, allow: &[&str]) -> Option<Args> {
    let mut theme = "shadcn".to_string();
    let mut summary = false;
    let mut json = false;
    let mut positional = Vec::new();
    let mut i = 0usize;
    while i < rest.len() {
        let a = &rest[i];
        match a.as_str() {
            "--theme" if allow.contains(&"theme") => {
                i += 1;
                match rest.get(i) {
                    Some(v) => theme.clone_from(v),
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
        summary,
        json,
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
                    "{{\"file\":\"{}\",\"token\":\"{}\",\"detail\":\"{}\"}}",
                    json_escape(file),
                    json_escape(&f.token),
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
            .filter(|(_, f)| is_unknown(&f.detail))
            .count(),
        "conflict" => findings
            .iter()
            .filter(|(_, f)| f.detail.contains("conflict"))
            .count(),
        "unsupported" => findings
            .iter()
            .filter(|(_, f)| !is_unknown(&f.detail) && !f.detail.contains("conflict"))
            .count(),
        _ => 0,
    };
    i32::from(fails > 0)
}

fn is_unknown(detail: &str) -> bool {
    detail.starts_with("unknown class") || detail.contains("did you mean")
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

THEME: shadcn (default) | shadcn-dark | default | <path-to-css>
FAIL-ON: unknown (default) | unsupported | conflict | any | none
EXIT: 0 success · 1 findings · 2 usage/theme/IO error
";
