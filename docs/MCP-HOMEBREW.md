# A repeatable MCP debugger session

Use Matcha when an agent needs to inspect and reproduce a small homebrew bug
through the same emulator session. This walkthrough uses an original 32 KiB
fixture in `mcp/fixtures/homebrew.mjs`; it contains no downloaded game or boot
ROM. Its header omits Nintendo's logo and it relies on Matcha's post-boot start.
It is a debugger fixture, not a cartridge certified to boot on real hardware.

## Run the evidence

Requires Node 18+. From a source checkout or source release:

```sh
node mcp/test.mjs
node mcp/debug-homebrew.mjs
```

The second command starts the actual MCP stdio server, makes a temporary ROM,
checks the sequence below three times, and removes the temporary file. It also
checks 20 invalid requests per session, including invalid ROMs, missing files,
frame bounds, button names, memory bytes, palette scale and RAM-search bounds.
After each rejection it compares registers/cycles, memory, image, breakpoints,
search candidates and saved-state replay. It then verifies that a *successful*
ROM replacement clears the per-ROM slots, breakpoints and search.

To check an extracted plugin, use the same test against its actual server:

```sh
node mcp/debug-homebrew.mjs /absolute/path/to/plugin/server/server.mjs
```

## Do it in an MCP client

Create a fixture file (the generator refuses to overwrite an existing file):

```sh
node mcp/fixtures/homebrew.mjs /tmp/matcha-debug.gb
```

Configure a local stdio server with command `node` and an absolute argument
pointing to `mcp/server.mjs`. Then make these tool calls:

| Tool | Arguments | Expected observation |
|---|---|---|
| `load_rom` | `{"path":"/tmp/matcha-debug.gb","frames":0,"ram":"zero"}` | DMG, frame 0 |
| `disassemble` | `{"address":"0150","count":5}` | Write `$42` to `$C000`, increment A, write to `$C001`, loop |
| `save_state` | `{"slot":"boot"}` | Save the starting state |
| `breakpoint` | `{"action":"add","address":"c000","kind":"write"}` | Watch writes to `$C000` |
| `run` | `{"frames":1,"screenshot":false}` | Instruction at `$0152` wrote `$42` to `$C000` |
| `read_memory` | `{"address":"c000","length":2}` | `42 00` |
| `step` | `{"count":2}` | Execute increment and second store |
| `read_memory` | `{"address":"c000","length":2}` | `42 43` |
| `load_state` | `{"slot":"boot"}` | Restore the original machine; watchpoint remains |

Repeat `run` and `step` with the same arguments: registers, cycles and memory
must agree. The displayed image is intentionally simple; the evidence is the
observed instruction and memory sequence, not an attractive game screenshot.

## Recovery and limits

Arguments are checked against the advertised input schema before execution.
Unknown fields, nulls for non-nullable values and out-of-range values produce
tool errors. Failed ROM loading preserves the existing machine and debugger
data. Successful loading deliberately clears save slots, breakpoints and search.

Save slots live only in this server process. Closing the connection or killing
the process loses them. Requests run synchronously with bounded frame/step
counts; cancellation cannot interrupt an active emulation call, and cancelling
an already completed call does not roll it back. Prefer short runs for interactive
debugging. This test verifies late-cancellation handling and clean EOF, not
mid-call cancellation or crash persistence.

The emulator core and save-state format are unchanged in 0.1.1. Known DMG/CGB
timing failures and missing CGB compatibility mode still apply. This fixture
does not revalidate the separate conformance scoreboards or real hardware.
