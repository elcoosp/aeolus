//! Consumer demonstration: the base-lane subcommands' library face —
//! `resolve`, `diff` and `css` driven in-process the way a build
//! script or an integration test would (the `run` entry point the
//! binary calls, arguments without argv-0, two sinks and the exit
//! code back). `resolve` prints the v0.x plan artifact's canonical
//! snapshot JSON (byte-identical for the same slots — class order
//! never changes the plan), `diff` lanes the base-layer changes
//! between two strings (the relayout lane the backends re-flow for),
//! `css` renders the v4-faithful verification stylesheet.
//!
//! ```sh
//! cargo run -p aeolus-cli --example resolve
//! ```

use aeolus_cli::run;
use aeolus_core::StylePlan;

fn drive(args: &[&str]) -> String {
    let owned: Vec<String> = args.iter().map(std::string::ToString::to_string).collect();
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = run(owned, &mut out, &mut err);
    print!("{}", String::from_utf8_lossy(&err));
    assert_eq!(code, 0, "{args:?} exits 0");
    String::from_utf8(out).expect("the sinks are UTF-8")
}

fn main() {
    // 1. `resolve`: the plan artifact's canonical snapshot JSON — the
    //    same bytes `aeolus plan`/`resolve` print, parseable back into
    //    the `StylePlan` the five backend crates consume.
    println!("── resolve p-4 flex-col (the snapshot parses back) ──");
    let snapshot = drive(&["resolve", "p-4 flex-col"]);
    let plan = StylePlan::from_snapshot_json(snapshot.trim()).expect("the snapshot parses");
    println!(
        "base padding: {:?}, layout: {:?}",
        plan.base.padding, plan.base.layout
    );

    // 2. The artifact is canonical: the same slots resolve to the SAME
    //    bytes in any class order (the diff-friendly contract).
    let reordered = drive(&["resolve", "flex-col p-4"]);
    println!("\n── resolve flex-col p-4 (canonical: identical bytes) ──");
    println!("byte-identical: {}", snapshot == reordered);

    // 3. `diff`: the two plans' base-layer changes, laned — a padding
    //    change is the relayout lane (the backends re-flow), a colour
    //    change the cosmetic one.
    println!("\n── diff \"p-4 bg-primary\" \"p-6 bg-primary/90\" ──");
    print!(
        "{}",
        drive(&["diff", "p-4 bg-primary", "p-6 bg-primary/90"])
    );

    // 4. `css`: the v4-faithful verification stylesheet — what the
    //    compiler claims, spelled the way the browser would.
    println!("\n── css \"p-4 hover:p-8\" ──");
    print!("{}", drive(&["css", "p-4 hover:p-8"]));

    println!(
        "\nthe base lane is the v0.x contract — resolve → plan, diff → lanes, css → the verification sheet."
    );
}
