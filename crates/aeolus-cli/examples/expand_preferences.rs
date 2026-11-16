//! Consumer demonstration: the preference families' CLI face (the
//! `expand --reduced-motion`/`--more-contrast`/`--less-contrast`/
//! `--forced-colors`/`--print` facts). The flags set the interp
//! [`Env`]'s media-feature bits — the frame-state facts the
//! `motion-*:`/`contrast-*:`/`forced-colors:`/`print:` layers answer
//! from (typed `Cond::Media` bits, no probe seam — the preference
//! tier's own reading), exactly the tier the capability flags assert.
//! One flag per FACT (the `--target` precedent), so unset is the quiet
//! miss a media query that does not hold is.
//!
//! ```sh
//! cargo run -p aeolus-cli --example expand_preferences
//! ```
//!
//! This is the headless form of:
//!
//! ```sh
//! aeolus expand --reduced-motion --more-contrast --forced-colors --print \
//!   'motion-safe:p-4 motion-reduce:p-8 contrast-more:ring-2 print:hidden'
//! ```

use aeolus_cli::run;

fn main() {
    // The fact-bearing form: the four preference facts (one flag per
    // FACT). The preference layers fire from the flags — the
    // reduced-motion fact answers motion-reduce (motion-safe the
    // complement), the contrast fact answers contrast-more, the
    // forced-palette and print facts their own layers (exit 0).
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = run(
        [
            "expand".to_string(),
            "--reduced-motion".to_string(),
            "--more-contrast".to_string(),
            "--forced-colors".to_string(),
            "--print".to_string(),
            "p-4 motion-safe:p-8 motion-reduce:hidden contrast-more:ring-2 print:mt-2".to_string(),
        ],
        &mut out,
        &mut err,
    );
    println!(
        "$ aeolus expand --reduced-motion --more-contrast --forced-colors --print … (exit {code})"
    );
    print!("{}", String::from_utf8(out).unwrap_or_default());

    // The resting form: no preference fact. Every media-feature query
    // rests unset (the quiet miss a media query that does not hold
    // is), while the base p-4 answers.
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = run(
        [
            "expand".to_string(),
            "p-4 motion-safe:p-8 motion-reduce:hidden contrast-more:ring-2 print:mt-2".to_string(),
        ],
        &mut out,
        &mut err,
    );
    println!("\n$ aeolus expand … (no fact, exit {code})");
    print!("{}", String::from_utf8(out).unwrap_or_default());
    print!("{}", String::from_utf8(err).unwrap_or_default());

    // The single-fact form: --reduced-motion alone (the complement
    // pair's own reading — motion-reduce fires, motion-safe rests).
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = run(
        [
            "expand".to_string(),
            "--reduced-motion".to_string(),
            "motion-safe:hidden motion-reduce:p-8".to_string(),
        ],
        &mut out,
        &mut err,
    );
    println!("\n$ aeolus expand --reduced-motion … (exit {code})");
    print!("{}", String::from_utf8(out).unwrap_or_default());
    print!("{}", String::from_utf8(err).unwrap_or_default());
}
