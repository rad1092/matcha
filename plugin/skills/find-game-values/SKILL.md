---
name: find-game-values
description: Finds where a Game Boy game stores a value such as lives, health, score, coins, timer or position using matcha's RAM search, then reads, changes or traces it. Use when asked to make a cheat, find a RAM address, give infinite lives, freeze a timer, or explain how a game keeps track of something.
---

# Finding a game value in RAM

`find_value` compares snapshots of work RAM ($C000–$DFFF), cartridge RAM
($A000–$BFFF, when present) and high RAM ($FF80–$FFFE). Each `filter` keeps
only the addresses that match; a handful of rounds usually leaves one.

## Search

1. Get to a moment where the value is visible and about to change (for
   lives: during play). `save_state` here so the search can be redone.
2. `find_value` with `action: "start"`.
3. Change the value in the game (lose a life, collect a coin, take damage),
   using `press`/`run`.
4. Filter:
   - If the on-screen number is known, `op: "eq"` with that `value` is the
     strongest filter. Games often store lives one lower than shown, or as
     the tile index of a digit — if `eq` empties the list, restart and use
     relative filters.
   - Otherwise `op: "down"` / `"up"` / `"changed"` against the previous
     snapshot.
5. Let time pass without touching the value, then filter `op: "same"`. This
   removes timers, animation counters and RNG state.
6. Repeat 3–5 until 1–5 candidates remain.

## Confirm

- `read_memory` on the candidates while the value changes in game.
- Poke a test value with `write_memory` and take a `screenshot`: the display
  may only refresh when the game next redraws that number, so change it
  in-game once more or wait a moment.
- `load_state` afterwards to put the game back.

## Explain or trace

- `breakpoint` with `kind: "write"` on the address, then `run`: the stop
  report names the instruction that changed it. `disassemble` around that PC
  shows the logic (a `dec [hl]` on death, an `add` for score).
- Scores are frequently BCD (each nibble one decimal digit) and span several
  bytes, low byte first or most significant first — check neighbours.
- 16-bit values (positions, money) are usually little-endian pairs.

## Deliver

Give the address, what the value means, its encoding, and how to freeze or
set it. A classic GameShark code for the DMG is `01VVLLHH`: `01`, the value,
then the address low byte and high byte (address $C0A2 set to 9 →
`0109A2C0`). Mention that such codes rewrite the value every frame.
