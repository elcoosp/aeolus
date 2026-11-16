//! Consumer demonstration: the validation family's CLI face (the
//! `expand --invalid`/`--user-valid`/`--user-invalid` facts). The flags
//! set the interp [`Env`]'s element-state bits — the constraint state
//! and the interaction gate, exactly the tier `--dark` and the
//! capability flags assert — and the walk answers the validation
//! layers from them. Unset is the quiet miss a selector that does not
//! match is; `valid:` is the complement of `--invalid` (CSS's own
//! pristine-field reading), and BOTH `user-` spellings stay quiet
//! until a `--user-*` flag asserts the interaction gate the HTML spec
//! requires.
//!
//! ```sh
//! cargo run -p aeolus-cli --example expand_validation
//! ```
//!
//! This is the headless form of:
//!
//! ```sh
//! aeolus expand --user-invalid 'p-4 invalid:p-8 valid:mt-2 user-invalid:hidden'
//! ```

use aeolus_cli::run;

fn main() {
    // The fact-bearing form: --user-invalid (the gate with the failing
    // fact). The invalid and user-invalid layers fire from the flags,
    // the cascade hands the longhands over (exit 0).
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = run(
        [
            "expand".to_string(),
            "--user-invalid".to_string(),
            "p-4 invalid:p-8 valid:mt-2 user-invalid:hidden".to_string(),
        ],
        &mut out,
        &mut err,
    );
    println!("$ aeolus expand --user-invalid … (exit {code})");
    print!("{}", String::from_utf8(out).unwrap_or_default());

    // The resting form: no validation flag. The invalid and user-
    // layers are the quiet miss (nothing reported), while `valid:`
    // fires — the complement of the unset `--invalid` bit.
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = run(
        [
            "expand".to_string(),
            "p-4 invalid:p-8 valid:mt-2 user-invalid:hidden".to_string(),
        ],
        &mut out,
        &mut err,
    );
    println!("\n$ aeolus expand … (no fact, exit {code})");
    print!("{}", String::from_utf8(out).unwrap_or_default());
    print!("{}", String::from_utf8(err).unwrap_or_default());

    // The gate-only form: --user-valid asserts the gate with the
    // holding fact — the user-valid layer fires, the user-invalid
    // layer misses (the gate is not the failing fact).
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = run(
        [
            "expand".to_string(),
            "--user-valid".to_string(),
            "user-valid:p-8 user-invalid:hidden".to_string(),
        ],
        &mut out,
        &mut err,
    );
    println!("\n$ aeolus expand --user-valid … (exit {code})");
    print!("{}", String::from_utf8(out).unwrap_or_default());
    print!("{}", String::from_utf8(err).unwrap_or_default());
}
