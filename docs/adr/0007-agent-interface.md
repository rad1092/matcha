# ADR-0007: A zero-dependency MCP server over the WASM core

**Status:** Accepted
**Date:** 2026-09-29
**Deciders:** HONGDAE KIM (owner), Claude (implementation)

## Context

An AI agent should be able to play, test and debug Game Boy software: see the
screen, press buttons, inspect memory, and repeat experiments. It has to
install as a Claude plugin with nothing but Node on the machine.

## Decision

`mcp/server.mjs` implements MCP (JSON-RPC 2.0, newline-delimited stdio)
directly, loads the same `matcha.wasm` as the web player, and exposes 17
task-level tools: `load_rom`, `press`, `run`, `screenshot`, `state`,
`read_memory`, `write_memory`, `disassemble`, `breakpoint`, `step`,
`save_state`, `load_state`, `find_value`, `serial_output`, `profile`,
`tiles`, `reset`. Screenshots are PNG image content. The plugin bundles
three skills that teach the workflows (play, debug, find values) and four
open-source sample ROMs.

Tool design rules:
- Each tool does one agent-sized job and returns what the agent needs next
  (e.g. `press` returns a screenshot).
- Invalid input returns `isError` results with a message that says how to
  fix the call; only unexpected exceptions are logged.
- Everything is deterministic, so the agent can save, try and reload.

## Options Considered

### Option A: Hand-written protocol layer (chosen)
| Dimension | Assessment |
|---|---|
| Install | Copy files; runs on any Node ≥ 18 |
| Size | ~700 lines incl. PNG encoder |
| Risk | Must track protocol changes ourselves |

### Option B: Official TypeScript SDK
| Dimension | Assessment |
|---|---|
| Install | `npm install` or a bundling step |
| Size | Larger; dependency tree |
| Risk | Lower protocol risk |

### Option C: Python + PyBoy
| Dimension | Assessment |
|---|---|
| Install | Python env + native wheels |
| Accuracy | PyBoy's, not ours; no shared core with the player |

## Trade-off Analysis

The protocol subset needed (initialize, tools/list, tools/call, ping) is
small and stable; version negotiation accepts 2024-11-05 through 2025-11-25.
Owning it keeps install to "copy a folder", which matters for a plugin.

## Consequences

- Easier: install anywhere; the agent and the web player see identical
  emulation.
- Harder: new protocol features (resources, progress) are manual work.
- Revisit: switch to the SDK if we need streaming progress or resources.

## Action Items

1. [x] Server, 17 tools, e2e protocol test (`mcp/test.mjs`).
2. [x] Plugin with skills, validated with `claude plugin validate`.
3. [x] Verified live in the owner's Claude app (boot → title → main menu via tools).
4. [ ] Resources for ROM header/memory maps if agents need them without tool calls.
