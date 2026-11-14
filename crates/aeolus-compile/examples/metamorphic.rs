//! Consumer demonstration: the metamorphic seam (architecture plan
//! §5.3's differential line — the `compile ⇒ interpret ⇒ equals emit`
//! target). One walk runs the three pipeline answers over a class
//! string — the compile, the CSS emit and the interp — and reports,
//! per probe situation, whether the interpreter's effective
//! declarations agree with what the CSS arm renders, every documented
//! divergence a typed note.
//!
//! ```sh
//! cargo run -p aeolus-compile --example metamorphic
//! ```
//!
//! The default walk covers the shadcn Button's core (base, hover and
//! the dark tier, baked and runtime tiers) plus the honest boundary
//! shapes (a stacked variant's fold note, a pseudo-element's
//! render-target note, an unknown root's diagnostic equality). Pass
//! class strings as arguments to check your own.

use aeolus_compile::{Config, MetamorphicNote, metamorphic_check, metamorphic_check_theme};
use aeolus_theme::{ThemeSource, TokenTable};

/// The demo battery: the closed-world surface the other examples key
/// on, the stacked-variant fold, the pseudo render target and an
/// unknown root (the diagnostics-equality half).
const DEMO: [&str; 6] = [
    "inline-flex items-center justify-center gap-2 rounded-md px-4 py-2 \
     text-sm font-medium bg-primary text-primary-foreground hover:bg-primary/90",
    "p-4 hover:px-8 md:p-6",
    "hover:md:p-8",
    "p-4 selection:p-4",
    "w-1/2 -mt-2 z-10 duration-500",
    "p-4 not-a-root",
];

fn main() {
    let config = Config::default();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let strings: Vec<String> = if args.is_empty() {
        DEMO.iter().map(|s| (*s).to_string()).collect()
    } else {
        args
    };

    // The baked tier (T0/T1): colour tokens bake to literals and const
    // pairs — the interp's bytes and the CSS texts compare directly.
    let source = ThemeSource::shadcn_dark().expect("embedded palettes parse");
    let baked = TokenTable::baked(&source).expect("default-seeded source builds");

    // The runtime tier (T2): the CSS keeps the var() references, the
    // interp resolves the probe's bytes — the typed TokenDynamic notes.
    let runtime = TokenTable::runtime(&source).expect("default-seeded source builds");

    let mut diverged = 0usize;
    for input in &strings {
        // The theme-less walk: the token/modifier boundaries ride BOTH
        // diagnostics channels identically (the equality contract).
        let report = match metamorphic_check(input, &config) {
            Ok(report) => report,
            Err(err) => {
                println!("{input:?}: {err}");
                continue;
            }
        };
        if !report.agree() {
            diverged += 1;
        }
        println!("{report}");

        // The themed walk: the tiered tables lower the colour tokens;
        // the baked tier compares text-for-text, the runtime tier rides
        // its notes.
        let themed = metamorphic_check_theme(input, &baked, &config).expect("non-empty input");
        if !themed.agree() {
            diverged += 1;
        }
        println!("{themed}");

        let runtime_report =
            metamorphic_check_theme(input, &runtime, &config).expect("non-empty input");
        if !runtime_report.agree() {
            diverged += 1;
        }
        let dynamic: Vec<&MetamorphicNote> = runtime_report
            .notes
            .iter()
            .filter(|note| matches!(note, MetamorphicNote::TokenDynamic { .. }))
            .collect();
        if !dynamic.is_empty() {
            println!(
                "  runtime tier: {} var() reference(s) the CSS keeps dynamic (the interp answers the probe's bytes)",
                dynamic.len()
            );
        }
        println!();
    }

    if diverged > 0 {
        println!("{diverged} walk(s) diverged — every divergence carries its typed note");
    } else {
        println!("every walk agrees: compile ⇒ interpret ⇒ equals emit");
    }
}
