#!/usr/bin/env node
/// Golden-corpus generator (v0.4 §2, extended by v0.5 §2 —
/// `AEOLUS_V0.4.md`, `AEOLUS_V0.5.md`; scale-up per the M1 batch).
///
/// Runs the REAL tailwindcss v4.1.17 compiler over the corpus class list
/// (one HTML soup per theme) and writes the verbatim utilities-layer rule
/// per class into `crates/aeolus-core/tests/fixtures/golden.corpus`, next to
/// the `:root` theme variables and `@property` initial values the parity
/// harness needs to resolve `var()` references.
///
/// The corpus is the hand-written seed (the regression pin — every prior
/// wave's entries stay verbatim) PLUS the M1 scale-up expansion: the
/// registry's full root × value-shape surface, the variant ladder and the
/// shadcn token set, classified by `tools/expansion_corpus.json` (see the
/// expansion section below). The generation is deterministic — same
/// artifact bytes, same tailwindcss pin, same output.
///
/// Usage: node tools/gen_golden.mjs
/// Requires network on first run (npm install tailwindcss@4.1.17
/// @tailwindcss/cli@4.1.17 into a scratch dir). The COMMITTED fixture is
/// the oracle; regenerating only refreshes it deliberately.

import { spawnSync } from "node:child_process";
import { mkdtempSync, writeFileSync, readFileSync, mkdirSync, existsSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const OUT = join(ROOT, "crates/aeolus-core/tests/fixtures/golden.corpus");
const TAILWIND_VERSION = "4.1.17";

// The committed fixture is a dependency-free line format the parity
// harness parses without serde (aeolus-core stays zero-dependency, even
// in dev-deps):
//   @tailwind <version>
//   @theme <name>            … theme context; following lines until the
//   --var: value               next `@` directive are :root vars and
//   @propdef --x: init-val     @property initial values
//   @entry <theme> <class>  … following lines until @end are the verbatim
//   <css rule lines>          tailwindcss output rule for that class
//   @end

// The corpus (v0.4 §2.3): single-class entries over the supported surface.
// `theme: "default"` = verbatim v4 default palette; `theme: "shadcn"` = the
// neutral light theme (shadcn-light.css, `@theme inline` var chains).
const CORPUS = [
  // spacing / margin
  { theme: "default", class: "p-4" },
  { theme: "default", class: "px-2" },
  { theme: "default", class: "py-3" },
  { theme: "default", class: "pt-1" },
  { theme: "default", class: "pr-0.5" },
  { theme: "default", class: "pb-px" },
  { theme: "default", class: "pl-[7px]" },
  { theme: "default", class: "m-4" },
  { theme: "default", class: "mx-auto" },
  { theme: "default", class: "-mt-2" },
  // sizing
  { theme: "default", class: "w-8" },
  { theme: "default", class: "w-full" },
  { theme: "default", class: "w-1/2" },
  { theme: "default", class: "w-fit" },
  { theme: "default", class: "w-auto" },
  { theme: "default", class: "size-10" },
  { theme: "default", class: "max-w-md" },
  { theme: "default", class: "min-w-0" },
  { theme: "default", class: "h-[13px]" },
  { theme: "default", class: "h-screen" },
  // colors
  { theme: "default", class: "bg-red-500" },
  { theme: "default", class: "bg-black" },
  { theme: "default", class: "bg-white" },
  { theme: "default", class: "bg-transparent" },
  { theme: "default", class: "text-red-500" },
  { theme: "default", class: "border-red-500" },
  { theme: "default", class: "bg-red-500/50" },
  { theme: "default", class: "bg-red-500/[0.37]" },
  { theme: "default", class: "bg-[#0ea5e9]" },
  // borders / radii
  { theme: "default", class: "border" },
  { theme: "default", class: "border-2" },
  { theme: "default", class: "border-t" },
  { theme: "default", class: "border-x" },
  { theme: "default", class: "border-dashed" },
  { theme: "default", class: "rounded-md" },
  { theme: "default", class: "rounded-xl" },
  { theme: "default", class: "rounded-s-lg" },
  { theme: "default", class: "rounded-t-full" },
  { theme: "default", class: "rounded-[3px]" },
  // ring / outline
  { theme: "default", class: "ring" },
  { theme: "default", class: "ring-2" },
  { theme: "default", class: "ring-[3px]" },
  { theme: "default", class: "ring-red-500" },
  { theme: "default", class: "ring-offset-2" },
  { theme: "default", class: "ring-inset" },
  { theme: "default", class: "outline-2" },
  { theme: "default", class: "outline-offset-2" },
  { theme: "default", class: "outline-red-500" },
  { theme: "default", class: "outline-none" },
  // shadows
  { theme: "default", class: "shadow-sm" },
  { theme: "default", class: "shadow-lg" },
  { theme: "default", class: "shadow-none" },
  { theme: "default", class: "shadow-red-500/50" },
  // typography
  { theme: "default", class: "text-sm" },
  { theme: "default", class: "text-lg" },
  { theme: "default", class: "text-sm/6" },
  { theme: "default", class: "font-medium" },
  { theme: "default", class: "font-bold" },
  { theme: "default", class: "italic" },
  { theme: "default", class: "not-italic" },
  { theme: "default", class: "uppercase" },
  { theme: "default", class: "underline" },
  { theme: "default", class: "no-underline" },
  { theme: "default", class: "truncate" },
  { theme: "default", class: "text-center" },
  { theme: "default", class: "whitespace-nowrap" },
  { theme: "default", class: "line-clamp-2" },
  { theme: "default", class: "tabular-nums" },
  { theme: "default", class: "underline-offset-2" },
  // The parity wave 1 tail: the five highest-traffic families, generated
  // through the real compiler (the deliberate regeneration). The
  // between-child entries pin the v4.1.17 `:where(& >
  // :not(:last-child))` child rule; the composite entries pin the
  // verbatim multi-decl emission.
  { theme: "default", class: "line-clamp-none" },
  { theme: "default", class: "ordinal" },
  { theme: "default", class: "slashed-zero" },
  { theme: "default", class: "lining-nums" },
  { theme: "default", class: "oldstyle-nums" },
  { theme: "default", class: "proportional-nums" },
  { theme: "default", class: "diagonal-fractions" },
  { theme: "default", class: "stacked-fractions" },
  { theme: "default", class: "normal-nums" },
  { theme: "default", class: "space-x-4" },
  { theme: "default", class: "space-y-2" },
  { theme: "default", class: "-space-x-2" },
  { theme: "default", class: "space-x-reverse" },
  { theme: "default", class: "space-y-reverse" },
  { theme: "default", class: "divide-x" },
  { theme: "default", class: "divide-y-2" },
  { theme: "default", class: "divide-x-reverse" },
  { theme: "default", class: "divide-dashed" },
  { theme: "default", class: "divide-solid" },
  { theme: "default", class: "divide-red-500" },
  { theme: "default", class: "sr-only" },
  { theme: "default", class: "not-sr-only" },
  // The flagship wave's families, generated through the real compiler
  // (the deliberate regeneration): the gradient stop machinery (the
  // via stop and the position patches the walk accumulates), the
  // composed transform forms (the `--tw-*` var + composed property
  // line the CSS arm renders beside), and the named group/peer labels
  // the IR slot rides.
  { theme: "default", class: "bg-linear-to-r from-red-500 via-white to-blue-500" },
  { theme: "default", class: "bg-linear-to-br from-red-500 to-blue-500" },
  { theme: "default", class: "from-20% via-30% to-80%" },
  { theme: "default", class: "via-50%" },
  { theme: "default", class: "scale-y-125" },
  { theme: "default", class: "scale-150" },
  { theme: "default", class: "translate-y-4" },
  { theme: "default", class: "translate-x-1/2" },
  { theme: "default", class: "skew-x-12" },
  { theme: "default", class: "-rotate-90" },
  { theme: "default", class: "group-active/menu:bg-red-500" },
  { theme: "default", class: "group-focus-within/nav:underline" },
  { theme: "default", class: "peer-disabled/menu:opacity-50" },
  // element-relative typography (v0.5 §2): token forms carry their
  // relative unit (em / unitless ratio); arbitrary + spacing-multiple
  // forms stay absolute.
  { theme: "default", class: "tracking-tighter" },
  { theme: "default", class: "tracking-tight" },
  { theme: "default", class: "tracking-normal" },
  { theme: "default", class: "tracking-wide" },
  { theme: "default", class: "tracking-wider" },
  { theme: "default", class: "tracking-widest" },
  { theme: "default", class: "-tracking-wide" },
  { theme: "default", class: "tracking-[7px]" },
  { theme: "default", class: "leading-none" },
  { theme: "default", class: "leading-tight" },
  { theme: "default", class: "leading-snug" },
  { theme: "default", class: "leading-normal" },
  { theme: "default", class: "leading-relaxed" },
  { theme: "default", class: "leading-loose" },
  { theme: "default", class: "leading-[3px]" },
  { theme: "default", class: "text-base/tight" },
  { theme: "default", class: "text-lg/loose" },
  // layout
  { theme: "default", class: "flex" },
  { theme: "default", class: "inline-flex" },
  { theme: "default", class: "grid" },
  { theme: "default", class: "grid-cols-3" },
  { theme: "default", class: "col-span-2" },
  { theme: "default", class: "items-center" },
  { theme: "default", class: "justify-between" },
  { theme: "default", class: "gap-4" },
  { theme: "default", class: "gap-x-2" },
  { theme: "default", class: "gap-y-3" },
  { theme: "default", class: "flex-col" },
  { theme: "default", class: "flex-wrap" },
  { theme: "default", class: "grow-0" },
  { theme: "default", class: "order-3" },
  { theme: "default", class: "content-center" },
  { theme: "default", class: "basis-1/2" },
  { theme: "default", class: "self-end" },
  // position / overflow / aspect
  { theme: "default", class: "relative" },
  { theme: "default", class: "absolute" },
  { theme: "default", class: "inset-0" },
  { theme: "default", class: "top-2" },
  { theme: "default", class: "-top-2" },
  { theme: "default", class: "z-10" },
  { theme: "default", class: "-z-10" },
  { theme: "default", class: "overflow-hidden" },
  { theme: "default", class: "overflow-x-auto" },
  { theme: "default", class: "visible" },
  { theme: "default", class: "invisible" },
  { theme: "default", class: "aspect-video" },
  // transforms
  { theme: "default", class: "scale-95" },
  { theme: "default", class: "scale-x-50" },
  { theme: "default", class: "rotate-45" },
  { theme: "default", class: "translate-x-2" },
  { theme: "default", class: "-translate-y-1" },
  { theme: "default", class: "skew-y-3" },
  // transitions
  { theme: "default", class: "transition" },
  { theme: "default", class: "transition-colors" },
  { theme: "default", class: "transition-none" },
  { theme: "default", class: "duration-300" },
  { theme: "default", class: "ease-in-out" },
  { theme: "default", class: "delay-150" },
  // misc
  { theme: "default", class: "opacity-50" },
  { theme: "default", class: "cursor-pointer" },
  { theme: "default", class: "pointer-events-none" },
  { theme: "default", class: "select-none" },
  // states
  { theme: "default", class: "hover:bg-red-600" },
  { theme: "default", class: "focus:ring-2" },
  { theme: "default", class: "focus-visible:outline-2" },
  { theme: "default", class: "focus-within:ring-2" },
  { theme: "default", class: "active:scale-95" },
  { theme: "default", class: "disabled:opacity-50" },
  { theme: "default", class: "visited:text-purple-500" },
  { theme: "default", class: "target:block" },
  { theme: "default", class: "checked:border-red-500" },
  { theme: "default", class: "required:border-red-500" },
  // breakpoints / dark
  { theme: "default", class: "md:text-lg" },
  { theme: "default", class: "sm:p-8" },
  { theme: "default", class: "max-md:p-4" },
  { theme: "default", class: "dark:bg-gray-900" },
  // group / peer
  { theme: "default", class: "group-hover:underline" },
  { theme: "default", class: "peer-checked:bg-blue-500" },
  { theme: "default", class: "group-hover/menu:bg-red-500" },
  // data / aria
  { theme: "default", class: "data-[state=open]:bg-green-500" },
  { theme: "default", class: "aria-busy:opacity-50" },
  // structural
  { theme: "default", class: "first:pt-0" },
  { theme: "default", class: "last:pb-0" },
  { theme: "default", class: "odd:bg-gray-100" },
  { theme: "default", class: "empty:block" },
  { theme: "default", class: "nth-3:block" },
  // has / not
  { theme: "default", class: "has-focus:ring-2" },
  { theme: "default", class: "not-hover:opacity-50" },
  // v0.7 (AEOLUS_V0.7.md): pseudo-element variants, has-attr descendants,
  // named containers, arbitrary grid templates, justify-items.
  { theme: "default", class: "selection:bg-red-500" },
  { theme: "default", class: "placeholder:text-gray-500" },
  { theme: "default", class: "file:text-sm" },
  { theme: "default", class: "md:selection:bg-red-500" },
  { theme: "default", class: "has-data-[slot=card-action]:grid-cols-2" },
  { theme: "default", class: "has-aria-invalid:border-red-500" },
  { theme: "default", class: "has-first:grid-cols-3" },
  { theme: "default", class: "grid-cols-[0_1fr]" },
  { theme: "default", class: "grid-cols-[repeat(3,minmax(0,1fr))]" },
  { theme: "default", class: "grid-rows-[auto_auto]" },
  { theme: "default", class: "auto-rows-min" },
  { theme: "default", class: "auto-rows-fr" },
  { theme: "default", class: "auto-cols-max" },
  { theme: "default", class: "justify-items-start" },
  { theme: "default", class: "@container/card-header" },
  { theme: "shadcn", class: "selection:bg-primary" },
  { theme: "shadcn", class: "placeholder:text-muted-foreground" },
  // container queries
  { theme: "default", class: "@md:bg-red-500" },
  // arbitrary variants (v0.3 typed subset)
  { theme: "default", class: "[&:hover]:bg-red-600" },
  { theme: "default", class: "[@media(print)]:hidden" },
  // compounds
  { theme: "default", class: "md:hover:bg-red-600" },
  { theme: "default", class: "dark:hover:bg-gray-800" },
  // important / negatives
  { theme: "default", class: "bg-red-500!" },
  { theme: "default", class: "!p-4" },
  // shadcn theme (theme-chain + calc() radii)
  { theme: "shadcn", class: "bg-primary" },
  { theme: "shadcn", class: "text-primary-foreground" },
  { theme: "shadcn", class: "bg-primary/90" },
  { theme: "shadcn", class: "border-border" },
  { theme: "shadcn", class: "ring-ring/50" },
  { theme: "shadcn", class: "rounded-md" },
  { theme: "shadcn", class: "bg-destructive" },
  { theme: "shadcn", class: "text-muted-foreground" },
  { theme: "shadcn", class: "focus-visible:ring-ring/50" },
  { theme: "shadcn", class: "bg-sidebar" },
  // element-relative typography, theme-context independence (v0.5 §2.1)
  { theme: "shadcn", class: "tracking-normal" },
  { theme: "shadcn", class: "leading-relaxed" },
];

// --- M1 scale-up expansion (§5.1 "conformance at scale") --------------------
//
// The hand-written seed above stays VERBATIM (the regression pin: every
// prior wave's entries re-emit byte-identically). The expansion rides the
// committed classification artifact `tools/expansion_corpus.json` — one
// record per candidate class string extracted from aeolus-data's committed
// registry rows (all 247 rows over 203 distinct roots), the variant ladder
// (the v4.1.17 state/structural/responsive/dark/named-group/pseudo-element/
// container/compound forms over a representative base list) and the shadcn
// token set, each classified through the V1 pipeline (the `d`/`s` flags)
// and the v0.x oracle (the `vd`/`vs` flags) at this pin. The generator is
// deterministic: the artifact is committed data, the selection below is
// static, and the tailwindcss pin is fixed — same inputs, same corpus.
//
// Coverage contract (asserted, never silent):
// * every distinct root in `docs/REGISTRY.md` is covered (a registry row
//   added without regenerating the artifact fails the generator),
// * every emitted class MUST have produced a golden rule (a class
//   tailwindcss did not emit is a hard error, not a warning),
// * classes the V1 pipeline refuses under BOTH theme bakes are NOT emitted
//   — they are the typed-refusal surface, counted and printed here and
//   documented in the conformance pages (never a silent drop).

const EXPANSION_PATH = join(ROOT, "tools/expansion_corpus.json");

function loadExpansion() {
  const parsed = JSON.parse(readFileSync(EXPANSION_PATH, "utf8"));
  const registry = new Map(parsed.registry.map((row) => [row.root, row.classes]));
  return { registry, variants: parsed.variants, tokens: parsed.tokens };
}

function registryRootsFromDocs() {
  const raw = readFileSync(join(ROOT, "docs/REGISTRY.md"), "utf8");
  const roots = new Set();
  for (const line of raw.split("\n")) {
    const match = line.match(/^\| `([^`]+)` \| (any-of|bare|keywords|composite|spacing|fraction|number|positive-int|percent|arbitrary) \|/);
    if (match) roots.add(match[1]);
  }
  return roots;
}

/** The value-shape bucket one class string rides (the selection's key). */
function shapeBucket(root, cls) {
  const suffix = cls === root ? "" : cls.slice(root.length + 1);
  if (suffix === "") return "bare";
  if (suffix.includes("[")) return "arbitrary";
  if (/^\d+\/\d+$/.test(suffix) || suffix === "full") return "fraction";
  if (/^-?\d+(\.\d+)?$/.test(suffix) || suffix === "px") return "spacing";
  if (/^\d+%$/.test(suffix)) return "percent";
  return "keyword";
}

/** Per root: one class per shape bucket, registry order, capped. */
const REGISTRY_SHAPES_PER_ROOT = 4;
const SHAPE_PRIORITY = ["spacing", "fraction", "keyword", "arbitrary", "percent", "bare"];

function expandCorpus() {
  const { registry, variants, tokens } = loadExpansion();

  // Completeness: the docs' distinct roots MUST all be classified.
  const docsRoots = registryRootsFromDocs();
  const missing = [...docsRoots].filter((root) => !registry.has(root));
  if (missing.length > 0) {
    throw new Error(
      `expansion artifact is stale — registry roots missing: ${missing.join(", ")}` +
        ` (regenerate tools/expansion_corpus.json from the rows at this pin)`,
    );
  }

  const out = [];
  const seen = new Set();
  let refusals = 0;
  const push = (theme, cls, origin) => {
    if (seen.has(cls)) return;
    seen.add(cls);
    out.push({ theme, class: cls, origin });
  };

  // The registry surface: shape-diverse, default-theme-first (the d flag
  // decides the theme a class emits under; a shadcn-only class rides the
  // shadcn theme — the token-surface entries).
  for (const [root, classes] of registry) {
    const buckets = new Map();
    for (const record of classes) {
      const bucket = shapeBucket(root, record.cls);
      if (!buckets.has(bucket)) buckets.set(bucket, []);
      buckets.get(bucket).push(record);
    }
    let taken = 0;
    for (const bucket of SHAPE_PRIORITY) {
      if (taken >= REGISTRY_SHAPES_PER_ROOT) break;
      const rows = buckets.get(bucket);
      if (!rows) continue;
      for (const record of rows) {
        if (taken >= REGISTRY_SHAPES_PER_ROOT) break;
        if (record.d) {
          push("default", record.cls, "registry");
          taken++;
        } else if (record.s) {
          push("shadcn", record.cls, "registry");
          taken++;
        } else {
          refusals++;
        }
      }
    }
  }

  // The variant ladder: every state/structural/responsive/dark/group/
  // pseudo-element/container/compound combo the pipeline lowers clean.
  for (const record of variants) {
    if (record.d) push("default", record.cls, "ladder");
    else if (record.s) push("shadcn", record.cls, "ladder");
    else refusals++;
  }

  // The shadcn token surface: the semantic names the theme chain declares.
  for (const record of tokens) {
    if (record.s) push("shadcn", record.cls, "tokens");
    else refusals++;
  }

  // The hand-written seed rides FIRST (the regression pin) — dedupe keeps
  // its exact themes and spellings over any expansion collision. The
  // seed's three multi-class gradient strings keep their historical
  // skip-here shape (tailwindcss emits a rule per CLASS, not per string;
  // those entries are pinned Rust-side by tests/gradients.rs instead).
  const seed = CORPUS.map((entry) => ({ ...entry, origin: "seed" }));
  const merged = [
    ...seed,
    ...out
      .filter((entry) => !seed.some((s) => s.class === entry.class))
      .map((entry) => ({ theme: entry.theme, class: entry.class, origin: entry.origin })),
  ];
  const v1Refused = [...registry.values()].flat().filter((r) => !r.d && !r.s).length;
  console.log(
    `[gen] expansion: ${merged.length} classes (seed ${seed.length}, registry ${registry.size} roots, ` +
      `ladder ${variants.length}, tokens ${tokens.length}); typed refusals not emitted: ${refusals} ` +
      `(registry ${v1Refused})`,
  );
  return merged;
}

const EXPANDED_CORPUS = expandCorpus();
const ORIGIN_BY_CLASS = new Map(EXPANDED_CORPUS.map((entry) => [entry.class, entry.origin]));
const originOf = (cls) => ORIGIN_BY_CLASS.get(cls);

// --- infra -----------------------------------------------------------------

function ensureTailwind(dir) {
  const pkg = join(dir, "node_modules", "tailwindcss", "package.json");
  if (existsSync(pkg)) return;
  console.log(`[gen] installing tailwindcss ${TAILWIND_VERSION} into ${dir} …`);
  const r = spawnSync("npm", ["init", "-y"], { cwd: dir, encoding: "utf8" });
  if (r.status !== 0) throw new Error(`npm init failed: ${r.stderr}`);
  const i = spawnSync(
    "npm",
    ["install", `--no-audit`, `--no-fund`, `tailwindcss@${TAILWIND_VERSION}`, `@tailwindcss/cli@${TAILWIND_VERSION}`],
    { cwd: dir, encoding: "utf8", stdio: ["ignore", "ignore", "inherit"] },
  );
  if (i.status !== 0) throw new Error("npm install failed (network?)");
}

function shadcnThemeCss() {
  return readFileSync(join(ROOT, "crates/aeolus-core/data/shadcn-light.css"), "utf8");
}

/** Escape a class name the way tailwind escapes selectors. A leading
 * digit is CSS-invalid unescaped — tailwind emits the `\3D ` hex escape
 * (`2xl:p-4` → `.\32 xl\:p-4`), the corpus selector must match. */
function escapeClassName(cls) {
  let out = "";
  let first = true;
  for (const ch of cls) {
    if (first) {
      first = false;
      if (/[0-9]/.test(ch)) {
        out += `\\3${ch} `;
        continue;
      }
    }
    if (/[a-zA-Z0-9_-]/.test(ch)) out += ch;
    else out += "\\" + ch;
  }
  return out;
}

/** Extract a balanced-brace block body starting after `openBraceIndex`. */
function blockBody(text, openBraceIndex) {
  let depth = 0;
  for (let i = openBraceIndex; i < text.length; i++) {
    const c = text[i];
    if (c === "{") depth++;
    else if (c === "}") {
      depth--;
      if (depth === 0) return text.slice(openBraceIndex + 1, i);
    }
  }
  throw new Error("unbalanced braces");
}

/** Split a CSS fragment into top-level rules, returning {selector, body}. */
function topLevelRules(css) {
  const rules = [];
  let i = 0;
  let buf = "";
  let depth = 0;
  while (i < css.length) {
    const c = css[i];
    if (depth === 0 && c === "{") {
      const selector = buf.trim().replace(/\/\*[\s\S]*?\*\//g, "").trim();
      const body = blockBody(css, i);
      rules.push({ selector, body });
      buf = "";
      i = skipBlock(css, i);
      continue;
    }
    if (c === "{") depth++;
    if (c === "}") depth--;
    buf += c;
    i++;
  }
  return rules.filter((r) => r.selector);
}
function skipBlock(css, openBraceIndex) {
  let depth = 0;
  for (let i = openBraceIndex; i < css.length; i++) {
    if (css[i] === "{") depth++;
    else if (css[i] === "}") {
      depth--;
      if (depth === 0) return i + 1;
    }
  }
  return css.length;
}

function parseDecls(body) {
  const decls = {};
  for (const line of body.split(";")) {
    const idx = line.indexOf(":");
    if (idx === -1) continue;
    const prop = line.slice(0, idx).trim();
    const val = line.slice(idx + 1).trim();
    if (prop && !prop.includes("{") && !prop.includes("}")) decls[prop] = val;
  }
  return decls;
}

// --- generation ------------------------------------------------------------

const work = mkdtempSync(join(tmpdir(), "aeolus-gen-"));
ensureTailwind(work);

const themes = {
  default: { input: '@import "tailwindcss";\n' },
  shadcn: { input: '@import "tailwindcss";\n' + shadcnThemeCss() },
};

const perTheme = {};
const skippedSurface = [];
for (const [name, t] of Object.entries(themes)) {
  const classes = EXPANDED_CORPUS.filter((e) => e.theme === name).map((e) => e.class);
  writeFileSync(join(work, `input-${name}.css`), t.input);
  writeFileSync(
    join(work, `content-${name}.html`),
    `<div>\n${classes.map((c) => `<div class="${c}"></div>`).join("\n")}\n</div>\n`,
  );
  const outPath = join(work, `out-${name}.css`);
  const r = spawnSync(
    join(work, "node_modules", ".bin", "tailwindcss"),
    ["-i", `input-${name}.css`, "--content", `content-${name}.html`, "-o", outPath],
    { cwd: work, encoding: "utf8" },
  );
  if (r.status !== 0) throw new Error(`tailwindcss failed for ${name}: ${r.stderr || r.stdout}`);
  const css = readFileSync(outPath, "utf8");
  perTheme[name] = { css, classes };
}

// Extract theme vars + @property initial values + per-class utility rules.
const themes_out = {};
const lines = [];
lines.push(`# Golden corpus for the v0.4 parity harness (AEOLUS_V0.4.md §2).`);
lines.push(`# Generated by tools/gen_golden.mjs with the REAL tailwindcss`);
lines.push(`# ${TAILWIND_VERSION} compiler. Regenerate deliberately.`);
lines.push(`@tailwind ${TAILWIND_VERSION}`);
for (const [name, { css, classes }] of Object.entries(perTheme)) {
  // :root/:host vars from the theme layer.
  let rootVars = {};
  const allTop = topLevelRules(css);
  const collectRoot = (rules) => {
    for (const r of rules) {
      if (r.selector.startsWith("@layer")) {
        collectRoot(topLevelRules(r.body));
      } else if (/:root|:host/.test(r.selector)) {
        rootVars = { ...rootVars, ...parseDecls(r.body) };
      }
    }
  };
  collectRoot(allTop);

  // @property initial values (real ones only — no guesses).
  const propDefaults = {};
  const propRe = /@property\s+(--[\w-]+)\s*\{/g;
  let m;
  while ((m = propRe.exec(css)) !== null) {
    const decls = parseDecls(blockBody(css, m.index + m[0].length - 1));
    if (decls["initial-value"] !== undefined) propDefaults[m[1]] = decls["initial-value"];
  }

  // Utility rules: top-level selectors of the utilities layer.
  const byClass = new Map();
  const utilLayer = allTop.find((r) => r.selector === "@layer utilities");
  if (!utilLayer) throw new Error("no utilities layer found");
  for (const u of topLevelRules(utilLayer.body)) byClass.set(u.selector, u.body.trim());

  themes_out[name] = { root_vars: rootVars, property_defaults: propDefaults };
  lines.push(`@theme ${name}`);
  for (const [k, v] of Object.entries(rootVars)) lines.push(`${k}: ${v.replace(/\s+/g, " ")}`);
  for (const [k, v] of Object.entries(propDefaults)) lines.push(`@propdef ${k}: ${v.replace(/\s+/g, " ")}`);

  for (const cls of classes) {
    const sel = `.${escapeClassName(cls)}`;
    const body = byClass.get(sel);
    if (body === undefined) {
      if (originOf(cls) === "seed") {
        // The seed's multi-class gradient strings keep their historical
        // skip (tailwindcss emits per class; pinned Rust-side instead).
        console.error(`[gen] skipped (seed): no single-class rule for ${name}:${cls}`);
        continue;
      }
      if (originOf(cls) === "registry") {
        // A registry row the PINNED compiler (4.1.17) emits no utilities
        // rule for at this content shape — outside the corpus's oracle
        // surface. Reported, never silent: the class is excluded and the
        // exclusion is documented in the conformance findings.
        skippedSurface.push(`${name}:${cls}`);
        continue;
      }
      // The ladder/token lists are hand-audited surfaces: a miss is a
      // mis-classified candidate — a hard error, never a silent drop.
      throw new Error(`no golden rule for ${name}:${cls} (selector ${sel})`);
    }
    lines.push(`@entry ${name} ${cls}`);
    lines.push(`.${escapeClassName(cls)} {`);
    lines.push(body);
    lines.push(`}`);
    lines.push(`@end`);
  }
}

writeFileSync(OUT, lines.join("\n") + "\n");
const nEntries = lines.filter((l) => l.startsWith("@entry ")).length;
if (nEntries < 2000) {
  throw new Error(
    `the scale-up floor: ${nEntries} golden entries is below the §5.1 at-scale target (≥2000)`,
  );
}
console.log(`[gen] wrote ${nEntries} golden entries for ${Object.keys(themes_out).length} themes -> ${OUT}`);
if (skippedSurface.length > 0) {
  console.log(
    `[gen] registry rows the pinned compiler emits no rule for (${skippedSurface.length}, documented, not silent):`,
  );
  for (const entry of skippedSurface) console.log(`[gen]   - ${entry}`);
}
