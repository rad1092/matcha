# ADR-0003: `no_std` core with zero dependencies and no `unsafe`

**Status:** Accepted
**Date:** 2026-09-28
**Deciders:** HONGDAE KIM (owner), Claude (implementation)

## Context

The same core runs natively (CLI, tests), in browsers and in Node (both
WebAssembly). Future targets could include microcontroller handhelds
(RP2040/ESP32-class). The core is also the part that must be most
trustworthy.

## Decision

`matcha-core` is `#![no_std]` + `alloc`, has no dependencies, and is
`#![forbid(unsafe_code)]`. All I/O (files, audio devices, clocks) lives in
the frontends. Wall-clock time enters only through explicit calls
(`rtc_advance_seconds`).

## Options Considered

### Option A: `no_std` + alloc, zero deps (chosen)
| Dimension | Assessment |
|---|---|
| Complexity | Low (no `f32::powf`: a small power helper in the APU) |
| Portability | Any target with an allocator |
| Supply chain | None |

### Option B: `std` with common crates (serde, bitflags, log)
| Dimension | Assessment |
|---|---|
| Complexity | Lower to write |
| Portability | std targets only |
| Supply chain | Transitive dependencies in the most sensitive crate |

## Trade-off Analysis

The costs of Option A were small (a hand-written state serializer — see
ADR-0006 — and a power function), and it keeps the core auditable and
deterministic. Dependencies are allowed in the frontends (`png`,
`serde_json` in the CLI).

## Consequences

- Easier: WASM builds, embedded ports, reasoning about determinism.
- Harder: no `HashMap` or float math intrinsics in the core.
- Revisit: never for `unsafe`; the WASM FFI boundary (ADR-0004) is the only
  `unsafe` in the project and is confined to `matcha-wasm`.

## Action Items

1. [x] `#![no_std]`, `#![forbid(unsafe_code)]`, empty `[dependencies]`.
2. [ ] An embedded frontend (e.g. RP2040 + 160x144 LCD) as a portability proof.
