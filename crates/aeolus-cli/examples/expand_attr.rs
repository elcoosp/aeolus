//! Consumer demonstration: the attribute-probe families' CLI face (the
//! `expand --data`/`--aria` facts). The flags build the attribute-bit
//! table the interp's `AttrProbe` seam answers the element-own
//! `data-*`/`aria-*` layers from — a bare fact is the presence
//! spelling, `name=value` the value spelling (the bare aria fact
//! asserts the fold's own "true" reading). A hit FIRES the layer; a
//! source-answered miss is the CSS-quiet skip; with no facts the
//! sourceless boundary is the reported `AttributeUnqueried` note.
//!
//! ```sh
//! cargo run -p aeolus-cli --example expand_attr
//! ```
//!
//! This is the headless form of:
//!
//! ```sh
//! aeolus expand --data state=open --aria busy \
//!   'p-4 data-state:p-8 data-[state=open]:mt-2 aria-busy:opacity-50'
//! ```

use aeolus_cli::run;

fn main() {
    // The fact-bearing form: --data state=open --aria busy. The
    // attribute layers fire from the fact table, the cascade hands the
    // longhands over (exit 0).
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = run(
        [
            "expand".to_string(),
            "--data".to_string(),
            "state=open".to_string(),
            "--aria".to_string(),
            "busy".to_string(),
            "p-4 data-state:p-8 data-[state=open]:mt-2 aria-busy:opacity-50".to_string(),
        ],
        &mut out,
        &mut err,
    );
    println!("$ aeolus expand --data state=open --aria busy … (exit {code})");
    print!("{}", String::from_utf8(out).unwrap_or_default());

    // The resting form: no attribute facts. The layers are the
    // reported boundary — the `AttributeUnqueried` note rides beside
    // the base declaration (never a silent drop).
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = run(
        [
            "expand".to_string(),
            "p-4 data-state:p-8 data-[state=open]:mt-2 aria-busy:opacity-50".to_string(),
        ],
        &mut out,
        &mut err,
    );
    println!("\n$ aeolus expand … (no facts, exit {code})");
    print!("{}", String::from_utf8(out).unwrap_or_default());

    // The answered miss: the fact table answers "no" (another value) —
    // a CSS selector that does not match, quiet, no note.
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = run(
        [
            "expand".to_string(),
            "--data".to_string(),
            "state=other".to_string(),
            "p-4 data-[state=open]:mt-2".to_string(),
        ],
        &mut out,
        &mut err,
    );
    println!("\n$ aeolus expand --data state=other … (answered miss, exit {code})");
    print!("{}", String::from_utf8(out).unwrap_or_default());
    print!("{}", String::from_utf8(err).unwrap_or_default());
}
