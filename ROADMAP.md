# Roadmap

Ordered by what the [corpus study](docs/analysis.md) says real software
needs, then by what the test suites still flag. Each item says why, what,
how to verify, and the first concrete step, so it can be picked up cold.

## 1. Game Boy Color mode

**Why.** 367 of the 1,266 Homebrew Hub cartridges (29%) are CGB-only and
cannot run at all. Among the 306 "CGB-enhanced" ones matcha does run as a
DMG, 27 hang in STOP attempting a CGB speed switch and 29 never draw a
picture (19 of them both) — on an original Game Boy they fail the same way,
but the people who made them expect a Color.

**What.** A `Model` choice at construction (`Dmg` | `Cgb`), CGB boot state,
double-speed mode (KEY1 + STOP), VRAM bank 1 and background attributes
(palette, bank, flips, priority), WRAM banks 1–7 (SVBK), background/object
palette RAM (BCPS/BCPD/OCPS/OCPD), general-purpose and HBlank HDMA, CGB
object priority (by OAM index), and the CGB's different PPU timing quirks.
The bus/PPU split already isolates these; the renderer's per-pixel colour
lookup becomes palette-RAM based.

**Verify.** cgb-acid2 and cgb-acid-hell (pixel-exact), Mooneye's CGB
acceptance tests, SameSuite, Blargg `cgb_sound`; then rerun the corpus with
`reference/` switched to SameBoy's CGB model and compare as for DMG.

**First step.** Add `Model` to `GameBoy::new_with_model`, thread it into
`SystemBus::new`, and implement WRAM/VRAM banking with a unit test; bump
`STATE_VERSION`.

## 2. Pixel FIFO renderer

**Why.** 39% of the homebrew that runs takes STAT interrupts — the tool for
raster effects — and Mealybug Tearoom, which checks mid-scanline register
changes, passes 1 of 24. The line renderer draws each line from the
registers as they are at the start of mode 3.

**What.** Keep the event scheduler and the measured mode-3 lengths (all
Mooneye timing tests depend on them); replace `render_line` with a
background fetcher + pixel FIFO advanced dot by dot through mode 3, so
SCX/SCY/BGP/LCDC/WX writes take effect mid-line. Only mode 3 needs per-dot
work, so the fast path elsewhere stays.

**Verify.** Mealybug (24 screenshots), dmg-acid2 and every current test
stay green; `matcha run` stays above 30× real time.

**First step.** Port the Mealybug `m3_bgp_change` case first: it only needs
BGP sampled per pixel.

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
