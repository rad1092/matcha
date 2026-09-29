---
name: play-gameboy
description: Plays or tests a Game Boy (.gb/.gbc) ROM with the matcha emulator tools — boots it, reads the screen, presses buttons, and reports what happens. Use when asked to play a Game Boy game, try or smoke-test a homebrew ROM, check whether a ROM boots, get past a title screen, or describe what a game looks like.
---

# Playing a Game Boy ROM with matcha

The `matcha` MCP server is a Game Boy / native Game Boy Color emulator. Every
tool call advances or inspects one machine. Emulation is deterministic: the
same state plus the same inputs always gives the same result, so experiments
can be repeated exactly.

## Start

1. Call `load_rom` with the file path the user gave. With no ROM supplied,
   offer the bundled open-source samples (`sample`: `tobu`, `libbet`,
   `shock-lobster`, `2048`) — the tool description lists what each one is.
2. Look at the returned screenshot before doing anything else. Describe what
   is on screen in one line so the user can follow along.
3. Check the returned model. `load_rom` defaults to `model: "auto"`, choosing
   native CGB for colour-capable cartridges and DMG otherwise. Explicit
   `dmg` / `cgb` overrides are available for debugging. DMG-on-CGB
   compatibility colourization is not implemented.

## Controls and timing

- 60 frames ≈ 1 second (the DMG runs at 59.73 Hz).
- `press` taps buttons: it holds them for `hold_frames` (default 6), releases,
  runs `wait_frames` (default 30), and returns a screenshot. Use it for menus,
  dialogue and single jumps.
- Movement and charging need holding: use `press` with a longer
  `hold_frames` (20–60), or `run` with `buttons` to hold for many frames.
- `run` without buttons lets time pass: cutscenes, intro logos, waiting for a
  timer.
- Title screens usually want `start`; dialogue advances with `a`.
- Set `screenshot: false` on intermediate calls when chaining many inputs,
  then take one `screenshot` at the end to keep responses small.

## Play safely and efficiently

- Call `save_state` before anything risky (a jump over a pit, a boss, an
  irreversible menu choice). If it goes wrong, `load_state` and try a
  different timing. Name slots after what they hold (`"before-boss"`).
- When a sequence of inputs works, note it (e.g. `start, wait 60, a, a`) so it
  can be replayed from a state.
- Text is easiest to read with `screenshot` at `scale: 3` and the default grey
  palette.
- Reading game variables directly is faster than squinting at the screen:
  after locating them with the find-game-values skill, `read_memory` gives
  exact numbers.

## When things look wrong

- Screen stays blank for more than ~5 seconds of `run`: check `state` — LCDC
  bit 7 off means the game turned the display off; a `locked` CPU means it
  executed an illegal opcode (inspect the ROM and trace before attributing the cause).
- The game ignores input: some games only poll the joypad once per frame, so
  hold buttons for at least 2 frames; some wait for a release before
  accepting the next press.
- For crashes, hangs or garbage graphics, switch to the debug-gameboy-rom
  skill.

## Reporting

End with what was verified, as observed facts: which screens were reached,
which inputs did what, anything that looked broken, and the save slots that
reproduce interesting moments. Include the final screenshot.
