//! Consumer demonstration: the `plan` command's library path (the §7
//! migration seam's consumer face) — the same artifact the
//! `aeolus plan` subcommand prints, driven in-process the way a build
//! script or a backend migration test would: a class string lowers
//! through the V1 pipeline into the v0.x StylePlan the five backend
//! crates consume, rendered as the canonical snapshot JSON. The typed
//! diagnostics and the plan arm's slot-boundary notes ride beside it —
//! never a silent drop.

use aeolus_compile::{Config, emit_plan, emit_plan_theme};
use aeolus_theme::{ThemeSource, TokenTable};

fn main() {
    // 1. The themeless bridge: structural classes lower to their v0.x
    //    slots; colour tokens are the typed boundary (the honest
    //    default the pipeline's compile has).
    let sheet = emit_plan(
        "p-4 flex-col hover:pointer-events-none bg-primary place-content-center",
        &Config::default(),
    )
    .expect("non-empty input compiles");

    println!("── snapshot (themeless) ──");
    println!("{}", sheet.plan.to_snapshot_json());
    println!();

    println!("── diagnostics (themeless) ──");
    for diagnostic in &sheet.diagnostics {
        println!("error: {diagnostic} [{}]", diagnostic.kind.code());
    }
    println!();

    println!("── notes (the typed boundary set) ──");
    for note in &sheet.notes {
        println!("note: {note}");
    }
    println!();

    // 2. The themed bridge (`--theme shadcn`): the same string lowers
    //    further — the baked table's sRGB values convert through the
    //    audited Color parser into the plan's Oklab slots, the same
    //    colours the v0.x resolver answers with.
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    let tokens = TokenTable::baked(&source).expect("default-seeded source builds");
    let themed = emit_plan_theme(
        "bg-primary hover:bg-primary/90",
        &tokens,
        &Config::default(),
    )
    .expect("non-empty input compiles");

    println!("── snapshot (themed, shadcn) ──");
    println!("{}", themed.plan.to_snapshot_json());
    println!();

    // 3. The differential contract: the themed bridge's base background
    //    agrees with the v0.x resolver's own answer — the two lanes,
    //    one plan.
    let v0 = aeolus_cli::load_theme("shadcn").expect("the built-in spec loads");
    let resolved = aeolus_core::Resolver::new(&v0)
        .resolve("bg-primary")
        .expect("the class resolves on the v0.x lane");
    println!("── the two lanes' background slot ──");
    println!("v1 (plan bridge): {:?}", themed.plan.base.background);
    println!("v0 (resolver):    {:?}", resolved.plan.base.background);
}
