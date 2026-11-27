# Conformance scores

Every backend's published score comes from its own capability impl —
the per-longhand tiered verdicts the emit gates actually run, never a
parallel claim. The page below is the GPUI conformance page
(`docs/CONFORMANCE.md`) included verbatim; the five interpreter lanes'
pages sit beside it in `docs/` (`CONFORMANCE-xilem.md`,
`CONFORMANCE-floem.md`, `CONFORMANCE-iced.md`, `CONFORMANCE-egui.md`,
`CONFORMANCE-slint.md`), all rendered from the same committed rows.

Two numbers answer different questions on each page: the per-longhand
ceiling (the share of the 160 longhands the backend can carry, native
or polyfill) and the corpus-at-scale line (the per-entry share of the
expanded golden corpus the backend can carry fully). A project can tier
its own classes against the same tables with `aeolus audit --target
<backend>`.

{{#include ../../CONFORMANCE.md}}
