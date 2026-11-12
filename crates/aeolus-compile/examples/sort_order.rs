//! Consumer demonstration: the class-sorting seam — plan §5.5's
//! tooling-parity line, "class sorting (official order from the
//! registry)". One read-side walk orders a class string into the
//! registry rows' canonical utility progression, variants bucketed
//! after the base by the IR's own cascade specificity — the same rank
//! the pipeline sorts layers by, so the sorted string reads in the
//! order the CSS emit arm renders rules.
//!
//! ```sh
//! cargo run -p aeolus-compile --example sort_order
//! ```
//!
//! The three scenes below are the seam's whole contract:
//!
//! * the registry order (base classes first, the rows' progression);
//! * the variant buckets (base < dark < probe states < media);
//! * the honest tail (unknown classes ride the source-order tail with
//!   the same typed, coded diagnostics the compile walk reports).

use aeolus_compile::sort;

fn main() {
    // 1. The registry order: spacing families precede sizing, sizing
    //    precedes layout, layout precedes colour — the rows' audited
    //    progression, whatever order the author typed.
    let sorted = sort("bg-red-500 hidden w-8 block p-4 mx-2").expect("non-empty input sorts");
    println!("input : bg-red-500 hidden w-8 block p-4 mx-2");
    println!("sorted: {}", sorted.to_class_string());
    for class in &sorted.classes {
        println!(
            "  {:?} -> bucket {}, row {}",
            class.raw, class.key.bucket, class.key.row
        );
    }

    // 2. The variant buckets ride AFTER every base class, ordered by
    //    the cascade specificity the layers themselves sort by.
    let sorted = sort("md:p-4 hover:p-3 dark:p-2 p-1").expect("non-empty input sorts");
    println!("\ninput : md:p-4 hover:p-3 dark:p-2 p-1");
    println!("sorted: {}", sorted.to_class_string());

    // 3. The honest tail: a class the registry cannot order is never
    //    reordered among its peers — it rides the tail and reports the
    //    typed miss (the no-silent-failures rule, applied to sorting).
    let sorted = sort("p-4 not-a-utility mx-2").expect("non-empty input sorts");
    println!("\ninput : p-4 not-a-utility mx-2");
    println!("sorted: {}", sorted.to_class_string());
    for diagnostic in &sorted.diagnostics {
        println!("  miss: {diagnostic} [{}]", diagnostic.kind.code());
    }
}
