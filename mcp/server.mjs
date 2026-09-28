#!/usr/bin/env node
// matcha MCP server: gives an AI agent a Game Boy it can see, play and debug.
//
// Zero dependencies (ADR-0007): speaks MCP (JSON-RPC 2.0 over newline-delimited
// stdio) directly and runs the same matcha.wasm core as the web player.
// Everything is deterministic, so an agent can save a state, try something,
// and reload to try again.
//
//   node server.mjs            # stdio MCP server
//   node server.mjs --self-test <rom>   # smoke-test the tools without a client

import { readFileSync, existsSync } from "node:fs";
import { resolve, dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { createInterface } from "node:readline";
import { deflateSync } from "node:zlib";

const HERE = dirname(fileURLToPath(import.meta.url));
// Packaged plugin: matcha.js/.wasm sit next to this file. Repo checkout: ../web.
const CORE_DIR = existsSync(join(HERE, "matcha.wasm")) ? HERE : join(HERE, "..", "web");
const SAMPLE_DIR = existsSync(join(HERE, "roms")) ? join(HERE, "roms") : join(HERE, "..", "web", "roms");
/** Open-source homebrew bundled with the plugin (see roms/LICENSES.md). */
const SAMPLES = {
  "tobu": { file: "tobu.gb", about: "Tobu Tobu Girl — arcade platformer (Tangram Games, MIT / CC BY 4.0)" },
  "libbet": { file: "libbet.gb", about: "Libbet and the Magic Floor — puzzle (Damian Yerrick, zlib)" },
  "shock-lobster": { file: "shocklobster.gb", about: "Shock Lobster — endless runner (Dave VanEe, zlib)" },
  "2048": { file: "2048.gb", about: "2048 — sliding tiles (Sanqui, zlib)" },
};
const { loadMatcha, buttonMask, PALETTES, WIDTH, HEIGHT } = await import(pathToFileURL(join(CORE_DIR, "matcha.js")).href);
const matcha = await loadMatcha(readFileSync(join(CORE_DIR, "matcha.wasm")));

const SERVER_INFO = { name: "matcha", title: "matcha Game Boy", version: "0.1.0" };
const PROTOCOLS = ["2025-11-25", "2025-06-18", "2025-03-26", "2024-11-05"];
const INSTRUCTIONS = `matcha is a cycle-accurate Game Boy (DMG) emulator.
Start with load_rom. Use press to tap buttons and see the result, run to let
time pass, and screenshot to look. Emulation is deterministic: save_state
before risky moves and load_state to retry. For debugging use state,
disassemble, breakpoint, step, read_memory and serial_output. find_value
locates where a game stores a number (lives, HP, score).`;

// --- helpers ---------------------------------------------------------------------

const hex2 = (v) => v.toString(16).padStart(2, "0");
const hex4 = (v) => v.toString(16).padStart(4, "0");

class ToolError extends Error {}

function addr(value, name = "address") {
  if (typeof value === "number" && Number.isInteger(value) && value >= 0 && value <= 0xffff) return value;
  if (typeof value === "string") {
    const t = value.trim().replace(/^\$|^0x/i, "");
    if (/^[0-9a-f]{1,4}$/i.test(t)) return parseInt(t, 16);
  }
  throw new ToolError(`${name} must be 0..0xFFFF (number, "0xC0A0" or "$C0A0"), got ${JSON.stringify(value)}`);
}

function int(value, name, min, max, fallback) {
  if (value === undefined || value === null) return fallback;
  if (!Number.isInteger(value) || value < min || value > max) throw new ToolError(`${name} must be an integer ${min}..${max}`);
  return value;
}

const crcTable = new Uint32Array(256).map((_, n) => {
  let c = n;
  for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
  return c >>> 0;
});

function crc32(buf) {
  let c = 0xffffffff;
  for (const b of buf) c = crcTable[(c ^ b) & 0xff] ^ (c >>> 8);
  return (c ^ 0xffffffff) >>> 0;
}

/** Encodes RGBA pixels as PNG, scaled by an integer factor (nearest neighbour). */
function png(rgba, w, h, scale = 1) {
  const W = w * scale;
  const H = h * scale;
  const raw = Buffer.alloc((W * 4 + 1) * H);
  for (let y = 0; y < H; y++) {
    const row = y * (W * 4 + 1);
    const sy = Math.floor(y / scale);
    for (let x = 0; x < W; x++) {
      const s = (sy * w + Math.floor(x / scale)) * 4;
      const d = row + 1 + x * 4;
      raw[d] = rgba[s];
      raw[d + 1] = rgba[s + 1];
      raw[d + 2] = rgba[s + 2];
      raw[d + 3] = 255;
    }
  }
  const chunk = (type, data) => {
    const len = Buffer.alloc(4);
    len.writeUInt32BE(data.length);
    const body = Buffer.concat([Buffer.from(type, "ascii"), data]);
    const crc = Buffer.alloc(4);
    crc.writeUInt32BE(crc32(body));
    return Buffer.concat([len, body, crc]);
  };
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(W, 0);
  ihdr.writeUInt32BE(H, 4);
  ihdr[8] = 8; // bit depth
  ihdr[9] = 6; // RGBA
  return Buffer.concat([
    Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]),
    chunk("IHDR", ihdr),
    chunk("IDAT", deflateSync(raw)),
    chunk("IEND", Buffer.alloc(0)),
  ]);
}

// --- emulator session ------------------------------------------------------------------

const session = {
  gb: null,
  romPath: "",
  header: null,
  slots: new Map(),
  /** (address << 2 | kind index) -> kind, for listing. */
  breaks: new Map(),
  search: null,
  palette: "grey",
};

const BREAK_KINDS = ["exec", "write", "read"];

function need() {
  if (!session.gb) throw new ToolError("No ROM loaded. Call load_rom first.");
  return session.gb;
}

function image(gb, scale = 2) {
  return { type: "image", data: png(gb.frameRGBA(), WIDTH, HEIGHT, scale).toString("base64"), mimeType: "image/png" };
}

function statusLine(gb) {
  const s = gb.snapshot();
  return `frame ${s.frames} · PC $${hex4(s.pc)} · LY ${s.ppu.ly} · ${s.power}`;
}

/** Runs `frames` frames holding `mask`; stops early at a break. */
function runFrames(gb, frames, mask) {
  gb.setButtons(mask);
  for (let i = 0; i < frames; i++) {
    const ev = gb.runFrame();
    gb.takeAudio();
    if (ev.event === "breakpoint") return `stopped at breakpoint $${hex4(ev.pc)} after ${i} frames`;
    if (ev.event === "watchpoint") {
      return `stopped after ${i} frames: instruction at $${hex4(ev.pc)} ${ev.write ? "wrote" : "read"} $${hex2(ev.value)} ${ev.write ? "to" : "from"} $${hex4(ev.addr)}`;
    }
  }
  return null;
}

function describeState(gb) {
  const s = gb.snapshot();
  const f = s.flags;
  const p = s.ppu;
  return [
    `AF=${hex4(s.af)} BC=${hex4(s.bc)} DE=${hex4(s.de)} HL=${hex4(s.hl)} SP=${hex4(s.sp)} PC=${hex4(s.pc)}`,
    `flags ${f.z ? "Z" : "-"}${f.n ? "N" : "-"}${f.h ? "H" : "-"}${f.c ? "C" : "-"}  IME=${s.ime ? 1 : 0}  CPU ${s.power}  IE=${hex2(s.ie)} IF=${hex2(s.if)}  ROM bank ${s.romBank}`,
    `PPU LCDC=${hex2(p.lcdc)} STAT=${hex2(p.stat)} LY=${p.ly} LYC=${p.lyc} SCX=${p.scx} SCY=${p.scy} WX=${p.wx} WY=${p.wy} BGP=${hex2(p.bgp)} (line ${p.line}, dot ${p.dot})`,
    `frame ${s.frames} · ${s.cycles.toLocaleString("en-US")} M-cycles`,
  ].join("\n");
}

function listing(gb, start, count) {
  const pc = gb.snapshot().pc;
  return gb
    .disassemble(start, count)
    .map((i) => `${i.addr === pc ? "→" : " "} ${hex4(i.addr)}  ${i.bytes.map(hex2).join(" ").padEnd(8)}  ${i.text}`)
    .join("\n");
}

const SEARCH_RANGES = [[0xc000, 0x2000], [0xa000, 0x2000], [0xff80, 0x7f]];

function searchSnapshot(gb) {
  const values = new Map();
  for (const [start, len] of SEARCH_RANGES) {
    if (start === 0xa000 && !session.header?.ramSize) continue;
    gb.peekRange(start, len).forEach((v, i) => values.set(start + i, v));
  }
  return values;
}

// --- tools -----------------------------------------------------------------------------

const buttonsSchema = {
  type: "array",
  items: { type: "string", enum: ["a", "b", "start", "select", "up", "down", "left", "right"] },
  description: "Buttons to hold, e.g. [\"a\"] or [\"right\", \"b\"].",
};

const TOOLS = [
  {
    name: "load_rom",
    title: "Load ROM",
    description: `Load a Game Boy ROM (.gb) and power on. Give a file path, or a bundled open-source sample: ${Object.entries(SAMPLES).map(([k, v]) => `"${k}" (${v.about})`).join("; ")}. Resets save slots and searches. Returns the cartridge header and the first screen.`,
    inputSchema: {
      type: "object",
      properties: {
        path: { type: "string", description: "Path to the .gb file (absolute, or relative to the server's working directory)." },
        sample: { type: "string", enum: Object.keys(SAMPLES), description: "Load a bundled sample instead of a file." },
        frames: { type: "integer", minimum: 0, maximum: 3600, description: "Frames to run before the screenshot (default 60 ≈ 1 s)." },
      },
    },
    run({ path, sample, frames }) {
      if (sample !== undefined && !(sample in SAMPLES)) throw new ToolError(`sample must be one of ${Object.keys(SAMPLES).join(", ")}`);
      if (!sample && (typeof path !== "string" || !path)) throw new ToolError("give a path to a .gb file, or a sample name");
      const full = sample ? join(SAMPLE_DIR, SAMPLES[sample].file) : resolve(process.cwd(), path);
      let bytes;
      try {
        bytes = readFileSync(full);
      } catch (e) {
        throw new ToolError(`cannot read ${full}: ${e.code ?? e.message}`);
      }
      const gb = matcha.create(bytes);
      session.gb?.destroy();
      Object.assign(session, { gb, romPath: full, header: gb.header(), slots: new Map(), breaks: new Map(), search: null });
      gb.setPalette(PALETTES[session.palette]);
      runFrames(gb, int(frames, "frames", 0, 3600, 60), 0);
      const h = session.header;
      const text = [
        `Loaded ${full}`,
        `title "${h.title}" · ${h.cartTypeName} · ROM ${h.romSize / 1024} KiB · RAM ${h.ramSize / 1024} KiB${h.battery ? " (battery)" : ""}${h.rtc ? " · RTC" : ""}`,
        h.cgbFlag === 0xc0 ? "Warning: CGB-only cartridge; matcha emulates the original DMG, so it may refuse to run." : "",
        statusLine(gb),
      ].filter(Boolean).join("\n");
      return [{ type: "text", text }, image(gb)];
    },
  },
  {
    name: "press",
    title: "Press buttons",
    description: "Hold buttons for a few frames, release, let the game react, then return a screenshot. The basic way to play.",
    inputSchema: {
      type: "object",
      properties: {
        buttons: buttonsSchema,
        hold_frames: { type: "integer", minimum: 1, maximum: 600, description: "Frames to hold (default 6)." },
        wait_frames: { type: "integer", minimum: 0, maximum: 3600, description: "Frames to run after releasing (default 30)." },
        screenshot: { type: "boolean", description: "Return a screenshot (default true)." },
      },
      required: ["buttons"],
    },
    run({ buttons, hold_frames, wait_frames, screenshot = true }) {
      const gb = need();
      const mask = buttonMask(buttons ?? []);
      const stop = runFrames(gb, int(hold_frames, "hold_frames", 1, 600, 6), mask) ?? runFrames(gb, int(wait_frames, "wait_frames", 0, 3600, 30), 0);
      gb.setButtons(0);
      const out = [{ type: "text", text: `${stop ? `${stop}\n` : ""}${statusLine(gb)}` }];
      if (screenshot) out.push(image(gb));
      return out;
    },
  },
  {
    name: "run",
    title: "Run",
    description: "Let emulated time pass (optionally holding buttons). Stops early at a breakpoint or watchpoint.",
    inputSchema: {
      type: "object",
      properties: {
        frames: { type: "integer", minimum: 1, maximum: 36000, description: "Frames to run (default 60; 59.73 frames = 1 s)." },
        buttons: buttonsSchema,
        screenshot: { type: "boolean", description: "Return a screenshot (default true)." },
      },
    },
    run({ frames, buttons, screenshot = true }) {
      const gb = need();
      const stop = runFrames(gb, int(frames, "frames", 1, 36000, 60), buttonMask(buttons ?? []));
      gb.setButtons(0);
      const out = [{ type: "text", text: `${stop ?? "ran to completion"}\n${statusLine(gb)}` }];
      if (screenshot) out.push(image(gb));
      return out;
    },
  },
  {
    name: "screenshot",
    title: "Screenshot",
    description: "Return the current screen as a PNG (160x144 pixels times scale).",
    annotations: { readOnlyHint: true },
    inputSchema: {
      type: "object",
      properties: {
        scale: { type: "integer", minimum: 1, maximum: 6, description: "Integer zoom (default 3)." },
        palette: { type: "string", enum: Object.keys(PALETTES), description: "Colour palette (default grey, the clearest for reading text)." },
      },
    },
    run({ scale, palette }) {
      const gb = need();
      if (palette !== undefined) {
        if (!(palette in PALETTES)) throw new ToolError(`palette must be one of ${Object.keys(PALETTES).join(", ")}`);
        session.palette = palette;
        gb.setPalette(PALETTES[palette]);
      }
      return [image(gb, int(scale, "scale", 1, 6, 3)), { type: "text", text: statusLine(gb) }];
    },
  },
  {
    name: "state",
    title: "Machine state",
    description: "CPU registers and flags, interrupt state, PPU registers and position, ROM bank, frame and cycle counters.",
    annotations: { readOnlyHint: true },
    inputSchema: { type: "object", properties: {} },
    run() {
      return [{ type: "text", text: describeState(need()) }];
    },
  },
  {
    name: "read_memory",
    title: "Read memory",
    description: "Hex dump of the CPU address space (ROM, VRAM $8000, cart RAM $A000, WRAM $C000, OAM $FE00, I/O $FF00, HRAM $FF80). No side effects.",
    annotations: { readOnlyHint: true },
    inputSchema: {
      type: "object",
      properties: {
        address: { type: ["string", "integer"], description: "Start address, e.g. \"C000\" or 49152." },
        length: { type: "integer", minimum: 1, maximum: 4096, description: "Bytes to read (default 64)." },
      },
      required: ["address"],
    },
    run({ address, length }) {
      const gb = need();
      const start = addr(address);
      const n = int(length, "length", 1, 4096, 64);
      const bytes = gb.peekRange(start, n);
      const lines = [];
      for (let i = 0; i < n; i += 16) {
        const row = Array.from(bytes.subarray(i, i + 16));
        const ascii = row.map((b) => (b >= 0x20 && b < 0x7f ? String.fromCharCode(b) : ".")).join("");
        lines.push(`${hex4((start + i) & 0xffff)}  ${row.map(hex2).join(" ").padEnd(47)}  ${ascii}`);
      }
      return [{ type: "text", text: lines.join("\n") }];
    },
  },
  {
    name: "write_memory",
    title: "Write memory",
    description: "Write bytes as the CPU would (writes to $0000-$7FFF go to the cartridge mapper). Use for cheats and experiments.",
    annotations: { destructiveHint: true },
    inputSchema: {
      type: "object",
      properties: {
        address: { type: ["string", "integer"], description: "Start address." },
        bytes: {
          description: "Values to write: an array of 0-255 integers or a hex string like \"3c00ff\".",
          anyOf: [{ type: "array", items: { type: "integer", minimum: 0, maximum: 255 } }, { type: "string" }],
        },
      },
      required: ["address", "bytes"],
    },
    run({ address, bytes }) {
      const gb = need();
      const start = addr(address);
      let values = bytes;
      if (typeof bytes === "string") {
        const t = bytes.replace(/\s+/g, "");
        if (!/^([0-9a-f]{2})+$/i.test(t)) throw new ToolError("hex string must have an even number of hex digits");
        values = t.match(/../g).map((b) => parseInt(b, 16));
      }
      if (!Array.isArray(values) || !values.length || values.length > 4096 || values.some((v) => !Number.isInteger(v) || v < 0 || v > 255)) {
        throw new ToolError("bytes must be 1..4096 values in 0..255");
      }
      values.forEach((v, i) => gb.poke(start + i, v));
      return [{ type: "text", text: `wrote ${values.length} byte(s) at $${hex4(start)}` }];
    },
  },
  {
    name: "disassemble",
    title: "Disassemble",
    description: "Disassemble instructions (RGBDS syntax). Defaults to the current PC, marked with →.",
    annotations: { readOnlyHint: true },
    inputSchema: {
      type: "object",
      properties: {
        address: { type: ["string", "integer"], description: "Start address (default PC)." },
        count: { type: "integer", minimum: 1, maximum: 200, description: "Instructions (default 20)." },
      },
    },
    run({ address, count }) {
      const gb = need();
      const start = address === undefined ? gb.snapshot().pc : addr(address);
      return [{ type: "text", text: listing(gb, start, int(count, "count", 1, 200, 20)) }];
    },
  },
  {
    name: "breakpoint",
    title: "Breakpoints and watchpoints",
    description: "Add/remove/list/clear breakpoints. kind \"exec\" stops before the instruction at address runs; \"write\"/\"read\" stop after an instruction touches the address. run/press stop when one hits.",
    inputSchema: {
      type: "object",
      properties: {
        action: { type: "string", enum: ["add", "remove", "list", "clear"] },
        address: { type: ["string", "integer"] },
        kind: { type: "string", enum: ["exec", "write", "read"], description: "Default exec." },
      },
      required: ["action"],
    },
    run({ action, address, kind = "exec" }) {
      const gb = need();
      if (action === "list") {
        const rows = [...session.breaks].map(([key, k]) => `${k} $${hex4(key >> 2)}`);
        return [{ type: "text", text: rows.length ? rows.join("\n") : "no breakpoints" }];
      }
      if (action === "clear") {
        gb.clearBreakpoints();
        gb.clearWatchpoints();
        session.breaks.clear();
        return [{ type: "text", text: "cleared all breakpoints and watchpoints" }];
      }
      if (!BREAK_KINDS.includes(kind)) throw new ToolError("kind must be exec, write or read");
      const a = addr(address);
      const key = (a << 2) | BREAK_KINDS.indexOf(kind);
      if (action === "add") {
        if (kind === "exec") gb.addBreakpoint(a);
        else gb.addWatchpoint(a, kind === "write");
        session.breaks.set(key, kind);
        return [{ type: "text", text: `${kind} breakpoint at $${hex4(a)}` }];
      }
      if (action === "remove") {
        if (kind === "exec") gb.removeBreakpoint(a);
        else gb.removeWatchpoint(a, kind === "write");
        const had = session.breaks.delete(key);
        return [{ type: "text", text: had ? `removed ${kind} breakpoint at $${hex4(a)}` : `no ${kind} breakpoint at $${hex4(a)}` }];
      }
      throw new ToolError("action must be add, remove, list or clear");
    },
  },
  {
    name: "step",
    title: "Step instructions",
    description: "Execute instructions one at a time (an interrupt dispatch or a halted cycle also counts as one step). Returns state and the code at PC.",
    inputSchema: {
      type: "object",
      properties: { count: { type: "integer", minimum: 1, maximum: 100000, description: "Steps (default 1)." } },
    },
    run({ count }) {
      const gb = need();
      const n = int(count, "count", 1, 100000, 1);
      const kinds = new Map();
      for (let i = 0; i < n; i++) {
        const k = gb.step();
        kinds.set(k, (kinds.get(k) ?? 0) + 1);
      }
      const summary = [...kinds].map(([k, v]) => `${v} ${k}`).join(", ");
      return [{ type: "text", text: `${summary}\n\n${describeState(gb)}\n\n${listing(gb, gb.snapshot().pc, 8)}` }];
    },
  },
  {
    name: "save_state",
    title: "Save state",
    description: "Snapshot the whole machine into a named in-memory slot. Emulation is deterministic, so load_state + the same inputs replays exactly.",
    inputSchema: {
      type: "object",
      properties: { slot: { type: "string", description: "Slot name (default \"quick\")." } },
    },
    run({ slot = "quick" }) {
      const gb = need();
      session.slots.set(String(slot), gb.saveState());
      return [{ type: "text", text: `saved "${slot}" at ${statusLine(gb)} · slots: ${[...session.slots.keys()].join(", ")}` }];
    },
  },
  {
    name: "load_state",
    title: "Load state",
    description: "Restore a named slot made by save_state and return the screen.",
    inputSchema: {
      type: "object",
      properties: { slot: { type: "string", description: "Slot name (default \"quick\")." } },
    },
    run({ slot = "quick" }) {
      const gb = need();
      const state = session.slots.get(String(slot));
      if (!state) throw new ToolError(`no slot "${slot}" (have: ${[...session.slots.keys()].join(", ") || "none"})`);
      gb.loadState(state);
      return [{ type: "text", text: `loaded "${slot}" · ${statusLine(gb)}` }, image(gb)];
    },
  },
  {
    name: "find_value",
    title: "Find a value in RAM",
    description: "Cheat-finder over WRAM, cartridge RAM and HRAM. action \"start\" snapshots memory; then play and call action \"filter\" with op eq/ne (vs value) or changed/same/up/down (vs the previous snapshot) until a few addresses remain.",
    inputSchema: {
      type: "object",
      properties: {
        action: { type: "string", enum: ["start", "filter"] },
        op: { type: "string", enum: ["eq", "ne", "changed", "same", "up", "down"] },
        value: { type: "integer", minimum: 0, maximum: 255 },
      },
      required: ["action"],
    },
    run({ action, op, value }) {
      const gb = need();
      const now = searchSnapshot(gb);
      if (action === "start") {
        session.search = { candidates: [...now.keys()], prev: now };
      } else if (action === "filter") {
        if (!session.search) throw new ToolError("call find_value with action \"start\" first");
        if (!["eq", "ne", "changed", "same", "up", "down"].includes(op)) throw new ToolError("op must be eq, ne, changed, same, up or down");
        if ((op === "eq" || op === "ne") && !Number.isInteger(value)) throw new ToolError(`op ${op} needs a value 0..255`);
        const prev = session.search.prev;
        const keep = (a) => {
          const before = prev.get(a);
          const after = now.get(a);
          switch (op) {
            case "eq": return after === value;
            case "ne": return after !== value;
            case "changed": return after !== before;
            case "same": return after === before;
            case "up": return after > before;
            default: return after < before;
          }
        };
        session.search = { candidates: session.search.candidates.filter(keep), prev: now };
      } else {
        throw new ToolError("action must be start or filter");
      }
      const c = session.search.candidates;
      const shown = c.slice(0, 32).map((a) => `$${hex4(a)}=${now.get(a)}`).join("  ");
      return [{ type: "text", text: `${c.length} candidate address(es)${c.length ? `: ${shown}${c.length > 32 ? " …" : ""}` : ""}` }];
    },
  },
  {
    name: "serial_output",
    title: "Serial output",
    description: "Text the ROM sent over the link port. Test ROMs (e.g. Blargg's) print their results here.",
    annotations: { readOnlyHint: true },
    inputSchema: { type: "object", properties: {} },
    run() {
      const text = need().serialText();
      return [{ type: "text", text: text || "(nothing sent over the link port)" }];
    },
  },
  {
    name: "profile",
    title: "CPU profile",
    description: "action \"start\" begins counting; \"report\" shows CPU utilisation, interrupt counts and the hottest opcodes since start.",
    annotations: { readOnlyHint: true },
    inputSchema: {
      type: "object",
      properties: { action: { type: "string", enum: ["start", "report"] } },
      required: ["action"],
    },
    run({ action }) {
      const gb = need();
      if (action === "start") {
        gb.setProfiling(true);
        return [{ type: "text", text: "profiling started; run the game, then call profile with action \"report\"" }];
      }
      const p = gb.profile();
      if (!p) throw new ToolError("profiling is off; call profile with action \"start\" first");
      const total = p.busy + p.halted + p.interrupt + p.stopped || 1;
      const pct = (v) => `${((v / total) * 100).toFixed(1)}%`;
      const hot = p.opcodes
        .map((n, op) => [n, op])
        .filter(([n, op]) => n && op !== 0xcb)
        .sort((a, b) => b[0] - a[0])
        .slice(0, 12)
        .map(([n, op]) => `  ${hex2(op)} ${gb.opcodeText(op).padEnd(18)} ${((n / p.instructions) * 100).toFixed(1)}%`);
      const names = ["VBlank", "STAT", "Timer", "Serial", "Joypad"];
      const text = [
        `${p.instructions.toLocaleString("en-US")} instructions; CPU busy ${pct(p.busy)}, in interrupt dispatch ${pct(p.interrupt)}, halted ${pct(p.halted + p.stopped)}`,
        `interrupts: ${p.interrupts.map((n, i) => `${names[i]} ${n}`).join(", ")}`,
        `ROM bytes where instructions started: ${p.covered.toLocaleString("en-US")}`,
        "hottest opcodes:",
        ...hot,
      ].join("\n");
      return [{ type: "text", text }];
    },
  },
  {
    name: "tiles",
    title: "VRAM tiles",
    description: "Image of all 384 tiles in VRAM (16 per row), shaded through BGP. Useful for checking graphics loading.",
    annotations: { readOnlyHint: true },
    inputSchema: { type: "object", properties: { scale: { type: "integer", minimum: 1, maximum: 4 } } },
    run({ scale }) {
      const gb = need();
      return [{ type: "image", data: png(gb.tilesRGBA(), 128, 192, int(scale, "scale", 1, 4, 2)).toString("base64"), mimeType: "image/png" }];
    },
  },
  {
    name: "reset",
    title: "Reset",
    description: "Power-cycle the Game Boy. Cartridge RAM (save data), breakpoints and save slots survive.",
    inputSchema: { type: "object", properties: {} },
    run() {
      const gb = need();
      gb.reset();
      return [{ type: "text", text: `reset · ${statusLine(gb)}` }];
    },
  },
];

const TOOL_MAP = new Map(TOOLS.map((t) => [t.name, t]));

// --- JSON-RPC over stdio --------------------------------------------------------------

function send(message) {
  process.stdout.write(`${JSON.stringify(message)}\n`);
}

function callTool(name, args) {
  const tool = TOOL_MAP.get(name);
  if (!tool) return { content: [{ type: "text", text: `unknown tool "${name}"` }], isError: true };
  try {
    return { content: tool.run(args ?? {}) };
  } catch (e) {
    // ToolErrors are expected input problems; anything else is logged for debugging.
    if (!(e instanceof ToolError)) console.error("[matcha]", e);
    return { content: [{ type: "text", text: e.message }], isError: true };
  }
}

function handle(msg) {
  const { id, method, params } = msg;
  const reply = (result) => id !== undefined && send({ jsonrpc: "2.0", id, result });
  const fail = (code, message) => id !== undefined && send({ jsonrpc: "2.0", id, error: { code, message } });
  switch (method) {
    case "initialize": {
      const asked = params?.protocolVersion;
      reply({
        protocolVersion: PROTOCOLS.includes(asked) ? asked : PROTOCOLS[0],
        capabilities: { tools: { listChanged: false } },
        serverInfo: SERVER_INFO,
        instructions: INSTRUCTIONS,
      });
      return;
    }
    case "ping":
      reply({});
      return;
    case "tools/list":
      reply({ tools: TOOLS.map(({ run, ...def }) => def) });
      return;
    case "tools/call":
      reply(callTool(params?.name, params?.arguments));
      return;
    default:
      if (method?.startsWith("notifications/")) return; // no response to notifications
      fail(-32601, `method not found: ${method}`);
  }
}

async function serve() {
  const rl = createInterface({ input: process.stdin, crlfDelay: Infinity });
  for await (const line of rl) {
    if (!line.trim()) continue;
    let msg;
    try {
      msg = JSON.parse(line);
    } catch {
      send({ jsonrpc: "2.0", id: null, error: { code: -32700, message: "parse error" } });
      continue;
    }
    for (const m of Array.isArray(msg) ? msg : [msg]) {
      try {
        handle(m);
      } catch (e) {
        console.error("[matcha] internal error", e);
        if (m?.id !== undefined) send({ jsonrpc: "2.0", id: m.id, error: { code: -32603, message: String(e.message ?? e) } });
      }
    }
  }
}

// --- self-test ------------------------------------------------------------------------

function selfTest(rom) {
  const show = (name, args) => {
    const res = callTool(name, args);
    const text = res.content.filter((c) => c.type === "text").map((c) => c.text).join(" | ");
    const images = res.content.filter((c) => c.type === "image").length;
    console.log(`${res.isError ? "✗" : "✓"} ${name.padEnd(13)} ${text.split("\n")[0].slice(0, 100)}${images ? `  [+${images} image]` : ""}`);
    return res;
  };
  show("state", {});
  show("load_rom", { path: rom });
  show("press", { buttons: ["start"] });
  show("run", { frames: 120 });
  show("save_state", { slot: "a" });
  show("find_value", { action: "start" });
  show("run", { frames: 30, screenshot: false });
  show("find_value", { action: "filter", op: "changed" });
  show("disassemble", { count: 3 });
  show("read_memory", { address: "C000", length: 32 });
  show("breakpoint", { action: "add", address: session.gb.snapshot().pc });
  show("step", { count: 5 });
  show("profile", { action: "start" });
  show("run", { frames: 60, screenshot: false });
  show("profile", { action: "report" });
  show("load_state", { slot: "a" });
  show("tiles", {});
  show("serial_output", {});
  show("write_memory", { address: "zz", bytes: [1] });
  show("breakpoint", { action: "clear" });
  show("reset", {});
}

if (process.argv[2] === "--self-test") {
  if (!process.argv[3]) {
    console.error("usage: node server.mjs --self-test <rom.gb>");
    process.exit(2);
  }
  selfTest(process.argv[3]);
} else {
  serve();
}
