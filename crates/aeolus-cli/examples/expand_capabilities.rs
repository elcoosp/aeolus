//! Consumer demonstration: the capability families' CLI face (the
//! `expand --pointer-fine`/`--portrait` facts). The flags set the
//! interp [`Env`]'s frame-state bits — the user agent's own
//! media-query answers, exactly the tier `--dark` and the preference
//! flags assert — and the walk answers the `pointer-*`/
//! `any-pointer-*`/`portrait`/`landscape` layers from them. Unset
//! flags are the quiet miss a media query that does not hold is;
//! `landscape:` is the complement of `--portrait`.
//!
//! ```sh
//! cargo run -p aeolus-cli --example expand_capabilities
//! ```
//!
//! This is the headless form of:
//!
//! ```sh
//! aeolus expand --pointer-fine --portrait \
//!   'p-4 pointer-fine:p-8 pointer-coarse:mt-2 landscape:flex'
//! ```

use aeolus_cli::run;

fn main() {
    // The fact-bearing form: --pointer-fine --portrait. The pointer
    // and orientation layers fire from the flags, the cascade hands
    // the longhands over (exit 0).
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = run(
        [
            "expand".to_string(),
            "--pointer-fine".to_string(),
            "--portrait".to_string(),
            "p-4 pointer-fine:p-8 pointer-coarse:mt-2 landscape:flex".to_string(),
        ],
        &mut out,
        &mut err,
    );
    println!("$ aeolus expand --pointer-fine --portrait … (exit {code})");
    print!("{}", String::from_utf8(out).unwrap_or_default());

    // The resting form: no capability flags. The pointer layers are
    // the quiet miss (nothing reported), while `landscape:` fires —
    // the complement of the unset `--portrait` bit.
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = run(
        [
            "expand".to_string(),
            "p-4 pointer-fine:p-8 pointer-coarse:mt-2 landscape:flex".to_string(),
        ],
        &mut out,
        &mut err,
    );
    println!("\n$ aeolus expand … (no facts, exit {code})");
    print!("{}", String::from_utf8(out).unwrap_or_default());
    print!("{}", String::from_utf8(err).unwrap_or_default());
}
