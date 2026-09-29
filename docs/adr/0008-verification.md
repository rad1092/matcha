# ADR-0008: The conformance scoreboard is the definition of done

**Status:** Accepted
**Date:** 2026-09-28
**Deciders:** HONGDAE KIM (owner), Claude (implementation)

## Context

Emulator code is easy to make "look right" and hard to make right. The
project needs an objective, automated measure that any contributor (human or
AI) can run after a change, and that stops regressions.

## Decision

Accuracy is defined by public test suites, run automatically:

| Layer | Suite | How it judges |
|---|---|---|
| CPU | SingleStepTests/sm83 (500 opcodes × 1000 cases) | final registers, memory, and every bus cycle |
| System | Blargg cpu/timing/sound/oam_bug | result code in cartridge RAM / serial text / screenshot |
| System | Mooneye acceptance + MBC (DMG-applicable) | Fibonacci registers at `ld b, b` |
| PPU | dmg-acid2 | pixel-exact screenshot |
| PPU (stretch) | Mealybug Tearoom | pixel-exact screenshots |
| Core API | unit tests | determinism, save-state round trip and atomicity, breakpoints, profiler |
| Frontends | `mcp/test.mjs` | MCP protocol, images, state round trip |

`matcha test <dir> --json docs/conformance.json` produces the scoreboard; the
web page's accuracy table is generated from that file, never typed by hand.
Test data is fetched by `scripts/fetch-testdata.sh`, not committed. CI sets
`MATCHA_REQUIRE_TESTDATA=1` so missing data fails instead of skipping.

## Options Considered

- **Suites as ground truth (chosen):** objective, community-maintained,
  covers timing that unit tests can't express.
- **Hand-written unit tests only:** tests encode the author's understanding,
  including its mistakes.
- **Differential testing against another emulator:** useful, but inherits
  the reference emulator's bugs; kept as a possible addition.

## Consequences

- Easier: every change has a measurable effect; regressions are loud.
- Harder: some suites need exact sub-cycle behaviour, pushing work toward
  accuracy that few games need (tracked, not required: Mealybug, oam_bug).
- Revisit: add SameSuite and Gambatte's DMG tests as the next layers.

## Action Items

1. [x] SST harness; conformance runner with Blargg/Mooneye/acid2/Mealybug judges.
2. [x] Scoreboard JSON/Markdown; page numbers generated from it.
3. [x] CI workflow (`.github/workflows/ci.yml`).
4. [ ] OAM corruption bug (Blargg `oam_bug`), pixel FIFO (Mealybug).
