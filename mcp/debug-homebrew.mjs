#!/usr/bin/env node
// Repeatable debugger and rejected-request checks against a real stdio server.
// Optional argument targets an extracted release/plugin server.
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { mkdtempSync, writeFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { createInterface } from "node:readline";
import { once } from "node:events";
import { homebrewRom } from "./fixtures/homebrew.mjs";

const here = dirname(fileURLToPath(import.meta.url));
const temp = mkdtempSync(join(tmpdir(), "matcha-homebrew-"));
const rom = join(temp, "debug.gb");
writeFileSync(rom, homebrewRom());
const badRom = join(temp, "invalid.gb");
writeFileSync(badRom, "not a cartridge");
const server = spawn(process.execPath, [resolve(process.argv[2] ?? join(here, "server.mjs"))], { stdio: ["pipe", "pipe", "pipe"] });
let stderr = "";
server.stderr.on("data", (data) => { stderr += data; });
const lines = createInterface({ input: server.stdout });
let id = 0;
const pending = new Map();
lines.on("line", (line) => {
  const message = JSON.parse(line);
  const done = pending.get(message.id);
  if (done) { pending.delete(message.id); done(message); }
});
const deadline = setTimeout(() => { server.kill(); }, 30_000);
function request(method, params = {}) {
  const key = ++id;
  return new Promise((done, reject) => {
    const onExit = () => reject(new Error(`server exited: ${stderr}`));
    server.once("exit", onExit);
    pending.set(key, (message) => { server.off("exit", onExit); done(message); });
    server.stdin.write(`${JSON.stringify({ jsonrpc: "2.0", id: key, method, params })}\n`);
  });
}
const call = async (name, args = {}) => (await request("tools/call", { name, arguments: args })).result;
const text = (result) => result.content.filter((c) => c.type === "text").map((c) => c.text).join("\n");
const ok = async (name, args = {}) => {
  const result = await call(name, args);
  assert.ok(!result.isError, `${name}: ${text(result)}`);
  return result;
};
const snapshot = async () => ({
  state: text(await ok("state")),
  memory: text(await ok("read_memory", { address: "c000", length: 16 })),
  screen: (await ok("screenshot", { scale: 1 })).content[0].data,
  breaks: text(await ok("breakpoint", { action: "list" })),
});

try {
  await request("initialize", { protocolVersion: "2025-11-25", capabilities: {} });
  for (let repeat = 0; repeat < 3; repeat++) {
    await ok("load_rom", { path: rom, frames: 0, ram: "zero" });
    const disassembly = text(await ok("disassemble", { address: "0150", count: 5 }));
    assert.match(disassembly, /0150/);
    assert.match(disassembly, /0159/);
    await ok("save_state", { slot: "boot" });
    await ok("breakpoint", { action: "add", address: "c000", kind: "write" });
    assert.match(text(await ok("run", { frames: 1, screenshot: false })), /instruction at \$0152 wrote \$42 to \$c000/);
    assert.match(text(await ok("read_memory", { address: "c000", length: 2 })), /c000  42 00/);
    await ok("step", { count: 2 });
    assert.match(text(await ok("read_memory", { address: "c000", length: 2 })), /c000  42 43/);
    await ok("save_state", { slot: "checkpoint" });
    await ok("find_value", { action: "start" });
    assert.match(text(await ok("find_value", { action: "filter", op: "eq", value: 67 })), /1 candidate address\(es\): \$c001=67/);
    const before = await snapshot();
    const invalid = [
      ["load_rom", { path: rom, frames: -1 }],
      ["load_rom", { path: rom, frames: 3601 }],
      ["load_rom", { path: rom, ram: "zero", seed: 7 }],
      ["load_rom", { path: badRom }],
      ["load_rom", { path: join(temp, "missing.gb") }],
      ["load_rom", { sample: "toString" }],
      ["press", { buttons: ["a"], hold_frames: 2, wait_frames: -1 }],
      ["press", { buttons: ["a"], screenshot: "false" }],
      ["press", { buttons: ["typo"] }],
      ["press", {}],
      ["screenshot", { palette: "dmg", scale: 0 }],
      ["save_state", { slot: { invalid: true } }],
      ["write_memory", { address: "c000", bytes: [1, 256] }],
      ["breakpoint", { action: "clear", kind: "typo" }],
      ["find_value", { action: "filter", op: "eq", value: 256 }],
      ["find_value", { action: "start", value: -1 }],
      ["reset", { ram: "noise", seed: -1 }],
      ["run", { frames: null }],
      ["run", { frames: 1, unexpected: true }],
      ["load_rom", null],
    ];
    for (const [name, args] of invalid) {
      const rejected = await call(name, args);
      assert.equal(rejected.isError, true, `${name} must reject ${JSON.stringify(args)}`);
      assert.deepEqual(await snapshot(), before, `${name} must preserve the whole observable session`);
      assert.match(text(await ok("find_value", { action: "filter", op: "same" })), /1 candidate address\(es\): \$c001=67/);
      await ok("load_state", { slot: "checkpoint" });
      assert.deepEqual(await snapshot(), before, `${name} must preserve the save slot`);
    }
    // A completed request cannot be cancelled retroactively. Notifications must
    // neither receive a reply nor erase an already committed session.
    server.stdin.write(`${JSON.stringify({ jsonrpc: "2.0", method: "notifications/cancelled", params: { requestId: id } })}\n`);
    await request("ping");
    assert.deepEqual(await snapshot(), before);
    await ok("write_memory", { address: "c000", bytes: "0000" });
    await ok("load_state", { slot: "boot" });
    await ok("run", { frames: 1, screenshot: false });
    await ok("step", { count: 2 });
    assert.deepEqual(await snapshot(), before, "restore + identical steps replay exactly");
  }
  // Successful replacement deliberately resets all per-ROM debugger data.
  await ok("load_rom", { path: rom, frames: 0 });
  assert.equal((await call("load_state", { slot: "checkpoint" })).isError, true);
  assert.equal(text(await ok("breakpoint", { action: "list" })), "no breakpoints");
  assert.equal((await call("find_value", { action: "filter", op: "same" })).isError, true);
  assert.equal(stderr, "", "invalid input must not become an internal server exception");
  const exit = once(server, "exit");
  server.stdin.end();
  assert.equal((await exit)[0], 0, "EOF shuts down cleanly");
  console.log("homebrew: disassembly → watchpoint → memory → save/replay; 20 rejected requests × 3 sessions preserved; replacement + EOF ok");
} finally {
  clearTimeout(deadline);
  server.kill();
  lines.close();
  rmSync(temp, { recursive: true, force: true });
}
