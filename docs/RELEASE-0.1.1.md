# 0.1.1 validation and release scope

The MCP server validates its advertised input contract before executing a tool.
ROM loading prepares the new machine and reply before replacing the old session.
This fixes a rejected `load_rom` (for example `frames: -1`) destroying the running
machine and save slots. It also prevents invalid `press.wait_frames`, screenshot
scale, save-slot types and search values from partially changing the session.

Reproduce with `node mcp/debug-homebrew.mjs`; the same check accepts the path to
an extracted plugin server. The generated ROM and the expected instruction and
memory values are documented in [MCP-HOMEBREW.md](MCP-HOMEBREW.md).

Validated locally on 2026-10-12 with Node 22 and Rust 1.99.0, macOS arm64:

- The new debugger regression fails against the previous server at `2ba268d`.
- `node mcp/test.mjs`: protocol, 17 tool contracts, PNG, DMG/CGB model selection,
  RAM initialization and saved-state round-trip using the original fixture.
- `node mcp/debug-homebrew.mjs`: watchpoint attribution, stepping, exact replay,
  20 rejected requests across three sessions, state/slots/debugger preservation,
  successful replacement cleanup, late cancellation and clean EOF.
- `cargo test --workspace --locked`: local unit/doc checks. The external-data
  SingleStepTests harness has no local data and is **not** counted as 498,000
  verified cases. DMG/CGB ROM scoreboards were not re-run in this local review.
- `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`.
- `scripts/build-plugin.sh`: assembled server passes the same original-ROM
  debugger checks; installed Claude CLI manifest validation passes.
- `node scripts/build-web.mjs`: web bundle builds. Interactive browser gameplay
  is outside this MCP correction's test scope.

The Rust emulator code, committed WASM and save-state layout are unchanged.
Existing conformance figures retain their original provenance, not new proof
from these synthetic checks. Known hardware timing limitations remain.

Independent review and clean public-release download verification are separate
release gates. A source test pass or locally assembled archive alone does not
establish that a published asset contains this fix.

The independent candidate review also exercised both DMG and CGB with an
original instruction fixture: rejected inputs preserved CPU/RAM/images and
per-session debugger data, while save/load replay was exact. ROM identity is
header-based; same-header, different-body ROMs are not distinguished. The CLI
and core are unchanged. Interactive Claude Desktop installation and full web
player interaction are not claimed by the MCP tests.

CI uploads the plugin, offline HTML, retained homebrew notices, a source/WASM
manifest and checksums. Publish only artifacts from a successful final main run.
The release remains dependent on clean-download consumption checks after upload.
