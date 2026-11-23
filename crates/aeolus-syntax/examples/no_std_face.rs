//! Consumer demonstration: the `no_std + alloc` face (plan §3.2's
//! crate-table note). The lexer crate builds `#![no_std]` — the same
//! API a `no_std` host (a build script, an embedded lexer) links:
//! borrowed candidates with byte spans, the typed `LexError` misses,
//! and the ONE heap touch the face carries, `Candidate::variant_prefix`
//! (the merge key the `cn()` engine keys on — the `String` that comes
//! from `extern crate alloc`, not `std`).
//!
//! ```sh
//! cargo run -p aeolus-syntax --example no_std_face
//! ```
//!
//! The differential gate proves the face stays honest:
//! `cargo check -p aeolus-syntax --target thumbv6m-none-eabi` (CI's
//! `no-std-syntax` job). This host example drives the identical public
//! surface — nothing a `no_std` host gets is missing here.

use aeolus_syntax::{parse_candidate, parse_class_string};

fn main() {
    println!("the lexer crate is #![no_std] + `extern crate alloc`:");
    println!("borrowed candidates, typed errors, one alloc-bearing read.\n");

    // The no_std host's usual input: one class string, borrowed end to
    // end (no `std` re-derivation — the spans index the input bytes).
    let classes = "dark:group-hover/menu:bg-primary/90 w-[calc(100%/3)] p--4";
    let (candidates, degraded) = parse_class_string(classes);

    println!("candidates ({}):", candidates.len());
    for c in &candidates {
        // The alloc-bearing read — the merge key `String`. In a no_std
        // host this is the `alloc::string::String` the face carries.
        println!(
            "  {:?} span {:?} variants {:?} merge key {:?}",
            c.raw,
            c.span,
            c.variants,
            c.variant_prefix(),
        );
    }
    println!("\ndegraded tokens ({}):", degraded.len());
    for f in &degraded {
        println!("  {:?} → {} (bytes {:?})", f.raw, f.message(), f.span);
    }

    // The typed miss rides `core::error::Error` — the trait impl the
    // no_std face keeps (a no_std host's `Box<dyn core::error::Error>`
    // carries it unchanged).
    let boxed: Box<dyn core::error::Error> = match parse_candidate("w-[]") {
        Ok(_) => unreachable!("`w-[]` refuses"),
        Err(e) => Box::new(e),
    };
    println!("\n`core::error::Error` object-safe miss: {boxed}");
}
