# ADR-0005: Audio — per-M-cycle mixing, box-filter resampling, DMG high-pass; AudioWorklet sink

**Status:** Accepted
**Date:** 2026-09-28
**Deciders:** HONGDAE KIM (owner), Claude (implementation)

## Context

The APU's channels change level at up to 2 MHz (wave channel) and must be
turned into 44.1/48 kHz stereo. In the browser, audio and video share one
thread with the emulator, and the two clocks (display refresh vs audio
device) drift.

## Decision

- Channel timers run at hardware rates (the wave channel at 2 T-cycle
  resolution). The mixer output is cached and recomputed only when a
  channel's level changes; each M-cycle adds the cached value to an
  accumulator (a box filter), and every output period the average is passed
  through the DMG's DC-blocking high-pass filter (factor
  0.999958^(4194304/rate)).
- The DMG wave-RAM access window is modelled: while channel 3 plays, CPU
  accesses reach wave RAM only in the M-cycle whose second half fetched a
  sample (Blargg `dmg_sound` 09/10/12).
- Headless hosts can switch sample output off; channel state still advances,
  so emulation is unaffected.
- Browser: an AudioWorklet (loaded from a Blob URL) plays a queue of sample
  blocks posted each frame; the main loop nudges its frame accumulator when
  the queue drifts outside 30–120 ms.

## Options Considered

### Option A: Box filter + cached mix (chosen)
| Dimension | Assessment |
|---|---|
| Quality | Good for chiptune; mild aliasing at very high pitches |
| Cost | One add per M-cycle between level changes |

### Option B: Band-limited step synthesis (blip_buf style)
| Dimension | Assessment |
|---|---|
| Quality | Best (no aliasing) |
| Cost | More code; a delta buffer and kernel |

### Option C: Point sampling at the output rate
| Dimension | Assessment |
|---|---|
| Quality | Audible aliasing |
| Cost | Cheapest |

## Trade-off Analysis

Option A sounds right for the source material at a fraction of Option B's
complexity, and the cached mix made it cheap (the APU was the largest item in
the profile before caching).

## Consequences

- Easier: deterministic audio (bit-identical across runs), visualisers
  (per-channel levels exposed).
- Harder: very high square-wave pitches alias slightly.
- Revisit: Option B if audio quality becomes a priority; SharedArrayBuffer
  ring buffer if a host provides cross-origin isolation.

## Action Items

1. [x] Channels, frame sequencer, obscure behaviours; Blargg `dmg_sound` 12/12.
2. [x] Cached mixer; headless output switch.
3. [x] AudioWorklet sink with drift control.
4. [ ] Band-limited synthesis (optional).
