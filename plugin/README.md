# matcha — Game Boy for AI agents

Gives an AI agent a Game Boy / native Game Boy Color it can look at, play
and debug. Everything runs locally in Node; no network, no dependencies.

## What an AI can do with it

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

Node.js 18 or newer and an AI application that supports local stdio MCP tools.
For visual play, the application must pass image results to an image-capable
model. The server is an ES module plus `matcha.wasm`; no npm install is needed.

## Installation and operator guide

The default host-neutral distribution is `dist/matcha-ai.zip`, built with
`node scripts/build-ai-package.mjs`. Extract it and run `node configure.mjs`
to print the absolute command and arguments for your AI application's MCP
settings. The JSON container used by a particular host may differ; use its
local stdio MCP settings and merge with existing entries.

This optional `.plugin` adapter retains the manifest and variable names its
original host requires. Its AI tools and operator instructions are shared
with the generic ZIP. Import it only into an application that supports this
plugin format; the file extension is not a universal installation format.

Built packages include `AI_GUIDE.md` (Korean operator manual), `configure.mjs`
and three English workflow skills. In a source checkout the full guide is
`mcp/AI_GUIDE.md`. Skills may be read as ordinary instructions when the host
does not automatically discover them. Test in a fresh temporary session,
retain a written result, and remove test processes and temporary files afterward.

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
