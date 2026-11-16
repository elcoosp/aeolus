//! The shadcn PROGRAM TABLES' non-button halves (architecture plan
//! §3.6's surface (2)): `BadgePrograms`, `AlertPrograms` and
//! `BasePrograms` — the verbatim v4 class strings compiled through the
//! V1 pipeline once per closed-world combo and interned, the same
//! const-table seam `program_tables`' Button demo walks.
//!
//! This demo resolves each table over one session's token table, walks
//! the effective declarations of one variant per family (the badge's
//!
//! (the badge's destructive, the alert's destructive) and the base shapes' shared pieces
//! (the input, the card), so the consumer face of every table is on
//! record.
//!
//! ```sh
//! cargo run -p aeolus-shadcn --example program_components
//! ```

use aeolus_compile::{Env, interp_theme};
use aeolus_rt::{ThemeSource, TokenTable};
use aeolus_shadcn::classes::{AlertVariant, BadgeVariant};
use aeolus_shadcn::program::{AlertPrograms, BadgePrograms, BasePrograms};

/// One family's effective declarations under the session's table.
fn effective(program: &aeolus_rt::StaticProgram, tokens: &TokenTable, name: &str) {
    let interped =
        interp_theme(&program.program(), tokens, &Env::default()).expect("the interp resolves");
    println!(
        "\n{name} — {} layers, {} longhands:",
        program.layers.len(),
        interped.decls.len()
    );
    for decl in &interped.decls {
        println!("  {:?} {:?}", decl.prop, decl.val);
    }
}

fn main() {
    // The session's table: the built-in shadcn theme baked (T0
    // literals) — the same source a `theme!` project would ship.
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    let tokens = TokenTable::baked(&source).expect("the T0/T1 bake succeeds");

    // The badge table: one program per closed-world variant combo
    // (default/secondary/destructive/outline/ghost/link).
    let badges = BadgePrograms::new(&tokens);
    let badge = badges.program(BadgeVariant::Destructive);
    println!("badge table: one interned program per variant (destructive demoed)");
    effective(&badge, &tokens, "badge (destructive)");

    // The alert table: the same seam, the alert's own class strings
    // (default/destructive).
    let alerts = AlertPrograms::new(&tokens);
    let alert = alerts.program(AlertVariant::Destructive);
    println!("\nalert table: one interned program per variant (destructive demoed)");
    effective(&alert, &tokens, "alert (destructive)");

    // The base shapes: the shared pieces every composition builds from.
    let base = BasePrograms::new(&tokens);
    let input = base.input();
    let card = base.card();
    println!("\nbase table: input/card/separator/skeleton … interned");
    effective(&input, &tokens, "base (input)");
    effective(&card, &tokens, "base (card)");
}
