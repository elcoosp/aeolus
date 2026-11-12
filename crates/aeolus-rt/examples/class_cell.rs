//! Consumer demonstration: the dynamic-strings slow lane (architecture
//! plan §3.6's surface (3)) — the per-view, UI-thread `ClassCell`
//! behind "compiled when the string changes, a hash compare per frame
//! otherwise".
//!
//! ```sh
//! cargo run -p aeolus-rt --example class_cell
//! ```
//!
//! The example walks a tiny view through five frames: the user's class
//! string changes twice, holds for the fast path, breaks once (an
//! empty string — the honest per-frame degradation, the last good
//! program standing), and returns. The cell's counters, diagnostics
//! channel and the interned programs themselves tell the story — the
//! dynamic lane is the SAME compiler as `tw!`, runtime-driven.

use aeolus_compile::Config;
use aeolus_rt::{ClassCell, StaticProgram, ThemeSource, TokenTable};

/// The view-state sketch from the plan: one slot per component, owned
/// by the view, consulted per render.
struct Card {
    user_classes: String,
    classes: ClassCell,
    table: TokenTable,
}

impl Card {
    /// One render frame: the content+identity compare first, the full
    /// compile on a change, the last good program on an error.
    fn render(&self) {
        println!("frame: user_classes = {:?}", self.user_classes);
        match self
            .classes
            .get_theme(&self.user_classes, &self.table, &Config::default())
        {
            Ok(program) => {
                println!(
                    "  compiled program: {} layer(s), {} declaration(s)",
                    program.layers.len(),
                    program.layers.iter().map(|l| l.decls.len()).sum::<usize>()
                );
                for diagnostic in self.classes.diagnostics() {
                    println!("  diagnostic: {diagnostic}");
                }
            }
            Err(err) => {
                // The empty input (or any whole-string refusal) keeps
                // the last good program standing — never a panic per
                // frame, never a silent reset.
                let standing: StaticProgram = self.classes.program();
                println!(
                    "  compile failed ({err}); the last good program stands: {} layer(s)",
                    standing.layers.len()
                );
            }
        }
        println!(
            "  counters: {} hit(s), {} miss(es)",
            self.classes.hits(),
            self.classes.misses()
        );
    }
}

fn main() {
    let source = ThemeSource::shadcn().expect("the built-in shadcn source parses");
    let table = TokenTable::runtime(&source).expect("the runtime table builds");

    let mut card = Card {
        user_classes: "p-4 rounded-md".to_string(),
        classes: ClassCell::new(),
        table,
    };

    card.render(); // miss 1: the first compile
    card.render(); // hit: the content compare
    card.user_classes = "p-8 rounded-lg bg-primary".to_string();
    card.render(); // miss 2: the string changed
    card.user_classes.clear();
    card.render(); // the typed Empty error; p-8 still stands
    card.user_classes = "p-8 rounded-lg bg-primary".to_string();
    card.render(); // hit: the errored frame left the last good slot standing
}
