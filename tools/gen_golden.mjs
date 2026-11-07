#!/usr/bin/env node
/// Golden-corpus generator (v0.4 §2) — AEOLUS_V0.4.md.
///
/// Runs the REAL tailwindcss v4.1.17 compiler over the corpus class list
/// (one HTML soup per theme) and writes the verbatim utilities-layer rule
/// per class into `crates/aeolus-core/tests/fixtures/golden.json`, next to
/// the `:root` theme variables and `@property` initial values the parity
/// harness needs to resolve `var()` references.
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
];

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

/** Escape a class name the way tailwind escapes selectors. */
function escapeClassName(cls) {
  let out = "";
  for (const ch of cls) {
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
for (const [name, t] of Object.entries(themes)) {
  const classes = CORPUS.filter((e) => e.theme === name).map((e) => e.class);
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
      console.error(`[gen] WARNING: no golden rule for ${name}:${cls} (selector ${sel})`);
      continue;
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
console.log(`[gen] wrote ${nEntries} golden entries for ${Object.keys(themes_out).length} themes -> ${OUT}`);
