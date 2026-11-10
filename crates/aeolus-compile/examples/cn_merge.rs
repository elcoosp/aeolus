//! Consumer demonstration: the IR-level `cn()` merge — the plan §3.2
//! "hosts the `cn()` engine at IR level" slice. Two separately compiled
//! programs merge rightmost-wins per longhand: the overlap the merge
//! removes is exactly what `Program::base_conflicts()` names on the raw
//! cascade, and the resolved result is what compiling the concatenated
//! string would resolve to.
//!
//! ```sh
//! cargo run -p aeolus-compile --example cn_merge
//! ```
//!
//! The STRING-level `cn()` (the audited `tw_merge` table, rightmost
//! class wins per conflict GROUP) is the v0.x surface —
//! `aeolus_core::cn` and the CLI's `cn` command. This one is the
//! COMPILED merge: the longhand fan-outs are the groups, so the merge
//! is mask algebra + copy (plan §3.3), idempotent and associative.

use aeolus_compile::{cn, compile, compile_theme, Config};
use aeolus_ir::Val;
use aeolus_theme::{ThemeSource, TokenTable};

fn main() {
    // 1. The component's table and the user's override, compiled
    //    separately — the `cn(base, user)` shape the merge exists for.
    let base = compile("p-4 rounded-md bg-primary text-primary-foreground", &Config::default())
        .expect("non-empty input compiles");
    let user = compile("px-8 rounded-full bg-red-500 hover:bg-red-600", &Config::default())
        .expect("non-empty input compiles");

    println!("── base: p-4 rounded-md bg-primary text-primary-foreground ──");
    for layer in base.program.layers {
        println!("  {:?}: {} decls, mask {:?}", layer.cond, layer.decls.len(), layer.mask);
    }

    // 2. The raw cascade keeps every fan-out (the v0.11.0 compile
    //    contract, byte-identical): compiling the concatenation
    //    surfaces the overlap as base_conflicts() bits.
    let concat = compile(
        "p-4 rounded-md bg-primary text-primary-foreground px-8 rounded-full bg-red-500 hover:bg-red-600",
        &Config::default(),
    )
    .expect("non-empty input compiles");
    println!("\n── the raw concatenation ──");
    println!("  base decls: {}", concat.program.layers[0].decls.len());
    println!("  base conflicts (the bits a later same-condition group re-sets): {:?}",
        concat.program.base_conflicts());

    // 3. The merge: same-condition layers fold, declarations cascade in
    //    Prop order, the LAST occurrence per longhand wins.
    let merged = cn(&[&base, &user]).expect("two compiled programs merge");
    println!("\n── cn(base, user) ──");
    for layer in merged.program.layers {
        println!("  {:?}: {} decls", layer.cond, layer.decls.len());
        for decl in layer.decls {
            println!("    {} = {decl:?}", decl.prop.name());
        }
    }
    println!(
        "  base conflicts after the merge: {:?} (the overlap is merged away)",
        merged.program.base_conflicts()
    );

    // 4. The differential contract: cn(a, b) resolves like compiling
    //    the concatenation — same slots, fewer duplicate decls.
    println!("\n── the differential contract ──");
    println!("  merged decls (all layers): {}", merged.program.layers.iter().map(|l| l.decls.len()).sum::<usize>());
    println!("  concat decls (all layers): {}", concat.program.layers.iter().map(|l| l.decls.len()).sum::<usize>());
    println!("  (equal resolved slots; the duplicates the merge drops are the conflict bits)");

    // 5. Diagnostics ride the merge honestly: left's reports first with
    //    their own input's spans, then right's — never dropped, never
    //    offset.
    let broken = compile("not-a-utility p-4", &Config::default()).expect("degrades, not fails");
    let merged_with_reports = cn(&[&broken, &user]).expect("merge");
    println!("\n── diagnostics ride the merge ──");
    for d in &merged_with_reports.diagnostics {
        println!("  {d}");
        println!("    code: {}", d.kind.code());
    }

    // 6. Theme tiers merge per longhand too: the rightmost program's
    //    alpha rides the token slot (T2's one-multiply contract).
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    let tokens = TokenTable::runtime(&source).expect("default-seeded source builds");
    let themed_base = compile_theme("bg-primary", &tokens, &Config::default())
        .expect("non-empty input compiles");
    let themed_user = compile_theme("bg-primary/40", &tokens, &Config::default())
        .expect("non-empty input compiles");
    let themed = cn(&[&themed_base, &themed_user]).expect("merge");
    if let Some(decl) = themed.program.layers.first().and_then(|l| l.decls.first()) {
        match decl.val {
            Val::Tok { id, alpha } => println!(
                "\n── theme tier merge ──\n  bg-primary ⊕ bg-primary/40 → Tok {{ id: {id}, alpha: {alpha} }} ({})",
                tokens.name(id).unwrap_or("?"),
            ),
            ref val => println!("\n── theme tier merge ──\n  {val:?}"),
        }
    }
}
