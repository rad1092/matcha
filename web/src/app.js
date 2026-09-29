// matcha web player. Bundled by scripts/build-web.mjs, which inlines
// matcha.js, the wasm (WASM_B64) and the cartridge shelf (SHELF) above this file.
import { loadMatcha, BUTTONS, PALETTES, WIDTH, HEIGHT, FRAME_RATE } from "../matcha.js"; // @bundle:strip

const FRAME_MS = 1000 / FRAME_RATE;
const REWIND_EVERY = 3; // frames between rewind snapshots
const REWIND_DEPTH = 400; // snapshots kept (≈20 s)
const $ = (id) => document.getElementById(id);
const hex2 = (v) => v.toString(16).padStart(2, "0");
const hex4 = (v) => v.toString(16).padStart(4, "0");

// --- small utilities -----------------------------------------------------------

const store = {
  get(key) { try { return localStorage.getItem(key); } catch { return null; } },
  set(key, value) { try { localStorage.setItem(key, value); return true; } catch { return false; } },
};

const b64 = {
  decode(s) {
    const bin = atob(s);
    const out = new Uint8Array(bin.length);
    for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
    return out;
  },
  encode(bytes) {
    let s = "";
    for (let i = 0; i < bytes.length; i += 0x8000) s += String.fromCharCode.apply(null, bytes.subarray(i, i + 0x8000));
    return btoa(s);
  },
};

function fnv32(bytes) {
  let h = 0x811c9dc5;
  for (let i = 0; i < bytes.length; i++) { h ^= bytes[i]; h = Math.imul(h, 0x01000193) >>> 0; }
  return h;
}

function parseAddr(text) {
  const t = String(text).trim().replace(/^\$|^0x/i, "");
  if (!/^[0-9a-f]{1,4}$/i.test(t)) return null;
  return parseInt(t, 16);
}

function parseValue(text) {
  const t = String(text).trim();
  const v = /^\$|^0x/i.test(t) ? parseInt(t.replace(/^\$|^0x/i, ""), 16) : parseInt(t, 10);
  return Number.isInteger(v) && v >= 0 && v <= 255 ? v : null;
}

function cssVar(name) {
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim();
}

// --- state ----------------------------------------------------------------------

const S = {
  mod: null,
  gb: null,
  romKey: "",
  stateKey: "",
  cartId: "",
  running: true,
  ff: false,
  rewinding: false,
  keys: 0,
  touch: 0,
  palette: store.get("matcha:palette") in PALETTES ? store.get("matcha:palette") : "matcha",
  acc: 0,
  last: 0,
  panelT: 0,
  fpsFrames: 0,
  fpsT: 0,
  history: [],
  frameNo: 0,
  frameDirty: true,
  lastSamples: new Float32Array(0),
  breakpoints: new Set(),
  viewAddr: null,
  tab: "vram",
  memPrev: null,
  search: null,
  quick: null,
};

const lcd = $("lcd").getContext("2d");

// --- audio ----------------------------------------------------------------------

const WORKLET = `
class MatchaOut extends AudioWorkletProcessor {
  constructor() {
    super();
    this.q = []; this.i = 0; this.n = 0; this.tick = 0;
    this.port.onmessage = (e) => {
      if (e.data === "flush") { this.q = []; this.i = 0; this.n = 0; return; }
      this.q.push(e.data); this.n += e.data.length >> 1;
    };
  }
  process(_inputs, outputs) {
    const L = outputs[0][0], R = outputs[0][1] || L;
    for (let k = 0; k < L.length; k++) {
      const b = this.q[0];
      if (!b) { L[k] = 0; R[k] = 0; continue; }
      L[k] = b[this.i]; R[k] = b[this.i + 1];
      this.i += 2; this.n--;
      if (this.i >= b.length) { this.q.shift(); this.i = 0; }
    }
    if ((++this.tick & 7) === 0) this.port.postMessage(this.n);
    return true;
  }
}
registerProcessor("matcha-out", MatchaOut);`;

const audio = {
  ctx: null,
  node: null,
  on: false,
  buffered: 0,
  async enable() {
    try {
      if (!this.ctx) {
        this.ctx = new AudioContext({ latencyHint: "interactive" });
        const url = URL.createObjectURL(new Blob([WORKLET], { type: "application/javascript" }));
        await this.ctx.audioWorklet.addModule(url);
        this.node = new AudioWorkletNode(this.ctx, "matcha-out", { outputChannelCount: [2] });
        this.node.port.onmessage = (e) => { this.buffered = e.data; };
        this.node.connect(this.ctx.destination);
      }
      await this.ctx.resume();
      this.node.port.postMessage("flush");
      this.buffered = 0;
      S.gb?.setSampleRate(this.ctx.sampleRate);
      this.on = true;
    } catch (err) {
      this.on = false;
      showEvent(`Sound is unavailable here (${err.message}).`);
    }
    renderSoundChip();
  },
  disable() {
    this.on = false;
    this.ctx?.suspend();
    renderSoundChip();
  },
  push(samples) {
    if (!this.on || !samples.length) return;
    if (this.buffered > this.ctx.sampleRate * 0.25) return; // never build up latency
    this.buffered += samples.length >> 1;
    this.node.port.postMessage(samples, [samples.buffer]);
  },
};

function renderSoundChip() {
  $("sound").setAttribute("aria-pressed", String(audio.on));
  $("sound-label").textContent = audio.on ? "Sound on" : "Sound off";
}

// --- emulation loop ---------------------------------------------------------------

function buttonsNow() {
  let pad = 0;
  try {
    for (const gp of navigator.getGamepads?.() ?? []) {
      if (!gp) continue;
      const b = (i) => gp.buttons[i]?.pressed;
      if (b(12) || gp.axes[1] < -0.5) pad |= BUTTONS.up;
      if (b(13) || gp.axes[1] > 0.5) pad |= BUTTONS.down;
      if (b(14) || gp.axes[0] < -0.5) pad |= BUTTONS.left;
      if (b(15) || gp.axes[0] > 0.5) pad |= BUTTONS.right;
      if (b(1)) pad |= BUTTONS.a;
      if (b(0)) pad |= BUTTONS.b;
      if (b(8)) pad |= BUTTONS.select;
      if (b(9)) pad |= BUTTONS.start;
    }
  } catch { /* gamepads may be blocked by the frame's permissions policy */ }
  return S.keys | S.touch | pad;
}

/** Runs one frame; returns false if execution stopped at a break. */
function emulateFrame() {
  const gb = S.gb;
  gb.setButtons(buttonsNow());
  const ev = gb.runFrame();
  const samples = gb.takeAudio();
  S.lastSamples = samples.slice(-2048);
  if (!S.ff) audio.push(samples);
  S.frameDirty = true;
  S.fpsFrames++;
  if (++S.frameNo % REWIND_EVERY === 0) {
    S.history.push(gb.saveState());
    if (S.history.length > REWIND_DEPTH) S.history.shift();
  }
  if (ev.event === "breakpoint") {
    pause(`Breakpoint at $${hex4(ev.pc)}. Step runs one instruction; Run continues.`);
    return false;
  }
  if (ev.event === "watchpoint") {
    pause(`$${hex4(ev.pc)} wrote $${hex2(ev.value)} to $${hex4(ev.addr)}.`);
    return false;
  }
  return true;
}

function loop(now) {
  const dt = Math.min(now - S.last, 250);
  S.last = now;
  if (S.gb) {
    if (S.rewinding) {
      const state = S.history.pop();
      if (state) {
        S.gb.loadState(state);
        S.frameDirty = true;
      }
    } else if (S.running) {
      S.acc += dt * (S.ff ? 4 : 1);
      if (audio.on && !S.ff) {
        const seconds = audio.buffered / audio.ctx.sampleRate;
        if (seconds < 0.03) S.acc += FRAME_MS * 0.25;
        else if (seconds > 0.12) S.acc -= FRAME_MS * 0.25;
      }
      let n = 0;
      const cap = S.ff ? 8 : 3;
      while (S.acc >= FRAME_MS && n < cap) {
        S.acc -= FRAME_MS;
        n++;
        if (!emulateFrame()) break;
      }
      if (S.acc > FRAME_MS * 4) S.acc = 0;
    }
    if (S.frameDirty) drawLcd();
    if (now - S.fpsT >= 500) {
      $("fps").textContent = ((S.fpsFrames * 1000) / (now - S.fpsT || 1)).toFixed(1);
      S.fpsFrames = 0;
      S.fpsT = now;
    }
    if (now - S.panelT >= 100) {
      S.panelT = now;
      updatePanels();
    }
  }
  requestAnimationFrame(loop);
}

function drawLcd() {
  lcd.putImageData(new ImageData(S.gb.frameRGBA(), WIDTH, HEIGHT), 0, 0);
  S.frameDirty = false;
}

function setRunning(on) {
  S.running = on;
  S.acc = 0;
  const run = $("run");
  run.setAttribute("aria-pressed", String(on));
  $("run-label").textContent = on ? "Pause" : "Run";
  $("run-icon").innerHTML = on
    ? '<path d="M4 3h3v10H4zM9 3h3v10H9z" fill="currentColor"/>'
    : '<path d="M4 2.5v11L13 8z" fill="currentColor"/>';
  if (on) showEvent("");
  updatePanels();
}

function pause(message) {
  setRunning(false);
  S.viewAddr = null;
  if (message) showEvent(message);
}

function showEvent(message) {
  $("event").textContent = message;
}

// --- cartridges ------------------------------------------------------------------

function insert(bytes, meta) {
  let gb;
  try {
    gb = S.mod.create(bytes);
  } catch (err) {
    flashOverlay(`Can't run this ROM: ${err.message}`);
    return false;
  }
  if (S.gb) {
    flushBattery();
    S.gb.destroy();
  }
  S.gb = gb;
  const header = gb.header();
  S.romKey = `${header.title}:${fnv32(bytes).toString(16)}`;
  S.stateKey = `${S.romKey}:${gb.model}`;
  S.cartId = meta.id;
  gb.setPalette(PALETTES[S.palette]);
  if (audio.ctx) gb.setSampleRate(audio.ctx.sampleRate);
  const sav = store.get(`matcha:sav:${S.romKey}`);
  if (sav) {
    try { gb.loadBattery(b64.decode(sav)); } catch { /* ignore a corrupt save */ }
  }
  S.history = [];
  S.quick = null;
  S.breakpoints.clear();
  S.search = null;
  S.memPrev = null;
  S.viewAddr = null;
  if (S.tab === "heat") gb.setProfiling(true);
  $("cart-name").textContent = meta.title || header.title || "untitled";
  $("cart-kind").textContent = `${gb.model.toUpperCase()} · ${header.cartTypeName.toLowerCase().replaceAll("+", " + ")}`;
  document.title = `${meta.title || header.title || "untitled"} · ${gb.model.toUpperCase()} · matcha`;
  $("swatches").title = gb.model === "cgb" ? "Game Boy Color uses the game's own colors" : "Screen palette";
  for (const swatch of $("swatches").children) swatch.disabled = gb.model === "cgb";
  for (const el of document.querySelectorAll(".cart[data-id]")) {
    el.setAttribute("aria-current", String(el.dataset.id === meta.id));
  }
  renderSearch();
  setRunning(true);
  S.frameDirty = true;
  return true;
}

function flushBattery() {
  if (!S.gb || !S.gb.batteryDirty()) return;
  const ram = S.gb.battery();
  if (ram) store.set(`matcha:sav:${S.romKey}`, b64.encode(ram));
}

async function openFile(file) {
  if (!file) return;
  if (file.size > 8 * 1024 * 1024) {
    flashOverlay("That file is larger than any Game Boy cartridge (8 MiB).");
    return;
  }
  const bytes = new Uint8Array(await file.arrayBuffer());
  insert(bytes, { id: `file:${file.name}`, title: file.name.replace(/\.(gbc?|bin)$/i, "") });
}

let overlayTimer = 0;
function flashOverlay(text, ms = 2600) {
  const o = $("overlay");
  o.textContent = text;
  o.hidden = false;
  clearTimeout(overlayTimer);
  if (ms) overlayTimer = setTimeout(() => { o.hidden = true; }, ms);
}

function buildShelf() {
  const shelf = $("shelf");
  for (const cart of SHELF) {
    const b = document.createElement("button");
    b.type = "button";
    b.className = "cart";
    b.dataset.id = cart.id;
    b.setAttribute("aria-current", "false");
    b.innerHTML = `<canvas width="160" height="144"></canvas><span class="cart-title"></span><span class="cart-meta"></span>`;
    b.querySelector(".cart-title").textContent = cart.title;
    b.querySelector(".cart-meta").textContent = `${cart.author} · ${cart.license}`;
    b.addEventListener("click", () => insert(cart.rom, cart));
    shelf.append(b);
  }
  const open = document.createElement("label");
  open.className = "cart open";
  open.innerHTML = `<input type="file" id="file" accept=".gb,.gbc,.bin"><span class="cart-title">Open a ROM…</span><span class="cart-meta">.gb / .gbc · or drop it on the screen</span>`;
  shelf.append(open);
  $("file").addEventListener("change", (e) => openFile(e.target.files[0]));

  const credits = $("credits");
  for (const cart of SHELF) {
    const li = document.createElement("li");
    const a = document.createElement("a");
    a.href = cart.url;
    a.target = "_blank";
    a.rel = "noopener";
    a.textContent = cart.title;
    li.append(a, ` by ${cart.author} — ${cart.license}`);
    credits.append(li);
  }
}

/**
 * Renders each shelf cartridge's attract screen with a throwaway emulator,
 * replaying the short input script in `cart.thumb` ([frame, button | "" | null]).
 * Works in small time slices so the running game keeps its frame rate.
 */
function paintThumbnails() {
  const jobs = [...document.querySelectorAll(".cart[data-id]")].map((el, i) => ({ el, cart: SHELF[i] }));
  let job = null;
  const slice = () => {
    const deadline = performance.now() + 5;
    try {
      while (performance.now() < deadline) {
        if (!job) {
          const next = jobs.shift();
          if (!next) return;
          const gb = S.mod.create(next.cart.rom);
          gb.setPalette(PALETTES[S.palette]);
          job = { ...next, gb, frame: 0, step: 0 };
        }
        const [at, button] = job.cart.thumb[job.step] ?? [job.frame, null];
        if (job.frame < at) {
          job.gb.runFrame();
          job.gb.takeAudio();
          job.frame++;
          continue;
        }
        if (button === null) {
          job.el.querySelector("canvas").getContext("2d").putImageData(new ImageData(job.gb.frameRGBA(), WIDTH, HEIGHT), 0, 0);
          job.gb.destroy();
          job = null;
          continue;
        }
        job.gb.setButtons(button ? BUTTONS[button] : 0);
        job.step++;
      }
    } catch {
      // A thumbnail is decoration: drop this one and carry on.
      job?.gb.destroy();
      job = null;
    }
    setTimeout(slice, 20);
  };
  setTimeout(slice, 1500);
}

// --- panels ----------------------------------------------------------------------

const REG_NAMES = ["af", "bc", "de", "hl", "sp", "pc"];

function buildPanels() {
  const regs = $("regs");
  for (const r of REG_NAMES) {
    const d = document.createElement("div");
    d.className = "reg";
    d.innerHTML = `<span>${r.toUpperCase()}</span><b id="reg-${r}">0000</b>`;
    regs.append(d);
  }
  const meters = $("meters");
  ["Pulse 1", "Pulse 2", "Wave", "Noise"].forEach((name, i) => {
    const row = document.createElement("div");
    row.className = "meter";
    row.id = `meter-${i}`;
    row.innerHTML = `<label><input type="checkbox" id="mute-${i}" checked> ${name}</label><div class="bar"><i></i></div><span class="readout">0</span>`;
    meters.append(row);
    row.querySelector("input").addEventListener("change", applyChannelMask);
  });
  for (const id of ["heat", "heat-cb"]) {
    const grid = $(id);
    for (let op = 0; op < 256; op++) {
      const cell = document.createElement("i");
      cell.dataset.op = op;
      grid.append(cell);
    }
    grid.addEventListener("pointerover", (e) => {
      const op = e.target.dataset?.op;
      if (op === undefined || !S.gb) return;
      const cb = id === "heat-cb";
      const text = S.gb.opcodeText(Number(op), cb);
      const count = e.target.dataset.count ?? "0";
      $("heat-cap").textContent = `${cb ? "cb " : ""}${hex2(Number(op))}  ${text}  ×${Number(count).toLocaleString()}`;
    });
  }
}

function applyChannelMask() {
  let mask = 0;
  for (let i = 0; i < 4; i++) if ($(`mute-${i}`).checked) mask |= 1 << i;
  S.gb?.setChannelMask(mask);
}

function updatePanels() {
  if (!S.gb) return;
  const s = S.gb.snapshot();
  $("hardware-spec").textContent = s.model === "cgb"
    ? `Game Boy Color · ${s.doubleSpeed ? "8.388608" : "4.194304"} MHz · 160×144 · game colors · 59.73 Hz`
    : "Game Boy · 4.194304 MHz · 160×144 · 4 shades · 59.73 Hz";
  updateCpu(s);
  updateTiming(s);
  if (S.tab === "vram") updateVram();
  if (S.tab === "sound") updateSound(s);
  if (S.tab === "heat") updateHeat();
  if (S.tab === "mem") updateMemory();
  const led = $("led");
  led.className = `led ${s.power === "running" && S.running ? "on" : s.power !== "running" ? "halt" : ""}`;
  $("speed-label").textContent = S.rewinding ? "◀◀" : S.ff ? "4×" : S.running ? "1×" : "paused";
}

function updateCpu(s) {
  for (const r of REG_NAMES) $(`reg-${r}`).textContent = hex4(s[r]).toUpperCase();
  const chips = [
    ["Z", s.flags.z], ["N", s.flags.n], ["H", s.flags.h], ["C", s.flags.c],
    ["IME", s.ime], [s.power.toUpperCase(), s.power !== "running"],
    [`IE ${hex2(s.ie)}`, s.ie !== 0], [`IF ${hex2(s.if)}`, (s.ie & s.if & 0x1f) !== 0],
    [`BANK ${s.romBank}`, false],
  ];
  $("cpu-chips").innerHTML = chips.map(([t, on]) => `<span class="chip${on ? " on" : ""}">${t}</span>`).join("");
  $("cpu-readout").textContent = `${s.model.toUpperCase()}${s.doubleSpeed ? " · double CPU speed" : ""} · ${s.cycles.toLocaleString()} M-cycles · frame ${s.frames.toLocaleString()}`;
  const start = S.viewAddr ?? s.pc;
  const lines = S.gb.disassemble(start, 12);
  const list = $("disasm");
  list.innerHTML = "";
  for (const ins of lines) {
    const li = document.createElement("li");
    li.dataset.addr = ins.addr;
    if (ins.addr === s.pc) li.classList.add("pc");
    if (S.breakpoints.has(ins.addr)) li.classList.add("bp");
    li.innerHTML = `<i class="dot"></i><span>${hex4(ins.addr)}</span><span class="bytes">${ins.bytes.map(hex2).join(" ")}</span><span></span>`;
    li.lastChild.textContent = ins.text;
    list.append(li);
  }
}

function updateTiming(s) {
  const c = $("timing").getContext("2d");
  const colors = { m0: cssVar("--mode0"), m1: cssVar("--mode1"), m2: cssVar("--mode2"), m3: cssVar("--mode3"), beam: cssVar("--bp") };
  const lines = S.gb.lineTiming();
  const lcdOn = (s.ppu.lcdc & 0x80) !== 0;
  c.fillStyle = colors.m0;
  c.fillRect(0, 0, 456, 154);
  let maxMode3 = 0;
  let sprited = 0;
  if (lcdOn) {
    for (let y = 0; y < 144; y++) {
      const t = lines[y];
      c.fillStyle = colors.m2;
      c.fillRect(4, y, 80, 1);
      c.fillStyle = colors.m3;
      c.fillRect(84, y, t.mode3 || 172, 1);
      maxMode3 = Math.max(maxMode3, t.mode3);
      if (t.objects) sprited++;
    }
    c.fillStyle = colors.m1;
    c.fillRect(0, 144, 456, 10);
    if (!S.running || s.power !== "running") {
      c.fillStyle = colors.beam;
      c.fillRect(Math.max(0, s.ppu.dot - 2), Math.max(0, s.ppu.line - 2), 5, 5);
    }
  }
  const mode = ["HBlank", "VBlank", "OAM scan", "Drawing"][s.ppu.mode];
  $("timing-readout").textContent = lcdOn
    ? `LY ${s.ppu.ly} · dot ${s.ppu.dot} · ${mode} · longest mode 3 ${maxMode3} dots · ${sprited} lines with sprites`
    : "LCD off";
  const L = s.ppu.lcdc;
  const bits = [
    ["LCD", L & 0x80], [L & 0x40 ? "WIN 9C00" : "WIN 9800", 0], ["WIN", L & 0x20],
    [L & 0x10 ? "TILES 8000" : "TILES 8800", 0], [L & 0x08 ? "BG 9C00" : "BG 9800", 0],
    [L & 0x04 ? "OBJ 8×16" : "OBJ 8×8", 0], ["OBJ", L & 0x02], ["BG", L & 0x01],
    [`SCX ${s.ppu.scx}`, 0], [`SCY ${s.ppu.scy}`, 0], [`WX ${s.ppu.wx}`, 0], [`WY ${s.ppu.wy}`, 0], [`LYC ${s.ppu.lyc}`, 0],
  ];
  $("lcdc").innerHTML = bits.map(([t, on]) => `<span class="chip${on ? " on" : ""}">${t}</span>`).join("");
}

function updateVram() {
  $("tiles").getContext("2d").putImageData(new ImageData(S.gb.tilesRGBA(), 128, 192), 0, 0);
}

function updateSound(s) {
  s.apu.forEach((ch, i) => {
    const row = $(`meter-${i}`);
    row.classList.toggle("off", !ch.on);
    row.querySelector(".bar i").style.width = `${(ch.level / 15) * 100}%`;
    row.querySelector(".readout").textContent = ch.on ? String(ch.level) : "off";
  });
  const cv = $("scope");
  const c = cv.getContext("2d");
  const w = cv.width;
  const h = cv.height;
  c.clearRect(0, 0, w, h);
  c.strokeStyle = cssVar("--accent");
  c.lineWidth = 1.5;
  c.beginPath();
  const smp = S.lastSamples;
  const frames = smp.length >> 1;
  for (let x = 0; x < w; x++) {
    const i = Math.floor((x / w) * frames) * 2;
    const v = frames ? (smp[i] + smp[i + 1]) * 0.5 : 0;
    const y = h / 2 - v * h * 1.6;
    if (x === 0) c.moveTo(x, y);
    else c.lineTo(x, y);
  }
  c.stroke();
}

function heatColor(count, max) {
  if (!count) return "var(--heat-0)";
  const t = Math.log(count + 1) / Math.log(max + 1);
  return t > 0.8 ? "var(--heat-4)" : t > 0.6 ? "var(--heat-3)" : t > 0.35 ? "var(--heat-2)" : "var(--heat-1)";
}

function updateHeat() {
  const p = S.gb.profile();
  if (!p) return;
  for (const [id, counts] of [["heat", p.opcodes], ["heat-cb", p.cb]]) {
    const max = Math.max(1, ...counts);
    const cells = $(id).children;
    for (let op = 0; op < 256; op++) {
      cells[op].style.background = heatColor(counts[op], max);
      cells[op].dataset.count = counts[op];
    }
  }
  const idle = p.halted + p.stopped + p.locked;
  const total = p.busy + p.interrupt + idle || 1;
  const pct = (v) => ((v / total) * 100).toFixed(1);
  $("util").innerHTML = `
    <span>busy ${pct(p.busy)}%</span><span>interrupts ${pct(p.interrupt)}%</span><span>${p.locked ? "halted / locked" : "halted"} ${pct(idle)}%</span><span class="readout" style="text-align:right">${p.instructions.toLocaleString()} instructions</span>
    <div class="stack"><i style="width:${pct(p.busy)}%;background:var(--heat-4)"></i><i style="width:${pct(p.interrupt)}%;background:var(--heat-2)"></i><i style="width:${pct(idle)}%;background:var(--heat-1)"></i></div>`;
}

function memStart() {
  const typed = parseAddr($("mem-addr").value);
  return typed ?? parseInt($("mem-region").value, 16);
}

function updateMemory() {
  const start = memStart() & 0xfff0;
  const bytes = S.gb.peekRange(start, 128);
  const prev = S.memPrev && S.memPrev.start === start ? S.memPrev.bytes : null;
  let out = "      " + [...Array(16).keys()].map((i) => ` ${hex2(i)}`).join("") + "\n";
  for (let row = 0; row < 8; row++) {
    out += `${hex4(start + row * 16)}  `;
    for (let col = 0; col < 16; col++) {
      const i = row * 16 + col;
      const v = hex2(bytes[i]);
      out += prev && prev[i] !== bytes[i] ? ` <span class="chg">${v}</span>` : ` ${v}`;
    }
    out += "\n";
  }
  $("hex").innerHTML = out;
  S.memPrev = { start, bytes };
  if (S.search) renderSearch();
}

// --- value finder -------------------------------------------------------------------

const SEARCH_RANGES = [[0xc000, 0x2000], [0xff80, 0x7f]];

function searchSnapshot() {
  const values = new Map();
  for (const [start, len] of SEARCH_RANGES) {
    const bytes = S.gb.peekRange(start, len);
    bytes.forEach((v, i) => values.set(start + i, v));
  }
  return values;
}

function renderSearch() {
  const list = $("cands");
  list.innerHTML = "";
  if (!S.search) {
    $("search-count").textContent = "not started";
    return;
  }
  const { candidates } = S.search;
  $("search-count").textContent = `${candidates.length.toLocaleString()} candidate${candidates.length === 1 ? "" : "s"}`;
  if (candidates.length > 60) return;
  for (const addr of candidates) {
    const li = document.createElement("li");
    const btn = document.createElement("button");
    btn.type = "button";
    btn.textContent = `$${hex4(addr)} = ${S.gb.peek(addr)}`;
    btn.addEventListener("click", () => {
      $("mem-addr").value = hex4(addr);
      S.gb.addWatchpoint(addr, true);
      showEvent(`Watching writes to $${hex4(addr)}; the game pauses when something changes it.`);
      updateMemory();
    });
    li.append(btn);
    list.append(li);
  }
}

function searchNew() {
  if (!S.gb) return;
  const snap = searchSnapshot();
  S.search = { candidates: [...snap.keys()], prev: snap };
  renderSearch();
}

function searchFilter() {
  if (!S.gb) return;
  if (!S.search) searchNew();
  const op = $("search-op").value;
  const target = parseValue($("search-val").value);
  if (op === "eq" && target === null) {
    showEvent("Type a value from 0 to 255 (decimal, or hex like $3C).");
    return;
  }
  const now = searchSnapshot();
  const keep = (a) => {
    const before = S.search.prev.get(a);
    const after = now.get(a);
    switch (op) {
      case "changed": return after !== before;
      case "same": return after === before;
      case "up": return after > before;
      case "down": return after < before;
      default: return after === target;
    }
  };
  S.search = { candidates: S.search.candidates.filter(keep), prev: now };
  renderSearch();
}

// --- input -----------------------------------------------------------------------------

const KEYMAP = {
  ArrowRight: BUTTONS.right, ArrowLeft: BUTTONS.left, ArrowUp: BUTTONS.up, ArrowDown: BUTTONS.down,
  KeyX: BUTTONS.a, KeyZ: BUTTONS.b, Enter: BUTTONS.start, Backspace: BUTTONS.select, ShiftRight: BUTTONS.select,
};

function isTyping(target) {
  return target instanceof HTMLInputElement || target instanceof HTMLSelectElement || target instanceof HTMLTextAreaElement;
}

function quickSave() {
  if (!S.gb) return;
  S.quick = S.gb.saveState();
  store.set(`matcha:state:${S.stateKey}`, b64.encode(S.quick));
  showEvent("Saved. L loads it back.");
}

function quickLoad() {
  if (!S.gb) return;
  const encoded = store.get(`matcha:state:${S.stateKey}`);
  const stored = S.quick ?? (encoded ? b64.decode(encoded) : null);
  if (!stored) {
    showEvent("No saved state for this cartridge yet. Press S first.");
    return;
  }
  try {
    S.gb.loadState(stored);
    S.frameDirty = true;
    showEvent("Loaded.");
  } catch (err) {
    showEvent(`Could not load that state: ${err.message}`);
  }
}

function stepInstruction() {
  if (!S.gb) return;
  if (S.running) setRunning(false);
  S.gb.setButtons(buttonsNow());
  S.gb.step();
  S.viewAddr = null;
  S.frameDirty = true;
  updatePanels();
}

function stepFrame() {
  if (!S.gb) return;
  if (S.running) setRunning(false);
  emulateFrame();
  updatePanels();
}

function wireInput() {
  addEventListener("keydown", (e) => {
    if (isTyping(e.target) || e.metaKey || e.ctrlKey || e.altKey) return;
    if (!audio.ctx && !audio.on && e.code in KEYMAP) audio.enable();
    if (e.code in KEYMAP) {
      S.keys |= KEYMAP[e.code];
      e.preventDefault();
      return;
    }
    if (e.repeat) return;
    const onButton = e.target instanceof HTMLButtonElement;
    switch (e.code) {
      case "Space": if (!onButton) { setRunning(!S.running); e.preventDefault(); } break;
      case "KeyR": S.rewinding = true; break;
      case "KeyF": S.ff = !S.ff; $("ff").setAttribute("aria-pressed", String(S.ff)); break;
      case "KeyN": stepFrame(); break;
      case "KeyI": stepInstruction(); break;
      case "KeyS": quickSave(); break;
      case "KeyL": quickLoad(); break;
      case "KeyM": audio.on ? audio.disable() : audio.enable(); break;
      default: return;
    }
  });
  addEventListener("keyup", (e) => {
    if (e.code in KEYMAP) S.keys &= ~KEYMAP[e.code];
    if (e.code === "KeyR") S.rewinding = false;
  });
  addEventListener("blur", () => { S.keys = 0; S.touch = 0; S.rewinding = false; });

  for (const btn of document.querySelectorAll("#pad [data-btn]")) {
    const mask = BUTTONS[btn.dataset.btn];
    const down = (e) => {
      e.preventDefault();
      btn.setPointerCapture?.(e.pointerId);
      S.touch |= mask;
      btn.classList.add("held");
      if (!audio.ctx) audio.enable();
    };
    const up = () => { S.touch &= ~mask; btn.classList.remove("held"); };
    btn.addEventListener("pointerdown", down);
    btn.addEventListener("pointerup", up);
    btn.addEventListener("pointercancel", up);
    btn.addEventListener("lostpointercapture", up);
  }

  $("run").addEventListener("click", () => setRunning(!S.running));
  const rw = $("rewind");
  rw.addEventListener("pointerdown", () => { S.rewinding = true; });
  for (const ev of ["pointerup", "pointerleave", "pointercancel"]) rw.addEventListener(ev, () => { S.rewinding = false; });
  $("ff").addEventListener("click", () => { S.ff = !S.ff; $("ff").setAttribute("aria-pressed", String(S.ff)); });
  $("frame").addEventListener("click", stepFrame);
  $("step").addEventListener("click", stepInstruction);
  $("save").addEventListener("click", quickSave);
  $("load").addEventListener("click", quickLoad);
  $("reset").addEventListener("click", () => { if (S.gb) { S.gb.reset(); S.history = []; setRunning(true); } });
  $("sound").addEventListener("click", () => (audio.on ? audio.disable() : audio.enable()));
  $("clear-bp").addEventListener("click", () => {
    S.gb?.clearBreakpoints();
    S.gb?.clearWatchpoints();
    S.breakpoints.clear();
    showEvent("Breakpoints and watches cleared.");
    updatePanels();
  });
  $("goto").addEventListener("change", (e) => {
    const addr = parseAddr(e.target.value);
    S.viewAddr = addr;
    e.target.value = addr === null ? "" : hex4(addr);
    updatePanels();
  });
  $("disasm").addEventListener("click", (e) => {
    const li = e.target.closest("li");
    if (!li || !S.gb) return;
    const addr = Number(li.dataset.addr);
    if (S.breakpoints.has(addr)) {
      S.breakpoints.delete(addr);
      S.gb.removeBreakpoint(addr);
    } else {
      S.breakpoints.add(addr);
      S.gb.addBreakpoint(addr);
    }
    updatePanels();
  });

  for (const tab of document.querySelectorAll(".tab")) {
    tab.addEventListener("click", () => selectTab(tab.id.replace("tab-", "")));
  }
  $("mem-region").addEventListener("change", () => { $("mem-addr").value = ""; updateMemory(); });
  $("mem-addr").addEventListener("change", updateMemory);
  $("search-new").addEventListener("click", searchNew);
  $("search-filter").addEventListener("click", searchFilter);

  const stage = $("stage");
  stage.addEventListener("dragover", (e) => { e.preventDefault(); stage.classList.add("drop"); });
  stage.addEventListener("dragleave", () => stage.classList.remove("drop"));
  stage.addEventListener("drop", (e) => {
    e.preventDefault();
    stage.classList.remove("drop");
    openFile(e.dataTransfer?.files?.[0]);
  });

  document.addEventListener("visibilitychange", () => { if (document.hidden) flushBattery(); });
  addEventListener("pagehide", flushBattery);
  setInterval(flushBattery, 2000);
}

function selectTab(name) {
  S.tab = name;
  for (const tab of document.querySelectorAll(".tab")) {
    const on = tab.id === `tab-${name}`;
    tab.setAttribute("aria-selected", String(on));
    $(tab.getAttribute("aria-controls")).hidden = !on;
  }
  S.gb?.setProfiling(name === "heat");
  updatePanels();
}

function buildSwatches() {
  const group = $("swatches");
  for (const [name, colors] of Object.entries(PALETTES)) {
    const b = document.createElement("button");
    b.type = "button";
    b.className = "swatch";
    b.title = `${name} palette`;
    b.setAttribute("aria-label", `${name} palette`);
    b.setAttribute("aria-pressed", String(name === S.palette));
    b.innerHTML = colors.map((c) => `<span style="background:#${c.toString(16).padStart(6, "0")}"></span>`).join("");
    b.addEventListener("click", () => {
      S.palette = name;
      store.set("matcha:palette", name);
      S.gb?.setPalette(colors);
      S.frameDirty = true;
      for (const el of group.children) el.setAttribute("aria-pressed", String(el === b));
    });
    group.append(b);
  }
}

// --- boot ------------------------------------------------------------------------------

async function boot() {
  buildSwatches();
  buildPanels();
  buildShelf();
  wireInput();
  try {
    S.mod = await loadMatcha(b64.decode(WASM_B64));
  } catch (err) {
    flashOverlay(`This browser could not start WebAssembly: ${err.message}`, 0);
    return;
  }
  const first = SHELF.find((c) => c.id === "tobu") ?? SHELF[0];
  insert(first.rom, first);
  requestAnimationFrame((t) => { S.last = t; S.fpsT = t; loop(t); });
  paintThumbnails();
}

boot();
