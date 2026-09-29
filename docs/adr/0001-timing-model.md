# ADR-0001: M-cycle bus timing ("access, then tick") with per-dot PPU events

**Status:** Accepted
**Date:** 2026-09-28
**Deciders:** HONGDAE KIM (owner), Claude (implementation)

## Context

Game Boy accuracy is mostly a timing problem. Test ROMs (Blargg `mem_timing`,
Mooneye `*_timing`, `intr_*`, `lcdon_*`) check *which M-cycle* of an
instruction touches memory, and games that write VRAM, poll STAT or race the
timer depend on the same thing. The model has to expose memory-access timing
inside instructions without making every component expensive.

Forces: accuracy verified by the suites; deterministic output; at least ~30x
realtime natively so the corpus analysis and CI stay fast; code a single
maintainer can hold in their head.

## Decision

The CPU is generic over a `CpuBus` trait whose `read`, `write` and `idle`
each represent exactly one M-cycle. `SystemBus` performs the access *first*
and then advances every peripheral by one M-cycle (4 dots). The PPU tracks
dots and fires a small set of per-line events at the dots where hardware tests
place them; CPU accesses land when the line-relative dot counter is ≡ 3
(mod 4) — a phase fixed by the 449-dot first line after LCD enable.
Interrupt requests raised in the last two dots of an M-cycle are visible in
IF immediately but only become dispatchable after the next M-cycle (the CPU
samples interrupt lines two dots before it samples the data bus).

## Options Considered

### Option A: Instruction-stepped with cycle counts (catch-up after each instruction)
| Dimension | Assessment |
|---|---|
| Complexity | Low |
| Accuracy | Fails memory-timing and most PPU timing tests |
| Performance | Highest |
| Familiarity | Common in hobby emulators |

**Pros:** simplest; fast. **Cons:** every intra-instruction timing effect is
wrong; can't pass Blargg `mem_timing` or Mooneye timing tests.

### Option B: M-cycle "access, then tick" (chosen)
| Dimension | Assessment |
|---|---|
| Complexity | Medium |
| Accuracy | Passes SST bus-cycle checks, Blargg timing, Mooneye 94/94 |
| Performance | 51x realtime headless after the fast paths in ADR-0002/0005 |
| Familiarity | The SameBoy-style model; well documented |

**Pros:** every access is timed; peripherals stay simple (`tick()` per
M-cycle). **Cons:** sub-M-cycle phases (e.g. mid-cycle STAT writes, the
interrupt sampling phase) must be modelled explicitly.

### Option C: T-cycle (dot) accurate CPU and bus
| Dimension | Assessment |
|---|---|
| Complexity | High |
| Accuracy | Highest ceiling (needed for a few Mealybug/SameSuite edge cases) |
| Performance | ~4x more work per emulated second |
| Familiarity | Rare; few references |

**Pros:** no phase approximations. **Cons:** cost and complexity far beyond
what current test failures justify.

## Trade-off Analysis

Option B buys nearly all measurable accuracy for a fraction of Option C's
cost. The two places it needed care — the PPU's dot phase and the interrupt
sampling phase — are now explicit, documented constants validated by
`lcdon_timing`, `lcdon_write_timing`, `hblank_ly_scx_timing` and
`intr_2_mode0_timing_sprites`. The same phase detail explains the DMG wave-RAM
access window (ADR-0005).

## Consequences

- Easier: adding peripherals (anything with `tick()` fits), testing CPU
  instructions in isolation (`FlatBus` in `tests/sst.rs` records bus cycles).
- Harder: mid-M-cycle register effects need explicit modelling rather than
  falling out of the simulation.
- Revisit: if the pixel FIFO (ADR-0002) needs write timing finer than an
  M-cycle for Mealybug tests, add per-register write-phase offsets rather
  than moving to Option C.

## Action Items

1. [x] `CpuBus` trait; `SystemBus` access-then-tick.
2. [x] SingleStepTests harness checking every bus cycle (498,000 cases).
3. [x] PPU event dots and IRQ sampling phase; Mooneye 94/94.
4. [ ] Per-register write-phase offsets if the FIFO work needs them.
