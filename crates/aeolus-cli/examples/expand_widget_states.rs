//! Consumer demonstration: the widget-state family's CLI face (the
//! `expand --open`/`--inert`/`--default` facts). The flags set the
//! interp [`Env`]'s widget-state bits — the disclosure/popover answer,
//! the frozen-subtree answer the compound `:is([inert], [inert] *)`
//! head reads and the form group's default-control answer, exactly the
//! tier `--target` and the capability flags assert — and the walk
//! answers the widget-state layers from them. Every family question is
//! AFFIRMATIVE (the `--target` precedent — no complement rides), so
//! unset is the quiet miss a selector that does not match is.
//!
//! ```sh
//! cargo run -p aeolus-cli --example expand_widget_states
//! ```
//!
//! This is the headless form of:
//!
//! ```sh
//! aeolus expand --open --inert --default 'p-4 open:p-8 inert:mt-2 default:ring-2'
//! ```

use aeolus_cli::run;

fn main() {
    // The fact-bearing form: --open --inert --default (one flag per
    // FACT). The widget-state layers fire from the flags, the cascade
    // hands the longhands over (exit 0).
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = run(
        [
            "expand".to_string(),
            "--open".to_string(),
            "--inert".to_string(),
            "--default".to_string(),
            "p-4 open:p-8 inert:mt-2 default:hidden".to_string(),
        ],
        &mut out,
        &mut err,
    );
    println!("$ aeolus expand --open --inert --default … (exit {code})");
    print!("{}", String::from_utf8(out).unwrap_or_default());

    // The resting form: no widget-state flag. Every family question is
    // AFFIRMATIVE, so all three layers are the quiet miss (nothing
    // reported) while the base p-4 answers.
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = run(
        [
            "expand".to_string(),
            "p-4 open:p-8 inert:mt-2 default:hidden".to_string(),
        ],
        &mut out,
        &mut err,
    );
    println!("\n$ aeolus expand … (no fact, exit {code})");
    print!("{}", String::from_utf8(out).unwrap_or_default());
    print!("{}", String::from_utf8(err).unwrap_or_default());

    // The mix form: --default asserts the form-default fact beside the
    // sibling --inert (the facts assert independently — one flag per
    // question, the element-state battery's own shape).
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = run(
        [
            "expand".to_string(),
            "--default".to_string(),
            "--inert".to_string(),
            "default:mt-2 inert:ring-2 open:hidden".to_string(),
        ],
        &mut out,
        &mut err,
    );
    println!("\n$ aeolus expand --default --inert … (exit {code})");
    print!("{}", String::from_utf8(out).unwrap_or_default());
    print!("{}", String::from_utf8(err).unwrap_or_default());
}
