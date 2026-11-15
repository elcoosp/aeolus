//! The wider v4 hint table's demo (architecture plan §3.1's value
//! pass — the hint line): one runnable walk over every landed hint —
//! `angle:` on the degree rows, `time:` on the millisecond rows,
//! `position:` on the object-position row, `family-name:`/
//! `generic-name:` on the font-family row, `absolute-size:`/
//! `relative-size:` on the font-size row and `string:` on the content
//! root (the NEW `Prop::Content` longhand, the CSS `content` property
//! the `content-["…"]` spelling writes).
//!
//! Run: `cargo run -p aeolus-compile --example hints`.
//!
//! The output walks the SAME pipeline every family shares: the IR
//! declaration per class, the CSS arm's rendering (the unit suffixes
//! reconstruct from the row's own number form — the keyword path's
//! oracle), the interp's effective answer and the honest boundaries
//! (the hint mismatches and the rows that take no hint refuse typed).

use aeolus_compile::{Config, Env, cn, compile, emit_css};

fn main() {
    let config = Config::default();

    // The landed hint battery: one form per family.
    let hinted = [
        "rotate-[angle:45deg]",
        "duration-[time:2s]",
        "object-[position:50%_25%]",
        "font-[family-name:\"Inter_Sans\"]",
        "font-[generic-name:serif]",
        "text-[absolute-size:larger]",
        "text-[relative-size:smaller]",
        "content-[\"hello_world\"]",
        // The free spellings ride too (the deg precedent: the row's
        // own number form IS the unit's oracle).
        "duration-[2s]",
    ];

    println!("== the wider v4 hint table ==\n");
    for class in hinted {
        let compiled = compile(class, &config).expect("non-empty input compiles");
        if compiled.diagnostics.is_empty() {
            let decls = compiled.program.layers[0]
                .decls
                .iter()
                .map(|d| format!("{} = {:?}", d.prop.name(), d.val))
                .collect::<Vec<_>>()
                .join(", ");
            println!("{class}");
            println!("  IR:      {decls}");
            let sheet = emit_css(class, &config).expect("the CSS walk is total");
            println!("  CSS:     {}", sheet.rules[0].body.replace('\n', " | "));
            let interped = compiled
                .interp(&Env::default())
                .expect("the base situation resolves");
            let effective = interped
                .decls
                .iter()
                .map(|d| d.prop.name())
                .collect::<Vec<_>>()
                .join(", ");
            println!("  interp:  {effective}");
        } else {
            println!("{class}\n  refused: {}", compiled.diagnostics[0]);
        }
        println!();
    }

    // The honest boundaries: the hint grammar's own mismatch and the
    // rows that take no hint.
    println!("== the typed boundaries ==\n");
    for class in [
        "rotate-[angle:0.5turn]",    // a non-deg spelling on a degree row
        "z-[angle:45deg]",           // no angle row in the z root
        "w-[family-name:\"Inter\"]", // no family row in the w root
        "duration-[time:2]",         // a unitless time spelling
    ] {
        let compiled = compile(class, &config).expect("the walk is total");
        println!("{class}\n  refused: {}\n", compiled.diagnostics[0]);
    }

    // The merge folds the hinted forms per longhand, same as ever.
    let first = compile("content-[\"first\"]", &config).expect("compiles");
    let second = compile("content-[\"second\"]", &config).expect("compiles");
    let merged = cn(&[&first, &second]).expect("the merge is total");
    let winner = merged.program.layers[0].decls[0].val;
    println!("== the merge ==");
    println!("content-[\"first\"] ⊕ content-[\"second\"] → {winner:?}");
}
