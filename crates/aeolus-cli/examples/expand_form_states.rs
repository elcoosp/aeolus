//! Consumer demonstration: the form-state family's CLI face (the
//! `expand --form-associated`/`--required`/`--range-bounded`/
//! `--range-in` facts). The flags set the interp [`Env`]'s form-state
//! bits — the form-membership answer every matches-neither pair gates
//! on, the required-attribute answer the required/optional pair splits
//! on, the bounded-universe membership and the value's position the
//! in-range/out-of-range pair reads — exactly the tier `--target` and
//! the widget-state flags assert. One flag per FACT (the `--target`
//! precedent — no complement flag rides: `optional:` answers within
//! the membership, `out-of-range:` within the bounds), so unset is the
//! quiet miss a selector that does not match is.
//!
//! ```sh
//! cargo run -p aeolus-cli --example expand_form_states
//! ```
//!
//! This is the headless form of:
//!
//! ```sh
//! aeolus expand --form-associated --required --range-bounded --range-in \
//!   'p-4 enabled:p-8 required:mt-2 in-range:ring-2'
//! ```

use aeolus_cli::run;

fn main() {
    // The fact-bearing form: --form-associated --required
    // --range-bounded --range-in (one flag per FACT). The form-state
    // layers fire from the flags, the cascade hands the longhands over
    // (exit 0).
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = run(
        [
            "expand".to_string(),
            "--form-associated".to_string(),
            "--required".to_string(),
            "--range-bounded".to_string(),
            "--range-in".to_string(),
            "p-4 enabled:p-8 required:mt-2 optional:hidden in-range:ring-2".to_string(),
        ],
        &mut out,
        &mut err,
    );
    println!(
        "$ aeolus expand --form-associated --required --range-bounded --range-in … (exit {code})"
    );
    print!("{}", String::from_utf8(out).unwrap_or_default());

    // The resting form: no form-state flag. Every family question
    // gates on the membership, so all five layers are the quiet miss
    // (nothing reported) while the base p-4 answers.
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = run(
        [
            "expand".to_string(),
            "p-4 enabled:p-8 required:mt-2 optional:hidden in-range:ring-2".to_string(),
        ],
        &mut out,
        &mut err,
    );
    println!("\n$ aeolus expand … (no fact, exit {code})");
    print!("{}", String::from_utf8(out).unwrap_or_default());
    print!("{}", String::from_utf8(err).unwrap_or_default());

    // The bounded-complement form: --range-bounded without --range-in
    // (the value sits outside the declared bounds) — `out-of-range:`
    // answers within the bounded universe while `in-range:` stays
    // quiet.
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = run(
        [
            "expand".to_string(),
            "--range-bounded".to_string(),
            "in-range:mt-2 out-of-range:hidden".to_string(),
        ],
        &mut out,
        &mut err,
    );
    println!("\n$ aeolus expand --range-bounded … (exit {code})");
    print!("{}", String::from_utf8(out).unwrap_or_default());
    print!("{}", String::from_utf8(err).unwrap_or_default());
}
