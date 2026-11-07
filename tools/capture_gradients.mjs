/// v0.6 capture: real tailwindcss 4.1.17 gradient output (plan §1 evidence).
/// Records the verbatim utilities-layer rules + @property initial values for
/// the gradient families so the aeolus composition semantics are pinned
/// against the real compiler (captured values go into unit tests).
import { spawnSync } from "node:child_process";
import { mkdtempSync, writeFileSync, readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const CLASSES = [
  "bg-linear-to-r from-red-500 to-blue-500",
  "bg-linear-to-r from-red-500",
  "bg-linear-to-r",
  "bg-linear-45 from-red-500 to-blue-500",
  "bg-linear-to-tr from-red-500 to-blue-500",
  "bg-linear-to-b",
  "bg-gradient-to-r from-red-500 to-blue-500",
  "bg-radial from-red-500 to-blue-500",
  "bg-conic from-red-500 to-blue-500",
  "bg-conic-45",
  "from-red-500 to-blue-500",
  "from-20% via-50% to-80%",
  "via-yellow-400",
  "from-red-500/50",
  "bg-linear-to-r in oklab from-red-500 to-blue-500",
  "bg-red-500 bg-linear-to-r from-red-500 to-blue-500",
  "bg-linear-to-r from-red-500 to-blue-500 bg-red-500",
];

const dir = mkdtempSync(join(tmpdir(), "tw-grad-"));
// The scratch dir needs tailwindcss resolvable: reuse the manual install when
// present (npm i tailwindcss@4.1.17 @tailwindcss/cli@4.1.17), else npx covers it.
const soup = CLASSES.flatMap((c) => c.split(" ").map((k) => `<div class="${k}"></div>`)).join("\n");
writeFileSync(join(dir, "soup.html"), soup);
writeFileSync(join(dir, "input.css"), '@import "tailwindcss";\n@source "../soup.html";\n');

const r = spawnSync("npx", ["@tailwindcss/cli@4.1.17", "-i", "input.css", "-o", "out.css"], {
  cwd: dir,
  encoding: "utf8",
  stdio: ["ignore", "pipe", "pipe"],
});
if (r.status !== 0) {
  console.error(r.stderr);
  process.exit(1);
}
const css = readFileSync(join(dir, "out.css"), "utf8");
// Print every rule whose selector mentions a corpus class, plus @property blocks.
const lines = css.split("\n");
let keep = false;
for (const line of lines) {
  if (line.startsWith("@property") || line.startsWith("  syntax:")) {
    console.log(line.trim());
    continue;
  }
  if (line.startsWith(".")) {
    keep = CLASSES.some((c) =>
      c.split(" ").some((k) => line.includes(k.replace(/[\/]/g, "\\/").slice(0, 12))),
    );
  }
  if (keep) console.log(line);
}
