//! Consumer demonstration: the direction family's CLI face (the
//! `expand --rtl` fact). The flag sets the interp [`Env`]'s frame-state
//! bit — the element's resolved text direction, exactly the tier
//! `--dark` and the capability flags assert — and the walk answers the
//! `rtl:` layers from it. Unset is the quiet miss a selector that does
//! not match is; `ltr:` is the complement of `--rtl` (CSS's own
//! initial `direction` value is ltr).
//!
//! ```sh
//! cargo run -p aeolus-cli --example expand_direction
//! ```
//!
//! This is the headless form of:
//!
//! ```sh
//! aeolus expand --rtl 'p-4 rtl:p-8 ltr:mt-2 rtl:hidden'
//! ```

use aeolus_cli::run;

fn main() {
    // The fact-bearing form: --rtl. The rtl layers fire from the flag,
    // the cascade hands the longhands over (exit 0).
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = run(
        [
            "expand".to_string(),
            "--rtl".to_string(),
            "p-4 rtl:p-8 ltr:mt-2 rtl:hidden".to_string(),
        ],
        &mut out,
        &mut err,
    );
    println!("$ aeolus expand --rtl … (exit {code})");
    print!("{}", String::from_utf8(out).unwrap_or_default());

    // The resting form: no direction flag. The rtl layers are the
    // quiet miss (nothing reported), while `ltr:` fires — the
    // complement of the unset `--rtl` bit.
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = run(
        [
            "expand".to_string(),
            "p-4 rtl:p-8 ltr:mt-2 rtl:hidden".to_string(),
        ],
        &mut out,
        &mut err,
    );
    println!("\n$ aeolus expand … (no fact, exit {code})");
    print!("{}", String::from_utf8(out).unwrap_or_default());
    print!("{}", String::from_utf8(err).unwrap_or_default());
}
