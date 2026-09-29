# ADR-0010: Add native CGB hardware to the shared core

**Status:** Accepted
**Date:** 2026-09-30

## Context

The DMG core cannot run CGB-only cartridges. Native color support requires
more than converting the framebuffer: tile and object attributes select
VRAM banks and palettes, CPU speed can change independently of the LCD,
and VRAM DMA stalls the CPU while other devices continue running.

The existing CPU bus boundary and dot-based PPU allow these behaviors to
share the same core across CLI, browser and MCP. Keeping DMG as a distinct
model is necessary to preserve its tested interrupt phases, OAM corruption
and audio behavior.

## Decision

### Explicit hardware model

Add `Model::{Dmg, Cgb}` to machine options. `GameBoy::new` keeps its DMG
default; `GameBoy::new_with_model` selects explicitly. Hosts can select
automatically from cartridge header bit 7 at `$0143`, choosing CGB for
color-capable cartridges and DMG otherwise. Reset preserves the selection.

`Model::Cgb` currently means native CGB hardware. It does not emulate
DMG-on-CGB compatibility mode or execute the CGB boot ROM. The deterministic
post-boot initialization provides the registers needed to start a native
cartridge; it does not reproduce every boot-duration-dependent divider or
LCD phase. Test selection must state this distinction rather than count
inapplicable compatibility references as failures or passes.

### One CPU bus, two clock domains

Keep a CPU M-cycle equal to four CPU T-cycles. Advance the PPU and RTC by
four base dots at normal speed or two in CGB double speed. Record elapsed
base time separately from CPU M-cycles so profiling, frame budgets and
host scheduling measure the same emulated duration in both speed modes.

Timer, serial and OAM DMA retain CPU-clock timing. APU channel timers and
sample generation follow base time, and DIV-APU uses divider bit 12 normally
or bit 13 in double speed. CGB fast serial selects the faster divider clock.

KEY1-armed STOP resets DIV, toggles CPU speed and clears the preparation
flag. With no pending interrupt, the modeled 2,050-M-cycle pause stops
CPU-side clocks while LCD/audio/RTC time advances. Pending-interrupt STOP
preserves its different padding-byte and pause behavior. HALT interrupt
sampling remains model-specific: ongoing DMG HALT samples halfway through
the idle cycle; CGB samples before it.

The same clock audit fixes a DMG sampling omission: late timer and serial
interrupts now join late PPU edges in the deferred half-cycle sample.
Running instructions still observe current IF. This changes visibility at
HALT's sample, without shifting the timer or serial event itself.

### Banked memory, DMA and rendering

- Provide 32 KiB WRAM with SVBK-controlled banks and echo mapping, plus two
  VRAM banks selected by VBK. SVBK bank zero aliases bank one.
- Implement GDMA and HBlank DMA with 16-byte blocks, CPU stalls, selected
  VRAM-bank destinations, cancellation, address wrapping and saved pending
  transfers. Blocks consume 32 base dots plus modeled transfer overhead.
  HDMA pauses during HALT and resumes when an eligible HBlank is available.
- Preserve separate CGB DMA buses; disable the DMG-only OAM corruption bug
  on CGB.
- Extend the existing pixel fetcher with CGB tile attributes, flips,
  per-tile/object VRAM bank and palette selection, and CGB BG/OBJ priority.
  Palette ports obey rendering access locks and auto-increment behavior.
- Preserve a native RGB555 framebuffer and convert it for all host image
  outputs. The existing color-index framebuffer remains available for
  debugging. User-selected four-color tints apply to DMG only.
- Use model-specific APU wave-RAM access, power-off length behavior,
  retrigger behavior and high-pass parameters. Expose CGB PCM registers;
  serial capture remains local rather than a link-cable implementation.

### Snapshot format 4

Serialize and validate all new memory banks, palettes, framebuffer data,
speed/base-clock phases, DMA state and model-specific component state.
Advance `STATE_VERSION` to 4. A state load cannot change the machine model;
cross-model snapshots and older snapshot versions are rejected atomically.
Battery saves retain their existing separate format. Browser quick-save
keys include the selected model.

The bus HALT gate is derived from CPU power state. CPU wake transitions
update it immediately so a snapshot taken just after wake already contains
the pending HDMA block. Loading that snapshot must not rely on a later
synthetic HALT transition to resume DMA.

### Separate conformance baselines

Keep the existing DMG baseline in `docs/conformance.json` and add native
CGB results in `docs/conformance-cgb.json`. Give CGB suites distinct
identities and use CGB C image references where silicon revision matters.
CI runs both models and uploads both JSON/Markdown result sets. Each model
must preserve its individually passing cases; a net gain cannot hide a
regression.

The initial native CGB selection contains 3,331 cases and records 33
excluded compatibility-mode cases with reasons: three Mooneye boot cases,
three Gambatte cases and 27 Mealybug cases. These references require a
DMG-compatible CGB boot path that this model does not implement. Exclusions
are absent from the pass total but remain visible in the report. Ordinary
shared CPU/timer/interrupt diagnostics remain selected. Gambatte audio
judges and SameSuite APU/SGB tests remain outside this scoreboard.

## Verification and current limits

The implementation preserves all earlier DMG named passes and adds 47
timer/serial interrupt-sampling passes: Gambatte is 1,642/1,783 and the
full DMG scoreboard is 1,789/1,945. CPU SingleStepTests remain a separate
498,000-case check. Native CGB starts at 2,398/3,331, including exact
`cgb-acid2` and `cgb-acid-hell` images and all six selected SameSuite tests.
See the generated [DMG](../CONFORMANCE.md) and
[CGB](../CONFORMANCE-CGB.md) reports for individual failures.

Focused tests cover memory-bank/echo behavior, DMA stalls at both speeds,
HDMA scheduling and cancellation, banked rendering and priorities, palette
ports, speed-switch interrupts, audio/serial differences, and whole-machine
save/replay. Snapshot tests include color rendering and a HALT-wake HDMA
boundary, plus invalid and cross-model states.

Native CGB support is usable but does not imply complete cycle accuracy.
Known failures remain in double-speed access windows, register/fetch
collisions, speed-switch timing, DMA boundaries, boot phases and some
interrupt/timer details. Speed-switch VRAM/OAM access-gate freezing and
hardware-instance-dependent timer glitches are not fully modeled.
DMG-on-CGB compatibility, a CGB boot-ROM path and link-cable/infrared peers
remain separate work. Performance and size measurements belong in the
[architecture document](../ARCHITECTURE.md), with their measured hardware
model and build context.

## Sources

- [Pan Docs: CGB registers and speed switching](https://gbdev.io/pandocs/CGB_Registers.html)
- [Pan Docs: OAM DMA transfer](https://gbdev.io/pandocs/OAM_DMA_Transfer.html)
- [Pan Docs: Timer obscure behavior](https://gbdev.io/pandocs/Timer_Obscure_Behaviour.html)
- [Pan Docs: Audio details](https://gbdev.io/pandocs/Audio_details.html)
- [Pan Docs: Power-up sequence](https://gbdev.io/pandocs/Power_Up_Sequence.html)
- [SameBoy CPU, STOP and HALT behavior](https://github.com/LIJI32/SameBoy/blob/213a12ce93d66b105a113debd9396306066a7cfc/Core/sm83_cpu.c)
- [SameBoy memory and VRAM DMA behavior](https://github.com/LIJI32/SameBoy/blob/213a12ce93d66b105a113debd9396306066a7cfc/Core/memory.c)
- [SameBoy display pipeline](https://github.com/LIJI32/SameBoy/blob/213a12ce93d66b105a113debd9396306066a7cfc/Core/display.c)
- [SameSuite LCD-off HDMA test](https://github.com/LIJI32/SameSuite/blob/master/dma/hdma_lcd_off.asm)
- [SameSuite mode-0 HDMA test](https://github.com/LIJI32/SameSuite/blob/master/dma/hdma_mode0.asm)
