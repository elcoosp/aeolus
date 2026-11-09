//! Consumer demonstration: the headless CLI's `--theme` seam used as a
//! library — `load_theme` resolves a theme SPEC (`shadcn`,
//! `shadcn-dark`, `default`, or a path to theme CSS) into a
//! `TailwindTheme`, and its failures are TYPED (`ThemeSpecError`), so
//! programmatic consumers match on the failure mode instead of parsing
//! prose. With the CLI's opt-in `miette` feature the variants
//! additionally carry structured help text (`--features miette`); the
//! `tracing` feature spans the same seam.
//!
//! ```sh
//! cargo run -p aeolus-cli --example theme_spec
//! ```

use aeolus_cli::{ThemeSpecError, load_theme};
use aeolus_core::Resolver;

/// The same Button string the core example and the benches resolve.
const BUTTON: &str = "inline-flex items-center justify-center gap-2 rounded-md text-sm font-medium bg-primary text-primary-foreground hover:bg-primary/90";

fn main() {
    // The happy path: a built-in spec parses into the theme.
    let theme = load_theme("shadcn").expect("the shadcn spec is built in");
    println!("load_theme(\"shadcn\") → ok (the embedded light palette)");

    // The typed misses: match on the VARIANT, render via Display.
    for spec in ["no-such-spec", "./definitely-missing.css"] {
        match load_theme(spec) {
            Ok(_) => println!("load_theme({spec:?}) unexpectedly ok"),
            Err(err @ ThemeSpecError::UnknownSpec { .. }) => {
                println!("load_theme({spec:?}) → unknown_spec: {err}");
            }
            Err(err @ ThemeSpecError::ReadFile { kind, .. }) => {
                println!("load_theme({spec:?}) → read_file ({kind}): {err}");
            }
            Err(err) => println!("load_theme({spec:?}) → {err}"),
        }
    }

    // From spec to plan: the theme resolves classes into the typed IR —
    // the same portable snapshot the CLI writes on stdout.
    let resolver = Resolver::new(&theme);
    let resolution = resolver
        .resolve(BUTTON)
        .expect("the Button table resolves warning-free");
    let json = resolution.plan.to_snapshot_json();
    println!(
        "\nresolved the Button table: {} base slots, {} layers, snapshot {} bytes (schema v{})",
        resolution.plan.base.slots().len(),
        resolution.plan.layers.len(),
        json.len(),
        aeolus_core::PLAN_SNAPSHOT_VERSION,
    );
}
