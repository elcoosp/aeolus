# Migrating from gpui-rsx

The migration guide lives at `docs/MIGRATION.md` and is included
verbatim below — the behavioural-differences table, the `rsx.toml`
import seam and the class-string story. The one-sentence version: hand
your `rsx.toml` to `aeolus_theme::import_rsx`, keep your class
vocabulary, and gain the v4 variant surface, the v4 CSS-first theme
and one compiler for static and dynamic strings — after reading the
spacing-scale row, which is the one difference that changes pixels.

{{#include ../../MIGRATION.md}}
