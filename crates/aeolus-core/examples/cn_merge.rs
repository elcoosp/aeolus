//! Consumer example: `cn()` — the tw_merge-compatible conflict
//! resolution that lets a user's classes merge LAST and win over a
//! component's table.
//!
//! ```sh
//! cargo run -p aeolus-core --example cn_merge
//! ```
//!
//! The function is total and idempotent on any input; conflicts are
//! decided by an audited group table (including the per-edge padding and
//! the `bg-color`/`bg-image` split), so a component's `cn(classes,
//! user_classes)` behaves like the web's tailwind-merge.

use aeolus_core::cn;

fn main() {
    // A component's table and the user's override, merged rightmost-wins:
    let merged = cn(&[
        "p-4 px-8 rounded-md bg-primary hover:bg-primary/90",
        "rounded-full bg-red-500",
    ]);
    println!("component : p-4 px-8 rounded-md bg-primary hover:bg-primary/90");
    println!("user      : rounded-full bg-red-500");
    println!("cn()      : {merged}\n");

    // The user's radii and color won; the component's hover layer rides on.
    let tokens: Vec<&str> = merged.split_ascii_whitespace().collect();
    assert!(
        tokens.contains(&"rounded-full") && !tokens.contains(&"rounded-md"),
        "the later radius wins: {merged}"
    );
    assert!(
        tokens.contains(&"bg-red-500") && !tokens.contains(&"bg-primary"),
        "the later color wins: {merged}"
    );

    // cn() is idempotent: merging the merge changes nothing.
    let once = cn(&[merged.as_str()]);
    assert_eq!(once, merged, "cn() is idempotent on its own output");

    // Non-conflicting classes pass through untouched, in order:
    let mixed = cn(&["p-4", "text-sm", "hover:bg-primary/90"]);
    assert_eq!(mixed, "p-4 text-sm hover:bg-primary/90");
    println!(
        "idempotent, order-preserving, conflicts by group — the same merge the CLI exposes as `aeolus cn`."
    );
}
