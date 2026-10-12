// Retain the tested build outputs and record their exact source and bytes.
import { copyFileSync, existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { join } from "node:path";

const output = process.argv[2] ?? "release";
if (existsSync(output)) throw new Error("Choose a new release output directory");
const version = JSON.parse(readFileSync("plugin/.claude-plugin/plugin.json", "utf8")).version;
const sourceCommit = execFileSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).trim();
const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
mkdirSync(output);
for (const [source, name] of [["dist/matcha.plugin", "matcha.plugin"], ["dist/matcha.html", "matcha.html"],
  ["web/roms/LICENSES.md", "HOMEBREW-LICENSES.md"]]) copyFileSync(source, join(output, name));
const names = ["matcha.plugin", "matcha.html", "HOMEBREW-LICENSES.md"];
const manifest = { version, sourceCommit, node: process.version,
  wasmSha256: sha256(readFileSync("web/matcha.wasm")),
  files: names.map((name) => ({ name, sha256: sha256(readFileSync(join(output, name))) })) };
writeFileSync(join(output, "release-manifest.json"), JSON.stringify(manifest, null, 2) + "\n");
names.push("release-manifest.json");
writeFileSync(join(output, "SHA256SUMS"), names.map((name) => `${sha256(readFileSync(join(output, name)))}  ${name}\n`).join(""));
console.log(`Prepared matcha ${version} from ${sourceCommit}: ${output}`);
