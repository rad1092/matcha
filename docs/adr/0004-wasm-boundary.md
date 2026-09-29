# ADR-0004: Hand-written C ABI for WebAssembly instead of wasm-bindgen

**Status:** Accepted
**Date:** 2026-09-28
**Deciders:** HONGDAE KIM (owner), Claude (implementation)

## Context

Two JavaScript hosts use the core: the browser player and the Node MCP server.
Both should load the same `.wasm` file, the web page must work as a single
offline HTML file, and builds should not depend on a tool whose version must
match a crate version.

## Decision

`matcha-wasm` exports `extern "C"` functions over an opaque emulator handle,
with plain numbers in and out. Bulk data (frames, audio, save states, text)
lives in buffers owned by the handle; JS copies it out. `web/matcha.js` (no
dependencies) wraps the exports in a class used unchanged by both hosts.

## Options Considered

### Option A: C ABI + tiny JS wrapper (chosen)
| Dimension | Assessment |
|---|---|
| Complexity | ~450 lines Rust + ~250 lines JS |
| Build | `cargo build --target wasm32-unknown-unknown`, nothing else |
| Output | One 148 KiB `.wasm` with no imports; loads in browsers and Node |

### Option B: wasm-bindgen / wasm-pack
| Dimension | Assessment |
|---|---|
| Complexity | Less glue to write |
| Build | CLI version must match the crate; separate targets for web and Node |
| Output | Generated JS per target |

## Trade-off Analysis

The API surface is small and numeric, so generated bindings save little and
add a toolchain coupling plus per-target output. With no imports the module
instantiates anywhere, which is what makes the single-file player and the
zero-dependency MCP server possible.

## Consequences

- Easier: one artifact for all JS hosts; offline builds.
- Harder: every export is `unsafe extern "C"` with a documented contract;
  WASM memory growth detaches views, so the wrapper always copies.
- Revisit: if the API grows past ~60 exports, consider generating the JS
  wrapper from a declarative list.

## Action Items

1. [x] Handle-based exports with a written safety contract.
2. [x] `web/matcha.js` shared by the player and the MCP server.
3. [x] `scripts/build-wasm.sh`.
