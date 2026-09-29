# ADR-0009: Advance the pixel fetcher and FIFOs during mode 3

**Status:** Accepted
**Date:** 2026-09-30

## Context

The scanline renderer in ADR-0002 sampled all registers at mode-3 entry.
Changing a palette, scroll position or tile-map selection while the LCD was
drawing therefore affected either the whole line or none of it. Mealybug
passed only 1/24 cases despite the existing timing and CPU suites passing.

The OAM corruption implementation and optional seeded power-on RAM are
already part of the baseline. Replacing their timing or reimplementing
them is outside this change.

## Decision

Retain the event scheduler for mode switches, interrupt edges, LY/LYC and
memory-access windows. During mode 3, advance a separate pixel pipeline
once per dot; outside mode 3, retain the existing M-cycle fast path.

The pipeline stores colour indices, rather than final palette shades:

- The background/window fetcher has separate tile-address, tile-read,
  low-plane address/read and high-plane address/read phases. Live SCX,
  SCY and LCDC values affect the appropriate fetch, including independent
  DMG bitplane addressing.
- An eight-pixel background queue is refilled when empty. Initial junk
  pixels and fine scrolling consume dots without writing the visible
  framebuffer.
- Window triggering restarts the fetcher and tracks the internal window
  row. Object fetch stalls share the transfer timeline; an object queue
  retains transparent-colour, palette and behind-background attributes.
- The output stage combines the queues and applies the live palettes.
  The actual last visible pixel determines the end of mode 3. STAT and
  access-window events continue to use that endpoint.

Use the hardware tests to resolve reference differences. In particular,
the first line samples WY on the mode-2 comparison clock; an extra check
at dot 0 latches the previous frame's LY=0 too early and fails Gambatte's
`window/{arg/,}late_wy_1`. This boundary is covered by a focused unit test.

Every in-flight fetch and queue field belongs to the emulated state.
Serialize and validate it, and bump the save-state format to version 2.
Version-1 snapshots are rejected with the existing version error; cartridge
battery-save bytes are a separate format and are unaffected.

### Register-address refinement

Scroll and LCDC writes reach the fetcher at different dot phases. Preserve
which address phase actually ran on the recent dots, including its pixel
position, so a write can update the affected address without changing an
older VRAM read. Tracking performed work is essential: an object stall can
hold the fetcher at a phase without performing a new address lookup.

These additional latches are part of the machine state; the refinement
advances the snapshot format to version 3. Battery-save format is unchanged.

The register phases follow the DMG cases in
[SameBoy's write-conflict handling](https://github.com/LIJI32/SameBoy/blob/213a12ce93d66b105a113debd9396306066a7cfc/Core/sm83_cpu.c):
SCY and the full LCDC value are visible one dot earlier, SCX two dots earlier.
SCX can therefore affect the map byte that has just been read as well as an
address waiting to be read. It does not change window addressing.

Object height is sampled when each bitplane is read, after the fetcher's
wait. A size write colliding with the last dot can replace that plane read.
For a high-plane collision, restore the eight queued object pixels from
before the overlay and apply it again with the corrected data. This
preserves earlier objects' priority and removes pixels that become
transparent; simply drawing over the old overlay cannot do either.
The read marker expires each dot, and its queue backup is validated and
serialized with the other in-flight state.

## Verification

The committed conformance scoreboard is the regression baseline. A net
gain cannot hide a formerly passing ROM that now fails: individual cases
are compared by name. Keep the hardware references and test exclusions
unchanged while evaluating the renderer.

In addition to the ROM suites, test palette changes part-way through a
line, independent bitplane fetching, static transfer lengths, and queue
validation. Restore full machines from multiple instruction boundaries
inside mode 3, including a scene with overlapping objects and a window;
replay register writes and compare the complete state and output.

Rebuild the single shared WASM file and run MCP tests before publishing
web or plugin bundles. The generated scoreboard and `CONTINUE.md` record
the measured result and remaining hardware quirks; the presence of a FIFO
alone does not imply that every Mealybug case passes.

## Sources

- [Pan Docs: Pixel FIFO](https://gbdev.io/pandocs/pixel_fifo.html)
- [SameBoy's display pipeline](https://github.com/LIJI32/SameBoy/blob/master/Core/display.c)
- [SameBoy's register-write conflicts](https://github.com/LIJI32/SameBoy/blob/master/Core/sm83_cpu.c)
- [Mealybug PPU measurements](https://github.com/mattcurrie/mealybug-tearoom-tests/blob/master/the-comprehensive-game-boy-ppu-documentation.md)
