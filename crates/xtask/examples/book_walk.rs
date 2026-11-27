//! The docs book's consumer example (plan §8's P5 row, the "docs book"
//! deliverable): run the book's drift gate against the workspace root
//! and print the report — the same walk `cargo xtask verify-book`
//! drives, from library code.
//!
//! ```sh
//! cargo run -p xtask --example book_walk
//! ```

use std::path::PathBuf;

fn main() {
    // The example lives at crates/xtask/examples, so the workspace root
    // is two directories up (the driver's own default-root arithmetic).
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..");

    let mut out = Vec::new();
    let exit = xtask::book::verify(&root, &mut out);
    print!("{}", String::from_utf8_lossy(&out));

    // The gate's verdict: exit 0 clean, 1 drift or structural break —
    // the same contract `verify-tables` keeps.
    std::process::exit(exit);
}
