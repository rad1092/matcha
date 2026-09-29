# Continuing matcha

Where the project stands and how to pick it up — written for the next
person or coding agent. Rules and commands are in [`AGENTS.md`](AGENTS.md);
how it fits together is in [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md).

## State (2026-09-29)

Done and verified:

- **Core** (`crates/matcha-core`): SM83 CPU, PPU with measured line timing,
  APU, timer, serial, joypad, OAM DMA with bus conflicts, the DMG OAM
  corruption bug, MBC1/1M/2/3+RTC/5, versioned save states with atomic,
  validated loading, optional DMG-like power-on RAM noise, debugger hooks,
  profiler. Scoreboard: [`docs/CONFORMANCE.md`](docs/CONFORMANCE.md).
- **Hosts**: web player (`web/`, bundled into `dist/matcha.html`), MCP server
  and Claude plugin (`mcp/`, `plugin/`, packaged as `dist/matcha.plugin`),
  CLI (`crates/matcha-cli`: run, test, profile, trace, disasm, info).
- **Docs**: architecture, eight ADRs, roadmap, this file.
- **Corpus study** (`analysis/`, report in [`docs/analysis.md`](docs/analysis.md)):
  868 homebrew programs through matcha and SameBoy; 866 same outcome.
- **CI** (`.github/workflows/ci.yml`, on [GitHub](https://github.com/rad1092/matcha/actions)):
  fmt, clippy, unit tests + 498k CPU cases, conformance against the
  committed scoreboard, wasm build, MCP e2e, plugin build.

Not done (in order — details, verification and first steps in
[`ROADMAP.md`](ROADMAP.md)):

1. Game Boy Color mode — the biggest compatibility gap (29% of the corpus).
2. Pixel FIFO renderer — Mealybug 1/24, Gambatte's mid-mode-3 tests.

## Set up a machine

| Need | For | Notes |
|---|---|---|
| Rust ≥ 1.85 | everything | `rustup target add wasm32-unknown-unknown` for the web build. Without network access to the prebuilt target, `MATCHA_BUILD_STD=1 scripts/build-wasm.sh` builds std from `rust-src` instead. |
| Node ≥ 18 | MCP server, web bundle, `mcp/test.mjs` | no npm packages |
| `zip` | `scripts/build-plugin.sh` | the Claude CLI is optional (`claude plugin validate`) |
| Python 3 + pandas, numpy, matplotlib | `analysis/` | |
| C compiler + [RGBDS](https://github.com/gbdev/rgbds) + a [SameBoy](https://github.com/LIJI32/SameBoy) checkout | `analysis/reference/` only | built here with RGBDS v1.0.4 and SameBoy `213a12c` |

Test data is fetched, never committed: `scripts/fetch-testdata.sh` (pinned
SingleStepTests commit, checksum-verified test-ROM release). The homebrew
corpus is a sparse checkout of [gbdev/database](https://github.com/gbdev/database)
— see `analysis/README.md`.

## Check that everything still works

```sh
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run --release -p matcha-cli -- test testdata/roms --baseline docs/conformance.json
scripts/build-wasm.sh && node mcp/test.mjs
node scripts/build-web.mjs && scripts/build-plugin.sh
```

All of these passed on the commit that added this file.

## Things that are easy to get wrong

- **The wasm is committed** (`web/matcha.wasm`) because the web player, the
  MCP server and the plugin all load it. Rebuild it after any core change,
  or they drift from the Rust code.
- **Scoreboard numbers are generated.** Regenerate `docs/conformance.json`
  and `docs/CONFORMANCE.md` with `matcha test … --json … --markdown …`, then
  rebuild the web bundle; update the summary table in `README.md` by hand.
- **Analysis numbers are generated.** `analysis/report.py` writes
  `summary.json`; `analysis/publish.py` renders `docs/analysis.md` and the
  page from it and asserts the facts its prose depends on, so a rerun that
  changes them fails instead of publishing a wrong sentence.
- **Save-state layout changes need a `STATE_VERSION` bump** and a validated
  `load` for every new field (the targeted fuzz test in `lib.rs` corrupts
  live fields and fails if a state it accepts panics while loading or
  running).
- **matcha skips the boot ROM** and starts in the DMG post-boot state
  (registers, I/O, the logo in VRAM, PPU at the hand-over point). Mooneye's
  `boot_*` tests pin this down; keep them green.

## Where the published pages are

Both are private Artifacts in the owner's Claude account (shared from each
page's Share menu):

- Web player: https://claude.ai/artifact/EfQ79j2rnaaS4NHw5CKgyG — from
  `dist/matcha.fragment.html` (`node scripts/build-web.mjs`)
- Corpus report: https://claude.ai/artifact/AKXM7XyW8TTq27dTuBgSrz — from
  `dist/analysis.fragment.html` (`python3 analysis/publish.py`)

Republish to the same links after rebuilding.
