# Roadmap

Ordered by what the [corpus study](docs/analysis.md) says real software
needs, then by what the test suites still flag. Each item says why, what,
how to verify, and the first concrete step, so it can be picked up cold.

## 1. CGB compatibility and timing refinement

**Implemented.** Native CGB mode now has RGB555 rendering, VRAM attributes
and banks, WRAM banks, KEY1 double speed, GDMA/HDMA, CGB object priority,
fast serial and model-specific sound behavior. cgb-acid2 and cgb-acid-hell
match their reference screenshots. Hosts auto-select CGB for cartridges
with the colour-capability flag. See ADR-0010 and `docs/CONFORMANCE-CGB.md`.

**What remains.** DMG-on-CGB compatibility palettes/KEY0 and boot-ROM state,
exact speed-switch access gates, DMA contention and mid-line timing. The
current CGB test runner explicitly excludes compatibility-mode tests whose
reference requires a mode that is not yet implemented; it lists every
excluded ROM and reason. Implement that mode before enabling those cases.

**Verify.** Preserve every named DMG and CGB baseline pass, retain exact
acid screenshots, then rerun the affected corpus cases. Full automatic
DMG/CGB corpus revalidation lives in `docs/cgb-analysis.md`; outcome
agreement is a coarse signal, not proof of game or pixel compatibility.

**First step.** Select a failing CGB DMA/speed-change family and trace its
bus phases against the test assembly and a pinned primary reference.
Alternatively, introduce a hardware-model/program-mode distinction for
DMG software on CGB, covering palette indirection and disabled CGB ports
before enabling the excluded compatibility references.

## 2. Remaining pixel-fetch and window quirks

**Why.** 39% of the homebrew that runs takes STAT interrupts — the tool for
raster effects. The new FIFO applies register changes during mode 3, but
the remaining Mealybug and Gambatte failures still expose fetch, window
restart and register-write collision details.

**Implemented.** A background/window fetcher and BG/OBJ queues advance
per dot during mode 3. Live palettes, independent bitplane fetches,
scrolling, window restarts and object stalls determine the picture and
HBlank boundary. The event scheduler remains the fast path outside drawing
(ADR-0009). In-flight state is serialized and validated.

**What remains.** Use the named failing ROMs in `docs/conformance.json`
to refine LCDC bitplane/address changes, window and object-fetch conflicts.
Do not introduce ROM-specific offsets or replace hardware screenshots.

**Verify.** Mealybug (24 screenshots), dmg-acid2 and every current test
stay green; `matcha run` stays above 30× real time.

**First step.** Compare a remaining single-register Mealybug failure with
its hardware screenshot and assembly, then trace the relevant fetch phase.
The palette-change cases now pass and provide a timing anchor.

## 3. Quality and reach

- **Band-limited audio** (ADR-0005 option B) if the box filter's aliasing at
  high square-wave pitches becomes noticeable.
- **Link cable**: two cores stepping in lockstep through the serial port —
  for two-player homebrew, and for two agents playing each other over MCP.
  Only 0.6% of the corpus takes serial interrupts, so this is for fun.
- **SGB borders and palettes**: 64 corpus cartridges set the SGB flag.
- **Embedded port**: the core is `no_std`; an ESP32-S3 or RP2350 with a
  small LCD is a good showcase (the APU uses `f32`, so prefer a chip with an
  FPU).
- **Mappers**: none needed yet — every corpus cartridge that passes the boot
  ROM's logo check uses MBC1/2/3/5 or no mapper; the 31 entries with other
  type bytes all fail that check, so they are not bootable cartridge images.
