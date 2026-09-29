# matcha — system design

matcha is a cycle-accurate Game Boy (DMG) emulator written as one portable
Rust core with three hosts: a single-file web player, a Model Context
Protocol (MCP) server that lets an AI agent play and debug games, and a
native command-line tool for conformance testing and profiling.

This document describes the system as built. The reasoning behind each
major choice is recorded in the ADRs under [`docs/adr/`](adr/).

## 1. Requirements

### Functional

| # | Requirement | Where |
|---|---|---|
| F1 | Run original DMG software: CPU, PPU, APU, timer, serial, joypad, DMA, MBC1/1M/2/3(+RTC)/5 | `crates/matcha-core` |
| F2 | Play in a browser with sound, keyboard/touch/gamepad input, battery saves, save states and rewind | `web/` |
| F3 | Inspect a running game: registers, disassembly, breakpoints, watchpoints, VRAM, audio channels, memory search | web debugger panels, MCP tools |
| F4 | Let an AI agent load, play, screenshot, inspect and debug ROMs through tools | `mcp/server.mjs`, `plugin/` |
| F5 | Measure accuracy against public test suites and publish the scoreboard | `matcha test`, `docs/CONFORMANCE.md` |
| F6 | Profile ROMs headlessly (opcodes, interrupts, memory regions, CPU idle time) for corpus analysis | `matcha profile`, `analysis/` |

### Non-functional

| Property | Target | Status |
|---|---|---|
| Accuracy | Pass the CPU, timing, sound and PPU suites real games depend on | SST 498,000/498,000; Blargg 38/44; Mooneye 94/94; dmg-acid2 1/1 ([scoreboard](CONFORMANCE.md)) |
| Speed | Full speed with audio in a browser on a phone-class CPU | 51× real time headless, 37× with audio (native, one core of the 2-vCPU build container) |
| Determinism | Same ROM + inputs + state ⇒ identical frames and audio, bit for bit | Unit-tested (two machines, save/replay) |
| Portability | One core for browser, Node and native; no OS services | `no_std` + `alloc`, zero dependencies |
| Safety | Any byte sequence is a valid ROM or state input: no panics, no UB | `#![forbid(unsafe_code)]` in the core; bounds-checked state reader; atomic state load |
| Footprint | Small enough to embed in one HTML file | `matcha.wasm` ≈ 148 KiB, no imports |
| Install | Plugin works with nothing but Node | MCP server has no npm dependencies |

### Constraints

- One maintainer; the design must stay readable by one person (and by an AI
  assistant resuming the work — see [`CONTINUE.md`](../CONTINUE.md)).
- Test ROMs and the homebrew corpus are fetched, never committed.
- The published page must be a single self-contained HTML file (the artifact
  host only allows scripts from a few CDNs; matcha needs none).

## 2. High-level design

```
                  ┌──────────────────────── matcha-core (no_std, safe Rust) ────────────────────────┐
                  │                                                                                 │
   ROM bytes ───▶ │  Cartridge ◀──┐                                                                 │
                  │  (MBC, RTC,   │        ┌─────────── SystemBus ────────────┐                     │
                  │   battery)    ├─ read/ │ memory map · IE/IF · OAM DMA     │                     │
                  │               │ write  │ tick() once per M-cycle:         │                     │
                  │   Cpu ────────┘ idle   │   Timer → PPU → APU → DMA → RTC  │                     │
                  │  (SM83)  ◀── pending ──┤ watchpoints · profiler hooks     │                     │
                  │                        └──────────────────────────────────┘                     │
                  │  GameBoy: run_frame / step / save_state / load_state / debugger API             │
                  └──────────────┬─────────────────────────────────────┬────────────────────────────┘
                                 │ Rust API                            │ Rust API
                 ┌───────────────┴──────────┐              ┌───────────┴────────────┐
                 │ matcha-wasm (C ABI)      │              │ matcha-cli (native)    │
                 │ handle + owned buffers   │              │ run · test · profile   │
                 └───────────────┬──────────┘              │ disasm · info          │
                                 │ matcha.wasm             └───────────┬────────────┘
                     ┌───────────┴────────────┐                        │
                     │ web/matcha.js wrapper  │              docs/conformance.json
                     └─────┬────────────┬─────┘              analysis/data/*.json
                           │            │
             ┌─────────────┴───┐   ┌────┴──────────────────┐
             │ Web player      │   │ MCP server (Node)     │◀── stdio JSON-RPC ── Claude
             │ canvas, audio   │   │ 17 tools, PNG encoder │
             │ worklet, rewind │   │ + plugin skills       │
             └─────────────────┘   └───────────────────────┘
```

### Core modules

| Module | Responsibility | Notes |
|---|---|---|
| `cpu.rs` | SM83 interpreter over the `CpuBus` trait | One bus access per M-cycle, EI delay, HALT bug, IE-push dispatch quirk, illegal-opcode lock |
| `bus.rs` | Memory map, interrupts, OAM DMA, clocking of all devices | Implements `CpuBus`; the only place devices are ticked |
| `ppu.rs` | LCD timing, STAT/LY interrupts, access blocking, rendering | Event-scheduled line timing (ADR-0001, ADR-0002) |
| `apu.rs` | Four channels, frame sequencer, mixer, resampler | Cached mix + box filter + DMG high-pass (ADR-0005) |
| `timer.rs` | 16-bit system counter, falling-edge TIMA, DIV-APU and serial clocks | TIMA reload state machine |
| `cartridge.rs` | Header parsing, MBC1/1M/2/3/5, RTC, battery RAM | RTC runs on emulated time only |
| `joypad.rs`, `serial.rs` | P1 matrix and interrupts; serial port with captured output | Serial output feeds Blargg tests and the MCP `serial_output` tool |
| `state.rs` | Versioned binary writer/reader | ADR-0006 |
| `disasm.rs` | RGBDS-syntax disassembler | Used by the debugger, MCP and CLI |
| `profile.rs` | Opt-in execution profile | Opcode counts, cycles by state, memory regions, coverage |
| `lib.rs` | `GameBoy` facade | Run loop, breakpoints, palettes, public API |

## 3. How time works

The CPU drives the clock. Every CPU memory access is a bus call that first
performs the access and then advances every other device by one M-cycle
(4 dots). Internal CPU cycles call `idle()`, which only advances time. This
"access, then tick" order makes each access land at a fixed phase of the
M-cycle, which is what the PPU access windows and interrupt timings are
measured against (ADR-0001).

`SystemBus::tick` for one M-cycle:

```
cycles += 1
timer.tick()         → TIMA interrupt, DIV-APU edge, serial clock
ppu.tick()           → interrupts raised "now" or "late" in this M-cycle
apu.tick(div_apu)    → channel timers, frame sequencer, sample accumulation
tick_dma()           → one OAM DMA byte
cart.tick_rtc(4)     → MBC3 clock on emulated time
```

Interrupts that the PPU raises in the last two dots of an M-cycle are marked
`late` and hidden from dispatch for one M-cycle (`if_deferred`), matching
when real hardware samples IF.

### Keeping it fast without losing accuracy

Three mechanisms keep the per-M-cycle work small:

1. **PPU event scheduling.** Within a line, the PPU knows the dot of its next
   state change (mode switch, STAT update, LY compare, access-window edge).
   When the next event is more than one M-cycle away, `tick` adds 4 to the
   dot counter and returns. Rendering happens a line at a time at the mode-3
   boundary, using the per-line register values (ADR-0002).
2. **HALT fast path.** A halted CPU loops on `idle()` inside `Cpu::step`
   until an interrupt is pending or the bus says to yield (end of frame,
   breakpoint budget). No instruction decode happens while halted — and most
   games spend most of their time halted (see [`analysis.md`](analysis.md)).
3. **Cached audio mix.** Channels report when their output level changes; the
   stereo mix is recomputed only then, and each M-cycle adds the cached value
   to an accumulator.

## 4. Frontends and data flow

### One frame in the browser

```
requestAnimationFrame(now)
  ├─ rewinding?  pop the newest snapshot from the rewind ring → loadState
  └─ running?    accumulate elapsed time (×4 when fast-forwarding);
                 nudge it when the audio queue leaves 30–120 ms
                 for each whole frame owed (at most 3; 8 when fast-forwarding):
                   gb.setButtons(keyboard | touch | gamepad)
                   gb.runFrame()                 ── WASM: 17,556 M-cycles
                   every 3rd frame: push saveState() (ring of 400 ≈ 20 s)
                   gb.takeAudio() → Float32Array block → AudioWorklet queue
  ├─ new frame?  gb.frameRGBA(palette) → putImageData on the 160×144 canvas
  └─ every 100 ms: repaint debugger panels (CPU, disassembly, frame timing,
                   VRAM tiles, channel levels, memory)

every 2 s: battery RAM dirty? → localStorage "matcha:sav:<rom key>" (best effort)
```

### One agent action through MCP

```
Claude ──tools/call press {buttons:["a"]}──▶ server.mjs
   server: validate args → hold A for 6 frames → release → run 30 frames
           (either stops early at a breakpoint/watchpoint)
           → encode the framebuffer as PNG (zlib deflate, 2× scale)
   ◀── { content: [ {type:"text", text:<stop reason + status line>},
                    {type:"image", mimeType:"image/png", data:<base64>} ] }
```

### Interfaces

| Boundary | Contract | Defined in |
|---|---|---|
| Core ↔ hosts | `GameBoy` Rust API (`run_frame`, `step`, `set_buttons`, `framebuffer`, `audio_samples`, `save_state`/`load_state`, `peek`/`poke`, breakpoints, watchpoints, profiling) | `crates/matcha-core/src/lib.rs` |
| Rust ↔ JavaScript | 45 `extern "C"` exports over an opaque handle; bulk data in handle-owned buffers; errors via `matcha_last_error_*` | `crates/matcha-wasm/src/lib.rs` (ADR-0004) |
| JS wrapper | `loadMatcha(bytes)` → `MatchaModule.create(rom)` → `Emulator` methods; shared by browser and Node | `web/matcha.js` |
| Agent ↔ server | MCP over stdio, protocol 2024-11-05 … 2025-11-25; 17 tools | `mcp/server.mjs` (ADR-0007) |
| Shell ↔ CLI | `matcha info/run/test/profile/disasm`; JSON and Markdown outputs | `crates/matcha-cli/src/main.rs` |

## 5. State and storage

| Data | Format | Lives in | Size |
|---|---|---|---|
| Save state | `MTCHST` + version + ROM id + components (ADR-0006) | memory, `localStorage` quick-save slot, MCP named slots | ≈ 39 KiB + cartridge RAM (39,944–48,136 B for the bundled ROMs) |
| Rewind history | ring of save states | browser memory | 400 × ≈ 40–48 KiB ≈ 16–19 MB |
| Battery RAM | raw cartridge RAM (+ RTC for MBC3) | `localStorage` (web), `.sav` file (CLI) | 0–128 KiB |
| Conformance results | JSON + Markdown | `docs/` | — |
| Corpus manifest and profiles | CSV + JSON | `analysis/data/` | — |

Loading a state is atomic: the current state is saved first and restored if
any field fails validation, so a bad file can never leave a half-loaded
machine.

## 6. Performance

Measured with `matcha run --seconds 60` (release build, native, audio off
unless stated) on one core of the 2-vCPU build container:

| Configuration | Speed |
|---|---|
| Before optimisation (per-dot PPU loop, uncached mixer) | 18× real time |
| Current, video only | 51× real time |
| Current, with 48 kHz audio | 37× real time |

A frame is 17,556 M-cycles (16.74 ms of Game Boy time). At 37× the core
needs ≈ 0.45 ms per frame, leaving the browser most of its 16.7 ms budget
for painting and the debugger panels even when WebAssembly runs a few times
slower than native. Fast-forward runs four frames per display frame.

## 7. Reliability and safety

- **Untrusted input.** ROMs, save states and battery files come from users.
  Unknown mapper types are rejected with an error; every other byte sequence
  runs. Illegal opcodes lock the CPU (as on hardware) instead of panicking.
  The state reader is bounds-checked and validates enumerations and
  counters.
- **No unsafe in the core.** `matcha-core` forbids `unsafe`; the unsafe code
  is confined to the thin WASM export layer, each function documenting its
  pointer contract.
- **Deterministic by construction.** The core never reads a clock or a random
  source; the MBC3 RTC advances with emulated cycles, and hosts that want
  wall-clock time call `rtc_advance_seconds` explicitly.
- **Failures are visible.** The CLI's test runner reports each ROM as pass,
  fail or error; CI sets `MATCHA_REQUIRE_TESTDATA=1` so missing test data
  fails instead of silently skipping.

## 8. Trade-offs

| Choice | Gained | Given up | ADR |
|---|---|---|---|
| CPU-driven M-cycle ticking | Simple, exact interleaving of CPU accesses and device state | Every access pays a function call per device | 0001 |
| Scanline renderer with measured line timing | Speed; all DMG timing tests pass | Mid-scanline register effects (Mealybug 1/24) | 0002 |
| `no_std`, zero dependencies, safe Rust | Runs anywhere; auditable | Writing small utilities (hashing, `powf`) ourselves | 0003 |
| Hand-written C ABI | One `.wasm` for browser and Node; no toolchain coupling | Manual glue; `unsafe` at the boundary | 0004 |
| Box-filter audio | Cheap, deterministic, sounds right for chiptune | Slight aliasing at very high pitches | 0005 |
| Hand-rolled state format | No dependencies; explicit, validated layout | Version bump on any layout change | 0006 |
| Hand-written MCP server | Install = copy a folder | Protocol features added by hand | 0007 |
| Test suites as the definition of done | Objective progress; loud regressions | Pressure toward sub-cycle behaviour few games use | 0008 |

## 9. What to revisit as it grows

In priority order (details in [`ROADMAP.md`](../ROADMAP.md)):

1. **Game Boy Color.** 29% of the corpus is CGB-only (see
   [`analysis.md`](analysis.md)); double-speed mode, VRAM/WRAM banking,
   palettes and HDMA fit the existing bus/PPU split.
2. **Pixel FIFO renderer** for mid-scanline effects (Mealybug), behind the
   same line-timing model so existing tests keep passing.
3. **OAM corruption bug** (the remaining Blargg failures) via hooks on the
   CPU's 16-bit increment/decrement unit.
4. **Band-limited audio** if audio quality becomes a priority.
5. **SharedArrayBuffer audio ring** where a host can provide cross-origin
   isolation.
6. **Link cable** between two cores (the serial port already clocks bits).
