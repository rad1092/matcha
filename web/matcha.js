// matcha.js — JavaScript bindings for matcha.wasm (browser + Node, no deps).
//
//   import { loadMatcha, BUTTONS } from "./matcha.js";
//   const matcha = await loadMatcha(wasmBytes);
//   const gb = matcha.create(romBytes);
//   gb.runFrame();
//   ctx.putImageData(new ImageData(gb.frameRGBA(), 160, 144), 0, 0);
//
// Every buffer returned here is a copy (or a fresh view that must be used
// before the next call): WebAssembly memory may grow and detach old views.

export const WIDTH = 160;
export const HEIGHT = 144;
export const FRAME_RATE = 4194304 / 70224;

export const BUTTONS = Object.freeze({
  right: 1 << 0, left: 1 << 1, up: 1 << 2, down: 1 << 3,
  a: 1 << 4, b: 1 << 5, select: 1 << 6, start: 1 << 7,
});

export const PALETTES = Object.freeze({
  matcha: [0xE8F2C8, 0xA9C47F, 0x5E7F4A, 0x1F3326],
  dmg: [0x9BBC0F, 0x8BAC0F, 0x306230, 0x0F380F],
  grey: [0xFFFFFF, 0xAAAAAA, 0x555555, 0x000000],
  pocket: [0xC4CFA1, 0x8B956D, 0x4D533C, 0x1F1F1F],
});

const RUN_EVENTS = ["frame", "breakpoint", "watchpoint", "budget"];
const STEP_KINDS = ["instruction", "interrupt", "halted", "stopped", "locked"];
const POWER = ["running", "halted", "stopped", "locked"];

const utf8 = new TextDecoder();

/** Parses a list like ["a", "start"] or "a,start" into a button mask. */
export function buttonMask(buttons) {
  const list = Array.isArray(buttons) ? buttons : String(buttons ?? "").split(",");
  let mask = 0;
  for (const raw of list) {
    const name = String(raw).trim().toLowerCase();
    if (!name) continue;
    if (!(name in BUTTONS)) throw new Error(`unknown button "${raw}" (use: ${Object.keys(BUTTONS).join(", ")})`);
    mask |= BUTTONS[name];
  }
  return mask;
}

/** Instantiates the module from bytes, a Response, or a URL. */
export async function loadMatcha(source) {
  let bytes = source;
  if (typeof source === "string" || source instanceof URL) bytes = await fetch(source);
  if (typeof Response !== "undefined" && bytes instanceof Response) bytes = await bytes.arrayBuffer();
  const { instance } = await WebAssembly.instantiate(bytes, {});
  return new MatchaModule(instance.exports);
}

export class MatchaModule {
  constructor(exports) {
    this.x = exports;
  }

  get memory() {
    return this.x.memory.buffer;
  }

  /** Copies bytes into wasm memory; returns [ptr, len]. Caller frees. */
  put(bytes) {
    const data = bytes instanceof Uint8Array ? bytes : new Uint8Array(bytes);
    const ptr = this.x.matcha_alloc(data.length);
    new Uint8Array(this.memory, ptr, data.length).set(data);
    return [ptr, data.length];
  }

  lastError() {
    const ptr = this.x.matcha_last_error_ptr();
    const len = this.x.matcha_last_error_len();
    return utf8.decode(new Uint8Array(this.memory, ptr, len));
  }

  /** Creates an emulator for a ROM image. Throws with the core's message. */
  create(rom) {
    const [ptr, len] = this.put(rom);
    const handle = this.x.matcha_new(ptr, len);
    this.x.matcha_free(ptr, len);
    if (!handle) throw new Error(this.lastError() || "could not load ROM");
    return new Emulator(this, handle);
  }
}

export class Emulator {
  constructor(mod, handle) {
    this.m = mod;
    this.x = mod.x;
    this.h = handle;
  }

  destroy() {
    if (this.h) this.x.matcha_destroy(this.h);
    this.h = 0;
  }

  reset() { this.x.matcha_reset(this.h); }

  /** Runs one frame. Returns { event, pc?, addr?, value?, write? }. */
  runFrame() {
    const code = this.x.matcha_run_frame(this.h);
    return this.#event(code);
  }

  #event(code) {
    const ev = { event: RUN_EVENTS[code] };
    if (code === 1 || code === 2) {
      const w = new Uint32Array(this.m.memory, this.x.matcha_words_ptr(this.h), 4);
      ev.pc = w[0];
      if (code === 2) Object.assign(ev, { addr: w[1], value: w[2], write: w[3] === 1 });
    }
    return ev;
  }

  /** Executes one instruction; returns its kind. */
  step() { return STEP_KINDS[this.x.matcha_step(this.h)]; }

  setButtons(mask) { this.x.matcha_set_buttons(this.h, mask & 0xFF); }

  setPalette(colors) {
    const [a, b, c, d] = colors;
    this.x.matcha_set_palette(this.h, a, b, c, d);
  }

  /** Current frame as a fresh RGBA copy (160x144). */
  frameRGBA() {
    const ptr = this.x.matcha_render(this.h);
    return new Uint8ClampedArray(this.m.memory, ptr, WIDTH * HEIGHT * 4).slice();
  }

  /** Last frame's lines: [{ mode3, objects, window }] x 144. */
  lineTiming() {
    const raw = new Uint16Array(this.m.memory, this.x.matcha_line_timing(this.h), HEIGHT).slice();
    return Array.from(raw, (v) => ({ mode3: v & 0x1FF, objects: (v >> 9) & 0xF, window: (v & 0x2000) !== 0 }));
  }

  /** All 384 VRAM tiles as a 128x192 RGBA sheet. */
  tilesRGBA() {
    const ptr = this.x.matcha_render_tiles(this.h);
    return new Uint8ClampedArray(this.m.memory, ptr, 128 * 192 * 4).slice();
  }

  setSampleRate(hz) { this.x.matcha_set_sample_rate(this.h, hz); }

  /** Interleaved stereo samples since the last call; clears the buffer. */
  takeAudio() {
    const len = this.x.matcha_audio_len(this.h);
    const out = new Float32Array(this.m.memory, this.x.matcha_audio_ptr(this.h), len).slice();
    this.x.matcha_audio_clear(this.h);
    return out;
  }

  setChannelMask(mask) { this.x.matcha_set_channel_mask(this.h, mask & 0x0F); }

  saveState() {
    const len = this.x.matcha_save_state(this.h);
    return new Uint8Array(this.m.memory, this.x.matcha_state_ptr(this.h), len).slice();
  }

  /** Throws on a bad state; the machine is left untouched. */
  loadState(bytes) {
    const [ptr, len] = this.m.put(bytes);
    const rc = this.x.matcha_load_state(this.h, ptr, len);
    this.x.matcha_free(ptr, len);
    if (rc !== 0) throw new Error(this.m.lastError());
  }

  /** Battery RAM copy, or null if the cartridge has none. */
  battery() {
    const len = this.x.matcha_battery_len(this.h);
    if (!len) return null;
    return new Uint8Array(this.m.memory, this.x.matcha_battery_ptr(this.h), len).slice();
  }

  batteryDirty() { return this.x.matcha_battery_dirty(this.h) === 1; }

  loadBattery(bytes) {
    const [ptr, len] = this.m.put(bytes);
    this.x.matcha_load_battery(this.h, ptr, len);
    this.x.matcha_free(ptr, len);
  }

  rtcAdvance(seconds) { this.x.matcha_rtc_advance(this.h, seconds); }

  peek(addr) { return this.x.matcha_peek(this.h, addr & 0xFFFF); }

  poke(addr, value) { this.x.matcha_poke(this.h, addr & 0xFFFF, value & 0xFF); }

  /** Side-effect-free read of `len` bytes from `addr` (wraps at 0xFFFF). */
  peekRange(addr, len) {
    const ptr = this.x.matcha_alloc(len);
    this.x.matcha_peek_range(this.h, addr & 0xFFFF, ptr, len);
    const out = new Uint8Array(this.m.memory, ptr, len).slice();
    this.x.matcha_free(ptr, len);
    return out;
  }

  /** CPU/PPU/APU state for debuggers. */
  snapshot() {
    const w = new Uint32Array(this.m.memory, this.x.matcha_snapshot(this.h), 32).slice();
    const hex16 = (v) => v & 0xFFFF;
    const apu = [0, 1, 2, 3].map((i) => ({ on: ((w[28] >> (i * 8)) & 0x10) !== 0, level: (w[28] >> (i * 8)) & 0x0F }));
    return {
      af: hex16(w[0]), bc: hex16(w[1]), de: hex16(w[2]), hl: hex16(w[3]), sp: hex16(w[4]), pc: hex16(w[5]),
      a: w[0] >> 8, f: w[0] & 0xFF, b: w[1] >> 8, c: w[1] & 0xFF, d: w[2] >> 8, e: w[2] & 0xFF, h: w[3] >> 8, l: w[3] & 0xFF,
      flags: { z: !!(w[0] & 0x80), n: !!(w[0] & 0x40), h: !!(w[0] & 0x20), c: !!(w[0] & 0x10) },
      ime: w[6] === 1, power: POWER[w[7]], ie: w[8], if: w[9],
      cycles: w[10] + w[11] * 2 ** 32, frames: w[12] + w[13] * 2 ** 32, romBank: w[14],
      ppu: { lcdc: w[15], stat: w[16], scy: w[17], scx: w[18], ly: w[19], lyc: w[20], bgp: w[21], obp0: w[22], obp1: w[23], wy: w[24], wx: w[25], line: w[26], dot: w[27], mode: w[16] & 3 },
      apu, buttons: w[29],
    };
  }

  #text(len) {
    return utf8.decode(new Uint8Array(this.m.memory, this.x.matcha_text_ptr(this.h), len));
  }

  /** [{ addr, bytes, text }] for `count` instructions from `addr`. */
  disassemble(addr, count = 16) {
    const text = this.#text(this.x.matcha_disassemble(this.h, addr & 0xFFFF, count));
    return text.trimEnd().split("\n").map((line) => {
      const [left, asm] = line.split("\t");
      const [a, ...bytes] = left.split(" ");
      return { addr: parseInt(a, 16), bytes: bytes.map((b) => parseInt(b, 16)), text: asm };
    });
  }

  header() { return JSON.parse(this.#text(this.x.matcha_header_json(this.h))); }

  serialText() { return this.#text(this.x.matcha_serial_text(this.h)); }

  addBreakpoint(addr) { return this.x.matcha_breakpoint(this.h, addr & 0xFFFF, 1) === 1; }
  removeBreakpoint(addr) { return this.x.matcha_breakpoint(this.h, addr & 0xFFFF, 0) === 1; }
  clearBreakpoints() { this.x.matcha_clear_breakpoints(this.h); }
  addWatchpoint(addr, write = true) { return this.x.matcha_watchpoint(this.h, addr & 0xFFFF, write ? 1 : 0) === 1; }
  clearWatchpoints() { this.x.matcha_clear_watchpoints(this.h); }

  /** Mnemonic of an opcode (operands zeroed), e.g. opcodeText(0x3E) = "ld a, $00". */
  opcodeText(op, cb = false) { return this.#text(this.x.matcha_opcode_text(this.h, op & 0xFF, cb ? 1 : 0)); }

  setProfiling(on) { this.x.matcha_profile(this.h, on ? 1 : 0); }
  profile() { return JSON.parse(this.#text(this.x.matcha_profile_json(this.h))); }
}
