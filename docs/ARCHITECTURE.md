# matcha — system design

matcha is a Game Boy (DMG) and native Game Boy Color (CGB) emulator written
as one portable Rust core with three hosts: a single-file web player, a Model Context
Protocol (MCP) server that lets an AI agent play and debug games, and a
native command-line tool for conformance testing and profiling.

This document describes the system as built. The reasoning behind each
major choice is recorded in the ADRs under [`docs/adr/`](adr/).

## 1. Requirements

### Functional

| # | Requirement | Where |
|---|---|---|
| F1 | Run DMG and native CGB software: CPU, PPU, APU, timer, serial, joypad, OAM/VRAM DMA, MBC1/1M/2/3(+RTC)/5 | `crates/matcha-core` |
| F2 | Play in a browser with sound, keyboard/touch/gamepad input, battery saves, save states and rewind | `web/` |
| F3 | Inspect a running game: registers, disassembly, breakpoints, watchpoints, VRAM, audio channels, memory search | web debugger panels, MCP tools |
| F4 | Let an AI agent load, play, screenshot, inspect and debug ROMs through tools | `mcp/server.mjs`, `plugin/` |
| F5 | Measure accuracy against public test suites and publish separate DMG and native CGB scoreboards | `matcha test`, `docs/CONFORMANCE.md`, `docs/CONFORMANCE-CGB.md` |
| F6 | Profile and trace ROMs headlessly (opcodes, interrupts, memory regions, CPU idle time; instruction traces) for corpus analysis and debugging | `matcha profile`, `matcha trace`, `analysis/` |

### Non-functional

| Property | Target | Status |
|---|---|---|
| Accuracy | Pass the CPU, timing, sound and PPU suites real games depend on; agree with a reference emulator on real software | SST 498,000/498,000; DMG 1,789/1,945, including Mooneye 94/94, dmg-acid2 1/1, Blargg 43/43, Gambatte 1,642/1,783 and Mealybug 9/24 ([DMG scoreboard](CONFORMANCE.md)). Native CGB: 2,398/3,331 with 33 explicitly excluded compatibility-mode cases and known failures retained ([CGB scoreboard](CONFORMANCE-CGB.md)). |
| Speed | Full speed with audio in a browser on a phone-class CPU | Current native build: 44–49× across the four bundled games, 40–44× with 48 kHz audio on Apple silicon. Phone performance has not been measured. |
| Determinism | Same ROM + inputs + state ⇒ identical frames and audio, bit for bit | Unit-tested (two machines, save/replay) |
| Portability | One core for browser, Node and native; no OS services | `no_std` + `alloc`, zero dependencies |
| Safety | Any byte sequence is a valid ROM or state input: no panics, no UB | `#![forbid(unsafe_code)]` in the core; bounds-checked state reader; atomic state load |
| Footprint | Small enough to embed in one HTML file | Shared WASM: 186,123 bytes (181.8 KiB), no imports |
| Install | Plugin works with nothing but Node | MCP server has no npm dependencies |

### Constraints

- One maintainer; the design must stay readable by one person (and by an AI
  assistant resuming the work — see [`CONTINUE.md`](../CONTINUE.md)).
- Test ROMs and the homebrew corpus are fetched, never committed.
- The published page must be a single self-contained HTML file (the artifact
  host only allows scripts from a few CDNs; matcha needs none).
- `Model::Cgb` selects native CGB hardware. CGB's DMG-compatibility mode and
  execution of the CGB boot ROM are not implemented; automatic model
  selection uses DMG for cartridges without the CGB header flag.

## 2. High-level design

```
                  ┌──────────────────────── matcha-core (no_std, safe Rust) ────────────────────────┐
                  │                                                                                 │
   ROM bytes ───▶ │  Cartridge ◀──┐                                                                 │
                  │  (MBC, RTC,   │        ┌─────────── SystemBus ────────────┐                     │
                  │   battery)    ├─ read/ │ memory map · IE/IF · OAM/VRAM DMA│                     │
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
                 └───────────────┬──────────┘              │ trace · disasm · info  │
                                 │ matcha.wasm             └───────────┬────────────┘
                     ┌───────────┴────────────┐                        │
                     │ web/matcha.js wrapper  │              docs/conformance{,-cgb}.json
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
| `cpu.rs` | SM83 interpreter over the `CpuBus` trait | One bus access per M-cycle, EI delay, HALT bug, IE-push dispatch quirk, STOP and CGB speed-switch hooks, model-specific HALT sampling, illegal-opcode lock; internal address-bus cycles use `idle_at` |
| `bus.rs` | Memory map, interrupts, OAM DMA, CGB GDMA/HDMA, clocking | Implements `CpuBus`; WRAM banking/echo, DMA bus conflicts, KEY1 and CPU/base-clock separation (ADR-0010) |
| `ppu.rs` | LCD timing, STAT/LY interrupts, access blocking, rendering | Per-dot BG/OBJ FIFOs (ADR-0009); CGB VRAM banks, tile attributes, RGB555 palettes and priority; DMG OAM corruption remains DMG-only |
| `apu.rs` | Four channels, frame sequencer, mixer, resampler | Cached mix + box filter (ADR-0005); model-specific wave RAM, length-counter and high-pass behavior |
| `timer.rs` | 16-bit system counter, falling-edge TIMA, DIV-APU and serial clocks | TIMA reload state machine; APU divider bit changes in double speed |
| `cartridge.rs` | Header parsing, MBC1/1M/2/3/5, RTC, battery RAM | RTC runs on emulated time only |
| `joypad.rs`, `serial.rs` | P1 matrix and interrupts; serial port with captured output | CGB fast serial clock; captured output feeds Blargg tests and MCP; no link-cable peer |
| `state.rs` | Versioned binary writer/reader | ADR-0006 |
| `disasm.rs` | RGBDS-syntax disassembler | Used by the debugger, MCP and CLI |
| `profile.rs` | Opt-in execution profile | Opcode counts, cycles by state, memory regions, coverage |
| `lib.rs` | `GameBoy` facade | Explicit hardware model, deterministic post-boot state, power-on RAM (zeros, or seeded DMG-like noise), run loop, breakpoints, color conversion, public API |

## 3. How time works

The CPU drives the clock. Every CPU memory access is a bus call that first
performs the access and then advances the machine by one M-cycle: four CPU
T-cycles, corresponding to four LCD/base-clock dots normally or two dots in
CGB double speed. Internal CPU cycles call `idle()`, which only advances
time. This "access, then tick" order makes each access land at a fixed phase of the
M-cycle, which is what the PPU access windows and interrupt timings are
measured against (ADR-0001).

`SystemBus::tick` for one M-cycle:

```
cycles += 1
base_dots = double_speed ? 2 : 4
base_clock_ticks += base_dots
timer.tick()          → four CPU T-cycles: TIMA, DIV-APU edge, serial clock
ppu.tick_dots(base_dots) → interrupt edges within this M-cycle
apu                   → DIV-APU sequencing; channels/mix once per four base dots
tick_dma()            → one OAM DMA byte
cart.tick_rtc(base_dots) → MBC3 clock on emulated base time
```

Running instructions sample the current interrupt lines at the instruction
boundary. An ongoing DMG HALT samples halfway through its idle M-cycle;
late PPU, timer and serial edges are hidden from that earlier sample
(`if_deferred`). CGB HALT samples before the idle cycle. Including timer and
serial edges in the DMG late sample fixes 47 hardware cases without changing
their event timing. Interrupt entry takes five M-cycles, with stack writes
at CPU T-cycles 12/16 and acknowledgement at T-cycle 18. Edges after
acknowledgement can reassert IF.

KEY1-armed STOP resets DIV and switches CGB CPU speed. With no pending
interrupt it pauses CPU-side clocks for 2,050 M-cycles while LCD, audio and
RTC base time continue. A pending interrupt changes the STOP padding and
pause behavior. APU frame sequencing uses DIV bit 12 normally and bit 13 in
double speed, so audio pitch and cadence stay tied to base time.

GDMA stalls the CPU while copying; HDMA schedules a 16-byte block in each
eligible visible HBlank and pauses while the CPU is halted. Each block
uses 32 base dots plus the modeled transfer overhead. Both modes use the
selected VRAM bank and continue ticking other devices during the stall.
The source/destination registers, pending block and remaining length are
machine state. Fine DMA and speed-switch access-gate timing still has known
conformance failures (ADR-0010).

### Keeping it fast without losing accuracy

Three mechanisms keep the per-M-cycle work small:

1. **PPU event scheduling.** Outside mode 3, the PPU skips M-cycles without
   a mode switch, STAT/LY comparison, delayed register write or access edge.
   During mode 3 the tile fetcher and pixel queues advance every dot, so
   mid-line register writes affect the correct fetched or output pixels
   (ADR-0009). The last visible pixel determines the HBlank boundary.
2. **HALT fast path.** A halted CPU loops on `idle()` inside `Cpu::step`
   until an interrupt is pending or the bus says to yield (end of frame,
   breakpoint budget). No instruction decode happens while halted — and
   homebrew made with GB Studio or GBDK spends more than half its time halted
   (see [`analysis.md`](analysis.md)).
3. **Cached audio mix.** Channels report when their output level changes; the
   stereo mix is recomputed only then, and each four-base-dot interval adds
   the cached value to an accumulator.

## 4. Frontends and data flow

### One frame in the browser

```
requestAnimationFrame(now)
  ├─ rewinding?  pop the newest snapshot from the rewind ring → loadState
  └─ running?    accumulate elapsed time (×4 when fast-forwarding);
                 nudge it when the audio queue leaves 30–120 ms
                 for each whole frame owed (at most 3; 8 when fast-forwarding):
                   gb.setButtons(keyboard | touch | gamepad)
                   gb.runFrame()                 ── WASM: one LCD frame
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
| Core ↔ hosts | `GameBoy` Rust API (`new_with_model`, `run_frame`, `step`, `render_rgba`, `base_clock_ticks`, audio, states, memory and debugger methods); `GameBoy::new` still defaults to DMG | `crates/matcha-core/src/lib.rs` |
| Rust ↔ JavaScript | `extern "C"` exports over an opaque handle; bulk data in handle-owned buffers; errors via `matcha_last_error_*` | `crates/matcha-wasm/src/lib.rs` (ADR-0004) |
| JS wrapper | `loadMatcha(bytes)` → `MatchaModule.create(rom, {model: "auto"})` → `Emulator` methods; shared by browser and Node | `web/matcha.js` |
| Agent ↔ server | MCP over stdio, protocol 2024-11-05 … 2025-11-25; 17 tools | `mcp/server.mjs` (ADR-0007) |
| Shell ↔ CLI | `matcha info/run/test/profile/trace/disasm`; ROM execution accepts `--model auto/dmg/cgb`; conformance defaults to explicit DMG and has a separate CGB baseline | `crates/matcha-cli/src/main.rs` |

## 5. State and storage

| Data | Format | Lives in | Size |
|---|---|---|---|
| Save state | `MTCHST` + version + ROM id + components (ADR-0006) | memory, model-specific `localStorage` quick-save slot, MCP named slots | Fixed state ≈ 116.3 KiB (119,120 bytes), plus cartridge RAM |
| Rewind history | ring of save states | browser memory | 400 snapshots ≈ 47.6–50.9 MB (45–49 MiB) with 0–8 KiB cartridge RAM; larger battery RAM increases this |
| Battery RAM | raw cartridge RAM (+ RTC for MBC3) | `localStorage` (web), `.sav` file (CLI) | 0–128 KiB |
| Conformance results | JSON + Markdown | `docs/` | — |
| Corpus manifest and profiles | CSV + JSON | `analysis/data/` | — |

The current snapshot format is version 4. It includes the selected model,
banked memory, CGB palettes and color framebuffer, CPU speed and base-clock
phase, GDMA/HDMA state, in-flight pixel queues, interrupt phases and recent
fetch-address latches. Loading a snapshot cannot change the machine's model:
cross-model snapshots and versions 1–3 are rejected. Battery RAM uses its
separate, unchanged format.

Measured format-4 snapshots are 119,120 bytes for Libbet (CGB, no cartridge
RAM) and 127,312 bytes for Tobu Tobu Girl (DMG, 8 KiB cartridge RAM). Both
models save the same component layout. The earlier ≈ 39 KiB + cartridge RAM
snapshot and 16–19 MB rewind-ring figures describe the pre-CGB build.

Loading a state is atomic: the current state is saved first and restored if
any field fails validation, so a bad file can never leave a half-loaded
machine.

## 6. Performance

The native CGB build was measured on 2026-09-30 with a release build on an
idle Apple silicon Mac (Rust 1.98.1), `--frames 3600 --input monkey` and
automatic model selection, one game at a time:

| Bundled game | Model | Video only (× real time) | With 48 kHz audio |
|---|---|---:|---:|
| 2048 | DMG | 47.6 | 43.8 |
| Libbet | CGB | 49.1 | 41.6 |
| Shock Lobster | DMG | 44.5 | 41.2 |
| Tobu Tobu Girl | DMG | 44.4 | 40.1 |

All four exceed the roadmap's 30× native target. The historical pre-CGB
FIFO build measured 45–53× without audio and 41–48× with audio; Libbet now
runs as CGB, so that row is not a comparison of identical hardware modes.
A same-machine pre-FIFO run measured 115–154× without audio: the extra
per-dot work has a real cost. These native timings do not establish
browser performance on a phone.

The following historical measurements predate the pixel FIFO. They were
measured with `matcha run --seconds 60 --input monkey` (release build,
native, one ROM at a time on one core of the 2-vCPU build container):

| Configuration | Speed (× real time) |
|---|---|
| Before optimisation (per-dot PPU loop, uncached mixer), one game | 18 |
| 150 random corpus programs, video only (`analysis/bench.py`) | median 52, slowest 37, fastest 155 |
| The four bundled games, video only | 53–66 |
| The four bundled games, with 48 kHz audio (`--audio`) | 40–47 |

Speed tracks how busy the game keeps the CPU: halted time is skipped in
bulk, so programs that wait for VBlank with HALT emulate fastest (see
[`analysis.md`](analysis.md), "Emulation speed").

A normal LCD frame is 70,224 base dots (16.74 ms of Game Boy time): 17,556
CPU M-cycles at normal speed or 35,112 at double speed. Profiling elapsed
time uses `base_clock_ticks`, so a speed switch cannot double the reported
emulated duration. Fast-forward runs at four times speed (at most eight
emulated frames per display frame).

## 7. Reliability and safety

- **Untrusted input.** ROMs, save states and battery files come from users.
  Unknown mapper types are rejected with an error; ROMs are truncated to what
  their mapper can address (8 MiB at most); every other byte sequence runs.
  Illegal opcodes lock the CPU (as on hardware) instead of panicking. The
  state reader is bounds-checked and validates every enumeration, counter and
  register that could break an invariant; a targeted fuzz test loads
  thousands of corrupted states with overflow checks on.
- **No unsafe in the core.** `matcha-core` forbids `unsafe`; the unsafe code
  is confined to the thin WASM export layer, each function documenting its
  pointer contract.
- **Deterministic by construction.** The core never reads a clock or a random
  source; the MBC3 RTC advances with emulated cycles, and hosts that want
  wall-clock time call `rtc_advance_seconds` explicitly.
- **Failures are visible.** The CLI's test runner reports each ROM as pass,
  fail or error, and `--baseline` turns any newly failing ROM into a CI
  failure; CI sets `MATCHA_REQUIRE_TESTDATA=1` so missing test data fails
  instead of silently skipping. DMG and CGB run against separate committed
  baselines, and CI uploads both result sets. The CGB report lists 33
  incompatible cases separately; exclusions never count as passes.
- **Checked against a second emulator.** The historical pre-FIFO corpus
  study ran 868 homebrew programs through matcha and SameBoy with identical
  input and compared outcomes and screens; 866 outcomes agreed, and it
  found three bugs the test suites had missed
  ([`analysis.md`](analysis.md)).

## 8. Trade-offs

| Choice | Gained | Given up | ADR |
|---|---|---|---|
| CPU-driven M-cycle ticking | Simple, exact interleaving of CPU accesses and device state | Every access pays a function call per device | 0001 |
| Per-dot BG/OBJ FIFO during mode 3, event scheduling elsewhere | Mid-scanline register effects and fetch-driven transfer length | More work per pixel; some hardware quirks remain | 0009 (supersedes 0002) |
| Explicit DMG/native CGB model in the shared core | Banked color rendering, CPU speed switching and DMA across all hosts | More saved state; CGB compatibility mode and some revision-specific timing remain unsupported | 0010 |
| `no_std`, zero dependencies, safe Rust | Runs anywhere; auditable | Writing small utilities (hashing, `powf`) ourselves | 0003 |
| Hand-written C ABI | One `.wasm` for browser and Node; no toolchain coupling | Manual glue; `unsafe` at the boundary | 0004 |
| Box-filter audio | Cheap, deterministic, sounds right for chiptune | Slight aliasing at very high pitches | 0005 |
| Hand-rolled state format | No dependencies; explicit, validated layout | Version bump on any layout change | 0006 |
| Hand-written MCP server | Install = copy a folder | Protocol features added by hand | 0007 |
| Test suites as the definition of done | Objective progress; loud regressions | Pressure toward sub-cycle behaviour few games use | 0008 |

## 9. What to revisit as it grows

In priority order (details in [`ROADMAP.md`](../ROADMAP.md)):

1. **CGB conformance and compatibility.** Refine speed-switch, DMA, access
   windows and interrupt phases against the separate native CGB baseline;
   implement DMG-on-CGB compatibility and boot behavior as explicit work.
2. **Remaining pixel-fetch/window quirks** in Mealybug and Gambatte,
   preserving all currently passing cases.
3. **Band-limited audio** if audio quality becomes a priority.
4. **SharedArrayBuffer audio ring** where a host can provide cross-origin
   isolation.
5. **Link cable** between two cores (the serial port already clocks bits).
