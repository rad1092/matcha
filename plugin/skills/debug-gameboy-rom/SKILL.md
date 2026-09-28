---
name: debug-gameboy-rom
description: Debugs Game Boy homebrew and test ROMs in the matcha emulator using breakpoints, watchpoints, single-stepping, disassembly, memory dumps, VRAM tile views and serial output. Use when a .gb ROM crashes, hangs, shows garbage graphics, a test ROM fails, or when asked to explain what some Game Boy code is doing.
---

# Debugging a Game Boy ROM with matcha

matcha passes the SM83 SingleStepTests, the DMG Mooneye suite and dmg-acid2,
so when a ROM misbehaves, suspect the ROM before the emulator. The one known
gap: mid-scanline PPU effects (writes to SCX/BGP/LCDC during mode 3) render
per line, not per pixel.

Hardware facts (memory map, registers, interrupt vectors, test-ROM result
protocols) are in `references/hardware.md`. Read it when a question depends
on an address or register meaning.

## Triage in this order

1. `load_rom`, then `run` about 120 frames and look at the screenshot.
2. `state` — read these first:
   - `CPU locked`: an illegal opcode ran. `disassemble` a few bytes before PC
     to find how execution got into data (a bad jump table, a `ret` with a
     corrupted stack, a missing bank switch).
   - `CPU halted` with `IE=00` or IME=0 and nothing pending: the program
     waits for an interrupt that can never arrive.
   - PC stuck in a tiny loop: `disassemble` at PC and read what it polls
     (often LY, STAT, or a flag in WRAM an interrupt handler never sets).
   - LCDC bit 7 clear: display is off (blank screen is expected).
3. Graphics wrong: `tiles` shows what reached VRAM. Missing tiles mean the
   copy never happened or happened while VRAM was locked (writes during mode
   3 are dropped); correct tiles with a wrong picture point to the tile map
   ($9800/$9C00), LCDC tile-data select, or palettes (BGP $FF47).

## Breakpoints and watchpoints

- `breakpoint` with `kind: "exec"` stops before the instruction at an
  address runs; `run`/`press` report the stop. Then `step` to walk through
  instructions one by one (each call returns registers and the next code).
- `kind: "write"` stops right after any instruction writes the address and
  reports the writing PC — the fastest way to answer "who clobbers this
  variable?". `kind: "read"` finds readers.
- Stepping into an interrupt shows as `interrupt` in the step summary; the
  handler starts at $40 (VBlank), $48 (STAT), $50 (timer), $58 (serial) or
  $60 (joypad).
- `breakpoint` with `action: "clear"` removes everything when done.

## Memory

- `read_memory` never has side effects (reading $FF00-$FF7F does not
  acknowledge anything), so dump I/O registers freely.
- The stack lives where SP points (usually HRAM $FFFE down, or WRAM). A
  corrupted return address shows as a strange value in `read_memory` at SP.
- Switchable ROM ($4000-$7FFF) shows the bank currently mapped (`state`
  reports it). Disassembling $4000+ shows that bank only.

## Test ROMs

- Blargg: output arrives over the link port — check `serial_output` for
  "Passed" or "Failed" with the failing sub-test, and cartridge RAM at $A000
  (status byte, then signature DE B0 61 at $A001, then text).
- Mooneye / dmg-acid2 / Mealybug: the test ends at `ld b, b` (opcode $40).
  Mooneye passes when B,C,D,E,H,L = 3,5,8,13,21,34; all $42 means failure.

## Report

State the root cause with the evidence that proves it (addresses,
disassembly lines, register values), the fix or next experiment, and which
save slot or breakpoint reproduces it.
