#!/usr/bin/env node
// Build a self-contained, host-neutral MCP package. No npm install is needed.
import { cpSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, renameSync, rmSync, writeFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { tmpdir } from "node:os";
import { dirname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";
import { execFileSync } from "node:child_process";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const staging = mkdtempSync(join(tmpdir(), "matcha-ai-build-"));
const bundle = join(staging, "matcha-ai");
const copy = (source, destination) => cpSync(join(root, source), join(bundle, destination), { recursive: true });

try {
  mkdirSync(join(bundle, "server", "roms"), { recursive: true });
  for (const name of ["README.md", "AI_GUIDE.md", "configure.mjs", "package.json"]) copy(`mcp/${name}`, name);
  copy("mcp/server.mjs", "server/server.mjs");
  copy("web/matcha.js", "server/matcha.js");
  copy("web/matcha.wasm", "server/matcha.wasm");
  const shelf = JSON.parse(readFileSync(join(root, "web/roms/shelf.json"), "utf8"));
  for (const cart of shelf) copy(`web/roms/${cart.file}`, `server/roms/${cart.file}`);
  copy("web/roms/LICENSES.md", "server/roms/LICENSES.md");
  copy("plugin/skills", "skills");
  for (const name of ["LICENSE-MIT", "LICENSE-APACHE"]) copy(name, name);

  const files = [];
  function collect(dir) {
    for (const entry of readdirSync(dir, { withFileTypes: true })) {
      const path = join(dir, entry.name);
      if (entry.isDirectory()) collect(path);
      else files.push(path);
    }
  }
  collect(bundle);
  const checksums = files.sort().map((path) => {
    const hash = createHash("sha256").update(readFileSync(path)).digest("hex");
    return `${hash}  ${relative(bundle, path).split("\\").join("/")}`;
  });
  writeFileSync(join(bundle, "SHA256SUMS"), `${checksums.join("\n")}\n`);

  const archive = join(staging, "matcha-ai.zip");
  execFileSync("zip", ["-qr", archive, "matcha-ai"], { cwd: staging, timeout: 30_000 });
  mkdirSync(join(root, "dist"), { recursive: true });
  // Copy to the destination filesystem before the atomic replacement.
  const pending = join(root, "dist", ".matcha-ai.zip.pending");
  try {
    cpSync(archive, pending);
    renameSync(pending, join(root, "dist", "matcha-ai.zip"));
  } finally {
    rmSync(pending, { force: true });
  }
  console.log(`dist/matcha-ai.zip (${files.length + 1} files; server, guide, settings helper, skills, ${shelf.length} sample ROMs)`);
} finally {
  rmSync(staging, { recursive: true, force: true });
}
