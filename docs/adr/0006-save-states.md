# ADR-0006: Hand-rolled, versioned binary save states with atomic load

**Status:** Accepted
**Date:** 2026-09-28
**Deciders:** HONGDAE KIM (owner), Claude (implementation)

## Context

Save states power quick save/load, 20-second rewind (a state every 3
frames), and the agent workflow (save, experiment, reload). They must be
small, fast, deterministic, and safe to load from untrusted bytes.

## Decision

A fixed-order little-endian format: `MTCHST` magic, a format version, a ROM
identity hash (FNV-1a of the header), then each component's `save`/`load`
pair in a fixed order. Loads validate every field that could break an
invariant (modes, counters, lengths) and are atomic: `GameBoy::load_state`
snapshots the current state first and restores it if anything fails.

## Options Considered

### Option A: Hand-rolled writer/reader (chosen)
| Dimension | Assessment |
|---|---|
| Size | ~40 KiB + cartridge RAM |
| Dependencies | None (fits ADR-0003) |
| Evolution | Manual: bump `STATE_VERSION` when a layout changes |

### Option B: serde + bincode
| Dimension | Assessment |
|---|---|
| Size | Similar |
| Dependencies | serde in the core |
| Evolution | Derive-based; easy to break silently by reordering fields |

## Trade-off Analysis

The component `save`/`load` pairs are short and reviewed together; tests
catch asymmetry. The atomicity test found a real bug (SC stored with its
read-mask bits), which is the kind of error this approach needs guarding.

## Consequences

- Easier: rewind, deterministic replays, agent experiments.
- Harder: any layout change needs a version bump (old states are rejected
  with a clear error rather than misread).
- Revisit: add a migration step if states become user files that must survive versions.

## Action Items

1. [x] Writer/reader with bounds-checked reads and typed errors.
2. [x] Round-trip, cross-instance replay and atomic-failure tests.
3. [ ] Optional compression for long rewind buffers.
