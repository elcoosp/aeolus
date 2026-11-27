//! Consumer demonstration: the builder-chain emitter's seam face —
//! `lower_class_builder`, the third-party entry the §5 integration
//! bullet names (the `resolve_style()` equivalent of gpui-rsx's
//! `StyleOperation`). One class literal in, the builder-call closure
//! out; a chain-inexpressible class refuses typed, the reason naming
//! the lane that works.
//!
//! ```sh
//! cargo run -p aeolus-codegen-gpui --example builder
//! ```

use aeolus_codegen_gpui::lower_class_builder;
use aeolus_compile::Config;

fn main() {
    for literal in [
        "flex flex-col p-4 bg-primary",
        "relative justify-between underline cursor-pointer",
        "opacity-50 line-clamp-2",
        // The chain gate's typed refusals: the theme-token slot (the
        // runtime table resolves it) and the ring polyfill's state
        // accumulator — the frozen-program lane keeps the real output.
        "rotate-45",
        "ring-2 ring-red-500",
    ] {
        println!("{literal}");
        match lower_class_builder(literal, &Config::default()) {
            Ok(tokens) => println!("  {}", tokens.to_string().replace('\n', " ")),
            Err(diagnostics) => {
                for message in &diagnostics.messages {
                    println!("  compile: {message}");
                }
                for refusal in &diagnostics.unsupported {
                    println!("  refuse {}: {}", refusal.prop.name(), refusal.reason);
                }
            }
        }
        println!();
    }
}
