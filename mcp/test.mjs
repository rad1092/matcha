#!/usr/bin/env node
// End-to-end test of the MCP server over real stdio JSON-RPC.
//   node mcp/test.mjs [rom.gb]     (defaults to web/roms/libbet.gb)

import { spawn } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { createInterface } from "node:readline";
import assert from "node:assert/strict";
import { inflateSync } from "node:zlib";

const here = dirname(fileURLToPath(import.meta.url));
const rom = process.argv[2] ?? join(here, "..", "web", "roms", "libbet.gb");

const server = spawn(process.execPath, [join(here, "server.mjs")], { stdio: ["pipe", "pipe", "inherit"] });
const lines = createInterface({ input: server.stdout });
const pending = new Map();
let nextId = 1;
lines.on("line", (line) => {
  const msg = JSON.parse(line);
  const resolve = pending.get(msg.id);
  if (resolve) {
    pending.delete(msg.id);
    resolve(msg);
  }
});

function request(method, params) {
  const id = nextId++;
  server.stdin.write(`${JSON.stringify({ jsonrpc: "2.0", id, method, params })}\n`);
  return new Promise((resolve) => pending.set(id, resolve));
}

const call = async (name, args = {}) => (await request("tools/call", { name, arguments: args })).result;
const textOf = (result) => result.content.filter((c) => c.type === "text").map((c) => c.text).join("\n");

/** Checks a PNG's signature and IHDR size, and that it inflates. */
function checkPng(b64, width, height) {
  const buf = Buffer.from(b64, "base64");
  assert.deepEqual([...buf.subarray(0, 8)], [137, 80, 78, 71, 13, 10, 26, 10], "PNG signature");
  assert.equal(buf.readUInt32BE(16), width);
  assert.equal(buf.readUInt32BE(20), height);
  const idatAt = buf.indexOf("IDAT");
  const idatLen = buf.readUInt32BE(idatAt - 4);
  const raw = inflateSync(buf.subarray(idatAt + 4, idatAt + 4 + idatLen));
  assert.equal(raw.length, (width * 4 + 1) * height, "decoded image size");
}

const init = await request("initialize", {
  protocolVersion: "2025-06-18",
  capabilities: {},
  clientInfo: { name: "matcha-test", version: "0" },
});
assert.equal(init.result.protocolVersion, "2025-06-18");
assert.equal(init.result.serverInfo.name, "matcha");
server.stdin.write(`${JSON.stringify({ jsonrpc: "2.0", method: "notifications/initialized" })}\n`);

const odd = await request("initialize", { protocolVersion: "1999-01-01", capabilities: {} });
assert.equal(odd.result.protocolVersion, "2025-11-25", "unknown versions get our latest");

const { tools } = (await request("tools/list", {})).result;
const names = tools.map((t) => t.name).sort();
assert.ok(names.includes("load_rom") && names.includes("press") && names.includes("find_value"));
for (const t of tools) assert.equal(t.inputSchema.type, "object", `${t.name} schema`);

const missing = await call("state");
assert.equal(missing.isError, true, "state before load_rom is an error result, not a crash");

const notRom = await call("load_rom", { path: fileURLToPath(import.meta.url) });
assert.equal(notRom.isError, true);
assert.match(textOf(notRom), /not a ROM matcha can run/);
const noFile = await call("load_rom", { path: join(here, "no-such.gb") });
assert.match(textOf(noFile), /cannot read .*ENOENT/);

const loaded = await call("load_rom", { path: rom, frames: 30 });
assert.ok(!loaded.isError, textOf(loaded));
checkPng(loaded.content.find((c) => c.type === "image").data, 320, 288);

const shot = await call("screenshot", { scale: 1 });
checkPng(shot.content.find((c) => c.type === "image").data, 160, 144);

await call("save_state", { slot: "x" });
const before = textOf(await call("state"));
await call("run", { frames: 90, screenshot: false });
await call("load_state", { slot: "x" });
assert.equal(textOf(await call("state")), before, "load_state restores exactly");

const bad = await call("read_memory", { address: "nope" });
assert.equal(bad.isError, true);
const dump = textOf(await call("read_memory", { address: "0134", length: 16 }));
assert.match(dump, /^0134 {2}/, "hex dump starts at the address");

const unknown = await request("does/not/exist", {});
assert.equal(unknown.error.code, -32601);

server.stdin.end();
server.kill();
console.log(`mcp: ${tools.length} tools, protocol negotiation, images, state round-trip — all ok`);
