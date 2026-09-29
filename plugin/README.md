# matcha — Game Boy for Claude

Gives Claude a Game Boy / native Game Boy Color it can look at, play
and debug. Everything runs locally in Node; no network, no dependencies.

## What Claude can do with it

- **Play and test ROMs** — boot a `.gb` or `.gbc`, read the screen, tap or hold
  buttons, let time pass, save and restore states (emulation is
  deterministic, so any moment can be replayed exactly).
- **Debug homebrew** — breakpoints, write/read watchpoints, single-stepping,
  RGBDS-syntax disassembly, memory dumps, VRAM tile sheets, serial output
  from test ROMs, and a CPU profile.
- **Find game values** — a RAM search (cheat finder) to locate lives, score,
  health or position, then trace the code that changes them.

Four open-source homebrew games are bundled as samples (Tobu Tobu Girl,
Libbet and the Magic Floor, Shock Lobster, 2048); see
`server/roms/LICENSES.md`.

## Skills

| Skill | Use it for |
|---|---|
| play-gameboy | Playing or smoke-testing a ROM |
| debug-gameboy-rom | Crashes, hangs, broken graphics, failing test ROMs, reading code |
| find-game-values | Locating and changing values in RAM, making cheats |

## MCP tools

`load_rom`, `press`, `run`, `screenshot`, `state`, `read_memory`,
`write_memory`, `disassemble`, `breakpoint`, `step`, `save_state`,
`load_state`, `find_value`, `serial_output`, `profile`, `tiles`, `reset`.

## Requirements

Node.js 18 or newer on the machine that runs Claude (the server is a single
ES module plus `matcha.wasm`). Claude Desktop's Cowork workspace already has
Node.

## Accuracy

The emulator core passes all 498,000 SingleStepTests SM83 cases, all 94
DMG-applicable Mooneye tests, dmg-acid2 and the Blargg CPU/timing/sound
suites. A per-dot pixel FIFO handles mid-scanline effects. Native CGB supports
banked memory, colour palettes, double-speed CPU mode and VRAM DMA;
cgb-acid2 and cgb-acid-hell match their reference images. Some precise
timing effects and DMG-on-CGB compatibility mode remain unsupported.

`load_rom` selects the model from the cartridge header by default; use
`model: "dmg"` or `model: "cgb"` to override it. Colour screenshots use
the game's palette RAM. Save states are model-specific format version 4.

## Source

The emulator (Rust core compiled to WebAssembly), the web player and this
plugin come from the `matcha` repository; rebuild the plugin with
`scripts/build-plugin.sh`.
