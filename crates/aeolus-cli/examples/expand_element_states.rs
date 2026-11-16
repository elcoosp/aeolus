//! Consumer demonstration: the element-state family's CLI face (the
//! `expand --target`/`--indeterminate`/`--autofill`/`--read-write`
//! facts). The flags set the interp [`Env`]'s element-state bits — the
//! fragment target, the checkbox/radio answer, the filled-form answer
//! and the editability fact, exactly the tier `--dark` and the
//! capability flags assert — and the walk answers the element-state
//! layers from them. Unset is the quiet miss a selector that does not
//! match is; `read-only:` is the complement of `--read-write` (CSS's
//! own reading for a non-editable element IS :read-only, the `valid:`
//! precedent), so the resting env answers it.
//!
//! ```sh
//! cargo run -p aeolus-cli --example expand_element_states
//! ```
//!
//! This is the headless form of:
//!
//! ```sh
//! aeolus expand --target 'p-4 target:p-8 read-only:ring-2 autofill:hidden'
//! ```

use aeolus_cli::run;

fn main() {
    // The fact-bearing form: --target (the fragment-target fact). The
    // target layers fire from the flag, the cascade hands the longhands
    // over (exit 0).
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = run(
        [
            "expand".to_string(),
            "--target".to_string(),
            "p-4 target:p-8 read-only:ring-2 autofill:hidden".to_string(),
        ],
        &mut out,
        &mut err,
    );
    println!("$ aeolus expand --target … (exit {code})");
    print!("{}", String::from_utf8(out).unwrap_or_default());

    // The resting form: no element-state flag. The target and autofill
    // layers are the quiet miss (nothing reported), while `read-only:`
    // fires — the complement of the unset editability fact.
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = run(
        [
            "expand".to_string(),
            "p-4 target:p-8 read-only:ring-2 autofill:hidden".to_string(),
        ],
        &mut out,
        &mut err,
    );
    println!("\n$ aeolus expand … (no fact, exit {code})");
    print!("{}", String::from_utf8(out).unwrap_or_default());
    print!("{}", String::from_utf8(err).unwrap_or_default());

    // The complement form: --read-write asserts the editability fact —
    // the read-only layer misses, the indeterminate layer still fires
    // from its own flag (the facts assert independently).
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = run(
        [
            "expand".to_string(),
            "--read-write".to_string(),
            "--indeterminate".to_string(),
            "read-only:ring-2 indeterminate:mt-2".to_string(),
        ],
        &mut out,
        &mut err,
    );
    println!("\n$ aeolus expand --read-write --indeterminate … (exit {code})");
    print!("{}", String::from_utf8(out).unwrap_or_default());
    print!("{}", String::from_utf8(err).unwrap_or_default());
}
