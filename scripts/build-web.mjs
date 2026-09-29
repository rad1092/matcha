#!/usr/bin/env node
// Bundles the web player into single files that work offline:
//   dist/matcha.html           complete document (open it from disk)
//   dist/matcha.fragment.html  same page without <!doctype>/<meta>, for hosts
//                              that wrap content in their own skeleton
//
// Inputs: web/src/index.html, web/src/app.js, web/matcha.js, web/matcha.wasm,
// web/roms/shelf.json (+ ROMs), docs/conformance.json (scoreboard numbers).

import { readFileSync, writeFileSync, mkdirSync, existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const read = (p) => readFileSync(join(root, p));
const text = (p) => read(p).toString("utf8");

const wasmPath = "web/matcha.wasm";
if (!existsSync(join(root, wasmPath))) {
  console.error(`missing ${wasmPath}: run scripts/build-wasm.sh first`);
  process.exit(1);
}

const shelf = JSON.parse(text("web/roms/shelf.json")).map((cart) => ({
  ...cart,
  romB64: read(`web/roms/${cart.file}`).toString("base64"),
}));

// Scoreboard numbers come from the last `matcha test --json` run.
const score = { blargg: ["?", "?"], mooneye: ["?", "?"], "dmg-acid2": ["?", "?"], gambatte: ["?", "?"], mealybug: ["?", "?"] };
const scorePath = join(root, "docs/conformance.json");
if (existsSync(scorePath)) {
  const { results } = JSON.parse(readFileSync(scorePath, "utf8"));
  for (const suite of Object.keys(score)) {
    const rows = results.filter((r) => r.suite === suite);
    score[suite] = [rows.filter((r) => r.status === "pass").length, rows.length];
  }
} else {
  console.warn("docs/conformance.json not found; scoreboard shows '?' (run: matcha test testdata/roms --json docs/conformance.json)");
}

const lib = text("web/matcha.js");
const app = text("web/src/app.js")
  .split("\n")
  .filter((line) => !line.includes("@bundle:strip"))
  .join("\n");

const meta = shelf.map(({ romB64, file, ...rest }) => rest);
const prelude = [
  `const WASM_B64 = ${JSON.stringify(read(wasmPath).toString("base64"))};`,
  `const SHELF_ROMS = ${JSON.stringify(shelf.map((c) => c.romB64))};`,
  `const SHELF = ${JSON.stringify(meta)}.map((cart, i) => ({ ...cart, rom: Uint8Array.from(atob(SHELF_ROMS[i]), (c) => c.charCodeAt(0)) }));`,
].join("\n");

let html = text("web/src/index.html")
  .replace("@BLARGG_PASS@", String(score.blargg[0]))
  .replace("@BLARGG_TOTAL@", String(score.blargg[1]))
  .replace("@MOONEYE_PASS@", String(score.mooneye[0]))
  .replace("@MOONEYE_TOTAL@", String(score.mooneye[1]))
  .replace("@ACID_PASS@", String(score["dmg-acid2"][0]))
  .replace("@ACID_TOTAL@", String(score["dmg-acid2"][1]))
  .replace("@GAMBATTE_PASS@", score.gambatte[0].toLocaleString("en-US"))
  .replace("@GAMBATTE_TOTAL@", score.gambatte[1].toLocaleString("en-US"))
  .replace("@MEALY_PASS@", String(score.mealybug[0]))
  .replace("@MEALY_TOTAL@", String(score.mealybug[1]));
const marker = "/*@BUNDLE@*/";
if (!html.includes(marker)) throw new Error("index.html lost its bundle marker");
// Function replacement: bundle contents may contain `$&`-style sequences.
html = html.replace(marker, () => `${lib}\n${prelude}\n${app}`);

mkdirSync(join(root, "dist"), { recursive: true });
const head = '<!doctype html>\n<html lang="en">\n<meta charset="utf-8">\n<meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover">\n';
writeFileSync(join(root, "dist/matcha.html"), head + html);
writeFileSync(join(root, "dist/matcha.fragment.html"), html);
const kb = (n) => `${(n / 1024).toFixed(0)} KiB`;
console.log(`dist/matcha.html ${kb(Buffer.byteLength(head + html))} (wasm ${kb(read(wasmPath).length)}, ${shelf.length} cartridges)`);
