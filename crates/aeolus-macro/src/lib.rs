//! `tw!` — the compile-time front door.
//!
//! Validates a class string against the v4 grammar at compile time and
//! returns it unchanged as `&'static str`. Implementation is deliberately
//! `syn`-free (mission brief §3.3): the macro parses a single string literal
//! straight off the `TokenStream`.
//!
//! ```ignore
//! const CLASSES: &str = tw!("p-4 bg-primary hover:bg-primary/90"); // compiles
//! const BAD: &str = tw!("bg-];:["); // compile error with diagnostics
//! ```

use proc_macro::{Delimiter, TokenStream, TokenTree};

/// Validate class strings at compile time.
#[proc_macro]
pub fn tw(input: TokenStream) -> TokenStream {
    let literal = match extract_single_string(&input) {
        Ok(s) => s,
        Err(msg) => return compile_error(&input, &msg),
    };

    // The full grammar check runs the same parser as the runtime path —
    // the macro crate depends on aeolus-core (zero-dep) so there is no
    // duplicated grammar (DRY).
    let (tokens, errors) = aeolus_core::parser::parse_class_string(&literal);
    if errors.is_empty() {
        let n = tokens.len();
        if cfg!(feature = "verbose-diagnostics") && n == 0 {
            return compile_error(&input, "tw!: empty class string");
        }
        // Emit the validated literal back. Later phases (backend
        // pre-registration via linkme) replace this with interned IDs.
        let out = format!("\"{}\"", literal.escape_debug());
        out.parse().unwrap()
    } else {
        // The macro keeps the HARD contract: a static class list must be
        // fully understood at compile time (dynamic strings take the
        // runtime cache path and only warn).
        let msg = errors
            .iter()
            .map(|(_, m)| format!("tw!: {m}"))
            .collect::<Vec<_>>()
            .join("; ");
        compile_error(&input, &msg)
    }
}

/// Validate + merge in one step (compile-time `cn` for static class lists).
#[proc_macro]
pub fn tw_cn(input: TokenStream) -> TokenStream {
    let strings = match extract_string_list(input.clone()) {
        Ok(s) => s,
        Err(msg) => return compile_error(&input, &msg),
    };
    let refs: Vec<&str> = strings.iter().map(String::as_str).collect();
    let merged = aeolus_core::cn(&refs);
    let out = format!("\"{}\"", merged.escape_debug());
    out.parse().unwrap()
}

fn extract_single_string(input: &TokenStream) -> Result<String, String> {
    let mut strings = extract_string_list(input.clone())?;
    if strings.len() != 1 {
        return Err("tw!: expected exactly one string literal".into());
    }
    Ok(strings.remove(0))
}

/// Extract string literals from a token stream, tolerating commas and
/// `concat!`-style concatenation of adjacent literals ("a" "b").
fn extract_string_list(input: TokenStream) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    let mut pending: Option<String> = None;
    for token in input {
        match token {
            TokenTree::Literal(lit) => {
                let s = lit.to_string();
                if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
                    let unquoted = unescape(&s[1..s.len() - 1]);
                    match pending.as_mut() {
                        Some(prev) => prev.push_str(&unquoted), // adjacent literal concat
                        None => pending = Some(unquoted),
                    }
                } else {
                    return Err(format!("tw!: unsupported literal {s} (expected a string)"));
                }
            }
            TokenTree::Punct(p) if p.as_char() == ',' => {
                if let Some(s) = pending.take() {
                    out.push(s);
                }
            }
            TokenTree::Group(g) if g.delimiter() == Delimiter::None => {
                // transparent groups: recurse
                let inner = extract_string_list(g.stream())?;
                out.extend(inner);
            }
            other => {
                return Err(format!("tw!: unexpected token {other} (expected a string literal)"));
            }
        }
    }
    if let Some(s) = pending.take() {
        out.push(s);
    }
    Ok(out)
}

/// Minimal unescape for the literal forms proc_macro hands us.
fn unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('r') => out.push('\r'),
            Some('\\') => out.push('\\'),
            Some('"') => out.push('"'),
            Some('\'') => out.push('\''),
            Some('0') => out.push('\0'),
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

fn compile_error(input: &TokenStream, msg: &str) -> TokenStream {
    // Preserve the span by attaching to the first token when possible.
    let _ = input;
    format!("compile_error!({:?})", msg).parse().unwrap()
}
