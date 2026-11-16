//! Consumer demonstration: the `cn` command's library path (the
//! tw_merge-compatible class merge, headless). The command compiles
//! each argument as its own class string and merges the programs —
//! later classes win their conflict group, a parent evicts its
//! children, non-conflicting groups survive.
//!
//! ```sh
//! cargo run -p aeolus-cli --example cn
//! ```
//!
//! This is the headless form of:
//!
//! ```sh
//! aeolus cn 'px-2 px-4' 'rounded-md'
//! aeolus cn 'px-4' 'p-8'
//! aeolus cn 'bg-red-500/80' 'text-sm font-medium' 'hover:bg-red-600'
//! ```

use aeolus_cli::run;

fn main() {
    // Conflict groups: later classes win (px-2 evicted by px-4),
    // non-conflicting groups survive (rounded-md rides along).
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = run(
        [
            "cn".to_string(),
            "px-2 px-4".to_string(),
            "rounded-md".to_string(),
        ],
        &mut out,
        &mut err,
    );
    println!("$ aeolus cn 'px-2 px-4' 'rounded-md' (exit {code})");
    println!("{}", String::from_utf8_lossy(&out).trim());
    print!("{}", String::from_utf8(err).unwrap_or_default());

    // A parent later in the string evicts its children: p-8 answers
    // alone (the px-4 half it subsumes is gone).
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = run(
        ["cn".to_string(), "px-4".to_string(), "p-8".to_string()],
        &mut out,
        &mut err,
    );
    println!("\n$ aeolus cn 'px-4' 'p-8' (exit {code})");
    println!("{}", String::from_utf8_lossy(&out).trim());
    print!("{}", String::from_utf8(err).unwrap_or_default());

    // The shadcn-shaped consumer call: a variant class beside its
    // modifier and base groups — every group survives, the merge is
    // conflict-aware (the /opacity modifier rides its own group).
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = run(
        [
            "cn".to_string(),
            "bg-red-500/80".to_string(),
            "text-sm font-medium".to_string(),
            "hover:bg-red-600".to_string(),
        ],
        &mut out,
        &mut err,
    );
    println!(
        "\n$ aeolus cn 'bg-red-500/80' 'text-sm font-medium' 'hover:bg-red-600' (exit {code})"
    );
    println!("{}", String::from_utf8_lossy(&out).trim());
    print!("{}", String::from_utf8(err).unwrap_or_default());
}
