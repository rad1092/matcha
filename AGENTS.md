# Working on matcha

Instructions for anyone — human or coding agent — changing this repository.
Read [`CONTINUE.md`](CONTINUE.md) first for the current state and next steps,
and [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) for how the pieces fit.

## Commands

```sh
cargo build --release                      # core + CLI (target/release/matcha)
cargo test --workspace                     # unit tests + 498k CPU cases; optimised, overflow-checked
scripts/fetch-testdata.sh                  # test ROMs + SingleStepTests into testdata/ (once)
cargo run --release -p matcha-cli -- test testdata/roms --baseline docs/conformance.json
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings

scripts/build-wasm.sh                      # core -> web/matcha.wasm (MATCHA_BUILD_STD=1 if the
                                           # wasm32 target can't be installed)
node mcp/test.mjs                          # MCP server end-to-end test (uses web/matcha.wasm)
node scripts/build-web.mjs                 # dist/matcha.html + dist/matcha.fragment.html
scripts/build-plugin.sh                    # dist/matcha.plugin
```

`matcha trace <rom> --input monkey --last 200` prints the instructions before
a crash; `--watch ADDR` shows which instructions changed a byte.

## Rules

- **The core stays portable and safe.** `matcha-core` is `no_std` + `alloc`,
  has no dependencies and forbids `unsafe` (ADR-0003). Only `matcha-wasm`
  contains `unsafe`, each export documenting its pointer contract.
- **Determinism is a feature.** The core never reads a clock or a random
  source. Same ROM + inputs + state ⇒ identical frames and audio.
- **Save states are versioned.** Any new piece of machine state must be
  written and read in the component's `save`/`load`, validated on load, and
  `STATE_VERSION` in `state.rs` bumped when the layout changes (ADR-0006).
- **Hardware behaviour needs a source and a test.** Cite Pan Docs, a test
  ROM or the reference emulator in a comment, and cover the change with a
  test ROM, a SingleStepTests case or a unit test.
- **Never regress the scoreboard.** CI runs the suites with `--baseline
  docs/conformance.json`. When accuracy improves, regenerate it:
  `matcha test testdata/roms --json docs/conformance.json --markdown docs/CONFORMANCE.md`,
  then `node scripts/build-web.mjs` so the page's numbers follow.
- **One wasm for every host.** After changing the core or `matcha-wasm`,
  rebuild `web/matcha.wasm` and run `node mcp/test.mjs`; the web player,
  MCP server and plugin all load that file through `web/matcha.js`.
- **Style.** `rustfmt.toml` (width 120) and clippy with `-D warnings`. Comments
  explain why, not what. No dead code, placeholders or commented-out code.
- **Commits.** Imperative subject with an area prefix (`core:`, `cli:`,
  `web:`, `mcp:`, `docs:`, `analysis:`); the body says what changed and why.

## Code map

| Path | What |
|---|---|
| `crates/matcha-core/src/cpu.rs` | SM83 interpreter over the `CpuBus` trait |
| `crates/matcha-core/src/bus.rs` | Memory map, interrupts, OAM DMA, per-M-cycle clocking |
| `crates/matcha-core/src/ppu.rs` | LCD timing (event-scheduled), access windows, per-dot BG/OBJ pixel FIFOs |
| `crates/matcha-core/src/apu.rs` | Sound: channels, frame sequencer, mixer, resampler |
| `crates/matcha-core/src/{timer,joypad,serial,cartridge,state,disasm,profile}.rs` | The rest of the machine and its tooling |
| `crates/matcha-core/src/lib.rs` | `GameBoy` facade: run loop, debugger API, unit tests |
| `crates/matcha-core/tests/sst.rs` | SingleStepTests harness |
| `crates/matcha-cli/` | `matcha` CLI: run, test (scoreboard), profile, trace, disasm, info |
| `crates/matcha-wasm/` | C-ABI WebAssembly exports |
| `web/` | `matcha.js` wrapper, `matcha.wasm`, player source (`src/`), bundled ROMs |
| `mcp/` | MCP server (`server.mjs`) and its end-to-end test |
| `plugin/` | Claude plugin: manifest, skills, README (server copied in at build) |
| `analysis/` | Corpus study: manifest, profiling, SameBoy reference runner, report |
| `docs/` | Architecture, ADRs, conformance scoreboard, analysis report |
