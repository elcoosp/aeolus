//! Consumer demonstration: the V1 zero-copy lexer — what a class string
//! looks like BEFORE any classification: borrowed candidates with byte
//! spans (the diagnostics surface for editor squiggles and LSP hover
//! hangs off those spans), plus the typed `LexError` miss.
//!
//! ```sh
//! cargo run -p aeolus-syntax --example lex_candidates
//! ```
//!
//! The lexer segments and spans; it does NOT resolve values or lower to
//! the IR — the registry (`aeolus-data`) interprets the body, and the
//! P2 compile stage lowers it. See `registry_lookup` on the data crate
//! for the next pipeline step.

use aeolus_syntax::{Candidate, Form, parse_candidate, parse_class_string};

fn main() {
    // A realistic shadcn-Button-ish class string: stacking variants, an
    // arbitrary value with a modifier, negation, the v4.1 `(...)` form.
    let classes = "inline-flex rounded-md p-4 hover:bg-primary/90 dark:-mt-2 w-[32px] bg-(--brand)";
    println!("class string : {classes}\n");
    let (candidates, degraded) = parse_class_string(classes);

    println!("candidates ({}):", candidates.len());
    for c in &candidates {
        print_candidate(c);
    }
    // Honest degradation: class STRINGS carry per-class failures; the
    // rest of the string still lexes.
    println!("\ndegraded tokens: {}", degraded.len());

    // The typed miss: `parse_candidate` on one token explains WHAT and
    // WHERE — a byte span inside the token, a stable message.
    for bad in ["p--4", "w-[]", "hover::p-4", "w-[4px]x"] {
        match parse_candidate(bad) {
            Ok(_) => println!("{bad:?}: unexpectedly lexed"),
            Err(e) => println!("{bad:?} → {} (bytes {}..{})", e.message(), e.start, e.end),
        }
    }
}

/// The audit view of one candidate: span, variant stack, body, the
/// arbitrary group (hint + value + form) and the raw modifier.
fn print_candidate(c: &Candidate<'_>) {
    let span = format!("{}..{}", c.span.0, c.span.1);
    let variants = if c.variants.is_empty() {
        String::from("-")
    } else {
        c.variant_prefix()
    };
    let flags = [c.negative.then_some("neg"), c.important.then_some("!")]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(",");
    let flags = if flags.is_empty() {
        String::from("-")
    } else {
        flags
    };
    print!(
        "  {span:>9}  variants [{variants}] body {:?} flags {flags}",
        c.body
    );
    match c.arbitrary {
        Some(arb) => {
            let form = match arb.form {
                Form::Bracket => "[]",
                Form::Paren => "()",
            };
            match arb.hint {
                Some(hint) => println!(" arb{form} hint {hint:?} value {:?}", arb.value),
                None => println!(" arb{form} value {:?}", arb.value),
            }
        }
        None => match c.modifier {
            Some(m) => println!(" /{m}"),
            None => println!(),
        },
    }
    // A candidate can carry BOTH an arbitrary group and a modifier —
    // print the modifier too when both are present.
    if c.arbitrary.is_some()
        && let Some(m) = c.modifier
    {
        println!("            modifier /{m}");
    }
}
