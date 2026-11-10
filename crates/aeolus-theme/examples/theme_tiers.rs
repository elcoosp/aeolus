//! Consumer demonstration: the P2 theme pass — a CSS-first theme source
//! becomes a tiered token table, and the same token resolves three ways
//! per the plan §3.4 tiers (T0 literal, T1 const pair, T2 runtime slot).
//!
//! ```sh
//! cargo run -p aeolus-theme --example theme_tiers
//! ```
//!
//! The theme pass is the compile seam's colour half: the TokenTable this
//! example builds is what `compile_theme` lowers `bg-primary/90`-shaped
//! classes through (`theme_seam` on `aeolus-compile` shows that side).
//! The v0.x reader is the interim oracle — the same `@theme`/`:root`/
//! `.dark` CSS a web Tailwind v4 project uses, unchanged.

use aeolus_ir::Val;
use aeolus_theme::{ThemeSource, TokenTable};

fn main() {
    // 1. The CSS-first source: the embedded shadcn light + dark
    //    palettes — verbatim shadcn-ui v4 globals.css content.
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    println!(
        "theme source: shadcn light{}",
        if source.dark().is_some() {
            " + dark"
        } else {
            ""
        }
    );

    // 2. The T2 runtime table: sorted-name ids, the u16 Tok slot — the
    //    budget the colour table's 270 entries forced (the u8 Kw slot
    //    could not hold them; the compile seam's discovered boundary).
    let runtime = TokenTable::runtime(&source).expect("default-seeded source builds");
    println!(
        "\n── runtime table ({} tokens, {} dark pairs) ───────────",
        runtime.len(),
        runtime.pairs()
    );
    for name in ["primary", "background", "ring", "red-500"] {
        let Some(id) = runtime.id(name) else {
            println!("  {name:<12} — not a token");
            continue;
        };
        println!(
            "  {name:<12} id {id:<4} {:?}",
            runtime.resolve(id, None).expect("resolves")
        );
    }

    // 3. The T0/T1 baked table: literals for light-only tokens, const
    //    pairs where the dark variant overrides.
    let baked = TokenTable::baked(&source).expect("default-seeded source builds");
    println!(
        "\n── baked table (tier {:?}) ─────────────────────────────",
        baked.tier()
    );
    for name in ["primary", "background"] {
        let Some(id) = baked.id(name) else { continue };
        println!(
            "  {name:<12} {:?}",
            baked.resolve(id, None).expect("resolves")
        );
    }

    // 4. The /opacity identity, on a light-only token (the v4 palette
    //    tokens carry no .dark override): the premultiplied bake is one
    //    multiply, pinned against the audited oracle
    //    (Color::mix_with_transparent).
    let id = baked
        .id("red-500")
        .expect("red-500 is a light-only palette token");
    match (baked.resolve(id, None), baked.resolve(id, Some(0.5))) {
        (Ok(Val::Rgba(plain)), Ok(Val::Rgba(half))) => {
            println!("\n── the /N identity (baked tier) ────────────────────────");
            println!("  red-500     {:?}  alpha byte {}", plain, plain[3]);
            println!("  red-500/50  {:?}  alpha byte {}", half, half[3]);
        }
        (plain, half) => println!("  unexpected resolves: {plain:?} / {half:?}"),
    }

    // 5. Tier shapes side by side, same token, same id: a dark-overridden
    //    token is a T1 pair when baked (and REFUSES an alpha — the typed
    //    boundary), and a Tok slot when runtime (the alpha rides).
    let id = baked.id("primary").expect("primary exists");
    println!("\n── one token, the tier forms ───────────────────────────");
    println!(
        "  T1 baked  : {:?}",
        baked.resolve(id, None).expect("resolves")
    );
    println!(
        "  T1 + /90  : {:?} (the typed boundary)",
        baked
            .resolve(id, Some(0.9))
            .expect_err("the pair form refuses alpha")
    );
    let id = runtime.id("primary").expect("primary exists");
    println!(
        "  T2 runtime: {:?}",
        runtime.resolve(id, Some(0.9)).expect("resolves")
    );
}
