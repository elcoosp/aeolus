//! Consumer demonstration: the backend-neutral `Emit` seam (plan §3.2's
//! `aeolus-codegen` row). A target crate implements [`Emit`]: it walks
//! the IR's layers and declarations and renders its own builder calls —
//! one program in, one output out, refusing per-longhand rather than
//! per-program ([`Unsupported`] is the capability report; never a
//! silent drop).
//!
//! The demo target is a TEXT emitter: it renders each longhand as a
//! `prop = value` line (the shape a doc tool or a linter renders),
//! refuses the longhands it has no text for, and reports them — the
//! same honest-failure contract the compiler itself follows. The
//! program is hand-built from the IR's own constructors (`Layer::new`),
//! exactly the shape the static-data arm freezes.
//!
//! ```sh
//! cargo run -p aeolus-codegen --example text_emit
//! ```

use aeolus_codegen::{Emit, Unsupported};
use aeolus_ir::{Cond, Decl, Layer, Program, Prop, Val};

/// The demo program: one base layer (padding + width + a ring the demo
/// target refuses), one hover layer — hand-built over the IR's own
/// `Layer::new` constructors, masks derived at construction.
static PROGRAM: Program = Program::new(&[
    Layer::new(
        Cond::Base,
        &[
            Decl {
                prop: Prop::PaddingTop,
                val: Val::Rem(1.0),
            },
            Decl {
                prop: Prop::Width,
                val: Val::Rem(16.0),
            },
            Decl {
                prop: Prop::RingWidth,
                val: Val::Px(1.0),
            },
        ],
    ),
    Layer::new(
        Cond::Hover,
        &[Decl {
            prop: Prop::Width,
            val: Val::Rem(17.0),
        }],
    ),
]);

/// The text target: every longhand it knows renders as one line, every
/// longhand it does not know refuses — the capability report in
/// cascade order.
struct TextTarget;

impl Emit for TextTarget {
    type Output = String;

    fn emit(&self, program: &Program) -> Result<String, Vec<Unsupported>> {
        let mut text = String::new();
        let mut unsupported = Vec::new();
        for layer in program.layers {
            for decl in layer.decls {
                match render(decl) {
                    Some(line) => {
                        text.push_str(&line);
                        text.push('\n');
                    }
                    None => unsupported.push(Unsupported::new(
                        decl.prop,
                        "the text target renders lengths and colours only",
                    )),
                }
            }
        }
        if unsupported.is_empty() {
            Ok(text)
        } else {
            Err(unsupported)
        }
    }
}

/// The target's own capability decision: `Some(line)` when it can
/// express the declaration, `None` when it refuses. The refusal is by
/// LONGHAND (the demo target has no text for the `--tw-ring-*` custom
/// properties — the same shape a real backend's capability table takes),
/// then by value family.
fn render(decl: &Decl) -> Option<String> {
    if matches!(decl.prop, Prop::RingWidth) {
        return None;
    }
    let prop = decl.prop.name();
    match decl.val {
        Val::Px(px) => Some(format!("{prop} = {px}px")),
        Val::Rem(rem) => Some(format!("{prop} = {rem}rem")),
        Val::Rgba([r, g, b, a]) => Some(format!("{prop} = rgba({r}, {g}, {b}, {a})")),
        Val::Kw(_) => Some(format!("{prop} = <keyword row>")),
        // Everything else (tokens, pairs, percentages, numbers) refuses
        // — the demo target's documented capability boundary.
        _ => None,
    }
}

fn main() {
    println!("── the demo program ──");
    for layer in PROGRAM.layers {
        let props: Vec<&str> = layer.mask.to_vec().iter().map(|p| p.name()).collect();
        println!("  {:?}: {}", layer.cond, props.join(", "));
    }

    println!("\n── the target's answer ──");
    match TextTarget.emit(&PROGRAM) {
        Ok(text) => println!("{text}"),
        Err(report) => {
            println!("  refused (the report is per-longhand, in cascade order):");
            for entry in &report {
                println!("  {} — {}", entry.prop.name(), entry.reason);
            }
            assert_eq!(report.len(), 1, "only the ring longhand refuses");
        }
    }

    println!("\nthe seam held: one program in, one report out, nothing dropped silently.");
}
