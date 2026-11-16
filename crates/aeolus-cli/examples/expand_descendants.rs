//! Consumer demonstration: the `expand` command's `--has` face (the
//! descendant-probe seam's CLI form, the named-container
//! `--container` flag's sibling) — the subtree facts the
//! `has-*:`/`group-has-*:`/`peer-has-*:` layers answer from, driven
//! in-process through the SAME entry point the binary runs
//! ([`run`]). `--has kind=fragment` names the `:has()` scope
//! (`element` — the element's own subtree, `group` — the `.group`
//! ancestor's, `peer` — the `.peer` sibling's) and the rendered
//! `:has()` inner fragment the condition carries.

use std::process::ExitCode;

use aeolus_cli::run;

/// One argv row, owned (the run seam takes owned strings).
fn args(rows: &[&str]) -> Vec<String> {
    rows.iter().map(|row| (*row).to_string()).collect()
}

fn main() -> ExitCode {
    println!("── the sourceless walk: the layer reports (no facts ride) ──");
    let code = run(
        args(&["expand", "has-focus:p-8"]),
        &mut std::io::stdout(),
        &mut std::io::stderr(),
    );
    println!();
    println!("── --has element=*:focus: the subtree answers, the layer fires ──");
    let code = code.max(run(
        args(&["expand", "--has", "element=*:focus", "has-focus:p-8"]),
        &mut std::io::stdout(),
        &mut std::io::stderr(),
    ));
    println!();
    println!("── the scope discriminates: the group answer never rides the peer question ──");
    let code = code.max(run(
        args(&[
            "expand",
            "--has",
            "group=*:focus",
            "group-has-focus:pt-2",
            "peer-has-focus:pb-8",
        ]),
        &mut std::io::stdout(),
        &mut std::io::stderr(),
    ));
    ExitCode::from(u8::try_from(code).unwrap_or(1))
}
