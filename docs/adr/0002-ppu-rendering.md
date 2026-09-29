# ADR-0002: Scanline renderer with modelled mode-3 timing; pixel FIFO deferred

**Status:** Superseded by [ADR-0009](0009-pixel-fifo.md) for rendering; the LCD timing event model is retained.
**Date:** 2026-09-28
**Deciders:** HONGDAE KIM (owner), Claude (implementation)

## Context

The DMG PPU draws each line with a pixel FIFO fed by a tile fetcher. Mode 3
lasts 172–289 dots depending on SCX, the window and objects. Two things
depend on it: STAT/interrupt *timing* (tested by Mooneye, used by games for
raster effects) and *mid-line raster effects* (register writes during mode 3
change pixels part-way along the line; tested by Mealybug Tearoom).

## Decision

Render each line in one pass at the start of mode 3 using the registers
current at that moment, and stretch mode 3 by the documented penalties
(SCX mod 8, 6 dots for the window, per-object penalties from Pan Docs) so
STAT timing stays exact. The PPU skips M-cycles in which no timing event
falls (event-dot lookahead). A real fetcher/FIFO is deferred.

## Options Considered

### Option A: Scanline render + timing model (chosen)
| Dimension | Assessment |
|---|---|
| Complexity | Low–medium (≈900 lines incl. timing) |
| Accuracy | dmg-acid2 pass; Mooneye PPU timing pass; Mealybug 1/24 |
| Performance | Cheap: one pass per line, M-cycles without events skipped |
| Familiarity | Standard |

### Option B: Dot-accurate fetcher + FIFO
| Dimension | Assessment |
|---|---|
| Complexity | High (fetcher states, object fetch aborts, window glitches) |
| Accuracy | Needed for Mealybug and a few games (e.g. Prehistorik Man) |
| Performance | Per-dot work during mode 3 (~40% of all dots) |
| Familiarity | SameBoy/Gambatte-level detail |

## Trade-off Analysis

Almost all software, and every conformance suite except Mealybug, is
satisfied by Option A. The corpus analysis (docs/analysis.md) lets us check
how common mid-line effects are before paying Option B's cost. Mode 3
lengths are recorded per line so the web player's frame-timing view shows the
real stretch from objects and the window.

## Consequences

- Easier: fast rendering; a small, testable renderer.
- Harder: effects that change BGP/SCX/LCDC mid-line render per line.
- Revisit: when Mealybug becomes the accuracy target. Keep the timing
  events (they are correct) and replace `render_line` with a fetcher that
  runs during mode 3.

## Action Items

1. [x] Scanline renderer, object priority, 10-object limit, 8x16, window line counter.
2. [x] Mode-3 penalty model; per-line timing exposed to debuggers.
3. [x] Fetcher/FIFO renderer (ADR-0009); remaining Mealybug quirks tracked in the scoreboard.
